# Evidencia del slice P74 UTIL-TIPOS-DESERIALIZACION, 2026-11-18

**Filas:** AC-UTIL-016.12 (ClassLoaderObjectInputStream).

**Resultado:** las filas quedaron **excluidas por autorización del Owner** (impl `done`, validation `excluded`, nota `EXCLUIDA (autorizada …)`): no se implementa nada y no cuentan como done ni en el alcance de `docs/progress.md`.

**Decisión:** Q-UTIL-11 (respuesta posterior del Owner, 2026-11-18): se autoriza excluir AC-UTIL-016.12 (deserialización Java con resolución por classloader; sin serialización Java).

**Comprobaciones:** no hay oráculo ni casos que grabar (no hay conducta observable que portar). `tools/package/check.py` sin fallos; las filas figuran como `excluded` en `docs/progress.md`.
