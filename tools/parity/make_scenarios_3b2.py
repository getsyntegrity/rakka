#!/usr/bin/env python3
"""Writes cases/scenario-3b2.jsonl: OneForOneStrategy with a configurable decider (Resume / Restart / Stop) and the restart limit (slice 3b-2).

The supervisor is a scripted actor with `supervisor: {"max": n, "decider": {<exception message>: "resume"|"restart"|"stop"}}`; its children throw
`IllegalStateException(<message>)` when they receive a message listed in `throw-on`. Exceptions with no entry get Pekko's default decider.
`max` is `maxNrOfRetries` (-1 = unlimited); the time window (`withinTimeRange`) is not covered here. Pekko 2.0.0 decides every expected trace.
Part A (a*, w*): the three directives and the healthy sibling. Part B (l*): `maxNrOfRetries`.
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, WATCH, CHILDREN, EV, NOW, TELL, DL, DO, GONE, STOPC, beh  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CASES = {}


def case(name, steps, system="sys"):
    assert name not in CASES, name
    assert len({n[:24] for n in CASES}) == len(CASES), f"case names must differ in the first 24 chars: {name}"
    CASES[name] = {"system": system, "steps": steps}


SYNC = lambda r: DO(r, "children")
DECIDER = {"bres": "resume", "brst": "restart", "bstp": "stop"}
THROW = ["bres", "brst", "bstp", "other"]


def P(max=None, decider=None, label="p"):
    sup = {"decider": DECIDER if decider is None else decider}
    if max is not None:
        sup["max"] = max
    return S(label, supervisor=sup)


def K(name, label=None):
    """a numbered child of `p` that throws on the messages of THROW"""
    return C("p", name, label or name, numbered=True, throw_on=THROW)


# --- Part A: the directives ---------------------------------------------------------------------------------------------
case("a01-resume-keeps-state", [*P(), K("c"), TELL("c", "m1"), TELL("c", "bres"), TELL("c", "m2"), EV("c", 4), SYNC("c"), NOW("c")])
case("a02-resume-next-queued", [*P(), K("c"), {"op": "tell-seq", "ref": "c", "prefix": "q", "count": 2}, TELL("c", "bres"),
                                {"op": "tell-seq", "ref": "c", "prefix": "r", "count": 2}, EV("c", 6), NOW("c")])
case("a03-resume-message-lost", [DL(), *P(), K("c"), TELL("c", "bres"), TELL("c", "x"), EV("c", 3), SYNC("c"), NOW("dl")])
case("a04-resume-ref-alive", [*P(), K("c"), WATCH("p", "c"), TELL("c", "bres"), TELL("c", "x"), EV("c", 3), SYNC("p"), NOW("p")])
case("a05-restart-resets-state", [*P(), K("c"), TELL("c", "m1"), TELL("c", "brst"), TELL("c", "m2"), EV("c", 8), NOW("c")])
case("a06-restart-next-queued", [*P(), K("c"), {"op": "tell-seq", "ref": "c", "prefix": "q", "count": 2}, TELL("c", "brst"),
                                 {"op": "tell-seq", "ref": "c", "prefix": "r", "count": 2}, EV("c", 10), NOW("c")])
case("a07-restart-message-lost", [DL(), *P(), K("c"), TELL("c", "brst"), TELL("c", "x"), EV("c", 7), SYNC("c"), NOW("dl")])
case("a08-restart-ref-alive", [*P(), K("c"), WATCH("p", "c"), TELL("c", "brst"), TELL("c", "x"), EV("c", 7), SYNC("p"), NOW("p")])
case("a09-stop-directive", [*P(), K("c"), WATCH("p", "c"), TELL("c", "m1"), TELL("c", "bstp"), EV("c", 4), EV("p", 2), CHILDREN("p"), NOW("c"), NOW("p")])
case("a10-stop-queued-dead", [DL(), *P(), K("c"), WATCH("p", "c"), {"op": "tell-seq", "ref": "c", "prefix": "q", "count": 3}, TELL("c", "bstp"), EV("p", 2), NOW("c")])
case("a11-stop-then-late-letter", [DL(), *P(), K("c"), WATCH("p", "c"), TELL("c", "bstp"), EV("p", 2), TELL("c", "late"), EV("dl", 1), NOW("dl")])
case("a12-unmapped-default-restart", [*P(), K("c"), TELL("c", "m1"), TELL("c", "other"), TELL("c", "m2"), EV("c", 8), NOW("c")])
case("a13-sequence-all-three", [*P(), K("c"), TELL("c", "m1"), TELL("c", "bres"), TELL("c", "m2"), TELL("c", "brst"), TELL("c", "m3"), TELL("c", "bstp"), EV("c", 12), NOW("c")])
# the healthy sibling: two children of the same supervisor, only c1 fails
for tag, bad, n1, n2 in (("resume", "bres", 4, 4), ("restart", "brst", 8, 4), ("stop", "bstp", 4, 4)):
    case(f"w-{tag}-healthy-sibling", [*P(), K("c1"), K("c2"), WATCH("p", "c1"), WATCH("p", "c2"),
                                      TELL("c2", "a"), TELL("c1", "m1"), TELL("c1", bad), TELL("c2", "b"), TELL("c1", "m2"), TELL("c2", "c"),
                                      EV("c1", n1), EV("c2", n2), *([EV("p", 2)] if tag == "stop" else []), SYNC("c2"), NOW("c1"), NOW("c2"), NOW("p")])
case("w-both-fail-different", [*P(), K("c1"), K("c2"), WATCH("p", "c2"), TELL("c1", "bres"), TELL("c2", "brst"), TELL("c1", "x"), TELL("c2", "y"),
                               EV("c1", 3), EV("c2", 7), NOW("c1"), NOW("c2")])
case("w-both-fail-same", [*P(), K("c1"), K("c2"), TELL("c1", "brst"), TELL("c2", "brst"), TELL("c1", "z1"), TELL("c2", "z2"), EV("c1", 7), EV("c2", 7), NOW("c1"), NOW("c2")])
case("w-stop-leaves-sibling-name", [*P(), K("c1"), K("c2"), WATCH("p", "c1"), TELL("c1", "bstp"), EV("p", 2), CHILDREN("p"), TELL("c2", "alive"), EV("c2", 2)])


# --- Part B: maxNrOfRetries (counted per child, restarts only; Resume and Stop never count) ----------------------------------------
BRST = lambda n: [TELL("c", "brst")] * n
case("l01-max-zero-first-stops", [*P(max=0), K("c"), WATCH("p", "c"), TELL("c", "brst"), EV("c", 3), EV("p", 2), NOW("c"), CHILDREN("p")])
case("l02-max-one-second-stops", [DL(), *P(max=1), K("c"), WATCH("p", "c"), *BRST(2), EV("c", 8), EV("p", 2), TELL("c", "late"), EV("dl", 1), NOW("c")])
case("l03-max-two-third-stops", [*P(max=2), K("c"), WATCH("p", "c"), *BRST(3), EV("c", 13), EV("p", 2), NOW("c")])
case("l04-max-three-fourth-stops", [*P(max=3), K("c"), WATCH("p", "c"), *BRST(4), EV("c", 18), EV("p", 2), NOW("c")])
case("l05-resume-does-not-count", [*P(max=1), K("c"), WATCH("p", "c"), TELL("c", "bres"), TELL("c", "bres"), TELL("c", "bres"), *BRST(2), EV("c", 11), EV("p", 2), NOW("c")])
case("l06-counter-per-child", [*P(max=1), K("c1"), K("c2"), WATCH("p", "c1"), WATCH("p", "c2"),
                               TELL("c1", "brst"), EV("c1", 6), TELL("c2", "brst"), EV("c2", 6), TELL("c2", "alive"), EV("c2", 7),
                               TELL("c1", "brst"), EV("c1", 8), EV("p", 2), TELL("c2", "brst"), EV("c2", 9), EV("p", 3), NOW("c1"), NOW("c2")])
case("l07-unlimited-minus-one", [*P(max=-1), K("c"), *BRST(5), TELL("c", "ok"), EV("c", 27), NOW("c")])
case("l08-negative-is-unlimited", [*P(max=-5), K("c"), *BRST(3), TELL("c", "ok"), EV("c", 17), NOW("c")])
case("l09-new-child-fresh-counter", [*P(max=1), K("c", "c1"), WATCH("p", "c1"), TELL("c1", "brst"), EV("c1", 6), TELL("c1", "bstp"), EV("p", 2),
                                     K("c", "c2"), TELL("c2", "brst"), TELL("c2", "ok"), EV("c2", 7), TELL("c2", "brst"), EV("c2", 9), NOW("c2")])
case("l10-stop-and-resume-then-restart-ok", [*P(max=1), K("c1"), K("c2"), TELL("c1", "bres"), TELL("c1", "brst"), TELL("c1", "ok"), TELL("c2", "bstp"), EV("c1", 8), NOW("c1")])


def main():
    out = os.path.join(ROOT, "cases", "scenario-3b2.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, sc in CASES.items():
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(sc, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
