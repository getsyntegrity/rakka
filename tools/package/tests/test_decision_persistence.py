#!/usr/bin/env python3
"""An Owner response recorded in docs/decisions/owner-decisions.tsv survives regeneration, and regenerating twice is a no-op.

Runs entirely in a temporary copy of the documentation package; the real repository is never modified.
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


class DecisionPersistence(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="rakka-decisions-")
        self.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        for entry in ("tools", "docs", "README.md", "AGENTS.md"):
            source = os.path.join(REPO, entry)
            target = os.path.join(self.root, entry)
            if os.path.isdir(source):
                shutil.copytree(source, target, ignore=shutil.ignore_patterns("__pycache__"))
            else:
                shutil.copy(source, target)

    def regenerate(self):
        env = dict(os.environ, RAKKA_PACKAGE_ROOT=self.root)
        subprocess.run([sys.executable, "-I", os.path.join(self.root, "tools/package/generate.py")], check=True, env=env, capture_output=True)

    def read(self, relative):
        return open(os.path.join(self.root, relative), encoding="utf-8").read()

    def test_accepted_decision_is_kept_and_second_generation_changes_nothing(self):
        self.regenerate()
        with open(os.path.join(self.root, OWNER_FILE), "a", encoding="utf-8") as owner_file:
            owner_file.write("Q-X-10\taccepted\t2030-01-02\ttest-owner\taceptar\n")

        self.regenerate()
        after_first = tree_digest(self.root)
        register = self.read("docs/decisions/register.tsv")
        self.assertIn("Q-X-10\t", register)
        accepted_line = next(line for line in register.splitlines() if line.startswith("Q-X-10\t"))
        self.assertIn("\taccepted\t", accepted_line)
        self.assertTrue(accepted_line.endswith("\t2030-01-02\ttest-owner\taceptar"))
        decisions_doc = self.read("docs/decisions.md")
        self.assertIn("1 aceptadas, 0 aceptadas con cambios, 0 rechazadas", decisions_doc)
        self.assertNotIn("177 propuestas, 0 aceptadas", decisions_doc)
        self.assertIn("(test-owner, 2030-01-02)", decisions_doc)
        self.assertIn("177 propuestas, 1 aceptadas", self.read("README.md"))

        self.regenerate()
        self.assertEqual(after_first, tree_digest(self.root))

    def test_unknown_decision_is_rejected(self):
        with open(os.path.join(self.root, OWNER_FILE), "a", encoding="utf-8") as owner_file:
            owner_file.write("Q-NOPE-1\taccepted\t2030-01-02\ttest-owner\taceptar\n")
        with self.assertRaises(subprocess.CalledProcessError):
            self.regenerate()


if __name__ == "__main__":
    unittest.main()
