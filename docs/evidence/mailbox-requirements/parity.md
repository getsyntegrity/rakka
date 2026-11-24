# Paridad y mutantes de los requisitos del actor (Slice 3)

Oráculo Pekko 2.0.0 (`9fbd2283`, JDK 17). Casos `MBX-REQ-01..03` = `mb-0024..mb-0026`; Rakka coincide byte a byte (`compare` oráculo/Rust) y el replay completo da **2970/2970, 0 divergencias** (2967 anteriores + 3).

| Caso | Contrato | Observado |
|---|---|---|
| `MBX-REQ-01` | requisito del actor → mailbox enlazado; actores sin requisito conservan el default | unbounded FIFO por la tabla de `reference.conf`; bounded → capacidad 4; marcador de usuario → su mailbox; `plain`/`plainAfter` LIFO (default) |
| `MBX-REQ-02` | requisito sin binding | `ConfigurationException` en `actorOf` con causa `Mailbox Mapping for [interface …] not configured`; el nombre queda libre |
| `MBX-REQ-03` | id del Props o del Deploy > requisito; id vacío no cuenta | LIFO/capacidad 2 por id; `emptyDeploy` y `emptyProps` conservan el requisito |

## Mutantes (`tools/parity/check_divergence.py`, 6 nuevos + control)

| Mutante | Resultado | Cómo se detectó |
|---|---|---|
| `mailbox-requirement-beats-the-selected-id` | DETECTED | **específica**: test unitario `a_selected_id_decides_the_mailbox_over_the_requirement` (1 de 15 falla); además lo vería `MBX-REQ-03` |
| `mailbox-requirement-error-names-the-class-without-its-kind` | DETECTED | **específica**: diverge 1 de 2970 (`mb-0025`) |
| `mailbox-actor-requirement-ignored` | DETECTED | **caída de la familia mailbox**: fast-kill en `mb-0001`; `rust_lines()` calcula todos los casos y el caso colgado hace caer al binario (limitación 8 del harness) |
| `mailbox-requirement-binding-ignored` | DETECTED | caída de la familia (26 casos divergen) |
| `mailbox-requirement-applied-to-every-actor` | DETECTED | caída de la familia (26) |
| `mailbox-requirement-without-binding-uses-the-default` | DETECTED | caída de la familia (26) |
| control sin mutaciones | CLEAN | 2970/2970 |

Ninguno terminó en error de infraestructura ni de compilación (`PATCH/BUILD/HARNESS_ERROR/TIMEOUT` = 0). Los tres reportes completos de caída de familia (683 líneas cada uno) no se publican por el límite de 400 líneas por PR; se regeneran con `python3.12 tools/parity/check_divergence.py <mutante>` y sus `replay-killers-*.txt` sí se publican.

Regresión de Slice 2 en la misma base: 13/13 mutantes DETECTED y control 2967/2967 (antes de añadir los casos de este slice).
