#!/usr/bin/env python3
"""Decisions declared by a contract block reach contract-map.tsv; check.py detects omissions; a `group` decision never blocks."""
import os, subprocess, sys, unittest
from package_sandbox import PackageSandbox


class DecisionTraceability(PackageSandbox):
    def run_script(self, relative):
        return subprocess.run([sys.executable, "-I", os.path.join(self.root, relative)], cwd=self.root, capture_output=True, text=True,
                              env=dict(os.environ, RAKKA_PACKAGE_ROOT=self.root))

    def decisions_of(self, row_id):
        for line in self.read("docs/contract-map.tsv").splitlines()[1:]:
            fields = line.split("\t")
            if fields[0] == row_id:
                return fields[9].split(",")

    def test_contract_declared_decisions_reach_the_map(self):
        self.regenerate()
        self.assertEqual(["Q-X-3", "Q-X-8"], self.decisions_of("AT-ASK-003"))

    def test_omitted_declared_decision_is_detected(self):
        self.regenerate()
        text = self.read("docs/contract-map.tsv").replace("Q-X-3,Q-X-8", "Q-X-3", 1)
        self.write("docs/contract-map.tsv", text)
        output = self.run_script("tools/package/check.py").stdout
        self.assertIn("AT-ASK-003: contract declares Q-X-8 but contract-map.tsv decisions column omits it", output)

    def test_group_decision_is_not_listed_as_pending(self):
        self.regenerate()
        self.assertEqual(0, self.run_script("tools/package/unit_plan.py").returncode)
        listing = self.read("docs/unit-decisions.md")
        self.assertNotIn("| Q-X-9 |", listing)
        self.assertIn("| Q-UTIL-3 |", listing)


if __name__ == "__main__":
    unittest.main()
