#!/usr/bin/env python3.12
"""Genera la matriz de trazabilidad contrato -> fuente Pekko -> código Rust -> test Rust -> caso del oráculo
-> estado anterior -> estado nuevo (issue #281). Sólo lee fuentes canónicas; no edita nada.

Uso: reconcile_matrix.py --before <actor-classic.tsv anterior> > matriz.tsv
"""
import argparse
import csv
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TESTS = ROOT / "crates/rakka-actor/tests"
RUNTIME_UNITS = {"REF", "CELL", "SUP", "DW", "MBX", "DSP", "SYS", "EXT", "EVT"}
# Módulo Rust que aloja cada unidad (granularidad de archivo; no es un símbolo).
RUST_MODULE = {
    "REF": "actor/actor_ref.rs, actor/cell.rs",
    "CELL": "actor/cell.rs, actor/children.rs",
    "SUP": "actor/supervision.rs, actor/cell.rs",
    "DW": "actor/cell.rs",
    "MBX": "actor/cell.rs",
    "DSP": "dispatch/dispatcher.rs",
    "SYS": "actor/system.rs",
    "EXT": "-",
    "EVT": "event/mod.rs",
}


def read_rows(path):
    with open(path, newline="") as handle:
        return {r["id"]: r for r in csv.DictReader(handle, delimiter="\t", quoting=csv.QUOTE_NONE)}


def cited_cases(notes):
    found = []
    for chunk in re.findall(r"casos ([a-z][a-z0-9_*,\-]*)", notes):
        found += [c for c in chunk.split(",") if c and not c.endswith("*")]
    return list(dict.fromkeys(found))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--before", required=True)
    args = parser.parse_args()
    before = read_rows(args.before)
    after = read_rows(ROOT / "docs/inventory/actor-classic.tsv")
    test_text = {p.name: p.read_text() for p in TESTS.glob("*.rs")}
    writer = csv.writer(sys.stdout, delimiter="\t", lineterminator="\n")
    writer.writerow(["contrato", "fuente_pekko", "codigo_rust", "test_rust", "casos_oraculo",
                     "estado_anterior", "estado_nuevo", "evidencia"])
    for row_id, row in after.items():
        if row["unit"] not in RUNTIME_UNITS:
            continue
        cases = cited_cases(row["notes"])
        def mentions(case, text):
            return re.search(rf"(?<![\w-]){re.escape(case)}(?![\w-])", text) is not None

        tests = sorted({name for name, text in test_text.items() for c in cases if mentions(c, text)})
        if cases:  # todos los casos se repiten en proceso en el replay grabado
            tests.append("recorded_pekko.rs::recorded_scenario_cases_match_pekko (replay)")
        replay_only = [c for c in cases if not any(mentions(c, text) for text in test_text.values())]
        old = before[row_id]
        evidence = "; ".join(filter(None, [
            re.search(r"comparado con casos 2\.0\.0:([^|]*)", row["notes"]) and
            re.search(r"comparado con casos 2\.0\.0:([^|;]*)", row["notes"]).group(1).strip()[:90],
            f"{len(replay_only)} caso(s) sólo en replay" if replay_only else "",
        ]))
        has_code = row["impl"] != "pending"
        writer.writerow([
            row_id,
            row["evidence"].split(" ; ")[0].split(" (")[0],
            RUST_MODULE[row["unit"]] if has_code or cases else "-",
            ", ".join(tests) or "-",
            ",".join(cases) or "-",
            f"{old['impl']}/{old['validation']}",
            f"{row['impl']}/{row['validation']}",
            evidence or "-",
        ])


if __name__ == "__main__":
    main()
