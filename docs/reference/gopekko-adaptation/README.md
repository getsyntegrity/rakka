# Adaptación del conocimiento de goPekko a Rakka

Este directorio **no copia** goPekko (eso es `docs/reference/gopekko/`, byte a byte y con su SHA): registra cómo se usa ese conocimiento en Rakka y lo que falta. Nada de aquí cambia el inventario, acepta una decisión ni declara paridad.

## Cadena de trazabilidad

Símbolo Apache Pekko → contrato observable → evidencia goPekko → unidad Rakka → implementación Rust → pruebas → brechas.

| Eslabón | Dónde vive (fuente única) | Estado de esta transferencia |
|---|---|---|
| Símbolo y contrato observable | `docs/inventory/*.tsv` y `docs/inventory/contracts/*.md` (Rakka) | Sin cambios |
| Evidencia goPekko | `docs/reference/gopekko/` + índice `GP-NNNN` | Copiada o vinculada; la **atribución por fila** del contrato (bloque «Evidencia goPekko») existe sólo para AC-PATH-016 (PR #10) |
| Unidad Rakka | `docs/unit-plan.md`, `docs/contract-map.tsv` | La columna `rakka_unit_hint` del índice es una pista gruesa por tema, **no** un mapeo validado |
| Implementación y pruebas Rust | `crates/`, columnas `impl`/`validation` | Sin cambios; ninguna fila avanza por importar documentación |
| Brechas | `contradictions-and-gaps.md` y `notes` del inventario | Registro inicial aquí |

## Cuatro categorías que no se mezclan

1. **Semántica de Apache Pekko:** sólo vale lo verificado en `upstream/pekko` (`9a54d129`). Una cita de Pekko dentro de un documento de goPekko es una afirmación de goPekko hasta que se verifica.
2. **Decisiones particulares de goPekko** (ADR, enmiendas del Owner de goPekko): referencias externas, escritas `gopekko:ADR-NNNN` o `gopekko:<id de decisión>`. **No son decisiones de Rakka**, aunque digan «accepted». Un ADR de goPekko se vuelve decisión de Rakka sólo si el Owner de Rakka la registra con la convención local (`Q-<UNIDAD>-n`, `docs/decisions.md`).
3. **Propuestas para Rust:** `go-decisions-to-evaluate.md` las enumera como candidatas. No están registradas como `Q-*`; registrarlas es un cambio del registro canónico y se hace por unidad.
4. **Decisiones Rust ya aceptadas:** hoy sólo Q-UTIL-1 (`docs/decisions/owner-decisions.tsv`, estado `rejected` con alternativa vigente). Ninguna toca Actor/Typed.

## Tipos de evidencia (no intercambiables)

| Tipo | Significado | Vale para Rakka como |
|---|---|---|
| Fuente leída | alguien leyó el `path:line` de Pekko | pista; se re-lee |
| Observación ejecutada en Pekko | corrida de Pekko (jars, copia desechable, rama distinta: ver brechas) | contraste; no es golden de Rakka |
| Prueba Go | test de goPekko | propiedad observable a trasladar, no test a copiar |
| Prueba Rust | test de `crates/` con aserciones verificadas | única que cuenta para `impl`/`validation` |

Las conclusiones de goPekko («V3», «closed», «done») se conservan con su fecha y SHA y **no** se traducen a estados de Rakka.

## Convenciones

- `GP-NNNN`: id estable de documento del índice (orden por ruta en el SHA `8ad83a6`).
- `gopekko:ADR-NNNN`, `gopekko:#NNN` (historias/PR de goPekko): referencias externas.
- Los números locales `AC-*`/`AT-*`/`Q-*` los asigna sólo Rakka.

## Balance de la transferencia (SHA de origen `8ad83a60b7229573b0af2145c6414d3d67371867`)

| Concepto | Cantidad |
|---|---|
| Documentos considerados | 1217 |
| Copiados (`copiar`, 571 archivos, partidos en 197 bloques de PR) | 571 |
| Vinculados con enlace permanente (`vincular`: código, scripts, salidas crudas) | 542 |
| Excluidos con motivo (`excluir`: persistencia, lectura, infra, proceso) | 104 |
| Adaptados al inventario canónico | 0 filas (sólo AC-PATH-016, PR #10) |
| Leídos en profundidad por quien transfirió | ver `contradictions-and-gaps.md`, sección «Lo no leído» |
