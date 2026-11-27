"""The mutant table must describe the code as it is now: a pattern that no longer applies exactly once is an ERROR, never a valid mutant.
Three of the four patterns that went stale with the mailbox refactors (#302, #304) were rewritten against the current code. The fourth,
`mailbox-is-lifo`, targets code that no longer exists and its behaviour (newest message first) is covered by the live mutant
`mailbox-queue-is-lifo`: the Owner authorised its retirement (Q-X-20 instruction, 2026-11-21) on the condition that the obsolescence and the
coverage are demonstrated, and that the history stays traceable. A retired mutant is not hidden: it lives in RETIRED_MUTANTS with its reason,
and the tests below keep the retirement honest (a pattern that applies again, or a covering mutant that stops covering, fails them)."""
import os, re, sys, unittest

import mutant_runner as mr

sys.path.insert(0, os.path.join(mr.ROOT, "tools", "parity"))
import check_divergence as cd

REWRITTEN = ("mailbox-close-forgets-queued-messages", "throughput-ignored", "failure-does-not-suspend-mailbox")
REPORTS = os.path.join(mr.ROOT, "rust", "divergence-reports")


def cases_listed_in(report_name):
    with open(os.path.join(REPORTS, report_name), encoding="utf-8") as report_file:
        return set(re.findall(r"(?m)^### (\S+)\s*$", report_file.read()))


def recorded_cases():
    return {name[:-5] for name in os.listdir(os.path.join(mr.ROOT, "rust", "app-cases")) if name.endswith(".toml")}


class RewrittenPatterns(unittest.TestCase):
    def test_each_rewritten_pattern_applies_exactly_once_to_the_current_source_and_changes_it(self):
        for name in REWRITTEN:
            relative_file, old, new, _why = cd.MUTANTS[name]
            with open(os.path.join(mr.ROOT, mr.CRATE, relative_file), encoding="utf-8") as source_file:
                source = source_file.read()
            self.assertEqual(source.count(old), 1, f"{name}: the pattern must occur exactly once in {relative_file}")
            self.assertNotEqual(source.replace(old, new), source, name)

    def test_no_stale_pattern_is_left_and_none_is_hidden(self):
        stale = mr.find_stale_patterns(cd.MUTANTS, mr.ROOT) + mr.find_stale_patterns(cd.TOOL_MUTANTS, mr.ROOT, "tools/parity/")
        self.assertEqual(stale, [])

    def test_a_new_stale_pattern_is_still_an_error_for_the_runner_and_is_not_excused_by_the_retirement(self):
        table = {**cd.MUTANTS, "a-pattern-that-went-stale-later": ("src/actor/cell.rs", "this text does not occur anywhere", "x", "w")}
        self.assertEqual(mr.find_stale_patterns(table, mr.ROOT), ["a-pattern-that-went-stale-later"])
        self.assertEqual(mr.exit_code(undetected=[], errors=["a-pattern-that-went-stale-later"], pending=[], control="CLEAN"), 2)


class RetiredMutants(unittest.TestCase):
    def test_a_retired_mutant_is_out_of_the_active_table_and_says_why_who_and_where_its_history_is(self):
        self.assertIn("mailbox-is-lifo", cd.RETIRED_MUTANTS)
        for name, entry in cd.RETIRED_MUTANTS.items():
            self.assertNotIn(name, cd.MUTANTS, name)
            for field in ("file", "old", "new", "why", "covered_by", "retired_on", "authorized_by", "evidence"):
                self.assertTrue(entry.get(field), f"{name}: missing {field}")
            self.assertTrue(os.path.exists(os.path.join(REPORTS, f"replay-report-{name}.md")), f"{name}: its historical report must stay")

    def test_a_retirement_is_only_valid_while_the_pattern_is_really_obsolete(self):
        for name, entry in cd.RETIRED_MUTANTS.items():
            with open(os.path.join(mr.ROOT, mr.CRATE, entry["file"]), encoding="utf-8") as source_file:
                occurrences = source_file.read().count(entry["old"])
            self.assertEqual(occurrences, 0, f"{name}: the pattern applies again, so it is not obsolete: it must be a valid mutant, not a retired one")

    def test_the_retired_behaviour_stays_covered_by_a_live_mutant_that_kills_every_case_that_killed_it(self):
        for name, entry in cd.RETIRED_MUTANTS.items():
            covering = entry["covered_by"]
            self.assertIn(covering, cd.MUTANTS, f"{name}: the covering mutant must be active")
            self.assertNotIn(covering, mr.find_stale_patterns(cd.MUTANTS, mr.ROOT))
            old_killers = cases_listed_in(f"replay-report-{name}.md") & recorded_cases()
            live_killers = cases_listed_in(f"replay-report-{covering}.md")
            self.assertTrue(old_killers, f"{name}: no historical killers to compare")
            self.assertEqual(sorted(old_killers - live_killers), [], f"{name}: cases that killed it and no longer kill its covering mutant")

    def test_check_patterns_reports_the_retired_mutants_separately_from_the_stale_ones(self):
        import contextlib, io
        output = io.StringIO()
        with contextlib.redirect_stdout(output), self.assertRaises(SystemExit) as stop:
            cd.check_patterns()
        self.assertEqual(stop.exception.code, 0)
        self.assertIn("RETIRED mailbox-is-lifo", output.getvalue())
        self.assertIn("covered by mailbox-queue-is-lifo", output.getvalue())
        self.assertIn("0 stale", output.getvalue())

    def test_a_retired_mutant_does_not_count_among_the_runnable_mutants(self):
        self.assertNotIn("mailbox-is-lifo", {**cd.MUTANTS, **cd.TOOL_MUTANTS})
        # exact, so that the corpus can neither shrink nor grow unnoticed: whoever adds or retires a mutant updates this number
        self.assertEqual(len(cd.MUTANTS) + len(cd.TOOL_MUTANTS), 405)


if __name__ == "__main__":
    unittest.main()
