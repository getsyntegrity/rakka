"""Integration of the hermetic mutation build with the real cargo and the real crate (about 10 s): offline success with the network made unreachable, and the
classification of a missing dependency (infrastructure) against a mutation that does not compile. It needs the dependencies in the Cargo cache (`cargo fetch --locked`)."""
import os
import tempfile
import unittest

import check_divergence as cd

NETWORK_UNREACHABLE = {"HTTP_PROXY": "http://127.0.0.1:9", "HTTPS_PROXY": "http://127.0.0.1:9", "ALL_PROXY": "http://127.0.0.1:9", "CARGO_NET_RETRY": "0"}


class RealCargo(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.work, problem = cd.prepare_scratch_workspace(os.path.join(self.directory.name, "mutant"))
        self.assertIsNone(problem)
        self.target = os.path.join(self.directory.name, "target")

    def build(self, environment):
        command = cd.hermetic(["cargo", "build", "--release", "--manifest-path", os.path.join(self.work, "Cargo.toml"), "--target-dir", self.target, "--example", "classic-parity"])
        return cd.sh(command, timeout=cd.BUILD_TIMEOUT, env={**os.environ, **environment})

    def test_the_mutant_builds_offline_with_the_network_unreachable_and_leaves_the_lock_untouched(self):
        with open(os.path.join(self.work, "..", "Cargo.lock")) as lock_before:
            before = lock_before.read()
        built = self.build(NETWORK_UNREACHABLE)
        self.assertEqual(built.returncode, 0, built.stderr[-600:])
        self.assertNotIn("Updating", built.stderr)
        self.assertTrue(os.path.exists(os.path.join(self.target, cd.PARITY)))
        with open(os.path.join(self.work, "..", "Cargo.lock")) as lock_after:
            self.assertEqual(lock_after.read(), before)

    def test_a_dependency_missing_from_the_cargo_cache_is_an_infrastructure_error(self):
        empty_cargo_home = os.path.join(self.directory.name, "empty-cargo-home")
        failed = self.build({**NETWORK_UNREACHABLE, "CARGO_HOME": empty_cargo_home})
        self.assertNotEqual(failed.returncode, 0)
        self.assertEqual(cd.classify_cargo_failure(failed)[0], cd.HARNESS_ERROR)

    def test_a_mutation_that_does_not_compile_is_a_build_error_with_the_compiler_message(self):
        with open(os.path.join(self.work, "src", "lib.rs"), "a") as source:
            source.write("\nfn broken() -> u8 { missing_value }\n")
        failed = self.build(NETWORK_UNREACHABLE)
        kind, detail = cd.classify_cargo_failure(failed)
        self.assertEqual(kind, cd.BUILD_ERROR)
        self.assertIn("missing_value", detail)


if __name__ == "__main__":
    unittest.main()
