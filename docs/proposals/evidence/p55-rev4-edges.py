"""Aristas fila->fila de la revisión 4 de la propuesta de partición de P55 (extiende p55-unit-edges.py).

Uso (desde la raíz del repositorio):  python3 -I docs/proposals/evidence/p55-rev4-edges.py

No es un control del repositorio ni prueba equivalencia semántica. Lee `depends_on` de
docs/inventory/actor-typed.tsv y la asignación fila->unidad de docs/unit-rows.tsv, y calcula
aristas entre unidades en TRES grafos:

  declarado   SOLO las aristas `depends_on` del inventario entre filas del conjunto revisado (sin clasificar ni añadir)
  inicio      aristas de IMPLEMENTACIÓN: sin las que la revisión 4 clasifica como validación o
              documentales (CLASIFICACION, abajo), más las añadidas (AGREGADAS)
  cierre      inicio + aristas de VALIDACIÓN (hace falta para cerrar, no para comenzar)

Conjunto revisado (alcance de lo VERIFICADO): las 31 filas de P55 (asignación rev. 4: S1..S6b) y
todas las filas AT-* asignadas a P48, P49, P52, P53, P54 y P61 en docs/unit-rows.tsv. Se ignoran
las aristas hacia filas AC-* (clásicas), hacia grupos sin ID de fila y hacia filas AT-* de otras
unidades (P50, P51, P56, P57, P62-P64...): NO están revisadas.

La clasificación (CLASIFICACION/AGREGADAS) es juicio manual con la evidencia citada en
p55-partition.md §13; el script solo muestra qué ciclos quedan con o sin esa clasificación.
"""
import csv
import re

csv.field_size_limit(10**9)

ROW_ID = re.compile(r"AT-[A-Z]+-\d+(?:\.\d+)?")

with open("docs/inventory/actor-typed.tsv", encoding="utf-8") as handle:
    rows = list(csv.reader(handle, delimiter="\t"))
header = rows[0]
by_id = {row[0]: dict(zip(header, row)) for row in rows[1:]}

unit_of_row = {}
with open("docs/unit-rows.tsv", encoding="utf-8") as handle:
    for line in handle.read().splitlines()[1:]:
        row_id, unit = line.split("\t")
        unit_of_row[row_id] = unit

# Asignación propuesta de las 31 filas de P55 (rev. 4). STASH-006 pasa de S4 a S1; S4 pasa a ser
# una unidad de validación (S4v). El resto de la organización es la de la rev. 3 (§12.4).
P55_REV4 = {
    "S1": "AT-SUP-001 AT-SUP-002 AT-SUP-003 AT-SUP-004 AT-SUP-010 AT-STASH-006 AT-BEH-024 AT-SIG-005 AT-SIG-006",
    "S2": "AT-SIG-002 AT-SUP-005 AT-SUP-006 AT-SUP-008 AT-SUP-009 AT-SUP-014 AT-SUP-016",
    "S3": "AT-SUP-007.1 AT-SUP-007.2 AT-SUP-007.3 AT-SUP-017",
    "S4v": "AT-SUP-011 AT-SUP-012 AT-SUP-018 AT-SUP-019",
    "S5": "AT-BEH-010 AT-SUP-013",
    "S6a": "AT-ADP-002 AT-SUP-020 AT-SUP-022",
    "S6b": "AT-ADP-005 AT-SUP-021",
}
P55_REV4 = {unit: ids.split() for unit, ids in P55_REV4.items()}
p55_rows = sorted(r for ids in P55_REV4.values() for r in ids)
assert len(p55_rows) == len(set(p55_rows)) == 31, "las 31 filas de P55, sin pérdidas ni duplicados"
assert all(unit_of_row[r] == "P55" for r in p55_rows), "todas las filas son de P55 en unit-rows.tsv"
assert sorted(r for r, u in unit_of_row.items() if u == "P55") == p55_rows

EXTERNAL_UNITS = ("P48", "P49", "P52", "P53", "P54", "P61")
assignment = {}
for unit, ids in P55_REV4.items():
    for row_id in ids:
        assignment[row_id] = unit
for row_id, unit in unit_of_row.items():
    if unit in EXTERNAL_UNITS:
        assignment[row_id] = unit


def declared(row_id):
    return ROW_ID.findall(by_id[row_id]["depends_on"])


# (origen, destino) -> (clase, evidencia breve). Clases: "validacion" | "documental".
# Solo las aristas que la revisión 4 analizó. Las demás aristas declaradas se tratan como
# implementación (no clasificadas = no revisadas una por una).
CLASIFICACION = {
    # P61 <-> S6b / S6a (§13.1)
    ("AT-CTX-017", "AT-SUP-021"): ("validacion", "A1 compartida con ADP-005 A1; A2 sobrecarga con nombre; sin código propio"),
    ("AT-CTX-017", "AT-JAPI-005.1"): ("documental", "asScala/asJava = identidad; sin lógica (ActorContextImpl.scala:133-135)"),
    ("AT-ADP-005", "AT-SUP-021"): ("validacion", "el código del decisor es ActorAdapter.supervisorStrategy = ADP-006.4 (P49)"),
    ("AT-SUP-021", "AT-ADP-005"): ("documental", "SUP-021 no usa la fachada actorOf como código; A1 de SUP-021 es el spec de ADP-006.4 A4"),
    # DeathPactException (§13.2)
    ("AT-SIG-006", "AT-SUP-014"): ("validacion", "A3 de SIG-006 = A1 de SUP-014 (SupervisionSpec:1408-1439)"),
    ("AT-BEH-007", "AT-SUP-001"): ("validacion", "A2/A6 observan Stop por defecto; el código es ADP-006.4/AC-CELL (P49/clásico)"),
    ("AT-BEH-003", "AT-SIG-006"): ("documental", "BEH-003 A1-A3 no producen Terminated; el lanzamiento es BEH-011 (interpretSignal)"),
    # Aristas ya clasificadas en la rev. 3 (§12.4)
    ("AT-BEH-024", "AT-SUP-022"): ("documental", "rev. 3 §12.3: el flag es de la rama del adaptador"),
    ("AT-ADP-006.2", "AT-BEH-024"): ("documental", "rev. 3 §12.3: sustituida por BEH-001/011/012 (P48)"),
    ("AT-CTX-018", "AT-SUP-016"): ("documental", "rev. 3 §12.2: invertida (SUP-016 consume cancelAllTimers)"),
    ("AT-ADP-006.3", "AT-SUP-005"): ("documental", "rev. 3 §12.1: referencia / solape de validación"),
    ("AT-ADP-006.4", "AT-SUP-001"): ("documental", "rev. 3 §4.3: SUP-001 describe el default que 006.4 implementa"),
    # S4 (§13.3)
    ("AT-STASH-006", "AT-SUP-018"): ("validacion", "A1/A2 de STASH-006 son las de SUP-018 (StashSpec:460-610)"),
    ("AT-SUP-020", "AT-SUP-005"): ("validacion", "A2 usa supervise(restart); la fachada solo envuelve en stop (scaladsl/adapter/package.scala:60)"),
}
# Aristas que la revisión 4 añade, con su clase (la mayoría son de implementación hacia el proveedor).
AGREGADAS = {
    ("AT-SUP-013", "AT-SUP-002"): ("implementacion", "rev. 3: el rechazo vive en Supervisor.apply"),
    ("AT-SUP-016", "AT-CTX-018"): ("implementacion", "rev. 3 §12.2: cancelAllTimers"),
    ("AT-SUP-004", "AT-STASH-006"): ("implementacion", "ResumeSupervisor devuelve e.behavior (Supervision.scala:174)"),
    ("AT-SUP-002", "AT-STASH-006"): ("implementacion", "isInstanceOfTheThrowableClass desenvuelve (Supervision.scala:76)"),
    ("AT-SUP-010", "AT-STASH-006"): ("implementacion", "log desenvuelve (Supervision.scala:102)"),
    ("AT-SUP-014", "AT-SIG-006"): ("documental", "SUP-014 no tiene código propio; referencia al contrato"),
    ("AT-SUP-011", "AT-SUP-002"): ("implementacion", "isSame + selección del más interno (Supervision.scala:70-83)"),
    ("AT-SUP-018", "AT-SUP-005"): ("validacion", "restart: PreRestart al Behavior desapilado (Supervision.scala:318,327,416)"),
    ("AT-SUP-018", "AT-SUP-007.1"): ("validacion", "A3 backoff"),
    ("AT-SUP-011", "AT-SUP-005"): ("validacion", "A1, A3, A4, A6 usan restart"),
    ("AT-SUP-011", "AT-SUP-007.1"): ("validacion", "A2, A5, A7 usan restartWithBackoff"),
    ("AT-SUP-012", "AT-SUP-005"): ("validacion", "evidencia real C2: restart tras cambio de estado"),
    ("AT-SUP-019", "AT-SUP-005"): ("validacion", "A1-A3 recorren Restart"),
    ("AT-SUP-019", "AT-SUP-007.1"): ("validacion", "A1-A3 recorren Backoff"),
    ("AT-BEH-007", "AT-SUP-005"): ("validacion", "A8/A9 (SupervisionSpec:353-375,1145-1153,1673-1691) usan restart; no declarada"),
    ("AT-ADP-002", "AT-SUP-005"): ("validacion", "A3 y la variante de override de A4 usan supervise(restart); no declarada (rev. 4b, S6a)"),
}

in_scope = set(assignment)


def build_edges(mode, assignment=assignment):
    """mode: 'declarado' | 'inicio' | 'cierre'. Devuelve {(unidad, unidad): [aristas fila con procedencia]}.

    declarado: SOLO aristas derivadas de `depends_on` del inventario, sin clasificar ni añadir nada.
    inicio:    inventario (reclasificado donde corresponda) + aristas añadidas; solo clase "implementacion".
    cierre:    inicio + las de clase "validacion" (se descartan las "documental").
    Procedencia en la etiqueta: [inv] inventario sin tocar; [reclas:<clase>] inventario reclasificado a mano;
    [anadida:<clase>] propuesta añadida a mano que el inventario no declara.
    """
    inventory = {(o, t) for o in in_scope for t in declared(o) if t in in_scope and o != t}
    pairs = [(o, t, "implementacion", "inv") for (o, t) in sorted(inventory)]
    if mode != "declarado":
        pairs = []
        for (o, t) in sorted(inventory):
            if (o, t) in CLASIFICACION:
                pairs.append((o, t, CLASIFICACION[(o, t)][0], "reclas"))
            elif (o, t) in AGREGADAS:
                pairs.append((o, t, AGREGADAS[(o, t)][0], "reclas"))
            else:
                pairs.append((o, t, "implementacion", "inv"))
        for (o, t), (cls, _) in sorted(AGREGADAS.items()):
            if (o, t) not in inventory:
                pairs.append((o, t, cls, "anadida"))
    edges = {}
    for origin, target, cls, provenance in pairs:
        if origin not in in_scope or target not in in_scope or origin == target:
            continue
        if mode == "inicio" and cls != "implementacion":
            continue
        if mode == "cierre" and cls == "documental":
            continue
        unit_a, unit_b = assignment[origin], assignment[target]
        if unit_a != unit_b:
            tag = "[inv]" if provenance == "inv" else f"[{provenance}:{cls}]"
            edges.setdefault((unit_a, unit_b), set()).add(f"{origin}>{target}{tag}")
    return {k: sorted(v) for k, v in edges.items()}


def find_cycles(edges):
    nodes = sorted({n for pair in edges for n in pair})
    adjacency = {n: sorted(b for (a, b) in edges if a == n) for n in nodes}
    found, state = set(), {}

    def visit(node, path):
        state[node] = 1
        for nxt in adjacency[node]:
            if state.get(nxt) == 1:
                cycle = path[path.index(nxt):] + [nxt] if nxt in path else [node, nxt]
                start = min(range(len(cycle) - 1), key=lambda i: cycle[i])
                body = cycle[:-1][start:] + cycle[:-1][:start]
                found.add(tuple(body))
            elif nxt not in state:
                visit(nxt, path + [nxt] if not path else path + [nxt])
        state[node] = 2

    for node in nodes:
        if node not in state:
            visit(node, [node])
    return sorted(found)


def involves_p55(unit):
    return unit.startswith("S")


print(f"Filas en el conjunto revisado: {len(in_scope)} (31 de P55 + filas AT-* de {', '.join(EXTERNAL_UNITS)})")
print("Tamaños P55 rev. 4:", {u: len(i) for u, i in P55_REV4.items()}, "total=31")
for mode in ("declarado", "inicio", "cierre"):
    edges = build_edges(mode)
    print(f"\n== grafo {mode}: {len(edges)} aristas entre unidades")
    for (a, b), evidence in sorted(edges.items()):
        if involves_p55(a) or involves_p55(b):
            print(f"{a} -> {b}  {evidence}")
    cycles = find_cycles(edges)
    if mode == "declarado":
        print("aristas entre unidades externas (para ubicar los ciclos externos):")
        for (a, b), evidence in sorted(edges.items()):
            if not involves_p55(a) and not involves_p55(b) and "P61" not in (a, b):
                print(f"  {a} -> {b}  {evidence}")
    with_p55 = [c for c in cycles if any(involves_p55(u) for u in c)]
    without_p55 = [c for c in cycles if not any(involves_p55(u) for u in c)]
    print(f"ciclos que incluyen unidades de P55: {with_p55 or 'ninguno'}")
    print(f"ciclos entre unidades externas (fuera del alcance de P55, no analizados): {without_p55 or 'ninguno'}")

# Contraste: sin mover STASH-006 (queda en S4v) el grafo de inicio tiene un ciclo de implementación.
contrast = dict(assignment)
contrast["AT-STASH-006"] = "S4v"
contrast_edges = build_edges("inicio", contrast)
print("\n== contraste (STASH-006 se queda en S4v), grafo inicio")
print("ciclos que incluyen unidades de P55:", [c for c in find_cycles(contrast_edges) if any(involves_p55(u) for u in c)] or "ninguno")
for key in (("S1", "S4v"), ("S4v", "S1")):
    print(key[0], "->", key[1], contrast_edges.get(key))

# Procedencia de las aristas: derivadas del inventario vs. introducidas a mano.
derived = {(o, t) for o in in_scope for t in declared(o) if t in in_scope and o != t}
print("\n== Procedencia de las aristas")
print(f"derivadas del inventario (depends_on, ambos extremos en el conjunto revisado): {len(derived)} pares fila->fila")
reclassified = sorted(k for k in CLASIFICACION if k in derived)
not_declared = sorted(k for k in CLASIFICACION if k not in derived)
print(f"declaradas que la revisión 4 RECLASIFICA a mano (validación/documental): {len(reclassified)}")
for k in reclassified:
    print("  ", k[0], ">", k[1], "->", CLASIFICACION[k][0])
print(f"clasificadas pero NO declaradas en el inventario (solo informativas): {not_declared or 'ninguna'}")
manual_new = sorted(k for k in AGREGADAS if k not in derived)
manual_same = sorted(k for k in AGREGADAS if k in derived)
print(f"añadidas a mano, no declaradas ({len(manual_new)}):")
for k in manual_new:
    print("  ", k[0], ">", k[1], "->", AGREGADAS[k][0])
print(f"listadas en AGREGADAS pero ya declaradas: {manual_same or 'ninguna'}")
print("Las demás aristas se toman tal cual del inventario y se tratan como implementación (no revisadas una por una).")

# Comprobaciones específicas de la separación declarado / inicio / cierre.
flat = lambda mode: {e.split("[")[0] for v in build_edges(mode).values() for e in v}
decl, ini, cie = flat("declarado"), flat("inicio"), flat("cierre")
assert "AT-ADP-002>AT-SUP-005" not in decl, "una arista añadida no puede figurar como declarada"
assert not any(f"{o}>{t}" in decl for (o, t) in AGREGADAS if (o, t) not in derived), "declarado contiene propuestas"
assert "AT-ADP-002>AT-SUP-005" not in ini and "AT-ADP-002>AT-SUP-005" in cie, "validación: solo en cierre"
assert "AT-SUP-013>AT-SUP-002" in ini and "AT-SUP-013>AT-SUP-002" in cie, "implementación añadida: inicio y cierre"
assert "AT-BEH-003>AT-SIG-006" in decl and "AT-BEH-003>AT-SIG-006" not in cie, "documental: declarada, ausente de inicio y cierre"
assert "AT-SIG-006>AT-SUP-014" in decl and "AT-SIG-006>AT-SUP-014" not in ini and "AT-SIG-006>AT-SUP-014" in cie
print("\n== Comprobaciones de la separación declarado/inicio/cierre: OK")
print("ADP-002>SUP-005 declarada en inventario:", ("AT-ADP-002", "AT-SUP-005") in derived)

# Grafo declarado en docs/unit-plan.md (dependencias entre unidades): alcanzabilidad.
with open("docs/unit-plan.md", encoding="utf-8") as handle:
    plan = handle.read()
unit_deps = {}
for match in re.finditer(r"^\| \d+ \| (P\d+) \| \d+ \| ([^|]*)\|", plan, re.M):
    unit_deps[match.group(1)] = re.findall(r"P\d+", match.group(2))


def closure(unit):
    seen, stack = set(), list(unit_deps.get(unit, []))
    while stack:
        current = stack.pop()
        if current not in seen:
            seen.add(current)
            stack.extend(unit_deps.get(current, []))
    return seen


print("\n== unit-plan.md (dependencias declaradas entre unidades)")
for unit in ("P48", "P49", "P52", "P53", "P54", "P55", "P61"):
    print(unit, "->", unit_deps.get(unit))
print("P61 en clausura de P55:", "P61" in closure("P55"), "| P55 en clausura de P61:", "P55" in closure("P61"))
for unit in ("P48", "P49", "P52", "P53", "P54"):
    print(f"P55 en clausura de {unit}:", "P55" in closure(unit), "| P61 en clausura de", unit + ":", "P61" in closure(unit))
