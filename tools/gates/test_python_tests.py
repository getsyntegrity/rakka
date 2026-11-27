"""The runner of the tools' Python tests: every test file is run or excluded with a reason, and a failure or an empty suite is never a pass."""
import contextlib, glob, io, os, subprocess, unittest
from unittest import mock

import gates
import python_tests

MUTATION_EXECUTORS = {"tools/parity/test_tool_mutant_isolation.py", "tools/parity/test_hermetic_builds_integration.py"}


class Selection(unittest.TestCase):
    def test_every_test_file_of_the_three_suites_runs_except_the_ones_that_execute_the_mutation_machinery(self):
        self.assertEqual(set(python_tests.EXCLUDED), MUTATION_EXECUTORS)      # exact: an exclusion cannot be added unnoticed
        self.assertTrue(all(python_tests.EXCLUDED.values()))
        run = {f"{suite}/{module}.py" for suite in python_tests.SUITES for module in python_tests.test_modules(suite)}
        present = {os.path.relpath(path, gates.ROOT) for path in glob.glob(os.path.join(gates.ROOT, "tools", "**", "test_*.py"), recursive=True)}
        self.assertEqual(run, present - MUTATION_EXECUTORS)
        self.assertIn("tools/parity/test_coverage_matches_recordings.py", run)


class Running(unittest.TestCase):
    def run_main(self, exit_codes, modules=("test_a",)):
        calls, codes = [], list(exit_codes)
        def fake(command, **options):
            calls.append((command, options))
            return subprocess.CompletedProcess(command, codes.pop(0))
        with mock.patch.object(python_tests.subprocess, "run", fake), mock.patch.object(python_tests, "test_modules", return_value=list(modules)), \
                mock.patch.dict(os.environ, {gates.RUN_ID_ENV: "run"}), contextlib.redirect_stdout(io.StringIO()):
            return python_tests.main(), calls

    def test_each_suite_runs_in_its_own_directory_without_the_gate_run_id(self):
        code, calls = self.run_main([0, 0, 0])
        self.assertEqual(code, 0)
        self.assertEqual([options["cwd"] for _command, options in calls], [os.path.join(gates.ROOT, suite) for suite in python_tests.SUITES])
        for command, options in calls:
            self.assertEqual(command[1:], ["-m", "unittest", "test_a"])
            self.assertNotIn(gates.RUN_ID_ENV, options["env"])       # the tests must not leave evidence as if they were this run's gates

    def test_a_failing_suite_fails_the_run_and_the_others_still_run(self):
        code, calls = self.run_main([1, 0, 0])
        self.assertEqual((code, len(calls)), (1, 3))

    def test_a_suite_with_no_test_file_is_a_failure_and_never_runs_a_discovery(self):
        code, calls = self.run_main([], modules=())
        self.assertEqual((code, calls), (1, []))


if __name__ == "__main__":
    unittest.main()
