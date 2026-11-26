# Slice 4, PR F: the guardians (root, `/user`, `/system`)

Upstream (Pekko 2.0.0 `9fbd2283`): `ActorRefProvider.scala:318-345` (`Guardian` and `SystemGuardian` both `RequiresMessageQueue[UnboundedMessageQueueSemantics]`),
`:543-606` (root, `/user` and `/system` take `defaultMailbox = mailboxes.lookup(DefaultMailboxId)` directly, not `getMailboxType`; `/user` takes the mailbox of
`guardianProps` instead when the system was given them), `Dispatch.scala:78-103` (the requirement is checked only there, only if the type promises its queue).
Probed on the oracle JVM (JDK 17) with a default mailbox that creates a queue without the unbounded semantics for the N-th creation (creation order: root, `/user`,
`/system`, then `deadLetterListener`, `eventStreamUnsubscriber`): N = 1, 2 and 3 each make `ActorSystem.create` throw (`ConfigurationException`, from the logger
start-up timeout after the guardian logs `guardian Actor[pekko://g/] failed, shutting down!`); a type that promises nothing, or an unbounded queue, starts normally.

| Guardian | Requirement | Mailbox | Check |
|---|---|---|---|
| root | `Unbounded` | registered default | `Dispatch.init`, if the default type promises its queue |
| `/user` | `Unbounded` | registered default (`guardianProps` mailbox: not in Rakka) | same |
| `/system` | `Unbounded` | registered default | same |

Rust: the guardians' props take the default type (`Props::with_resolved_mailbox`), declare `Unbounded`, and the failure of a guardian follows `ActorRefProvider.scala:455-490,525-529`
(read from the Scala; **not probed on the JVM**, where `ActorSystem.create` already fails by the logger start-up timeout, so the termination future is not observable there):

| Path | Pekko 2.0.0 (read) | Rakka |
|---|---|---|
| root fails, then its death notification arrives | the walker runs `causeOfTermination.tryFailure(ex)` and `child.stop()`; it stops walking (`isWalking` false, `:462`), so the notification is discarded (`:483`) and `stop()` (the only place that links `terminationPromise`, `:464-468`) does not run: **`whenTerminated` stays pending** | the same: pending, neither `Ok` nor `Err` |
| root fails, then `stop()` is called on the walker directly | `stop()` tries `causeOfTermination.trySuccess(Terminated)` (a no-op, it already failed) and `terminationPromise.completeWith(causeOfTermination.future)`: **the public future fails with the cause** | the same: `Some(Err(cause))` (`TerminationFuture` is one `Result<Terminated, Throwable>`, write-once) |
| `/user`, `/system` fail | the root's strategy stops them without recording a cause (`:525-529`); the system ends normally | `Some(Ok(Terminated))` |
| a requested `terminate()` | normal | `Some(Ok(Terminated))` |

The walker's `causeOfTermination` is a separate internal promise. Tests, all deterministic: `actor::guardian_termination_tests` drives a standalone walker message by message (the two root paths, and a signal that is set only by the
discarded `DeathWatchNotification` of the root, never by another message or actor), the root test over a real system waits for that signal before asserting the future is pending, and a RAII guard shuts the dispatchers down even when
the test unwinds (`the_guard_shuts_the_dispatchers_down_when_the_test_unwinds`); `tests/classic_guardian_mailbox.rs` covers `/user`, `/system`, a requested termination and that each guardian takes the default.
`MBX-GRD-01` (`mb-0031`) is identical to the oracle for the three guardian queues.

Not claimed: what `ActorSystem.create` throws on the JVM; `deadLetterListener` and `eventStreamUnsubscriber` (not in Rakka); `guardianProps`. Selection tests that count the default type's queues now include the 3 guardians.

RED/GREEN (cada uno con su test, `cargo test -p rakka-actor --lib guardian_termination`): (3) `stop_called_directly_after_the_failure_fails_the_public_future_with_the_cause` falló con "completed by stop()" y pasó al enlazar `terminationPromise` al resultado interno;
(2) `only_the_death_notification_of_the_failed_root_is_the_synchronisation_signal` falló en `!walker.notification_discarded.is_completed()` con la señal genérica y pasó con la señal específica;
(1) `the_guard_shuts_the_dispatchers_down_when_the_test_unwinds` falló en `walker.dispatchers_are_shut_down()` con un guard sin efecto y pasó con el `Drop` que cierra los dispatchers.
Gates en este HEAD: 823 tests, replay 2975/2975, cobertura de líneas 95,33 % (gate ≥95 %; regiones 95,96 %, funciones 94,90 %), mutantes 11/11 DETECTED con control CLEAN 2975/2975, fmt/clippy/deny/package check limpios.

Corrección de la revisión de F3: el test del raíz **no** armaba el guard (el reemplazo del parche no coincidió) y dejaba 12 threads vivos. RED: `a_failed_root_guardian_leaves_when_terminated_pending` falló en `walker.dispatchers_are_shut_down()`; GREEN con el guard dentro del test. Ahora el mutante `dispatchers-guard-does-not-shut-down` lo matan dos tests.
Tras ese último cambio (sólo tests) se corrieron: fmt, clippy, 823 tests y el mutante del guard con control CLEAN 2975/2975; replay (2975/2975) y cobertura (líneas 95,33 %) son del HEAD anterior, con el mismo código de producción.
