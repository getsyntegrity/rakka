#!/usr/bin/env python3
"""Derives the mechanical parts of docs/unit-plan.md from its declared dependencies and the registers.

Generated:
  docs/unit-decisions.md   decisions per unit, from docs/contract-map.tsv (row -> Q-ids) and docs/decisions/register.tsv
  docs/unit-plan.md        (a) the `Decisiones` line of every unit card, (b) the execution-order block

Source of truth, never rewritten here: the dependencies declared in the unit tables of docs/unit-plan.md, the row ->
unit assignment in docs/unit-rows.tsv, the decision register. Usage:
  python3 -I tools/package/unit_plan.py           rewrite the generated parts
  python3 -I tools/package/unit_plan.py --check   fail (exit 1) if they are stale or the plan is inconsistent
"""
import collections, csv, os, re, sys

csv.field_size_limit(10**9)
ROOT = os.environ.get("RAKKA_PACKAGE_ROOT") or os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
os.chdir(ROOT)
PLAN, DECISIONS_DOC = "docs/unit-plan.md", "docs/unit-decisions.md"
ALIASES = {"Q-CFG-1": "Q-X-1", "Q-CFG-2": "Q-X-2"}
MARK_DECISIONS = "<!-- generated:unit-decisions -->"
ORDER_BEGIN, ORDER_END = "<!-- BEGIN GENERATED EXECUTION ORDER -->", "<!-- END GENERATED EXECUTION ORDER -->"
SEAMS_BEGIN, SEAMS_END = "<!-- BEGIN GENERATED SEAMS -->", "<!-- END GENERATED SEAMS -->"
ROW_ID = r"A[CT]-[A-Z]+-\d+(?:\.\d+)?"


def read_tsv(path):
    return list(csv.DictReader(open(path, encoding="utf-8", newline=""), delimiter="\t", quoting=csv.QUOTE_NONE))


def load_plan(text):
    """Units, waves and declared dependencies, read from the unit tables and cards of the plan."""
    wave, deps_table = {}, {}
    for block in re.finditer(r"^## (Ola \d+)[^\n]*\n(.*?)(?=^## |\Z)", text, re.M | re.S):
        for m in re.finditer(r"^\| (P\d\d) \| (\S+) \| (\d+) \| ([^|]*) \|", block.group(2), re.M):
            wave[m.group(1)] = int(block.group(1)[4:])
            deps_table[m.group(1)] = set(re.findall(r"P\d\d", m.group(4)))
    cards = {}
    for m in re.finditer(r"(?ms)^### (P\d\d) (\S+)\n(.*?)(?=^### |^## |\Z)", text):
        dep_line = re.search(r"^- \*\*Dependencias[^\n]*?\*\*\s*(.*)$", m.group(3), re.M)
        cards[m.group(1)] = dict(name=m.group(2), deps=set(re.findall(r"P\d\d", dep_line.group(1))) if dep_line else None)
    return wave, deps_table, cards


def expand_braces(pattern):
    m = re.search(r"\{([^{}]*)\}", pattern)
    if not m:
        return [pattern]
    return [x for alt in m.group(1).split(",") for x in expand_braces(pattern[: m.start()] + alt + pattern[m.end():])]


def check_files(text, unit_rows):
    """Planned files: crate matches the module of the unit's rows; files declared as existing do exist."""
    module_of = collections.defaultdict(set)
    for r in unit_rows:
        module_of[r["unit"]].add(r["row_id"][:2])
    problems = []
    for m in re.finditer(r"(?ms)^### (P\d\d) \S+\n(.*?)(?=^### |^## |\Z)", text):
        unit, line = m.group(1), re.search(r"^- \*\*Archivos previstos:\*\*\s*(.*)$", m.group(2), re.M)
        if not line:
            problems.append(f"{unit}: no Archivos previstos line"); continue
        body = line.group(1)
        crate = {"AC": "crates/rakka-actor/", "AT": "crates/rakka-actor-typed/"}[sorted(module_of[unit])[0]]
        paths = re.findall(r"`((?:crates|testkit|testkit-typed)/[^`]*)`", body)
        if "no existe" in body or "sin archivo Rust previsto" in body:   # testkit units (location undecided) and documentary-resolution units
            continue
        if not paths: problems.append(f"{unit}: Archivos previstos without a crate path")
        for path in paths:
            if path.startswith("crates/rakka-actor/tests/") or path.startswith("crates/rakka-actor-typed/tests/"): continue
            if not path.startswith(crate): problems.append(f"{unit}: {path} is not under {crate}")
            if "existentes" in body:
                for f in expand_braces(path):
                    if not os.path.isfile(f): problems.append(f"{unit}: {f} is declared existing but is not in the repository")
    return problems


def row_edges():
    """Row -> row `depends_on` edges of both inventories; a reference to a decomposed parent means all its children."""
    inventory = {}
    for module in ("classic", "typed"):
        for row in read_tsv(f"docs/inventory/actor-{module}.tsv"):
            inventory[row["id"]] = row
    children = collections.defaultdict(list)
    for row_id in inventory:
        if "." in row_id:
            children[row_id.rsplit(".", 1)[0]].append(row_id)
    edges = set()
    for row_id, row in inventory.items():
        for ref in re.findall(ROW_ID, row["depends_on"]):
            for target in (children[ref] if ref in children else [ref]):
                if target != row_id and target in inventory:
                    edges.add((row_id, target))
    return inventory, edges


def seams_block(order, deps, unit_of, edges):
    """Row edges that point to a unit executed later (seams), grouped by unit pair; and the backward edges not covered by the declared graph."""
    position = {p: n for n, p in enumerate(order)}
    closure = {}
    for p in order:
        closure[p] = set()
        for q in deps[p]:
            closure[p] |= {q} | closure.get(q, set())
    forward, uncovered = collections.defaultdict(list), []
    for a, b in sorted(edges):
        ua, ub = unit_of.get(a), unit_of.get(b)
        if not ua or not ub or ua == ub or ua not in position or ub not in position:
            continue
        if position[ub] > position[ua]:
            forward[(ua, ub)].append(f"{a}>{b}")
        elif ub not in closure[ua]:
            uncovered.append(f"{a}>{b} ({ua} does not reach {ub})")
    total = sum(len(v) for v in forward.values())
    lines = [SEAMS_BEGIN, "",
             f"Despues de ordenar las unidades (bloque de orden de ejecucion), {total} aristas `depends_on` fila->fila apuntan a una unidad posterior. "
             "Ninguna crea un ciclo entre unidades (el grafo de unidades es aciclico, verificado). Cada par se resuelve segun `docs/cycles.md` (causa: referencia documental/consumidor, dependencia de validacion o interfaz temporal). "
             "Bloque **generado** por `tools/package/unit_plan.py` desde `depends_on` de los TSV del inventario y `docs/unit-rows.tsv`; pares unidad->unidad con hasta 3 ejemplos:", "",
             "| Unidad | Necesita (posterior) | n | Ejemplos |", "|---|---|---|---|"]
    for (ua, ub) in sorted(forward, key=lambda k: (position[k[0]], position[k[1]])):
        lines.append(f"| {ua} | {ub} | {len(forward[(ua, ub)])} | {', '.join(forward[(ua, ub)][:3])} |")
    lines += ["", SEAMS_END]
    return "\n".join(lines), uncovered, total


def card_rows(text):
    """Rows listed in the `Filas` line of each card."""
    found = {}
    for m in re.finditer(r"(?ms)^### (P\d\d) \S+\n(.*?)(?=^### |^## |\Z)", text):
        line = re.search(r"^- \*\*Filas[^\n]*$", m.group(2), re.M)
        found[m.group(1)] = re.findall(ROW_ID, line.group(0).split("`:", 1)[-1]) if line else None
    return found


def execution_order(wave, deps):
    """Topological order; among the units that are ready, lower wave first, then lower id. Raises on a cycle."""
    done, order, pending = set(), [], sorted(wave, key=lambda p: (wave[p], p))
    while pending:
        ready = [p for p in pending if deps[p] <= done]
        if not ready:
            raise ValueError("dependency cycle among " + ", ".join(pending))
        order.append(ready[0]); done.add(ready[0]); pending.remove(ready[0])
    return order


def excerpt(recommendation, pattern, limit=230):
    m = re.search(pattern, recommendation, re.S)
    if not m:
        return ""
    text = re.sub(r"\s+", " ", m.group(0)).strip()
    return text if len(text) <= limit else text[: limit - 1].rsplit(" ", 1)[0] + "…"


def build_decisions(unit_rows, contract_map, register, cards):
    rows_of_unit = collections.defaultdict(list)
    for r in unit_rows:
        rows_of_unit[r["unit"]].append(r["row_id"])
    reg = {r["id"]: r for r in register}
    # A `group` decision only labels its members: it has no answer of its own, so it never blocks. A row that cites a group
    # is tied to the members it cites itself; the group id is dropped from the row's decisions.
    groups = {q for q, r in reg.items() if r["status"] == "group"}
    decisions_of_row = {r["row_id"]: [q for q in (ALIASES.get(q, q) for q in re.findall(r"Q-(?:T-)?[A-Z]+-\d+", r["decisions"])) if q not in groups]
                        for r in contract_map}
    unknown = sorted({q for qs in decisions_of_row.values() for q in qs if q not in reg})
    per_unit = {}
    for unit, rows in rows_of_unit.items():
        by_decision = collections.defaultdict(list)
        for row in sorted(rows):
            for q in decisions_of_row.get(row, []):
                if q in reg:
                    by_decision[q].append(row)
        per_unit[unit] = by_decision

    def kind(q):
        r = reg[q]
        if r["status"] in ("accepted", "accepted_with_changes", "rejected"):
            return "respondida"
        return "bloqueante" if r["status"] == "blocking" else "propuesta"

    lines = [
        "# Decisiones por unidad",
        "",
        "**Generado** por `tools/package/unit_plan.py` desde `docs/contract-map.tsv` (columna `decisions` de cada fila), `docs/unit-rows.tsv` y "
        "`docs/decisions/register.tsv`. No editar a mano. El estado es el del registro; este documento no registra ni infiere aprobaciones.",
        "",
        "## Cómo leerlo",
        "",
        "- **Agrupaciones:** una decisión de estado `group` (p. ej. Q-X-9) sólo rotula a sus miembros: no tiene respuesta propia, no se lista ni impide cerrar. Cada fila se asocia a los miembros que cita.",
        "- **Estado:** `respondida` (el Owner contestó: se muestra su respuesta), `bloqueante` (registrada como bloqueante, sin respuesta), `propuesta` (recomendación sin respuesta).",
        "- **Impide cerrar la unidad:** toda decisión no respondida cuyo ID figura en alguna fila de la unidad (regla de `AGENTS.md`: una pregunta abierta deja la fila `partial`, nunca `done`). "
        "Se cumple por construcción para las columnas `bloqueante` y `propuesta`.",
        "- **Impide comenzar la unidad:** **no se deduce mecánicamente.** El registro no tiene un campo para eso. Las columnas *Ficha: bloquea* y *Ficha: puede avanzar* citan, tal cual y truncadas, lo que la propia ficha de la decisión afirma "
        "(cuando lo afirma; `—` = la ficha no lo dice y falta leerlo). Hasta que se lea el contrato de cada fila, una decisión sólo debe tratarse como impedimento para comenzar si la ficha lo dice explícitamente.",
        "- Cada excerpt describe la decisión **completa**: puede nombrar filas de otras unidades. Para esta unidad sólo cuentan las filas de la columna 3.",
        "- Los excerpts de la ficha no son aceptaciones: toda recomendación sigue sin aprobar.",
        "",
    ]
    totals = collections.Counter()
    for unit in sorted(per_unit):
        name = cards.get(unit, {}).get("name", "")
        by_decision = per_unit[unit]
        counts = collections.Counter(kind(q) for q in by_decision)
        totals.update(counts)
        lines += [f"## {unit} {name}", "",
                  f"{len(rows_of_unit[unit])} filas; {len(by_decision)} decisiones distintas en sus filas: "
                  f"{counts['respondida']} respondidas, {counts['bloqueante']} bloqueantes, {counts['propuesta']} propuestas.", ""]
        if not by_decision:
            lines += ["Sin decisiones asociadas a sus filas en `contract-map.tsv`.", ""]
            continue
        lines += ["| Decisión | Estado | Filas de la unidad que la citan | Ficha: bloquea | Ficha: puede avanzar | Fuente |", "|---|---|---|---|---|---|"]
        for q in sorted(by_decision, key=lambda x: ({"bloqueante": 0, "respondida": 1, "propuesta": 2}[kind(x)], x)):
            r = reg[q]
            answered = f"{r['status']} ({r['decided_on']}): {excerpt(r['decision'], r'.+', 160)}" if kind(q) == "respondida" else kind(q)
            rows = by_decision[q]
            shown = ", ".join(rows[:6]) + (f" … (+{len(rows) - 6})" if len(rows) > 6 else "")
            blocks = excerpt(r["recommendation"], r"Bloquea\b.*?(?=\s*[|\[]\s*Recomendaci|\s*(?:Puede avanzar|Avanza\b)|$)") or "—"
            advances = excerpt(r["recommendation"], r"(?:Puede avanzar|Avanza\b).*?(?=\s*[|\[]\s*Recomendaci|$)") or "—"
            cell = lambda s: s.replace("|", "\\|")
            lines.append(f"| {q} | {cell(answered)} | {len(rows)}: {shown} | {cell(blocks)} | {cell(advances)} | `{r['source']}` |")
        lines.append("")
    summary = f"Total de pares unidad–decisión: {sum(totals.values())} ({totals['respondida']} respondidas, {totals['bloqueante']} bloqueantes, {totals['propuesta']} propuestas)."
    if unknown:
        summary += f" **IDs de decisión de `contract-map.tsv` ausentes del registro: {', '.join(unknown)}.**"
    lines += ["## Totales", "", summary, ""]
    return "\n".join(lines), per_unit, kind, reg, unknown


def decisions_line(unit, per_unit, kind, reg):
    by_decision = per_unit.get(unit, {})
    if not by_decision:
        return f"- **Decisiones:** ninguna asociada a sus filas en `contract-map.tsv`. {MARK_DECISIONS}"
    groups = collections.defaultdict(list)
    for q in sorted(by_decision):
        groups[kind(q)].append(q)
    parts = [f"{len(by_decision)} distintas en sus filas"]
    for k, label in (("respondida", "respondidas"), ("bloqueante", "bloqueantes sin respuesta"), ("propuesta", "propuestas sin respuesta")):
        if groups[k]:
            ids = ", ".join(groups[k]) if len(groups[k]) <= 6 else f"{len(groups[k])} (ver detalle)"
            parts.append(f"{label}: {ids}")
    return (f"- **Decisiones:** {'; '.join(parts)}. Impiden cerrar la unidad (filas `partial`) todas las no respondidas; "
            f"si impiden comenzar no se deduce mecánicamente: detalle, estado y lo que dice cada ficha en `docs/unit-decisions.md` (sección {unit}). {MARK_DECISIONS}")


def order_block(order, wave, deps):
    lines = [ORDER_BEGIN, "",
             "Orden **derivado** de las dependencias declaradas en las tablas de cada ola (esas dependencias son la fuente de verdad; este bloque lo genera `tools/package/unit_plan.py` y `check.py` lo verifica). "
             "Entre las unidades listas se elige la de menor ola y luego la de menor ID. Es un orden válido, no el único.", "",
             "| # | Unidad | Ola | Depende de |", "|---|---|---|---|"]
    names = {}
    for n, p in enumerate(order, 1):
        lines.append(f"| {n} | {p} | {wave[p]} | {', '.join(sorted(deps[p])) or '-'} |")
    lines += ["", ORDER_END]
    return "\n".join(lines)


def main():
    check = "--check" in sys.argv
    text = open(PLAN, encoding="utf-8").read()
    wave, deps_table, cards = load_plan(text)
    problems = []
    for p in sorted(set(wave) | set(cards)):
        if p not in wave: problems.append(f"{p}: has a card but is not in any wave table")
        elif p not in cards: problems.append(f"{p}: is in a wave table but has no card")
        elif cards[p]["deps"] is None: problems.append(f"{p}: card without Dependencias line")
        elif cards[p]["deps"] != deps_table[p]: problems.append(f"{p}: table depends on {sorted(deps_table[p])}, card on {sorted(cards[p]['deps'])}")
    for p, ds in deps_table.items():
        for q in ds:
            if q not in wave: problems.append(f"{p}: depends on unknown unit {q}")
            elif wave[q] > wave[p]: problems.append(f"{p} (wave {wave[p]}) depends on {q} (later wave {wave[q]})")
    try:
        order = execution_order(wave, deps_table)
    except ValueError as e:
        problems.append(str(e)); order = []
    problems += check_files(text, read_tsv("docs/unit-rows.tsv"))
    unit_rows = read_tsv("docs/unit-rows.tsv")
    unit_of = {r["row_id"]: r["unit"] for r in unit_rows}
    rows_by_unit = collections.defaultdict(set)
    for r in unit_rows:
        rows_by_unit[r["unit"]].add(r["row_id"])
    for p, listed in card_rows(text).items():
        if listed is None: problems.append(f"{p}: card without Filas line")
        elif len(listed) != len(set(listed)) or set(listed) != rows_by_unit.get(p, set()):
            problems.append(f"{p}: Filas of the card differ from docs/unit-rows.tsv (card-only {sorted(set(listed) - rows_by_unit.get(p, set()))}, tsv-only {sorted(rows_by_unit.get(p, set()) - set(listed))})")
    inventory, edges = row_edges()
    seams, uncovered = None, []
    if order:
        seams, uncovered, _ = seams_block(order, deps_table, unit_of, edges)
        for u in uncovered: problems.append(f"row dependency not covered by the declared unit graph: {u}")
    doc, per_unit, kind, reg, unknown = build_decisions(read_tsv("docs/unit-rows.tsv"), read_tsv("docs/contract-map.tsv"), read_tsv("docs/decisions/register.tsv"), cards)
    if unknown: problems.append(f"contract-map.tsv cites decisions missing from the register: {unknown}")
    new = text
    # (a) Decisiones line of each card
    out, current = [], None
    for line in new.split("\n"):
        h = re.match(r"^### (P\d\d) ", line)
        if h: current = h.group(1)
        if current and line.startswith("- **Decisiones:**"):
            line = decisions_line(current, per_unit, kind, reg)
        out.append(line)
    new = "\n".join(out)
    # (b) execution order block
    if order:
        block = order_block(order, wave, deps_table)
        if ORDER_BEGIN in new:
            new = re.sub(re.escape(ORDER_BEGIN) + r".*?" + re.escape(ORDER_END), lambda m: block, new, flags=re.S)
        else:
            new = new.replace("## Ola 0 ", "## Orden de ejecución\n\n" + block + "\n\n## Ola 0 ", 1)
    if seams:
        if SEAMS_BEGIN in new:
            new = re.sub(re.escape(SEAMS_BEGIN) + r".*?" + re.escape(SEAMS_END), lambda m: seams, new, flags=re.S)
        else:
            problems.append("docs/unit-plan.md has no generated seams block")
    current_doc = open(DECISIONS_DOC, encoding="utf-8").read() if os.path.exists(DECISIONS_DOC) else ""
    stale = [f for f, a, b in ((PLAN, text, new), (DECISIONS_DOC, current_doc, doc + "\n" if not doc.endswith("\n") else doc)) if a != b]
    if check:
        for s in stale: problems.append(f"{s} is stale: run python3 -I tools/package/unit_plan.py")
        for pr in problems: print("FAIL", pr)
        print(f"unit_plan: units={len(wave)} order={len(order)} stale={len(stale)} problems={len(problems)}")
        sys.exit(1 if problems else 0)
    if problems:
        for pr in problems: print("PROBLEM", pr)
    open(PLAN, "w", encoding="utf-8").write(new)
    open(DECISIONS_DOC, "w", encoding="utf-8").write(doc if doc.endswith("\n") else doc + "\n")
    print(f"unit_plan: units={len(wave)} order={len(order)} wrote {PLAN}, {DECISIONS_DOC}; problems={len(problems)}")


if __name__ == "__main__":
    main()
