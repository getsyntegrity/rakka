"""check.py needs the `upstream/pekko` submodule at the pinned commit: a missing or moved one is said once, with the command to
run, instead of one failure per citation and attribution (issue #273)."""
import os, shutil, subprocess, sys, tempfile, unittest

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import upstream_submodule
from package_sandbox import REPO, PackageSandbox

INIT_COMMAND = "git submodule update --init upstream/pekko"


def git(folder, *arguments):
    settings = ["-c", "user.name=test", "-c", "user.email=test@example.invalid", "-c", "commit.gpgsign=false"]
    return subprocess.run(["git", "-C", folder, *settings, *arguments], check=True, capture_output=True, text=True).stdout.strip()


class SubmodulePrerequisite(unittest.TestCase):
    """A throwaway repository whose HEAD records a gitlink at upstream/pekko."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="rakka-upstream-")
        self.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        self.checkout = os.path.join(self.root, "upstream", "pekko")
        os.makedirs(self.checkout)
        git(self.root, "init", "-q")
        git(self.checkout, "init", "-q")
        git(self.checkout, "commit", "-q", "--allow-empty", "-m", "pinned")
        self.pinned = git(self.checkout, "rev-parse", "HEAD")
        git(self.root, "update-index", "--add", "--cacheinfo", f"160000,{self.pinned},upstream/pekko")
        git(self.root, "commit", "-q", "-m", "gitlink")

    def problem(self, approved=None):
        return upstream_submodule.problem(self.root, approved=approved or self.pinned)

    def test_the_pinned_checkout_is_accepted(self):
        self.assertIsNone(self.problem())

    def test_an_uninitialized_submodule_names_the_command(self):
        shutil.rmtree(self.checkout)
        os.makedirs(self.checkout)  # the empty directory `git worktree add` leaves: git would answer for the outer repository
        self.assertIn("is not initialized", self.problem())
        self.assertIn(INIT_COMMAND, self.problem())

    def test_a_checkout_other_than_the_gitlink_is_reported_with_both_commits(self):
        git(self.checkout, "commit", "-q", "--allow-empty", "-m", "moved")
        message = self.problem()
        self.assertIn(git(self.checkout, "rev-parse", "HEAD"), message)
        self.assertIn(self.pinned, message)
        self.assertIn(INIT_COMMAND, message)

    def test_a_gitlink_other_than_the_approved_commit_is_reported(self):
        message = self.problem(approved="0" * 40)
        self.assertIn("approved", message)
        self.assertIn(self.pinned, message)

    def test_the_approved_commit_is_the_documented_one(self):
        with open(os.path.join(REPO, "docs/upstream.md"), encoding="utf-8") as document:
            self.assertIn(f"| Commit | `{upstream_submodule.APPROVED_COMMIT}` |", document.read())


class CheckWithoutTheSubmodule(PackageSandbox):
    def test_one_failure_with_the_command_instead_of_one_per_citation_and_attribution(self):
        check = os.path.join(self.root, "tools/package/check.py")
        result = subprocess.run([sys.executable, check], cwd=self.root, capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertEqual(result.stdout.count(INIT_COMMAND), 1, result.stdout)
        self.assertNotIn("does not exist in upstream/pekko", result.stdout)
        self.assertNotIn("out of range or missing", result.stdout)
        self.assertIn("citation_problems=not-checked", result.stdout)


if __name__ == "__main__":
    unittest.main()
