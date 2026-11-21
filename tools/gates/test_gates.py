"""Unit tests of the gate runner's pure decisions: which mutants a change selects, when evidence may be reused, what reaches the console."""
import unittest

import gates

MUTANTS = {
    "queue-is-lifo": ("src/dispatch/message_queue.rs", "o", "n", "why"),
    "cell-throughput": ("src/actor/cell.rs", "o", "n", "why"),
    "cell-dead-letters": ("src/actor/cell.rs", "o", "n", "why"),
}
TOOL_MUTANTS = {"replay-lib-mutant": ("replay_lib.py", "o", "n", "why")}


class MutantSelection(unittest.TestCase):
    def test_a_change_selects_only_the_mutants_of_the_changed_source_file(self):
        picked = gates.select_mutants(["crates/rakka-actor/src/actor/cell.rs"], MUTANTS, TOOL_MUTANTS)
        self.assertEqual(picked, ["cell-throughput", "cell-dead-letters"])

    def test_a_change_outside_every_mutated_file_selects_nothing(self):
        self.assertEqual(gates.select_mutants(["docs/decisions.md"], MUTANTS, TOOL_MUTANTS), [])

    def test_a_change_to_the_replay_corpus_or_the_build_selects_every_mutant(self):
        everything = list(MUTANTS) + list(TOOL_MUTANTS)
        for changed in ("Cargo.lock", "cases/path-from-string.jsonl", "tools/parity/check_divergence.py", "rust/app-cases/a01.stdout", "oracle/pom.xml"):
            self.assertEqual(gates.select_mutants([changed], MUTANTS, TOOL_MUTANTS), everything, changed)

    def test_a_tool_file_change_selects_its_tool_mutants(self):
        self.assertEqual(gates.select_mutants(["tools/parity/replay_lib.py"], MUTANTS, TOOL_MUTANTS), ["replay-lib-mutant"])


class EvidenceReuse(unittest.TestCase):
    base = dict(command=["cargo", "test"], input_trees={"crates": "aaa"}, toolchain="rustc 1.80", dirty_inputs=False)

    def test_same_inputs_give_the_same_key(self):
        self.assertEqual(gates.evidence_key(**self.base), gates.evidence_key(**self.base))

    def test_each_component_changes_the_key(self):
        reference = gates.evidence_key(**self.base)
        for override in (dict(command=["cargo", "test", "-q"]), dict(input_trees={"crates": "bbb"}), dict(toolchain="rustc 1.81")):
            self.assertNotEqual(reference, gates.evidence_key(**{**self.base, **override}), override)

    def test_uncommitted_edits_in_the_inputs_forbid_reuse(self):
        self.assertIsNone(gates.evidence_key(**{**self.base, "dirty_inputs": True}))


class ConsoleOutput(unittest.TestCase):
    def test_a_passing_gate_prints_one_line_and_no_log(self):
        line = gates.verdict_line("clippy", 0, 12.3, reused=False, log_path="x.log", log_text="lots\nof\noutput")
        self.assertEqual(line, "PASS clippy (12s) log=x.log")

    def test_a_reused_gate_says_so(self):
        self.assertEqual(gates.verdict_line("deny", 0, 0.0, reused=True, log_path="x.log", log_text=""), "PASS deny (reused evidence) log=x.log")

    def test_a_failing_gate_shows_a_bounded_tail_only(self):
        log = "\n".join(f"line {i}" for i in range(1000))
        text = gates.verdict_line("test", 101, 5.0, reused=False, log_path="x.log", log_text=log)
        self.assertTrue(text.startswith("FAIL test exit=101 (5s) log=x.log"))
        self.assertIn("line 999", text)
        self.assertNotIn("line 500", text)
        self.assertLessEqual(len(text), gates.MAX_FAILURE_CHARS + 200)


class DirtyCheckPathspecs(unittest.TestCase):
    def test_only_gates_that_regenerate_files_exclude_them_from_the_dirty_check(self):
        self.assertEqual(gates.status_pathspecs(["crates"], regenerates=()), ["crates"])
        self.assertEqual(gates.status_pathspecs(["crates"], regenerates=("rust/divergence-reports",)), ["crates", ":(exclude)rust/divergence-reports"])

    def test_with_no_existing_input_there_is_nothing_to_check_instead_of_checking_the_whole_repository(self):
        self.assertIsNone(gates.status_pathspecs([], regenerates=("rust/divergence-reports",)))

    def test_the_replay_and_mutant_gates_declare_the_files_they_rewrite_and_the_others_declare_none(self):
        by_name = {g.name: g for g in gates.tier_gates("milestone", ["x"])}
        for rewriting in ("replay", "mutants"):
            self.assertIn("rust/app-cases/replay-report.md", by_name[rewriting].regenerates)
        for plain in ("fmt", "clippy", "deny", "package-check"):
            self.assertEqual(by_name[plain].regenerates, ())

    def test_the_replay_gate_is_invalidated_by_a_changed_recording(self):
        self.assertIn("rust/app-cases", next(g for g in gates.tier_gates("pr", []) if g.name == "replay").inputs)


class MutantFlagForwarding(unittest.TestCase):
    def test_flags_with_a_value_are_forwarded_for_the_milestone_tier(self):
        self.assertEqual(gates.mutant_arguments("milestone", ["milestone", "--max-mutants", "5", "--authorize-full-mutants"]),
                         ["--max-mutants", "5", "--authorize-full-mutants"])

    def test_a_flag_without_a_value_or_followed_by_another_flag_is_a_usage_error(self):
        for argv in (["milestone", "--max-minutes"], ["milestone", "--max-mutants", "--no-reuse"]):
            with self.assertRaises(gates.UsageError):
                gates.mutant_arguments("milestone", argv)

    def test_other_tiers_do_not_look_at_the_mutant_flags(self):
        self.assertEqual(gates.mutant_arguments("fast", ["fast", "--max-mutants"]), [])


class Tiers(unittest.TestCase):
    def test_every_existing_mandatory_gate_stays_in_the_pr_tier_and_milestone_adds_to_it(self):
        pr = {g.name for g in gates.tier_gates("pr", [])}
        for mandatory in ("fmt", "clippy", "test", "deny", "coverage", "replay", "package-check", "mutants"):
            self.assertIn(mandatory, pr)
        milestone = {g.name for g in gates.tier_gates("milestone", [])}
        self.assertTrue(pr < milestone)
        self.assertTrue(any(name.startswith("fuzz-") for name in milestone - pr))

    def test_fast_is_a_subset_of_pr(self):
        self.assertTrue({g.name for g in gates.tier_gates("fast", [])} < {g.name for g in gates.tier_gates("pr", [])})

    def test_the_milestone_mutants_gate_is_invalidated_by_its_own_runner_and_by_the_recordings(self):
        gate = next(g for g in gates.tier_gates("milestone", []) if g.name == "mutants")
        self.assertIn("tools/gates", gate.inputs)
        self.assertIn("rust/app-cases", gate.inputs)

    def test_milestone_runs_every_mutant_and_pr_only_the_selected_ones(self):
        mutants_gate = lambda tier, sel: next(g for g in gates.tier_gates(tier, sel) if g.name == "mutants")
        self.assertEqual(mutants_gate("pr", ["a", "b"]).command[-2:], ["a", "b"])
        self.assertNotIn("a", mutants_gate("milestone", ["a"]).command)


if __name__ == "__main__":
    unittest.main()
