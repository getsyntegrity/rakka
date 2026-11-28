# Texto de `ActorRef` con `#uid` (AC-DW-007, AC-CELL-004): RED y GREEN

Base `9f752b15`. Esperados grabados del oráculo (Pekko 2.0.0, JDK 17) con `rust-anything app record`.

## RED (producción sin cambiar)

`rust-anything app replay . --case <caso> --build target/release/examples/classic-parity`: `0/1 cases passed` (`stdout`) en `rt-death-pact-order`, `rt-incarnation-text` y `mb-0030`. `rt-guardians-unhandled`, `ww-rejects` y `ww-restart`: `1/1`.

`cargo test -p rakka-actor --test classic_actor_ref_text`: `1 passed; 2 failed`.

```
---- the_text_of_a_ref_carries_its_uid_only_when_it_has_one ----
  left: "Actor[pekko://t-ref-text/user/child]"
 right: "Actor[pekko://t-ref-text/user/child#264587204]"
---- a_monitor_that_does_not_handle_terminated_fails_with_a_death_pact_and_stops ----
  left: "org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://t-death-pact/user/$a/$a]] terminated"
 right: "org.apache.pekko.actor.DeathPactException: Monitored actor [Actor[pekko://t-death-pact/user/$a/$a#-806583390]] terminated"
```

El test de los guardianes pasó desde el principio: es caracterización, no TDD.

Tras la revisión, el test del death pact lee los `Error` de `logFailure` (su `logSource` es el hijo fallado, el `lastSender` del spec) en vez de la causa que veía el decider; se añadió con la producción ya corregida, así que no tiene RED propio.

## GREEN

Los mismos comandos: `3 passed`; los seis casos `1/1`; `cargo test -p rakka-actor --test recorded_pekko`: `23 passed`.

## Arnés: qué ocultaba la diferencia

- `MailboxOracle.java` borraba `#<uid>` y `mailbox_cases.rs` cortaba el mensaje en el primer `#`.
- `Scenario.java` y `scenario.rs` sustituían cualquier número por `#uid` sin comprobar de quién era (sólo en `watch`).

Ahora sólo se sustituye el uid del propio actor, en toda la salida del escenario; con varias encarnaciones del mismo path, por `#uid:<etiqueta>`. Un uid ausente o ajeno queda como se imprimió y diverge. Se regraba una sola grabación existente, `mb-0030` (gana `#uid` y el resto del mensaje); `ww-rejects` y `ww-restart` quedan idénticas.
