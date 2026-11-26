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

Rust: the guardians' props take the default type (`Props::with_resolved_mailbox`), declare `Unbounded`, and a failed guardian is stopped by its supervisor
(`BubbleWalker`, as `theOneWhoWalksTheBubblesOfSpaceTime`'s decider), which ends the system. `tests/classic_guardian_mailbox.rs` checks each of the three
individually. `MBX-GRD-01` (`mb-0031`) is identical to the oracle for the three guardian queues.

Not claimed: what `ActorSystem.create` throws on the JVM (the logger start-up timeout; Rakka has no loggers: its `create` returns and the system terminates);
`deadLetterListener` and `eventStreamUnsubscriber` (not in Rakka); `guardianProps`. Selection tests that count the default type's queues now include the 3 guardians.

RED: 2 of 3 new tests failed (guardians ignored the default; the system stayed up). GREEN: 3 passed; replay 2975/2975.
Mutants: 6 (default ignored by each guardian, requirement dropped on `Guardian`/`SystemGuardian`, failed root not ending the system): 6/6 DETECTED, control CLEAN 2974/2974.
