# AGENTS.md — rakka

Port a Rust de los módulos de actores de Apache Pekko 2.0.0: `rakka-actor` (Classic, antes `pekko-actor`) y `rakka-actor-typed` (Typed, antes `pekko-actor-typed`). Apache Pekko 2.0.0 es la fuente de verdad (Q-X-11); goPekko (`/home/pablog/workspace/getsyntegrity/gopekko`) sólo aporta aprendizajes.

## Estructura

- `crates/rakka-actor/` — Classic: `src/actor` (paths, direcciones, runtime: sistema, celdas, hijos, supervisión), `src/dispatch`, `src/event`, `src/util`; `examples/classic-parity/` es el arnés CLI que imprime lo mismo que el oráculo (`path-from-string`, `scenario`, `path-address-case`, `util-core-case`); `tests/recorded_pekko.rs` repite en proceso todos los casos grabados.
- `oracle/` — oráculo oficial (Q-X-12): CLI Java contra Pekko 2.0.0 en JDK 17 (`oracle/run.sh <comando>`, se construye con `mvn -f oracle/pom.xml package`).
- `cases/*.jsonl` — entradas de los casos; `rust/app-cases/` — grabaciones de Pekko (`rust-anything app record/replay`, no se editan a mano); `cases/not-applicable.tsv` — entradas fuera del dominio `&str` (Q-X-17), con motivo y decisión; `cases/oracle-ids/` — todos los ids del oráculo por familia; `cases/oracle-case-ids.tsv` — caso → id.
- `tools/parity/` — `replay.py` (replay con NOT_APPLICABLE: informa "N/N aplicables + K no aplicables"), `check_divergence.py` (mutantes; `--selftest` debe salir con 1), `fuzz_*.py` (fuzzers diferenciales JVM vs Rust), `make_*.py` (generadores de casos), `coverage.sh`, `reproduce-all.sh`. `tools/gates/` — runner de gates por niveles (política Q-X-20, ver "Reglas de calidad"); tests: `python3.12 -I -m unittest discover -s tools/gates`.
- `rust/divergence-reports/` y `docs/evidence/pilot/` — evidencia del piloto (reportes por mutante, `INVENTORY.md`, `REPORT-3b.md`, `MIGRATION-README.md`).
- `upstream/pekko/` — submódulo fijado en v2.0.0 (`9fbd2283…`; Scala 2.13.18, Config 1.4.9, JDK mínima 17). **Sólo lectura.** Ver `docs/upstream.md`. Las citas `path:line` del inventario apuntan a 2.0.0; `docs/inventory/pekko-2.0.0-reanchor.md` resume el reanclaje desde 1.7.1.
- `crates/rakka-actor-typed/` — Typed (vacío hasta el slice correspondiente).
- `crates/rakka-actor/tests/oracle/config-hocon.golden` — golden de HOCON grabado con el CLI del oráculo (`oracle/run.sh config-hocon-all`, Q-CFG-4), con los casos de contrato y una línea `KEY` por clave de los `reference.conf` de 2.0.0 (`docs/evidence/config-hocon-golden.md`); P07 lo convierte a casos `rust-anything`. Los de PATH y UTIL ya son casos grabados.
- `tools/oracle/` (jshell, jars 1.7.1) se eliminó (Q-X-12); su historia está en git.
- `docs/inventory/` — inventario de alcance (`actor-classic.tsv`, `actor-typed.tsv` + resúmenes `.md`) y `contracts/` (contrato, fuente `path:line`, aserciones y oráculo de cada fila). Es la definición de "100%". Convenciones en `docs/inventory/README.md`.
- `docs/decisions.md` (+ `docs/decisions/register.tsv`) — registro único de decisiones del Owner: autorizadas, propuestas y bloqueantes.
- `docs/implementation-order.md` — orden de ejecución; `docs/superpowers/plans/units/` — 49 specs preliminares generadas (agrupaciones de filas, máximo 5 tareas cada una; requieren revisión de atomicidad y dependencias antes de considerarse ejecutables); `docs/contract-map.tsv` — fila → spec → validación.
- `tools/package/generate.py` y `check.py` — regeneran y verifican el paquete documental desde los TSV (no editar a mano los archivos generados).
- `docs/superpowers/plans/*.md` — planes detallados previos (PATH, UTIL, detalle HOCON de CFG).
- `docs/research/` — evaluaciones de herramientas (REA).
- `docs/testing-policy.md` — política de testing (única fuente normativa; ver "Capas de test").
- `skills/` — skills compartidas, agnósticas del agente (ver "Índice de skills" y `skills/README.md`). `.claude/skills/testing/` es sólo un adaptador mínimo para Claude Code que remite a `skills/testing/SKILL.md`.

## Comandos

| Qué | Comando |
|---|---|
| Build | `cargo build --workspace` |
| Tests | `cargo test --workspace` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Formato | `cargo fmt --all --check` |
| Licencias | `cargo deny check licenses` (`deny.toml`: solo Apache-2.0, MIT, BSD-2/3, ISC, Unicode-3.0, Zlib) |
| Oráculo | `oracle/run.sh <comando>` (JDK 17) + `rust-anything app record/replay` (Q-X-12); replay: `python3.12 tools/parity/replay.py` |
| Paquete documental | `uv run tools/package/generate.py` y `uv run tools/package/check.py` (`.python-version` fija 3.12; el `python3` del sistema, 3.9, no sirve: f-strings anidados). Sin uv: `python3.12 tools/package/check.py`. Tests: `uv run python -m unittest discover -s tools/package/tests`. Replay: `uv run tools/parity/replay.py` |

No usar `-race`/sanitizers ni correr workbench salvo pedido explícito.

## Ramas y PR

- `feat/pekko-rust-pilot-migration` es la rama de integración y la rama por defecto del repositorio. Las ramas de trabajo salen de `origin/feat/pekko-rust-pilot-migration` (nunca de otra rama de trabajo) y sus PR apuntan a ella.
- `main` y `develop` están discontinuadas: no se ramifica desde ellas ni se apunta ningún PR a ellas. Quedan como referencia histórica hasta que el Owner decida archivarlas.
- No hay push directo a `feat/pekko-rust-pilot-migration` salvo pedido explícito del Owner, ni force push, ni borrado de ramas sin autorización. Los merges los decide el Owner, salvo el merge de PR de `slice/*`, `fix/*` o `chore/*` contra la rama de integración con todas las verificaciones locales en verde, que está autorizado (ver "Reglas para agentes y subagentes"); los PR que el Owner reserve (p. ej. los que modifican este archivo) los mergea él.
- Antes de ramificar: verificar `git status`, preservar los cambios existentes y actualizar las referencias remotas (`git fetch --all --prune`).

## Flujo por unidad

1. Extraer el contrato desde la fuente y los specs de Pekko (`path:line`).
2. Escribir escenarios con aserciones concretas (en el plan de la unidad), con trazabilidad contrato → aserción.
3. Implementar en Rust idiomático con TDD estricto (ver "Reglas de calidad").
4. Validar: specs de Pekko portados 1:1 y, cuando aplique, casos `rust-anything` contra Pekko 2.0.0 (Q-X-12).
5. Actualizar `impl`/`validation` en el inventario.

Ninguna diferencia semántica con Pekko se acepta sin decisión explícita del Owner: se anota con `Q:` en la columna `notes` del inventario. Una pregunta abierta impide cerrar el comportamiento afectado: la fila queda `partial`, no `done`. Compilar, correr una demo o tener tests verdes no es paridad.

## Reglas de calidad

- **TDD estricto por comportamiento: RED → GREEN → REFACTOR.** Cada comportamiento empieza con un test que falla por la razón correcta; se guarda la evidencia real (comando y salida del fallo) antes de escribir el código de producción, después la del GREEN. No se inventa evidencia ni se declara RED sin haberlo observado.
- **Gates por slice (Q-X-14):** replay completo contra Pekko 2.0.0, tests, `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo deny check licenses`, cobertura de producción **≥95 % por crate** con `cargo llvm-cov` (sin exclusiones; sustituye al 90 % de líneas nuevas anterior) y mutantes nuevos en cada slice. En cada hito (supervisión, Classic, Typed) o refactor grande: regresión completa de mutantes y fuzzers. Comando, resultado y líneas no cubiertas (con motivo) se documentan en el PR.
- **Nombres que expresan propósito.** Variables, parámetros, campos, funciones, tipos, módulos, archivos y tests dicen qué hacen. Se evitan nombres genéricos (`data`, `tmp`, `helper`, `util2`), abreviaturas ambiguas y los identificadores de specs (`AC-CFG-001`, `T3`) como sustituto del significado; el identificador de la fila va en comentarios o docs, no en el nombre.

- **Niveles de gates FAST / PR / MILESTONE (política aprobada por el Owner, Q-X-20, 2026-11-21; efectiva desde la integración de #308 y #310. Los gates por slice de arriba siguen vigentes: el nivel PR los contiene todos, y la regresión de mutantes de cada hito es la del nivel MILESTONE con presupuesto).** Runner: `python3.12 tools/gates/gates.py fast|pr|milestone` (`--list`, `--base REF`, `--no-reuse`).
  - **FAST** (bucle interno, no sirve para PR ni merge): `cargo fmt --check`, `tools/package/check.py`, tests de lib.
  - **PR** (equivale a los gates por slice vigentes): FAST + clippy, `cargo test --workspace`, `cargo deny`, cobertura ≥95 % por crate, replay y mutantes. Los mutantes se seleccionan por archivo cambiado respecto de `--base`; un cambio en el corpus (`cases/`, `rust/app-cases/`), `oracle/`, `Cargo.*`, `examples/` o el arnés `check_divergence.py`/`replay.py` selecciona todos, salvo lo que demuestren las reglas Q-X-22. Los mutantes nuevos del slice siguen siendo obligatorios.
  - **Reglas Q-X-22 (Owner, 2026-11-26; aceptada con cambios; sólo nivel PR).** Ante cualquier duda rige la selección anterior. **R1** (`tools/gates/pr_selection_corpus.py`): casos nuevos no seleccionan todos los mutantes sólo si se demuestra que grabaciones, entradas, goldens y `not-applicable.tsv` existentes siguen intactos, que el arnés Rust es idéntico ítem a ítem salvo escenarios nuevos autocontenidos y que en `check_divergence.py` sólo cambiaron las tablas de mutantes; la aditividad textual no basta. **R2** (`pr_selection.py`): un archivo que sólo añade `pub mod x;` no selecciona mutantes si el módulo nuevo no puede alcanzar código existente (impls sobre tipos ajenos, macros o atributos desconocidos). **R3** (`pr_mutants.py`): sólo se reutiliza `DETECTED` por mutante con identidad vigente, incluidas las herramientas externas (`rust-anything` incluido); nunca `TIMEOUT`, `ERROR`, sobreviviente ni corrida incompleta; los mutantes de herramientas no se reutilizan; el control corre siempre. R4 (sustituir tests por cobertura) no está aceptada. Las reglas aplicadas y los mutantes no ejecutados quedan en `.gates/selection/` y `.gates/mutants/pr/runs/` (no se podan; conservar esos logs antes de borrar un worktree) y el PR los cita. Cobertura ≥95 %, cero divergencias no autorizadas, control limpio y garantías de MILESTONE no cambian.
  - **MILESTONE** (hitos y refactors grandes): PR + mutantes con presupuesto y reanudables (`tools/gates/mutant_runner.py`) + los cuatro fuzzers (3 semillas). Costo medido de un mutante con el arnés anterior: 7 s de build y 132 s de replay completo (~705 casos, sin corte temprano), unas 7,5 h para 206. El runner mata primero con los casos que ya mataron al mutante (reportes en `rust/divergence-reports/`; ~8 s) y sólo cae al replay completo si ninguno diverge. Orden: archivos cambiados, vecinos por `use crate::`/`mod`, resto. Tope por defecto 40 mutantes y 30 min (`--max-mutants`, `--max-minutes`); superarlo exige `--authorize-full-mutants` (autorización explícita del Owner). Si quedan mutantes sin resultado fresco el gate termina con código 3 (no es un pass) y se reanuda con el mismo comando. Cada resultado se guarda por mutante con clave = parche + huella conservadora de todo el crate (cada archivo de `crates/rakka-actor` salvo `target/`, más `Cargo.toml` y `Cargo.lock` de la raíz) + huella del arnés (casos, grabaciones, el propio runner, lógica de `check_divergence.py`) + configuración + `rustc -V`; sólo `DETECTED` se reutiliza. La clave se vuelve a comprobar antes de aceptar un `DETECTED` guardado y antes y después de cada mutante, y el control verificado debe seguir correspondiendo al estado del veredicto; si no, `INPUTS_CHANGED` (error, nunca PASS). Cada corrida escribe sus resultados y logs en su propio directorio. El control (crate sin mutar) corre una vez por estado, no por mutante. Veredictos (Q-X-20, aceptada por el Owner el 2026-11-21): PASS sólo con todos los controles completos; INCOMPLETE con mutantes pendientes; FAIL ante un sobreviviente (salvo clasificación justificada y revisada); ERROR ante fallo de compilación, arnés o parche, incluido un patrón obsoleto, nunca DETECTED; CANCELLED conserva los checkpoints válidos. El fast-kill sólo declara DETECTED si el build sin mutar coincide con la salida grabada de Pekko en un caso conocido, el mutante diverge igual en dos ejecuciones y la propia herramienta (`rust-anything app replay --case`, validada) falla ese caso como divergencia aplicable (no NOT_APPLICABLE ni NA:Q-CFG-30), es decir, lo que el replay completo clasificaría igual; si no se puede demostrar, corre el replay completo. Cada informe por mutante empieza con `Verdict path:` (fast-kill o replay completo) y el resumen cuenta ambos caminos. Los replays de un checkout se serializan con un bloqueo cuya espera consume el presupuesto de la corrida (si se agota: pendiente/INCOMPLETE), cada corrida clasifica con una copia inmutable del binario, protegida por un lease entre procesos desde su publicación hasta su último uso (la poda sólo limita a 20 las copias sin uso), y lo que deja un proceso muerto con SIGKILL se recupera en la ejecución siguiente. Consecuencia: cualquier cambio en el crate invalida todos los resultados guardados (seguro, pero la reanudación sólo ahorra trabajo dentro de un mismo estado del crate); el grafo de imports se usa únicamente para priorizar el orden de ejecución.
  - **Reutilización de evidencia por SHA.** (El gate de mutantes del nivel PR no se reutiliza entero: aplica R3 y corre siempre.) Un gate no se repite sólo si coinciden los ids de árbol git de todas las rutas que lee (`HEAD:<ruta>`), el comando exacto y `rustc -V`, y esas rutas no tienen cambios sin commitear ni archivos sin seguimiento. Cualquier otro caso re-ejecuta. La evidencia vive en `.gates/` (ignorado por git); el PR cita comando, resultado y SHA, no la evidencia reutilizada de otra rama sin repetir el SHA de árbol.
  - **Logs fuera del contexto del LLM.** Cada gate escribe su salida completa en `.gates/logs/<sha>/<gate>.log`; la consola recibe una línea por gate y, si falla, las últimas 20 líneas (≤2000 caracteres). Está prohibido pegar o leer logs completos en el contexto del modelo: usar `grep`/`tail` sobre el archivo.
  - Ningún gate se desactiva en esta política: PR contiene todos los vigentes; lo que cambia es el orden, la reutilización segura y la selección de mutantes en PR.
- **Compatibilidad semver (AC-UTIL-021, Owner 2026-11-18).** Los filtros MiMa de Pekko no se excluyen: su equivalente es `cargo semver-checks`, que entra en los gates **a partir del primer release** (hasta entonces no hay versión publicada contra la que comparar).
- **Procedencia (Q-X-18).** Está **prohibido leer o portar código de OpenJDK u otras fuentes GPL/LGPL o sin licencia**: la conducta del JDK se toma sólo de su documentación pública (Javadoc de Java SE), de los RFC y estándares, y del oráculo (casos grabados). Todo archivo de `crates/` lleva un encabezado `Derived from Apache Pekko 2.0.0 <paths> (Apache-2.0)` o `Clean-room implementation from docs/legal/cleanroom/<archivo>-spec.md (Apache-2.0)`; `check.py` falla si falta, o si algún archivo de `crates/`, `tools/` u `oracle/` menciona OpenJDK, GPL o Classpath exception (sólo `docs/legal/` puede). Lo derivado de fuentes dudosas se reescribe en sala limpia (`docs/legal/provenance-audit.md`, `docs/legal/cleanroom/`). No se publica ningún release hasta cerrar las reescrituras pendientes.

## Índice de skills

Las skills son archivos del repositorio y se leen directamente por ruta desde la raíz; no se presupone soporte nativo de skills ni descubrimiento automático. Estructura y acceso por herramienta: [`skills/README.md`](skills/README.md).

| Acción | Lectura obligatoria |
|---|---|
| Crear, modificar, portar, clasificar o revisar pruebas | [`skills/testing/SKILL.md`](skills/testing/SKILL.md) y [`docs/testing-policy.md`](docs/testing-policy.md) |

## Capas de test

La lectura obligatoria está en el "Índice de skills". Reglas esenciales (el detalle, los criterios de elección y los ejemplos están en [`docs/testing-policy.md`](docs/testing-policy.md)):

- **Dos capas, definidas por alcance y no por carpeta.** Unitario: un comportamiento aislado y atómico de una función, componente o transición de estado; sin ActorSystem ni runtime de actores, sin red/archivos reales/entorno/servicios, sin sleeps, con tiempo y azar controlados, y sin reproducir el algoritmo de producción para calcular lo esperado. Integración: un escenario contractual completo con los componentes reales involucrados; toda interacción entre actores reales es integración aunque sea en memoria. Probar un componente real de forma aislada (mailbox, decisión de supervisión) sigue siendo unitario; la integración aparece al conectar componentes y validar el proceso. Los dobles sólo aíslan colaboradores ajenos al escenario y la frontera sustituida se declara.
- **Ubicación.** Unitarios junto al componente; integración de un crate en su `tests/`; integración transversal Classic/Typed o multi-crate en el crate interno `rakka-testint` (**previsto, aún no existe**; no publicable y distinto del testkit público). No crear ese crate ni tocar `Cargo.toml` sin tarea explícita.
- **TDD y cobertura se mantienen en ambas capas:** RED real antes del código, GREEN, refactor; cobertura mínima según "Reglas de calidad" (≥95 % por crate, Q-X-14), que puede sumar ambas capas (no hay un porcentaje obligatorio por capa) pero no sustituye los unitarios necesarios, ni demuestra paridad, ni permite omitir escenarios contractuales.
- **Por unidad:** identificar aserciones unitarias y de integración; TDD unitario; agregar y ejecutar la integración en cuanto existan los componentes reales y antes de cerrar la unidad (no acumularla al final); verificar contrato → escenario → prueba → evidencia. Un doble no acredita la integración con el componente sustituido: mientras falte esa integración necesaria, la fila permanece `partial` y se identifica el componente real y la prueba pendiente.
- **Paridad.** Se conservan las aserciones observables de los specs de Pekko; un spec que arranca actores reales es integración. Los diferenciales contra el jar oficial complementan ambas capas y no reemplazan aserciones. Tests verdes y cobertura no prueban paridad.

## Reglas

- Push y PR: autorizados (Owner, 2026-11-19) para ramas `slice/*`, `fix/*` y `chore/*`, con PR contra `feat/pekko-rust-pilot-migration`. Nunca push a `main` ni a `develop`, ni directo a `feat/pekko-rust-pilot-migration`; sin force push. Cualquier otra rama o destino requiere autorización del Owner.
- Las decisiones `Q-*` siguen siendo del Owner (ver "Reglas para agentes y subagentes").
- Cada archivo Rust que traduzca un archivo de Pekko lleva el encabezado de atribución de `docs/upstream.md`.

## Reglas para agentes y subagentes (Owner, 2026-11-19)

Valen para toda sesión que trabaje en el repositorio, y la instrucción de cada subagente debe incluirlas.

- **Decisiones del Owner.** No aceptar ni responder ninguna decisión `Q-*` (tampoco las ya respondidas). Toda decisión nueva se anota como `Q-<ÁREA>-n` en `docs/owner-batch-02.md` y en su `*.decisions.tsv` con estado `proposed`. `docs/decisions/owner-decisions.tsv` solo lo edita el Owner.
- **Ramas.** No borrar ninguna rama, ni local ni remota, ni siquiera las `slice/*` ya mergeadas.
- **Bloqueos.** Si el clasificador de permisos o este archivo bloquean una acción, no buscarla por otro camino ni con una variante más estrecha: parar e informar al Owner.
- **Push y PR.** Si un `git push` o un `gh pr create` se bloquea o se cuelga, parar e informar; no se busca otra vía ni se reintenta con otro comando.
- **Merge.** Solo dentro del flujo autorizado (rama propia, PR a `feat/pekko-rust-pilot-migration`) y solo si todas las verificaciones locales salen verdes (el repositorio no tiene CI): replay, tests, clippy, fmt, `cargo deny`, cobertura por crate, mutantes con control y `tools/package/check.py`. Si alguna falla, no se mergea.
- **Verificación independiente.** El esperado de cada caso grabado debe venir del oráculo (Pekko 2.0.0 en la JVM), nunca de la implementación Rust; el Owner puede volver a ejecutarlo contra el oráculo y comparar byte a byte.
