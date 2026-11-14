# Scope: Publish events through the subscribed actors' mailboxes (#870, EVT-03)

Status: **approved by the Owner at gate G2 on 2026-11-07** ("#870: apruebo D-1 a D-10. D-8 (mutación que falla por desbordamiento de pila) solo vale porque las mutaciones se corren a mano; ningún test normal puede depender de un crash."). Written on 2026-11-06 against `develop` `f95458cb` (the unit of #836 has since merged — #1438, #1441, #1442 — and the shapes cited from it are those of the merged code). #870 is `prio:P0`, `status:blocked`; its criteria are those of the issue plus the Update of 2026-11-06 (ADR 0038). It starts after the unit of [#836](836-eventstream-registration.md) is closed (slice 4 merged).
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: the unit ports `SubchannelClassification.publish` and the publish cache (`event/EventBus.scala:177-190, 206-209` and `addToCache` in `subscribe`, `:159-163`) and `EventStream.publish`/`classify` (`event/EventStream.scala:44-54`; the `sys == null` branch of `publish` (`:51-54`) is not ported: a goPekko stream always belongs to a `System`), so the Dev runs the migration kit (`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK under `migration/stage-1/eventstream-publish/`. The Dev is a separate session, QA is independent of both, and the TL does not implement.

Contract sources: [ADR 0038](../adr/0038-event-stream-subscription-semantics.md) (Accepted: D1, D2, D3, D5, D7, D8, Amendment (a)), the oracle [eventstream-spec-2026-11-06.md](../parity/oracles/eventstream-spec-2026-11-06.md) (`S1` to `S10` are its section 1; `EV3`, `EV4`, `EV6`, `EV7`, `EV8` its scenarios; `[nnn]` lines of its logs), the scope of #836 (the registration this unit publishes to), the issue #870.

## What this delivers

`Publish` on the `EventStream` of `pekko-actor`: an event is delivered, by a `Tell` into the mailbox, once to each subscriber registered for a channel the event's type matches; the publisher is never blocked by a subscriber; a full or stopped subscriber turns the event into a dead letter through the normal send path. It adds the publish cache of Pekko (`Subscribe` now feeds it), publishes the runtime's own `DeadLetter` events to the stream (the precondition of the migration story [#1426](https://github.com/getsyntegrity/gopekko/issues/1426)) and **deprecates** `System.SubscribeDeadLetters` (ADR 0038 D8, Owner's O-2 A).

## Proposed Go API (for the Owner at G2)

```go
// Publish delivers event to every subscriber registered for a channel that event's dynamic type matches
// (ADR 0038 D2), once each (D3), by Tell into its mailbox (D1). It does not block on a subscriber.
// An event matching no subscription is delivered to nobody and is not a dead letter (EV3 [033]-[040]).
func Publish(s *System, event any) error

var ErrNilEvent = errors.New("event stream: nil event") // ADR 0038 D4
```

`SubscribeDeadLetters` keeps its signature and gets a `Deprecated:` paragraph pointing at `Subscribe[DeadLetter, ...]` (ADR 0038 D8).

## Evidence: oracle, and what goPekko does today

| Case | Pekko 1.7.1 (oracle) | goPekko `develop` `f95458cb` (reading) |
|---|---|---|
| A `Sub` event reaches subscribers of `Sub`, `Base`, `Iface`, `AnyRef` | EV3 [002]-[005]; the order among subscribers varies between runs | `Subscribe` registers; nothing publishes |
| Duplicate class matches | one copy per subscriber (EV4 [008]-[010]) | the index keeps a set of subscriber IDs (#836 X12) |
| An unmatched event | delivered to nobody, no `DeadLetter` (EV3 [033]-[040]) | — |
| `publish(null)` | `NullPointerException` (EV3 [042]; Inferred from `null.getClass`, `E:49`) | — |
| A blocked subscriber | the publisher returns; all 1001 events arrive in publish order (EV6 [003], [006]) | `Tell` never blocks (`ref.go:65-93`) |
| A full bounded mailbox | `DeadLetter(event, deadLetters, subscriber)`, no `Dropped`, no error to the publisher (EV8 [006], [008]) | `Tell` publishes `DeadLetter{Msg, To: cid}` (`ref.go:78-83`); there is no sender field (#871/#906) |
| A stopped but still registered subscriber | the same `DeadLetter` in the window before its removal (EV7 [049], [354]) | `Tell` to a stopped cell publishes `DeadLetter` (`ref.go:72-76`); the removal itself is #1056 |
| `DeadLetter` as an event on the stream | published by `deadLetters` for every dead letter (`ActorRef.scala:739-743`) | `System.publish` feeds only the `Hub` callbacks (`deadletter.go:29`) |

## Criteria, scenarios, sources and mutations

Tests: go-specs, channels or gates, no sleeps, no `-race`. A mutation is applied to production code in a disposable worktree and never committed; the named test must fail by assertion.

| # | Criterion | Scenario (test) | Source / oracle | Mutation | Slice |
|---|---|---|---|---|---|
| P1 | An event reaches each matching subscriber once: subscribers of the exact type, of an interface the type implements, and of `any`; under several matching registrations of one subscriber, one copy | `TestPublishMatchingSubscribersOnce` (also asserts that `Publish` to a native subscriber never panics, D-10) | EV3 [002]-[005], [024], [026]; EV4 [008]-[010]; `EventBus.scala:177-190` | deliver once per registration instead of once per subscriber; and the delivery closure built by `Subscribe` (`e.(M)` then `Tell`, merged in #1438 and covered only by an internal test) is the one `Publish` calls | 1a |
| P2 | An event that matches no subscription is delivered to nobody and is not a dead letter | `TestPublishUnmatchedIsNotADeadLetter` | EV3 [033]-[040] | publish an unmatched event as a `DeadLetter` | 1a |
| P3 | `Publish(nil)` is `ErrNilEvent` before anything else | `TestPublishRejectsNilEvent` | EV3 [042] (Inferred) ; ADR 0038 D4 | remove the nil check | 1a |
| P4 | The publish cache: the first publication of a type adds the type to the index (`AddKey`) under the lock; later ones read a snapshot without the lock; `Subscribe` updates the cache with its diff, so a subscription made after the first publication is addressed by the next one | `TestPublishSeesSubscriptionsMadeAfterTheFirstPublication` | `EventBus.scala:162, 177-190, 206-209` | `Subscribe` does not update the cache | 1a |
| P5 | The publisher is not blocked by a blocked subscriber and the events arrive in publish order | `TestPublishDoesNotBlockOnASlowSubscriber` | EV6 [003], [006] | `Publish` waits for the subscriber (synchronous send) | 1b |
| P6 | A subscriber whose bounded mailbox is full: the event becomes `DeadLetter{Msg: event, To: subscriber}` through `Tell`, there is no `Dropped`, `Publish` reports no error; a stopped subscriber that is still registered gets the same. The test asserts the FIRST dead letter only: until #1056 removes a stopped subscriber it stays registered, so later publications keep producing dead letters (EV7 [351] counts after the removal) | `TestPublishToAFullOrStoppedSubscriberIsADeadLetter` | EV8 [006], [008]; EV7 [049], [354]; `ref.go:72-83` | drop the event instead of publishing the dead letter | 1b |
| P7 | A concurrent `Publish` and `Subscribe` never lose a registration made before the publication started (64 goroutines released by a gate, `-count=100`; the mutation is probabilistic by nature) | `TestPublishConcurrentWithSubscribe` | `EventBus.scala:146-147, 177-190` (S6) | remove the lock around the cache update | 1b |
| P8 | Every `DeadLetter` the runtime publishes (`System.publish`, the eight call sites of the survey) is also published as an event to the stream; the `Hub` callbacks still run (until #1426) | `TestDeadLettersAreEventsOnTheStream` | `ActorRef.scala:739-743` (`deadLetters` publishes every dead letter); oracle section 4 | `System.publish` feeds only the hub | 2 |
| P9 | A `DeadLetter` event that fails to reach a stream subscriber (full or stopped) does **not** loop: a `DeadLetter` whose `Msg` is itself a `DeadLetter` (what the failed delivery produces in Go) reaches the hub callbacks and is not published to the stream again; the subscriber with a bounded mailbox of capacity 1 blocked in `Receive` receives its first dead letter and the stream publishes a bounded, asserted number of `DeadLetter` events. Marker `TODO(EVT-03-deadletter-cycle)` | `TestDeadLetterDeliveryFailureDoesNotLoop` | Pekko is predicted to recurse (see D-3); EV11 observes it | remove the guard: `Tell` → `System.publish` → stream → `Tell` recurses on one stack until Go's **fatal** stack overflow, which a test cannot recover: the mutation fails by the test binary crashing (`goroutine stack exceeds ... limit`), not by an assertion (decision D-8) | 2 |
| P10 | `SubscribeDeadLetters` carries a `Deprecated:` paragraph; no non-test file uses it (`git grep`) | review | ADR 0038 D8, Consequences | none (review) | 2 |
| P11 | Exports are justified in "Open API choices" and the README says what exists and what is planned | review | module README contract | none (documentation) | 3 |

Every sentence of the PRs that says something was observed names the oracle line or the assertion.

## Slices (one PR each, at most 400 authored lines, sequential)

Estimates are the TL's raw numbers times 1.55 (the deviation measured in #976); the TL wrote no prototype. A slice above 400 is split by the Dev before opening the PR.

| Slice | Delivers | Files | Criteria | Est. lines (raw → re-estimated) |
|---|---|---|---|---|
| **0. Kit package** | manifest, RULEBOOK, RUNBOOK | `migration/stage-1/eventstream-publish/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | kit run | ~120 docs |
| **0b. Oracle addendum EV11** | the scenario EV11 on Pekko 1.7.1: a subscriber of `DeadLetter` whose bounded mailbox (capacity 1, push timeout 0) is blocked and full; N dead letters are produced; what the stream and the subscriber see (a loop? one `DeadLetter` per overflow? nothing?); harness, trimmed logs, oracle section, as EV1t and EV4f did | `docs/parity/harness/...`, `docs/parity/oracles/eventstream-spec-2026-11-06.md`, `.../eventstream-spec-2026-11-06.logs/` | P9 | ~100 docs and logs |
| **1a. `Publish` and the cache** | `Publish`, `ErrNilEvent`, the cache and its update in `Subscribe` | `pekko-actor/actor/eventstream.go`, `eventstream_publish_test.go` | P1 to P4 | ~80 + ~140 → ~340 |
| **1b. Delivery behavior** | tests only | `pekko-actor/actor/eventstream_publish_delivery_test.go` | P5 to P7 | ~140 → ~220 |
| **2. `DeadLetter` events on the stream, deprecation** | `System.publish` also publishes the event; the cycle rule; the `Deprecated:` paragraph | `pekko-actor/actor/deadletter.go`, `eventstream_deadletter_test.go` | P8 to P10 | ~40 + ~120 → ~250 |
| **3. Docs and record** | README of `pekko-actor`, `EXECUTION-RECORD.md`, cost-log row, the record linked from #1020 | `pekko-actor/README.md`, `migration/stage-1/eventstream-publish/EXECUTION-RECORD.md`, `migration/cost-log.tsv` | P11 | ~120 docs |

Order: 0, 0b, 1a, 1b, 2, 3. `eventstream.go` is edited by 1a only; `deadletter.go` by 2 only. Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md` or another module changes; a needed change outside the table is a Mandatory Stop and goes back to the TL.

| Pair | Shared file | Resolution |
|---|---|---|
| 0, 0b, 3 | none | each owns new files; `README.md` only in 3; `migration/cost-log.tsv` only in 3 |
| 1a, 1b | none | 1b owns a new test file |
| 1a, 2 | none | 2 edits `deadletter.go` and a new test file; it uses the `Publish` of 1a |

## Open API choices (ADR 0011 owner decision 2, recorded here per O-2 b; the index #1020 links this section)

| Candidate export | Consumers | Concrete benefit (ADR 0011 criterion) | Alternative considered |
|---|---|---|---|
| `actor.Publish(s *System, event any) error` | `pekko-actor-typed` (#959 `EventStream.Publish`, ADR 0036 EVT-1: `UnhandledMessage` and `Dropped`), the testkit probes (M-03), producers of `SuppressedDeadLetter` and `Dropped` (#871, #906) | Typed and the testkit cannot publish otherwise; ADR 0036 EVT-1 puts the stream in `pekko-actor` and Typed publishes to it | a method on an exported stream type: the stream type is not exported by decision of the #836 scope; a function keeps one entry point beside `Subscribe` |
| `actor.ErrNilEvent` | callers that branch with `errors.Is`; tests | ADR 0038 D4 | unexported: callers could not tell the rejection |

## Decisions for the Owner

- **D-1. Kit run required** (a port). Alternative: waive it; not proposed.
- **D-2. `Publish(s *System, event any) error`**, returning only `ErrNilEvent`. A failed delivery to a subscriber is never an error of `Publish` (EV8: the publisher is not told). Alternative: return the number of subscribers addressed; not proposed (Pekko's `publish` returns `Unit`).
- **D-3. The cycle rule for `DeadLetter` events (P9).** The oracle does not observe what Pekko does when a `DeadLetter` cannot be delivered to a subscriber of `DeadLetter` (in EV8 [006], [008] the `DeadLetter` subscriber `dl` is not the full one). **Prediction by code reading, to be confirmed or refuted by EV11:** a full bounded mailbox does `deadLetters.tell(DeadLetter(handle.message, handle.sender, receiver))` (`Mailbox.scala:560-568`), and `DeadLetterActorRef.!` publishes a `DeadLetter` message as it is (`ActorRef.scala:739-743`), so for a full `DeadLetter` subscriber the stream is re-entered with `DeadLetter(DeadLetter(...))`; Pekko probably recurses too. The risk is real in goPekko: `tell` calls `System.publish` synchronously (`ref.go:78-83`). Per kit rule 8 the unit takes the most conservative rule (a failed delivery of a `DeadLetter` event reaches the hub callbacks and is not published to the stream again) and leaves `TODO(EVT-03-deadletter-cycle)`; slice 0b observes it and, if Pekko differs in a way that matters, the Dev stops and reports so the Owner decides before slice 2. Alternative: do not publish `DeadLetter` events to the stream in this unit (and let #1426 wait for the Owner's decision on the cycle); not proposed, because #1426 needs the events.
- **D-4. `SubscribeDeadLetters` is deprecated here, not removed** (removal is the last PR of #1426); the two coexist inside `pekko-actor` as an implementation state, not as a contract. Alternative: remove it here; not proposed (41 test files still use it).
- **D-5. Exports:** `Publish` and `ErrNilEvent` as in "Open API choices". Alternative: export only `Publish` and report nil through the existing `ErrNilSubscriber`-style sentinel; not proposed (a different cause needs a different error).
- **D-6. Order among subscribers is unspecified** (EV3: it varies between runs); no test depends on it. Alternative: promise registration order; not proposed (Pekko does not).
- **D-7. #870 to `status:ready`** when this scope is approved, and the order of the unit's PRs as above; it cannot start until the unit of #836 is closed. Alternative: set `status:ready` only when #836 closes; the Owner chooses.
- **D-8. The mutation of P9 fails by a fatal stack overflow, not by an assertion.** Go cannot recover a stack overflow, so the test binary dies. Proposed: QA accepts it explicitly (as the 30 s timeout was accepted for #1394 and #1391); the test itself, with the guard, asserts bounded counts. Alternative: add a depth-counting seam to the production path so the mutation becomes an assertion; not proposed (production code shaped by a test). **Owner's condition (G2, 2026-11-07): this is accepted only because mutations are run by hand in a disposable worktree; no ordinary test may depend on a crash, so the guard test itself must pass or fail without crashing the binary.**
- **D-9. The linearization criterion of ADR 0038 D5 (accepted deliveries complete after `Unsubscribe`, EV5 and EV10)** that the Update of #870 lists moves to [#905](https://github.com/getsyntegrity/gopekko/issues/905) (its Update carries it): it can only be tested through `Unsubscribe`. Alternative: test the publish side here with an internal removal seam; not proposed.
- **D-10. The delivery closure is keyed by the subscriber's `ID`, the first one registered is kept** (merged in #1438; QA noted it; `eventstream.go` stores it only if the ID is not already there, and the second case of `eventstream_internal_test.go` pins "first kept"). For a native `Ref[M]` `Subscribe` has checked that the channel is assignable to `M`, so `e.(M)` holds for every event `Publish` can route to it; the criterion P1 therefore also asserts that publishing to a native subscriber never panics. A foreign `Ref` implementation that reuses an `ID` with another `M` is not supported (unknown in Pekko, conservative rule of kit 8) and could panic at publish time; that is a known limit, not a promise. Alternative: key by (`ID`, `M`); not proposed.

## Not carried here (each with its owner)

| Item | Owner |
|---|---|
| `Unsubscribe` and the unsubscribe linearization tested through the API | [#905](https://github.com/getsyntegrity/gopekko/issues/905) |
| Removal of a subscriber that stops; the window of dead letters before it | [#1056](https://github.com/getsyntegrity/gopekko/issues/1056) |
| The `sender` of `DeadLetter` and `Dropped`; `SuppressedDeadLetter`; `AllDeadLetters` | [#871](https://github.com/getsyntegrity/gopekko/issues/871), [#906](https://github.com/getsyntegrity/gopekko/issues/906), [#1057](https://github.com/getsyntegrity/gopekko/issues/1057) |
| Migration of the 42 test files and removal of the callback and the hub | [#1426](https://github.com/getsyntegrity/gopekko/issues/1426) |
| Typed `EventStream` commands | [#959](https://github.com/getsyntegrity/gopekko/issues/959) |
| `UnhandledMessage` publication | [#897](https://github.com/getsyntegrity/gopekko/issues/897), [#1418](https://github.com/getsyntegrity/gopekko/issues/1418) |

## Allowed files

Exactly the paths in the slice table. `scripts/deps.allow` is not touched.

## Dependencies

- The unit of [#836](836-eventstream-registration.md) closed (`Subscribe`, the index and the relation on `develop`).
- ADR 0038 (Accepted). The oracle addendum EV11 (slice 0b) before slice 2.

## Appetite

Six small PRs. The risk is slice 2: a `DeadLetter` produced while delivering a `DeadLetter` must not loop, and Pekko's own behavior is unobserved until EV11.

## Definition of Done

- P1 to P11 have their tests (or review) and pass; P9 stays provisional until EV11 is observed; the PRs name the mutations in a "Mutations" table, reproduced by QA (unmutated green, mutated red).
- `scripts/each_module.sh go test -count=1 ./...`, `scripts/check_deps.sh`, the complexity lint (gocyclo ≤ 10) and the coverage gate (85 % per module; diff coverage ≥ 85 %, with the real `diff-cover`) with the commands of `AGENTS.md`; `-count=100` on the concurrent tests; no `-race`; `unshare` does not exist on macOS and the evidence says so.
- Every ported file carries Apache-2.0 and the one `// Ported from ...` line the RULEBOOK §6 allows (`check-headers.sh`).
- The PRs say `Refs #870`, never `Closes`; the issue is closed by hand with its closure Update after slice 3.
- Every sentence that says something is observed names the oracle line or the assertion; every Pekko citation is verified by the Dev; the evidence is a comment bound to the final head SHA with real exit codes.
