#!/usr/bin/env python3
"""Writes cases/scenario-restart-survivors.jsonl: a restarted parent restarts the children its `preRestart` did not stop (AC-SUP-015).

Behaviour options (same names in Scenario.java and scenario.rs): `keep-children` makes `preRestart` only record its event;
`throw-in-post-restart: "once"` makes only the first `postRestart` of the label throw. The failed restart is AC-SUP-019.
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW, TELL, CHILDREN  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OPEN = {"op": "open-hold", "hold": "H"}
KILL = lambda r: {"op": "tell", "ref": r, "msg": {"special": "Kill"}}
# every actor of the tree restarts a child killed with `Kill` (SupervisorTreeSpec restarts on any Exception)
TREE = {"decider": {"Kill": "restart"}}
POST_RESTART_TEXT = "pekko://sys/user/g/p: exception post restart (class java.lang.IllegalStateException)"


def keeper(parent, name, **cfg):
    return C(parent, name, name, keep_children=True, **cfg)


CASES = {
    # k waits for its blocked child to die when p restarts: the parent's Suspend and Recreate are deferred, so k restarts twice
    "sv-mid-restart": [*S("g"), keeper("g", "p", throw_on="boom"), C("p", "k", "k", throw_on="k-boom"), C("k", "gc", "gc", "gate"),
                       TELL("gc", "block"), EV("gc", 2), TELL("k", "k-boom"), EV("k", 4), TELL("p", "boom"), EV("p", 5), CHILDREN("p"),
                       {"op": "release"}, EV("k", 10), TELL("k", "after"), EV("k", 11), NOW("k"), NOW("gc")],
    # k is suspended while g decides: the message queued meanwhile is processed by the restarted instance
    "sv-queued-before-restart": [*S("g", supervisor={"decider": {"boom": "restart"}, "log-decisions": True, "hold": "H"}),
                                 keeper("g", "p", throw_on="boom"), C("p", "k", "k"), TELL("k", "m0"), EV("k", 2), TELL("p", "boom"), EV("g", 2),
                                 TELL("k", "queued"), {"op": "await-quiet", "target": "k", "count": 2, "ms": 300}, OPEN, EV("k", 7),
                                 NOW("k"), NOW("p"), CHILDREN("p")],
    # p fails by escalating k's failure: k survives p's restart and is restarted with the message it had failed on
    "sv-escalating-child": [*S("g", supervisor={"decider": {"boom": "restart"}}), keeper("g", "p", supervisor={"decider": {"boom": "escalate"}}),
                            C("p", "k", "k", throw_on="boom"), EV("k", 1), TELL("k", "boom"), EV("k", 6), TELL("k", "after"), EV("k", 7),
                            NOW("k"), NOW("p")],
    # PostRestartException with the default decider stops p: the survivor is never restarted, only stopped
    "sv-postrestart-fails": [*S("p", throw_on="boom", keep_children=True, throw_in_post_restart=True), C("p", "k", "k"), EV("k", 1),
                             TELL("p", "boom"), EV("k", 2), {"op": "await-stopped", "ref": "k"}, NOW("k"), NOW("p")],
    "sv-three-levels-kill": [*S("head", supervisor=TREE), keeper("head", "middle", supervisor=TREE), keeper("middle", "last", supervisor=TREE),
                             EV("last", 1), KILL("middle"), EV("middle", 4), EV("last", 4), NOW("middle"), NOW("last"), NOW("head")],
}


def failed_once(on_post_restart_failure):
    """`p` fails its first `postRestart`: the next restart skips `preRestart`, and the survivor `k`, suspended once only, restarts"""
    return [*S("g", supervisor={"decider": {"boom": "restart", POST_RESTART_TEXT: on_post_restart_failure}}),
            keeper("g", "p", throw_on="boom", throw_in_post_restart="once"), C("p", "k", "k"), EV("k", 1), TELL("p", "boom"), EV("k", 5),
            TELL("k", "after"), EV("k", 6), TELL("p", "alive"), EV("p", 7), NOW("k"), NOW("p")]


CASES["sv-postrestart-once"] = failed_once("restart")
# `Resume` of an actor that failed fatally is turned into a restart
CASES["sv-fatal-resume"] = failed_once("resume")


def main():
    assert len({n[:24] for n in CASES}) == len(CASES) and max(map(len, CASES)) <= 24
    out = os.path.join(ROOT, "cases", "scenario-restart-survivors.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
