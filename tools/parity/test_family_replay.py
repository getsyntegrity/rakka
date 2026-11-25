"""Tests of the detection ladder of a replay mutant: known killers, then the families its file is affine to, then the full replay.

A known killer or a family may prove DETECTED; they can never prove UNDETECTED. Everything else (nothing found, an unknown mapping, a stale
killer) hands over to the complete replay, and an error or a timeout of a step is never a detection."""
import os, tempfile, unittest

import family_replay as ladder


class Doubles:
    """Records what the ladder asked for, and answers from tables."""

    def __init__(self, demonstrated=(), by_family=None, failing=None):
        self.demonstrated, self.by_family, self.failing = set(demonstrated), by_family or {}, failing or {}
        self.known_asked, self.families_asked, self.learned = [], [], []

    def kill_known(self, case):
        self.known_asked.append(case)
        return case in self.demonstrated

    def kill_family(self, family):
        self.families_asked.append(family)
        if family in self.failing:
            raise ladder.StepFailure(self.failing[family], f"{family} failed")
        return self.by_family.get(family, [])

    def learn(self, cases):
        self.learned.append(list(cases))


def climb(known, families, doubles):
    return ladder.run_ladder(known, families, doubles.kill_known, doubles.kill_family, doubles.learn)


class TheLadder(unittest.TestCase):
    def test_a_demonstrated_known_killer_detects_without_asking_for_any_family(self):
        doubles = Doubles(demonstrated={"k2"})
        result = climb(["k1", "k2", "k3"], ["fam-a"], doubles)
        self.assertEqual((result.path, result.killers), (ladder.KNOWN_KILLER, ["k2"]))
        self.assertEqual(doubles.known_asked, ["k1", "k2"])      # stops at the first demonstrated one
        self.assertEqual(doubles.families_asked, [])

    def test_when_the_known_killers_fail_the_first_family_with_a_confirmed_divergence_detects_and_its_cases_are_learned(self):
        doubles = Doubles(by_family={"fam-b": ["c9", "c10"]})
        result = climb(["k1"], ["fam-a", "fam-b", "fam-c"], doubles)
        self.assertEqual((result.path, result.killers), (ladder.FAMILY, ["c9", "c10"]))
        self.assertEqual(doubles.families_asked, ["fam-a", "fam-b"])   # fam-c is never replayed
        self.assertEqual(doubles.learned, [["c9", "c10"]])

    def test_when_no_family_confirms_a_divergence_the_full_replay_decides(self):
        doubles = Doubles()
        result = climb(["k1"], ["fam-a", "fam-b"], doubles)
        self.assertEqual((result.path, result.killers), (ladder.FULL_REPLAY, []))
        self.assertEqual(doubles.families_asked, ["fam-a", "fam-b"])
        self.assertEqual(doubles.learned, [])

    def test_an_unknown_family_mapping_goes_straight_to_the_full_replay(self):
        doubles = Doubles()
        result = climb([], [], doubles)
        self.assertEqual(result.path, ladder.FULL_REPLAY)
        self.assertEqual((doubles.known_asked, doubles.families_asked), ([], []))

    def test_only_the_first_families_are_tried_before_the_full_replay(self):
        doubles = Doubles()
        climb([], ["f1", "f2", "f3", "f4", "f5"], doubles)
        self.assertEqual(doubles.families_asked, ["f1", "f2", "f3"])
        self.assertEqual(ladder.MAX_FAMILIES, 3)

    def test_a_stale_known_killer_is_skipped_not_a_detection(self):
        doubles = Doubles(by_family={"fam-a": ["c1"]})   # "gone" is not demonstrated: not recorded any more, or the control differs
        result = climb(["gone"], ["fam-a"], doubles)
        self.assertEqual(result.path, ladder.FAMILY)
        self.assertEqual(doubles.known_asked, ["gone"])

    def test_a_timeout_or_a_harness_error_of_a_family_is_never_a_detection_and_does_not_fall_through(self):
        for kind, expected in (("timeout", ladder.TIMED_OUT), ("harness", ladder.HARNESS_ERROR)):
            doubles = Doubles(by_family={"fam-b": ["c1"]}, failing={"fam-a": kind})
            result = climb([], ["fam-a", "fam-b"], doubles)
            self.assertEqual((result.path, result.killers), (expected, []), kind)
            self.assertEqual(doubles.families_asked, ["fam-a"], kind)    # fam-b is not tried after the failure
            self.assertEqual(doubles.learned, [], kind)

    def test_the_ladder_never_answers_undetected(self):
        # a surviving mutant: nothing is found anywhere. The only paths a ladder can end in are these; "undetected" is the full replay's to say
        result = climb(["k"], ["fam-a"], Doubles())
        self.assertIn(result.path, {ladder.KNOWN_KILLER, ladder.FAMILY, ladder.FULL_REPLAY, ladder.TIMED_OUT, ladder.HARNESS_ERROR})
        self.assertEqual(result.path, ladder.FULL_REPLAY)


class ConfirmedKillers(unittest.TestCase):
    def test_a_divergence_that_the_clean_control_has_too_is_not_a_kill(self):
        only_in_mutant = {"c1": True, "c2": False, "c3": True}
        confirmed = ladder.confirmed_killers(["c1", "c2", "c3"], lambda case: only_in_mutant[case], lambda case: True)
        self.assertEqual(confirmed, ["c1", "c3"])

    def test_a_case_the_real_tool_does_not_confirm_as_applicable_is_not_a_kill(self):
        confirmed = ladder.confirmed_killers(["c1", "c2"], lambda case: True, lambda case: case == "c2")
        self.assertEqual(confirmed, ["c2"])

    def test_at_most_the_candidate_limit_is_confirmed_and_the_order_is_kept(self):
        cases = [f"c{n}" for n in range(20)]
        self.assertEqual(ladder.confirmed_killers(cases, lambda c: True, lambda c: True, limit=4), cases[:4])


class Affinity(unittest.TestCase):
    FAMILY = {"a1": "deployer", "a2": "deployer", "b1": "deploy", "c1": "hocon", "d1": "deployer"}

    def test_families_are_ordered_by_how_often_they_killed_mutants_of_the_same_file_then_by_name(self):
        killers = {"m1": ["a1", "b1"], "m2": ["a2", "b1"], "m3": ["c1"], "self": ["d1"], "other-file": ["c1"]}
        files = {"m1": "deployer.rs", "m2": "deployer.rs", "m3": "deployer.rs", "self": "deployer.rs", "other-file": "cell.rs"}
        order = ladder.affinity("self", "deployer.rs", killers, files, self.FAMILY.get)
        self.assertEqual(order, ["deploy", "deployer", "hocon"])   # deploy 2, deployer 2 (name order), hocon 1; `self` and other files ignored

    def test_the_order_is_deterministic(self):
        killers = {"m1": ["c1", "a1"], "m2": ["b1"]}
        files = {"m1": "f.rs", "m2": "f.rs"}
        first = ladder.affinity("x", "f.rs", killers, files, self.FAMILY.get)
        self.assertEqual(first, ladder.affinity("x", "f.rs", dict(reversed(list(killers.items()))), files, self.FAMILY.get))

    def test_a_file_with_no_known_killers_has_no_affinity(self):
        self.assertEqual(ladder.affinity("x", "new.rs", {"m1": ["a1"]}, {"m1": "other.rs"}, self.FAMILY.get), [])

    def test_a_killer_case_that_is_not_in_the_coverage_table_is_ignored(self):
        self.assertEqual(ladder.affinity("x", "f.rs", {"m1": ["ghost", "a1"]}, {"m1": "f.rs"}, self.FAMILY.get), ["deployer"])


class TokenAffinity(unittest.TestCase):
    FAMILIES = ["oracle-deployer", "oracle-deployer-settings", "oracle-deploy", "oracle-actor-system-setup", "oracle-config-hocon",
                "oracle-config-settings", "scenario-2a", "oracle-mailbox"]

    def order(self, file, name):
        return ladder.token_affinity(name, file, self.FAMILIES)

    def test_the_stem_of_the_file_picks_the_family_that_names_it(self):
        self.assertEqual(self.order("src/actor/deployer.rs", "x")[:2], ["oracle-deployer", "oracle-deployer-settings"])   # fewer extra tokens first
        self.assertEqual(self.order("src/actor/deploy.rs", "x"), ["oracle-deploy"])                  # `deploy` is not `deployer`
        self.assertEqual(self.order("src/config/setup.rs", "x"), ["oracle-actor-system-setup"])

    def test_the_words_of_the_mutant_name_add_evidence(self):
        self.assertEqual(self.order("src/config/settings.rs", "settings-version-check-skipped")[0], "oracle-config-settings")
        self.assertEqual(self.order("src/other/file.rs", "deployer-lookup-keeps-the-guardian")[0], "oracle-deployer")

    def test_a_file_and_name_that_no_family_names_have_no_token_affinity(self):
        self.assertEqual(self.order("src/actor/cell.rs", "failure-does-not-suspend-mailbox"), ["oracle-mailbox"])   # the word `mailbox` names a family
        self.assertEqual(self.order("src/actor/cell.rs", "restart-limit-allows-one-more"), [])

    def test_it_is_deterministic_and_never_repeats_a_family(self):
        order = self.order("src/actor/deployer.rs", "deployer-settings-x")
        self.assertEqual(order, self.order("src/actor/deployer.rs", "deployer-settings-x"))
        self.assertEqual(len(order), len(set(order)))

    def test_the_combined_order_puts_the_killer_families_first_and_does_not_repeat(self):
        self.assertEqual(ladder.combined(["hocon", "deploy"], ["deploy", "deployer"]), ["hocon", "deploy", "deployer"])
        self.assertEqual(ladder.combined([], ["deployer"]), ["deployer"])


class LearnedKillers(unittest.TestCase):
    def test_new_confirmed_cases_are_merged_without_shrinking_or_duplicating(self):
        self.assertEqual(ladder.merged_killers(["c1", "c2"], ["c2", "c3"]), ["c1", "c2", "c3"])
        self.assertEqual(ladder.merged_killers([], ["c9"]), ["c9"])
        self.assertEqual(ladder.merged_killers(["c1"], []), ["c1"])

    def test_the_learned_killers_survive_in_the_killers_file_for_the_next_run(self):
        with tempfile.TemporaryDirectory() as directory:
            path = os.path.join(directory, "replay-killers-m.txt")
            ladder.learn_into(path, ["c1"])
            ladder.learn_into(path, ["c2", "c1"])
            with open(path, encoding="utf-8") as stored:
                self.assertEqual(stored.read().split(), ["c1", "c2"])


if __name__ == "__main__":
    unittest.main()
