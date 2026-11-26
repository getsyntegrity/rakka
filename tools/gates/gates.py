#!/usr/bin/env python3
"""Tiered gate runner: FAST (inner loop), PR (the mandatory per-slice gates), MILESTONE (PR + full mutants + fuzzers).

  python3.12 tools/gates/gates.py fast|pr|milestone [--base REF] [--no-reuse] [--list]
                                  [--max-mutants N] [--max-minutes M] [--authorize-full-mutants]   (MILESTONE mutants, see mutant_runner.py)

Three rules, none of which weakens a gate:
  * Evidence reuse by content: a gate that already passed is not re-run only when the git tree ids of every path it reads,
    its exact command and the toolchain version are identical and those paths have no uncommitted edits (see `evidence_key`).
  * Mutant selection (PR tier): only the mutants whose mutated file changed since --base (default: origin/feat/pekko-rust-pilot-migration);
    changes to the corpus, build or harness select all of them. MILESTONE always runs all of them.
  * Output discipline: every gate writes its full output to .gates/logs/ and the console gets one line per gate
    (plus a bounded tail when it fails). Never paste a whole log into an LLM context: read the file with grep/tail.

Derived-from: none (project tooling, Apache-2.0).
"""
import collections, dataclasses, hashlib, json, os, subprocess, sys, time, uuid

RUN_ID_ENV = "RAKKA_GATE_RUN_ID"      # same name as tools/parity/clean_replay_evidence.RUN_ID_ENV (checked by a test): the gates of one invocation form one run
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
GATES_DIR = os.path.join(ROOT, ".gates")
DEFAULT_BASE = "origin/feat/pekko-rust-pilot-migration"
MAX_FAILURE_CHARS = 2000
CRATE_PREFIX = "crates/rakka-actor/"
PY = "python3.12"

# Changes here can alter what any mutant does or how it is judged: every mutant is selected.
GLOBAL_MUTANT_TRIGGERS = ("Cargo.toml", "Cargo.lock", "cases/", "rust/app-cases/", "oracle/", "tools/parity/check_divergence.py",
                          "tools/parity/replay.py", "tools/parity/replay_lib.py", "tools/parity/na_q_cfg_30.py",
                          "crates/rakka-actor/Cargo.toml", "crates/rakka-actor/examples/")


# Files that every replay REWRITES as an output (tools/parity/replay_lib.py writes it, and the verdict of a replay reads only the copy its own run produced under the lock): they are never an
# input of a verdict, so a change to exactly them selects nothing. Only the exact path is excused: any other file under a trigger still selects everything.
NON_INPUT_ARTIFACTS = ("rust/app-cases/replay-report.md",)


def unit_mutants_of_changed_tests(changed, unit_mutants):
    """The unit mutants whose verdict reads a changed test: the integration test file they run (`test:<file>`), the helpers all of those share (`tests/common/`), or the test module inside
    the crate that their `cargo test --lib <filter>` selects. The filter is a SUBSTRING of the test path, so a module path that contains the filter, or that the filter extends, selects too
    (over-selection is the safe error)."""
    tests_directory, source_directory = CRATE_PREFIX + "tests/", CRATE_PREFIX + "src/"
    # code that exists only to be shared by the lib tests (test_support.rs, test_clock.rs, tests.rs, *_tests.rs) can flip ANY of their verdicts: it selects every unit mutant that runs a lib test
    shared_test_code = any(f.startswith(source_directory) and f.endswith(".rs") and "test" in f.rsplit("/", 1)[-1] for f in changed)
    picked = set()
    for name, (module, _expected_test) in unit_mutants.items():
        if module.startswith("test:"):
            picked_by_file = tests_directory + module[len("test:"):] + ".rs" in changed or any(f.startswith(tests_directory + "common/") for f in changed)
        else:
            def module_path(path):
                inside = path[len(source_directory):-len(".rs")].split("/")
                return "::".join(inside[:-1] if inside[-1] == "mod" else inside)
            picked_by_file = any(f.startswith(source_directory) and f.endswith(".rs") and (module in module_path(f) or module_path(f).startswith(module) or module.startswith(module_path(f)))
                                 for f in changed)
        if picked_by_file or (shared_test_code and not module.startswith("test:")):
            picked.add(name)
    return picked


def select_mutants(changed_files, mutants, tool_mutants, unit_mutants=None):
    """Names of the mutants a change must re-check; [] when no mutated file changed. `unit_mutants` (name -> (module or `test:<file>`, test)) adds the unit mutants whose test changed."""
    changed = [path for path in changed_files if path not in NON_INPUT_ARTIFACTS]
    if any(f == t or f.startswith(t) for f in changed for t in GLOBAL_MUTANT_TRIGGERS):
        return list(mutants) + list(tool_mutants)
    by_test = unit_mutants_of_changed_tests(changed, unit_mutants) if unit_mutants else set()
    picked = [n for n, (rel, *_rest) in mutants.items() if CRATE_PREFIX + rel in changed or n in by_test]
    picked += [n for n, (rel, *_rest) in tool_mutants.items() if "tools/parity/" + rel in changed]
    return picked


def evidence_key(command, input_trees, toolchain, dirty_inputs):
    """Cache key of a passing gate, or None when evidence must not be reused (uncommitted edits in the inputs)."""
    if dirty_inputs:
        return None
    payload = json.dumps({"command": command, "inputs": input_trees, "toolchain": toolchain}, sort_keys=True)
    return hashlib.sha256(payload.encode()).hexdigest()


def gate_label(gate, exit_code):
    return "PASS" if exit_code == 0 else dict(gate.exit_labels).get(exit_code, "FAIL")


def summary_line(tier, problems, gate_count, head):
    """`problems` = [(gate name, verdict label)] of the gates that did not pass."""
    outcome = ", ".join(f"{label} {name}" for name, label in problems) if problems else "all gates passed"
    return f"{tier.upper()}: {outcome} ({gate_count} gates, HEAD {head})"


def verdict_line(name, exit_code, seconds, reused, log_path, log_text, label="FAIL"):
    if exit_code == 0:
        return f"PASS {name} (reused evidence) log={log_path}" if reused else f"PASS {name} ({seconds:.0f}s) log={log_path}"
    tail = "\n".join(log_text.splitlines()[-20:])[-MAX_FAILURE_CHARS:]
    return f"{label} {name} exit={exit_code} ({seconds:.0f}s) log={log_path}\n{tail}"


@dataclasses.dataclass(frozen=True)
class Gate:
    name: str
    command: list
    inputs: tuple            # repo paths whose git tree ids key the evidence
    skip_reason: str = ""    # non-empty: nothing to run (reported, never silent)
    regenerates: tuple = ()  # tracked files this gate rewrites on every run: not "uncommitted edits" of its own inputs
    exit_labels: tuple = ()  # (exit code, verdict label) pairs; any other non-zero code is a plain FAIL
    reusable: bool = True    # False: the gate keeps its own finer evidence and must run every time


RUST_INPUTS = ("crates", "Cargo.toml", "Cargo.lock")
ORACLE_INPUTS = RUST_INPUTS + ("cases", "tools/parity", "oracle")           # what the differential fuzzers read
PARITY_INPUTS = ORACLE_INPUTS + ("rust/app-cases",)                          # plus the recordings the replay compares against
MUTANT_RUNNER_LABELS = ((1, "FAIL"), (2, "ERROR"), (3, "INCOMPLETE"), (143, "CANCELLED"))   # exit codes of tools/gates/mutant_runner.py (Q-X-20, accepted)
REPLAY_REGENERATED = ("rust/app-cases/replay-report.md", "rust/divergence-reports")
FUZZ_COUNTS = [("fuzz_diff.py", 100000), ("fuzz_scenarios.py", 1500), ("fuzz_tell.py", 1000), ("fuzz_hier.py", 1000)]


def tier_gates(tier, selected_mutants, mutant_args=()):
    fast = [
        Gate("fmt", ["cargo", "fmt", "--all", "--check"], RUST_INPUTS),
        Gate("package-check", [PY, "tools/package/check.py"], ("docs", "tools/package", "crates", "tools", "oracle", "AGENTS.md")),
        Gate("unit-tests", ["cargo", "test", "--workspace", "--lib"], RUST_INPUTS),
    ]
    pr_extra = [
        Gate("clippy", ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"], RUST_INPUTS),
        Gate("test", ["cargo", "test", "--workspace"], RUST_INPUTS),
        Gate("deny", ["cargo", "deny", "check", "licenses"], ("Cargo.toml", "Cargo.lock", "deny.toml", "crates")),
        Gate("coverage", ["tools/parity/coverage.sh"], RUST_INPUTS + ("tools/parity/coverage.sh",)),
        Gate("replay", ["tools/gates/replay.sh"], PARITY_INPUTS, regenerates=REPLAY_REGENERATED),
    ]
    # per-mutant reuse and a control on every run (Q-X-22 R3): a passing record of the whole gate would not carry the external tool versions
    mutants = Gate("mutants", [PY, "tools/gates/pr_mutants.py", *selected_mutants], PARITY_INPUTS + ("tools/gates",),
                   "" if selected_mutants else "no mutated file changed since the base", REPLAY_REGENERATED, MUTANT_RUNNER_LABELS, reusable=False)
    if tier == "fast":
        return fast
    if tier == "pr":
        return fast + pr_extra + [mutants]
    # budgeted + resumable (tools/gates/mutant_runner.py): never all 206 without --authorize-full-mutants
    full_mutants = Gate("mutants", [PY, "tools/gates/mutant_runner.py", *mutant_args], PARITY_INPUTS + ("tools/gates",), regenerates=REPLAY_REGENERATED, exit_labels=MUTANT_RUNNER_LABELS)
    fuzz = [Gate("fuzz-" + s[5:-3], [PY, "-u", f"tools/parity/{s}", "--count", str(n), "--seed", str(seed)], ORACLE_INPUTS)
            for s, n in FUZZ_COUNTS for seed in (1, 2, 3)]
    return fast + pr_extra + [full_mutants] + fuzz


def git_raw(*args, repo=ROOT):
    """git stdout, unstripped (NUL-separated output must stay intact); a failing git command is a usage error, never an empty answer."""
    result = subprocess.run(["git", *args], cwd=repo, capture_output=True, text=True)
    if result.returncode != 0:
        raise UsageError(f"git {' '.join(args)} failed: {result.stderr.strip()[:200]}")
    return result.stdout


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()


class UsageError(Exception):
    pass


def status_pathspecs(input_paths, regenerates):
    """Pathspecs of the dirty check: the gate's existing inputs minus the files it rewrites itself; None when there is nothing to check
    (an empty list would make git look at the whole repository)."""
    if not input_paths:
        return None
    return [*input_paths, *(":(exclude)" + path for path in regenerates)]


TIERS = ("fast", "pr", "milestone")
VALUE_FLAGS = ("--base", "--max-mutants", "--max-minutes")


def parse_arguments(argv):
    """-> dict(tier, base, no_reuse, list, mutant_args). `--base` is validated for every tier (it needs a value that is not another flag);
    the value of a value-taking flag is never read as the tier, so a ref named like a tier is fine."""
    consumed = set()
    for flag in VALUE_FLAGS:
        if flag in argv:
            position = argv.index(flag) + 1
            if flag == "--base" and (position >= len(argv) or argv[position].startswith("--")):
                raise UsageError("--base needs a ref")
            if position < len(argv) and not argv[position].startswith("--"):
                consumed.update({position - 1, position})
    positionals = [token for index, token in enumerate(argv) if index not in consumed and not token.startswith("--")]
    if len(positionals) != 1 or positionals[0] not in TIERS:
        raise UsageError("expected exactly one tier: fast, pr or milestone")
    tier = positionals[0]
    base = argv[argv.index("--base") + 1] if "--base" in argv else DEFAULT_BASE
    return {"tier": tier, "base": base, "no_reuse": "--no-reuse" in argv, "list": "--list" in argv, "mutant_args": mutant_arguments(tier, argv)}


def mutant_arguments(tier, argv):
    """The MILESTONE mutant flags forwarded to tools/gates/mutant_runner.py; a value-taking flag must be followed by a value."""
    if tier != "milestone":
        return []
    forwarded = []
    for flag in ("--max-mutants", "--max-minutes", "--base"):
        if flag in argv:
            position = argv.index(flag) + 1
            if position >= len(argv) or argv[position].startswith("--"):
                raise UsageError(f"{flag} needs a value")
            forwarded += [flag, argv[position]]
    return forwarded + [flag for flag in ("--authorize-full-mutants", "--no-fast-kill") if flag in argv]


InputState = collections.namedtuple("InputState", "trees dirty fingerprint")
_CONTENT_DIGEST_CACHE = {}


def file_content_digest(path):
    """sha256 of a file's content ('absent' when it does not exist); a file untouched for 2 s is re-read only when its stat changes."""
    try:
        status = os.stat(path)
    except OSError:
        return "absent"
    settled = time.time() - status.st_mtime > 2
    cache_key = (path, status.st_mtime_ns, status.st_size, status.st_ino)
    if settled and cache_key in _CONTENT_DIGEST_CACHE:
        return _CONTENT_DIGEST_CACHE[cache_key]
    with open(path, "rb") as content_file:
        digest = hashlib.sha256(content_file.read()).hexdigest()
    if settled:
        _CONTENT_DIGEST_CACHE[cache_key] = digest
    return digest


def inputs_fingerprint(paths, regenerates=()):
    """The effective state of a gate's inputs, local changes included: which declared paths exist, which files are under them (tracked or
    untracked, not ignored; a tracked file deleted from the working tree is listed and read as absent) and the content of each, minus the files
    the gate itself regenerates. Appearance, removal, rename and any edit of an input change it, whether or not the tree was already dirty."""
    digest = hashlib.sha256()
    for path in paths:
        digest.update(f"declared:{path}:{os.path.exists(os.path.join(ROOT, path))};".encode())
    listing = git_raw("ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *paths, repo=ROOT)
    for name in sorted({entry for entry in listing.split("\0") if entry}):
        if not is_regenerated(name, regenerates):
            digest.update(f"{name}:{file_content_digest(os.path.join(ROOT, name))};".encode())
    return digest.hexdigest()


def input_state(paths, regenerates=()):
    """-> InputState(trees, dirty, fingerprint): the HEAD tree id of each existing input ('untracked' for one HEAD does not have), whether the
    inputs have uncommitted changes (the evidence of a gate is reused only for clean inputs), and the fingerprint of their effective content."""
    trees = {}
    for path in paths:
        if os.path.exists(os.path.join(ROOT, path)):
            try:
                trees[path] = git("rev-parse", f"HEAD:{path}")
            except subprocess.CalledProcessError:
                trees[path] = "untracked"
    pathspecs = status_pathspecs(list(trees), regenerates)
    dirty = bool(pathspecs) and bool(git("status", "--porcelain", "--untracked-files=normal", "--", *pathspecs))
    return InputState(trees, dirty, inputs_fingerprint(paths, regenerates))


def run_gate(gate, toolchain, reuse, head):
    log_dir = os.path.join(GATES_DIR, "logs", head)
    os.makedirs(log_dir, exist_ok=True)
    log_path = os.path.relpath(os.path.join(log_dir, gate.name + ".log"), ROOT)
    if gate.skip_reason:
        return f"SKIP {gate.name}: {gate.skip_reason}", 0, "PASS"
    state_before = input_state(gate.inputs, gate.regenerates)
    trees, dirty = state_before[0], state_before[1]
    key = evidence_key(gate.command, trees, toolchain, dirty) if gate.reusable else None
    evidence = os.path.join(GATES_DIR, "evidence", f"{gate.name}-{key}.json") if key else None
    if reuse and evidence and os.path.exists(evidence):
        with open(evidence) as evidence_file:
            recorded_log = json.load(evidence_file)["log"]
        return verdict_line(gate.name, 0, 0.0, True, recorded_log, ""), 0, "PASS"
    started = time.time()
    with open(os.path.join(ROOT, log_path), "w") as log:
        code = subprocess.run(gate.command, cwd=ROOT, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT).returncode
    seconds = time.time() - started
    text = ""
    if code:
        with open(os.path.join(ROOT, log_path)) as log_file:
            text = log_file.read()
    if code == 0 and input_state(gate.inputs, gate.regenerates) != state_before:
        # HEAD moved or the inputs were edited while the gate ran: what it proved describes a state that no longer exists, so it is neither
        # recorded as evidence nor reported as a pass
        return (f"INPUTS_CHANGED {gate.name}: its inputs changed while it ran; the result is not recorded (log={log_path})", 1, "INPUTS_CHANGED")
    if code == 0 and evidence:
        os.makedirs(os.path.dirname(evidence), exist_ok=True)
        with open(evidence, "w") as evidence_file:
            json.dump({"gate": gate.name, "command": gate.command, "head": head, "inputs": trees, "log": log_path, "seconds": seconds}, evidence_file)
    label = gate_label(gate, code)
    return verdict_line(gate.name, code, seconds, False, log_path, text, label), code, label


def is_regenerated(path, regenerated):
    return any(path == entry or path.startswith(entry.rstrip("/") + "/") for entry in regenerated)


def changed_paths_from_git(diff_output, status_output, regenerated=REPLAY_REGENERATED):
    """Paths from `git diff --name-status -z` (committed) and `git status --porcelain -z` (working tree): spaces survive, and a rename
    or copy contributes both its source and its destination, committed or not. A working-tree change to a file that the gates themselves
    regenerate on every run (the replay report) is not a change of the code under test: without this, each replay would make the next PR gate
    select every mutant."""
    paths = set()
    diff_tokens = diff_output.split("\0")
    token = 0
    while token < len(diff_tokens):
        status = diff_tokens[token]
        token += 1
        if not status:
            continue
        path_count = 2 if status[0] in "RC" else 1
        paths.update(path for path in diff_tokens[token:token + path_count] if path)
        token += path_count
    entries = status_output.split("\0")
    position = 0
    while position < len(entries):
        entry = entries[position]
        position += 1
        if len(entry) < 4:
            continue
        original = None
        if entry[0] in "RC" or entry[1] in "RC":      # the next entry is the original path
            if position < len(entries) and entries[position]:
                original = entries[position]
            position += 1
        for path in (entry[3:], original):
            if path and not is_regenerated(path, regenerated):
                paths.add(path)
    return sorted(paths)


def changed_files(base, repo=ROOT):
    return changed_paths_from_git(git_raw("diff", "--name-status", "-z", "-M", f"{base}...HEAD", repo=repo), git_raw("status", "--porcelain", "-z", repo=repo))


def main(argv):
    try:
        parsed = parse_arguments(argv)
    except UsageError as problem:
        print(f"refused: {problem}")
        return 2
    os.environ[RUN_ID_ENV] = uuid.uuid4().hex      # a fresh id per invocation, inherited by every gate: evidence one gate leaves is valid for this run only
    tier, base, mutant_args = parsed["tier"], parsed["base"], parsed["mutant_args"]
    selected = []
    if tier == "pr":
        sys.path.insert(0, os.path.join(ROOT, "tools", "parity"))
        import check_divergence
        try:
            selected = select_mutants(changed_files(base), check_divergence.MUTANTS, check_divergence.TOOL_MUTANTS, check_divergence.UNIT_MUTANTS)
        except UsageError as problem:
            print(f"refused: {problem}")
            return 2
    gates = tier_gates(tier, selected, mutant_args)
    if parsed["list"]:
        for g in gates:
            print(g.name, " ".join(g.command) if not g.skip_reason else f"(skip: {g.skip_reason})")
        return 0
    toolchain = subprocess.run(["rustc", "-V"], capture_output=True, text=True).stdout.strip()
    head = git("rev-parse", "--short=12", "HEAD")
    failed = []
    for g in gates:
        line, code, label = run_gate(g, toolchain, not parsed["no_reuse"], head)
        print(line, flush=True)
        if code:
            failed.append((g.name, label))
    print(summary_line(tier, failed, len(gates), head))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
