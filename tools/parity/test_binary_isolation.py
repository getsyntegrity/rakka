"""The binary a run uses to classify must not be changeable by another run: a build publishes an immutable, content-addressed copy under
a build lock, and every consumer uses that copy, never the shared target/ path that the next build overwrites."""
import fcntl, os, stat, tempfile, threading, time, unittest

import check_divergence as cd
import replay_lib


def write_executable(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as handle:
        handle.write(text)
    os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR)


def read_text(path):
    with open(path) as handle:
        return handle.read()


class PublishedBinaries(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.store = os.path.join(self.tmp.name, "binaries")
        self.source = os.path.join(self.tmp.name, "target", "classic-parity")

    def tearDown(self):
        self.tmp.cleanup()

    def test_a_published_copy_has_the_content_and_the_executable_bit_of_the_build(self):
        write_executable(self.source, "#!/bin/sh\necho v1\n")
        published = replay_lib.publish_binary(self.source, self.store)
        self.assertEqual(read_text(published), "#!/bin/sh\necho v1\n")
        self.assertTrue(os.access(published, os.X_OK))
        self.assertTrue(published.startswith(self.store))

    def test_a_later_build_that_overwrites_the_shared_path_cannot_change_the_copy_another_run_uses(self):
        write_executable(self.source, "#!/bin/sh\necho v1\n")
        first = replay_lib.publish_binary(self.source, self.store)
        write_executable(self.source, "#!/bin/sh\necho v2\n")          # a concurrent build replaces target/
        second = replay_lib.publish_binary(self.source, self.store)
        self.assertNotEqual(first, second)
        self.assertEqual(read_text(first), "#!/bin/sh\necho v1\n")      # the run that holds `first` still sees what it built
        self.assertEqual(read_text(second), "#!/bin/sh\necho v2\n")

    def test_publishing_the_same_content_again_gives_the_same_path_and_leaves_no_partial_file(self):
        write_executable(self.source, "#!/bin/sh\necho v1\n")
        first = replay_lib.publish_binary(self.source, self.store)
        again = replay_lib.publish_binary(self.source, self.store)
        self.assertEqual(first, again)
        leftovers = [name for _directory, _subdirs, names in os.walk(self.store) for name in names if name.endswith(".tmp")]
        self.assertEqual(leftovers, [])

    def test_only_the_newest_published_binaries_are_kept_and_the_one_just_published_is_never_removed(self):
        paths = []
        for version in range(6):
            write_executable(self.source, f"#!/bin/sh\necho v{version}\n")
            paths.append(replay_lib.publish_binary(self.source, self.store, keep=3))
            time.sleep(0.02)
        self.assertTrue(os.path.exists(paths[-1]))
        self.assertEqual(len(os.listdir(self.store)), 3)


class BuildLock(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.saved_root = replay_lib.ROOT
        replay_lib.ROOT = self.tmp.name

    def tearDown(self):
        replay_lib.ROOT = self.saved_root
        self.tmp.cleanup()

    def test_two_builds_of_one_checkout_never_overlap(self):
        order = []
        def build(label, hold):
            with replay_lib.build_lock():
                order.append(f"{label}-start")
                time.sleep(hold)
                order.append(f"{label}-end")
        first = threading.Thread(target=build, args=("a", 0.5))
        first.start()
        time.sleep(0.15)
        second = threading.Thread(target=build, args=("b", 0.0))
        second.start()
        first.join()
        second.join()
        self.assertEqual(order, ["a-start", "a-end", "b-start", "b-end"])


class ControlBinaryUsedForClassification(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.saved = (cd.ROOT, replay_lib.ROOT, cd.sh, cd._control_binary)
        cd.ROOT = replay_lib.ROOT = self.tmp.name
        self.built_version = [1]
        built_path = os.path.join(self.tmp.name, "target", cd.PARITY)
        def fake_cargo(command, **options):
            write_executable(built_path, f"#!/bin/sh\necho build-{self.built_version[0]}\n")
            return type("Completed", (), {"returncode": 0, "stdout": "", "stderr": ""})()
        cd.sh = fake_cargo
        cd._control_binary = None

    def tearDown(self):
        cd.ROOT, replay_lib.ROOT, cd.sh, cd._control_binary = self.saved
        self.tmp.cleanup()

    def test_the_control_binary_is_an_immutable_copy_not_the_shared_target_path(self):
        control = cd.control_binary_for_fast_kill()
        shared_target = os.path.join(self.tmp.name, "target", cd.PARITY)
        self.assertNotEqual(os.path.realpath(control), os.path.realpath(shared_target))
        self.built_version[0] = 2                      # another run rebuilds target/ with different code...
        cd._control_binary = None
        other_run = cd.control_binary_for_fast_kill()
        self.assertNotEqual(control, other_run)
        self.assertIn("build-1", read_text(control))   # ...and the first run keeps classifying with the binary it built
        self.assertIn("build-2", read_text(other_run))

    def test_building_and_publishing_happen_under_the_build_lock(self):
        held_during_build = []
        original = cd.sh
        def build_and_probe(command, **options):
            probe = open(os.path.join(self.tmp.name, ".gates", "build.lock"), "w")
            try:
                fcntl.flock(probe, fcntl.LOCK_EX | fcntl.LOCK_NB)
                held_during_build.append(False)
            except BlockingIOError:
                held_during_build.append(True)
            finally:
                probe.close()
            return original(command, **options)
        cd.sh = build_and_probe
        cd.control_binary_for_fast_kill()
        self.assertEqual(held_during_build, [True])


if __name__ == "__main__":
    unittest.main()
