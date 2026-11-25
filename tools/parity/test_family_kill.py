"""Tests of how check_divergence turns the ladder into a verdict (`family_kill`), with the replay steps replaced by doubles: a family may prove
DETECTED, a failing step is never a detection, and nothing found hands over to the full replay (None), never to UNDETECTED."""
import collections, unittest
from unittest import mock

import check_divergence as cd
import family_replay as ladder

Confirmation = collections.namedtuple("Confirmation", "confirmed reason report_text")


class FamilyKill(unittest.TestCase):
    def patched(self, families=("fam-a", "fam-b"), control="/control", divergent=None, confirm=True, replay_error=None):
        divergent = divergent or {}

        def replay_family(binary, family):
            if replay_error is not None:
                raise replay_error
            return divergent.get(family, [])

        self.published, self.learned = [], []
        stack = [
            mock.patch.object(cd, "families_for", return_value=list(families)),
            mock.patch.object(cd, "control_binary_for_fast_kill", return_value=control),
            mock.patch.object(cd, "replay_family", side_effect=replay_family),
            mock.patch.object(cd.replay_lib, "diverges_only_in_mutant", return_value=True),
            mock.patch.object(cd.replay_lib, "confirm_applicable_divergence", return_value=Confirmation(confirm, "r", "REPORT")),
            mock.patch.object(cd, "publish_mutant_report", side_effect=lambda *args: self.published.append(args)),
            mock.patch.object(cd, "preserve_killers"),
            mock.patch.object(cd.family_replay, "learn_into", side_effect=lambda path, cases: self.learned.append((path, cases))),
        ]
        for patch in stack:
            patch.start()
            self.addCleanup(patch.stop)

    def test_a_family_with_a_confirmed_divergence_detects_publishes_the_evidence_and_learns_the_killers(self):
        self.patched(divergent={"fam-b": ["c1", "c2"]})
        verdict = cd.family_kill("/mutant", "m")
        self.assertEqual(verdict[0], cd.DETECTED)
        self.assertIn("family replay", verdict[3])
        self.assertEqual(self.published[0][0], "m")
        self.assertIn("REPORT", self.published[0][2])
        self.assertEqual(self.learned, [(cd.killers_file("m"), ["c1", "c2"])])

    def test_nothing_found_hands_over_to_the_full_replay_never_to_undetected(self):
        self.patched()
        self.assertIsNone(cd.family_kill("/mutant", "m"))
        self.assertEqual((self.published, self.learned), ([], []))

    def test_a_divergence_the_real_tool_does_not_confirm_is_not_a_detection(self):
        self.patched(divergent={"fam-a": ["c1"]}, confirm=False)
        self.assertIsNone(cd.family_kill("/mutant", "m"))

    def test_an_unknown_mapping_or_a_missing_control_goes_to_the_full_replay(self):
        self.patched(families=())
        self.assertIsNone(cd.family_kill("/mutant", "m"))
        self.patched(control=None, divergent={"fam-a": ["c1"]})
        self.assertIsNone(cd.family_kill("/mutant", "m"))

    def test_a_timeout_or_a_harness_error_is_reported_as_such_and_never_as_a_detection(self):
        self.patched(replay_error=ladder.StepFailure("timeout", "slow"))
        self.assertEqual(cd.family_kill("/mutant", "m")[0], cd.TIMEOUT)
        self.patched(replay_error=ladder.StepFailure("harness", "bad output"))
        self.assertEqual(cd.family_kill("/mutant", "m")[0], cd.HARNESS_ERROR)
        self.assertEqual(self.learned, [])


if __name__ == "__main__":
    unittest.main()
