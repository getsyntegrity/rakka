"""Q-X-22 R3: the PR tier reuses only DETECTED mutant results whose whole identity is unchanged, and the control always runs."""
import tempfile, unittest

import gates
import pr_mutants
from mutant_runner import ResultStore

TOOLS = {"rustc": "rustc 1", "cargo": "cargo 1", "rust-anything": "rust-anything 2.0.0", "python": "3.12", "platform": "mac"}
STATE = {"crate": "crate-a", "harness": "harness-a", "tools": TOOLS, "environment": {}}
PATCH = ["src/actor/cell.rs", "old", "new"]


class Identity(unittest.TestCase):
    def test_the_same_state_gives_the_same_key(self):
        self.assertEqual(pr_mutants.reuse_key("m", PATCH, STATE), pr_mutants.reuse_key("m", list(PATCH), dict(STATE)))

    def test_every_part_of_the_identity_changes_the_key(self):
        reference = pr_mutants.reuse_key("m", PATCH, STATE)
        changed = [{**STATE, "crate": "crate-b"}, {**STATE, "harness": "harness-b"}, {**STATE, "environment": {"RUSTFLAGS": "-g"}}]
        changed += [{**STATE, "tools": {**TOOLS, name: "other"}} for name in TOOLS]
        for state in changed:
            self.assertNotEqual(reference, pr_mutants.reuse_key("m", PATCH, state), state)
        self.assertNotEqual(reference, pr_mutants.reuse_key("m", PATCH + ["a_unit_test"], STATE))

    def test_an_external_tool_that_cannot_answer_forbids_reuse(self):
        self.assertIsNone(pr_mutants.reuse_key("m", PATCH, {**STATE, "tools": {**TOOLS, "rust-anything": None}}))


class Run(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.store = ResultStore(directory.name)
        self.executed = []

    def run_selected(self, outcomes, names=("a",), key=lambda name: "key-" + name, control="CLEAN", reusable=lambda name: True):
        def execute(name):
            self.executed.append(name)
            return {"outcome": outcomes[name]}
        return pr_mutants.run_selected(list(names), key, self.store, execute, lambda: control, reusable)

    def test_a_detected_result_is_reused_and_reported_as_not_executed(self):
        self.run_selected({"a": "DETECTED"})
        report = self.run_selected({"a": "DETECTED"})
        self.assertEqual((self.executed, report["reused"], report["exit"]), (["a"], ["a"], 0))

    def test_no_other_outcome_is_ever_reused(self):
        for outcome in ("TIMEOUT", "UNDETECTED", "HARNESS_ERROR", "BUILD_ERROR", "PATCH_ERROR", "INPUTS_CHANGED"):
            self.executed.clear()
            self.run_selected({"a": outcome}, key=lambda name: outcome)
            report = self.run_selected({"a": outcome}, key=lambda name: outcome)
            self.assertEqual((self.executed, report["reused"]), (["a", "a"], []), outcome)
            self.assertNotEqual(report["exit"], 0, outcome)

    def test_a_changed_identity_runs_the_mutant_again(self):
        self.run_selected({"a": "DETECTED"})
        self.run_selected({"a": "DETECTED"}, key=lambda name: "another")
        self.assertEqual(self.executed, ["a", "a"])

    def test_without_an_identity_or_for_a_mutant_that_is_not_reusable_nothing_is_reused(self):
        for options in (dict(key=lambda name: None), dict(reusable=lambda name: False)):
            self.executed.clear()
            self.run_selected({"a": "DETECTED"}, **options)
            self.run_selected({"a": "DETECTED"}, **options)
            self.assertEqual(self.executed, ["a", "a"], options)

    def test_inputs_that_change_while_a_mutant_runs_leave_no_evidence(self):
        states = iter(["before", "after"])
        report = self.run_selected({"a": "DETECTED"}, key=lambda name: next(states))
        self.assertEqual(report["outcomes"], {"a": "INPUTS_CHANGED"})
        self.assertFalse(self.store.is_fresh("before"))
        self.assertEqual(report["exit"], 2)

    def test_an_interrupted_run_keeps_what_was_already_proved(self):
        def execute(name):
            if name == "b":
                raise KeyboardInterrupt
            return {"outcome": "DETECTED"}
        with self.assertRaises(KeyboardInterrupt):
            pr_mutants.run_selected(["a", "b"], lambda name: name, self.store, execute, lambda: "CLEAN", lambda name: True)
        self.assertEqual((self.store.is_fresh("a"), self.store.is_fresh("b")), (True, False))

    def test_the_control_runs_every_time_and_a_dirty_one_never_passes(self):
        self.run_selected({"a": "DETECTED"})
        self.assertEqual(self.run_selected({"a": "DETECTED"}, control="FAILED")["exit"], 1)
        self.assertEqual(self.run_selected({"a": "DETECTED"}, control="ERROR")["exit"], 2)


class GateWiring(unittest.TestCase):
    def test_the_pr_mutants_gate_runs_the_per_mutant_runner_and_is_never_reused_as_a_whole(self):
        gate = next(g for g in gates.tier_gates("pr", ["a", "b"]) if g.name == "mutants")
        self.assertEqual(gate.command[1:], ["tools/gates/pr_mutants.py", "a", "b"])
        self.assertFalse(gate.reusable)
        self.assertTrue(all(g.reusable for g in gates.tier_gates("pr", ["a"]) if g.name != "mutants"))


if __name__ == "__main__":
    unittest.main()
