# Reconciliación del piloto Classic con el inventario y el oráculo (issue #281)

Base: `feat/pekko-rust-pilot-migration` @ `c0f25c5`. Upstream: Apache Pekko 2.0.0 (`9fbd2283b1cf701c90a1223bf621942c191db61f`).
Alcance: sólo documentación e inventario. No se tocó código de producción, Typed, `owner-decisions.tsv` ni decisiones `Q-*`.
La matriz fila a fila es `p01-reconciliacion-matriz.tsv`, **generada** con `python3.12 tools/parity/reconcile_matrix.py --before <actor-classic.tsv anterior>`; no se edita a mano.

## 1. Métricas (verificadas contra el HEAD, no copiadas de la auditoría previa)

La línea base de la auditoría (51 / 109 / 365) coincide con el inventario sobre el TSV, pero mezcla dos cosas: de las 51 `done`, 4 son filas **excluidas** (`done/excluded`). Las filas en alcance son 521.

| Classic | Antes | Después |
|---|---|---|
| Filas totales / en alcance | 525 / 521 | 525 / 521 |
| `done` (en alcance, validadas) | 47 | 47 |
| `partial` | 109 | **113** |
| `pending` | 365 | **361** |
| Excluidas | 4 | 4 |
| Avance contractual (`done` / en alcance) | 9,0 % | 9,0 % |
| Parcialmente implementado (`partial` / en alcance) | 20,9 % | 21,7 % |
| Sin implementar (`pending` / en alcance) | 70,1 % | 69,3 % |
| Typed (260 filas) | 0 / 0 / 260 | 0 / 0 / 260 (no se tocó) |

Avance funcional (escenarios integrados que funcionan): el replay ejecuta 2810/2810 casos aplicables contra Pekko 2.0.0, de ellos **407** son escenarios de runtime Classic (2a: 125, 2b: 77, 3a: 88, 3b1–3b7: 117); 0 divergencias. No son comparables con el avance contractual: 7 filas del runtime están `done` porque exigen **todas** sus aserciones `A#`, mientras que un escenario sólo prueba un subconjunto.

## 2. Filas que cambian de estado

Cuatro filas pasan de `pending/pending` a `partial/oracle`. En las cuatro el código existe y hay casos grabados del oráculo cuyo texto o efecto se compara byte a byte; en cada nota del inventario queda dicho qué aserciones siguen sin cubrir.

| Fila | Evidencia que la respalda | Lo que falta |
|---|---|---|
| AC-CELL-002.2 | `ctor-*` (3b6), `init-*` (3b7), `i01`–`i03`; test `classic_failures` | A1, A2 (Restart con reintentos), A4, A5 |
| AC-CELL-007.4 | `ctor-restart-*`, `postr-*` (3b6/3b7): `PostRestartException` y `postStop` de la instancia | A1, A2 |
| AC-SUP-013 | `dec-*` (3b6): un decider que lanza hace fallar al supervisor (sólo replay) | `handleChildTerminated`, `handleFailure`, `sender()` del decider |
| AC-SUP-017 | textos de `ActorInitializationException` y `PostRestartException`; `DeathPactException` (`d01`–`d03`) | path en el mensaje, payload, `ActorInterruptedException` |

Ninguna fila `done` se degradó: las 7 del runtime (AC-REF-009.3, AC-SUP-009, AC-DW-001/003/004/008/009) cubren todas sus aserciones `A#` y no tienen decisión abierta.

## 3. Contratos que siguen `pending` aunque haya código o casos

Se dejaron `pending` por no tener ninguna aserción del contrato verificada (los casos que los citan prueban otra fila):
AC-CELL-007.2 (`Recreation`), AC-CELL-015.2 (`Terminating`), AC-CELL-016 (`make_child`), AC-CELL-027, AC-CELL-031, AC-SUP-002, AC-SUP-003 (el enum de directivas existe; falta el nivel de log), AC-SUP-005, AC-SUP-011, AC-SUP-015.
Con código pero sin casos del oráculo: AC-REF-006 (`actor_of`), AC-REF-007.1/007.2 (`Props::new` es un subconjunto), EVT (`EventStream` con dos canales: `DeadLetter` y `UnhandledMessage`).
Criterio aplicado: no pasar a `partial` una fila porque un caso la cite; sólo si algo del contrato de **esa** fila se compara.

## 4. Matriz (resumen; detalle en el TSV)

Filas del runtime (REF, CELL, SUP, DW, MBX, DSP, SYS, EXT, EVT): 210. 7 `done`, 53 `partial`, 150 `pending`. 70 filas citan casos del oráculo; 140 no. El TSV vincula cada una con la fuente Pekko (`path:line`), el módulo Rust (granularidad de archivo), los tests Rust que nombran sus casos, el replay `recorded_pekko.rs` y el cambio de estado.

Trazabilidad por familia de escenario → test Rust: 2b → `classic_tell`; 3a → `classic_hierarchy`; 3b1 → `classic_failures`; 3b2 → `classic_supervision`; 3b3 → `classic_escalate`; 3b4 → `classic_suspension`; 3b5 → `classic_all_for_one`. **2a, 3b6 y 3b7 no tienen test Rust que nombre sus casos**: sólo los ejerce el replay (`classic_api` cubre contratos de la API pública que los escenarios no alcanzan, no los casos 2a).

## 5. Divergencias y límites del oráculo

Divergencias observables nuevas: **ninguna** (replay sin divergencias, 0 `NA:Q-CFG-30`, 1 caso no aplicable de `cases/not-applicable.tsv`). No se corrigió nada.
Hallazgos de la reconciliación, para issues posteriores:
- `ActorSystem` usa un `Settings` propio (`parallelism`, `throughput`) que **no** está conectado a `config::Settings`/`Config`; sólo `Deployer` consume `Config`. La integración con Config del runtime no existe.
- `rust_equivalent` de varias filas propone tipos (`ActorInitializationError`, `Directive` con `LogLevel`) que el código no tiene: las excepciones son valores con nombre de clase y texto. La columna es una propuesta, no el estado.
- `rust/app-cases/replay-report.md` (generado) dice 2303/2304 y una ruta local ajena; el replay vigente es 2810/2811. No se regeneró para no versionar rutas locales.
- Límites del oráculo (de las notas del piloto): no se comparan intercalados entre emisores; no se afirma nada entre `terminate` y `await-system-terminated`; la reutilización de un nombre tras `stop` es una carrera en Pekko y sólo se afirma con barrera; `throughput` sólo con un dispatcher de un hilo; sin `null` (un `&str` o un valor Rust no puede serlo); no hay casos de mailbox acotado, prioridad, stash, `become`, `Identify`, `ActorSelection`, scheduler ni logging.

## 6. Decisiones del Owner que condicionan filas del runtime

Según `docs/decisions/register.tsv`: Q-DSP-1..7, Q-MBX-1..7, Q-CELL-1..5, Q-DW-1/2, Q-REF-1/5/6, Q-SUP-1/2 y Q-SYS-1..6 están `proposed`; Q-X-1/2/4/5 `accepted_with_changes`; Q-X-3/7/8 y Q-EVT-1 `accepted`. Las notas del inventario siguen diciendo "Q abierta" también para algunas ya aceptadas; no se editaron (no se reinterpreta una decisión). Ver `docs/decisions.md`.

## 7. P13–P26: qué existe ya

| Unidad | Filas | done | partial | pending | Lectura |
|---|---|---|---|---|---|
| P13 MBX-COLAS | 12 | 0 | 0 | 12 | ausente (el piloto sólo tiene la cola no acotada de la celda) |
| P14 REF-BASE | 15 | 0 | 6 | 9 | `ActorRef`, `tell`, `forward`, `sender` existen; falta `compareTo`, `toString`, `Props` |
| P15 MBX-NUCLEO | 7 | 0 | 4 | 3 | cola de la celda y turnos de `throughput` validados |
| P16 DSP-EJECUTORES | 19 | 0 | 0 | 19 | ausente (un solo pool fijo) |
| P17 REF-MUERTOS | 4 | 0 | 3 | 1 | dead letters validadas |
| P18 CELL-HIJOS | 6 | 1 | 4 | 1 | hijos y nombres validados |
| P19 CELL-EJECUCION | 11 | 0 | 3 | 8 | invoke básico; sin `become`, `Identify`, `receiveTimeout` |
| P20 SUP-ESTRATEGIAS | 10 | 0 | 6 | 4 | `OneForOne`/`AllForOne` validadas |
| P21 SUP-FALLO | 11 | 1 | 5 | 5 | Resume/Restart/Stop/Escalate validadas |
| P22 CELL-REINICIO-PARADA | 21 | 2 | 10 | 9 | reinicio y parada validados en gran parte |
| P23 DW-VIGILANCIA | 13 | 3 | 5 | 5 | DeathWatch en gran parte |
| P24 MBX-STASH | 5 | 0 | 0 | 5 | ausente |
| P25 SYS-GUARDIANES | 12 | 0 | 2 | 10 | guardianes simples, sin Config |
| P26 SYS-PROVEEDOR | 8 | 0 | 3 | 5 | nombres y `actorOf` de sistema |
| **Total** | 154 | 7 | 51 | 96 | 58 filas (38 %) con código o evidencia; 96 ausentes |

## 8. Gates ejecutados (sobre este árbol, antes de commitear)

| Gate | Resultado |
|---|---|
| `cargo build --workspace --all-targets` | OK |
| `cargo test --workspace` | OK (425 unitarios en la lib, más integración; 0 fallos) |
| `python3.12 tools/parity/replay.py` | 2810/2810 aplicables + 1 no aplicable, 0 divergencias |
| `cargo clippy --workspace --all-targets -- -D warnings` | OK |
| `cargo fmt --all --check` | OK |
| `cargo deny check licenses` | OK |
| `tools/parity/coverage.sh` (llvm-cov, `rakka-actor`) | 98,41 % de líneas (umbral 95 %) |
| `tools/package/generate.py` + `check.py` | regenerado; `citation_problems=0 failures=0` (con `upstream/pekko` enlazado al submódulo del checkout principal: el worktree no lo trae) |
| `check_divergence.py --selftest` | exit 1, como se exige (0/1 detectado: el mutante equivalente no se detecta) |
| `python3.12 -I -m unittest discover -s tools/package/tests` | 35 tests OK |
| Mutantes completos y fuzzers diferenciales | **no ejecutados**: no cambió código de producción; sin `cargo-mutants` ni mutantes nuevos que controlar |

## 9. Recomendación para #282–#289

1. No reescribir el runtime: conservar `cell.rs`, `children.rs`, `supervision.rs`, `system.rs` y el arnés del oráculo. Cada slice debe **ampliar** y registrar RED→GREEN sobre lo existente.
2. Primero conectar `ActorSystem::Settings` con `Config` (P25/P26): condiciona dispatcher, mailbox y guardianes.
3. Antes de P13/P16, resolver Q-MBX-* y Q-DSP-* (siguen `proposed`): son los que más filas bloquean.
4. Cubrir con test Rust propio los escenarios 2a, 3b6 y 3b7 que hoy sólo ejerce el replay.
5. Refactor previsible: `Actor::receive -> bool` y `Props::new` son subconjuntos de la API de Pekko; cualquier cambio de API pública debe ir con decisión del Owner.
