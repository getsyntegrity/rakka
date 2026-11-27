# Panics del SPI de mailbox al crear un actor

## RED (2026-11-27, antes de tocar producción)

`cargo test -q -p rakka-actor --test classic_mailbox_spi_failure` → 0 passed; 4 failed:

- `a_mailbox_type_that_throws_fails_context_actor_of_and_frees_the_name`: `timed out waiting for the parent to report`
- `a_mailbox_type_that_throws_while_it_is_selected_fails_actor_of_the_same_way`: `timed out waiting for the parent to report`
- `an_ordinary_panic_of_a_mailbox_type_is_reported_as_a_runtime_exception`: `plain panic` en `spi-panic-pekko.actor.default-dispatcher-0`; `timed out waiting for the parent to report`
- `system_actor_of_returns_and_the_actor_that_cannot_be_made_is_stopped`: `timed out waiting for the actor to be stopped`

## GREEN

El mismo comando tras el cambio: `4 passed; 0 failed`.

## Mutantes (no ejecutados, Q-X-23)

Nuevo `thrown-mailbox-failure-loses-its-message` (UNIT_MUTANTS). Reescritos sobre el código nuevo, sin ejecutar: `mailbox-invalid-id-falls-back-to-the-default-and-creates-the-actor`, `mailbox-invalid-id-keeps-the-name-reserved`, `replacement-cell-honours-the-selected-mailbox`, `init-with-failure-forgets-the-failure`.
