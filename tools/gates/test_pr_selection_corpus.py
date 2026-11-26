"""Q-X-22 R1: new independent cases do not select every mutant, only when the existing scenarios, their expected values and the way they run
are proven untouched."""
import unittest

import pr_selection

HARNESS = "tools/parity/check_divergence.py"
EXAMPLE = "crates/rakka-actor/examples/classic-parity/mailbox_cases.rs"
CASES, ORACLE = "cases/mailbox.jsonl", "oracle/src/main/java/pilot/MailboxOracle.java"
SOURCE = "crates/rakka-actor/src/dispatch/message_queue.rs"
MUTANTS = {"queue-is-lifo": ("src/dispatch/message_queue.rs", "o", "n", "why"), "cell-throughput": ("src/actor/cell.rs", "o", "n", "why")}
TABLE = 'MUTANTS = {"queue-is-lifo": ("src/dispatch/message_queue.rs", "o", "n", "why")}\ndef judge():\n    return 1\n'
NEW_ENTRY = 'TABLE_ENTRY'
OLD_RUST = '''use rakka_actor::dispatch::message_queue::MessageQueue;

fn fifo() -> String {
    format!("a{}", drain())
}

pub fn rust_lines() -> Vec<(String, String)> {
    vec![
        (
            "MB-01 first in first out",
            fifo(),
        ),
    ]
}
'''
ADDED_RUST = OLD_RUST.replace("MessageQueue;\n", "MessageQueue;\nuse rakka_actor::dispatch::single::SingleQueue;\n").replace(
    "pub fn rust_lines", '''/// MB-02
struct Probe;
impl Probe {
    fn run(&self) -> String { format!("{}", SingleQueue.len()) }
}

fn single_consumer() -> String {
    Probe.run()
}

pub fn rust_lines''').replace('    ]\n', '''        (
            "MB-02 single consumer",
            single_consumer(),
        ),
    ]
''')
BASE = {CASES: '{"id": "a"}\n', EXAMPLE: OLD_RUST, ORACLE: "class MailboxOracle {\n}\n", HARNESS: TABLE, "cases/not-applicable.tsv": "case\treason\n",
        "rust/app-cases/mb-0001.stdout": "a\n", "cases/oracle-only/mailbox.golden": "g\n", "oracle/pom.xml": "<p/>\n"}
ADDITION = {CASES: '{"id": "a"}\n{"id": "b"}\n', "rust/app-cases/mb-0002.stdout": "b\n", EXAMPLE: ADDED_RUST,
            ORACLE: 'class MailboxOracle {\n  void more() {}\n}\n'}


def narrow(changes, mutants=MUTANTS):
    head = {**BASE, **changes}
    return pr_selection.narrow(sorted(changes), mutants, {}, {}, BASE.get, lambda path: head.get(path), lambda prefix: [])


class AdditiveCorpus(unittest.TestCase):
    def test_new_independent_cases_select_no_mutant_and_the_report_lists_what_did_not_run(self):
        report = narrow(ADDITION)
        self.assertEqual((report["selected"], report["not_selected_by_rule"]), ([], ["queue-is-lifo", "cell-throughput"]))
        self.assertEqual(report["rules"], ["Q-X-22 R1: " + path for path in sorted(ADDITION)])

    def test_a_golden_line_with_a_new_case_id_leaves_the_existing_expected_values_untouched(self):
        report = narrow({**ADDITION, "cases/oracle-only/mailbox.golden": "g\nh -> new\n"})
        self.assertEqual(report["selected"], [])

    def test_the_new_mutants_and_the_mutants_of_changed_sources_still_run(self):
        harness = TABLE.replace('"why")}', '"why"), "queue-drops": ("src/dispatch/single.rs", "o", "n", "why")}')
        inventory = {**MUTANTS, "queue-drops": ("src/dispatch/single.rs", "o", "n", "why")}
        report = narrow({**ADDITION, HARNESS: harness, SOURCE: "fn changed() {}\n"}, inventory)
        self.assertEqual(report["selected"], ["queue-is-lifo", "queue-drops"])

    def assert_conservative(self, change, why=""):
        report = narrow({**ADDITION, **change})
        self.assertEqual((report["selected"], report["rules"]), (["queue-is-lifo", "cell-throughput"], []), why or sorted(change))

    def test_a_touched_expected_value_input_or_domain_list_keeps_the_conservative_selection(self):
        for change in ({"rust/app-cases/mb-0001.stdout": "z\n"}, {"rust/app-cases/mb-0001.stdout": None}, {CASES: '{"id": "z"}\n'},
                       {"cases/not-applicable.tsv": "case\treason\nmb-0001\tout\n"}, {"cases/oracle-only/mailbox.golden": "g\ng again\n"},
                       {"cases/oracle-only/mailbox.golden": "h\n"},
                       {ORACLE: "class MailboxOracle {\n  int x;\n"}, {"oracle/pom.xml": "<p/>\n<q/>\n"}):
            self.assert_conservative(change)

    def test_a_change_to_how_existing_scenarios_run_keeps_the_conservative_selection(self):
        entry = '            "MB-01 first in first out",\n            fifo(),\n'
        for why, harness in (("a literal of an existing function", ADDED_RUST.replace('"a{}"', '"b{}"')),
                             ("an existing entry removed", ADDED_RUST.replace("        (\n" + entry + "        ),\n", "")),
                             ("a new entry running an old function", ADDED_RUST.replace("single_consumer(),", "fifo(),")),
                             ("a repeated label", ADDED_RUST.replace('"MB-02 single consumer"', '"MB-01 first in first out"')),
                             ("a new function taking a name the old code uses", ADDED_RUST + "fn drain() -> String { String::new() }\n"),
                             ("an impl for an existing type", ADDED_RUST + "impl Drop for Envelope {}\n"),
                             ("a glob import", "use evil::*;\n" + ADDED_RUST),
                             ("a new static", ADDED_RUST + "static HITS: u8 = 0;\n"),
                             ("a custom macro", ADDED_RUST + "fn other() { register!(A); }\n")):
            self.assert_conservative({EXAMPLE: harness}, why)

    def test_a_change_to_the_logic_that_judges_or_a_removed_mutant_keeps_the_conservative_selection(self):
        self.assert_conservative({HARNESS: TABLE.replace("return 1", "return 0")})
        self.assert_conservative({HARNESS: "MUTANTS = {}\ndef judge():\n    return 1\n"})


if __name__ == "__main__":
    unittest.main()
