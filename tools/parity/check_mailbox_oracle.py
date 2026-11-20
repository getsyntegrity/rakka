#!/usr/bin/env python3
"""Checks the Pekko 2.0.0 mailbox oracle (ADR-0001, PR 1/10) against its golden and against itself.

The cases are oracle-only: Rust has no MailboxStatus/MessageQueue yet, so they are not part of `rust-anything app replay`
(a recorded case would diverge by construction). The golden is what the real JVM printed; nothing here is derived from Rust.

  tools/parity/check_mailbox_oracle.py [--runs N]   run `oracle/run.sh mailbox-all` N times (default 3): every run must be
                                                    identical to the others and to cases/oracle-only/mailbox.golden
  tools/parity/check_mailbox_oracle.py --record     rewrite the golden from one JVM run (review the diff before committing)
  tools/parity/check_mailbox_oracle.py --selftest   perturb the golden (swap, drop, lose a message) and require the check to fail;
                                                    exits 1 on success, like check_divergence.py --selftest
"""
import argparse, os, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
GOLDEN = os.path.join(ROOT, "cases", "oracle-only", "mailbox.golden")
MANIFEST = os.path.join(ROOT, "cases", "oracle-only", "mailbox-cases.tsv")


def case_id(line):
    return line.split(" -> ", 1)[0]


def compare(golden, runs):
    """Problems between the golden and each run, and between the runs. Empty list = pass."""
    problems = []
    for n, run in enumerate(runs, 1):
        if run != runs[0]:
            problems.append(f"unstable: run {n} differs from run 1")
        expected, got = {case_id(l): l for l in golden}, {case_id(l): l for l in run}
        for cid in expected.keys() - got.keys():
            problems.append(f"{cid}: missing in run {n}")
        for cid in got.keys() - expected.keys():
            problems.append(f"{cid}: not in the golden (run {n})")
        for cid in expected.keys() & got.keys():
            if expected[cid] != got[cid]:
                problems.append(f"{cid}: differs from the golden (run {n})\n    golden: {expected[cid]}\n    run:    {got[cid]}")
    for l in golden:
        if " -> ERR " in l:
            problems.append(f"{case_id(l)}: the golden holds an ERR line: {l}")
    return problems


def run_oracle():
    r = subprocess.run([os.path.join(ROOT, "oracle", "run.sh"), "mailbox-all"], stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=300)
    if r.returncode != 0:
        sys.exit(f"oracle failed ({r.returncode}): {r.stderr.strip()[-300:]}")
    return r.stdout.splitlines()


def read_golden():
    with open(GOLDEN, encoding="ascii") as fh:
        return fh.read().splitlines()


def manifest_problems(golden):
    with open(MANIFEST, encoding="utf-8") as fh:
        rows = [l.split("\t") for l in fh.read().splitlines()[1:]]
    ids = [r[0] for r in rows]
    if ids != [case_id(l) for l in golden]:
        return ["cases/oracle-only/mailbox-cases.tsv: ids differ from the golden (same ids, same order required)"]
    return [f"{r[0]}: incomplete manifest row" for r in rows if len(r) != 5 or not all(r)]


def selftest():
    golden = read_golden()
    swapped = [l.replace("r:x1, r:x2, r:y1", "r:x2, r:x1, r:y1") for l in golden]
    lost = [l.replace("deadLetters=[u2, u3, u4, after]", "deadLetters=[u2, u3, after]") for l in golden]
    state = [l.replace("suspendCount=100", "suspendCount=99") for l in golden]
    mutants = {"reordered messages": swapped, "lost dead letter": lost, "lost suspend increment": state, "dropped case": golden[1:]}
    unchanged = [name for name, m in mutants.items() if m == golden]
    if unchanged:
        sys.exit(f"selftest: mutants that do not change the golden: {unchanged}")
    caught = {name: bool(compare(golden, [m])) for name, m in mutants.items()}
    for name, ok in caught.items():
        print(f"selftest: {name}: {'caught' if ok else 'NOT CAUGHT'}")
    return 1 if all(caught.values()) else 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--record", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        sys.exit(selftest())
    if a.record:
        with open(GOLDEN, "w", encoding="ascii") as fh:
            fh.write("".join(l + "\n" for l in run_oracle()))
        return
    golden = read_golden()
    problems = manifest_problems(golden) + compare(golden, [run_oracle() for _ in range(a.runs)])
    for p in problems:
        print("FAIL", p)
    print(f"mailbox oracle: {len(golden)} cases x {a.runs} runs, {'OK' if not problems else str(len(problems)) + ' problems'}")
    sys.exit(1 if problems else 0)


if __name__ == "__main__":
    main()
