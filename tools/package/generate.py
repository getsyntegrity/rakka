#!/usr/bin/env python3
"""Generates the decision register, execution plans and the contract map from the inventory TSVs."""
import csv, glob, re, os, collections, json, sys
csv.field_size_limit(10**9)
ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
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

# contract file per row
contract_file = {}
for p in sorted(glob.glob("docs/inventory/contracts/*.md")):
    g = os.path.basename(p)[:-3]
    for x in R(f"/dev/null") if False else []: pass
group_of_unit = {}
for g in ["classic-foundation", "classic-cell", "classic-dispatch", "classic-system", "classic-patterns", "classic-io-ser", "typed-core", "typed-extended"]:
    txt = open(f"docs/inventory/contracts/{g}.md", encoding="utf-8").read()
    for m in re.finditer(r'(A[CT]-[A-Z]+-\d+(?:\.\d+)?)', txt):
        contract_file.setdefault(m.group(1), g)
for i, d in by_id.items():
    d["contract_doc"] = f"docs/inventory/contracts/{contract_file.get(i, '?')}.md"

# ------------------------------------------------------------------ decisions
DEC = []
for p in sorted(glob.glob("docs/inventory/contracts/*.decisions.tsv")):
    g = os.path.basename(p).replace(".decisions.tsv", "")
    for x in R(p)[1:]:
        DEC.append(dict(zip(["id", "title", "evidence", "alternatives", "recommendation", "affected_rows", "units", "status"], x), group=g))

TRANSVERSAL = [
 ("Q-X-1", "Implementación de HOCON (parser y carga)", "Plan docs/superpowers/plans/2026-10-30-ac-cfg-parser-loading.md: golden de 161 casos contra Typesafe Config 1.4.9; `hocon-parser` 1.14.2 diverge en `+=` entre capas, duraciones negativas, `\"ON\"` e `include classpath()`; `hocon-rs` 0.2.0 no tiene `withFallback`; `hocon` 0.9.0 sin mantenimiento desde 2022.", "(A) subconjunto propio sin dependencias; (B) `hocon-parser` tras fachada con una `Q:` por divergencia; (C) (B) más parche upstream.", "(A), con el golden como suite de aceptación y ampliado antes de implementar. Alias: Q-CFG-1.", "AC-CFG-001; AC-CFG-017.*; todo lo que lee Config", "AC-CFG, AC-SER (WireConfig), todas las que leen reference.conf"),
 ("Q-X-2", "Registro nombre → fábrica en lugar de reflexión por FQCN", "AC-CFG-016, AC-EXT-003, DynamicAccessSpec.scala:40,50,59,72; el inventario ya propone el registro; `inventory`/`linkme` descartado como mecanismo principal (dependen del linker).", "(A) `FactoryRegistry<Producto>` por punto de extensión; (B) un registro único con `Any`; (C) macros de registro global.", "(A); un nombre no registrado devuelve un error de configuración que lleva el nombre. Abierto dentro de la decisión: tipo del error (`ConfigurationException` o propio). Alias: Q-CFG-2.", "AC-CFG-016; AC-EXT-003; AC-MBX-004; AC-DSP-010; AC-EVT-011; AC-SCH-010; AC-SER-*", "AC-CFG, AC-EXT, AC-MBX, AC-DSP, AC-EVT, AC-SCH, AC-SER"),
 ("Q-X-3", "Tipo de los mensajes en Classic", "actor-classic.md pregunta 2; Classic es dinámico (`Any` + `PartialFunction`); `unhandled`, `become` y `UnhandledMessage` dependen de poder rechazar un mensaje.", "(A) `Box<dyn Any + Send>` y `receive` que hace downcast y devuelve Handled/Unhandled; (B) enum por actor.", "(A); el tipado fuerte queda en `rakka-actor-typed`.", "AC-REF-002; AC-CELL-001; AC-ASK-001", "AC-REF, AC-CELL, AC-ASK, AC-FSM, AC-RT"),
 ("Q-X-4", "Runtime y modelo de ejecución", "actor-classic.md pregunta 3; `BoundedMailbox` bloquea al emisor (no encaja en async); claves de `fork-join-executor`; rueda de timers `LightArrayRevolverScheduler`.", "(A) pools de hilos propios con interfaz de ejecutor y futuros agnósticos del runtime; (B) atar el port a tokio.", "(A): preserva el bloqueo del emisor, `PinnedDispatcher` y las claves de configuración; tokio puede ser un adaptador opcional más adelante.", "AC-DSP-008; AC-MBX-007; AC-SCH-008", "AC-DSP, AC-MBX, AC-SCH, AC-CELL, AC-ASK"),
 ("Q-X-5", "Qué es un fallo supervisable", "actor-classic.md pregunta 1; Pekko distingue Exception (reinicia) de Error/Throwable (escala).", "(A) `Err` y `panic` son fallos supervisables, tratados como Exception; (B) `panic` escala; (C) no capturar panics.", "(A): capturar panics preserva el aislamiento entre actores; (C) lo rompe.", "AC-SUP-001; AC-SUP-004; AC-SYS-015; AT-SUP-002", "AC-SUP, AC-CELL, AT-SUP"),
 ("Q-X-6", "Suscripción por jerarquía de tipos en EventStream", "actor-classic.md pregunta 5; Rust no tiene herencia; AC-UTIL-008, AC-EVT-003.", "(A) `TypeId` exacto más canales padre declarados por el tipo de evento; (B) enum cerrado de eventos.", "(A).", "AC-UTIL-008; AC-EVT-003", "AC-EVT, AC-UTIL, AT-EVT"),
 ("Q-X-7", "Loggers como actores o backend directo", "actor-classic.md pregunta 8; `LoggerSpec` y `EventFilter` dependen del handshake InitializeLogger/LoggerInitialized (AC-EVT-011).", "(A) loggers como actores con el handshake; (B) backend directo a `tracing`.", "(A): permite portar `LoggerSpec` y `EventFilter` sin cambios; `tracing` puede ser un logger más.", "AC-EVT-011; AC-TK-*", "AC-EVT, AC-TK, AT-LOG"),
 ("Q-X-8", "Typed sobre Classic o runtime tipado nativo", "docs/implementation-order.md (sección Decisión previa); `ActorAdapter`, `ActorContextAdapter`, `ActorSystemAdapter` en actor-typed/.../internal/adapter/.", "(a) replicar las capas de Pekko; (b) runtime tipado nativo con interop aparte.", "(a): cada contrato heredado se implementa y valida una sola vez y la interop sale casi gratis.", "AT-SYS-011; AT-ADP-*; todas las AT-*", "todas las AT-*"),
 ("Q-X-9", "Done, Timeout, null y unidades de duración (Q-UTIL-1..5)", "docs/superpowers/plans/2026-10-30-ac-util-core.md, sección Preguntas abiertas; filas AC-UTIL-001..004 `partial`.", "Ver las cinco preguntas del plan (negativos y rango de `Timeout`, `toString` con unidad, `null` y mensajes diferidos, jerarquía de excepciones, `Done` propio o `()`).", "Mantener lo implementado (struct `Done`, `Timeout` sobre `std::time::Duration`) como estado `partial` hasta que el Owner responda cada una; no se cierra ninguna de las cinco por defecto.", "AC-UTIL-001..004", "AC-UTIL"),
 ("Q-X-10", "Límites del módulo: remote, cluster, streams, persistence, http", "actor-classic.md sección Límites del módulo; ninguno vive en `pekko-actor`/`pekko-actor-typed`; hay ganchos (AC-SYS-018, AC-DW-010/012, AC-SER-007).", "(A) fuera del alcance; el trait `ActorRefProvider` se deja abierto; (B) incluir alguno.", "(A). Es el límite con que se armó el inventario; no fue autorizado explícitamente.", "AC-SYS-018; AC-CFG-008", "AC-SYS, AC-CFG, AC-DW, AC-SER"),
]
for t in TRANSVERSAL:
    DEC.append(dict(id=t[0], title=t[1], evidence=t[2], alternatives=t[3], recommendation=t[4], affected_rows=t[5], units=t[6], status="proposed", group="transversal"))
ALIASES = {"Q-CFG-1": "Q-X-1", "Q-CFG-2": "Q-X-2", "Q-UTIL-1": "Q-X-9", "Q-UTIL-2": "Q-X-9", "Q-UTIL-3": "Q-X-9", "Q-UTIL-4": "Q-X-9", "Q-UTIL-5": "Q-X-9"}
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
dec_by_id = {d["id"]: d for d in DEC}

def status_label(s): return {"blocking": "bloqueante", "proposed": "propuesta (sin decisión del Owner)"}[s]

os.makedirs("docs/decisions", exist_ok=True)
with open("docs/decisions/register.tsv", "w", encoding="utf-8", newline="") as o:
    o.write("\t".join(["id", "title", "status", "recommendation", "affected_rows", "units", "source"]) + "\n")
    for d in sorted(DEC, key=lambda d: d["id"]):
        o.write("\t".join([d["id"], d["title"], d["status"], d["recommendation"], d["affected_rows"], d["units"], d["group"]]).replace("\n", " ") + "\n")
        pass

with open("docs/decisions.md", "w", encoding="utf-8") as o:
    cnt = collections.Counter(d["status"] for d in DEC)
    o.write("# Registro único de decisiones del Owner\n\n")
    o.write("Generado desde `docs/inventory/contracts/*.decisions.tsv` y las decisiones transversales por `tools/` (ver `docs/inventory/README.md`). Fuente de la tabla machine-readable: `docs/decisions/register.tsv`.\n\n")
    o.write(f"**Estado: {len(DEC)} decisiones, {cnt['blocking']} bloqueantes, {cnt['proposed']} propuestas, 0 aceptadas.** Ninguna recomendación fue aceptada por el Owner: todas las filas afectadas siguen en alcance con `impl=pending` y no se cierran hasta que la decisión se registre acá con fecha y autor.\n\n")
    o.write("## 1. Ya autorizado (instrucciones explícitas del Owner)\n\n| ID | Autorización | Alcance |\n|---|---|---|\n")
    o.write("| A-1 | Apache Pekko 1.7.1 es la fuente de verdad, fijada en `upstream/pekko` @ `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (= `v1.7.1^{}`, verificado con `git ls-remote` el 2026-10-30). | todo |\n")
    o.write("| A-2 | `develop` es la rama de integración; `main` sólo recibe PR de release; sin push directo ni force push; las ramas salen de `origin/develop`. | proceso |\n")
    o.write("| A-3 | TDD estricto RED → GREEN → REFACTOR con evidencia real; cobertura mínima 90 % de las líneas de producción nuevas o modificadas; nombres que expresan propósito. | todas las unidades |\n")
    o.write("| A-4 | Ninguna diferencia semántica se acepta sin decisión explícita del Owner; una pregunta abierta impide cerrar el comportamiento afectado. | todas las filas con `Q-` |\n")
    o.write("| A-5 | Ningún comportamiento se elimina por ser difícil de expresar en Rust; las exclusiones requieren autorización explícita. | todo el inventario |\n")
    o.write("| A-6 | Classic y Typed completos, con interoperabilidad y testkits necesarios, están en alcance. | inventario |\n\n")
    o.write("## 2. Cómo responder\n\nResponder con una de estas formas por decisión (o por bloque): `aceptar` (se adopta la recomendación tal cual), `aceptar con cambios: <texto>`, `rechazar: <alternativa>`. Una respuesta global válida es `aceptar todas las recomendaciones excepto: <lista>`. Cada decisión aceptada se registra en la columna `status` como `accepted` con la fecha; hasta entonces no hay aprobación.\n\n")
    o.write("## 3. Bloqueos reales\n\nImpiden cerrar el comportamiento afectado aunque el resto de la unidad avance.\n\n")
    def block(d):
        o.write(f"### {d['id']} — {d['title']}\n\n- **Estado:** {status_label(d['status'])}\n- **Evidencia:** {d['evidence']}\n- **Alternativas:** {d['alternatives']}\n- **Recomendación:** {d['recommendation']}\n- **Filas afectadas:** {d['affected_rows']}\n- **Unidades afectadas:** {d['units']}\n- **Origen:** {d['group']}\n\n")
    for d in sorted([d for d in DEC if d["status"] == "blocking"], key=lambda d: d["id"]): block(d)
    o.write("## 4. Decisiones transversales (afectan a varias unidades)\n\nDecidirlas primero: desbloquean la mayor parte de las propuestas por unidad.\n\n")
    for d in [d for d in DEC if d["group"] == "transversal"]: block(d)
    o.write("Alias: " + ", ".join(f"`{k}` = `{v}`" for k, v in ALIASES.items()) + ".\n\n")
    o.write("## 5. Propuestas por unidad\n\n")
    def unitkey(d):
        m = re.match(r'Q-(T-)?([A-Z]+)-(\d+)', d["id"]); return ((m.group(1) or "") + m.group(2), int(m.group(3))) if m else (d["id"], 0)
    for d in sorted([d for d in DEC if d["group"] != "transversal" and d["status"] == "proposed"], key=unitkey): block(d)

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
            for c in row_deps.get(r["id"], ()):
                sid = row_spec.get(c)
                if sid and sid != s["id"]: deps.setdefault(sid, set()).add(c)
    s["deps"] = deps
cycles = row_cycles
def specfile(s): return f"docs/superpowers/plans/units/{s['seq']:02d}-{s['id'].lower()}.md"
def kebab(txt): return re.sub(r'[^a-z0-9]+', '-', txt.lower()).strip('-')[:50]

ORACLE_BY_UNIT = {
 "PATH": "jars oficiales pekko-actor 1.7.1 + JDK (jshell); ya existe tools/oracle/path-address.jsh", "UTIL": "jars oficiales pekko-actor + scala-library 1.7.1; ya existe tools/oracle/util-core.jsh",
 "CFG": "config-1.4.9.jar (Typesafe Config, no es código de Pekko); existe tools/oracle/config-hocon.jsh",
}
def oracle_for(s):
    u = s["unit"][1]
    if u in ORACLE_BY_UNIT: return ORACLE_BY_UNIT[u]
    if s["unit"][0] == "AT" and u == "TK": return "jars pekko-actor-typed; el jar `pekko-actor-testkit-typed` NO está en tools/oracle/fetch-jars.sh: agregarlo con su SHA-1 es parte de la primera tarea"
    return "valores puros: jars oficiales 1.7.1 vía `ORACLE=jars tools/oracle/run.sh`; comportamiento con actores: Pekko compilado desde upstream/pekko vía `tools/oracle/sbt.sh` (procedencia: commit 9a54d12); donde ningún oráculo es ejecutable, la aserción es el cuerpo del spec de Pekko portado 1:1"

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
    L.append(f"**Estado:** {'bloqueada' if blocking else ('pendiente de decisiones propuestas' if proposed else 'lista para ejecutar')}. "
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
        L.append(f"### Tarea {n}: {first(t[0]['feature'], 90)}" + (f" (+{len(t)-1} filas)" if len(t) > 1 else "") + "\n")
        L.append(f"Filas: {ids}. Contratos y aserciones: `{t[0]['contract_doc']}` (bloque de cada fila; sección **Aserciones**).\n")
        L.append("1. **RED:** por cada aserción `A#` de las filas, escribir el test que la expresa con la interfaz pública y observar que falla por la razón correcta; guardar comando y salida del fallo en la descripción del PR.")
        L.append("2. **GREEN:** el mínimo código de producción que lo hace pasar; guardar la salida verde.")
        L.append("3. **REFACTOR:** con los tests en verde; sin cambiar comportamiento.")
        L.append("4. Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de `docs/upstream.md`.\n")
    L.append("## Trazabilidad contrato → aserción → validación\n")
    L.append("| Fila | Comportamiento | Equivalente Rust propuesto | Validación (fuente) | Decisiones |\n|---|---|---|---|---|")
    for t in s["tasks"]:
        for r in t:
            val = first(r["tests"], 90) if r["tests"].strip() not in ("", "-") else "derivada del código (ver bloque de contrato)"
            L.append(f"| {r['id']} | {first(r['feature'], 80)} | {first(r['rust_equivalent'], 90)} | {val} | {', '.join(sorted(row_decisions[r['id']])) or '-'} |".replace("\n", " "))
    L.append("")
    L.append("## Gates y criterio de cierre\n")
    L.append(GATES + "\n")
    L.append("La spec se cierra únicamente cuando: (1) todos los gates pasan; (2) cada aserción `A#` de cada fila tiene un test que se observó fallar y luego pasar; (3) la cobertura de líneas nuevas o modificadas es >= 90 % y está documentada; (4) las columnas `impl` y `validation` del inventario reflejan lo demostrado (`partial` si queda una decisión abierta o una diferencia); (5) las diferencias observables respecto de Pekko están anotadas con su `Q-`. Tests verdes no equivalen a paridad.\n")
    nxt = order[s["seq"]] if s["seq"] < len(order) else None
    L.append(f"## Siguiente\n\n{('Spec %02d `%s` (`%s`).' % (nxt['seq'], nxt['id'], os.path.basename(specfile(nxt)))) if nxt else 'Última spec del plan.'}\n")
    open(specfile(s), "w", encoding="utf-8").write("\n".join(L))

# ------------------------------------------------------------------ order doc + map
with open("docs/superpowers/plans/units/README.md", "w", encoding="utf-8") as o:
    o.write("# Specs ejecutables (generadas desde el inventario)\n\nOrden de ejecución, dependencias resueltas y estado. Regenerar con `python3 -I tools/package/generate.py` (ver `docs/inventory/README.md`).\n\n| # | Spec | Filas | Tareas | Estado | Depende de |\n|---|---|---|---|---|---|\n")
    for s in order:
        nrows = sum(len(t) for t in s["tasks"])
        blocking = {q for t in s["tasks"] for r in t for q in row_decisions[r["id"]] if dec_by_id[q]["status"] == "blocking"}
        proposed = {q for t in s["tasks"] for r in t for q in row_decisions[r["id"]] if dec_by_id[q]["status"] == "proposed"}
        st = "bloqueada" if blocking else ("pendiente de decisiones" if proposed else "lista")
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
