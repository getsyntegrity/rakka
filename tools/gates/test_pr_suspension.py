"""Q-X-23 (temporary): the PR tier runs no mutant; what it would have run is reported as pending, never as passed. MILESTONE is unchanged."""
import contextlib, io, unittest
from unittest import mock

import gates


def mutants_gate(tier, selected):
    return next(gate for gate in gates.tier_gates(tier, selected) if gate.name == "mutants")


class PrSuspension(unittest.TestCase):
    def test_selected_mutants_are_not_run_and_are_reported_as_pending(self):
        with mock.patch.object(gates.subprocess, "run", side_effect=AssertionError("a campaign was started")):
            line, code, label = gates.run_gate(mutants_gate("pr", ["a", "b"]), "rustc", True, "head")
        self.assertEqual((code, label), (0, "PENDING"))
        self.assertTrue(line.startswith("PENDING mutants: NOT RUN"), line)
        self.assertIn("2 selected", line)
        self.assertIn("Q-X-23", line)

    def test_the_summary_never_reads_as_if_the_mutants_had_passed(self):
        summary = gates.summary_line("pr", [], 9, "abc", pending_mutants=2)
        self.assertIn("mutants NOT RUN: 2 pending", summary)
        self.assertNotIn("all gates passed", summary)
        self.assertEqual(gates.summary_line("pr", [], 9, "abc"), "PR: all gates passed (9 gates, HEAD abc)")

    def test_the_entry_point_counts_what_is_pending(self):
        printed = io.StringIO()
        report = {"selected": ["a", "b", "c"], "rules": [], "not_selected_by_rule": []}
        stub = [gates.Gate("mutants", ["false"], (), "NOT RUN: suspended", pending_mutants=3)]
        with mock.patch("pr_selection.select_for_pr", return_value=report), mock.patch.object(gates, "tier_gates", lambda *a, **k: stub), \
                contextlib.redirect_stdout(printed):
            code = gates.main(["pr"])
        self.assertEqual(code, 0)
        self.assertIn("mutants NOT RUN: 3 pending", printed.getvalue())

    def test_nothing_selected_is_a_plain_skip_with_nothing_pending(self):
        gate = mutants_gate("pr", [])
        self.assertEqual((gate.pending_mutants, gate.skip_reason), (0, "no mutated file changed since the base"))

    def test_the_milestone_tier_keeps_its_mutants(self):
        gate = mutants_gate("milestone", [])
        self.assertEqual((gate.skip_reason, gate.pending_mutants), ("", 0))
        self.assertIn("tools/gates/mutant_runner.py", gate.command)


if __name__ == "__main__":
    unittest.main()
