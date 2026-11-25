"""Tests of the real steps of the ladder in check_divergence: the staged replay of one family (with a fake `rust-anything`) and the affinity order
computed from the real tables (killers recorded so far and cases/coverage.tsv)."""
import json, os, subprocess, unittest

import check_divergence as cd
import family_replay as ladder
import replay_subset


def a_family_of_the_corpus():
    entries = replay_subset.load_coverage(os.path.join(cd.ROOT, "cases", "coverage.tsv"))
    smallest = min({entry.family for entry in entries}, key=lambda family: sum(entry.family == family for entry in entries))
    return smallest, [entry.recorded_name for entry in entries if entry.family == smallest]


def replay_output(total, failures=(), skipped=0):
    return json.dumps({"passed": total - len(failures), "total": total, "skipped": skipped,
                       "failures": [{"case": case, "problems": ["differs"]} for case in failures]})


class ReplayOfAFamily(unittest.TestCase):
    def setUp(self):
        self.family, self.names = a_family_of_the_corpus()
        self.applicable = [name for name in self.names if name not in cd.replay_lib.load_not_applicable()]

    def run_with(self, answer):
        staged_cases = []

        def fake(staged_root, binary):
            staged_cases.extend(sorted(f[:-5] for f in os.listdir(os.path.join(staged_root, "rust", "app-cases")) if f.endswith(".toml")))
            return answer(staged_cases)

        return cd.replay_family("/bin/unused", self.family, run=fake), staged_cases

    def test_it_stages_only_the_applicable_cases_of_the_family_and_returns_the_divergent_ones(self):
        victim = self.applicable[0]
        divergent, staged = self.run_with(lambda staged: (1, replay_output(len(staged), [victim])))
        self.assertEqual(divergent, [victim])
        self.assertEqual(sorted(staged), sorted(self.applicable))      # the NOT_APPLICABLE ones are not staged

    def test_a_clean_family_gives_no_divergent_case(self):
        divergent, _ = self.run_with(lambda staged: (0, replay_output(len(staged))))
        self.assertEqual(divergent, [])

    def test_a_timeout_is_a_step_failure_not_a_detection(self):
        def timeout(staged):
            raise subprocess.TimeoutExpired("rust-anything", 1)
        with self.assertRaises(ladder.StepFailure) as raised:
            self.run_with(timeout)
        self.assertEqual(raised.exception.kind, "timeout")

    def test_an_output_the_harness_rejects_is_a_step_failure_not_a_detection(self):
        for output in ("not json", replay_output(3, skipped=1), replay_output(2)):
            with self.assertRaises(ladder.StepFailure, msg=output) as raised:
                self.run_with(lambda staged, output=output: (0, output))
            self.assertEqual(raised.exception.kind, "harness")

    def test_a_total_different_from_the_number_of_staged_cases_is_rejected(self):
        with self.assertRaises(ladder.StepFailure) as raised:
            self.run_with(lambda staged: (0, replay_output(len(staged) + 1)))
        self.assertEqual(raised.exception.kind, "harness")


class AffinityFromTheRealTables(unittest.TestCase):
    def test_a_mutant_of_a_file_with_recorded_killers_gets_families_in_a_stable_order(self):
        name = next(n for n, v in cd.MUTANTS.items() if v[0] == "src/actor/deployer.rs")
        first = cd.families_for(name)
        self.assertTrue(first)
        self.assertEqual(first, cd.families_for(name))
        self.assertEqual(len(first), len(set(first)))

    def test_a_mutant_of_a_file_nobody_else_has_gets_no_affinity(self):
        self.assertEqual(cd.families_for("a-mutant-that-does-not-exist"), [])


if __name__ == "__main__":
    unittest.main()
