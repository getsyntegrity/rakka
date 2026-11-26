#!/usr/bin/env python3
"""Writes cases/scenario-current-message.jsonl: what `currentMessage` holds when a lifecycle hook runs (`preRestart`'s message and
`sender()`, AC-CELL-007).

`invoke` clears it only after a successful message; `handleFailure` sets it to the child's `Failed` and nothing clears that, and
`Resume` keeps the failed message. A sibling `s` under an all-for-one supervisor `g` restarts `p` without `p` running a user message.
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW, TELL, STOP  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def family(p_cfg, **g_decides):
    """`g` (all-for-one) > `p`, `s`; `probe` is a possible sender"""
    g = {"strategy": "all-for-one", "decider": {"sboom": "restart", **g_decides}, "log-decisions": True}
    return [*S("probe"), *S("g", supervisor=g), C("g", "p", "p", log_sender=True, **p_cfg), C("g", "s", "s", throw_on="sboom"),
            EV("p", 1), EV("s", 1)]


def handled(**p_cfg):
    """`p` decides (resume) about its child `k` and stays healthy"""
    supervisor = {"decider": {"kboom": "resume"}, "log-decisions": True}
    return [*family({"supervisor": supervisor, **p_cfg}), C("p", "k", "k", throw_on="kboom"), EV("k", 1), TELL("k", "kboom"), EV("p", 2)]


def resumed():
    """`p` fails on `pboom`, sent by `probe`, and is resumed"""
    return [*family({"throw_on": "pboom"}, pboom="resume"), TELL("p", "pboom", "probe"), EV("g", 2)]


def sibling_restarts(events):
    return [TELL("s", "sboom"), EV("p", events), NOW("p")]


STOPPED = {"op": "await-stopped", "ref": "p"}

CASES = {
    # preRestart is told the child's Failed, and its sender is the child
    "cm-handled-failure": [*handled(), *sibling_restarts(6)],
    # a later successful message clears it
    "cm-then-message": [*handled(), TELL("p", "x", "probe"), EV("p", 3), *sibling_restarts(7)],
    # an unhandled message is a successful one
    "cm-then-unhandled": [{"op": "subscribe", "channel": "unhandled", "as": "un"}, *handled(handles=["y"]), TELL("p", "x", "probe"),
                          EV("un", 1), *sibling_restarts(6)],
    # an auto-received message clears it too
    "cm-then-identify": [*handled(), TELL("p", {"special": "Identify", "id": "q"}, "probe"), EV("probe", 2), *sibling_restarts(6)],
    "cm-then-poison-pill": [*handled(), TELL("p", {"special": "PoisonPill"}, "probe"), STOPPED, NOW("p")],
    # postStop sees the sender of the stale Failed
    "cm-stop-after-failure": [*handled(), STOP("p"), STOPPED, NOW("p")],
    # Resume keeps the failed message and its sender
    "cm-resumed": [*resumed(), *sibling_restarts(5)],
    "cm-resumed-then-message": [*resumed(), TELL("p", "x", "probe"), EV("p", 3), *sibling_restarts(7)],
    # postStop of an actor stopped for its failure sees the sender of the message it failed on
    "cm-failed-then-stopped": [*S("probe"), *S("g", supervisor={"decider": {"pboom": "stop"}}),
                               C("g", "p", "p", throw_on="pboom", log_sender=True), EV("p", 1), TELL("p", "pboom", "probe"), STOPPED,
                               NOW("p")],
    # PreRestartException names the class of the stale Failed
    "cm-hook-error-class": [{"op": "subscribe", "channel": "errors", "as": "err", "prefix": "hook"},
                            *handled(pre_restart_throws="hook"), TELL("s", "sboom"), EV("err", 1), NOW("err")],
}


def main():
    assert len({n[:24] for n in CASES}) == len(CASES) and max(map(len, CASES)) <= 24
    out = os.path.join(ROOT, "cases", "scenario-current-message.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
