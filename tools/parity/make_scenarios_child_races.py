#!/usr/bin/env python3
"""Writes cases/scenario-child-races.jsonl: a child name is reserved before its actor exists and only one of several simultaneous
creations wins it (AC-REF-009.2: LocalActorRefProviderSpec.scala:133-153).

`cn-reserved-name` is deterministic. In `cnc-*` cases several threads call `system.actorOf` with one name at once; they are also
meant to be run in parallel. `cnc-held-*` hit the race by construction (the first creation is held with its name reserved);
`cnc-four-*` reports that calls did overlap."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, EV  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
RELEASE = {"op": "release-held"}


def identify(path, correlation):
    return {"op": "select", "path": path, "msg": {"special": "Identify", "id": correlation}, "from": "w"}


def held(name):
    return {"op": "spawn-held", "name": name}


def at_once(threads, rounds=1, **kw):
    return {"op": "spawn-concurrently", "threads": threads, "rounds": rounds, **kw}


CASES = {
    # a reserved name: taken, found by no selection, not among the children a wildcard sees
    "cn-reserved-name": [*S("w"), held("kid"), {"op": "spawn", "name": "kid"}, identify("/user/kid", "direct"),
                         identify("/user/*", "all"), EV("w", 3), {"op": "await-quiet", "target": "w", "count": 3, "ms": 200},
                         RELEASE, identify("/user/kid", "made"), EV("w", 4)],
    "cnc-four-at-once-100-names": [at_once(4, 100, prefix="new-actor", overlap=True)],
    "cnc-held-name-4-refused": [held("kid"), at_once(4, name="kid"), RELEASE],
    "cnc-held-name-16-refused": [held("kid"), at_once(16, name="kid"), RELEASE],
}


def main():
    assert len({name[:24] for name in CASES}) == len(CASES), "case names must differ in the first 24 chars"
    out = os.path.join(ROOT, "cases", "scenario-child-races.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
