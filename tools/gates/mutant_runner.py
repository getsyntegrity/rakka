#!/usr/bin/env python3
"""Budgeted, resumable mutant runner (MILESTONE tier).

  python3.12 tools/gates/mutant_runner.py [--base REF] [--max-mutants N] [--max-minutes M] [--authorize-full-mutants]
                                          [--no-fast-kill] [--list]

Why it exists: measured on `mailbox-queue-is-lifo`, one mutant costs 7 s of build and 132 s of replay (all ~705 recorded cases,
no early exit), i.e. ~7.5 h for the 206 mutants. This runner keeps every guarantee and bounds the cost:
  * Each mutant is its own process (`check_divergence.py <name> --skip-control`) in its own process group; cancelling the runner
    kills the whole group and leaves the results already written.
  * Results are checkpointed per mutant in .gates/mutants/ under a key of: mutant patch, a conservative fingerprint of the
    WHOLE crate (every file of crates/rakka-actor except target/, plus the root Cargo.toml and Cargo.lock), harness fingerprint (corpus,
    recordings, this runner, check_divergence logic), configuration and rustc version. The import graph only orders the work.
    The key is re-checked before a cached DETECTED is accepted and before/after each mutant; the control must still match the state
    the verdict is issued on; otherwise INPUTS_CHANGED (error, never PASS). Only DETECTED results are reused; misses and errors are retried.
  * Each run writes its result files and logs into its own directory (.gates/mutants/runs/<stamp>-<pid>-<random>/), so two runs of the
    same mutant from one checkout never delete, overwrite or read each other's JSON; the checkpoints stay shared and atomic.
  * Order: mutants of the changed files, then their import neighbours, then the rest. The run stops at --max-mutants (default 40)
    or --max-minutes (default 30); anything not proved fresh is reported as pending and the exit code is 3 (not a pass).
  * Asking for more than the default cap needs --authorize-full-mutants.
  * The control (unmodified crate) replay runs once per crate/harness state, not once per mutant.
Exit: 0 all fresh+detected and control clean; 1 undetected mutant or control failed; 2 invalid run or refused; 3 pending mutants remain.
The last lines of the log are a <=12 line summary: read those, never the whole log.

Derived-from: none (project tooling, Apache-2.0).
"""
import ast, hashlib, json, os, re, shutil, signal, subprocess, sys, time, traceback

from gates import UsageError, changed_files

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
STORE_DIR = os.path.join(ROOT, ".gates", "mutants")
DEFAULT_BASE = "origin/feat/pekko-rust-pilot-migration"
DEFAULT_MAX_MUTANTS = 40
DEFAULT_MAX_MINUTES = 30
HARNESS_VERSION = 1       # bump when the semantics of this runner change: it invalidates every stored result
PER_MUTANT_TIMEOUT = 1800
GENUINE_TIMEOUT_SECONDS = 600   # a mutant needs ~150 s even with the full replay: given at least this long, running out of time is a real hang
TIMED_OUT = -999
CRATE = "crates/rakka-actor/"
PY = "python3.12"


# ---------- source relationships ----------

def import_edges(root):
    """file -> set of files related to it through `use crate::a::b` or `mod x;` (undirected, one hop; `super::` is not followed)."""
    src_root = os.path.join(root, CRATE, "src")
    files = [os.path.join(d, f) for d, _s, fs in os.walk(src_root) for f in fs if f.endswith(".rs")]
    rel = lambda p: os.path.relpath(p, root)
    edges = {rel(f): set() for f in files}

    def resolve(segments):
        for k in range(len(segments), 0, -1):
            base = os.path.join(src_root, *segments[:k])
            for candidate in (base + ".rs", os.path.join(base, "mod.rs")):
                if rel(candidate) in edges:   # a known file, not merely a path the (case-insensitive) filesystem accepts
                    return rel(candidate)
        return None

    def link(a, b):
        if b and a != b:
            edges[a].add(b)
            edges[b].add(a)

    for f in files:
        with open(f, encoding="utf-8", errors="replace") as source_file:
            text = source_file.read()
        here = rel(f)
        directory = os.path.relpath(os.path.dirname(f), src_root)
        parent = [] if directory == "." else directory.split(os.sep)
        stem = os.path.splitext(os.path.basename(f))[0]
        own = parent if stem in ("mod", "lib", "main") else parent + [stem]
        for m in re.finditer(r"\bcrate::([A-Za-z0-9_]+(?:::[A-Za-z0-9_]+)*)", text):
            link(here, resolve(m.group(1).split("::")))
        for m in re.finditer(r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;", text):
            link(here, resolve(own + [m.group(1)]))
    return edges


def find_stale_patterns(table, root, base=CRATE):
    """Mutants whose `old` text no longer occurs exactly once in their file: the code they target changed or disappeared.
    They are never run and never DETECTED; the run reports them as errors until they are rewritten or removed."""
    stale = []
    for name, (rel, old, _new, _why) in table.items():
        try:
            with open(os.path.join(root, base, rel), encoding="utf-8") as source_file:
                occurrences = source_file.read().count(old)
        except OSError:
            occurrences = 0
        if occurrences != 1:
            stale.append(name)
    return stale


def order_mutants(names, mutant_files, changed, edges):
    """[(name, 'changed'|'neighbour'|'other')], stable inside each class."""
    changed = set(changed)
    near = set().union(*(edges.get(c, set()) for c in changed)) if changed else set()
    def tier(name):
        f = mutant_files[name]
        return "changed" if f in changed else "neighbour" if f in near else "other"
    ranked = {"changed": 0, "neighbour": 1, "other": 2}
    return sorted(((n, tier(n)) for n in names), key=lambda pair: ranked[pair[1]])


# ---------- evidence keys and checkpoints ----------

_DIGEST_CACHE = {}


def file_digest(path):
    """sha256 of a file's content; a file that has not been touched for 2 s is re-hashed only when its stat changes (the run asks for
    the same ~6000 files before and after every mutant)."""
    try:
        status = os.stat(path)
        settled = time.time() - status.st_mtime > 2
        cache_key = (path, status.st_mtime_ns, status.st_size, status.st_ino)
        if settled and cache_key in _DIGEST_CACHE:
            return _DIGEST_CACHE[cache_key]
        with open(path, "rb") as file_to_hash:
            digest = hashlib.sha256(file_to_hash.read()).hexdigest()
        if settled:
            _DIGEST_CACHE[cache_key] = digest
        return digest
    except OSError:
        return "absent"


def crate_fingerprint(root):
    """Conservative state of everything a mutant build and replay depend on inside the repository's Rust side: every file of the crate
    (not its build directory) plus the root manifest and lock file. The import graph only orders the work; it never decides validity."""
    digest = hashlib.sha256()
    crate = os.path.join(root, CRATE)
    for directory, subdirs, names in os.walk(crate):
        subdirs[:] = sorted(d for d in subdirs if d != "target")
        for filename in sorted(names):
            path = os.path.join(directory, filename)
            digest.update(f"{os.path.relpath(path, root)}:{file_digest(path)};".encode())
    for build_input in ("Cargo.toml", "Cargo.lock"):
        digest.update(f"{build_input}:{file_digest(os.path.join(root, build_input))};".encode())
    return digest.hexdigest()


_LOGIC_DIGEST_CACHE = {}


def divergence_logic_digest(path):
    """check_divergence.py with the MUTANTS/TOOL_MUTANTS literals emptied: adding a mutant must not invalidate the others (the patch is in each key).
    Parsing the 1500-line file is memoised by its content digest."""
    content_digest = file_digest(path)
    if content_digest not in _LOGIC_DIGEST_CACHE:
        with open(path, encoding="utf-8") as source_file:
            tree = ast.parse(source_file.read())
        for node in ast.walk(tree):
            if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id in ("MUTANTS", "TOOL_MUTANTS") for t in node.targets):
                node.value = ast.Dict(keys=[], values=[])
        _LOGIC_DIGEST_CACHE[content_digest] = hashlib.sha256(ast.unparse(tree).encode()).hexdigest()
    return _LOGIC_DIGEST_CACHE[content_digest]


def harness_fingerprint(root):
    digest = hashlib.sha256(f"v{HARNESS_VERSION};".encode())
    roots = ["cases", "rust/app-cases", "tools/parity", "tools/gates/mutant_runner.py", "Cargo.toml", "Cargo.lock", CRATE + "Cargo.toml", CRATE + "examples"]
    for entry in roots:
        full = os.path.join(root, entry)
        paths = [full] if os.path.isfile(full) else [os.path.join(d, f) for d, _s, fs in os.walk(full) for f in fs]
        for p in sorted(paths):
            name = os.path.relpath(p, root)
            if name.endswith("replay-report.md") or "__pycache__" in name or name == "tools/parity/check_divergence.py":
                continue
            digest.update(f"{name}:{file_digest(p)};".encode())
    digest.update(divergence_logic_digest(os.path.join(root, "tools/parity/check_divergence.py")).encode())
    return digest.hexdigest()


def mutant_key(name, patch, crate_state, harness, config, toolchain):
    payload = json.dumps({"name": name, "patch": list(patch), "crate": crate_state, "harness": harness, "config": config, "toolchain": toolchain}, sort_keys=True)
    return hashlib.sha256(payload.encode()).hexdigest()


class ResultStore:
    def __init__(self, directory):
        self.directory = directory
        os.makedirs(directory, exist_ok=True)

    def _path(self, key):
        return os.path.join(self.directory, key + ".json")

    def get(self, key):
        try:
            with open(self._path(key)) as checkpoint_file:
                return json.load(checkpoint_file)
        except (OSError, ValueError):
            return None

    def put(self, key, record):
        partial_checkpoint_path = self._path(key) + f".{os.getpid()}.tmp"
        with open(partial_checkpoint_path, "w") as checkpoint_file:
            json.dump(record, checkpoint_file)
        os.replace(partial_checkpoint_path, self._path(key))     # atomic: a cancelled run never leaves a half-written checkpoint

    def is_fresh(self, key):
        record = self.get(key)
        return bool(record) and record.get("outcome") == "DETECTED"


# ---------- budget, authorization, planning ----------

def authorization_error(max_mutants, max_minutes, authorized):
    if authorized:
        return None
    if max_mutants is not None and max_mutants > DEFAULT_MAX_MUTANTS:
        return f"refused: --max-mutants {max_mutants} exceeds the default cap of {DEFAULT_MAX_MUTANTS}; a larger run needs --authorize-full-mutants (explicit Owner authorization)"
    if max_minutes is not None and max_minutes > DEFAULT_MAX_MINUTES:
        return f"refused: --max-minutes {max_minutes} exceeds the default cap of {DEFAULT_MAX_MINUTES}; a longer run needs --authorize-full-mutants (explicit Owner authorization)"
    return None


def mutant_timeout(remaining_seconds):
    """One mutant never outlives the time budget (at least 1 s so the child gets to start and be reported as TIMEOUT)."""
    if remaining_seconds is None:
        return PER_MUTANT_TIMEOUT
    return max(1, min(PER_MUTANT_TIMEOUT, int(remaining_seconds)))


def ensure_control(store, control_key, run_control, budget_limited=False):
    """'CLEAN' when the unmutated crate replays clean for this state: from the checkpoint, or by running `run_control()` (exit code) now.
    Called on every run, including a resumed one that executes no mutant: a DETECTED result proves nothing if the control never passed."""
    if store.is_fresh("control-" + control_key):
        return "CLEAN"
    code = run_control()
    if code == TIMED_OUT and budget_limited:   # the time budget ended before the control could: unverified (INCOMPLETE), neither passed nor failed
        return "UNVERIFIED"
    if code == 0:
        store.put("control-" + control_key, {"outcome": "DETECTED"})
        return "CLEAN"
    if code == 1:   # the control replayed and diverged from Pekko: a real regression of the crate
        store.put("control-" + control_key, {"outcome": "FAILED"})
        return "FAILED"
    return "ERROR"   # could not build/run (exit 2), cancelled (143), hung, killed: an invalid run says nothing about the code, and is not stored


def is_budget_limited(remaining_seconds):
    """True when the time budget, not the per-mutant limit, bounds this mutant's timeout."""
    return remaining_seconds is not None and remaining_seconds < GENUINE_TIMEOUT_SECONDS


def control_key(root, harness, toolchain):
    """Key of the unmutated-crate control: the whole crate state (a regression in a file no mutant targets must invalidate it), the harness and rustc."""
    return hashlib.sha256(f"{harness};{toolchain};{crate_fingerprint(root)}".encode()).hexdigest()


def run_guarded(store, control_key_value, run_control, run_mutants, budget_limited=False):
    """Control first: mutants run, and so can be stored as DETECTED, only against a crate whose unmutated replay is clean.
    -> (control verdict, mutant report or None when the control was FAILED or UNVERIFIED)."""
    control = ensure_control(store, control_key_value, run_control, budget_limited)
    if control != "CLEAN":
        return control, None
    return control, run_mutants()


def execute_mutant(name, result_file, launch, elapsed, budget_limited=False):
    """Runs one mutant through `launch()` (its exit code). A result file from an earlier run is deleted first, so a child that
    dies without writing is a HARNESS_ERROR and never inherits an old verdict."""
    if os.path.exists(result_file):
        os.unlink(result_file)
    code = launch()
    seconds = round(elapsed(), 1)
    if code == TIMED_OUT:   # killed by the shrinking time budget: not a verdict on the mutant, it stays pending
        return {"outcome": "BUDGET_EXHAUSTED" if budget_limited else "TIMEOUT", "seconds": seconds}
    try:
        with open(result_file) as result_json_file:
            outcome, passed, total, line = json.load(result_json_file)["mutants"][name]
        return {"outcome": outcome, "passed": passed, "total": total, "line": line[:200], "seconds": seconds}
    except (OSError, ValueError, KeyError):
        return {"outcome": "HARNESS_ERROR", "seconds": seconds, "line": f"no result (exit {code})"}


def run_plan(ordered, keys, store, execute, clock, max_mutants, max_seconds, key_now=None):
    """Runs the not-yet-fresh mutants in order until a budget is spent. Every result is stored before the next mutant starts.
    `key_now(name)` recomputes a mutant's key from the files as they are now: a result is stored only if the key is the same before
    and after the run (edits or a branch switch during a long run would otherwise file a verdict under the key of different code)."""
    start = clock()
    ran, pending, fresh_before, stopped_by, outcomes = 0, [], 0, None, {}
    for name, _tier in ordered:
        fresh = store.is_fresh(keys[name])
        if not fresh:
            if stopped_by is None and max_mutants is not None and ran >= max_mutants:
                stopped_by = "count budget"
            if stopped_by is None and max_seconds is not None and clock() - start >= max_seconds:
                stopped_by = "time budget"
            if stopped_by:       # pending: it claims no evidence, so there is nothing to re-validate
                pending.append(name)
                continue
        if key_now is not None and key_now(name) != keys[name]:   # a cached DETECTED, or a mutant about to run, must belong to the code as it is now
            outcomes[name] = "INPUTS_CHANGED"
            continue
        if fresh:
            fresh_before += 1
            outcomes[name] = "DETECTED"
            continue
        record = execute(name)
        if key_now is not None and key_now(name) != keys[name]:
            outcomes[name] = "INPUTS_CHANGED"
            ran += 1
            continue
        if record["outcome"] == "BUDGET_EXHAUSTED":
            stopped_by = "time budget"
            pending.append(name)
            continue
        store.put(keys[name], {**record, "name": name})
        outcomes[name] = record["outcome"]
        ran += 1
    return {"ran": ran, "pending": pending, "fresh_before": fresh_before, "stopped_by": stopped_by, "outcomes": outcomes}


def exit_code(undetected, errors, pending, control):
    if undetected or control == "FAILED":
        return 1
    if errors or control == "ERROR":
        return 2
    return 3 if pending or control == "UNVERIFIED" else 0


def summary_text(total, detected, undetected, errors, pending, stopped_by, control):
    lines = [f"MUTANTS fresh-detected {detected}/{total}  undetected {len(undetected)}  errors {len(errors)}  pending {len(pending)}  control {control}"]
    for label, names in (("UNDETECTED", undetected), ("ERROR", errors), ("PENDING", pending)):
        if names:
            lines.append(f"{label} ({len(names)}): " + ", ".join(names[:4]) + (" ..." if len(names) > 4 else ""))
    if stopped_by:
        lines.append(f"stopped by {stopped_by}; rerun the same command to resume (fresh results are kept)")
    return "\n".join(lines)


# ---------- process control ----------

def _leader_has_exited(pid):
    """True once the group leader is dead (a zombie), WITHOUT reaping it: the zombie keeps the pid (and so the group id) reserved, so the
    signals below can only ever reach our own group, never a foreign process that got the same pid. (`os.waitid` does not exist on macOS.)"""
    state = subprocess.run(["ps", "-o", "stat=", "-p", str(pid)], capture_output=True, text=True).stdout.strip()
    return state == "" or state.startswith("Z")


def _kill_group(process, term_grace=5.0, after_leader_grace=1.0):
    """SIGTERM to the group, then SIGKILL to whatever is still in it (descendants may ignore SIGTERM or outlive the leader)."""
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except (ProcessLookupError, PermissionError):   # macOS answers EPERM for a group holding only our zombie leader: it is empty
        pass
    deadline = time.time() + term_grace
    while not _leader_has_exited(process.pid) and time.time() < deadline:
        time.sleep(0.1)
    time.sleep(after_leader_grace)       # descendants get the same chance to exit cleanly
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError):
        pass
    process.wait()                       # reap the leader last


def run_cancellable(command, timeout, log_path, cwd=None):
    """Runs `command` in its own process group, output to log_path. Returns its exit code or TIMED_OUT. Any interruption
    (SIGTERM turned into SystemExit, Ctrl-C) kills the whole group before propagating."""
    with open(log_path, "w") as log:
        process = subprocess.Popen(command, cwd=cwd, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            return process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            _kill_group(process)
            return TIMED_OUT
        except BaseException:
            _kill_group(process)
            raise


# ---------- driver ----------

def git_changed(base):
    return changed_files(base)      # shared, NUL-separated parser; an unknown base is a UsageError


def option(argv, flag, default=None, cast=int):
    if flag not in argv:
        return default
    position = argv.index(flag) + 1
    if position >= len(argv) or argv[position].startswith("--"):
        raise UsageError(f"{flag} needs a valid value")
    try:
        return cast(argv[position])
    except ValueError:
        raise UsageError(f"{flag} needs a valid value") from None


def settle_control(outcomes, control_key_when_verified, control_key_now):
    """The control verified at the start only vouches for the state it ran on: if the crate or harness moved since, the run cannot pass."""
    if control_key_when_verified == control_key_now:
        return outcomes
    return {**outcomes, "(control)": "INPUTS_CHANGED"}


def new_run_directory(base_directory, clock=time.time, pid=os.getpid):
    """A private directory for one run's result files and logs (checkpoints stay shared in the store, keyed by content)."""
    os.makedirs(base_directory, exist_ok=True)
    name = f"{time.strftime('%Y%m%dT%H%M%S', time.gmtime(clock()))}-{pid()}-{os.urandom(4).hex()}"
    path = os.path.join(base_directory, name)
    os.makedirs(path)
    return path


RUN_DIRECTORY_NAME = re.compile(r"^\d{8}T\d{6}-\d+-[0-9a-f]+$")
RUN_DIRECTORIES_KEPT = 10


def prune_run_directories(base_directory, keep=RUN_DIRECTORIES_KEPT):
    """Removes all but the newest `keep` run directories (names sort by time); anything not named like a run directory is left alone."""
    try:
        runs = sorted(name for name in os.listdir(base_directory) if RUN_DIRECTORY_NAME.match(name))
    except OSError:
        return []
    removed = runs[:-keep] if keep else runs
    for name in removed:
        shutil.rmtree(os.path.join(base_directory, name), ignore_errors=True)
    return removed


def mutant_result_file(run_directory, mutant_name):
    return os.path.join(run_directory, mutant_name + ".json")


def main(argv):
    sys.path.insert(0, os.path.join(ROOT, "tools", "parity"))
    import check_divergence as cd
    authorized = "--authorize-full-mutants" in argv
    max_mutants = option(argv, "--max-mutants", None if authorized else DEFAULT_MAX_MUTANTS)
    max_minutes = option(argv, "--max-minutes", None if authorized else DEFAULT_MAX_MINUTES)
    refusal = authorization_error(max_mutants, max_minutes, authorized)
    if refusal:
        print(refusal)
        return 2
    fast_kill = "--no-fast-kill" not in argv
    all_mutants = {**cd.MUTANTS, **cd.TOOL_MUTANTS}
    files = {n: (("tools/parity/" if n in cd.TOOL_MUTANTS else CRATE) + m[0]) for n, m in all_mutants.items()}
    edges = import_edges(ROOT)
    stale = find_stale_patterns(cd.MUTANTS, ROOT) + find_stale_patterns(cd.TOOL_MUTANTS, ROOT, "tools/parity/")
    ordered = order_mutants([n for n in all_mutants if n not in stale], files, [c for c in git_changed(option(argv, "--base", DEFAULT_BASE, str))], edges)
    toolchain = subprocess.run(["rustc", "-V"], capture_output=True, text=True).stdout.strip()
    harness = harness_fingerprint(ROOT)
    crate_state = crate_fingerprint(ROOT)
    keys = {n: mutant_key(n, all_mutants[n][:3], crate_state, harness, {"fast_kill": fast_kill}, toolchain) for n in all_mutants}
    store = ResultStore(STORE_DIR)
    if "--list" in argv:
        for n, tier in ordered:
            print(f"{tier:9} {'fresh' if store.is_fresh(keys[n]) else 'stale':5} {n}")
        return 0
    run_directory = new_run_directory(os.path.join(STORE_DIR, "runs"))
    prune_run_directories(os.path.join(STORE_DIR, "runs"))
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(143))

    budget_seconds = None if max_minutes is None else max_minutes * 60
    run_started = time.time()

    def execute(name):
        result_file = mutant_result_file(run_directory, name)
        started = time.time()
        command = [PY, os.path.join(ROOT, "tools/parity/check_divergence.py"), name, "--skip-control", f"--result-json={result_file}"] + ([] if fast_kill else ["--no-fast-kill"])
        remaining = None if budget_seconds is None else budget_seconds - (started - run_started)
        launch = lambda: run_cancellable(command, mutant_timeout(remaining), os.path.join(run_directory, name + ".log"), cwd=ROOT)
        return execute_mutant(name, result_file, launch, lambda: time.time() - started, budget_limited=is_budget_limited(remaining))

    ckey = control_key(ROOT, harness, toolchain)
    run_control = lambda: run_cancellable([PY, os.path.join(ROOT, "tools/parity/check_divergence.py"), "--control-only"], mutant_timeout(budget_seconds), os.path.join(run_directory, "control.log"), cwd=ROOT)

    def key_now(name):
        return mutant_key(name, all_mutants[name][:3], crate_fingerprint(ROOT), harness_fingerprint(ROOT), {"fast_kill": fast_kill}, toolchain)

    def run_mutants():
        left = None if budget_seconds is None else budget_seconds - (time.time() - run_started)   # the control's time counts against the budget
        return run_plan(ordered, keys, store, execute, time.time, max_mutants, left, key_now=key_now)

    control, report = run_guarded(store, ckey, run_control, run_mutants, budget_limited=is_budget_limited(budget_seconds))
    if report is None:   # nothing ran: report what is already proved and what is pending
        report = run_plan(ordered, keys, store, execute, time.time, 0, None, key_now=key_now)
        report["stopped_by"] = f"control {control}"
    outcomes = {**report["outcomes"], **{n: "STALE_PATTERN" for n in stale}}
    outcomes = settle_control(outcomes, ckey, control_key(ROOT, harness_fingerprint(ROOT), toolchain))
    undetected = [n for n, o in outcomes.items() if o == "UNDETECTED"]
    errors = [n for n, o in outcomes.items() if o not in ("DETECTED", "UNDETECTED")]
    detected = sum(1 for o in outcomes.values() if o == "DETECTED")
    text = summary_text(len(all_mutants), detected, undetected, errors, report["pending"], report["stopped_by"], control)
    print(text)
    if stale:
        print(f"STALE patterns, not executed ({len(stale)}): " + ", ".join(stale[:6]) + (" ..." if len(stale) > 6 else ""))
    return exit_code(undetected, errors, report["pending"], control)


VERDICTS = {0: "PASS", 1: "FAIL", 2: "ERROR", 3: "INCOMPLETE", 143: "CANCELLED"}   # proposed Q-X-20, not yet accepted by the Owner


def verdict(code):
    return VERDICTS.get(code, "ERROR")


def run_as_script(argv):
    """Runs main(), prints the one-line verdict and exits with its code (143 after a cancellation, checkpoints already on disk)."""
    try:
        code = main(argv)
    except UsageError as problem:
        print(f"refused: {problem}")
        code = 2
    except SystemExit as stop:
        code = stop.code if isinstance(stop.code, int) else 2
    except KeyboardInterrupt:      # Ctrl-C: the process group was already killed by run_cancellable and the checkpoints are on disk
        code = 143
    except Exception:              # the run itself was invalid (disk full, missing git/rustc...): ERROR, never the FAIL of a surviving mutant
        traceback.print_exc()
        code = 2
    print(f"VERDICT {verdict(code)} (exit {code})")
    sys.exit(code)


if __name__ == "__main__":
    run_as_script(sys.argv[1:])
