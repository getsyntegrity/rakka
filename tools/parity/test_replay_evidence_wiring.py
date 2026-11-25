"""replay.py leaves evidence only for a complete clean replay inside a gate run; a replay that fails, is incomplete or interrupted leaves none, nor does an earlier one survive."""
import contextlib, io, os, pathlib, shutil, tempfile, types, unittest
from unittest import mock

import clean_replay_evidence as evidence
import replay

TOOLS = {"rustc": "r", "cargo": "c", "rust-anything": "a", "python": "p", "platform": "x"}
CLEAN = dict(ok=True, line="2/2 applicable", applicable_failing=[], na_problems=[], passed=2, total=2, na_q_cfg_30=[], other_na=1)


class ReplayEvidence(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.root, True)
        for name in ("rust/app-cases/a.toml", "rust/app-cases/b.toml", "rust/app-cases/c.toml", "binary"):
            pathlib.Path(self.root, name).parent.mkdir(parents=True, exist_ok=True)
            pathlib.Path(self.root, name).write_text(name)
        self.path = evidence.evidence_path(self.root)

    def run_replay(self, evaluation=CLEAN, run_id="run-1", error=None, replayed=3, during=lambda: None):
        def run(*_):
            during()
            if error:
                raise error
            return types.SimpleNamespace(total=replayed, passed=replayed)
        with contextlib.ExitStack() as stack, contextlib.redirect_stdout(io.StringIO()):
            for patch in (mock.patch.object(replay.lib, "ROOT", self.root), mock.patch.object(replay, "prepare_binary", return_value=os.path.join(self.root, "binary")),
                          mock.patch.object(replay.lib, "holding", return_value=contextlib.nullcontext()), mock.patch.object(replay.lib, "run_replay", side_effect=run),
                          mock.patch.object(replay.lib, "evaluate", return_value=evaluation), mock.patch.object(evidence, "current_tool_versions", return_value=TOOLS),
                          mock.patch.dict(os.environ, {evidence.RUN_ID_ENV: run_id} if run_id else {})):
                stack.enter_context(patch)
            if not run_id:
                os.environ.pop(evidence.RUN_ID_ENV, None)
            return replay.main(["--build"])

    def test_a_complete_clean_replay_inside_a_gate_run_leaves_evidence_that_is_reusable_only_in_that_run(self):
        self.assertEqual(self.run_replay(), 0)
        identity = evidence.identity(self.root, os.path.join(self.root, "binary"), {}, TOOLS)
        self.assertIsNotNone(evidence.reusable(self.path, identity, "run-1", corpus_cases=3)[0])
        self.assertIsNone(evidence.reusable(self.path, identity, "run-2", corpus_cases=3)[0])

    def test_a_replay_that_is_outside_a_gate_run_diverges_is_rejected_incomplete_or_interrupted_leaves_no_evidence_not_even_an_earlier_one(self):
        outcomes = (dict(run_id=None), dict(evaluation=dict(CLEAN, ok=False, applicable_failing=["c"])), dict(evaluation=dict(CLEAN, ok=None, passed=None, total=None)),
                    dict(evaluation=dict(CLEAN, na_problems=["x"], ok=False)), dict(replayed=2), dict(error=KeyboardInterrupt()),     # 2 of the 3 recorded cases
                    dict(during=lambda: open(os.path.join(self.root, "rust/app-cases/a.toml"), "a").write(" changed while the replay ran")))
        for outcome in outcomes:
            self.run_replay()
            self.assertTrue(os.path.exists(self.path))
            with contextlib.suppress(KeyboardInterrupt):
                self.run_replay(**outcome)
            self.assertFalse(os.path.exists(self.path), outcome)


if __name__ == "__main__":
    unittest.main()
