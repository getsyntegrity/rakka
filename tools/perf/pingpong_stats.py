#!/usr/bin/env python3
"""Timing statistics of the single consumer mailbox ping pong, comparing prebuilt test binaries.

Each round runs every binary once, alternating the order to spread machine drift. The time is libtest's `finished in`; a run
whose test fails (the 30 s of `MailboxConfigSpec` among other reasons) is counted in `failed` and never retried. Documented in
`docs/evidence/dispatcher-queue-contention.md`.

    python3.12 tools/perf/pingpong_stats.py --binary before=PATH --binary after=PATH --runs 30 --log runs.tsv
"""

import argparse
import math
import os
import re
import statistics
import subprocess
import sys
import time

TESTS = {
    "unbounded": "pathological_ping_pong_for_the_unbounded_single_consumer_mailbox",
    "bounded": "pathological_ping_pong_for_the_bounded_single_consumer_mailbox",
}
FINISHED = re.compile(r"finished in ([0-9.]+)s")
LOAD_POLL_SECONDS = 15
RUN_TIMEOUT_SECONDS = 600


def wait_for_load_below(limit, give_up_after):
    deadline = time.monotonic() + give_up_after
    while os.getloadavg()[0] > limit:
        if time.monotonic() > deadline:
            sys.exit(f"load stayed above {limit} for {give_up_after} s: not an idle measurement")
        time.sleep(LOAD_POLL_SECONDS)


def run_once(binary, test, background):
    command = ["taskpolicy", "-c", "background"] if background else []
    command.append(binary)
    if test != "both":
        command += ["--exact", TESTS[test]]
    environment = {**os.environ, "LLVM_PROFILE_FILE": os.devnull}
    done = subprocess.run(command, capture_output=True, text=True, env=environment, timeout=RUN_TIMEOUT_SECONDS)
    finished = FINISHED.search(done.stdout)
    if finished is None:
        sys.exit(f"{binary} printed no timing:\n{done.stdout}{done.stderr}")
    return float(finished.group(1)), done.returncode == 0


def summary(seconds):
    ordered = sorted(seconds)
    p95 = ordered[math.ceil(0.95 * len(ordered)) - 1]  # nearest rank
    return f"n={len(ordered)} min={ordered[0]:.2f} median={statistics.median(ordered):.2f} p95={p95:.2f} max={ordered[-1]:.2f}"


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--binary", action="append", required=True, metavar="LABEL=PATH")
    parser.add_argument("--runs", type=int, required=True)
    parser.add_argument("--log", required=True, help="one line per run")
    parser.add_argument("--together", action="store_true", help="both tests in one process, as a test gate runs them")
    parser.add_argument("--burners", type=int, default=0, help="CPU bound processes running meanwhile")
    parser.add_argument("--background", action="store_true", help="run the tests in the background QoS class")
    parser.add_argument("--max-load", type=float, help="start a round only when the 1 min load is below this")
    parser.add_argument("--load-wait", type=int, default=3600, help="seconds to wait for --max-load")
    options = parser.parse_args()
    binaries = [entry.split("=", 1) for entry in options.binary]
    tests = ["both"] if options.together else list(TESTS)
    results = {(label, test): [] for label, _ in binaries for test in tests}
    burners = [subprocess.Popen(["yes"], stdout=subprocess.DEVNULL) for _ in range(options.burners)]
    try:
        with open(options.log, "a", encoding="utf-8") as log:
            for round_number in range(options.runs):
                if options.max_load is not None:
                    wait_for_load_below(options.max_load, options.load_wait)
                for test in tests:
                    for label, binary in binaries if round_number % 2 == 0 else binaries[::-1]:
                        load = os.getloadavg()[0]
                        seconds, passed = run_once(binary, test, options.background)
                        results[(label, test)].append((seconds, passed, load))
                        print(label, test, round_number, seconds, passed, f"{load:.1f}", sep="\t", file=log, flush=True)
    finally:
        for burner in burners:
            burner.kill()
            burner.wait()
    for (label, test), runs in results.items():
        loads = [load for _, _, load in runs]
        failed = sum(not passed for _, passed, _ in runs)
        print(f"{label} {test}: {summary([s for s, _, _ in runs])} failed={failed} load={min(loads):.1f}..{max(loads):.1f}")


if __name__ == "__main__":
    main()
