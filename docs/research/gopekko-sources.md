# Fuentes de goPekko: registro de lectura y reglas de uso

Este documento **no es una fuente de estado**: el estado de cada funcionalidad vive sólo en `docs/inventory/*.tsv` y en `docs/inventory/contracts/`. Aquí se registra qué se leyó de goPekko, a qué SHA, qué está integrado y qué no, y cómo se usa. Apache Pekko 1.7.1 (`9a54d129…`) manda; goPekko sólo aporta evidencia y análisis que se **verifica contra Pekko** antes de citarse.

## Qué se consultó

| Dato | Valor |
|---|---|
| Repositorio | `getsyntegrity/gopekko`, rama por defecto `develop` |
| SHA consultado | `8ad83a60b7229573b0af2145c6414d3d67371867` (HEAD de `develop` al 2026-11-14, leído por la API de GitHub; no hay clon local) |
| Pin de Pekko de goPekko | `v1.7.1` / `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, el mismo de Rakka (`docs/parity/actor-inventory.md`, ADR 0031) |
| Estado integrado vs. propuesto | Todo lo listado abajo está en `develop` (integrado). Eso no lo vuelve aprobado: el encabezado de cada archivo declara su estado y se copia tal cual |
| PR abiertos en goPekko | #1572 y #1573-#1581: evidencia y auditoría del restart con stash de Typed (`evidence/999-*`, `docs/999-906-audit-20261114`). **Propuestas no integradas, no leídas, sin relación con paths.** |

## Documentos canónicos localizados

| Tema | Ruta en goPekko | Estado declarado en el archivo | Leído |
|---|---|---|---|
| Inventario de capacidades (actor + typed) | `docs/parity/actor-inventory.md` | «inventory document, proposed for Owner confirmation»; no afirma paridad; niveles V asignados por decisión del Owner | cabecera, columnas, reglas y filas de `C-01..C-08` |
| Juez de paridad | `docs/judge.md` | «proposed, for owner review» (2026-10-08): decide paridad por eventos observables contra invariantes y relaciones de orden garantizadas, nunca por traza idéntica | sólo cabecera |
| ADR de `Address`/`ActorPath` | `docs/adr/0031-local-address-and-actor-path-values.md` | **accepted** (2026-10-24), con enmiendas del Owner 2026-11-04 (a), (b) y nota U-2; último commit que lo toca `53cb678` (2026-11-04) | completo |
| Adendas de gramática y de token de incarnation | `docs/adr/proposals/0031-addendum-*-118{6,7}.md` | **PROPOSAL, no aplicadas**; revisadas en parte por las enmiendas del 2026-11-04 | cabeceras y la fila E4 |
| Alcance por historia | `docs/scopes/{812,852,853,1089}-*.md` | alcance de historias de goPekko; 852 último commit `8873b10` (2026-10-29) | sólo `852`: línea de alcance y negativos S8/S9; `812`, `853`, `1089` descargados y **no leídos** |
| Trazabilidad ejecutada | `migration/stage-1/<historia>/manifest.tsv` y `pekko-repro/observed.txt` | columnas `source, target, disposition, pekko_row, scenario, evidence`; la evidencia «Executed» es de una copia desechable de Pekko, no del árbol pinneado | sólo el manifest de 852; el de 812 y los `pekko-repro` **no leídos** |
| Auditorías | `docs/audit/auditoria-3-*.md`, `docs/parity/audits/*`, `docs/parity/actor-local-*-audit-*.md` | auditorías de goPekko | **no leídas** |
| Oráculos y escenarios | `docs/parity/oracles/` (405 archivos), `judge/` | | **no leídos** |

Los inventarios de goPekko usan otra numeración (`C-01`, `IDN-03`, historias `#812`) y otra unidad de trabajo. **No se importan IDs**: la correspondencia se anota dentro del bloque de la fila de Rakka (sección «Evidencia goPekko»).

## Reglas de uso

1. Un hecho de goPekko entra al contrato de una fila de Rakka sólo con su `path:line` de Pekko verificado en `upstream/pekko`. La cita de goPekko es evidencia secundaria.
2. «Executed» en goPekko significa que alguien corrió Pekko en una copia desechable; para Rakka vale sólo si se reproduce con `tools/oracle/` o se cita como contraste, nunca como golden.
3. Una decisión de goPekko motivada por Go (valores locales, `error` centinela, `==` con uid, sin `Address` global) **no se traslada**. Si afecta lo observable, pasa por el registro de decisiones de Rakka (`docs/decisions.md`).
4. Un test de Go se traduce a la propiedad observable que afirma, no se copia. Para concurrencia, se expresan las relaciones de orden garantizadas (criterio del juez) y no un interleaving.
5. Lo no leído de esta tabla no cuenta como evidencia.

## Rakka frente a goPekko en PATH (hallazgo de esta lectura)

En `Address`/`ActorPath`, Rakka va por delante de goPekko: tiene `Address` global, parser sobre semántica de `java.net.URI`, comparadores, uid y 405 líneas de golden de Pekko, mientras ADR 0031 de goPekko fija sólo-local y su propia enmienda del 2026-11-04 (b) corrige hacia «tal cual Pekko» lo que Rakka ya cumple. De goPekko sirve el contraste de `ChildActorPath` (ADR 0031 P5/C3), útil para la fila AC-PATH-016, y las filas «Executed» de `/` y `descendant` en `migration/stage-1/actor-852-path-navigation/manifest.tsv`.
