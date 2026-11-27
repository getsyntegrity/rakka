#!/usr/bin/env python3
"""Writes cases/scenario-throwable-class.jsonl: how deciders classify what a child throws (Q-SUP-1; AC-SUP-004, AC-SUP-005,
AC-SUP-018). `c` throws the class named by `throw-class` on "boom"; its parent `p` decides, and `g` decides when `p` escalates."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW, TELL  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LOGS = {"op": "subscribe", "channel": "logs", "prefix": "", "as": "lg", "causes": True}
ARGUMENT = "java.lang.IllegalArgumentException"
RUNTIME = "java.lang.RuntimeException"

# how many events `c` has recorded once it has handled "next": all of them must be awaited, or the trace races
# (restart: pre-start, boom, pre-restart, post-stop, post-restart, pre-start, next)
EVENTS = {"resume": 3, "restart": 7}

def outcome(decision, supervisor):
    """`c` after the decision taken by `supervisor`; the published log is the last thing compared"""
    after = [{"op": "await-stopped", "ref": "c"}] if decision == "stop" else [TELL("c", "next"), EV("c", EVENTS[decision])]
    return [*after, NOW("c"), NOW(supervisor), EV("lg", 1), {"op": "await-quiet", "target": "lg", "count": 1, "ms": 300}, NOW("lg")]

def child(parent, kind):
    return C(parent, "c", "c", throw_class={"boom": kind})

def decided_by_parent(kind, decision, **supervisor):
    return [LOGS, *S("p", supervisor=supervisor), child("p", kind), EV("c", 1), TELL("c", "boom"), *outcome(decision, "p")]

def escalated(kind, decision="resume", **parent):
    """`p` has no directive for it: `g` records what it is asked and answers `decision` about `p`"""
    top = {"decider": {"boom": decision}, "log-decisions": True}
    return [LOGS, *S("g", supervisor=top), C("g", "p", "p", supervisor=parent), child("p", kind), EV("c", 1), TELL("c", "boom"),
            EV("g", 2), *outcome(decision, "g")]

def matching(*pairs):
    return {"match": [list(pair) for pair in pairs]}

def through_user():
    """an Error escalates from `c` through `a` and `/user`, which the root guardian stops: the system ends. No log collector:
    it would live under /user, which stops while the errors are on their way"""
    return [*S("a"), child("a", "error"), EV("c", 1), TELL("c", "boom"), {"op": "await-system-terminated"},
            {"op": "await-life", "count": 4, "before": [["c:post-stop", "a:post-stop"]]}, NOW("a"), NOW("c")]

def failing_top_level_creator():
    """AC-SUP-004, the creator-failure variant of A3: a top-level actor whose creator fails is stopped; /user and its other
    children go on. The log is not recorded here. What the spec of A3 tests (SupervisorSpec.scala:510-514, a mailbox that
    cannot be made, with its one logged ActorInitializationException) is recorded by mb-0035"""
    failing = {"op": "spawn", "name": "a", "as": "a", "behavior": {"kind": "log", "constructor-fails-from": 1}}
    return [failing, {"op": "await-stopped", "ref": "a"}, NOW("a"), *S("b"), TELL("b", "alive"), EV("b", 2), NOW("b")]

CASES = {
    "cl-error-through-user": through_user(),
    "cl-top-creator-stops": failing_top_level_creator(),
    # the default decider
    "cl-user-restarts": decided_by_parent("user", "restart"),
    "cl-checked-restarts": decided_by_parent("checked", "restart"),
    "cl-derived-restarts": decided_by_parent("child", "restart"),
    "cl-init-subtype-stops": decided_by_parent("init", "stop"),
    "cl-init-subtype-cause": decided_by_parent("init-caused", "stop"),
    "cl-error-escalates": escalated("error"),
    "cl-assertion-escalates": escalated("assertion"),
    "cl-user-error-escalates": escalated("user-error"),
    "cl-bare-escalates": escalated("bare"),
    "cl-error-g-stops": escalated("error", "stop"),
    # custom deciders match by subtype, in the order written; what matches nothing escalates
    "cl-match-superclass": decided_by_parent("child", "resume", **matching((ARGUMENT, "resume"), (RUNTIME, "stop"))),
    "cl-match-runtime": decided_by_parent("user", "stop", **matching((ARGUMENT, "resume"), (RUNTIME, "stop"))),
    "cl-match-first-wins": decided_by_parent("number", "stop", **matching((RUNTIME, "stop"), (ARGUMENT, "resume"))),
    "cl-match-error": decided_by_parent("assertion", "resume", **matching(("java.lang.Error", "resume"))),
    "cl-match-exception": decided_by_parent("checked", "resume", **matching(("java.lang.Exception", "resume"))),
    "cl-match-none-escalates": escalated("checked", **matching((RUNTIME, "stop"))),
    # stoppingStrategy
    "cl-stopping-stops": decided_by_parent("checked", "stop", stopping=True),
    "cl-stopping-escalates": escalated("error", stopping=True),
}

def main():
    assert len({n[:24] for n in CASES}) == len(CASES) and max(map(len, CASES)) <= 24
    out = os.path.join(ROOT, "cases", "scenario-throwable-class.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")

if __name__ == "__main__":
    main()
