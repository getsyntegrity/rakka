#!/usr/bin/env python3
"""Writes cases/scenario-throwable-cause.jsonl: the cause chain of the lifecycle exceptions (Q-SUP-3). `logFailure` publishes the
message of the cause of an `ActorInitializationException` (AC-SUP-018), a creation failure names the root cause (AC-CELL-002.2) and a
`PreRestartException` keeps what `preRestart` threw (AC-CELL-007.1). The `logs` collector records the chain with `causes`."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW, TELL  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LOGS = {"op": "subscribe", "channel": "logs", "prefix": "", "as": "lg", "causes": True}
STOPPED = {"op": "await-stopped", "ref": "c"}

def logged(count):
    return [EV("lg", count), {"op": "await-quiet", "target": "lg", "count": count, "ms": 300}, NOW("lg")]

def failing_start(**child):
    """`c` fails to be created under `p`, whose default decider stops it"""
    return [LOGS, *S("p", supervisor={}), C("p", "c", "c", **child), STOPPED, *logged(1)]

def failing_restart(**child):
    """`c` throws "boom" and `p` restarts it; the restart then meets the configured failure"""
    return [LOGS, *S("p", supervisor={}), C("p", "c", "c", throw_on="boom", **child), EV("c", 1), TELL("c", "boom")]

CREATION = "pekko://sys/user/g/p/c: exception during creation, root cause message: [init]"

CASES = {
    "tc-prestart": failing_start(throw_in_prestart=True),
    "tc-prestart-nested": failing_start(throw_in_prestart="nested"),
    "tc-prestart-invocation": failing_start(throw_in_prestart="invocation"),
    "tc-prestart-no-message": failing_start(throw_in_prestart="no-message"),
    "tc-constructor": failing_start(constructor_fails_from=1),
    # `p` escalates the creation failure: `g` publishes the same cause message, with `p` as source
    "tc-escalated": [LOGS, *S("g", supervisor={}), C("g", "p", "p", supervisor={"decider": {CREATION: "escalate"}}),
                     C("p", "c", "c", throw_in_prestart=True), {"op": "await-stopped", "ref": "p"}, *logged(1)],
    "tc-post-restart": [*failing_restart(throw_in_post_restart=True), STOPPED, *logged(2)],
    "tc-restart-constructor": [*failing_restart(constructor_fails_from=2), STOPPED, *logged(2)],
    "tc-pre-restart": [*failing_restart(pre_restart_throws="hook"), *logged(2)],
}

def main():
    assert len({n[:24] for n in CASES}) == len(CASES) and max(map(len, CASES)) <= 24
    out = os.path.join(ROOT, "cases", "scenario-throwable-cause.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")

if __name__ == "__main__":
    main()
