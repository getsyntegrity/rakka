"""Tests of the BoundedMailbox oracle checker: it reuses the PR 1 comparison, so these pin the bounded-specific parts (manifest, selftest)."""
import os, sys, unittest
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_bounded_mailbox_oracle as chk


class BoundedOracleCheckerTest(unittest.TestCase):
    def test_golden_and_manifest_agree(self):
        self.assertEqual(chk.manifest_problems(chk.read_golden()), [])

    def test_golden_is_stable_against_itself(self):
        golden = chk.read_golden()
        self.assertTrue(golden)
        self.assertEqual(chk.compare(golden, [golden, golden]), [])

    def test_selftest_catches_every_mutant(self):
        # selftest returns 1 when every mutant was caught (the convention of check_divergence.py --selftest)
        self.assertEqual(chk.selftest(), 1)


if __name__ == "__main__":
    unittest.main()
