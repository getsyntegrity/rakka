# Slice 4, PR A: the queue a mailbox type promises (`ProducesMessageQueue`)

Upstream (Pekko 2.0.0 `9fbd2283`): `Mailbox.scala:672` (`ProducesMessageQueue[T <: MessageQueue]`), `Mailboxes.scala:137-153`
(`getProducedMessageQueueType`: a type without the marker produces `classOf[MessageQueue]`).

Model: `MailboxType::produced_queue() -> ProducedQueue` (class name + every semantics the queue satisfies; default `ProducedQueue::PLAIN`).
Pekko tests `requirement.isAssignableFrom(queueClass)`, so the subtyping between markers (for example `UnboundedDequeBasedMessageQueueSemantics`
extends `UnboundedMessageQueueSemantics` and `DequeBasedMessageQueueSemantics`) is listed by the type instead of derived: the set of
`MessageQueueSemantics` stays open (Q-MBX-13) and a third party lists the markers its queue extends. `MessageQueue` is satisfied by every queue.
No verification happens in this PR (that is PR B); the declarations of `UnboundedMailbox`, `NonBlockingBoundedMailbox` and of the parity
harness mailboxes (`Lifo`, `AuditTrail`, which the oracle declares with `ProducesMessageQueue`) are what PR B verifies.

| Gate | Result (HEAD of the branch) |
|---|---|
| `cargo test --workspace`, clippy `-D warnings`, `cargo fmt --check`, `cargo deny check licenses`, `tools/package/check.py` | green |
| Coverage (`tools/parity/coverage.sh`) | 98.22 % regions, 98.26 % lines |
| Replay | 2971/2971, 0 divergences |
| Mutants of this PR (`check_divergence.py`, killed by unit tests) | 6/6 DETECTED, control CLEAN 2971/2971 |
| `python3.12 -I -m unittest discover -s tools/gates` | 4 failures, all from the 3 anchors already stale on the base (`config-push-timeout-not-read`, `deploy-fallback-keeps-the-tags`, `render-concatenation-keeps-white-space-as-a-separate-string`) |
