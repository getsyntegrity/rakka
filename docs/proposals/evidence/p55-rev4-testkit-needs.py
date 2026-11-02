"""Herramientas de testkit que usan las aserciones de las 31 filas de P55 y cierre de dependencias de una capacidad mínima.

Uso (desde la raíz del repositorio):
    python3 -I docs/proposals/evidence/p55-rev4-testkit-needs.py [RUTA_UPSTREAM]
RUTA_UPSTREAM por defecto: upstream/pekko (checkout de apache/pekko en 9a54d1294c3ac7821bbed3c940769c3d5027d1ff).

No es un control del repositorio. Parte 1: para cada aserción de cada fila de P55 extrae las citas `Archivo.scala:a-b`
de los contratos, lee ESOS rangos del upstream y detecta herramientas por texto. Es una COTA INFERIOR: los helpers
definidos fuera del rango citado (p. ej. `targetBehavior`, `WatchSetup`) y las citas con rangos adicionales sin nombre de
archivo (`:174-200 y 202-232`) no se ven. Parte 2: cierre de `depends_on` (solo filas AT-*) de dos cortes mínimos de
testkit y las unidades del plan a las que pertenecen; informa si alguna cae en P55 o en unidades posteriores a P55.
"""
import csv
import os
import re
import sys

csv.field_size_limit(10**9)
UPSTREAM = sys.argv[1] if len(sys.argv) > 1 else "upstream/pekko"

P55 = """AT-BEH-010 AT-BEH-024 AT-SIG-002 AT-SIG-005 AT-SIG-006 AT-SUP-001 AT-SUP-002 AT-SUP-003 AT-SUP-004 AT-SUP-005
AT-SUP-006 AT-SUP-007.1 AT-SUP-007.2 AT-SUP-007.3 AT-SUP-008 AT-SUP-009 AT-SUP-010 AT-SUP-011 AT-SUP-012 AT-SUP-013
AT-SUP-014 AT-SUP-016 AT-SUP-017 AT-SUP-018 AT-SUP-019 AT-SUP-020 AT-SUP-021 AT-SUP-022 AT-STASH-006 AT-ADP-002
AT-ADP-005""".split()
assert len(P55) == len(set(P55)) == 31

TOOLS = {
    "BehaviorTestKit": r"BehaviorTestKit",
    "TestInbox": r"TestInbox",
    "LoggingTestKit": r"LoggingTestKit",
    "LogCapturing": r"LogCapturing",
    "ManualTime": r"ManualTime",
    "typed ActorTestKit/TestProbe[T]": r"ScalaTestWithActorTestKit|ActorTestKit|TestProbe\[|createTestProbe|testKit\.",
    "classic TestKit/TestProbe": r"\bTestProbe\(\)|classicTestKit|expectMsg|import pekko\.testkit",
    "sleep": r"Thread\.sleep",
}

# --- contratos: aserciones por fila ---
blocks = {}
for name in ("typed-core.md", "typed-extended.md"):
    text = open(f"docs/inventory/contracts/{name}", encoding="utf-8").read()
    for chunk in re.split(r"\n(?=## AT-)", text):
        head = re.match(r"## (AT-[A-Z]+-\d+(?:\.\d+)?)", chunk)
        if head:
            blocks[head.group(1)] = chunk
assertion = re.compile(r"(?m)^\s*-\s+(A\d+)[.:]\s*(.*)$")
citation = re.compile(r"([A-Za-z0-9]+\.(?:scala|java)):(\d+)(?:-(\d+))?")

paths = {}
for root, _dirs, files in os.walk(UPSTREAM):
    for f in files:
        if f.endswith((".scala", ".java")):
            paths.setdefault(f, os.path.join(root, f))


def read_range(filename, first, last):
    path = paths.get(filename)
    if path is None:
        return None
    lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
    return "\n".join(lines[first - 1:last])


print("== Parte 1: herramientas en los rangos citados por las aserciones (cota inferior)")
per_row = {}
for row in P55:
    found = {}
    count = 0
    for match in assertion.finditer(blocks[row]):
        label, body = match.groups()
        used = set()
        for filename, first, last in citation.findall(body):
            if "Spec" not in filename and "Test" not in filename:
                continue
            code = read_range(filename, int(first), int(last or first))
            if code is None:
                continue
            for tool, pattern in TOOLS.items():
                if re.search(pattern, code):
                    used.add(tool)
        count += 1
        found[label] = sorted(used)
    per_row[row] = found
    print(row, f"({count} aserciones)")
    for label, used in found.items():
        print(f"   {label}: {', '.join(used) if used else '(ninguna detectada en el rango)'}")

print("\nResumen: aserciones por herramienta detectada (cota inferior)")
for tool in TOOLS:
    hits = [f"{r}.{a}" for r, d in per_row.items() for a, used in d.items() if tool in used]
    print(f"{tool}: {len(hits)}")

# --- cortes mínimos de testkit: cierre de depends_on ---
inventory = {}
with open("docs/inventory/actor-typed.tsv", encoding="utf-8") as handle:
    rows = list(csv.reader(handle, delimiter="\t"))
for row in rows[1:]:
    inventory[row[0]] = row[7]
unit_of = {}
for line in open("docs/unit-rows.tsv", encoding="utf-8").read().splitlines()[1:]:
    row_id, unit = line.split("\t")
    unit_of[row_id] = unit
plan = open("docs/unit-plan.md", encoding="utf-8").read()
order = {m.group(2): int(m.group(1)) for m in re.finditer(r"^\| (\d+) \| (P\d+) \| \d+ \|", plan, re.M)}
ROW_ID = re.compile(r"AT-[A-Z]+-\d+(?:\.\d+)?")


# Aristas declaradas que la revisión 4 clasifica como DOCUMENTALES o de VALIDACIÓN (no de implementación) (juicio manual, ver p55-rev4-edges.py y §13 de la propuesta).
DOCUMENTAL = {
    ("AT-ADP-006.2", "AT-BEH-024"), ("AT-ADP-006.4", "AT-SUP-001"), ("AT-ADP-006.3", "AT-SUP-005"),
    ("AT-BEH-003", "AT-SIG-006"), ("AT-CTX-018", "AT-SUP-016"), ("AT-BEH-007", "AT-SUP-001"),
}


def closure(seeds, skip=frozenset()):
    seen, stack = set(), list(seeds)
    while stack:
        current = stack.pop()
        if current in seen or current not in inventory:
            continue
        seen.add(current)
        stack.extend(t for t in ROW_ID.findall(inventory[current]) if (current, t) not in skip)
    return seen


CUTS = {
    "corte A (ActorTestKit + TestProbe tipado)": ["AT-TK-001.1", "AT-TK-001.2", "AT-TK-001.3", "AT-TK-002.1", "AT-TK-002.3", "AT-TK-010", "AT-TK-014"],
    "corte B (LoggingTestKit)": ["AT-TK-006.1", "AT-TK-006.2", "AT-TK-006.3", "AT-TK-006.4", "AT-TK-006.5", "AT-TK-013"],
    "corte C (BehaviorTestKit)": ["AT-TK-003.1", "AT-TK-003.2", "AT-TK-005.1", "AT-TK-011", "AT-TK-012"],
}
print("\n== Parte 2: cierre de depends_on (solo filas AT-*) de cortes mínimos de testkit")
print(f"orden del plan: P55={order['P55']} P62={order['P62']} P63={order['P63']} P64={order['P64']}")
for label, seeds in CUTS.items():
    reached = closure(seeds)
    units = sorted({unit_of[r] for r in reached}, key=lambda u: order[u])
    in_p55 = sorted(r for r in reached if unit_of[r] == "P55")
    after = sorted({unit_of[r] for r in reached if order[unit_of[r]] > order["P55"]}, key=lambda u: order[u])
    print(f"{label}: {len(reached)} filas; unidades={units}")
    print(f"   filas de P55 alcanzadas: {in_p55 or 'ninguna'}")
    print(f"   unidades posteriores a P55 alcanzadas: {after or 'ninguna'}")
    reached_skip = closure(seeds, DOCUMENTAL)
    in_p55_skip = sorted(r for r in reached_skip if unit_of[r] == "P55")
    after_skip = sorted({unit_of[r] for r in reached_skip if order[unit_of[r]] > order["P55"]}, key=lambda u: order[u])
    print(f"   sin las aristas clasificadas como no-implementación: {len(reached_skip)} filas; filas de P55: {in_p55_skip or 'ninguna'}; posteriores a P55: {after_skip or 'ninguna'}")

# --- Parte 3: corte B contrastado con el código leído de LoggingTestKit (rev. 4c) ---
# Lectura manual de actor-testkit-typed @ 9a54d129: internal/LoggingTestKitImpl.scala:44-176, internal/TestAppender.scala:35-137,
# internal/LogbackUtil.scala:25-59, TestKitSettings.scala, scaladsl/LoggingTestKit.scala, LoggingEvent.scala; y
# actor-typed internal/ActorMdc.scala:23-45. Esos seis archivos de actor-testkit-typed no mencionan supervisión, reinicio ni
# ActorAdapter (búsqueda por texto sin distinguir mayúsculas: 0 coincidencias de supervis|restart|ActorAdapter en cada uno).
CODIGO_COMPROBADO = {
    "AT-TK-006.1": "directa: LoggingTestKitImpl.matches (:61-71)",
    "AT-TK-006.2": "directa: fábricas de scaladsl/LoggingTestKit.scala (error[A] = nivel ERROR + causa)",
    "AT-TK-006.3": "directa: LoggingTestKitImpl.expect/awaitDone/awaitNoExcess (:86-121)",
    "AT-TK-006.4": "directa: TestAppender.append/filter (:84-120)",
    "AT-TK-013": "directa: LogbackUtil.getLogbackLogger/convertLevel (:25-59)",
    "AT-TK-010": "directa: TestKitSettings.FilterLeeway y ExpectNoMessageDefaultTimeout (LoggingTestKitImpl.scala:103,109,111)",
    "AT-LOG-001": "transitiva: los eventos observados nacen en ctx.log (ActorContextImpl)",
    "AT-LOG-002": "transitiva: MDC pekkoSource/sourceActorSystem que consulta `source` (ActorMdc.scala:23-45)",
    "AT-ADP-006.1": "transitiva parcial: limpieza del MDC tras cada mensaje (ActorAdapter.aroundReceive, solo esa parte)",
}
print("\n== Parte 3: corte B (LoggingTestKit): dependencias del inventario frente a dependencias leídas en el código")
cut_b = closure(CUTS["corte B (LoggingTestKit)"], DOCUMENTAL)
proven = set(CODIGO_COMPROBADO)
print(f"filas del cierre declarado sin las aristas no-implementación: {len(cut_b)}; con evidencia de código: {len(proven & cut_b)} de {len(proven)}")
for row_id in sorted(proven):
    print(f"   {row_id} [{unit_of.get(row_id)}] {CODIGO_COMPROBADO[row_id]}")
declared_only = sorted(cut_b - proven)
by_unit = {}
for row_id in declared_only:
    by_unit.setdefault(unit_of[row_id], []).append(row_id)
print(f"relación declarada SOLO en el inventario (sin respaldo en el código leído): {len(declared_only)} filas")
for unit in sorted(by_unit, key=lambda u: order[u]):
    print(f"   {unit}: {len(by_unit[unit])} filas")
print("filas de P55 en el cierre declarado:", sorted(r for r in cut_b if unit_of[r] == "P55") or "ninguna")
print("filas de P55 con evidencia de código:", sorted(r for r in proven if unit_of.get(r) == "P55") or "ninguna")

# --- Parte 4: capacidades concretas que usan las aserciones (rev. 4c) ---
# Para cada aserción cuyo rango citado usa LoggingTestKit o un TestProbe, qué condiciones/métodos concretos se invocan.
# Cota inferior (solo el rango citado). Sirve para justificar por escenario qué capacidad mínima hace falta; no define una API.
LOG_FACTORY = re.compile(r"LoggingTestKit\s*\.\s*(error|warn|info|debug|trace|messageContains|custom|deadLetters|empty)\b")
LOG_WITH = re.compile(r"\.\s*(withOccurrences|withLogLevel|withMessageRegex|withMessageContains|withCheckExcess|withSource|withMdc|withLoggerName|withCustom|withCause)\b")
LOG_CAUSE_TYPED = re.compile(r"LoggingTestKit\s*\.\s*(?:error|warn)\s*\[")
PROBE = re.compile(r"\.\s*(expectMessageType|expectMessage|expectNoMessage|expectTerminated|receiveMessages|receiveMessage|fishForMessage|awaitAssert|expectMsgType|expectMsgPF|expectNoMsg|expectMsg|within)\b")
from collections import Counter

log_factory, log_with, probe_methods = Counter(), Counter(), Counter()
log_assertions = no_cond_assertions = 0
for row in P55:
    for match in assertion.finditer(blocks[row]):
        label, body = match.groups()
        code = ""
        for filename, first, last in citation.findall(body):
            if "Spec" not in filename and "Test" not in filename:
                continue
            piece = read_range(filename, int(first), int(last or first))
            if piece:
                code += piece + "\n"
        if "LoggingTestKit" in code:
            log_assertions += 1
            for name in LOG_FACTORY.findall(code):
                log_factory[name] += 1
            for name in LOG_WITH.findall(code):
                log_with[name] += 1
            if LOG_CAUSE_TYPED.search(code):
                log_with["(factoría tipada por causa: error[E]/warn[E])"] += 1
        for name in PROBE.findall(code):
            probe_methods[name] += 1
print("\n== Parte 4: capacidades concretas usadas en los rangos citados (cota inferior; número de apariciones)")
print(f"aserciones cuyo rango usa LoggingTestKit: {log_assertions}")
print("fábricas:", dict(sorted(log_factory.items())))
print("condiciones/ajustes:", dict(sorted(log_with.items())))
print("métodos de sonda/espera:", dict(sorted(probe_methods.items())))
