"""Evidence of the clean replay of one gate run, reused by the mutation clean control instead of replaying again. Content-addressed, fail-closed: anything missing, changed or foreign means "replay again"."""
import contextlib, hashlib, json, os, platform, subprocess, sys, tempfile

SCHEMA = 1
RUN_ID_ENV = "RAKKA_GATE_RUN_ID"
EVIDENCE_RELATIVE = os.path.join(".gates", "evidence", "clean-replay.json")
CLEAN_REPLAY_COMMAND = ("cargo", "build", "--release", "-p", "rakka-actor", "--example", "classic-parity")   # how both callers build the binary they replay
REPLAY_TIMEOUT_SECONDS = 900
INPUTS = {"source": ("crates", "Cargo.toml", "Cargo.lock"), "corpus": ("rust/app-cases", "cases"), "oracle": ("oracle",),
          "tooling": ("tools/parity/replay.py", "tools/parity/replay_lib.py", "tools/parity/na_q_cfg_30.py", "tools/parity/clean_replay_evidence.py")}
GENERATED = ("rust/app-cases/replay-report.md",)      # rewritten by every replay: not an input
IGNORED_DIRECTORIES = ("target", "__pycache__", ".git", ".gates")
ENVIRONMENT_PREFIXES = ("CARGO", "RUSTC", "RUSTFLAGS", "RUSTDOC", "RUST_", "LLVM_", "CC", "CXX", "CFLAGS", "LDFLAGS", "SDKROOT", "MACOSX_", "DYLD_", "LD_")


def evidence_path(root):
    return os.path.join(root, EVIDENCE_RELATIVE)


def files_under(top):
    """The files of a path; build output and VCS directories BELOW it are pruned (what the checkout itself is called or lives under never matters)."""
    found = [top] if os.path.isfile(top) else []
    for folder, folders, names in os.walk(top):
        folders[:] = sorted(name for name in folders if name not in IGNORED_DIRECTORIES)
        found += [os.path.join(folder, name) for name in names]
    return found


def digest_paths(root, relatives):
    """One digest over the names and bytes of every file under `relatives` (a missing path is part of it as nothing)."""
    digest = hashlib.sha256()
    for relative in relatives:
        digest.update(f"{relative}\0".encode())
        top = os.path.join(root, relative)
        for path in sorted(files_under(top)) if os.path.exists(top) else []:
            if os.path.relpath(path, root) not in GENERATED:
                with open(path, "rb") as source_file:
                    digest.update(os.path.relpath(path, root).encode() + b"\0" + hashlib.sha256(source_file.read()).digest())
    return digest.hexdigest()


def current_tool_versions():
    """Versions of what shapes the replayed binary or its judgement; a tool that cannot answer is None (and then nothing is reused)."""
    def version(command):
        try:
            done = subprocess.run(command, capture_output=True, text=True, timeout=60)
        except (OSError, subprocess.SubprocessError):
            return None
        return done.stdout.strip() or None if done.returncode == 0 else None
    return {"rustc": version(["rustc", "-vV"]), "cargo": version(["cargo", "-V"]), "rust-anything": version(["rust-anything", "--version"]),
            "python": sys.version, "platform": platform.platform()}


def identity(root, binary, environment, tools=None, command=CLEAN_REPLAY_COMMAND, timeout=REPLAY_TIMEOUT_SECONDS):
    """Everything the verdict of a clean replay depends on (tool versions are looked up unless given); None when any of it is unknown."""
    tools = tools or current_tool_versions()
    try:
        with open(binary, "rb") as binary_file:
            binary_digest = hashlib.sha256(binary_file.read()).hexdigest()
    except OSError:
        return None
    if any(value is None for value in tools.values()):
        return None
    return {"binary": binary_digest, **{name: digest_paths(root, paths) for name, paths in INPUTS.items()}, "tools": tools,
            "environment": {name: value for name, value in sorted(environment.items()) if name.startswith(ENVIRONMENT_PREFIXES)}, "command": list(command), "timeout": timeout}


def key_of(content):
    return hashlib.sha256(json.dumps(content, sort_keys=True).encode()).hexdigest()


def corpus_case_count(root):
    folder = os.path.join(root, "rust", "app-cases")
    return sum(name.endswith(".toml") for name in os.listdir(folder)) if os.path.isdir(folder) else 0


def refusal(evaluation, replayed_cases, corpus_cases):
    """None when the replay is complete evidence of a clean crate; else why it is not."""
    if evaluation.get("ok") is not True or evaluation["na_problems"] or not evaluation.get("total") or evaluation["passed"] != evaluation["total"]:
        return "the replay did not end clean (a divergence, a rejected output or an out-of-domain input accepted)"
    return None if replayed_cases == corpus_cases else f"the replay covered {replayed_cases} of {corpus_cases} recorded cases"


def record(identity_record, evaluation, replayed_cases, run_id):
    """The result is part of what the key covers: a record is the identity AND the clean replay that was verified for it."""
    result = {"applicable_passed": evaluation["passed"], "applicable_total": evaluation["total"], "other_not_applicable": evaluation["other_na"], "replayed_cases": replayed_cases}
    return {"schema": SCHEMA, "key": key_of({"identity": identity_record, "result": result}), "run_id": run_id, "identity": identity_record, "result": result}


def store(path, stored_record):
    """Atomic: a reader sees the whole record or nothing."""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with tempfile.NamedTemporaryFile("w", dir=os.path.dirname(path), suffix=".tmp", delete=False) as out:
        json.dump(stored_record, out, indent=1)
    os.replace(out.name, path)


def discard(path):
    with contextlib.suppress(FileNotFoundError):
        os.unlink(path)


def leave_evidence(root, binary, before, replayed_cases, evaluation, environment):
    """Stores the evidence only for a complete clean replay in a gate run whose inputs did not change while it ran (`before` = identity captured before it). -> None, else why not."""
    run_id = environment.get(RUN_ID_ENV)
    why = None if run_id else "not inside a gate run"
    why = why or refusal(evaluation, replayed_cases, corpus_case_count(root))
    if not why and (before is None or identity(root, binary, environment) != before):
        why = "an input changed while the replay ran, or the identity is incomplete"
    if why:
        return why
    store(evidence_path(root), record(before, evaluation, replayed_cases, run_id))
    return None


def reusable(path, identity_record, run_id, corpus_cases):
    """-> (record, None) when this run's own complete clean replay of exactly this identity is on disk, else (None, why)."""
    if identity_record is None or not run_id:
        return None, "the identity is incomplete or this is not a gate run"
    try:
        with open(path) as stored_file:
            stored = json.load(stored_file)
    except (OSError, ValueError):
        return None, "no readable evidence"
    if stored.get("schema") != SCHEMA or stored.get("run_id") != run_id:
        return None, "the evidence is of another format or of another run"
    result = stored.get("result", {})
    if stored.get("identity") != identity_record or stored.get("key") != key_of({"identity": identity_record, "result": result}):
        return None, "an input changed since the replay or the record was altered"
    if result.get("replayed_cases") != corpus_cases or not result.get("applicable_total") or result.get("applicable_passed") != result.get("applicable_total"):
        return None, "the record does not describe a complete clean replay of this corpus"
    return stored, None
