# ADR 0037: `Port` as a Go adaptation; type mismatches fail (Typed semantics)

- Status: **Accepted** by the Owner on 2026-11-05 (wording: "si"), points 1 to 5. Point 5 was **revised on 2026-11-05** by Owner instruction (per-result
  default logging); see the Owner table, Resolutions of 2026-11-05, `Port`. **Citation corrections of 2026-11-05**: two Pekko citations corrected in place; no decision changes (see the last section).
- Date: 2026-11-05
- Stage: 1
- Principle applied: ADR 0033 D14 (behave exactly as Pekko), D14.3 (a deviation needs its own ADR).
- Related: ADR 0030 (D6, E13), ADR 0033 (D1, D2, D14), ADR 0034 (P1, P5, `Port` inventory row), ADR 0035 (supervision), ADR 0018 (persistence),
  audit `docs/parity/audits/port-and-persistence-receiveport-vs-pekko-2026-11-05.md`.
- Placement: `Port` had no ADR of its own. This ADR takes the next free number (0037; no open PR uses it).

## Context

`Port` is the way back into one actor cell for work that runs outside its turn. The source states what it is: "an adaptation API of this port,
not a Pekko API" (`pekko-actor/actor/port.go:14`). Pekko has no such type. Its counterpart is sending a message to the `ActorRef`.

ADR 0033 D14 makes Pekko the source of truth, and D14.3 requires an ADR for every deviation. ADR 0030 D6 decided one rule about `Port`
(posts share the mailbox capacity and may be lost under saturation, as in Pekko) and records "no JVM counterpart" (ADR 0030, "Not verified by the reviewer"). ADR 0034
lists `Port` in an inventory row, in a section headed "Decision (proposed)" (`0034:89`). No ADR decided `Port` itself, nor what happens when a
post cannot be accepted by type.

The audit `docs/parity/audits/port-and-persistence-receiveport-vs-pekko-2026-11-05.md` found two behaviors that differ from Pekko Typed:

1. A post to an instance that is not a `PortHandler` publishes a `DeadLetter` (`port.go:115-119`).
2. The persistence `ReceivePort` ignores a foreign type with `return nil` (`pekko-persistence/persistent/snapshot_ops.go:141-145`).

In Pekko Typed a message the behavior cannot accept by type is a failure: oracle O2 of the audit sent `Integer(42)` to an
`EventSourcedBehavior[String, ...]` and observed a `ClassCastException` before the command handler ran, after which the actor stopped under
the default Typed supervision. A Classic `PersistentActor` instead published `UnhandledMessage` and stayed alive (O1). Neither deviation is
reachable today: the only producer of `Port.Post` is `postGuarded` (`snapshot_worker.go:242-245`) and it posts only snapshot answers.

## Decision

1. **`Port` is a Go adaptation.** Its API belongs to gopekko; its Pekko counterpart is the send to the `ActorRef`. Admission, order and capacity
   are those of ADR 0030 D6 (not repeated here). That the API does not exist in Pekko does not allow a different behavior: each observable case
   is decided by D14.
2. **An instance that is not a `PortHandler`:** a post to it is a failure of the actor and goes through its supervision, as a message of the
   wrong type does in Pekko Typed. This replaces the `DeadLetter` published today.
3. **Persistence `ReceivePort` with a foreign type:** it is a failure of the actor and goes through its supervision. This replaces the
   `return nil` of today.
4. **Why a failure and not `UnhandledMessage`.** `UnhandledMessage` is the Classic semantics (the `PersistentActor` stays alive, O1). The scope
   of goPekko is Typed, per ADR 0033 D1 and D2 (a public `Behavior[T]` layer; the Typed profile has its own contract). ADR 0033 has no
   clause that says "type mismatch fails", so the reason is stated as "per ADR 0033 scope" and D14. The decision depends neither on EVT-1 nor on
   the existence of an `UnhandledMessage` type. Note: ADR 0033 lists `persistence-typed` as out of its own scope (`0033:76`); persistence is
   governed by ADR 0018. D14 applies to it as a project-wide rule: see the "Amendment (2026-11-05): scope of D14" at the end of ADR 0033,
   and D14.4 there (ADRs from 0035 onward apply the principle, so this ADR does).
5. **Default logging of a snapshot result that the receiver does not handle** (replaces the second `return nil` of `ReceivePort`,
   `snapshot_ops.go:146-148`), by D14. In Pekko 1.7.1 the snapshot result reaches the behavior as a signal
   (`persistence-typed/.../internal/Running.scala:887-925`, emitted through `setup.onSignal` at `:921`). The default handler is **only a
   fallback, applied per signal**: `EventSourcedBehaviorImpl.scala:156` builds `signalHandler.orElse { ... }` with the comment "default signal
   handler is always the fallback", so the handler of the user runs first and the default acts on each signal that it does not take.
   The observable requirement for gopekko: the default log applies to **each** snapshot result that the receiver does not handle, and a
   receiver that does not handle a result does not silence it. With the project logger:
   - `SnapshotCompleted` (`SaveSnapshotSuccess`), at DEBUG: `Save snapshot successful, snapshot metadata [{}].` with the metadata
     (`EventSourcedBehaviorImpl.scala:158-159`);
   - `SnapshotFailed` (`SaveSnapshotFailure`), at ERROR: `Save snapshot failed, snapshot metadata [$meta] due to: ${failure.getMessage}`, with the
     failure as the cause of the log entry (`:160-162`).

   This also applies when the Behavior does not implement `SnapshotResultReceiver` at all. The current interface is all or nothing
   (`ReceiveSnapshotResult(ctx, result) error`, `snapshot_ops.go:92-94`): it cannot say that a given result was not handled, so it has to change.
   How the receiver says it did not handle a result is an API detail and is outside this ADR. Implementation: issue #1344.

   Basis for "nothing is published to the event stream": the source. `Running.scala:921-923` calls `onSignal`, which returns true because the
   default handler took the signal, and only a debug log follows; nothing reaches `UnhandledMessage`. Oracle U4 of `docs/parity/oracles/evt-1-unhandled-dropped-2026-11-05.md` corroborates the
   absence of an event for a handled or unhandled signal, but it ran with a plain Typed behavior, not an `EventSourcedBehavior`. The remaining
   entries of the default handler (`:163-181`, deletions and `GetPersistenceId`) are not part of this point.

Docs: [EventSourcedBehavior signals](https://pekko.apache.org/docs/pekko/current/typed/persistence.html).

## Consequences

- Decisions 2 and 3 are unreachable today. They are implemented when a new producer of `Port.Post` exists, or together with a test-only
  `PortHandler` that posts a foreign value, whichever comes first. Decision 5 is reachable today (the snapshot worker posts `SnapshotResult`) and has its own implementation issue: #1344.
- Requirement for any PR that adds a producer of `Port.Post`: say which types it posts and confirm that the receiver is a `PortHandler` that
  accepts them.
- Additive notes: ADR 0030 (after the two places that describe `Port`: the "Port in detail" paragraph and D6) and ADR 0034 (status line) get a
  "See ADR 0037" note. Nothing is rewritten.

## Not covered

- The `ReceivePort` of the Typed interpreter (ADR 0034 P1) needs its own audit with the criterion of that audit.
- Pekko also logs the snapshot result outside the default handler, whatever the user handler does: debug "Persistent snapshot [{}] saved
  successfully" and warn "Failed to save snapshot given metadata [{}] due to: {}" (`Running.scala:890`, `:911`). Whether gopekko must log them is not
  decided here.

## Alternatives rejected

- Publish `UnhandledMessage`: it is the Classic semantics (reason in 4).
- Keep `DeadLetter` and `return nil`: a deviation with no justification under D14.

## Post-merge checklist

Each reference below is plain text today and becomes a path once its PR is on `develop`. Current text → target:

- [x] `docs/adr/0037-port-as-a-go-adaptation.md`, Related (line 8): "(PR #1342)" → remove; keep the path
  `docs/parity/audits/port-and-persistence-receiveport-vs-pekko-2026-11-05.md`.
- [x] Same file, Context (line 21): "The audit of PR #1342" → "The audit `docs/parity/audits/port-and-persistence-receiveport-vs-pekko-2026-11-05.md`".
- [x] Same file, Decision 5: "oracle U4 of PR #1337" → `docs/parity/oracles/evt-1-unhandled-dropped-2026-11-05.md`, scenario U4.
- [x] Same file, Not covered: "the criterion of PR #1342" → the same audit path.
- [x] `docs/parity/typed-owner-decisions-2026-10-27.md`, Resolutions of 2026-11-05, `Port` (line 139): "the audit of PR #1342" → the same audit path.
- [x] Retarget this PR's base to `develop` when #1338 merges.

## Amendment (2026-11-05): citation corrections (#976 slice 4)

Every Pekko citation of this ADR was re-read from `upstream/pekko` at v1.7.1 / `9a54d12`. Two were off and are corrected in place; no decision, claim or status changes: the remaining default-handler entries of `EventSourcedBehaviorImpl` are `:163-181` (`DeleteEventsFailed` ends at 180 and `GetPersistenceId` is at 181; the text said `:163-176`), and the debug "Persistent snapshot [{}] saved successfully" of `Running.scala` is line 890 (889 is the `case SaveSnapshotSuccess(meta) =>`; the warn at `:911` was right). The other citations matched as written.
