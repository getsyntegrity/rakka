#!/usr/bin/env python3
"""Writes cases/scenario-3b6.jsonl: an exception escaping the actor factory or the supervisor's decider (refinement 01, A-BUG-03).

Behaviour option `constructor-fails-from: n` makes the actor factory (Props creator) throw from the n-th instance on (1 = always) IllegalStateException("ctor"); the supervisor option
`decider-throws` makes the decider throw IllegalStateException("decider") instead of choosing a directive. Same option names in Scenario.java and
scenario.rs. Each case runs on the default dispatcher and on a single-thread dispatcher (`parallelism: 1`), where a thread killed by the
exception would stop the whole system. Pekko 2.0.0 decides every expected trace.
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, WATCH, CHILDREN, EV, NOW, TELL, DO, GONE, beh  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CASES = {}
ONE = {"parallelism": 1, "throughput": 5}


def case(name, steps, system="sys", dispatcher=None):
    assert name not in CASES, name
    assert len({n[:24] for n in CASES}) == len(CASES), f"case names must differ in the first 24 chars: {name}"
    CASES[name] = {"system": system, "steps": steps, **({"dispatcher": dispatcher} if dispatcher else {})}


SYNC = lambda r: DO(r, "children")
LOG = {"log-decisions": True, "decider": {}}

for tag, disp in (("", None), ("-1t", ONE)):
    # --- the factory throws -------------------------------------------------------------------------------------------
    case(f"ctor{tag}-child-stops", [*S("p", supervisor=LOG), C("p", "k", "k", constructor_fails_from=1), EV("p", 2), NOW("p"), GONE("p", "k"), CHILDREN("p"), NOW("k")], dispatcher=disp)
    case(f"ctor{tag}-then-works", [*S("p", supervisor=LOG), C("p", "k", "k", constructor_fails_from=1), EV("p", 2),
                                           C("p", "ok", "ok"), EV("ok", 1), TELL("p", "alive"), EV("p", 3), GONE("p", "k"), CHILDREN("p")], dispatcher=disp)
    case(f"ctor{tag}-top-level", [{"op": "spawn", "name": "a", "as": "a", "behavior": beh("log", constructor_fails_from=1)}, *S("b"), TELL("b", "alive"), EV("b", 2), NOW("a")], dispatcher=disp)
    case(f"ctor{tag}-restart-fails", [*S("p", supervisor=LOG), C("p", "k", "k", throw_on="boom", constructor_fails_from=2), EV("k", 1), TELL("k", "boom"), EV("k", 4), EV("p", 2),
                                      GONE("p", "k"), CHILDREN("p"), NOW("p"), NOW("k")], dispatcher=disp)
    case(f"ctor{tag}-restart-fails-ok", [*S("p", supervisor=LOG), C("p", "k", "k", throw_on="boom", constructor_fails_from=2), EV("k", 1), TELL("k", "boom"), EV("p", 2),
                                            *S("b"), TELL("b", "alive"), EV("b", 2)], dispatcher=disp)
    # --- the decider throws -------------------------------------------------------------------------------------------
    gsup = {"log-decisions": True, "decider": {"decider": "stop"}}
    case(f"dec{tag}-sup-fails", [*S("g", supervisor=gsup), C("g", "p", "p", supervisor={"decider-throws": True}), EV("p", 1),
                                        C("p", "k", "k", throw_in_prestart=True), EV("g", 2), EV("p", 2), EV("k", 2), CHILDREN("g"), NOW("g"), NOW("p"), NOW("k")], dispatcher=disp)
    case(f"dec{tag}-then-works", [*S("g", supervisor=gsup), C("g", "p", "p", supervisor={"decider-throws": True}), EV("p", 1),
                                          C("p", "k", "k", throw_in_prestart=True), EV("g", 2), *S("b"), TELL("b", "alive"), EV("b", 2)], dispatcher=disp)
    case(f"dec{tag}-restart-dir", [*S("g", supervisor={"log-decisions": True, "decider": {"decider": "restart"}}),
                                          C("g", "p", "p", supervisor={"decider-throws": True}), EV("p", 1),
                                          C("p", "k", "k", throw_in_prestart=True), EV("g", 2), EV("p", 4), SYNC("p"), NOW("p"), NOW("g")], dispatcher=disp)


def main():
    out = os.path.join(ROOT, "cases", "scenario-3b6.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, sc in CASES.items():
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(sc, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
