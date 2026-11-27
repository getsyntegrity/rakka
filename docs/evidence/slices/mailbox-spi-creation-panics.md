# Panics del SPI de mailbox al crear un actor

Comando: `cargo test -q -p rakka-actor --test classic_mailbox_spi_failure`.

- RED antes del código (2026-11-27): `0 passed; 4 failed` (los tres de `context.actorOf`: `timed out waiting for the parent to report`; el de `system.actorOf`: `timed out waiting for the actor to be stopped`). GREEN: `4 passed`.
- `a_default_mailbox_that_panics_...` se escribió DESPUÉS del código. Defecto original reproducido sobre 24c5c447 (sin el cambio): `left: None`, `right: Some("java.lang.RuntimeException: plain panic")`. Sólo cambia el payload de un `panic!` ordinario; con `throw(Throwable)` no cambia nada. Con el cambio: `5 passed`.
- Mutantes (no ejecutados, Q-X-23): nuevo `thrown-mailbox-failure-loses-its-message`; reescritos `mailbox-invalid-id-falls-back-to-the-default-and-creates-the-actor`, `mailbox-invalid-id-keeps-the-name-reserved`, `replacement-cell-honours-the-selected-mailbox`, `init-with-failure-forgets-the-failure`.
