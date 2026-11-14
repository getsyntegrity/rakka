# Shared foundations for Typed: clock, event stream and message-adapter ref (audit #004, block 4, 2026-10-26)

> **Reconciliation 2026-10-27 (no decision taken).** F2 misstated Pekko: after the owner stops, the `FunctionRef` forwards to
> `deadLetters` (`actor/ActorRef.scala:862-866, 986-990`), so the DeadLetter recipient is `deadLetters` (or the owner before
> `stopFunctionRefs`), never the adapter ref; goPekko's `To: owner ID` is closer to Pekko than stated. F1 is a deferred capability owned by
> #888, not an observable difference to accept (B-5 in [`typed-owner-decisions-2026-10-27.md`](typed-owner-decisions-2026-10-27.md)).
> "Difference F1" in the answer table and in sections 4 and 5 reads accordingly. The text below is kept as written on 2026-10-26.

**Status: documentation only.** This document checks whether three mechanisms that Typed needs require new `pekko-actor` exports. For
each one it records which part of #986, #999, #925 and #975 the mechanism would enable, and which blockers remain.

Nothing is implemented, accepted or changed on GitHub. ADR 0034 and ADR 0035 are **proposed**, and so is the justification form of
Owner decision 6 (readiness document, section 5). The records in section 6 use that form **as a proposal**.

Baselines:
- goPekko `develop` `13fb9953879462adba4218114d6615dc3d3156b9`.
- This branch is stacked on PR #1211 (`b259e1e`), which is stacked on PR #1210.
- Apache Pekko v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.

Path prefixes:
- `A` = `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor`
- `T` = `upstream/pekko/actor-typed/src/main/scala/org/apache/pekko/actor/typed`
- `TT` = `upstream/pekko/actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed`
- `G` = `pekko-actor/actor`

Evidence levels: SPEC (a test asserts it, body opened) and READ (code only; "no spec found" is not "no oracle"). Every line cited here was
opened by the author.

## Answer first

The rule of ADR 0011 owner decision 2 is: no export without a concrete benefit that the public API cannot give. Applied to Typed alone,
**none of the three mechanisms needs a new `pekko-actor` export.**
- **Clock and message-adapter ref.** They can be built in `pekko-actor-typed` on the existing public API.
- **Event stream.** It can be built in `pekko-actor-typed` too. The reasons to put it in `pekko-actor` come from Classic producers and
  from the shared dead-letter family. Typed alone does not justify it, so where it lives stays Owner decision 5.

| Mechanism | New export needed by Typed? | Why | What an export would add, and for whom |
|---|---|---|---|
| Clock (backoff delays, reset window, Typed timers) | **No** | The Typed layer arms its own timers with `time.AfterFunc` and re-enters the cell through `Port.Post` (`G/port.go:25-62`). A clock interface local to `pekko-actor-typed` makes them deterministic in tests | One clock for the whole System: Ask deadlines (`G/ask.go:58, 166-176`), ManualTime (#1149) and the judge's controlled mode. That is #1129's own justification, not Typed's |
| Event stream (`UnhandledMessage`, `Dropped`) | **No**, for Typed alone | A bus owned by `pekko-actor-typed` per System can carry the Typed events, and can bridge `SubscribeDeadLetters` (`G/deadletter.go:36-38`) so one subscription sees the whole family | One stream shared with Classic producers (#897 `UnhandledMessage`, #906 `Dropped`) and with the runtime's `DeadLetter`, as in Pekko, where all three extend `AllDeadLetters` (`A/ActorRef.scala:579, 592, 622`; `A/Actor.scala:297-303`). This is **Owner decision 5**, still pending |
| Message-adapter ref | **No** | `actor.Ref[T]` has only exported methods (`ID()`, `Tell`; `G/ref.go:18-21`; `G/id.go:9-12`), so the Typed module can implement it, and the cell's `Port` reaches the layer through `PortHandler.AttachPort` (`G/port.go:43-62`) | Watching or asking an adapter ref. No Pekko spec found that does either (section 4); it is recorded as an observable difference, not as a need |

## 1. Clock

**What Typed needs.** These are needed for #986 (backoff) and, later, for the Typed timers stories:
- delays for backoff (`T/internal/Supervision.scala:185-196, 377-383`);
- the reset window (`Supervision.scala:241-246, 397-399`);
- the restart-limit window (`Supervision.scala:430-440`);
- timer cancellation on restart (`Supervision.scala:298, 334`).

**What exists.**
- The System clock (`G/clock.go:8-26`) is unexported, and only Ask reads it (`G/ask.go:58`; armed at `G/ask.go:166-176`).
- Nothing else in the runtime uses it.
- `Port.Post` puts work back into the cell in mailbox order and is rejected after a stop (`G/port.go:25-41`).

**What the Typed layer can build without an export.** A module-local clock interface with the same three operations (`Now`,
`AfterFunc`, `Stop`):
- in production it defaults to the wall clock;
- tests and a Typed testkit can inject one, and the injection point is a Typed System or spawn option.

When a timer fires, it posts a module-internal message through the cell's `Port`. The layer drops that message if its generation no
longer matches. This gives deterministic backoff tests for #986.

**What remains without the export.** Typed timers on a module clock and Ask deadlines on the System clock are **two time sources**:
- Advancing a manual Typed clock does not expire an `actor.Ask`.
- Pekko's ManualTime replaces the whole System scheduler (`upstream/pekko/actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ManualTime.scala:34-37`),
  and the Typed ask pattern takes that scheduler (`T/scaladsl/AskPattern.scala:43, 84`).

So the export of #1129 is justified when a story needs **one** time source for Ask and timers, that is ManualTime parity (#1149) and the
judge. #986 does not need it.

**Recommendation.**
- #986 uses a module-local clock and does not depend on #1129.
- #1129 keeps its own scope: the Ask deadline on a caller-supplied clock. Its justification is recorded in section 6 for the ManualTime
  and judge consumers.
- If the Owner prefers one clock from the start, #986 consumes #1129 instead. That changes only where the clock comes from, not the
  backoff contract.

## 2. Event stream

**What Typed needs.**
- #925 publishes `UnhandledMessage` (ADR 0034 P7, B13; `TT/scaladsl/ActorContextDelegateSpec.scala:82`).
- #999 publishes `Dropped` when the restart stash overflows (ADR 0035 contract 5; `Supervision.scala:120-124`; `TT/SupervisionSpec.scala:882`).
- The testkit probes for both are #1148.

**What exists.** Dead letters only:
- `DeadLetter{Msg, To}` and `SubscribeDeadLetters` (`G/deadletter.go:22-38`);
- no generic event stream, no `UnhandledMessage`, no `Dropped`.

**Pekko facts.**
- `DeadLetter`, `Dropped` and `UnhandledMessage` all extend `AllDeadLetters` (`A/ActorRef.scala:579, 592, 622`; `A/Actor.scala:297-303`)
  and are published on one event stream.
- `Dropped(message, reason, recipient)` has no sender (`A/ActorRef.scala:627-630`).
- `UnhandledMessage` carries the sender (`A/Actor.scala:297-299`).

**Alternatives.**

| Alternative | What it needs | Consequence |
|---|---|---|
| (a) The event stream lives in `pekko-actor` (plan E-01a; stories #836, #870, #906, #897) and Typed publishes into it | exported event types and a publish/subscribe API, justified by their Classic stories | one stream and one subscription for the whole dead-letter family, as in Pekko; Typed publication waits for those stories |
| (b) A bus owned by `pekko-actor-typed` per System, bridging `SubscribeDeadLetters` | no `pekko-actor` export | Typed can publish sooner; a later Classic stream means two streams to reconcile, or a migration of the Typed bus |
| (c) No publication: the module-local hook of ADR 0034 P7 only | nothing | #925 and #999 publication rows stay unmet |

**Recommendation for decision 5:** (a). Typed adds no reason of its own for the export. The reasons are that Classic needs the same
events (#897, #906) and that Pekko delivers the family on one stream. Until (a) exists, Typed records the events through the module-local
hook (ADR 0034 P7) and claims no publication.

**Refined question for decision 5** (it replaces the wording in the readiness document): "Is the event stream that carries
`UnhandledMessage` and `Dropped` part of `pekko-actor`, introduced and justified by its Classic stories (#870, #906, #897), with Typed
publishing into it once it exists? Or does `pekko-actor-typed` own a per-System bus that bridges dead letters?"

## 3. Message-adapter ref

**What Typed needs.** These are needed for #888 (register an adapter) and #975 (its failure is supervised):
- a ref of type `U` that other actors can `Tell`;
- a turn of the owning actor that runs the adapter function;
- dead letters after the actor stops.

**Pekko mechanism.** One adapter ref per actor (`T/internal/ActorContextImpl.scala:313-330`):
- Registering a class replaces the earlier adapter for the same class.
- On delivery the adapters are tried latest-first by class assignability; no match means unhandled (`T/internal/adapter/ActorAdapter.scala:182-195`).
- The ref is a `FunctionRef` that enqueues `AdaptMessage` to the actor itself (`T/internal/adapter/ActorContextAdapter.scala:135-140`).
- After termination, a message to the adapter ref is a dead letter carrying the **original** message (`TT/scaladsl/MessageAdapterSpec.scala:314-333`, SPEC).

**Design without an export.** The interpreter already implements `PortHandler` (ADR 0034 P1), so it receives the cell's `Port` in
`AttachPort` (`G/port.go:43-62`). The adapter ref is a type of `pekko-actor-typed` that implements `actor.Ref[U]`:
- **`Tell`** posts the **raw** `U` value to the `Port`. `ReceivePort` then tries the registered adapters latest-first with a Go type
  assertion; no match is an unhandled message (ADR 0034 P7).
- **After the stop**, `Post` is rejected and publishes one `DeadLetter` whose `Msg` is the original value (`G/port.go:26-31`), the
  observable of `MessageAdapterSpec.scala:314`.
- **The function runs inside the actor's turn.** `ReceivePort` never overlaps `Receive`, so #888's first criterion holds.
- **The `Port` belongs to the cell** and is rejected after the stop, so a stale ref never reaches another cell (#888's third criterion).
- **A failing function** becomes `MessageAdaptionFailure` inside the layer (ADR 0035 contract 9, #975).
- **`ID()`** returns an ID minted by the Typed module with its own prefix. `actor.ID` is a string type (`G/id.go:9`), and the prefix
  keeps it distinct from runtime actor IDs (`actor-<n>`, `G/id.go:18`).

**Observable differences (proposed, for #888):**

| # | Difference | Evidence |
|---|---|---|
| F1 | `Watch`, `WatchWith` or `Ask` with an adapter ref as target fails: the watcher fails with `ErrInvalidHandle`, Ask returns it. Pekko's `FunctionRef` can be watched (`A/ActorRef.scala:886, 903-905`) | `G/watch.go:121, 132-136`; no Pekko spec found that watches or asks an adapter ref (searched `TT/scaladsl/MessageAdapterSpec.scala` and every `actor-typed-tests` file that uses `messageAdapter`) |
| F2 | The `DeadLetter` after the stop names the actor's own ID as `To`; Pekko names the adapter ref as recipient | `G/port.go:26-31`; `MessageAdapterSpec.scala:314-333` asserts the message, not the recipient |

Exporting a "native adapter ref" from `pekko-actor` would remove F1 and F2. That is a concrete benefit only if a story needs to watch or
ask an adapter, and no such story was found. It is not proposed.

## 4. What each contract enables in the four stories

"Enabled" means the part no longer waits for this mechanism. Every row still waits for the blockers in the last column.

| Story | Part enabled | By | Remaining blockers |
|---|---|---|---|
| #986 TYP-11 backoff | Delay formula, reset after `ResetBackoffAfter`, owner-only signals, `WithMaxRestarts` failure, deterministic tests (criteria 1 to 3 of block 3) | module-local clock and `Port` timers (section 1) | Owner decisions 1 and 2 (ADR 0034, 0035); #1096, #847, #954, #956, #976, #997, #999 |
| #986 | One time source shared with Ask (ManualTime parity) | #1129 export | #1129 and #1149; not required by any #986 criterion |
| #999 TYP-26 stash bound | Capacity, replay order, leftovers, and recording the overflow through the module-local hook (block 3 criteria 1, 2 and the recording part of 3) | nothing from this block | decisions 1 and 2; #1096, #847, #954, #956, #997 |
| #999 | **Publishing** `Dropped` on a stream | the event stream (section 2) | decision 5; then #870/#906 (option a) or the Typed bus (option b) |
| #925 TYP-03 unhandled | `Unhandled` keeps the behavior; one record through the module-local hook; signals dropped (block 3 criteria) | nothing from this block | decision 1; #1096, #847 |
| #925 | **Publishing** `UnhandledMessage` | the event stream | decision 5; #897 (option a), which itself depends on #1036 and #870 |
| #975 TYP-06 adapter failure | The adapter path: `Tell` to the adapter ref, the transform in the turn, `MessageAdaptionFailure` on failure, no partial delivery | the adapter ref of section 3, delivered by #888 | decisions 1 and 2; #1096, #847, #956, #888, #976, #925 |
| #975 | Observing an adapter ref with `Watch` or `Ask` | not provided (F1) | no story needs it |

## 5. Dependency corrections proposed for the producer stories

These are proposals, written in English, for the Owner to apply. No issue was changed.

- **#888 TYP-05.** Replace the row "#1020 → #888 `depends_on` | export justification for consumed pekko-actor APIs" with "#1020 → #888 `independent` | the adapter ref uses only public `pekko-actor` API (`actor.Ref`, `PortHandler`); no export is consumed". Add to its criteria: "The adapter ref is one per actor; registering a type replaces the earlier adapter for that type; a message with no matching adapter is unhandled; after the actor stops, a message to the adapter ref is one `DeadLetter` carrying the original message (`MessageAdapterSpec.scala:314`). Watching or asking an adapter ref is not supported (difference F1)."
- **#986 TYP-11.** In addition to block 3, record "#1129 → #986: not required; the backoff clock is local to `pekko-actor-typed`; #986 may consume #1129 later without changing its criteria".
- **#925 TYP-03 and #999 TYP-26.** Split the publication criterion into its own checkbox. It is blocked by decision 5 and its producer; the other criteria do not wait for it.
- **#1129 TIM-11.** No change to its scope. Its export justification names Ask (#1129 itself), ManualTime (#1149) and the judge's controlled mode as consumers, not the Typed backoff.

## 6. Justification records, in the proposed form of decision 6

The form is a proposal (readiness document, decision 6): five fixed lines in the story that introduces the export, indexed by #1020. Only
one mechanism of this block leads to an export, and that export belongs to an existing Classic or runtime story, not to Typed.

| Field | #1129 System clock seam |
|---|---|
| Symbol | an exported clock and timer contract (`Now`, `AfterFunc`, `Stop`) and a seam to supply it at System creation (names to be chosen by #1129) |
| Consumer | Ask deadlines (#1129 itself); ManualTime (#1149); the judge's controlled mode; optionally the Typed backoff and timers |
| Why the public API is not enough | the clock is unexported (`G/clock.go:8-26`) and reachable only through the unexported `options.clock`; Ask reads `sys.clock` (`G/ask.go:58`) |
| What it does not expose | cells, the registry, the executor, the restart-window `nanotime` (#1129 out of scope) |
| Test that pins it | #1129 AC-R1-1 and the deadline-on-the-supplied-clock criterion |

The event stream, if decision 5 chooses (a), gets its record in #870/#906 (the stories that introduce it), not here. The message-adapter
ref needs no record: it exports nothing.

## 7. Corrections applied to earlier documents of this audit

- **Readiness document (block 2), section 3:**
  - "Manual time" row: the clock export is not needed by Typed backoff or timers, only by one-System-clock consumers.
  - "Message adapter" row: no export; differences F1 and F2.
  - `UnhandledMessage`/`Dropped` row: Typed alone does not justify the export.
  - Decision 5 takes the refined question of section 2.
- **ADR 0035, "Capabilities" table:** manual time for contract 4 can be a module-local clock.

Both documents carry a revision-log entry. ADR 0035 stays **proposed**.

## 8. Not verified, and limits

- No code was written and no test was run, Pekko or Go.
- The design of sections 1 and 3 is a reading of the public API at `13fb995`: `G/ref.go`, `G/id.go`, `G/port.go`, `G/watch.go:121-136`,
  `G/clock.go`, `G/ask.go:58, 166-176`, `G/deadletter.go:22-38`. No prototype was built.
- Stories #836, #871, #1149 and #1148 are named from story bodies and titles; their bodies were not re-read.
- The absence of a Pekko spec that watches or asks a message-adapter ref is a search result over `actor-typed-tests`. It is not proof
  that no oracle applies (ADR 0033 D13).
