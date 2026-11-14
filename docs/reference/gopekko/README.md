# Copia de referencia de la documentación de goPekko

**Esto describe goPekko, no el estado de Rakka.** Nada de lo que está bajo este directorio es una decisión aceptada de Rakka, una implementación Rust ni una validación de paridad. El estado de Rakka vive sólo en `docs/inventory/`, `docs/decisions.md` y el código.

## Procedencia

| Dato | Valor |
|---|---|
| Origen | https://github.com/getsyntegrity/gopekko, rama `develop` |
| SHA de origen | `8ad83a60b7229573b0af2145c6414d3d67371867` (todos los archivos; ningún PR abierto de goPekko está incluido) |
| Referencia semántica | Apache Pekko v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, el mismo pin de Rakka (`docs/upstream.md`). goPekko declara el mismo pin; la equivalencia no se supone: se verifica por documento |
| Licencia y avisos | goPekko es Apache-2.0 y su `LICENSE` es idéntico al `LICENSE` de Rakka (verificado con `diff`); `NOTICE.gopekko` es copia sin cambios de su `NOTICE`. Los documentos conservan sus avisos |

## Reglas de la copia

1. **Los archivos se copian byte a byte** bajo la misma ruta relativa que en goPekko (`docs/reference/gopekko/<ruta de origen>`). No se reescribe, se corrige ni se anota ninguna afirmación dentro de ellos; las aclaraciones van fuera, en el índice y en `docs/reference/gopekko-adaptation/`.
2. **Archivos largos** se dividen en `<ruta>.part-NN-of-MM` de a lo sumo 360 líneas, sin pérdida: concatenados en orden dan el original (se verifica por el SHA de blob git).
3. **El estado declarado** de cada documento (accepted, proposed, histórico, etc.) es el de su encabezado en goPekko, copiado en la columna `declared_status` del índice. «Accepted» en goPekko no es aceptación de Rakka.
4. **Enlaces relativos** dentro de un documento se resuelven contra el árbol de goPekko en el SHA de origen. Si el destino se copió, está en la misma ruta bajo este directorio; si no, el índice da su enlace permanente (`blob/<SHA>/<ruta>`).
5. **Tratamientos:** `copiar` (copia aquí), `vincular` (sólo enlace permanente: código, scripts, salidas crudas de ejecución), `excluir` (fuera de Actor y Typed, con motivo).

## Índice

`index/gopekko-transfer-index-NN.tsv` tiene una fila por documento considerado: `id` (`GP-NNNN`, estable, ordenado por ruta), `source_path`, `blob_sha` (SHA de blob git en el SHA de origen), `bytes`, `lines`, `topic`, `declared_status`, `treatment`, `destination`, `parts`, `block` (bloque de PR que la copia), `rakka_unit_hint` (pista gruesa de unidad Rakka, **no** mapeo validado) y `why`.

Verificación: `python3 -I tools/package/check_gopekko_transfer.py`.

## Alcance

Entra: documentación de Actor Classic y Typed (ADR, alcances, paridad, auditorías, cierres de diseño, registros de migración, reproducciones de Pekko, escenarios del juez, README de los módulos `pekko-actor` y `pekko-actor-typed`). Quedan fuera por motivo registrado en el índice: persistencia, lectura/proyecciones, serialización, tenancy, infraestructura de test y CI de goPekko, y las skills y plantillas de proceso de goPekko (`.claude/`, `skills/`, `.github/`, `migration-kit-ext/`, README de otros módulos).
