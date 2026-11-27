#!/usr/bin/env python3
"""Writes cases/scenario-guardian-failure.jsonl: `Kill` fails a guardian and the root guardian's strategy stops it, which ends
the system (actor/ActorRefProvider.scala:322-381, 525-529). Only public API: `actorSelection` reaches the guardians. No actor
collects the errors: it would live under /user, which stops while the errors are on their way (MBX-GRD-02/04 record them)."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
KILL = {"special": "Kill"}
ENDED = {"op": "await-system-terminated"}
TREE = [*S("a"), C("a", "c", "c"), EV("c", 1)]
AFTER = [ENDED, {"op": "await-life", "count": 4, "before": [["c:post-stop", "a:post-stop"]]}, NOW("a"), NOW("c")]
CASES = {name: [*TREE, {"op": "select", "path": path, "msg": KILL}, *AFTER]
         for name, path in (("gf-kill-user", "/user"), ("gf-kill-system", "/system"))}


def main():
    out = os.path.join(ROOT, "cases", "scenario-guardian-failure.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
