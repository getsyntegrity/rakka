#!/usr/bin/env python3
"""Writes cases/scenario-fatal-resume.jsonl: `Resume` with a cause of an actor that failed fatally is published and turned into a
restart (AC-SUP-019), and a terminating actor is its own perpetrator, so it hands no child the cause (AC-CELL-008.1).
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW, TELL  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ERRORS = {"op": "subscribe", "channel": "errors", "prefix": "changing ", "as": "er"}
OPEN = {"op": "open-hold", "hold": "H"}

def post_restart_failed(path):
    return f"pekko://sys/user/{path}: exception post restart (class java.lang.IllegalStateException)"

def quiet(count):
    return {"op": "await-quiet", "target": "er", "count": count, "ms": 300}

def fatal(parent, name):  # keeps its children and fails its first `postRestart`
    return C(parent, name, name, keep_children=True, throw_on="boom", throw_in_post_restart="once")


def escalated():
    """`c`'s restart fails, `p` escalates that to `g`, which holds (hold `H`) its `Resume`; `d`, busy under `c`, makes `c` slow to die"""
    holds = {"decider": {post_restart_failed("g/p/c"): "resume"}, "log-decisions": True, "hold": "H"}
    escalates = {"decider": {"boom": "restart", post_restart_failed("g/p/c"): "escalate"}}
    return [ERRORS, *S("g", supervisor=holds), C("g", "p", "p", supervisor=escalates), fatal("p", "c"), C("c", "d", "d", "gate"),
            C("p", "e", "e"), TELL("d", "block"), EV("d", 2), TELL("c", "boom"), EV("g", 2)]


CASES = {
    "fr-fatal-resume-error": [ERRORS, *S("g", supervisor={"decider": {"boom": "restart", post_restart_failed("g/p"): "resume"}}),
                              fatal("g", "p"), TELL("p", "boom"), quiet(1), TELL("p", "alive"), EV("p", 7), NOW("er")],
    "fr-escalated-resume": [*escalated(), OPEN, quiet(1), {"op": "release"}, TELL("c", "alive"), EV("c", 7), NOW("c"), NOW("er")],
    # unlike fr-escalated-resume, `p` is stopped while `g` decides: it is its own perpetrator, `c` gets `Resume(null)`
    "fr-stopped-resume": [*escalated(), {"op": "stop", "ref": "p"}, EV("e", 2), quiet(0), OPEN, quiet(0), {"op": "release"},
                          {"op": "await-stopped", "ref": "p"}, NOW("c"), NOW("er")],
}


def main():
    assert len({n[:24] for n in CASES}) == len(CASES) and max(map(len, CASES)) <= 24
    out = os.path.join(ROOT, "cases", "scenario-fatal-resume.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
