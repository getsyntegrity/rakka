# Specs preliminares (generadas desde el inventario)

**Estado de las specs:** las 49 specs actuales son agrupaciones preliminares de filas del inventario. Aún requieren revisión de atomicidad y de dependencias antes de considerarse ejecutables; no empezar la implementación de una unidad hasta que esa revisión la apruebe.

Orden de ejecución, dependencias resueltas y estado. Regenerar con `python3 -I tools/package/generate.py` (ver `docs/inventory/README.md`).

| # | Spec | Filas | Tareas | Estado | Depende de |
|---|---|---|---|---|---|
| 01 | [AC-PATH-1](01-ac-path-1.md) | 9 | 2 | pendiente de decisiones | 02 |
| 02 | [AC-UTIL-1](02-ac-util-1.md) | 30 | 5 | bloqueada | 05, 03, 11, 08 |
| 03 | [AC-UTIL-2](03-ac-util-2.md) | 30 | 5 | bloqueada | 02 |
| 04 | [AC-UTIL-3](04-ac-util-3.md) | 4 | 1 | pendiente de decisiones | 03, 02, 10, 24 |
| 05 | [AC-CFG-1](05-ac-cfg-1.md) | 29 | 5 | pendiente de decisiones | 11, 18, 15, 22, 06, 16, 23, 02, 21 |
| 06 | [AC-REF-1](06-ac-ref-1.md) | 30 | 5 | pendiente de decisiones | 01, 08, 13, 05, 19, 11, 15, 17, 23 |
| 07 | [AC-REF-2](07-ac-ref-2.md) | 2 | 1 | sin decisiones abiertas (preliminar) | 06, 15, 19 |
| 08 | [AC-MBX-1](08-ac-mbx-1.md) | 23 | 4 | pendiente de decisiones | 09, 11, 05, 02, 21 |
| 09 | [AC-DSP-1](09-ac-dsp-1.md) | 22 | 4 | pendiente de decisiones | 08, 05, 21, 11 |
| 10 | [AC-SCH-1](10-ac-sch-1.md) | 15 | 3 | pendiente de decisiones | 09, 05, 13, 08 |
| 11 | [AC-EVT-1](11-ac-evt-1.md) | 24 | 4 | pendiente de decisiones | 02, 17, 15, 06, 05, 12, 13, 08, 26 |
| 12 | [AC-TK-1](12-ac-tk-1.md) | 25 | 5 | bloqueada | 17, 09, 15, 16, 05, 18, 11, 06, 20, 10, 23 |
| 13 | [AC-CELL-1](13-ac-cell-1.md) | 30 | 5 | pendiente de decisiones | 16, 08, 17, 11, 06, 10, 09 |
| 14 | [AC-CELL-2](14-ac-cell-2.md) | 8 | 2 | pendiente de decisiones | 05, 13, 11, 08, 06, 23 |
| 15 | [AC-SYS-1](15-ac-sys-1.md) | 27 | 5 | pendiente de decisiones | 05, 18, 11, 13, 16, 17, 22, 02, 09, 08, 21, 19, 01, 06 |
| 16 | [AC-SUP-1](16-ac-sup-1.md) | 21 | 4 | pendiente de decisiones | 13, 01, 11, 08, 05, 15, 17 |
| 17 | [AC-DW-1](17-ac-dw-1.md) | 13 | 3 | pendiente de decisiones | 13, 06, 16, 18, 08 |
| 18 | [AC-EXT-1](18-ac-ext-1.md) | 5 | 1 | pendiente de decisiones | 15, 05 |
| 19 | [AC-ASK-1](19-ac-ask-1.md) | 28 | 5 | pendiente de decisiones | 02, 15, 06, 10, 01, 17, 05, 18, 16 |
| 20 | [AC-FSM-1](20-ac-fsm-1.md) | 11 | 2 | bloqueada | 13, 21, 10, 05 |
| 21 | [AC-RT-1](21-ac-rt-1.md) | 22 | 4 | bloqueada | 06, 19, 16, 13, 17, 05, 08, 23, 10, 09 |
| 22 | [AC-CS-1](22-ac-cs-1.md) | 15 | 3 | pendiente de decisiones | 05, 10, 15, 17 |
| 23 | [AC-SER-1](23-ac-ser-1.md) | 23 | 4 | bloqueada | 05, 06, 15, 25, 18 |
| 24 | [AC-IO-1](24-ac-io-1.md) | 30 | 5 | pendiente de decisiones | 15, 18, 25, 05, 09, 16 |
| 25 | [AC-IO-2](25-ac-io-2.md) | 5 | 1 | pendiente de decisiones | 24, 09 |
| 26 | [AC-JAPI-1](26-ac-japi-1.md) | 19 | 4 | pendiente de decisiones | 13, 20, 16, 06, 11, 19, 23, 24 |
| 27 | [AT-BEH-1](27-at-beh-1.md) | 26 | 5 | bloqueada | 28, 37, 35, 38, 33, 34, 29 |
| 28 | [AT-SIG-1](28-at-sig-1.md) | 8 | 2 | pendiente de decisiones | 35, 31, 27 |
| 29 | [AT-REF-1](29-at-ref-1.md) | 7 | 2 | pendiente de decisiones | 47, 27, 41, 40 |
| 30 | [AT-TK-1](30-at-tk-1.md) | 19 | 4 | pendiente de decisiones | 31, 41, 46, 38, 37 |
| 31 | [AT-CTX-1](31-at-ctx-1.md) | 18 | 3 | pendiente de decisiones | 29, 32, 39, 28, 41, 27, 37, 36, 49, 35, 34 |
| 32 | [AT-SYS-1](32-at-sys-1.md) | 15 | 3 | pendiente de decisiones | 36, 29, 34, 46, 38, 37 |
| 33 | [AT-TMR-1](33-at-tmr-1.md) | 9 | 2 | pendiente de decisiones | 27, 31 |
| 34 | [AT-STASH-1](34-at-stash-1.md) | 6 | 1 | pendiente de decisiones | 27, 37, 35 |
| 35 | [AT-SUP-1](35-at-sup-1.md) | 24 | 4 | bloqueada | 28, 27, 33, 31, 32, 34, 37, 38, 36 |
| 36 | [AT-ADP-1](36-at-adp-1.md) | 13 | 3 | pendiente de decisiones | 32, 35, 39, 40, 46 |
| 37 | [AT-EVT-1](37-at-evt-1.md) | 4 | 1 | pendiente de decisiones | 27 |
| 38 | [AT-LOG-1](38-at-log-1.md) | 5 | 1 | pendiente de decisiones | 39 |
| 39 | [AT-DSP-1](39-at-dsp-1.md) | 5 | 1 | pendiente de decisiones | 38 |
| 40 | [AT-EXT-1](40-at-ext-1.md) | 7 | 2 | pendiente de decisiones | 36 |
| 41 | [AT-ASK-1](41-at-ask-1.md) | 4 | 1 | pendiente de decisiones | 29, 37 |
| 42 | [AT-RCP-1](42-at-rcp-1.md) | 10 | 2 | pendiente de decisiones | 40, 47, 31, 43 |
| 43 | [AT-RT-1](43-at-rt-1.md) | 8 | 2 | pendiente de decisiones | 27, 31, 39, 42, 37 |
| 44 | [AT-PS-1](44-at-ps-1.md) | 3 | 1 | pendiente de decisiones | 31, 42, 37 |
| 45 | [AT-SPV-1](45-at-spv-1.md) | 1 | 1 | pendiente de decisiones | 31, 41 |
| 46 | [AT-CS-1](46-at-cs-1.md) | 1 | 1 | sin decisiones abiertas (preliminar) | - |
| 47 | [AT-SER-1](47-at-ser-1.md) | 3 | 1 | pendiente de decisiones | 29, 42 |
| 48 | [AT-DLV-1](48-at-dlv-1.md) | 7 | 2 | pendiente de decisiones | - |
| 49 | [AT-JAPI-1](49-at-japi-1.md) | 6 | 1 | pendiente de decisiones | - |

## Ciclos y dependencias hacia adelante

El grafo `depends_on` del inventario es cíclico a nivel de fila (Classic: CELL, SYS, EVT, SCH, TK se necesitan entre sí), por eso el orden es por unidad. 97 dependencias de spec van hacia una spec posterior. Regla de costura para todas: la spec que consume define un trait con el contrato mínimo que necesita, lo prueba con un doble de prueba que cumple las aserciones del bloque de contrato de la fila, y cuando llega la spec que lo implementa el doble se reemplaza y se re-ejecutan los tests; nada de eso reduce el contrato de la fila posterior. Lista:

- `AC-PATH-1` necesita `AC-UTIL-1`.
- `AC-UTIL-1` necesita `AC-CFG-1`.
- `AC-UTIL-1` necesita `AC-UTIL-2`.
- `AC-UTIL-1` necesita `AC-EVT-1`.
- `AC-UTIL-1` necesita `AC-MBX-1`.
- `AC-UTIL-3` necesita `AC-SCH-1`.
- `AC-UTIL-3` necesita `AC-IO-1`.
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
- `AC-REF-1` necesita `AC-DW-1`.
- `AC-REF-1` necesita `AC-SER-1`.
- `AC-REF-2` necesita `AC-SYS-1`.
- `AC-REF-2` necesita `AC-ASK-1`.
- `AC-MBX-1` necesita `AC-DSP-1`.
- `AC-MBX-1` necesita `AC-EVT-1`.
- `AC-MBX-1` necesita `AC-RT-1`.
- `AC-DSP-1` necesita `AC-RT-1`.
- `AC-DSP-1` necesita `AC-EVT-1`.
- `AC-SCH-1` necesita `AC-CELL-1`.
- `AC-EVT-1` necesita `AC-DW-1`.
- `AC-EVT-1` necesita `AC-SYS-1`.
- `AC-EVT-1` necesita `AC-TK-1`.
- `AC-EVT-1` necesita `AC-CELL-1`.
- `AC-EVT-1` necesita `AC-JAPI-1`.
- `AC-TK-1` necesita `AC-DW-1`.
- `AC-TK-1` necesita `AC-SYS-1`.
- `AC-TK-1` necesita `AC-SUP-1`.
- `AC-TK-1` necesita `AC-EXT-1`.
- `AC-TK-1` necesita `AC-FSM-1`.
- `AC-TK-1` necesita `AC-SER-1`.
- `AC-CELL-1` necesita `AC-SUP-1`.
- `AC-CELL-1` necesita `AC-DW-1`.
- `AC-CELL-2` necesita `AC-SER-1`.
- `AC-SYS-1` necesita `AC-EXT-1`.
- `AC-SYS-1` necesita `AC-SUP-1`.
- `AC-SYS-1` necesita `AC-DW-1`.
- `AC-SYS-1` necesita `AC-CS-1`.
- `AC-SYS-1` necesita `AC-RT-1`.
- `AC-SYS-1` necesita `AC-ASK-1`.
- `AC-SUP-1` necesita `AC-DW-1`.
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
- `AT-TK-1` necesita `AT-CTX-1`.
- `AT-TK-1` necesita `AT-ASK-1`.
- `AT-TK-1` necesita `AT-CS-1`.
- `AT-TK-1` necesita `AT-LOG-1`.
- `AT-TK-1` necesita `AT-EVT-1`.
- `AT-CTX-1` necesita `AT-SYS-1`.
- `AT-CTX-1` necesita `AT-DSP-1`.
- `AT-CTX-1` necesita `AT-ASK-1`.
- `AT-CTX-1` necesita `AT-EVT-1`.
- `AT-CTX-1` necesita `AT-ADP-1`.
- `AT-CTX-1` necesita `AT-JAPI-1`.
- `AT-CTX-1` necesita `AT-SUP-1`.
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
- `AT-ADP-1` necesita `AT-EXT-1`.
- `AT-ADP-1` necesita `AT-CS-1`.
- `AT-LOG-1` necesita `AT-DSP-1`.
- `AT-RCP-1` necesita `AT-SER-1`.
- `AT-RCP-1` necesita `AT-RT-1`.
