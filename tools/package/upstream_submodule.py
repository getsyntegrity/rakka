"""The prerequisite of check.py: `upstream/pekko` checked out at the commit HEAD records, which is the approved one."""
import os, subprocess

APPROVED_COMMIT = "9fbd2283b1cf701c90a1223bf621942c191db61f"  # Apache Pekko v2.0.0, docs/upstream.md
INIT_COMMAND = "git submodule update --init upstream/pekko"


def _git(folder, *arguments):
    done = subprocess.run(["git", "-C", folder, *arguments], capture_output=True, text=True)
    return done.stdout.strip() if done.returncode == 0 else ""


def problem(root, approved=APPROVED_COMMIT):
    """Why the submodule of the repository at `root` cannot be used, or None."""
    checkout = os.path.join(root, "upstream", "pekko")
    # without its own `.git`, git would answer for the outer repository
    actual = _git(checkout, "rev-parse", "HEAD") if os.path.exists(os.path.join(checkout, ".git")) else ""
    if not actual:
        return f"upstream/pekko is not initialized; run: {INIT_COMMAND}"
    entry = _git(root, "ls-tree", "HEAD", "upstream/pekko").split()
    pinned = entry[2] if len(entry) > 2 else "none"
    if pinned != approved:
        return f"upstream/pekko: HEAD records the gitlink {pinned}, not the approved commit {approved} (docs/upstream.md)"
    if actual != pinned:
        return f"upstream/pekko is checked out at {actual}, but HEAD records {pinned}; run: {INIT_COMMAND}"
    return None
