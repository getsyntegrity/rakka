"""Aristas fila->fila entre las unidades propuestas de P55 (revisión 3).

Uso (desde la raíz del repositorio):  python3 -I docs/proposals/evidence/p55-unit-edges.py

Lee `depends_on` de docs/inventory/actor-typed.tsv y calcula, para tres asignaciones de
filas a unidades, las aristas entre unidades y los ciclos. No es un control del
repositorio ni prueba equivalencia semántica: solo muestra qué aristas declaradas
(que pueden ser documentales o de validación) cierran ciclos.

  orig        asignación de la revisión 2 (§3 de p55-partition.md)
  rev3        asignación propuesta en §12.4 (SUP-010 -> S1; SUP-018 y STASH-006 -> S4; S6 -> S6a/S6b)
  rev3+deltas rev3 aplicando los deltas de arista de §12 (retirar SIG-006>SUP-014 y BEH-024>SUP-022,
              que se clasifican como validación/documental; añadir SUP-013>SUP-002; SUP-016>CTX-018 no se modela porque CTX-018 no es de P55)
"""
import csv
import re

csv.field_size_limit(10**9)

with open("docs/inventory/actor-typed.tsv", encoding="utf-8") as handle:
    rows = list(csv.reader(handle, delimiter="\t"))
header = rows[0]
by_id = {row[0]: dict(zip(header, row)) for row in rows[1:]}
ROW_ID = re.compile(r"AT-[A-Z]+-\d+(?:\.\d+)?")


def declared_dependencies(row_id):
    return ROW_ID.findall(by_id[row_id]["depends_on"])


ORIGINAL = {
    "S1": "AT-SUP-001 AT-SUP-002 AT-SUP-003 AT-SUP-004 AT-BEH-024 AT-SIG-005 AT-SIG-006",
    "S2": "AT-SIG-002 AT-SUP-005 AT-SUP-006 AT-SUP-008 AT-SUP-009 AT-SUP-014 AT-SUP-016 AT-SUP-018 AT-STASH-006",
    "S3": "AT-SUP-007.1 AT-SUP-007.2 AT-SUP-007.3 AT-SUP-017",
    "S4": "AT-SUP-010 AT-SUP-011 AT-SUP-012 AT-SUP-019",
    "S5": "AT-BEH-010 AT-SUP-013",
    "S6": "AT-ADP-002 AT-ADP-005 AT-SUP-020 AT-SUP-021 AT-SUP-022",
}
original = {unit: ids.split() for unit, ids in ORIGINAL.items()}

revised = {unit: list(ids) for unit, ids in original.items()}
revised["S4"].remove("AT-SUP-010")
revised["S1"].append("AT-SUP-010")
for moved in ("AT-SUP-018", "AT-STASH-006"):
    revised["S2"].remove(moved)
    revised["S4"].append(moved)
revised["S6a"] = ["AT-ADP-002", "AT-SUP-020", "AT-SUP-022"]
revised["S6b"] = ["AT-ADP-005", "AT-SUP-021"]
del revised["S6"]

DROPPED = {("AT-SIG-006", "AT-SUP-014"), ("AT-BEH-024", "AT-SUP-022")}
ADDED = {("AT-SUP-013", "AT-SUP-002")}


def unit_edges(assignment, dropped=frozenset(), added=frozenset()):
    unit_of = {row: unit for unit, ids in assignment.items() for row in ids}
    edges = {}
    for row in unit_of:
        targets = [t for t in declared_dependencies(row) if (row, t) not in dropped]
        targets += [t for (r, t) in added if r == row]
        for target in targets:
            if target in unit_of and unit_of[target] != unit_of[row]:
                edges.setdefault((unit_of[row], unit_of[target]), []).append(f"{row}>{target}")
    return edges


def cycles(assignment, edges):
    adjacency = {unit: [b for (a, b) in edges if a == unit] for unit in assignment}
    state, found = {}, []

    def visit(node, path):
        state[node] = 1
        for nxt in adjacency[node]:
            if state.get(nxt) == 1:
                found.append(path + [node, nxt])
            elif nxt not in state:
                visit(nxt, path + [node])
        state[node] = 2

    for unit in assignment:
        if unit not in state:
            visit(unit, [])
    return found


for label, assignment, dropped, added in (
    ("orig", original, frozenset(), frozenset()),
    ("rev3", revised, frozenset(), frozenset()),
    ("rev3+deltas", revised, DROPPED, ADDED),
):
    all_rows = sorted(r for ids in assignment.values() for r in ids)
    assert len(all_rows) == len(set(all_rows)) == 31, label
    print(f"== {label}: sizes={ {u: len(i) for u, i in assignment.items()} } total={len(all_rows)}")
    edges = unit_edges(assignment, dropped, added)
    for (source, target), evidence in sorted(edges.items()):
        print(f"{source} -> {target}  {evidence}")
    print("cycles:", cycles(assignment, edges) or "none")
