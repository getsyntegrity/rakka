#!/usr/bin/env python3
"""Writes cases/scenario-actor-selection.jsonl: `actorSelection` from a string path (AC-REF-011) and what a path that
matches nothing does (AC-REF-014, AC-REF-013). No wildcards.

`select` is `system.actorSelection(path).tell(msg, from)`; the `select` command does the same from an actor's context, with
that actor as sender; the `select-forward` behaviour forwards what it receives. Every step that reads events waits for them.
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import DL, EV, NOW, S, TELL  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def select(path, msg, sender="r"):
    return {"op": "select", "path": path, "msg": msg, **({"from": sender} if sender else {})}


def select_uid(path, msg, of="c"):
    return {**select(path, msg), "uid-of": of}


def ident(correlation):
    return {"special": "Identify", "id": correlation}


def inside(actor, path, msg):
    return {"op": "do", "ref": actor, "cmd": {"c": "select", "path": path, "msg": msg}}


def child(parent, name):
    spawn = {"c": "spawn-child", "name": name, "as": name, "behavior": {"kind": "log"}}
    return [{"op": "do", "ref": parent, "cmd": spawn}, EV(name, 1)]


TREE = [DL(), *S("r"), *S("p"), *child("p", "c"), *child("p", "d")]


def forwarder(name, path):
    return {"op": "spawn", "name": name, "as": name, "behavior": {"kind": "select-forward", "path": path}}

UNHANDLED = {"op": "subscribe", "as": "u"}

CASES = {
    # `name#uid`: another incarnation gets nothing (dead letter, or no actor for `Identify`); the live uid and 0 deliver
    "sel-uid": [*TREE, select("/user/p/c#999999999", "stale-absolute"), select("user/p/c#999999999", "stale-relative"),
                select("pekko://sys/user/p/c#999999999", "stale-uri"), select("/user/p/c#999999999", ident("stale")),
                select_uid("/user/p/c#", "absolute"), select_uid("user/p/c#", "relative"),
                select_uid("pekko://sys/user/p/c#", "uri"), select("/user/p/c#0", "any"),
                EV("c", 5), EV("dl", 3), EV("r", 2), NOW("c"), NOW("dl"), NOW("r")],
    # absolute, relative to the root guardian, full URI, repeated and trailing slashes, `..` from the root
    "sel-paths": [*TREE, select("/user/p/c", "absolute"), select("user/p/c", "relative"), select("pekko://sys/user/p/c", "uri"),
                 select("/user//p//c/", "slashes"), select("/../user/p/c", "above-root"), select("/user/p/c/..", "up"),
                 select("/user/p/c", "unsent", None), EV("c", 7), EV("p", 2), NOW("c"), NOW("p")],
    # anchored at an actor: a child by name, a sibling and the parent through `..`, an absolute path
    "sel-context": [*TREE, inside("p", "c", "to-child"), inside("c", "../d", "to-sibling"), inside("c", "..", "to-parent"),
                   inside("c", "../../p/d", "back-down"), inside("d", "/user/p/c", "from-root"),
                   EV("c", 3), EV("d", 3), EV("p", 2), NOW("c"), NOW("d"), NOW("p")],
    # a suppressed message (PoisonPill) to nothing publishes no dead letter: `last` is the barrier
    "sel-missing": [*TREE, select("/user/missing", "a"), select("/user/p/missing/deeper", "b"), select("missing", "c", None),
                   inside("c", "../nobody", "e"), inside("c", "x/../y", "f"), select("pekko://other/user/p", "g"),
                   select("pekko://sys2/user/p", "h"), select("not a path", "i"), select("/deadLetters/x", "j"),
                   select("/deadLetters", "k"),
                   select("//user/p/c", "authority"), select("/user/missing", {"special": "PoisonPill"}), select("/user/missing", "last"),
                   EV("dl", 12), NOW("dl"), NOW("p")],
    # the empty path is the root guardian, which does not handle a String
    "sel-empty-path": [*TREE, UNHANDLED, select("", "root"), EV("u", 1), NOW("u"), NOW("dl")],
    "sel-identify": [*TREE, select("/user/p/c", ident("found")), EV("r", 2), select("/user/missing", ident("none")), EV("r", 3),
                    select("/user/p/c/no/deeper", ident("deep")), EV("r", 4), select("pekko://other/user/p", ident("foreign")),
                    EV("r", 5), select("not a path", ident("unparsed")), EV("r", 6), select("/deadLetters", ident("dead")),
                    EV("r", 7), select("/deadLetters/x", ident("under-dead")), EV("r", 8), TELL("c", "barrier"), EV("c", 2),
                    NOW("r"), NOW("c"), NOW("dl")],
    # `forward` keeps the sender; without one the receiver sees `deadLetters`
    "sel-forward": [*TREE, forwarder("f", "../p/c"), forwarder("g", "/user/gone"), TELL("f", "kept", "r"), EV("c", 2),
                   TELL("f", "nobody-sent"), EV("c", 3), TELL("g", "lost", "r"), EV("dl", 1), NOW("c"), NOW("dl")],
}


def main():
    out = os.path.join(ROOT, "cases", "scenario-actor-selection.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            line = {"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}
            f.write(json.dumps(line, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
