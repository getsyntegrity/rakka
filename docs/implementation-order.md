# Orden de implementación

> **Cierre documental 2026-10-31:** el plan de unidades vigente es `docs/unit-plan.md` (77 unidades en 10 olas tras la particion documental de P05 del 2026-11-01, sin ciclos entre unidades; asignación en `docs/unit-rows.tsv`; costuras en `docs/cycles.md`). Las 50 specs generadas de abajo siguen siendo agrupaciones preliminares por unidad de inventario y no deben ejecutarse.

Este documento ordena las 785 filas del inventario (525 de Classic en `docs/inventory/actor-classic.tsv`, 260 de Typed en `docs/inventory/actor-typed.tsv`) en 50 specs **preliminares**: son agrupaciones de filas que aún requieren revisión de atomicidad y de dependencias antes de considerarse ejecutables. Cada spec tiene como máximo 5 tareas y vive en `docs/superpowers/plans/units/` (índice con estado y dependencias: `docs/superpowers/plans/units/README.md`, generado por `tools/package/generate.py`). Las 492 filas del inventario inicial se revisaron contra el código, los cuerpos de los specs y `reference.conf` de Pekko 1.7.1 (`9a54d129…`); 69 se descompusieron en hijos y se agregaron las omisiones detectadas. **La documentación completa no es implementación ni paridad**: las columnas `impl` y `validation` del inventario son la única medida de avance.

## Cómo se ejecuta

1. El Owner responde las decisiones de `docs/decisions.md` (bloque 3 y 4 primero).
2. El agente toma la spec con menor número cuyo estado sea "lista" o "pendiente de decisiones" con todas sus decisiones registradas como aceptadas. Una spec "bloqueada" no se ejecuta.
3. Por cada tarea: RED → GREEN → REFACTOR con evidencia (`superpowers:test-driven-development`), los gates de formato, build, tests, clippy y cobertura >= 90 % de las líneas nuevas o modificadas (`superpowers:verification-before-completion`), y actualización de `impl`/`validation` del inventario.
4. Una fila con decisión abierta queda `partial`; nunca se cierra sobre una recomendación no aceptada. Ninguna fila se excluye sin autorización explícita.
5. Una rama y un PR hacia `feat/pekko-rust-pilot-migration` por spec (o por unidad de trabajo de ~400 líneas, según `AGENTS.md`); el merge lo decide el Owner.

## Decisión previa que cambia todo el orden: Typed sobre Classic

En Pekko, un actor tipado corre dentro de una celda clásica (`ActorAdapter`, `ActorContextAdapter`, `ActorSystemAdapter`). Las dos opciones, (a) replicar las capas y (b) runtime tipado nativo, y la recomendación (a) están en la decisión `Q-X-8` de `docs/decisions.md`. **Es decisión del Owner y todavía no está aceptada**; el orden de abajo asume (a), por lo que las specs `AT-*` quedan "pendientes de decisiones" hasta que se acepte.

## Decisiones bloqueantes

El registro único está en `docs/decisions.md` (282 decisiones: 0 bloqueantes, 195 propuestas, 66 aceptadas, 18 aceptadas con cambios, 3 rechazadas). Las transversales `Q-X-1` a `Q-X-10` (HOCON, registro de fábricas, mensajes dinámicos, runtime, fallos, EventStream, loggers, Typed sobre Classic, `Done`/`Timeout`, límites del módulo) condicionan a casi todas las specs.

## Orden de las specs

Orden por unidad, de las hojas hacia el sistema funcionando y recién después Typed. El grafo `depends_on` del inventario es cíclico a nivel de fila (la celda necesita EventStream y scheduler; el sistema crea ambos; el testkit necesita un sistema), así que ninguna ordenación puede evitar dependencias hacia adelante. Cada una está listada en `units/README.md` con la regla de costura: la spec que consume define un trait con el contrato mínimo que necesita, lo prueba con un doble de prueba que cumple las aserciones del bloque de contrato de la fila, y lo reemplaza cuando llega la spec que lo implementa, sin reducir el contrato de esa fila. Un doble no acredita la integración con el componente sustituido. Mientras falte esa integración necesaria, la fila permanece `partial` y se identifica el componente real y la prueba pendiente (ver `docs/testing-policy.md`).

Fases: 1 valores sin concurrencia (PATH, UTIL, CFG); 2 runtime Classic (REF, MBX, DSP, SCH, EVT, TK, CELL, SYS, SUP, DW, EXT); 3 patrones y periferia Classic (ASK, FSM, RT, CS, SER, IO, JAPI); 4 Typed (BEH, SIG, REF, TK, CTX, SYS, TMR, STASH, SUP, ADP, EVT, LOG, DSP, EXT, ASK, RCP, RT, PS, SPV, CS, SER, DLV, JAPI). "Oráculo" significa comparar contra los jars oficiales o contra Pekko compilado de `upstream/pekko` con `tools/oracle/`; "specs portadas" significa traducir los tests de Pekko 1:1.

<!-- BEGIN GENERATED UNIT TABLE -->
| Unidad | Filas pendientes | Specs (orden de ejecución) |
|---|---|---|
| AC-PATH | 5 | 01 |
| AC-UTIL | 26 | 02 |
| AC-CFG | 29 | 03 |
| AC-REF | 31 | 04, 05 |
| AC-MBX | 23 | 06 |
| AC-DSP | 25 | 07 |
| AC-SCH | 16 | 08 |
| AC-EVT | 24 | 09 |
| AC-TK | 27 | 10 |
| AC-CELL | 35 | 11, 12 |
| AC-SYS | 25 | 13 |
| AC-SUP | 19 | 14 |
| AC-DW | 8 | 15 |
| AC-EXT | 5 | 16 |
| AC-ASK | 30 | 17 |
| AC-FSM | 11 | 18 |
| AC-RT | 22 | 19 |
| AC-CS | 15 | 20 |
| AC-SER | 23 | 21 |
| AC-IO | 35 | 22, 23 |
| AC-JAPI | 21 | 24 |
| AT-BEH | 26 | 25 |
| AT-SIG | 8 | 26 |
| AT-REF | 7 | 27 |
| AT-TK | 42 | 28, 29 |
| AT-CTX | 18 | 30 |
| AT-SYS | 15 | 31 |
| AT-TMR | 9 | 32 |
| AT-STASH | 6 | 33 |
| AT-SUP | 24 | 34 |
| AT-ADP | 13 | 35 |
| AT-EVT | 4 | 36 |
| AT-LOG | 5 | 37 |
| AT-DSP | 5 | 38 |
| AT-EXT | 7 | 39 |
| AT-ASK | 4 | 40 |
| AT-RCP | 10 | 41 |
| AT-RT | 8 | 42 |
| AT-PS | 3 | 43 |
| AT-SPV | 1 | 44 |
| AT-CS | 1 | 45 |
| AT-SER | 3 | 46 |
| AT-DLV | 25 | 47 |
| AT-JAPI | 16 | 48 |
<!-- END GENERATED UNIT TABLE -->

## Ya implementado (no se repite)

AC-PATH (spec previa `docs/superpowers/plans/2026-10-30-ac-path-address.md`) y la parte del núcleo de AC-UTIL (`2026-10-30-ac-util-core.md`) están implementadas con filas `partial` por decisiones abiertas. El plan `2026-10-30-ac-cfg-parser-loading.md` es el detalle de HOCON de la decisión `Q-X-1` y es la referencia de la spec de AC-CFG; no se implementó nada.
