#!/usr/bin/env python3
"""Builds the recorded cases that replace the deprecated jshell oracle (decision Q-X-12) for AC-PATH and AC-UTIL.

Each oracle case id (e.g. `FS-05 [pekko://sys/\\u00FCser]`) becomes one rust-anything case: the Java oracle and the
Rust harness both run `<family>-case` with the id on stdin. Only ids Rust implements are recorded; the others
(PENDING on an open question, or checked by a unit test) are listed in cases/oracle-ids/*.txt, which holds every id
the oracle has, so that a missing id is noticed.

  tools/parity/make_oracle_cases.py      (needs the built oracle and the release classic-parity binary)
"""
import json, os, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
RUST = os.path.join(ROOT, "target", "release", "examples", "classic-parity")
ORACLE = os.path.join(ROOT, "oracle", "run.sh")
FAMILIES = {"path-address": "pa", "util-core": "uc", "config-hocon": "ch", "config-settings": "cs", "dynamic-access": "da", "provider-selection": "ps", "actor-system-setup": "as", "deploy": "dp", "deployer": "dr", "deployer-settings": "ds", "mailbox": "mb", "scheduler": "sc"}


def ids(cmd):
    out = subprocess.run(cmd, stdin=subprocess.DEVNULL, capture_output=True, text=True, check=True).stdout
    return [l for l in out.split("\n") if l]


def main():
    os.makedirs(os.path.join(ROOT, "cases", "oracle-ids"), exist_ok=True)
    mapping = []
    for family, prefix in FAMILIES.items():
        java = ids([ORACLE, f"{family}-ids"])
        rust = set(ids([RUST, f"{family}-ids"]))
        assert len(set(java)) == len(java), f"duplicate oracle ids in {family}"
        assert rust <= set(java), f"Rust produces ids the oracle does not have: {sorted(rust - set(java))}"
        with open(os.path.join(ROOT, "cases", "oracle-ids", f"{family}.txt"), "w", encoding="ascii") as fh:
            fh.write("".join(i + "\n" for i in java))
        cases = []
        for n, case_id in enumerate((i for i in java if i in rust), 1):
            name = f"{prefix}-{n:04d}"           # unique even when rust-anything truncates names to 24 chars
            cases.append({"name": name, "args": [f"{family}-case"], "stdin": case_id})
            mapping.append((name, family, case_id))
        with open(os.path.join(ROOT, "cases", f"oracle-{family}.jsonl"), "w", encoding="ascii") as fh:
            fh.write("".join(json.dumps(c, ensure_ascii=True) + "\n" for c in cases))
        print(f"{family}: {len(cases)} cases recorded of {len(java)} oracle ids ({len(java) - len(cases)} not produced by Rust)")
    with open(os.path.join(ROOT, "cases", "oracle-case-ids.tsv"), "w", encoding="ascii") as fh:
        fh.write("case\tfamily\toracle_id\n" + "".join(f"{n}\t{f}\t{i}\n" for n, f, i in mapping))


if __name__ == "__main__":
    main()
