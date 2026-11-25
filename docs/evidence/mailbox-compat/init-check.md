# Slice 4, PR E: `Dispatch.init` re-checks the queue the type really created

Upstream (Pekko 2.0.0 `9fbd2283`): `Dispatch.scala:78-103`. Only when the `MailboxType` is a `ProducesMessageQueue` and the actor requires a type, the
queue the dispatcher really created (`mbox.messageQueue`) must be an instance of the required type; otherwise `Create` carries
`ActorInitializationException(self, "Actor [self] requires mailbox type [req] got [queueClass]")`. The check runs after `getMailboxType` (selection and
`verifyRequirements` look at what the type *declares*), when the cell is made; the failure is delayed into `Create` so `actorOf` returns.

Rust: `MessageQueue::satisfies` (effective semantics, default only `MessageQueue`) and `class_name`; `Props::unmet_queue_requirement`; `ActorCell::build`
puts the failure in `Create`. Declared (`ProducedQueue`, a property of the type) and effective (`satisfies`, a property of the queue) are separate:
a type may promise what its queue lacks, and that is exactly what `MBX-CMP-03` observes (oracle `LiarType`: promises `AuditTrail`, creates a `LifoQueue`).
`MBX-CMP-03` output, identical to the oracle (stable over 3 runs; the uid of the reference is removed on both sides):
`created | failure:...ActorInitializationException: .../kid: Actor [Actor[.../kid]] requires mailbox type [interface ...AuditTrail] got [...LifoQueue] | terminated | again | started=1`.

RED (`dispatch::mailboxes`): 1 failed (the check was a no-op), then GREEN 28 passed; replay 2974/2974.

Mutants (3, replay-visible): 3/3 DETECTED, control CLEAN 2974/2974. `mailbox-recheck-names-the-required-class-as-created` is killed by exactly `mb-0030`;
`mailbox-created-queue-is-not-rechecked` and `mailbox-unbounded-queue-has-no-effective-semantics` by a family fall (30 divergences: the harness runs the family in one
process, so a failure of actors that require Unbounded takes the family down). Their 783-line reports are not committed (regenerable by the command).

## Corrección de la revisión independiente (queda `satisfies` con dos fuentes)

Hallazgo medio: un `MailboxType` que promete `UNBOUNDED` y cuya cola no sobrescribe `satisfies` hacía fallar `Dispatch.init` aunque la promesa fuera honesta
(en Pekko la jerarquía de clases es la única fuente). Ahora `MessageQueue::satisfies` devuelve `Option<bool>`: `None` (por defecto) es una cola que no declara
sus semánticas y vale lo que su tipo prometió; `Some(..)` la sujeta a su propia respuesta (así se sigue encontrando el tipo que promete lo que su cola no tiene,
`MBX-CMP-03`). RED: `a_queue_that_does_not_declare_its_semantics_keeps_the_promise_of_its_type` falló con `Some((UNBOUNDED, ...UndeclaredQueue))`; GREEN con el cambio.
Mutante nuevo `mailbox-undeclared-queue-does-not-keep-the-promise` (unitario). Replay 2974/2974.
