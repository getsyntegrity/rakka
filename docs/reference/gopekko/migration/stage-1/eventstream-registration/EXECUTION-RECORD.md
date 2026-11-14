# EXECUTION RECORD: Classified event subscriptions on the EventStream (#836, EVT-02)

Scope `docs/scopes/836-eventstream-registration.md` (PR #1428, approved at gate G2 by the Owner on 2026-11-06, D-1 to D-7). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`), oracle `docs/parity/oracles/eventstream-spec-2026-11-06.md`. Delivered in `develop` by the PRs of section 7, all merged on 2026-11-07 UTC between 01:49Z and 02:55Z (`gh pr view <n> --json mergedAt`; the scope was approved on 2026-11-06 local time). Every PR says `Refs #836`, none `Closes`: #836 stays open for the TL's closure by hand.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`). The cost-log row counts **22 runs**, which is the orchestrator's count of the Dev and QA sessions of slices 0 to 3b plus slice 4, taken from the TL's conversation. It is not derivable from GitHub, so a QA can check the arithmetic but not the source. **The 22 excludes** the QA runs on the scope PR #1428 (3 comments) and on the ADR amendment #1429 (1 comment), the TL's survey agent, and the oracle and TL documents. Composition: 10 Dev runs (slice 0: 1; 0b: 1; slices 1a-i, 1a-ii, 1b, 1b-ii and the first stopped run of slice 1a: 5 runs of the same Dev session resumed; slice 2: 1; slices 3a and 3b: 2) + 10 QA runs (0: 1; 0b: 2; 1a-i: 1; 1a-ii: 1; 1b: 1; 1b-ii: 1; 2: 1; 3a: 1; 3b: 1) + 2 for this slice (its Dev and its QA) = 22. Tokens and wall clock were not measured (`unknown`).
2. Inputs: `SubclassifiedIndex.scala`, `EventBus.scala` (159-163), `EventStream.scala` (56-62), the oracle (EV1, EV1t, EV1tv2, EV1tv3, EV2, EV3, EV4, EV4f), ADR 0038 and its Amendment (a), ADR 0036 EVT-1, ADR 0033 D13.
3. Outputs: `Subscribe`, `ErrNilSubscriber`, `ErrChannelNotAccepted` and the unexported `eventStream` (`actor/eventstream.go`, one field in `system.go`), `internal/classification` (`index.go`, `channel.go`), their tests, the kit package, the EV4f oracle section, the README section "EventStream registration" and this record.
4. Deviations: the 3a/3b split was adjusted (section 5); slice 1a was split by the TL (section 5).

## 2. Criteria and mutations

Mutations were applied in disposable worktrees, never committed; the per-PR "Mutations" tables are in the PR bodies and the QA reproductions in the PR threads (`gh pr view <n> --comments`).

| Criterion | Test | Mutation | PR |
|---|---|---|---|
| X1 | `TestIndexAddValueReportsChangesOnce` | `AddValue` always returns a non-empty set | #1432 |
| X2 | `TestIndexSubKeyInheritsFromSuperKey` | `AddKey` starts the node with an empty set | #1432 |
| X3 | `TestIndexRemoveValueEv1tVariants` | `RemoveValue` removes only at the exact key | #1435 |
| X4a | `TestIndexFindValues` | `FindValues` returns the parent's set | #1433 |
| X4b | `TestIndexRemoveValueEverywhere` | `RemoveValue(value)` stops after the first key | #1435 |
| X5 | `TestChannelRelation` | M1 drops the NOT clause, M2 drops `x == y` | #1434 |
| X6 | `TestSubscribeRepeatIsNoChange` | `changed = true` always | #1438 (minimal), #1441 |
| X7a | `TestSubscribeCoveredByExistingNodeIsNoChange` | `changed = true` always | #1441 |
| X7b | `TestSubscribeFreshStreamNewNodeIsChange`; **observed** by EV4f [002] (oracle #1431) | `changed` false for a new node | #1441 |
| X8a | `TestSubscribeRejectsUnacceptableChannel` | `AssignableTo` check replaced by `false` | #1438 |
| X8b | `TestSubscribeSeveralChannels` | `AddValue` with the channel of `M` instead of `E` | #1441 |
| X9 | `TestSubscribeRejectsNil` | `isNilValue` replaced by `false` | #1438 |
| X10 | review (no test): the stream holds a delivery closure and the `ID`, never the `Actor` | none | #1438 |
| X11 | review: README and this record | none | this PR |
| X12 | `TestIndexValuesAreASet` | values kept in a slice | #1433 |
| X13 | `TestSubscribeConcurrentSamePairOneChange` | remove the lock | #1441 |
| W1 (QA of #1438) | `TestSubscribeStoresADeliveryClosure` | the closure does not `Tell` | #1441 |

**Surviving mutations** (all taken from the QA comments, none left unrecorded): (a) #1432, `excepts(...)` removed from `innerFindSubKeys`: judged equivalent in reachable paths by the QA; (b) #1432, the `IsEqual` branch of `innerAddKey`: closed by the `AddKey` on a known key test of #1433; (c) #1435 W1, the node created by `innerRemoveValue` starting with an empty set: closed by #1436; (d) #1441, the QA's own mutation "register `E` before returning `ErrChannelNotAccepted`": **still survives**, because `eventstream_errors_test.go:49-60` checks "registers nothing" indirectly through another channel (`evImpl`). The X13 mutation (no lock) fails by a Go fatal `concurrent map writes`, accepted by the QA of #1441 (the probabilistic assertion `eventstream_test.go:144` also fired once).

## 3. Gates

Per code or test PR, on the pushed head: `gofmt -l`, `go vet`, `go test -count=1 -covermode=atomic` (module `pekko-actor` 98.3% on #1438 and #1441), `-count=100` of the new tests, `golangci-lint` v2.14.0 (0 issues), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh` (7 errors, identical on `origin/develop`: `pekko-actor/actor/names.go` and six `pekko-persistence` files; none in this unit), diff coverage by `diff-cover` 11.0.0. `unshare` does not exist on macOS, so **no unit run of this unit had network isolation**. This slice has no code; its evidence is the Evidence comment of the PR.

## 4. Evidence class and pending approval (ADR 0033 D13)

- **ORACLE, executed on Pekko 1.7.1:** EV1 [001], [002], EV2 [001], EV3, EV4 [001]-[010], EV4f [002] (#1431), EV1t/EV1tv2/EV1tv3 (removal).
- **READ, not run:** `SubclassifiedIndex.scala` (no Pekko spec exists), `EventBus.scala:159-163`, `EventStream.scala:56-62`.
- **No Pekko counterpart:** the assignability check (D-2), the structural channel relation (D-3), identity by `Handle.ID()` (D-4).
- **Not claimed:** V3, parity of the stream (nothing is published), behavior on a stopping System.

## 5. Stops, decisions and adjustments

- **Slice 1a measured 471 lines** before review and was split by the TL into 1a-i and 1a-ii (#1432 399 lines, #1433 78); 1b also has a follow-up (#1436).
- **3a/3b split adjusted:** `Subscribe` is exported only when complete, never as an interim stub, so #1438 carries the whole production code and #1441 is tests only.
- **D-2 and D-3** are the Owner's, with his grounds in the scope (contravariance of `ActorRef[-T]`; Scala types nominally, Go structurally, equivalent interfaces unrelated). D-2 is recorded by the ADR 0038 Amendment (a), #1429.
- **D-5:** a stopping System is not refused; `TODO(EVT-02-stopping)` stays in `Subscribe`, handed to #1056 (oracle scenario EV12).

## 6. QA findings that changed the code or the scope (from the PR threads)

- **#1428 (scope): REJECT, then APPROVE.** CRITICAL: X7 said `Subscribe[any]` then `Subscribe[Base]` for the same subscriber reports `false`, citing EV4 [007], which observes a different case (the node already exists). The Scala code (`SubclassifiedIndex.scala:137-150`) reports the new node. The scope was fixed (X7a covered node, X7b fresh stream) and X7b became pending on EV4f, which #1431 observed. Also fixed: the `Subclassification` reflexivity wording of X5/D-3.
- **#1430 (kit package): APPROVE** with W1: the manifest cited `Nonroot :38-61`, but `Nonroot` runs 38-73 in v1.7.1. Separately, the manifest cites `EventBus.scala:159-163` for subscribe where the scope's prose said 159-175 (the larger range also covers unsubscribe code; the Dev of this slice did not re-open either, and the manifest range is the one used).
- **#1431 (oracle): REJECT, then APPROVE** after a wrong Pekko citation was fixed.
- **#1432: APPROVE** with W1 (no test of `AddKey` on a known key, closed by #1433). **#1433, #1434, #1436: APPROVE** with wording suggestions. **#1435: APPROVE** with W1 (closed by #1436) and the note that `RemoveValue` returns the live internal set (documented).
- **#1438: APPROVE** with three WARNINGs: the delivery closure body was never executed (W1, closed in #1441); the first closure per ID is kept (documented as a known limit in #1441); and `deliver` and the index only grow until #905 and #1056 (documented in the comment of `eventStream` in #1441 and in the README).
- **#1441: APPROVE** with SUGGESTION 1 (the surviving mutation (d) above).

## 7. PRs and merge commits (`gh pr view <n> --json mergeCommit,additions,deletions,mergedAt`)

| PR | What | Merge commit | Lines |
|---|---|---|---|
| #1428 | scope | `b14e7950` | 142 |
| #1429 | ADR 0038 Amendment (a) | `1735b125` | 21 |
| #1430 | slice 0, kit package | `8d0bb04b` | 103 |
| #1431 | slice 0b, oracle EV4f | `b038e78c` | 91 |
| #1432 | slice 1a-i, `Index` port: `AddKey`, `AddValue`, `FindValues` | `2b9ab5f8` | 399 |
| #1433 | slice 1a-ii, tests: `FindValues`, set values, known `AddKey` | `57d76f30` | 78 |
| #1434 | slice 2, `Channels` relation | `c767564d` | 143 |
| #1435 | slice 1b, removal by key and by value | `ad1627f1` | 153 |
| #1436 | slice 1b-ii, removal tests and the aliasing note | `f95458cb` | 25 |
| #1438 | slice 3a, `Subscribe` complete (production, error tests) | `9725e542` | 182 |
| #1441 | slice 3b, registration, several channels, concurrency tests | `22231837` | 223 |
| this PR | slice 4: README, record, cost-log row | see `git log` | see its description |

All are at or under the 400 budget; #1432 sits at 399 after the split.

## 8. Known gaps and follow-ups

- Nothing publishes (#870), nothing unsubscribes (#905), a stopped subscriber is not removed (#1056): the index and the closure map only grow.
- A stopping System is not refused (`TODO(EVT-02-stopping)`, #1056). The first delivery closure per ID is kept; the decision for #870 is recorded in its scope.
- Surviving mutation (d) of section 2: assert "registers nothing" through a covering channel or the index state.
- `unshare` is not available on macOS; `check-headers.sh` has 7 pre-existing errors on `develop`.
- `SubscribeDeadLetters` and the hub are replaced in stages (#1426); diamond hierarchies and the reusable bus (#891) are outside this unit.

## 9. Recurring failures (`AGENTS.md`, kit rule 6)

A scope or oracle citation that claimed more than its source showed appeared in #1428 (X7), #1430 (W1, the `Nonroot` range) and #1431 (a wrong citation): three occurrences. A candidate rule, for the TL to accept or drop: each cited Pekko line range is re-opened and its first and last line quoted in the manifest `note`. This is a proposal, not an observed fact about the cause.

## 10. Note of 2026-11-07: network isolation

No unit run of this unit had network isolation: `unshare -rn` (the Linux variant of `AGENTS.md`) does not exist on macOS and the container variant was not run. Only the evidence of #1430 and #1432 mentions it; the other PRs of the unit are silent about it, which is the gap this note closes. From 2026-11-07 the evidence states `sin aislamiento de red (macOS)` for that run instead of treating it as done (Owner's decision), and the Owner performs the real isolation at the close of each module with the container variant of `AGENTS.md`.
