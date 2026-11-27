"""cases/coverage.tsv and rust/app-cases must name exactly the same cases (issue #315)."""
import os
import unittest

import replay_subset as subset

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def entry(case):
    return subset.CoverageEntry(case=case, family="f", rows=frozenset(), basis="none")


class ReconcileFindsEveryMismatch(unittest.TestCase):
    def test_a_case_without_recording_and_a_recording_without_case_are_both_reported(self):
        unrecorded, unreferenced = subset.reconcile([entry("kept"), entry("name-ok-28")], {"kept", "orphan"})
        self.assertEqual((unrecorded, unreferenced), (["name-ok-28"], ["orphan"]))

    def test_names_are_compared_as_rust_anything_records_them(self):
        long_name = "i03-watch-old-after-new-exists"      # recorded as its first 24 characters without a trailing dash
        self.assertEqual(subset.reconcile([entry(long_name)], {"i03-watch-old-after-new"}), ([], []))

    def test_a_given_name_that_rust_anything_2_keeps_whole_is_matched_by_its_full_name(self):
        long_name = "st-terminated-perpetrator"      # 25 characters: 1.x cut every name at 24, 2.x keeps a name given in --inputs
        self.assertEqual(subset.reconcile([entry(long_name)], {long_name}), ([], []))

    def test_only_the_full_name_or_the_24_character_one_match_and_a_second_recording_is_reported(self):
        long_name = "st-terminated-perpetrator"
        self.assertEqual(subset.reconcile([entry(long_name)], {long_name[:23]}), ([long_name], [long_name[:23]]))
        self.assertEqual(subset.reconcile([entry(long_name)], {long_name, long_name[:24]}), ([], [long_name[:24]]))

    def test_a_selector_picks_the_name_under_which_the_case_is_recorded(self):
        long_name = "st-terminated-perpetrator"
        self.assertEqual(subset.select_cases([entry(long_name)], families=["f"], recorded_names={long_name}), [long_name])
        self.assertEqual(subset.select_cases([entry(long_name)], families=["f"]), [long_name[:24]])


class TheRepositoryIsReconciled(unittest.TestCase):
    def test_every_coverage_case_is_recorded_and_every_recording_is_covered(self):
        recordings_dir = os.path.join(ROOT, "rust", "app-cases")
        recorded = {name[:-len(".toml")] for name in os.listdir(recordings_dir) if name.endswith(".toml")}
        unrecorded, unreferenced = subset.reconcile(subset.load_coverage(os.path.join(ROOT, "cases", "coverage.tsv")), recorded)
        self.assertEqual((unrecorded, unreferenced), ([], []))


if __name__ == "__main__":
    unittest.main()
