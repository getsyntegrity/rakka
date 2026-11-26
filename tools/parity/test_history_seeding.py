"""The history of known killers is seeded only by a CONFIRMED detection and holds only cases that can confirm one again: a verdict that is not DETECTED never fills it, a case that is
NOT_APPLICABLE or NA:Q-CFG-30 never enters it, and an infrastructure failure never touches it. The replay itself is replaced (the boundary substituted is `check_divergence.replay` and
`replay_lib.evaluate`); every other part is the real one."""
import os
import tempfile
import unittest

import check_divergence as cd
import replay_lib


def replayed(failures, report="### c-a1\n"):
    return replay_lib.ReplayResult(1, 1, 4, failures, report, "", "", None)


def evaluation(ok, applicable_failing, passed=1, total=4):
    return dict(ok=ok, line="a line", applicable_failing=applicable_failing, na_problems=[], passed=passed, total=total, na_q_cfg_30=[], other_na=1)


class HistorySeeding(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.saved = (cd.REPORTS, cd.replay, replay_lib.evaluate)
        cd.REPORTS = self.directory.name
        self.addCleanup(self.restore)

    def restore(self):
        cd.REPORTS, cd.replay, replay_lib.evaluate = self.saved
        self.directory.cleanup()

    def verdict_of(self, failures, ev, name="a-mutant"):
        cd.replay = lambda binary: replayed(failures)
        replay_lib.evaluate = lambda binary, result: ev
        return cd.full_replay_verdict("/some/binary", name)

    def killers_text(self, name="a-mutant"):
        with open(cd.killers_file(name)) as handle:
            return handle.read()

    def test_a_confirmed_detection_records_only_the_applicable_divergent_cases_in_the_order_of_the_replay(self):
        outcome = self.verdict_of(["c-a1", "c-na", "c-q", "c-a2"], evaluation(False, ["c-a1", "c-a2"]))
        self.assertEqual(outcome[0], cd.DETECTED)
        self.assertEqual(self.killers_text(), "c-a1\nc-a2\n")

    def test_a_mutant_that_survives_clears_its_history_and_never_records_the_divergent_cases_that_do_not_count(self):
        with open(cd.killers_file("a-mutant"), "w") as earlier:
            earlier.write("c-old\n")
        outcome = self.verdict_of(["c-na"], evaluation(True, [], passed=3, total=3))
        self.assertEqual(outcome[0], cd.UNDETECTED)
        self.assertEqual(self.killers_text(), "")

    def test_a_detection_without_an_applicable_divergence_leaves_no_killer_to_try_again(self):
        ev = evaluation(False, [])
        ev["na_problems"] = ["c-na: the harness did not reject the out-of-domain input explicitly"]
        outcome = self.verdict_of(["c-na"], ev)
        self.assertEqual(outcome[0], cd.DETECTED)
        self.assertEqual(self.killers_text(), "")

    def test_the_history_is_written_in_a_deterministic_order_without_duplicates_whatever_the_order_the_replay_reported(self):
        outcome = self.verdict_of(["c-z9", "c-a2", "c-a1", "c-a2"], evaluation(False, ["c-z9", "c-a2", "c-a1", "c-a2"]))
        self.assertEqual(outcome[0], cd.DETECTED)
        self.assertEqual(self.killers_text(), "c-a1\nc-a2\nc-z9\n")

    def test_the_history_holds_nothing_but_case_names_so_that_no_machine_specific_path_can_enter_it(self):
        self.verdict_of(["c-a1"], evaluation(False, ["c-a1"]))
        self.assertEqual([line for line in self.killers_text().splitlines() if "/" in line or "\\" in line or line != line.strip()], [])

    def test_a_replay_whose_output_is_not_understood_never_touches_the_history(self):
        with open(cd.killers_file("a-mutant"), "w") as earlier:
            earlier.write("c-a1\nc-a2\n")
        cd.replay = lambda binary: replay_lib.ReplayResult(2, None, None, [], None, "", "", "output not understood")
        replay_lib.evaluate = lambda binary, result: dict(ok=None, line="rejected", applicable_failing=[], na_problems=[], passed=None, total=None, na_q_cfg_30=[], other_na=0)
        outcome = cd.full_replay_verdict("/some/binary", "a-mutant")
        self.assertEqual(outcome[0], cd.HARNESS_ERROR)
        self.assertEqual(self.killers_text(), "c-a1\nc-a2\n")
        self.assertTrue(os.path.exists(cd.killers_file("a-mutant")))


if __name__ == "__main__":
    unittest.main()
