"""Shared replay logic: runs `rust-anything app replay` and applies the versioned NOT_APPLICABLE list.

An out-of-domain case (cases/not-applicable.tsv, decision Q-X-17) is neither skipped nor counted as a pass:
it must (1) diverge in the replay, because the Rust harness does not produce Pekko's output, and (2) be rejected
by the harness with an explicit error (exit 2, stdout empty, stderr naming the domain), never decoded lossily.

A second, separate category is NA:Q-CFG-30 (decision Q-CFG-30): a divergence that is only the JDK 17 bug in
`Double.toString`, recognised by its signature (na_q_cfg_30.py), never by case id. It is reported apart and is
neither a pass nor a divergence.
"""
import csv, os, re, subprocess

import na_q_cfg_30

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
REPORT = os.path.join(ROOT, "rust", "app-cases", "replay-report.md")
NA_FILE = os.path.join(ROOT, "cases", "not-applicable.tsv")
SUMMARY = re.compile(r"(\d+)/(\d+) cases passed")
REJECTION = "outside the API domain"


def load_not_applicable():
    with open(NA_FILE, encoding="utf-8", newline="") as fh:
        rows = list(csv.DictReader(fh, delimiter="\t", quoting=csv.QUOTE_NONE))
    for r in rows:
        assert r["case"] and r["reason"] and r["decision"] and r["link"] and len(r["case"]) <= 24, f"{NA_FILE}: incomplete row {r}"
    return {r["case"]: r for r in rows}


def run_replay(binary, timeout):
    return subprocess.run(["rust-anything", "app", "replay", ROOT, "--build", binary], stdin=subprocess.DEVNULL,
                          capture_output=True, text=True, timeout=timeout)


def failing_cases():
    """Names of the cases the last replay reported as divergent (from the report, which lists all of them)."""
    with open(REPORT, encoding="utf-8") as fh:
        return re.findall(r"(?m)^### (\S+)\s*$", fh.read())


def rejects_explicitly(binary, case):
    base = os.path.join(ROOT, "rust", "app-cases", case)
    import tomllib
    args = tomllib.load(open(base + ".toml", "rb"))["args"]
    stdin = open(base + ".stdin", "rb").read()
    r = subprocess.run([binary, *args], input=stdin, capture_output=True, timeout=60)
    return r.returncode == 2 and r.stdout == b"" and REJECTION.encode() in r.stderr


def case_outputs(binary, case):
    """-> (recorded Pekko stdout, stdout of `binary` for the same input) of a recorded case."""
    base = os.path.join(ROOT, "rust", "app-cases", case)
    import tomllib
    args = tomllib.load(open(base + ".toml", "rb"))["args"]
    recorded = open(base + ".stdout", "rb").read()
    produced = subprocess.run([binary, *args], input=open(base + ".stdin", "rb").read(), capture_output=True, timeout=60)
    return recorded, produced.stdout


DEMONSTRATION_RUNS = 2


def diverges_only_in_mutant(mutant_binary, control_binary, case, cases_dir=None, timeout=60):
    """True only when the divergence from Pekko on `case` is demonstrated and caused by the mutation: the unmutated build
    reproduces the recorded Pekko status code and stdout (the two things `rust-anything app replay` compares) and the mutant does not.
    False in every other situation (hang, unreadable recording, missing binary, control that already differs): the caller must then
    run the full replay; this function never concludes that a mutant survived."""
    import tomllib
    base = os.path.join(cases_dir or os.path.join(ROOT, "rust", "app-cases"), case)
    try:
        with open(base + ".toml", "rb") as fh:
            recording = tomllib.load(fh)
        args, expected_code = recording["args"], recording["status"]["code"]
        with open(base + ".stdin", "rb") as fh:
            stdin = fh.read()
        with open(base + ".stdout", "rb") as fh:
            expected_stdout = fh.read()
        outcome = {}
        for label, binary in (("control", control_binary), ("mutant", mutant_binary)):
            outcome[label] = []
            for _attempt in range(DEMONSTRATION_RUNS):   # a timing-dependent case must not decide on one sample
                run = subprocess.run([binary, *args], input=stdin, capture_output=True, timeout=timeout)
                outcome[label].append((run.returncode, run.stdout))
    except (OSError, ValueError, KeyError, subprocess.SubprocessError):
        return False
    recorded = (expected_code, expected_stdout)
    control_always_matches = all(sample == recorded for sample in outcome["control"])
    mutant_always_diverges_the_same_way = all(sample != recorded and sample == outcome["mutant"][0] for sample in outcome["mutant"])
    return control_always_matches and mutant_always_diverges_the_same_way


def evaluate(binary, result):
    """-> dict(ok, line, applicable_failing, na_problems, passed, total, na_q_cfg_30, other_na). `ok` means every
    applicable case matches Pekko, every NOT_APPLICABLE case is rejected explicitly, and what is left over is only
    NA:Q-CFG-30 (reported apart: not a pass, not a divergence)."""
    m = SUMMARY.search(result.stdout)
    if not m:
        return dict(ok=None, line=(result.stdout.strip() or result.stderr.strip())[-300:], applicable_failing=[], na_problems=[], passed=None, total=None,
                    na_q_cfg_30=[], other_na=0)
    passed, total = int(m.group(1)), int(m.group(2))
    na = load_not_applicable()
    failing = failing_cases() if passed < total else []
    unexplained = [c for c in failing if c not in na]
    na_q_cfg_30_cases = [c for c in unexplained if na_q_cfg_30.is_q_cfg_30(*case_outputs(binary, c))]
    applicable_failing = [c for c in unexplained if c not in na_q_cfg_30_cases]
    problems = []
    for case in na:
        if case not in failing:
            problems.append(f"{case}: listed as NOT_APPLICABLE but the replay matches it (stale list)")
        if not rejects_explicitly(binary, case):
            problems.append(f"{case}: the harness did not reject the out-of-domain input explicitly")
    applicable_total = total - len(na) - len(na_q_cfg_30_cases)
    applicable_passed = applicable_total - len(applicable_failing)
    line = (f"{applicable_passed}/{applicable_total} applicable + {len(na_q_cfg_30_cases)} NA:Q-CFG-30 + "
            f"{len(na)} not applicable (diverge {len(applicable_failing)})")
    return dict(ok=not applicable_failing and not problems, line=line, applicable_failing=applicable_failing,
                na_problems=problems, passed=applicable_passed, total=applicable_total,
                na_q_cfg_30=na_q_cfg_30_cases, other_na=len(na))
