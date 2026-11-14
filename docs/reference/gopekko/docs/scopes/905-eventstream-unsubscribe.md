# Scope: Unsubscribe one classification or all registrations of an actor subscriber (#905, EVT-04)

Status: **approved by the Owner at gate G2 on 2026-11-07** ("#905: apruebo D-1 a D-7. EV13 (slice 0b) va antes de cualquier código de producción, y U1b/U3b quedan 'inferidos' hasta entonces."). Written on 2026-11-06 against `develop` `9725e542` (the stream of #836 has `Subscribe` and the index with removal; `Publish` of #870 is not merged yet: this scope assumes it). #905 is `prio:P0`, `status:blocked`; its criteria are those of the issue plus its Update of 2026-11-06 (ADR 0038). It starts after the unit of [#870](870-eventstream-publish.md) is closed. The removal of a subscriber that **stops** is [#1056](1056-eventstream-termination.md), not this unit.
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: the unit ports `SubchannelClassification.unsubscribe` (both forms, `event/EventBus.scala:165-175`, with the cache update) and `EventStream.unsubscribe` (both forms, `event/EventStream.scala:64-66, 74-76`: the null check; the unwatch half is #1056), so the Dev runs the migration kit (`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK under `migration/stage-1/eventstream-unsubscribe/`. The Dev is a separate session, QA is independent of both, and the TL does not implement.

Contract sources: [ADR 0038](../adr/0038-event-stream-subscription-semantics.md) (Accepted: D3 incl. the unsubscribe rule of O-1, D4, D5; Amendment (a)), the oracle [eventstream-spec-2026-11-06.md](../parity/oracles/eventstream-spec-2026-11-06.md) (`S1` to `S10` its section 1; `EV1`, `EV1t`, `EV1tv2`, `EV1tv3`, `EV4`, `EV5a`, `EV5b`, `EV10` its scenarios; `[nnn]` lines of its logs), the scopes of [#836](836-eventstream-registration.md) and [#870](870-eventstream-publish.md), the issue #905. The issue's cited `ActorRef.scala` is wrong (its own reconciliation says so); the sources are `EventStream.scala:64-66, 74-76`, `EventBus.scala:165-175`, `EventStreamUnsubscriber.scala:49-58` and `EventStreamSpec:298, 327`.

## What this delivers

`Unsubscribe` (one channel) and `UnsubscribeAll` (every registration of a subscriber) on the `EventStream`, with Pekko's unsubscribe rule (a removal at a channel cascades to its sub-channels, whichever channel supplied the registration), the publish cache kept in step, and the guarantee of ADR 0038 D5: deliveries already accepted into a mailbox complete, a publication that starts after `Unsubscribe` returned does not address the subscriber. The index already has the removal (#836 slice 1b); this unit exposes it and wires the cache.

## Proposed Go API (for the Owner at G2)

```go
// Unsubscribe removes the registrations of subscriber at channel E and at every sub-channel of E, whichever channel
// supplied them (ADR 0038 D3, O-1). changed is Pekko's diff.nonEmpty (EV1t [005], EV1tv2 [005]/[008], EV1tv3 [006]/[009]):
// true when the removal changed the index. NOTE: that includes the nodes the index creates when E was not a node and
// adopts known sub-channels (SubclassifiedIndex.scala:157-163, 222-228), so changed can be true for a subscriber that was
// never registered (see U1b); it is not 'the subscriber was removed'.
func Unsubscribe[E any](s *System, subscriber Handle) (changed bool, err error)

// UnsubscribeAll removes every registration of subscriber; it reports nothing (Pekko: Unit, EV1 [012]).
func UnsubscribeAll(s *System, subscriber Handle) error
```

Both return `ErrNilSubscriber` for a nil handle (ADR 0038 D4, EV2 [001]-[003]). They take a `Handle` and not a `Ref[M]`: a removal needs only the identity, and a `Ref[M]` whose `M` does not accept `E` cannot be registered at any channel that covers `E` (a channel that covers `E` and is accepted by `M` makes `E` assignable to `M`), so there is nothing to check.

## Evidence: oracle, and what goPekko does today

| Case | Pekko 1.7.1 (oracle) | goPekko `develop` `9725e542` (reading) |
|---|---|---|
| Unsubscribe a pair never registered | `false` (EV1 [006], [007]) | no `Unsubscribe`; the index's `RemoveValue` exists (#1435) |
| Unsubscribe twice | `true`, then `false` (EV1 [008], [009]) | same |
| Unsubscribe from everything | returns `()`; no later delivery (EV1 [012]) | `RemoveValueEverywhere` (#1435) |
| The unsubscribe rule, with interfaces/traits | v1 `true` and delivery stops, v2 `true` then `false`, v3 `true` and delivery via the other registration stops too (EV1t [005]/[008], EV1tv2 [005]/[008], EV1tv3 [006]/[009]) | the index reproduces it (#836 X3, tested at index level) |
| Accepted deliveries complete after the call; a later publication does not reach | `e2`-`e4` delivered, `e5` not (EV5a [006], [008]-[010]; both forms) | `Publish` (#870) delivers by `Tell`; the cache is read without the lock |
| Concurrent publish and unsubscribe | a gap-free prefix is delivered, nothing after the call returned (EV10 [004]; 3 runs) | — |

## Criteria, scenarios, sources and mutations

Tests: go-specs, channels or gates, no sleeps, no `-race`. A mutation is applied to production code in a disposable worktree and never committed; the named test must fail by assertion. The public tests use the types `I`, `J` (J's method set includes I's) and `C` (implements J), as in EV1t.

| # | Criterion | Scenario (test) | Source / oracle | Mutation | Slice |
|---|---|---|---|---|---|
| U1 | `Unsubscribe` reports `false` for a pair never registered **on a stream where the channel and its sub-channels are unknown or hold no other subscriber**, `true` for a registered one, `false` the second time; on a stream never used (zero value) it returns `false` with no panic | `TestUnsubscribeReturnValues` | EV1 [006]-[009] | return `true` always | 1a |
| U1b | **Observed (EV13, merged #1452):** `Unsubscribe[J]` for a subscriber never registered, on a stream where J is not a node but a known sub-channel `C` holds another subscriber `x`, returns `true` (EV13 [006]; the index creates the node J and adopts `C`: Pekko's `diff.nonEmpty`) and `x` keeps receiving `C` (EV13 [002], [007]); when J already is a node (another subscriber `y` on J) it returns `false` (EV13v3 [008]) and `C` reaches `y` and `x` ([009], [010]) | `TestUnsubscribeOfANeverRegisteredSubscriberCanReportTrue` (both cases) | EV13 [002], [006], [007]; EV13v3 [008]-[010]; `SubclassifiedIndex.scala:157-163, 222-228` (explanation, Inferred) | return `false` when the subscriber was not registered; or `true` also when J is already a node | 1a |
| U2 | `UnsubscribeAll` removes every registration of the subscriber and reports nothing; a later `Publish` addresses it on none of its channels; **the cache of a channel shared with another subscriber keeps that other subscriber** (the all-form updates the cache by set difference, `removeFromCache`, `EventBus.scala:201-204`; the cache stores copies, because the sets the index returns are live views, see #1436) | `TestUnsubscribeAllRemovesEveryRegistration` | EV1 [012]; `EventBus.scala:171-175, 201-204` | assign the returned set to the cache entry instead of subtracting it (the other subscriber disappears from the channel) | 1a |
| U3 | The unsubscribe rule, three variants through the public API: v1 `a` on `I` only, `Unsubscribe[J]` → `true`, `Publish(C)` no longer reaches `a`, `Unsubscribe[I]` → `true`; v2 `a` on `J` only, `Unsubscribe[I]` → `true` and delivery stops, `Unsubscribe[J]` → `false`; v3 `a` on `I` and `J`, `Unsubscribe[J]` → `true` and `C` is **not** delivered via `I` either, `Unsubscribe[I]` → `true`; and the class case of EV1 [015]-[018] with its Go analog: interface `Base` and concrete `Sub` (a on `Base`, `Unsubscribe[Sub]` → `true` and `Sub` events stop, later `Unsubscribe[Base]` → `true`). **Also (U3b, observed by EV13v2 and EV13v4, merged #1452):** `a` registered on `I` only, `K` an interface unrelated to `I`, `C2` implements both: `Unsubscribe[K]` does **not** stop the delivery of `C2` to `a` (EV13v2 [002], [006]; EV13v4 [003]). The Boolean it returns **depends on which event classes the stream has already seen**: `false` when `C2` was never published (EV13v4 [002]) and `true` when it had been published before (EV13v2 [005]: the publish added `C2` to the index as a key, `EventBus.scala:177-187`, `SubclassifiedIndex.scala:111`, and the new node K adopts it, `SubclassifiedIndex.scala:202-214`: the explanation is Inferred). The test asserts both Booleans. The merged Go index reproduces both (QA of #1452, by a throwaway test); `Unsubscribe` must return `len(diff) > 0` and `Publish` must add the key as Pekko does (it does: `eventstream.go`) | `TestUnsubscribeRuleV1V2V3` and `TestUnsubscribeOfAnUnrelatedChannelKeepsASharedSubChannel` | EV1t [005], [008]; EV1tv2 [005], [008]; EV1tv3 [006], [009]; EV1 [015], [018]; ADR 0038 O-1; U3b: EV13v2 [002], [005], [006]; EV13v4 [002], [003] | U3: the cascade removed (remove only at the exact channel); U3b: unsubscribe also removes `a` from the shared sub-channel of the other registration; and for the Boolean of an unrelated channel, `Unsubscribe` returns a constant (the test asserts `false` without the prior publish of `C2` and `true` after it) | 1b |
| U4 | Registrations at channels that are not the removed one nor its sub-channels survive: `a` on `I` and on an unrelated interface `K`; `Unsubscribe[I]` keeps `K` (issue AC1); and the multi-parent case: `C` implements `I` and `K`, `a` on `I` and `K`, `Unsubscribe[I]`: `C` is still delivered to `a` through `K` (the Go analog of EV4 [015]-[017], where the delivery that survives is through `Iface`, an inferred reading) | `TestUnsubscribeKeepsOtherChannels` | the issue AC1; EV4 [015]-[017] (multi-parent analog, Inferred) | `UnsubscribeAll` where `Unsubscribe` is called | 1a |
| U5 | The publish cache follows the removal: a `Publish` after `Unsubscribe` returned does not address the subscriber (the cache is updated with the whole set of changed keys, `EventBus.scala:165-169`, not with the removed value) | `TestUnsubscribeUpdatesTheCache` | `EventBus.scala:165-175`; EV5a [006] | the cache keeps the subscriber | 1a |
| U6 | Linearization (ADR 0038 D5): a subscriber with a mailbox of capacity at least 3 is inside `Receive` on `e1` (entered gate); `Publish` of `e2`, `e3`, `e4`; `Unsubscribe`; `Publish` of `e5`; a fence message is told directly to the subscriber; the gate opens: the subscriber receives `e2`, `e3`, `e4` and then the fence, never `e5` (the fence proves the absence without a sleep); same with `UnsubscribeAll` | `TestUnsubscribeLinearization` (both forms) | EV5a/EV5b [004]-[010]; ADR 0038 D5 | the cache is not updated (the subscriber still receives `e5`) | 1b |
| U7 | Concurrent `Publish` and `Unsubscribe` (ADR 0038 D5: a publication already running may still address the subscriber): the delivered ids are a gap-free prefix `0..m-1` and every delivered id is **at most the started-count read right after `Unsubscribe` returned** (EV10 [004], `maxId <= startedAtReturn`); nothing is asserted about publications that were already running | `TestUnsubscribeConcurrentWithPublish` | EV10 [004] and its 2 other runs; `EventBus.scala:146-147` (S6) | remove the lock around the unsubscribe (probabilistic: `-count=100`, the report says how often it fails) | 1b |
| U8 | Nil: `Unsubscribe` and `UnsubscribeAll` with a nil or typed-nil handle are `ErrNilSubscriber`, before anything else | `TestUnsubscribeRejectsNil` | EV2 [001]-[003] | remove the nil check | 1a |
| U9 | **No stale reference** (the story's purpose): when the last registration of a subscriber is removed the stream drops its delivery closure, and **only then**: the stream detects it with `hasSubscriptions(id)`, a scan of the cache snapshot for the ID (Pekko's own `hasSubscriptions`, `EventBus.scala:192-199`, 'expensive call' by its comment), taken under the stream lock; the index nodes stay (as in Pekko, 'c will never be removed from cache') | internal tests `TestUnsubscribeDropsTheDeliveryClosureWithTheLastRegistration` and `TestUnsubscribeKeepsTheClosureWhileARegistrationRemains` (a on `I` and `K`, `Unsubscribe[I]`: the closure stays) | issue story ('avoid stale-reference retention'); `EventBus.scala:192-199` | (1) the closure is kept after the last registration; (2) the closure is dropped on any unsubscribe | 1a |
| U10 | Exports are justified in "Open API choices"; the README says what exists and what is planned | review | module README contract | none (documentation) | 2 |

Every sentence of the PRs that says something was observed names the oracle line or the assertion.

## Slices (one PR each, at most 400 authored lines, sequential)

Estimates are the TL's raw numbers times 1.55; the TL wrote no prototype. A slice above 400 is split by the Dev before opening the PR.

| Slice | Delivers | Files | Criteria | Est. lines (raw → re-estimated) |
|---|---|---|---|---|
| **0. Kit package** | manifest, RULEBOOK, RUNBOOK | `migration/stage-1/eventstream-unsubscribe/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | kit run | ~110 docs |
| **0b. Oracle addendum EV13** | the scenarios EV13 on Pekko 1.7.1: (1) `unsubscribe(b, J)` for a subscriber never registered, J not a node, a known sub-channel `C` holding another subscriber: the Boolean and the deliveries; (2) `a` on `I`, `K` unrelated to `I`, a class implementing both: `unsubscribe(a, K)`: the Boolean and whether the shared sub-channel still reaches `a` (observed: it still does, EV13v2 [006], EV13v4 [003]); harness, trimmed logs, oracle section as EV1t and EV4f did | `docs/parity/harness/...`, `docs/parity/oracles/eventstream-spec-2026-11-06.md`, `.../eventstream-spec-2026-11-06.logs/` | U1b, U3b | ~100 docs and logs |
| **1a. `Unsubscribe`, `UnsubscribeAll`** | the two functions, the cache update, the closure cleanup | `pekko-actor/actor/eventstream.go`, `eventstream_unsubscribe_test.go`, `eventstream_unsubscribe_internal_test.go` | U1, U1b, U2, U4, U5, U8, U9 | ~70 + ~150 → ~340 |
| **1b. The rule and the timing** | tests only | `pekko-actor/actor/eventstream_unsubscribe_rule_test.go` | U3, U6, U7 | ~170 → ~265 |
| **2. Docs and record** | README of `pekko-actor`, `EXECUTION-RECORD.md`, cost-log row, the record linked from #1020 | `pekko-actor/README.md`, `migration/stage-1/eventstream-unsubscribe/EXECUTION-RECORD.md`, `migration/cost-log.tsv` | U10 | ~110 docs |

Order: 0, 0b, 1a, 1b, 2. `eventstream.go` is edited by 1a only here (it is also edited by #870 and #1056, sequenced by the chain). Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `pekko-actor/internal/` or another module changes; a needed change outside the table is a Mandatory Stop and goes back to the TL.

| Pair | Shared file | Resolution |
|---|---|---|
| 0, 0b, 2 | none | each owns new files; `README.md` and `migration/cost-log.tsv` only in 2 |
| 1a, 1b | none | 1b owns a new test file |

## Open API choices (ADR 0011 owner decision 2, recorded here per O-2 b; the index #1020 links this section)

| Candidate export | Consumers | Concrete benefit (ADR 0011 criterion) | Alternative considered |
|---|---|---|---|
| `actor.Unsubscribe[E any](s *System, subscriber Handle) (bool, error)` | `pekko-actor-typed` (#959 `EventStream.Unsubscribe`), the testkit probes (M-03), `pekko-persistence` tests after #1426 | the counterpart of `Subscribe`; Typed and the testkit cannot unsubscribe otherwise (ADR 0036 EVT-1 puts the stream in `pekko-actor`) | a `Ref[M]` parameter as in `Subscribe`: needs `M` for no gain (only the identity matters) |
| `actor.UnsubscribeAll(s *System, subscriber Handle) error` | the same | Pekko's `unsubscribe(subscriber)`; Go has no overloading, so a second function | one function with an optional channel: not expressible with a type parameter |

## Decisions for the Owner

- **D-1. Kit run required** (a port). Alternative: waive it; not proposed.
- **D-2. The signatures above** (`Handle` instead of `Ref[M]`; two functions because Go has no overloading; `Unsubscribe` per channel with `E` explicit). Why no assignability check: a removal needs only the identity, and the identity is the `ID` (foreign refs and `M = any` need nothing more); a channel `E` that `M` does not accept simply covers no registration of the subscriber, so the call changes nothing it was registered for (its Boolean follows Pekko's `diff.nonEmpty`, see U1b). Alternative: `Unsubscribe[E, M any](s, Ref[M])`, symmetric with `Subscribe`; not proposed (it asks the caller for a type the removal does not use).
- **D-3. The rule of O-1 is exposed as is** (a removal at `E` cascades to the sub-channels of `E`; v3 cuts the delivery via `I` too): it is already the Owner's decision, restated here only because it is the visible surprise of this API. No new decision (a restatement). Alternative, not taken: expose only exact-channel removal; it was the Owner's alternative in O-1 and is closed.
- **D-4. U9 (the closure cleanup) is Go housekeeping with no Pekko counterpart** (Pekko's stream stores no closure). Alternative: leave the closures until #1056; not proposed, because the story's purpose is "avoid stale-reference retention".
- **D-5. No barrier in the public API** (ADR 0038 O-7, already decided). The tests synchronize with gates. A restatement of a decided point; no alternative.
- **D-6. #905 to `status:ready`** when this scope is approved, and the order of the unit's PRs as above; it cannot start until the unit of #870 is closed. Alternative: set `status:ready` only when #870 closes; the Owner chooses.
- **D-7. Exports:** `Unsubscribe` and `UnsubscribeAll` as in "Open API choices". Alternative: one function with a variadic channel list; not expressible with type parameters.

## Not carried here (each with its owner)

| Item | Owner |
|---|---|
| Removal of a subscriber that stops, and the unwatch when it unsubscribes from everything (`unregisterIfNoMoreSubscribedChannels`, `EventStream.scala:67, 77`) | [#1056](https://github.com/getsyntegrity/gopekko/issues/1056) |
| `Publish` and the cache read | [#870](https://github.com/getsyntegrity/gopekko/issues/870) |
| The debug log of `EventStream.unsubscribe` ("unsubscribing ... from channel", `:68-70`) | not ported: goPekko routes logs through the logger (ADR 0038 Deviation 2) |
| Typed `EventStream.Unsubscribe` command | [#959](https://github.com/getsyntegrity/gopekko/issues/959) |

## Allowed files

Exactly the paths in the slice table. `scripts/deps.allow` is not touched.

## Dependencies

- The unit of [#870](870-eventstream-publish.md) closed (`Publish`, the cache). The unit of #836 closed (`Subscribe`, the index with removal).
- ADR 0038 (Accepted). The oracle addendum EV13 (slice 0b, merged #1452: it observed U1b and U3b; the Boolean of U3b depends on the event classes already published) before slice 1a; EV1, EV1t, EV1tv2, EV1tv3, EV4, EV5a/EV5b and EV10 are already in the merged oracle.

## Appetite

Five small PRs. The risk is U3 (the surprising v3 cascade through the public API) and U7 (a race test that cannot use `-race`): both are written from the oracle lines before the code.

## Definition of Done

- U1 to U10 have their tests (or review) and pass; the PRs name the mutations in a "Mutations" table, reproduced by QA (unmutated green, mutated red).
- `scripts/each_module.sh go test -count=1 ./...`, `scripts/check_deps.sh`, the complexity lint (gocyclo ≤ 10) and the coverage gate (85 % per module; diff coverage ≥ 85 %, with the real `diff-cover`) with the commands of `AGENTS.md`; `-count=100` on the concurrent tests; no `-race`; `unshare` does not exist on macOS and the evidence says so.
- Every ported file carries Apache-2.0 and the one `// Ported from ...` line the RULEBOOK §6 allows (`check-headers.sh`).
- The PRs say `Refs #905`, never `Closes`; the issue is closed by hand with its closure Update after slice 2.
- Every sentence that says something is observed names the oracle line or the assertion; every Pekko citation is verified by the Dev; the evidence is a comment bound to the final head SHA with real exit codes.
