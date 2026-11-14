# Scope: Register classified event subscriptions on the `EventStream` (#836, EVT-02)

Status: **approved by the Owner at gate G2 on 2026-11-06** ("si — apruebo el scope de #836 (G2), D-1 a D-7"), with the Owner's grounds for D-2 and D-3 and the notes on D-5 and D-6 recorded in the decisions below. Written on 2026-11-06 against `develop` `9e12e346`. #836 is `prio:P0`, `status:blocked`; its criteria are those of the issue plus the Update of 2026-11-06 (ADR 0038).
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: the unit ports `SubclassifiedIndex` (`util/SubclassifiedIndex.scala`) and the subscribe half of `SubchannelClassification` (`event/EventBus.scala:159-175`) and `EventStream.subscribe` (`event/EventStream.scala:56-62`), so the Dev runs the migration kit (`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK under `migration/stage-1/eventstream-registration/`. The Dev is a separate session, QA is independent of both, and the TL does not implement.

Contract sources: [ADR 0038](../adr/0038-event-stream-subscription-semantics.md) (Accepted 2026-11-06: D1 to D4, D9, Deviation 1), [ADR 0036](../adr/0036-typed-event-stream-and-interceptors.md) (EVT-1, point 2), the oracle [eventstream-spec-2026-11-06.md](../parity/oracles/eventstream-spec-2026-11-06.md) (`S1` to `S10` are its section 1; `EV1`, `EV2`, `EV3`, `EV4`, `EV1t` its scenarios; `[nnn]` lines of its logs), the issue #836.

## What this delivers

Registration, and nothing else, on a new `EventStream` of `pekko-actor`: a subscriber `Ref` is registered for one **channel** (a Go type), the call says whether it changed anything, and the registrations are kept in the same index structure as Pekko's (so that #870 publishes and #905 unsubscribes on it without reshaping it). It publishes nothing, unsubscribes nothing and watches nothing (#870, #905, #1056).

## Proposed Go API (for the Owner at G2)

Signatures only; bodies belong to the Dev. Package `actor` (`pekko-actor/actor`), the stream is a field of `System` and is not exported as a type.

```go
// Subscribe registers subscriber for the events of channel E: an event is addressed to it when its dynamic type is E,
// or implements the interface E, or E is any (ADR 0038 D2). changed is false when the call changed nothing (the pair
// was already registered, or an existing registration already covers it, EV1 [001]-[003], EV4 [007]).
// M is the message type of subscriber; E must be assignable to M (see D-2).
func Subscribe[E, M any](s *System, subscriber Ref[M]) (changed bool, err error)

var ErrNilSubscriber = errors.New("event stream: nil subscriber")            // ADR 0038 D4, EV2 [001]-[003]
var ErrChannelNotAccepted = errors.New("event stream: channel type is not assignable to the subscriber's message type") // D-2
```

Internal (not exported): `pekko-actor/internal/classification` with `Index[K, V]` (the port of `SubclassifiedIndex`), the `Subclassification[K]` typeclass and the relation for `reflect.Type` channels; `pekko-actor/actor/eventstream.go` with the unexported `eventStream` (a mutex, an `Index[reflect.Type, subscriberEntry]`, no cache yet: the publish cache arrives with #870). A `subscriberEntry` is `{id ID; deliver func(event any)}` and is equal to another by `id`; `deliver` is built here (a type assertion to `M` and `Tell`) and used by #870.

## Evidence: oracle, and what goPekko does today

| Case | Pekko 1.7.1 (oracle) | goPekko `develop` `9e12e346` (reading) |
|---|---|---|
| `subscribe` twice, same pair | `true`, `false`; one delivery (EV1 [001]-[003]) | no event stream (oracle section 4: only `SubscribeDeadLetters`, synchronous callbacks) |
| Registration covered by another (`AnyRef`, then `Base`) | `false` (EV4 [007]) | none |
| Null subscriber | `IllegalArgumentException("subscriber is null")`, before anything else (EV2 [001]-[003]) | none |
| Subscribing a class, a trait, `AnyRef`, a JDK interface | all match their subtypes (EV3 [002]-[005], [024], [026]) | none |
| Hierarchy index | `SubclassifiedIndex`: a node per known channel, each with its value set; a new node inherits the set of the most specific known super-channel (`addKey`); `addValue` adds to the key and all its sub-keys (`util/SubclassifiedIndex.scala:92-160`; the claim "inherits" is code reading) | no equivalent: `System.byID map[ID]record` and `deadletter.Hub` are the only registries (survey of 2026-11-06); no `reflect.Type`-keyed registry exists |

There is no Pekko spec of `SubclassifiedIndex` (`find upstream/pekko -name 'SubclassifiedIndexSpec*'` is empty): the index-level tests are ours and each one is tied to an oracle scenario.

## Criteria, scenarios, sources and mutations

Tests: go-specs, channels or gates, no sleeps, no `-race`. A mutation is applied to production code in a disposable worktree and never committed (`qa-review`); the named test must fail by assertion.

| # | Criterion | Scenario (test) | Source / oracle | Mutation | Slice |
|---|---|---|---|---|---|
| X1 | `Index.AddValue(key, value)` returns the changes (the key and every sub-key that gained the value); a second add of the same pair returns none | `TestIndexAddValueReportsChangesOnce` | `S4`; EV1 [001]-[003] | `AddValue` always returns a non-empty set | 1 |
| X2 | A value added to a broader key is inherited by a key added later (`AddKey` starts the new node with the value set of the most specific known super-key), so adding the same value to that sub-key afterwards reports no change | `TestIndexSubKeyInheritsFromSuperKey` | code reading `SubclassifiedIndex.scala:113-132` (`AddKey`) and `:137-150`; observed analog only for a node that already exists: EV4 [007] | `AddKey` starts the new node with an empty set | 1 |
| X3 | `Index.RemoveValue(key, value)` removes the value from the key **and every sub-key**, whichever key supplied it, and reports whether it removed anything; the three variants of EV1t (`I`, `J` under `I`, a leaf `C` under `J`) | `TestIndexRemoveValueEv1tVariants` (v1, v2, v3) | EV1t [005]/[008], EV1tv2 [005]/[008], EV1tv3 [006]/[009] | `RemoveValue` removes only at the exact key | 1 |
| X4a | `Index.FindValues(key)` returns the values of the node of the key | `TestIndexFindValues` | code reading `SubclassifiedIndex.scala:190-200` | `FindValues` returns the parent's set | 1a |
| X4b | `Index.RemoveValue(value)` (all keys) removes the value everywhere and reports the keys that changed | `TestIndexRemoveValueEverywhere` | EV1 [012] shows only `returned ()`; the rest is code reading (`SubclassifiedIndex.scala:185`) | `RemoveValue(value)` stops after the first key | 1b |
| X5 | The channel relation. `isEqual(x, y)` is `x == y`. `isSubclass(x, y)` is `x == y` (equal classes are sub-classes, `SubclassifiedIndex.scala:23-34`) **or** strictly: `x != y`, `y` is an interface, `x.Implements(y)` and **not** (`x` is an interface and `y.Implements(x)`). `any` (the alias of `interface{}`) is the root: every channel except a named empty interface is a sub-channel of it. A named empty interface (`type Empty interface{}`) is *unrelated* to `any` (same method set, so mutually implementing); every non-interface type that implements it is still its sub-channel. A pointer type and its value type are different channels. Two interfaces with identical method sets are unrelated (D-3) | `TestChannelRelation` (includes `isSubclass(x, x)`, the equivalent-interface pair, `any` against `Empty`, `*T` against `T`) | ADR 0038 D2, Deviation 1; EV3 (matching); the Scala contract above | M1: drop the NOT clause (equivalent interfaces become related in both directions and the index would loop); M2: `isSubclass` drops `x == y` (a second `AddValue` of the same key creates a duplicate node and X1 fails) | 2 |
| X6 | `Subscribe` registers and returns `true`, then `false` for the same pair | `TestSubscribeRepeatIsNoChange` | EV1 [001]-[003]; #836 AC1 | return `true` always | 3b |
| X7a | Registration through a channel that is **already a node** and already covers the subscriber reports no change. Setup, as in the oracle: subscribers `x` and `y` registered on `Base` (so `Base` is a node), then subscriber `z` on `any`, then `z` on `Base` reports `false` | `TestSubscribeCoveredByExistingNodeIsNoChange` | EV4 [001]-[007] (`x` and `y` on `Base`, then `z` on `AnyRef`, then `z` on `Base`: `false` [007]) | `Subscribe` always returns `true` | 3b |
| X7b | On a **fresh** stream, `Subscribe[any]` followed by `Subscribe[Base]` for the same subscriber reports `true` (the new node `Base` is created and reported, `SubclassifiedIndex.scala:137-150`, branch `!found`): **code reading, not observed** (delivery is tested by #870, not here); the oracle is extended first with the scenario EV4f (slice 0b), and X7b is pending that observation (ADR 0033 D13) | `TestSubscribeFreshStreamNewNodeIsChange` | code reading; EV4f when it exists | `Subscribe` reports `false` for a channel that was not a node | 3b |
| X12 | A subscriber registered under two matching channels appears **once** in the set of the event's channel (the values of a node are a set, `SubclassifiedIndex.scala` `Nonroot.addValue`), which is what makes #870's "once per subscriber" hold | `TestIndexValuesAreASet` (index) | EV4 [008]-[010] (once under duplicate matches), code reading `:47-53` of `SubclassifiedIndex` values | values kept in a slice | 1 |
| X13 | Concurrent registration is linearizable: 64 goroutines, released together by a gate, subscribe the same pair and exactly one call reports `true`; distinct pairs all report `true` (`-count=100`; the mutation is probabilistic by nature, no `-race`) | `TestSubscribeConcurrentSamePairOneChange` | EV1 [001]-[003]; `EventBus.scala:159-166` (`subscriptions.synchronized`) | remove the lock | 3b |
| X8a | A channel `E` that is not assignable to the subscriber's message type `M` is `ErrChannelNotAccepted`, and nothing is registered | `TestSubscribeRejectsUnacceptableChannel` | ADR 0038 D1, O-6, D-2 | drop the assignability check | 3a |
| X8b | One `Ref[M]` can be registered on several channels it accepts (`E` assignable to `M`): `M = Iface`, channels `Iface` and `*Impl` both report `true` and are two registrations of the same subscriber | `TestSubscribeSeveralChannels` | ADR 0038 D1, O-6; EV4 setup | `Subscribe` keys the registration by `M` instead of `E` | 3b |
| X9 | A nil `Ref` (interface nil or typed nil) is `ErrNilSubscriber`, before anything else, with no registration | `TestSubscribeRejectsNil` | EV2 [001]-[003]; ADR 0038 D4 | remove the nil check | 3a |
| X10 | Registration retains the `Ref` (its `ID` and a `deliver` closure over it), never the `Actor` value: `subscriberEntry` has no field that can hold it, and `Ref` does not expose it | review of the type (no executable test: a reference-retention test would need `runtime.GC` and finalizers, which are nondeterministic) | #836 AC3; `releaseTerminated` (`stop.go:127`) nils the cell's actor on termination | none (review) | 3a |
| X11 | Exports are justified (section "Open API choices") and the README says what exists (registration only) and what is planned (#870, #905, #1056) | review | module README contract | none (documentation) | 4 |

Every sentence of the PRs that says something was observed names the oracle line or the assertion (rule of 2026-11-06).

## Slices (one PR each, at most 400 authored lines, sequential)

The raw estimate is the TL's; the second number is the raw one times 1.55, the deviation measured in #976. The TL wrote no prototype for this unit: if a slice measures above 400 the Dev splits it before opening the PR.

| Slice | Delivers | Files | Criteria | Est. lines (raw → re-estimated) |
|---|---|---|---|---|
| **0. Kit package** | manifest, RULEBOOK, RUNBOOK | `migration/stage-1/eventstream-registration/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | kit run | ~140 docs |
| **0b. Oracle addendum EV4f** | the scenario EV4f on Pekko 1.7.1 (fresh stream: `z` on `AnyRef`, then `Base`, return values and deliveries; plus the same with `Base` registered first), harness, trimmed logs and a section in the oracle, as EV1t did | `docs/parity/harness/...`, `docs/parity/oracles/eventstream-spec-2026-11-06.md`, `.../eventstream-spec-2026-11-06.logs/` | X7b | ~100 docs and logs |
| **1. Index port** | `Index[K, V]` (`AddKey`, `AddValue`, `RemoveValue(key, value)`, `RemoveValue(value)`, `FindValues`, `FindSubKeys`), `Subclassification[K]`, with "Ported from" headers for `util/SubclassifiedIndex.scala` | `pekko-actor/internal/classification/index.go`, `index_test.go` | X1, X2, X4a, X12 (1a); X3, X4b (1b) | production ~170 + tests ~200 → ~370 (second number ~575: **the Dev splits it in 1a, `AddKey`/`AddValue`/`FindValues` + X1, X2, X4a, X12; 1b, removal + X3, X4b**) |
| **2. Channel relation** | `Subclassification[reflect.Type]` implemented as in X5 | `pekko-actor/internal/classification/channel.go`, `channel_test.go` | X5 | ~45 + ~100 → ~225 |
| **3a. Errors and the entry point** | `ErrNilSubscriber`, `ErrChannelNotAccepted`, `Subscribe` with the nil and assignability checks, the unexported `eventStream` shell and the field in `System` | `pekko-actor/actor/eventstream.go`, `eventstream_errors_test.go`, `pekko-actor/actor/system.go` (one field and its initialization only) | X8a, X9, X10 | ~70 + ~90 → ~250 |
| **3b. Index wiring** | `Subscribe` calls the index under the lock; entries by `id`; `deliver` closure | `pekko-actor/actor/eventstream.go`, `eventstream_test.go` | X6, X7a, X7b, X8b, X13 | ~60 + ~120 → ~280 |
| **4. Docs and record** | README of `pekko-actor` (registration implemented; publication, unsubscribe and termination planned), `EXECUTION-RECORD.md`, cost-log row, the "Open API choices" record linked from #1020 | `pekko-actor/README.md`, `migration/stage-1/eventstream-registration/EXECUTION-RECORD.md`, `migration/cost-log.tsv` | X11 | ~120 docs |

Order: 0, 0b, 1(a, b), 2, 3a, 3b, 4. `system.go` is also edited by #870 later (sequenced by the story chain); `deadletter.go` and the hub are not touched here. Internal test files (`<file>_internal_test.go`, same package) are allowed in any slice with the same budget.

## Open API choices (ADR 0011 owner decision 2, recorded here per O-2 b; the index #1020 links this section)

| Candidate export | Consumers | Concrete benefit (ADR 0011 criterion) | Alternative considered |
|---|---|---|---|
| `actor.Subscribe[E, M any]` | `pekko-actor-typed` (#959: `EventStream.Subscribe` commands), the testkit probes (M-03), `pekko-persistence` tests (migration #1426) | Typed and the testkit cannot reach the stream otherwise; ADR 0036 EVT-1 point 2 puts the stream in `pekko-actor`. A function, not a method, because Go has no generic methods and `E` must be explicit | a method on an exported `*EventStream`: needs `E` as a type argument anyway, so it would still be a function; exporting the type adds surface for no benefit |
| `actor.ErrNilSubscriber`, `actor.ErrChannelNotAccepted` | callers that distinguish the rejection; tests | the errors of ADR 0038 D4 and D-2; `errors.Is` is the way a Go caller tells them apart | unexported errors: callers could not branch on them |
| `eventStream`, `Index`, `Subclassification`, the channel relation | none outside `pekko-actor` | **not exported** | — |

This section approves no export beyond these; each later story (#870 `Publish`, #905 `Unsubscribe`) records its own.

## Decisions for the Owner

- **D-1. Kit run required** (a port). Alternative: waive it; not proposed (the unit ports 236 lines of Scala).
- **D-2. The signature `Subscribe[E, M any](s *System, subscriber Ref[M])`, approved by the Owner.** Ground (the Owner's): in Pekko Typed `ActorRef[-T]` is contravariant (`ActorRef.scala:35`), so an `ActorRef[Iface]` subscribes directly to the channel `Impl`; `Subscribe[E, M any]` with `E` assignable to `M` is the Go equivalent of that contravariance. The check is at run time because Go cannot express that constraint between two type parameters, so a violation is `ErrChannelNotAccepted` (ADR 0038 D4: errors, not panics). It is not the second entry point with a conversion function that O-6 rejected: there is one entry point and no conversion. **O-6 keeps governing a `Ref` whose message type does not accept `E`: that one needs the adapter of [#959](https://github.com/getsyntegrity/gopekko/issues/959).** Consequence of the one-parameter alternative, not taken: one subscriber on several channels (EV4's `x`, the variants v1 to v3 of EV1t, "one copy per subscriber") would be unreachable from the public API. Owner's literal text for the Owner table of the amendment of ADR 0038 that records this (merged **before slice 3a**, own PR, own review): "si — apruebo Subscribe[E, M any](s, Ref[M]) con E asignable a M, como equivalente en Go de la contravarianza de ActorRef en Pekko".
- **D-3. Interface relation, approved by the Owner.** As X5: equal types are sub-classes of each other (the Scala contract), and otherwise `x` is a sub-channel of `y` when `x != y`, `y` is an interface, `x.Implements(y)` and it is not the case that `x` is an interface with `y.Implements(x)`; so two interface types with **identical method sets** are *unrelated*. Ground (the Owner's): Scala types **nominally** (two traits with the same members have no relation), and the unrelated reading avoids the cycle in the index (the antisymmetry the `Subclassification` contract needs); Go's relation is structural (any type with the methods implements the interface), and ADR 0038 D2 already maps traits to interfaces. **Scope of the effect:** it concerns only the sub-channel cascade (what `Unsubscribe` removes and what a new node inherits, hence some "no change" results); it does **not** change which subscribers an event is delivered to, since an event's concrete type implements both interfaces and reaches the subscribers of each. Consequence: `type Empty interface{}` is not related to `any`. Alternative, not taken: reject the registration of such an interface.
- **D-4. Identity.** A subscriber is identified by `Handle.ID()` and any `Ref[M]` implementation is accepted, native or foreign (an ask endpoint, a Typed adapter): equal IDs are the same subscriber. Alternative: accept only native `Ref`s (reject foreign ones with `ErrChannelNotAccepted`-like error); not proposed, because the Typed adapter (#959) and the ask endpoint are `Ref`s.
- **D-5. A stopping or stopped `System`.** Not decided by Pekko's source for the stream and not stated by the repository (`SubscribeDeadLetters` does not refuse it either): per kit rule 8 ("unknown is an answer") `Subscribe` does **not** refuse it, and leaves the greppable marker `TODO(EVT-02-stopping)` for #1056 to decide with the termination work. Pekko's `EventStream.subscribe` first registers the subscriber with the unsubscriber (`EventStream.scala:60`, `registerWithUnsubscriber`), which is #1056's part; what Pekko does on a terminated system is not studied by the oracle (its Limits). Alternative: refuse with `ErrSystemStopping` as `Spawn` does; not proposed without an oracle scenario. **Approved by the Owner, with a handoff:** the oracle of [#1056](https://github.com/getsyntegrity/gopekko/issues/1056) must cover `Subscribe` on a System that is stopping (recorded in an Update of #1056).
- **D-6. The exports** of "Open API choices" above, **approved by the Owner**, with the justification entry of each export in this section and in the index [#1020](https://github.com/getsyntegrity/gopekko/issues/1020) (an Update of #1020 links this section now; slice 4 keeps the link current).
- **D-7. #836 to `status:ready`** when this scope is approved, and the order of the unit's PRs (0, 0b, 1a, 1b, 2, 3a, 3b, 4) as above. Alternative: keep `status:blocked` until slice 0b has observed EV4f; not proposed (0b is the second PR).

## Not carried here (each with its owner)

| Item | Owner |
|---|---|
| `Publish` and the delivery by mailbox, the publish cache, bounded overflow, `SubscribeDeadLetters` deprecation | [#870](https://github.com/getsyntegrity/gopekko/issues/870) |
| `Unsubscribe` (one channel, all) and its linearization, tested through the API | [#905](https://github.com/getsyntegrity/gopekko/issues/905) (the index-level removal is here, X3, X4) |
| Removal of a subscriber that stops | [#1056](https://github.com/getsyntegrity/gopekko/issues/1056) |
| Event types `SuppressedDeadLetter`, `Dropped`, the sender of `DeadLetter` | [#871](https://github.com/getsyntegrity/gopekko/issues/871), [#906](https://github.com/getsyntegrity/gopekko/issues/906), [#1057](https://github.com/getsyntegrity/gopekko/issues/1057) |
| Migration of the 42 test files from `SubscribeDeadLetters` | [#1426](https://github.com/getsyntegrity/gopekko/issues/1426) |
| Typed command surface and the adapter for a `Ref` of another message type | [#959](https://github.com/getsyntegrity/gopekko/issues/959) |
| The reusable subclassified bus ([#891](https://github.com/getsyntegrity/gopekko/issues/891), EVT-12; the reconciliation of #836 requires this boundary to be stated) | #891 exports a general bus; this unit covers exact and interface matching on the System stream only, and its `internal/classification` index is what #891 may later export (it is not exported here) |
| Diamond hierarchies (a type with two unrelated parents) beyond the unit tests of the index; `ActorEventBus` outside the stream | not studied by the oracle (its Limits); the index port supports multiple parents as `SubclassifiedIndex` does |

## Allowed files

Exactly the paths in the slice table. Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `pekko-actor/internal/deadletter/` or any other module changes; `scripts/deps.allow` is not touched (`actor` imports no other goPekko module). A needed change outside this list is a Mandatory Stop and goes back to the TL.

| Pair | Shared file | Resolution |
|---|---|---|
| 0, 0b, 2, 4 | none | each owns new files; `README.md` only in 4; `migration/cost-log.tsv` is shared across stages and only slice 4 appends its row |
| 1a, 1b | `internal/classification/index.go`, `index_test.go` | sequenced (1a first) |
| 3a, 3b | `actor/eventstream.go` | sequenced (3a first) |
| 1, 2 | `internal/classification/` (different files) | sequenced (1 first); 2 depends on 1's `Subclassification` |
| 3a | `actor/system.go` (one field) | sequenced after 2; also edited by #870 (the chain) |

## Dependencies

- ADR 0038 (Accepted) and the merged oracle. [#1020](https://github.com/getsyntegrity/gopekko/issues/1020) is the index of the export records (this scope's "Open API choices" is linked there in slice 4); it is not a prerequisite of the code.
- Nothing else: the unit consumes `Ref`, `Handle`, `ID` and `System` as they are on `develop`.

## Appetite

Eight small PRs (kit package, oracle addendum, index in two, relation, `Subscribe` in two, docs). The risk is the index port: `SubclassifiedIndex` keeps one value set per node and inherits it on `AddKey`, and its removal semantics (X3) are exactly the surprising behavior the oracle recorded (EV1tv3), so the index tests are written from the oracle lines before the code.

## Definition of Done

- X1 to X13 have their tests and pass, except X7b, which stays **pending** until slice 0b has observed EV4f on Pekko (ADR 0033 D13); X11 is a review; the PRs name the mutations in a "Mutations" table, reproduced by QA (unmutated green, mutated red).
- `scripts/each_module.sh go test -count=1 ./...`, `scripts/check_deps.sh`, the complexity lint (gocyclo ≤ 10) and the coverage gate (85 % per module; diff coverage ≥ 85 %) with the commands of `AGENTS.md`; `-count=100` on the concurrent tests; no `-race`.
- Every ported file carries the Apache-2.0 header and exactly one `// Ported from upstream/pekko/actor/src/main/scala/org/apache/pekko/util/SubclassifiedIndex.scala` line (`check-headers.sh`); the kit's RULEBOOK §6 applies to files that mix specs.
- The PRs say `Refs #836`, never `Closes`; the issue is closed by hand with its closure Update after slice 4.
- Every sentence that says something is observed names the oracle line or the assertion; every Pekko citation is verified by the Dev; the evidence is a comment bound to the final head SHA with real exit codes.
