#!/usr/bin/env python3
"""PR-tier mutants with per-mutant evidence reuse (Q-X-22 R3).

  python3.12 tools/gates/pr_mutants.py [--no-reuse] NAME...

The control (unmodified crate) runs on every invocation and must be clean before any mutant result counts. Only DETECTED is reused, and only
while its whole identity is unchanged: patch, crate, harness, this file, the build environment and the external tools (rustc, cargo,
rust-anything (version and bytes), python, platform) and the cargo configuration files. Tool mutants are never reused: their tests read repository files that the identity does not cover.
Every run keeps its logs and a report.json (what ran, what was reused) under .gates/mutants/pr/runs/; nothing is pruned.
Exit: 0 all detected and control clean; 1 survivor or dirty control; 2 invalid run (timeout, error, inputs changed).

Derived-from: none (project tooling, Apache-2.0).
"""
import hashlib, json, os, re, shutil, signal, sys, time

import mutant_runner as runner

ROOT = runner.ROOT
STORE_DIR = os.path.join(runner.STORE_DIR, "pr")


def reuse_key(name, patch, state):
    """The identity of one result; None (never reuse) when an external tool cannot say which version it is."""
    if any(version is None for version in state["tools"].values()):
        return None
    payload = json.dumps({"name": name, "patch": list(patch), "state": state}, sort_keys=True)
    return hashlib.sha256(payload.encode()).hexdigest()


def external_identity(root, environment, rust_anything):
    """What shapes a verdict and is not in the repository: the bytes of rust-anything (the version alone does not tell two builds apart) and
    the cargo configuration files that apply to the checkout."""
    cargo_home = environment.get("CARGO_HOME") or os.path.join(os.path.expanduser("~"), ".cargo")
    folders, current = [cargo_home], os.path.abspath(root)
    while True:
        folders.append(os.path.join(current, ".cargo"))
        if os.path.dirname(current) == current:
            break
        current = os.path.dirname(current)
    configurations = {path: runner.file_digest(path) for folder in folders for path in (os.path.join(folder, "config"), os.path.join(folder, "config.toml"))
                      if os.path.exists(path)}
    binary = runner.file_digest(rust_anything) if rust_anything and os.path.exists(rust_anything) else None
    return {"rust-anything-binary": binary, "cargo-configuration": configurations}


def control_result(code, log_text):
    """-> (verdict, replayed cases). A clean exit that does not say how much it replayed cannot vouch for the mutants' corpus size: ERROR."""
    counted = re.search(r"\| control \(unmodified\) \| CLEAN \| \d+/(\d+) \|", log_text)
    if code == 0:
        return ("CLEAN", int(counted.group(1))) if counted else ("ERROR", None)
    return ("FAILED" if code == 1 else "ERROR"), None


def run_selected(names, key_now, store, execute, run_control, reusable, reuse=True, state_now=lambda: None, save=lambda report: None):
    """Control first; then each mutant, reused only if its stored DETECTED belongs to the identity as it is now. Each proof is stored, and the
    report saved, before the next mutant starts, so an interrupted run keeps both."""
    checked_state = state_now()
    control, corpus_size = run_control()
    report = {"control": control, "executed": [], "reused": [], "reused_from": {}, "outcomes": {}, "not_executed": list(names)}
    reused_keys = {}
    save(report)
    for name in names if control == "CLEAN" else []:
        key = key_now(name)
        if reuse and key is not None and reusable(name) and store.is_fresh(key):
            report["reused"].append(name)
            report["reused_from"][name] = store.get(key).get("run")
            report["outcomes"][name] = "DETECTED"
            reused_keys[name] = key
        else:
            record = execute(name)
            report["executed"].append(name)
            report["not_executed"].remove(name)
            total = record.get("total")
            if key_now(name) != key:
                report["outcomes"][name] = "INPUTS_CHANGED"
            elif total is not None and corpus_size is not None and total != corpus_size:
                report["outcomes"][name] = "CORPUS_MISMATCH"     # a partial replay is an invalid experiment, whatever it says
            else:
                report["outcomes"][name] = record["outcome"]
                if key is not None and reusable(name) and record["outcome"] == "DETECTED":
                    store.put(key, {**record, "name": name})
        save(report)
    for name, key in reused_keys.items():
        if key_now(name) != key:
            report["outcomes"][name] = "INPUTS_CHANGED"
    if state_now() != checked_state:
        report["outcomes"]["(control)"] = "INPUTS_CHANGED"          # the clean control vouches only for the state it ran on
    undetected = [name for name, outcome in report["outcomes"].items() if outcome == "UNDETECTED"]
    errors = [name for name, outcome in report["outcomes"].items() if outcome not in ("DETECTED", "UNDETECTED")]
    report["exit"] = runner.exit_code(undetected, errors, [], control)
    save(report)
    return report


def main(arguments):
    sys.path.insert(0, os.path.join(ROOT, "tools", "parity"))
    import check_divergence as cd
    import clean_replay_evidence as evidence
    reuse = "--no-reuse" not in arguments
    names = [argument for argument in arguments if argument != "--no-reuse"]
    definitions = {**cd.MUTANTS, **cd.TOOL_MUTANTS}
    unknown = [name for name in names if name not in definitions]
    if unknown or not names:
        print(f"refused: unknown or missing mutant names {unknown}")
        return 2
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(143))

    def tool_identity():
        external = external_identity(ROOT, os.environ, shutil.which("rust-anything"))
        return {**evidence.current_tool_versions(), "rust-anything-binary": external["rust-anything-binary"]}, external["cargo-configuration"]

    tools, cargo_configuration = tool_identity()
    environment = {name: value for name, value in sorted(os.environ.items()) if name.startswith(evidence.ENVIRONMENT_PREFIXES)}

    def repository_state():
        return {"crate": runner.crate_fingerprint(ROOT), "harness": runner.harness_fingerprint(ROOT) + runner.file_digest(__file__)}

    def key_now(name):
        state = {**repository_state(), "tools": tools, "environment": environment, "cargo-configuration": cargo_configuration}
        return reuse_key(name, [*definitions[name][:3], *cd.UNIT_MUTANTS.get(name, ())], state)

    run_directory = runner.new_run_directory(os.path.join(STORE_DIR, "runs"))
    run_name = os.path.relpath(run_directory, ROOT)
    script = os.path.join(ROOT, "tools", "parity", "check_divergence.py")

    def log_path(log_name):
        return os.path.join(run_directory, log_name + ".log")

    def launch(log_name, *check_arguments):
        return runner.run_cancellable([runner.PY, script, *check_arguments], runner.PER_MUTANT_TIMEOUT, log_path(log_name), cwd=ROOT)

    def execute(name):
        result_file = runner.mutant_result_file(run_directory, name)
        started = time.time()
        record = runner.execute_mutant(name, result_file, lambda: launch(name, name, "--skip-control", f"--result-json={result_file}"),
                                       lambda: time.time() - started)
        print(f"{record['outcome']} {name} ({record['seconds']}s)", flush=True)
        return {**record, "run": run_name}

    def run_control():
        code = launch("control", "--control-only")
        with open(log_path("control"), errors="replace") as log:
            return control_result(code, log.read())

    def save(report):
        report.update(rules=["Q-X-22 R3"] if report["reused"] else [], selected=names, tools=tools, reuse_requested=reuse)
        with open(os.path.join(run_directory, "report.json"), "w") as out:
            json.dump(report, out, indent=1)

    report = run_selected(names, key_now, runner.ResultStore(STORE_DIR), execute, run_control, lambda name: name not in cd.TOOL_MUTANTS,
                          reuse=reuse, state_now=lambda: (repository_state(), tool_identity()), save=save)
    detected = sum(1 for outcome in report["outcomes"].values() if outcome == "DETECTED")
    print(f"MUTANTS detected {detected}/{len(names)} executed {len(report['executed'])} reused {len(report['reused'])} control {report['control']}")
    for name, outcome in report["outcomes"].items():
        if outcome != "DETECTED":
            print(f"{outcome} {name}")
    print(f"logs and report: {run_name}")
    return report["exit"]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
