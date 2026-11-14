# Actor core, first cut: approved subset and limits

Status: a scoped decision record for this first cut of `pekko-actor`. It is **not** an ADR. ADR 0008 and ADR 0010 stay **Proposed**;
nothing here accepts them, and nothing here claims parity with Pekko.

## What this cut contains

A system, `Spawn` with a factory, `Ref[T]` and `Tell`, serial processing, an injectable executor (internal), start and stop lifecycle
(`PreStarter`, `PostStopper`, `StopSelf`, `Stop`, `Done`), and dead letters with `SubscribeDeadLetters`.

Exported (the whole public surface): `ID`, `Handle`, `Ref[T]`, `Actor[T]`, `PreStarter[T]`, `PostStopper[T]`, `Context[T]` (`Self`, `StopSelf`),
`System`, `NewSystem`, `Config{Throughput}`, `Spawn`, `System.Stop`, `System.Done`, `DeadLetter`, `System.SubscribeDeadLetters`,
`ErrNilActor`, `ErrFactoryPanic`, `ErrNameTaken`, `ErrUnknownHandle`, `FactoryPanicValue`. (Since U4, ADR 0017 S10:
`ErrNilActor`, `ErrFactoryPanic` and `FactoryPanicValue` are the **cause** of an `InitializationError` that a decider or an
event receives, not errors that `Spawn` returns; this first cut described the earlier contract.)

Kept unexported on purpose, and tested from inside the package: the executor seam, the event sink, `stats`, `deadLettersPublished` and
`deadLetterBarrier`. They exist to prove behavior; making them public would freeze an API nobody has asked for.

## Behavior this cut commits to

- Per-producer FIFO; one `Receive` at a time per actor; `Throughput` steps per activation (below 1 means 1).
- `Received` counts a message taken from the queue, not the end of `Receive`. Conservation: `Told == Received + DeadLettered + Depth`.
- After a stop is accepted nothing else is received; waiting and late messages become dead letters, each published once; `PostStop` runs once
  if the start step began; `Done` closes after `PostStop` and the dead-letter cleanup; the name is free again after `Done`.
- A failure after an already accepted stop is reported, and no second stop event is emitted.
- A nil message for an interface `T` is delivered as the zero `T`. IDs are opaque.
- `runtime.Goexit` in a hook, in `Receive` or in the factory is treated as a failure, not as a silent exit (since U4 a
  factory failure is an `InitializationError` supervised by the parent, and `Spawn` does not call the factory).
- The dead-letter barrier (internal): returns nil once publications that started before the call have finished calling their subscribers;
  does not wait for later ones; returns the context error when the context ends first. It must not be called from an actor or from a
  subscriber, because it may wait for that very call. The runtime never uses it to block actor processing.

## Limits (not in this cut)

Children, restart, supervision, watch, ask, tenancy, observers, persistence, remote and cluster. No bounded mailbox by default (opt-in `WithMailboxCapacity`, ADR 0030). A subscriber that
panics or calls `runtime.Goexit` is not recovered, and if it does so while the finish step publishes dead letters, `Done` may never
close. A `PostStop` that calls `runtime.Goexit` leaves the finish step incomplete (the name stays held). A handler that calls
`runtime.Goexit` and then panics in its own deferred call, with that panic recovered, strands the activation. All three are known,
contrived edges, found by review, not covered by this cut. The system keeps every
spawned cell for its lifetime so that stale handles resolve (memory grows with the number of actors ever spawned).

## What these tests do and do not show

They run under `-race` on a few machines and repetitions; that is evidence, not proof, of absence of races. Nothing here shows parity with
Pekko, and the judge harness and recorder approval are not involved.
