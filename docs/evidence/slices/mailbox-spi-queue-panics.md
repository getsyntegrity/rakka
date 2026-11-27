# Panics de un `MessageQueue` del usuario

## RED (2026-11-27, antes de tocar producción)

`cargo test -q -p rakka-actor --test classic_queue_spi_failure` → 0 passed; 4 failed:

- `an_enqueue_that_throws_does_not_reach_the_caller_of_tell`: el `Throwable` llegó al hilo que llamó a `tell` (falla sin texto: `throw` no pasa por el panic hook)
- `a_dequeue_that_throws_is_reported_as_uncaught_and_the_actor_goes_on`: `timed out waiting for the actor's messages`
- `a_has_messages_that_throws_after_a_turn_leaves_the_mailbox_idle`: `timed out waiting for the observer`
- `a_clean_up_that_throws_does_not_stop_the_termination`: `timed out waiting for Terminated`

## GREEN

El mismo comando tras el cambio: `4 passed; 0 failed`.

## Oráculo (valores esperados)

`oracle/run.sh mailbox-case` con `MBX-TQ-01..05` (Pekko 2.0.0, JDK 17); grabados en el PR siguiente. El informe de error no capturado pasa por un `LoggingAdapter`: el oráculo lo observa con `loglevel = ERROR` y `use-slf4j = off`.

## Tests existentes que cambian

`tests/lifecycle_hook_errors.rs` (`gates.py pr`: `test` y `coverage` FAIL en 5563206a, `left: ...swallowing exception during message send`, `right: "warning|None"`). Un suscriptor del event stream cuyo `enqueue` lanza se tragaba en silencio; ahora, como en 2.0.0, publica además el envío tragado a los otros suscriptores (oráculo `MBX-TQ-05`). Se añaden esas aserciones; ninguna se quita. Los mutantes reapuntados al código nuevo: `mailbox-close-forgets-queued-messages` y los tres `nonblocking-cell-*`.

## Mutantes (no ejecutados, Q-X-23)

Nuevo `uncaught-turn-failure-is-not-reported` (UNIT_MUTANTS).
