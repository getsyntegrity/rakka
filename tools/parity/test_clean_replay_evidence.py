"""The clean replay of a run is authoritative only for the same identity (binary, build and corpus inputs, oracle, tooling, toolchain, environment) and the same run."""
import os, pathlib, shutil, tempfile, unittest

import clean_replay_evidence as evidence

TOOLS = {"rustc": "rustc 1.100.0", "cargo": "cargo 1.100.0", "rust-anything": "rust-anything 9.9", "python": "3.12.1", "platform": "darwin-arm64"}
FILES = {"crates/rakka-actor/src/lib.rs": "fn a() {}", "Cargo.toml": "[workspace]", "Cargo.lock": "lock", "rust/app-cases/c1.toml": "args=[]", "rust/app-cases/c1.stdout": "out",
         "rust/app-cases/rust-anything.json": "{}", "rust/app-cases/replay-report.md": "generated 1", "cases/not-applicable.tsv": "x", "oracle/src/Main.java": "class A {}",
         "tools/parity/replay.py": "r", "tools/parity/replay_lib.py": "l", "tools/parity/na_q_cfg_30.py": "n", "tools/parity/clean_replay_evidence.py": "e", "binary": "BINARY-BYTES"}
COMPLETE = dict(ok=True, na_problems=[], passed=3, total=3, other_na=1)


class Identity(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.root, True)
        for relative, text in FILES.items():
            self.write(relative, text)

    def write(self, relative, text, root=None):
        path = pathlib.Path(root or self.root, relative)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def key(self, tools=None, environment=None, command=("cargo", "build"), timeout=900):
        return evidence.key_of(evidence.identity(self.root, os.path.join(self.root, "binary"), tools or TOOLS, environment or {}, command, timeout))

    def test_every_relevant_input_changes_the_key_and_restoring_it_restores_the_key(self):
        base = self.key()
        for relative in ("crates/rakka-actor/src/lib.rs", "Cargo.lock", "rust/app-cases/c1.stdout", "rust/app-cases/rust-anything.json", "cases/not-applicable.tsv",
                         "oracle/src/Main.java", "tools/parity/replay_lib.py", "tools/parity/na_q_cfg_30.py", "binary"):
            self.write(relative, FILES[relative] + " changed")
            self.assertNotEqual(self.key(), base, relative)
            self.write(relative, FILES[relative])
        self.assertEqual(self.key(), base)
        self.write("rust/app-cases/c2.toml", "args=[]")
        self.assertNotEqual(self.key(), base)

    def test_toolchain_environment_and_configuration_change_the_key_but_the_run_id_and_generated_or_build_files_do_not(self):
        base = self.key()
        for changed in (self.key(tools={**TOOLS, "rustc": "rustc 1.101.0"}), self.key(tools={**TOOLS, "rust-anything": "rust-anything 9.10"}),
                        self.key(environment={"RUSTFLAGS": "-C opt-level=0"}), self.key(command=("cargo", "build", "--features", "x")), self.key(timeout=901)):
            self.assertNotEqual(changed, base)
        self.write("rust/app-cases/replay-report.md", "generated 2")
        self.write("crates/rakka-actor/target/debug/artifact", "build output")
        self.assertEqual(self.key(environment={"PATH": "/x", "RAKKA_GATE_RUN_ID": "run-1"}), base)

    def test_a_checkout_under_a_target_directory_is_fully_hashed_and_an_unknown_tool_or_binary_makes_the_identity_unusable(self):
        inside = os.path.join(self.root, "target", "checkout")
        self.write("crates/lib.rs", "v1", inside)
        first = evidence.digest_paths(inside, ("crates",))
        self.write("crates/lib.rs", "v2", inside)
        self.assertNotEqual(evidence.digest_paths(inside, ("crates",)), first)
        self.assertIsNone(evidence.identity(self.root, os.path.join(self.root, "binary"), {**TOOLS, "rustc": None}, {}, ("cargo",), 900))
        self.assertIsNone(evidence.identity(self.root, os.path.join(self.root, "missing"), TOOLS, {}, ("cargo",), 900))

    def test_only_a_complete_successful_replay_is_evidence(self):
        self.assertIsNone(evidence.refusal(COMPLETE, replayed_cases=4, corpus_cases=4))
        for evaluation in (dict(COMPLETE, ok=None), dict(COMPLETE, ok=False), dict(COMPLETE, na_problems=["c: x"]), dict(COMPLETE, passed=2)):
            self.assertIsNotNone(evidence.refusal(evaluation, replayed_cases=4, corpus_cases=4), evaluation)
        for replayed in (3, None):
            self.assertIsNotNone(evidence.refusal(COMPLETE, replayed_cases=replayed, corpus_cases=4), replayed)


class Storage(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.directory, True)
        self.path, binary = os.path.join(self.directory, "evidence", "clean-replay.json"), os.path.join(self.directory, "b")
        pathlib.Path(binary).write_text("x")
        self.identity = evidence.identity(self.directory, binary, TOOLS, {}, ("cargo",), 900)
        self.other = evidence.identity(self.directory, binary, {**TOOLS, "rustc": "rustc 2"}, {}, ("cargo",), 900)

    def stored(self, **changes):
        evidence.store(self.path, {**evidence.record(self.identity, COMPLETE, replayed_cases=4, run_id="run-1"), **changes})

    def reusable(self, identity="same", run_id="run-1", corpus_cases=4):
        return evidence.reusable(self.path, self.identity if identity == "same" else identity, run_id, corpus_cases)[0]

    def test_a_record_is_reusable_only_in_its_own_run_for_the_same_identity_and_corpus_and_leaves_no_partial_file(self):
        self.stored()
        self.assertEqual(self.reusable()["result"]["applicable_passed"], 3)
        self.assertEqual(os.listdir(os.path.dirname(self.path)), ["clean-replay.json"])
        for refused in (self.reusable(run_id="run-2"), self.reusable(run_id=None), self.reusable(identity=self.other), self.reusable(identity=None), self.reusable(corpus_cases=5)):
            self.assertIsNone(refused)

    def test_missing_corrupt_foreign_or_tampered_records_are_never_reusable_and_discard_is_idempotent(self):
        self.assertIsNone(self.reusable())
        os.makedirs(os.path.dirname(self.path))
        with open(self.path, "w") as handle:
            handle.write("{not json")
        self.assertIsNone(self.reusable())
        for change in ({"schema": 99}, {"key": "tampered"}):
            self.stored(**change)
            self.assertIsNone(self.reusable(), change)
        evidence.discard(self.path)
        evidence.discard(self.path)
        self.assertFalse(os.path.exists(self.path))


if __name__ == "__main__":
    unittest.main()
