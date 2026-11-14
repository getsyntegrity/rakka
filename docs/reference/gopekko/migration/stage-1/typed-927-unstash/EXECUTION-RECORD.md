# EXECUTION RECORD: Typed `Unstash` and `UnstashAll`, slices A1, A2 and B (#927, TYP-14)

Scope `docs/scopes/927-typed-unstash.md` (PR #1524, approved at gate G2 by the Owner on 2026-11-10). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`). The code is PR #1534, final head `07ac953b326e7582df03ba64883aff200066cc65` (`Refs #927`, never `Closes`); this PR is the kit package. Written by the Dev of slice A1; the QA reviews are not in it. The record is extended by each later slice.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`), one Dev session; wall clock, tokens and subagents were not measured and stay `unknown` in the cost-log row.
2. Inputs: `StashBuffer.scala:142, 165`, `StashBufferImpl.scala:156-185, 192-216`, `StashBufferSpec.scala:106-259`.
3. Outputs: `typed/unstash.go`, `typed/unstash_test.go`, the README section "Replay", the exported-names list, this package and one cost-log row.
4. Deviations from the scope: the kit package is its own PR (the code measures 399 authored lines without it); the A1 estimate of the scope was about 220 to 250 and the measure is 399 with the tests and the README; A1 also starts a `Setup` start or result, which the scope table lists under A2 (U2), because the engine uses `startLayers` for every start; U2 stays an A2 criterion and its port test is not in A1.

## 2. RED evidence

The tests were written first and failed to build without `Unstash` (`b.Unstash undefined`), then the engine was added and they passed. The table below is the mutation proof (unmutated green, mutated red).

## 3. Mutations (RULEBOOK section 5)

Applied by hand in a disposable edit of `unstash.go` by a script outside the repository, restored after each run (the file is rewritten from the original), never committed.

| Id | Mutation | Failing tests |
|---|---|---|
| M1 | the replay takes the last message | 14 FAIL: ReplaysInOrder (U1), OnlyTheMessagesPresentAtTheCall (U3), NAndWrap (U4) and more |
| M2 | the head is not removed | 14 FAIL, the same set |
| M3 | the count is lost: `left := 1 << 30` | OnlyTheMessagesPresentAtTheCall (U3, both cases), NAndWrap; 0.00 s, by assertion |
| M3b | loop until the buffer is empty: `for b.NonEmpty() {` | OnlyTheMessagesPresentAtTheCall (U3, both cases): the re-stashing handler returns `errUnstashRunaway` at the third pass; 0.00 s, by assertion |
| M4 | `wrap` is not applied | NAndWrap (U4) |
| M5 | `n` is ignored | NAndWrap (U4, n and n=0) |
| M6 | the replay iterates a snapshot | ClearWhileUnstashing (U5) |
| M7 | an empty buffer starts the behavior | EmptyBufferReturnsBehaviorUntouched (U8) |
| M8 | a nil `wrap` is not the identity | 17 FAIL: ReplaysInOrder (U1), U3, U4 and more |
| M9 | `Same` restarts the behavior | 15 FAIL: U1 and more |
| M10 | a `Supervise` layer or a `Stopped` result is accepted | NotYetSupportedBehaviors |
| M11 | the starting behavior is not started | 22 FAIL: U1 and more; the D-6 test also ends by its 30 s wait |

## 4. Gates

On the final head of PR #1534, `07ac953b326e7582df03ba64883aff200066cc65` (macOS arm64, go1.27.2): `gofmt`, `go vet`, `go test -race`, coverage 99.4% of the module, diff coverage 100% (`diff-cover` 11.0.0, 37 lines), `golangci-lint` v2.14.0 with gocyclo 0 and 3 pre-existing staticcheck findings in other test files, `scripts/check_deps.sh`, `make ci-fast` and `scripts/check-judge-matrix.py` (exit 0; this slice touches no file of `pekko-actor/`). `check-headers.sh` reports no error in `pekko-actor-typed`. `unshare -rn` is not available on macOS. The full table is in the PR body.

## 5. Size (authored lines)

| PR | Content | Estimate | Measured |
|---|---|---|---|
| #1534 | slice A1, code, tests, README | ~220-250 | 399 (394 added, 5 deleted) |
| this PR | kit package, record, cost-log row | n/a | 159 |

## 6. Evidence class and pending approval (ADR 0033 D13)

- **SPEC/READ, read against Pekko v1.7.1 and not run:** every row of the manifest. Not V3; no Typed differential run exists for the replay.
- **Alternative evidence, pending the Owner's per-row approval:** U8 (the empty buffer returns the behavior unstarted, no Pekko test row), the D-6 test, the nil `wrap`, and the Go-only scenarios of the Setup returned by a handler and of the handler error.
- **Not claimed:** parity of the module with Pekko Typed; the dead letters and the interruption by `Stopped` (slice C); the failure per strategy (slice B).

## 7. Not delivered, with owners

Slice A2 (`Same`/`Unhandled` as the start, `ErrUnstashUnhandled`, the record of an unhandled message, `Supervise` layers); slice B; slice C was delivered after it (section 12). `PORT-GAP(TYP-14-signal-message)`; `PORT-GAP(TYP-14-backoff-1063)` and `PORT-GAP(TYP-14-stop-strategy-1063)`: #1063; `PORT-GAP(TYP-14-interceptor-978)`: #978.

## 8. Criteria against the scope

| Criterion | Result |
|---|---|
| U1, U3, U4, U5, U8, D-6 | delivered (#1534) |
| U2 | the `Setup` start works; its port test is A2 |
| U6, U7 | slice A2 |
| F1, F2 | slice B |
| F3, F4 | PORT-GAP |
| S1 to S5 | slice C |

## 8b. Corrections after the first QA of #1534 (head `43088015`, APPROVE with two warnings)

- **W1.** The first version of the PR body said M3 died in about 35 s by assertion. It did not: the U3 handler re-stashed `m2` on every pass and the test spun until the Go timeout. The handlers now count their passes (`uRestash`) and return `errUnstashRunaway` at the third, so M3 and M3b fail at once by assertion; the PR body was corrected. M8, M9 and M11 also end the D-6 test by its 30 s wait, but each already fails other tests by assertion.
- **W2.** The README section "Replay" states that U8 and the D-6 test have no Pekko test row and are alternative evidence pending the Owner's approval (ADR 0033 D13).
- The citation of `wrap(node.message)` is `StashBufferImpl.scala:198` (not 199) and `interpretOne` is `:192-216` (not `:193-217`), in the code, the README, the tests and the manifest.

## 9. Recurring failures (`AGENTS.md`, kit rule 6)

None in this slice.

## 10. Slice A2 (U2, U6, U7; code PR #1536, `feat/927-unstash-slice-a2`, head `03c18494480de57640faa89c33b8c7dbadb6bebe`)

Origin of the figures: everything below marked **declared by the Dev** is the Dev's own run; it becomes "reproduced by the QA of #1536" only where the QA comment on #1536 says so (the QA reports are not in this record).

Run: Claude Sonnet 5.5, one Dev session; wall clock, tokens and subagents `unknown` (cost-log row `927-typed-unstash-a2`). Inputs: `StashBufferImpl.scala:166-168, 192-236`, `StashBufferSpec.scala:129-150, 261-265`, `StashSpec.scala:331-435, 636-672`.

- **RED evidence (declared by the Dev).** The tests were written first. First run: the package did not build (`undefined: ErrUnstashUnhandled`). With only that variable declared, 9 tests failed by assertion: initial Same (plain and supervised, `err:typed: Same is not a valid initial behavior` instead of `one`), the nested Same (`[a b1]` instead of `[a b1 b2]`), the unhandled record (no `rec:unhandled`), `Unhandled` as the start (wrong error) and the Supervise layers (`errUnstashLater`). The intermediate-Same cases and the U2 case passed from the start (A1 already did them); they pin the behavior.
- **Mutations (declared by the Dev; the independent QA of #1536 reproduced MA1..MA8, the gates and the 400 lines on its final head `03c18494480de57640faa89c33b8c7dbadb6bebe`, verdict published in a comment of #1536).** By hand, by a script outside the repository, restored after each run; each fails by assertion in 1.3 s (3.3 s when the check waits 2 s for a message that never comes, and the compared value reads `TIMEOUT`): MA1 Same resolves to the argument (4 FAIL); MA2 Same ignores the behavior in progress (2); MA3 an Unhandled start is accepted (2); MA4 an unhandled answer is not recorded (2); MA5 the layers are dropped from the result (2, 3.3 s); MA6 the behavior in progress is never reset (2; the check is now inside the nested-Same case); MA7 `startInto` does not accept the started result (2, 3.3 s); MA8 the context has no interpreter link (5). Re-run on the final head with the same results.
- **Gates (declared by the Dev)**, final head of #1536 (macOS arm64, go1.27.2): `gofmt`, `go vet`, `go test -race`, module coverage 99.5%, diff coverage 98% (`diff-cover` 11.0.0, 59 lines, the empty marker method `startedB.behavior` is the one missing), `golangci-lint` v2.14.0 gocyclo 0 (3 pre-existing staticcheck findings), `scripts/check_deps.sh`, `make ci-fast`, `scripts/check-judge-matrix.py` (exit 0; no file of `pekko-actor/` changed), `check-headers.sh` no error in `pekko-actor-typed`. `unshare -rn` is not available on macOS.
- **Size.** #1536: 400 authored lines (336 added, 64 deleted, declared by the Dev; the QA recounts). This PR: 44 lines plus the changes of this round, measured in its body.
- **Deviations.** (1) A new test file, `typed/unstash_same_test.go` (internal, it needs the unexported record hook), not named in the allowed files; `typed/stash.go` gets the `inProgress` field, which the scope table lists. (2) `startInto` returns its error through a new `invalidStart` helper so that the new `startedB` case keeps gocyclo at 10 or less. (3) The diagram of the replay, removed from the README in A1 for the line budget, is restored here (a docs PR) after the "Evidence" bullet of the section "Replay", where the code PR does not edit. (4) U2 has no separate port test: the A1 case "a Setup returned by a handler is started before the next message" is labeled U2. (5) `PORT-GAP(TYP-14-setup-same)`: a `Setup` that resolves to `Same` or `Unhandled` is still `ErrInvalidInitialBehavior`; only a direct `Same` or `Unhandled` is resolved; Pekko resolves both (`StashBufferImpl.scala:227-236`); `Setup` to `Same` is pinned by the test `PORT-GAP(TYP-14-setup-same)`, and `Setup` to `Unhandled` only at the level of `start` in `unhandled_internal_test.go:159-166`, not through `Unstash`. (6) The Stop strategy of the supervised cases is `Restart` (`PORT-GAP(TYP-14-stop-strategy-1063)`) and `Behaviors.stopped` of the nested case is a log line (slice C). (7) `StashSpec.scala:712-728` ("work with initial same", the buffer built at the actor setup and `unstashAll(Behaviors.same)` called from a handler) is **not ported as its own test**: it resolves `Same` through the same path as `:331` (the current behavior of the actor) from a handler instead of a `Setup` factory; the manifest row says `omitted`, so that it is not dropped silently.
- **Evidence class.** SPEC/READ for every row; the record hook is internal, nothing observes it outside this package. Not V3; no parity claimed. Alternative evidence pending the Owner: the Go-only record of an unhandled replayed message, U8, D-6.
- **Not delivered.** B (the behavior that threw, supervision per strategy), C (`Stopped` and the dead letters).

## 11. Slice B (F1, F2, D-4; code PR #1542, `feat/927-unstash-slice-b`, head `72a0a6a2e5128f6e73eae241639105cb5b33c991`)

Origin of the figures: everything below is **declared by the Dev** until the QA comment on #1542 says it was reproduced. Run: Claude Sonnet 5.5, one Dev session; wall clock, tokens and subagents `unknown` (cost-log row `927-typed-unstash-b`).

- **Inputs.** `StashBufferImpl.scala:186-217, 271-286`, `ActorAdapter.scala:152-160`, `Supervision.scala:74-76, 98-102, 169-178, 318, 327, 416`, `StashSpec.scala:277-316, 477-515, 517-538, 565-611`.
- **RED evidence.** The tests were written first. First run: the package did not build (`undefined: unstashError`). With only the type declared, F1 PostStop, F1 PreRestart, F2 and the layers test failed by assertion and the panic case reported a panic escaping the handler; the mandatory `TestUnstashAll_SuperviseErrorIsCatchesFailureDuringReplay` passed before the wiring (an unwrapped error matches `ErrorIs`) and is a mutation guard: it dies under MB1.
- **Mutations.** MB1 to MB6 by hand with a script outside the repository, restored after each run, never committed; each fails by assertion (table in the body of #1542): MB1 `Unwrap` removed (15.8 s), MB2 `fail` does not install the behavior (9.7 s), MB3 `fail` does not install the layers (5.4 s), MB4 a panic escapes (3.4 s), MB5 a failure of a start is wrapped (3.4 s), MB6 the error carries no layers (8.9 s).
- **Gates.** Final head of #1542 (macOS arm64, go1.27.2): `gofmt`, `go vet`, `go test -race`, module coverage 99.5%, diff coverage 100% (`diff-cover` 11.0.0, 14 lines), `golangci-lint` v2.14.0 without gocyclo findings (3 preexisting SA1019), `scripts/check_deps.sh`, `make ci-fast`, `scripts/check-judge-matrix.py` exit 0, no `check-headers.sh` finding in `pekko-actor-typed`. Not run: `unshare -rn`.
- **Size.** #1542: 343 authored lines (330 added, 13 deleted). This PR: measured in its body.
- **Deviations.** (1) A panic of a replayed handler is recovered and returned as an error (Pekko catches NonFatal, `StashBufferImpl.scala:204-207`); before, it propagated. (2) The Stop-strategy and Backoff cases of `StashSpec.scala:483-564` stay `PORT-GAP(TYP-14-stop-strategy-1063)` and `PORT-GAP(TYP-14-backoff-1063)`; the default Stop is F1. (3) `Setup` resolving to `Same` or `Unhandled` stays `PORT-GAP(TYP-14-setup-same)` (issue #1538). (4) `StashSpec.scala:527-538` (PreRestart on failure with restart supervision and a slow stopping child) is not ported: `PORT-GAP(TYP-14-slow-child)` in the manifest; the restart wait for children is tested in `restart_children_test.go` and `restart_stopwait_test.go` (oracle `typed-restart-child-wait-2026-11-06`), but nothing exercises a failed unstash together with a slow child. (5) Only the direct branch of `ActorAdapter.scala:152-160` is ported (`behavior = e.behavior; throw e.cause`), not `TypedActorFailedException` nor `ActorInitializationException`; the manifest row D-4 says so.
- **Evidence class.** SPEC/READ for every row, not run; not V3; no Typed parity claimed (ADR 0033 D13).
- **Not delivered.** C (`Stopped` interrupts the replay and the dead letters), after #871, PR #1525 and slice B of #1182.

## 12. Slice C (S1 to S5, with #1169; code PR `feat/927-unstash-slice-c`, this package included)

Origin of the figures: everything below is **declared by the Dev** until the QA comment on the PR says it was reproduced. Run: Claude Sonnet 5.5, one Dev session; wall clock, tokens and subagents `unknown` (cost-log row `927-typed-unstash-c`).

- **Inputs.** `StashBufferImpl.scala:162-185, 192-252`, `StashSpec.scala:673-710`, the oracle `unstash-deadletters-2026-11-10.md` (US1 [007], US2, US3 [005], US4 [008], observed values: the contract) and the reuse of the mechanism of `deadLetterHeld` (`interpreter.go`): `ctx.System().DeadLetters()`, `DeadLetter{Msg, Sender: deadLetters, To: self}`, classification by `System.publish` (#871).
- **RED evidence.** The tests were written first (`unstash_stopped_test.go`); with the code of slice B, 7 of the 8 cases failed by assertion (`a dead letter that the scenario requires was not published`, `expected nil, got typed: a Stopped behavior cannot be replayed by Unstash yet`); S5 (a living replay publishes nothing) passed from the start and pins the behavior.
- **Mutations** (MC1 to MC10, by hand with a script outside the repository, restored after each run, never committed; each fails by assertion): MC1 the rest in reverse (6 FAIL, 3.5 s); MC2 the interpreted message is published too (2, 3.5 s); MC3 no dead letters (6, 18.5 s: six cases wait 3 s for the missing event); MC4 `msg` and not `wrap(msg)` (4, 3.5 s); MC5 the sender is the actor (6, 3.4 s); MC6 the recipient is `deadLetters` (6, 3.5 s); MC7 a living replay publishes the rest of the buffer (8, 3.4 s); MC8 a Stopped start publishes only the first (5, 12.5 s); MC9 `n` ignored (5, 3.4 s); MC10 a Setup that starts as Stopped publishes the rest (2, 3.5 s).
- **Gates** (macOS arm64, go1.27.2): `gofmt`, `go vet`, `go test -race`, module coverage 99.5%, diff coverage 100% (`diff-cover` 11.0.0, 30 lines), `golangci-lint` v2.14.0 (the 3 preexisting SA1019, no gocyclo finding), `scripts/check_deps.sh`, `make ci-fast`, `scripts/check-judge-matrix.py` exit 0 (CURRENT 0/21: develop is already stale by the rule since #1545; this PR changes no file of `pekko-actor/`). `check-headers.sh` could not be run from the Dev sandbox; its rule was applied by hand (`unstash.go`: Apache-2.0 and one `Ported from`; the new test: no `Ported from`). Not run: `unshare -rn` (macOS).
- **Deviations.** (1) `errUnstashLater` is removed: a `Stopped` start or result is returned for the handler to return, with the rest as dead letters. (2) A buffer with no actor behind it (the nil-context buffers of the tests) takes the messages of a `Stopped` start and publishes nothing; Pekko would have an actor. (3) A `Setup` returned by a handler that starts as `Stopped` ends the replay with the rest still in the buffer and no dead letter, as `StashBufferImpl.scala:195` does (the branch is easy to read as an omission). (4) The kit package is in the same PR as the code (it fits the 400 lines). (5) No new export; no `pekko-actor/` file changed.
- **Evidence class.** S1, S2: SPEC/READ (the Pekko spec is read, not run). S3, S4, S5: ORACLE for the sender, the recipient, the n prefix and the suppression (US1 to US4, Pekko 1.7.1, Java 1.8); the wrap of the dead letter and the case "Setup starts as Stopped" are read. Not V3; no Typed parity claimed (ADR 0033 D13).
- **Not delivered.** `PORT-GAP(TYP-14-setup-same)` (#1538), `PORT-GAP(TYP-14-slow-child)`, `PORT-GAP(TYP-14-backoff-1063)`, `PORT-GAP(TYP-14-stop-strategy-1063)`, `PORT-GAP(TYP-14-interceptor-978)`, `PORT-GAP(TYP-14-signal-message)`.
