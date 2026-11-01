"""Shared fixture: a temporary copy of the documentation package in which the generator can be run.

The real repository is never modified by tests that use it.
"""
import hashlib, os, shutil, subprocess, sys, tempfile, unittest

REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
OWNER_FILE = "docs/decisions/owner-decisions.tsv"


def tree_digest(root):
    digest = hashlib.sha256()
    for folder, subfolders, files in os.walk(root):
        subfolders.sort()
        for name in sorted(files):
            path = os.path.join(folder, name)
            digest.update(os.path.relpath(path, root).encode())
            digest.update(open(path, "rb").read())
    return digest.hexdigest()


class PackageSandbox(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="rakka-package-")
        self.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        for entry in ("tools", "docs", "crates", "README.md", "AGENTS.md"):
            source = os.path.join(REPO, entry)
            target = os.path.join(self.root, entry)
            if os.path.isdir(source):
                shutil.copytree(source, target, ignore=shutil.ignore_patterns("__pycache__", "target"))
            else:
                shutil.copy(source, target)
        # Mechanism tests must not depend on the real Owner answers: start from an empty response file.
        owner_path = os.path.join(self.root, OWNER_FILE)
        header = open(owner_path, encoding="utf-8").readline()
        with open(owner_path, "w", encoding="utf-8", newline="") as owner_file:
            owner_file.write(header)

    def regenerate(self):
        env = dict(os.environ, RAKKA_PACKAGE_ROOT=self.root)
        return subprocess.run([sys.executable, "-I", os.path.join(self.root, "tools/package/generate.py")], check=True, env=env, capture_output=True, text=True)

    def read(self, relative):
        return open(os.path.join(self.root, relative), encoding="utf-8").read()

    def write(self, relative, text):
        with open(os.path.join(self.root, relative), "w", encoding="utf-8", newline="") as target:
            target.write(text)

    def append(self, relative, text):
        with open(os.path.join(self.root, relative), "a", encoding="utf-8", newline="") as target:
            target.write(text)
