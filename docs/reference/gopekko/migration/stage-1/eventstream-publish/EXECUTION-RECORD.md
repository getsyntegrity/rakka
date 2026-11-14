# EXECUTION RECORD: Publish events through the subscribed actors' mailboxes (#870, EVT-03)

Scope `docs/scopes/870-eventstream-publish.md` (PR #1437, approved at gate G2 by the Owner on 2026-11-07, D-1 to D-10). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`), oracle `docs/parity/oracles/eventstream-spec-2026-11-06.md`. Delivered in `develop` by the PRs of section 7, all merged on 2026-11-07 UTC between 03:23Z and 07:12Z (`gh pr view <n> --json mergedAt`). Every PR says `Refs #870`, none `Closes`: #870 stays open for the TL's closure by hand.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`). The cost-log row counts **16 runs**, the orchestrator's count of the Dev and QA sessions of slices 0 to 3, taken from the TL's conversation. It is not derivable from GitHub, so a QA can check the arithmetic but not the source. Dev = slice 0 (1), 0b (2: the oracle and the fix after the QA), 1a (2: the feature and the fix after the QA), 1b (1), 2 (1), 3 (1) = 8. QA = slice 0 (1), 0b (2: review and delta), 1a (2: review and delta), 1b (1), 2 (1), 3 (1) = 8. **The 16 excludes** the QA runs on the scope PR #1437 and on the amendment PR #1449, and the TL's own documents. Tokens and wall clock were not measured (`unknown`).
   The final heads of the Dev sessions of the slices with two Dev runs, so the count can be checked: slice 0b is `181ddab3e885a4284bf2d728e329da55049f8670` (head of #1445) and slice 1a is `6c27693df382c150068cc915e1bbdb917b50c8de` (head of #1446).
2. Inputs: `EventBus.scala` (159-163, 177-190, 206-209), `EventStream.scala` (44-62), `Mailbox.scala:560-568`, `ActorRef.scala` (739-743, 862-866), the oracle (EV3, EV4, EV6, EV7, EV8, EV11 to EV11v4), ADR 0038 and its Amendments (a) and (b), ADR 0033 D13/D14/D15.
3. Outputs: `Publish`, `ErrNilEvent` and the publish cache (`actor/eventstream.go`), `DeadLetter` events on the stream and the deprecation (`actor/deadletter.go`), five test files, the kit package, the oracle section EV11, the README section "EventStream: registration and publication" and this record.
4. Rules of 2026-11-07 applied by this unit: every cited Pekko range is re-opened before the PR and listed as `path:lines -> first line` in the Evidence comment; `git fetch` only in the Dev's own worktree, never touching refs of the main checkout; the evidence says `sin aislamiento de red (macOS)` instead of an `unshare` claim.

## 2. Criteria and mutations

Mutations ran in disposable worktrees, never committed; the per-PR "Mutations" tables are in the PR bodies and the QA reproductions in the PR threads (`gh pr view <n> --comments`).

| Criterion | Test | Mutation | PR |
|---|---|---|---|
| P1 | `TestPublishMatchingSubscribersOnce` | deliver once per matching registration | #1446 |
| P2 | `TestPublishUnmatchedIsNotADeadLetter` | publish an unmatched event as a `DeadLetter` | #1446 |
| P3 | `TestPublishRejectsNilEvent` | remove the nil check | #1446 |
| P4 | `TestPublishSeesSubscriptionsMadeAfterTheFirstPublication` | `Subscribe` does not update the cache | #1446 |
| P5 | `TestPublishDoesNotBlockOnASlowSubscriber` | `select {}` after the delivery loop (fails after the 30 s failure bound of `await`) | #1447 |
| P6 | `TestPublishToAFullOrStoppedSubscriberIsADeadLetter` | skip the dead-letter publication in the full branch, and in the stopped branch, of `tell` | #1447 |
| P7 | `TestPublishConcurrentWithSubscribe` | remove the lock of `Subscribe` (10 of 10 invocations ended in `fatal error: concurrent map writes`) | #1447 |
| re-entrancy (QA of #1446) | `TestPublishDeliversOutsideTheStreamLock`, `TestDeadLetterPublishIsReentrantThroughTheStream` | deliver, or publish to the stream, holding the stream lock | #1447, #1448 |
| P8 | `TestDeadLettersAreEventsOnTheStream` | `System.publish` feeds only the hub | #1448 |
| P9 | `TestDeadLetterDeliveryFailureDoesNotLoop` | remove the nested-`DeadLetter` guard | #1448 |
| P10 | review (`git grep SubscribeDeadLetters -- '*.go' ':!*_test.go'`) | none | #1448 |
| P11 | review: README and this record | none | this PR |

**The P9 mutation ends in Go's fatal stack overflow** (`runtime: goroutine stack exceeds 1000000000-byte limit`, `fatal error: stack overflow`, 1.0 s of test time, no hang; #1448 body). It is accepted **only because mutations are run by hand** in a disposable worktree (the Owner's condition on D-8): with the guard in place the test passes quickly without a crash. The P7 mutation also ends in a fatal crash, accepted for mutations only.

**Survivors and weak spots** (from the QA threads): in #1446 the double-check under the lock and the mutation "deliver under the lock" survived; the second was closed by the re-entrancy tests of #1447. The cache's safety net (the channel absent from the diff is filled from `index.FindValues`) is unreachable with the current index and untested; diff coverage of #1446 was 91% for that reason. In #1448 the mutation "stream before hub" survives by design: the order is ours and no criterion promises it. The P7 mutation is probabilistic by nature.

## 3. Gates

Per code or test PR, on the pushed head: `gofmt -l`, `go vet`, `go test -count=1 -covermode=atomic` (module `pekko-actor` 98.2% to 98.3%), `-count=100` of the new tests, `golangci-lint` v2.14.0 (0 issues for `pekko-actor` only: with `.golangci.yml` (`tests: true`) the deprecation of `SubscribeDeadLetters` raises 6 SA1019 findings in test files of `pekko-actor-typed` (3) and `pekko-persistence` (3), as the README says; they disappear with #1426), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh` (7 errors identical on `origin/develop`, none in this unit), diff coverage by `diff-cover` 11.0.0 (91% on #1446, none to measure on #1447 which has no production line, 100% on #1448). `sin aislamiento de red (macOS)`. This slice has no code; its evidence is the Evidence comment of its PR.

## 4. Evidence class and pending approval (ADR 0033 D13)

- **ORACLE, executed on Pekko 1.7.1:** EV3 [002]-[005], [024], [033]-[040]; EV4 [008]-[010]; EV6 [003], [006]; EV7 [049], [354]; EV8 [006], [008]; EV11 [010] and EV11v4 [013] (the cycle).
- **Inferred:** EV3 [042] (`publish(null)` throws), from `null.getClass`.
- **READ, not run:** `EventBus.scala:177-190, 206-209`; `ActorRef.scala:862-866` (half 1 of the cycle).
- **No Pekko counterpart / deviation:** the provisional rule of P9 (known gap, section 8).
- **Not claimed:** V3, parity of the cycle, behavior on a stopping System.

## 5. Stops, decisions and adjustments

- **D-3, the cycle.** The scope predicted by code reading that Pekko recurses; the oracle EV11 observed it (EV11 [010], EV11v4 [013]: up to 1001 nested in run 1, 977 in run 2). The Owner decided on 2026-11-07 for the conservative rule, then narrowed that decision to the second half after the QA found [akka-core #31241](https://github.com/akka/akka-core/issues/31241), which explains half 1. ADR 0038 Amendment (2026-11-07, b) (#1449) records both halves, with the fatal path owned by #1345.
- **D-8:** the mutation of P9 fails by a fatal stack overflow, accepted only because mutations are run by hand (the Owner's condition).
- **Order hub then stream** (slice 2) is the Dev's choice, documented on `System.publish`; the scope promises no order. The Dev read the eight call sites of `System.publish`: none holds a lock the stream or `Tell` path needs, so there was no Mandatory Stop.

## 6. QA findings that changed the code or the docs (from the PR threads)

- **#1445 (oracle EV11):** claims marked Observed that no log line backed were moved to Inferred.
- **#1446 (slice 1a): APPROVE** with one WARNING: the empty-channel fallback of the cache was unreachable and, if reached, would have cached the channel empty for ever. It was replaced by a fill from `index.FindValues` (commit `6c27693d`), with a comment saying it is a safety net, and the O(channels) copy cost of `Subscribe` was documented.
- **#1449 (amendment):** the first D15 search for an acknowledged bug missed akka-core #31241; the QA found it and the amendment was corrected and narrowed to half 2.
- **#1447: APPROVE.** The QA found that the P7 mutation test is probabilistic: the narrow mutation (only the lock around the cache update) survives `-count=10` and fails at `-count=300`; the PR keeps the broad mutation (the whole lock) as the reproducible one.
- **#1448: APPROVE** with two findings. The D-3 deviation (no republication of a `DeadLetter` whose `Msg` is a `DeadLetter`) had no ADR of its own, which is the origin of ADR 0038 Amendment (2026-11-07, b) (#1449; the amendment is credited to this QA as well as to the QA of #1449). The PR body said `lint 0 issues` where it held only for `pekko-actor`; it was corrected in the PR body (SA1019 in `pekko-actor-typed` and `pekko-persistence`).
- The QA of #1446 also asked for the re-entrancy tests, delivered in #1447 and #1448.

## 7. PRs and merge commits (`gh pr view <n> --json mergeCommit,additions,deletions,mergedAt`)

| PR | What | Merge commit | Lines | Merged (UTC) |
|---|---|---|---|---|
| #1437 | scope | `95cd9c71` | 129 | 2026-11-07T03:23Z |
| #1444 | slice 0, kit package | `2aeba5e7` | 91 | 2026-11-07T03:29Z |
| #1445 | slice 0b, oracle EV11 | `8236b5af` | 236 | 2026-11-07T03:39Z |
| #1446 | slice 1a, `Publish`, `ErrNilEvent`, the cache | `4b7b5819` | 294 | 2026-11-07T03:52Z |
| #1447 | slice 1b, delivery tests (slow, full, stopped, concurrent, re-entrancy) | `175406f0` | 246 | 2026-11-07T04:04Z |
| #1448 | slice 2, `DeadLetter` events, the cycle guard, the deprecation | `19539c22` | 238 | 2026-11-07T04:16Z |
| #1449 | ADR 0038 Amendment (b) (TL docs PR, for traceability) | `e43360d4` | 25 | 2026-11-07T07:12Z |
| this PR | slice 3: README, record, cost-log row | see `git log` | see its description | |

All are at or under the 400 budget.

## 8. Known gaps and follow-ups

- **The `DeadLetter` cycle:** goPekko does not republish a `DeadLetter` whose `Msg` is a `DeadLetter` (hub callbacks only). This is a provisional rule and a known gap, not parity; its owner is #1345 (marker `TODO(EVT-03-deadletter-cycle)`).
- `Unsubscribe` (#905); removal of a stopped subscriber (#1056; until then each publication to it is a dead letter); `Subscribe` on a stopping System (#1056, `TODO(EVT-02-stopping)`).
- `SuppressedDeadLetter`, `Dropped`, the sender of `DeadLetter` (#871, #906, #1057); Typed commands (#959); migration of the test files and removal of the callback (#1426), after which the 6 SA1019 findings in test files of `pekko-actor-typed` and `pekko-persistence` disappear.
- The first delivery closure per ID is kept; a foreign `Ref` that changes `M` under one ID is unsupported (scope D-10).
- The cache safety net is untested because it is unreachable; `check-headers.sh` has 7 pre-existing errors on `develop`.

## 9. Recurring failures (`AGENTS.md`, kit rule 6)

None reaches three occurrences in this unit. The QA findings of section 6 (an unreachable fallback, unbacked Observed claims, an incomplete search) are one each.
