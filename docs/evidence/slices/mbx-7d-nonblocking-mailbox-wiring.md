# MBX PR 7d: `NonBlockingBoundedMailbox` cableado en el runtime (AC-MBX-008)

Implementa la alternativa A de PROP-7B-1, autorizada por el Owner (continuación de `mbx-7c-nonblocking-mailbox-runtime-wiring.md`, PR #335). No implementa Q-MBX-1 ni Q-MBX-2 completos, ni lectura de `mailbox-capacity` desde `Config`.

## Cambios de API

- `MessageQueue::enqueue(&self, receiver: &ActorRef, envelope: Envelope) -> bool` (antes `enqueue(&self, envelope)`): `true` si se almacenó. Una cola que rechaza ya envió el `DeadLetter` a `deadLetters` dirigido a `receiver`.
- `MessageQueue::clean_up(&self, owner: &ActorRef, dead_letters: &dyn MessageQueue)` (Pekko `cleanUp(owner, deadLetters)`).
- `NonBlockingBoundedMailbox::create(dead_letters)` (antes `create(owner, dead_letters)`): la cola ya no guarda al dueño, así que no hay ciclo celda → cola → dueño.
- Nuevo, público: `Props::with_mailbox(NonBlockingBoundedMailbox)`; sólo acepta ese tipo hasta que Q-MBX-2 defina `Mailboxes`.
- Interno: `mailbox_close::Enqueued { awaits_consumption, dead }` reemplaza el `Vec<Envelope>` de `enqueue_unless_closed`.

## Punto de integración

`ActorCell::build` crea la cola con `Props::new_message_queue(&system.dead_letters)` (la celda de reemplazo de `new_with_failure` sigue ilimitada). `enqueue_user` pasa `self_ref()` como receptor y sólo llama a `register_for_execution` si `awaits_consumption`; `finish_terminate` limpia hacia el actor.

Diferencia consciente con Pekko: `Dispatcher.dispatch` registra un turno aunque la cola rechace; aquí no (contrato pedido por el Owner). Un turno vacío no es observable salvo por `scheduled`, y el estado `scheduled` sólo lo mira el oráculo de estado del mailbox, que no se ve afectado (replay sin divergencias).

## Pruebas

- RED (antes de cablear la celda): `cargo test --test classic_nonblocking_bounded_mailbox` → 3 fallas por `timed out waiting for ... dead-lettered` (la cola seguía ilimitada).
- GREEN: las 3 pasan; ver abajo.
- `tests/classic_nonblocking_bounded_mailbox.rs` (API pública, actores reales): cola llena rechaza al instante y el `DeadLetter` lleva mensaje, emisor y receptor; capacidad 0 rechaza todo; 4 productores concurrentes sin pérdida ni duplicación.
- `src/actor/nonblocking_mailbox_tests.rs` (interno): un rechazo no genera turno (ejecutor que cuenta), uno aceptado sí; `stop` con cola llena envía lo encolado a dead letters dirigido al actor; el reinicio conserva cola y cota; un actor terminado se libera (sin ciclo de `Arc`).
- La prueba de `stop` por la API pública se descartó: `system.stop` pasa por el guardián y no es determinista.

## Gates (sobre el HEAD de la rama)

| Gate | Resultado |
|---|---|
| `cargo test --workspace` | verde |
| `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` | limpios |
| `cargo deny check licenses` | ok |
| Replay completo | 2962/2962 aplicables + 1 no aplicable, 0 divergencias |
| Cobertura (`tools/parity/coverage.sh`) | 98,20 % líneas del crate; las líneas modificadas de producción están cubiertas (sin cubrir sólo dobles de prueba preexistentes) |
| Mutantes | 16/16 DETECTED, control CLEAN: 6 nuevos (`nonblocking-cell-*`, `nonblocking-queue-reports-a-refused-message-as-stored`, `enqueue-unless-closed-ignores-the-queue-answer`, `props-ignores-the-selected-mailbox`) y los 10 existentes que tocan estos archivos; 5 patrones actualizados a la nueva firma |
| `tools/package/check.py` | `failures=0` |

No se usó el runner `tools/gates/gates.py` porque necesita el submódulo `upstream/pekko` inicializado y el clon falló en este entorno; se ejecutó cada gate por separado con los mismos comandos. No se ejecutó la batería global de mutantes.

## Contratos JVM

Los 10 casos del oráculo (`cases/oracle-only/nonblocking-bounded-mailbox.golden`) siguen pasando con la nueva firma (`nb_*_matches_the_oracle`). No se modificó ningún golden ni oráculo. El cableado con actores reales no tiene gemelo grabado propio.

## Riesgos y pendientes

- `Props::with_mailbox` es transitoria (Q-MBX-2); `NB-CF-02` (config) queda para el PR 10.
- `BoundedMailbox` bloqueante (Q-MBX-1) y PROP-7A-1 no se tocan.
- Tamaño: ~110 líneas de producción; el resto son pruebas, mutantes y esta evidencia (~800 líneas en total), por encima del objetivo; el cambio de trait no se puede partir sin dejar la rama rota.
