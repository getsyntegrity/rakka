# Avance del port

Generado por `tools/package/generate.py` desde `docs/inventory/actor-*.tsv` (columna `impl`), `docs/unit-rows.tsv` y `docs/unit-plan.md`. **No editar a mano.**
`done` = todas las aserciones del contrato cubiertas por casos grabados de Pekko 2.0.0 que pasan y sin decisión abierta; `partial` = algunas cubiertas o una `Q:` abierta; `pending` = ninguna (o sin comparar); `excluded` = fila excluida por autorización del Owner: **no cuenta como done** y sale del denominador. Los porcentajes se calculan sobre las filas **en alcance** (total menos excluidas).

## Resumen

| Módulo | En alcance | done | partial | pending | excluded | % done | % done + partial |
|---|---|---|---|---|---|---|---|
| Classic (`rakka-actor`) | 521 | 47 | 128 | 346 | 4 | 9.0 % | 33.6 % |
| Typed (`rakka-actor-typed`) | 260 | 0 | 0 | 260 | 0 | 0.0 % | 0.0 % |
| **Total** | 781 | 47 | 128 | 606 | 4 | 6.0 % | 22.4 % |

## Por ola

| Ola | En alcance | done | partial | pending | excluded | % done | % done + partial |
|---|---|---|---|---|---|---|---|
| 0 - valores sin runtime | 57 | 36 | 20 | 1 | 4 | 63.2 % | 98.2 % |
| 1 - configuracion, eventos y scheduler | 70 | 4 | 40 | 26 | 0 | 5.7 % | 62.9 % |
| 2 - nucleo del runtime Classic | 134 | 7 | 56 | 71 | 0 | 5.2 % | 47.0 % |
| 3 - sistema, proveedor, referencias y extensiones | 45 | 0 | 11 | 34 | 0 | 0.0 % | 24.4 % |
| 4 - patrones, routers y logging | 81 | 0 | 0 | 81 | 0 | 0.0 % | 0.0 % |
| 5 - apagado, serializacion e IO | 84 | 0 | 1 | 83 | 0 | 0.0 % | 1.2 % |
| 6 - testkit Classic y superficie Java | 50 | 0 | 0 | 50 | 0 | 0.0 % | 0.0 % |
| 7 - Typed: nucleo, adaptador y sistema | 105 | 0 | 0 | 105 | 0 | 0.0 % | 0.0 % |
| 8 - Typed: contexto, supervision, receptionist y entrega fiable | 96 | 0 | 0 | 96 | 0 | 0.0 % | 0.0 % |
| 9 - Typed: javadsl y testkits | 59 | 0 | 0 | 59 | 0 | 0.0 % | 0.0 % |

## Por unidad del plan

| Unidad | Ola | En alcance | done | partial | pending | excluded | % done | % done + partial | Áreas comparadas |
|---|---|---|---|---|---|---|---|---|---|
| P01 PATH | 1 | 16 | 3 | 13 | 0 | 0 | 18.8 % | 100.0 % | sí |
| P02 UTIL-NUCLEO | 0 | 8 | 6 | 2 | 0 | 1 | 75.0 % | 100.0 % | sí |
| P03 UTIL-ESTRUCTURAS | 0 | 16 | 14 | 2 | 0 | 0 | 87.5 % | 100.0 % | sí |
| P04 UTIL-BYTES | 0 | 9 | 4 | 5 | 0 | 0 | 44.4 % | 100.0 % | sí |
| P05 UTIL-GLOB | 0 | 1 | 1 | 0 | 0 | 0 | 100.0 % | 100.0 % | sí |
| P06 UTIL-PLATAFORMA | 0 | 8 | 4 | 3 | 1 | 0 | 50.0 % | 87.5 % | sí |
| P07 CFG-HOCON | 1 | 12 | 1 | 11 | 0 | 0 | 8.3 % | 100.0 % | parcial |
| P08 CFG-AJUSTES | 1 | 10 | 0 | 10 | 0 | 0 | 0.0 % | 100.0 % | no |
| P09 CFG-ENTORNO | 1 | 3 | 0 | 3 | 0 | 0 | 0.0 % | 100.0 % | no |
| P10 CFG-DEPLOY | 1 | 3 | 0 | 3 | 0 | 0 | 0.0 % | 100.0 % | no |
| P11 EVT-BUS | 1 | 11 | 0 | 0 | 11 | 0 | 0.0 % | 0.0 % | no |
| P12 SCH-RUEDA | 1 | 13 | 0 | 0 | 13 | 0 | 0.0 % | 0.0 % | no |
| P13 MBX-COLAS | 2 | 12 | 0 | 3 | 9 | 0 | 0.0 % | 25.0 % | sí |
| P14 REF-BASE | 2 | 15 | 0 | 6 | 9 | 0 | 0.0 % | 40.0 % | sí |
| P15 MBX-NUCLEO | 2 | 7 | 0 | 4 | 3 | 0 | 0.0 % | 57.1 % | sí |
| P16 DSP-EJECUTORES | 2 | 19 | 0 | 0 | 19 | 0 | 0.0 % | 0.0 % | sí |
| P17 REF-MUERTOS | 2 | 4 | 0 | 3 | 1 | 0 | 0.0 % | 75.0 % | sí |
| P18 CELL-HIJOS | 2 | 6 | 1 | 4 | 1 | 0 | 16.7 % | 83.3 % | sí |
| P19 CELL-EJECUCION | 2 | 11 | 0 | 4 | 7 | 0 | 0.0 % | 36.4 % | sí |
| P20 SUP-ESTRATEGIAS | 2 | 10 | 0 | 6 | 4 | 0 | 0.0 % | 60.0 % | sí |
| P21 SUP-FALLO | 2 | 11 | 1 | 7 | 3 | 0 | 9.1 % | 72.7 % | sí |
| P22 CELL-REINICIO-PARADA | 2 | 21 | 2 | 13 | 6 | 0 | 9.5 % | 71.4 % | sí |
| P23 DW-VIGILANCIA | 2 | 13 | 3 | 6 | 4 | 0 | 23.1 % | 69.2 % | parcial |
| P24 MBX-STASH | 2 | 5 | 0 | 0 | 5 | 0 | 0.0 % | 0.0 % | sí |
| P25 SYS-GUARDIANES | 3 | 12 | 0 | 2 | 10 | 0 | 0.0 % | 16.7 % | sí |
| P26 SYS-PROVEEDOR | 3 | 8 | 0 | 3 | 5 | 0 | 0.0 % | 37.5 % | sí |
| P27 SCH-ACTORES | 3 | 4 | 0 | 0 | 4 | 0 | 0.0 % | 0.0 % | no |
| P28 SELECCION-CREACION | 3 | 9 | 0 | 3 | 6 | 0 | 0.0 % | 33.3 % | sí |
| P29 REF-SELECCION | 3 | 6 | 0 | 3 | 3 | 0 | 0.0 % | 50.0 % | sí |
| P30 EXT | 3 | 5 | 0 | 0 | 5 | 0 | 0.0 % | 0.0 % | no |
| P31 ASK | 4 | 17 | 0 | 0 | 17 | 0 | 0.0 % | 0.0 % | parcial |
| P32 CIRCUIT-BREAKER-BACKOFF | 4 | 15 | 0 | 0 | 15 | 0 | 0.0 % | 0.0 % | no |
| P33 FSM | 4 | 11 | 0 | 0 | 11 | 0 | 0.0 % | 0.0 % | no |
| P34 EVT-LOGGING | 4 | 11 | 0 | 0 | 11 | 0 | 0.0 % | 0.0 % | no |
| P35 RT-NUCLEO | 4 | 9 | 0 | 0 | 9 | 0 | 0.0 % | 0.0 % | no |
| P36 RT-ESTRATEGIAS | 4 | 9 | 0 | 0 | 9 | 0 | 0.0 % | 0.0 % | parcial |
| P37 RT-CONFIG | 4 | 9 | 0 | 0 | 9 | 0 | 0.0 % | 0.0 % | no |
| P38 CS-NUCLEO | 5 | 13 | 0 | 0 | 13 | 0 | 0.0 % | 0.0 % | no |
| P39 TERMINACION | 5 | 10 | 0 | 1 | 9 | 0 | 0.0 % | 10.0 % | parcial |
| P40 SER-NUCLEO | 5 | 18 | 0 | 0 | 18 | 0 | 0.0 % | 0.0 % | no |
| P41 SER-SISTEMA | 5 | 8 | 0 | 0 | 8 | 0 | 0.0 % | 0.0 % | parcial |
| P42 IO-TCP | 5 | 16 | 0 | 0 | 16 | 0 | 0.0 % | 0.0 % | no |
| P43 IO-UDP-DNS | 5 | 19 | 0 | 0 | 19 | 0 | 0.0 % | 0.0 % | no |
| P44 TK-DISPATCHER-REF | 6 | 5 | 0 | 0 | 5 | 0 | 0.0 % | 0.0 % | no |
| P45 TK-API | 6 | 22 | 0 | 0 | 22 | 0 | 0.0 % | 0.0 % | no |
| P46 JAPI-ACTORES | 6 | 16 | 0 | 0 | 16 | 0 | 0.0 % | 0.0 % | no |
| P47 JAPI-EXTRA | 6 | 7 | 0 | 0 | 7 | 0 | 0.0 % | 0.0 % | no |
| P48 T-COMPORTAMIENTO-NUCLEO | 7 | 21 | 0 | 0 | 21 | 0 | 0.0 % | 0.0 % | parcial |
| P49 T-ADAPTADOR-CLASICO | 7 | 19 | 0 | 0 | 19 | 0 | 0.0 % | 0.0 % | parcial |
| P50 T-SISTEMA-TIPADO | 7 | 25 | 0 | 0 | 25 | 0 | 0.0 % | 0.0 % | parcial |
| P51 T-ASK-TIPADO | 7 | 5 | 0 | 0 | 5 | 0 | 0.0 % | 0.0 % | parcial |
| P52 T-OBSERVABILIDAD | 7 | 14 | 0 | 0 | 14 | 0 | 0.0 % | 0.0 % | parcial |
| P53 T-INTERCEPTORES-TIMERS-STASH | 7 | 21 | 0 | 0 | 21 | 0 | 0.0 % | 0.0 % | no |
| P54 T-CONTEXTO-SENALES | 8 | 14 | 0 | 0 | 14 | 0 | 0.0 % | 0.0 % | parcial |
| P55 T-SUPERVISION | 8 | 31 | 0 | 0 | 31 | 0 | 0.0 % | 0.0 % | parcial |
| P56 T-RECEPCIONISTA-SERIALIZACION | 8 | 14 | 0 | 0 | 14 | 0 | 0.0 % | 0.0 % | parcial |
| P57 T-ROUTERS-PUBSUB-SPAWN | 8 | 12 | 0 | 0 | 12 | 0 | 0.0 % | 0.0 % | no |
| P58 T-ENTREGA-FIABLE-BASE | 8 | 13 | 0 | 0 | 13 | 0 | 0.0 % | 0.0 % | no |
| P59 T-ENTREGA-FIABLE-DURABLE | 8 | 6 | 0 | 0 | 6 | 0 | 0.0 % | 0.0 % | no |
| P60 T-ENTREGA-FIABLE-WORKPULLING | 8 | 6 | 0 | 0 | 6 | 0 | 0.0 % | 0.0 % | no |
| P61 T-JAVADSL | 9 | 17 | 0 | 0 | 17 | 0 | 0.0 % | 0.0 % | no |
| P62 T-TK-SONDA | 9 | 17 | 0 | 0 | 17 | 0 | 0.0 % | 0.0 % | no |
| P63 T-TK-COMPORTAMIENTO | 9 | 19 | 0 | 0 | 19 | 0 | 0.0 % | 0.0 % | no |
| P64 T-TK-LOGGING | 9 | 6 | 0 | 0 | 6 | 0 | 0.0 % | 0.0 % | no |
| P65 UTIL-HASHCODE | 0 | 1 | 1 | 0 | 0 | 0 | 100.0 % | 100.0 % | sí |
| P66 UTIL-PEKKOVERSION | 0 | 1 | 1 | 0 | 0 | 0 | 100.0 % | 100.0 % | sí |
| P67 UTIL-VERSION | 0 | 1 | 1 | 0 | 0 | 0 | 100.0 % | 100.0 % | sí |
| P68 UTIL-PRETTYDURATION | 0 | 1 | 1 | 0 | 0 | 0 | 100.0 % | 100.0 % | sí |
| P69 UTIL-UUID-ORDEN | 0 | 1 | 1 | 0 | 0 | 0 | 100.0 % | 100.0 % | sí |
| P70 UTIL-RELOJ | 0 | 1 | 1 | 0 | 0 | 0 | 100.0 % | 100.0 % | sí |
| P71 UTIL-REGISTRO-FABRICAS | 0 | 0 | 0 | 0 | 0 | 2 | - | - | sí |
| P72 UTIL-MANIFEST | 3 | 1 | 0 | 0 | 1 | 0 | 0.0 % | 0.0 % | sí |
| P73 UTIL-JFR | 1 | 2 | 0 | 0 | 2 | 0 | 0.0 % | 0.0 % | sí |
| P74 UTIL-TIPOS-DESERIALIZACION | 0 | 0 | 0 | 0 | 0 | 1 | - | - | sí |
| P75 UTIL-ORIGEN-CLOSURES | 0 | 1 | 1 | 0 | 0 | 0 | 100.0 % | 100.0 % | sí |
| P76 UTIL-EQUIVALENCIAS | 0 | 5 | 0 | 5 | 0 | 0 | 0.0 % | 100.0 % | sí |
| P77 UTIL-FRONTERAS | 0 | 3 | 0 | 3 | 0 | 0 | 0.0 % | 100.0 % | sí |

## Más avanzadas y más atrasadas

Orden por (done + ½·partial) / filas. Entre las atrasadas hay empates en 0 %: se desempata por el tamaño de la unidad.

**5 más avanzadas:** P05 UTIL-GLOB (1 done, 0 partial de 1), P65 UTIL-HASHCODE (1 done, 0 partial de 1), P66 UTIL-PEKKOVERSION (1 done, 0 partial de 1), P67 UTIL-VERSION (1 done, 0 partial de 1), P68 UTIL-PRETTYDURATION (1 done, 0 partial de 1)

**5 más atrasadas:** P55 T-SUPERVISION (31 filas), P50 T-SISTEMA-TIPADO (25 filas), P45 TK-API (22 filas), P48 T-COMPORTAMIENTO-NUCLEO (21 filas), P53 T-INTERCEPTORES-TIMERS-STASH (21 filas). Unidades en 0 %: 41 de 75.

## Por área del inventario y comparación aserción por aserción

Un área figura como comparada cuando se contrastaron las aserciones de cada una de sus filas con los casos grabados (`docs/inventory/compared-areas.txt`). En las áreas **sin comparar** todas las filas siguen `pending` aunque algún caso grabado las roce: falta hacer la comparación al tomar su unidad.

| Área | En alcance | done | partial | pending | excluded | % done | % done + partial | Comparada |
|---|---|---|---|---|---|---|---|---|
| AC-ASK | 30 | 0 | 0 | 30 | 0 | 0.0 % | 0.0 % | no |
| AC-CELL | 39 | 0 | 20 | 19 | 0 | 0.0 % | 51.3 % | sí |
| AC-CFG | 29 | 0 | 26 | 3 | 0 | 0.0 % | 89.7 % | no |
| AC-CS | 15 | 0 | 0 | 15 | 0 | 0.0 % | 0.0 % | no |
| AC-DSP | 25 | 0 | 3 | 22 | 0 | 0.0 % | 12.0 % | sí |
| AC-DW | 13 | 5 | 5 | 3 | 0 | 38.5 % | 76.9 % | sí |
| AC-EVT | 24 | 0 | 0 | 24 | 0 | 0.0 % | 0.0 % | no |
| AC-EXT | 5 | 0 | 0 | 5 | 0 | 0.0 % | 0.0 % | no |
| AC-FSM | 11 | 0 | 0 | 11 | 0 | 0.0 % | 0.0 % | no |
| AC-IO | 35 | 0 | 0 | 35 | 0 | 0.0 % | 0.0 % | no |
| AC-JAPI | 21 | 0 | 0 | 21 | 0 | 0.0 % | 0.0 % | no |
| AC-MBX | 23 | 0 | 8 | 15 | 0 | 0.0 % | 34.8 % | sí |
| AC-PATH | 16 | 3 | 13 | 0 | 0 | 18.8 % | 100.0 % | sí |
| AC-REF | 32 | 1 | 13 | 18 | 0 | 3.1 % | 43.8 % | sí |
| AC-RT | 22 | 0 | 0 | 22 | 0 | 0.0 % | 0.0 % | no |
| AC-SCH | 17 | 0 | 0 | 17 | 0 | 0.0 % | 0.0 % | no |
| AC-SER | 23 | 0 | 0 | 23 | 0 | 0.0 % | 0.0 % | no |
| AC-SUP | 22 | 1 | 13 | 8 | 0 | 4.5 % | 63.6 % | sí |
| AC-SYS | 27 | 0 | 6 | 21 | 0 | 0.0 % | 22.2 % | sí |
| AC-TK | 27 | 0 | 0 | 27 | 0 | 0.0 % | 0.0 % | no |
| AC-UTIL | 65 | 37 | 21 | 7 | 4 | 56.9 % | 89.2 % | sí |
| AT-ADP | 13 | 0 | 0 | 13 | 0 | 0.0 % | 0.0 % | no |
| AT-ASK | 4 | 0 | 0 | 4 | 0 | 0.0 % | 0.0 % | no |
| AT-BEH | 26 | 0 | 0 | 26 | 0 | 0.0 % | 0.0 % | no |
| AT-CS | 1 | 0 | 0 | 1 | 0 | 0.0 % | 0.0 % | no |
| AT-CTX | 18 | 0 | 0 | 18 | 0 | 0.0 % | 0.0 % | no |
| AT-DLV | 25 | 0 | 0 | 25 | 0 | 0.0 % | 0.0 % | no |
| AT-DSP | 5 | 0 | 0 | 5 | 0 | 0.0 % | 0.0 % | no |
| AT-EVT | 4 | 0 | 0 | 4 | 0 | 0.0 % | 0.0 % | no |
| AT-EXT | 7 | 0 | 0 | 7 | 0 | 0.0 % | 0.0 % | no |
| AT-JAPI | 16 | 0 | 0 | 16 | 0 | 0.0 % | 0.0 % | no |
| AT-LOG | 5 | 0 | 0 | 5 | 0 | 0.0 % | 0.0 % | no |
| AT-PS | 3 | 0 | 0 | 3 | 0 | 0.0 % | 0.0 % | no |
| AT-RCP | 10 | 0 | 0 | 10 | 0 | 0.0 % | 0.0 % | no |
| AT-REF | 7 | 0 | 0 | 7 | 0 | 0.0 % | 0.0 % | no |
| AT-RT | 8 | 0 | 0 | 8 | 0 | 0.0 % | 0.0 % | no |
| AT-SER | 3 | 0 | 0 | 3 | 0 | 0.0 % | 0.0 % | no |
| AT-SIG | 8 | 0 | 0 | 8 | 0 | 0.0 % | 0.0 % | no |
| AT-SPV | 1 | 0 | 0 | 1 | 0 | 0.0 % | 0.0 % | no |
| AT-STASH | 6 | 0 | 0 | 6 | 0 | 0.0 % | 0.0 % | no |
| AT-SUP | 24 | 0 | 0 | 24 | 0 | 0.0 % | 0.0 % | no |
| AT-SYS | 15 | 0 | 0 | 15 | 0 | 0.0 % | 0.0 % | no |
| AT-TK | 42 | 0 | 0 | 42 | 0 | 0.0 % | 0.0 % | no |
| AT-TMR | 9 | 0 | 0 | 9 | 0 | 0.0 % | 0.0 % | no |

## Filas excluidas por el Owner

4 fila(s) con exclusión autorizada (no cuentan como done ni en el alcance; la nota de la fila cita la decisión): AC-UTIL-004.3, AC-UTIL-016.7, AC-UTIL-016.12, AC-UTIL-016.13

## Filas con nota «revisar contra 2.0.0»

540 filas (24 done, 77 partial, 439 pending): su texto citado cambió entre 1.7.1 y 2.0.0 (ver `docs/inventory/pekko-2.0.0-reanchor.md`). Se revisan cuando un slice toma su unidad; su estado no se tocó al reanclar.

## Casos grabados y filas que cubren

`cases/coverage.tsv` declara las filas que cubre cada uno de los 3058 casos (`tools/parity/apply_comparison.py`). Base `cited`: el caso lo cita una fila comparada; `same-group`: hereda las filas de su grupo de nombres; `none`: **256 casos sin fila asignada**, a mapear cuando un slice tome su unidad.
