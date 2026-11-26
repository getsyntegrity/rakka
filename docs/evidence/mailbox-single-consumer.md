# AC-MBX-006: `SingleConsumerOnlyUnboundedMailbox`

Pekko 2.0.0: `Mailbox.scala:412-434,699-708`; spec `MailboxConfigSpec.scala:279-329`. Caso grabado MBX-SCQ-01 (`mb-0033`), esperado producido por la JVM.

- Cola ilimitada, varios productores y un consumidor, FIFO; `pathologicalPingPong` portado (sin límite y acotada). Declara lo que declara `NodeMessageQueue` (`Mailbox.scala:412`): `UnboundedMessageQueueSemantics`, sin `MultipleConsumerSemantics` ni `Bounded...`, clase `org.apache.pekko.dispatch.NodeMessageQueue` (Q-MBX-14, aceptada con cambios por el Owner el 2026-11-26).
- Límite: la cola de Pekko es la de Vyukov, sin bloqueos; ésta guarda un `Mutex` sólo durante el push o el pop (comportamiento observable, no velocidad); si un hilo hace panic con el cerrojo tomado, la cola queda inutilizable (`lock().unwrap()`), un modo que la de Pekko no tiene. AC-MBX-006 sigue `partial`.

## TDD

RED 1 (`cargo test -p rakka-actor --lib single_consumer`): `error[E0425]: cannot find value SingleConsumerOnlyUnboundedMailbox`, 7 errores de compilación. GREEN 1: 3 passed.

RED 2 (declaración de semánticas, antes de implementarla): `left: [None, None, None, None]`, `right: [Some(true), Some(true), Some(false), Some(false)]`. GREEN 2: 4 passed.

## Contra la JVM y verificado

- `mb-0033`: clase, `unbounded=true multipleConsumer=false bounded=false`, cola vacía, FIFO, conteos y `cleanUp`. Era `mb-0028`; renumerado al reapilar (la pila ocupa `mb-0028`..`mb-0032`) y regrabado con `rust-anything app record`; su `stdout` es idéntico, byte a byte, a la línea del golden de la JVM.
- Los dos ping-pong de 2.000.000 de mensajes pasan (`cargo test --release`, 9,4 s) y dentro de `cargo test --workspace`. Tres mutantes `single-consumer-*`: resultados en el PR.
