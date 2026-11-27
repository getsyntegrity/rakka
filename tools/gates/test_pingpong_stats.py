"""tools/perf/pingpong_stats.py must not report a time for a run in which libtest executed no test (a mistyped or renamed
`--exact` name makes it exit 0 at once)."""
import os, sys, tempfile, unittest

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "perf"))
import pingpong_stats


class RunOnce(unittest.TestCase):
    def run_fake_test_binary(self, counts):
        with tempfile.TemporaryDirectory() as folder:
            binary = os.path.join(folder, "test-binary")
            with open(binary, "w", encoding="utf-8") as script:
                script.write(f"#!/bin/sh\necho 'test result: ok. {counts}; 0 ignored; finished in 0.25s'\n")
            os.chmod(binary, 0o755)
            return pingpong_stats.run_once(binary, "some_exact_name", False)

    def test_a_run_that_executed_no_test_is_an_error(self):
        with self.assertRaisesRegex(SystemExit, "ran no test"):
            self.run_fake_test_binary("0 passed; 0 failed")

    def test_a_run_that_executed_a_test_gives_its_time(self):
        self.assertEqual(self.run_fake_test_binary("1 passed; 0 failed"), (0.25, True, False))


if __name__ == "__main__":
    unittest.main()
