# Panics de un `MessageQueue` del usuario: envío y `cleanUp`

## RED (2026-11-27, antes de tocar producción)

`cargo test -q -p rakka-actor --test classic_queue_spi_failure` → failed:

- `an_enqueue_that_throws_does_not_reach_the_caller_of_tell`: el `Throwable` llegó al hilo que llamó a `tell` (falla sin texto: `throw` no pasa por el panic hook)
- `a_clean_up_that_throws_does_not_stop_the_termination`: `timed out waiting for Terminated`

GREEN tras el cambio: `2 passed; 0 failed`. Valores esperados: Pekko 2.0.0 (casos grabados en un cambio posterior).

## Tests existentes que cambian

`tests/lifecycle_hook_errors.rs` (`gates.py pr`: `test` y `coverage` FAIL en 5563206a, `left: ...swallowing exception during message send`, `right: "warning|None"`). Un suscriptor del event stream cuyo `enqueue` lanza se tragaba en silencio; ahora, como en 2.0.0, publica además el envío tragado a los otros suscriptores. Se añaden esas aserciones; ninguna se quita.

## Mutantes (no ejecutados, Q-X-23)

Nuevo `thrown-enqueue-failure-is-not-reported` (UNIT_MUTANTS). Reapuntados: `mailbox-close-forgets-queued-messages` y los tres `nonblocking-cell-*`.
