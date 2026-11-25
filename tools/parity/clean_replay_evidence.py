"""Evidence of the clean replay of one gate run: the replay gate writes it, the mutation clean control reuses it instead of replaying the same binary against the
same corpus again. Content-addressed and fail-closed: any missing, changed, foreign or incomplete input means "replay again"."""
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


def relevant_environment(environment):
    return {name: value for name, value in sorted(environment.items()) if name.startswith(ENVIRONMENT_PREFIXES)}


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


def identity(root, binary, tools, environment, command, timeout):
    """Everything the verdict of a clean replay depends on; None when any of it is unknown."""
    if not os.path.isfile(binary) or any(value is None for value in tools.values()):
        return None
    with open(binary, "rb") as binary_file:
        binary_digest = hashlib.sha256(binary_file.read()).hexdigest()
    return {"binary": binary_digest, **{name: digest_paths(root, paths) for name, paths in INPUTS.items()}, "tools": tools,
            "environment": relevant_environment(environment), "command": list(command), "timeout": timeout}


def key_of(identity_record):
    return hashlib.sha256(json.dumps(identity_record, sort_keys=True).encode()).hexdigest()


def corpus_case_count(root):
    folder = os.path.join(root, "rust", "app-cases")
    return sum(name.endswith(".toml") for name in os.listdir(folder)) if os.path.isdir(folder) else 0


def refusal(evaluation, replayed_cases, corpus_cases):
    """None when the replay is complete evidence of a clean crate; else why it is not."""
    if evaluation.get("ok") is not True or evaluation["na_problems"] or not evaluation.get("total") or evaluation["passed"] != evaluation["total"]:
        return "the replay did not end clean (a divergence, a rejected output or an out-of-domain input accepted)"
    return None if replayed_cases == corpus_cases else f"the replay covered {replayed_cases} of {corpus_cases} recorded cases"


def record(identity_record, evaluation, replayed_cases, run_id):
    return {"schema": SCHEMA, "key": key_of(identity_record), "run_id": run_id, "identity": identity_record,
            "result": {"applicable_passed": evaluation["passed"], "applicable_total": evaluation["total"], "other_not_applicable": evaluation["other_na"], "replayed_cases": replayed_cases}}


def store(path, stored_record):
    """Atomic: a reader sees the whole record or nothing."""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    handle, temporary = tempfile.mkstemp(dir=os.path.dirname(path), suffix=".tmp")
    with os.fdopen(handle, "w") as out:
        json.dump(stored_record, out, indent=1)
    os.replace(temporary, path)


def discard(path):
    with contextlib.suppress(FileNotFoundError):
        os.unlink(path)


def leave_evidence(root, binary, replayed_cases, evaluation, environment):
    """Stores the evidence when, and only when, this is a complete clean replay inside a gate run. -> None when stored, else why not (always reported)."""
    run_id = environment.get(RUN_ID_ENV)
    why = None if run_id else "not inside a gate run"
    why = why or refusal(evaluation, replayed_cases, corpus_case_count(root))
    identity_record = None if why else identity(root, binary, current_tool_versions(), environment, CLEAN_REPLAY_COMMAND, REPLAY_TIMEOUT_SECONDS)
    if identity_record is None:
        return why or "the identity of the build is incomplete"
    store(evidence_path(root), record(identity_record, evaluation, replayed_cases, run_id))
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
    if stored.get("key") != key_of(identity_record) or stored.get("identity") != identity_record:
        return None, "an input changed since the replay"
    if stored.get("result", {}).get("replayed_cases") != corpus_cases:
        return None, "the corpus size changed since the replay"
    return stored, None
