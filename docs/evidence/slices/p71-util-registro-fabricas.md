# Evidencia del slice P71 UTIL-REGISTRO-FABRICAS, 2026-11-18

**Filas:** AC-UTIL-016.7 (Reflect) y AC-UTIL-016.13 (BoxedType).

**Resultado:** las filas quedaron **excluidas por autorización del Owner** (impl `done`, validation `excluded`, nota `EXCLUIDA (autorizada …)`): no se implementa nada y no cuentan como done ni en el alcance de `docs/progress.md`.

**Decisión:** Q-UTIL-11 (respuesta posterior del Owner, 2026-11-18): se autoriza excluir AC-UTIL-016.7 (cubierto por el registro de nombres de Q-X-2, AC-CFG-016) y AC-UTIL-016.13 (sin boxing en Rust).

**Comprobaciones:** no hay oráculo ni casos que grabar (no hay conducta observable que portar). `tools/package/check.py` sin fallos; las filas figuran como `excluded` en `docs/progress.md`.
