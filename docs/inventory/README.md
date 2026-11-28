# Inventario de migración: convenciones

- `actor-classic.tsv` y `actor-typed.tsv` son las fuentes. 14 columnas: `id, unit, feature, visibility, contract, evidence, tests, depends_on, rust_equivalent, scenarios, analysis, impl, validation, notes`. Sin tabuladores ni saltos de línea dentro de un campo.
- `contracts/<grupo>.md` tiene un bloque por fila con contrato observable, fuente `path:line`, condiciones, orden/concurrencia, dependencias, equivalente Rust, aserciones `A#` con la cita del cuerpo del test (o "no afirmada por spec"), oráculo y decisiones; al final, descomposiciones, omisiones, contradicciones corregidas, filas que siguen `listed` y cobertura del árbol de Pekko. `contracts/<grupo>.decisions.tsv` son las decisiones del grupo.
- **Descomposición:** una fila que agrupaba comportamientos no verificables como uno se reemplaza por hijos `<ID>.<n>`. Una referencia a un padre descompuesto en `depends_on` significa todos sus hijos.
- **`analysis`:** `extracted` = se leyó la implementación y los cuerpos de los specs de la fila y su contrato cita aserciones; `listed` = contrato por extraer, no ejecutable. El motivo de cada `listed` está en `notes`.
- **`impl` / `validation`:** `pending`, `partial` o `done`. Sólo se avanza con evidencia (tests RED→GREEN, oráculo). `partial` es lo que queda con una decisión o una diferencia abierta. `validation=documented` (nuevo, 2026-11-28) es sólo para filas `done` sin casos grabados porque el Owner decidió cerrarlas sin implementación propia (Q-UTIL-20, Q-UTIL-27); su nota lleva `SIN IMPLEMENTACIÓN PROPIA: ...` y `progress.md` las lista aparte. No se usa sin una decisión del Owner.
- **Decisiones:** `Q-<UNIDAD>-<n>` (Classic), `Q-T-<UNIDAD>-<n>` (Typed), `Q-X-<n>` (transversales). Alias: `Q-CFG-1`=`Q-X-1`, `Q-CFG-2`=`Q-X-2`. `Q-X-9` sólo agrupa `Q-UTIL-1..5` para presentación: cada una se decide por separado. Registro único: `../decisions.md`.
- **Respuestas del Owner:** se registran sólo en `docs/decisions/owner-decisions.tsv` (`id`, `status` = `accepted`/`accepted_with_changes`/`rejected`, `decided_on`, `decided_by`, `decision`); `register.tsv`, `decisions.md` y los conteos se derivan de ahí. Prueba: `python3 -I tools/package/tests/test_decision_persistence.py`. **Regenerar y verificar:** `python3 -I tools/package/generate.py` regenera `docs/decisions*`, `docs/contract-map.tsv` y `docs/superpowers/plans/units/`; `python3 -I tools/package/check.py` verifica la coherencia del paquete.

## Cobertura del árbol de Pekko (2026-10-30)

Criterio: el nombre de cada archivo `.scala`/`.java`/`.conf` de los módulos aparece en el inventario o en los contratos (es un piso, no una prueba de exhaustividad: citar un archivo no implica haber leído todo su cuerpo; eso lo dice `analysis`).

| Área | Archivos | Sin cita | Motivo de los que quedan |
|---|---|---|---|
| `actor/src/main` | 263 | 3 | `PartialFunction.scala` (3 copias, shims de versión de Scala sin comportamiento propio) |
| `testkit/src/main` | 19 | 0 | |
| `actor-typed/src/main` | 92 | 0 | |
| `actor-testkit-typed/src/main` | 46 | 0 | los 5 archivos que no tenían cita se agregaron como `AT-TK-011..013` (`listed`) |
| `actor-tests/src/test` | 200 | 13 | ejemplos Java, fixtures y tests de rendimiento/hilos virtuales (`ForkJoinPool*Spec`, `ActorCreationPerfSpec`) sin contrato nuevo; revisar al ejecutar AC-DSP |
| `testkit/src/test` | 30 | 10 | `MetricsKit` y utilidades de métricas (fila `listed` de TK) |
| `actor-typed-tests/src/test` | 142 | 63 | casi todos ejemplos de documentación Java/Scala (`*DocTest`, `*Sample`) |
| `actor-testkit-typed/src/test` | 28 | 20 | ejemplos y tests Java del testkit |
