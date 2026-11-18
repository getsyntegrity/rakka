# Specs preliminares (generadas desde el inventario)

**Estado de las specs:** las 49 specs actuales son agrupaciones preliminares de filas del inventario. Aún requieren revisión de atomicidad y de dependencias antes de considerarse ejecutables; no empezar la implementación de una unidad hasta que esa revisión la apruebe.

Orden de ejecución, dependencias resueltas y estado. Regenerar con `python3 -I tools/package/generate.py` (ver `docs/inventory/README.md`).

| # | Spec | Filas | Tareas | Estado | Depende de |
|---|---|---|---|---|---|
| 01 | [AC-PATH-1](01-ac-path-1.md) | 13 | 3 | sin decisiones abiertas (preliminar) | 02 |
| 02 | [AC-UTIL-1](02-ac-util-1.md) | 30 | 5 | sin decisiones abiertas (preliminar) | 04, 03, 22, 10, 07, 17 |
| 03 | [AC-UTIL-2](03-ac-util-2.md) | 17 | 3 | pendiente de decisiones | 02, 09 |
| 04 | [AC-CFG-1](04-ac-cfg-1.md) | 29 | 5 | sin decisiones abiertas (preliminar) | 10, 17, 14, 21, 05, 15, 22, 02, 20 |
| 05 | [AC-REF-1](05-ac-ref-1.md) | 30 | 5 | pendiente de decisiones | 01, 07, 12, 04, 18, 10, 14, 13, 22 |
| 06 | [AC-REF-2](06-ac-ref-2.md) | 1 | 1 | sin decisiones abiertas (preliminar) | 05, 14, 18 |
| 07 | [AC-MBX-1](07-ac-mbx-1.md) | 23 | 4 | pendiente de decisiones | 12, 08, 10, 04, 02, 20, 05 |
| 08 | [AC-DSP-1](08-ac-dsp-1.md) | 25 | 5 | pendiente de decisiones | 07, 04, 20, 10, 12 |
| 09 | [AC-SCH-1](09-ac-sch-1.md) | 17 | 3 | pendiente de decisiones | 08, 02, 04, 12, 07 |
| 10 | [AC-EVT-1](10-ac-evt-1.md) | 24 | 4 | pendiente de decisiones | 02, 14, 16, 05, 04, 11, 12, 01, 19, 07, 25 |
| 11 | [AC-TK-1](11-ac-tk-1.md) | 27 | 5 | pendiente de decisiones | 08, 14, 15, 04, 17, 10, 05, 12, 07, 19, 09, 25, 22, 23 |
| 12 | [AC-CELL-1](12-ac-cell-1.md) | 30 | 5 | pendiente de decisiones | 15, 07, 16, 10, 05, 09, 08 |
| 13 | [AC-CELL-2](13-ac-cell-2.md) | 9 | 2 | pendiente de decisiones | 04, 12, 10, 07, 05, 22 |
| 14 | [AC-SYS-1](14-ac-sys-1.md) | 27 | 5 | pendiente de decisiones | 04, 17, 10, 12, 15, 21, 08, 05, 11, 16, 09, 01, 07, 20, 18, 22, 02 |
| 15 | [AC-SUP-1](15-ac-sup-1.md) | 21 | 4 | pendiente de decisiones | 12, 01, 10, 07, 04, 14 |
| 16 | [AC-DW-1](16-ac-dw-1.md) | 8 | 2 | pendiente de decisiones | 05, 12, 15, 17, 07 |
| 17 | [AC-EXT-1](17-ac-ext-1.md) | 5 | 1 | pendiente de decisiones | 14, 04 |
| 18 | [AC-ASK-1](18-ac-ask-1.md) | 30 | 5 | pendiente de decisiones | 02, 14, 05, 09, 01, 04, 17, 15 |
| 19 | [AC-FSM-1](19-ac-fsm-1.md) | 11 | 2 | pendiente de decisiones | 12, 20, 09, 04 |
| 20 | [AC-RT-1](20-ac-rt-1.md) | 22 | 4 | pendiente de decisiones | 05, 18, 15, 12, 04, 07, 22, 09, 08 |
| 21 | [AC-CS-1](21-ac-cs-1.md) | 15 | 3 | pendiente de decisiones | 04, 09, 14 |
| 22 | [AC-SER-1](22-ac-ser-1.md) | 23 | 4 | pendiente de decisiones | 04, 01, 05, 14, 24, 17 |
| 23 | [AC-IO-1](23-ac-io-1.md) | 30 | 5 | pendiente de decisiones | 14, 17, 24, 04, 08, 15 |
| 24 | [AC-IO-2](24-ac-io-2.md) | 5 | 1 | pendiente de decisiones | 23, 08 |
| 25 | [AC-JAPI-1](25-ac-japi-1.md) | 21 | 4 | pendiente de decisiones | 03, 12, 19, 15, 05, 10, 01, 14, 09, 18, 22, 23 |
| 26 | [AT-BEH-1](26-at-beh-1.md) | 26 | 5 | pendiente de decisiones | 27, 37, 35, 38, 33, 34, 28 |
| 27 | [AT-SIG-1](27-at-sig-1.md) | 8 | 2 | pendiente de decisiones | 35, 31, 26 |
| 28 | [AT-REF-1](28-at-ref-1.md) | 7 | 2 | pendiente de decisiones | 47, 26, 41, 40 |
| 29 | [AT-TK-1](29-at-tk-1.md) | 30 | 5 | pendiente de decisiones | 04, 32, 30, 31, 41, 46, 38, 42, 33, 28, 37, 11 |
| 30 | [AT-TK-2](30-at-tk-2.md) | 12 | 2 | pendiente de decisiones | 29, 09, 11, 32, 33, 37, 22, 47, 02, 04 |
| 31 | [AT-CTX-1](31-at-ctx-1.md) | 18 | 3 | pendiente de decisiones | 28, 32, 39, 27, 41, 26, 37, 35, 36, 49, 34 |
| 32 | [AT-SYS-1](32-at-sys-1.md) | 15 | 3 | pendiente de decisiones | 36, 28, 34, 46, 38, 37 |
| 33 | [AT-TMR-1](33-at-tmr-1.md) | 9 | 2 | pendiente de decisiones | 26, 31 |
| 34 | [AT-STASH-1](34-at-stash-1.md) | 6 | 1 | pendiente de decisiones | 26, 37, 35 |
| 35 | [AT-SUP-1](35-at-sup-1.md) | 24 | 4 | pendiente de decisiones | 27, 26, 33, 31, 32, 34, 37, 38, 36 |
| 36 | [AT-ADP-1](36-at-adp-1.md) | 13 | 3 | pendiente de decisiones | 32, 35, 39, 31, 26, 37, 40, 46 |
| 37 | [AT-EVT-1](37-at-evt-1.md) | 4 | 1 | pendiente de decisiones | 26 |
| 38 | [AT-LOG-1](38-at-log-1.md) | 5 | 1 | pendiente de decisiones | 26, 31, 36, 39, 29, 30 |
| 39 | [AT-DSP-1](39-at-dsp-1.md) | 5 | 1 | pendiente de decisiones | 31, 38, 32, 30 |
| 40 | [AT-EXT-1](40-at-ext-1.md) | 7 | 2 | pendiente de decisiones | 36 |
| 41 | [AT-ASK-1](41-at-ask-1.md) | 4 | 1 | pendiente de decisiones | 28, 37 |
| 42 | [AT-RCP-1](42-at-rcp-1.md) | 10 | 2 | pendiente de decisiones | 40, 47, 31, 43 |
| 43 | [AT-RT-1](43-at-rt-1.md) | 8 | 2 | pendiente de decisiones | 26, 31, 39, 42, 37 |
| 44 | [AT-PS-1](44-at-ps-1.md) | 3 | 1 | pendiente de decisiones | 31, 42, 37 |
| 45 | [AT-SPV-1](45-at-spv-1.md) | 1 | 1 | pendiente de decisiones | 31, 41 |
| 46 | [AT-CS-1](46-at-cs-1.md) | 1 | 1 | sin decisiones abiertas (preliminar) | - |
| 47 | [AT-SER-1](47-at-ser-1.md) | 3 | 1 | pendiente de decisiones | 28, 42 |
| 48 | [AT-DLV-1](48-at-dlv-1.md) | 25 | 5 | pendiente de decisiones | 31, 26, 42, 40 |
| 49 | [AT-JAPI-1](49-at-japi-1.md) | 16 | 3 | pendiente de decisiones | 26, 35, 27, 38, 31, 41, 33, 34, 43, 36 |

## Ciclos y dependencias hacia adelante

El grafo `depends_on` del inventario es cíclico a nivel de fila (Classic: CELL, SYS, EVT, SCH, TK se necesitan entre sí), por eso el orden es por unidad. 112 dependencias de spec van hacia una spec posterior. Regla de costura para todas: la spec que consume define un trait con el contrato mínimo que necesita, lo prueba con un doble de prueba que cumple las aserciones del bloque de contrato de la fila, y cuando llega la spec que lo implementa el doble se reemplaza y se re-ejecutan los tests; nada de eso reduce el contrato de la fila posterior. Lista:

- `AC-PATH-1` necesita `AC-UTIL-1`.
- `AC-UTIL-1` necesita `AC-CFG-1`.
- `AC-UTIL-1` necesita `AC-UTIL-2`.
- `AC-UTIL-1` necesita `AC-SER-1`.
- `AC-UTIL-1` necesita `AC-EVT-1`.
- `AC-UTIL-1` necesita `AC-MBX-1`.
- `AC-UTIL-1` necesita `AC-EXT-1`.
- `AC-UTIL-2` necesita `AC-SCH-1`.
- `AC-CFG-1` necesita `AC-EVT-1`.
- `AC-CFG-1` necesita `AC-EXT-1`.
- `AC-CFG-1` necesita `AC-SYS-1`.
- `AC-CFG-1` necesita `AC-CS-1`.
- `AC-CFG-1` necesita `AC-REF-1`.
- `AC-CFG-1` necesita `AC-SUP-1`.
- `AC-CFG-1` necesita `AC-SER-1`.
- `AC-CFG-1` necesita `AC-RT-1`.
- `AC-REF-1` necesita `AC-MBX-1`.
- `AC-REF-1` necesita `AC-CELL-1`.
- `AC-REF-1` necesita `AC-ASK-1`.
- `AC-REF-1` necesita `AC-EVT-1`.
- `AC-REF-1` necesita `AC-SYS-1`.
- `AC-REF-1` necesita `AC-CELL-2`.
- `AC-REF-1` necesita `AC-SER-1`.
- `AC-REF-2` necesita `AC-SYS-1`.
- `AC-REF-2` necesita `AC-ASK-1`.
- `AC-MBX-1` necesita `AC-CELL-1`.
- `AC-MBX-1` necesita `AC-DSP-1`.
- `AC-MBX-1` necesita `AC-EVT-1`.
- `AC-MBX-1` necesita `AC-RT-1`.
- `AC-DSP-1` necesita `AC-RT-1`.
- `AC-DSP-1` necesita `AC-EVT-1`.
- `AC-DSP-1` necesita `AC-CELL-1`.
- `AC-SCH-1` necesita `AC-CELL-1`.
- `AC-EVT-1` necesita `AC-SYS-1`.
- `AC-EVT-1` necesita `AC-DW-1`.
- `AC-EVT-1` necesita `AC-TK-1`.
- `AC-EVT-1` necesita `AC-CELL-1`.
- `AC-EVT-1` necesita `AC-FSM-1`.
- `AC-EVT-1` necesita `AC-JAPI-1`.
- `AC-TK-1` necesita `AC-SYS-1`.
- `AC-TK-1` necesita `AC-SUP-1`.
- `AC-TK-1` necesita `AC-EXT-1`.
- `AC-TK-1` necesita `AC-CELL-1`.
- `AC-TK-1` necesita `AC-FSM-1`.
- `AC-TK-1` necesita `AC-JAPI-1`.
- `AC-TK-1` necesita `AC-SER-1`.
- `AC-TK-1` necesita `AC-IO-1`.
- `AC-CELL-1` necesita `AC-SUP-1`.
- `AC-CELL-1` necesita `AC-DW-1`.
- `AC-CELL-2` necesita `AC-SER-1`.
- `AC-SYS-1` necesita `AC-EXT-1`.
- `AC-SYS-1` necesita `AC-SUP-1`.
- `AC-SYS-1` necesita `AC-CS-1`.
- `AC-SYS-1` necesita `AC-DW-1`.
- `AC-SYS-1` necesita `AC-RT-1`.
- `AC-SYS-1` necesita `AC-ASK-1`.
- `AC-SYS-1` necesita `AC-SER-1`.
- `AC-DW-1` necesita `AC-EXT-1`.
- `AC-FSM-1` necesita `AC-RT-1`.
- `AC-RT-1` necesita `AC-SER-1`.
- `AC-SER-1` necesita `AC-IO-2`.
- `AC-IO-1` necesita `AC-IO-2`.
- `AT-BEH-1` necesita `AT-SIG-1`.
- `AT-BEH-1` necesita `AT-EVT-1`.
- `AT-BEH-1` necesita `AT-SUP-1`.
- `AT-BEH-1` necesita `AT-LOG-1`.
- `AT-BEH-1` necesita `AT-TMR-1`.
- `AT-BEH-1` necesita `AT-STASH-1`.
- `AT-BEH-1` necesita `AT-REF-1`.
- `AT-SIG-1` necesita `AT-SUP-1`.
- `AT-SIG-1` necesita `AT-CTX-1`.
- `AT-REF-1` necesita `AT-SER-1`.
- `AT-REF-1` necesita `AT-ASK-1`.
- `AT-REF-1` necesita `AT-EXT-1`.
- `AT-TK-1` necesita `AT-SYS-1`.
- `AT-TK-1` necesita `AT-TK-2`.
- `AT-TK-1` necesita `AT-CTX-1`.
- `AT-TK-1` necesita `AT-ASK-1`.
- `AT-TK-1` necesita `AT-CS-1`.
- `AT-TK-1` necesita `AT-LOG-1`.
- `AT-TK-1` necesita `AT-RCP-1`.
- `AT-TK-1` necesita `AT-TMR-1`.
- `AT-TK-1` necesita `AT-EVT-1`.
- `AT-TK-2` necesita `AT-SYS-1`.
- `AT-TK-2` necesita `AT-TMR-1`.
- `AT-TK-2` necesita `AT-EVT-1`.
- `AT-TK-2` necesita `AT-SER-1`.
- `AT-CTX-1` necesita `AT-SYS-1`.
- `AT-CTX-1` necesita `AT-DSP-1`.
- `AT-CTX-1` necesita `AT-ASK-1`.
- `AT-CTX-1` necesita `AT-EVT-1`.
- `AT-CTX-1` necesita `AT-SUP-1`.
- `AT-CTX-1` necesita `AT-ADP-1`.
- `AT-CTX-1` necesita `AT-JAPI-1`.
- `AT-CTX-1` necesita `AT-STASH-1`.
- `AT-SYS-1` necesita `AT-ADP-1`.
- `AT-SYS-1` necesita `AT-STASH-1`.
- `AT-SYS-1` necesita `AT-CS-1`.
- `AT-SYS-1` necesita `AT-LOG-1`.
- `AT-SYS-1` necesita `AT-EVT-1`.
- `AT-STASH-1` necesita `AT-EVT-1`.
- `AT-STASH-1` necesita `AT-SUP-1`.
- `AT-SUP-1` necesita `AT-EVT-1`.
- `AT-SUP-1` necesita `AT-LOG-1`.
- `AT-SUP-1` necesita `AT-ADP-1`.
- `AT-ADP-1` necesita `AT-DSP-1`.
- `AT-ADP-1` necesita `AT-EVT-1`.
- `AT-ADP-1` necesita `AT-EXT-1`.
- `AT-ADP-1` necesita `AT-CS-1`.
- `AT-LOG-1` necesita `AT-DSP-1`.
- `AT-RCP-1` necesita `AT-SER-1`.
- `AT-RCP-1` necesita `AT-RT-1`.
