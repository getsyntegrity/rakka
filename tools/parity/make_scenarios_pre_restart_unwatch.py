#!/usr/bin/env python3
"""Writes cases/scenario-pre-restart-unwatch.jsonl: Pekko's default `preRestart` unwatches its children before it stops them
(Actor.scala:627-633, AC-CELL-006), and what an override that calls it, or does not, gets after the restart."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, DO, EV, NOW, TELL, STOP, STOPC, WATCH, CHILDREN  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
WATCH_WITH = DO("p", "watch-with", target="c", msg="gone")

def restart(events, watch=WATCH("p", "c"), **parent):
    """`p` watches its child `c`, fails and is restarted by `/user`; `events` is how many events `p` has once it is back"""
    return [*S("p", throw_on="boom", **parent), C("p", "c", "c"), EV("c", 1), watch, TELL("p", "boom"), EV("p", events)]

def then_ping(events):
    """`ping` is queued behind anything the restart left in the mailbox"""
    return [CHILDREN("p"), TELL("p", "ping"), EV("p", events), NOW("p")]

CASES = {
    # the actor does not override preRestart: the child's termination is not told, the non-child `t` stays watched
    "ur-default": [*S("t"), *S("p", throw_on="boom", default_pre_restart=True), C("p", "c", "c"), EV("c", 1), WATCH("p", "c"),
                   WATCH("p", "t"), TELL("p", "boom"), EV("p", 5), *then_ping(6), STOP("t"), EV("p", 7), NOW("p")],
    "ur-default-pact": [*restart(5, default_pre_restart=True, ignore_terminated=True), *then_ping(6)],
    "ur-default-watch-with": [*restart(5, WATCH_WITH, default_pre_restart=True), *then_ping(6)],
    "ur-calls-super": [*restart(6), *then_ping(7)],
    # an override that does not call super keeps the watch
    "ur-no-super": [*restart(7, stop_without_unwatch=True), *then_ping(8)],
    "ur-no-super-watch-with": [*restart(7, WATCH_WITH, stop_without_unwatch=True), *then_ping(8)],
    "ur-no-super-pact": [*restart(6, stop_without_unwatch=True, ignore_terminated=True), {"op": "await-stopped", "ref": "p"}, NOW("p")],
    "ur-no-super-keeps": [*restart(5, keep_children=True), CHILDREN("p"), STOPC("p", "c"), EV("p", 6), NOW("p")],
}

def main():
    assert len({n[:24] for n in CASES}) == len(CASES) and max(map(len, CASES)) <= 24
    out = os.path.join(ROOT, "cases", "scenario-pre-restart-unwatch.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")

if __name__ == "__main__":
    main()
