# Scope: Remove the registrations of a subscriber that terminated (#1056, EVT-17)

Status: **closed, delivered on 2026-11-07** by the PRs #1440 (this scope), #1457 (kit package), #1458 (oracle EV12), #1459 (slice 1a-i), #1460 (slice 1a-ii), #1461 (slice 1b) and the docs and record PR of slice 2 (`migration/stage-1/eventstream-termination/EXECUTION-RECORD.md`). Approved by the Owner at gate G2 (2026-11-07), D-1 to D-7 with two conditions (D-2, D-3) and D-8 rejected; D-4 confirmed by the Owner on 2026-11-07 as 'behave as Pekko' (EV12). The text below is the scope as approved, with three corrections made at the close: the file `pekko-actor/actor/eventstream_publish_delivery_test.go` joined the allowed files of slice 1a-i (approved by the TL, PR #1459), the citation of the `Terminated` case reads `EventStreamUnsubscriber.scala:59-64` (line 58 is blank), and this status line.
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: the unit ports the behavior of `EventStreamUnsubscriber` (`event/EventStreamUnsubscriber.scala:35-65`: `Register`, `UnregisterIfNoMoreSubscribedChannels`, `Terminated`) and the calls to it from `EventStream.subscribe/unsubscribe` (`event/EventStream.scala:56-62, 64-78`), with the mechanism adapted (decision D-2), so the Dev runs the migration kit (`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK under `migration/stage-1/eventstream-termination/`. The Dev is a separate session, QA is independent of both, and the TL does not implement.

Contract sources: [ADR 0038](../adr/0038-event-stream-subscription-semantics.md) (Accepted: D6), the oracle [eventstream-spec-2026-11-06.md](../parity/oracles/eventstream-spec-2026-11-06.md) (`S7` its section 1; `EV7` its scenario; `[nnn]` lines of its logs), the scopes of [#836](836-eventstream-registration.md), [#870](870-eventstream-publish.md), [#905](905-eventstream-unsubscribe.md), the issue #1056.

## What this delivers

When a subscribed actor of the System stops, the stream removes every registration it owns, **asynchronously**, so a dead subscriber is neither retained nor messaged; and when a subscriber unsubscribes from its last channel the stream stops watching it. It exports nothing new.

## Evidence: oracle, and what goPekko does today

| Case | Pekko 1.7.1 (oracle) | goPekko `develop` `9725e542` (reading) |
|---|---|---|
| A subscriber stops; later publications | removed after the unsubscriber processed its `Terminated`: `dead letters for victim in total = 0` for 40 publications [045], the live subscriber got all 40 [047] | `Subscribe` registers; nothing watches the subscriber |
| The window before the removal | a publication reaches `deadLetters` as `DeadLetter(event, deadLetters, subscriber)` [049]; 0 or 1 dead letters in the kept runs (`racer`: `in total = 1` of 300 [351]) | the dead letter is `DeadLetter{Msg, To}` through `Tell` (`ref.go:72-76`) |
| Subscribing an already terminated actor | returns `true` [353]; "removed shortly after" is Inferred from the dead-letter count of [396] and the second `subscribe` of [397] (in run 2 these lines are numbered [394] and [395]) | — |
| Unwatch when no channel is left | `UnregisterIfNoMoreSubscribedChannels` checks `hasSubscriptions` and unwatches (`EventStreamUnsubscriber.scala:49-57`; `EventStream.scala:67, 77`); the spec tests are `EventStreamSpec:375, 395` | — |
| Subscribe on a System that is stopping | **not observed** (the oracle does not cover it); `subscribe` first calls `registerWithUnsubscriber` (`EventStream.scala:60`) | `Subscribe` does not refuse it; marker `TODO(EVT-02-stopping)` (`eventstream.go:175`, merged in #1438) |

How a termination reaches the stream in goPekko (read on `origin/develop`; the Dev re-verifies): a watched cell keeps its watchers as `watcherLink` values (`id()`, `notifyTerminated(subject, gen)`; `watch.go:79-82`), registered with `addWatcher` (`watch.go:384-395`, refused once the cell is closed) and notified by `finalize` through `notifyWatchers` after PostStop and before `close(done)` (`stop.go:126`, `watch.go:438-442`). No other registration for a termination exists except `System.Done(h)` (a channel closed after the watchers were told; survey of 2026-11-06). **A terminated native actor is retired from `System.byID` after `close(done)`** (`hierarchy.go` `complete`/`retireLocked`, `registry.go` `lookupID`: nil once retired), so a stopped `Ref` has **no record** to call `addWatcher` on; the merged `watch.go` treats `rec == nil || !rec.addWatcher(...)` as "the subject has finished" and notifies at once (`watch.go:206-208`). The `record` interface already has `addWatcher` and `removeWatcher` (`registry.go:18-19`).

## Criteria, scenarios, sources and mutations

Tests: go-specs, channels or gates, no sleeps, no `-race`. A mutation is applied to production code in a disposable worktree and never committed; the named test must fail by assertion. **No test asserts a count of dead letters inside the window** (it is timing; ADR 0038 D6): they assert invariants after the removal is observed.

| # | Criterion | Scenario (test) | Source / oracle | Mutation | Slice |
|---|---|---|---|---|---|
| T1 | When a subscribed actor stops, all its registrations (on every channel) are removed; once the removal is observed (the fence is `<-System.Done(ref)`: the stream's callback runs in `finalize` before `close(done)`, so after `Done` the removal has happened; never a sleep), further publications neither reach it nor produce dead letters for it, and a live subscriber still receives every event | `TestSubscriberTerminationRemovesItsRegistrations` | EV7 [045], [047], [351]; `EventStreamUnsubscriber.scala:59-64` (the `Terminated` case) | the watcher does not remove on termination | 1a |
| T2 | Subscribing an actor that is already stopped reports `true`, registers it, and the stream **removes it asynchronously after `Subscribe` returned** (ADR 0038 D6; EV7 [353], [396]-[397]); once the removal is observed it holds no registration and no closure. The window is not asserted (D-6). **Both paths**: (a) the actor is stopped but its record still exists and `addWatcher` returns false (the window between `closed` and retirement is reached deterministically by a test `watcherLink` registered on the subscriber that calls `Subscribe` from its `notifyTerminated`, which `finalize` runs after the cell is closed and before it is retired); (b) the actor is stopped **and retired**, so the lookup returns nil for a native handle (the test subscribes after `<-System.Done(ref)`, the deterministic fence). A `Ref` owned by **another System** is resolved with `owner().lookup`, not in the stream's own System (as `watch.go` `resolve` does): a live `Ref` of another System is not removed | `TestSubscribeAnAlreadyStoppedActor` (subtests a, b; the removal after a refused `addWatcher` or a nil record runs on its own goroutine, which the Dev names in the PR; an unexported seam holds that goroutine on a gate the test releases and closes a `done` channel when the removal ended; never a sleep) and `TestSubscribeALiveRefOfAnotherSystemIsKept` | EV7 [353], [396]-[397]; `T:327-356`; `watch.go:136-150, 206-208` | (a) ignore the refusal of `addWatcher`; (b) treat a nil record as 'not stopped' (the registration stays); (c) remove synchronously inside `Subscribe` (with the gate closed the test sees the registration present when `Subscribe` returns, then releases the gate and waits for `done`; no timing involved) | 1a |
| T3 | One watcher per subscriber, not per channel: the stream's link has an identity **minted per stream** (a fresh ID when the stream is created, so two Systems' streams never overwrite each other in `cell.watchers`, which is keyed by `w.id()`); registering the same actor on several channels leaves one entry in its `watchers` map, and stopping it removes all its registrations once | internal test `TestOneWatcherPerSubscriber` | `EventStreamUnsubscriber.scala:47` (`context.watch(actor)` on `Register`) | the link ID is minted per `Subscribe` call **and** the register-at-first-registration guard is dropped (so `addWatcher` runs on every `Subscribe` with a new ID: the number of entries in `cell.watchers` grows with the channels) | 1a |
| T4 | No stale reference: after the removal the stream holds neither the index entry nor the delivery closure of the stopped subscriber, and the publish cache of #870 no longer lists it (the termination removal goes through the same path as `UnsubscribeAll` so the cache is invalidated) | internal test `TestTerminationDropsTheDeliveryClosure` | issue story; ADR 0038 D6 | the closure is kept; or the cache is not invalidated | 1a |
| T4b | A termination removes **only the terminated subscriber**: another live subscriber on the same channels keeps every registration and still receives every event (issue AC-R3-1) | `TestTerminationRemovesOnlyTheTerminatedSubscriber` | EV7 [047]; issue #1056 AC-R3-1 | clear all subscribers on any termination | 1a |
| T5 | Unwatch: when `Unsubscribe`/`UnsubscribeAll` leaves a subscriber with no registration the stream withdraws its termination watcher; if it still has registrations in other channels the watcher stays (Pekko's `hasSubscriptions` check) | internal test `TestUnsubscribeWithdrawsTheWatcherOnlyWhenNoChannelIsLeft` | `EventStreamUnsubscriber.scala:49-57`; `T:375, 395`; `EventStream.scala:67, 77` | always withdraw (the watcher goes away with channels left) | 1b |
| T6 | No lock inversion: the lock order is **stream lock, then the registry lock (`s.mu`, through the lookup in `Subscribe`), then the cell's lock (in `addWatcher`)**, and `Unsubscribe`/`removeWatcher` keep it; `finalize` takes the watchers under the cell lock, releases it, then notifies, so the callback takes the stream lock with no cell lock held. A concurrent `Subscribe`/`Unsubscribe` and the termination of the same subscriber never deadlock and lose no removal (64 goroutines released by a gate, `-count=100`; the mutation is probabilistic by nature, no `-race`) | `TestTerminationConcurrentWithSubscribe` | `watch.go:438-442`, `stop.go:119-126` | the callback holds the cell lock while calling the stream (not reachable without changing `stop.go`, outside this unit): the test therefore asserts no deadlock and no lost removal, and its mutation is 'the callback skips the removal when the stream lock is busy' (a lost removal) | 1b |
| T7 | `Subscribe` on a System that is stopping behaves as the oracle scenario EV12 observed on Pekko (decision D-4); the marker `TODO(EVT-02-stopping)` is resolved | `TestSubscribeOnAStoppingSystem` | **pending EV12** (slice 0b) | the opposite behavior | 1b |
| T8 | A `Ref` that is **not an `ownedHandle`** (a foreign implementation: no cell, no owner System) cannot be watched: `Subscribe` accepts it and the stream never removes it automatically (known limit, written in the doc comment, in the D14.3 section below and in one line of the README); no test can terminate it, so this is a review item | review | D-3; ADR 0033 D14.3 | none (review) | 2 |
| T9 | The README says what exists and what is planned; no export is added | review | module README contract | none (documentation) | 2 |

Every sentence of the PRs that says something was observed names the oracle line or the assertion.

## Slices (one PR each, at most 400 authored lines, sequential)

Estimates are the TL's raw numbers times 1.55; the TL wrote no prototype. A slice above 400 is split by the Dev before opening the PR.

| Slice | Delivers | Files | Criteria | Est. lines (raw → re-estimated) |
|---|---|---|---|---|
| **0. Kit package** | manifest, RULEBOOK, RUNBOOK | `migration/stage-1/eventstream-termination/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | kit run | ~110 docs |
| **0b. Oracle addendum EV12** | the scenario EV12 on Pekko 1.7.1: `subscribe` on an `EventStream` of a System that is terminating and on one that is terminated (`terminate()` called, then `whenTerminated`), the return value, whether it throws, whether a publication reaches the subscriber, whether the system's unsubscriber is gone; harness, trimmed logs, oracle section as EV1t/EV4f did | `docs/parity/harness/...`, `docs/parity/oracles/eventstream-spec-2026-11-06.md`, `.../eventstream-spec-2026-11-06.logs/` | T7 | ~100 docs and logs |
| **1a. The termination watcher** | the stream's internal `watcherLink`, registered at the first registration of a subscriber, removing all its registrations on notification; the refusal path | `pekko-actor/actor/eventstream_publish_delivery_test.go` (case 2 of P6 adapted, approved by the TL, PR #1459), `pekko-actor/actor/eventstream_watch.go` (new), `pekko-actor/actor/eventstream.go` (the call at `Subscribe`), `eventstream_watch_internal_test.go`, `eventstream_termination_test.go` | T1 to T4, T4b | ~90 + ~170 → ~400 (**the Dev splits it if above 400: the termination watcher and T1, T2 first; T3, T4, T4b second**) |
| **1b. Unwatch, concurrency, stopping System** | the unwatch hook in `Unsubscribe`/`UnsubscribeAll`; the resolution of the stopping-System marker | `pekko-actor/actor/eventstream.go`, `eventstream_termination_rule_test.go`, `eventstream_termination_internal_test.go` | T5 to T7 | ~50 + ~130 → ~280 |
| **2. Docs and record** | README of `pekko-actor`, `EXECUTION-RECORD.md`, cost-log row | `pekko-actor/README.md` (the foreign-`Ref` line), `migration/stage-1/eventstream-termination/EXECUTION-RECORD.md` (records the D14.3 sections of D-2 and D-3 of this scope), `migration/cost-log.tsv` | T8, T9 | ~110 docs |

Order: 0, 0b, 1a, 1b, 2. The `record` interface already has `addWatcher` and `removeWatcher` (`registry.go:18-19`), so no edit of the registry is needed. Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `pekko-actor/internal/` or another module changes; any other change is a Mandatory Stop and goes back to the TL.

| Pair | Shared file | Resolution |
|---|---|---|
| 0, 0b, 2 | none | each owns new files; `README.md` and `migration/cost-log.tsv` only in 2 |
| 1a, 1b | `actor/eventstream.go` | sequenced (1a first) |

## Open API choices (ADR 0011 owner decision 2)

None: the unit exports nothing. The watcher, the hook and the internal tests are unexported. This section is the record for the index #1020: "no export".

## Decisions for the Owner

- **D-1. Kit run required** (an adaptation of a Pekko actor's behavior). Alternative: waive it; not proposed.
- **D-2. The mechanism.** Proposed: **an internal `watcherLink` owned by the stream**, registered on the subscriber's cell with `addWatcher` and notified by `finalize`; it removes the registrations when notified. Why: Pekko's mechanism is a system actor (`eventStreamUnsubscriber`) that `watch`es the subscribers (S7); the equivalent in goPekko would need a system actor under `/system`, which does not exist yet (plan row F-01, NEW-09: "`spawnTarget` uses `/user` only"; the story [#1055](https://github.com/getsyntegrity/gopekko/issues/1055), SYS-18, exists and is open); a `watcherLink` is the runtime's own watch mechanism without an actor, and the observable behavior is the same (asynchronous removal on termination, ADR 0038 D6). **Condition of the Owner (G2): the `watcherLink` must preserve every observable of ADR 0038 D6** (asynchronous removal, no promised window, an already stopped subscriber registered and removed shortly after); if it cannot in Go, the Dev stops and the TL proposes an amendment to ADR 0038, not written without the Owner's OK. Alternatives: (A) wait for #1055 and port the unsubscriber as an actor (faithful in structure, blocked by that story); (C) one goroutine per subscriber blocked on `System.Done(ref)` (works for any `Ref` that exposes it, but costs a goroutine per subscriber and has no hook for the unwatch).
- **D-2 deviation (D14.3 section, minor).** Pekko's unsubscriber is an actor under `/system`; goPekko has no `/system` actor and the stream's own link replaces it. The only difference a program can see is the absence of that actor in the tree. **Note for the day #1055 lands: do not migrate the link to an actor unless the migration changes something observable.**
- **D-3. A foreign `Ref` implementation** (not an `ownedHandle`: no cell, no owner System) cannot be watched and is never auto-removed (T8). Alternative: reject its registration with an error; not proposed (D-4 of the scope of #836 accepts any `Ref[M]`).
- **D-3 deviation (D14.3 section).** Pekko removes any terminated subscriber because every `ActorRef` is watchable; a foreign `Ref` of goPekko has no cell to watch, so its registrations stay until `Unsubscribe`/`UnsubscribeAll`; observable as a retained subscriber and a dead letter per publication after it stopped. Slice 2 puts this in one line of the `pekko-actor` README.
- **D-4. `Subscribe` on a System that is stopping** (the oracle EV12 covers `subscribe` on a System that is terminating and on a terminated one, as the scope of #836 D-5 asked) is decided by the observation of EV12 (slice 0b), not by this scope: the Dev reports the result to the TL and the Owner confirms before slice 1b. The default until then stays as merged in #1438 (not refused). Alternative: decide now; not proposed (no evidence).
- **D-5. The removal order among the subscribers of one terminating actor** is unspecified (`notifyWatchers` iterates a map, `watch.go:438-442`), and no test depends on it.
- **D-6. No count of dead letters in the window** is promised or asserted (T1; ADR 0038 D6).
- **D-7. #1056 to `status:ready`** at the merge of this scope; it starts after #905 (closed 2026-11-07).
- **D-8** was proposed (remove inside `Subscribe`) and **rejected by the Owner at G2**: it contradicted ADR 0038 D6. T2 follows D6.

## Not carried here (each with its owner)

| Item | Owner |
|---|---|
| Explicit `Unsubscribe` and its linearization | [#905](https://github.com/getsyntegrity/gopekko/issues/905) |
| `Publish` and the cache | [#870](https://github.com/getsyntegrity/gopekko/issues/870) |
| The debug logs of the unsubscriber ("watching ...", "unwatching ...", "unsubscribe ... because it was terminated") | not ported: logs go through the logger (ADR 0038 Deviation 2) |
| A `/system` actor spawn (alternative A of D-2) | [#1055](https://github.com/getsyntegrity/gopekko/issues/1055) (SYS-18, open; plan row F-01, NEW-09) |
| `Terminated` of an actor of another System (cross-system watch) | a `Ref` owned by another System is resolved with `owner().lookup` (T2) and its cell, if live, gets the stream's link like a local one: when it terminates, its registrations in this stream are removed through the same callback; the cross-system watch machinery (`watch_cross_system`) is not used |

## Allowed files

Exactly the paths in the slice table; internal test files (`<file>_internal_test.go`, same package) are allowed in any slice with the same budget. `scripts/deps.allow` is not touched.

## Dependencies

(The links to the scopes of #870 and #905 resolve once PRs #1437 and #1439 are merged.)

- The unit of [#905](905-eventstream-unsubscribe.md) closed (`Unsubscribe`, `UnsubscribeAll`, the closure cleanup). The units of #836 and #870 closed.
- ADR 0038 (Accepted). The oracle addendum EV12 (slice 0b) before slice 1b.

## Appetite

Five small PRs. The risks are T6 (lock order between the stream and the cell, a probabilistic test without `-race`) and the open question of the stopping System.

## Definition of Done

- T1 to T9 have their tests (or review) and pass; T7 is pending EV12 until slice 0b; the PRs name the mutations in a "Mutations" table, reproduced by QA (unmutated green, mutated red).
- `scripts/each_module.sh go test -count=1 ./...`, `scripts/check_deps.sh`, the complexity lint (gocyclo ≤ 10) and the coverage gate (85 % per module; diff coverage ≥ 85 %, with the real `diff-cover`) with the commands of `AGENTS.md`; `-count=100` on the concurrent tests; no `-race`; `unshare` does not exist on macOS and the evidence says so.
- Every ported file carries Apache-2.0 and the one `// Ported from ...` line the RULEBOOK §6 allows (`check-headers.sh`; a file adapted from an actor's behavior may carry none: the manifest records its provenance).
- The PRs say `Refs #1056`, never `Closes`; the issue is closed by hand with its closure Update after slice 2.
- Every sentence that says something is observed names the oracle line or the assertion; every Pekko citation is verified by the Dev; the evidence is a comment bound to the final head SHA with real exit codes.
