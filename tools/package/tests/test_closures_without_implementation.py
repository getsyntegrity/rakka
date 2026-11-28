#!/usr/bin/env python3
"""A row the Owner closed without own implementation is never shown as implemented, and `validation=documented` is only
accepted on a done row that carries the marker of an accepted decision. Expectations are literal. Runs in a temporary copy."""
import os, shutil, subprocess, sys, unittest
from package_sandbox import OWNER_FILE, REPO, PackageSandbox

INVENTORY = "docs/inventory/actor-classic.tsv"
CLOSED = "AC-UTIL-018.1"       # removed in 2.0.0, validation=documented
IMPLEMENTED = "AC-PATH-001"
MARKER = "SIN IMPLEMENTACIÓN PROPIA: ELIMINADO EN 2.0.0 (Q-UTIL-27, aceptada)"


class ClosuresWithoutImplementation(PackageSandbox):
    def setUp(self):
        super().setUp()
        shutil.copy(os.path.join(REPO, OWNER_FILE), os.path.join(self.root, OWNER_FILE))  # these rules need the real answers

    def fields(self, relative, row_id):
        return next(line.split("\t") for line in self.read(relative).splitlines() if line.split("\t")[0] == row_id)

    def set_row(self, row_id, impl=None, validation=None, notes=None):
        lines = self.read(INVENTORY).split("\n")
        for n, line in enumerate(lines):
            row = line.split("\t")
            if row[0] != row_id: continue
            for column, value in ((11, impl), (12, validation), (13, notes)):
                if value is not None: row[column] = value
            lines[n] = "\t".join(row)
        self.write(INVENTORY, "\n".join(lines))

    def check_output(self):
        return subprocess.run([sys.executable, "-I", os.path.join(self.root, "tools/package/check.py")], cwd=self.root,
                              capture_output=True, text=True, env=dict(os.environ, RAKKA_PACKAGE_ROOT=self.root)).stdout

    def test_contract_map_does_not_call_a_closed_row_implemented(self):
        self.regenerate()
        closed = self.fields("docs/contract-map.tsv", CLOSED)
        self.assertEqual(("(sin implementación propia)", "decisión del Owner"), (closed[3], closed[8]))
        self.assertEqual("(implementada)", self.fields("docs/contract-map.tsv", IMPLEMENTED)[3])

    def test_unit_spec_lists_closed_rows_apart_from_done_rows(self):
        self.regenerate()
        folder = os.path.join(self.root, "docs/superpowers/plans/units")
        name = next(f for f in os.listdir(folder) if f.endswith("-ac-util-1.md"))
        lines = self.read("docs/superpowers/plans/units/" + name).splitlines()
        done = next(line for line in lines if line.startswith("Filas ya `done`"))
        closed = next(line for line in lines if line.startswith("Filas cerradas sin implementación propia"))
        self.assertNotIn(CLOSED, done)
        self.assertIn(CLOSED, closed)

    def test_progress_says_closed_rows_have_no_recorded_cases_by_owner_decision(self):
        self.regenerate()
        self.assertIn("por decisión del Owner no tienen casos grabados propios", self.read("docs/progress.md"))

    def test_check_accepts_the_repository_closures(self):
        self.regenerate()
        self.assertNotIn(CLOSED + ":", self.check_output())

    def test_check_rejects_documented_on_a_row_that_is_not_done(self):
        self.set_row(CLOSED, impl="partial")
        self.assertIn(CLOSED + ": validation=documented needs impl=done", self.check_output())

    def test_check_rejects_documented_without_the_marker(self):
        self.set_row(CLOSED, notes="sin marca")
        self.assertIn(CLOSED + ": validation=documented needs the closure marker", self.check_output())

    def test_check_rejects_a_marker_whose_decision_is_not_accepted(self):
        self.set_row(CLOSED, notes=MARKER.replace("Q-UTIL-27", "Q-DSP-3"))
        self.assertIn(CLOSED + ": closure marker cites Q-DSP-3, which the Owner has not accepted", self.check_output())

    def test_applying_a_comparison_keeps_a_closed_row(self):
        os.makedirs(os.path.join(self.root, "cases"))
        compare = os.path.join(self.root, "compare")
        os.makedirs(compare)
        self.write("compare/util.tsv", "row_id\tstatus\tcovered\tmissing\tcases\topen_q\tnote\n" + CLOSED + "\tpending\t-\tall\t-\t-\t-\n")
        before = self.fields(INVENTORY, CLOSED)
        subprocess.run([sys.executable, "-I", os.path.join(self.root, "tools/parity/apply_comparison.py"), compare, "--apply"],
                       check=True, capture_output=True, text=True)
        self.assertEqual(before, self.fields(INVENTORY, CLOSED))


if __name__ == "__main__":
    unittest.main()
