#!/usr/bin/env python3
"""Writes cases/scenario-hook-errors.jsonl: the `Error` of a throwing `preRestart` (AC-CELL-007.1) or `postStop`
(AC-CELL-008.2, AC-CELL-009), and what the actor, its children and a watcher see next."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW, TELL  # noqa: E402
from make_scenarios_fatal_resume import quiet  # noqa: E402  (waits on the `er` collector)

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def errors(prefix, name="er", **extra):
    return {"op": "subscribe", "channel": "errors", "prefix": prefix, "as": name, **extra}

def parent(name="g", **sup):
    return S(name, supervisor={"decider": {"boom": "restart", "Kill": "restart"}, **sup})

def failing(under="g", **behaviour):  # child `p` of `under`, with its own child `k`
    return [C(under, "p", "p", throw_on="boom", **behaviour), C("p", "k", "k")]

CASES = {
    # `g/p` throws before `super` (its child survives), its all-for-one sibling `q` has no current message (`p` and `q` race: own
    # collector), the `postStop` of `h/p`'s default `preRestart` throws, and `i/p` throws a null message while processing `Kill`
    "he-pre-restart": [errors("hook"), errors("sib", "sb"), errors("null", "nu"), *parent(strategy="all-for-one"), *failing(pre_restart_throws="hook"),
                       C("g", "q", "q", pre_restart_throws="sib"), TELL("p", "boom"), EV("p", 5), EV("k", 4), quiet(1),
                       EV("sb", 1), NOW("p"), NOW("k"), *parent("h"),
                       C("h", "p", "r", throw_on="boom", post_stop_throws="hook"), C("r", "k", "s"), TELL("r", "boom"),
                       EV("r", 6), EV("s", 2), quiet(2), NOW("r"), NOW("s"), *parent("i"),
                       C("i", "p", "t", pre_restart_throws=None), TELL("t", {"special": "Kill"}), EV("t", 4), EV("nu", 1)],
    # `postStop` throws on stop, with and without message: the `Error` comes before the watcher's `Terminated`
    "he-post-stop": [*S("p", post_stop_throws="hook"), C("p", "k", "k"), *S("n", post_stop_throws=None),
                     errors("", watch="p"), TELL("p", "x"), EV("p", 2), {"op": "stop", "ref": "p"},
                     {"op": "await-stopped", "ref": "p"}, EV("er", 2), NOW("k"), {"op": "stop", "ref": "n"},
                     {"op": "await-stopped", "ref": "n"}, quiet(3)],
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
