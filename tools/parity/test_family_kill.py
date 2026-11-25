"""Tests of how check_divergence turns the ladder into a verdict (`family_kill`) and orders the three paths (`verdict_for_binary`), with the replay steps
replaced by doubles that PIN their arguments: a family may prove DETECTED only on a case that diverges in the mutant and not in the clean control, a failing
step is never a detection and hands over to the full replay, and nothing found hands over to the full replay (None), never to UNDETECTED."""
import collections, unittest
from unittest import mock

import check_divergence as cd
import family_replay as ladder

Confirmation = collections.namedtuple("Confirmation", "confirmed reason report_text")


class FamilyKill(unittest.TestCase):
    def patched(self, families=("fam-a", "fam-b"), control="/control", divergent=None, replay_error=None, only_in_mutant=None, confirmations=None):
        divergent, self.calls, self.published, self.learned = divergent or {}, [], [], []
        only_in_mutant = only_in_mutant if only_in_mutant is not None else (lambda case: True)
        confirmations = confirmations if confirmations is not None else {}

        def replay_family(binary, family):
            if replay_error is not None:
                raise replay_error
            return divergent.get(family, [])

        def diverges_only_in_mutant(mutant_binary, control_binary, case):   # true only for the right binaries in the right order
            return (mutant_binary, control_binary) == ("/mutant", "/control") and only_in_mutant(case)

        def confirm_applicable_divergence(binary, case):
            return confirmations.get(case, Confirmation(binary == "/mutant" and case in confirmations, "r", "REPORT-" + case))

        def record(event, result=None):
            return lambda *args: (self.calls.append(event), result)[1]

        stack = [
            mock.patch.object(cd, "families_for", return_value=list(families)),
            mock.patch.object(cd, "control_binary_for_fast_kill", return_value=control),
            mock.patch.object(cd, "replay_family", side_effect=replay_family),
            mock.patch.object(cd.replay_lib, "diverges_only_in_mutant", side_effect=diverges_only_in_mutant),
            mock.patch.object(cd.replay_lib, "confirm_applicable_divergence", side_effect=confirm_applicable_divergence),
            mock.patch.object(cd, "publish_mutant_report", side_effect=lambda *args: (self.calls.append("publish"), self.published.append(args))),
            mock.patch.object(cd, "preserve_killers", side_effect=record("preserve")),
            mock.patch.object(cd.family_replay, "learn_into", side_effect=lambda path, cases: (self.calls.append("learn"), self.learned.append((path, cases)))),
        ]
        for patch in stack:
            patch.start()
            self.addCleanup(patch.stop)

    def confirmed(self, *cases):
        return {case: Confirmation(True, "r", "REPORT-" + case) for case in cases}

    def test_the_first_killer_confirmed_in_the_family_is_the_evidence_and_every_confirmed_killer_is_learned(self):
        self.patched(divergent={"fam-b": ["c1", "c2", "c3"]}, only_in_mutant=lambda case: case != "c1", confirmations=self.confirmed("c2", "c3"))
        verdict = cd.family_kill("/mutant", "m")
        self.assertEqual(verdict[0], cd.DETECTED)
        self.assertIn("family replay", verdict[3])
        self.assertEqual((self.published[0][0], self.published[0][2]), ("m", "REPORT-c2"))
        self.assertEqual(self.learned, [(cd.killers_file("m"), ["c2", "c3"])])

    def test_a_divergence_the_clean_control_shares_is_not_a_detection_even_when_the_tool_confirms_it(self):
        self.patched(divergent={"fam-a": ["c1"]}, only_in_mutant=lambda case: False, confirmations=self.confirmed("c1"))
        self.assertIsNone(cd.family_kill("/mutant", "m"))

    def test_the_mutant_and_the_control_are_compared_in_that_order_and_the_mutant_is_the_one_the_tool_confirms(self):
        self.patched(divergent={"fam-a": ["c1"]}, confirmations=self.confirmed("c1"))
        self.assertEqual(cd.family_kill("/mutant", "m")[0], cd.DETECTED)   # the double is true only for ("/mutant", "/control") and confirms only "/mutant"

    def test_a_divergence_the_real_tool_does_not_confirm_is_not_a_detection(self):
        self.patched(divergent={"fam-a": ["c1"]}, confirmations={"c1": Confirmation(False, "r", "x")})
        self.assertIsNone(cd.family_kill("/mutant", "m"))

    def test_the_killers_of_the_previous_report_are_preserved_before_the_report_is_replaced_and_learning_comes_with_them(self):
        self.patched(divergent={"fam-a": ["c1"]}, confirmations=self.confirmed("c1"))
        cd.family_kill("/mutant", "m")
        self.assertEqual(self.calls, ["preserve", "learn", "publish"])

    def test_an_unknown_mapping_or_a_missing_control_goes_to_the_full_replay(self):
        self.patched(families=())
        self.assertIsNone(cd.family_kill("/mutant", "m"))
        self.patched(control=None, divergent={"fam-a": ["c1"]}, confirmations=self.confirmed("c1"))
        self.assertIsNone(cd.family_kill("/mutant", "m"))

    def test_nothing_found_hands_over_to_the_full_replay_never_to_undetected(self):
        self.patched()
        self.assertIsNone(cd.family_kill("/mutant", "m"))
        self.assertEqual((self.published, self.learned), ([], []))

    def test_a_failing_family_step_falls_back_to_the_full_replay_and_is_never_a_detection(self):
        for kind in ("timeout", "harness"):
            self.patched(replay_error=ladder.StepFailure(kind, "slow or rejected"), divergent={"fam-a": ["c1"]}, confirmations=self.confirmed("c1"))
            self.assertIsNone(cd.family_kill("/mutant", "m"), kind)
            self.assertEqual((self.published, self.learned), ([], []))


class VerdictForBinary(unittest.TestCase):
    """The order of the shortcuts: fast-kill, then family-kill, then the authoritative full replay; --no-fast-kill disables both shortcuts."""
    def patched(self, fast=None, family=None, fast_error=None, family_error=None, fast_kill_enabled=True):
        self.calls = []

        def step(name, result, error=None):
            def run(*args):
                self.calls.append(name)
                if error:
                    raise error
                return result
            return run

        for target, replacement in (("fast_kill", step("fast", fast, fast_error)), ("family_kill", step("family", family, family_error)),
                                    ("full_replay_verdict", step("full", ("UNDETECTED", 1, 1, "full"))), ("publish_fast_kill_report", step("publish", None))):
            patch = mock.patch.object(cd, target, side_effect=replacement)
            patch.start()
            self.addCleanup(patch.stop)
        flag = mock.patch.object(cd, "FAST_KILL", fast_kill_enabled)
        flag.start()
        self.addCleanup(flag.stop)

    def test_a_fast_kill_decides_and_neither_the_family_nor_the_full_replay_runs(self):
        self.patched(fast=cd.FastKill("c1", "report", "reason"))
        self.assertEqual(cd.verdict_for_binary("/mutant", "m")[0], cd.DETECTED)
        self.assertEqual(self.calls, ["fast", "publish"])

    def test_without_a_fast_kill_the_family_decides_before_the_full_replay(self):
        self.patched(family=(cd.DETECTED, None, None, "family replay (x)"))
        self.assertEqual(cd.verdict_for_binary("/mutant", "m")[3], "family replay (x)")
        self.assertEqual(self.calls, ["fast", "family"])

    def test_with_no_shortcut_the_full_replay_decides_after_both_were_tried_in_that_order(self):
        self.patched()
        self.assertEqual(cd.verdict_for_binary("/mutant", "m")[0], "UNDETECTED")
        self.assertEqual(self.calls, ["fast", "family", "full"])

    def test_no_fast_kill_disables_both_shortcuts(self):
        self.patched(fast=cd.FastKill("c1", "r", "x"), family=(cd.DETECTED, None, None, "family"), fast_kill_enabled=False)
        self.assertEqual(cd.verdict_for_binary("/mutant", "m")[0], "UNDETECTED")
        self.assertEqual(self.calls, ["full"])

    def test_the_replay_lock_budget_running_out_in_either_shortcut_is_pending_never_a_verdict(self):
        for kwargs in ({"fast_error": cd.replay_lib.ReplayLockTimeout()}, {"family_error": cd.replay_lib.ReplayLockTimeout()}):
            self.patched(**kwargs)
            self.assertEqual(cd.verdict_for_binary("/mutant", "m")[0], cd.BUDGET_EXHAUSTED)
            self.assertNotIn("full", self.calls)


if __name__ == "__main__":
    unittest.main()
