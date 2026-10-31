#!/usr/bin/env python3
"""docs/unit-rows.tsv assigns each contract-map row to exactly one unit; check.py rejects a row without unit or in two units. Runs in a temporary copy."""
import os, subprocess, sys, unittest
from package_sandbox import PackageSandbox

UNIT_ROWS = "docs/unit-rows.tsv"


class UnitRows(PackageSandbox):
    def run_check(self):
        return subprocess.run([sys.executable, os.path.join(self.root, "tools/package/check.py")], cwd=self.root, capture_output=True, text=True)

    def lines(self):
        return self.read(UNIT_ROWS).splitlines()

    def test_real_assignment_passes(self):
        result = self.run_check()
        # the sandbox has no upstream/ submodule, so citation checks fail there; only unit-rows findings matter here
        self.assertNotIn("unit-rows.tsv", result.stdout)
        self.assertNotIn("unit-plan.md", result.stdout)

    def test_row_without_unit_is_rejected(self):
        lines = self.lines()
        removed = lines.pop(1).split("\t")[0]
        self.write(UNIT_ROWS, "\n".join(lines) + "\n")
        result = self.run_check()
        self.assertEqual(1, result.returncode)
        self.assertIn(f"{removed} is in 0 units", result.stdout)

    def test_row_in_two_units_is_rejected(self):
        lines = self.lines()
        row_id = lines[1].split("\t")[0]
        self.write(UNIT_ROWS, "\n".join(lines + [f"{row_id}\tP64"]) + "\n")
        result = self.run_check()
        self.assertEqual(1, result.returncode)
        self.assertIn(f"{row_id} is in 2 units", result.stdout)


if __name__ == "__main__":
    unittest.main()
