# Slice 4, PR D: MBX-CMP-02 against Pekko 2.0.0 (requirement of the dispatcher, simultaneous requirements)

`MBX-CMP-02` (recorded as `mb-0029`): a default dispatcher whose `mailbox-requirement` is Unbounded; Bounded is bound to `cap4-mailbox`,
`AuditTrail` to `audit-mailbox`. Oracle stable over 3 runs; Rust output byte-identical to the oracle's (actor class spelled as the JVM does).

| Attempt | Outcome (both) |
|---|---|
| mailbox selected by id, bounded queue | `IllegalArgumentException ... [class ...BoundedNodeMessageQueue] does not fulfill requirement for dispatcher [pekko.actor.default-dispatcher]. Must be a subclass of [interface ...Unbounded...]` |
| selected, queue with Unbounded and AuditTrail | created |
| actor requires Bounded, no selection | binding (`cap4`) fails the dispatcher, so the dispatcher's binding is tried and verified: fails the *actor* (`for actor class`, `[class ...UnboundedMailbox$MessageQueue]`) |
| actor requires AuditTrail, no selection | created (the binding fulfills both) |
| actor requires Bounded, `unbounded` selected | the actor's failure (the dispatcher is fulfilled) |
| both unmet (`cap4` selected, actor requires Bounded) | the dispatcher's failure is the one reported |
| nothing required, no selection | created |

`MBX-CMP-01` now shares its attempts helper with this case (output unchanged: same md5 before and after).
The requirement of a dispatcher chosen by `Props` and a dispatcher with its own `mailbox-type` remain debt (Q-MBX-13 limits): the requirement exists only for the default dispatcher.

Replay 2973/2973, 0 divergences. Mutants: 2 new replay-visible (each killed by exactly `mb-0029`), and the 3 unit-detected verification mutants re-run: 5/5
DETECTED, control CLEAN 2973/2973.
