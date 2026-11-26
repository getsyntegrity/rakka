# Slice 4, PR B: `verifyRequirements` at mailbox selection

Upstream (Pekko 2.0.0 `9fbd2283`): `Mailboxes.scala:157-204`, probed against the oracle JVM (JDK 17) with a throwaway class (cases follow in PR C).

| Probe | Pekko 2.0.0 |
|---|---|
| actor requirement unmet by the selected type | `actorOf` throws `ConfigurationException`, cause `IllegalArgumentException: produced message queue type [interface org.apache.pekko.dispatch.MessageQueue \| class org.apache...UnboundedMailbox$MessageQueue] does not fulfill requirement for actor class [class X]. Must be a subclass of [interface Y].` |
| dispatcher requirement unmet | same, `... for dispatcher [pekko.actor.default-dispatcher]. ...` |
| actor binding does not fit the dispatcher | the dispatcher's binding is tried: `catch NonFatal if hasMailboxRequirement` wraps the *verification* too |
| no requirement | any selected type is accepted |

Pending (confirmed by the probe): `Dispatch.init` re-verifies the queue really created, only for a `ProducesMessageQueue` type and an actor that
requires a type; the failure is delayed into `Create` (`ActorInitializationException`): `actorOf` returns, the actor terminates, its name is freed.

Rust: `Props::resolve_mailbox` verifies the selected type (dispatcher first, then actor); only the binding of the actor's requirement has the
fallback. The actor class is `std::any::type_name`. `Props::with_mailbox(type)` (no Pekko counterpart) is not verified.

RED (`cargo test -p rakka-actor --lib dispatch::mailboxes`): 5 failed, 24 passed; GREEN: 26 passed. Eight older tests then failed because they
bound the unbounded semantics to a bounded mailbox (Pekko rejects it); they use a stand-in that declares its semantics.
Mutants: 6 new + 11 existing mailbox-requirement ones: 17/17 DETECTED, control CLEAN 2971/2971 (the first run left
`mailbox-fallback-binding-is-not-verified` UNDETECTED; the test now covers it).
