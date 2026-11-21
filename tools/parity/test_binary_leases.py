"""A published binary copy that a replay still uses must never be pruned. Every copy is protected from the moment it is published (or acquired) until
its last use ends, across processes, by a lease; publication, acquisition and pruning are serialised so that there is no window between obtaining
the path and protecting it. The limit of copies applies to the ones nobody uses."""
import os, signal, stat, subprocess, sys, tempfile, threading, time, unittest

import check_divergence as cd
import replay_lib

HERE = os.path.dirname(os.path.abspath(__file__))


def write_executable(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as handle:
        handle.write(text)
    os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR)


class LeasedCopies(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.store = os.path.join(self.tmp.name, "binaries")
        self.source = os.path.join(self.tmp.name, "target", "classic-parity")

    def tearDown(self):
        self.tmp.cleanup()

    def publish(self, version, keep=20, **options):
        write_executable(self.source, f"#!/bin/sh\necho v{version}\n")
        return replay_lib.publish_binary(self.source, self.store, keep=keep, **options)

    def publish_and_release(self, version):
        replay_lib.release_lease(self.publish(version))

    def copies(self):
        return sorted(name for name in os.listdir(self.store) if len(name) == 64)

    def output_of(self, path):
        return subprocess.run([str(path)], capture_output=True, text=True).stdout.strip()

    # --- the copy in use survives; the limit counts the unused ones ---
    def test_a_copy_in_use_survives_more_than_twenty_other_publications_and_is_prunable_again_after_its_last_consumer(self):
        held = self.publish(0)
        for version in range(1, 26):
            self.publish_and_release(version)
        self.assertTrue(os.path.exists(held), "a copy in use was pruned")
        self.assertEqual(self.output_of(held), "v0")                    # and it can still be executed
        self.assertEqual(len(self.copies()), 21)                        # the one in use + the 20 newest unused ones
        replay_lib.release_lease(held)
        self.publish_and_release(26)
        self.assertFalse(os.path.exists(held))                          # eligible again once its last consumer finished
        self.assertEqual(len(self.copies()), 20)

    def test_two_consumers_share_one_copy_and_it_stays_protected_until_the_last_one_finishes(self):
        first, second = self.publish(0), self.publish(0)
        self.assertEqual(str(first), str(second))
        for version in range(1, 26):
            self.publish_and_release(version)
        self.assertTrue(os.path.exists(first))
        replay_lib.release_lease(first)
        self.publish_and_release(26)
        self.assertTrue(os.path.exists(second), "the second consumer still uses it")
        replay_lib.release_lease(second)
        self.publish_and_release(27)
        self.assertFalse(os.path.exists(second))

    def test_the_protection_is_released_on_error(self):
        with self.assertRaises(RuntimeError):
            with replay_lib.holding(self.publish(0)) as held:
                for version in range(1, 26):
                    self.publish_and_release(version)
                self.assertTrue(os.path.exists(held))
                raise RuntimeError("the replay failed")
        self.publish_and_release(26)
        self.assertEqual(len(self.copies()), 20)                        # the failed run's copy is an ordinary unused copy again

    def test_releasing_twice_is_harmless(self):
        lease = self.publish(0)
        replay_lib.release_lease(lease)
        replay_lib.release_lease(lease)
        self.assertTrue(os.path.exists(lease))

    # --- across processes ---
    def start_holder(self, version, terminate_handler=False):
        write_executable(self.source, f"#!/bin/sh\necho v{version}\n")
        handler = "signal.signal(signal.SIGTERM, lambda *_: sys.exit(143));" if terminate_handler else ""
        script = (f"import signal, sys; sys.path.insert(0, {HERE!r}); import replay_lib; {handler}"
                  f"lease = replay_lib.publish_binary({self.source!r}, {self.store!r});"
                  "print(lease, flush=True);"
                  "with_block = replay_lib.holding(lease); with_block.__enter__(); sys.stdin.read()")
        process = subprocess.Popen([sys.executable, "-c", script], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        self.addCleanup(process.stdout.close)             # no pipe of the stand-in consumer outlives the test
        self.addCleanup(process.stdin.close)
        return process, process.stdout.readline().strip()

    def test_a_holder_in_another_process_protects_the_copy(self):
        holder, path = self.start_holder(0)
        try:
            for version in range(1, 26):
                self.publish_and_release(version)
            self.assertTrue(os.path.exists(path), "a copy another process still uses was pruned")
            self.assertEqual(self.output_of(path), "v0")
        finally:
            holder.stdin.close()
            holder.wait(timeout=20)
        self.publish_and_release(26)
        self.assertFalse(os.path.exists(path))                           # its consumer ended normally

    def test_a_holder_killed_with_sigkill_releases_its_protection(self):
        holder, path = self.start_holder(0)
        for version in range(1, 26):
            self.publish_and_release(version)
        self.assertTrue(os.path.exists(path))
        holder.send_signal(signal.SIGKILL)
        holder.wait(timeout=20)
        self.publish_and_release(26)
        self.assertFalse(os.path.exists(path))                           # the OS released the lock of the dead process

    def test_a_holder_cancelled_with_sigterm_releases_its_protection(self):
        holder, path = self.start_holder(0, terminate_handler=True)
        self.assertTrue(os.path.exists(path))
        holder.send_signal(signal.SIGTERM)
        self.assertEqual(holder.wait(timeout=20), 143)
        for version in range(1, 26):
            self.publish_and_release(version)
        self.assertFalse(os.path.exists(path))

    # --- publication, acquisition and pruning leave no window ---
    def test_a_concurrent_prune_cannot_delete_a_copy_between_obtaining_its_path_and_protecting_it(self):
        pruned = []
        def prune_while_the_path_is_not_yet_protected():
            pruner = threading.Thread(target=lambda: pruned.append(replay_lib.prune_binaries(self.store, 0)))
            pruner.start()
            time.sleep(0.4)           # long enough for an unsynchronised prune to run to completion
            prune_while_the_path_is_not_yet_protected.pruner = pruner
        lease = self.publish(0, after_published=prune_while_the_path_is_not_yet_protected)
        prune_while_the_path_is_not_yet_protected.pruner.join(timeout=20)
        self.assertFalse(prune_while_the_path_is_not_yet_protected.pruner.is_alive())
        self.assertTrue(os.path.exists(lease), "the prune deleted the copy before its path was protected")
        self.assertEqual(self.output_of(lease), "v0")

    def test_pruning_without_any_lease_still_removes_the_old_copies_beyond_the_limit(self):
        for version in range(25):
            self.publish_and_release(version)
        self.assertEqual(len(self.copies()), 20)


class Consumers(unittest.TestCase):
    """Every consumer holds its copy until its last use ends: the replay gate (classification included), the control and fast-kill."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.saved = (cd.ROOT, replay_lib.ROOT, cd.sh, cd._control_binary)
        cd.ROOT = replay_lib.ROOT = self.tmp.name
        cd._control_binary = None
        self.version = [0]
        built = os.path.join(self.tmp.name, "target", cd.PARITY)
        def fake_cargo(command, **options):
            write_executable(built, f"#!/bin/sh\necho control-{self.version[0]}\n")
            return type("Completed", (), {"returncode": 0, "stdout": "", "stderr": ""})()
        cd.sh = fake_cargo
        self.store = os.path.join(self.tmp.name, ".gates", "binaries")
        self.other_source = os.path.join(self.tmp.name, "other", "binary")

    def tearDown(self):
        cd.release_control_binary()
        cd.ROOT, replay_lib.ROOT, cd.sh, cd._control_binary = self.saved
        self.tmp.cleanup()

    def publish_others(self, count, start=100):
        for version in range(start, start + count):
            write_executable(self.other_source, f"#!/bin/sh\necho other-{version}\n")
            replay_lib.release_lease(replay_lib.publish_binary(self.other_source))

    def test_the_control_and_fast_kill_use_a_protected_copy_until_the_process_releases_it(self):
        control = cd.control_binary_for_fast_kill()
        self.publish_others(25)
        self.assertTrue(os.path.exists(control), "the control copy was pruned while the process still uses it")
        self.assertEqual(subprocess.run([str(control)], capture_output=True, text=True).stdout.strip(), "control-0")
        cd.release_control_binary()
        self.publish_others(25, start=200)
        self.assertFalse(os.path.exists(control))

    def test_the_replay_script_holds_its_copy_through_the_replay_and_the_classification_and_releases_it_on_error(self):
        import replay
        events = []
        class RecordingPath(str):
            def release(self_inner):
                events.append("release")
        saved = (replay.lib.publish_binary, replay.lib.run_replay, replay.lib.evaluate, replay.lib.build_lock, replay.os.path.isfile)
        replay.lib.publish_binary = lambda path: events.append("publish") or RecordingPath("/copy")
        replay.lib.run_replay = lambda binary, timeout: events.append("replay") or object()
        replay.lib.evaluate = lambda binary, result: events.append("classify") or {"ok": True, "line": "ok", "passed": 1, "na_q_cfg_30": [],
                                                                                    "other_na": 0, "applicable_failing": [], "na_problems": []}
        replay.os.path.isfile = lambda path: True
        try:
            self.assertEqual(replay.main(["/some/binary"]), 0)
            self.assertEqual(events, ["publish", "replay", "classify", "release"])
            del events[:]
            replay.lib.run_replay = lambda binary, timeout: events.append("replay") or (_ for _ in ()).throw(RuntimeError("tool crashed"))
            with self.assertRaises(RuntimeError):
                replay.main(["/some/binary"])
            self.assertEqual(events, ["publish", "replay", "release"])      # released on error too
        finally:
            replay.lib.publish_binary, replay.lib.run_replay, replay.lib.evaluate, replay.lib.build_lock, replay.os.path.isfile = saved


if __name__ == "__main__":
    unittest.main()
