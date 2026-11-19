#!/usr/bin/env python3
"""Writes cases/scenario-3b7.jsonl: the text of ActorInitializationException (preStart failure) and PostRestartException (refinement 01, A-BUG-01/02).

The supervisor logs what its decider saw (`log-decisions`), so the exception text, which includes the child's path, the root cause message and (for
PostRestartException) the class of the failure that caused the restart, is compared with Pekko 2.0.0. Behaviour option `throw-in-post-restart` makes
`postRestart` record its event and throw IllegalStateException("post-restart") (same name in Scenario.java and scenario.rs).
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import S, C, EV, NOW, TELL, DO, GONE  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CASES = {}
ONE = {"parallelism": 1, "throughput": 5}


def case(name, steps, system="sys", dispatcher=None):
    assert name not in CASES, name
    assert len({n[:24] for n in CASES} | {name[:24]}) == len(CASES) + 1, f"case names must differ in the first 24 chars: {name}"
    CASES[name] = {"system": system, "steps": steps, **({"dispatcher": dispatcher} if dispatcher else {})}


LOG = {"log-decisions": True, "decider": {}}

for tag, disp in (("", None), ("-1t", ONE)):
    case(f"init{tag}-prestart-text", [*S("p", supervisor=LOG), C("p", "k", "k", throw_in_prestart=True), EV("p", 2), GONE("p", "k"), NOW("p")], dispatcher=disp)
    case(f"init{tag}-grandchild-text", [*S("g"), C("g", "p", "p", supervisor=LOG), EV("p", 1), C("p", "k", "k", throw_in_prestart=True), EV("p", 2), NOW("p")], dispatcher=disp)
    case(f"postr{tag}-text", [*S("p", supervisor=LOG), C("p", "k", "k", throw_on="boom", throw_in_post_restart=True), EV("k", 1), TELL("k", "boom"), EV("k", 5), EV("p", 3),
                              GONE("p", "k"), NOW("k"), NOW("p")], dispatcher=disp)
    case(f"postr{tag}-then-works", [*S("p", supervisor=LOG), C("p", "k", "k", throw_on="boom", throw_in_post_restart=True), EV("k", 1), TELL("k", "boom"), EV("p", 3),
                                    *S("b"), TELL("b", "alive"), EV("b", 2)], dispatcher=disp)


def main():
    out = os.path.join(ROOT, "cases", "scenario-3b7.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, sc in CASES.items():
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(sc, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
