#!/usr/bin/env python3
"""Decisions declared by a contract block reach contract-map.tsv in both declaration formats; check.py detects omissions,
normalizes aliases, reports unknown ids; declarations do not leak between blocks. Expectations are literal, not computed."""
import os, re, subprocess, sys, unittest
from package_sandbox import PackageSandbox

TYPED = "docs/inventory/contracts/typed-extended.md"
ASK_003_LINE = "- **Decisiones**: Q-X-3; Q-X-8"
MAP = "docs/contract-map.tsv"


class DecisionTraceability(PackageSandbox):
    def run_script(self, relative):
        return subprocess.run([sys.executable, "-I", os.path.join(self.root, relative)], cwd=self.root, capture_output=True, text=True,
                              env=dict(os.environ, RAKKA_PACKAGE_ROOT=self.root))

    def decisions_of(self, row_id):
        for line in self.read(MAP).splitlines()[1:]:
            fields = line.split("\t")
            if fields[0] == row_id:
                return fields[9]

    def declare_for_ask_003(self, declaration):
        self.write(TYPED, self.read(TYPED).replace(ASK_003_LINE, declaration, 1))

    def omit_from_map(self, row_id, decisions_text):
        self.write(MAP, re.sub(r"(?m)^(%s\t(?:[^\t]*\t){8})[^\t]*" % re.escape(row_id), lambda m: m.group(1) + decisions_text, self.read(MAP), count=1))

    def test_bullet_format_reaches_the_map(self):
        self.regenerate()
        self.assertEqual("Q-X-3,Q-X-8", self.decisions_of("AT-ASK-003"))

    def test_bold_period_format_reaches_the_map(self):
        self.declare_for_ask_003("**Decisiones.** Q-X-3, Q-X-8")
        self.regenerate()
        self.assertEqual("Q-X-3,Q-X-8", self.decisions_of("AT-ASK-003"))

    def test_serialization_declarations_reach_the_map(self):
        self.regenerate()
        self.assertIn("Q-X-2", self.decisions_of("AC-SER-001.1").split(","))
        self.assertIn("Q-X-7", self.decisions_of("AC-SER-004.2").split(","))
        self.assertIn("Q-X-4", self.decisions_of("AC-SER-006").split(","))

    def test_omission_is_detected_in_both_formats(self):
        for declaration in (ASK_003_LINE, "**Decisiones.** Q-X-3; Q-X-8"):
            with self.subTest(declaration=declaration):
                self.declare_for_ask_003(declaration)
                self.regenerate()
                self.omit_from_map("AT-ASK-003", "Q-X-3")
                self.assertIn("AT-ASK-003: contract declares Q-X-8 but contract-map.tsv decisions column omits it", self.run_script("tools/package/check.py").stdout)

    def test_alias_is_normalized_and_its_omission_detected(self):
        self.declare_for_ask_003("**Decisiones.** Q-CFG-1")
        self.regenerate()
        self.assertEqual("Q-X-1", self.decisions_of("AT-ASK-003"))
        self.omit_from_map("AT-ASK-003", "")
        self.assertIn("AT-ASK-003: contract declares Q-X-1 but contract-map.tsv decisions column omits it", self.run_script("tools/package/check.py").stdout)

    def test_declarations_do_not_leak_to_neighbours_or_later_sections(self):
        self.write(TYPED, self.read(TYPED).replace("## AT-ASK-004:", "## Notas ajenas al contrato\n\n**Decisiones.** Q-X-5\n\n## AT-ASK-004:", 1))
        self.regenerate()
        self.assertEqual("Q-X-3,Q-X-8", self.decisions_of("AT-ASK-003"))
        self.assertEqual("Q-T-ASK-2", self.decisions_of("AT-ASK-004"))

    def test_unknown_declared_decision_is_reported(self):
        self.declare_for_ask_003("**Decisiones.** Q-X-3, Q-X-99")
        self.regenerate()
        self.assertIn("AT-ASK-003: contract declares unknown decision Q-X-99", self.run_script("tools/package/check.py").stdout)

    def test_group_decision_is_not_listed_as_pending(self):
        self.regenerate()
        self.assertEqual(0, self.run_script("tools/package/unit_plan.py").returncode)
        listing = self.read("docs/unit-decisions.md")
        self.assertNotIn("| Q-X-9 |", listing)
        self.assertIn("| Q-UTIL-3 |", listing)


if __name__ == "__main__":
    unittest.main()
