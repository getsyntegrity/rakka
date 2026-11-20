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
import dataclasses, hashlib, json, os, subprocess, sys, time

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
GATES_DIR = os.path.join(ROOT, ".gates")
DEFAULT_BASE = "origin/feat/pekko-rust-pilot-migration"
MAX_FAILURE_CHARS = 2000
CRATE_PREFIX = "crates/rakka-actor/"
PY = "python3.12"

# Changes here can alter what any mutant does or how it is judged: every mutant is selected.
GLOBAL_MUTANT_TRIGGERS = ("Cargo.toml", "Cargo.lock", "cases/", "rust/app-cases/", "oracle/", "tools/parity/check_divergence.py",
                          "tools/parity/replay.py", "crates/rakka-actor/Cargo.toml", "crates/rakka-actor/examples/")


def select_mutants(changed_files, mutants, tool_mutants):
    """Names of the mutants a change must re-check; [] when no mutated file changed."""
    changed = list(changed_files)
    if any(f == t or f.startswith(t) for f in changed for t in GLOBAL_MUTANT_TRIGGERS):
        return list(mutants) + list(tool_mutants)
    picked = [n for n, (rel, *_rest) in mutants.items() if CRATE_PREFIX + rel in changed]
    picked += [n for n, (rel, *_rest) in tool_mutants.items() if "tools/parity/" + rel in changed]
    return picked


def evidence_key(command, input_trees, toolchain, dirty_inputs):
    """Cache key of a passing gate, or None when evidence must not be reused (uncommitted edits in the inputs)."""
    if dirty_inputs:
        return None
    payload = json.dumps({"command": command, "inputs": input_trees, "toolchain": toolchain}, sort_keys=True)
    return hashlib.sha256(payload.encode()).hexdigest()


def verdict_line(name, exit_code, seconds, reused, log_path, log_text):
    if exit_code == 0:
        return f"PASS {name} (reused evidence) log={log_path}" if reused else f"PASS {name} ({seconds:.0f}s) log={log_path}"
    tail = "\n".join(log_text.splitlines()[-20:])[-MAX_FAILURE_CHARS:]
    return f"FAIL {name} exit={exit_code} ({seconds:.0f}s) log={log_path}\n{tail}"


@dataclasses.dataclass(frozen=True)
class Gate:
    name: str
    command: list
    inputs: tuple            # repo paths whose git tree ids key the evidence
    skip_reason: str = ""    # non-empty: nothing to run (reported, never silent)


RUST_INPUTS = ("crates", "Cargo.toml", "Cargo.lock")
PARITY_INPUTS = RUST_INPUTS + ("cases", "tools/parity", "oracle")
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
        Gate("replay", ["tools/gates/replay.sh"], PARITY_INPUTS),
    ]
    mutants = Gate("mutants", [PY, "tools/parity/check_divergence.py", *selected_mutants], PARITY_INPUTS,
                   "" if selected_mutants else "no mutated file changed since the base")
    if tier == "fast":
        return fast
    if tier == "pr":
        return fast + pr_extra + [mutants]
    # budgeted + resumable (tools/gates/mutant_runner.py): never all 206 without --authorize-full-mutants
    full_mutants = Gate("mutants", [PY, "tools/gates/mutant_runner.py", *mutant_args], PARITY_INPUTS + ("tools/gates", "rust/app-cases"))
    fuzz = [Gate("fuzz-" + s[5:-3], [PY, "-u", f"tools/parity/{s}", "--count", str(n), "--seed", str(seed)], PARITY_INPUTS)
            for s, n in FUZZ_COUNTS for seed in (1, 2, 3)]
    return fast + pr_extra + [full_mutants] + fuzz


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()


def input_state(paths):
    trees = {p: git("rev-parse", f"HEAD:{p}") for p in paths if os.path.exists(os.path.join(ROOT, p))}
    # files the gates themselves rewrite on every run are not "uncommitted edits" of the inputs
    generated = (":(exclude)rust/app-cases/replay-report.md", ":(exclude)rust/divergence-reports")
    dirty = bool(git("status", "--porcelain", "--untracked-files=normal", "--", *trees, *generated))
    return trees, dirty


def run_gate(gate, toolchain, reuse, head):
    log_dir = os.path.join(GATES_DIR, "logs", head)
    os.makedirs(log_dir, exist_ok=True)
    log_path = os.path.relpath(os.path.join(log_dir, gate.name + ".log"), ROOT)
    if gate.skip_reason:
        return f"SKIP {gate.name}: {gate.skip_reason}", 0
    trees, dirty = input_state(gate.inputs)
    key = evidence_key(gate.command, trees, toolchain, dirty)
    evidence = os.path.join(GATES_DIR, "evidence", f"{gate.name}-{key}.json") if key else None
    if reuse and evidence and os.path.exists(evidence):
        return verdict_line(gate.name, 0, 0.0, True, json.load(open(evidence))["log"], ""), 0
    started = time.time()
    with open(os.path.join(ROOT, log_path), "w") as log:
        code = subprocess.run(gate.command, cwd=ROOT, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT).returncode
    seconds = time.time() - started
    text = open(os.path.join(ROOT, log_path)).read() if code else ""
    if code == 0 and evidence:
        os.makedirs(os.path.dirname(evidence), exist_ok=True)
        json.dump({"gate": gate.name, "command": gate.command, "head": head, "log": log_path, "seconds": seconds}, open(evidence, "w"))
    return verdict_line(gate.name, code, seconds, False, log_path, text), code


def changed_files(base):
    committed = git("diff", "--name-only", f"{base}...HEAD").splitlines()
    working = git("status", "--porcelain").splitlines()
    return sorted(set(committed) | {line[3:] for line in working})


def main(argv):
    tiers = [a for a in argv if a in ("fast", "pr", "milestone")]
    if len(tiers) != 1:
        sys.exit(__doc__)
    base = argv[argv.index("--base") + 1] if "--base" in argv else DEFAULT_BASE
    selected = []
    if tiers[0] == "pr":
        sys.path.insert(0, os.path.join(ROOT, "tools", "parity"))
        import check_divergence
        selected = select_mutants(changed_files(base), check_divergence.MUTANTS, check_divergence.TOOL_MUTANTS)
    mutant_args = []
    for flag in ("--max-mutants", "--max-minutes", "--base"):
        if flag in argv:
            mutant_args += [flag, argv[argv.index(flag) + 1]]
    mutant_args += [f for f in ("--authorize-full-mutants", "--no-fast-kill") if f in argv]
    gates = tier_gates(tiers[0], selected, mutant_args)
    if "--list" in argv:
        for g in gates:
            print(g.name, " ".join(g.command) if not g.skip_reason else f"(skip: {g.skip_reason})")
        return 0
    toolchain = subprocess.run(["rustc", "-V"], capture_output=True, text=True).stdout.strip()
    head = git("rev-parse", "--short=12", "HEAD")
    failed = []
    for g in gates:
        line, code = run_gate(g, toolchain, "--no-reuse" not in argv, head)
        print(line, flush=True)
        if code:
            failed.append(g.name)
    print(f"{tiers[0].upper()}: {'FAILED ' + ', '.join(failed) if failed else 'all gates passed'} ({len(gates)} gates, HEAD {head})")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
