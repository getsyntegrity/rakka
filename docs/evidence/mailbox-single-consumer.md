# AC-MBX-006: `SingleConsumerOnlyUnboundedMailbox`

Pekko 2.0.0: `Mailbox.scala:412-434,699-708`; spec `MailboxConfigSpec.scala:279-329`. Caso grabado MBX-SCQ-01, esperado producido por la JVM.

## TDD

RED (tests antes que el código; `cargo test -p rakka-actor --lib single_consumer`):

```
error[E0425]: cannot find value `SingleConsumerOnlyUnboundedMailbox` in this scope
error[E0405]: cannot find trait `MessageQueue` in this scope
error: could not compile `rakka-actor` (lib test) due to 7 previous errors
```

GREEN: `test result: ok. 3 passed; 0 failed`.

## Pendiente

- Ejecutar `tests/classic_single_consumer_mailbox.rs` (2.000.000 mensajes): diferido mientras corre la campaña de mutantes.
- Replay completo, `cargo test --workspace`, cobertura, `cargo deny`; mutantes `single-consumer-*` definidos, sin ejecutar.
- La rama de integración tiene 4 fallos previos en `tools/gates/test_mutant_patterns.py` (anclas obsoletas y contador), que repara #372.
- El caso se grabó como `mb-0028`; la pila de mailbox también usa ese número: hay que renumerar y regrabar al integrar.
