"""Shared replay logic: runs `rust-anything app replay` and applies the versioned NOT_APPLICABLE list.

An out-of-domain case (cases/not-applicable.tsv, decision Q-X-17) is neither skipped nor counted as a pass:
it must (1) diverge in the replay, because the Rust harness does not produce Pekko's output, and (2) be rejected
by the harness with an explicit error (exit 2, stdout empty, stderr naming the domain), never decoded lossily.

A second, separate category is NA:Q-CFG-30 (decision Q-CFG-30): a divergence that is only the JDK 17 bug in
`Double.toString`, recognised by its signature (na_q_cfg_30.py), never by case id. It is reported apart and is
neither a pass nor a divergence.
"""
import contextlib, csv, dataclasses, fcntl, json, os, re, subprocess

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


@dataclasses.dataclass
class ReplayResult:
    """Everything one replay produced, taken from that replay alone: the count and the divergent cases from its own process output
    (`rust-anything app replay --json`), and the report it wrote, read while no other replay of the checkout could overwrite it."""
    returncode: int
    passed: object      # None when the output is incomplete or contradicts the tool's contract (a harness error)
    total: object
    failures: list      # names of the divergent cases
    report_text: object # the report of this replay, or None
    stdout: str
    stderr: str
    problem: object = None   # why the output was rejected, or None


@contextlib.contextmanager
def replay_lock():
    """One replay at a time per checkout: `rust-anything app replay` always writes rust/app-cases/replay-report.md."""
    lock_directory = os.path.join(ROOT, ".gates")
    os.makedirs(lock_directory, exist_ok=True)
    with open(os.path.join(lock_directory, "replay.lock"), "w") as lock_file:
        fcntl.flock(lock_file, fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(lock_file, fcntl.LOCK_UN)


def report_signature():
    """Identity of the report file as it is now (None when absent): a replay that wrote a report changes it."""
    try:
        status = os.stat(REPORT)
    except OSError:
        return None
    return (status.st_mtime_ns, status.st_ino, status.st_size)


def read_report_text():
    try:
        with open(REPORT, encoding="utf-8") as report_file:
            return report_file.read()
    except OSError:
        return None


REPORT_COUNT_LINE = re.compile(r"(?m)^Passed: (\d+)/(\d+)\s*$")


def validate_replay_output(returncode, stdout):
    """-> (passed, total, failure case names, problem). Checks what `rust-anything app replay --json` guarantees (observed on the real tool):
    always the keys passed/total/skipped/failures; passed + skipped + len(failures) == total; every failure has a case name and a
    non-empty list of problems; exit 0 <=> no failures; exit 1 <=> failures. Anything else (a partial or contradictory output, a skipped
    case, another exit code) is rejected: a rejected output is a harness error, never evidence of a clean replay."""
    try:
        payload = json.loads(stdout)
    except ValueError:
        return None, None, [], "the output is not JSON"
    if not isinstance(payload, dict):
        return None, None, [], "the output is not a JSON object"
    missing = [key for key in ("passed", "total", "skipped", "failures") if key not in payload]
    if missing:
        return None, None, [], "missing key(s): " + ", ".join(missing)
    counts = [payload["passed"], payload["total"], payload["skipped"]]
    if any(isinstance(count, bool) or not isinstance(count, int) or count < 0 for count in counts):
        return None, None, [], "passed, total and skipped must be non-negative integers"
    passed, total, skipped = counts
    entries = payload["failures"]
    if not isinstance(entries, list):
        return None, None, [], "failures is not a list"
    cases = []
    for entry in entries:
        if not isinstance(entry, dict) or not isinstance(entry.get("case"), str) or not entry["case"]:
            return None, None, [], "a failure without a case name"
        if not isinstance(entry.get("problems"), list) or not entry["problems"]:
            return None, None, [], f"the failure of {entry['case']} lists no problems"
        cases.append(entry["case"])
    if len(set(cases)) != len(cases):
        return None, None, [], "the same case is listed as failing twice"
    if skipped:
        return None, None, [], f"{skipped} skipped case(s): they give no evidence"
    if passed + len(cases) != total:
        return None, None, [], f"the counts do not add up: passed {passed} + failures {len(cases)} != total {total}"
    if returncode == 0 and cases:
        return None, None, [], "exit code 0 but failures are listed"
    if returncode == 1 and not cases:
        return None, None, [], "exit code 1 but no failure is listed"
    if returncode not in (0, 1):
        return None, None, [], f"exit code {returncode} is not part of the tool's contract"
    return passed, total, cases, None


def report_belongs_to(report_text, passed, total):
    """False when the report states a `Passed: P/T` line that contradicts the counts of the replay that is being classified."""
    stated = REPORT_COUNT_LINE.search(report_text)
    return stated is None or (int(stated.group(1)), int(stated.group(2))) == (passed, total)


def parse_replay_output(completed, report_text):
    passed, total, failures, problem = validate_replay_output(completed.returncode, completed.stdout)
    if problem is None and report_text is not None and not report_belongs_to(report_text, passed, total):
        report_text = None
    if problem is not None:
        report_text = None       # a rejected replay publishes nothing
    return ReplayResult(completed.returncode, passed, total, failures, report_text, completed.stdout, completed.stderr, problem)


def run_replay(binary, timeout):
    """Runs the recorded replay of `binary` and returns its own evidence. The lock spans from the start of the replay to the reading
    of its report, so the report a caller publishes is always the report of the replay it classified."""
    with replay_lock():
        signature_before = report_signature()
        completed = subprocess.run(["rust-anything", "app", "replay", ROOT, "--build", binary, "--json"], stdin=subprocess.DEVNULL,
                                   capture_output=True, text=True, timeout=timeout)
        # an earlier replay's report is never published: it counts only if this replay wrote (changed) the file
        report_text = read_report_text() if report_signature() != signature_before else None
    return parse_replay_output(completed, report_text)


def read_bytes(path):
    with open(path, "rb") as source_file:
        return source_file.read()


def load_recorded_args(base):
    import tomllib
    with open(base + ".toml", "rb") as recording_file:
        return tomllib.load(recording_file)["args"]


def rejects_explicitly(binary, case):
    base = os.path.join(ROOT, "rust", "app-cases", case)
    args = load_recorded_args(base)
    stdin = read_bytes(base + ".stdin")
    r = subprocess.run([binary, *args], input=stdin, capture_output=True, timeout=60)
    return r.returncode == 2 and r.stdout == b"" and REJECTION.encode() in r.stderr


def case_outputs(binary, case):
    """-> (recorded Pekko stdout, stdout of `binary` for the same input) of a recorded case."""
    base = os.path.join(ROOT, "rust", "app-cases", case)
    args = load_recorded_args(base)
    recorded = read_bytes(base + ".stdout")
    produced = subprocess.run([binary, *args], input=read_bytes(base + ".stdin"), capture_output=True, timeout=60)
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
    if result.passed is None:
        reason = result.problem or "output not understood"
        tail = (result.stdout.strip() or result.stderr.strip())[-200:]
        return dict(ok=None, line=f"replay output rejected: {reason}; exit {result.returncode}; {tail}", applicable_failing=[], na_problems=[],
                    passed=None, total=None, na_q_cfg_30=[], other_na=0)
    passed, total = result.passed, result.total
    na = load_not_applicable()
    failing = result.failures if passed < total else []
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
