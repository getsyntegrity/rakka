#!/usr/bin/env python3
"""Writes cases/scenario-closed-cell.jsonl: a system message for a stopped actor is `DeadLetter(message, receiver, receiver)`
(AC-CELL-013.3).

The only system message a script can make Pekko send late is the `Suspend` a failing parent `p` sends a child `c` it still lists:
`w`'s `Terminated` proves that `c` closed and that its death notification is queued at `p`, which cannot process it before its turn
ends. The message is private to Pekko, so both harnesses render it as `{"class":"?"}`.
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, TELL, WATCH  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DEAD_LETTERS = {"op": "subscribe", "channel": "dead-letters", "as": "dl"}
STOP_C = {"op": "stop", "ref": "c"}
OPEN = {"op": "open-hold", "hold": "H"}
RELEASE = {"op": "release"}


def quiet(count):
    return {"op": "await-quiet", "target": "dl", "count": count, "ms": 300}


def supervisor_that_holds(message):
    """`g` records its decision about `p` and holds it until `OPEN`, so `p` resumes no child before it forgot `c`"""
    return S("g", supervisor={"decider": {message: "resume"}, "log-decisions": True, "hold": "H"})


CASES = {
    # `c` is closed when `p` suspends it; the later `Terminate` is suppressed, a `DeadLetter` is dropped, a user message is not
    "dl-sysmsg-after-close": [
        DEAD_LETTERS, *S("w"), *supervisor_that_holds("block"), C("g", "p", "p", "gate", throw_after_gate=True), C("p", "c", "c"),
        EV("c", 1), WATCH("w", "c"), TELL("p", "block"), EV("p", 2), STOP_C, EV("w", 2), RELEASE, EV("g", 2), EV("dl", 1), OPEN,
        TELL("p", "after"), EV("p", 3), STOP_C, {"op": "tell-dead-letter", "ref": "c"}, TELL("c", "late"), EV("dl", 2), quiet(2)],
    # `c` is still in `postStop` when `p` suspends it: the pending message moves to the dead-letter mailbox when `c` closes
    "dl-sysmsg-pending-close": [
        DEAD_LETTERS, *S("w"), *supervisor_that_holds("boom"), C("g", "p", "p", throw_on="boom"),
        C("p", "c", "c", post_stop_gate=True), EV("c", 1), WATCH("w", "c"), STOP_C, EV("c", 2), TELL("p", "boom"), EV("g", 2),
        quiet(0), RELEASE, EV("w", 2), EV("dl", 1), OPEN, TELL("p", "after"), EV("p", 3), quiet(1)],
}


def main():
    assert len({n[:24] for n in CASES}) == len(CASES) and max(map(len, CASES)) <= 24
    out = os.path.join(ROOT, "cases", "scenario-closed-cell.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
