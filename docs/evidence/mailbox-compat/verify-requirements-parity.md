# Slice 4, PR C: MBX-CMP-01 against Pekko 2.0.0

`MBX-CMP-01` (oracle `MailboxOracle`, recorded as `mb-0028`): seven `actorOf` attempts with a selected mailbox. Oracle stable over 3 runs (same md5);
Rust output byte-identical to the oracle's once the actor class is spelled as the JVM does.

| Attempt | Outcome (both) |
|---|---|
| requires Unbounded, mailbox whose type promises nothing (its queue does have `AuditTrail`) | `ConfigurationException`, cause `IllegalArgumentException ... [interface ...MessageQueue] does not fulfill ... actor class [class ...]` |
| requires AuditTrail / Bounded, `unbounded` | same, queue `[class ...UnboundedMailbox$MessageQueue]` |
| requires Unbounded or AuditTrail, mailbox promising both | created |
| requires `MessageQueue`; requires nothing; type promises nothing | created |

Declared vs effective: a type that does not promise its queue is rejected even though the queue it creates has the capability (pre-creation check uses
`ProducesMessageQueue`; the post-creation `Dispatch.init` check is not implemented yet).

Normalization (the only one): the JVM class name of the actor (`pilot.MailboxOracle$UnboundedWorker`) replaces its Rust type path in the message.
Replay 2972/2972, 0 divergences. Mutants (3, replay-visible): 3/3 DETECTED, each by exactly `mb-0028` (diverge 1); control CLEAN 2972/2972.
