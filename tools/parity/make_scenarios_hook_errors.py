#!/usr/bin/env python3
"""Writes cases/scenario-hook-errors.jsonl: the `Error` published when `preRestart` (AC-CELL-007.1) or `postStop`
(AC-CELL-008.2) throws, and what the actor, its children and a watcher see next. `-null`: a null message and a current
message that is not a String; `-none`: a sibling restarted by all-for-one has no current message."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW, TELL  # noqa: E402
from make_scenarios_fatal_resume import quiet  # noqa: E402  (waits on the `er` collector)

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def errors(prefix, **extra):
    return {"op": "subscribe", "channel": "errors", "prefix": prefix, "as": "er", **extra}

def parent(**sup):
    return S("g", supervisor={"decider": {"boom": "restart", "Kill": "restart"}, **sup})

def failing(**behaviour):  # child `p` of `g`, with its own child `k`
    return [C("g", "p", "p", throw_on="boom", **behaviour), C("p", "k", "k")]

CASES = {
    # `preRestart` throws before `super`: no child is stopped, no `postStop`; the restart goes on and restarts the child
    "he-pre-restart": [errors("hook"), *parent(), *failing(pre_restart_throws="hook"), TELL("p", "boom"),
                       EV("p", 5), EV("k", 4), quiet(1), NOW("p"), NOW("k")],
    "he-pre-restart-null": [errors("null"), *parent(), *failing(pre_restart_throws=None),
                            TELL("p", {"special": "Kill"}), EV("p", 4), quiet(1)],
    "he-pre-restart-none": [errors("hook"), *parent(strategy="all-for-one"), C("g", "q", "q", pre_restart_throws="hook"),
                            C("g", "p", "p", throw_on="boom"), TELL("p", "boom"), EV("q", 4), quiet(1)],
    # the `postStop` of the default `preRestart` throws: the children were already stopped
    "he-pre-restart-stop": [errors("hook"), *parent(), *failing(post_stop_throws="hook"), TELL("p", "boom"),
                            EV("p", 6), EV("k", 2), quiet(1), NOW("p"), NOW("k")],
    # `postStop` throws on stop: the `Error` comes before the watcher's `Terminated`, and termination completes
    "he-post-stop": [*S("p", post_stop_throws="hook"), C("p", "k", "k"), errors("hook", watch="p"), TELL("p", "x"),
                     EV("p", 2), {"op": "stop", "ref": "p"}, {"op": "await-stopped", "ref": "p"}, EV("er", 2), NOW("k"),
                     quiet(2)],
    "he-post-stop-null": [*S("p", post_stop_throws=None), errors("null", watch="p"), {"op": "stop", "ref": "p"},
                          EV("er", 2), quiet(2)],
}

def main():
    assert len({n[:24] for n in CASES}) == len(CASES) and max(map(len, CASES)) <= 24
    out = os.path.join(ROOT, "cases", "scenario-hook-errors.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")

if __name__ == "__main__":
    main()
