# Plan final de unidades

Sustituye a las 50 specs preliminares de `docs/superpowers/plans/units/` (no se editan; este documento no es generado). Cubre las 783 filas de `docs/contract-map.tsv`, cada una en exactamente una unidad. Los ciclos que obligaban a los saltos hacia adelante y su resolucion estan en `docs/cycles.md`. **Nada de lo aqui descrito esta implementado**: el estado `impl`/`validation` de los TSV del inventario es la unica medida de avance; las filas ya `done`/`partial` (AC-UTIL y AC-PATH) conservan su estado y no se replanifican como pendientes.

## Plantilla comun (aplica a todas las unidades; no se repite)

- TDD estricto por comportamiento: RED -> GREEN -> REFACTOR con comando y salida reales guardados en el PR; cobertura `cargo llvm-cov` >= 90 % de las lineas de produccion nuevas o modificadas (comando, resultado y lineas no cubiertas con motivo); gates `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- Todo archivo Rust que traduce un archivo de Pekko lleva la atribucion de `docs/upstream.md`. Nombres que expresan proposito; los IDs de fila van en comentarios, no en nombres.
- Una fila con decision abierta queda `partial` (nunca `done`) y se anota `Q:`. Validacion: specs de Pekko portados 1:1 y, cuando aplica, golden del oraculo (`tools/oracle/`). Un doble de prueba de una interfaz temporal NO cierra la fila que depende de el: esa fila queda `partial` hasta la prueba de integracion indicada en `docs/cycles.md`.
- Tamano: ~400 lineas de diff por PR. La estimacion de cada unidad es orientativa (~40 lineas de produccion+tests por fila, calculada, no medida); las unidades que superan 400 se parten en PR secuenciales (se indica el numero). Cada PR apunta a `develop`; el merge lo decide el Owner.
- "Decisiones": se listan solo las bloqueantes o rechazadas del registro (`docs/decisions/register.tsv`, `owner-decisions.tsv`) que tocan filas de la unidad, con su estado. Estado global al leer: 1 rechazada (Q-UTIL-1, vigente la alternativa del Owner), 8 bloqueantes sin respuesta, 202 propuestas sin respuesta. Las propuestas de cada fila estan en `docs/contract-map.tsv` (columna `decisions`); regla: la fila avanza en lo independiente y queda `partial`. `Q-X-8` (Typed sobre Classic) esta propuesta, no aceptada: toda la Ola 7-9 asume (a) y se rehace si el Owner elige (b).

## Resumen por ola

| Ola | Unidades | Filas |
|---|---|---|
| Ola 0 - valores sin runtime | P02..P06 (5) | 65 |
| Ola 1 - configuracion, eventos y scheduler | P01..P12 (7) | 68 |
| Ola 2 - nucleo del runtime Classic | P13..P24 (12) | 132 |
| Ola 3 - sistema, proveedor, referencias y extensiones | P25..P30 (6) | 43 |
| Ola 4 - patrones, routers y logging | P31..P37 (7) | 81 |
| Ola 5 - apagado, serializacion e IO | P38..P43 (6) | 84 |
| Ola 6 - testkit Classic y superficie Java | P44..P47 (4) | 50 |
| Ola 7 - Typed: nucleo, adaptador y sistema | P48..P53 (6) | 105 |
| Ola 8 - Typed: contexto, supervision, receptionist y entrega fiable | P54..P60 (7) | 96 |
| Ola 9 - Typed: javadsl y testkits | P61..P64 (4) | 59 |

Total: 64 unidades, 783 filas.

## Ola 0 - valores sin runtime

| ID | Unidad | Filas | Depende de | Diff est. | PR |
|---|---|---|---|---|---|
| P02 | UTIL-NUCLEO | 8 | - | ~320 | 1 |
| P03 | UTIL-ESTRUCTURAS | 16 | P02, P05 | ~640 | 2 |
| P04 | UTIL-BYTES | 9 | P02, P05 | ~360 | 1 |
| P05 | UTIL-VARIOS | 26 | P02 | ~1040 | 3 |
| P06 | UTIL-PLATAFORMA | 6 | P05 | ~240 | 1 |

### P02 UTIL-NUCLEO

- **Resultado verificable:** Timeout, Done, NotUsed, excepciones base y Helpers puros cerrados contra los jars.
- **Filas (8; impl: partial 8):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-001, AC-UTIL-002.1, AC-UTIL-002.2, AC-UTIL-002.3, AC-UTIL-003.1, AC-UTIL-003.2, AC-UTIL-004.1, AC-UTIL-004.2.
- **Exclusiones/costuras:** Q-UTIL-2..5 y Q-UTIL-7 siguen abiertas: filas partial hasta decision; no se replanifica lo ya implementado.
- **Decisiones:** Q-UTIL-1 (rechazada: alternativa del Owner vigente).
- **Dependencias:** ninguna.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/util/{timeout,done,exceptions,helpers}.rs).
- **Tests/oraculo:** unidad + oraculo jars.
- **Diff:** ~320 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P03 UTIL-ESTRUCTURAS

- **Resultado verificable:** Estructuras de datos concurrentes/inmutables (Index, WildcardIndex, SubclassifiedIndex, colas, LFU/LRU).
- **Filas (16; impl: pending 16):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-007, AC-UTIL-008, AC-UTIL-009, AC-UTIL-010, AC-UTIL-011, AC-UTIL-012, AC-UTIL-013, AC-UTIL-014, AC-UTIL-015.1, AC-UTIL-015.2, AC-UTIL-016.4, AC-UTIL-017.1, AC-UTIL-017.2, AC-UTIL-017.3, AC-UTIL-017.4, AC-UTIL-017.5.
- **Exclusiones/costuras:** UTIL-008/009/012/013 se prueban solo con sus propios specs; el uso por EVT/CFG/MBX es de las unidades consumidoras.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P02, P05.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/util/{index,wildcard_index,queues,lfu}.rs).
- **Tests/oraculo:** specs Pekko 1:1 (Index/WildcardIndex/queues).
- **Diff:** ~640 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P04 UTIL-BYTES

- **Resultado verificable:** ByteString, ByteStringBuilder/Iterator y adaptadores de flujo.
- **Filas (9; impl: pending 9):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-006.1, AC-UTIL-006.2, AC-UTIL-006.3, AC-UTIL-006.4, AC-UTIL-006.5, AC-UTIL-006.8, AC-UTIL-006.7, AC-UTIL-006.6, AC-UTIL-022.
- **Exclusiones/costuras:** Q-UTIL-12 (bloqueante) fija el alcance: sin respuesta solo avanzan las filas que no dependen de la representacion.
- **Decisiones:** Q-UTIL-12 (bloqueante, sin respuesta).
- **Dependencias:** P02, P05.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/util/bytes/*.rs).
- **Tests/oraculo:** specs ByteStringSpec portados + oraculo jars.
- **Diff:** ~360 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P05 UTIL-VARIOS

- **Resultado verificable:** Version, PekkoVersion, conversores, reloj, utilidades varias.
- **Filas (26; impl: pending 26):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-004.3, AC-UTIL-005.1, AC-UTIL-005.2, AC-UTIL-016.1, AC-UTIL-016.2, AC-UTIL-016.3, AC-UTIL-016.5, AC-UTIL-016.6, AC-UTIL-016.7, AC-UTIL-016.8, AC-UTIL-016.9, AC-UTIL-016.10, AC-UTIL-016.11, AC-UTIL-016.12, AC-UTIL-016.13, AC-UTIL-016.14, AC-UTIL-016.15, AC-UTIL-016.16, AC-UTIL-016.17, AC-UTIL-016.18, AC-UTIL-018.1, AC-UTIL-018.2, AC-UTIL-018.3, AC-UTIL-018.4, AC-UTIL-018.5, AC-UTIL-019.
- **Exclusiones/costuras:** UTIL-016.9/.11 (ManifestInfo/JFR) se validan solos; CFG-007 los consume despues. Q-UTIL-6 (bloqueante) afecta UTIL-005.2.
- **Decisiones:** Q-UTIL-6 (bloqueante, sin respuesta).
- **Dependencias:** P02.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/util/*.rs).
- **Tests/oraculo:** unidad + oraculo jars.
- **Diff:** ~1040 lineas, 3 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P06 UTIL-PLATAFORMA

- **Resultado verificable:** Helpers de plataforma (makePattern, isWindows, timestamp, Unsafe).
- **Filas (6; impl: pending 6):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-020, AC-UTIL-021, AC-UTIL-023, AC-UTIL-024, AC-UTIL-025, AC-UTIL-026.
- **Exclusiones/costuras:** UTIL-020/021 son parte de Unsafe/MiMa: pueden quedar no aplicables solo con autorizacion del Owner (A-5); aqui se planifican.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P05.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/util/platform.rs).
- **Tests/oraculo:** unidad + oraculo jars.
- **Diff:** ~240 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 1 - configuracion, eventos y scheduler

| ID | Unidad | Filas | Depende de | Diff est. | PR |
|---|---|---|---|---|---|
| P01 | PATH | 16 | P02, P07 | ~640 | 2 |
| P07 | CFG-HOCON | 12 | P02, P05 | ~480 | 2 |
| P08 | CFG-AJUSTES | 10 | P02, P07 | ~400 | 1 |
| P09 | CFG-ENTORNO | 3 | P08 | ~120 | 1 |
| P10 | CFG-DEPLOY | 3 | P01, P03, P07 | ~120 | 1 |
| P11 | EVT-BUS | 11 | P02, P03, P08 | ~440 | 2 |
| P12 | SCH-RUEDA | 13 | P02, P06, P07, P09 | ~520 | 2 |

### P01 PATH

- **Resultado verificable:** Address/ActorPath completos (filas partial restantes) con los goldens de PATH.
- **Filas (16; impl: done 7, partial 8, pending 1):** asignacion explicita en `docs/unit-rows.tsv`: AC-PATH-001, AC-PATH-002, AC-PATH-003, AC-PATH-004, AC-PATH-005, AC-PATH-006, AC-PATH-007, AC-PATH-008, AC-PATH-009, AC-PATH-010, AC-PATH-011, AC-PATH-012, AC-PATH-013, AC-PATH-014, AC-PATH-015, AC-PATH-016.
- **Exclusiones/costuras:** Las 7 filas done no se tocan; PATH-015 (paths especiales) usa solo constantes, el proveedor llega en P26.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P02, P07.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/path/*.rs).
- **Tests/oraculo:** unidad + oraculo jars (golden existente).
- **Diff:** ~640 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P07 CFG-HOCON

- **Resultado verificable:** Cargador HOCON + semantica de duraciones/tamanos/literales + validacion contra reference.conf.
- **Filas (12; impl: pending 12):** asignacion explicita en `docs/unit-rows.tsv`: AC-UTIL-002.4, AC-UTIL-002.5, AC-CFG-001, AC-CFG-003, AC-CFG-017.1, AC-CFG-017.2, AC-CFG-017.3, AC-CFG-017.4, AC-CFG-017.5, AC-CFG-017.6, AC-CFG-017.7, AC-CFG-019.
- **Exclusiones/costuras:** Detalle en docs/superpowers/plans/2026-10-30-ac-cfg-parser-loading.md; Q-X-1 abierta => partial.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P02, P05.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/config/*.rs).
- **Tests/oraculo:** unidad + golden jars (Typesafe Config).
- **Diff:** ~480 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P08 CFG-AJUSTES

- **Resultado verificable:** ActorSystem.Settings fail-fast: todas las claves pekko.actor.* leidas y validadas.
- **Filas (10; impl: pending 10):** asignacion explicita en `docs/unit-rows.tsv`: AC-CFG-002, AC-CFG-004, AC-CFG-005, AC-CFG-006, AC-CFG-007, AC-CFG-009, AC-CFG-010, AC-CFG-011, AC-CFG-018, AC-CFG-020.
- **Exclusiones/costuras:** Costura SETTINGS-S1: struct Settings con los campos; los consumidores (EVT-011 niveles, EXT cargas, CS hooks, SER bindings) leen el campo, no lo implementan.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P02, P07.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/config/settings.rs).
- **Tests/oraculo:** unidad con configs sinteticas.
- **Diff:** ~400 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P09 CFG-ENTORNO

- **Resultado verificable:** ProviderSelection, Setup/BootstrapSetup y DynamicAccess (registro de fabricas).
- **Filas (3; impl: pending 3):** asignacion explicita en `docs/unit-rows.tsv`: AC-CFG-008, AC-CFG-015, AC-CFG-016.
- **Exclusiones/costuras:** Q-X-2 (registro de fabricas) abierta; CFG-008 queda partial hasta que el proveedor exista (P26).
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P08.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/config/{setup,dynamic_access}.rs).
- **Tests/oraculo:** unidad.
- **Diff:** ~120 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P10 CFG-DEPLOY

- **Resultado verificable:** Deployer: parseo pekko.actor.deployment, lookup con comodines, Deploy/Scope.
- **Filas (3; impl: pending 3):** asignacion explicita en `docs/unit-rows.tsv`: AC-CFG-012.1, AC-CFG-012.2, AC-CFG-013.
- **Exclusiones/costuras:** CFG-012.3/.4 (RouterConfig y resizers) se difieren a P37 (RT-CONFIG); aqui el campo router es un valor opaco (costura DEPLOY-S1).
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P01, P03, P07.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/{deploy,deployer}.rs).
- **Tests/oraculo:** specs DeployerSpec parciales + unidad.
- **Diff:** ~120 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P11 EVT-BUS

- **Resultado verificable:** EventBus, EventStream y tipos de log/dead letters/adaptadores de log sin logger real.
- **Filas (11; impl: pending 11):** asignacion explicita en `docs/unit-rows.tsv`: AC-EVT-001, AC-EVT-002, AC-EVT-003, AC-EVT-004, AC-EVT-006, AC-EVT-008, AC-EVT-012, AC-EVT-013.1, AC-EVT-013.2, AC-EVT-015, AC-EVT-016.
- **Exclusiones/costuras:** Costura EVT-S1: EventStream sin vigilancia (EVT-007 en P34) ni LoggingBus (EVT-011 en P34).
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P02, P03, P08.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/event/*.rs).
- **Tests/oraculo:** specs EventStreamSpec/LoggingSpec parciales.
- **Diff:** ~440 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P12 SCH-RUEDA

- **Resultado verificable:** Scheduler (rueda LARS) con reloj y ticks verificables.
- **Filas (13; impl: pending 13):** asignacion explicita en `docs/unit-rows.tsv`: AC-SCH-001, AC-SCH-002, AC-SCH-003, AC-SCH-004, AC-SCH-005, AC-SCH-006, AC-SCH-008.1, AC-SCH-008.2, AC-SCH-008.3, AC-SCH-009, AC-SCH-010, AC-SCH-014, AC-SCH-015.
- **Exclusiones/costuras:** SCH-007/011/012/013 (actores y timers) a P27; la cancelacion de tarea a actor terminado usa solo la interfaz ActorRefLike.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P02, P06, P07, P09.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/scheduler/*.rs).
- **Tests/oraculo:** LightArrayRevolverSchedulerSpec 1:1 con reloj manual.
- **Diff:** ~520 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 2 - nucleo del runtime Classic

| ID | Unidad | Filas | Depende de | Diff est. | PR |
|---|---|---|---|---|---|
| P13 | MBX-COLAS | 12 | P03, P08, P09 | ~480 | 2 |
| P14 | REF-BASE | 15 | P01, P05, P10, P13 | ~600 | 2 |
| P15 | MBX-NUCLEO | 7 | P08, P13, P14 | ~280 | 1 |
| P16 | DSP-EJECUTORES | 17 | P08, P09, P15 | ~680 | 2 |
| P17 | REF-MUERTOS | 4 | P11, P14, P15 | ~160 | 1 |
| P18 | CELL-HIJOS | 6 | P14 | ~240 | 1 |
| P19 | CELL-EJECUCION | 11 | P08, P11, P12, P15, P16, P17, P18, P20 | ~440 | 2 |
| P20 | SUP-ESTRATEGIAS | 10 | P11, P14 | ~400 | 1 |
| P21 | SUP-FALLO | 11 | P19, P20 | ~440 | 2 |
| P22 | CELL-REINICIO-PARADA | 21 | P17, P18, P21 | ~840 | 3 |
| P23 | DW-VIGILANCIA | 13 | P11, P17, P20, P22 | ~520 | 2 |
| P24 | MBX-STASH | 5 | P13, P22, P23 | ~200 | 1 |

### P13 MBX-COLAS

- **Resultado verificable:** Colas de mailbox (unbounded, bounded, prioridad, deque, control-aware) y vista de solo lectura.
- **Filas (12; impl: pending 12):** asignacion explicita en `docs/unit-rows.tsv`: AC-MBX-004, AC-MBX-005, AC-MBX-006, AC-MBX-007, AC-MBX-008, AC-MBX-009, AC-MBX-010, AC-MBX-011, AC-MBX-012, AC-MBX-015, AC-MBX-019, AC-MBX-022.
- **Exclusiones/costuras:** MBX-013/014/020 (seleccion) a P28; MBX-019 expone vista interna, Q-RT-4 (bloqueante) decide su forma.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P03, P08, P09.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/dispatch/mailbox/*.rs).
- **Tests/oraculo:** specs *MailboxSpec 1:1.
- **Diff:** ~480 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P14 REF-BASE

- **Resultado verificable:** Identidad/entrega/Props de ActorRef sobre una interfaz de celda minima.
- **Filas (15; impl: pending 15):** asignacion explicita en `docs/unit-rows.tsv`: AC-REF-001.1, AC-REF-001.2, AC-REF-001.3, AC-REF-002, AC-REF-004, AC-REF-005, AC-REF-006, AC-REF-007.1, AC-REF-007.2, AC-REF-008, AC-REF-018, AC-REF-022, AC-REF-024, AC-REF-025, AC-DSP-020.
- **Exclusiones/costuras:** Costura CELL-S1: trait ActorRefLike {tell, send_system}; REF-002/018 se prueban con un buzon fake; la celda real llega en P15-P22.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P01, P05, P10, P13.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/{actor_ref,props}.rs).
- **Tests/oraculo:** ActorRefSpec parcial + unidad.
- **Diff:** ~600 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P15 MBX-NUCLEO

- **Resultado verificable:** Estado atomico del mailbox, ejecucion con prioridad de sistema, throughput, attach/detach y suspend/resume de dispatcher.
- **Filas (7; impl: pending 7):** asignacion explicita en `docs/unit-rows.tsv`: AC-MBX-001, AC-MBX-002, AC-CELL-013.1, AC-DSP-001, AC-DSP-002, AC-DSP-003, AC-DSP-023.
- **Exclusiones/costuras:** Costura DISPATCH-S1: la celda se sustituye por trait CellInvoke {invoke, system_invoke}; DSP-023 usa suspend/resume sin restart real.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P08, P13, P14.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/dispatch/{mailbox,dispatcher}.rs).
- **Tests/oraculo:** Dispatchers/Mailbox specs 1:1.
- **Diff:** ~280 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P16 DSP-EJECUTORES

- **Resultado verificable:** Registro de dispatchers, ejecutores (fork-join, pool, pinned, affinity, virtual), batching y contextos de ejecucion.
- **Filas (17; impl: pending 17):** asignacion explicita en `docs/unit-rows.tsv`: AC-DSP-004, AC-DSP-005, AC-DSP-006, AC-DSP-008, AC-DSP-009, AC-DSP-010, AC-DSP-011, AC-DSP-013, AC-DSP-014, AC-DSP-015, AC-DSP-016.1, AC-DSP-016.2, AC-DSP-017.1, AC-DSP-017.2, AC-DSP-018, AC-DSP-019, AC-DSP-022.
- **Exclusiones/costuras:** Q-DSP-1 decide los ejecutores; DSP-007 (seleccion por actor) a P28.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P08, P09, P15.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/dispatch/{dispatchers,executors}.rs).
- **Tests/oraculo:** DispatchersSpec + unidad.
- **Diff:** ~680 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P17 REF-MUERTOS

- **Resultado verificable:** deadLetters, EmptyLocalActorRef y mailbox cerrado: el sumidero de cartas muertas.
- **Filas (4; impl: pending 4):** asignacion explicita en `docs/unit-rows.tsv`: AC-REF-015, AC-REF-016, AC-MBX-003, AC-MBX-021.
- **Exclusiones/costuras:** Watch->DWN de REF-015/016 usa la interfaz DeathWatchSink (DW-S1); la vigilancia real llega en P23.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P11, P14, P15.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/dead_letters.rs).
- **Tests/oraculo:** unidad + oraculo de comportamiento.
- **Diff:** ~160 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P18 CELL-HIJOS

- **Resultado verificable:** Contenedor de hijos y actorOf: reserva atomica de nombres.
- **Filas (6; impl: pending 6):** asignacion explicita en `docs/unit-rows.tsv`: AC-CELL-015.1, AC-REF-009.1, AC-REF-009.2, AC-REF-009.3, AC-REF-010, AC-REF-019.
- **Exclusiones/costuras:** CELL-015.1 y REF-009.2 son un solo comportamiento (R28): se implementan juntos aqui.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P14.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/children.rs).
- **Tests/oraculo:** ActorCreationSpec/ActorRefSpec parciales.
- **Diff:** ~240 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P19 CELL-EJECUCION

- **Resultado verificable:** Celda de actor: ciclo de mensajes, become, mensajes de sistema, ReceiveTimeout y API de contexto.
- **Filas (11; impl: pending 11):** asignacion explicita en `docs/unit-rows.tsv`: AC-CELL-001, AC-CELL-002.1, AC-CELL-003, AC-CELL-005, AC-CELL-013.2, AC-CELL-014, AC-CELL-017, AC-CELL-018, AC-CELL-019, AC-CELL-022, AC-CELL-031.
- **Exclusiones/costuras:** Los hooks de reinicio/parada (CELL-006/007/008) llegan en P21-P22; 013.3 en P23.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P08, P11, P12, P15, P16, P17, P18, P20.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/cell.rs).
- **Tests/oraculo:** ActorSpec/ActorLifeCycleSpec parciales.
- **Diff:** ~440 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P20 SUP-ESTRATEGIAS

- **Resultado verificable:** SupervisorStrategy: directivas, deciders, One/AllForOne, ventanas de reintentos.
- **Filas (10; impl: pending 10):** asignacion explicita en `docs/unit-rows.tsv`: AC-SUP-003, AC-SUP-004, AC-SUP-005, AC-SUP-006.1, AC-SUP-006.2, AC-SUP-007, AC-SUP-012, AC-SUP-013, AC-SUP-017, AC-SUP-021.
- **Exclusiones/costuras:** Pura sobre tipos; no depende de la celda (SUP-004 solo lista excepciones de ciclo de vida).
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P11, P14.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/supervision.rs).
- **Tests/oraculo:** SupervisorHierarchySpec parcial.
- **Diff:** ~400 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P21 SUP-FALLO

- **Resultado verificable:** Propagacion de fallos: suspender, enviar Failed al padre, aplicar la directiva, hooks de ciclo de vida.
- **Filas (11; impl: pending 11):** asignacion explicita en `docs/unit-rows.tsv`: AC-SUP-001, AC-SUP-002, AC-SUP-008, AC-SUP-009, AC-SUP-010, AC-SUP-016, AC-SUP-018, AC-SUP-020, AC-CELL-006, AC-CELL-020, AC-CELL-021.
- **Exclusiones/costuras:** Reinicio/parada completos en P22.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P19, P20.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/{fault,lifecycle}.rs).
- **Tests/oraculo:** SupervisorSpec parcial.
- **Diff:** ~440 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P22 CELL-REINICIO-PARADA

- **Resultado verificable:** Restart, stop, terminacion y espera de hijos como una maquina de estados unica.
- **Filas (21; impl: pending 21):** asignacion explicita en `docs/unit-rows.tsv`: AC-CELL-002.2, AC-CELL-007.1, AC-CELL-007.2, AC-CELL-007.3, AC-CELL-007.4, AC-CELL-015.2, AC-CELL-008.1, AC-CELL-008.2, AC-CELL-009, AC-CELL-010, AC-CELL-011, AC-CELL-012, AC-CELL-023, AC-CELL-024, AC-CELL-030, AC-SUP-011, AC-SUP-015, AC-SUP-019, AC-DW-008, AC-DW-009, AC-DW-010.
- **Exclusiones/costuras:** Resuelve R4/R8/R25/R26/R27; DW-008/009 se implementan aqui con el hook de Terminated (DW-001 llega en P23).
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P17, P18, P21.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/{restart,terminate}.rs).
- **Tests/oraculo:** ActorLifeCycleSpec, SupervisorSpec.
- **Diff:** ~840 lineas, 3 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P23 DW-VIGILANCIA

- **Resultado verificable:** Watch/unwatch, Terminated, death pact, unhandled y entrega a celda terminada.
- **Filas (13; impl: pending 13):** asignacion explicita en `docs/unit-rows.tsv`: AC-DW-001, AC-DW-002, AC-DW-003, AC-DW-004, AC-DW-005, AC-DW-006, AC-DW-007, AC-DW-011, AC-DW-012, AC-CELL-004, AC-CELL-013.3, AC-CELL-025, AC-EVT-010.
- **Exclusiones/costuras:** Resuelve R1(a): DW-002 + CELL-013.3 juntas; CELL-004 + DW-007 juntas.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P11, P17, P20, P22.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/{deathwatch,unhandled}.rs).
- **Tests/oraculo:** DeathWatchSpec 1:1 (oraculo por comportamiento).
- **Diff:** ~520 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P24 MBX-STASH

- **Resultado verificable:** Stash (stash/unstash/unstashAll) y su interaccion con restart/stop/Terminated.
- **Filas (5; impl: pending 5):** asignacion explicita en `docs/unit-rows.tsv`: AC-MBX-016, AC-MBX-017, AC-MBX-018, AC-MBX-023, AC-DW-013.
- **Exclusiones/costuras:** MBX-013 (tipo de mailbox exigido) se valida en P28.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P13, P22, P23.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/stash.rs).
- **Tests/oraculo:** StashSpec 1:1.
- **Diff:** ~200 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 3 - sistema, proveedor, referencias y extensiones

| ID | Unidad | Filas | Depende de | Diff est. | PR |
|---|---|---|---|---|---|
| P25 | SYS-GUARDIANES | 12 | P10, P16, P20, P22, P23 | ~480 | 2 |
| P26 | SYS-PROVEEDOR | 7 | P09, P10, P16, P25 | ~280 | 1 |
| P27 | SCH-ACTORES | 4 | P12, P22, P24 | ~160 | 1 |
| P28 | SELECCION-CREACION | 9 | P10, P13, P16, P24, P26 | ~360 | 1 |
| P29 | REF-SELECCION | 6 | P26 | ~240 | 1 |
| P30 | EXT | 5 | P08, P09, P26 | ~200 | 1 |

### P25 SYS-GUARDIANES

- **Resultado verificable:** ActorSystem: constructores, arranque, jerarquia de guardianes y guardian de usuario.
- **Filas (12; impl: pending 12):** asignacion explicita en `docs/unit-rows.tsv`: AC-SYS-001.1, AC-SYS-001.2, AC-SYS-002, AC-SYS-003.1, AC-SYS-003.2, AC-SYS-007, AC-SYS-008, AC-SYS-009, AC-SYS-013, AC-SYS-020, AC-SYS-023, AC-SUP-014.
- **Exclusiones/costuras:** Costura SYS-S1: hooks de terminacion via trait TerminationHooks (P39); registro de extensiones via ExtensionRegistry (P30).
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P10, P16, P20, P22, P23.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/system/{system,guardians}.rs).
- **Tests/oraculo:** ActorSystemSpec parcial + oraculo.
- **Diff:** ~480 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P26 SYS-PROVEEDOR

- **Resultado verificable:** Proveedor local: actorOf con Deploy, systemActorOf, resolveActorRef y SPI.
- **Filas (7; impl: pending 7):** asignacion explicita en `docs/unit-rows.tsv`: AC-SYS-010, AC-SYS-012, AC-SYS-016, AC-SYS-018, AC-SYS-019, AC-SYS-024, AC-REF-017.
- **Exclusiones/costuras:** SYS-016 parte router->P37 (CFG-012.3), seleccion de dispatcher->P28.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P09, P10, P16, P25.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/system/provider.rs).
- **Tests/oraculo:** LocalActorRefProviderSpec.
- **Diff:** ~280 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P27 SCH-ACTORES

- **Resultado verificable:** Timers y mensajes de scheduler dirigidos a actores.
- **Filas (4; impl: pending 4):** asignacion explicita en `docs/unit-rows.tsv`: AC-SCH-007, AC-SCH-011, AC-SCH-012, AC-SCH-013.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P12, P22, P24.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/timers.rs).
- **Tests/oraculo:** TimerSpec 1:1.
- **Diff:** ~160 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P28 SELECCION-CREACION

- **Resultado verificable:** Seleccion de dispatcher/mailbox por actor, arranque diferido (RepointableActorRef) y arranque de hijo.
- **Filas (9; impl: pending 9):** asignacion explicita en `docs/unit-rows.tsv`: AC-DSP-007, AC-MBX-013, AC-MBX-014, AC-MBX-020, AC-CELL-016, AC-CELL-026, AC-CELL-027, AC-REF-020, AC-REF-003.
- **Exclusiones/costuras:** CFG-012.3/.4 (router) ->P37: aqui Deploy.router opaco.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P10, P13, P16, P24, P26.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/{repointable,selection}.rs).
- **Tests/oraculo:** ActorCreationSpec, RepointableActorRefSpec.
- **Diff:** ~360 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P29 REF-SELECCION

- **Resultado verificable:** ActorSelection, Identify/ActorIdentity y entrega con comodines.
- **Filas (6; impl: pending 6):** asignacion explicita en `docs/unit-rows.tsv`: AC-REF-011, AC-REF-012, AC-REF-013, AC-REF-014, AC-REF-021, AC-REF-026.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P26.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/actor/selection.rs).
- **Tests/oraculo:** ActorSelectionSpec 1:1.
- **Diff:** ~240 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P30 EXT

- **Resultado verificable:** Extensiones: registro, carga desde config, extensiones provistas.
- **Filas (5; impl: pending 5):** asignacion explicita en `docs/unit-rows.tsv`: AC-EXT-001.1, AC-EXT-001.2, AC-EXT-002, AC-EXT-003, AC-EXT-004.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P08, P09, P26.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/system/extension.rs).
- **Tests/oraculo:** ExtensionSpec.
- **Diff:** ~200 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 4 - patrones, routers y logging

| ID | Unidad | Filas | Depende de | Diff est. | PR |
|---|---|---|---|---|---|
| P31 | ASK | 17 | P23, P26, P27, P29 | ~680 | 2 |
| P32 | CIRCUIT-BREAKER-BACKOFF | 15 | P22, P27, P30, P31 | ~600 | 2 |
| P33 | FSM | 11 | P22, P27, P35 | ~440 | 2 |
| P34 | EVT-LOGGING | 11 | P23, P26, P30, P33 | ~440 | 2 |
| P35 | RT-NUCLEO | 9 | P21, P23, P26, P31 | ~360 | 1 |
| P36 | RT-ESTRATEGIAS | 9 | P13, P15, P16, P28, P35 | ~360 | 1 |
| P37 | RT-CONFIG | 9 | P10, P28, P35, P36 | ~360 | 1 |

### P31 ASK

- **Resultado verificable:** ask, PromiseActorRef, actores temporales, StatusReply, pipeTo, gracefulStop, after/retry.
- **Filas (17; impl: pending 17):** asignacion explicita en `docs/unit-rows.tsv`: AC-SYS-017, AC-SYS-025, AC-REF-027, AC-ASK-001.1, AC-ASK-001.2, AC-ASK-001.3, AC-ASK-002, AC-ASK-003, AC-ASK-004, AC-ASK-005, AC-ASK-006, AC-ASK-007, AC-ASK-008, AC-ASK-009.1, AC-ASK-009.2, AC-ASK-009.3, AC-ASK-010.
- **Exclusiones/costuras:** Q-ASK-1 abierta fija la forma del Future.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P23, P26, P27, P29.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/pattern/{ask,promise_ref,temp}.rs).
- **Tests/oraculo:** AskSpec 1:1.
- **Diff:** ~680 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P32 CIRCUIT-BREAKER-BACKOFF

- **Resultado verificable:** CircuitBreaker y BackoffSupervisor.
- **Filas (15; impl: pending 15):** asignacion explicita en `docs/unit-rows.tsv`: AC-ASK-011.1, AC-ASK-011.2, AC-ASK-011.3, AC-ASK-011.4, AC-ASK-011.5, AC-ASK-012, AC-ASK-013, AC-ASK-014, AC-ASK-015.1, AC-ASK-015.2, AC-ASK-015.3, AC-ASK-015.4, AC-ASK-015.5, AC-ASK-015.6, AC-ASK-016.
- **Exclusiones/costuras:** ASK-011.5 (API Java) a P47; ASK-013/014 consumen EXT.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P22, P27, P30, P31.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/pattern/{circuit_breaker,backoff}.rs).
- **Tests/oraculo:** CircuitBreakerSpec, BackoffSupervisorSpec.
- **Diff:** ~600 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P33 FSM

- **Resultado verificable:** FSM: DSL, transiciones, timers, terminacion y LoggingFSM.
- **Filas (11; impl: pending 11):** asignacion explicita en `docs/unit-rows.tsv`: AC-FSM-001.1, AC-FSM-001.2, AC-FSM-002, AC-FSM-003, AC-FSM-004, AC-FSM-005, AC-FSM-006.1, AC-FSM-006.2, AC-FSM-007, AC-FSM-008, AC-FSM-009.
- **Exclusiones/costuras:** Q-FSM-4 bloquea FSM-006.1/.2; el resto avanza.
- **Decisiones:** Q-FSM-4 (bloqueante, sin respuesta).
- **Dependencias:** P22, P27, P35.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/pattern/fsm.rs).
- **Tests/oraculo:** FSMActorSpec 1:1.
- **Diff:** ~440 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P34 EVT-LOGGING

- **Resultado verificable:** Logging completo: LoggingBus, loggers, DeadLetterListener, EventStreamUnsubscriber, LoggingReceive.
- **Filas (11; impl: pending 11):** asignacion explicita en `docs/unit-rows.tsv`: AC-EVT-005, AC-EVT-007, AC-EVT-009, AC-EVT-011, AC-EVT-013.3, AC-EVT-014, AC-EVT-017, AC-EVT-018, AC-EVT-019, AC-EVT-020, AC-EVT-021.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P23, P26, P30, P33.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/event/{logging,listeners}.rs).
- **Tests/oraculo:** LoggingSpec, EventStreamSpec.
- **Diff:** ~440 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P35 RT-NUCLEO

- **Resultado verificable:** Router, RoutedActorCell, Pool/Group, RoundRobin/Random/Broadcast, listeners.
- **Filas (9; impl: pending 9):** asignacion explicita en `docs/unit-rows.tsv`: AC-RT-001, AC-RT-002, AC-RT-003.1, AC-RT-003.2, AC-RT-003.3, AC-RT-005, AC-RT-006, AC-RT-007, AC-RT-015.
- **Exclusiones/costuras:** Costura RT-S1 para ask: ROUTER-GATHER usa trait Asker hasta P31.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P21, P23, P26, P31.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/routing/*.rs).
- **Tests/oraculo:** RoutingSpec parcial.
- **Diff:** ~360 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P36 RT-ESTRATEGIAS

- **Resultado verificable:** SmallestMailbox, ConsistentHashing, ScatterGather, TailChopping, BalancingPool/Dispatcher.
- **Filas (9; impl: pending 9):** asignacion explicita en `docs/unit-rows.tsv`: AC-RT-008, AC-RT-009.1, AC-RT-009.2, AC-RT-009.3, AC-RT-010, AC-RT-011, AC-RT-012, AC-DSP-012, AC-DSP-021.
- **Exclusiones/costuras:** Q-RT-4 (bloqueante) afecta RT-008.
- **Decisiones:** Q-RT-4 (bloqueante, sin respuesta).
- **Dependencias:** P13, P15, P16, P28, P35.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/routing/*.rs).
- **Tests/oraculo:** RoutingSpec, ConsistentHashingRouterSpec.
- **Diff:** ~360 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P37 RT-CONFIG

- **Resultado verificable:** FromConfig/NoRouter, resizers, router.type-mapping y secciones de resizer en deployment.
- **Filas (9; impl: pending 9):** asignacion explicita en `docs/unit-rows.tsv`: AC-RT-004, AC-RT-013.1, AC-RT-013.2, AC-RT-013.3, AC-RT-014, AC-RT-016, AC-CFG-012.3, AC-CFG-012.4, AC-CFG-014.
- **Exclusiones/costuras:** Q-RT-4 afecta RT-013.*/014.
- **Decisiones:** Q-RT-4 (bloqueante, sin respuesta).
- **Dependencias:** P10, P28, P35, P36.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/routing/{config,resizer}.rs).
- **Tests/oraculo:** ResizerSpec, RouterConfigSpec.
- **Diff:** ~360 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 5 - apagado, serializacion e IO

| ID | Unidad | Filas | Depende de | Diff est. | PR |
|---|---|---|---|---|---|
| P38 | CS-NUCLEO | 13 | P08, P26, P27, P30 | ~520 | 2 |
| P39 | TERMINACION | 10 | P08, P25, P38 | ~400 | 1 |
| P40 | SER-NUCLEO | 18 | P04, P08, P09, P42 | ~720 | 2 |
| P41 | SER-SISTEMA | 8 | P26, P29, P30, P40 | ~320 | 1 |
| P42 | IO-TCP | 16 | P04, P08, P16, P26, P30 | ~640 | 2 |
| P43 | IO-UDP-DNS | 19 | P08, P42 | ~760 | 2 |

### P38 CS-NUCLEO

- **Resultado verificable:** CoordinatedShutdown: fases, tareas, run, timeouts, reasons y hooks de JVM.
- **Filas (13; impl: pending 13):** asignacion explicita en `docs/unit-rows.tsv`: AC-CS-001.1, AC-CS-001.2, AC-CS-002, AC-CS-003.1, AC-CS-003.2, AC-CS-004.1, AC-CS-004.2, AC-CS-004.3, AC-CS-005, AC-CS-007.1, AC-CS-007.2, AC-CS-008, AC-CS-009.
- **Exclusiones/costuras:** CS-006.* en P39.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P08, P26, P27, P30.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/system/coordinated_shutdown.rs).
- **Tests/oraculo:** CoordinatedShutdownSpec 1:1.
- **Diff:** ~520 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P39 TERMINACION

- **Resultado verificable:** terminate/close/whenTerminated/abort y la fase actor-system-terminate.
- **Filas (10; impl: pending 10):** asignacion explicita en `docs/unit-rows.tsv`: AC-SYS-004, AC-SYS-005, AC-SYS-006, AC-SYS-011, AC-SYS-014, AC-SYS-015, AC-SYS-021, AC-SYS-022, AC-CS-006.1, AC-CS-006.2.
- **Exclusiones/costuras:** Resuelve R16/R49 (SYS-004/005/006 + CS-006).
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P08, P25, P38.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/system/termination.rs).
- **Tests/oraculo:** ActorSystemSpec, CoordinatedShutdownSpec.
- **Diff:** ~400 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P40 SER-NUCLEO

- **Resultado verificable:** Serialization: resolucion, serializers por defecto y limites.
- **Filas (18; impl: pending 18):** asignacion explicita en `docs/unit-rows.tsv`: AC-SER-001.1, AC-SER-001.2, AC-SER-001.3, AC-SER-002, AC-SER-003.1, AC-SER-003.2, AC-SER-003.3, AC-SER-003.4, AC-SER-003.5, AC-SER-004.1, AC-SER-004.2, AC-SER-005, AC-SER-006, AC-SER-009, AC-SER-010, AC-SER-011, AC-SER-012, AC-SER-013.
- **Exclusiones/costuras:** Q-SER-1 (bloqueante): sin respuesta solo SER-002/003.1/003.4/010 pueden avanzar.
- **Decisiones:** Q-SER-1 (bloqueante, sin respuesta).
- **Dependencias:** P04, P08, P09, P42.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/serialization/*.rs).
- **Tests/oraculo:** SerializeSpec 1:1.
- **Diff:** ~720 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P41 SER-SISTEMA

- **Resultado verificable:** SerializationExtension, serialized paths, verificacion de mensajes/creadores.
- **Filas (8; impl: pending 8):** asignacion explicita en `docs/unit-rows.tsv`: AC-SER-007.1, AC-SER-007.2, AC-SER-008.1, AC-SER-008.2, AC-SER-014, AC-REF-023, AC-CELL-028, AC-CELL-029.
- **Exclusiones/costuras:** Resuelve R3 (SER-001.3 + SER-007.2 + SYS-018).
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P26, P29, P30, P40.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/serialization/ext.rs).
- **Tests/oraculo:** SerializationSetupSpec.
- **Diff:** ~320 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P42 IO-TCP

- **Resultado verificable:** IO: extension, Tcp y gestores de selector.
- **Filas (16; impl: pending 16):** asignacion explicita en `docs/unit-rows.tsv`: AC-IO-001, AC-IO-002.1, AC-IO-002.2, AC-IO-002.3, AC-IO-002.4, AC-IO-002.5, AC-IO-002.6, AC-IO-002.7, AC-IO-003, AC-IO-004.1, AC-IO-004.2, AC-IO-004.3, AC-IO-004.4, AC-IO-010, AC-IO-012, AC-IO-013.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P04, P08, P16, P26, P30.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/io/tcp/*.rs).
- **Tests/oraculo:** TcpIntegrationSpec.
- **Diff:** ~640 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P43 IO-UDP-DNS

- **Resultado verificable:** IO: Udp, UdpConnected, DNS (inet y async).
- **Filas (19; impl: pending 19):** asignacion explicita en `docs/unit-rows.tsv`: AC-IO-005, AC-IO-006.1, AC-IO-006.2, AC-IO-006.3, AC-IO-007, AC-IO-008.1, AC-IO-008.2, AC-IO-008.3, AC-IO-008.4, AC-IO-008.5, AC-IO-009.1, AC-IO-009.2, AC-IO-009.3, AC-IO-009.4, AC-IO-009.5, AC-IO-009.6, AC-IO-009.7, AC-IO-009.8, AC-IO-011.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P08, P42.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/io/{udp,dns}/*.rs).
- **Tests/oraculo:** UdpIntegrationSpec, AsyncDnsResolverSpec.
- **Diff:** ~760 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 6 - testkit Classic y superficie Java

| ID | Unidad | Filas | Depende de | Diff est. | PR |
|---|---|---|---|---|---|
| P44 | TK-DISPATCHER-REF | 5 | P15, P22, P26, P28 | ~200 | 1 |
| P45 | TK-API | 22 | P11, P23, P27, P34, P39, P44, P46 | ~880 | 3 |
| P46 | JAPI-ACTORES | 16 | P20, P22, P28, P29, P33, P39, P40 | ~640 | 2 |
| P47 | JAPI-EXTRA | 7 | P32, P34, P40, P42, P43, P46 | ~280 | 1 |

### P44 TK-DISPATCHER-REF

- **Resultado verificable:** TestActorRef y CallingThreadDispatcher.
- **Filas (5; impl: pending 5):** asignacion explicita en `docs/unit-rows.tsv`: AC-TK-001.1, AC-TK-006.1, AC-TK-006.2, AC-TK-008, AC-TK-016.
- **Exclusiones/costuras:** Q-TK-1 (bloqueante).
- **Decisiones:** Q-TK-1 (bloqueante, sin respuesta).
- **Dependencias:** P15, P22, P26, P28.
- **Archivos previstos:** `crates/rakka-actor/src/...` (testkit/src/{test_actor_ref,calling_thread}.rs).
- **Tests/oraculo:** CallingThreadDispatcherSpec.
- **Diff:** ~200 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P45 TK-API

- **Resultado verificable:** TestKit/TestProbe, EventFilter, TestFSMRef, utilidades y SocketUtil.
- **Filas (22; impl: pending 22):** asignacion explicita en `docs/unit-rows.tsv`: AC-TK-001.2, AC-TK-001.3, AC-TK-001.4, AC-TK-001.5, AC-TK-002.1, AC-TK-002.2, AC-TK-002.3, AC-TK-003, AC-TK-004, AC-TK-005.1, AC-TK-005.2, AC-TK-005.3, AC-TK-007, AC-TK-009, AC-TK-010, AC-TK-011, AC-TK-012.1, AC-TK-012.2, AC-TK-013, AC-TK-014, AC-TK-015, AC-TK-017.
- **Exclusiones/costuras:** TK-011 (javadsl) tras P46.
- **Decisiones:** Q-TK-1 (bloqueante, sin respuesta).
- **Dependencias:** P11, P23, P27, P34, P39, P44, P46.
- **Archivos previstos:** `crates/rakka-actor/src/...` (testkit/src/*.rs).
- **Tests/oraculo:** TestKitSpec 1:1.
- **Diff:** ~880 lineas, 3 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P46 JAPI-ACTORES

- **Resultado verificable:** Superficie Java-facing de actores: AbstractActor, AbstractFSM, japi.pf/function, Props Java.
- **Filas (16; impl: pending 16):** asignacion explicita en `docs/unit-rows.tsv`: AC-JAPI-001, AC-JAPI-002, AC-JAPI-003, AC-JAPI-004.1, AC-JAPI-004.2, AC-JAPI-004.3, AC-JAPI-004.4, AC-JAPI-005.1, AC-JAPI-005.2, AC-JAPI-005.3, AC-JAPI-005.4, AC-JAPI-013, AC-JAPI-006, AC-JAPI-008, AC-JAPI-011, AC-JAPI-012.
- **Exclusiones/costuras:** Q-JAPI-1..4 deciden si se porta.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P20, P22, P28, P29, P33, P39, P40.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/japi/*.rs).
- **Tests/oraculo:** specs Java de Pekko.
- **Diff:** ~640 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P47 JAPI-EXTRA

- **Resultado verificable:** EventBus Java, TypedActor clasico, fabricas IO Java, API Java de patrones.
- **Filas (7; impl: pending 7):** asignacion explicita en `docs/unit-rows.tsv`: AC-JAPI-007, AC-JAPI-009.1, AC-JAPI-009.2, AC-JAPI-009.3, AC-JAPI-010, AC-EVT-022, AC-ASK-017.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P32, P34, P40, P42, P43, P46.
- **Archivos previstos:** `crates/rakka-actor/src/...` (classic/src/japi/*.rs).
- **Tests/oraculo:** specs Java.
- **Diff:** ~280 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 7 - Typed: nucleo, adaptador y sistema

| ID | Unidad | Filas | Depende de | Diff est. | PR |
|---|---|---|---|---|---|
| P48 | T-COMPORTAMIENTO-NUCLEO | 21 | P22 | ~840 | 3 |
| P49 | T-ADAPTADOR-CLASICO | 19 | P23, P26, P30, P48 | ~760 | 2 |
| P50 | T-SISTEMA-TIPADO | 25 | P30, P39, P49, P53 | ~1000 | 3 |
| P51 | T-ASK-TIPADO | 5 | P31, P50, P52 | ~200 | 1 |
| P52 | T-OBSERVABILIDAD | 14 | P34, P49, P50, P53 | ~560 | 2 |
| P53 | T-INTERCEPTORES-TIMERS-STASH | 21 | P24, P27, P49 | ~840 | 3 |

### P48 T-COMPORTAMIENTO-NUCLEO

- **Resultado verificable:** Typed: Behavior, interprete, receive/setup, ActorRef tipado, senales basicas.
- **Filas (21; impl: pending 21):** asignacion explicita en `docs/unit-rows.tsv`: AT-BEH-001, AT-BEH-002, AT-BEH-004, AT-BEH-005, AT-BEH-006, AT-BEH-007, AT-BEH-008.1, AT-BEH-008.2, AT-BEH-008.3, AT-BEH-009, AT-BEH-011, AT-BEH-012, AT-BEH-022, AT-BEH-023, AT-SIG-001, AT-SIG-003, AT-REF-001, AT-REF-002, AT-REF-003, AT-REF-005, AT-REF-006.
- **Exclusiones/costuras:** Q-T-BEH-1 (bloqueante) fija la forma; Costura BEH-S1: Behavior sin supervision/timers/stash, que llegan en P53/P55.
- **Decisiones:** Q-T-BEH-1 (bloqueante, sin respuesta).
- **Dependencias:** P22.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/{behavior,interpreter,actor_ref}.rs).
- **Tests/oraculo:** BehaviorSpec 1:1.
- **Diff:** ~840 lineas, 3 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P49 T-ADAPTADOR-CLASICO

- **Resultado verificable:** Adaptador typed sobre celda clasica (ActorAdapter, ActorSystemAdapter) y contexto basico.
- **Filas (19; impl: pending 19):** asignacion explicita en `docs/unit-rows.tsv`: AT-ADP-001, AT-ADP-003, AT-ADP-004, AT-ADP-006.1, AT-ADP-006.2, AT-ADP-006.3, AT-ADP-006.4, AT-ADP-007, AT-ADP-008, AT-ADP-009, AT-ADP-010, AT-EXT-002.3, AT-SYS-011, AT-CTX-001, AT-CTX-002, AT-CTX-003, AT-CTX-004, AT-CTX-016, AT-CTX-018.
- **Exclusiones/costuras:** Q-X-8 (proposed): asume runtime sobre Classic; si el Owner elige nativo, esta unidad y P50 se rehacen.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P23, P26, P30, P48.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/adapter/*.rs).
- **Tests/oraculo:** ActorContextSpec.
- **Diff:** ~760 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P50 T-SISTEMA-TIPADO

- **Resultado verificable:** ActorSystem tipado, guardian, Props/Dispatcher/Mailbox selectors, extensiones y CS.
- **Filas (25; impl: pending 25):** asignacion explicita en `docs/unit-rows.tsv`: AT-SYS-001, AT-SYS-002, AT-SYS-003, AT-SYS-004, AT-SYS-005, AT-SYS-006, AT-SYS-007.1, AT-SYS-007.2, AT-SYS-007.3, AT-SYS-007.4, AT-SYS-008, AT-SYS-009, AT-SYS-010, AT-SYS-012, AT-CS-001, AT-DSP-001, AT-DSP-002, AT-DSP-003, AT-DSP-005, AT-EXT-001.1, AT-EXT-001.2, AT-EXT-001.3, AT-EXT-002.1, AT-EXT-002.2, AT-EXT-003.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P30, P39, P49, P53.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/{system,props,extension}.rs).
- **Tests/oraculo:** ActorSystemSpec.
- **Diff:** ~1000 lineas, 3 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P51 T-ASK-TIPADO

- **Resultado verificable:** ask tipado, StatusReply, RecipientRef.
- **Filas (5; impl: pending 5):** asignacion explicita en `docs/unit-rows.tsv`: AT-ASK-001, AT-ASK-002, AT-ASK-003, AT-ASK-004, AT-REF-004.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P31, P50, P52.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/ask.rs).
- **Tests/oraculo:** AskSpec.
- **Diff:** ~200 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P52 T-OBSERVABILIDAD

- **Resultado verificable:** EventStream tipado, logging (MDC), withMdc/monitor/logMessages, unhandled.
- **Filas (14; impl: pending 14):** asignacion explicita en `docs/unit-rows.tsv`: AT-EVT-001, AT-EVT-002, AT-EVT-003, AT-EVT-004, AT-LOG-001, AT-LOG-002, AT-LOG-003, AT-LOG-004, AT-LOG-005, AT-BEH-003, AT-BEH-017, AT-BEH-018, AT-BEH-019, AT-DSP-004.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P34, P49, P50, P53.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/{eventstream,logging}.rs).
- **Tests/oraculo:** LoggingSpec, MDCSpec.
- **Diff:** ~560 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P53 T-INTERCEPTORES-TIMERS-STASH

- **Resultado verificable:** Interceptores, timers y stash tipados.
- **Filas (21; impl: pending 21):** asignacion explicita en `docs/unit-rows.tsv`: AT-BEH-013, AT-BEH-014, AT-BEH-015, AT-BEH-016, AT-BEH-020, AT-BEH-021, AT-SIG-008, AT-TMR-001, AT-TMR-002, AT-TMR-003, AT-TMR-004, AT-TMR-005, AT-TMR-006, AT-TMR-007.1, AT-TMR-007.2, AT-TMR-008, AT-STASH-001, AT-STASH-002, AT-STASH-003, AT-STASH-004, AT-STASH-005.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P24, P27, P49.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/{intercept,timers,stash}.rs).
- **Tests/oraculo:** InterceptSpec, TimerSpec, StashSpec.
- **Diff:** ~840 lineas, 3 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 8 - Typed: contexto, supervision, receptionist y entrega fiable

| ID | Unidad | Filas | Depende de | Diff est. | PR |
|---|---|---|---|---|---|
| P54 | T-CONTEXTO-SENALES | 14 | P49, P51, P53 | ~560 | 2 |
| P55 | T-SUPERVISION | 31 | P21, P49, P52, P53, P54 | ~1240 | 4 |
| P56 | T-RECEPCIONISTA-SERIALIZACION | 14 | P30, P41, P50, P54 | ~560 | 2 |
| P57 | T-ROUTERS-PUBSUB-SPAWN | 12 | P54, P55, P56 | ~480 | 2 |
| P58 | T-ENTREGA-FIABLE-BASE | 13 | P53, P54, P56 | ~520 | 2 |
| P59 | T-ENTREGA-FIABLE-DURABLE | 6 | P58 | ~240 | 1 |
| P60 | T-ENTREGA-FIABLE-WORKPULLING | 6 | P56, P59 | ~240 | 1 |

### P54 T-CONTEXTO-SENALES

- **Resultado verificable:** Contexto tipado completo (watch, receive timeout, adapters, pipeToSelf) y senales ligadas.
- **Filas (14; impl: pending 14):** asignacion explicita en `docs/unit-rows.tsv`: AT-CTX-005, AT-CTX-006, AT-CTX-007, AT-CTX-008, AT-CTX-009, AT-CTX-010, AT-CTX-011, AT-CTX-012, AT-CTX-013, AT-CTX-014, AT-CTX-015, AT-SIG-004, AT-SIG-007, AT-SUP-015.
- **Exclusiones/costuras:** Resuelve R19/R64.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P49, P51, P53.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/context.rs).
- **Tests/oraculo:** ActorContextSpec.
- **Diff:** ~560 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P55 T-SUPERVISION

- **Resultado verificable:** Supervision tipada: estrategias, backoff, stash de restart, interop clasica.
- **Filas (31; impl: pending 31):** asignacion explicita en `docs/unit-rows.tsv`: AT-SUP-001, AT-SUP-002, AT-SUP-003, AT-SUP-004, AT-SUP-005, AT-SUP-006, AT-SUP-007.1, AT-SUP-007.2, AT-SUP-007.3, AT-SUP-008, AT-SUP-009, AT-SUP-010, AT-SUP-011, AT-SUP-012, AT-SUP-013, AT-SUP-014, AT-SUP-016, AT-SUP-017, AT-SUP-018, AT-SUP-019, AT-SUP-020, AT-SUP-021, AT-SUP-022, AT-SIG-002, AT-SIG-005, AT-SIG-006, AT-STASH-006, AT-BEH-010, AT-BEH-024, AT-ADP-002, AT-ADP-005.
- **Exclusiones/costuras:** Q-T-SUP-3 (bloqueante) afecta SUP-007.2/.3.
- **Decisiones:** Q-T-BEH-1 (bloqueante, sin respuesta); Q-T-SUP-3 (bloqueante, sin respuesta).
- **Dependencias:** P21, P49, P52, P53, P54.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/supervision.rs).
- **Tests/oraculo:** SupervisionSpec.
- **Diff:** ~1240 lineas, 4 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P56 T-RECEPCIONISTA-SERIALIZACION

- **Resultado verificable:** Receptionist y serializadores tipados.
- **Filas (14; impl: pending 14):** asignacion explicita en `docs/unit-rows.tsv`: AT-RCP-001, AT-RCP-002, AT-RCP-003, AT-RCP-004, AT-RCP-005.1, AT-RCP-005.2, AT-RCP-005.3, AT-RCP-006, AT-RCP-007, AT-RCP-008, AT-SER-001, AT-SER-002, AT-SER-003, AT-REF-007.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P30, P41, P50, P54.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/{receptionist,serializer}.rs).
- **Tests/oraculo:** ReceptionistSpec.
- **Diff:** ~560 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P57 T-ROUTERS-PUBSUB-SPAWN

- **Resultado verificable:** Routers tipados, pub-sub local y SpawnProtocol.
- **Filas (12; impl: pending 12):** asignacion explicita en `docs/unit-rows.tsv`: AT-RT-001, AT-RT-002.1, AT-RT-002.2, AT-RT-002.3, AT-RT-003, AT-RT-004, AT-RT-005, AT-RT-006, AT-PS-001.1, AT-PS-001.2, AT-PS-001.3, AT-SPV-001.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P54, P55, P56.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/{routing,pubsub,spawn_protocol}.rs).
- **Tests/oraculo:** RoutersSpec.
- **Diff:** ~480 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P58 T-ENTREGA-FIABLE-BASE

- **Resultado verificable:** Reliable delivery: Producer/ConsumerController, control de flujo y reenvio.
- **Filas (13; impl: pending 13):** asignacion explicita en `docs/unit-rows.tsv`: AT-DLV-006, AT-DLV-001.1, AT-DLV-001.2, AT-DLV-001.3, AT-DLV-001.4, AT-DLV-001.5, AT-DLV-002.1, AT-DLV-002.2, AT-DLV-002.3, AT-DLV-002.4, AT-DLV-002.5, AT-DLV-002.6, AT-DLV-002.7.
- **Exclusiones/costuras:** Resuelve R2 parte 1: DLV-001.6 (durable) se difiere a P59 con interfaz DurableQueue.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P53, P54, P56.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/delivery/*.rs).
- **Tests/oraculo:** ReliableDeliverySpec.
- **Diff:** ~520 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P59 T-ENTREGA-FIABLE-DURABLE

- **Resultado verificable:** Cola duradera y chunking.
- **Filas (6; impl: pending 6):** asignacion explicita en `docs/unit-rows.tsv`: AT-DLV-001.6, AT-DLV-004, AT-DLV-005.1, AT-DLV-005.2, AT-DLV-005.3, AT-DLV-007.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P58.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/delivery/{durable,chunking}.rs).
- **Tests/oraculo:** DurableProducerQueueSpec.
- **Diff:** ~240 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P60 T-ENTREGA-FIABLE-WORKPULLING

- **Resultado verificable:** WorkPullingProducerController.
- **Filas (6; impl: pending 6):** asignacion explicita en `docs/unit-rows.tsv`: AT-DLV-003.1, AT-DLV-003.2, AT-DLV-003.3, AT-DLV-003.4, AT-DLV-003.5, AT-DLV-003.6.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P56, P59.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/delivery/work_pulling.rs).
- **Tests/oraculo:** WorkPullingProducerControllerSpec.
- **Diff:** ~240 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Ola 9 - Typed: javadsl y testkits

| ID | Unidad | Filas | Depende de | Diff est. | PR |
|---|---|---|---|---|---|
| P61 | T-JAVADSL | 17 | P48, P49, P51, P53, P54, P55, P57 | ~680 | 2 |
| P62 | T-TK-SONDA | 17 | P40, P44, P50, P52, P54, P56 | ~680 | 2 |
| P63 | T-TK-COMPORTAMIENTO | 19 | P53, P54, P55, P56, P62 | ~760 | 2 |
| P64 | T-TK-LOGGING | 6 | P45, P52, P62 | ~240 | 1 |

### P61 T-JAVADSL

- **Resultado verificable:** javadsl tipado.
- **Filas (17; impl: pending 17):** asignacion explicita en `docs/unit-rows.tsv`: AT-JAPI-001.1, AT-JAPI-001.2, AT-JAPI-001.3, AT-JAPI-002.1, AT-JAPI-002.2, AT-JAPI-003, AT-JAPI-004, AT-JAPI-005.1, AT-JAPI-005.2, AT-JAPI-005.3, AT-JAPI-005.4, AT-JAPI-006.1, AT-JAPI-006.2, AT-JAPI-006.3, AT-JAPI-006.4, AT-JAPI-006.5, AT-CTX-017.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P48, P49, P51, P53, P54, P55, P57.
- **Archivos previstos:** `crates/rakka-actor-typed/src/...` (typed/src/javadsl/*.rs).
- **Tests/oraculo:** specs Java.
- **Diff:** ~680 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P62 T-TK-SONDA

- **Resultado verificable:** ActorTestKit, TestProbe, ManualTime, adaptadores de test.
- **Filas (17; impl: pending 17):** asignacion explicita en `docs/unit-rows.tsv`: AT-TK-001.1, AT-TK-001.2, AT-TK-001.3, AT-TK-002.1, AT-TK-002.2, AT-TK-002.3, AT-TK-002.4, AT-TK-002.5, AT-TK-002.6, AT-TK-007, AT-TK-008, AT-TK-009.1, AT-TK-009.2, AT-TK-009.3, AT-TK-009.4, AT-TK-010, AT-TK-014.
- **Exclusiones/costuras:** ninguna.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P40, P44, P50, P52, P54, P56.
- **Archivos previstos:** `crates/rakka-actor/src/...` (testkit-typed/src/*.rs).
- **Tests/oraculo:** ActorTestKitSpec.
- **Diff:** ~680 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P63 T-TK-COMPORTAMIENTO

- **Resultado verificable:** BehaviorTestKit, Effects, inboxes, ActorSystemStub.
- **Filas (19; impl: pending 19):** asignacion explicita en `docs/unit-rows.tsv`: AT-TK-003.1, AT-TK-003.2, AT-TK-003.3, AT-TK-003.4, AT-TK-003.5, AT-TK-003.6, AT-TK-003.7, AT-TK-004.1, AT-TK-004.2, AT-TK-004.3, AT-TK-004.4, AT-TK-004.5, AT-TK-004.6, AT-TK-004.7, AT-TK-005.1, AT-TK-005.2, AT-TK-005.3, AT-TK-011, AT-TK-012.
- **Exclusiones/costuras:** Resuelve R5/R7/R24 como una sola unidad.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P53, P54, P55, P56, P62.
- **Archivos previstos:** `crates/rakka-actor/src/...` (testkit-typed/src/behavior_testkit/*.rs).
- **Tests/oraculo:** BehaviorTestKitSpec.
- **Diff:** ~760 lineas, 2 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

### P64 T-TK-LOGGING

- **Resultado verificable:** LoggingTestKit y captura de logs.
- **Filas (6; impl: pending 6):** asignacion explicita en `docs/unit-rows.tsv`: AT-TK-006.1, AT-TK-006.2, AT-TK-006.3, AT-TK-006.4, AT-TK-006.5, AT-TK-013.
- **Exclusiones/costuras:** Resuelve R12/R14.
- **Decisiones:** ninguna bloqueante; propuestas segun `contract-map.tsv`.
- **Dependencias:** P45, P52, P62.
- **Archivos previstos:** `crates/rakka-actor/src/...` (testkit-typed/src/logging.rs).
- **Tests/oraculo:** LoggingTestKitSpec.
- **Diff:** ~240 lineas, 1 PR secuencial(es).
- **Cierre:** aserciones A# de las filas verdes con evidencia RED->GREEN, cobertura >= 90 %, `impl`/`validation` actualizados solo donde no queda decision abierta ni costura sin reemplazar.

## Verificacion de cobertura de filas

Script de solo lectura (`python3 -I`), descrito y ejecutado: lee `docs/unit-rows.tsv` y comprueba que cada `row_id` aparece en exactamente una unidad.

```python
import csv,collections
rows=[r[0] for r in list(csv.reader(open("docs/contract-map.tsv"),delimiter="\t"))[1:]]
# unidades: lista (id, filas) tal como aparece en este documento, con rangos A..B segun el orden del TSV
c=collections.Counter(r for u in UNIDADES for r in expandir(u))
assert [r for r in rows if c[r]!=1]==[] and len(c)==len(rows)==783
```

Resultado de la ejecucion al redactar: 783 filas del mapa, 783 cubiertas, 0 sin unidad, 0 en mas de una unidad. La asignacion es explicita (sin rangos) en `docs/unit-rows.tsv`; `python3 -I tools/package/check.py` verifica que cada `row_id` aparece una vez, que el recuento por unidad coincide con la ficha y que no hay IDs inexistentes.

## Anexo: dependencias de fila que quedan hacia adelante (costuras)

Despues de ordenar las unidades, 153 aristas `depends_on` fila->fila apuntan a una unidad posterior. Ninguna crea un ciclo entre unidades (el grafo de unidades es aciclico, verificado). Cada par se resuelve segun `docs/cycles.md` (causa: referencia documental/consumidor, dependencia de validacion o interfaz temporal). Pares unidad->unidad con ejemplos:

| Unidad | Necesita (posterior) | n | Ejemplos |
|---|---|---|---|
| P03 | P10 | 2 | AC-UTIL-009>AC-CFG-012.1, AC-UTIL-009>AC-CFG-012.2 |
| P03 | P11 | 1 | AC-UTIL-008>AC-EVT-003 |
| P03 | P13 | 2 | AC-UTIL-012>AC-MBX-009, AC-UTIL-013>AC-MBX-010 |
| P03 | P37 | 2 | AC-UTIL-009>AC-CFG-012.3, AC-UTIL-009>AC-CFG-012.4 |
| P04 | P40 | 2 | AC-UTIL-006.8>AC-SER-004.1, AC-UTIL-006.8>AC-SER-004.2 |
| P05 | P08 | 2 | AC-UTIL-016.11>AC-CFG-007, AC-UTIL-016.9>AC-CFG-007 |
| P05 | P09 | 1 | AC-UTIL-016.11>AC-CFG-016 |
| P05 | P30 | 1 | AC-UTIL-016.9>AC-EXT-001.1 |
| P05 | P40 | 2 | AC-UTIL-016.12>AC-SER-004.1, AC-UTIL-016.12>AC-SER-004.2 |
| P06 | P12 | 3 | AC-UTIL-026>AC-SCH-008.1, AC-UTIL-026>AC-SCH-008.2, AC-UTIL-026>AC-SCH-008.3 |
| P06 | P42 | 1 | AC-UTIL-026>AC-IO-003 |
| P07 | P08 | 2 | AC-CFG-003>AC-CFG-002, AC-CFG-019>AC-CFG-002 |
| P08 | P09 | 1 | AC-CFG-018>AC-CFG-016 |
| P08 | P25 | 1 | AC-CFG-009>AC-SUP-014 |
| P08 | P28 | 1 | AC-CFG-009>AC-REF-020 |
| P08 | P30 | 1 | AC-CFG-006>AC-EXT-003 |
| P08 | P34 | 3 | AC-CFG-004>AC-EVT-011, AC-CFG-005>AC-EVT-009, AC-CFG-011>AC-EVT-019 |
| P08 | P38 | 2 | AC-CFG-007>AC-CS-007.1, AC-CFG-007>AC-CS-007.2 |
| P08 | P39 | 3 | AC-CFG-007>AC-SYS-015, AC-CFG-020>AC-CS-006.1, AC-CFG-020>AC-CS-006.2 |
| P08 | P40 | 3 | AC-CFG-010>AC-SER-001.1, AC-CFG-010>AC-SER-001.2, AC-CFG-010>AC-SER-001.3 |
| P09 | P26 | 1 | AC-CFG-008>AC-SYS-018 |
| P10 | P37 | 2 | AC-CFG-013>AC-CFG-012.3, AC-CFG-013>AC-CFG-012.4 |
| P11 | P17 | 1 | AC-EVT-008>AC-REF-016 |
| P11 | P34 | 2 | AC-EVT-006>AC-EVT-007, AC-EVT-012>AC-EVT-011 |
| P12 | P15 | 1 | AC-SCH-009>AC-DSP-001 |
| P12 | P16 | 2 | AC-SCH-001>AC-DSP-016.1, AC-SCH-001>AC-DSP-016.2 |
| P13 | P28 | 2 | AC-MBX-015>AC-MBX-013, AC-MBX-022>AC-MBX-014 |
| P13 | P36 | 1 | AC-MBX-019>AC-RT-008 |
| P14 | P15 | 2 | AC-REF-002>AC-MBX-001, AC-REF-018>AC-CELL-013.1 |
| P14 | P19 | 2 | AC-REF-005>AC-CELL-003, AC-REF-006>AC-CELL-002.1 |
| P14 | P22 | 1 | AC-REF-006>AC-CELL-002.2 |
| P14 | P26 | 1 | AC-REF-025>AC-SYS-012 |
| P15 | P22 | 1 | AC-DSP-023>AC-CELL-007.1 |
| P16 | P34 | 1 | AC-DSP-016.1>AC-EVT-011 |
| P17 | P19 | 1 | AC-MBX-003>AC-CELL-014 |
| P17 | P22 | 1 | AC-MBX-003>AC-CELL-008.2 |
| P17 | P23 | 1 | AC-MBX-003>AC-CELL-013.3 |
| P18 | P22 | 1 | AC-REF-019>AC-CELL-008.2 |
| P19 | P21 | 1 | AC-CELL-022>AC-CELL-006 |
| P19 | P22 | 3 | AC-CELL-013.2>AC-CELL-007.2, AC-CELL-013.2>AC-CELL-008.1, AC-CELL-031>AC-CELL-030 |
| P19 | P23 | 1 | AC-CELL-031>AC-CELL-013.3 |
| P19 | P28 | 1 | AC-CELL-031>AC-REF-020 |
| P21 | P22 | 3 | AC-CELL-006>AC-CELL-007.1, AC-CELL-020>AC-CELL-007.1, AC-SUP-020>AC-DW-008 |
| P22 | P23 | 1 | AC-DW-010>AC-DW-001 |
| P23 | P30 | 2 | AC-DW-012>AC-EXT-001.1, AC-DW-012>AC-EXT-001.2 |
| P24 | P28 | 1 | AC-MBX-018>AC-MBX-013 |
| P25 | P26 | 4 | AC-SYS-001.2>AC-SYS-018, AC-SYS-002>AC-SYS-010, AC-SYS-013>AC-SYS-010 |
| P25 | P30 | 4 | AC-SYS-002>AC-EXT-003, AC-SYS-013>AC-EXT-001.2, AC-SYS-020>AC-EXT-001.2 |
| P25 | P34 | 4 | AC-SYS-002>AC-EVT-007, AC-SYS-002>AC-EVT-009, AC-SYS-002>AC-EVT-011 |
| P25 | P39 | 1 | AC-SYS-007>AC-SYS-004 |
| P26 | P28 | 2 | AC-SYS-016>AC-DSP-007, AC-SYS-016>AC-MBX-013 |
| P26 | P29 | 1 | AC-SYS-024>AC-REF-012 |
| P26 | P31 | 2 | AC-SYS-018>AC-SYS-017, AC-SYS-018>AC-SYS-025 |
| P26 | P35 | 1 | AC-SYS-016>AC-RT-003.1 |
| P26 | P39 | 2 | AC-SYS-012>AC-SYS-022, AC-SYS-019>AC-SYS-005 |
| P26 | P40 | 1 | AC-SYS-018>AC-SER-001.3 |
| P26 | P44 | 1 | AC-SYS-010>AC-TK-001.1 |
| P28 | P37 | 2 | AC-DSP-007>AC-CFG-012.3, AC-DSP-007>AC-CFG-012.4 |
| P29 | P31 | 3 | AC-REF-012>AC-ASK-001.1, AC-REF-012>AC-ASK-001.2, AC-REF-012>AC-ASK-001.3 |
| P32 | P47 | 1 | AC-ASK-011.5>AC-ASK-017 |
| P34 | P45 | 1 | AC-EVT-009>AC-TK-005.2 |
| P36 | P40 | 3 | AC-RT-009.1>AC-SER-001.1, AC-RT-009.1>AC-SER-001.2, AC-RT-009.1>AC-SER-001.3 |
| P40 | P41 | 3 | AC-SER-001.2>AC-SER-014, AC-SER-001.3>AC-SER-007.2, AC-SER-013>AC-REF-023 |
| P42 | P43 | 1 | AC-IO-002.3>AC-IO-008.1 |
| P48 | P52 | 2 | AT-BEH-005>AT-BEH-003, AT-BEH-008.2>AT-BEH-003 |
| P48 | P54 | 1 | AT-BEH-008.1>AT-SIG-007 |
| P48 | P55 | 1 | AT-BEH-007>AT-SUP-001 |
| P48 | P56 | 2 | AT-REF-001>AT-SER-001, AT-REF-001>AT-SER-003 |
| P49 | P50 | 6 | AT-ADP-003>AT-DSP-003, AT-ADP-010>AT-CS-001, AT-ADP-010>AT-SYS-003 |
| P49 | P52 | 2 | AT-ADP-006.2>AT-BEH-003, AT-ADP-006.2>AT-EVT-003 |
| P49 | P53 | 1 | AT-CTX-018>AT-STASH-005 |
| P49 | P54 | 4 | AT-ADP-006.1>AT-CTX-008, AT-ADP-006.1>AT-CTX-011, AT-ADP-006.1>AT-CTX-014 |
| P49 | P55 | 4 | AT-ADP-006.2>AT-BEH-024, AT-ADP-006.3>AT-SUP-005, AT-ADP-006.4>AT-SUP-001 |
| P50 | P52 | 2 | AT-SYS-007.3>AT-LOG-001, AT-SYS-010>AT-EVT-002 |
| P50 | P63 | 1 | AT-DSP-005>AT-TK-011 |
| P52 | P55 | 1 | AT-BEH-003>AT-SIG-006 |
| P52 | P64 | 5 | AT-LOG-004>AT-TK-006.1, AT-LOG-004>AT-TK-006.2, AT-LOG-004>AT-TK-006.3 |
| P53 | P52 | 2 | AT-STASH-005>AT-EVT-002, AT-STASH-005>AT-EVT-003 |
| P53 | P54 | 1 | AT-TMR-007.2>AT-CTX-008 |
| P56 | P57 | 1 | AT-RCP-008>AT-RT-004 |
| P58 | P59 | 5 | AT-DLV-001.1>AT-DLV-001.6, AT-DLV-001.2>AT-DLV-001.6, AT-DLV-001.5>AT-DLV-001.6 |
| P58 | P60 | 1 | AT-DLV-006>AT-DLV-003.2 |
| P59 | P60 | 1 | AT-DLV-004>AT-DLV-003.6 |
