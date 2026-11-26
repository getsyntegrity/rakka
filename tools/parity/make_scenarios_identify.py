#!/usr/bin/env python3
"""Writes cases/scenario-identify.jsonl: `Identify` / `ActorIdentity` (AC-REF-013), without actor selection.

A tell step takes `{"special": "Identify", "id": ...}`; a received `ActorIdentity` is printed with its correlation id and the path of its ref.
"""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_scenarios_3a import DL, EV, NOW, PILL, S, STOP, TELL  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def ask(target, correlation, sender="p"):
    return TELL(target, {"special": "Identify", "id": correlation}, sender)


SPAWN = [DL(), *S("p"), *S("k")]
STOPPED = [STOP("k"), {"op": "await-stopped", "ref": "k"}]

CASES = {
    # the cell answers before `receive`: k records only the message sent after the Identify
    "identify-live": [*SPAWN, ask("k", "first"), TELL("k", "after"), EV("k", 2), EV("p", 2), NOW("k"), NOW("p")],
    "identify-no-sender": [*SPAWN, ask("k", "lost", None), EV("dl", 1), NOW("dl"), NOW("k")],
    "identify-stopped": [*SPAWN, *STOPPED, ask("k", "gone"), EV("p", 2), TELL("k", "barrier"), EV("dl", 1), NOW("p"), NOW("dl")],
    "identify-stopped-no-sender": [*SPAWN, *STOPPED, ask("k", "nobody", None), EV("dl", 1), NOW("dl")],
    "identify-deadletters": [*SPAWN, ask("deadLetters", "dead"), EV("p", 2), NOW("p"), NOW("dl")],
    # queued behind a PoisonPill: the closing mailbox hands it to deadLetters, which answers it
    "identify-behind-pill": [DL(), *S("p"), *S("k", kind="gate"), TELL("k", "block"), EV("k", 2), PILL("k"), ask("k", "queued"),
                             {"op": "release"}, EV("p", 2), NOW("p"), NOW("dl")],
}


def main():
    out = os.path.join(ROOT, "cases", "scenario-identify.jsonl")
    with open(out, "w", encoding="ascii") as f:
        for name, steps in CASES.items():
            scenario = {"system": "sys", "steps": steps}
            f.write(json.dumps({"name": name, "args": ["scenario"], "stdin": json.dumps(scenario, ensure_ascii=True)}, ensure_ascii=True) + "\n")
    print(f"wrote {len(CASES)} scenarios to {os.path.relpath(out, ROOT)}")


if __name__ == "__main__":
    main()
