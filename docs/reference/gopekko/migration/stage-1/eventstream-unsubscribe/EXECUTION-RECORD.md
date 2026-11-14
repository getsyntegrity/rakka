# EXECUTION RECORD: Unsubscribe one classification or all registrations of an actor subscriber (#905, EVT-04)

Scope `docs/scopes/905-eventstream-unsubscribe.md` (PR #1439, approved at gate G2 by the Owner on 2026-11-07, D-1 to D-7). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`), oracle `docs/parity/oracles/eventstream-spec-2026-11-06.md`. Delivered in `develop` by the PRs of section 7, all merged on 2026-11-07 UTC between 07:21Z and 07:49Z (`gh pr view <n> --json mergedAt`). Every PR says `Refs #905`, none `Closes`: #905 stays open for the TL's closure by hand.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`). The cost-log row counts **10 runs**, the orchestrator's count of the Dev and QA sessions of slices 0 to 2, taken from the TL's conversation. It is not derivable from GitHub, so a QA can check the arithmetic but not the source. Dev = slice 0 (1), 0b (1), 1a (1), 1b (1), 2 (1) = 5. QA = slice 0 (1), 0b (1), 1a (1), 1b (1), 2 (1) = 5. **The 10 excludes** the QA runs on the scope PRs #1439 (four review comments: REJECT, delta REJECT, REJECT, APPROVE) and #1453 (one), and it counts no extra Dev session for the wording fixes after the QA of #1451, #1452 and #1453: the TL applied them itself. Cross-check against `gh pr view <n> --json comments`: #1451, #1452, #1454 and #1455 each carry exactly two comments, one Dev Evidence comment and one QA verdict, which is one Dev and one QA session per PR as counted; #1453 has one comment (a QA verdict, no Evidence comment, a TL docs PR). Tokens and wall clock were not measured (`unknown`).
2. Inputs: `EventBus.scala` (165-175, 192-204), `EventStream.scala` (64-80), the oracle (EV1, EV1t, EV1tv2, EV1tv3, EV2, EV4, EV5a/EV5b, EV10, EV13 to EV13v4), ADR 0038, ADR 0033 D13/D14.
3. Outputs: `Unsubscribe`, `UnsubscribeAll` and the cache update and closure cleanup (`actor/eventstream.go`), three test files, the kit package, the oracle section EV13, the README section "Unsubscription" and this record.
4. Rules of 2026-11-07 applied: every cited Pekko range is re-opened before the PR and listed as `path:lines -> first line` in the Evidence comment; `git fetch` only in the Dev's own worktree, never touching refs of the main checkout; the evidence says `sin aislamiento de red (macOS)`.

## 2. Criteria and mutations

Mutations ran in disposable worktrees by hand, never committed; the per-PR "Mutations" tables are in the PR bodies (#1454, #1455) and the QA reproductions in their threads.

| Criterion | Test | Mutation | PR |
|---|---|---|---|
| U1 | `TestUnsubscribeReturnValues` | return `true` always | #1454 |
| U1b | `TestUnsubscribeOfANeverRegisteredSubscriberCanReportTrue` | false when not registered; or true also when the node exists | #1454 |
| U2 | `TestUnsubscribeAllRemovesEveryRegistration` | the all-form assigns the removed set to the cache entry instead of subtracting | #1454 |
| U3 | `TestUnsubscribeRuleV1V2V3` | `removeValue` no longer cascades to the sub-keys | #1455 |
| U3b | `TestUnsubscribeOfAnUnrelatedChannelKeepsASharedSubChannel` | remove `a` everywhere; or `Unsubscribe` returns a constant | #1455 |
| U4 | `TestUnsubscribeKeepsOtherChannels` | `RemoveValueEverywhere` where `Unsubscribe` removes | #1454 |
| U5 | `TestUnsubscribeUpdatesTheCache` | the cache keeps the subscriber | #1454 |
| U6 | `TestUnsubscribeLinearization` | the cache is never updated (the subscriber receives e5) | #1455 |
| U7 | `TestUnsubscribeConcurrentWithPublish` | the lock removed in `Unsubscribe` and `UnsubscribeAll` | #1455 |
| U8 | `TestUnsubscribeRejectsNil` | the nil checks removed | #1454 |
| U9 | `TestUnsubscribeDropsTheDeliveryClosureWithTheLastRegistration`, `TestUnsubscribeKeepsTheClosureWhileARegistrationRemains` | the closure kept after the last registration; the closure dropped on any unsubscribe | #1454 |
| U10 | review: README and this record | none | this PR |

**Fatal-crash mutations.** The U7 mutation was caught in 10 of 10 invocations (5 at `-count=100`, 5 at `-count=300`, #1455 body), always by Go's fatal `concurrent map` crash and never by an assertion; it is accepted for mutations only, and the test passes at `-count=300` without a crash with the lock in place. The churn goroutines of that test are what make unlocked code race. The U8 mutation ends as a recovered nil-pointer panic that go-specs reports as a spec failure (#1454 body), not as a binary crash.

**Survivors and equivalents** (from the QA comments of #1454 and #1455): "the lock removed" was killed only by U7 in slice 1b, so it had no killer in #1454; "`Unsubscribe` purges the mailbox" and "`Publish` from a stale snapshot" are not expressible through the public API; "`UnsubscribeAll` drops the closure unconditionally" is equivalent, because after the all-form no registration of the ID remains; the unreachable safety net of the publish cache (from #870) remains untested.

## 3. Gates

Per code or test PR, on the pushed head: `gofmt -l`, `go vet`, `go test -count=1 -covermode=atomic` (module `pekko-actor` 98.32%), `-count=100` of the new tests (and `-count=300` for the concurrent one in #1455), `golangci-lint` v2.14.0 (0 issues for `pekko-actor`; the 6 SA1019 findings of the deprecated `SubscribeDeadLetters` in `pekko-actor-typed` and `pekko-persistence` remain until #1426), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh` (7 errors identical on `origin/develop`, none in this unit), diff coverage by `diff-cover` 11.0.0 (100% on #1454, none to measure on #1455). `sin aislamiento de red (macOS)`. This slice has no code; its evidence is the Evidence comment of its PR.

## 4. Evidence class and pending approval (ADR 0033 D13)

- **ORACLE, executed on Pekko 1.7.1:** EV1 [006]-[009], [012], [015], [018]; EV1t [005], [008]; EV1tv2 [005], [008]; EV1tv3 [006], [009]; EV5a/EV5b [004]-[010]; EV10 [004]; EV13 [002], [006], [007]; EV13v2 [002], [005], [006]; EV13v3 [008]-[010]; EV13v4 [002], [003].
- **READ, not run:** `EventBus.scala:165-175, 201-204`; `SubclassifiedIndex.scala:157-163, 222-228` (the explanation of why the Boolean depends on the nodes created, Inferred).
- **No Pekko counterpart:** the cleanup of the delivery closure (scope D-4).
- **Not claimed:** V3, the unwatch half of `unsubscribe`, behavior on a stopping System.

## 5. The finding that changed the plan, and decisions

- **EV13v2 refuted a prediction.** The TL's draft predicted that the Boolean of `Unsubscribe` for an unrelated channel is `false`. EV13v2 [005] observed `true` after the shared sub-class had been published, and EV13v4 [002] observed `false` when it had not; delivery was never stopped (EV13v2 [006]). The scope was corrected by #1453 (rows U1b and U3b) **before any production code**, as the Owner required.
- **D-1 to D-7** as approved at G2 on 2026-11-07: kit run; `Handle` instead of `Ref[M]` and two functions; the O-1 rule exposed as is; the closure cleanup as Go housekeeping; no public barrier (O-7); `status:ready`; the two exports. The Owner's condition, EV13 before any production code on U1b and U3b: **met** (#1452 merged at 07:28Z, #1454 at 07:39Z).

## 6. QA findings that changed code or docs (from the PR threads)

- **#1439 (scope): REJECT, REJECT, REJECT, then APPROVE** over four review comments (one stale citation was left in the U2 row at the third); fixed before the approval.
- **#1451 (kit package): APPROVE** with a WARNING: the manifest cited `EventBus.scala:165-169`, but the function ends at :171 (`diff.nonEmpty` is :170); corrected to 165-171.
- **#1452 (oracle EV13): APPROVE** with two wording WARNINGs: the EV13 intro said every variant publishes a control event, false for EV13v4; and the trimmed logs drop the `+ms` part of the raw `[nnn +ms]` prefix, which the addendum did not repeat. Both fixed in the text.
- **#1454: APPROVE.** **#1455: APPROVE**, noting that EV5b was missing from the Evidence list (the QA checked it); the QA's own mutations on #1454 (wrong channel type, union instead of replacement in the cache, `hasSubscriptions` scanning one channel) were killed.

## 7. PRs and merge commits (`gh pr view <n> --json mergeCommit,additions,deletions,mergedAt`)

| PR | What | Merge commit | Lines | Merged (UTC) |
|---|---|---|---|---|
| #1439 | scope (approved at G2) | `bb101987` | 123 | 2026-11-07T07:21Z |
| #1451 | slice 0, kit package | `ebc8e947` | 88 | 2026-11-07T07:26Z |
| #1452 | slice 0b, oracle EV13 | `0fce0c25` | 140 | 2026-11-07T07:28Z |
| #1453 | scope correction (U1b, U3b observed) | `776e2e2a` | 8 | 2026-11-07T07:31Z |
| #1454 | slice 1a, `Unsubscribe`, `UnsubscribeAll`, cache update, closure cleanup | `c22d3b7c` | 367 | 2026-11-07T07:39Z |
| #1455 | slice 1b, rule, linearization and concurrency tests | `b43c90b1` | 206 | 2026-11-07T07:49Z |
| this PR | slice 2: README, record, cost-log row | see `git log` | see its description | |

All are under the 400 budget.

## 8. Known gaps and follow-ups

- A stopped subscriber is still not removed: each publication to it is a dead letter until #1056. The unwatch half of `unsubscribe` (`EventStream.scala:67`, `:77`) and `Subscribe` on a stopping System (`TODO(EVT-02-stopping)`) are #1056.
- The `DeadLetter` cycle stays a known gap, owned by #1345.
- SA1019 on `SubscribeDeadLetters` in test files of `pekko-actor-typed` and `pekko-persistence` until #1426.
- The safety net of the publish cache is unreachable and untested.
- The "Open API choices" of the scope (`Unsubscribe`, `UnsubscribeAll`) is linked from the index #1020 by an Update the TL posts.

## 9. Recurring failures (`AGENTS.md`, kit rule 6)

A manifest citation range that stopped short of its claim appeared in #1439 (a stale citation in U2) and #1451 (165-169); evidence lists that omitted a scenario appeared in #1455 (EV5b). None reaches three occurrences, so no rule amendment is proposed.
