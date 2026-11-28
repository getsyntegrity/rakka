# Specs preliminares (generadas desde el inventario)

**Estado de las specs:** las 49 specs actuales son agrupaciones preliminares de filas del inventario. Aún requieren revisión de atomicidad y de dependencias antes de considerarse ejecutables; no empezar la implementación de una unidad hasta que esa revisión la apruebe.

Orden de ejecución, dependencias resueltas y estado. Regenerar con `python3 -I tools/package/generate.py` (ver `docs/inventory/README.md`).

| # | Spec | Filas | Tareas | Estado | Depende de |
|---|---|---|---|---|---|
| 01 | [AC-PATH-1](01-ac-path-1.md) | 7 | 2 | pendiente de decisiones | 02 |
| 02 | [AC-UTIL-1](02-ac-util-1.md) | 27 | 5 | pendiente de decisiones | 03, 21, 09, 16, 08 |
| 03 | [AC-CFG-1](03-ac-cfg-1.md) | 29 | 5 | pendiente de decisiones | 09, 16, 13, 20, 04, 14, 21, 02, 19 |
| 04 | [AC-REF-1](04-ac-ref-1.md) | 30 | 5 | pendiente de decisiones | 06, 11, 03, 01, 17, 09, 13, 12, 21 |
| 05 | [AC-REF-2](05-ac-ref-2.md) | 1 | 1 | sin decisiones abiertas (preliminar) | 04, 13, 17 |
| 06 | [AC-MBX-1](06-ac-mbx-1.md) | 23 | 4 | pendiente de decisiones | 11, 07, 09, 03, 19, 04 |
| 07 | [AC-DSP-1](07-ac-dsp-1.md) | 25 | 5 | pendiente de decisiones | 06, 03, 19, 09, 11 |
| 08 | [AC-SCH-1](08-ac-sch-1.md) | 17 | 3 | pendiente de decisiones | 07, 03, 11, 06 |
| 09 | [AC-EVT-1](09-ac-evt-1.md) | 24 | 4 | pendiente de decisiones | 02, 13, 15, 04, 03, 10, 11, 01, 18, 06, 24 |
| 10 | [AC-TK-1](10-ac-tk-1.md) | 27 | 5 | pendiente de decisiones | 07, 13, 14, 03, 16, 09, 04, 11, 06, 18, 08, 24, 21, 22 |
| 11 | [AC-CELL-1](11-ac-cell-1.md) | 30 | 5 | pendiente de decisiones | 14, 06, 15, 09, 04, 08, 07, 03 |
| 12 | [AC-CELL-2](12-ac-cell-2.md) | 7 | 2 | pendiente de decisiones | 11, 09, 06, 04, 21 |
| 13 | [AC-SYS-1](13-ac-sys-1.md) | 27 | 5 | pendiente de decisiones | 03, 16, 09, 11, 14, 20, 07, 04, 10, 15, 08, 01, 06, 19, 17, 21, 02 |
| 14 | [AC-SUP-1](14-ac-sup-1.md) | 19 | 4 | pendiente de decisiones | 11, 09, 06, 03, 13 |
| 15 | [AC-DW-1](15-ac-dw-1.md) | 8 | 2 | pendiente de decisiones | 04, 11, 14, 16, 06 |
| 16 | [AC-EXT-1](16-ac-ext-1.md) | 5 | 1 | pendiente de decisiones | 13, 03 |
| 17 | [AC-ASK-1](17-ac-ask-1.md) | 30 | 5 | pendiente de decisiones | 02, 13, 04, 08, 03, 16, 14 |
| 18 | [AC-FSM-1](18-ac-fsm-1.md) | 11 | 2 | pendiente de decisiones | 11, 19, 08, 03 |
| 19 | [AC-RT-1](19-ac-rt-1.md) | 22 | 4 | pendiente de decisiones | 04, 17, 14, 11, 03, 06, 21, 08, 07 |
| 20 | [AC-CS-1](20-ac-cs-1.md) | 15 | 3 | pendiente de decisiones | 03, 08, 13 |
| 21 | [AC-SER-1](21-ac-ser-1.md) | 23 | 4 | pendiente de decisiones | 03, 04, 13, 23, 16 |
| 22 | [AC-IO-1](22-ac-io-1.md) | 30 | 5 | pendiente de decisiones | 13, 16, 23, 03, 07, 14 |
| 23 | [AC-IO-2](23-ac-io-2.md) | 5 | 1 | pendiente de decisiones | 22, 07 |
| 24 | [AC-JAPI-1](24-ac-japi-1.md) | 21 | 4 | pendiente de decisiones | 02, 11, 18, 14, 04, 09, 13, 08, 17, 21, 22 |
| 25 | [AT-BEH-1](25-at-beh-1.md) | 26 | 5 | pendiente de decisiones | 26, 36, 34, 37, 32, 33, 27 |
| 26 | [AT-SIG-1](26-at-sig-1.md) | 8 | 2 | pendiente de decisiones | 34, 30, 25 |
| 27 | [AT-REF-1](27-at-ref-1.md) | 7 | 2 | pendiente de decisiones | 46, 25, 40, 39 |
| 28 | [AT-TK-1](28-at-tk-1.md) | 30 | 5 | pendiente de decisiones | 03, 31, 29, 30, 40, 45, 37, 41, 32, 27, 36, 10 |
| 29 | [AT-TK-2](29-at-tk-2.md) | 12 | 2 | pendiente de decisiones | 28, 08, 10, 31, 32, 36, 21, 46, 02, 03 |
| 30 | [AT-CTX-1](30-at-ctx-1.md) | 18 | 3 | pendiente de decisiones | 27, 31, 38, 26, 40, 25, 36, 34, 35, 48, 33 |
| 31 | [AT-SYS-1](31-at-sys-1.md) | 15 | 3 | pendiente de decisiones | 35, 27, 33, 45, 37, 36 |
| 32 | [AT-TMR-1](32-at-tmr-1.md) | 9 | 2 | pendiente de decisiones | 25, 30 |
| 33 | [AT-STASH-1](33-at-stash-1.md) | 6 | 1 | pendiente de decisiones | 25, 36, 34 |
| 34 | [AT-SUP-1](34-at-sup-1.md) | 24 | 4 | pendiente de decisiones | 26, 25, 32, 30, 31, 33, 36, 37, 35 |
| 35 | [AT-ADP-1](35-at-adp-1.md) | 13 | 3 | pendiente de decisiones | 31, 34, 38, 30, 25, 36, 39, 45 |
| 36 | [AT-EVT-1](36-at-evt-1.md) | 4 | 1 | pendiente de decisiones | 25 |
| 37 | [AT-LOG-1](37-at-log-1.md) | 5 | 1 | pendiente de decisiones | 25, 30, 35, 38, 28, 29 |
| 38 | [AT-DSP-1](38-at-dsp-1.md) | 5 | 1 | pendiente de decisiones | 30, 37, 31, 29 |
| 39 | [AT-EXT-1](39-at-ext-1.md) | 7 | 2 | pendiente de decisiones | 35 |
| 40 | [AT-ASK-1](40-at-ask-1.md) | 4 | 1 | pendiente de decisiones | 27, 36 |
| 41 | [AT-RCP-1](41-at-rcp-1.md) | 10 | 2 | pendiente de decisiones | 39, 46, 30, 42 |
| 42 | [AT-RT-1](42-at-rt-1.md) | 8 | 2 | pendiente de decisiones | 25, 30, 38, 41, 36 |
| 43 | [AT-PS-1](43-at-ps-1.md) | 3 | 1 | pendiente de decisiones | 30, 41, 36 |
| 44 | [AT-SPV-1](44-at-spv-1.md) | 1 | 1 | pendiente de decisiones | 30, 40 |
| 45 | [AT-CS-1](45-at-cs-1.md) | 1 | 1 | sin decisiones abiertas (preliminar) | - |
| 46 | [AT-SER-1](46-at-ser-1.md) | 3 | 1 | pendiente de decisiones | 27, 41 |
| 47 | [AT-DLV-1](47-at-dlv-1.md) | 25 | 5 | pendiente de decisiones | 30, 25, 41, 39 |
| 48 | [AT-JAPI-1](48-at-japi-1.md) | 16 | 3 | pendiente de decisiones | 25, 34, 26, 37, 30, 40, 32, 33, 42, 35 |

## Ciclos y dependencias hacia adelante

El grafo `depends_on` del inventario es cíclico a nivel de fila (Classic: CELL, SYS, EVT, SCH, TK se necesitan entre sí), por eso el orden es por unidad. 110 dependencias de spec van hacia una spec posterior. Regla de costura para todas: la spec que consume define un trait con el contrato mínimo que necesita, lo prueba con un doble de prueba que cumple las aserciones del bloque de contrato de la fila, y cuando llega la spec que lo implementa el doble se reemplaza y se re-ejecutan los tests; nada de eso reduce el contrato de la fila posterior. Lista:

- `AC-PATH-1` necesita `AC-UTIL-1`.
- `AC-UTIL-1` necesita `AC-CFG-1`.
- `AC-UTIL-1` necesita `AC-SER-1`.
- `AC-UTIL-1` necesita `AC-EVT-1`.
- `AC-UTIL-1` necesita `AC-EXT-1`.
- `AC-UTIL-1` necesita `AC-SCH-1`.
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
