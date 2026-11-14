# EXECUTION RECORD: Remove the registrations of a subscriber that terminated (#1056, EVT-17)

Scope `docs/scopes/1056-eventstream-termination.md` (PR #1440, approved at gate G2 by the Owner on 2026-11-07, D-1 to D-7 with conditions on D-2 and D-3, D-8 rejected). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`), oracle `docs/parity/oracles/eventstream-spec-2026-11-06.md`. Delivered in `develop` by the PRs of section 7, all merged on 2026-11-07 UTC between 07:59Z and 12:13Z (`gh pr view <n> --json mergedAt`). Every PR says `Refs #1056`, none `Closes`: #1056 stays open for the TL's closure by hand.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`). The cost-log row counts **18 runs**, the orchestrator's count of the Dev and QA sessions, taken from the TL's conversation. It is not derivable from GitHub, so a QA can check the arithmetic but not the source. Dev = slice 0 kit (2, including the fix), 0b oracle (2, including the fix), 1a-i (3: the stop, the build, the fix), 1a-ii (1), 1b (1), 2 (1) = 10. QA = #1457 (2), #1458 (1), #1459 (2), #1460 (1), #1461 (1), this slice (1) = 8. **The 18 excludes** the QA of the scope PR #1440. Cross-check with `gh pr view <n> --json comments` (Evidence and QA comments together): #1457 has 4, #1458 has 3, #1459 has 4, #1460 has 2, #1461 has 2, and #1440 has 4 (excluded); the counts are consistent with the sessions above (a Dev Evidence comment and a QA verdict per round) but cannot confirm them. Tokens and wall clock were not measured (`unknown`).
2. Inputs: `EventStreamUnsubscriber.scala` (35-64), `EventStream.scala` (56-80), the oracle (EV7, EV12, EV12v2), ADR 0038 D6, ADR 0033 D13/D14.
3. Outputs: the stream's internal termination link (`actor/eventstream_watch.go`), the unwatch hook and the end of `TODO(EVT-02-stopping)` (`actor/eventstream.go`), five test files, the kit package, the oracle section EV12, the README section "Termination of a subscriber" and this record.
4. Rules of 2026-11-07 applied: every cited Pekko range is re-opened before the PR and listed as `path:lines -> first line` in the Evidence comment; `git fetch` only in the Dev's own worktree; the evidence says `sin aislamiento de red (macOS)`.

## 2. Criteria and mutations

Mutations ran in disposable worktrees by hand, never committed; the per-PR "Mutations" tables are in the PR bodies and the QA reproductions in their threads.

| Criterion | Test | Mutation | PR |
|---|---|---|---|
| T1 | `TestSubscriberTerminationRemovesItsRegistrations` | the watcher does not remove on termination: killed | #1459 |
| T2 | `TestSubscribeAnAlreadyStoppedActor` (a, b), `TestSubscribeALiveRefOfAnotherSystemIsKept` | (a) ignore the refusal of `addWatcher`, (b) nil record as not stopped, (c) `go func()` replaced by a plain call, removal without the stream lock, record looked up in the wrong System: all killed | #1459 |
| T3 | `TestOneWatcherPerSubscriber` | link ID minted per call, watch on every `Subscribe`, both together, one constant ID for every stream: killed (the first two were survivors of #1459, benign there) | #1460 |
| T4 | `TestTerminationDropsTheDeliveryClosure` | closure kept; cache not invalidated: killed | #1460 |
| T4b | `TestTerminationRemovesOnlyTheTerminatedSubscriber` | clear all subscribers on any termination: killed | #1460 |
| T5 | `TestUnsubscribeWithdrawsTheWatcherOnlyWhenNoChannelIsLeft` | always withdraw; never withdraw: killed | #1461 |
| T6 | `TestTerminationConcurrentWithSubscribe` | the callback skips the removal when the stream lock is busy: killed in 5 of 5 invocations at `-count=100` and 5 of 5 at `-count=300`, by assertion; probabilistic by nature | #1461 |
| T7 | `TestSubscribeOnAStoppingSystem` | `Subscribe` refuses with `ErrSystemStopping`: killed | #1461 |
| T8 | review: the foreign-`Ref` limit in the doc comment of `eventstream_watch.go`, in this record and in one README line | none | this PR |
| T9 | review: README and this record; no export | none | this PR |

**Not expressible or equivalent.** The unwatch running under the stream lock is covered by design (the call sits inside the locked `Unsubscribe`/`UnsubscribeAll`), not by a test; the QA of #1461 suggested it and it stays a gap. The `owner() == nil` guard of `watch` is reached only by a hand-built cell (covered by a case of `TestOneWatcherPerSubscriber`).

## 3. Gates

Per code or test PR, on the pushed head: `gofmt -l`, `go vet`, `go test -count=1 -covermode=atomic` (module `pekko-actor` 98.25% to 98.35%), `-count=100` of the new tests, `golangci-lint` v2.14.0 (0 issues for `pekko-actor`; the SA1019 findings of the deprecated `SubscribeDeadLetters` in test files of `pekko-actor-typed` and `pekko-persistence` remain until #1426), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh` (7 errors identical on `origin/develop`, none in this unit), diff coverage by `diff-cover` 11.0.0 (90% on the first head of #1459, 94% on its final head, 100% on #1460 and #1461). `sin aislamiento de red (macOS)`. This slice has no code; its evidence is the Evidence comment of its PR.

## 4. Evidence class and pending approval (ADR 0033 D13)

- **ORACLE, executed on Pekko 1.7.1:** EV7 [045], [047], [049], [351], [353]; EV12 and EV12v2 ([006], [007], [010], [011], [014]-[016], [019]-[022], [026], [073]-[084]); the numbers are those of run 1.
- **Inferred (the removal is not logged):** "removed shortly after" for an already terminated subscriber is the wording of ADR 0038 D6; EV7 [353] shows `true`, [396] the dead letters at 1 over 40 publications and [397] a second `subscribe`, from which the removal is Inferred; likewise the removal of the two victims of EV12 ([071] shows only 0 dead letters). Neither is asserted by a test as Observed.
- **READ, not run:** `EventStreamUnsubscriber.scala:39-47, 49-57, 59-64`; `EventStream.scala:56-62, 64-80`.
- **Deviations (ADR 0033 D14.3, minor):** D-2, no `/system` actor (the stream's own link replaces it); D-3, a foreign `Ref` is never retired automatically.
- **Not claimed:** V3, the cross-system watch machinery.

## 5. Decisions

D-1 to D-7 as approved at G2 on 2026-11-07: kit run; the internal `watcherLink` with the Owner's condition that it preserve every observable of ADR 0038 D6 (met: the removal is asynchronous, no window is asserted, an already stopped subscriber is registered and removed after `Subscribe` returned); a foreign `Ref` is not watched (a README line); D-4, `Subscribe` on a stopping System, decided by EV12 and **confirmed by the Owner on 2026-11-07 as "behave as Pekko"**; D-5, no order among removals; D-6, no dead-letter count in the window; D-7, `status:ready`. **D-8** (remove synchronously inside `Subscribe`) was proposed and **rejected by the Owner** because it contradicted ADR 0038 D6.

## 6. QA findings that changed the work (from the PR threads)

- **Scope (#1440) and G2:** the Owner's conditions on D-2 and D-3 and the rejection of D-8 shaped T2 and the seam (the removal must run after `Subscribe` returned).
- **#1457 (kit package): REJECT, then APPROVE** (QA comment 6704240316 on head `e08955dc`). Manifest row 1 cited `EventStreamUnsubscriber.scala:38-47`: line 38 is blank and `case Register(actor)` is at :40; it was fixed to `39-47` (first line `def receive = {`). The Evidence list was also incomplete against RULEBOOK section 6: it lacked `EventStreamSpec.scala:375, 395`, the goPekko ranges `watch.go:438-442`, `stop.go:119-126` and the EV7 log lines; the delta verdict (APPROVE, head `aa588198`) found the list complete. The same review confirmed that the scope's `58-64` is off by one (line 58 is blank, the case is `59-64`): a scope typo, separate from the PR, corrected in this slice.
- **#1458 (oracle EV12): APPROVE** after a report-based review. The QA comment on the PR (6704299226) is a misposted copy of the delta verdict of #1457; the TL posted a correction note (6706861066). The real review was a report on head `50f90811` (APPROVE) with W1 (the summary said the stopped subscribers "are removed" while no log shows the unsubscribe), W2 ("user guardian alive" stated as fact while only inferred) and S1 (the exit code of `sbt -batch compile` and the `check-headers.sh` baseline missing from the Evidence). The names W1, W2 and S1 are taken from the Dev's second Evidence comment (6704311355), which fixed them on head `c61c7472`, merged as `c59b19b7`; they moved claims from Observed to Inferred.
- **#1459 (slice 1a-i): REJECT, then APPROVE.** S1-1: replacing `go run()` with `run()` left the whole suite green because the test seam replaced the goroutine, so no test ran the `go` statement and that mutant would self-deadlock in production; the seam now gates the goroutine body, and a test with the gate closed sees the registration present and `Subscribe` returned. The same slice adapted case 2 of the P6 test (`eventstream_publish_delivery_test.go`), whose premise T1 made false, with a foreign `Ref`; approved by the TL and recorded in the scope. The QA also asked to check that the removal runs under the stream lock (now asserted with `TryLock` at the `locked` stage).
- **#1460, #1461: APPROVE**, no finding that changed the work; #1461 suggested a test of the unwatch running under the lock (not added, section 8).

## 7. PRs and merge commits (`gh pr view <n> --json mergeCommit,additions,deletions,mergedAt`)

| PR | What | Merge commit | Lines | Merged (UTC) |
|---|---|---|---|---|
| #1440 | scope (approved at G2) | `5163f92e` | 110 | 2026-11-07T07:59Z |
| #1457 | slice 0, kit package | `348ded06` | 91 | 2026-11-07T08:05Z |
| #1458 | slice 0b, oracle EV12 | `c59b19b7` | 334 | 2026-11-07T08:10Z |
| #1459 | slice 1a-i, the termination watcher, T1, T2, and the P6 adaptation | `19e85ca1` | 340 | 2026-11-07T08:29Z |
| #1460 | slice 1a-ii, T3, T4, T4b | `c88f9942` | 128 | 2026-11-07T08:36Z |
| #1461 | slice 1b, the unwatch, T6, T7, end of `TODO(EVT-02-stopping)` | `8158d689` | 232 | 2026-11-07T12:13Z |
| this PR | slice 2: README, record, cost-log row, three corrections to the scope text | see `git log` | see its description | |

All are under the 400 budget; the split of slice 1a was the scope's own.

## 8. Known gaps and follow-ups

- #1055: the `/system` actor of Pekko (SYS-18, plan row F-01); do not migrate the link unless the migration changes something observable.
- #1345: the `DeadLetter` cycle stays a known gap. #1426: the 6 SA1019 findings and the removal of `SubscribeDeadLetters`.
- The T6 test is probabilistic; the unwatch under the stream lock is covered by design, not by a test (QA of #1461); the EV12 line numbers are run-1 numbers.
- A foreign `Ref` is never retired automatically (D-3).

## 9. Open API choices

None: the unit exports nothing (the link, the seam and the tests are unexported). The index #1020 records "no export".

## 10. Recurring failures (`AGENTS.md`, kit rule 6)

A test seam that replaced production code instead of gating it appeared once (#1459 S1-1); a Pekko citation off by one line twice (#1457, and the scope's `58-64`). None reaches three occurrences, so no rule amendment is proposed.
