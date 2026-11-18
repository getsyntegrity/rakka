#!/usr/bin/env python3
"""Writes cases/scenario-3b1.jsonl: failure handling with the default supervisor strategy (slice 3b, part 1).

Contracts: an exception in `receive` restarts the actor (preRestart -> postStop, new instance, postRestart -> preStart; the failed
message is lost, the mailbox and the watchers are untouched, children are stopped); `Kill` stops the actor (ActorKilledException);
an unhandled `Terminated` stops the watcher (DeathPactException); an exception in `preStart` stops the actor
(ActorInitializationException). Pekko 2.0.0 decides every expected trace; nothing here is written from the Rust side.

Behaviour options (same in Scenario.java and scenario.rs): `throw-on: <msg>` throws IllegalStateException(<msg>) after recording the
receive; `throw-in-prestart` throws from preStart; `ignore-terminated` leaves `Terminated` unhandled; `numbered` adds the per-instance
receive counter `n` to `received` events (a restart resets it). Message `{"special": "Kill"}` is Pekko's `Kill`.
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, WATCH, CHILDREN, EV, NOW, TELL, PILL, LIFE, DL, DO, STOPC, STOP, beh  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CASES = {}


def case(name, steps, system="sys"):
    assert name not in CASES, name
    assert len({n[:24] for n in CASES}) == len(CASES), f"case names must differ in the first 24 chars: {name}"
    CASES[name] = {"system": system, "steps": steps}


KILL = lambda r: {"op": "tell", "ref": r, "msg": {"special": "Kill"}}
# a round trip through an actor's own mailbox: everything sent before it has been processed
SYNC = lambda r: DO(r, "children")

# --- restart on an exception in receive -----------------------------------------------------------------------------
case("r01-restart-lifecycle", [*S("a", throw_on="boom"), TELL("a", "hi"), TELL("a", "boom"), TELL("a", "after"), EV("a", 8)])
case("r02-state-resets", [*S("a", throw_on="boom", numbered=True), TELL("a", "m1"), TELL("a", "m2"), TELL("a", "boom"), TELL("a", "m3"), EV("a", 9)])
case("r03-failing-message-lost", [DL(), *S("a", throw_on="boom"), TELL("a", "boom"), TELL("a", "x"), EV("a", 6), SYNC("a"), NOW("dl")])
case("r04-twice", [*S("a", throw_on="boom", numbered=True), TELL("a", "boom"), TELL("a", "boom"), TELL("a", "z"), EV("a", 12)])
case("r05-mailbox-kept-in-order", [*S("a", throw_on="boom"), {"op": "tell-seq", "ref": "a", "prefix": "q", "count": 3}, TELL("a", "boom"),
                                   {"op": "tell-seq", "ref": "a", "prefix": "r", "count": 3}, EV("a", 12)])
case("r06-watcher-not-notified", [*S("a", throw_on="boom"), *S("w"), WATCH("w", "a"), TELL("a", "boom"), TELL("a", "x"), EV("a", 7), SYNC("w"), NOW("w")])
case("r07-restart-stops-children", [*S("p", throw_on="boom"), C("p", "k1", "k1"), C("p", "k2", "k2"), WATCH("p", "k1"), TELL("p", "boom"), EV("k1", 2), EV("k2", 2),
                                    EV("p", 6), CHILDREN("p")])
case("r08-new-child-after-restart", [*S("p", throw_on="boom"), C("p", "k", "k1"), TELL("p", "boom"), EV("k1", 2), TELL("p", "go"), EV("p", 7), C("p", "k", "k2"), CHILDREN("p")])
case("r09-child-restarts-parent-untouched", [*S("p"), C("p", "k", "k", throw_on="boom"), WATCH("p", "k"), TELL("k", "boom"), TELL("k", "x"), EV("k", 7), SYNC("p"), NOW("p")])
case("r10-sender-of-failed-message", [*S("a", throw_on="boom"), *S("b"), TELL("a", "boom", "b"), TELL("a", "y", "b"), EV("a", 7)])
case("r11-restart-keeps-identity", [*S("a", throw_on="boom"), TELL("a", "boom"), EV("a", 6), TELL("a", "z"), EV("a", 7)])
case("r12-fail-in-batch-throughput", [{"op": "spawn", "name": "a", "as": "a", "behavior": beh("log", throw_on="boom")}, EV("a", 1),
                                      {"op": "tell-seq", "ref": "a", "prefix": "q", "count": 2}, TELL("a", "boom"), {"op": "tell-seq", "ref": "a", "prefix": "r", "count": 2}, EV("a", 10)])

# --- Kill ------------------------------------------------------------------------------------------------------------
case("k01-kill-stops", [*S("a"), *S("w"), WATCH("w", "a"), KILL("a"), EV("w", 2), EV("a", 2), NOW("a"), NOW("w")])
case("k02-kill-no-restart-hooks", [*S("a", throw_on="boom"), KILL("a"), EV("a", 2), {"op": "await-stopped", "ref": "a"}, NOW("a")])
case("k03-after-kill-dead-letters", [DL(), *S("a"), *S("w"), WATCH("w", "a"), KILL("a"), EV("w", 2), TELL("a", "late"), EV("dl", 1), NOW("dl")])
case("k04-queued-after-kill-dead", [DL(), *S("a", kind="gate"), TELL("a", "block"), EV("a", 2), KILL("a"), TELL("a", "m1"), TELL("a", "m2"), {"op": "release"}, EV("dl", 2), NOW("a")])
case("k05-kill-child-parent-sees-it", [*S("p"), C("p", "k", "k"), WATCH("p", "k"), KILL("k"), EV("p", 2), CHILDREN("p"), NOW("p")])
case("k06-kill-parent-stops-children", [*S("p"), C("p", "k", "k"), *S("w"), WATCH("w", "p"), KILL("p"), EV("w", 2), LIFE(6, ("k:post-stop", "p:post-stop")), CHILDREN("w")])
case("k07-kill-twice", [DL(), *S("a"), KILL("a"), KILL("a"), EV("a", 2), EV("dl", 1), NOW("dl")])

# --- DeathPactException: an unhandled Terminated stops the watcher --------------------------------------------------
case("d01-unhandled-terminated-stops", [*S("x"), *S("w", ignore_terminated=True), *S("v"), WATCH("w", "x"), WATCH("v", "w"), STOP("x"), EV("v", 2), EV("w", 2), NOW("w")])
case("d02-handled-terminated-survives", [*S("x"), *S("w"), WATCH("w", "x"), STOP("x"), EV("w", 2), TELL("w", "alive"), EV("w", 3)])
case("d03-no-restart-on-deathpact", [*S("x"), *S("w", ignore_terminated=True), WATCH("w", "x"), STOP("x"), EV("w", 2), {"op": "await-stopped", "ref": "w"}, NOW("w")])
case("d04-deathpact-in-child", [*S("p"), C("p", "x", "x"), C("p", "c", "c", ignore_terminated=True), WATCH("c", "x"), WATCH("p", "c"), STOPC("p", "x"), EV("p", 2), CHILDREN("p"), NOW("c")])
case("d05-handles-list-ignores-terminated", [*S("x"), {"op": "spawn", "name": "w", "as": "w", "behavior": beh("log", handles=["only"], ignore_terminated=True)}, EV("w", 1),
                                              WATCH("w", "x"), STOP("x"), EV("w", 2), {"op": "await-stopped", "ref": "w"}, NOW("w")])

# --- ActorInitializationException: preStart failure stops the actor ---------------------------------------------------
case("i01-prestart-failure-stops", [*[{"op": "spawn", "name": "a", "as": "a", "behavior": beh("log", throw_in_prestart=True)}], EV("a", 1), {"op": "await-stopped", "ref": "a"}, NOW("a")])
case("i02-prestart-failure-watcher", [*S("p"), C("p", "k", "k", throw_in_prestart=True), EV("k", 1), {"op": "await-stopped", "ref": "k"}, WATCH("p", "k"), EV("p", 2), CHILDREN("p"), NOW("k")])
case("i03-prestart-fail-dead-letters", [DL(), {"op": "spawn", "name": "a", "as": "a", "behavior": beh("log", throw_in_prestart=True)}, EV("a", 1), {"op": "await-stopped", "ref": "a"}, TELL("a", "late"), EV("dl", 1)])


def main():
    out = os.path.join(ROOT, "cases", "scenario-3b1.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, sc in CASES.items():
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(sc, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
