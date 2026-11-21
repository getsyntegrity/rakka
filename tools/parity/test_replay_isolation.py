"""Two replays of one checkout must never mix their evidence: the classification and the published report of each replay come from that
replay's own output (the count and failures of its own process, the report read under a per-checkout lock)."""
import os, stat, tempfile, threading, time, unittest

import replay_lib

FAKE_RUST_ANYTHING = r'''#!/bin/sh
# stand-in for `rust-anything app replay REPO --build BIN [--json]`; the result depends on the build's file name
repo="$3"; build=""; json=0
while [ $# -gt 0 ]; do
  case "$1" in --build) build="$2"; shift;; --json) json=1;; esac
  shift
done
label=$(basename "$build")
report="$repo/rust/app-cases/replay-report.md"
if [ "$label" = "build-a" ]; then
  printf '# App replay\n\n## Failing cases\n\n### c-a1\n\n### c-a2\n' > "$report"
  sleep 1.2
  if [ $json = 1 ]; then
    printf '{"build":["%s"],"passed":1,"total":3,"skipped":0,"failures":[{"case":"c-a1"},{"case":"c-a2"}],"fallback":null,"report":"%s"}\n' "$build" "$report"
  else
    echo "Passed: 1/3 cases passed"
  fi
  exit 1
fi
sleep 0.4
printf '# App replay\n\nAll cases passed.\n' > "$report"
if [ $json = 1 ]; then
  printf '{"build":["%s"],"passed":2,"total":2,"skipped":0,"failures":[],"fallback":null,"report":"%s"}\n' "$build" "$report"
else
  echo "Passed: 2/2 cases passed"
fi
exit 0
'''


def make_executable(path, text):
    with open(path, "w") as script:
        script.write(text)
    os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR)


class ReplayIsolation(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = self.tmp.name
        os.makedirs(os.path.join(root, "rust", "app-cases"))
        os.makedirs(os.path.join(root, "cases"))
        os.makedirs(os.path.join(root, "bin"))
        with open(os.path.join(root, "cases", "not-applicable.tsv"), "w") as na_file:
            na_file.write("case\treason\tdecision\tlink\n")
        for case in ("c-a1", "c-a2"):
            base = os.path.join(root, "rust", "app-cases", case)
            with open(base + ".toml", "w") as recording:
                recording.write('bin.name = "x"\nargs = ["scenario"]\nstatus.code = 0\n')
            with open(base + ".stdin", "w") as stdin_file:
                stdin_file.write("input")
            with open(base + ".stdout", "w") as stdout_file:
                stdout_file.write("pekko")
        make_executable(os.path.join(root, "bin", "rust-anything"), FAKE_RUST_ANYTHING)
        self.build_a, self.build_b = (os.path.join(root, name) for name in ("build-a", "build-b"))
        for build in (self.build_a, self.build_b):
            make_executable(build, '#!/bin/sh\nprintf "mutated"\n')
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

    def test_a_missing_report_is_none_not_a_stale_one(self):
        os.remove(os.path.join(self.tmp.name, "bin", "rust-anything"))
        make_executable(os.path.join(self.tmp.name, "bin", "rust-anything"), '#!/bin/sh\necho \'{"passed":1,"total":1,"failures":[]}\'\n')
        with open(replay_lib.REPORT, "w") as stale:
            stale.write("### stale-case\n")
        os.remove(replay_lib.REPORT)
        self.assertIsNone(replay_lib.run_replay(self.build_b, 30).report_text)

    def test_the_lock_is_released_after_a_replay_that_times_out(self):
        make_executable(os.path.join(self.tmp.name, "bin", "rust-anything"), '#!/bin/sh\nsleep 30\n')
        import subprocess
        with self.assertRaises(subprocess.TimeoutExpired):
            replay_lib.run_replay(self.build_b, 1)
        make_executable(os.path.join(self.tmp.name, "bin", "rust-anything"), '#!/bin/sh\necho \'{"passed":1,"total":1,"failures":[]}\'\n')
        self.assertEqual(replay_lib.run_replay(self.build_b, 5).passed, 1)   # would block forever if the lock leaked


if __name__ == "__main__":
    unittest.main()
