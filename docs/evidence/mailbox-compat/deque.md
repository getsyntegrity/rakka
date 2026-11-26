# AC-MBX-011: `UnboundedDequeBasedMailbox` y `enqueueFirst` (rebanada de la cola)

Pekko 2.0.0: `Mailbox.scala:535-549,600-622,851-865`. Caso grabado MBX-DQ-01 (`mb-0032`), esperado producido por la JVM.

## TDD

RED (tests escritos antes que el código; `cargo test -p rakka-actor --lib deque_mailbox`):

```
error[E0422]: cannot find struct, variant or union type `Envelope` in this scope
error[E0425]: cannot find value `UnboundedDequeBasedMailbox` in this scope
error[E0405]: cannot find trait `MessageQueue` in this scope
error: could not compile `rakka-actor` (lib test) due to 12 previous errors
```

GREEN (`-- deque_mailbox message_queue`): `test result: ok. 15 passed; 0 failed`.

Los tests de integración (`tests/classic_deque_mailbox.rs`) se escribieron con el componente ya implementado: pasaron en la primera
ejecución y no son evidencia de RED.

## Pendiente

- Replay completo, `cargo test --workspace`, cobertura, `cargo deny`.
- Mutantes `deque-enqueue-first-goes-to-the-tail` y `deque-dequeue-takes-the-tail`: definidos, sin ejecutar.
- Decisiones: Q-MBX-2 (semánticas Deque y `requirements`), Q-MBX-14 y Q-MBX-15 (`proposed`), Q-MBX-1 (acotada). Stash.
