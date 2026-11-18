#!/usr/bin/env python3
"""Writes cases/scenario-3a.jsonl: Classic local hierarchy and DeathWatch scenarios (slice 3a).

Checkpoint 1 (h*): children, lookup by name, sibling collisions, parent termination, name reuse and the identity
(path + uid) of the old and the new reference. Checkpoint 2 (w*): watch / unwatch on live and dead actors, repeated
watch, a new actor with the old path. The oracle (real Pekko 2.0.0) decides every expected trace.

Coordination rules (nothing here relies on ActorRef.isTerminated):
  * a child's name is free again once its *parent* has processed the child's termination. A parent that watches the child
    has that confirmation when it receives `Terminated` (the name is released in the same system-message step that queues
    it), or the harness polls `context.child(name)` (`await-child-gone`). A third-party watcher is NOT a confirmation.
  * an actor's mailbox is closed before any watcher is notified, so a watcher's `Terminated` confirms the death.
  * "nothing else happened" is checked with `events-now` after a command round trip through the same mailbox.
  * order between independent actors is only compared through `await-life` constraints that Pekko guarantees.
"""
import json, os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CASES = {}


def case(name, steps, system="sys"):
    assert name not in CASES, name
    assert len({n[:24] for n in CASES}) == len(CASES), f"case names must differ in the first 24 chars: {name}"
    CASES[name] = {"system": system, "steps": steps}


def beh(kind="log", **cfg):
    return {"kind": kind, **{k.replace("_", "-"): v for k, v in cfg.items()}}


def S(label, kind="log", **cfg):
    """top-level actor named after its label, started (pre-start confirmed)"""
    return [{"op": "spawn", "name": label, "as": label, "behavior": beh(kind, **cfg)}, {"op": "await-events", "target": label, "count": 1}]


def C(parent, name, label, kind="log", **cfg):
    """child created by `parent` itself (context.actorOf)"""
    return {"op": "do", "ref": parent, "cmd": {"c": "spawn-child", "name": name, "as": label, "behavior": beh(kind, **cfg)}}


def CA(parent, label=None):
    """anonymous child"""
    return {"op": "do", "ref": parent, "cmd": {"c": "spawn-child", **({"as": label, "behavior": beh()} if label else {})}}


def DO(ref, c, **kw):
    return {"op": "do", "ref": ref, "cmd": {"c": c, **kw}}


def DOA(ref, c, **kw):
    return {"op": "do", "ref": ref, "async": True, "cmd": {"c": c, **kw}}


WATCH = lambda who, target: DO(who, "watch", target=target)
UNWATCH = lambda who, target: DO(who, "unwatch", target=target)
STOPC = lambda who, target: DO(who, "stop", target=target)
CHILD = lambda who, name: DO(who, "child", name=name)
CHILDREN = lambda who: DO(who, "children")
EV = lambda t, n: {"op": "await-events", "target": t, "count": n}
NOW = lambda t: {"op": "events-now", "target": t}
STOP = lambda r: {"op": "stop", "ref": r}
TELL = lambda r, m, frm=None: {"op": "tell", "ref": r, "msg": m, **({"from": frm} if frm else {})}
PILL = lambda r: {"op": "tell", "ref": r, "msg": {"special": "PoisonPill"}}
CMP = lambda a, b: {"op": "compare-refs", "a": a, "b": b}
LIFE = lambda n, *before: {"op": "await-life", "count": n, "before": [list(b) for b in before]}
DL = lambda label="dl": {"op": "subscribe", "channel": "dead-letters", "as": label}
GONE = lambda ref, name: {"op": "await-child-gone", "ref": ref, "name": name}


def barrier(ref):
    """a command round trip through the actor's own mailbox: everything queued before it has been processed"""
    return CHILDREN(ref)


# =========================== checkpoint 1: hierarchy and identity ===========================
case("h01-create-children", [*S("p"), C("p", "c1", "c1"), C("p", "c2", "c2"), C("c1", "g", "g"), CHILDREN("p"), CHILDREN("c1"), CHILDREN("c2"),
                             CHILD("p", "c1"), CHILD("p", "g"), CHILD("p", "nope"), CHILD("c1", "g")])
case("h02-anonymous-children", [*S("p"), CA("p"), CA("p"), CA("p", "a3"), CHILDREN("p"), CHILD("p", "$a"), CHILD("p", "$c")])
case("h03-sibling-collision", [*S("p"), C("p", "c", "c1"), C("p", "c", "c2"), C("p", "c", "c3"), CHILDREN("p"), CHILD("p", "c")])
case("h04-same-name-two-parents", [*S("p1"), *S("p2"), C("p1", "c", "a"), C("p2", "c", "b"), CHILDREN("p1"), CHILDREN("p2"), CMP("a", "b")])
case("h05-collision-in-grandchild", [*S("p"), C("p", "c", "c"), C("c", "g", "g1"), C("c", "g", "g2"), C("p", "g", "g3"), CHILDREN("c"), CHILDREN("p")])
for i, n in enumerate(["", "$x", "$", "a b", "a/b", "a#b", "a%", "a%zz", "é", "ok%41", "a%41b", "..", "x" * 200, "a?b", "-", "a:b"]):
    case(f"h06-child-name-{i:02d}", [*S("p"), {"op": "do", "ref": "p", "cmd": {"c": "spawn-child", "name": n, "as": "k"}}, CHILDREN("p")])
case("h07-names-case-and-escape", [*S("p"), C("p", "a", "k1"), C("p", "A", "k2"), C("p", "%61", "k3"), C("p", "a", "k4"), CHILDREN("p")])

# parent termination with live children (lifecycle order only where Pekko guarantees it)
TREE = [*S("p"), C("p", "c1", "c1"), C("p", "c2", "c2"), C("c1", "g1", "g1"), C("c1", "g2", "g2")]
PRE = 5  # pre-starts of the tree
case("h10-stop-parent-system-stop", [*TREE, LIFE(PRE), STOP("p"),
                                     LIFE(PRE + 5, ("g1:post-stop", "c1:post-stop"), ("g2:post-stop", "c1:post-stop"), ("c1:post-stop", "p:post-stop"), ("c2:post-stop", "p:post-stop"))])
case("h11-stop-parent-poison-pill", [*TREE, LIFE(PRE), PILL("p"),
                                     LIFE(PRE + 5, ("g1:post-stop", "c1:post-stop"), ("g2:post-stop", "c1:post-stop"), ("c1:post-stop", "p:post-stop"), ("c2:post-stop", "p:post-stop"))])
case("h12-stop-parent-self-stop-cmd", [*TREE, LIFE(PRE), STOPC("p", "p"),
                                       LIFE(PRE + 5, ("g1:post-stop", "c1:post-stop"), ("c1:post-stop", "p:post-stop"), ("c2:post-stop", "p:post-stop"))])
case("h13-stop-middle-keeps-siblings", [*TREE, LIFE(PRE), STOPC("p", "c1"),
                                        LIFE(PRE + 3, ("g1:post-stop", "c1:post-stop"), ("g2:post-stop", "c1:post-stop")), CHILDREN("c2")])
case("h14-watcher-sees-whole-subtree", [*S("w"), *TREE, LIFE(PRE + 1), WATCH("w", "p"), WATCH("w", "c1"), WATCH("w", "g1"), WATCH("w", "g2"), WATCH("w", "c2"), STOP("p"),
                                        LIFE(PRE + 1 + 5 + 5, ("g1:post-stop", "c1:post-stop"), ("c1:post-stop", "p:post-stop"), ("c2:post-stop", "p:post-stop"),
                                             ("p:post-stop", "w:terminated:p"), ("c1:post-stop", "w:terminated:c1"), ("g1:post-stop", "w:terminated:g1"))])
case("h15-parent-watching-child-no-terminated-when-stopping", [*S("p"), C("p", "c1", "c1"), C("p", "c2", "c2"), WATCH("p", "c1"), WATCH("p", "c2"), LIFE(3), STOP("p"),
                                                               LIFE(3 + 3), {"op": "await-events", "target": "p", "count": 2}, NOW("p")])
case("h16-post-stop-spawn-no-children", [*S("p", post_stop_spawn="late"), STOP("p"), EV("p", 3), NOW("p")])
case("h17-post-stop-spawn-with-child", [*S("p", post_stop_spawn="late"), C("p", "c", "c"), LIFE(2), STOP("p"), EV("p", 3), NOW("p")])
case("h18-post-stop-spawn-in-child", [*S("p"), C("p", "c", "c", post_stop_spawn="late"), LIFE(2), STOPC("p", "c"), EV("c", 3), NOW("c"), CHILDREN("p")])
case("h19-stop-child-twice", [*S("p"), C("p", "c", "c"), WATCH("p", "c"), STOPC("p", "c"), STOPC("p", "c"), EV("p", 2), barrier("p"), NOW("p")])
case("h20-self-stop-removes-child", [*S("p"), C("p", "c", "c", "stop-on", msg="bye"), WATCH("p", "c"), TELL("c", "bye"), EV("p", 2), CHILDREN("p"), CHILD("p", "c")])
case("h21-poison-pill-child", [*S("p"), C("p", "c", "c"), WATCH("p", "c"), PILL("c"), EV("p", 2), CHILDREN("p")])
case("h22-context-stop-of-non-child", [*S("a"), *S("b"), LIFE(2), STOPC("a", "b"), LIFE(3, ("b:pre-start", "b:post-stop")), CHILDREN("a")])
case("h23-stop-ref-of-grandchild-by-grandparent", [*S("p"), C("p", "c", "c"), C("c", "g", "g"), LIFE(3), STOPC("p", "g"), LIFE(4), CHILDREN("c"), CHILDREN("p")])

# name reuse after the parent confirmed the release; identity of old and new reference
REUSE = [DL(), *S("p"), C("p", "c", "c1"), EV("c1", 1), WATCH("p", "c1"), STOPC("p", "c1"), EV("p", 2)]
case("h30-reuse-name-watch-barrier", [*REUSE, C("p", "c", "c2"), EV("c2", 1), CMP("c1", "c2"), CHILD("p", "c"), CHILDREN("p")])
case("h31-reuse-name-child-gone-barrier", [*S("p"), C("p", "c", "c1"), EV("c1", 1), STOPC("p", "c1"), GONE("p", "c"), CHILDREN("p"),
                                           C("p", "c", "c2"), EV("c2", 1), CMP("c1", "c2"), CHILD("p", "c")])
case("h32-old-ref-tell-is-dead-letter", [*REUSE, C("p", "c", "c2"), EV("c2", 1), TELL("c1", "to-old"), EV("dl", 1), TELL("c2", "to-new"), EV("c2", 2), NOW("dl"), NOW("c1")])
case("h33-old-ref-tell-with-sender", [*REUSE, C("p", "c", "c2"), EV("c2", 1), TELL("c1", "from-p", "p"), TELL("c2", "from-p", "p"), EV("dl", 1), EV("c2", 2)])
case("h34-stop-by-old-ref-spares-new", [*REUSE, C("p", "c", "c2"), EV("c2", 1), STOPC("p", "c1"), TELL("c2", "alive"), EV("c2", 2), CHILD("p", "c"), barrier("p"), NOW("c2")])
case("h35-poison-old-ref-spares-new", [*REUSE, C("p", "c", "c2"), EV("c2", 1), PILL("c1"), TELL("c2", "alive"), EV("c2", 2), CHILD("p", "c"), barrier("p"), NOW("c2")])
case("h36-reuse-four-times", [*S("p"), *[s for i in range(1, 5) for s in (
    [C("p", "c", f"c{i}"), EV(f"c{i}", 1)] + ([WATCH("p", f"c{i}"), STOPC("p", f"c{i}"), EV("p", 1 + i)] if i < 4 else []))],
    CMP("c1", "c2"), CMP("c2", "c3"), CMP("c1", "c4"), CMP("c3", "c4"), CHILD("p", "c")])
case("h37-compare-self", [*S("p"), C("p", "c", "c1"), CMP("c1", "c1"), CMP("p", "c1")])
case("h38-find-by-name-returns-new", [*REUSE, C("p", "c", "c2"), CHILD("p", "c"), CHILDREN("p")])
case("h39-reuse-after-parent-child-stop-by-self", [*S("p"), C("p", "c", "c1", "stop-on", msg="bye"), WATCH("p", "c1"), TELL("c1", "bye"), EV("p", 2),
                                                   C("p", "c", "c2"), CMP("c1", "c2"), CHILD("p", "c")])
case("h40-reuse-under-grandparent", [*S("p"), C("p", "c", "c"), C("c", "g", "g1"), WATCH("c", "g1"), STOPC("c", "g1"), EV("c", 2), C("c", "g", "g2"), CMP("g1", "g2"), CHILDREN("c")])
case("h41-reuse-anonymous-names-differ", [*S("p"), CA("p", "x1"), STOPC("p", "x1"), CA("p", "x2"), CMP("x1", "x2")])
case("h42-dead-letter-to-child-of-dead-parent", [DL(), *S("w"), *S("p"), C("p", "c", "c"), LIFE(3), WATCH("w", "c"), WATCH("w", "p"), STOP("p"), LIFE(7, ("c:post-stop", "p:post-stop")),
                                                 TELL("c", "late-c"), TELL("p", "late-p"), EV("dl", 2)])

# =========================== checkpoint 2: watch / unwatch ===========================
case("w01-watch-live-then-stop", [*S("w"), *S("t"), WATCH("w", "t"), STOP("t"), EV("w", 2)])
case("w02-watch-live-poison-pill", [*S("w"), *S("t"), WATCH("w", "t"), PILL("t"), EV("w", 2)])
case("w03-watch-live-stop-on-self", [*S("w"), *S("t", "stop-on", msg="bye"), WATCH("w", "t"), TELL("t", "bye"), EV("w", 2)])
case("w04-watch-already-terminated", [DL(), *S("w0"), *S("t"), *S("w"), WATCH("w0", "t"), STOP("t"), EV("w0", 2), WATCH("w", "t"), EV("w", 2), TELL("t", "x"), EV("dl", 1), NOW("dl")])
case("w05-watch-dead-twice-two-terminated", [*S("w0"), *S("t"), *S("w"), WATCH("w0", "t"), STOP("t"), EV("w0", 2), WATCH("w", "t"), WATCH("w", "t"), EV("w", 2), TELL("w", "ping"), EV("w", 3), barrier("w"), NOW("w")])
case("w06-repeated-watch-one-terminated", [*S("w"), *S("t"), WATCH("w", "t"), WATCH("w", "t"), WATCH("w", "t"), STOP("t"), EV("w", 2), TELL("w", "ping"), EV("w", 3), barrier("w"), NOW("w")])
case("w07-unwatch-before-stop", [*S("w"), *S("w0"), *S("t"), WATCH("w", "t"), WATCH("w0", "t"), UNWATCH("w", "t"), STOP("t"), EV("w0", 2), TELL("w", "ping"), EV("w", 2), barrier("w"), NOW("w")])
case("w08-unwatch-never-watched", [*S("w"), *S("w0"), *S("t"), WATCH("w0", "t"), UNWATCH("w", "t"), STOP("t"), EV("w0", 2), TELL("w", "ping"), EV("w", 2), barrier("w"), NOW("w")])
case("w09-unwatch-twice", [*S("w"), *S("w0"), *S("t"), WATCH("w", "t"), WATCH("w0", "t"), UNWATCH("w", "t"), UNWATCH("w", "t"), STOP("t"), EV("w0", 2), TELL("w", "ping"), EV("w", 2), barrier("w"), NOW("w")])
case("w10-watch-unwatch-watch", [*S("w"), *S("t"), WATCH("w", "t"), UNWATCH("w", "t"), WATCH("w", "t"), STOP("t"), EV("w", 2), TELL("w", "ping"), EV("w", 3), barrier("w"), NOW("w")])
case("w11-two-watchers-one-each", [*S("w1"), *S("w2"), *S("t"), WATCH("w1", "t"), WATCH("w2", "t"), STOP("t"), EV("w1", 2), EV("w2", 2), TELL("w1", "ping"), TELL("w2", "ping"), EV("w1", 3), EV("w2", 3), barrier("w1"), barrier("w2"), NOW("w1"), NOW("w2")])
case("w12-watch-self-is-noop", [*S("w"), WATCH("w", "w"), TELL("w", "ping"), EV("w", 2), barrier("w"), NOW("w")])
case("w13-unwatch-self-is-noop", [*S("w"), UNWATCH("w", "w"), TELL("w", "ping"), EV("w", 2), barrier("w"), NOW("w")])
case("w14-watcher-dies-first", [DL(), *S("w"), *S("w0"), *S("t"), WATCH("w", "t"), WATCH("w0", "t"), STOP("w"), LIFE(4, ("w:pre-start", "w:post-stop")), STOP("t"), EV("w0", 2), TELL("w", "x"), EV("dl", 1), NOW("dl")])
case("w15-watch-then-stop-watcher-no-late-terminated", [*S("w"), *S("t"), WATCH("w", "t"), STOP("w"), LIFE(3), NOW("w")])
case("w16-terminated-sender-is-watched", [*S("w"), *S("t"), WATCH("w", "t"), STOP("t"), EV("w", 2), CMP("t", "t")])
case("w17-watch-child-and-grandchild", [*S("w"), *S("p"), C("p", "c", "c"), LIFE(3), WATCH("w", "c"), STOPC("p", "c"), EV("w", 2)])
case("w18-watch-parent-from-child", [*S("p"), C("p", "c", "c"), LIFE(2), WATCH("c", "p"), STOP("p"), EV("c", 1), LIFE(4, ("p:post-stop", "c:post-stop"))])
case("w19-child-watches-sibling", [*S("p"), C("p", "a", "a"), C("p", "b", "b"), LIFE(3), WATCH("a", "b"), STOPC("p", "b"), EV("a", 2)])
case("w20-watch-from-many-then-stop-all", [*S("t"), *[s for i in range(3) for s in S(f"w{i}")], *[WATCH(f"w{i}", "t") for i in range(3)], STOP("t"),
                                            *[EV(f"w{i}", 2) for i in range(3)]])

# unwatch cancels a Terminated that is already queued for the watcher (terminatedQueued): the watcher is blocked while the
# watched actor dies, and the unwatch command sits in its mailbox before the Terminated can be queued behind it.
case("w21-unwatch-cancels-queued-terminated", [*S("w", "gate"), *S("w0"), *S("t"), WATCH("w", "t"), WATCH("w0", "t"), TELL("w", "block"), EV("w", 2),
                                               DOA("w", "unwatch", target="t"), STOP("t"), EV("w0", 2), {"op": "release"},
                                               TELL("w", "ping"), EV("w", 3), barrier("w"), NOW("w")])
case("w22-queued-terminated-delivered-without-unwatch", [*S("w", "gate"), *S("w0"), *S("t"), WATCH("w", "t"), WATCH("w0", "t"), TELL("w", "block"), EV("w", 2),
                                                         STOP("t"), EV("w0", 2), {"op": "release"}, EV("w", 3)])

# the same, but the watcher is the *parent* of the watched actor: the parent gets its DeathWatchNotification before any other
# watcher, so once w0 has seen the death the notification is already in the parent's system queue (no timing assumption).
case("w23-parent-unwatch-cancels-queued-terminated", [*S("w", "gate"), *S("w0"), C("w", "t", "t"), WATCH("w", "t"), WATCH("w0", "t"), TELL("w", "block"), EV("w", 2),
                                                      DOA("w", "unwatch", target="t"), PILL("t"), EV("w0", 2), {"op": "release"},
                                                      GONE("w", "t"), TELL("w", "ping"), EV("w", 3), barrier("w"), NOW("w")])
case("w24-parent-queued-terminated-delivered", [*S("w", "gate"), *S("w0"), C("w", "t", "t"), WATCH("w", "t"), WATCH("w0", "t"), TELL("w", "block"), EV("w", 2),
                                                PILL("t"), EV("w0", 2), {"op": "release"}, EV("w", 3), GONE("w", "t")])

# a new actor with the path of the previous one: every notification belongs to the instance that was watched
IDENT = [*S("p"), *S("w"), C("p", "t", "t1"), WATCH("p", "t1"), WATCH("w", "t1"), STOPC("p", "t1"), EV("p", 2), EV("w", 2)]
case("i01-new-instance-not-notified-for-old", [*IDENT, C("p", "t", "t2"), EV("t2", 1), STOPC("p", "t2"), LIFE(8, ("t2:pre-start", "t2:post-stop")), TELL("w", "ping"), EV("w", 3), barrier("w"), NOW("w")])
case("i02-watch-new-then-stop", [*IDENT, C("p", "t", "t2"), WATCH("w", "t2"), WATCH("p", "t2"), STOPC("p", "t2"), EV("w", 3), EV("p", 3), NOW("w")])
case("i03-watch-old-after-new-exists", [*IDENT, C("p", "t", "t2"), EV("t2", 1), WATCH("w", "t1"), EV("w", 3), barrier("w"), NOW("w"), TELL("t2", "alive"), EV("t2", 2)])
case("i04-watch-old-and-new", [*IDENT, C("p", "t", "t2"), EV("t2", 1), WATCH("w", "t1"), WATCH("w", "t2"), WATCH("p", "t2"), EV("w", 3), STOPC("p", "t2"), EV("w", 4), EV("p", 3), barrier("w"), NOW("w")])
case("i05-unwatch-old-keeps-watch-on-new", [*IDENT, C("p", "t", "t2"), WATCH("w", "t2"), WATCH("p", "t2"), UNWATCH("w", "t1"), STOPC("p", "t2"), EV("p", 3), EV("w", 3), barrier("w"), NOW("w")])
case("i06-unwatch-new-keeps-nothing", [*IDENT, C("p", "t", "t2"), WATCH("w", "t2"), WATCH("p", "t2"), UNWATCH("w", "t2"), STOPC("p", "t2"), EV("p", 3), TELL("w", "ping"), EV("w", 3), barrier("w"), NOW("w")])
case("i07-terminated-events-name-the-instance", [*IDENT, C("p", "t", "t2"), WATCH("p", "t2"), STOPC("p", "t2"), EV("p", 3), C("p", "t", "t3"), WATCH("p", "t3"), STOPC("p", "t3"), EV("p", 4), NOW("p")])
case("i08-old-ref-watch-existence-not-confirmed", [*IDENT, C("p", "t", "t2"), EV("t2", 1), WATCH("p", "t1"), EV("p", 3), NOW("p"), CMP("t1", "t2"), CHILD("p", "t"), CHILDREN("p"), TELL("t2", "alive"), EV("t2", 2)])
case("i15-stale-terminated-keeps-new-child", [*IDENT, C("p", "t", "t2"), EV("t2", 1), WATCH("p", "t1"), EV("p", 3), WATCH("p", "t2"), STOPC("p", "t2"), EV("p", 4), GONE("p", "t"), CHILDREN("p"), C("p", "t", "t3"), CHILD("p", "t")])
case("i09-watch-via-parent-barrier-only", [*S("p"), C("p", "t", "t1"), WATCH("p", "t1"), STOPC("p", "t1"), EV("p", 2), C("p", "t", "t2"), WATCH("p", "t2"), STOPC("p", "t2"), EV("p", 3), C("p", "t", "t3"), CMP("t2", "t3"), CMP("t1", "t3")])
case("i10-two-watchers-different-instances", [*IDENT, *S("w2"), C("p", "t", "t2"), WATCH("w2", "t2"), WATCH("p", "t2"), STOPC("p", "t2"), EV("w2", 2), EV("p", 3), TELL("w2", "ping"), EV("w2", 3), NOW("w"), NOW("w2")])
case("i11-dead-letters-name-the-old-path", [DL(), *S("p"), C("p", "t", "t1"), WATCH("p", "t1"), STOPC("p", "t1"), EV("p", 2), TELL("t1", "m1"), EV("dl", 1), C("p", "t", "t2"), EV("t2", 1), TELL("t1", "m2"), TELL("t2", "m3"), EV("dl", 2), EV("t2", 2), NOW("dl"), NOW("t2")])
case("i12-new-instance-watches-old-ref", [*IDENT, C("p", "t", "t2"), EV("t2", 1), WATCH("t2", "t1"), EV("t2", 2), NOW("t2"), TELL("t2", "alive"), EV("t2", 3)])
case("i13-new-instance-unwatches-old-ref", [*IDENT, C("p", "t", "t2"), EV("t2", 1), WATCH("t2", "t1"), EV("t2", 2), UNWATCH("t2", "t1"), WATCH("p", "t2"), STOPC("p", "t2"), EV("p", 3), NOW("t2")])
case("i14-new-instance-watches-itself-by-old-path", [*IDENT, C("p", "t", "t2"), EV("t2", 1), WATCH("t2", "t2"), WATCH("p", "t2"), STOPC("p", "t2"), EV("p", 3), NOW("t2")])


def main():
    out = os.path.join(ROOT, "cases", "scenario-3a.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, sc in CASES.items():
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(sc, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
