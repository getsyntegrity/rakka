#!/usr/bin/env python3
"""Ping pong timings of prebuilt test binaries, interleaved; see docs/evidence/dispatcher-queue-contention.md.

    caffeinate -i python3.12 -I tools/perf/pingpong_stats.py --binary before=A --binary after=B --runs 30 --log runs.tsv

The time is libtest's `finished in`. Failures (the spec's 30 s among them) are counted, never retried. A run across a
machine sleep (the wall clock ran ahead of the monotonic one) is logged as slept and left out of the statistics.
"""

import argparse, math, os, re, signal, statistics, subprocess, sys, time

TESTS = [f"pathological_ping_pong_for_the_{kind}_single_consumer_mailbox" for kind in ("unbounded", "bounded")]


def run_once(binary, test, background):
    command = (["taskpolicy", "-c", "background"] if background else []) + [binary] + (["--exact", test] if test else [])
    environment = {**os.environ, "LLVM_PROFILE_FILE": os.devnull}
    wall, awake = time.time(), time.monotonic()
    done = subprocess.run(command, capture_output=True, text=True, env=environment, timeout=600)
    slept = (time.time() - wall) - (time.monotonic() - awake) > 1
    ran = re.search(r"(\d+) passed; (\d+) failed", done.stdout)
    if ran is None or ran.groups() == ("0", "0"):  # libtest exits 0 when `--exact` matches nothing
        sys.exit(f"{binary} ran no test (--exact {test}):\n{done.stdout}{done.stderr}")
    return float(re.search(r"finished in ([0-9.]+)s", done.stdout)[1]), done.returncode == 0, slept


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--binary", action="append", required=True, metavar="LABEL=PATH")
    parser.add_argument("--runs", type=int, required=True)
    parser.add_argument("--log", required=True, help="one line per run")
    parser.add_argument("--together", action="store_true", help="both tests in one process, as a test gate runs them")
    parser.add_argument("--burners", type=int, default=0, help="CPU bound processes running meanwhile")
    parser.add_argument("--background", action="store_true", help="run the tests in the background QoS class")
    parser.add_argument("--max-load", type=float, help="start a round only below this 1 min load (waits as long as needed)")
    options = parser.parse_args()
    binaries, results = [entry.split("=", 1) for entry in options.binary], {}
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(1))  # so that `finally` also stops the burners
    burners = [subprocess.Popen(["yes"], stdout=subprocess.DEVNULL) for _ in range(options.burners)]
    try:
        with open(options.log, "a", encoding="utf-8") as log:
            for round_number in range(options.runs):
                while options.max_load is not None and os.getloadavg()[0] > options.max_load:
                    time.sleep(15)
                for test in [None] if options.together else TESTS:
                    for label, binary in binaries if round_number % 2 == 0 else binaries[::-1]:
                        load = os.getloadavg()[0]
                        seconds, passed, slept = run_once(binary, test, options.background)
                        results.setdefault((label, test or "both"), []).append((seconds, passed, slept, load))
                        print(label, test or "both", round_number, seconds, passed, slept, f"{load:.1f}", sep="\t", file=log, flush=True)
    finally:
        for burner in burners:
            burner.kill()
    for (label, test), runs in results.items():
        times = sorted(seconds for seconds, _, slept, _ in runs if not slept)
        loads = [load for *_, load in runs]
        if not times:
            print(f"{label} {test}: every run slept")
            continue
        p95 = times[math.ceil(0.95 * len(times)) - 1]  # nearest rank
        print(f"{label} {test}: n={len(times)} min={times[0]:.2f} median={statistics.median(times):.2f} p95={p95:.2f} max={times[-1]:.2f}"
              f" failed={sum(not passed for _, passed, _, _ in runs)} slept={len(runs) - len(times)} load={min(loads):.1f}..{max(loads):.1f}")


if __name__ == "__main__":
    main()
