#!/usr/bin/env python3
"""Differential fuzzer for Classic local ActorSystem scenarios: real Pekko 2.0.0 vs the Rust port.

Random sequences of spawn / stop / await-stopped / terminate against one system, run through
`scenario-lines` on both sides (one JVM, one Rust process). Steps only reference actors the
generator knows were created, and only await actors that are stopping, so every trace is deterministic and fast.

  tools/fuzz_scenarios.py [--count N] [--seed S] [--show K]
"""
import argparse, json, os, random, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ORACLE = [os.path.join(ROOT, "oracle", "run.sh"), "scenario-lines"]
RUST = [os.environ.get("CLASSIC_PARITY_BIN", os.path.join(ROOT, "target", "release", "examples", "classic-parity")), "scenario-lines"]

GOOD = ["a", "b", "c", "d", "worker", "w-1", "w_2", "x.y", "A", "%41", "a%41", "z" * 40]
BAD = ["$a", "a b", "a/b", "", "a%", "a%zz", "é", "a#b", "$", "a\U0001F600"]
SYSTEMS = ["sys", "s", "Sys-1_x", "a" * 30, "9z"]


def scenario(rng):
    steps, live, used, n, terminated, stopped = [], [], set(), 0, False, set()
    name_of = {}  # label -> explicit name, to avoid reusing the name of an actor that was stopped
    settled = False  # the system has fully terminated and the trace has observed it
    for _ in range(rng.randint(0, 25)):
        r = rng.random()
        # Between `terminate` and `await-system-terminated` the guardian is shutting down asynchronously,
        # so whether a `spawn` wins that race is not defined: never emit one in that window.
        if r < 0.40 and terminated and not settled:
            steps.append({"op": rng.choice(["terminate", "await-system-terminated"])})
            settled = steps[-1]["op"] == "await-system-terminated" or settled
        elif r < 0.40:
            kind = rng.random()
            if kind < 0.35:
                st = {"op": "spawn"}
                name = None
            else:
                name = rng.choice(BAD if kind > 0.85 else GOOD)
                # A stopped actor's name is released when its parent processes the death notification,
                # slightly after `isTerminated` turns true: reusing it is a race, so never emit that.
                if name in {name_of[l] for l in stopped if l in name_of}:
                    name = "fresh%d" % n
                st = {"op": "spawn", "name": name}
            ok = not terminated and (name is None or ((name in GOOD or name.startswith("fresh")) and name not in used))
            if ok:
                label = f"r{n}"
                n += 1
                st["as"] = label
                live.append(label)
                if name is not None:
                    used.add(name)
                    name_of[label] = name
            steps.append(st)
        elif r < 0.60 and live:
            ref = rng.choice(live)
            stopped.add(ref)
            steps.append({"op": "stop", "ref": ref})
        elif r < 0.78 and (terminated and live or stopped):
            # only wait for actors that are really going down; otherwise the step burns its 10 s timeout
            steps.append({"op": "await-stopped", "ref": rng.choice(live if terminated else sorted(stopped))})
        elif r < 0.90 and not terminated:
            steps.append({"op": "terminate"})
            terminated = True
        elif terminated:
            steps.append({"op": rng.choice(["await-system-terminated", "terminate"])})
            settled = settled or steps[-1]["op"] == "await-system-terminated"
    if terminated and not any(s["op"] == "await-system-terminated" for s in steps):
        steps.append({"op": "await-system-terminated"})
    return {"system": rng.choice(SYSTEMS), "steps": steps}


def run(cmd, data):
    p = subprocess.run(cmd, input=data.encode("ascii"), capture_output=True)
    if p.returncode != 0:
        sys.exit(f"{cmd[0]} failed ({p.returncode}): {p.stderr.decode(errors='replace')[:800]}")
    return p.stdout.decode("ascii").split("\n")[:-1]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--count", type=int, default=1500)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--show", type=int, default=5)
    a = ap.parse_args()
    rng = random.Random(a.seed)
    cases = [scenario(rng) for _ in range(a.count)]
    data = "".join(json.dumps(c, ensure_ascii=True) + "\n" for c in cases)
    want, got = run(ORACLE, data), run(RUST, data)
    assert len(want) == len(got) == len(cases), (len(want), len(got), len(cases))
    bad = [(c, w, g) for c, w, g in zip(cases, want, got) if w != g]
    steps = sum(len(c["steps"]) for c in cases)
    print(f"seed={a.seed} scenarios={len(cases)} steps={steps} divergences={len(bad)}")
    for c, w, g in bad[: a.show]:
        print("SCENARIO:", json.dumps(c))
        print("ORACLE  :", w)
        print("RUST    :", g)
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
