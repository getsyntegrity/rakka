"""The control replay of check_divergence must run against a binary built from the current tree, never a leftover one."""
import unittest

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


if __name__ == "__main__":
    unittest.main()
