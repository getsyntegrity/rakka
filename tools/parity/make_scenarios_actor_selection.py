#!/usr/bin/env python3
"""Writes cases/scenario-actor-selection.jsonl: `actorSelection` from a string path (AC-REF-011) and what a path that
matches nothing does (AC-REF-014, AC-REF-013), and the `*` / `?` patterns (`selw-*`).

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

# the tree of ActorSelectionSpec "identify actors with wildcard selection correctly": a/b1, a/b2/c/d
WILD = [DL(), *S("r"), *S("a"), *child("a", "b1"), *child("a", "b2"), *child("b2", "c"), *child("c", "d")]
NODES = ("a", "b1", "b2", "c", "d")


def got(actor, count):
    """the first `count` events of an actor as a set: replies and copies from different actors arrive in any order"""
    return {**EV(actor, count), "sorted": True}


def finish(**counts):
    """a barrier through each actor, then all it recorded: `counts` includes its start and the barrier"""
    return [*[TELL(actor, "barrier") for actor in counts], *[got(actor, n) for actor, n in counts.items()],
            *[{"op": "events-sorted", "target": actor} for actor in counts]]


def asks(path, correlation, replies):
    """`Identify` through `path`, then the first `replies` events of the asker `r`"""
    return [select(path, ident(correlation)), got("r", replies)]


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
    # every match gets the message; `**` is no more than `*`; in a string path `?` starts the URI query, which is dropped
    "selw-fanout": [*WILD, select("/user/a/*", "children"), select("/user/a/b*", "prefix"), select("/user/a/**", "double"),
                    select("/user/a/*1", "suffix"), select("/user/*/b2/c", "inner"), select("user/a/*/c", "relative"),
                    select("pekko://sys/user/a/*/c/*", "uri"), inside("c", "../*", "own"), inside("b1", "../*", "siblings"),
                    inside("a", "*", "context"), select("/user/a/*/..", "twice"), select("/user/a/b?", "query"),
                    select("/user/a/b1?x", "query-name"), select("/user/a/?", "query-bare"), EV("dl", 1),
                    *finish(a=5, b1=9, b2=7, c=5, d=3), NOW("dl")],
    # after a single match a miss is a dead letter below that match; after two or more it is silent, except under `deadLetters`
    "selw-miss": [*WILD, select("/user/a/x*", "none"), select("/user/a/x*/deeper", "none-deep"), select("/user/a/b1/*", "leafless"),
                  select("/user/a/*1/nothing", "single-then-name"), select("/user/a/b*2/zz*/yy", "single-then-pattern"),
                  select("/user/a/*1/../x*", "single-then-up"), select("/user/a/*/zz", "fan-name"),
                  select("/user/a/*/zz*", "fan-pattern"), select("/user/a/*/c/zz", "fan-later"),
                  select("/user/a/x*", {"special": "PoisonPill"}), select("/*/../deadLetters/x", "fan-dead"),
                  inside("c", "../c*/nope", "context"), select("/user/missing", "last"), EV("dl", 10),
                  *finish(a=2, b1=2, b2=2, c=2, d=2), NOW("dl")],
    # `Identify`: each match answers; a miss answers None only while no wildcard has matched two or more
    "selw-identify": [*WILD, *asks("/user/a/*", "one", 3), *asks("/user/a/b1/*", "two", 4), *asks("/user/a/*/c", "three", 5),
                      *asks("/user/a/b2/*/d", "four", 6), *asks("/user/a/*/*/d", "five", 7), *asks("/user/a/*/c/*", "six", 8),
                      *asks("/user/a/b2/*/d/e", "seven", 9), *asks("/user/a/*/c/d/e", "eight", 9), *asks("/user/a/x*", "nine", 10),
                      *asks("/user/a/*1/nothing", "ten", 11), *asks("/*/../deadLetters/x", "eleven", 11),
                      *asks("/*/../deadLetters", "twelve", 13), *finish(a=2, b1=2, b2=2, c=2, d=2), *finish(r=14), NOW("dl")],
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
