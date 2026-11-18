#!/usr/bin/env python3
"""Differential fuzzer for Classic tell/sender/forward/echo/PoisonPill/dead letters: real Pekko 2.0.0 vs the Rust port.

The generator simulates the effect of every step (which actor receives, who replies, who dies, which
messages become dead letters) and, after each step, waits for exactly the events that step must produce.
Every step is therefore serialised by an explicit confirmation, so the full trace is deterministic and the
two implementations must agree on it byte for byte (no interleaving is ever compared).

  tools/fuzz_tell.py [--count N] [--seed S] [--show K]
"""
import argparse, json, os, random, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ORACLE = [os.path.join(ROOT, "oracle", "run.sh"), "scenario-lines"]
RUST = [os.environ.get("CLASSIC_PARITY_BIN", os.path.join(ROOT, "target", "release", "examples", "classic-parity")), "scenario-lines"]
PILL = {"special": "PoisonPill"}


def scenario(rng):
    steps, kinds, counts, dead = [], {}, {}, set()
    dl = [0]

    def spawn(label, kind, **cfg):
        steps.append({"op": "spawn", "name": label, "as": label, "behavior": {"kind": kind, **cfg}})
        steps.append({"op": "await-events", "target": label, "count": 1})
        kinds[label] = kind
        counts[label] = 1

    steps.append({"op": "subscribe", "channel": "dead-letters", "as": "dl"})
    logs = []
    for i in range(rng.randint(1, 3)):
        spawn(f"l{i}", "log"); logs.append(f"l{i}")
    targets_for_forward = list(logs)
    for i in range(rng.randint(0, 3)):
        kind = rng.choice(["echo", "forward", "stop-on", "log"])
        label = f"x{i}"
        if kind == "forward":
            spawn(label, "forward", to=rng.choice(targets_for_forward)); targets_for_forward.append(label)
        elif kind == "stop-on":
            spawn(label, "stop-on", msg="bye")
        else:
            spawn(label, kind)
            if kind == "log":
                logs.append(label); targets_for_forward.append(label)
    labels = list(kinds)
    cfg_to = {s["as"]: s["behavior"].get("to") for s in steps if s["op"] == "spawn"}

    def deliver(t, msg, sender):
        """Simulates one delivery; sender is a label or None."""
        if t == "deadLetters":
            if msg != "PILL":  # PoisonPill is DeadLetterSuppression
                dl[0] += 1
            return
        if t in dead:
            if msg != "PILL":
                dl[0] += 1
            return
        if msg == "PILL":
            dead.add(t); counts[t] += 1  # post-stop
            return
        counts[t] += 1
        k = kinds[t]
        if k == "echo":
            deliver(sender, msg, t) if sender else deliver("deadLetters", msg, t)
        elif k == "forward":
            deliver(cfg_to[t], msg, sender)
        elif k == "stop-on" and msg == "bye":
            dead.add(t); counts[t] += 1  # post-stop

    def barrier(before):
        for l in labels:
            if counts[l] != before.get(l):
                steps.append({"op": "await-events", "target": l, "count": counts[l]})
        if dl[0] != before.get("dl"):
            steps.append({"op": "await-events", "target": "dl", "count": dl[0]})

    for i in range(rng.randint(3, 25)):
        before = dict(counts); before["dl"] = dl[0]
        target = rng.choice(labels + ["deadLetters"] if rng.random() < 0.1 else labels)
        sender = rng.choice([None, None] + logs)
        r = rng.random()
        if r < 0.12:
            msg, wire = "PILL", PILL
        elif r < 0.30:
            msg, wire = "bye", "bye"
        else:
            msg, wire = f"m{i}", f"m{i}"
        if rng.random() < 0.12 and msg.startswith("m") and kinds.get(target) in ("log", "forward") and target not in dead:
            n = rng.randint(2, 12)
            st = {"op": "tell-seq", "ref": target, "prefix": f"p{i}", "count": n}
            if sender: st["from"] = sender
            steps.append(st)
            for j in range(n):
                deliver(target, f"p{i}-{j}", sender)
        else:
            st = {"op": "tell", "ref": target, "msg": wire}
            if sender: st["from"] = sender
            steps.append(st)
            deliver(target, msg, sender)
        barrier(before)
    return {"system": "fz", "steps": steps}


def run(cmd, data):
    p = subprocess.run(cmd, input=data.encode("ascii"), capture_output=True)
    if p.returncode != 0:
        sys.exit(f"{cmd[0]} failed ({p.returncode}): {p.stderr.decode(errors='replace')[:800]}")
    return p.stdout.decode("ascii").split("\n")[:-1]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--count", type=int, default=800)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--show", type=int, default=3)
    a = ap.parse_args()
    rng = random.Random(a.seed)
    cases = [scenario(rng) for _ in range(a.count)]
    data = "".join(json.dumps(c, ensure_ascii=True) + "\n" for c in cases)
    os.environ.setdefault("SCENARIO_AWAIT_MS", "1500")  # a wrong expectation fails fast instead of waiting 10 s
    want, got = run(ORACLE, data), run(RUST, data)
    assert len(want) == len(got) == len(cases), (len(want), len(got), len(cases))
    timeouts = sum(1 for w in want if '"ok":false,"events"' in w)
    bad = [(c, w, g) for c, w, g in zip(cases, want, got) if w != g]
    steps = sum(len(c["steps"]) for c in cases)
    print(f"seed={a.seed} scenarios={len(cases)} steps={steps} oracle_await_timeouts={timeouts} divergences={len(bad)}")
    for c, w, g in bad[: a.show]:
        print("SCENARIO:", json.dumps(c))
        print("ORACLE  :", w[:1500])
        print("RUST    :", g[:1500])
    sys.exit(1 if bad or timeouts else 0)


if __name__ == "__main__":
    main()
