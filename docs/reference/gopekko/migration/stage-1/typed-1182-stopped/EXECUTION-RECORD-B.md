# EXECUTION RECORD: held inputs as dead letters and `Context.System()`, slice B (#1182, TYP-48)

Scope `docs/scopes/1182-typed-stopped.md` (approved at G2 by the Owner, 2026-11-10; D-7 confirmed 2026-11-11). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`). The code is PR #1545 at head `bcc217712823d22353c4da562169f0583105345b` (`Refs #1182`, never `Closes`); this PR is the record, the manifest rows and the cost-log row. The RULEBOOK of slice A applies unchanged. Evidence classes: ORACLE (a Pekko run, in `docs/parity/oracles/`), READ (Pekko code, read), as in the manifest.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`), one Dev session; wall clock, tokens and subagents were not measured: `unknown` in the cost-log row.
2. Inputs: oracle W3c (`typed-restart-child-wait-2026-11-06.md:66`) and W3a (`:64`); oracle `restart-stash-held-2026-11-11.md` RS1 [019, 021], RS2 [017, 021] (PR #1548, head `d5d39985d6e859bdd43e743a74f98d9b90f912e2`; the file reaches `develop` with that PR); oracle `unstash-deadletters-2026-11-10.md` US3 [005]; `StashBufferImpl.scala:221, 241, 246-251`, `Supervision.scala:210, 266-285, 407`, `ActorCell.scala:187`, typed `ActorContext.scala:76`. Each of those ranges was re-opened in `upstream/pekko`.
3. Outputs: `Context.System()` (`pekko-actor/actor/actor.go`, EXP-1 in its comment), `deadLetterHeld` in `typed/interpreter.go`, tests, two README paragraphs.
4. Deviations from the scope: (a) the record, manifest rows and cost-log row are a separate PR (slice B measured 395 authored lines, 391 added and 4 deleted, without them; the budget is 400); (b) the code is 395 lines against the scope's estimate of about 220: the held-input scenarios, the held `Terminated` test and the W3a test were not in the estimate; (c) `Context.System()` was **added** here, because `develop` `9e95d8be` has none and #821 slice 1 had not merged; (d) the head `bdd0196b` published a held `Terminated` as `SuppressedDeadLetter`, an inference from the code (the oracle US6 observed a `Terminated` in a user queue, a different route); QA rejected it, the oracle of PR #1548 observed an **ordinary `DeadLetter`** of the **typed** `Terminated` (RS2 [017]), and the head `bcc21771` follows it by publishing `typed.Terminated`. The classic `actor.Terminated` keeps its `DeadLetterSuppression` marker of #871; the marker and the single funnel `System.publish` are untouched.

## 2. RED evidence

Tests first: `TestContextSystem` did not build (`ctx.System undefined`) and the held-input specs failed before `deadLetterHeld` existed: "expected after-stop to equal m-behind-1" (the held messages were dropped) and timeouts waiting for a dead letter. A first version of the W3a test failed on the correct tree ("expected m1 to equal after-stop"): the two messages were still in the mailbox when the stop was accepted, and the runtime's own mailbox cleanup dead-letters those. The scenario was changed to wait for the `Terminated` of a buddy that arrives after them, so that both are held first. The `Terminated` test of the head `bcc21771` (typed signal, ordinary `DeadLetter`) was written against the oracle RS2 and fails under mutation M6.

## 3. Mutations (RULEBOOK section 5)

Applied by hand, restored by `git checkout` of the file after each, never committed. Details with the failing assertions are in the PR body of #1545. All are declared by the Dev until the QA of #1545 reproduces them.

| Id | Criterion | Mutation | Failing tests |
|---|---|---|---|
| M1 | R7, R8 | `stopWith` does not call `deadLetterHeld` | R7, R8, US3 shape, held `Terminated` |
| M2 | R7 | `deadLetterHeld` reverses the order | R7, US3 shape, held `Terminated` |
| M3a | R8 | `Sender` is the actor's own handle | R8, US3 shape, held `Terminated` |
| M3b | R8 | `To` is the dead-letter sink | R8, US3 shape |
| M4 | W3a | `PostStop` calls `deadLetterHeld` first | `TestRestartStopped_StopDuringWaitPublishesNothingHeld` |
| M5 | D-7 | `Context.System` returns `nil` | `TestContextSystem` |
| M6 | R8 (RS2) | the held `Terminated` is published as the classic `actor.Terminated` (`msg = h.term`), so it is suppressed | `TestRestartStopped_HeldTerminatedIsAnOrdinaryDeadLetter` at `restart_stopped_test.go:222` ("expected true, got false") |

M4 first sat in `stop()` and no test failed: an external stop never goes through `stop()`. It was moved to `PostStop`. The runs of M1 take about 9 s because three waits of 3 s end in an assertion.

## 4. Gates

Declared by the Dev, bound to `bcc217712823d22353c4da562169f0583105345b`: `gofmt` empty; `go vet` 0; `go test -race` ok in `pekko-actor` and `pekko-actor-typed` (coverage 98.4% and 99.5%); diff coverage 100% on 14 lines (`diff-cover` 11.0.0); `golangci-lint` v2.14.0 gocyclo 0 (3 SA1019 as on `develop`); `check_deps.sh` 0; `make ci-fast` 0; `check-judge-matrix.py` 0 (goPekko CURRENT already 0/21 on `develop`, stale by the rule); `check-headers.sh` exit 1 only on files this PR does not touch; no network isolation (macOS). They become "reproduced by the QA" only where its comment says so.

## 5. Evidence class and what is not claimed

OBSERVED in the restart's own stash (oracle `restart-stash-held-2026-11-11.md`, PR #1548): held messages are `DeadLetter(msg, deadLetters, A)`, in order, once (RS1 [019, 021]); a held typed `Terminated` is an ordinary `DeadLetter` with the same sender and recipient, not a `SuppressedDeadLetter`, and `A` never sees it (RS2 [017]). Count and order: ORACLE W3c. NOT run in a restart's stash, so not observed: a new `Setup` that is itself `Stopped` (the US3 shape was observed on a `StashBuffer` only, US3 [005]), `ChildFailed`, `watchWith`, a full stash (`Dropped`). Not claimed: V3, Typed parity, the interruption of an `unstashAll` replay (slice C of #927), the failure policy per strategy, an interceptor around `Stopped` (`PORT-GAP(TYP-48-interceptor-978)`), the `Stop` strategy (`PORT-GAP(TYP-48-stop-variants-1063)`). The Go campaign after the merge refreshes CURRENT; until then V2/V3 have no validity.
