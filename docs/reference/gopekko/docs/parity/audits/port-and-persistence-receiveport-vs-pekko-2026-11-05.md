# Parity audit: `Port` and the persistence `ReceivePort` against Pekko 1.7.1

Audit only. No code, ADR or status changed. Follows the untyped-delivery-paths audit (`untyped-delivery-paths-2026-11-05.md`). Pekko paths are relative to
`upstream/pekko` (v1.7.1), gopekko paths to the repository root. The ADR 0033 D14 / D14.3 text is read from [ADR 0033](../../adr/0033-actor-typed-scope-and-architecture.md).

## Answers

| # | Question | Answer |
|---|---|---|
| 1 | Does `Port` have a Pekko equivalent? | Not as a type or API. The need (work outside the turn hands its result back) is met in Pekko by sending a message to the actor's `ActorRef`. Classic accepts `Any`. Typed `EventSourcedBehavior` accepts `Any` in an interceptor and wraps it (`persistence-typed/.../internal/EventSourcedBehaviorImpl.scala:213-226`; `JournalProtocol.Response` → `JournalResponse`, `SnapshotProtocol.Response` → `SnapshotterResponse`, anything else → `IncomingCommand(cmd.asInstanceOf[Command])`). |
| 2 | Is there an ADR that justifies `Port`? | Partly. ADR 0030 D6 (accepted) decides that Port posts share the mailbox capacity and may be lost under saturation as in Pekko (`docs/adr/0030-bounded-mailbox.md`, decision D6; row E13 of its table). ADR 0030 itself records "Port (no JVM counterpart)" as not verified on the JVM (its bullet "Not verified by the reviewer"). `port.go:14` calls it "an adaptation API of this port, not a Pekko API". ADR 0034 lists it in an inventory row (`docs/adr/0034-typed-behavior-boundary.md:89`) under a section headed "Decision (proposed)". No ADR decides `Port` as a deviation in the sense of D14.3. |
| 3 | What does Pekko do with a message `EventSourcedBehavior` does not handle? | Four cases, in the table below. |

### Pekko, by kind of message (source, then oracle)

| Message | Pekko source | Oracle result (2 runs, identical) |
|---|---|---|
| Wrong-class command to a Typed `EventSourcedBehavior` | `EventSourcedBehaviorImpl.scala:225`: unchecked cast, no check | O2: `ClassCastException` (Integer to String) before the command handler body runs; the actor fails and, under the default Typed supervision, stops; the next valid command is a `DeadLetter` |
| Wrong-class message to a Classic `PersistentActor` | `Eventsourced.scala:311,339`: `unhandled` falls to `super.unhandled` | O1: `UnhandledMessage(42, sender=deadLetters, recipient=classic1)` on the event stream; the actor keeps working |
| Internal protocol message in a state that cannot take it | `Running.scala:283` (the catch-all of `HandlingCommands`) and, in the `PersistingEvents` state while it waits for the journal write, `:736-737` (`Running.scala:714` is the class): `Behaviors.unhandled` | not run (state-dependent; source only) |
| Command handler returns `Effect.unhandled` | `Running.scala:671-674`: publishes `UnhandledMessage(msg, deadLetters, self)` | not run (source only) |

## Per path

| Path | Pekko behavior | gopekko today | Verdict |
|---|---|---|---|
| `Port.Post(any)`: admission, order, capacity, stop | message to the `ActorRef`; mailbox order; stopped actor sends to dead letters | `port.go:40`, `:76-95`; same observable result per ADR 0030 D6 | matches Pekko (mechanism is an adaptation) |
| `Port` as a concept | no Pekko API | exists; ADR 0030 D6 covers only capacity | deviates from "no extra API"; no ADR decides it |
| `Post` to an instance that is not a `PortHandler` (`port.go:115-119`, doc at `:57-58`) | Classic: `UnhandledMessage` (O1); Typed: failure (O2) | `DeadLetter` published once | deviates (event kind). gopekko has no `UnhandledMessage` type yet (`grep` finds none in `pekko-actor`, `pekko-actor-typed`, `pekko-persistence`) |
| persistence `ReceivePort` with a foreign type (`snapshot_ops.go:141-145`) | Classic: `UnhandledMessage` (O1); Typed: failure (O2) | silently ignored (`return nil`) | deviates |

## Reachability (read from the code, not run)

`Port.Post` has one producer in the repository, `postGuarded` in `pekko-persistence/persistent/snapshot_worker.go:242-245`, and it posts only
the snapshot store answers (`SnapshotResult`). The persistence `typed[T]` keeps the `Port` in an unexported field (`snapshot_ops.go:136`), so
application code cannot post to it. A foreign type reaches `ReceivePort` only if another module that holds the `Port` posts it, or if a restart
builds an instance of another type (the `PortHandler` doc says this can happen, `port.go:57-58`). Neither exists today. The two deviations are
therefore not observable in the current code.

The second `return nil` in `ReceivePort` (`snapshot_ops.go:148`, the Behavior does not implement `SnapshotResultReceiver`) is a different
question (Pekko tells the application through signals, not messages). It is not audited here.

## Proposals for Owner decision (not applied)

1. **Port as a concept:** an ADR that records it as a Go adaptation under D14.3, with the Pekko mechanism (send to the `ActorRef`) as its
   counterpart and ADR 0030 D6 as its capacity rule. No code change.
2. **Non-`PortHandler` instance:** when `UnhandledMessage` exists (ADR 0036 EVT-1), publish it in place of the `DeadLetter` at `port.go:117-119`.
   Until then, keep the `DeadLetter`.
3. **Persistence `ReceivePort`:** replace `return nil` for a foreign type with a failure, as Typed Pekko does (O2). No test reaches it today,
   so the change needs one that posts a foreign value through a test-only `PortHandler`.

## Oracle scenarios

Harness (not committed): sbt 1.10.7, Scala 2.13.18, JDK Temurin 11.0.32, `pekko-persistence-typed` 1.7.1, in-memory journal. Two actors under
one Typed guardian: a Typed `EventSourcedBehavior[String, String, Int]` and a Classic `PersistentActor` accepting `String`. An event-stream
subscriber prints `UnhandledMessage` and `DeadLetter`; the harness subscribed to all events, so journal and logging traffic appears in the raw
output and is not part of the result.

1. O0: valid `"ok"` to both: both persist it.
2. O2: `Integer(42)` through `toClassic !` to the Typed actor: `ClassCastException`, no handler call, then `"after"` is a `DeadLetter`.
3. O1: `Integer(42)` to the Classic actor: `UnhandledMessage`; a following `"ok"` is handled.

Limits: one machine, one JDK; supervision of the Typed actor is the default (stop), so O2 does not show restart or resume.

Correction (2026-11-05, #976 slice 4): in section 4, the row "Internal protocol message in a state that cannot take it" cited `Running.scala:283,292`. Line 283 is right; line 292 (`else Behaviors.unhandled` after `setup.onSignal` returned false) is the path of an unhandled *signal*, not of a protocol message, so it is removed from the row; `Running.scala:736-737` (`RecoveryTickEvent`, `RecoveryPermitGranted` in `PersistingEvents.onMessage`, the state that waits for the journal write; the class starts at 714) is added as a protocol-message case. The file is `persistence-typed/.../persistence/typed/internal/Running.scala`. Every other citation was re-read and matched.
