"""One replay's evidence is its own: its classification comes from its own process output (validated against the real contract of
`rust-anything app replay --json`) and the report it publishes is the one that replay wrote, read under a per-checkout lock."""
import json, os, stat, subprocess, tempfile, threading, time, types, unittest

import replay_lib

FAKE_RUST_ANYTHING = r'''#!/bin/sh
# stand-in for `rust-anything app replay REPO --build BIN --json`; the result depends on the build's file name
repo="$3"; build=""
while [ $# -gt 0 ]; do
  case "$1" in --build) build="$2"; shift;; esac
  shift
done
label=$(basename "$build")
report="$repo/rust/app-cases/replay-report.md"
if [ "$label" = "build-a" ]; then
  printf '# App replay\n\nPassed: 1/3\n\n## Failing cases\n\n### c-a1\n\n### c-a2\n' > "$report"
  sleep 1.2
  printf '{"build":["%s"],"passed":1,"total":3,"skipped":0,"failures":[{"case":"c-a1","args":["scenario"],"problems":[{"field":"stdout","detail":"d"}]},{"case":"c-a2","args":["scenario"],"problems":[{"field":"stdout","detail":"d"}]}],"fallback":null,"report":"%s"}\n' "$build" "$report"
  exit 1
fi
if [ "$label" = "build-na" ]; then
  printf '# App replay\n\nPassed: 2/3\n\n## Failing cases\n\n### c-na\n' > "$report"
  printf '{"build":["%s"],"passed":2,"total":3,"skipped":0,"failures":[{"case":"c-na","args":["scenario"],"problems":[{"field":"status","detail":"d"}]}],"fallback":null,"report":"%s"}\n' "$build" "$report"
  exit 1
fi
sleep 0.4
printf '# App replay\n\nPassed: 2/2\n\nAll cases passed.\n' > "$report"
printf '{"build":["%s"],"passed":2,"total":2,"skipped":0,"failures":[],"fallback":null,"report":"%s"}\n' "$build" "$report"
exit 0
'''
PASSING_JSON_ONLY = '#!/bin/sh\necho \'{"build":["b"],"passed":1,"total":1,"skipped":0,"failures":[],"fallback":null,"report":"r"}\'\n'


def make_executable(path, text):
    with open(path, "w") as script:
        script.write(text)
    os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR)


def read_text(path):
    with open(path) as handle:
        return handle.read()


class ReplayIsolation(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = self.tmp.name
        os.makedirs(os.path.join(root, "rust", "app-cases"))
        os.makedirs(os.path.join(root, "cases"))
        os.makedirs(os.path.join(root, "bin"))
        with open(os.path.join(root, "cases", "not-applicable.tsv"), "w") as na_file:
            na_file.write("case\treason\tdecision\tlink\n")
        for case in ("c-a1", "c-a2", "c-na"):
            base = os.path.join(root, "rust", "app-cases", case)
            with open(base + ".toml", "w") as recording:
                recording.write('bin.name = "x"\nargs = ["scenario"]\nstatus.code = 0\n')
            with open(base + ".stdin", "w") as stdin_file:
                stdin_file.write("input")
            with open(base + ".stdout", "w") as stdout_file:
                stdout_file.write("pekko")
        self.tool = os.path.join(root, "bin", "rust-anything")
        make_executable(self.tool, FAKE_RUST_ANYTHING)
        self.build_a, self.build_b, self.build_na = (os.path.join(root, name) for name in ("build-a", "build-b", "build-na"))
        for build in (self.build_a, self.build_b):
            make_executable(build, '#!/bin/sh\nprintf "mutated"\n')
        make_executable(self.build_na, '#!/bin/sh\necho "input outside the API domain: x" >&2\nexit 2\n')
        self.saved = (replay_lib.ROOT, replay_lib.REPORT, replay_lib.NA_FILE, os.environ["PATH"])
        replay_lib.ROOT = root
        replay_lib.REPORT = os.path.join(root, "rust", "app-cases", "replay-report.md")
        replay_lib.NA_FILE = os.path.join(root, "cases", "not-applicable.tsv")
        os.environ["PATH"] = os.path.join(root, "bin") + os.pathsep + self.saved[3]

    def tearDown(self):
        replay_lib.ROOT, replay_lib.REPORT, replay_lib.NA_FILE, os.environ["PATH"] = self.saved
        self.tmp.cleanup()

    def crossed_replays(self):
        """Replay A starts first and finishes last; replay B starts and finishes in the middle of it."""
        results = {}
        def run(label, binary):
            results[label] = replay_lib.run_replay(binary, 30)
        first = threading.Thread(target=run, args=("a", self.build_a))
        second = threading.Thread(target=run, args=("b", self.build_b))
        first.start()
        time.sleep(0.3)
        second.start()
        first.join()
        second.join()
        return results

    # --- isolation between replays ---
    def test_a_replay_that_diverged_is_never_reported_clean_because_another_replay_rewrote_the_shared_report(self):
        results = self.crossed_replays()
        verdict_a = replay_lib.evaluate(self.build_a, results["a"])
        verdict_b = replay_lib.evaluate(self.build_b, results["b"])
        self.assertFalse(verdict_a["ok"], verdict_a["line"])
        self.assertEqual(verdict_a["applicable_failing"], ["c-a1", "c-a2"])
        self.assertTrue(verdict_b["ok"], verdict_b["line"])
        self.assertEqual(verdict_b["applicable_failing"], [])

    def test_every_replay_carries_the_report_that_its_own_run_wrote(self):
        results = self.crossed_replays()
        self.assertIn("c-a1", results["a"].report_text)
        self.assertNotIn("c-a1", results["b"].report_text)
        self.assertIn("All cases passed", results["b"].report_text)

    def test_the_lock_is_released_after_a_replay_that_times_out(self):
        make_executable(self.tool, '#!/bin/sh\nsleep 30\n')
        with self.assertRaises(subprocess.TimeoutExpired):
            replay_lib.run_replay(self.build_b, 1)
        make_executable(self.tool, PASSING_JSON_ONLY)
        self.assertEqual(replay_lib.run_replay(self.build_b, 5).passed, 1)   # would block forever if the lock leaked

    # --- valid replays keep their classification ---
    def test_a_valid_replay_with_divergences_keeps_its_classification(self):
        result = replay_lib.run_replay(self.build_a, 30)
        self.assertEqual((result.returncode, result.passed, result.total, result.failures), (1, 1, 3, ["c-a1", "c-a2"]))
        verdict = replay_lib.evaluate(self.build_a, result)
        self.assertFalse(verdict["ok"])
        self.assertEqual(verdict["applicable_failing"], ["c-a1", "c-a2"])

    def test_a_not_applicable_case_that_is_rejected_explicitly_still_counts_as_expected(self):
        with open(replay_lib.NA_FILE, "a") as na_file:
            na_file.write("c-na\tout of domain\tQ-X-17\tdocs\n")
        verdict = replay_lib.evaluate(self.build_na, replay_lib.run_replay(self.build_na, 30))
        self.assertTrue(verdict["ok"], verdict["line"])
        self.assertEqual(verdict["line"], "2/2 applicable + 0 NA:Q-CFG-30 + 1 not applicable (diverge 0)")

    # --- the report published is the report of this replay ---
    def test_a_report_left_by_an_earlier_replay_is_never_published(self):
        with open(replay_lib.REPORT, "w") as old_report:
            old_report.write("# App replay\n\nPassed: 1/1\n\n### stale-case-from-an-earlier-run\n")
        make_executable(self.tool, PASSING_JSON_ONLY)          # this replay prints JSON and writes no report
        self.assertIsNone(replay_lib.run_replay(self.build_b, 30).report_text)

    def test_a_report_written_by_this_replay_replaces_the_earlier_one(self):
        with open(replay_lib.REPORT, "w") as old_report:
            old_report.write("# App replay\n\nPassed: 9/9\n\nold\n")
        result = replay_lib.run_replay(self.build_b, 30)
        self.assertIn("All cases passed", result.report_text)
        self.assertNotIn("old", result.report_text.replace("All cases passed", ""))

    def test_a_report_whose_counts_contradict_the_json_of_the_same_replay_is_not_published(self):
        make_executable(self.tool, '#!/bin/sh\nrepo="$3"\nprintf "# App replay\\n\\nPassed: 5/9\\n" > "$repo/rust/app-cases/replay-report.md"\n'
                                   'echo \'{"passed":1,"total":1,"skipped":0,"failures":[]}\'\n')
        self.assertIsNone(replay_lib.run_replay(self.build_b, 30).report_text)

    def test_a_missing_report_is_none(self):
        make_executable(self.tool, PASSING_JSON_ONLY)
        self.assertFalse(os.path.exists(replay_lib.REPORT))
        self.assertIsNone(replay_lib.run_replay(self.build_b, 30).report_text)


def completed(returncode, payload):
    return types.SimpleNamespace(returncode=returncode, stdout=payload if isinstance(payload, str) else json.dumps(payload), stderr="")


def failure(case="c1", problems=None):
    return {"case": case, "args": ["scenario"], "problems": [{"field": "stdout", "detail": "d"}] if problems is None else problems}


def payload(passed, total, failures=(), skipped=0):
    return {"build": ["b"], "passed": passed, "total": total, "skipped": skipped, "failures": list(failures), "fallback": None, "report": "r"}


class ReplayContract(unittest.TestCase):
    """The contract observed on the real tool: exit 0 <=> no failures and passed == total; exit 1 <=> failures; always
    passed + skipped + len(failures) == total; every failure has a case name and a non-empty list of problems; exit 2 prints no JSON."""

    def test_a_valid_clean_replay_is_accepted(self):
        result = replay_lib.parse_replay_output(completed(0, payload(3, 3)), None)
        self.assertEqual((result.passed, result.total, result.failures, result.problem), (3, 3, [], None))

    def test_a_valid_replay_with_divergences_is_accepted_even_though_the_exit_code_is_not_zero(self):
        result = replay_lib.parse_replay_output(completed(1, payload(1, 3, [failure("c1"), failure("c2")])), None)
        self.assertEqual((result.passed, result.total, result.failures, result.problem), (1, 3, ["c1", "c2"], None))

    def test_the_incomplete_json_that_used_to_give_a_false_clean_is_a_harness_error(self):
        result = replay_lib.parse_replay_output(completed(1, '{"passed":1,"total":3}'), None)
        self.assertIsNone(result.passed)
        self.assertTrue(result.problem)
        verdict = replay_lib.evaluate("/nonexistent/binary", result)
        self.assertIsNone(verdict["ok"])                                    # HARNESS_ERROR, never CLEAN
        self.assertIn("missing", verdict["line"])

    def test_contradictory_or_incomplete_results_are_harness_errors(self):
        invalid = {
            "exit 0 but a failure is listed": (0, payload(2, 3, [failure()])),
            "exit 0 but passed < total": (0, payload(2, 3)),
            "exit 1 but no failure is listed": (1, payload(3, 3)),
            "exit 1, passed < total, no failure listed": (1, payload(1, 3)),
            "counts do not add up (passed + failures != total)": (1, payload(1, 3, [failure()])),
            "more failures than cases": (1, payload(0, 1, [failure("a"), failure("b")])),
            "a skipped case is no evidence": (0, payload(2, 3, skipped=1)),
            "exit 2 with a JSON body": (2, payload(1, 1)),
            "failure without a case name": (1, payload(0, 1, [{"args": [], "problems": [{"field": "x", "detail": "d"}]}])),
            "failure without problems": (1, payload(0, 1, [failure("c1", problems=[])])),
            "failure that is not an object": (1, payload(0, 1, ["c1"])),
            "the same case twice": (1, payload(0, 2, [failure("c1"), failure("c1")])),
            "negative count": (1, payload(-1, 0, [failure()])),
            "non-integer count": (0, {"passed": "3", "total": 3, "skipped": 0, "failures": []}),
            "missing failures key": (0, {"passed": 3, "total": 3, "skipped": 0}),
            "missing skipped key": (0, {"passed": 3, "total": 3, "failures": []}),
            "failures is not a list": (1, {"passed": 0, "total": 1, "skipped": 0, "failures": {"case": "c1"}}),
            "not JSON at all": (0, "Passed: 3/3 cases passed"),
            "empty output": (0, ""),
            "JSON that is not an object": (0, "[1, 2, 3]"),
        }
        for label, (code, body) in invalid.items():
            result = replay_lib.parse_replay_output(completed(code, body), None)
            self.assertIsNone(result.passed, label)
            self.assertTrue(result.problem, label)
            self.assertIsNone(replay_lib.evaluate("/nonexistent/binary", result)["ok"], label)

    def test_a_result_that_is_a_harness_error_publishes_no_report(self):
        result = replay_lib.parse_replay_output(completed(1, '{"passed":1,"total":3}'), "# App replay\n")
        self.assertIsNone(result.report_text)


if __name__ == "__main__":
    unittest.main()
