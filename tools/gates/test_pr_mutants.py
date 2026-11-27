"""Q-X-22 R3: the PR tier reuses only DETECTED mutant results whose whole identity is unchanged, and the control always runs."""
import os, tempfile, unittest
from unittest import mock

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

    def run_selected(self, outcomes, names=("a",), key=lambda name: "key-" + name, control="CLEAN", reusable=lambda name: True, total=3000, **options):
        def execute(name):
            self.executed.append(name)
            return {"outcome": outcomes[name], "total": total, "run": "run-1"}
        return pr_mutants.run_selected(list(names), key, self.store, execute, lambda: (control, 3000), reusable, **options)

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
        saved = []
        with self.assertRaises(KeyboardInterrupt):
            pr_mutants.run_selected(["a", "b"], lambda name: name, self.store, execute, lambda: ("CLEAN", None), lambda name: True, save=saved.append)
        self.assertEqual((self.store.is_fresh("a"), self.store.is_fresh("b")), (True, False))
        self.assertEqual((saved[-1]["executed"], saved[-1]["not_executed"]), (["a"], ["b"]))

    def test_no_reuse_executes_every_mutant_and_still_records_the_new_proof(self):
        self.run_selected({"a": "DETECTED"})
        report = self.run_selected({"a": "DETECTED"}, reuse=False)
        self.assertEqual((self.executed, report["reused"]), (["a", "a"], []))
        self.assertTrue(self.store.is_fresh("key-a"))

    def test_a_mutant_replay_of_another_corpus_size_than_the_control_is_an_error_and_proves_nothing(self):
        report = self.run_selected({"a": "DETECTED"}, total=12)
        self.assertEqual((report["outcomes"], report["exit"]), ({"a": "CORPUS_MISMATCH"}, 2))
        self.assertFalse(self.store.is_fresh("key-a"))
        self.assertEqual(self.run_selected({"a": "DETECTED"}, total=None)["exit"], 0)      # fast-kill and unit mutants replay no corpus

    def test_a_state_that_moves_after_the_control_ran_never_passes(self):
        states = iter(["checked", "moved"])
        report = self.run_selected({"a": "DETECTED"}, state_now=lambda: next(states))
        self.assertEqual((report["outcomes"]["(control)"], report["exit"]), ("INPUTS_CHANGED", 2))

    def test_a_reused_result_names_the_run_that_proved_it(self):
        self.run_selected({"a": "DETECTED"})
        self.assertEqual(self.run_selected({"a": "DETECTED"})["reused_from"], {"a": "run-1"})

    def test_the_control_runs_every_time_and_a_dirty_one_never_passes(self):
        self.run_selected({"a": "DETECTED"})
        self.assertEqual(self.run_selected({"a": "DETECTED"}, control="FAILED")["exit"], 1)
        self.assertEqual(self.run_selected({"a": "DETECTED"}, control="ERROR")["exit"], 2)


class ControlResult(unittest.TestCase):
    ROW = "| control (unmodified) | CLEAN | 2977/2977 |\n"

    def test_the_corpus_size_is_read_from_the_control_row(self):
        self.assertEqual(pr_mutants.control_result(0, "noise\n" + self.ROW), ("CLEAN", 2977))

    def test_a_clean_exit_that_does_not_say_how_much_it_replayed_is_an_error_not_an_unchecked_pass(self):
        self.assertEqual(pr_mutants.control_result(0, "| control (unmodified) | CLEAN | - |\n"), ("ERROR", None))

    def test_any_other_exit_keeps_its_meaning(self):
        self.assertEqual([pr_mutants.control_result(code, self.ROW)[0] for code in (1, 2, 75)], ["FAILED", "ERROR", "ERROR"])


class ToolIdentity(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = os.path.realpath(directory.name)

    def write(self, relative, text):
        path = os.path.join(self.root, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w") as out:
            out.write(text)
        return path

    def test_the_bytes_of_rust_anything_are_part_of_the_identity_even_at_the_same_version(self):
        binary = self.write("bin/rust-anything", "one")
        before = pr_mutants.external_identity(self.root, {}, binary)
        self.write("bin/rust-anything", "two!")
        self.assertNotEqual(before, pr_mutants.external_identity(self.root, {}, binary))
        self.assertIsNone(pr_mutants.external_identity(self.root, {}, None)["rust-anything-binary"])

    def test_a_cargo_configuration_above_the_checkout_or_in_cargo_home_is_part_of_the_identity(self):
        checkout, home = os.path.join(self.root, "outer", "checkout"), os.path.join(self.root, "home")
        os.makedirs(checkout)
        before = pr_mutants.external_identity(checkout, {"CARGO_HOME": home}, None)
        for relative in ("outer/.cargo/config.toml", "home/config.toml"):
            self.write(relative, "[build]\n")
            after = pr_mutants.external_identity(checkout, {"CARGO_HOME": home}, None)
            self.assertNotEqual(before, after, relative)
            before = after


class GateWiring(unittest.TestCase):
    def test_no_reuse_reaches_the_per_mutant_runner(self):
        command = lambda **options: next(g for g in gates.tier_gates("pr", ["a"], **options) if g.name == "mutants").command[1:]
        self.assertEqual(command(no_reuse=True), ["tools/gates/pr_mutants.py", "a", "--no-reuse"])
        self.assertEqual(command(), ["tools/gates/pr_mutants.py", "a"])
        with mock.patch.object(gates, "PR_MUTATION_SUSPENDED", None), \
                mock.patch.object(gates, "changed_files", return_value=["crates/rakka-actor/src/actor/cell.rs"]), mock.patch("builtins.print") as printed:
            gates.main(["pr", "--no-reuse", "--list"])
        listed = [call.args for call in printed.call_args_list if call.args[0] == "mutants"]
        self.assertTrue(listed[0][1].endswith("--no-reuse"), listed)

    def test_the_pr_mutants_gate_runs_the_per_mutant_runner_and_is_never_reused_as_a_whole(self):
        gate = next(g for g in gates.tier_gates("pr", ["a", "b"]) if g.name == "mutants")
        self.assertEqual(gate.command[1:], ["tools/gates/pr_mutants.py", "a", "b"])
        self.assertFalse(gate.reusable)
        self.assertEqual([g.name for g in gates.tier_gates("pr", ["a"]) if not g.reusable], ["python-tests", "mutants"])


if __name__ == "__main__":
    unittest.main()
