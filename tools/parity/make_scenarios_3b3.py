#!/usr/bin/env python3
"""Writes cases/scenario-3b3.jsonl: `Escalate` (slice 3b-3).

Three levels: `g` (grandparent, a top-level actor), `p` (its child, a supervisor whose decider escalates) and `c` (p's child, which
throws `IllegalStateException(<message>)` when it receives a message listed in `throw-on`). `supervisor.log-decisions` makes a supervisor
record `{"ev":"decide","cause":<class: message>,"directive":<name>}` every time its decider runs, so a scenario sees WHO decided and WHAT
cause the decider was given. Pekko 2.0.0 decides every expected trace.

Coordination: only event counts (`await-events`) and command round trips; no fixed sleeps. Messages sent to `c` before the failure are
processed before it (its own mailbox is FIFO); messages queued behind the failing one are either processed (Resume) or dead letters.
Part E (e*): escalation seen by the grandparent. Part F (f*): the children of the escalating parent. Part G (g*): the grandparent's retry limit.
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, WATCH, CHILD, CHILDREN, EV, NOW, TELL, DL, DO  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CASES = {}


def case(name, steps, system="sys"):
    assert name not in CASES, name
    assert len({n[:24] for n in CASES}) == len(CASES), f"case names must differ in the first 24 chars: {name}"
    CASES[name] = {"system": system, "steps": steps}


THROW = ["esc", "loc", "other"]
SYNC = lambda r: DO(r, "children")
KILL = {"special": "Kill"}
QUEUE = {"op": "tell-seq", "ref": "c", "prefix": "q", "count": 2}


def sup(decider, max=None):
    d = {"decider": decider, "log-decisions": True}
    if max is not None:
        d["max"] = max
    return d


def tree(g, p=None, gmax=None, pmax=None):
    """g (decides about p) -> p (escalates `esc`, decides `loc` itself) -> c (throws)"""
    pd = {"esc": "escalate", "Kill": "escalate", "loc": "resume"} if p is None else p
    return [*S("g", supervisor=sup(g, gmax)),
            C("g", "p", "p", supervisor=sup(pd, pmax)),
            C("p", "c", "c", numbered=True, throw_on=THROW)]


def kid(name):
    return C("p", name, name, numbered=True, throw_on=THROW)


G = lambda directive: {"esc": directive, "Kill": directive}
WP = WATCH("g", "p")

# --- Part E: the grandparent decides about the escalating parent -----------------------------------------------------------------
case("e01-g-resumes-p", [*tree(G("resume")), WP, CHILD("g", "p"), TELL("c", "m1"), TELL("c", "esc"), QUEUE,
                         EV("g", 2), EV("p", 2), EV("c", 5), SYNC("c"), TELL("p", "pm"), EV("p", 3),
                         CHILD("g", "p"), CHILD("p", "c"), NOW("g"), NOW("p"), NOW("c")])
case("e02-g-restarts-p", [DL(), *tree(G("restart")), WP, CHILD("g", "p"), TELL("c", "m1"), TELL("c", "esc"), QUEUE,
                          EV("g", 2), EV("p", 6), EV("c", 4), EV("dl", 2), TELL("p", "pm"), EV("p", 7),
                          CHILD("g", "p"), CHILD("p", "c"), CHILDREN("p"), NOW("g"), NOW("p"), NOW("c"), NOW("dl")])
case("e03-g-stops-p", [DL(), *tree(G("stop")), WP, CHILD("g", "p"), TELL("c", "m1"), TELL("c", "esc"), QUEUE,
                       EV("g", 3), EV("p", 3), EV("c", 4), EV("dl", 2), TELL("p", "late"), EV("dl", 3),
                       CHILD("g", "p"), CHILDREN("g"), NOW("g"), NOW("p"), NOW("c"), NOW("dl")])
# the cause the supervisor above sees is the one the child threw
case("e04-cause-is-kill-resume", [*tree(G("resume")), TELL("c", KILL), QUEUE, EV("g", 2), EV("p", 2), EV("c", 3), NOW("g"), NOW("p"), NOW("c")])
case("e05-cause-is-kill-restart", [DL(), *tree(G("restart")), TELL("c", KILL), EV("g", 2), EV("p", 6), EV("c", 2), NOW("g"), NOW("p"), NOW("c")])
# no mapping at the grandparent: the default decider decides (IllegalStateException -> Restart, ActorKilledException -> Stop)
case("e06-g-default-restarts", [*tree({}), TELL("c", "esc"), EV("g", 2), EV("p", 6), EV("c", 3), NOW("g"), NOW("p"), NOW("c")])
case("e07-g-default-stops-kill", [*tree({}), WP, TELL("c", KILL), EV("g", 3), EV("p", 3), EV("c", 2), NOW("g"), NOW("p"), NOW("c")])
# the parent resolves what it can and escalates the rest: the grandparent sees only the escalated failure
case("e08-local-and-escalated", [*tree(G("resume")), TELL("c", "m1"), TELL("c", "loc"), TELL("c", "m2"), TELL("c", "esc"), TELL("c", "m3"),
                                 EV("g", 2), EV("p", 3), EV("c", 6), NOW("g"), NOW("p"), NOW("c")])
case("e09-only-local-no-escalation", [*tree(G("restart")), TELL("c", "m1"), TELL("c", "loc"), TELL("c", "m2"), EV("p", 2), EV("c", 4), SYNC("p"), NOW("g"), NOW("p"), NOW("c")])
# the parent's own retry limit is irrelevant to an escalation (it is not a restart)
case("e10-escalate-ignores-p-max", [*tree(G("resume"), pmax=0), TELL("c", "esc"), TELL("c", "m1"), TELL("c", "esc"), EV("g", 3), EV("p", 3), EV("c", 4), NOW("g"), NOW("p"), NOW("c")])
case("e11-resume-twice", [*tree(G("resume")), TELL("c", "esc"), TELL("c", "m1"), TELL("c", "esc"), TELL("c", "m2"), EV("g", 3), EV("p", 3), EV("c", 5), NOW("g"), NOW("p"), NOW("c")])
# after the grandparent restarts p, the new instance still escalates (its strategy comes from the same Props)
case("e12-escalate-after-restart", [*tree(G("restart")), TELL("c", "esc"), EV("p", 6), kid("d"), TELL("d", "esc"), EV("g", 3), EV("p", 11), EV("d", 3), NOW("g"), NOW("p"), NOW("d")])

# --- Part F: the escalating parent's other children ---------------------------------------------------------------------------------
for tag, n_c2 in (("resume", 3), ("restart", 3), ("stop", 3)):
    p_n = {"resume": 2, "restart": 6, "stop": 3}[tag]
    g_n = 3 if tag == "stop" else 2
    c1_n = 4
    c2_steps = [TELL("c2", "b")] if tag == "resume" else []
    case(f"f-{tag}-healthy-sibling", [DL(), *tree(G(tag)), kid("c2"), WP, TELL("c2", "a"), EV("c2", 2), TELL("c", "m1"), TELL("c", "esc"), QUEUE, *c2_steps,
                                     EV("g", g_n), EV("p", p_n), EV("c", c1_n if tag != "resume" else 5), EV("c2", n_c2), SYNC("g"), *([] if tag == "stop" else [CHILDREN("p")]), CHILD("g", "p"),
                                     NOW("g"), NOW("p"), NOW("c"), NOW("c2")])

# --- Part G: the grandparent's retry limit counts escalated failures of p -----------------------------------------------------------------
case("g01-g-max-zero-stops-p", [*tree(G("restart"), gmax=0), WP, TELL("c", "esc"), EV("g", 3), EV("p", 3), EV("c", 2), NOW("g"), NOW("p"), NOW("c")])
case("g02-g-max-one-second-stops", [*tree(G("restart"), gmax=1), WP, TELL("c", "esc"), EV("p", 6), kid("d"), TELL("d", "esc"),
                                    EV("g", 4), EV("p", 8), EV("d", 3), NOW("g"), NOW("p"), NOW("d")])


def main():
    out = os.path.join(ROOT, "cases", "scenario-3b3.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, sc in CASES.items():
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(sc, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
