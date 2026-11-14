# Registro de los ADR de goPekko en alcance

Estado declarado = primera línea `Status:` del ADR en goPekko `8ad83a6` (resumida; el texto exacto está en la copia). **Ninguno es decisión de Rakka.** «Contraparte Rakka» nombra el lugar canónico donde Rakka decide o decidirá lo mismo; «sin leer a fondo» significa que sólo se leyó cabecera y tablas de decisión.

| ADR goPekko | Tema | Estado declarado | Contraparte Rakka | Lectura |
|---|---|---|---|---|
| gopekko:ADR-0008 | Envelope y metadatos | proposed (nada aceptado) | AC-REF/AC-CELL (sender, envelope) | cabecera |
| gopekko:ADR-0010 | Estilo de API del actor | proposed, parcialmente aceptado (watch local; supervisión: modelo Classic, vía ADR 0017) | AC-REF, AT-REF, AC-ASK | cabecera y tabla D1-D9 |
| gopekko:ADR-0011 | Layout del módulo `pekko-actor` | accepted | ninguna (organización de paquetes Go); Rakka: `docs/implementation-order.md` | cabecera |
| gopekko:ADR-0012 | Stop aceptado antes de la primera activación (divergencia D2) | accepted para el caso acotado, con corrección #690 | AC-CELL | cabecera |
| gopekko:ADR-0014 (+ `-implementation-decisions`) | Hijos y terminación jerárquica | accepted design (D1-D7); O1-O3 aceptadas | AC-CELL, AC-DW | cabecera y tabla D1-D7 |
| gopekko:ADR-0015 | Ask y reply | proposed; decisiones marcadas [Owner] aprobadas 2026-10-10 | AC-ASK | cabecera |
| gopekko:ADR-0017 | Supervisión (modelo Classic) | accepted 2026-10-14 | AC-SUP, AT-SUP (decisión previa «qué es un fallo»: filas AC-SUP-001/004, `docs/implementation-order.md`) | cabecera y tabla de divergencias D-1..D-5 |
| gopekko:ADR-0024 | Retención de actores terminados en `System.byID` | accepted D1-D5 (2026-10-20) | AC-REF/AC-CELL (registro de hijos) | cabecera |
| gopekko:ADR-0030 | Mailbox acotado (`BoundedMailbox`, push-timeout cero) | accepted D1-D8 | AC-MBX (`BoundedMailbox` que bloquea al emisor: AC-MBX-007, `docs/implementation-order.md`) | cabecera y tabla de fuentes P1-P12 |
| gopekko:ADR-0031 | Valores `Address`/`ActorPath` locales | accepted (O1-O5) con enmiendas del 2026-11-04 | AC-PATH (**ya implementada en Rakka con alcance mayor**; ver `docs/research/gopekko-sources.md`) | completo |
| gopekko:ADR-0032 | Envelope local del sender | parcialmente aceptado (D5a/D5b «seguir a Pekko») | AC-REF (`sender`, `forward`, dead letters) | cabecera y tabla P1-P11 |
| gopekko:ADR-0033 | Alcance y arquitectura de actor + actor-typed | accepted 2026-10-24 (con interpretación de D13) | `docs/implementation-order.md`, inventario Typed; el significado de «100 %» difiere (ver brechas) | cabecera y tabla D-04..D-13 |
| gopekko:ADR-0034 | Frontera de `Behavior[T]` | parcialmente aceptado 2026-10-27 | AT-BEH (forma de `setup`/reuso de `Behavior`: AT-BEH-007, `docs/implementation-order.md`) | cabecera y tabla B1-B12 |
| gopekko:ADR-0035 | Supervisión Typed | accepted (S-1..S-6, enmendado hasta 2026-11-10) | AT-SUP (incluye discrepancia doc/código AT-SUP-007) | cabecera y tabla padre/hijo |
| gopekko:ADR-0036 | Typed: unhandled/dropped e interceptores | accepted 2026-11-04, enmendado 2026-11-05 | AT-EVT, AT-BEH | cabecera |
| gopekko:ADR-0037 | `Port` como adaptación Go; tipo incorrecto falla | accepted 2026-11-05 | ninguna (`Port` no existe en Pekko); ver evaluación Rust | completo |
| gopekko:ADR-0038 | Semántica de suscripción del EventStream | accepted 2026-11-06 (O-1..O-7) | AC-EVT (suscripción por jerarquía de tipos sin herencia: AC-EVT-003, `docs/implementation-order.md`) | cabecera y tabla O-1..O-7 |
| gopekko:ADR-0039 | Ciclo de vida y cancelación del scheduler | accepted 2026-11-10 (O-1..O-10) | AC-SCH, AT-TMR | cabecera |

**ADR de goPekko excluidos** (persistencia, tenancy, serialización, infra de test; motivo en el índice): 0016, 0018, 0019, 0021, 0022, 0023, 0025, 0026, 0027, 0028, 0029; la plantilla 0000 y los `signatures/` (código Go) se vinculan o excluyen según el índice.
**Propuestas de ADR copiadas** (`docs/adr/proposals/0031-addendum-*-118{6,7}.md`): estado PROPOSAL, no aplicadas; la enmienda del 2026-11-04 del propio ADR 0031 las supera en lo que difiere de Pekko.
