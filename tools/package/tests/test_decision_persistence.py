#!/usr/bin/env python3
"""An Owner response recorded in docs/decisions/owner-decisions.tsv survives regeneration, and regenerating twice is a no-op.

Runs entirely in a temporary copy of the documentation package; the real repository is never modified.
"""
import subprocess, unittest
from package_sandbox import PackageSandbox, OWNER_FILE, tree_digest


class DecisionPersistence(PackageSandbox):
    def proposed_count(self):
        statuses = [line.split("\t")[2] for line in self.read("docs/decisions/register.tsv").splitlines()[1:]]
        return statuses.count("proposed")

    def test_accepted_decision_is_kept_and_second_generation_changes_nothing(self):
        self.regenerate()
        proposed_before = self.proposed_count()
        self.append(OWNER_FILE, "Q-X-10\taccepted\t2030-01-02\ttest-owner\taceptar\n")

        self.regenerate()
        after_first = tree_digest(self.root)
        register = self.read("docs/decisions/register.tsv")
        accepted_line = next(line for line in register.splitlines() if line.startswith("Q-X-10\t"))
        self.assertIn("\taccepted\t", accepted_line)
        self.assertTrue(accepted_line.endswith("\t2030-01-02\ttest-owner\taceptar"))
        decisions_doc = self.read("docs/decisions.md")
        self.assertIn(f"{proposed_before - 1} propuestas, 1 aceptadas, 0 aceptadas con cambios, 0 rechazadas", decisions_doc)
        self.assertNotIn(f"{proposed_before} propuestas, 0 aceptadas", decisions_doc)
        self.assertIn("(test-owner, 2030-01-02)", decisions_doc)
        self.assertIn(f"{proposed_before - 1} propuestas, 1 aceptadas", self.read("README.md"))

        self.regenerate()
        self.assertEqual(after_first, tree_digest(self.root))

    def test_unknown_decision_is_rejected(self):
        self.append(OWNER_FILE, "Q-NOPE-1\taccepted\t2030-01-02\ttest-owner\taceptar\n")
        with self.assertRaises(subprocess.CalledProcessError):
            self.regenerate()


if __name__ == "__main__":
    unittest.main()
