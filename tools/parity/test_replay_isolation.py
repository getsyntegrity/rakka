"""One replay's evidence is its own: its classification comes from its own process output (validated against the real contract of
`rust-anything app replay --json`) and the report it publishes is the one that replay wrote, read under a per-checkout lock."""
import json, os, stat, subprocess, tempfile, threading, time, types, unittest

import replay_lib

FAKE_RUST_ANYTHING = r'''#!/bin/sh
# stand-in for `rust-anything app replay REPO --build BIN --json`; the result depends on the build's file name
repo="$3"; build=""; case_id=""
while [ $# -gt 0 ]; do
  case "$1" in --build) build="$2"; shift;; --case) case_id="$2"; shift;; esac
  shift
done
label=$(basename "$build")
report="$repo/rust/app-cases/replay-report.md"
if [ -n "$case_id" ]; then
  if [ "$label" = "build-a" ] || [ "$label" = "build-q" ]; then
    printf '# App replay\n\nPassed: 0/1\n\n## Failing cases\n\n### %s\n' "$case_id" > "$report"
    printf '{"build":["%s"],"passed":0,"total":1,"skipped":0,"failures":[{"case":"%s","args":["scenario"],"problems":[{"field":"stdout","detail":"d"}]}],"fallback":null,"report":"%s"}\n' "$build" "$case_id" "$report"
    exit 1
  fi
  printf '# App replay\n\nPassed: 1/1\n' > "$report"
  printf '{"build":["%s"],"passed":1,"total":1,"skipped":0,"failures":[],"fallback":null,"report":"%s"}\n' "$build" "$report"
  exit 0
fi
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
        for case in ("c-a1", "c-a2", "c-na", "c-q"):
            base = os.path.join(root, "rust", "app-cases", case)
            with open(base + ".toml", "w") as recording:
                recording.write('bin.name = "x"\nargs = ["scenario"]\nstatus.code = 0\n')
            with open(base + ".stdin", "w") as stdin_file:
                stdin_file.write("input")
            with open(base + ".stdout", "w") as stdout_file:
                stdout_file.write("NUM-SHORT x -> ok 1.9999999999999998E23" if case == "c-q" else "pekko")
        self.tool = os.path.join(root, "bin", "rust-anything")
        make_executable(self.tool, FAKE_RUST_ANYTHING)
        self.build_a, self.build_b, self.build_na = (os.path.join(root, name) for name in ("build-a", "build-b", "build-na"))
        for build in (self.build_a, self.build_b):
            make_executable(build, '#!/bin/sh\nprintf "mutated"\n')
        make_executable(self.build_na, '#!/bin/sh\necho "input outside the API domain: x" >&2\nexit 2\n')
        self.build_q = os.path.join(root, "build-q")
        make_executable(self.build_q, '#!/bin/sh\nprintf "NUM-SHORT x -> ok 2.0E23"\n')    # differs only like the JDK 17 Double.toString bug
        self.build_control = os.path.join(root, "build-control")
        make_executable(self.build_control, '#!/bin/sh\nprintf "pekko"\n')                  # the unmutated build: matches the recording
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

    def test_reading_a_recorded_case_leaves_no_file_open(self):
        import gc, warnings
        with warnings.catch_warnings(record=True) as caught:
            warnings.simplefilter("always", ResourceWarning)
            replay_lib.case_outputs(self.build_a, "c-a1")
            replay_lib.rejects_explicitly(self.build_na, "c-na")
            gc.collect()
        self.assertEqual([str(warning.message) for warning in caught if issubclass(warning.category, ResourceWarning)], [])

    # --- fast-kill must demonstrate what the full replay would classify as an applicable divergence ---
    def test_a_divergence_is_confirmed_when_the_real_single_case_replay_fails_it_as_an_applicable_case(self):
        confirmation = replay_lib.confirm_applicable_divergence(self.build_a, "c-a1")
        self.assertTrue(confirmation.confirmed, confirmation.reason)
        self.assertIn("### c-a1", confirmation.report_text)

    def test_a_divergence_the_single_case_replay_does_not_confirm_is_not_a_kill(self):
        confirmation = replay_lib.confirm_applicable_divergence(self.build_b, "c-a1")     # the tool passes this case
        self.assertFalse(confirmation.confirmed)
        self.assertIn("passes", confirmation.reason)

    def test_a_case_listed_as_not_applicable_is_never_a_kill(self):
        with open(replay_lib.NA_FILE, "a") as na_file:
            na_file.write("c-a1\tout of domain\tQ-X-17\tdocs\n")
        confirmation = replay_lib.confirm_applicable_divergence(self.build_a, "c-a1")
        self.assertFalse(confirmation.confirmed)
        self.assertIn("not applicable", confirmation.reason)

    def test_a_difference_that_is_only_the_jdk_17_number_signature_is_not_an_applicable_divergence(self):
        confirmation = replay_lib.confirm_applicable_divergence(self.build_q, "c-q")
        self.assertFalse(confirmation.confirmed)
        self.assertIn("Q-CFG-30", confirmation.reason)

    def test_an_invalid_single_case_output_is_not_a_kill(self):
        make_executable(self.tool, '#!/bin/sh\necho \'{"passed":0,"total":1}\'\nexit 1\n')
        confirmation = replay_lib.confirm_applicable_divergence(self.build_a, "c-a1")
        self.assertFalse(confirmation.confirmed)
        self.assertIn("rejected", confirmation.reason)

    def test_fast_kill_returns_a_confirmed_case_and_otherwise_falls_back_to_the_full_replay(self):
        import check_divergence as cd
        saved = (cd.known_killers, cd.control_binary_for_fast_kill)
        cd.known_killers = lambda name: ["c-a1"]
        cd.control_binary_for_fast_kill = lambda: self.build_control
        try:
            kill = cd.fast_kill(self.build_a, "some-mutant")
            self.assertEqual(kill.case, "c-a1")
            self.assertIn("### c-a1", kill.report_text)
            self.assertIsNone(cd.fast_kill(self.build_b, "some-mutant"))   # build-b agrees with the tool: nothing to confirm, the full replay decides
        finally:
            cd.known_killers, cd.control_binary_for_fast_kill = saved

    def test_every_verdict_report_says_which_path_produced_the_verdict(self):
        import check_divergence as cd
        reports = os.path.join(self.tmp.name, "divergence-reports")
        os.makedirs(reports)
        saved = (cd.REPORTS, cd.replay)
        cd.REPORTS = reports
        cd.replay = lambda binary: replay_lib.run_replay(binary, 30)
        try:
            cd.full_replay_verdict(self.build_a, "full-path-mutant")
            cd.publish_fast_kill_report("fast-path-mutant", cd.FastKill("c-a1", "### c-a1\n", "the unmutated build matches Pekko"))
        finally:
            cd.REPORTS, cd.replay = saved
        full = read_text(os.path.join(reports, "replay-report-full-path-mutant.md"))
        fast = read_text(os.path.join(reports, "replay-report-fast-path-mutant.md"))
        self.assertTrue(full.startswith("Verdict path: full replay (3 cases)"), full[:80])
        self.assertTrue(fast.startswith("Verdict path: fast-kill"), fast[:80])
        self.assertIn("c-a1", fast.splitlines()[0])
        self.assertIn("no full replay", fast.splitlines()[0])

    # --- reports: the killers list survives a fast-kill, every verdict rewrites its report ---
    def with_reports_directory(self):
        import check_divergence as cd
        reports = os.path.join(self.tmp.name, "divergence-reports")
        os.makedirs(reports, exist_ok=True)
        self.saved_cd = (cd.REPORTS, cd.replay)
        cd.REPORTS = reports
        cd.replay = lambda binary: replay_lib.run_replay(binary, 30)
        self.addCleanup(lambda: (setattr(cd, "REPORTS", self.saved_cd[0]), setattr(cd, "replay", self.saved_cd[1])))
        return cd, reports

    def test_a_fast_kill_keeps_the_full_list_of_known_killers_instead_of_shrinking_it_to_the_confirmed_case(self):
        cd, reports = self.with_reports_directory()
        with open(os.path.join(reports, "replay-report-m.md"), "w") as earlier:
            earlier.write("# App replay\n\nPassed: 2/5\n\n### c-a1\n\n### c-a2\n\n### c-q\n")
        cd.publish_fast_kill_report("m", cd.FastKill("c-a1", "### c-a1\n", "the unmutated build matches Pekko on it"))
        self.assertEqual(cd.known_killers("m"), ["c-a1", "c-a2", "c-q"])               # not just the one confirmed case
        self.assertEqual(read_text(os.path.join(reports, "replay-report-m.md")).count("### "), 1)   # the report is the verdict's own evidence

    def test_a_published_report_has_no_trailing_whitespace_though_the_quoted_diff_has_blank_lines(self):
        cd, reports = self.with_reports_directory()
        cd.publish_mutant_report("blank", "full replay", "### c-a1\n```diff\n+ \n+ text  \n```\n")
        published = read_text(os.path.join(reports, "replay-report-blank.md"))
        self.assertEqual([line for line in published.split("\n") if line != line.rstrip()], [])
        self.assertIn("+\n+ text\n", published)

    def test_a_full_replay_records_every_divergent_case_as_a_known_killer_for_the_next_fast_kill(self):
        cd, reports = self.with_reports_directory()
        cd.full_replay_verdict(self.build_a, "m2")
        self.assertEqual(cd.known_killers("m2"), ["c-a1", "c-a2"])

    def test_a_mutant_that_survives_gets_a_report_that_states_its_verdict_and_not_the_old_divergences(self):
        cd, reports = self.with_reports_directory()
        with open(os.path.join(reports, "replay-report-m3.md"), "w") as earlier:
            earlier.write("# App replay\n\nPassed: 1/3\n\n### c-a1\n\n### c-a2\n")
        outcome = cd.full_replay_verdict(self.build_b, "m3")                              # the replay passes everything: the mutant survives
        self.assertEqual(outcome[0], cd.UNDETECTED)
        report = read_text(os.path.join(reports, "replay-report-m3.md"))
        self.assertTrue(report.startswith("Verdict path: full replay (2 cases); outcome UNDETECTED"), report[:90])
        self.assertNotIn("### c-a1", report)
        self.assertEqual(cd.known_killers("m3"), [])

    def test_a_replay_that_produced_no_report_still_states_its_verdict(self):
        cd, reports = self.with_reports_directory()
        make_executable(self.tool, PASSING_JSON_ONLY)
        with open(os.path.join(reports, "replay-report-m4.md"), "w") as earlier:
            earlier.write("# App replay\n\n### stale\n")
        cd.full_replay_verdict(self.build_b, "m4")
        report = read_text(os.path.join(reports, "replay-report-m4.md"))
        self.assertTrue(report.startswith("Verdict path: full replay (1 cases); outcome UNDETECTED"), report[:90])
        self.assertIn("produced no report", report)
        self.assertNotIn("stale", report)

    def test_a_rejected_replay_output_is_also_stated_in_the_report(self):
        cd, reports = self.with_reports_directory()
        make_executable(self.tool, '#!/bin/sh\necho \'{"passed":1,"total":3}\'\nexit 1\n')
        cd.full_replay_verdict(self.build_b, "m5")
        report = read_text(os.path.join(reports, "replay-report-m5.md"))
        self.assertTrue(report.startswith("Verdict path: full replay (output rejected); outcome HARNESS_ERROR"), report[:100])

    # --- a binary path with whitespace cannot be passed to `--build` ---
    def test_a_binary_path_with_whitespace_is_rejected_clearly_before_the_tool_runs(self):
        called = os.path.join(self.tmp.name, "tool-was-called")
        make_executable(self.tool, f'#!/bin/sh\ntouch "{called}"\n' + PASSING_JSON_ONLY.split("\n", 1)[1])
        result = replay_lib.run_replay("/Users/some one/rakka/classic-parity", 5)
        self.assertIsNone(result.passed)
        self.assertIn("whitespace", result.problem)
        self.assertFalse(os.path.exists(called))                                   # `--build` would have split the path: the tool never ran
        verdict = replay_lib.evaluate("/Users/some one/rakka/classic-parity", result)
        self.assertIsNone(verdict["ok"])
        self.assertIn("whitespace", verdict["line"])

    def test_a_single_case_confirmation_with_such_a_path_is_not_a_kill(self):
        confirmation = replay_lib.confirm_applicable_divergence("/Users/some one/rakka/classic-parity", "c-a1")
        self.assertFalse(confirmation.confirmed)
        self.assertIn("whitespace", confirmation.reason)

    # --- waiting for the lock consumes the run's budget ---
    def hold_the_replay_lock(self):
        import fcntl
        os.makedirs(os.path.join(self.tmp.name, ".gates"), exist_ok=True)
        holder = open(os.path.join(self.tmp.name, ".gates", "replay.lock"), "w")
        fcntl.flock(holder, fcntl.LOCK_EX)
        return holder

    def test_waiting_for_the_replay_lock_gives_up_when_the_run_budget_is_spent(self):
        holder = self.hold_the_replay_lock()               # another replay of this checkout is running
        os.environ[replay_lib.REPLAY_DEADLINE_ENV] = str(time.time() + 0.6)
        try:
            started = time.time()
            with self.assertRaises(replay_lib.ReplayLockTimeout):
                replay_lib.run_replay(self.build_b, 30)
            self.assertLess(time.time() - started, 5)       # it waited for the budget, not for the other replay
        finally:
            del os.environ[replay_lib.REPLAY_DEADLINE_ENV]
            holder.close()

    def test_after_a_give_up_the_lock_is_still_usable_and_no_replay_ran(self):
        holder = self.hold_the_replay_lock()
        os.environ[replay_lib.REPLAY_DEADLINE_ENV] = str(time.time() + 0.3)
        try:
            with self.assertRaises(replay_lib.ReplayLockTimeout):
                replay_lib.run_replay(self.build_b, 30)
        finally:
            del os.environ[replay_lib.REPLAY_DEADLINE_ENV]
            holder.close()
        self.assertFalse(os.path.exists(replay_lib.REPORT))            # the replay never started
        self.assertEqual(replay_lib.run_replay(self.build_b, 30).passed, 2)

    def test_a_spent_budget_does_not_stop_a_replay_that_finds_the_lock_free(self):
        os.environ[replay_lib.REPLAY_DEADLINE_ENV] = str(time.time() - 100)
        try:
            self.assertEqual(replay_lib.run_replay(self.build_b, 30).passed, 2)
        finally:
            del os.environ[replay_lib.REPLAY_DEADLINE_ENV]

    def test_the_budget_exit_code_is_not_one_of_the_codes_the_tools_use_for_verdicts(self):
        self.assertNotIn(replay_lib.EXIT_BUDGET_EXHAUSTED, (0, 1, 2, 3, 143))

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
