# Orden de implementación

Este documento ordena las 492 filas del inventario (322 de Classic en `docs/inventory/actor-classic.tsv`, 170 de Typed en `docs/inventory/actor-typed.tsv`) en una cadena de specs chicas. Cada spec tiene como máximo 5 tareas. Cuando una unidad no entra en 5 tareas se parte en varias specs consecutivas, y cada spec nombra a la siguiente.

## La idea en una frase

Se va de las hojas del grafo (valores sin concurrencia, que se validan contra el jar oficial) hacia el sistema de actores funcionando, y recién sobre eso se monta Typed, igual que en Pekko.

## Decisión previa que cambia todo el orden: Typed sobre Classic

En Pekko, un actor tipado corre dentro de una celda clásica (`ActorAdapter`, `ActorContextAdapter`, `ActorSystemAdapter` en `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/`). Hay dos caminos:

- **(a) Replicar las capas de Pekko.** Typed se monta sobre la celda clásica portada. DeathWatch, el orden de `PostStop`, el receive timeout y la supervisión subyacente se implementan y validan una sola vez. La interop Classic↔Typed (unidad `AT-ADP`, en alcance) sale casi gratis.
- **(b) Runtime tipado nativo** con interop como capa aparte. Es más idiomático en Rust, pero cada contrato heredado se implementa y valida dos veces, y la interop queda como un puente entre dos runtimes.

**Recomendación: (a).** El objetivo es completar los dos módulos y la interop está dentro del alcance, así que (b) duplica trabajo sin sacar nada del alcance. Lo que se pierde con (a) es que la API interna de la celda Classic usa mensajes dinámicos (`Box<dyn Any + Send>`); la API pública Typed igual queda fuertemente tipada. El resumen de Typed (`docs/inventory/actor-typed.md`, pregunta 1) plantea la misma disyuntiva. **Es decisión del Owner** y el orden de abajo asume (a).

## Decisiones bloqueantes por fase

No conviene empezar una fase sin cerrar las preguntas que la bloquean. Están detalladas en los resúmenes de inventario; acá sólo se indica cuándo hacen falta.

| Antes de... | Decidir | Filas |
|---|---|---|
| Fase 1 (CFG) | Parser HOCON propio del subconjunto o crate existente; registro nombre → fábrica en lugar de reflexión | AC-CFG-016, AC-CFG-006 |
| Fase 2 (REF, MBX, DSP) | Mensajes classic como `Box<dyn Any + Send>`; runtime (tokio vs. pools propios); `BoundedMailbox` que bloquea al emisor | AC-REF-002, AC-DSP-008, AC-MBX-007 |
| Fase 2 (CELL, SUP) | Qué es un fallo: `Err`, `panic` o ambos, y cuál escala | AC-SUP-001, AC-SUP-004, AT-SUP-002 |
| Fase 2 (EVT) | Suscripción por jerarquía de tipos sin herencia; loggers como actores o backend directo | AC-EVT-003, AC-EVT-011 |
| Fase 3 (Typed) | Arquitectura (a)/(b); forma de `setup`/reuso de `Behavior` (`Arc<dyn Fn>` o `Clone`); `narrow` e igualdad de refs | AT-SYS-011, AT-BEH-007, AT-BEH-022 |
| Fase 4 | Qué piezas JVM/Java/deprecadas se excluyen (propuestas, todavía **dentro** del alcance) | ver pregunta 7 de `actor-classic.md` |

## Fases y cadena de specs

Las cifras de specs son estimaciones a partir de la cantidad de filas y se ajustan al escribir cada spec. "Oráculo" significa comparar contra el jar oficial con `tools/oracle/run.sh`; "specs portadas" significa traducir los tests de Pekko 1:1.

### Fase 1 — Valores sin concurrencia (Classic)

Se valida con tests comunes y el oráculo; no hace falta un sistema de actores.

1. **AC-PATH** — `Address`, `ActorPath`, validación de nombres (15 filas). **1 spec, ya escrita:** `docs/superpowers/plans/2026-10-30-ac-path-address.md`. Primera unidad verificable.
2. **AC-UTIL (parte que usa el núcleo)** — `Timeout`, `Done`, excepciones base, helpers de duración. 1 spec. `ByteString` y lo periférico quedan para la fase 4.
3. **AC-CFG** — HOCON, `reference.conf` embebido sin cambios, `Settings`, `Deploy` (17 filas). 2 specs: parser y carga; settings y validación de defaults (`ConfigSpec`).

### Fase 2 — Runtime Classic

Acá aparece el ciclo que detectó el inventario: la celda (CELL) necesita EventStream (EVT) y scheduler (SCH), el sistema (SYS) los crea, y las specs de CELL/SUP/DW necesitan un TestKit que a su vez necesita un sistema. Se corta así: SCH y EVT en forma mínima antes de CELL, y un TestKit mínimo (probe con `expectMsg`, `expectTerminated`, captura de logs) junto con el primer SYS. Las specs de CELL/SUP/DW se portan completas después de eso.

4. **AC-REF** — `ActorRef`, `tell`/`forward`/`sender`, `Props`, deadLetters (25 filas). 2 specs.
5. **AC-MBX** — estados del mailbox, colas, acotadas, prioridad, Stash (19 filas). 2 specs.
6. **AC-DSP** — dispatchers, executors, throughput (19 filas). 2 specs.
7. **AC-SCH mínimo + AC-EVT mínimo** — programar una tarea, publicar un evento. 1 spec.
8. **AC-CELL + AC-SYS mínimo + AC-TK mínimo** — crear, recibir, `become`, hooks, stop, con un `ActorSystem` y un `TestProbe` básicos. 2 specs.
9. **AC-SUP** (17), **AC-DW** (13). 2 + 1 specs.
10. **AC-SYS completo**, **AC-EXT**, **AC-SCH completo**, **AC-EVT completo** (logging, dead letters), **AC-TK completo**. 4-5 specs.

### Fase 3 — Typed núcleo

Asumiendo (a). Los tests usan `actor-testkit-typed`, así que su parte mínima (`ActorTestKit`, `TestProbe`) va con la primera spec, como recomienda `actor-typed.md`.

11. **AT-BEH + AT-SIG + AT-REF + AT-TK mínimo** (38 filas). 3 specs.
12. **AT-CTX** (18), **AT-SYS** (11). 2 + 1 specs.
13. **AT-TMR**, **AT-STASH**, **AT-SUP** (21, incluye la discrepancia doc/código de backoff, AT-SUP-007). 1 + 1 + 2 specs.
14. **AT-ADP** (interop), **AT-EVT**, **AT-LOG**, **AT-DSP**, **AT-EXT**, **AT-ASK**. 3 specs.

### Fase 4 — Periferia de ambos módulos

Todo esto está en alcance; va al final porque el núcleo no depende de ello. Las unidades marcadas `listed` necesitan primero extraer sus contratos (leer los cuerpos de los tests).

15. Classic: **AC-ASK** (ask, pipe, CircuitBreaker, Backoff), **AC-FSM**, **AC-RT**, **AC-CS**, **AC-SER**, **AC-IO**, resto de **AC-UTIL**, **AC-JAPI**.
16. Typed: **AT-RCP**, **AT-RT**, **AT-PS**, **AT-SPV**, **AT-CS**, **AT-SER**, **AT-DLV**, **AT-JAPI**, **AT-TK completo**.

En total, unas 40-50 specs. La cifra exacta sale de escribir cada una.

## Cómo se marca el avance

Al cerrar cada spec se actualizan las columnas del TSV de sus filas: `analysis=extracted`, `impl=done`, `validation=spec` (specs de Pekko portadas), `validation=oracle+spec` (además, golden del jar), o `validation=diff:<nota>` si queda una diferencia aceptada por el Owner. "100%" es que todas las filas de los dos TSV tengan `impl=done` y una validación que no sea `pending`, o una exclusión decidida por el Owner y anotada.
