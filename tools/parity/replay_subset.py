#!/usr/bin/env python3
"""Replays only the recorded Pekko 2.0.0 cases that belong to a contract row, a family or a name pattern.

`rust-anything app replay` replays everything (about 80 s here) or exactly one case by exact name (`--case`). The inner TDD loop of a
slice needs the middle: "all the cases that cite AC-PATH-007". cases/coverage.tsv already says which rows each case covers, so this tool
selects from it, stages a copy of just those recordings in a temporary tree, and runs the real `rust-anything app replay` over that tree.

  tools/parity/replay_subset.py --row AC-PATH-007 [--row ...] [--family oracle-util-core] [--match 'pa-*'] [--build] [binary]

Selectors add up (union). A selector that matches nothing is an error (exit 2): an empty selection must never look like a pass.
A `--row` also picks its dotted sub-rows (AC-UTIL-002 picks AC-UTIL-002.1 and .2), never a longer id that merely starts with it.

This is a development loop, NOT gate evidence: the gates still run the complete replay (tools/gates/replay.sh). It never touches the
versioned rust/app-cases/replay-report.md, because the staged tree has its own report. Classification of NOT_APPLICABLE and NA:Q-CFG-30
cases is the one of replay.py, restricted to the cases that were selected.
Exit codes: 0 every selected case matches, 1 divergences, 2 refused usage / harness error.
"""
import argparse, csv, dataclasses, fnmatch, os, shutil, subprocess, sys, tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import na_q_cfg_30
import replay_lib as lib

CASE_NAME_LIMIT = 24        # rust-anything stores a case under the first 24 characters of its name
RECORDING_EXTENSIONS = ("toml", "stdin", "stdout", "stderr")
APP_MANIFEST = "rust-anything.json"
NO_ROWS = "-"


class SelectionError(Exception):
    pass


@dataclasses.dataclass(frozen=True)
class CoverageEntry:
    case: str
    family: str
    rows: frozenset
    basis: str

    @property
    def recorded_name(self):
        return self.case[:CASE_NAME_LIMIT].rstrip("-")   # rust-anything also drops a dash the cut leaves at the end


def load_coverage(path):
    with open(path, encoding="utf-8", newline="") as coverage_file:
        table = csv.DictReader(coverage_file, delimiter="\t", quoting=csv.QUOTE_NONE)
        return [CoverageEntry(row["case"], row["family"], frozenset(() if row["rows"] == NO_ROWS else row["rows"].split(",")), row["basis"])
                for row in table]


def reconcile(entries, recorded_names):
    """-> (covered cases with no recording, recordings no covered case names), both sorted, comparing recorded names."""
    covered = {entry.recorded_name: entry.case for entry in entries}
    return sorted(case for name, case in covered.items() if name not in recorded_names), sorted(set(recorded_names) - set(covered))


def row_matches(cited_row, wanted_row):
    return cited_row == wanted_row or cited_row.startswith(wanted_row + ".")


def select_cases(entries, rows=(), families=(), globs=()):
    """-> the recorded names (first 24 characters) of every case picked by any selector, in catalog order, without duplicates."""
    if not (rows or families or globs):
        raise SelectionError("no selector: give --row, --family or --match")
    picked = {kind: {} for kind in ("row", "family", "match")}
    chosen = []
    for entry in entries:
        hits = (
            [("row", wanted) for wanted in rows if any(row_matches(cited, wanted) for cited in entry.rows)]
            + [("family", wanted) for wanted in families if entry.family == wanted]
            + [("match", pattern) for pattern in globs if fnmatch.fnmatchcase(entry.recorded_name, pattern) or fnmatch.fnmatchcase(entry.case, pattern)])
        for kind, selector in hits:
            picked[kind][selector] = True
        if hits and entry.recorded_name not in chosen:
            chosen.append(entry.recorded_name)
    unmatched = [f"--{kind} {selector}" for kind, wanted in (("row", rows), ("family", families), ("match", globs))
                 for selector in wanted if selector not in picked[kind]]
    if unmatched:
        raise SelectionError("selector(s) matching no recorded case: " + ", ".join(unmatched))
    return chosen


def stage_subset(source_root, names, destination):
    """Copies the manifest and the recordings of `names` into `<destination>/rust/app-cases`, ready for `rust-anything app replay`."""
    source = os.path.join(source_root, "rust", "app-cases")
    staged = os.path.join(destination, "rust", "app-cases")
    os.makedirs(staged)
    shutil.copy2(os.path.join(source, APP_MANIFEST), staged)
    for name in names:
        if not os.path.isfile(os.path.join(source, name + ".toml")):
            raise SelectionError(f"{name}: selected by cases/coverage.tsv but not recorded in rust/app-cases")
        for extension in RECORDING_EXTENSIONS:
            recording = os.path.join(source, f"{name}.{extension}")
            if os.path.isfile(recording):
                shutil.copy2(recording, staged)


def recorded_and_produced(binary, staged_root, case):
    base = os.path.join(staged_root, "rust", "app-cases", case)
    produced = subprocess.run([binary, *lib.load_recorded_args(base)], input=lib.read_bytes(base + ".stdin"), capture_output=True, timeout=60)
    return lib.read_bytes(base + ".stdout"), produced.stdout, produced


def classify(binary, staged_root, names, passed, total, failures):
    """Same verdict as replay_lib.evaluate, over the selected cases only (evaluate checks the whole NOT_APPLICABLE list, which a subset
    cannot answer for; replay_lib is a global mutant trigger, so it is not parameterised here)."""
    not_applicable = {case for case in lib.load_not_applicable() if case in names}
    unexplained = [case for case in failures if case not in not_applicable]
    q_cfg_30 = [case for case in unexplained if na_q_cfg_30.is_q_cfg_30(*recorded_and_produced(binary, staged_root, case)[:2])]
    applicable_failing = [case for case in unexplained if case not in q_cfg_30]
    problems = []
    for case in sorted(not_applicable):
        if case not in failures:
            problems.append(f"{case}: listed as NOT_APPLICABLE but the replay matches it (stale list)")
        produced = recorded_and_produced(binary, staged_root, case)[2]
        if not (produced.returncode == 2 and produced.stdout == b"" and lib.REJECTION.encode() in produced.stderr):
            problems.append(f"{case}: the harness did not reject the out-of-domain input explicitly")
    applicable_total = total - len(not_applicable) - len(q_cfg_30)
    return dict(ok=not applicable_failing and not problems, applicable_total=applicable_total,
                applicable_passed=applicable_total - len(applicable_failing), q_cfg_30=q_cfg_30,
                not_applicable=len(not_applicable), failing=applicable_failing, problems=problems)


def parse_arguments(arguments):
    parser = argparse.ArgumentParser(description="Replay only the recorded cases of some contract rows, families or names.")
    parser.add_argument("--row", action="append", default=[], help="contract row id from cases/coverage.tsv, e.g. AC-PATH-007 (repeatable)")
    parser.add_argument("--family", action="append", default=[], help="coverage family, e.g. oracle-util-core (repeatable)")
    parser.add_argument("--match", action="append", default=[], help="glob over case names, e.g. 'pa-*' (repeatable)")
    parser.add_argument("--build", action="store_true", help="compile the default parity binary first")
    parser.add_argument("--root", default=lib.ROOT, help=argparse.SUPPRESS)
    parser.add_argument("--coverage", default=None, help=argparse.SUPPRESS)
    parser.add_argument("binary", nargs="?", help="parity binary (default: target/release/examples/classic-parity)")
    return parser.parse_args(arguments)


def main(arguments):
    options = parse_arguments(arguments)
    default_binary = os.path.join(options.root, "target", "release", "examples", "classic-parity")
    binary = os.path.abspath(options.binary or default_binary)   # rust-anything runs it from the staged tree, not from here
    try:
        if options.build and options.binary:
            raise SelectionError("--build compiles the default parity binary: it cannot be combined with an explicit binary")
        names = select_cases(load_coverage(options.coverage or os.path.join(options.root, "cases", "coverage.tsv")),
                             options.row, options.family, options.match)
        if any(character.isspace() for character in binary):
            raise SelectionError("the binary path contains whitespace: `rust-anything app replay --build` takes one command-line string")
        if options.build:
            with lib.build_lock():
                subprocess.run(["cargo", "build", "--release", "-p", "rakka-actor", "--example", "classic-parity"], cwd=options.root, check=True)
        if not os.path.isfile(binary):
            raise SelectionError(f"the binary does not exist: {binary} (use --build)")
        with tempfile.TemporaryDirectory(prefix="replay-subset-") as staged_root:
            stage_subset(options.root, names, staged_root)
            completed = subprocess.run(["rust-anything", "app", "replay", staged_root, "--build", binary, "--json"],
                                       stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=900)
            passed, total, failures, problem = lib.validate_replay_output(completed.returncode, completed.stdout)
            if problem:
                print(f"HARNESS ERROR: replay output rejected: {problem}; exit {completed.returncode}; {(completed.stderr.strip() or completed.stdout.strip())[-200:]}")
                return 2
            if total != len(names):
                print(f"HARNESS ERROR: replayed {total} cases but {len(names)} were selected")
                return 2
            verdict = classify(binary, staged_root, set(names), passed, total, failures)
    except SelectionError as problem:
        print(f"refused: {problem}", file=sys.stderr)
        return 2
    except subprocess.CalledProcessError as failure:
        print(f"refused: the build failed (exit {failure.returncode})", file=sys.stderr)
        return 2
    print(f"{verdict['applicable_passed']}/{verdict['applicable_total']} selected cases match Pekko + {len(verdict['q_cfg_30'])} NA:Q-CFG-30 + "
          f"{verdict['not_applicable']} not applicable (diverge {len(verdict['failing'])}) -- SUBSET of {len(names)} cases, not gate evidence")
    for case in verdict["q_cfg_30"]:
        print("NA:Q-CFG-30:", case)
    for case in verdict["failing"]:
        print("DIVERGES:", case)
    for problem in verdict["problems"]:
        print("NOT_APPLICABLE problem:", problem)
    return 0 if verdict["ok"] else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
