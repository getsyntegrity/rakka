#!/usr/bin/env python3
"""Writes cases/scenario-stash-failed.jsonl: a suspended supervisor defers its child's `Failed` (`StashWhenFailed`, AC-CELL-013.2).

The failure has to reach `p` while it is already suspended, and a suspended child runs no user message: `k` is held inside its own
decider (hold `K`) while `p` gets suspended, and escalates once the hold is opened.
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW, TELL  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def decides(directive, on="boom", **extra):
    return {"decider": {on: directive}, "log-decisions": True, **extra}


def opened(hold):
    return {"op": "open-hold", "hold": hold}


def quiet(count=2):
    return {"op": "await-quiet", "target": "p", "count": count, "ms": 300}


def deferred(top, failing, **p_cfg):
    """`top` builds `g`; `p` > `k` > `gc` hang from it. `failing` throws and its supervisor holds (hold `H`); only then does `k` escalate."""
    supervisor = {"p": "g", "g": "h"}[failing]
    return [*top, C("g", "p", "p", supervisor=decides("resume", "gc-boom"), **p_cfg),
            C("p", "k", "k", supervisor=decides("escalate", "gc-boom", hold="K")), C("k", "gc", "gc", throw_on="gc-boom"), EV("gc", 1),
            TELL("gc", "gc-boom"), EV("k", 2), TELL(failing, "boom"), EV(supervisor, 2), opened("K")]


def failed_p(g_answers, **p_cfg):
    return deferred(S("g", supervisor=decides(g_answers, hold="H")), "p", throw_on="boom", **p_cfg)


CASES = {
    # p failed itself: it decides about k only once it is resumed
    "sf-failed-until-resumed": [*failed_p("resume"), quiet(), opened("H"), EV("p", 3), TELL("gc", "after"), EV("gc", 3), NOW("p"), NOW("k")],
    # p is only suspended, along with its failed parent g: the same deferral
    "sf-suspended-with-parent": [*deferred([*S("h", supervisor=decides("resume", hold="H")), C("h", "g", "g", throw_on="boom")], "g"),
                                 quiet(1), opened("H"), EV("p", 2), TELL("gc", "after"), EV("gc", 3), NOW("p"), NOW("g")],
    # p is restarted and keeps k: the deferred failure is decided by the new instance, after postRestart
    "sf-decided-after-restart": [*failed_p("restart", keep_children=True), quiet(), opened("H"), EV("p", 6), NOW("p")],
    # p is stopped: the deferred failure is forgotten, neither decided nor a dead letter (`clearFieldsForTermination`)
    "sf-dropped-on-stop": [{"op": "subscribe", "as": "dl", "channel": "dead-letters"}, *failed_p("stop"), quiet(), opened("H"),
                               {"op": "await-stopped", "ref": "p"},
                               {"op": "await-quiet", "target": "dl", "count": 0, "ms": 300}, NOW("p"), NOW("dl")],
}


def main():
    assert len({n[:24] for n in CASES}) == len(CASES) and max(map(len, CASES)) <= 24
    out = os.path.join(ROOT, "cases", "scenario-stash-failed.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
