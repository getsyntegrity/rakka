#!/usr/bin/env python3
"""Differential fuzzer for Classic hierarchy + DeathWatch (slice 3a): real Pekko 2.0.0 vs the Rust port.

Random trees of actors, children created by their parents, stops of whole subtrees, watch / unwatch of live and dead
actors, lookups by name and name reuse. The generator simulates the expected effects and, after each step, waits for
exactly what that step must produce (`await-life` counts, `await-child-gone` for the parent's release of a name), so
every step is serialised by an explicit confirmation and the whole trace must match byte for byte. Orders that Pekko does
not guarantee (siblings dying in parallel, which watcher is notified first) are never compared: `await-life` reports the
sorted set of lifecycle events plus only the orderings Pekko guarantees (child post-stop before parent post-stop, post-stop
before the watcher's Terminated).

  tools/fuzz_hier.py [--count N] [--seed S] [--show K]
"""
import argparse, json, os, random, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ORACLE = [os.path.join(ROOT, "oracle", "run.sh"), "scenario-lines"]
RUST = [os.environ.get("CLASSIC_PARITY_BIN", os.path.join(ROOT, "target", "release", "examples", "classic-parity")), "scenario-lines"]
NAMES = ["a", "b", "c"]
BAD_NAMES = ["$x", "a b", "", "a/b"]


def scenario(rng):
    steps, nodes, watching = [], {}, {}
    life = [0]
    seq = [0]
    W = ["w0", "w1", "w2"]

    def top(label):
        steps.append({"op": "spawn", "name": label, "as": label, "behavior": {"kind": "log"}})
        steps.append({"op": "await-events", "target": label, "count": 1})
        nodes[label] = {"parent": None, "name": label, "alive": True, "kids": {}}
        life[0] += 1

    for w in W:
        top(w)
        watching[w] = set()
    roots = ["r0", "r1"]
    for r in roots:
        top(r)
    targets = list(roots)  # every node label that can be watched / stopped (not the watchers)

    def alive():
        return [n for n in targets if nodes[n]["alive"]]

    def do(ref, **cmd):
        steps.append({"op": "do", "ref": ref, "cmd": cmd})

    for _ in range(rng.randint(8, 22)):
        r = rng.random()
        live = alive()
        if r < 0.38 and live:
            p = rng.choice(live)
            if rng.random() < 0.07:
                do(p, c="spawn-child", name=rng.choice(BAD_NAMES), **{"as": "bad"})
            elif rng.random() < 0.15:
                seq[0] += 1
                label = f"n{seq[0]}"
                do(p, c="spawn-child", **{"as": label}, behavior={"kind": "log"})
                nodes[label] = {"parent": p, "name": None, "alive": True, "kids": {}}
                targets.append(label)
                life[0] += 1
            else:
                name = rng.choice(NAMES)
                seq[0] += 1
                label = f"n{seq[0]}"
                do(p, c="spawn-child", name=name, **{"as": label}, behavior={"kind": "log"})
                if name not in nodes[p]["kids"]:  # otherwise InvalidActorNameException and no new actor
                    nodes[label] = {"parent": p, "name": name, "alive": True, "kids": {}}
                    nodes[p]["kids"][name] = label
                    targets.append(label)
                    life[0] += 1
        elif r < 0.55:
            cands = [n for n in live if nodes[n]["parent"] is None or nodes[n]["name"] is not None]
            if not cands:
                continue
            x = rng.choice(cands)
            sub, stack = [], [x]
            while stack:  # live subtree
                n = stack.pop()
                sub.append(n)
                stack += [k for k in targets if nodes[k]["parent"] == n and nodes[k]["alive"]]
            before = []
            for n in sub:
                p = nodes[n]["parent"]
                if p is not None and p in sub:
                    before.append([f"{n}:post-stop", f"{p}:post-stop"])
            life[0] += len(sub)
            for n in sub:
                for w in W:
                    if n in watching[w]:
                        watching[w].discard(n)
                        life[0] += 1
                        before.append([f"{n}:post-stop", f"{w}:terminated:{n}"])
                nodes[n]["alive"] = False
            par = nodes[x]["parent"]
            if par is None:
                steps.append({"op": "stop", "ref": x})
            else:
                do(par, c="stop", target=x)
            steps.append({"op": "await-life", "count": life[0], "before": before})
            if par is not None:
                steps.append({"op": "await-child-gone", "ref": par, "name": nodes[x]["name"]})
                nodes[par]["kids"].pop(nodes[x]["name"], None)
        elif r < 0.77 and targets:
            w, t = rng.choice(W), rng.choice(targets)
            do(w, c="watch", target=t)
            if nodes[t]["alive"]:
                watching[w].add(t)
            else:  # an already terminated actor: an immediate Terminated (existenceConfirmed = false)
                life[0] += 1
                steps.append({"op": "await-life", "count": life[0], "before": []})
        elif r < 0.87 and targets:
            w, t = rng.choice(W), rng.choice(targets)
            do(w, c="unwatch", target=t)
            watching[w].discard(t)
        elif live:
            p = rng.choice(live)
            if rng.random() < 0.5:
                do(p, c="child", name=rng.choice(NAMES))
            else:
                do(p, c="children")
    for w in W:
        steps.append({"op": "events-sorted", "target": w})
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
    timeouts = 0
    for w in want:
        for st in json.loads(w)["steps"]:
            if st.get("ok") is False and "exception" not in st:
                timeouts += 1
    bad = [(c, w, g) for c, w, g in zip(cases, want, got) if w != g]
    steps = sum(len(c["steps"]) for c in cases)
    print(f"seed={a.seed} scenarios={len(cases)} steps={steps} oracle_await_timeouts={timeouts} divergences={len(bad)}")
    for c, w, g in bad[: a.show]:
        ws, gs = json.loads(w)["steps"], json.loads(g)["steps"]
        i = next((k for k, (x, y) in enumerate(zip(ws, gs)) if x != y), None)
        print("SCENARIO:", json.dumps(c))
        if i is None:
            print("ORACLE  :", w[:1500]); print("RUST    :", g[:1500])
        else:
            print(f"first differing step #{i}: {json.dumps(c['steps'][i])}")
            print("ORACLE  :", json.dumps(ws[i])[:1500])
            print("RUST    :", json.dumps(gs[i])[:1500])
    sys.exit(1 if bad or timeouts else 0)


if __name__ == "__main__":
    main()
