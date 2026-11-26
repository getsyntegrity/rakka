#!/usr/bin/env python3
"""Writes cases/scenario-log-failure.jsonl: the `Error` / `Warning` a supervisor publishes when it applies a directive to a failed
child (`SupervisorStrategy.logFailure`, AC-SUP-018). The `logs` collector records both, with source, message and strategy class."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW, TELL  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LOGS = {"op": "subscribe", "channel": "logs", "prefix": "", "as": "lg"}

def quiet(count):
    return {"op": "await-quiet", "target": "lg", "count": count, "ms": 300}

def supervised(supervisor, messages, *children):
    """`p` with this strategy and the children (default `c`), each throwing `IllegalStateException(m)` on the messages `m`"""
    return [LOGS, *S("p", supervisor=supervisor), *[C("p", name, name, throw_on=messages) for name in children or ("c",)]]

def logged(count):
    return [EV("lg", count), quiet(count), NOW("lg")]

CASES = {
    "lf-each-directive": [*supervised({"decider": {"res": "resume", "rst": "restart", "stp": "stop"}}, ["res", "rst", "stp"]),
                          TELL("c", "res"), TELL("c", "rst"), TELL("c", "stp"), *logged(3)],
    "lf-escalate": [LOGS, *S("g", supervisor={"decider": {"esc": "resume"}}), C("g", "p", "p", supervisor={"decider": {"esc": "escalate"}}),
                    C("p", "c", "c", throw_on="esc"), TELL("c", "esc"), *logged(1)],
    "lf-all-for-one": [*supervised({"strategy": "all-for-one", "decider": {"rst": "restart"}}, "rst", "c", "d"),
                       TELL("c", "rst"), *logged(1)],
    "lf-refused-restart": [*supervised({"max": 0, "decider": {"rst": "restart"}}, "rst"), TELL("c", "rst"),
                           {"op": "await-stopped", "ref": "c"}, *logged(1)],
    "lf-logging-disabled": [*supervised({"logging": False, "decider": {"res": "resume", "stp": "stop"}}, ["res", "stp"]),
                            TELL("c", "res"), TELL("c", "stp"), {"op": "await-stopped", "ref": "c"}, quiet(0), NOW("lg")],
    # `p` publishes nothing about `c`; the `/user` guardian logs the exception the decider threw, with `p` as source
    "lf-decider-throws": [*supervised({"decider-throws": True}, "dec"), TELL("c", "dec"), *logged(1)],
}

def main():
    assert len({n[:24] for n in CASES}) == len(CASES) and max(map(len, CASES)) <= 24
    out = os.path.join(ROOT, "cases", "scenario-log-failure.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")

if __name__ == "__main__":
    main()
