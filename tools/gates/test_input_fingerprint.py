"""A gate may record evidence or pass only for the state of its inputs that it really ran on. `(HEAD tree ids, dirty)` is not enough: an input that was
already modified before the run and changes again during it leaves both values equal. These tests use a real temporary git repository and commands
that modify, create, delete and rename input files while the gate runs."""
import json, os, subprocess, sys, tempfile, unittest

import gates

REPORT = "rust/app-cases/replay-report.md"


class StateThatReallyRan(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = self.tmp.name
        self.saved = (gates.ROOT, gates.GATES_DIR)
        gates.ROOT = self.repo
        gates.GATES_DIR = os.path.join(self.repo, ".gates")
        self.git("init", "-q")
        for name, text in (("crates/x.rs", "original x\n"), ("crates/y.rs", "original y\n"), (REPORT, "# App replay\n")):
            self.write(name, text)
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "base")
        self.write(".gitignore", ".gates/\n")                      # the runner's own output is not an input
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "ignore .gates")

    def tearDown(self):
        gates.ROOT, gates.GATES_DIR = self.saved
        self.tmp.cleanup()

    def git(self, *arguments):
        return subprocess.run(["git", "-C", self.repo, "-c", "user.email=t@example.org", "-c", "user.name=t", *arguments],
                              capture_output=True, text=True, check=True).stdout.strip()

    def write(self, name, text):
        path = os.path.join(self.repo, name)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w") as handle:
            handle.write(text)

    def evidence_files(self):
        directory = os.path.join(gates.GATES_DIR, "evidence")
        return os.listdir(directory) if os.path.isdir(directory) else []

    def run_command(self, python_code, regenerates=(), inputs=("crates", "rust/app-cases"), reuse=True):
        gate = gates.Gate("sample", [sys.executable, "-c", python_code], inputs, regenerates=regenerates)
        return gates.run_gate(gate, "rustc 1", reuse, "head")

    def make_dirty(self):
        self.write("crates/x.rs", "local edit A\n")

    # --- an input that was ALREADY modified changes again during the run ---
    def test_a_dirty_file_that_changes_again_during_the_run_is_inputs_changed_and_nothing_is_recorded(self):
        self.make_dirty()
        line, code, label = self.run_command("open('crates/x.rs', 'w').write('local edit B\\n')")
        self.assertEqual(label, "INPUTS_CHANGED", line)
        self.assertNotEqual(code, 0)
        self.assertEqual(self.evidence_files(), [])

    def test_an_input_that_appears_during_the_run_on_an_already_dirty_tree_is_inputs_changed(self):
        self.make_dirty()
        line, code, label = self.run_command("open('crates/appeared.rs', 'w').write('new\\n')")
        self.assertEqual((label, self.evidence_files()), ("INPUTS_CHANGED", []), line)
        self.assertNotEqual(code, 0)

    def test_an_input_that_disappears_during_the_run_on_an_already_dirty_tree_is_inputs_changed(self):
        self.make_dirty()
        line, code, label = self.run_command("import os; os.remove('crates/y.rs')")
        self.assertEqual((label, self.evidence_files()), ("INPUTS_CHANGED", []), line)

    def test_an_input_renamed_during_the_run_on_an_already_dirty_tree_is_inputs_changed(self):
        self.make_dirty()
        line, code, label = self.run_command("import os; os.rename('crates/y.rs', 'crates/z.rs')")
        self.assertEqual((label, self.evidence_files()), ("INPUTS_CHANGED", []), line)

    def test_an_untracked_input_edited_during_the_run_is_inputs_changed(self):
        self.write("crates/untracked.rs", "first\n")
        line, code, label = self.run_command("open('crates/untracked.rs', 'w').write('second\\n')")
        self.assertEqual((label, self.evidence_files()), ("INPUTS_CHANGED", []), line)

    # --- the same changes on a clean tree are still caught ---
    def test_changes_during_the_run_on_a_clean_tree_are_inputs_changed_too(self):
        for code in ("open('crates/x.rs', 'w').write('changed\\n')", "open('crates/new.rs', 'w').write('n\\n')", "import os; os.remove('crates/y.rs')",
                     "import os; os.rename('crates/y.rs', 'crates/z.rs')"):
            self.git("checkout", "-q", "--", ".")
            self.git("clean", "-fdq", "--", "crates")
            line, _code, label = self.run_command(code)
            self.assertEqual((label, self.evidence_files()), ("INPUTS_CHANGED", []), (code, line))

    def test_an_input_directory_that_did_not_exist_and_appears_during_the_run_is_inputs_changed(self):
        line, code, label = self.run_command("import os; os.makedirs('oracle'); open('oracle/pom.xml', 'w').write('x')", inputs=("crates", "oracle"))
        self.assertEqual((label, self.evidence_files()), ("INPUTS_CHANGED", []), line)

    # --- what must still pass ---
    def test_stable_local_changes_may_pass_but_never_record_evidence(self):
        self.make_dirty()
        line, code, label = self.run_command("pass")
        self.assertEqual((code, label), (0, "PASS"), line)
        self.assertEqual(self.evidence_files(), [])                  # evidence stays reserved to clean inputs

    def test_a_clean_stable_run_records_evidence_and_a_second_run_reuses_it(self):
        command = "open('/dev/null', 'a'); import os; open(os.path.join(os.environ.get('RAN_MARKER', '/tmp'), 'ran-sample'), 'a').write('x')"
        os.environ["RAN_MARKER"] = self.repo
        try:
            line, code, label = self.run_command(command)
            self.assertEqual((code, label), (0, "PASS"), line)
            self.assertEqual(len(self.evidence_files()), 1)
            line, code, label = self.run_command(command)                 # the same command on the same clean inputs
            self.assertEqual((code, label), (0, "PASS"))
            self.assertIn("reused evidence", line)
            with open(os.path.join(self.repo, "ran-sample")) as marker:
                self.assertEqual(marker.read(), "x")                       # it ran once: the second run reused the evidence
        finally:
            del os.environ["RAN_MARKER"]

    def test_the_report_the_gate_itself_regenerates_is_not_a_change_of_its_inputs(self):
        line, code, label = self.run_command(f"open({REPORT!r}, 'a').write('Passed: 2821/2822\\n')", regenerates=(REPORT,))
        self.assertEqual((code, label), (0, "PASS"), line)
        self.assertEqual(len(self.evidence_files()), 1)

    def test_a_gate_that_does_not_declare_the_report_it_rewrites_is_inputs_changed(self):
        line, code, label = self.run_command(f"open({REPORT!r}, 'a').write('Passed: 2821/2822\\n')")
        self.assertEqual((label, self.evidence_files()), ("INPUTS_CHANGED", []), line)

    def test_touching_a_file_without_changing_its_content_is_not_a_change(self):
        self.make_dirty()
        line, code, label = self.run_command("open('crates/x.rs', 'w').write('local edit A\\n')")      # same content again
        self.assertEqual((code, label), (0, "PASS"), line)

    def test_a_failing_gate_stays_a_failure_even_if_its_inputs_changed(self):
        self.make_dirty()
        line, code, label = self.run_command("import sys; open('crates/x.rs', 'w').write('B\\n'); sys.exit(1)")
        self.assertEqual((code, label), (1, "FAIL"), line)

    # --- the fingerprint itself ---
    def test_the_fingerprint_covers_content_names_and_declared_paths_but_not_regenerated_outputs(self):
        fingerprint = lambda **kw: gates.input_state(("crates", "rust/app-cases"), kw.get("regenerates", ())).fingerprint
        self.make_dirty()
        before = fingerprint(regenerates=(REPORT,))
        self.write("crates/x.rs", "local edit B\n")
        self.assertNotEqual(before, fingerprint(regenerates=(REPORT,)))
        self.write("crates/x.rs", "local edit A\n")
        self.assertEqual(before, fingerprint(regenerates=(REPORT,)))
        self.write(REPORT, "# App replay\nrewritten\n")
        self.assertEqual(before, fingerprint(regenerates=(REPORT,)))
        self.assertNotEqual(before, fingerprint(regenerates=()))

    def test_the_state_keeps_exposing_the_tree_ids_and_the_dirty_flag(self):
        state = gates.input_state(("crates",), ())
        trees, dirty = state[0], state[1]
        self.assertEqual(set(trees), {"crates"})
        self.assertFalse(dirty)
        self.make_dirty()
        self.assertTrue(gates.input_state(("crates",), ())[1])


if __name__ == "__main__":
    unittest.main()
