#!/usr/bin/env python3
"""Writes cases/scenario-2b.jsonl: Classic tell / receive / sender / mailbox / dead-letter scenarios.

Every scenario is deterministic by construction: steps wait for explicit confirmations (events,
`await-stopped`) or barriers, and races (tell vs stop, several concurrent senders) report only
invariants, never an interleaving. Pekko 2.0.0 (oracle) decides the expected trace.
"""
import json, os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CASES = {}


def case(name, steps, system="sys", dispatcher=None):
    assert name not in CASES, name
    sc = {"system": system, "steps": steps}
    if dispatcher:
        sc["dispatcher"] = dispatcher
    CASES[name] = sc


def spawn(label, kind="log", **cfg):
    return {"op": "spawn", "name": label, "as": label, "behavior": {"kind": kind, **cfg}}


tell = lambda ref, msg, frm=None: {"op": "tell", "ref": ref, "msg": msg, **({"from": frm} if frm else {})}
seq = lambda ref, prefix, n, frm=None: {"op": "tell-seq", "ref": ref, "prefix": prefix, "count": n, **({"from": frm} if frm else {})}
events = lambda target, n: {"op": "await-events", "target": target, "count": n}
PILL = {"special": "PoisonPill"}
DL = {"op": "subscribe", "channel": "dead-letters", "as": "dl"}
UN = {"op": "subscribe", "channel": "unhandled", "as": "un"}
stop = lambda r: {"op": "stop", "ref": r}
stopped = lambda r: {"op": "await-stopped", "ref": r}

# --- 1. tell and the sender the receiver observes ------------------------------------------------
case("snd-none", [spawn("a"), tell("a", "m"), events("a", 2)])
case("snd-explicit", [spawn("a"), spawn("b"), tell("a", "m", "b"), events("a", 2)])
case("snd-explicit-self", [spawn("a"), tell("a", "m", "a"), events("a", 2)])
case("snd-deadletters-ref", [spawn("a"), tell("a", "m", "deadLetters"), events("a", 2)])
case("snd-mixed-sequence", [spawn("a"), spawn("b"), tell("a", "1"), tell("a", "2", "b"), tell("a", "3"), tell("a", "4", "b"), events("a", 5)])
case("snd-anonymous-actor", [{"op": "spawn", "as": "a", "behavior": {"kind": "log"}}, {"op": "spawn", "as": "b", "behavior": {"kind": "log"}}, tell("a", "m", "b"), events("a", 2)])
case("snd-echo-replies-to-sender", [spawn("e", "echo"), spawn("b"), tell("e", "ping", "b"), events("b", 2), events("e", 2)])
case("snd-echo-without-sender", [spawn("e", "echo"), tell("e", "ping"), events("e", 2)])
case("snd-echo-sees-reply-sender", [spawn("e", "echo"), spawn("b"), tell("e", "x", "b"), tell("e", "y", "b"), events("b", 3)])
case("snd-forward-keeps-sender", [spawn("a"), spawn("f", "forward", to="a"), spawn("b"), tell("f", "m", "b"), events("a", 2), events("f", 2)])
case("snd-forward-no-sender", [spawn("a"), spawn("f", "forward", to="a"), tell("f", "m"), events("a", 2)])
case("snd-forward-chain", [spawn("a"), spawn("g", "forward", to="a"), spawn("f", "forward", to="g"), spawn("b"), tell("f", "m", "b"), events("a", 2)])
case("snd-first-event-is-pre-start", [spawn("a"), events("a", 1)])
case("snd-tell-before-create-runs", [spawn("a"), tell("a", "early"), tell("a", "early2"), events("a", 3)])

# --- 2. ordering ------------------------------------------------------------------------------
case("ord-three-messages", [spawn("a"), tell("a", "m1"), tell("a", "m2"), tell("a", "m3"), events("a", 4)])
case("ord-same-sender-seq-100", [spawn("a"), spawn("b"), seq("a", "m", 100, "b"), {"op": "await-received", "target": "a", "total": 100}])
case("ord-no-sender-seq-1000", [spawn("a"), seq("a", "m", 1000), {"op": "await-received", "target": "a", "total": 1000}])
case("ord-seq-two-receivers", [spawn("a"), spawn("b"), seq("a", "a", 200), seq("b", "b", 200),
                               {"op": "await-received", "target": "a", "total": 200}, {"op": "await-received", "target": "b", "total": 200}])
case("ord-interleaved-tells-two-receivers", [spawn("a"), spawn("b")] + [x for i in range(20) for x in (tell("a", f"a-{i}"), tell("b", f"b-{i}"))]
     + [{"op": "await-received", "target": "a", "total": 20}, {"op": "await-received", "target": "b", "total": 20}])
case("ord-barrier-exact-small", [spawn("a"), tell("a", "x"), tell("a", "y"), tell("a", "z"), tell("a", "barrier"), events("a", 5)])
for n in (2, 3, 4):
    steps = [spawn("t")]
    for i in range(n):
        steps += [spawn(f"s{i}", "blast", to="t", prefix=f"s{i}", count=150)]
    steps += [tell(f"s{i}", "go") for i in range(n)]
    steps += [{"op": "await-received", "target": "t", "total": 150 * n}]
    case(f"ord-{n}-concurrent-senders", steps)
steps = [spawn("t")] + [spawn(f"s{i}", "blast", to="t", prefix=f"s{i}", count=100) for i in range(8)]
steps += [tell(f"s{i}", "go") for i in range(8)] + [{"op": "await-received", "target": "t", "total": 800}]
case("ord-8-concurrent-senders", steps)
case("ord-same-sender-two-bursts", [spawn("t"), spawn("s", "blast", to="t", prefix="s", count=50), tell("s", "go"), tell("s", "go2"),
                                 {"op": "await-received", "target": "t", "total": 100}], )

# --- 3. dead letters ---------------------------------------------------------------------------
case("dl-after-stop-no-sender", [spawn("a"), DL, stop("a"), stopped("a"), tell("a", "late"), events("dl", 1)])
case("dl-after-stop-explicit-sender", [spawn("a"), spawn("b"), DL, stop("a"), stopped("a"), tell("a", "late", "b"), events("dl", 1)])
case("dl-after-stop-several", [spawn("a"), spawn("b"), DL, stop("a"), stopped("a"), tell("a", "1"), tell("a", "2", "b"), tell("a", "3"), events("dl", 3)])
case("dl-after-stop-seq-50", [spawn("a"), DL, stop("a"), stopped("a"), seq("a", "d", 50), events("dl", 50)])
case("dl-after-poison-pill", [spawn("a"), DL, tell("a", PILL), stopped("a"), tell("a", "late"), events("dl", 1)])
case("dl-to-deadletters-no-sender", [DL, tell("deadLetters", "m"), events("dl", 1)])
case("dl-to-deadletters-explicit-sender", [spawn("b"), DL, tell("deadLetters", "m", "b"), events("dl", 1)])
case("dl-two-stopped-actors", [spawn("a"), spawn("b"), DL, stop("a"), stop("b"), stopped("a"), stopped("b"), tell("a", "x"), tell("b", "y"), events("dl", 2)])
case("dl-forward-to-stopped", [spawn("a"), spawn("f", "forward", to="a"), spawn("b"), DL, stop("a"), stopped("a"), tell("f", "m", "b"), events("dl", 1)])
case("dl-echo-reply-to-stopped-sender", [spawn("e", "echo"), spawn("b"), DL, stop("b"), stopped("b"), tell("e", "m", "b"), events("dl", 1)])
case("dl-pill-to-dead-actor-suppressed", [spawn("a"), DL, tell("a", PILL), stopped("a"), tell("a", PILL), tell("a", "after"), events("dl", 1)])
case("dl-none-before-stop", [spawn("a"), DL, tell("a", "m"), events("a", 2), tell("a", "barrier"), events("a", 3), stop("a"), stopped("a"), tell("a", "late"), events("dl", 1)])
case("dl-unsubscribed-actor-name", [DL, spawn("a"), stop("a"), stopped("a"), tell("a", "x"), events("dl", 1)])

# --- 4. tell vs stop: invariants only -------------------------------------------------------------
for n in (1, 10, 100, 1000):
    case(f"race-tell-then-stop-{n}", [spawn("a"), DL, seq("a", "m", n), stop("a"), seq("a", "late", 3),
                                      {"op": "await-accounted", "target": "a", "dl": "dl", "total": n + 3}])
case("race-pill-after-seq-10", [spawn("a"), DL, seq("a", "m", 10), tell("a", PILL), seq("a", "late", 5),
                                {"op": "await-accounted", "target": "a", "dl": "dl", "total": 15, "counts": True}])
case("race-pill-after-seq-500", [spawn("a"), DL, seq("a", "m", 500), tell("a", PILL), seq("a", "late", 20),
                                 {"op": "await-accounted", "target": "a", "dl": "dl", "total": 520, "counts": True}])
case("race-stop-then-pill", [spawn("a"), DL, seq("a", "m", 50), stop("a"), tell("a", PILL), seq("a", "late", 5),
                             {"op": "await-accounted", "target": "a", "dl": "dl", "total": 55}])
case("race-concurrent-senders-and-stop", [spawn("t"), DL] + [spawn(f"s{i}", "blast", to="t", prefix=f"s{i}", count=100) for i in range(4)]
     + [tell(f"s{i}", "go") for i in range(4)] + [stop("t"), stopped("t")])

# --- 5. throughput: messages per turn on a single-thread dispatcher -----------------------------------
# Barrier first: every actor must have run its `Create` (pre-start event) so that no mailbox is still
# queued from creation; otherwise the order in which mailboxes enter the single-thread queue is a race.
GATED = lambda labels, plan: ([spawn("gate", "gate")] + [spawn(l) for l in labels]
                              + [events(l, 1) for l in ["gate"] + labels] + [tell("gate", "block")] + plan
                              + [{"op": "release"}])
for thr in (1, 2, 3, 5, 7, 10, 100):
    case(f"tput-2-actors-thr{thr}", GATED(["a", "b"], [seq("a", "a", 12), seq("b", "b", 12)]) + [{"op": "await-order", "count": 25}],
         dispatcher={"parallelism": 1, "throughput": thr})
case("tput-3-uneven-thr3", GATED(["a", "b", "c"], [seq("a", "a", 12), seq("b", "b", 3), seq("c", "c", 7)]) + [{"op": "await-order", "count": 23}],
     dispatcher={"parallelism": 1, "throughput": 3})
case("tput-3-uneven-thr5", GATED(["a", "b", "c"], [seq("a", "a", 12), seq("b", "b", 3), seq("c", "c", 7)]) + [{"op": "await-order", "count": 23}],
     dispatcher={"parallelism": 1, "throughput": 5})
case("tput-fill-order-b-first", GATED(["a", "b"], [seq("b", "b", 8), seq("a", "a", 8)]) + [{"op": "await-order", "count": 17}],
     dispatcher={"parallelism": 1, "throughput": 3})
case("tput-interleaved-enqueue", GATED(["a", "b"], [x for i in range(6) for x in (tell("a", f"a-{i}"), tell("b", f"b-{i}"))]) + [{"op": "await-order", "count": 13}],
     dispatcher={"parallelism": 1, "throughput": 4})
case("tput-single-actor-thr2", GATED(["a"], [seq("a", "a", 9)]) + [{"op": "await-order", "count": 10}], dispatcher={"parallelism": 1, "throughput": 2})
case("tput-default-settings-single-actor", [spawn("a"), seq("a", "a", 30), {"op": "await-received", "target": "a", "total": 30}])
case("tput-thr1-two-threads-invariants", [spawn("a"), spawn("b"), seq("a", "a", 100), seq("b", "b", 100),
                                          {"op": "await-received", "target": "a", "total": 100}, {"op": "await-received", "target": "b", "total": 100}],
     dispatcher={"parallelism": 2, "throughput": 1})

# --- 6. system messages between user messages: stop-on / PoisonPill inside a queue -------------------------
case("turn-stop-on-in-queue-dl", [DL, spawn("gate", "gate"), spawn("a", "stop-on", msg="bye"), events("gate", 1), events("a", 1), tell("gate", "block"),
                                  tell("a", "m1"), tell("a", "bye"), tell("a", "m2"), tell("a", "m3"), {"op": "release"},
                                  events("a", 4), events("dl", 2)], dispatcher={"parallelism": 1, "throughput": 5})
case("turn-pill-in-queue", [DL, spawn("gate", "gate"), spawn("a"), events("gate", 1), events("a", 1), tell("gate", "block"),
                            tell("a", "m1"), tell("a", "m2"), tell("a", PILL), tell("a", "m3"), {"op": "release"},
                            events("a", 4), events("dl", 1)], dispatcher={"parallelism": 1, "throughput": 5})
case("turn-pill-first", [DL, spawn("gate", "gate"), spawn("a"), events("gate", 1), events("a", 1), tell("gate", "block"), tell("a", PILL), tell("a", "m1"), tell("a", "m2"),
                         {"op": "release"}, events("a", 2), events("dl", 2)], dispatcher={"parallelism": 1, "throughput": 5})
case("turn-stop-op-vs-queue-invariant", [spawn("a"), DL, seq("a", "m", 20), stop("a"), {"op": "await-accounted", "target": "a", "dl": "dl", "total": 20}])
case("turn-terminate-with-queued", [spawn("gate", "gate"), spawn("a"), DL, events("gate", 1), events("a", 1), tell("gate", "block"), seq("a", "m", 5), {"op": "terminate"}, {"op": "release"},
                                    {"op": "await-system-terminated"}], dispatcher={"parallelism": 1, "throughput": 5})

# --- 7. unhandled messages ---------------------------------------------------------------------
case("unh-handles-only-ok", [UN, spawn("h", "log", handles=["ok"]), tell("h", "ok"), tell("h", "nope"), tell("h", "ok"), events("h", 3), events("un", 1)])
case("unh-sender-explicit", [UN, spawn("h", "log", handles=["ok"]), spawn("b"), tell("h", "nope", "b"), events("un", 1)])
case("unh-sender-none", [UN, spawn("h", "log", handles=["ok"]), tell("h", "nope"), events("un", 1)])
case("unh-noop-actor-everything", [UN, {"op": "spawn", "name": "n", "as": "n"}, tell("n", "x"), tell("n", "y"), events("un", 2)])
case("unh-order-preserved", [UN, spawn("h", "log", handles=["ok"])] + [tell("h", f"u{i}") for i in range(10)] + [events("un", 10)])
case("unh-handled-not-published", [UN, spawn("h", "log", handles=["ok"]), tell("h", "ok"), tell("h", "nope"), events("un", 1), events("h", 2)])
case("unh-pill-never-unhandled", [UN, DL, {"op": "spawn", "name": "n", "as": "n"}, tell("n", PILL), stopped("n"), tell("n", "late"), events("dl", 1)])

# --- 8. lifecycle hooks around messages ---------------------------------------------------------------
case("life-pre-start-then-messages-then-post-stop", [spawn("a"), tell("a", "m1"), tell("a", "m2"), tell("a", PILL), events("a", 4)])
case("life-stop-op-post-stop", [spawn("a"), tell("a", "m"), tell("a", "barrier"), events("a", 3), stop("a"), stopped("a"), events("a", 4)])
case("life-terminate-runs-post-stop", [spawn("a"), spawn("b"), events("a", 1), events("b", 1), {"op": "terminate"}, {"op": "await-system-terminated"}, events("a", 2), events("b", 2)])
case("life-stop-on-self-stop", [spawn("a", "stop-on", msg="bye"), tell("a", "hi"), tell("a", "bye"), events("a", 4)])
case("life-two-stop-ons", [spawn("a", "stop-on", msg="bye"), spawn("b", "stop-on", msg="bye"), tell("a", "bye"), tell("b", "bye"), events("a", 3), events("b", 3)])


def main():
    assert len({n[:24] for n in CASES}) == len(CASES), "case names must differ in the first 24 chars (rust-anything truncates recording names)"
    out = os.path.join(ROOT, "cases", "scenario-2b.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, sc in CASES.items():
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(sc, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
