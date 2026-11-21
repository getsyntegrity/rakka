"""replay.py: the binary it replays is the one it was told to (or built), copied under the build lock, with clear refusals instead of tracebacks."""
import os, subprocess, sys, unittest

import replay
import replay_lib

SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "replay.py")


class CommandLineRefusals(unittest.TestCase):
    def run_script(self, *arguments):
        return subprocess.run([sys.executable, SCRIPT, *arguments], capture_output=True, text=True, timeout=60)

    def test_build_cannot_be_combined_with_an_explicit_binary(self):
        result = self.run_script("--build", "/some/other/binary")
        self.assertEqual(result.returncode, 2)
        self.assertIn("refused: --build compiles the default parity binary", result.stdout)
        self.assertNotIn("Traceback", result.stderr)

    def test_a_missing_binary_is_a_clear_refusal_not_a_traceback(self):
        result = self.run_script("/nonexistent/directory/classic-parity")
        self.assertEqual(result.returncode, 2)
        self.assertIn("refused: the binary does not exist", result.stdout)
        self.assertNotIn("Traceback", result.stderr)


class BinaryPreparation(unittest.TestCase):
    def setUp(self):
        self.events = []
        self.saved = (replay.lib.build_lock, replay.lib.publish_binary, replay.subprocess.run)
        events = self.events

        class RecordingLock:
            def __enter__(self_inner):
                events.append("lock acquired")
            def __exit__(self_inner, *exc):
                events.append("lock released")
        replay.lib.build_lock = lambda: RecordingLock()
        replay.lib.publish_binary = lambda path: events.append(f"publish {path}") or "/immutable/copy"
        replay.subprocess.run = lambda command, **options: events.append("cargo build") or None

    def tearDown(self):
        replay.lib.build_lock, replay.lib.publish_binary, replay.subprocess.run = self.saved

    def test_the_copy_is_made_under_the_build_lock_also_without_build(self):
        existing = os.path.abspath(__file__)
        self.assertEqual(replay.prepare_binary([existing]), "/immutable/copy")
        self.assertEqual(self.events, ["lock acquired", f"publish {existing}", "lock released"])

    def test_build_compiles_and_publishes_the_default_binary_under_one_lock(self):
        default = os.path.join(replay_lib.ROOT, "target", "release", "examples", "classic-parity")
        saved_isfile = os.path.isfile
        os.path.isfile = lambda path: True if path == default else saved_isfile(path)
        try:
            replay.prepare_binary(["--build"])
        finally:
            os.path.isfile = saved_isfile
        self.assertEqual(self.events, ["lock acquired", "cargo build", f"publish {default}", "lock released"])


if __name__ == "__main__":
    unittest.main()
