#!/usr/bin/env python3
"""Differential fuzzer for the concise render of concatenations with substitutions (D2, D3, D4 of PR-C): real Pekko 2.0.0
(Deploy.toString over Typesafe Config 1.4.9) vs the Rust port. Each input is `a = <atoms separated by white space>` made of
numbers, booleans, null, strings, lists, objects and substitutions with path elements of every shape. Concatenations that mix
scalars with lists/objects are not generated: Pekko rejects them and Rust does not yet (D5, a parser task). Divergent inputs
are printed and kept in the file given by --keep (default: none).

  tools/parity/fuzz_config_render.py [--count N] [--seed S] [--show K] [--keep FILE]
"""
import argparse, os, random, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ORACLE = [os.path.join(ROOT, "oracle", "run.sh"), "config-render-lines"]
RUST = [os.environ.get("CLASSIC_PARITY_BIN", os.path.join(ROOT, "target", "release", "examples", "classic-parity")), "config-render-lines"]
SCALARS = ["1", "1.50", "1e2", "-3", "0.0", "007", "10", "true", "false", "null", '"q"', '"a b"', '""', "foo", "bar.baz", "x-y", "é", "1.5e-7", "100000"]
CONTAINERS = {"list": ["[1]", "[2, 3]", "[]", '["s"]', "[${p}]", "[[1], {a=1}]"], "object": ["{x=1}", "{y=2}", "{}", '{z="s"}', "{w=${p}}", "{x=2}", "{x={k=1}}", "{x={j=2}}"]}
ELEMENTS = ["x", "x.y", "1", "007", "1.2", "-1", "-", "1-a", "1e5", "true", "null", "a-b", "_c", '"q k"', '"1"', "é", "a.007", "0x10", "1.5", "false.x"]


def subst(rng):
    path = ".".join(rng.choice(ELEMENTS) for _ in range(rng.randint(1, 3)))
    return "${" + ("?" if rng.random() < 0.25 else "") + path + "}"


def case(rng):
    kind = rng.choice(["scalar", "scalar", "list", "object"])
    pool = SCALARS if kind == "scalar" else CONTAINERS[kind]
    atoms = [subst(rng) if rng.random() < 0.35 else rng.choice(pool) for _ in range(rng.randint(1, 5))]
    out = []
    for i, atom in enumerate(atoms):
        if i:
            glue_free_ok = not (atom[0].isalnum() or atoms[i - 1][-1].isalnum()) or atom.startswith("$") or atoms[i - 1].endswith('}') or atoms[i - 1].endswith('"')
            out.append(rng.choice([" ", "  ", "\t"] + ([""] if glue_free_ok else [])))
        out.append(atom)
    return "a = " + "".join(out)


def run(command, text):
    return subprocess.run(command, input=text, capture_output=True, text=True, encoding="utf-8", check=True).stdout.split("\n")[:-1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--count", type=int, default=5000)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--show", type=int, default=8)
    parser.add_argument("--keep")
    args = parser.parse_args()
    rng = random.Random(args.seed)
    cases = sorted({case(rng) for _ in range(args.count)})
    text = "\n".join(cases) + "\n"
    expected, actual = run(ORACLE, text), run(RUST, text)
    if len(expected) != len(cases) or len(actual) != len(cases):
        sys.exit(f"line count mismatch: {len(cases)} inputs, {len(expected)} oracle, {len(actual)} rust")
    same = lambda e, a: e == a or (e.startswith("ERR") and a.startswith("ERR"))
    diverging = [(c, e, a) for c, e, a in zip(cases, expected, actual) if not same(e, a)]
    print(f"config-render fuzz seed={args.seed}: {len(cases)} inputs, {len(diverging)} diverge")
    for c, e, a in diverging[: args.show]:
        print(f"  {c}\n    pekko={e}\n    rust ={a}")
    if args.keep and diverging:
        with open(args.keep, "w", encoding="utf-8") as fh:
            fh.write("".join(f"{c}\n" for c, _, _ in diverging))
    sys.exit(1 if diverging else 0)


main()
