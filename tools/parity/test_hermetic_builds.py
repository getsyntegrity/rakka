"""Mutation builds are hermetic: the authoritative Cargo.lock, locked and offline, and a failure of the infrastructure is never reported as a mutation that does not compile.
Unit layer: the cargo process is replaced by a recorder (the boundary substituted is `check_divergence.sh`); test_hermetic_builds_integration.py runs the real cargo."""
import json
import os
import subprocess
import tempfile
import unittest

import check_divergence as cd

COMPILE_ERROR = json.dumps({"reason": "compiler-message", "message": {"level": "error", "rendered": "error[E0425]: cannot find value `x`\n"}})
WARNING_ONLY = json.dumps({"reason": "compiler-message", "message": {"level": "warning", "rendered": "warning: unused\n"}})
RESOLUTION_FAILURE = "error: no matching package named `bytes` found\nnote: offline mode (via `--offline`) can sometimes cause surprising resolution failures\n"


def finished(stdout="", stderr="", returncode=101):
    return subprocess.CompletedProcess(["cargo"], returncode, stdout, stderr)


class Classification(unittest.TestCase):
    def test_a_compiler_error_diagnostic_is_a_mutation_that_does_not_compile(self):
        kind, detail = cd.classify_cargo_failure(finished(stdout=WARNING_ONLY + "\n" + COMPILE_ERROR + "\n"))
        self.assertEqual(kind, cd.BUILD_ERROR)
        self.assertIn("cannot find value", detail)

    def test_a_failure_without_a_compiler_error_is_the_infrastructure_whatever_the_wording(self):
        for stdout, stderr in (("", RESOLUTION_FAILURE), (WARNING_ONLY + "\n", "error: could not compile `x`, but nothing the compiler said"), ("not json at all\n", "")):
            kind, _detail = cd.classify_cargo_failure(finished(stdout=stdout, stderr=stderr))
            self.assertEqual(kind, cd.HARNESS_ERROR, (stdout, stderr))

    def test_the_words_could_not_compile_in_stderr_alone_do_not_make_a_mutation_failure(self):
        self.assertEqual(cd.classify_cargo_failure(finished(stderr="error: could not compile `rakka-actor`"))[0], cd.HARNESS_ERROR)


class Recorder:
    def __init__(self, outcome):
        self.outcome, self.commands = outcome, []

    def __call__(self, command, timeout=None, **kwargs):
        self.commands.append(command)
        return self.outcome(command)


class ScratchWorkspace(unittest.TestCase):
    def setUp(self):
        self.saved = (cd.ROOT, cd.CRATE, cd.sh)
        self.directory = tempfile.TemporaryDirectory()
        self.root = self.directory.name
        os.makedirs(os.path.join(self.root, "crates", "rakka-actor", "src"))
        with open(os.path.join(self.root, "Cargo.toml"), "w") as manifest:
            manifest.write('[package]\nname = "rakka"\n[workspace]\nmembers = [".", "crates/rakka-actor", "crates/rakka-actor-typed"]\n')
        with open(os.path.join(self.root, "crates", "rakka-actor", "Cargo.toml"), "w") as manifest:
            manifest.write('[package]\nname = "rakka-actor"\n')
        with open(os.path.join(self.root, "crates", "rakka-actor", "src", "lib.rs"), "w") as source:
            source.write("pub fn one() -> u8 { 1 }\n")
        self.lock_text = "# the authoritative lock\nversion = 4\n"
        with open(os.path.join(self.root, "Cargo.lock"), "w") as lock:
            lock.write(self.lock_text)
        cd.ROOT, cd.CRATE = self.root, os.path.join(self.root, "crates", "rakka-actor")
        self.scratch = os.path.join(self.root, "scratch")
        os.makedirs(self.scratch)

    def tearDown(self):
        cd.ROOT, cd.CRATE, cd.sh = self.saved
        self.directory.cleanup()

    def mutant(self, outcome):
        cd.sh = Recorder(outcome)
        scratch = tempfile.mkdtemp(dir=self.scratch)      # every mutant has a scratch directory of its own
        return cd.run_mutant(scratch, "a-mutant", "src/lib.rs", "{ 1 }", "{ 2 }", "why"), cd.sh

    def test_the_scratch_workspace_carries_the_authoritative_lock_byte_for_byte(self):
        self.mutant(lambda command: finished(stdout=COMPILE_ERROR + "\n"))
        (scratch,) = [os.path.join(self.scratch, entry) for entry in os.listdir(self.scratch)]
        with open(os.path.join(scratch, "a-mutant", "Cargo.lock")) as scratch_lock:
            self.assertEqual(scratch_lock.read(), self.lock_text)

    def test_without_the_authoritative_lock_no_mutant_is_built_and_it_is_an_infrastructure_error(self):
        os.remove(os.path.join(self.root, "Cargo.lock"))
        (outcome, _passed, _total, line), recorder = self.mutant(lambda command: finished())
        self.assertEqual(outcome, cd.HARNESS_ERROR)
        self.assertIn("Cargo.lock", line)
        self.assertEqual(recorder.commands, [])

    def test_the_build_is_locked_offline_and_reads_structured_diagnostics(self):
        _result, recorder = self.mutant(lambda command: finished(stdout=COMPILE_ERROR + "\n"))
        (command,) = recorder.commands
        for flag in ("--locked", "--offline", "--message-format=json"):
            self.assertIn(flag, command)

    def test_a_mutation_that_does_not_compile_is_a_build_error_and_a_resolution_failure_is_not(self):
        (compile_outcome, *_rest), _ = self.mutant(lambda command: finished(stdout=COMPILE_ERROR + "\n"))
        (infrastructure_outcome, _p, _t, line), _ = self.mutant(lambda command: finished(stderr=RESOLUTION_FAILURE))
        self.assertEqual(compile_outcome, cd.BUILD_ERROR)
        self.assertEqual(infrastructure_outcome, cd.HARNESS_ERROR)
        self.assertIn("no matching package", line)

    def test_the_unit_mutants_control_and_mutant_are_both_locked_offline(self):
        cd.UNIT_MUTANTS["a-unit-mutant"] = ("actor::module", "the_expected_test")
        try:
            cd.sh = Recorder(lambda command: finished(returncode=0, stdout="test result: ok.\n"))
            cd.run_unit_mutant(self.scratch, "a-unit-mutant", os.path.join(self.scratch, "a-unit-mutant", "rakka-actor"))
        finally:
            del cd.UNIT_MUTANTS["a-unit-mutant"]
        self.assertEqual(len(cd.sh.commands), 2)
        for command in cd.sh.commands:
            self.assertIn("--locked", command)
            self.assertIn("--offline", command)

    def test_a_unit_mutant_that_does_not_compile_is_a_build_error_even_when_libtest_text_shares_the_stream(self):
        cd.UNIT_MUTANTS["a-unit-mutant"] = ("actor::module", "the_expected_test")
        try:
            def outcome(command):
                if "target-control-a-unit-mutant" in " ".join(command):
                    return finished(returncode=0, stdout="test result: ok.\n")
                return finished(stdout="running 0 tests\n" + COMPILE_ERROR + "\n")
            cd.sh = Recorder(outcome)
            verdict, _passed, _total, detail = cd.run_unit_mutant(self.scratch, "a-unit-mutant", os.path.join(self.scratch, "a-unit-mutant", "rakka-actor"))
        finally:
            del cd.UNIT_MUTANTS["a-unit-mutant"]
        self.assertEqual(verdict, cd.BUILD_ERROR)
        self.assertIn("cannot find value", detail)

    def test_a_unit_mutant_that_cannot_resolve_its_dependencies_is_an_infrastructure_error_not_a_build_error(self):
        cd.UNIT_MUTANTS["a-unit-mutant"] = ("actor::module", "the_expected_test")
        try:
            cd.sh = Recorder(lambda command: finished(returncode=0, stdout="test result: ok.\n") if "target-control-a-unit-mutant" in " ".join(command) else finished(stderr=RESOLUTION_FAILURE))
            outcome, *_rest = cd.run_unit_mutant(self.scratch, "a-unit-mutant", os.path.join(self.scratch, "a-unit-mutant", "rakka-actor"))
        finally:
            del cd.UNIT_MUTANTS["a-unit-mutant"]
        self.assertEqual(outcome, cd.HARNESS_ERROR)


class Provisioning(unittest.TestCase):
    def test_dependencies_are_provisioned_explicitly_from_the_locked_manifest(self):
        saved = cd.sh
        cd.sh = Recorder(lambda command: finished(returncode=0))
        try:
            ok, _detail = cd.provision_dependencies()
        finally:
            recorder, cd.sh = cd.sh, saved
        self.assertTrue(ok)
        (command,) = recorder.commands
        self.assertEqual(command[:3], ["cargo", "fetch", "--locked"])

    def test_a_provisioning_failure_is_reported_with_its_reason_and_never_raises(self):
        saved = cd.sh
        cd.sh = Recorder(lambda command: finished(stderr=RESOLUTION_FAILURE))
        try:
            ok, detail = cd.provision_dependencies()
        finally:
            cd.sh = saved
        self.assertFalse(ok)
        self.assertIn("no matching package", detail)


if __name__ == "__main__":
    unittest.main()
