#!/usr/bin/env python3
"""Writes cases/scenario-child-names.jsonl: a child keeps its name until its parent processes its termination (AC-REF-009.2:
LocalActorRefProviderSpec.scala:155-165, SupervisorMiscSpec.scala:101-141) and `child`/`children` see the children by name
(AC-REF-010: ActorRefSpec.scala:465-477). Every case is deterministic."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, DO, EV, NOW, TELL, CHILD, CHILDREN, GONE  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def in_turn(ref, *commands):
    return DO(ref, "in-turn", cmds=[{"c": c, **kw} for c, kw in commands])


STOP_K = ("stop", {"target": "k"})
AGAIN = ("spawn-child", {"name": "foo"})
FOO = ("child", {"name": "foo"})
WITH_KID = [*S("p"), C("p", "foo", "k"), EV("k", 1)]
AFTER_GONE = [GONE("p", "foo"), C("p", "foo", "k2"), EV("k2", 1), {"op": "compare-refs", "a": "k", "b": "k2"}]


def kids(*names):
    return [("spawn-child", {"name": n, "as": n}) for n in names]


CASES = {
    # the second creation of a name in one message fails the actor; it is logged once
    "cn-duplicate-in-one-message": [{"op": "subscribe", "channel": "errors", "prefix": "actor name", "as": "lg"},
                                    *S("s", kind="spawn-twice", msg="duplicate"), TELL("s", "go"), EV("s", 6), EV("lg", 1),
                                    {"op": "await-quiet", "target": "lg", "count": 1, "ms": 200}, CHILDREN("s")],
    # a stopped child keeps its name in the same turn
    "cn-stop-then-recreate-same-turn": [*WITH_KID, in_turn("p", STOP_K, AGAIN, FOO, ("children", {})), *AFTER_GONE],
    # ... also once it is already terminated, as long as the parent has not processed its termination
    "cn-terminated-unprocessed-keeps-name": [*WITH_KID, in_turn("p", STOP_K, ("await-terminated", {"target": "k"}), AGAIN,
                                                                 FOO, ("children", {})),
                                             *AFTER_GONE],
    # `children` is in name order, whatever the order of creation (observable, reproduced, not a documented Pekko guarantee); a
    # stopping child stays in it until its termination is processed
    "cn-children-in-name-order": [*S("p"), in_turn("p", *kids("b", "a", "10", "9", "a-b", "B"), ("spawn-child", {}),
                                                   ("children", {"unsorted": True})),
                                  in_turn("p", ("stop", {"target": "a"}), ("children", {"unsorted": True})), GONE("p", "a"),
                                  DO("p", "children", unsorted=True)],
    # `child` of a name that exists, one that never did and one that is gone (`getChild`/`findChild`: no case, Q-REF-13)
    "cn-existence-of-children": [*S("p"), C("p", "child", "k"), EV("k", 1), CHILD("p", "child"), CHILD("p", "whatnot"),
                                 DO("p", "stop", target="k"), GONE("p", "child"), CHILD("p", "child"), NOW("k")],
}


def main():
    assert len({name[:24] for name in CASES}) == len(CASES), "case names must differ in the first 24 chars"
    out = os.path.join(ROOT, "cases", "scenario-child-names.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
