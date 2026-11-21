#!/usr/bin/env python3
"""Differential fuzzer for the choice among overlapping `prefix*` entries (Q-UTIL-21, Q-X-21): real Pekko 2.0.0 vs the Rust port.

Prefixes are never empty (the Java oracle splits on commas and drops a trailing empty one). Each input is a set of prefixes of one probe element, inserted in a random order (Pekko picks the first match in the
iteration order of a Scala HashMap, so the order of insertion must not matter except among keys whose scrambled hash is fully
equal; those are generated on purpose from the classic String.hashCode collisions "Aa"/"BB").

  tools/parity/fuzz_wildcard_order.py [--count N] [--seed S] [--show K]
"""
import argparse, os, random, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ORACLE = [os.path.join(ROOT, "oracle", "run.sh"), "wildcard-winners"]
RUST = [os.environ.get("CLASSIC_PARITY_BIN", os.path.join(ROOT, "target", "release", "examples", "classic-parity")), "wildcard-winners"]
ALPHABET = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEF_-.éΩ😀"


def case(rng):
    length = rng.randint(2, 40)
    probe = "".join(rng.choice(ALPHABET) for _ in range(length)) + "zz"
    if rng.random() < 0.2:  # a probe built of colliding blocks, so its prefixes share long runs of hash bits
        probe = "".join(rng.choice(["Aa", "BB"]) for _ in range(rng.randint(1, 6))) + "zz"
    cuts = sorted(rng.sample(range(1, len(probe)), rng.randint(2, min(24, len(probe) - 1))))
    prefixes = [probe[:cut] for cut in cuts]
    prefixes += [rng.choice(prefixes) for _ in range(rng.choice([0, 0, 1, 3]))]  # repeated prefixes
    rng.shuffle(prefixes)
    return ",".join(prefixes) + "|" + probe


def run(command, text):
    result = subprocess.run(command, input=text, capture_output=True, text=True, encoding="utf-8", check=True)
    return result.stdout.split("\n")[:-1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--count", type=int, default=20000)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--show", type=int, default=5)
    args = parser.parse_args()
    rng = random.Random(args.seed)
    cases = [case(rng) for _ in range(args.count)]
    text = "\n".join(cases) + "\n"
    expected, actual = run(ORACLE, text), run(RUST, text)
    diverging = [(c, e, a) for c, e, a in zip(cases, expected, actual) if e != a]
    print(f"wildcard-order fuzz seed={args.seed}: {args.count} sets, {len(diverging)} diverge")
    for c, e, a in diverging[: args.show]:
        print(f"  {c}\n    pekko={e!r} rust={a!r}")
    sys.exit(1 if diverging or len(expected) != len(actual) else 0)


main()
