"""The history of known killers is VERSIONABLE and TRACEABLE, and it is never evidence: a manifest names, for every mutant, the definition of the mutant it was recorded for (a hash of its patch),
the bytes of its killers file, the number of cases, the tree of the code under test and the size of the corpus; no timestamp, no path, no machine detail, sorted by mutant, so the same detection
writes the same bytes. A history recorded for another definition of the mutant is ignored (more work, never a different verdict). Every candidate is still confirmed by the authoritative tool."""
import os
import tempfile
import unittest

import check_divergence as cd
import replay_lib

HEADER = "mutant\tpatch\tcases\tkillers\tcrate_tree\tcorpus\n"


def replayed(failures, total=4):
    return replay_lib.ReplayResult(1, 1, total, failures, "### c-a1\n", "", "", None)


def evaluation(ok, applicable_failing, total=4):
    return dict(ok=ok, line="a line", applicable_failing=applicable_failing, na_problems=[], passed=total - len(applicable_failing), total=total, na_q_cfg_30=[], other_na=0)


class HistoryProvenance(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.saved = (cd.REPORTS, cd.replay, replay_lib.evaluate, dict(cd.MUTANTS), getattr(cd, "crate_tree", None))
        cd.REPORTS = self.directory.name
        cd.MUTANTS.clear()
        cd.MUTANTS.update({"mutant-b": ("src/b.rs", "old b", "new b", "why"), "mutant-a": ("src/a.rs", "old a", "new a", "why")})
        cd.crate_tree = lambda: "tree0123"
        self.addCleanup(self.restore)

    def restore(self):
        cd.REPORTS, cd.replay, replay_lib.evaluate = self.saved[:3]
        cd.MUTANTS.clear()
        cd.MUTANTS.update(self.saved[3])
        cd.crate_tree = self.saved[4]
        self.directory.cleanup()

    def detect(self, name, failures, ok=False, total=4):
        cd.replay = lambda binary: replayed(failures, total)
        replay_lib.evaluate = lambda binary, result: evaluation(ok, [] if ok else failures, total)
        return cd.full_replay_verdict("/some/binary", name)

    def manifest(self):
        with open(cd.provenance_file()) as handle:
            return handle.read()

    def test_a_confirmed_detection_records_its_provenance_in_a_sorted_manifest_without_time_or_path(self):
        self.detect("mutant-b", ["c-2", "c-1"])
        self.detect("mutant-a", ["c-9"])
        lines = self.manifest().splitlines(keepends=True)
        self.assertEqual(lines[0], HEADER)
        self.assertEqual([line.split("\t")[0] for line in lines[1:]], ["mutant-a", "mutant-b"])
        fields = lines[2].rstrip("\n").split("\t")
        self.assertEqual((fields[2], fields[4], fields[5]), ("2", "tree0123", "4"))
        self.assertEqual(len(fields[1]), 16)
        self.assertEqual(len(fields[3]), 16)
        for line in lines:
            self.assertNotIn("/", line.replace("\t", " "), line)

    def test_the_same_detections_write_the_same_bytes_whatever_the_order_they_happened_in(self):
        self.detect("mutant-b", ["c-1"])
        self.detect("mutant-a", ["c-9"])
        first = self.manifest()
        os.remove(cd.provenance_file())
        self.detect("mutant-a", ["c-9"])
        self.detect("mutant-b", ["c-1"])
        self.assertEqual(self.manifest(), first)

    def test_a_survivor_leaves_no_entry_and_an_infrastructure_failure_does_not_touch_the_manifest(self):
        self.detect("mutant-a", ["c-9"])
        before = self.manifest()
        cd.replay = lambda binary: replay_lib.ReplayResult(2, None, None, [], None, "", "", "output not understood")
        replay_lib.evaluate = lambda binary, result: dict(ok=None, line="rejected", applicable_failing=[], na_problems=[], passed=None, total=None, na_q_cfg_30=[], other_na=0)
        self.assertEqual(cd.full_replay_verdict("/some/binary", "mutant-a")[0], cd.HARNESS_ERROR)
        self.assertEqual(self.manifest(), before)
        self.detect("mutant-a", [], ok=True)
        self.assertNotIn("mutant-a", self.manifest())

    def test_a_history_recorded_for_another_definition_of_the_mutant_is_ignored_and_the_same_definition_is_used(self):
        self.detect("mutant-a", ["c-9"])
        self.assertEqual(cd.known_killers("mutant-a"), ["c-9"])
        cd.MUTANTS["mutant-a"] = ("src/a.rs", "old a", "a DIFFERENT patch", "why")
        self.assertEqual(cd.known_killers("mutant-a"), [])

    def test_a_history_without_a_manifest_entry_is_still_only_a_candidate_list_and_a_mutant_that_is_not_defined_has_none_to_trust(self):
        with open(cd.killers_file("mutant-a"), "w") as handle:
            handle.write("c-7\n")
        self.assertEqual(cd.known_killers("mutant-a"), ["c-7"])                   # legacy history: a candidate, confirmed by the authoritative tool before it counts
        self.detect("mutant-b", ["c-1"])
        del cd.MUTANTS["mutant-b"]
        self.assertEqual(cd.known_killers("mutant-b"), [])


if __name__ == "__main__":
    unittest.main()
