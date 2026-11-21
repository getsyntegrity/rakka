"""Tests of the subset replay: which recorded cases a selector picks, how they are staged, and the end-to-end verdict."""
import contextlib
import io
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

import replay_subset as subset

HERE = os.path.dirname(os.path.abspath(__file__))

LONG_NAME = "bad-bare-percent-user-info"  # 26 characters: rust-anything stores it as its first 24


def entry(case, family, rows, basis="cited"):
    return subset.CoverageEntry(case=case, family=family, rows=frozenset(rows), basis=basis)


CATALOG = [
    entry("pa-0001", "oracle-path-address", ["AC-PATH-007", "AC-PATH-011"]),
    entry("pa-0002", "oracle-path-address", ["AC-PATH-006"]),
    entry("ut-0001", "oracle-util-core", ["AC-UTIL-002.1", "AC-UTIL-002.2"]),
    entry("ut-0002", "oracle-util-core", ["AC-UTIL-001"]),
    entry(LONG_NAME, "path-from-string", ["AC-PATH-007"]),
    entry("ok-local-user", "path-from-string", [], basis="none"),
]


class ParsesTheCoverageTable(unittest.TestCase):
    def test_rows_are_split_and_the_dash_means_no_rows(self):
        text = "case\tfamily\trows\tbasis\npa-0001\toracle-path-address\tAC-PATH-007,AC-PATH-011\tcited\nok-x\tpath-from-string\t-\tnone\n"
        with tempfile.NamedTemporaryFile("w", suffix=".tsv", encoding="utf-8", delete=False) as fh:
            fh.write(text)
        self.addCleanup(os.unlink, fh.name)
        self.assertEqual(
            subset.load_coverage(fh.name),
            [entry("pa-0001", "oracle-path-address", ["AC-PATH-007", "AC-PATH-011"]),
             entry("ok-x", "path-from-string", [], basis="none")])


class SelectsCases(unittest.TestCase):
    def test_by_contract_row(self):
        self.assertEqual(subset.select_cases(CATALOG, rows=["AC-PATH-006"]), ["pa-0002"])

    def test_a_row_picks_every_case_that_cites_it_across_families_with_names_as_recorded(self):
        self.assertEqual(subset.select_cases(CATALOG, rows=["AC-PATH-007"]),
                         ["pa-0001", LONG_NAME[:24]])

    def test_a_row_without_suffix_includes_its_dotted_sub_rows_only_at_the_dot(self):
        self.assertEqual(subset.select_cases(CATALOG, rows=["AC-UTIL-002"]), ["ut-0001"])
        with self.assertRaises(subset.SelectionError):
            subset.select_cases(CATALOG, rows=["AC-UTIL-00"])

    def test_by_family(self):
        self.assertEqual(subset.select_cases(CATALOG, families=["oracle-util-core"]), ["ut-0001", "ut-0002"])

    def test_by_name_glob_over_the_recorded_names(self):
        self.assertEqual(subset.select_cases(CATALOG, globs=["pa-*"]), ["pa-0001", "pa-0002"])
        self.assertEqual(subset.select_cases(CATALOG, globs=["bad-bare-percent-user-in"]), [LONG_NAME[:24]])

    def test_selectors_add_up_without_duplicates_and_keep_catalog_order(self):
        self.assertEqual(subset.select_cases(CATALOG, rows=["AC-PATH-006"], families=["oracle-path-address"], globs=["pa-0001"]),
                         ["pa-0001", "pa-0002"])

    def test_no_selector_is_an_error_not_an_empty_pass(self):
        with self.assertRaisesRegex(subset.SelectionError, "no selector"):
            subset.select_cases(CATALOG)

    def test_a_selector_that_matches_nothing_is_named_in_the_error(self):
        with self.assertRaisesRegex(subset.SelectionError, "AC-NOPE-001"):
            subset.select_cases(CATALOG, rows=["AC-PATH-006", "AC-NOPE-001"])
        with self.assertRaisesRegex(subset.SelectionError, "no-such-family"):
            subset.select_cases(CATALOG, families=["no-such-family"])
        with self.assertRaisesRegex(subset.SelectionError, "zz-\\*"):
            subset.select_cases(CATALOG, globs=["zz-*"])


class StagesTheSubset(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.tmp)
        self.source = os.path.join(self.tmp, "source")
        os.makedirs(os.path.join(self.source, "rust", "app-cases"))
        self.cases_dir = os.path.join(self.source, "rust", "app-cases")
        self.write("rust-anything.json", '{"name": "classic-parity"}')
        for name in ("one", "two"):
            for ext, body in (("toml", 'args = ["x"]\n'), ("stdin", "in"), ("stdout", "out"), ("stderr", "")):
                self.write(f"{name}.{ext}", body)
        self.write("replay-report.md", "old report")

    def write(self, name, body):
        with open(os.path.join(self.cases_dir, name), "w", encoding="utf-8") as fh:
            fh.write(body)

    def test_copies_only_the_selected_cases_and_the_app_manifest(self):
        dest = os.path.join(self.tmp, "dest")
        subset.stage_subset(self.source, ["one"], dest)
        self.assertEqual(sorted(os.listdir(os.path.join(dest, "rust", "app-cases"))),
                         ["one.stderr", "one.stdin", "one.stdout", "one.toml", "rust-anything.json"])

    def test_a_selected_case_that_was_not_recorded_is_an_error(self):
        with self.assertRaisesRegex(subset.SelectionError, "ghost"):
            subset.stage_subset(self.source, ["one", "ghost"], os.path.join(self.tmp, "dest"))


def write_text(path, body):
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(body)


STUB_HARNESS = """#!/bin/sh
input=$(cat)
case "$input" in OUT*) echo "input is outside the API domain" >&2; exit 2;; esac
printf '%s' "$input"
"""


@unittest.skipUnless(shutil.which("rust-anything"), "rust-anything is not installed")
class EndToEndThroughRustAnythingReplay(unittest.TestCase):
    """The real `rust-anything app replay` over a staged copy; a stub harness echoes its stdin, so each case's stdin is what the Rust side produces."""

    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.tmp)
        self.root = os.path.join(self.tmp, "src")
        self.cases = os.path.join(self.root, "rust", "app-cases")
        os.makedirs(self.cases)
        os.makedirs(os.path.join(self.root, "cases"))
        write_text(os.path.join(self.cases, "rust-anything.json"),
                   '{"schema": "rust-anything/app-cases@1", "name": "stub", "run": ["true"], "timeout_secs": 10}')
        self.stub = os.path.join(self.tmp, "stub.sh")
        write_text(self.stub, STUB_HARNESS)
        os.chmod(self.stub, os.stat(self.stub).st_mode | stat.S_IXUSR)
        self.coverage_rows = []
        self.not_applicable_rows = []
        self.add_case("good", "hello", "hello", "AC-X-001")
        self.add_case("bad", "hello", "something else", "AC-X-002")

    def add_case(self, name, produced_by_stub, recorded_by_pekko, row):
        write_text(os.path.join(self.cases, f"{name}.toml"), 'bin.name = "stub"\nargs = ["x"]\nstatus.code = 0\nfs.sandbox = true\n')
        write_text(os.path.join(self.cases, f"{name}.stdin"), produced_by_stub)
        write_text(os.path.join(self.cases, f"{name}.stdout"), recorded_by_pekko)
        write_text(os.path.join(self.cases, f"{name}.stderr"), "")
        self.coverage_rows.append(f"{name}\tf\t{row}\tcited\n")

    def mark_not_applicable(self, name):
        self.not_applicable_rows.append(f"{name}\tsource\tout of domain\tQ-X-17\tdocs/x.md\n")

    def run_tool(self, *arguments):
        write_text(os.path.join(self.root, "cases", "coverage.tsv"), "case\tfamily\trows\tbasis\n" + "".join(self.coverage_rows))
        not_applicable_file = os.path.join(self.tmp, "not-applicable.tsv")
        write_text(not_applicable_file, "case\tsource_name\treason\tdecision\tlink\n" + "".join(self.not_applicable_rows))
        out, err = io.StringIO(), io.StringIO()
        with mock.patch.object(subset.lib, "NA_FILE", not_applicable_file), \
                contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = subset.main(["--root", self.root, *arguments])
        return code, out.getvalue(), err.getvalue()

    def test_a_matching_subset_passes_and_reports_how_many_ran(self):
        code, out, err = self.run_tool("--row", "AC-X-001", self.stub)
        self.assertEqual(code, 0, out + err)
        self.assertIn("1/1 selected cases match Pekko", out)
        self.assertIn("SUBSET of 1 cases, not gate evidence", out)

    def test_a_diverging_case_in_the_subset_fails_and_is_named(self):
        code, out, _ = self.run_tool("--family", "f", self.stub)
        self.assertEqual(code, 1)
        self.assertIn("1/2 selected cases match Pekko", out)
        self.assertIn("DIVERGES: bad", out)

    def test_cases_outside_the_selection_are_not_replayed(self):
        code, out, _ = self.run_tool("--row", "AC-X-001", self.stub)   # `bad` diverges but is not selected
        self.assertEqual(code, 0)
        self.assertNotIn("bad", out)

    def test_a_not_applicable_case_that_diverges_and_is_rejected_explicitly_is_not_a_failure(self):
        self.add_case("far", "OUT of domain", "pekko output", "AC-X-003")
        self.mark_not_applicable("far")
        code, out, err = self.run_tool("--row", "AC-X-003", self.stub)
        self.assertEqual(code, 0, out + err)
        self.assertIn("0/0 selected cases match Pekko + 0 NA:Q-CFG-30 + 1 not applicable (diverge 0)", out)

    def test_a_not_applicable_case_that_the_replay_matches_is_a_stale_list_problem(self):
        self.mark_not_applicable("good")
        code, out, _ = self.run_tool("--row", "AC-X-001", self.stub)
        self.assertEqual(code, 1)
        self.assertIn("good: listed as NOT_APPLICABLE but the replay matches it (stale list)", out)
        self.assertIn("good: the harness did not reject the out-of-domain input explicitly", out)

    def test_a_divergence_that_is_only_the_jdk_17_double_to_string_bug_is_reported_apart(self):
        self.add_case("num", "NUM-SHORT x -> ok 9.8411002603808E18", "NUM-SHORT x -> ok 9.841100260380799E18", "AC-X-004")
        code, out, _ = self.run_tool("--row", "AC-X-004", self.stub)
        self.assertEqual(code, 0)
        self.assertIn("0/0 selected cases match Pekko + 1 NA:Q-CFG-30", out)
        self.assertIn("NA:Q-CFG-30: num", out)

    def test_a_bad_selector_is_refused_without_running_anything(self):
        code, out, err = self.run_tool("--row", "AC-NOPE-001", self.stub)
        self.assertEqual((code, out), (2, ""))
        self.assertIn("AC-NOPE-001", err)

    def test_an_unrecorded_binary_is_refused(self):
        code, _, err = self.run_tool("--row", "AC-X-001", os.path.join(self.tmp, "missing"))
        self.assertEqual(code, 2)
        self.assertIn("the binary does not exist", err)

    def test_a_binary_path_with_whitespace_is_refused(self):
        code, _, err = self.run_tool("--row", "AC-X-001", os.path.join(self.tmp, "a b"))
        self.assertEqual(code, 2)
        self.assertIn("whitespace", err)

    def test_build_cannot_be_combined_with_an_explicit_binary(self):
        code, _, err = self.run_tool("--row", "AC-X-001", "--build", self.stub)
        self.assertEqual(code, 2)
        self.assertIn("cannot be combined", err)

    def test_a_failed_build_is_refused(self):
        with mock.patch.object(subset.subprocess, "run", side_effect=subprocess.CalledProcessError(101, "cargo")):
            code, _, err = self.run_tool("--row", "AC-X-001", "--build")
        self.assertEqual(code, 2)
        self.assertIn("the build failed (exit 101)", err)

    def test_build_compiles_the_default_binary_then_replays_it(self):
        default_binary = os.path.join(self.root, "target", "release", "examples", "classic-parity")
        real_run = subprocess.run

        def fake_cargo_then_real(command, *args, **kwargs):
            if command[0] == "cargo":
                os.makedirs(os.path.dirname(default_binary))
                shutil.copy(self.stub, default_binary)
                return subprocess.CompletedProcess(command, 0)
            return real_run(command, *args, **kwargs)

        with mock.patch.object(subset.subprocess, "run", side_effect=fake_cargo_then_real):
            code, out, err = self.run_tool("--row", "AC-X-001", "--build")
        self.assertEqual(code, 0, out + err)

    def test_an_output_the_tool_contract_does_not_allow_is_a_harness_error(self):
        garbage = subprocess.CompletedProcess([], 0, stdout="not json", stderr="")
        real_run = subprocess.run
        with mock.patch.object(subset.subprocess, "run",
                               side_effect=lambda command, *a, **k: garbage if command[0] == "rust-anything" else real_run(command, *a, **k)):
            code, out, _ = self.run_tool("--row", "AC-X-001", self.stub)
        self.assertEqual(code, 2)
        self.assertIn("HARNESS ERROR: replay output rejected: the output is not JSON", out)

    def test_the_script_runs_as_a_program(self):
        write_text(os.path.join(self.root, "cases", "coverage.tsv"), "case\tfamily\trows\tbasis\n" + "".join(self.coverage_rows))
        result = subprocess.run([sys.executable, "-I", os.path.join(HERE, "replay_subset.py"), "--root", self.root, "--row", "AC-X-001", self.stub],
                                capture_output=True, text=True, timeout=120)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
