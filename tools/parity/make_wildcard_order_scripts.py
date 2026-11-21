#!/usr/bin/env python3
"""Appends DL-60.. to crates/rakka-actor/examples/classic-parity/deployer.tsv: overlapping `prefix*` deployments (Q-UTIL-21, Q-X-21).

Pekko picks, among the `prefix*` entries that match an element, the first one in the iteration order of a Scala
`immutable.HashMap`; the scripts insert the same prefixes in a seeded shuffled order and in the reverse order, so a twin that
follows the insertion order cannot match both. Run once; the output is committed. Usage: make_wildcard_order_scripts.py [--check]
"""
import pathlib, random, sys

TSV = pathlib.Path(__file__).resolve().parents[2] / "crates/rakka-actor/examples/classic-parity/deployer.tsv"
FIRST_ID = 60
WORDS = ["pqzzz", "abczzz", "userzzz", "workerzzz", "stzzz", "mnxyz", "service", "dispatcherx", "akkapekko", "routees", "zeta", "orders"]


def prefix_sets():
    rng = random.Random(21)
    for word in WORDS:
        for size in (2, 3, 5):
            cuts = sorted(rng.sample(range(1, len(word) + 1), min(size, len(word))))
            yield word, [word[:cut] for cut in cuts]


def scripts():
    for word, prefixes in prefix_sets():
        shuffled = prefixes[:]
        random.Random(len(word) * 31 + len(prefixes)).shuffle(shuffled)
        for label, order in (("in a shuffled order", shuffled), ("in the reverse order", shuffled[::-1])):
            deploys = "; ".join(f"deploy /a/{p}*" for p in order)
            yield f"{len(prefixes)} overlapping prefixes of {word} {label}", f"deploy /a/q; {deploys}; find a/{word}; find a/{word}x"


# Two prefixes of one element with the same String.hashCode (263630388), so they share a collision node of the HashMap, whose
# order is the insertion order (found by tools/parity/find_prefix_hash_collision.py).
COLLIDING_SHORT = "bvmxiku"
COLLIDING_LONG = COLLIDING_SHORT + "drlnvgcneunetqmtswnkyfbflscxdksgeqwbcxgzthmnhmuntxiajllwoxnwfvdg"


def collision_scripts():
    probe = COLLIDING_LONG + "zz"
    for label, order in (("short first", (COLLIDING_SHORT, COLLIDING_LONG)), ("long first", (COLLIDING_LONG, COLLIDING_SHORT))):
        deploys = "; ".join(f"deploy /a/{p}*" for p in order)
        yield f"two prefixes with the same hash code, {label}", f"deploy /a/q; {deploys}; find a/{probe}"
    for label, order in (("short first", (COLLIDING_SHORT, COLLIDING_LONG)), ("long first", (COLLIDING_LONG, COLLIDING_SHORT))):
        deploys = "; ".join(f"deploy /a/{p}*" for p in (COLLIDING_SHORT[:3], order[0], "bv", order[1]))
        yield f"two prefixes with the same hash code among others, {label}", f"deploy /a/q; {deploys}; find a/{probe}"


def main():
    lines = [f"DL-{FIRST_ID + i} {d}\t{s}" for i, (d, s) in enumerate([*scripts(), *collision_scripts()])]
    text = TSV.read_text(encoding="utf-8")
    if f"\nDL-{FIRST_ID} " in text:
        sys.exit("already appended")
    if "--check" in sys.argv:
        print("\n".join(lines))
        return
    TSV.write_text(text.rstrip("\n") + "\n" + "\n".join(lines) + "\n", encoding="utf-8")
    print(f"appended {len(lines)} scripts DL-{FIRST_ID}..DL-{FIRST_ID + len(lines) - 1}")


main()
