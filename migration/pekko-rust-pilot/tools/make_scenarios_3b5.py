#!/usr/bin/env python3
"""Writes cases/scenario-3b5.jsonl: `AllForOneStrategy` (slice 3b-5).

Same harness as 3b-2 (a scripted supervisor `p` with `supervisor: {"decider": {<message>: directive}, "max": n}` and children that throw
`IllegalStateException(<message>)`), with `"strategy": "all-for-one"` and `"log-decisions": true` (what the decider saw). Two children, `c1` fails, `c2` is healthy.
Part O: directives, identity, queued messages, a decision held pending by a barrier (`hold`, no sleeps). Part R: the restart limit (`max`) when one of the
children exhausts it. Pekko 2.0.0 decides every expected trace.
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


SYNC = lambda r: DO(r, "children")
DECIDER = {"bres": "resume", "brst": "restart", "bstp": "stop"}
THROW = ["bres", "brst", "bstp", "other"]
OPEN = {"op": "open-hold", "hold": "H"}
EVS = lambda t, n: {"op": "await-events", "target": t, "count": n, "sorted": True}   # two Terminated arrive in a racy order
SORTED = lambda t: {"op": "events-sorted", "target": t}


def P(max=None, hold=None):
    sup = {"strategy": "all-for-one", "decider": DECIDER, "log-decisions": True}
    if max is not None:
        sup["max"] = max
    if hold:
        sup["hold"] = hold
    return S("p", supervisor=sup)


def K(name):
    return C("p", name, name, numbered=True, throw_on=THROW)


BOTH = [K("c1"), K("c2")]
Q = lambda r: {"op": "tell-seq", "ref": r, "prefix": "q", "count": 2}

# --- Part O: directives -----------------------------------------------------------------------------------------------------------
case("o01-resume-only-failed", [*P(), *BOTH, TELL("c2", "a"), EV("c2", 2), TELL("c1", "m1"), TELL("c1", "bres"), TELL("c1", "m2"), TELL("c2", "b"),
                                EV("c1", 4), EV("c2", 3), EV("p", 2), SYNC("c1"), SYNC("c2"), NOW("c1"), NOW("c2"), NOW("p")])
case("o02-restart-both", [*P(), *BOTH, WATCH("p", "c1"), WATCH("p", "c2"), TELL("c2", "a"), EV("c2", 2), TELL("c1", "m1"), TELL("c1", "brst"),
                          EV("c1", 7), EV("c2", 6), EV("p", 2), TELL("c1", "m2"), TELL("c2", "b"), EV("c1", 8), EV("c2", 7), NOW("c1"), NOW("c2"), NOW("p")])
case("o03-restart-identity", [*P(), *BOTH, WATCH("p", "c1"), WATCH("p", "c2"), CHILD("p", "c1"), CHILD("p", "c2"), TELL("c1", "brst"), EV("c1", 6), EV("c2", 5),
                              EV("p", 2), CHILD("p", "c1"), CHILD("p", "c2"), CHILDREN("p"), NOW("p")])
case("o04-stop-both", [DL(), *P(), *BOTH, WATCH("p", "c1"), WATCH("p", "c2"), TELL("c2", "a"), EV("c2", 2), TELL("c1", "m1"), TELL("c1", "bstp"),
                       EV("c1", 4), EV("c2", 3), EVS("p", 4), TELL("c1", "late1"), TELL("c2", "late2"), EV("dl", 2), CHILDREN("p"), SORTED("p"), NOW("c1"), NOW("c2"), NOW("dl")])
case("o05-restart-queued", [*P(), *BOTH, TELL("c1", "brst"), Q("c1"), EV("c1", 8), EV("c2", 5), EV("p", 2), NOW("c1"), NOW("c2")])
case("o06-stop-queued-dead", [DL(), *P(), *BOTH, WATCH("p", "c1"), WATCH("p", "c2"), TELL("c1", "bstp"), Q("c1"), EVS("p", 4), EV("dl", 2), NOW("c1"), NOW("c2"), NOW("dl")])
case("o07-default-restarts-both", [*P(), *BOTH, TELL("c2", "a"), EV("c2", 2), TELL("c1", "other"), EV("c1", 6), EV("c2", 6), EV("p", 2), NOW("c1"), NOW("c2"), NOW("p")])
# the decision is pending (held): the healthy sibling is not suspended yet and keeps working; then both restart
case("o08-held-sibling-active", [*P(hold="H"), *BOTH, TELL("c2", "a"), EV("c2", 2), TELL("c1", "brst"), EV("p", 2), TELL("c2", "y1"), TELL("c2", "y2"), EV("c2", 4),
                                 OPEN, EV("c1", 6), EV("c2", 8), TELL("c2", "z"), EV("c2", 9), NOW("c1"), NOW("c2"), NOW("p")])
case("o09-held-resume-sibling", [*P(hold="H"), *BOTH, TELL("c1", "bres"), EV("p", 2), TELL("c2", "y1"), EV("c2", 2), OPEN, TELL("c1", "m"), EV("c1", 3), NOW("c1"), NOW("c2")])

# --- Part R: the restart limit ----------------------------------------------------------------------------------------------------
case("r01-max-zero-stops-both", [*P(max=0), *BOTH, WATCH("p", "c1"), WATCH("p", "c2"), TELL("c1", "brst"), EV("c1", 3), EV("c2", 2), EVS("p", 4), CHILDREN("p"), NOW("c1"), NOW("c2"), SORTED("p")])
case("r02-max-one-second-stops", [*P(max=1), *BOTH, WATCH("p", "c1"), WATCH("p", "c2"), TELL("c1", "brst"), EV("c1", 6), EV("c2", 5), TELL("c2", "brst"),
                                  EV("c2", 7), EV("c1", 7), EVS("p", 5), CHILDREN("p"), NOW("c1"), NOW("c2"), SORTED("p")])
case("r03-resume-not-counted", [*P(max=1), *BOTH, WATCH("p", "c1"), WATCH("p", "c2"), TELL("c1", "bres"), TELL("c1", "bres"), TELL("c1", "brst"), EV("c1", 8), EV("c2", 5),
                                TELL("c2", "brst"), EV("c2", 7), EV("c1", 9), EVS("p", 7), NOW("c1"), NOW("c2"), SORTED("p")])
case("r04-max-two-third-stops", [*P(max=2), *BOTH, WATCH("p", "c1"), WATCH("p", "c2"), TELL("c1", "brst"), EV("c1", 6), EV("c2", 5), TELL("c2", "brst"), EV("c2", 10), EV("c1", 10),
                                 TELL("c1", "brst"), EV("c1", 12), EV("c2", 11), EVS("p", 6), NOW("c1"), NOW("c2"), SORTED("p")])
case("r05-unlimited-both-restart", [*P(max=-1), *BOTH, TELL("c1", "brst"), EV("c1", 6), EV("c2", 5), TELL("c2", "brst"), EV("c2", 10), EV("c1", 10), TELL("c1", "brst"), EV("c1", 15), EV("c2", 14), EV("p", 4), NOW("c1"), NOW("c2"), NOW("p")])


def main():
    out = os.path.join(ROOT, "cases", "scenario-3b5.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, sc in CASES.items():
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(sc, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
