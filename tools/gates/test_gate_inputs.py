"""Evidence is reusable only while nothing a gate reads has changed, and the Python tests of the tools are a gate of the PR tier."""
import os, subprocess, tempfile, unittest
from unittest import mock

import gates

RUST_TARGET_GATES = ("unit-tests", "clippy", "test", "coverage")
RECORDING = "rust/app-cases/a.stdout"
CASE_FILES = ("cases/not-applicable.tsv", "cases/oracle-only/mailbox.golden")


def pr_gate(name):
    return next(gate for gate in gates.tier_gates("pr", []) if gate.name == name)


def covered(path, inputs):
    return any(path == entry or path.startswith(entry + "/") for entry in inputs)


class DeclaredInputs(unittest.TestCase):
    def test_every_gate_declares_the_repository_paths_it_reads(self):
        rust_targets = ["src/main.rs", RECORDING, *CASE_FILES]
        reads = {"fmt": ["src/main.rs"], "deny": ["src/main.rs", "deny.toml"], "replay": ["tools/gates/replay.sh", "oracle/run.sh", "src/main.rs"],
                 "package-check": ["README.md", "AGENTS.md", "upstream/pekko"], **{name: rust_targets for name in RUST_TARGET_GATES}}
        reads["coverage"] = rust_targets + ["tools/parity/coverage.sh"]
        missing = [(name, path) for name, paths in reads.items() for path in paths if not covered(path, pr_gate(name).inputs)]
        self.assertEqual(missing, [])


class TemporaryRepository(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = self.tmp.name
        for attribute, value in (("ROOT", self.repo), ("GATES_DIR", os.path.join(self.repo, ".gates"))):
            patch = mock.patch.object(gates, attribute, value)
            patch.start()
            self.addCleanup(patch.stop)
        self.git("init", "-q")
        self.write(".gitignore", ".gates/\n")
        for path in ("Cargo.toml", "crates/a.rs", "src/main.rs", RECORDING, "rust/app-cases/replay-report.md", *CASE_FILES):
            self.write(path, "one\n")
        self.commit()

    def git(self, *arguments, cwd=None):
        return subprocess.run(["git", "-c", "user.email=t@example.org", "-c", "user.name=t", *arguments], cwd=cwd or self.repo,
                              capture_output=True, text=True, check=True).stdout.strip()

    def write(self, relative, text):
        os.makedirs(os.path.dirname(os.path.join(self.repo, relative)), exist_ok=True)
        with open(os.path.join(self.repo, relative), "w") as target:
            target.write(text)

    def commit(self):
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "state")

    def key(self, name):
        gate = pr_gate(name)
        state = gates.input_state(gate.inputs, gate.regenerates)
        return gates.evidence_key(gate.command, state.trees, "rustc", state.dirty)


class ChangedCasesAndRecordings(TemporaryRepository):
    def test_a_committed_change_to_a_recording_or_a_case_changes_the_key_of_every_gate_that_compiles_or_runs_it(self):
        for path in (RECORDING, *CASE_FILES):
            before = {name: self.key(name) for name in RUST_TARGET_GATES}
            self.write(path, f"changed {path}\n")
            self.commit()
            for name in RUST_TARGET_GATES:
                self.assertIsNotNone(before[name], name)
                self.assertNotEqual(before[name], self.key(name), (name, path))

    def test_an_uncommitted_change_to_a_recording_forbids_reuse(self):
        self.write(RECORDING, "edited\n")
        self.assertEqual([self.key(name) for name in RUST_TARGET_GATES], [None] * len(RUST_TARGET_GATES))

    def test_the_report_every_replay_rewrites_does_not_forbid_reuse(self):
        before = [self.key(name) for name in RUST_TARGET_GATES]
        self.write("rust/app-cases/replay-report.md", "rewritten\n")
        self.assertEqual(before, [self.key(name) for name in RUST_TARGET_GATES])


class PinnedSubmodule(TemporaryRepository):
    def add_submodule(self):
        directory = os.path.join(self.repo, "upstream", "pekko")
        self.write("upstream/pekko/file", "pekko\n")
        self.git("init", "-q", cwd=directory)
        self.git("add", "-A", cwd=directory)
        self.git("commit", "-q", "-m", "pekko", cwd=directory)
        self.commit()
        return directory

    def test_a_submodule_input_is_read_without_error_and_its_content_is_part_of_the_state(self):
        self.add_submodule()
        before = gates.input_state(("upstream/pekko",))
        self.assertFalse(before.dirty)
        self.write("upstream/pekko/file", "edited\n")
        after = gates.input_state(("upstream/pekko",))
        self.assertTrue(after.dirty)
        self.assertNotEqual(before.fingerprint, after.fingerprint)

    def test_a_submodule_that_is_not_checked_out_forbids_reuse(self):
        directory = self.add_submodule()
        import shutil
        shutil.rmtree(directory)
        os.makedirs(directory)
        self.assertTrue(gates.input_state(("upstream/pekko",)).dirty)


class PythonTestsGate(unittest.TestCase):
    def test_it_runs_in_the_pr_and_milestone_tiers_every_time_and_not_in_fast(self):
        for tier in ("pr", "milestone"):
            gate = next(g for g in gates.tier_gates(tier, []) if g.name == "python-tests")
            self.assertEqual(gate.command, [gates.PY, "tools/gates/python_tests.py"])
            self.assertFalse(gate.reusable)       # python and rust-anything versions are not part of the evidence key
            for path in ("tools/parity/replay_subset.py", "cases/coverage.tsv", RECORDING, "rust/divergence-reports/x.md", "docs/a.md"):
                self.assertTrue(covered(path, gate.inputs), path)
        self.assertNotIn("python-tests", [g.name for g in gates.tier_gates("fast", [])])


if __name__ == "__main__":
    unittest.main()
