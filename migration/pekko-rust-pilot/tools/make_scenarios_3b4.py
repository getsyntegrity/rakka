#!/usr/bin/env python3
"""Writes cases/scenario-3b4.jsonl: suspension of the subtree while an actor that failed on its own waits for its supervisor (slice 3b-4).

`g` supervises `p`; `p` throws `IllegalStateException("boom")` in `receive` and owns an active child `c` (and in some scenarios a grandchild `gc`).
`g`'s decider logs `decide` and then waits on the hold `H` (`supervisor.hold`), so the decision stays pending until the harness opens the hold
(`open-hold`) - a barrier, not a sleep. While it is pending the harness sends messages to the children and checks with `await-quiet` (a bounded wait for an
absence, the only timed step) that they were not processed; after the hold opens recovery is awaited by events only.
Part S: Resume / Restart / Stop, several children, a grandchild. Pekko 2.0.0 decides every expected trace.
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, WATCH, CHILD, CHILDREN, EV, NOW, TELL, DL, DO  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CASES = {}


def case(name, steps, system="sys"):
    assert name not in CASES, name
    assert len({n[:24] for n in CASES}) == len(CASES), f"case names must differ in the first 24 chars: {name}"
    CASES[name] = {"system": system, "steps": steps}


OPEN = {"op": "open-hold", "hold": "H"}
QUIET = lambda t, n, ms=300: {"op": "await-quiet", "target": t, "count": n, "ms": ms}


def top(directive):
    return [*S("g", supervisor={"decider": {"boom": directive}, "log-decisions": True, "hold": "H"}),
            C("g", "p", "p", numbered=True, throw_on=["boom"])]


def kid(parent, name, label=None):
    return C(parent, name, label or name, numbered=True)


PRE = [TELL("c", "m0"), EV("c", 2), TELL("p", "boom"), EV("g", 2)]   # c is busy and p failed: g is deciding

case("s01-resume-subtree", [*top("resume"), kid("p", "c"), *PRE, TELL("c", "x1"), TELL("c", "x2"), QUIET("c", 2),
                            OPEN, EV("c", 4), TELL("p", "pm"), EV("p", 3), CHILD("g", "p"), CHILD("p", "c"), NOW("g"), NOW("p"), NOW("c")])
case("s02-restart-subtree", [DL(), *top("restart"), kid("p", "c"), *PRE, TELL("c", "x1"), TELL("c", "x2"), QUIET("c", 2),
                             OPEN, EV("p", 6), EV("c", 3), EV("dl", 2), CHILD("g", "p"), CHILD("p", "c"), NOW("g"), NOW("p"), NOW("c"), NOW("dl")])
case("s03-stop-subtree", [DL(), *top("stop"), kid("p", "c"), WATCH("g", "p"), *PRE, TELL("c", "x1"), TELL("c", "x2"), QUIET("c", 2),
                          OPEN, EV("g", 3), EV("p", 3), EV("c", 3), EV("dl", 2), NOW("g"), NOW("p"), NOW("c"), NOW("dl")])
case("s04-two-children-resume", [*top("resume"), kid("p", "c"), kid("p", "c2"), TELL("c2", "n0"), EV("c2", 2), *PRE,
                                 TELL("c", "x1"), TELL("c2", "y1"), TELL("c2", "y2"), QUIET("c", 2), QUIET("c2", 2),
                                 OPEN, EV("c", 3), EV("c2", 4), NOW("c"), NOW("c2"), NOW("p")])
case("s05-grandchild-resume", [*top("resume"), kid("p", "c"), kid("c", "gc"), TELL("gc", "g0"), EV("gc", 2), *PRE,
                               TELL("gc", "z1"), TELL("gc", "z2"), QUIET("gc", 2), QUIET("c", 2),
                               OPEN, EV("gc", 4), TELL("c", "after"), EV("c", 3), NOW("gc"), NOW("c"), NOW("p")])
case("s06-grandchild-restart", [DL(), *top("restart"), kid("p", "c"), kid("c", "gc"), TELL("gc", "g0"), EV("gc", 2), *PRE,
                                TELL("gc", "z1"), QUIET("gc", 2), OPEN, EV("p", 6), EV("gc", 3), EV("dl", 1), NOW("gc"), NOW("c"), NOW("p"), NOW("dl")])


def main():
    out = os.path.join(ROOT, "cases", "scenario-3b4.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, sc in CASES.items():
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(sc, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
