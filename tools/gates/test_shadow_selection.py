"""The shadow selector: it explains what a finer mutant selection WOULD pick, next to the authoritative one, and never influences a verdict. Every doubt widens the selection (fail closed)."""
import unittest

import shadow_selection as shadow

OLD_TABLES = '''
MUTANTS = {"kept": ("src/a.rs", "old", "new", "why"), "edited": ("src/b.rs", "old", "new", "why"), "gone": ("src/c.rs", "old", "new", "why")}
TOOL_MUTANTS = {"tool-kept": ("t.py", "old", "new", "why")}
UNIT_MUTANTS = {"kept": ("m", "t")}
def run():
    return 1
'''
ADDED = OLD_TABLES.replace('"edited": ("src/b.rs", "old", "new", "why"),', '"edited": ("src/b.rs", "old", "new", "why"), "fresh": ("src/d.rs", "old", "new", "why"),')
EDITED = OLD_TABLES.replace('"edited": ("src/b.rs", "old", "new", "why")', '"edited": ("src/b.rs", "old", "other", "why")')
REMOVED = OLD_TABLES.replace(', "gone": ("src/c.rs", "old", "new", "why")', "")
LOGIC = OLD_TABLES.replace("return 1", "return 2")
NOT_LITERAL = OLD_TABLES.replace('TOOL_MUTANTS = {"tool-kept": ("t.py", "old", "new", "why")}', 'TOOL_MUTANTS = dict(extra())')

INVENTORY = {"kept": "crates/rakka-actor/src/a.rs", "edited": "crates/rakka-actor/src/b.rs", "gone": "crates/rakka-actor/src/c.rs", "fresh": "crates/rakka-actor/src/d.rs", "tool-kept": "tools/parity/t.py"}
HARNESS = "tools/parity/check_divergence.py"


def pick(changes, old=OLD_TABLES, new=OLD_TABLES):
    return shadow.shadow_selection(changes, old, new, INVENTORY)


class MutantTableDelta(unittest.TestCase):
    def test_unchanged_definitions_are_identical_and_only_a_new_or_edited_one_is_reported(self):
        for new, expected in ((ADDED, ["fresh"]), (EDITED, ["edited"]), (OLD_TABLES, [])):
            delta = shadow.table_delta(OLD_TABLES, new)
            self.assertTrue(delta.table_only, new)
            self.assertEqual(delta.new_or_changed, expected)
            self.assertEqual(delta.unchanged, len(["kept", "edited", "gone", "tool-kept"]) - len([n for n in expected if n != "fresh"]))

    def test_a_removed_definition_logic_outside_the_tables_or_a_table_that_is_not_a_literal_is_never_table_only(self):
        self.assertTrue(shadow.table_delta(OLD_TABLES, REMOVED).removed)
        self.assertFalse(shadow.table_delta(OLD_TABLES, LOGIC).table_only)
        self.assertFalse(shadow.table_delta(OLD_TABLES, NOT_LITERAL).table_only)
        self.assertFalse(shadow.table_delta("def broken(:", OLD_TABLES).table_only)


class ShadowSelection(unittest.TestCase):
    def test_documentation_and_the_report_every_replay_rewrites_select_nothing(self):
        result = pick([("M", "docs/decisions.md"), ("M", "README.md"), ("M", "rust/app-cases/replay-report.md"), ("A", "rust/divergence-reports/replay-report-x.md")])
        self.assertEqual((result.everything, result.selected), (False, []))

    def test_a_table_only_edit_selects_exactly_the_new_and_the_edited_mutants(self):
        result = pick([("M", HARNESS)], OLD_TABLES, ADDED.replace('"old", "new", "why"), "fresh"', '"old", "new", "why"), "fresh"'))
        self.assertEqual((result.everything, result.selected), (False, ["fresh"]))
        self.assertEqual(pick([("M", HARNESS)], OLD_TABLES, EDITED).selected, ["edited"])

    def test_a_product_file_selects_the_mutants_of_that_file_and_nothing_else(self):
        self.assertEqual(pick([("M", "crates/rakka-actor/src/b.rs")]).selected, ["edited"])

    def test_a_removed_definition_or_logic_of_the_harness_selects_everything(self):
        self.assertTrue(pick([("M", HARNESS)], OLD_TABLES, REMOVED).everything)
        self.assertTrue(pick([("M", HARNESS)], OLD_TABLES, LOGIC).everything)

    def test_the_global_harness_the_build_and_the_shared_modules_of_the_example_select_everything(self):
        for path in ("tools/parity/replay_lib.py", "tools/parity/na_q_cfg_30.py", "Cargo.lock", "crates/rakka-actor/Cargo.toml", "crates/rakka-actor/examples/classic-parity/main.rs",
                     "crates/rakka-actor/examples/classic-parity/scenario.rs"):
            self.assertTrue(pick([("M", path)]).everything, path)

    def test_corpus_additions_and_the_oracle_alone_need_no_mutant_but_any_modification_or_deletion_selects_everything(self):
        self.assertEqual(pick([("A", "rust/app-cases/mb-0099.toml"), ("M+", "cases/oracle-ids/mailbox.txt"), ("M", "oracle/src/main/java/pilot/MailboxOracle.java")]).everything, False)
        self.assertTrue(pick([("M", "cases/oracle-ids/mailbox.txt")]).everything)          # modified without proof that nothing was deleted
        for status in ("M", "D"):
            self.assertTrue(pick([(status, "rust/app-cases/mb-0001.stdout")]).everything, status)

    def test_a_family_module_of_the_example_is_reported_as_family_specific_but_is_not_narrowed_yet(self):
        result = pick([("M", "crates/rakka-actor/examples/classic-parity/mailbox_cases.rs")])
        self.assertTrue(result.everything)
        self.assertEqual(result.family_modules, ["mailbox_cases.rs"])

    def test_the_tests_of_the_gate_runner_are_not_the_judge_but_its_code_is(self):
        self.assertEqual(pick([("M", "tools/gates/test_mutant_patterns.py")]).everything, False)
        self.assertTrue(pick([("M", "tools/gates/mutant_runner.py")]).everything)

    def test_an_integration_test_of_the_crate_fails_closed_with_its_own_reason_because_its_unit_mutants_are_not_mapped(self):
        result = pick([("M", "crates/rakka-actor/tests/classic_guardian_mailbox.rs")])
        self.assertTrue(result.everything)
        self.assertIn(("crates/rakka-actor/tests/classic_guardian_mailbox.rs", "product tests (the unit mutants they kill are not mapped: fail closed)"), result.reasons)

    def test_a_file_of_an_unknown_kind_fails_closed(self):
        self.assertTrue(pick([("M", "some/new/thing.bin")]).everything)

    def test_every_reason_is_recorded_so_that_the_comparison_with_the_authoritative_selection_can_be_read(self):
        result = pick([("M", HARNESS), ("M", "docs/x.md")], OLD_TABLES, EDITED)
        self.assertIn(("docs/x.md", "documentation"), result.reasons)
        self.assertTrue(any(path == HARNESS and kind == "mutant definitions" for path, kind in result.reasons))


if __name__ == "__main__":
    unittest.main()
