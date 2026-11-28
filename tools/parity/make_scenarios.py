#!/usr/bin/env python3
"""Writes cases/scenario-2a.jsonl: Classic local ActorSystem scenarios for `rust-anything app record`.

Each case feeds one scenario JSON on stdin to the `scenario` command (oracle: real Pekko 2.0.0;
port: pekko-actor's classic-parity example). A scenario is {"system": name, "steps": [...]}; see oracle/.../Scenario.java.
The oracle decides the expected trace, never this file.
"""
import json, os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
anon = lambda as_=None: {"op": "spawn", **({"as": as_} if as_ else {})}
named = lambda n, as_=None: {"op": "spawn", "name": n, **({"as": as_} if as_ else {})}
stop = lambda r: {"op": "stop", "ref": r}
await_stopped = lambda r: {"op": "await-stopped", "ref": r}
TERM = {"op": "terminate"}
AWAIT_SYS = {"op": "await-system-terminated"}

CASES = {}


def case(name, system, steps):
    assert name not in CASES, name
    CASES[name] = {"system": system, "steps": steps}


# --- ActorSystem names ---------------------------------------------------------------------
for i, n in enumerate(["sys", "a", "A", "0abc", "a-b", "a_b", "Sys-1_x", "a-", "a_", "z9"]):
    case(f"sysname-ok-{i}", n, [])
for i, n in enumerate(["", "-a", "_a", "a b", "a.b", "é", "aé", "a$", "a/b", "a@b", "a:b", "a%41", "a\n", "١a", "Ａ", "a\U0001F600"]):
    case(f"sysname-bad-{i}", n, [])
# the literal names of ActorSystemSpec.scala:138-153, and the one of :163-165 with every valid character
for i, n in enumerate(["-hallowelt", "_hallowelt", "hallo welt", "hallo*welt", "hallo@welt", "hallo#welt", "hallo$welt",
                       "hallo%welt", "hallo/welt"]):
    case(f"sysname-spec-bad-{i}", n, [])
case("sysname-spec-all-valid", "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_", [])

# --- actor names: accepted ------------------------------------------------------------------
GOOD = ["a", "A1", "a-b", "a_b", "a.b", "a*b", "a+b", "a:b", "a@b", "a&b", "a=b", "a,b", "a!b", "a~b", "a'b", "a;b", "a$b",
        "%41", "a%41b", "a%4A", "a%aF", "-", "_", ".", "..", "*", "x" * 300, "a%41%42%43", "%41%"[:3]]
for i, n in enumerate(GOOD):
    case(f"name-ok-{i}", "sys", [named(n, "x")])

# --- actor names: rejected ------------------------------------------------------------------
BAD = ["$a", "$", "$$", "a b", " a", "a ", "a/b", "/a", "a#b", "a?b", "a[b]", "a{b}", "a\\b", 'a"b', "a<b>", "a|b", "a^b", "a`b",
       "a%", "a%4", "a%zz", "%", "%4", "%g1", "é", "abé", "a٠", "a\U0001F600", "\U0001F600", "a ", "a\tb", "a\nb",
       "a\u0000b", "ÿ", "aĀ", "a b", "a%éé", "%41%4"]
for i, n in enumerate(BAD):
    case(f"name-bad-{i}", "sys", [named(n)])
case("name-empty", "sys", [named("")])
# the literal names of LocalActorRefProviderSpec.scala:170-197 (`""` is name-empty; `null` cannot be expressed, Q-PATH-4)
for i, n in enumerate(["$hallo", "a%", "%3", "%xx", "%0G", "%gg", "%", "%1t", "a?", "üß", 'he"llo', "$hello", "hell>o"]):
    case(f"name-spec-{i}", "sys", [named(n)])

# --- uniqueness and anonymous names ----------------------------------------------------------
case("dup-same-name", "sys", [named("a", "a"), named("a")])
case("dup-three-times", "sys", [named("a"), named("a"), named("a")])
case("dup-case-sensitive", "sys", [named("a"), named("A")])
case("dup-escape-differs-from-char", "sys", [named("a"), named("%61")])
case("dup-after-failed-bad-name", "sys", [named("a b"), named("a b"), named("ok"), named("ok")])
case("anon-first-names", "sys", [anon() for _ in range(5)])
case("anon-64-names-wrap", "sys", [anon() for _ in range(70)])
case("anon-130-two-char-names", "sys", [anon() for _ in range(130)])
case("anon-mixed-with-named", "sys", [anon(), named("a"), anon(), named("b"), anon()])
case("anon-counter-consumed-by-failure", "sys", [anon(), named("a"), named("a"), anon()])
case("anon-vs-explicit-dollar", "sys", [anon(), named("$a")])

# --- paths and refs --------------------------------------------------------------------------
case("path-nested-system-name", "my-sys_2", [named("a", "a"), anon("b")])
case("path-long-name-and-escape", "sys", [named("a%41b"), named("x" * 100)])

# --- stop ----------------------------------------------------------------------------------
case("stop-then-await", "sys", [named("a", "a"), stop("a"), await_stopped("a")])
case("stop-twice", "sys", [named("a", "a"), stop("a"), stop("a"), await_stopped("a")])
case("stop-anonymous", "sys", [anon("a"), stop("a"), await_stopped("a")])
case("stop-one-of-many", "sys", [named("a", "a"), named("b", "b"), named("c", "c"), stop("b"), await_stopped("b"), named("d")])
case("stop-all", "sys", [named("a", "a"), named("b", "b"), anon("c"), stop("a"), stop("b"), stop("c"),
                          await_stopped("a"), await_stopped("b"), await_stopped("c")])
case("stop-many-concurrently", "sys", [anon(f"r{i}") for i in range(40)] + [stop(f"r{i}") for i in range(40)] + [await_stopped(f"r{i}") for i in range(40)])
case("stop-then-terminate", "sys", [named("a", "a"), stop("a"), TERM, AWAIT_SYS])

# --- terminate -------------------------------------------------------------------------------
case("terminate-empty", "sys", [TERM, AWAIT_SYS])
case("terminate-with-children", "sys", [named("a"), named("b"), anon(), TERM, AWAIT_SYS])
case("terminate-many-children", "sys", [anon() for _ in range(200)] + [TERM, AWAIT_SYS])
case("terminate-twice", "sys", [TERM, TERM, AWAIT_SYS])
case("terminate-await-twice", "sys", [TERM, AWAIT_SYS, AWAIT_SYS])
case("terminate-then-spawn", "sys", [TERM, AWAIT_SYS, named("late")])
case("terminate-then-spawn-anon", "sys", [TERM, AWAIT_SYS, anon()])
case("terminate-then-spawn-bad-name", "sys", [TERM, AWAIT_SYS, named("a b"), named("")])
case("terminate-then-stop", "sys", [named("a", "a"), TERM, AWAIT_SYS, stop("a"), await_stopped("a")])
case("terminate-then-terminate", "sys", [TERM, AWAIT_SYS, TERM, AWAIT_SYS])
# ActorSystemSpec.scala:300-326, concurrent: spawn without pause while a timer terminates the system; only invariants are printed
case("terminate-while-spawning", "sys", [{"op": "spawn-until-terminated", "after-ms": 100}])
case("child-terminated-after-terminate", "sys", [named("a", "a"), TERM, AWAIT_SYS, await_stopped("a")])


def main():
    assert len({n[:24] for n in CASES}) == len(CASES), "case names must differ in the first 24 chars (rust-anything truncates recording names)"
    out = os.path.join(ROOT, "cases", "scenario-2a.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, sc in CASES.items():
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(sc, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
