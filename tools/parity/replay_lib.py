"""Shared replay logic: runs `rust-anything app replay` and applies the versioned NOT_APPLICABLE list.

An out-of-domain case (cases/not-applicable.tsv, decision Q-X-17) is neither skipped nor counted as a pass:
it must (1) diverge in the replay, because the Rust harness does not produce Pekko's output, and (2) be rejected
by the harness with an explicit error (exit 2, stdout empty, stderr naming the domain), never decoded lossily.
"""
import csv, os, re, subprocess

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


def evaluate(binary, result):
    """-> dict(ok, line, applicable_failing, na_problems, passed, total). `ok` means every applicable case matches
    Pekko and every NOT_APPLICABLE case is rejected explicitly."""
    m = SUMMARY.search(result.stdout)
    if not m:
        return dict(ok=None, line=(result.stdout.strip() or result.stderr.strip())[-300:], applicable_failing=[], na_problems=[], passed=None, total=None)
    passed, total = int(m.group(1)), int(m.group(2))
    na = load_not_applicable()
    failing = failing_cases() if passed < total else []
    applicable_failing = [c for c in failing if c not in na]
    problems = []
    for case in na:
        if case not in failing:
            problems.append(f"{case}: listed as NOT_APPLICABLE but the replay matches it (stale list)")
        if not rejects_explicitly(binary, case):
            problems.append(f"{case}: the harness did not reject the out-of-domain input explicitly")
    applicable_total = total - len(na)
    applicable_passed = applicable_total - len(applicable_failing)
    line = f"{applicable_passed}/{applicable_total} applicable + {len(na)} not applicable"
    return dict(ok=not applicable_failing and not problems, line=line, applicable_failing=applicable_failing,
                na_problems=problems, passed=applicable_passed, total=applicable_total)
