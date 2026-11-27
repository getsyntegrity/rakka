#!/usr/bin/env python3
"""Writes cases/scenario-guardian-failure.jsonl: `Kill` fails a guardian and the root guardian's strategy stops it, which ends
the system (actor/ActorRefProvider.scala:322-381, 525-529). Only public API: `actorSelection` reaches the guardians."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
KILL = {"special": "Kill"}
ENDED = {"op": "await-system-terminated"}
# every published `Error`; the collector lives under /user, so it only hears the errors while /user is not suspended
ERRORS = {"op": "subscribe", "channel": "errors", "prefix": "", "as": "er"}
TREE = [*S("a"), C("a", "c", "c"), EV("c", 1)]
AFTER = [ENDED, {"op": "await-life", "count": 4, "before": [["c:post-stop", "a:post-stop"]]}, NOW("a"), NOW("c")]


def kill(path):
    return {"op": "select", "path": path, "msg": KILL}


CASES = {
    "gf-kill-user": [*TREE, kill("/user"), *AFTER],
    "gf-kill-system": [ERRORS, *TREE, kill("/system"), EV("er", 2), *AFTER, NOW("er")],
}


def main():
    out = os.path.join(ROOT, "cases", "scenario-guardian-failure.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            # the guardian logs with `log.error`, which the oracle's default level OFF would drop; Rakka publishes it always
            scenario = {"system": "sys", "log-errors": True, "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
