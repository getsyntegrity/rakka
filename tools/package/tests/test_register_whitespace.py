#!/usr/bin/env python3
"""The generated decision register has no trailing whitespace (`git diff --check` stays clean) and keeps every value.

An unanswered decision has empty `decided_on`, `decided_by` and `decision`; the generator must not leave their tab separators
dangling at the end of the line. Runs in a temporary copy of the package; the real repository is never modified.
"""
import csv, io, unittest
from package_sandbox import PackageSandbox, OWNER_FILE

REGISTER = "docs/decisions/register.tsv"
ANSWER_COLUMNS = ("decided_on", "decided_by", "decision")


def parse(text):
    return list(csv.DictReader(io.StringIO(text, newline=""), delimiter="\t", quoting=csv.QUOTE_NONE))


class RegisterWhitespace(PackageSandbox):
    def test_no_line_ends_with_whitespace(self):
        self.regenerate()
        offenders = [n for n, line in enumerate(self.read(REGISTER).splitlines(), 1) if line != line.rstrip()]
        self.assertEqual(offenders, [], f"{REGISTER}: lines with trailing whitespace")

    def test_unanswered_decisions_still_read_as_empty_answers(self):
        self.regenerate()
        unanswered = [r for r in parse(self.read(REGISTER)) if r["status"] == "proposed"]
        self.assertTrue(unanswered)
        for row in unanswered:
            self.assertTrue(all(not row[c] for c in ANSWER_COLUMNS), row["id"])

    def test_answered_decision_keeps_all_its_values(self):
        self.append(OWNER_FILE, "Q-X-10\taccepted\t2030-01-02\ttest-owner\taceptar\n")
        self.regenerate()
        row = next(r for r in parse(self.read(REGISTER)) if r["id"] == "Q-X-10")
        self.assertEqual((row["status"], row["decided_on"], row["decided_by"], row["decision"]), ("accepted", "2030-01-02", "test-owner", "aceptar"))


if __name__ == "__main__":
    unittest.main()
