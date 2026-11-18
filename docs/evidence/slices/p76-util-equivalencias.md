# Evidencia del slice P76 (UTIL-EQUIVALENCIAS), 2026-11-18

Resolucion documental: no hay codigo Rust ni casos nuevos del oraculo (los equivalentes son `Option<T>`, closures, iteradores, `_nombre`, visibilidad/cargo-semver-checks; ver Q-UTIL-11 (a)).

Releido contra el arbol 2.0.0 (upstream/pekko, v2.0.0):
- OptionVal.scala, ConstantFun.scala, anotaciones: sin cambios de comportamiento.
- Collections.scala: EmptyImmutableSeq.iterator = Iterator.empty; nuevo IterableOps.collectToImmutableSeq (sin consumidores en main).
- Unused.scala: eliminado en 2.0.0 (no es exclusion).

Filas: AC-UTIL-016.1, 016.2, 016.3, 016.18, 019 quedan `partial`/`pending` hasta que el Owner responda Q-UTIL-20 (docs/owner-batch-02.md).
Gates ejecutados (cambio solo de docs): `tools/package/generate.py`, `unit_plan.py`, `check.py` (failures=0), `unittest discover -s tools/package/tests` (35 OK).
