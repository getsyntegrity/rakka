#!/usr/bin/env python3
"""The `python-tests` gate of the PR tier: the Python tests of tools/gates, tools/package and tools/parity.

  python3.12 tools/gates/python_tests.py        (not with -I: the test modules import their neighbours from the working directory)

Each suite runs in its own process, from its own directory. No test run here may execute a mutant or a check_divergence campaign (Q-X-23):
the files that run the mutation machinery are in EXCLUDED, with the reason. A new test file is picked up by itself: read it before merging it.

Derived-from: none (project tooling, Apache-2.0).
"""
import os, subprocess, sys

import gates

SUITES = ("tools/gates", "tools/package/tests", "tools/parity")
EXCLUDED = {
    "tools/parity/test_tool_mutant_isolation.py": "runs check_divergence.run_tool_mutant, the tool-mutant execution itself",
    "tools/parity/test_hermetic_builds_integration.py": "builds deliberately broken copies of the crate through the mutation harness with real cargo",
}


def test_modules(suite):
    return sorted(name[:-len(".py")] for name in os.listdir(os.path.join(gates.ROOT, suite))
                  if name.startswith("test_") and name.endswith(".py") and f"{suite}/{name}" not in EXCLUDED)


def main():
    # without the run id, nothing a test leaves can pass for evidence of this run's gates
    environment = {name: value for name, value in os.environ.items() if name != gates.RUN_ID_ENV}
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    failed = []
    for path, reason in EXCLUDED.items():
        print(f"NOT RUN {path}: {reason}")
    for suite in SUITES:
        modules = test_modules(suite)
        if not modules:      # `python -m unittest` without names would run a discovery, exclusions included
            print(f"FAIL {suite}: no test file")
            failed.append(suite)
            continue
        print(f"== {suite}: {len(modules)} modules", flush=True)
        if subprocess.run([sys.executable, "-m", "unittest", *modules], cwd=os.path.join(gates.ROOT, suite), env=environment).returncode != 0:
            failed.append(suite)
    print(f"FAILED suites: {', '.join(failed)}" if failed else "all suites passed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
