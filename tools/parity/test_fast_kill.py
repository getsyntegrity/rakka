"""Tests of the fast-kill criterion: a mutant is DETECTED early only when the unmutated build agrees with the recorded Pekko output
on that case and the mutant does not (same status code and stdout that `rust-anything app replay` compares)."""
import os, stat, tempfile, unittest

import replay_lib


def make_binary(directory, name, body):
    path = os.path.join(directory, name)
    with open(path, "w") as fh:
        fh.write("#!/bin/sh\n" + body + "\n")
    os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR)
    return path


def record_case(cases_dir, case, stdout, code=0, args='["scenario"]', toml_text=None):
    base = os.path.join(cases_dir, case)
    with open(base + ".toml", "w") as fh:
        fh.write(toml_text or f'bin.name = "classic-parity"\nargs = {args}\nstatus.code = {code}\nfs.sandbox = true\n')
    with open(base + ".stdin", "w") as fh:
        fh.write("input")
    with open(base + ".stdout", "w") as fh:
        fh.write(stdout)


class FastKillCriterion(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = self.tmp.name
        self.cases = os.path.join(self.dir, "cases")
        os.makedirs(self.cases)
        self.oracle_like = make_binary(self.dir, "control", 'printf "pekko"')
        record_case(self.cases, "c1", "pekko")

    def tearDown(self):
        self.tmp.cleanup()

    def check(self, mutant, control=None, case="c1", timeout=5):
        return replay_lib.diverges_only_in_mutant(mutant, control or self.oracle_like, case, self.cases, timeout)

    def test_the_mutant_output_differs_while_the_unmutated_build_matches_pekko(self):
        self.assertTrue(self.check(make_binary(self.dir, "m", 'printf "other"')))

    def test_the_mutant_exit_status_differs(self):
        self.assertTrue(self.check(make_binary(self.dir, "m", 'printf "pekko"; exit 101')))

    def test_a_mutant_that_still_matches_pekko_is_not_detected(self):
        self.assertFalse(self.check(make_binary(self.dir, "m", 'printf "pekko"')))

    def test_when_the_unmutated_build_does_not_match_pekko_the_difference_is_not_attributable_to_the_mutant(self):
        broken_control = make_binary(self.dir, "control2", 'printf "not pekko"')
        self.assertFalse(self.check(make_binary(self.dir, "m", 'printf "other"'), control=broken_control))

    def test_a_mutant_that_hangs_is_not_detected_by_the_fast_path(self):
        self.assertFalse(self.check(make_binary(self.dir, "m", "sleep 30"), timeout=1))

    def test_a_malformed_recording_or_a_missing_binary_falls_back_instead_of_deciding(self):
        record_case(self.cases, "bad", "x", toml_text="this is = not [valid toml")
        self.assertFalse(self.check(make_binary(self.dir, "m", 'printf "other"'), case="bad"))
        self.assertFalse(self.check(os.path.join(self.dir, "missing-binary")))
        self.assertFalse(self.check(make_binary(self.dir, "m2", 'printf "other"'), case="no-such-case"))


if __name__ == "__main__":
    unittest.main()
