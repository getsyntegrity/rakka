# MailboxType selection through `Props::with_mailbox` (AC-MBX-008, ADR-0001 PR 7/10 prefix)

Slice: `trait MailboxType { fn create(&self, dead_letters: &ActorRef) -> Box<dyn MessageQueue> }`, implemented by `UnboundedMailbox` and
`NonBlockingBoundedMailbox`; `Props::with_mailbox(impl MailboxType)` stores an `Arc<dyn MailboxType>`; the cell asks it once per actor
(restart keeps the queue). No `Mailboxes`, `requirements`, config/dispatcher lookup, blocking `BoundedMailbox` (Q-MBX-1) or priority/control
queues: those stay behind Q-MBX-1/2/3 (all `proposed`). Pekko's `create(owner, system)` becomes `create(dead_letters)` (PROP-7B-1 alt. A,
already approved: no strong reference to the owner).

- RED: `red.txt` (the test file did not compile: `MailboxType` did not exist). GREEN: `green.txt`.
- Integration (`crates/rakka-actor/tests/classic_mailbox_type_selection.rs`, real actors, public API): a LIFO type unknown to the crate
  supplies the queue and is created exactly once; a restart keeps that queue (still one `create`); stop sweeps the custom queue to dead
  letters once each, addressed to the stopped actor; explicit `UnboundedMailbox` is FIFO; the default holds 5000 queued messages in order.
  The stop test stops the actor from inside its own turn: `ActorSystem::stop` goes through the guardian and raced the gate (observed, fixed;
  15/15 stable runs afterwards).
- Oracle: no new case. The slice adds no Pekko-observable behaviour: queue capacity, rejection, dead letters and stop/restart of the
  bounded queue are already covered by the recorded `NB-QU-*`, `NB-QC-01`, `MBX-SC-*` cases; replay `2962/2962 applicable, diverge 0`.
- Manual mutants (control CLEAN, then DETECTED by 3 failing test binaries each): selected type ignored in `Props::new_message_queue`;
  `NonBlockingBoundedMailbox` trait impl with a wrong capacity; `with_mailbox` as a no-op.
- Coverage (`cargo llvm-cov -p rakka-actor`): modified lines of `props.rs`, `message_queue.rs` and the new `MailboxType` impl are 100 %.
