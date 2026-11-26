#!/usr/bin/env python3
"""Writes cases/scenario-stop-mid-restart.jsonl: an actor stopped while its restart waits for a child gets `postStop` on the
instance it was replacing (FaultHandling.scala:247-258, AC-CELL-008.2).

The child is held inside `receive` until `release`, so the restart cannot finish. The stop is sent by the actor's parent from a
command it answers, so the actor has its `Terminate` queued ahead of the news that the child died."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW, TELL, STOPC, WATCH  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
QUIET = lambda count: {"op": "await-quiet", "target": "p", "count": count, "ms": 200}
RELEASE = {"op": "release"}
ERRORS = {"op": "subscribe", "channel": "errors", "as": "e", "prefix": "ps", "watch": "p"}


def restarting(before_failure=(), waiting=4, collect=(), **actor):
    """`g/p`, watched by `w`, fails on a message from `s` while its child `c` is held; `waiting` is how many events `p` has then"""
    return [*S("s"), *S("g"), C("g", "p", "p", throw_on="boom", log_sender=True, **actor), EV("p", 1), C("p", "c", "c", "gate"),
            EV("c", 1), *S("w"), WATCH("w", "p"), *collect, TELL("c", "block"), EV("c", 2), *before_failure,
            TELL("p", "boom", "s"), EV("p", waiting)]


def stopped(waiting, total):
    """`g` stops `p`; nothing happens to `p` until the child is released"""
    return [STOPC("g", "p"), QUIET(waiting), RELEASE, EV("p", total), EV("w", 2), EV("c", 3), NOW("p")]


CASES = {
    # preRestart calls postStop and the stop calls it again, after the strategy is asked about the dead child; no postRestart
    "mr-stop": [*restarting(supervisor={"log-calls": True}), *stopped(4, 6)],
    # an override that never calls postStop: the only one comes from the stop
    "mr-stop-keeps": [*restarting(before_failure=[STOPC("p", "c")], waiting=3, keep_children=True), *stopped(3, 4)],
    # both calls fail: the second Error is published before the watchers are told
    "mr-stop-throws": [*restarting(collect=[ERRORS], post_stop_throws="ps"), EV("e", 1), *stopped(4, 5), EV("e", 3), NOW("e")],
}


def main():
    out = os.path.join(ROOT, "cases", "scenario-stop-mid-restart.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
