# Panics de un `MessageQueue` del usuario durante un turno

Comando: `cargo test -q -p rakka-actor --test classic_queue_spi_failure`.

- RED antes del código (2026-11-27): `a_dequeue_that_throws_...`: `timed out waiting for the actor's messages`; `a_has_messages_that_throws_...`: `timed out waiting for the observer`.
- RED de la revisión, antes de cambiar `report_uncaught` (publicaba bajo la guarda de recursión): `5 passed; 1 failed`, `the_uncaught_report_to_a_subscriber_that_cannot_take_it_...`: `timed out waiting for the observer`; el envío tragado salió por stdout (`[ERROR] [pekko://tq-subscriber/user/broken] swallowing exception during message send`).
- GREEN: `6 passed`.
- Caracterización (escrita después del código, sin RED): `a_turn_and_its_rescheduling_that_both_throw_report_only_the_second`.
- Oráculo (sonda desechable sobre Pekko 2.0.0, JDK 17, no grabada): con un suscriptor de `Error` cuyo `enqueue` lanza, los demás reciben el informe no capturado y después `swallowing exception during message send` de ese suscriptor.
- Mutantes (no ejecutados, Q-X-23): `uncaught-turn-failure-is-not-reported`, `uncaught-report-is-published-guarded` (UNIT_MUTANTS).
