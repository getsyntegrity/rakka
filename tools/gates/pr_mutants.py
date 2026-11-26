#!/usr/bin/env python3
"""PR-tier mutants with per-mutant evidence reuse (Q-X-22 R3).

  python3.12 tools/gates/pr_mutants.py NAME...

The control (unmodified crate) runs on every invocation and must be clean before any mutant result counts. Only DETECTED is reused, and only
while its whole identity is unchanged: patch, crate, harness, this file, the build environment and the external tools (rustc, cargo,
rust-anything, python, platform). Tool mutants are never reused: their tests read repository files that the identity does not cover.
Every run keeps its logs and a report.json (what ran, what was reused) under .gates/mutants/pr/runs/; nothing is pruned.
Exit: 0 all detected and control clean; 1 survivor or dirty control; 2 invalid run (timeout, error, inputs changed).

Derived-from: none (project tooling, Apache-2.0).
"""
import hashlib, json, os, signal, sys, time

import mutant_runner as runner

ROOT = runner.ROOT
STORE_DIR = os.path.join(runner.STORE_DIR, "pr")


def reuse_key(name, patch, state):
    """The identity of one result; None (never reuse) when an external tool cannot say which version it is."""
    if any(version is None for version in state["tools"].values()):
        return None
    payload = json.dumps({"name": name, "patch": list(patch), "state": state}, sort_keys=True)
    return hashlib.sha256(payload.encode()).hexdigest()


def run_selected(names, key_now, store, execute, run_control, reusable):
    """Control first; then each mutant, reused only if its stored DETECTED belongs to the identity as it is now. Each proof is stored before
    the next mutant starts, so an interrupted run keeps it."""
    report = {"executed": [], "reused": [], "outcomes": {}, "control": run_control()}
    reused_keys = {}
    for name in names if report["control"] == "CLEAN" else []:
        key = key_now(name)
        if key is not None and reusable(name) and store.is_fresh(key):
            report["reused"].append(name)
            report["outcomes"][name] = "DETECTED"
            reused_keys[name] = key
            continue
        record = execute(name)
        report["executed"].append(name)
        if key_now(name) != key:
            report["outcomes"][name] = "INPUTS_CHANGED"
            continue
        report["outcomes"][name] = record["outcome"]
        if key is not None and reusable(name) and record["outcome"] == "DETECTED":
            store.put(key, {**record, "name": name})
    for name, key in reused_keys.items():
        if key_now(name) != key:
            report["outcomes"][name] = "INPUTS_CHANGED"
    undetected = [name for name, outcome in report["outcomes"].items() if outcome == "UNDETECTED"]
    errors = [name for name, outcome in report["outcomes"].items() if outcome not in ("DETECTED", "UNDETECTED")]
    report["exit"] = runner.exit_code(undetected, errors, [], report["control"])
    return report


def main(names):
    sys.path.insert(0, os.path.join(ROOT, "tools", "parity"))
    import check_divergence as cd
    import clean_replay_evidence as evidence
    definitions = {**cd.MUTANTS, **cd.TOOL_MUTANTS}
    unknown = [name for name in names if name not in definitions]
    if unknown or not names:
        print(f"refused: unknown or missing mutant names {unknown}")
        return 2
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(143))
    tools = evidence.current_tool_versions()
    environment = {name: value for name, value in sorted(os.environ.items()) if name.startswith(evidence.ENVIRONMENT_PREFIXES)}

    def key_now(name):
        state = {"crate": runner.crate_fingerprint(ROOT), "harness": runner.harness_fingerprint(ROOT) + runner.file_digest(__file__),
                 "tools": tools, "environment": environment}
        return reuse_key(name, [*definitions[name][:3], *cd.UNIT_MUTANTS.get(name, ())], state)

    run_directory = runner.new_run_directory(os.path.join(STORE_DIR, "runs"))
    script = os.path.join(ROOT, "tools", "parity", "check_divergence.py")

    def launch(log_name, *arguments):
        return runner.run_cancellable([runner.PY, script, *arguments], runner.PER_MUTANT_TIMEOUT, os.path.join(run_directory, log_name + ".log"), cwd=ROOT)

    def execute(name):
        result_file = runner.mutant_result_file(run_directory, name)
        started = time.time()
        record = runner.execute_mutant(name, result_file, lambda: launch(name, name, "--skip-control", f"--result-json={result_file}"),
                                       lambda: time.time() - started)
        print(f"{record['outcome']} {name} ({record['seconds']}s)", flush=True)
        return record

    def run_control():
        return {0: "CLEAN", 1: "FAILED"}.get(launch("control", "--control-only"), "ERROR")

    report = run_selected(names, key_now, runner.ResultStore(STORE_DIR), execute, run_control, lambda name: name not in cd.TOOL_MUTANTS)
    report.update(rules=["Q-X-22 R3"] if report["reused"] else [], selected=names, tools=tools)
    with open(os.path.join(run_directory, "report.json"), "w") as out:
        json.dump(report, out, indent=1)
    detected = sum(1 for outcome in report["outcomes"].values() if outcome == "DETECTED")
    print(f"MUTANTS detected {detected}/{len(names)} executed {len(report['executed'])} reused {len(report['reused'])} control {report['control']}")
    for name, outcome in report["outcomes"].items():
        if outcome != "DETECTED":
            print(f"{outcome} {name}")
    print(f"logs and report: {os.path.relpath(run_directory, ROOT)}")
    return report["exit"]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
