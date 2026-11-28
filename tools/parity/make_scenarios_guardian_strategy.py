#!/usr/bin/env python3
"""Writes cases/scenario-guardian-strategy.jsonl: `pekko.actor.guardian-supervisor-strategy` decides about the failures of the
top-level actors (actor/ActorRefProvider.scala:517-534, actor/FaultHandling.scala:94-105; ActorSystemSpec.scala:353-392).
`GuardianStrategies` (oracle and Rust harness) are the configurators.

`gs-*` cases are deterministic. In `gsc-*` cases several top-level actors fail at once: each is first confirmed inside `receive`
(blocked on one gate), so all are in flight when it opens; they are also meant to be run in parallel. Collectors live under
/user, so no case subscribes one where the guardian escalates (MBX-GRD-05 records those errors, oracle-only)."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW, TELL, WATCH, LIFE  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
STOPPING = "org.apache.pekko.actor.StoppingSupervisorStrategy"
DEFAULT = "org.apache.pekko.actor.DefaultSupervisorStrategy"
PILOT = "pilot.GuardianStrategies$"
ENDED = {"op": "await-system-terminated"}
KILL = {"special": "Kill"}


def logs(prefix):
    """`Error` and `Warning` events whose message starts with `prefix`"""
    return {"op": "subscribe", "channel": "logs", "prefix": prefix, "as": "lg"}


def quiet(target, count):
    return {"op": "await-quiet", "target": target, "count": count, "ms": 200}


def failing(label, *messages, **cfg):
    return S(label, throw_on=list(messages), **cfg)


def watched(label, *messages, **cfg):
    """a probe `w` that watches `label`, which throws on each of `messages`"""
    return [*S("w"), *failing(label, *messages, **cfg), WATCH("w", label)]


def resumed(label, count):
    """`label` goes on: it processes `next` as its `count`-th event"""
    return [TELL(label, "next"), EV(label, count), NOW(label)]


def top_dies(message="hello"):
    return [*failing("a", message), TELL("a", message), ENDED, NOW("a")]


# `throw-class` instead of `throw-on`: the message throws a `java.lang.Error`
ERROR_ENDS = [*S("a", throw_class={"hello": "error"}), TELL("a", "hello"), ENDED, NOW("a")]
TREE = [*failing("a", "hello"), C("a", "c", "c"), EV("c", 1), *S("b")]
TREE_ENDS = [TELL("a", "hello"), ENDED, LIFE(6, ("c:post-stop", "a:post-stop")), NOW("a"), NOW("b"), NOW("c")]
ESCALATING_PARENT = {"decider": {"resume-me": "escalate", "restart-me": "escalate", "stop-me": "escalate"}, "log-decisions": True}


def under_parent(message):
    """top-level `a` escalates the failure of its child `c`, so the guardian decides about `a`"""
    return [logs(message), *S("w"), *S("a", supervisor=ESCALATING_PARENT), WATCH("w", "a"),
            C("a", "c", "c", throw_on=[message]), EV("c", 1), TELL("c", message), TELL("c", "queued")]


CASES = {
    # StoppingSupervisorStrategy
    "gs-stop-spec": (STOPPING, [logs("hello"), *watched("a", "hello"), TELL("a", "hello"), EV("w", 2), EV("lg", 1), EV("a", 3),
                                quiet("lg", 1), quiet("a", 3), *resumed("w", 3)]),
    "gs-stop-children": (STOPPING, [*TREE, WATCH("b", "a"), TELL("a", "hello"), EV("b", 2),
                                    LIFE(6, ("c:post-stop", "a:post-stop"), ("a:post-stop", "b:terminated:a")), NOW("c"),
                                    *resumed("b", 3)]),
    "gs-stop-error-escalates": (STOPPING, ERROR_ENDS),
    "gs-stop-not-inherited": (STOPPING, [*S("a"), C("a", "c", "c", throw_on=["boom"]), EV("c", 1), TELL("c", "boom"), EV("c", 6),
                                         *resumed("c", 7)]),
    "gs-stop-kill": (STOPPING, [logs("Kill"), *watched("a"), TELL("a", KILL), EV("w", 2), EV("lg", 1), EV("a", 2), NOW("a")]),
    # DefaultSupervisorStrategy, explicit
    "gs-default-restarts": (DEFAULT, [logs("hello"), *failing("a", "hello"), TELL("a", "hello"), EV("a", 6), EV("lg", 1),
                                      quiet("lg", 1), *resumed("a", 7)]),
    "gs-default-kill-stops": (DEFAULT, [*watched("a"), TELL("a", KILL), EV("w", 2), EV("a", 2), NOW("a")]),
    "gs-default-error-ends": (DEFAULT, ERROR_ENDS),
    # a configurator that escalates everything (the spec's `Strategy`)
    "gs-escalate-ends": (PILOT + "Escalate", top_dies() + [LIFE(2)]),
    "gs-escalate-tree": (PILOT + "Escalate", [*TREE, *TREE_ENDS]),
    "gs-escalate-kill": (PILOT + "Escalate", [*S("a"), TELL("a", KILL), ENDED, NOW("a")]),
    "gs-escalate-prestart": (PILOT + "Escalate", [{"op": "spawn", "name": "a", "as": "a",
                             "behavior": {"kind": "log", "throw-in-prestart": True}}, ENDED, NOW("a")]),
    "gs-escalate-handled-below": (PILOT + "Escalate", [*S("a"), C("a", "c", "c", throw_on=["boom"]), EV("c", 1), TELL("c", "boom"),
                                  EV("c", 6), *resumed("c", 7), *resumed("a", 2)]),
    # directives of a custom strategy
    "gs-custom-resumes": (PILOT + "ByMessage", [logs("resume"), *failing("a", "resume-me"), TELL("a", "resume-me"), EV("lg", 1),
                                                *resumed("a", 3), quiet("lg", 1)]),
    "gs-custom-restarts": (PILOT + "ByMessage", [logs("restart"), *failing("a", "restart-me"), TELL("a", "restart-me"), EV("a", 6),
                                                 EV("lg", 1), *resumed("a", 7)]),
    "gs-custom-stops": (PILOT + "ByMessage", [logs("stop"), *watched("a", "stop-me"), TELL("a", "stop-me"), EV("w", 2), EV("lg", 1),
                                              EV("a", 3), quiet("a", 3)]),
    "gs-custom-escalates": (PILOT + "ByMessage", top_dies("escalate-me")),
    # AC-SUP-008: the escalating parent and its child go on after one decision of the guardian (neither was suspended twice)
    "gs-parent-resumed": (PILOT + "ByMessage", [*under_parent("resume-me"), EV("lg", 1), EV("c", 3), *resumed("c", 4),
                                                *resumed("a", 3), NOW("lg")]),
    "gs-parent-restarted": (PILOT + "ByMessage", [*under_parent("restart-me"), EV("lg", 1), EV("a", 6), LIFE(8), NOW("c"), NOW("lg")]),
    "gs-parent-stopped": (PILOT + "ByMessage", [*under_parent("stop-me"), EV("w", 2), EV("lg", 1), EV("a", 3), NOW("c"), NOW("lg")]),
    # AC-SUP-008: until the escalation is decided, the child and the escalating parent stay suspended
    "gs-escalated-held": (DEFAULT, [*S("g", supervisor={"decider": {"esc": "resume"}, "hold": "H"}),
                          C("g", "p", "p", supervisor={"decider": {"esc": "escalate"}}), EV("p", 1),
                          C("p", "c", "c", throw_on=["esc"]), EV("c", 1), TELL("c", "esc"), TELL("c", "q"), LIFE(4),
                          TELL("p", "pm"), quiet("c", 2), quiet("p", 1), {"op": "open-hold", "hold": "H"}, EV("c", 3), EV("p", 2)]),
    "gs-one-retry": (PILOT + "OneRetry", [*watched("a", "restart-me"), TELL("a", "restart-me"), EV("a", 6),
                                          TELL("a", "restart-me"), EV("w", 2), EV("a", 8), quiet("a", 8)]),
    "gs-all-for-one-restarts": (PILOT + "AllForOne", [*S("b"), *failing("a", "restart-me"), TELL("a", "restart-me"), EV("a", 6),
                                                      EV("b", 5), *resumed("b", 6)]),
    "gs-all-for-one-stops": (PILOT + "AllForOne", [*S("b"), *failing("a", "stop-me"), TELL("a", "stop-me"), LIFE(4),
                                                   NOW("a"), NOW("b")]),
    "gs-quiet-restarts": (PILOT + "Quiet", [logs("restart"), *failing("a", "restart-me"), TELL("a", "restart-me"), EV("a", 6),
                                            quiet("lg", 0), *resumed("a", 7)]),
}

# concurrent failures
ACTORS = ["a", "b", "c", "d"]


def together(actors, **cfg):
    """every actor blocks in `receive` on the gate; once all are in, it opens and all of them throw"""
    steps = []
    for x in actors:
        steps += S(x, kind="gate", throw_after_gate=True, **cfg)
    steps += [TELL(x, "block") for x in actors] + [EV(x, 2) for x in actors]
    return steps + [{"op": "release"}]


def stopping_together(actors):
    steps = together(actors)
    at = steps.index(TELL(actors[0], "block"))
    steps[at:at] = [*S("w"), *[WATCH("w", x) for x in actors]]
    # the order of the `Terminated`s is a race
    steps.append({"op": "await-events", "target": "w", "count": 1 + len(actors), "sorted": True})
    return steps + [step for x in actors for step in (EV(x, 3), quiet(x, 3))]


def restarting_together(actors):
    return together(actors) + [step for x in actors for step in (EV(x, 6), TELL(x, "next"), EV(x, 7), quiet(x, 7))]


def ending_together(actors):
    return together(actors) + [ENDED, LIFE(2 * len(actors))] + [NOW(x) for x in actors]


for count in (2, 4):
    actors = ACTORS[:count]
    CASES[f"gsc-stop-{count}"] = (STOPPING, stopping_together(actors))
    CASES[f"gsc-restart-{count}"] = (DEFAULT, restarting_together(actors))
    CASES[f"gsc-escalate-{count}"] = (PILOT + "Escalate", ending_together(actors))


# SupervisorHierarchySpec.scala:769-790 "restart manager and workers in AllForOne" (AC-SUP-008 A1): `boss` restarts any
# Exception, the AllForOne `manager` has an empty trapExit (it escalates), all keep their children, killing one worker
# restarts the manager and the three workers.
KEEPER = {"keep_children": True, "post_restart_without_pre_start": True}
WORKERS = ["w1", "w2", "w3"]
RESTARTED = ["m", *WORKERS]
CASES["gsc-manager-workers"] = (DEFAULT, [
    *S("boss", supervisor={"match": [["java.lang.Exception", "restart"]]}, **KEEPER),
    C("boss", "m", "m", supervisor={"strategy": "all-for-one", "match": [], "log-decisions": True}, **KEEPER), EV("m", 1),
    *[step for w in WORKERS for step in (C("m", w, w, **KEEPER), EV(w, 1))],
    TELL("w1", KILL),
    # 5 pre-starts, then a pre-restart and a post-restart of the manager and of each worker; the manager restarts first
    LIFE(13, *[("m:post-restart", f"{w}:post-restart") for w in WORKERS]),
    *[step for x in RESTARTED for step in (EV(x, 3), quiet(x, 4 if x == "m" else 3))],
    NOW("boss"),
])


def main():
    out = os.path.join(ROOT, "cases", "scenario-guardian-strategy.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, (strategy, steps) in CASES.items():
            scenario = {"system": "sys", "guardian-strategy": strategy, "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
