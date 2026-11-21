"""Unit tests of the gate runner's pure decisions: which mutants a change selects, when evidence may be reused, what reaches the console."""
import unittest

import gates

MUTANTS = {
    "queue-is-lifo": ("src/dispatch/message_queue.rs", "o", "n", "why"),
    "cell-throughput": ("src/actor/cell.rs", "o", "n", "why"),
    "cell-dead-letters": ("src/actor/cell.rs", "o", "n", "why"),
}
TOOL_MUTANTS = {"fuzz-diff-mutant": ("fuzz_diff.py", "o", "n", "why")}


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
        self.assertEqual(gates.select_mutants(["tools/parity/fuzz_diff.py"], MUTANTS, TOOL_MUTANTS), ["fuzz-diff-mutant"])

    def test_the_helpers_that_decide_whether_a_mutant_is_detected_select_every_mutant(self):
        everything = list(MUTANTS) + list(TOOL_MUTANTS)
        for helper in ("tools/parity/replay_lib.py", "tools/parity/na_q_cfg_30.py"):
            self.assertEqual(gates.select_mutants([helper], MUTANTS, TOOL_MUTANTS), everything, helper)


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


class RecordingsAndRegeneratedFiles(unittest.TestCase):
    def test_a_gate_that_reads_the_recordings_declares_the_report_it_rewrites_so_the_others_can_reuse_evidence(self):
        for gate in gates.tier_gates("milestone", ["x"]):
            if "rust/app-cases" in gate.inputs:
                self.assertIn("rust/app-cases/replay-report.md", gate.regenerates, gate.name)

    def test_the_fuzz_gates_do_not_depend_on_the_recordings(self):
        fuzz = [g for g in gates.tier_gates("milestone", []) if g.name.startswith("fuzz-")]
        self.assertTrue(fuzz)
        for gate in fuzz:
            self.assertNotIn("rust/app-cases", gate.inputs, gate.name)


class VerdictLabels(unittest.TestCase):
    def test_the_mutants_gate_reports_the_runner_verdict_and_any_other_gate_just_fails(self):
        mutants = next(g for g in gates.tier_gates("milestone", []) if g.name == "mutants")
        clippy = next(g for g in gates.tier_gates("pr", []) if g.name == "clippy")
        self.assertEqual([gates.gate_label(mutants, code) for code in (0, 1, 2, 3, 143)], ["PASS", "FAIL", "ERROR", "INCOMPLETE", "CANCELLED"])
        self.assertEqual([gates.gate_label(clippy, code) for code in (0, 1, 101)], ["PASS", "FAIL", "FAIL"])

    def test_the_failure_line_starts_with_the_given_label(self):
        line = gates.verdict_line("mutants", 3, 9.0, reused=False, log_path="x.log", log_text="tail", label="INCOMPLETE")
        self.assertTrue(line.startswith("INCOMPLETE mutants exit=3"))
        self.assertTrue(gates.verdict_line("test", 101, 1.0, reused=False, log_path="x.log", log_text="t").startswith("FAIL test exit=101"))

    def test_the_tier_summary_names_each_non_passing_gate_with_its_verdict(self):
        self.assertEqual(gates.summary_line("milestone", [("mutants", "INCOMPLETE"), ("clippy", "FAIL")], 12, "abc"),
                         "MILESTONE: INCOMPLETE mutants, FAIL clippy (12 gates, HEAD abc)")
        self.assertEqual(gates.summary_line("pr", [], 9, "abc"), "PR: all gates passed (9 gates, HEAD abc)")


class ChangedPaths(unittest.TestCase):
    def test_paths_with_spaces_and_renames_are_parsed_from_nul_separated_git_output(self):
        committed = "a b.rs\0src/x.rs\0"
        working = " M tools/with space.py\0R  new.rs\0old.rs\0?? untracked.txt\0"
        self.assertEqual(gates.changed_paths_from_git(committed, working),
                         ["a b.rs", "new.rs", "old.rs", "src/x.rs", "tools/with space.py", "untracked.txt"])

    def test_nothing_changed_is_an_empty_list(self):
        self.assertEqual(gates.changed_paths_from_git("", ""), [])

    def test_a_base_that_does_not_exist_is_refused_instead_of_silently_ignored(self):
        with self.assertRaises(gates.UsageError):
            gates.changed_files("origin/this-ref-does-not-exist-anywhere")


class BaseArgument(unittest.TestCase):
    def test_a_valid_base_is_kept_for_the_pr_gate_and_has_a_default(self):
        self.assertEqual(gates.parse_arguments(["pr", "--base", "origin/feat/x"])["base"], "origin/feat/x")
        self.assertEqual(gates.parse_arguments(["pr"])["base"], gates.DEFAULT_BASE)
        self.assertEqual(gates.parse_arguments(["milestone", "--base", "origin/x"])["mutant_args"], ["--base", "origin/x"])

    def test_base_without_a_value_or_followed_by_another_flag_is_a_usage_error_in_every_tier(self):
        for argv in (["pr", "--base"], ["milestone", "--base"], ["fast", "--base"], ["milestone", "--base", "--no-reuse"],
                     ["pr", "--base", "--list"]):
            with self.assertRaises(gates.UsageError, msg=argv):
                gates.parse_arguments(argv)

    def test_a_ref_named_like_a_tier_is_a_value_not_a_second_tier(self):
        parsed = gates.parse_arguments(["--base", "fast", "pr"])
        self.assertEqual((parsed["tier"], parsed["base"]), ("pr", "fast"))

    def test_the_tier_is_required_and_unique(self):
        for argv in ([], ["fast", "pr"], ["--list"]):
            with self.assertRaises(gates.UsageError, msg=argv):
                gates.parse_arguments(argv)

    def test_main_reports_refused_with_exit_2_and_no_traceback(self):
        import io, contextlib
        for argv in (["milestone", "--base"], ["pr", "--base"], ["milestone", "--base", "--no-reuse"]):
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                code = gates.main(argv)
            self.assertEqual(code, 2, argv)
            self.assertIn("refused", out.getvalue())


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
