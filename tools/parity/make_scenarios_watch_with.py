#!/usr/bin/env python3
"""Writes cases/scenario-watch-with.jsonl: `context.watchWith` (AC-DW-005).

`watch-with` takes a string, `{"custom": name}` (a user type with value equality), a number or a special message. `no-text` keeps only
the exception class: Java renders non-string values in ways Rust cannot (Q-DW-4). Cases where Rust equality differs on purpose from
Scala (Q-DW-3 (a), e.g. 1 and 1L) are Rust tests, never recorded cases.
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import DO, DOA, EV, NOW, S, STOP, TELL, barrier  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def WW(msg, target="t", who="w", **kw):
    return DO(who, "watch-with", target=target, msg=msg, **kw)


def custom(name):
    return {"custom": name}


def queued(c, **kw):
    return DOA("w", c, target="t", **kw)


def end(count):
    """waits for `count` events of `w`, then proves nothing else arrived"""
    return [EV("w", count), barrier("w"), NOW("w")]


W = [*S("w"), *S("t")]
U = S("u")
NO_TEXT = {"no-text": True}


# `w` is blocked while `t` dies: its DeathWatchNotification is processed before what is queued here, the Terminated after it.
# Receiving "before" proves the Terminated is already enqueued, so the barrier of `end` comes after it (no race).
def while_queued(events, *commands):
    return [*S("w", "gate"), *S("w0"), *S("t"), WW("first"), DO("w0", "watch", target="t"), TELL("w", "block"), EV("w", 2), *commands,
            TELL("w", "before"), STOP("t"), EV("w0", 2), {"op": "release"}, *end(events)]


CASES = {
    "ww-delivered": [*W, *U, WW("bye"), WW("bye"), WW(custom("a"), "u"), WW(custom("a"), "u"), STOP("t"), EV("w", 2), STOP("u"), *end(3)],
    "ww-rejects": [*W, *U, WW("a"), WW("b"), DO("w", "watch", target="t"), WW(custom("a"), **NO_TEXT), WW(1, **NO_TEXT), WW("a"),
                   DO("w", "watch", target="u"), WW("a", "u"), STOP("t"), EV("w", 2), STOP("u"), *end(3)],
    "ww-rejects-typed": [*W, WW(custom("a")), WW(custom("b"), **NO_TEXT), WW("a", **NO_TEXT), WW(custom("a")), STOP("t"), *end(2)],
    "ww-unwatch-resets": [*W, WW("a"), DO("w", "unwatch", target="t"), WW("b"), DO("w", "unwatch", target="t"),
                          DO("w", "watch", target="t"), DO("w", "unwatch", target="t"), WW("c"), STOP("t"), *end(2)],
    "ww-dead-and-self": [*W, WW("me", "w"), WW("other", "w"), STOP("t"), {"op": "await-stopped", "ref": "t"}, WW("gone"), EV("w", 2),
                         WW("again"), *end(3)],
    "ww-unwatch-queued": while_queued(3, queued("unwatch")),
    # watched again before the queued Terminated is handled: the first message stays, and there is only one notification
    "ww-queued-keeps-first": while_queued(4, queued("watch-with", msg="second")),
    "ww-queued-replaced": while_queued(4, queued("unwatch"), queued("watch-with", msg="second")),
    # an ordinary message: unhandled gives no DeathPactException, and a PoisonPill reaches receive instead of stopping
    "ww-ordinary": [{"op": "subscribe", "channel": "unhandled", "as": "un"}, *S("w", handles=["ping"]), *S("t"), WW(custom("x")),
                    STOP("t"), EV("un", 1), TELL("w", "ping"), NOW("un"), *end(2)],
    "ww-poison-pill": [*W, WW({"special": "PoisonPill"}), STOP("t"), EV("w", 2), TELL("w", "ping"), *end(3)],
    # currentMessage is the custom one: preRestart gets it; the restart keeps what is watched
    "ww-restart": [*S("w", throw_on=["boom"]), *S("t"), *U, WW("boom"), WW("kept", "u"), STOP("t"), EV("w", 6), WW("changed", "u"),
                   STOP("u"), *end(7)],
}


def main():
    out = os.path.join(ROOT, "cases", "scenario-watch-with.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
