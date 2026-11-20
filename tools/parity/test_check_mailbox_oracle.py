"""Tests of the mailbox oracle checker: a golden that differs in order, content, loss of a case or an unstable run must be reported."""
import os, sys, unittest
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_mailbox_oracle as chk

GOLDEN = ["MBX-A -> log=[r:1, r:2]", "MBX-B -> deadLetters=[u2, u3]"]


class CompareTest(unittest.TestCase):
    def test_identical_runs_and_golden_pass(self):
        self.assertEqual(chk.compare(GOLDEN, [GOLDEN, GOLDEN]), [])

    def test_reordered_messages_are_reported(self):
        swapped = ["MBX-A -> log=[r:2, r:1]", GOLDEN[1]]
        self.assertTrue(any("MBX-A" in p for p in chk.compare(GOLDEN, [swapped])))

    def test_lost_message_is_reported(self):
        lost = [GOLDEN[0], "MBX-B -> deadLetters=[u2]"]
        self.assertTrue(any("MBX-B" in p for p in chk.compare(GOLDEN, [lost])))

    def test_missing_and_extra_cases_are_reported(self):
        problems = chk.compare(GOLDEN, [[GOLDEN[0], "MBX-C -> x"]])
        self.assertTrue(any("MBX-B" in p and "missing" in p for p in problems))
        self.assertTrue(any("MBX-C" in p and "not in the golden" in p for p in problems))

    def test_unstable_runs_are_reported_even_if_one_matches_the_golden(self):
        flaky = [GOLDEN[0], "MBX-B -> deadLetters=[u3, u2]"]
        self.assertTrue(any("unstable" in p for p in chk.compare(GOLDEN, [GOLDEN, flaky])))

    def test_error_line_is_never_accepted(self):
        err = [GOLDEN[0], "MBX-B -> ERR NullPointerException []"]
        self.assertTrue(any("ERR" in p for p in chk.compare(err, [err])))


if __name__ == "__main__":
    unittest.main()
