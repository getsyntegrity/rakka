#!/usr/bin/env python3
"""Writes cases/scenario-throwing-strategy.jsonl: a `supervisorStrategy` that throws fails the supervisor itself, from
`handleFailure` and from `handleChildTerminated` (dungeon/FaultHandling.scala:305-351), and when Pekko calls it."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, DO, EV, NOW, TELL, STOP, CHILDREN  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ERRORS = {"op": "subscribe", "channel": "errors", "prefix": "handleChildTerminated", "as": "er"}
OPEN = {"op": "open-hold", "hold": "h"}

def top(strategy="restart", **extra):
    """`top` records what it decides: `strategy` about a failed strategy, restart about `boom`"""
    return {"log-decisions": True, "decider": {"strategy": strategy, "boom": "restart"}, **extra}

def tree(supervisor, children=("c",), behavior=None, **p):
    """`top` supervises `p`, whose calls to `supervisorStrategy` are recorded and configured by `p`"""
    steps = [*S("top", supervisor=supervisor),
             C("top", "p", "p", supervisor={"log-calls": True, **p}, **(behavior or {})), EV("p", 1)]
    for name in children:
        steps += [C("p", name, name, throw_on="boom"), EV(name, 1)]
    return steps

CASES = {
    # `handleFailure`: the supervisor fails with what its strategy threw and its own supervisor decides
    "st-failed-restart": [*tree(top(), **{"throw-on-call": 1}), TELL("c", "boom"), EV("top", 1), EV("p", 6), EV("c", 3),
                          NOW("top"), NOW("p"), NOW("c")],
    # `Resume` gives the cause back to the child that had failed, the perpetrator
    "st-failed-resume": [*tree(top("resume"), decider={"boom": "resume"}, **{"throw-on-call": 1}), TELL("c", "boom"),
                         EV("top", 1), TELL("c", "ping"), EV("c", 3), CHILDREN("p"), NOW("p"), NOW("c")],
    # one call for each failure and each termination of a child, none for any other message
    "st-calls": [*tree(top(), decider={"boom": "resume"}), TELL("c", "boom"), EV("p", 2), TELL("c", "boom"), EV("p", 3),
                 TELL("p", "x"), EV("p", 4), STOP("c"), EV("p", 5), TELL("p", "y"), EV("p", 6), NOW("p")],
    "st-terminated": [ERRORS, *tree(top("resume"), **{"throw-on-call": 1}), STOP("c"), EV("er", 1), EV("top", 1),
                      TELL("p", "ping"), EV("p", 3), CHILDREN("p"), NOW("er"), NOW("top"), NOW("p")],
    # the handled `Failed` is still current and names the perpetrator: `a` is not suspended while `top` decides
    "st-terminated-perpetrator": [ERRORS, *tree(top("resume", hold="h"), ("a", "b"), decider={"boom": "resume"},
                                                **{"throw-on-call": 2}),
                                  TELL("a", "boom"), EV("p", 2), STOP("b"), EV("er", 1), TELL("a", "x"), EV("a", 3), OPEN,
                                  TELL("p", "ping"), EV("p", 4), TELL("a", "y"), EV("a", 4), NOW("p"), NOW("a")],
    # the old instance is asked while it restarts: the restart goes on and `top` then decides about the new failure
    "st-restarting": [ERRORS, *tree(top("resume"), behavior={"throw_on": "boom"}, **{"throw-on-call": 1}), TELL("p", "boom"),
                      EV("top", 2), EV("er", 1), TELL("p", "ping"), EV("p", 8), NOW("er"), NOW("top"), NOW("p")],
    # while it terminates the `Failed` still reaches `top`, before the termination
    "st-terminating": [ERRORS, *tree(top("stop"), **{"throw-on-call": 1}), STOP("p"), {"op": "await-stopped", "ref": "p"},
                       EV("top", 1), NOW("er"), NOW("top"), NOW("p")],
}

def main():
    assert len({n[:24] for n in CASES}) == len(CASES)
    out = os.path.join(ROOT, "cases", "scenario-throwing-strategy.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")

if __name__ == "__main__":
    main()
