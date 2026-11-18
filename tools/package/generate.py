#!/usr/bin/env python3
"""Generates the decision register, execution plans and the contract map from the inventory TSVs."""
import csv, glob, re, os, collections, json, sys
csv.field_size_limit(10**9)
ROOT = os.environ.get("RAKKA_PACKAGE_ROOT") or os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
os.chdir(ROOT)
R = lambda p: list(csv.reader(open(p, encoding="utf-8", newline=""), delimiter="\t", quoting=csv.QUOTE_NONE))

HEAD = R("docs/inventory/actor-classic.tsv")[0]
rows = []
for f in ("classic", "typed"):
    for x in R(f"docs/inventory/actor-{f}.tsv")[1:]:
        d = dict(zip(HEAD, x)); d["module"] = f; rows.append(d)
by_id = {d["id"]: d for d in rows}
children = collections.defaultdict(list)
for i in by_id:
    if "." in i: children[i.rsplit(".", 1)[0]].append(i)

def expand(token):
    """A reference to a decomposed parent means all of its children."""
    if token in by_id: return [token]
    return children.get(token, [])

# Contract document per row: the one whose heading defines the contract (a mention as a dependency does not count).
DEFINING_HEADING = re.compile(r'(?m)^#+ (A[CT]-[A-Z]+-\d+(?:\.\d+)?)\b')
contract_blocks = collections.defaultdict(list)
for p in sorted(glob.glob("docs/inventory/contracts/*.md")):
    for m in DEFINING_HEADING.finditer(open(p, encoding="utf-8").read()):
        contract_blocks[m.group(1)].append(p)
contract_errors = [f"{i}: no contract block (no heading defines it)" for i in by_id if not contract_blocks[i]]
contract_errors += [f"{i}: {len(contract_blocks[i])} contract blocks ({', '.join(contract_blocks[i])})" for i in by_id if len(contract_blocks[i]) > 1]
if contract_errors:
    sys.exit("contract location errors:\n" + "\n".join(contract_errors))
for i, d in by_id.items():
    d["contract_doc"] = contract_blocks[i][0]

# ------------------------------------------------------------------ decisions
DEC = []
for p in sorted(glob.glob("docs/inventory/contracts/*.decisions.tsv")):
    g = os.path.basename(p).replace(".decisions.tsv", "")
    for x in R(p)[1:]:
        DEC.append(dict(zip(["id", "title", "evidence", "alternatives", "recommendation", "affected_rows", "units", "status"], x), group=g))

GROUPS = {"Q-X-9": ["Q-UTIL-1", "Q-UTIL-2", "Q-UTIL-3", "Q-UTIL-4", "Q-UTIL-5"]}   # presentation only: members are decided one by one
TRANSVERSAL = [
 ("Q-X-1", "Implementación de HOCON (parser y carga)", "Plan docs/superpowers/plans/2026-10-30-ac-cfg-parser-loading.md: golden de 161 casos contra Typesafe Config 1.4.9; `hocon-parser` 1.14.2 diverge en `+=` entre capas, duraciones negativas, `\"ON\"` e `include classpath()`; `hocon-rs` 0.2.0 no tiene `withFallback`; `hocon` 0.9.0 sin mantenimiento desde 2022.", "(A) subconjunto propio sin dependencias; (B) `hocon-parser` tras fachada con una `Q:` por divergencia; (C) (B) más parche upstream.", "(A), con el golden como suite de aceptación y ampliado antes de implementar. Alias: Q-CFG-1.", "AC-CFG-001; AC-CFG-017.*; todo lo que lee Config", "AC-CFG, AC-SER (WireConfig), todas las que leen reference.conf"),
 ("Q-X-2", "Registro nombre → fábrica en lugar de reflexión por FQCN", "AC-CFG-016, AC-EXT-003, DynamicAccessSpec.scala:40,50,59,72; el inventario ya propone el registro; `inventory`/`linkme` descartado como mecanismo principal (dependen del linker).", "(A) `FactoryRegistry<Producto>` por punto de extensión; (B) un registro único con `Any`; (C) macros de registro global.", "(A); un nombre no registrado devuelve un error de configuración que lleva el nombre. Abierto dentro de la decisión: tipo del error (`ConfigurationException` o propio). Alias: Q-CFG-2.", "AC-CFG-016; AC-EXT-003; AC-MBX-004; AC-DSP-010; AC-EVT-011; AC-SCH-010; AC-SER-*", "AC-CFG, AC-EXT, AC-MBX, AC-DSP, AC-EVT, AC-SCH, AC-SER"),
 ("Q-X-3", "Tipo de los mensajes en Classic", "actor-classic.md pregunta 2; Classic es dinámico (`Any` + `PartialFunction`); `unhandled`, `become` y `UnhandledMessage` dependen de poder rechazar un mensaje.", "(A) `Box<dyn Any + Send>` y `receive` que hace downcast y devuelve Handled/Unhandled; (B) enum por actor.", "(A); el tipado fuerte queda en `rakka-actor-typed`.", "AC-REF-002; AC-CELL-001; AC-ASK-001", "AC-REF, AC-CELL, AC-ASK, AC-FSM, AC-RT"),
 ("Q-X-4", "Runtime y modelo de ejecución", "actor-classic.md pregunta 3; `BoundedMailbox` bloquea al emisor (no encaja en async); claves de `fork-join-executor`; rueda de timers `LightArrayRevolverScheduler`.", "(A) pools de hilos propios con interfaz de ejecutor y futuros agnósticos del runtime; (B) atar el port a tokio.", "(A): preserva el bloqueo del emisor, `PinnedDispatcher` y las claves de configuración; tokio puede ser un adaptador opcional más adelante.", "AC-DSP-008; AC-MBX-007; AC-SCH-008", "AC-DSP, AC-MBX, AC-SCH, AC-CELL, AC-ASK"),
 ("Q-X-5", "Qué es un fallo supervisable", "actor-classic.md pregunta 1; Pekko distingue Exception (reinicia) de Error/Throwable (escala).", "(A) `Err` y `panic` son fallos supervisables, tratados como Exception; (B) `panic` escala; (C) no capturar panics.", "(A): capturar panics preserva el aislamiento entre actores; (C) lo rompe.", "AC-SUP-001; AC-SUP-004; AC-SYS-015; AT-SUP-002", "AC-SUP, AC-CELL, AT-SUP"),
 ("Q-X-6", "Suscripción por jerarquía de tipos en EventStream", "actor-classic.md pregunta 5; Rust no tiene herencia; AC-UTIL-008, AC-EVT-003.", "(A) `TypeId` exacto más canales padre declarados por el tipo de evento; (B) enum cerrado de eventos.", "(A).", "AC-UTIL-008; AC-EVT-003", "AC-EVT, AC-UTIL, AT-EVT"),
 ("Q-X-7", "Loggers como actores o backend directo", "actor-classic.md pregunta 8; `LoggerSpec` y `EventFilter` dependen del handshake InitializeLogger/LoggerInitialized (AC-EVT-011).", "(A) loggers como actores con el handshake; (B) backend directo a `tracing`.", "(A): permite portar `LoggerSpec` y `EventFilter` sin cambios; `tracing` puede ser un logger más.", "AC-EVT-011; AC-TK-*", "AC-EVT, AC-TK, AT-LOG"),
 ("Q-X-8", "Typed sobre Classic o runtime tipado nativo", "docs/implementation-order.md (sección Decisión previa); `ActorAdapter`, `ActorContextAdapter`, `ActorSystemAdapter` en actor-typed/.../internal/adapter/.", "(a) replicar las capas de Pekko; (b) runtime tipado nativo con interop aparte.", "(a): cada contrato heredado se implementa y valida una sola vez y la interop sale casi gratis.", "AT-SYS-011; AT-ADP-*; todas las AT-*", "todas las AT-*"),
 ("Q-X-9", "Agrupación de presentación: Done, Timeout, null y unidades de duración (Q-UTIL-1..5)", "docs/superpowers/plans/2026-10-30-ac-util-core.md, sección Preguntas abiertas. Es sólo un encabezado: no tiene estado ni respuesta propios.", "No aplica: cada miembro tiene sus propias alternativas.", "Sin recomendación propia. Responder cada miembro por separado (Q-UTIL-1, Q-UTIL-2, Q-UTIL-3, Q-UTIL-4, Q-UTIL-5); una respuesta a Q-X-9 se rechaza y ninguna respuesta de un miembro aprueba ni rechaza a los demás.", "(las filas de cada miembro)", "AC-UTIL"),
 ("Q-X-10", "Límites del módulo: remote, cluster, streams, persistence, http", "actor-classic.md sección Límites del módulo; ninguno vive en `pekko-actor`/`pekko-actor-typed`; hay ganchos (AC-SYS-018, AC-DW-010/012, AC-SER-007).", "(A) fuera del alcance; el trait `ActorRefProvider` se deja abierto; (B) incluir alguno.", "(A). Es el límite con que se armó el inventario; no fue autorizado explícitamente.", "AC-SYS-018; AC-CFG-008", "AC-SYS, AC-CFG, AC-DW, AC-SER"),
 ("Q-X-11", "Versión de referencia: Apache Pekko 2.0.0", "upstream/pekko @ v2.0.0 (commit 9fbd2283b1cf701c90a1223bf621942c191db61f); docs/upstream.md. Sustituye a A-1 (1.7.1).", "(a) 2.0.0; (b) mantener 1.7.1.", "(a).", "todas las filas", "todas"),
 ("Q-X-12", "Oráculo oficial: CLI Java del piloto con rust-anything", "oracle/ (Java 17), `rust-anything app record/replay`. El oráculo jshell + goldens de tools/oracle queda deprecado y sus escenarios se convierten a casos rust-anything.", "(a) CLI del piloto + rust-anything; (b) jshell + goldens; (c) ambos.", "(a).", "todas las filas", "todas"),
 ("Q-X-13", "docs/unit-plan.md aprobado como guía de orden y alcance", "docs/unit-plan.md: 77 unidades, 783 filas, 10 olas. Las filas y las olas valen; los apartados Tests/oráculo de cada unidad se sustituyen por casos rust-anything (Q-X-12). Levanta la nota Este plan no está aprobado para implementar.", "(a) aprobar con ese reemplazo; (b) seguir sin aprobar.", "(a).", "todas las filas", "todas"),
 ("Q-X-14", "Gates de calidad por slice e hito", "Replay completo, tests, Clippy -D warnings, fmt, cobertura de producción >=95 % por crate, mutantes nuevos en cada slice, y regresión completa de mutantes y fuzzers en cada hito. Sustituye a A-3 (cobertura 90 % de líneas nuevas).", "(a) estos gates; (b) mantener el 90 % de líneas nuevas.", "(a).", "todas las filas", "todas"),
 ("Q-X-15", "Nombres de los crates: rakka-actor y rakka-actor-typed", "crates/rakka-actor, crates/rakka-actor-typed; el piloto usaba pekko-actor y pekko-actor-typed.", "(a) rakka-*; (b) pekko-*.", "(a).", "todas las filas", "todas"),
 ("Q-X-16", "JDK del oráculo fijo en 17", "oracle/ (CLI Java, JDK 17), project/JdkOptions.scala:26 (targetJavaVersion 17). Los mensajes de excepción del JDK forman parte de lo observable y cambian entre versiones: p. ej. URI.scanByte lanza NumberFormatException con `Error at index N in: \"…\"` en JDK 17.", "(a) JDK 17 fijo; (b) JDK 21; (c) ambos.", "(a); las grabaciones de JDK 21 de los goldens deprecados no cuentan.", "AC-PATH-*; AC-UTIL-*; todas las filas con mensajes de excepción del JDK", "todas"),
 ("Q-X-17", "Dominio de entrada de la API pública: &str", "Q-PATH-2; la entrada con un sustituto UTF-16 suelto (p. ej. `pekko://sys\\ud800@host:2552/user`) es representable en un String de Java y no en un &str de Rust. Caso grabado: ok-user-info-lone-surrogate.", "(a) &str y las entradas con sustitutos sueltos quedan fuera de dominio (NOT_APPLICABLE explícito y rechazo explícito en el arnés); (b) UTF-16 en la API.", "(a).", "AC-PATH-*; AC-UTIL-*", "todas"),
 ("Q-X-18", "Procedencia del código: reescritura en sala limpia de lo derivado de fuentes no compatibles o dudosas", "Auditoría de procedencia (docs/legal/provenance-audit.md); uri.rs y jstring.rs declaran estar portados de OpenJDK (GPL-2.0 con Classpath exception), ver Q-PATH-6.", "(a) sala limpia documentada; (b) revisión legal del código existente.", "(a).", "AC-PATH-*; todo archivo de procedencia dudosa", "todas"),
]
for t in TRANSVERSAL:
    DEC.append(dict(id=t[0], title=t[1], evidence=t[2], alternatives=t[3], recommendation=t[4], affected_rows=t[5], units=t[6], status="group" if t[0] in GROUPS else "proposed", group="transversal"))
ALIASES = {"Q-CFG-1": "Q-X-1", "Q-CFG-2": "Q-X-2"}
dec_ids = {d["id"] for d in DEC}
assert len(dec_ids) == len(DEC), "duplicate decision ids"
for d in DEC:
    d["rows"] = set()
    for tok in re.findall(r'A[CT]-[A-Z]+-\d+(?:\.\d+)?', d["affected_rows"]):
        d["rows"] |= set(expand(tok))
    for m in re.finditer(r'(A[CT]-[A-Z]+-\d+)\.\*', d["affected_rows"]):
        d["rows"] |= set(children.get(m.group(1), []))
row_decisions = collections.defaultdict(set)
for d in DEC:
    for r in d["rows"]: row_decisions[r].add(d["id"])
for i, r in by_id.items():           # also decisions cited in notes
    for q in re.findall(r'Q-(?:T-)?[A-Z]+-\d+', r["notes"]):
        q = ALIASES.get(q, q)
        if q in dec_ids: row_decisions[i].add(q)
        elif q not in dec_ids: print("UNDEFINED", i, q, file=sys.stderr)
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))   # -I drops the script directory from sys.path
from contract_decisions import declared_decisions
def contract_declared_decisions():
    declared = {}
    for path in sorted(glob.glob("docs/inventory/contracts/*.md")):
        for row_id, ids in declared_decisions(open(path, encoding="utf-8").read()).items():
            declared.setdefault(row_id, set()).update(ids)
    return declared
for row_id, declared in contract_declared_decisions().items():   # explicit contract declarations count too
    for q in declared:
        if q in dec_ids and row_id in by_id: row_decisions[row_id].add(q)
        elif q not in dec_ids: print("UNDEFINED", row_id, q, file=sys.stderr)
dec_by_id = {d["id"]: d for d in DEC}

# ------------------------------------------------------- Owner responses (the single editable source)
# docs/decisions/owner-decisions.tsv is the only file edited to record an Owner response; everything else
# (register.tsv, decisions.md, unit specs, counts in README.md / implementation-order.md) is derived from it.
OWNER_FILE = "docs/decisions/owner-decisions.tsv"
OWNER_HEADER = ["id", "status", "decided_on", "decided_by", "decision"]
RESOLVED = ("accepted", "accepted_with_changes", "rejected")
STATUS_LABEL = {"blocking": "bloqueante", "proposed": "propuesta (sin decisión del Owner)", "accepted": "aceptada",
                "accepted_with_changes": "aceptada con cambios", "rejected": "rechazada",
                "group": "agrupación de presentación (se responde por miembro, nunca en bloque)"}
def status_label(s): return STATUS_LABEL[s]
for d in DEC: d.update(decided_on="", decided_by="", decision="")
owner_rows = R(OWNER_FILE)
assert owner_rows and owner_rows[0] == OWNER_HEADER, f"{OWNER_FILE}: header must be {OWNER_HEADER}"
for n, x in enumerate(owner_rows[1:], 2):
    if not x or not any(x): continue
    assert len(x) == len(OWNER_HEADER), f"{OWNER_FILE}:{n}: expected {len(OWNER_HEADER)} fields"
    oid, st, on, by, text = x
    oid = ALIASES.get(oid, oid)
    assert oid in dec_by_id, f"{OWNER_FILE}:{n}: unknown decision {oid}"
    assert oid not in GROUPS, f"{OWNER_FILE}:{n}: {oid} only groups {', '.join(GROUPS[oid])}; answer each member separately"
    assert not dec_by_id[oid]["decided_on"], f"{OWNER_FILE}:{n}: duplicate response for {oid}"
    assert st in RESOLVED, f"{OWNER_FILE}:{n}: status must be one of {RESOLVED}"
    assert re.fullmatch(r"\d{4}-\d{2}-\d{2}", on), f"{OWNER_FILE}:{n}: decided_on must be YYYY-MM-DD"
    assert by.strip() and text.strip(), f"{OWNER_FILE}:{n}: decided_by and decision are required"
    dec_by_id[oid].update(status=st, decided_on=on, decided_by=by, decision=text)
counts = collections.Counter(d["status"] for d in DEC if d["status"] != "group")
def counts_text():
    return (f"{sum(counts.values())} decisiones: {counts['blocking']} bloqueantes, {counts['proposed']} propuestas, {counts['accepted']} aceptadas, "
            f"{counts['accepted_with_changes']} aceptadas con cambios, {counts['rejected']} rechazadas")
COUNTS_RE = r"\d+ decisiones: \d+ bloqueantes, \d+ propuestas, \d+ aceptadas(?:, \d+ aceptadas con cambios, \d+ rechazadas)?"
for doc in ("README.md", "docs/implementation-order.md"):
    s = open(doc, encoding="utf-8").read()
    s2 = re.sub(COUNTS_RE, counts_text(), s)
    if s2 != s: open(doc, "w", encoding="utf-8").write(s2)

os.makedirs("docs/decisions", exist_ok=True)
with open("docs/decisions/register.tsv", "w", encoding="utf-8", newline="") as o:
    o.write("\t".join(["id", "title", "status", "recommendation", "affected_rows", "units", "source", "decided_on", "decided_by", "decision"]) + "\n")
    for d in sorted(DEC, key=lambda d: d["id"]):
        o.write("\t".join([d["id"], d["title"], d["status"], d["recommendation"], d["affected_rows"], d["units"], d["group"], d["decided_on"], d["decided_by"], d["decision"]]).replace("\n", " ") + "\n")

with open("docs/decisions.md", "w", encoding="utf-8") as o:
    o.write("# Registro único de decisiones del Owner\n\n")
    o.write("Generado desde `docs/inventory/contracts/*.decisions.tsv` y las decisiones transversales por `tools/` (ver `docs/inventory/README.md`). Fuente de la tabla machine-readable: `docs/decisions/register.tsv`.\n\n")
    o.write(f"**Estado: {counts_text()}.** " + ("Ninguna recomendación fue aceptada por el Owner: todas las filas afectadas siguen en alcance con `impl=pending` y no se cierran hasta que la decisión se registre en `docs/decisions/owner-decisions.tsv` con fecha y autor.\n\n" if not any(counts[s] for s in RESOLVED) else "Las respuestas del Owner (con fecha y autor) figuran en la sección 6 y se leen de `docs/decisions/owner-decisions.tsv`; una respuesta no cierra por sí sola las filas afectadas: eso exige implementación y validación.\n\n"))
    o.write("## 1. Ya autorizado (instrucciones explícitas del Owner)\n\n| ID | Autorización | Alcance |\n|---|---|---|\n")
    o.write("| A-1 | Apache Pekko 2.0.0 es la fuente de verdad, fijada en `upstream/pekko` @ `9fbd2283b1cf701c90a1223bf621942c191db61f` (= `v2.0.0^{}`, verificado con `git ls-remote` el 2026-11-18). Sustituye a la referencia 1.7.1 (decisión Q-X-11). | todo |\n")
    o.write("| A-2 | `feat/pekko-rust-pilot-migration` es la rama de integración; `main` y `develop` están discontinuadas (referencia histórica hasta que el Owner decida archivarlas); sin push directo a la rama de integración salvo pedido explícito del Owner, sin force push ni borrado de ramas sin autorización; las ramas salen de `origin/feat/pekko-rust-pilot-migration`. | proceso |\n")
    o.write("| A-3 | TDD estricto RED → GREEN → REFACTOR con evidencia real; gates de la decisión Q-X-14 (cobertura de producción ≥95 % por crate, que sustituye al 90 % de líneas nuevas anterior); nombres que expresan propósito. | todas las unidades |\n")
    o.write("| A-4 | Ninguna diferencia semántica se acepta sin decisión explícita del Owner; una pregunta abierta impide cerrar el comportamiento afectado. | todas las filas con `Q-` |\n")
    o.write("| A-5 | Ningún comportamiento se elimina por ser difícil de expresar en Rust; las exclusiones requieren autorización explícita. | todo el inventario |\n")
    o.write("| A-6 | Classic y Typed completos, con interoperabilidad y testkits necesarios, están en alcance. | inventario |\n\n")
    o.write("## 2. Cómo responder\n\nResponder con una de estas formas por decisión (o por bloque): `aceptar` (se adopta la recomendación tal cual), `aceptar con cambios: <texto>`, `rechazar: <alternativa>`. Una respuesta global válida es `aceptar todas las recomendaciones excepto: <lista>`. Cada respuesta se registra en una fila de `docs/decisions/owner-decisions.tsv` (`id`, `status` = `accepted` | `accepted_with_changes` | `rejected`, `decided_on` AAAA-MM-DD, `decided_by`, `decision`); es la única fuente editable y este documento, `register.tsv` y los conteos se regeneran desde ella sin perder respuestas. Hasta entonces no hay aprobación.\n\n")
    o.write("## 3. Bloqueos reales\n\nImpiden cerrar el comportamiento afectado aunque el resto de la unidad avance.\n\n")
    def block(d):
        members = f"- **Miembros (decisiones independientes):** {', '.join(GROUPS[d['id']])}\n" if d["id"] in GROUPS else ""
        o.write(f"### {d['id']} — {d['title']}\n\n- **Estado:** {status_label(d['status'])}\n{members}- **Evidencia:** {d['evidence']}\n- **Alternativas:** {d['alternatives']}\n- **Recomendación:** {d['recommendation']}\n- **Filas afectadas:** {d['affected_rows']}\n- **Unidades afectadas:** {d['units']}\n- **Origen:** {d['group']}\n" + (f"- **Respuesta del Owner:** {d['decision']} ({d['decided_by']}, {d['decided_on']})\n" if d['decided_on'] else "") + "\n")
    for d in sorted([d for d in DEC if d["status"] == "blocking"], key=lambda d: d["id"]): block(d)
    o.write("## 4. Decisiones transversales (afectan a varias unidades)\n\nDecidirlas primero: desbloquean la mayor parte de las propuestas por unidad.\n\n")
    for d in [d for d in DEC if d["group"] == "transversal"]: block(d)
    o.write("Alias: " + ", ".join(f"`{k}` = `{v}`" for k, v in ALIASES.items()) + ".\n\n")
    o.write("## 5. Propuestas por unidad\n\n")
    def unitkey(d):
        m = re.match(r'Q-(T-)?([A-Z]+)-(\d+)', d["id"]); return ((m.group(1) or "") + m.group(2), int(m.group(3))) if m else (d["id"], 0)
    for d in sorted([d for d in DEC if d["group"] != "transversal" and d["status"] == "proposed"], key=unitkey): block(d)
    o.write("## 6. Respuestas del Owner por unidad\n\nDecisiones por unidad que el Owner ya resolvió (las transversales figuran en la sección 4).\n\n")
    resolved_units = sorted([d for d in DEC if d["group"] != "transversal" and d["status"] in RESOLVED], key=unitkey)
    for d in resolved_units: block(d)
    if not resolved_units: o.write("Ninguna.\n\n")

# ------------------------------------------------------------------ units / specs
CLASSIC_ORDER = ["PATH", "UTIL", "CFG", "REF", "MBX", "DSP", "SCH", "EVT", "TK", "CELL", "SYS", "SUP", "DW", "EXT", "ASK", "FSM", "RT", "CS", "SER", "IO", "JAPI"]
TYPED_ORDER = ["BEH", "SIG", "REF", "TK", "CTX", "SYS", "TMR", "STASH", "SUP", "ADP", "EVT", "LOG", "DSP", "EXT", "ASK", "RCP", "RT", "PS", "SPV", "CS", "SER", "DLV", "JAPI"]
unit_order = [("AC", u) for u in CLASSIC_ORDER] + [("AT", u) for u in TYPED_ORDER]
unit_rows = collections.defaultdict(list)
for d in rows: unit_rows[(d["id"][:2], d["unit"])].append(d)
ROWS_PER_TASK = 6
TASKS_PER_SPEC = 5
MAX_ROWS_PER_SPEC = ROWS_PER_TASK * TASKS_PER_SPEC
unit_index = {k: i for i, k in enumerate(unit_order)}
pending = [d for d in rows if d["impl"] != "done"]
pending_ids = {d["id"] for d in pending}
row_deps = {}
for d in pending:
    ds = set()
    for tok in re.findall(r'A[CT]-[A-Z]+-\d+(?:\.\d+)?', d["depends_on"]):
        ds |= {c for c in expand(tok) if c in pending_ids and c != d["id"]}
    row_deps[d["id"]] = ds
natural = {d["id"]: i for i, d in enumerate(rows)}
# The depends_on graph is cyclic at row level (e.g. CELL <-> SYS <-> EVT), so the order is by unit
# (docs/implementation-order.md) and every dependency on a later spec is reported as a forward dependency.
sequence = sorted(pending_ids, key=lambda i: (unit_index[(i[:2], by_id[i]["unit"])], natural[i]))
row_cycles = []
specs = []
for key in unit_order:
    done = [d for d in unit_rows[key] if d["impl"] == "done"]
    if done and not any(by_id[i]["unit"] == key[1] and i[:2] == key[0] for i in sequence):
        specs.append(dict(unit=key, index=1, total=1, tasks=[], done=done, seq=0))
run = []
def flush():
    global run
    if not run: return
    key = (run[0][:2], by_id[run[0]]["unit"])
    rs = [by_id[i] for i in run]
    tasks = [rs[i:i + ROWS_PER_TASK] for i in range(0, len(rs), ROWS_PER_TASK)]
    specs.append(dict(unit=key, index=0, total=0, tasks=tasks, done=[d for d in unit_rows[key] if d["impl"] == "done"], seq=0))
    run = []
for i in sequence:
    key = (i[:2], by_id[i]["unit"])
    if run and ((run[0][:2], by_id[run[0]]["unit"]) != key or len(run) >= MAX_ROWS_PER_SPEC): flush()
    run.append(i)
flush()
count_by_unit = collections.Counter(s["unit"] for s in specs); seen = collections.Counter()
order = []
for s in specs:
    seen[s["unit"]] += 1; s["index"] = seen[s["unit"]]; s["total"] = count_by_unit[s["unit"]]
    s["id"] = f"{s['unit'][0]}-{s['unit'][1]}-{s['index']}"
row_spec = {}
for s in specs:
    for t in s["tasks"]:
        for r in t: row_spec[r["id"]] = s["id"]
spec_by_id = {s["id"]: s for s in specs}
def first_pos(s): return min((sequence.index(r["id"]) for t in s["tasks"] for r in t), default=-1)
specs.sort(key=first_pos)
for n, s in enumerate(specs, 1): s["seq"] = n
order = specs
for s in specs:
    deps = collections.OrderedDict()
    for t in s["tasks"]:
        for r in t:
            for c in sorted(row_deps.get(r["id"], ()), key=natural.get):   # sets iterate in hash order: keep output deterministic
                sid = row_spec.get(c)
                if sid and sid != s["id"]: deps.setdefault(sid, set()).add(c)
    s["deps"] = deps
cycles = row_cycles
def specfile(s): return f"docs/superpowers/plans/units/{s['seq']:02d}-{s['id'].lower()}.md"
def kebab(txt): return re.sub(r'[^a-z0-9]+', '-', txt.lower()).strip('-')[:50]

ORACLE_BY_UNIT = {
 "PATH": "casos rust-anything contra Pekko 2.0.0 (JDK 17): `oracle/` (comando `path-address-case`) y `cases/oracle-path-address.jsonl`",
 "UTIL": "casos rust-anything contra Pekko 2.0.0 (JDK 17): `oracle/` (comando `util-core-case`) y `cases/oracle-util-core.jsonl`",
 "CFG": "casos rust-anything contra Pekko 2.0.0 (JDK 17) a grabar al tomar la unidad; `crates/rakka-actor/tests/oracle/config-hocon.golden` (Typesafe Config 1.4.9, jshell deprecado) queda como dato hasta convertirlo",
}
def oracle_for(s):
    u = s["unit"][1]
    if u in ORACLE_BY_UNIT: return ORACLE_BY_UNIT[u]
    if s["unit"][0] == "AT" and u == "TK": return "casos rust-anything contra pekko-actor-testkit-typed 2.0.0: agregar la dependencia a `oracle/pom.xml` es parte de la primera tarea"
    return "casos rust-anything contra Pekko 2.0.0 (JDK 17) con `oracle/` y `tools/parity/` (decisión Q-X-12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1"

def cell(txt):
    """A value inside a Markdown table cell: a literal `|` would split the column."""
    return txt.replace("|", "\\|")

def first(txt, n=110):
    txt = re.sub(r'\s+', ' ', txt).strip()
    return txt if len(txt) <= n else txt[:n - 1].rsplit(' ', 1)[0] + "…"

GATES = """```bash
cargo fmt --all --check
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo llvm-cov --workspace --lcov --output-path target/lcov.info   # cobertura de líneas nuevas o modificadas >= 90 %; reportar comando, resultado y líneas no cubiertas con motivo
```"""

os.makedirs("docs/superpowers/plans/units", exist_ok=True)
for f in glob.glob("docs/superpowers/plans/units/*.md"): os.remove(f)
for s in order:
    u = s["unit"]
    blocking = sorted({q for t in s["tasks"] for r in t for q in row_decisions[r["id"]] if dec_by_id[q]["status"] == "blocking"})
    proposed = sorted({q for t in s["tasks"] for r in t for q in row_decisions[r["id"]] if dec_by_id[q]["status"] == "proposed"})
    nrows = sum(len(t) for t in s["tasks"])
    L = []
    L.append(f"# Spec {s['seq']:02d} — {u[0]}-{u[1]} ({s['index']}/{s['total']})\n")
    L.append(f"**Estado:** {'bloqueada' if blocking else ('pendiente de decisiones propuestas' if proposed else 'sin decisiones abiertas; spec preliminar, pendiente de revisión de atomicidad y dependencias')}. "
             f"**Filas:** {nrows}. **Tareas:** {len(s['tasks'])} (máximo 5). **Ejecutar con:** `superpowers:test-driven-development` por comportamiento y `superpowers:verification-before-completion` al cerrar.\n")
    L.append("## Alcance y exclusiones\n")
    if s["tasks"]:
        L.append(f"Entra: las filas listadas en la tabla de trazabilidad (inventario `docs/inventory/actor-{'classic' if u[0]=='AC' else 'typed'}.tsv`). Exclusiones: ninguna fila se excluye; lo que no figura acá pertenece a otra spec (ver `docs/implementation-order.md`). Una exclusión sólo puede venir de una autorización explícita del Owner registrada en `docs/decisions.md`.\n")
    else:
        L.append("Sin filas pendientes en esta unidad: ya está implementada. Sólo se verifica que no haya regresión.\n")
    if s["done"]:
        L.append("Filas ya `done` en esta unidad (no se reabren): " + ", ".join(r["id"] for r in s["done"]) + ".\n")
    L.append("## Prerrequisitos\n")
    if s["deps"]:
        for sid, via in s["deps"].items():
            tgt = spec_by_id[sid]; fwd = " **(dependencia hacia adelante: aplicar la regla de costura de `units/README.md`)**" if tgt["seq"] > s["seq"] else ""
            L.append(f"- Spec {tgt['seq']:02d} `{sid}` (`{os.path.basename(specfile(tgt))}`), por las filas {', '.join(sorted(via))}.{fwd}")
        L.append("")
    else:
        L.append("- Ninguno entre specs de este plan.\n")
    L.append("Decisiones:\n")
    L.append("- Bloqueantes: " + (", ".join(f"`{q}`" for q in blocking) if blocking else "ninguna") + ".")
    L.append("- Propuestas aún sin decisión del Owner: " + (", ".join(f"`{q}`" for q in proposed) if proposed else "ninguna") + ".")
    L.append("- Regla: una fila con decisión abierta se implementa hasta donde el contrato no depende de ella y queda `partial`; nunca se cierra sobre una recomendación no aceptada.\n")
    L.append(f"## Estrategia de oráculo y procedencia\n\n{oracle_for(s)}. Todo golden lleva en su cabecera la fuente exacta (jars con SHA-1 verificado, o commit de Pekko) y el hash del script, como los goldens existentes.\n")
    L.append("## Tareas\n")
    for n, t in enumerate(s["tasks"], 1):
        ids = ", ".join(r["id"] for r in t)
        contract_docs = ", ".join(f"`{p}`" for p in sorted({r["contract_doc"] for r in t}))
        L.append(f"### Tarea {n}: {first(t[0]['feature'], 90)}" + (f" (+{len(t)-1} filas)" if len(t) > 1 else "") + "\n")
        L.append(f"Filas: {ids}. Contratos y aserciones: {contract_docs} (bloque de cada fila; sección **Aserciones**).\n")
        L.append("1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.")
        L.append("2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.")
        L.append("3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.")
        L.append("4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.\n")
    L.append("## Trazabilidad contrato → aserción → validación\n")
    L.append("| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |\n|---|---|---|---|---|")
    for t in s["tasks"]:
        for r in t:
            val = first(r["tests"], 90) if r["tests"].strip() not in ("", "-") else "derivada del código (ver bloque de contrato)"
            L.append(f"| {r['id']} | {cell(first(r['feature'], 80))} | {cell(first(r['rust_equivalent'], 90))} | {cell(val)} | {', '.join(sorted(row_decisions[r['id']])) or '-'} |".replace("\n", " "))
    L.append("")
    L.append("## Gates y criterio de cierre\n")
    L.append(GATES + "\n")
    L.append("La spec se cierra únicamente cuando: (1) todos los gates pasan; (2) cada aserción `A#` de cada fila tiene un test que se observó fallar y luego pasar; (3) la cobertura de líneas nuevas o modificadas es >= 90 % y está documentada; (4) las columnas `impl` y `validation` del inventario reflejan lo demostrado (`partial` si queda una decisión abierta o una diferencia); (5) las diferencias observables respecto de Pekko están anotadas con su `Q-`. Tests verdes no equivalen a paridad.\n")
    nxt = order[s["seq"]] if s["seq"] < len(order) else None
    L.append(f"## Siguiente\n\n{('Spec %02d `%s` (`%s`).' % (nxt['seq'], nxt['id'], os.path.basename(specfile(nxt)))) if nxt else 'Última spec del plan.'}\n")
    open(specfile(s), "w", encoding="utf-8").write("\n".join(L))

# ------------------------------------------------------------------ order doc + map
with open("docs/superpowers/plans/units/README.md", "w", encoding="utf-8") as o:
    o.write("# Specs preliminares (generadas desde el inventario)\n\n**Estado de las specs:** las 49 specs actuales son agrupaciones preliminares de filas del inventario. Aún requieren revisión de atomicidad y de dependencias antes de considerarse ejecutables; no empezar la implementación de una unidad hasta que esa revisión la apruebe.\n\nOrden de ejecución, dependencias resueltas y estado. Regenerar con `python3 -I tools/package/generate.py` (ver `docs/inventory/README.md`).\n\n| # | Spec | Filas | Tareas | Estado | Depende de |\n|---|---|---|---|---|---|\n")
    for s in order:
        nrows = sum(len(t) for t in s["tasks"])
        blocking = {q for t in s["tasks"] for r in t for q in row_decisions[r["id"]] if dec_by_id[q]["status"] == "blocking"}
        proposed = {q for t in s["tasks"] for r in t for q in row_decisions[r["id"]] if dec_by_id[q]["status"] == "proposed"}
        st = "bloqueada" if blocking else ("pendiente de decisiones" if proposed else "sin decisiones abiertas (preliminar)")
        o.write(f"| {s['seq']:02d} | [{s['id']}]({os.path.basename(specfile(s))}) | {nrows} | {len(s['tasks'])} | {st} | {', '.join(f'{spec_by_id[d]['seq']:02d}' for d in s['deps']) or '-'} |\n")
    o.write("\n## Ciclos y dependencias hacia adelante\n\n")
    fwd = [(s["id"], d) for s in order for d in s["deps"] if spec_by_id[d]["seq"] > s["seq"]]
    o.write(f"El grafo `depends_on` del inventario es cíclico a nivel de fila (Classic: CELL, SYS, EVT, SCH, TK se necesitan entre sí), por eso el orden es por unidad. {len(fwd)} dependencias de spec van hacia una spec posterior. Regla de costura para todas: la spec que consume define un trait con el contrato mínimo que necesita, lo prueba con un doble de prueba que cumple las aserciones del bloque de contrato de la fila, y cuando llega la spec que lo implementa el doble se reemplaza y se re-ejecutan los tests; nada de eso reduce el contrato de la fila posterior. Lista:\n\n")
    for a, b in fwd: o.write(f"- `{a}` necesita `{b}`.\n")


with open("docs/contract-map.tsv", "w", encoding="utf-8", newline="") as o:
    o.write("\t".join(["row_id", "unit", "module", "spec", "spec_seq", "analysis", "impl", "validation_source", "oracle", "decisions", "contract_doc"]) + "\n")
    for d in rows:
        sid = row_spec.get(d["id"], "(implementada)")
        seq = spec_by_id[sid]["seq"] if sid in spec_by_id else ""
        o.write("\t".join([d["id"], d["unit"], d["module"], sid, str(seq), d["analysis"], d["impl"], first(d["tests"], 120) if d["tests"].strip() not in ("", "-") else "derivada del código", oracle_for(spec_by_id[sid])[:40] if sid in spec_by_id else "existente", ",".join(sorted(row_decisions[d["id"]])), d["contract_doc"]]).replace("\n", " ") + "\n")

# unit -> specs table for docs/implementation-order.md (between markers)
tbl = ["| Unidad | Filas pendientes | Specs (orden de ejecución) |", "|---|---|---|"]
for key in unit_order:
    ss = [x for x in order if x["unit"] == key]
    n = sum(len(t) for x in ss for t in x["tasks"])
    tbl.append(f"| {key[0]}-{key[1]} | {n} | " + (", ".join(f"{x['seq']:02d}" for x in ss) or "ninguna (implementada)") + " |")
p_ord = "docs/implementation-order.md"
txt = open(p_ord, encoding="utf-8").read()
a, b = "<!-- BEGIN GENERATED UNIT TABLE -->", "<!-- END GENERATED UNIT TABLE -->"
if a in txt:
    txt = txt[:txt.index(a) + len(a)] + "\n" + "\n".join(tbl) + "\n" + txt[txt.index(b):]
    open(p_ord, "w", encoding="utf-8").write(txt)

stats = dict(rows=len(rows), specs=len(specs), forward=len([1 for s in order for d in s["deps"] if spec_by_id[d]["seq"] > s["seq"]]),
             unmapped=[d["id"] for d in rows if d["id"] not in row_spec and d["impl"] != "done"], decisions=len(DEC), cycles=len(cycles),
             max_tasks=max(len(s["tasks"]) for s in specs), listed=sum(1 for d in rows if d["analysis"] == "listed"))
print(json.dumps(stats))
json.dump({s["id"]: dict(seq=s["seq"], file=specfile(s)) for s in order}, open("/dev/null", "w"))


# ------------------------------------------------------------------ docs/progress.md (generated from the TSVs)
def _progress_md():
    """Progress of the port, computed from the inventory (impl column), docs/unit-rows.tsv and docs/unit-plan.md."""
    import csv as _csv
    _csv.field_size_limit(10**9)
    read = lambda p: list(_csv.reader(open(p, encoding="utf-8", newline=""), delimiter="\t", quoting=_csv.QUOTE_NONE))
    inv = []
    for kind in ("classic", "typed"):
        t = read(f"docs/inventory/actor-{kind}.tsv")
        inv += [dict(zip(t[0], x)) for x in t[1:]]
    unit_of = {x[0]: x[1] for x in read("docs/unit-rows.tsv")[1:]}
    plan = open("docs/unit-plan.md", encoding="utf-8").read().split("\n")
    wave_of, name_of, wave_title, cur = {}, {}, {}, None
    for ln in plan:
        m = re.match(r"^## (Ola (\d+)) - (.*)$", ln)
        if m: cur = int(m.group(2)); wave_title[cur] = m.group(3); continue
        m = re.match(r"^### (P\d+) (\S+)", ln)
        if m and cur is not None: wave_of[m.group(1)] = cur; name_of[m.group(1)] = m.group(2)
    compared = set()
    if os.path.exists("docs/inventory/compared-areas.txt"):
        compared = {l.strip() for l in open("docs/inventory/compared-areas.txt", encoding="utf-8") if l.strip() and not l.startswith("#")}
    area_of = lambda i: i.split("-")[1]
    flagged = {r["id"] for r in inv if "revisar contra 2.0.0" in r["notes"]}
    def tally(rs):
        c = collections.Counter(r["impl"] for r in rs)
        n = len(rs)
        return n, c["done"], c["partial"], c["pending"]
    pct = lambda a, n: f"{100.0 * a / n:.1f} %" if n else "-"
    def line(label, rs, extra=""):
        n, d, p, q = tally(rs)
        return f"| {label} | {n} | {d} | {p} | {q} | {pct(d, n)} | {pct(d + p, n)} |{extra}"
    HEAD = "| {} | Filas | done | partial | pending | % done | % done + partial |"
    SEP = "|---|---|---|---|---|---|---|"
    out = ["# Avance del port", "",
           "Generado por `tools/package/generate.py` desde `docs/inventory/actor-*.tsv` (columna `impl`), `docs/unit-rows.tsv` y `docs/unit-plan.md`. **No editar a mano.**",
           "`done` = todas las aserciones del contrato cubiertas por casos grabados de Pekko 2.0.0 que pasan y sin decisión abierta; `partial` = algunas cubiertas o una `Q:` abierta; `pending` = ninguna (o sin comparar).", ""]
    classic = [r for r in inv if r["id"].startswith("AC-")]
    typed = [r for r in inv if r["id"].startswith("AT-")]
    out += ["## Resumen", "", HEAD.format("Módulo"), SEP, line("Classic (`rakka-actor`)", classic), line("Typed (`rakka-actor-typed`)", typed), line("**Total**", inv), ""]
    # by wave
    out += ["## Por ola", "", HEAD.format("Ola"), SEP]
    for w in sorted(wave_title):
        rs = [r for r in inv if wave_of.get(unit_of.get(r["id"])) == w]
        out.append(line(f"{w} - {wave_title[w]}", rs))
    out.append("")
    # by unit
    units = sorted(set(unit_of.values()), key=lambda u: int(u[1:]))
    scores = {}
    out += ["## Por unidad del plan", "", "| Unidad | Ola | Filas | done | partial | pending | % done | % done + partial | Áreas comparadas |", "|---|---|---|---|---|---|---|---|---|"]
    for u in units:
        rs = [r for r in inv if unit_of.get(r["id"]) == u]
        n, d, p, q = tally(rs)
        areas = sorted({area_of(r["id"]) for r in rs})
        done_cmp = [a for a in areas if a in compared]
        cmp_text = "sí" if len(done_cmp) == len(areas) and areas else ("parcial" if done_cmp else "no")
        scores[u] = ((d + 0.5 * p) / n if n else 0.0, n, d, p)
        out.append(f"| {u} {name_of.get(u, '')} | {wave_of.get(u, '?')} | {n} | {d} | {p} | {q} | {pct(d, n)} | {pct(d + p, n)} | {cmp_text} |")
    out.append("")
    ranked = [u for u in units if scores[u][1]]
    best = sorted(ranked, key=lambda u: (-scores[u][0], -scores[u][1], u))[:5]
    worst = sorted(ranked, key=lambda u: (scores[u][0], -scores[u][1], u))[:5]
    zero = [u for u in ranked if scores[u][2] == 0 and scores[u][3] == 0]
    out += ["## Más avanzadas y más atrasadas", "", "Orden por (done + ½·partial) / filas. Entre las atrasadas hay empates en 0 %: se desempata por el tamaño de la unidad.", "",
            "**5 más avanzadas:** " + ", ".join(f"{u} {name_of.get(u, '')} ({scores[u][2]} done, {scores[u][3]} partial de {scores[u][1]})" for u in best), "",
            "**5 más atrasadas:** " + ", ".join(f"{u} {name_of.get(u, '')} ({scores[u][1]} filas)" for u in worst) + f". Unidades en 0 %: {len(zero)} de {len(ranked)}.", ""]
    # by area, with the comparison flag
    areas = sorted({(r["id"][:2], area_of(r["id"])) for r in inv})
    out += ["## Por área del inventario y comparación aserción por aserción", "",
            "Un área figura como comparada cuando se contrastaron las aserciones de cada una de sus filas con los casos grabados (`docs/inventory/compared-areas.txt`). En las áreas **sin comparar** todas las filas siguen `pending` aunque algún caso grabado las roce: falta hacer la comparación al tomar su unidad.", "",
            HEAD.format("Área").replace("Filas", "Filas") + " Comparada |", SEP[:-1] + "|---|"]
    for kind, a in areas:
        rs = [r for r in inv if r["id"].startswith(kind + "-" + a + "-")]
        out.append(line(f"{kind}-{a}", rs, f" {'sí' if a in compared and kind == 'AC' else 'no'} |"))
    out.append("")
    nf = [r for r in inv if r["id"] in flagged]
    n, d, p, q = tally(nf)
    out += ["## Filas con nota «revisar contra 2.0.0»", "",
            f"{n} filas ({d} done, {p} partial, {q} pending): su texto citado cambió entre 1.7.1 y 2.0.0 (ver `docs/inventory/pekko-2.0.0-reanchor.md`). Se revisan cuando un slice toma su unidad; su estado no se tocó al reanclar.", ""]
    if os.path.exists("cases/coverage.tsv"):
        cov = read("cases/coverage.tsv")[1:]
        none = [x[0] for x in cov if x[2] == "-"]
        out += ["## Casos grabados y filas que cubren", "",
                f"`cases/coverage.tsv` declara las filas que cubre cada uno de los {len(cov)} casos (`tools/parity/apply_comparison.py`). Base `cited`: el caso lo cita una fila comparada; `same-group`: hereda las filas de su grupo de nombres; `none`: **{len(none)} casos sin fila asignada**, a mapear cuando un slice tome su unidad.", ""]
    return "\n".join(out)

open("docs/progress.md", "w", encoding="utf-8").write(_progress_md())
