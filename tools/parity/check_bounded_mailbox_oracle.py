#!/usr/bin/env python3
"""Checks the Pekko 2.0.0 BoundedMailbox oracle (ADR-0001 PR 7a) against its golden and against itself.

Oracle-only like the PR 1 mailbox oracle (check_mailbox_oracle.py): Rust has no BoundedMailbox yet, so no case is recorded
for `rust-anything app replay` (a recorded case would diverge by construction). The golden is what the real JVM printed.

  tools/parity/check_bounded_mailbox_oracle.py [--runs N]   run `oracle/run.sh bounded-mailbox-all` N times (default 3): every run
                                                            identical to the others and to cases/oracle-only/bounded-mailbox.golden
  tools/parity/check_bounded_mailbox_oracle.py --per-case   run each case alone (`bounded-mailbox-case`) and compare with the golden
  tools/parity/check_bounded_mailbox_oracle.py --record     rewrite the golden from one JVM run (review the diff before committing)
  tools/parity/check_bounded_mailbox_oracle.py --selftest   perturb the golden and require the check to fail; exits 1 on success
"""
import argparse, os, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from check_mailbox_oracle import compare, case_id  # noqa: E402  (same comparison rules as the PR 1 oracle)

ROOT = os.path.dirname(os.path.dirname(HERE))
GOLDEN = os.path.join(ROOT, "cases", "oracle-only", "bounded-mailbox.golden")
MANIFEST = os.path.join(ROOT, "cases", "oracle-only", "bounded-mailbox-cases.tsv")
ORACLE = os.path.join(ROOT, "oracle", "run.sh")


def run_oracle():
    r = subprocess.run([ORACLE, "bounded-mailbox-all"], stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=300)
    if r.returncode != 0:
        sys.exit(f"oracle failed ({r.returncode}): {r.stderr.strip()[-300:]}")
    return r.stdout.splitlines()


def run_case(cid):
    r = subprocess.run([ORACLE, "bounded-mailbox-case"], input=cid, capture_output=True, text=True, timeout=120)
    if r.returncode != 0:
        sys.exit(f"oracle failed for {cid!r} ({r.returncode}): {r.stderr.strip()[-300:]}")
    return r.stdout.rstrip("\n")


def read_golden():
    with open(GOLDEN, encoding="ascii") as fh:
        return fh.read().splitlines()


def manifest_problems(golden):
    with open(MANIFEST, encoding="utf-8") as fh:
        rows = [l.split("\t") for l in fh.read().splitlines()[1:]]
    if [r[0] for r in rows] != [case_id(l) for l in golden]:
        return ["cases/oracle-only/bounded-mailbox-cases.tsv: ids differ from the golden (same ids, same order required)"]
    return [f"{r[0]}: incomplete manifest row" for r in rows if len(r) != 5 or not all(r)]


def selftest():
    golden = read_golden()

    def mutate(old, new):
        return [l.replace(old, new) for l in golden]

    mutants = {
        "overflow kept instead of refused": mutate("log=[block, m1, m2, m3] queued=0", "log=[block, m1, m2, m3, m4] queued=0"),
        "refused messages reordered": mutate("dead=[m4<-snd->w, m5<-snd->w, m6<-snd->w]", "dead=[m5<-snd->w, m4<-snd->w, m6<-snd->w]"),
        "lost dead letter": mutate("dead=[d<-snd->recv, e<-snd->recv]", "dead=[d<-snd->recv]"),
        "zero timeout waits": mutate("BM-QU-05 timeout > 0: the producer parks for at least the timeout, then the message is a dead letter -> waitedAtLeastTimeout=true", "BM-QU-05 timeout > 0: the producer parks for at least the timeout, then the message is a dead letter -> waitedAtLeastTimeout=false"),
        "negative timeout times out": mutate("state=WAITING n=1 has=true remaining=0 | first=a producerAlive=false rest=[b] dead=[]", "state=WAITING n=1 has=true remaining=0 | first=a producerAlive=false rest=[] dead=[b<-recv->recv]"),
        "capacity zero accepted by create": mutate("cap=0 => ERR IllegalArgumentException [null]", "cap=0 => org.apache.pekko.dispatch.BoundedMailbox$MessageQueue n=0 has=false remaining=0"),
        "system message refused when full": mutate("hasSystemMessages=true dead=[]", "hasSystemMessages=false dead=[]"),
        "dropped case": golden[1:],
    }
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
    ap.add_argument("--per-case", action="store_true")
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
    problems = manifest_problems(golden)
    if a.per_case:
        problems += compare(golden, [[run_case(case_id(l)) for l in golden]])
        label = "per-case"
    else:
        problems += compare(golden, [run_oracle() for _ in range(a.runs)])
        label = f"{a.runs} runs"
    for p in problems:
        print("FAIL", p)
    print(f"bounded mailbox oracle: {len(golden)} cases x {label}, {'OK' if not problems else str(len(problems)) + ' problems'}")
    sys.exit(1 if problems else 0)


if __name__ == "__main__":
    main()
