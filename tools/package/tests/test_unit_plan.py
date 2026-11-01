#!/usr/bin/env python3
"""unit_plan.py derives the execution order and the per-unit decisions; --check rejects stale generated parts, a dependency
on a later wave, a dependency cycle and a card whose dependencies differ from its table. Runs in a temporary copy."""
import os, re, subprocess, sys, unittest
from package_sandbox import PackageSandbox

PLAN = "docs/unit-plan.md"


class UnitPlan(PackageSandbox):
    def check(self):
        return subprocess.run([sys.executable, "-I", os.path.join(self.root, "tools/package/unit_plan.py"), "--check"], cwd=self.root,
                              capture_output=True, text=True, env=dict(os.environ, RAKKA_PACKAGE_ROOT=self.root))

    def test_committed_plan_is_current(self):
        result = self.check()
        self.assertEqual(0, result.returncode, result.stdout)

    def test_stale_execution_order_is_rejected(self):
        text = self.read(PLAN)
        self.write(PLAN, text.replace("| 1 | P02 |", "| 1 | P99 |", 1))
        result = self.check()
        self.assertEqual(1, result.returncode)
        self.assertIn("is stale", result.stdout)

    def test_card_and_table_dependencies_must_agree(self):
        text = self.read(PLAN)
        self.write(PLAN, re.sub(r"(### P03 .*?- \*\*Dependencias:\*\* )[^\n]*", r"\1P02", text, count=1, flags=re.S))
        result = self.check()
        self.assertEqual(1, result.returncode)
        self.assertIn("P03: table depends on", result.stdout)

    def test_dependency_cycle_is_rejected(self):
        text = self.read(PLAN)
        # P02 (wave 0) now "depends on" P77 (wave 0), which depends on P02: both table and card, so only the cycle is reported
        text = re.sub(r"(\| P02 \| UTIL-NUCLEO \| \d+ \| )-( \|)", r"\1P77\2", text, count=1)
        text = re.sub(r"(### P02 .*?- \*\*Dependencias:\*\* )[^\n]*", r"\1P77", text, count=1, flags=re.S)
        self.write(PLAN, text)
        result = self.check()
        self.assertEqual(1, result.returncode)
        self.assertIn("dependency cycle", result.stdout)

    def test_dependency_on_a_later_wave_is_rejected(self):
        text = self.read(PLAN)
        text = re.sub(r"(\| P02 \| UTIL-NUCLEO \| \d+ \| )-( \|)", r"\1P64\2", text, count=1)
        text = re.sub(r"(### P02 .*?- \*\*Dependencias:\*\* )[^\n]*", r"\1P64", text, count=1, flags=re.S)
        self.write(PLAN, text)
        result = self.check()
        self.assertEqual(1, result.returncode)
        self.assertIn("later wave", result.stdout)

    def test_crate_must_match_the_module_of_the_rows(self):
        text = self.read(PLAN)
        self.write(PLAN, re.sub(r"(### P48 .*?- \*\*Archivos previstos:\*\* `)crates/rakka-actor-typed/", r"\1crates/rakka-actor/", text, count=1, flags=re.S))
        result = self.check()
        self.assertEqual(1, result.returncode)
        self.assertIn("P48: crates/rakka-actor/", result.stdout)

    def test_file_declared_existing_must_exist(self):
        text = self.read(PLAN)
        self.write(PLAN, text.replace("util/{timeout,time_unit,helpers}.rs", "util/{timeout,time_unit,missing_helper}.rs", 1))
        result = self.check()
        self.assertEqual(1, result.returncode)
        self.assertIn("missing_helper.rs is declared existing", result.stdout)

    def test_card_rows_must_match_unit_rows(self):
        text = self.read(PLAN)
        # a card that no longer lists AC-UTIL-016.6 (the only row of P05) while docs/unit-rows.tsv still assigns it
        text = re.sub(r"(### P05 .*?unit-rows.tsv`: )AC-UTIL-016\.6", r"\1AC-UTIL-016.5", text, count=1, flags=re.S)
        self.write(PLAN, text)
        result = self.check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Filas of the card differ", result.stdout)

    def test_row_dependency_must_be_covered_by_the_unit_graph(self):
        text = self.read(PLAN)
        # AC-SYS-020 depends on AC-UTIL-005.2 (P67): without P67 declared by P25 (or reached through it) the dependency is lost
        text = re.sub(r"(\| P25 \| \S+ \| \d+ \| )([^|]*)\|", lambda m: m.group(1) + m.group(2).replace(", P67", "") + "|", text, count=1)
        text = re.sub(r"(### P25 .*?- \*\*Dependencias:\*\* )[^\n]*", lambda m: m.group(1) + re.search(r"\| P25 \| \S+ \| \d+ \| ([^|]*)\|", text).group(1).strip() + ".", text, count=1, flags=re.S)
        self.write(PLAN, text)
        result = self.check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("not covered by the declared unit graph", result.stdout)

if __name__ == "__main__":
    unittest.main()
