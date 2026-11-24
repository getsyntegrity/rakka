# Seguimiento de Q-MBX-12 y de la selección de mailbox por id

Q-MBX-12 está **aceptada como limitación temporal** (Owner, 2026-11-24). No es paridad completa: `AC-MBX-014` sigue `partial`.

## Tareas pendientes (ninguna se implementó en el cierre del slice)

| # | Tarea | Depende de |
|---|---|---|
| 1 | Aplicar el mailbox por defecto registrado (`pekko.actor.default-mailbox`) también a los actores sin selección explícita | — |
| 2 | Resolver la precedencia por `Deploy.mailbox` y por configuración (`deployer.lookup` antes de `getMailboxType`) | Q-CFG |
| 3 | Soportar `bounded` y `bounded-capacity:N` | Q-MBX-1 |
| 4 | Completar la identificación del dispatcher en el mensaje de error de `actorOf` (hoy fijo) | selección de dispatcher en `Props` |
| 5 | Completar la representación de la causa del error (hoy texto; sin `source()`), con su clase real | — |
| 6 | Casos pendientes: nombre autogenerado con id desconocido y nombre duplicado con id desconocido (`InvalidActorName` primero) | — |
| 7 | Investigar `MBX-RJ-03` | ver abajo |
| 8 | Mejorar la granularidad de detección de mutantes del harness (`rust_lines()` calcula todas las líneas: un pánico hace divergir los 21 casos) | — |

## Fallos intermitentes: seguimiento abierto (no resueltos)

- **`MBX-RJ-03` (oráculo Java):** falló una vez en `check_mailbox_oracle.py --runs 1` con `ERR IllegalStateException [condition not reached]` (el techo de 10 s de `awaitUntil`). Sin reproducción: 0/116 corridas en base y PR (aisladas, con carga y completas). Causa sin determinar; no atribuible al slice (el oráculo cambia sólo por adición, después de ese caso).
- **`cargo test --workspace` en #343 (2026-11-24):** una ejecución terminó con código 101 y 0 pruebas contadas, sin salida guardada; la repetición pasó (768) y 20 ejecuciones de la librería no fallaron. Causa sin determinar; el flujo de integración ahora conserva el log de una ejecución fallida.
