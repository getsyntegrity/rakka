# ADR 0038: The `EventStream` of `pekko-actor`: subscription semantics, and the fate of the existing channels

- Status: **Accepted** by the Owner on 2026-11-06 (literal texts in the Owner table: O-2 to O-7 as recommended, and O-1 "reproducirlo" after the trait scenario EV1t). Issue #793, `EVT-01`. No code changes. **Clarified on 2026-11-06 (a)**: a `Ref[M]` subscribes to every channel `E` assignable to `M`, see "Amendment (2026-11-06, a)" at the end. **Amended on 2026-11-07 (b)**: the cycle of a `DeadLetter` that cannot reach a `DeadLetter` subscriber, in two halves: a stopped `FunctionRef` subscriber (Pekko 1.7.1 already does not republish, D14) and a bounded mailbox that is full (oracle EV11; Pekko recurses: fatal path through #1345 as the destination, provisional non-republication as a known gap), see "Amendment (2026-11-07, b)" at the end. **Amended on 2026-11-10 (c)**: which story gives `DeadLetter` its sender (D7), see "Amendment (2026-11-10, c)" at the end.
- Date: 2026-11-06
- Stage: 1
- Principle applied: ADR 0033 D14 (behave exactly as Pekko), D14.3 ("every deviation needs its own ADR": this ADR is the ADR of Deviations 1 and 2 and, only if the Owner chooses alternative B of O-2, of 3; each has its own marked section below).
- Related: ADR 0036 (EVT-1, point 2: the stream is the one in `pekko-actor`; each export is justified in the story that introduces it), ADR 0034 (X4, X6), ADR 0033 (D3, D11), ADR 0037 (the form of an adaptation), ADR 0032 (sender), ADR 0011 (exports).
- Evidence: the oracle [`eventstream-spec-2026-11-06.md`](../parity/oracles/eventstream-spec-2026-11-06.md) (Pekko 1.7.1, scenarios `EV1` to `EV10`, three runs each). "S1" to "S10" are its section 1; `[nnn]` are lines of its logs.
- Owner decisions already taken that bind this ADR (2026-11-06): the ADR decides what happens to `SubscribeDeadLetters`, the dead-letter hub and `eventSink` against the stream, and whether the synchronous callbacks stay beside the actor subscriptions (each deviation justified in a marked section, D14.3); the justification of each export is a record inside the story that introduces it (O-2, option b, [#1021](https://github.com/getsyntegrity/gopekko/issues/1021)).

## Context

`pekko-actor` has no `EventStream`. It has three separate channels:

1. `System.SubscribeDeadLetters(fn func(DeadLetter)) (cancel func())` (`pekko-actor/actor/deadletter.go:35-42`), over `deadletter.Hub` (`pekko-actor/internal/deadletter/hub.go`). The callback runs **synchronously on the publishing goroutine**, in registration order, possibly concurrently, and must not block or panic. `DeadLetter` is `{Msg any; To ID}`: it has no sender. `Hub.Barrier` lets a test wait for publications that started before it.
2. `eventSink` (`pekko-actor/actor/system.go:17-25`): unexported, string-kinded (`"restart"`, `"receive-failed"`, `"poststop-panic"`, ...), replaced only by tests.
3. The logger (`kit-logger`, ADR 0033 D11).

Its users: the callbacks of (1) are used by tests only: 37 test files in `pekko-actor/actor`, 2 in `pekko-persistence/persistent` and 3 in `pekko-actor-typed/typed`, 42 in all (the only non-test file that mentions it is its own definition, `deadletter.go`; counted with `git grep` on `origin/develop` at `c728b6ae`). Pekko has one bus: `EventStream` (S1), whose subscribers are `ActorRef`s and whose diagnostic events (`Error`, `Warning`, ...) travel on it too.

Five stories wait for this contract: #836 (registration), #870 (publication), #905 (explicit unsubscribe), #1056 (removal on termination), #871 and #906 (event types). Their ACs say "follow EVT-01".

## Decision

Each point is the Pekko behavior unless it says "Deviation". Go signatures are the stories' business; this ADR fixes the observable contract.

**D1. Subscribers are actor references; delivery is a mailbox enqueue from the publishing goroutine** (S2, EV6). A subscriber is a `Ref[E]` where `E` is the channel type *(clarified by Amendment (2026-11-06, a): a `Ref[M]` subscribes to every channel `E` assignable to `M`)* (the `Subscribe[E](subscriber: ActorRef[E])` shape of Pekko Typed, `actor-typed/src/main/scala/org/apache/pekko/actor/typed/eventstream/EventStream.scala:56`: a ref that cannot accept `E` cannot subscribe; a ref of another message type subscribes through an adapter, which is the Typed layer's story, [#959](https://github.com/getsyntegrity/gopekko/issues/959)). Mailbox admission, bounded overflow and stop apply as for any other send. Observed: the publisher returns while the subscriber is blocked, and all 1001 events arrive later in order (EV6 [003], [006]); a bounded mailbox that is full gives `DeadLetter(event, deadLetters, subscriber)`, not a `Dropped`, and the publisher is not told (EV8 [006], [008]).

**D2. Matching.** The channel of an event is its dynamic Go type; the channel of a subscription is `E`. A subscription receives an event if the event's type is `E`, or `E` is an interface that the event's type implements, or `E` is `any` (Pekko: the class, a trait, `AnyRef`; EV3 [002]-[005], [024], [026]). A pointer type and its value type are different types. An event that matches no subscription is delivered to nobody and is **not** a dead letter (EV3 [033]-[040]).
*Deviation 1 (D14.3): "superclass".* Pekko also matches a subscription to a superclass. Go has no subtyping between concrete types (embedding is composition, not inheritance), so there is no counterpart to reproduce. The equivalent that Go has is an interface. Decision: not supported; a user who wants "all events of a family" defines an interface. This is the only place where D2 differs from S1, and the difference is not observable by a Go program that has no superclass relation to subscribe through.

**D3. Registration is idempotent; one copy per subscriber; the unsubscribe rule of Pekko is reproduced** (S4, S5, EV1, EV1t, EV4).
- Subscribing the same (subscriber, channel) twice delivers once; the second call reports "no change" (`true` then `false`, EV1 [001], [002], [003]). A registration already covered by another (`AnyRef` then `Base`) also reports "no change" (EV4 [007]).
- Under any number of matching registrations, one publication addresses each subscriber **once** (EV4 [008]-[010]). This is the answer to #870 AC1.
- `Unsubscribe(subscriber, channel)` reports whether it changed anything; `Unsubscribe(subscriber)` removes every registration and reports nothing (EV1 [008], [009], [012]).
- Unsubscribing a classifier keeps the registrations of channels that are **neither it nor one of its sub-channels** (see the rule below). EV4 [015]-[017]: `x` is registered on `Base`, `Iface` and `Sub`, unsubscribes from `Base`, and `Sub(d2)` still reaches it, through its registration on `Iface`, which is not a sub-channel of `Base` (inferred reading: the oracle does not isolate which registration delivered).
- **The unsubscribe rule of Pekko, reproduced (O-1, decided by the Owner on 2026-11-06 after EV1t).** Observed with classes and with traits (Go interfaces are the counterpart of traits), for `I`, `J extends I` and a type `C` that implements `J`: (v1) `a` registered **only on `I`**: `unsubscribe(a, J)` returns `true` and `C` events are no longer delivered to `a` (EV1t [005], and no `a recv` between the markers, only the control [002]); a later `unsubscribe(a, I)` returns `true` (EV1t [008]). (v2) `a` registered only on `J`: `unsubscribe(a, I)` (the broader channel) returns `true` and delivery stops (EV1tv2 [005]); then `unsubscribe(a, J)` returns `false` (EV1tv2 [008]). (v3) `a` registered on **both** `I` and `J`: `unsubscribe(a, J)` returns `true` and `C` is **not** delivered via `I` either (EV1tv3 [006]; control [003]); `unsubscribe(a, I)` then returns `true` (EV1tv3 [009]). With classes the same: EV1 [015]-[018]. *Inferred from the source, not a log line* (`SubclassifiedIndex.scala:157-163, 174-176, 222-228`, `EventBus.scala:165-171`): the index keeps one set of subscribers per channel and does not record through which classifier a subscriber entered it, so `Unsubscribe(s, X)` removes `s` from `X` **and from every sub-channel of `X`**, whichever classifier it registered through, and reports `true` if it removed `s` from at least one channel. Not studied: whether events that match `I` but not `J` are still delivered after `unsubscribe(a, J)` in v3, and diamond hierarchies (oracle Limits). The stories that implement unsubscribe ([#905](https://github.com/getsyntegrity/gopekko/issues/905), [#836](https://github.com/getsyntegrity/gopekko/issues/836)) reproduce the three variants as tests, and the oracle is extended first if they need a case it did not run (D13).

**D4. Invalid input is rejected before anything else** (S3, EV2). A nil subscriber is rejected by `Subscribe`, `Unsubscribe(s, c)` and `Unsubscribe(s)` (Pekko: `IllegalArgumentException("subscriber is null")`, EV2 [001]-[003]). A nil event passed to `Publish` is rejected (Pekko: `NullPointerException`, EV3 [042], inferred from `null.getClass`, `E:49`). In Go the rejection is an error that names the cause, never a silent accept and never a panic inside the stream. There is no "nil channel": the channel is a type parameter. (Pekko accepts a null channel and then breaks every later `publish` of that stream, EV2 [006]-[008]; that is a defect of the JVM API that Go's typing removes, not a behavior to copy.)

**D5. Unsubscribe linearization** (S6, EV5, EV10).
- Deliveries already **accepted into the mailbox** before `Unsubscribe` returned complete; the actor processes them after the call returns (EV5 [008]-[010]).
- A publication that **starts after `Unsubscribe` returned** does not address the subscriber (EV5 [006] and no `recv Base(e5)`; EV10 `delivered=3031 maxId=3030` [004] and its two runs).
- A publication that was already running when `Unsubscribe` was called **may** still address the subscriber (S6: it reads a snapshot of the registrations). The contract promises the first two lines only; no barrier is part of the public API.
- This is the answer to #905 AC3 and to #793 AC2 ("whether an already accepted delivery may complete").

**D6. Termination removes the subscriber, asynchronously** (S7, EV7). When a subscribed actor stops, the stream removes all its registrations. The removal is not instantaneous: in the window a publication to it becomes `DeadLetter(event, deadLetters, subscriber)` (EV7 [049], [354]), and after the removal nothing more is produced (EV7 [045], [351]). Subscribing an already stopped actor reports success and removes it shortly after (EV7 [353]; "removes" is inferred from the dead-letter count of [396], the second `subscribe` of [397] returned success again). The size of the window is timing (0 or 1 dead letters were seen); the contract promises no number. Implementation: #1056.

**D7. The stream carries any event; the event types are other stories' business.** `DeadLetter` gains its sender in #896 under ADR 0032 D3/D4/D6 *(amended on 2026-11-10, Amendment (c): it said #871/#906)*; `SuppressedDeadLetter`, `Dropped` (ADR 0036, including Amendment (e)) and `UnhandledMessage` (ADR 0034 X6) are published through this stream when their stories land. Pekko publishes a `Dropped` from `MessageBuffer.drop` as a `DeadLetter` that wraps it (EV9 [003], [004]; [011] is the `NullPointerException` of `drop` of an unknown id); that is ADR 0036 (e), now backed by an observation.

**D8. The existing channels.** Recommended decision, with the alternative's D14.3 section ready (see O-2):

| Channel | Decision | Why |
|---|---|---|
| `SubscribeDeadLetters` and the synchronous `Hub` | **Replace** (alternative A of O-2) | Pekko has no synchronous callback subscription. They are the opposite of D1 and EV6: a blocked callback blocks the publisher, and a callback can run after `cancel` returned (`hub.go:50-70`, oracle section 4). Tests that need determinism subscribe a probe `Ref` and wait on a channel, as Pekko's own tests use `TestProbe`. |
| `eventSink` (unexported, string kinds) | **Stays an internal seam**; not the public stream | Pekko's diagnostic events are `LogEvent`s on the stream (`classic-poststop-throws-2026-11-06.md`, scenario PS1 [003]: `EVENT Error cause=...`). goPekko routes logs through `kit-logger` (ADR 0033 D11), so `LogEvent` on the stream is **Deviation 2 (D14.3): the log channel is the logger, not the stream.** The seam is not extended and is not a second public bus. Whether the string kinds become typed events on the stream is not decided here. |
| `DeadLetter` publication (`ref.go`, `stop.go`, `port.go`, `ask_endpoint.go`) | **Unchanged call sites**: they publish to the stream | One publication, subscribers by D1 to D6. |

Order of work (not an obligation of this ADR): the stream and the dead-letter publication land first (#836, #870); `SubscribeDeadLetters` is removed only in the unit that migrates its 42 test files to probe subscribers, after the stream can carry `DeadLetter`. Until then the two coexist **inside `pekko-actor` only**, as an implementation state and not as a contract.

*If the Owner chooses alternative B of O-2 (keep the synchronous callbacks beside the actor subscriptions), this is the D14.3 section it needs:* **Deviation 3: synchronous dead-letter callbacks.** No Pekko counterpart. Kept because (reason to be given by the Owner; the TL's candidate: deterministic tests without a probe actor). Its contract would be: the callback runs on the publishing goroutine after the actor subscribers have been addressed; it must not block (a blocked callback blocks the publisher, unlike D1); it observes a publication that has begun by the time `cancel` returned only if the callback was running when `cancel` was called (the `Hub` behavior); it receives `DeadLetter` only, not the other event types. Every difference from EV5 and EV6 would be a documented deviation.

**D9. Exports.** The stream, its event types and its entry points (`Subscribe`, `Unsubscribe`, `Publish`) are exported from `pekko-actor` because Typed (#959) and the testkit need them (ADR 0036 EVT-1, point 2; ADR 0033 D3). Per O-2 (b), the introducing story (#836 for registration, #870 for publication) records the ADR 0011 justification in its "Open API choices" section and links it in the index #1020. This ADR approves no export.

## Consequences

- #793 is satisfied by this ADR once accepted (it is a decision story). #836, #870, #905, #1056 take their criteria from D1 to D6; each keeps its own oracle citations in the oracle.
- ACs to correct in the issues after acceptance: #870 AC1 (answered by D3), #905 AC3 (D5), #836 AC2 ("supported superclass/interface relationships": superclass is Deviation 1).
- `DeadLetter` without a sender (`deadletter.go:22-25`) is not enough for D1/D7; its change belongs to #896 *(amended on 2026-11-10, Amendment (c): it said #871/#906)*.
- A Typed `EventStream` command surface (#959) sits on top of this stream; its adapter for a `Ref` of another message type is a Typed story.
- The oracle is Classic-only: the Typed `EventStream.Subscribe/Publish` adapters were not run (oracle section 5).

- **Unsubscribe tests (O-1).** #905 and #836 include the three variants of D3 (EV1t, EV1tv2, EV1tv3: return values and delivery afterwards) among their criteria, besides the classes-and-traits cases of EV1 and EV4.
- **O-2 A, in stages (Owner, 2026-11-06).** `SubscribeDeadLetters` is **deprecated from the story that publishes ([#870](https://github.com/getsyntegrity/gopekko/issues/870))** and no new use is added to it. The migration of the 42 test files to probe subscribers is its own story, [#1426](https://github.com/getsyntegrity/gopekko/issues/1426) (`EVT-19`), split into PRs of at most 400 authored lines (`pekko-actor/actor` in as many PRs as needed, `pekko-persistence`, `pekko-actor-typed`). The callback and the synchronous hub are **deleted in the last PR of that story**; until then both coexist inside `pekko-actor` as an implementation state, not as a contract.

## Decisions for the Owner

| # | Question | Recommendation | Alternative |
|---|---|---|---|
| O-1 | The unsubscribe rule of Pekko (D3; EV1 [015]-[018], EV1t, EV1tv2, EV1tv3) | **Decided: reproduce it**, with the interface counterpart, already observed (Owner, 2026-11-06) | Exact-registration only would have been a deviation; not chosen |
| O-2 | The fate of `SubscribeDeadLetters` and the synchronous hub (D8) | **A. Replace**: remove it once its tests use probe subscribers; no deviation | **B. Keep it** beside the actor subscriptions, with Deviation 3 of D8 |
| O-3 | "Superclass" matching (Deviation 1) | Not supported; interfaces are the Go counterpart | Emulate a class family by an explicit registry; no known need |
| O-4 | `LogEvent` on the stream (Deviation 2) | The logger is the log channel; the stream carries no log events | Publish log events on the stream too; a later ADR with the logging stories (plan rows E-04a to E-04c) |
| O-5 | D4 turns Pekko's thrown `IllegalArgumentException`/`NullPointerException` for a nil subscriber or event into an error return (never a panic inside the stream) | Error return that names the cause | A panic, as the nearest Go image of an exception; no known need |
| O-6 | D1: a `Ref` whose message type does not accept the channel type *(Amendment (2026-11-06, a): "is not the channel type" read "is not assignable from it")* cannot subscribe directly; it subscribes through an adapter, which is the Typed layer's story (#959) | As written | A second subscription entry point with a conversion function in `pekko-actor` |
| O-7 | D5 promises less than the oracle showed: no barrier in the public API (the hub's `Barrier` stays internal); Pekko has no counterpart | As written | Export a barrier for tests; no Pekko counterpart, so a deviation |

## Not covered

- The Go signatures, package placement and the `Handle`/`Ref` details (the stories).
- Remote and cluster subscribers, `ActorEventBus` outside `EventStream`, the `debug` flag of the stream (not exercised in the oracle).
- The group-router stash-full site of ADR 0036 (still without an oracle).

## Alternatives rejected

- A callback-only stream (every subscriber a `func(event)`): it is the current `Hub` generalized; it cannot satisfy D1 (a blocked callback blocks the publisher) and keeps the unsubscribe window the hub has.
- Subscribers as untyped `any` handles: loses the type safety that `Ref[E]` gives and moves the "wrong class" failure to run time.

## Owner table

| Date | Owner decision (literal text) | Effect on this ADR |
|---|---|---|
| 2026-11-06 | "si — apruebo ADR 0038 con O-2 A en etapas y O-3 a O-7 como están recomendados; O-1 se decide con el escenario de traits" | O-2 A (staged replacement, Consequences), O-3 (no superclass matching), O-4 (no log events on the stream), O-5 (error return), O-6 (adapter is a Typed story), O-7 (no public barrier) are decided as recommended. O-1 was left open until the trait scenario. |
| 2026-11-06 | "Sí, cerrá O-1 como reproducirlo y pasá el ADR a Accepted" | O-1 decided: reproduce the unsubscribe rule of Pekko with the interface counterpart (observed by EV1t; rule in D3). Deviation 4 and the analogy of D3 are removed. The ADR is Accepted. |

## Amendment (2026-11-06, a): a `Ref[M]` subscribes to every channel `E` assignable to `M`

- Status: **Accepted** by the Owner on 2026-11-06 (literal text in the table below). Additive: the text above is unchanged; the sentence of D1 and the O-6 row carry a "clarified" mark.
- Principle applied: ADR 0033 D14. Ground (the Owner's): in Pekko Typed `ActorRef[-T]` is contravariant (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/ActorRef.scala:35`), so an `ActorRef[Iface]` is an `ActorRef[Impl]` and subscribes directly to the channel `Impl` through `EventStream.Subscribe[E](subscriber: ActorRef[E])` (`.../eventstream/EventStream.scala:56`). This is the reading of D1 ("a ref that cannot accept `E` cannot subscribe"): D1's "`Ref[E]`" is the case `M = E`.

**Decision.**
1. The Go form is `Subscribe[E, M any](s *System, subscriber Ref[M])`: `E` is the channel (explicit), `M` is the message type of the subscriber (inferred), and `E` must be assignable to `M`. That is the Go equivalent of the contravariance of `ActorRef`. The same `Ref[M]` can therefore be registered on several channels it accepts (the case that the observations of D3, EV1t and EV4 need).
2. The check is made **at run time**: Go cannot express a constraint "assignable" between two type parameters. A violation is the error `ErrChannelNotAccepted` (D4: errors, not panics).
3. **O-6 keeps governing** a `Ref` whose message type does not accept `E` (`E` not assignable to `M`): it cannot subscribe directly and needs the adapter of [#959](https://github.com/getsyntegrity/gopekko/issues/959). This amendment adds no second entry point and no conversion function.
4. Scope of the amendment: the signature and the check. D2 (matching), D3 (idempotence, the unsubscribe rule), D5 and the other decisions are unchanged. The scope of #836 (`docs/scopes/836-eventstream-registration.md`, D-2) carries it out; this amendment merges before its slice 3a.

| Date | Owner decision (literal text) | Effect on this ADR |
|---|---|---|
| 2026-11-06 | "si — apruebo Subscribe[E, M any](s, Ref[M]) con E asignable a M, como equivalente en Go de la contravarianza de ActorRef en Pekko" | Points 1 to 4 above: the signature, the run-time check and `ErrChannelNotAccepted`; O-6 stays for a `Ref` whose message type does not accept `E`. No other decision changes. |

## Amendment (2026-11-07, b): the cycle of a `DeadLetter` that cannot reach a `DeadLetter` subscriber (two halves)

- Status: **Accepted** by the Owner on 2026-11-07 (literal text in the table below; the Owner narrowed its scope to the second half on 2026-11-07 after the QA found the context of the first). Additive: the text above is unchanged.
- Principle applied: ADR 0033 D14 and D14.3. Evidence: the oracle [`eventstream-spec-2026-11-06.md`](../parity/oracles/eventstream-spec-2026-11-06.md), scenarios EV11 to EV11v4 (PR #1445), and the scope of #870 (D-3, P9).

The cycle is a `DeadLetter` that cannot be delivered to a subscriber of `DeadLetter`: the failure produces another `DeadLetter` (a `DeadLetter` whose message is a `DeadLetter`), which the stream would publish again. It has two halves in Pekko 1.7.1.

**Half 1: a stopped subscriber that is a `FunctionRef` (a message adapter). Pekko 1.7.1 does not republish; goPekko copies it by D14.** `FunctionRef.deadLetterMessageHandler` is `if (!msg.isInstanceOf[DeadLetter]) system.deadLetters.tell(msg, sender)`, with the comment "avoid infinite loop (StackOverflow) if FunctionRef is used for subscribing to DeadLetter from eventStream" (`ActorRef.scala:862-866` at v1.7.1). This is the behavior of the baseline, so it is followed under D14, **not** as an application of D15. Context, for why it exists: [akka-core #31241](https://github.com/akka/akka-core/issues/31241) (`bug`, closed: `StackOverflowError` when terminating the application, a message adapter subscribed to `DeadLetter` looping through the event bus) and its merged fixes [#31249](https://github.com/akka/akka-core/pull/31249) and [#31265](https://github.com/akka/akka-core/pull/31265) (March 2022, before the fork, in the baseline). This half rests on code reading and on the upstream issue and fixes (code reading, not run): the oracle EV11 uses a plain actor, and no scenario with a stopped `FunctionRef` subscriber, nor with a stopped plain actor, was run on Pekko. Pekko's guard sits in the `FunctionRef` handler, while goPekko's sits at `System.publish` and also covers subscribers that are not `FunctionRef`s; for the `FunctionRef` case the observable result is the same.

**Half 2: a subscriber whose bounded mailbox is full (oracle EV11). No acknowledged bug and no fix; Pekko recurses until a `StackOverflowError`.** A `DeadLetter` event that cannot be delivered to a full subscriber is republished as `DeadLetter(DeadLetter(...))` to the same subscriber again and again until a `StackOverflowError` reaches the sender (EV11 [010]; a subscriber registered ahead of the full one saw up to 1001 nested wrappers in run 1 of EV11v4, 977 in run 2 and run 3 had a harness failure, EV11v4 [013]; the mechanism is `Mailbox.scala:560-568` plus `ActorRef.scala:739-743`).

**Search for an acknowledged bug of half 2, made on 2026-11-07 (D15 does not apply to it).** Searched with `gh search` in `apache/pekko` (issues and PRs) and in `akka/akka-core` (the former `akka/akka`), with several wordings ("StackOverflowError", "stack overflow", "DeadLetter", "dead letters", "DeadLetter loop", "infinite loop", "bounded mailbox", "DeadLetter recursion"); the first pass missed #31241, which the QA found with the bare wordings "StackOverflowError" and "stack overflow". No issue describes the bounded-mailbox cycle and none has a merged fix. The nearest results are other problems: `apache/pekko` [#3620](https://github.com/apache/pekko/issues/3620) (bug, closed, fixed by #3621 and #3627: the dispatcher's fatal-error or rejected-execution path, the one behind D-a of ADR 0035) and [#3613](https://github.com/apache/pekko/issues/3613) (bug, open: rejected execution strands a mailbox); `akka-core` [#27405](https://github.com/akka/akka-core/issues/27405) (closed, a `StackOverflowError` from an executor that rejected tasks after the user shut it down) and [#27265](https://github.com/akka/akka-core/issues/27265) (open: "Use Dropped in bounded mailboxes", no change of the cycle); `apache/pekko` PR [#3658](https://github.com/apache/pekko/pull/3658) documents the `EventStream` subscription semantics "without changing event dispatch". The code of `apache/pekko` `main` was read on 2026-11-07: `Mailbox.scala:584-592` (bounded `enqueue`) and `ActorRef.scala:708-715` (`DeadLetterActorRef.!`) are the same as at v1.7.1 (`Mailbox.scala:560-568`, `ActorRef.scala:736-743`): there is no change to these two blocks on `main`. **Limit of this check:** it is a text search of the issue trackers and a reading of two files, not an exhaustive history of either repository, and the first pass missed a relevant issue.

**Two options for half 2.**
- **(a) Fatal path, as D-a (ADR 0035 S-3).** Detect the cycle and treat it as a fatal error through the fatal-error path of [#1345](https://github.com/getsyntegrity/gopekko/issues/1345), instead of recursing until Go's unrecoverable stack overflow.
- **(b) Conservative rule.** A `DeadLetter` whose `Msg` is itself a `DeadLetter` reaches the hub callbacks but is not republished to the stream. For half 2 it is an observable deviation from Pekko, justified under D14.3: Pekko ends in a `StackOverflowError` at the sender and Go cannot recover a stack overflow.

**Decision (Owner, 2026-11-07), for half 2.** (a) is the destination and (b) is the provisional measure until #1345, recorded as a **known gap whose owner is #1345** (as c-sync of D-a). Until #1345: rule (b), implemented in PR #1448 (`System.publish`, marker `TODO(EVT-03-deadletter-cycle)`). When #1345 lands: the cycle is detected and handled by the fatal path, and rule (b) is removed. Criterion P9 is added to #1345. For half 1 rule (b) is not provisional: it is the 1.7.1 behavior; the code of #1448 applies it to both halves (it tests the type of the message, not why the delivery failed), and the part of it that is half 2 is the part that #1345 replaces.

| Date | Owner decision (literal text) | Effect on this ADR |
|---|---|---|
| 2026-11-07 | "si — P9: camino fatal como D-a mediante #1345; hasta entonces, no republicar como known gap" | Half 2: (a) as the destination through #1345, (b) as the provisional rule, known gap owned by #1345. Half 1 is Pekko 1.7.1 behavior followed under D14. No other decision of this ADR changes. |

## Amendment (2026-11-10, c): D7, the sender of `DeadLetter` belongs to #896 (editorial)

- Status: **Accepted** by the Owner on 2026-11-10 (literal text in the table below). Editorial and additive: it changes which story is named in one sentence of D7 and no behavior, event or decision.
- Principle applied: ADR 0033 D14 (the sender belongs to the story that ports `DeadLetter(message, sender, recipient)`, `ActorRef.scala:592-595`).
- Ground: D7, and the sentence of Consequences about `deadletter.go:22-25`, attributed the sender of `DeadLetter` to #871/#906. The story that gives it is #896 (`MSG-08`): ADR 0032 D6 "Keep" and the Owner's criterion of 2026-11-07 (#1481) add the sender assertion to P1 and remove `PORT-GAP(dl-sender-896)`, and the Owner ordered on 2026-11-09 that #896 comes first and #871 after it. The scope is [`docs/scopes/896-deadletters-ref-and-sender.md`](../scopes/896-deadletters-ref-and-sender.md) (approved at G2 on 2026-11-10; its D-10 asks for this amendment).

**Decision.**
1. `DeadLetter` gains its sender in #896, under ADR 0032 D3/D4/D6: the explicit sender of a `TellFrom`, or the `deadLetters` reference of the `System` when none was given, in the dead-letter view only (`ActorRef.scala:589`); `Context.Sender` keeps ADR 0032 D3.
2. #871 adds the `DeadLetterSuppression` marker and `SuppressedDeadLetter`; #906 adds `Dropped`; #897 adds `UnhandledMessage`. Each is published through this stream when its story lands, as D7 already says. None of them gives `DeadLetter` its sender.
3. D7 is otherwise unchanged. The same correction (#871/#906 to #896) is made in the sentence of Consequences about `deadletter.go:22-25`.

| Date | Owner decision (literal text) | Effect on this ADR |
|---|---|---|
| 2026-11-10 | "G2 aprobado, 2026-11-10, con D-1 a D-10 como están" (D-10 of the scope of #896: the editorial amendment of ADR 0038 D7) and "Abrí la enmienda editorial de ADR 0038 D7 (D-10) como PR propia" | Points 1 to 3 above: D7 names #896 for the sender of `DeadLetter`; nothing else changes |
