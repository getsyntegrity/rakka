"""The clean replay of a run is authoritative only for the same identity (binary, build and corpus inputs, oracle, tooling, toolchain, environment) and the same run."""
import os, pathlib, shutil, tempfile, unittest
from unittest import mock

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
        return evidence.key_of(evidence.identity(self.root, os.path.join(self.root, "binary"), environment or {}, tools or TOOLS, command, timeout))

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
        for name in ("CARGO_HOME", "RUSTC_WRAPPER", "RUSTFLAGS", "RUSTDOCFLAGS", "RUST_BACKTRACE", "LLVM_PROFILE_FILE", "CC", "CXX", "CFLAGS", "LDFLAGS", "SDKROOT", "MACOSX_DEPLOYMENT_TARGET",
                     "DYLD_LIBRARY_PATH", "LD_LIBRARY_PATH"):
            self.assertNotEqual(self.key(environment={name: "1"}), base, name)

    def test_a_checkout_under_a_target_directory_is_fully_hashed_and_an_unknown_tool_or_binary_makes_the_identity_unusable(self):
        inside = os.path.join(self.root, "target", "checkout")
        self.write("crates/lib.rs", "v1", inside)
        first = evidence.digest_paths(inside, ("crates",))
        self.write("crates/lib.rs", "v2", inside)
        self.assertNotEqual(evidence.digest_paths(inside, ("crates",)), first)
        self.assertIsNone(evidence.identity(self.root, os.path.join(self.root, "binary"), {}, {**TOOLS, "rustc": None}, ("cargo",), 900))
        self.assertIsNone(evidence.identity(self.root, os.path.join(self.root, "missing"), {}, TOOLS, ("cargo",), 900))
        os.chmod(os.path.join(self.root, "binary"), 0)          # present but unreadable: unknown, not a crash
        self.assertIsNone(evidence.identity(self.root, os.path.join(self.root, "binary"), {}, TOOLS, ("cargo",), 900))
        os.chmod(os.path.join(self.root, "binary"), 0o644)
        os.symlink("/nonexistent/target", os.path.join(self.root, "crates/dangling-editor-lock"))     # a dangling file in the tree, an unreadable corpus file: unknown too
        self.write("rust/app-cases/unreadable.toml", "x")
        os.chmod(os.path.join(self.root, "rust/app-cases/unreadable.toml"), 0)
        self.assertIsNone(evidence.identity(self.root, os.path.join(self.root, "binary"), {}, TOOLS, ("cargo",), 900))

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
        self.identity = evidence.identity(self.directory, binary, {}, TOOLS, ("cargo",), 900)
        self.other = evidence.identity(self.directory, binary, {}, {**TOOLS, "rustc": "rustc 2"}, ("cargo",), 900)

    def stored(self, evaluation=COMPLETE, **changes):
        evidence.store(self.path, {**evidence.record(self.identity, evaluation, replayed_cases=4, run_id="run-1"), **changes})

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
        altered = {"applicable_passed": 9, "applicable_total": 9, "other_not_applicable": 1, "replayed_cases": 4}      # self-consistent, but not what the replay produced
        for change in ({"schema": 99}, {"key": "tampered"}, {"result": altered}, {"result": {}}):
            self.stored(**change)
            self.assertIsNone(self.reusable(), change)
        for shape in ("[1, 2]", '{"schema": 1, "run_id": "run-1", "result": "text"}'):
            with open(self.path, "w") as handle:
                handle.write(shape)
            self.assertIsNone(self.reusable(), shape)
        self.stored(evaluation=dict(COMPLETE, passed=2))          # a record whose own result is not a clean replay, even with a key that matches it
        self.assertIsNone(self.reusable())
        evidence.discard(self.path)
        evidence.discard(self.path)
        self.assertFalse(os.path.exists(self.path))
        os.makedirs(self.path)
        evidence.discard(self.path)         # a path that cannot be removed is not a failure of the gate that tried


class EvidenceNeverRaises(unittest.TestCase):
    """Evidence is an optimisation: whatever goes wrong inside a step (not only I/O) is its failure value, never an exception for the gate that called it."""
    def test_a_surprise_inside_any_step_is_an_unknown_identity_not_stored_not_reusable_or_nothing_never_an_exception(self):
        surprise = UnicodeEncodeError("utf-8", "name", 0, 1, "surrogates not allowed")      # e.g. a file name or a tool output that is not valid text
        with tempfile.TemporaryDirectory() as root:
            binary = pathlib.Path(root, "binary")
            binary.write_text("x")
            with mock.patch.object(evidence, "digest_paths", side_effect=surprise):
                self.assertIsNone(evidence.identity(root, str(binary), {}, TOOLS))
            with mock.patch.object(evidence, "current_tool_versions", side_effect=surprise):
                self.assertIsNone(evidence.identity(root, str(binary), {}))
            with mock.patch.object(evidence, "refusal", side_effect=KeyError("other_na")):
                self.assertIn("failed", evidence.leave_evidence(root, str(binary), {}, 1, COMPLETE, {evidence.RUN_ID_ENV: "run-1"}))
            with mock.patch.object(evidence, "key_of", side_effect=TypeError("not serialisable")):
                self.assertIsNone(evidence.reusable(os.path.join(root, "none.json"), {"binary": "x"}, "run-1", 1)[0])
            with mock.patch.object(evidence.os, "unlink", side_effect=RuntimeError("surprise")):
                evidence.discard(os.path.join(root, "none.json"))
            with mock.patch.object(evidence.glob, "glob", side_effect=RuntimeError("surprise")):
                self.assertEqual(evidence.corpus_case_count(root), -1)

    def test_a_record_that_cannot_be_serialised_leaves_no_temporary_file_behind(self):
        with tempfile.TemporaryDirectory() as root:
            path = os.path.join(root, "evidence", "clean-replay.json")
            with self.assertRaises(TypeError):
                evidence.store(path, {"result": object()})
            self.assertEqual(os.listdir(os.path.dirname(path)), [])


if __name__ == "__main__":
    unittest.main()
