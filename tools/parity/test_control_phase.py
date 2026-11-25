"""The control replay of check_divergence must run against a binary built from the current tree, never a leftover one."""
import os, pathlib, sys, unittest

import check_divergence as cd


class ControlPhase(unittest.TestCase):
    def setUp(self):
        self.saved = (cd.control_binary_for_fast_kill, cd.replay, cd.classify)

    def tearDown(self):
        cd.control_binary_for_fast_kill, cd.replay, cd.classify = self.saved

    def test_the_control_is_built_before_it_is_replayed(self):
        calls = []
        cd.control_binary_for_fast_kill = lambda: calls.append("build") or "/fresh/binary"
        cd.replay = lambda binary: calls.append(("replay", binary)) or object()
        cd.classify = lambda result, binary: (cd.UNDETECTED, 10, 10, "ok")
        ok, passed, total, line, _result = cd.run_control_replay()
        self.assertEqual(calls, ["build", ("replay", "/fresh/binary")])
        self.assertTrue(ok)

    def test_a_control_that_cannot_be_built_fails_and_is_never_replayed_against_a_stale_binary(self):
        calls = []
        cd.control_binary_for_fast_kill = lambda: None
        cd.replay = lambda binary: calls.append(binary)
        ok, _passed, _total, line, _result = cd.run_control_replay()
        self.assertFalse(ok)
        self.assertEqual(calls, [])
        self.assertIn("build", line)


class BudgetExhaustedWhileWaitingForTheReplayLock(unittest.TestCase):
    def setUp(self):
        self.saved = (cd.control_binary_for_fast_kill, cd.replay)

    def tearDown(self):
        cd.control_binary_for_fast_kill, cd.replay = self.saved

    def raise_lock_timeout(self, binary):
        raise cd.replay_lib.ReplayLockTimeout("the run budget ended while waiting for the replay lock")

    def test_the_control_reports_budget_exhausted_not_a_failure_or_an_error(self):
        cd.control_binary_for_fast_kill = lambda: "/fresh/binary"
        cd.replay = self.raise_lock_timeout
        ok, passed, total, line, _result = cd.run_control_replay()
        self.assertEqual((ok, passed, total, line), (False, None, None, cd.BUDGET_EXHAUSTED_LINE))

    def test_a_mutant_whose_replay_waited_out_the_budget_is_budget_exhausted_never_detected(self):
        cd.replay = self.raise_lock_timeout
        outcome, passed, total, line = cd.full_replay_verdict("/some/binary", "some-mutant")
        self.assertEqual((outcome, passed, total), (cd.BUDGET_EXHAUSTED, None, None))
        self.assertIn("budget", line)

    def test_the_final_exit_code_is_the_budget_code_when_the_budget_ran_out_and_otherwise_unchanged(self):
        counts = {cd.UNDETECTED: 0, cd.PATCH_ERROR: 0, cd.BUILD_ERROR: 0, cd.HARNESS_ERROR: 0, cd.TIMEOUT: 0}
        exit_code = lambda **kw: cd.final_exit_code(**{"counts": counts, "sizes": {2821}, "control_ok": True, "replayed_cases": 2821, "budget_exhausted": False, **kw})
        self.assertEqual(exit_code(), 0)
        self.assertEqual(exit_code(budget_exhausted=True), cd.replay_lib.EXIT_BUDGET_EXHAUSTED)
        self.assertEqual(exit_code(counts={**counts, cd.UNDETECTED: 1}), 1)
        self.assertEqual(exit_code(counts={**counts, cd.HARNESS_ERROR: 1}), 2)
        self.assertEqual(exit_code(control_ok=False, replayed_cases=None), 2)
        self.assertEqual(exit_code(control_ok=False), 1)
        self.assertEqual(exit_code(sizes={1, 2}), 2)
        self.assertEqual(exit_code(counts={**counts, cd.UNDETECTED: 1}, budget_exhausted=True), cd.replay_lib.EXIT_BUDGET_EXHAUSTED)


class ControlExitCode(unittest.TestCase):
    def test_a_clean_control_exits_0_a_diverging_one_exits_1_and_one_that_could_not_run_exits_2(self):
        self.assertEqual(cd.control_exit_code(control_ok=True, replayed_cases=2821), 0)
        self.assertEqual(cd.control_exit_code(control_ok=False, replayed_cases=2821), 1)   # replayed and diverged: a real regression
        self.assertEqual(cd.control_exit_code(control_ok=False, replayed_cases=None), 2)   # build failure, timeout or harness error: invalid run


class ControlReusesTheCleanReplayOfTheRun(unittest.TestCase):
    """The control replays the unmodified crate; when this gate run's replay gate already did exactly that for the same binary and inputs, it is not run twice."""
    TOOLS = {"rustc": "r", "cargo": "c", "rust-anything": "a", "python": "p", "platform": "x"}

    def setUp(self):
        import shutil, tempfile
        from unittest import mock
        self.root = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.root, True)
        for name in ("rust/app-cases/a.toml", "rust/app-cases/b.toml", "rust/app-cases/c.toml", "crates/x.rs", "binary"):
            self.edit(name, name)
        self.binary, self.control_builder, self.replays = os.path.join(self.root, "binary"), cd.control_binary_for_fast_kill, []
        for patch in (mock.patch.object(cd, "ROOT", self.root), mock.patch.object(cd, "control_binary_for_fast_kill", return_value=self.binary),
                      mock.patch.object(cd.clean_replay_evidence, "current_tool_versions", return_value=self.TOOLS),
                      mock.patch.object(cd, "replay", side_effect=lambda binary: self.replays.append(binary) or object()),
                      mock.patch.object(cd, "classify", return_value=(cd.UNDETECTED, 3, 3, "replayed")),
                      mock.patch.dict(os.environ, {cd.clean_replay_evidence.RUN_ID_ENV: "run-1"})):
            patch.start()
            self.addCleanup(patch.stop)

    def edit(self, relative, text):
        pathlib.Path(self.root, relative).parent.mkdir(parents=True, exist_ok=True)
        pathlib.Path(self.root, relative).write_text(text)

    def leave_evidence(self, run_id="run-1"):
        ev = cd.clean_replay_evidence
        identity = ev.identity(self.root, self.binary, self.TOOLS, dict(os.environ), ev.CLEAN_REPLAY_COMMAND, ev.REPLAY_TIMEOUT_SECONDS)
        ev.store(ev.evidence_path(self.root), ev.record(identity, dict(ok=True, passed=3, total=3, other_na=1), replayed_cases=3, run_id=run_id))

    def test_a_clean_replay_of_this_run_for_the_same_inputs_is_reused_and_the_replay_is_not_repeated(self):
        self.leave_evidence()
        ok, passed, total, line, replay_result = cd.run_control_replay()
        self.assertEqual((ok, passed, total, replay_result, self.replays), (True, 3, 3, None, []))
        self.assertIn("reused", line)

    def test_any_changed_input_foreign_run_or_missing_evidence_repeats_the_replay(self):
        for label, prepare in {"no evidence": lambda: None, "outside a gate run": lambda: (self.leave_evidence(), os.environ.pop(cd.clean_replay_evidence.RUN_ID_ENV)),
                               "another run": lambda: self.leave_evidence(run_id="run-2"),
                               "a source changed after the replay": lambda: (self.leave_evidence(), self.edit("crates/x.rs", "edited"))}.items():
            self.replays.clear()
            os.environ[cd.clean_replay_evidence.RUN_ID_ENV] = "run-1"
            self.edit("crates/x.rs", "crates/x.rs")
            prepare()
            _ok, _passed, _total, line, _result = cd.run_control_replay()
            self.assertEqual(self.replays, [self.binary], label)
            self.assertNotIn("reused", line, label)
            cd.clean_replay_evidence.discard(cd.clean_replay_evidence.evidence_path(self.root))

    def test_the_control_the_replay_gate_and_the_runner_agree_on_the_build_the_judgement_and_the_run(self):
        import inspect
        sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "gates"))
        import gates
        self.assertEqual(cd.REPLAY_TIMEOUT, cd.clean_replay_evidence.REPLAY_TIMEOUT_SECONDS)
        self.assertEqual(gates.RUN_ID_ENV, cd.clean_replay_evidence.RUN_ID_ENV)
        self.assertIn("clean_replay_evidence.CLEAN_REPLAY_COMMAND", inspect.getsource(self.control_builder))
        self.assertIn("evidence.CLEAN_REPLAY_COMMAND", inspect.getsource(__import__("replay").prepare_binary))


if __name__ == "__main__":
    unittest.main()
