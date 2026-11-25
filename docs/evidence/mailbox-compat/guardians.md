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

| Guardian that fails to initialise | Pekko 2.0.0 (read) | Rakka |
|---|---|---|
| root | the bubble walker runs `causeOfTermination.tryFailure(ex)` and `child.stop()`; it stops walking (`isWalking` false, `:462`), so the death notification of the root is discarded (`:483`), its `stop()` (the only place that links `terminationPromise`, `:467`) never runs: **`whenTerminated` stays pending** | the same: `whenTerminated` stays pending, neither `Ok` nor `Err`; the cause lives only in the walker (`causeOfTermination`) and nothing reads it |
| `/user`, `/system` | the root's strategy stops them without recording a cause (`:525-529`); the system ends normally | `whenTerminated` completes with `Terminated` |
| a requested `terminate()` | normal | `whenTerminated` completes with `Terminated` |

`TerminationFuture` keeps one value (`Promise<Terminated>`); the walker's own `causeOfTermination` is a separate internal promise. Tests: `actor::guardian_termination_tests` (root: waits for the walker
to discard the notification, so a pending future is not just "not yet"; shuts the dispatchers down at the end) and `tests/classic_guardian_mailbox.rs` (`/user`, `/system`, requested termination, and each guardian
takes the default). `MBX-GRD-01` (`mb-0031`) is identical to the oracle for the three guardian queues.

Not claimed: what `ActorSystem.create` throws on the JVM; `deadLetterListener` and `eventStreamUnsubscriber` (not in Rakka); `guardianProps`. Selection tests that count the default type's queues now include the 3 guardians.

RED: `a_failed_root_guardian_leaves_when_terminated_pending` falló con el walker antiguo (la señal de descarte de la notificación del raíz nunca llegó); GREEN con `isWalking` y `causeOfTermination`.
Gates en este HEAD: 819 tests, replay 2975/2975, cobertura de líneas 95,37 % (gate ≥95 %; regiones 95,99 %, funciones 94,98 %), mutantes 8/8 DETECTED con control CLEAN 2975/2975.
