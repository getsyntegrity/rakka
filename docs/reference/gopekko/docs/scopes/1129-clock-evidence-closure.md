# Evidence closure: injectable System clock (#1129, TIM-11), Squad B

Status: **TL proposal (sections 1-8); the Owner's answers O1-O5 of 2026-10-29 are recorded in section 9.**
Written 2026-10-29 against `origin/develop` `0ecfc3cff5304c812ca57686b6b365b2cf4c62ad`. Integrated feature: PR #1267, merge commit `e786818792e1a26d1f50159482b5b5e826b80e13` (`Config.Clock`, `Clock`, `Timer`; not redesigned here).
Pekko: v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, submodule checked out and equal to the pin. Scope of record: `docs/scopes/1129-injectable-clock.md` (unchanged by this document).
**Labels used below.** EXECUTED-GO = a Go test run in this worktree. EXECUTED-JVM = a JVM program run against the pin in a disposable copy. READ = source read, not run. PENDING = not run.
This document adds no production code, no scheduler, no timer and no ManualTime.

## 1. The four things, separated (Owner brief of 2026-10-28)

| # | Category | Content |
|---|---|---|
| 1 | Executed in Go | `TestClockAPI_*` (3 tests), `TestRealClockFiresOnceAndStopPreventsIt`, `TestSystemClockDefaultsToTheRealClockAndTakesAnInjectedOne`, `TestFakeClock*`; run focused on this tree: all PASS (section 6). |
| 2 | Read from Pekko | Section 3. |
| 3 | Differential scenarios against Pekko | P1 and P2 EXECUTED-JVM, section 4. Their Go counterparts are the existing tests named there; no new Go test is needed. P3 is informational and is a **non-equivalence**, not a match. |
| 4 | Needs Owner acceptance | Section 7 only. |

## 2. Where Pekko 1.7.1 defines the equivalents (READ, path:line at the pin)

| Concept | Pekko | Go counterpart |
|---|---|---|
| Ask deadline armed on the scheduler | `actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala:725` `scheduler.scheduleOnce(timeout.duration)`, `PromiseActorRef.apply` at `:709-742` | `ask.go` `askArm`: `clk.AfterFunc(rem, e.expire)` |
| Timeout completes the future once | `AskSupport.scala:726-737` `result.tryComplete(Failure(AskTimeoutException))` | `askEndpoint.expire` |
| Cancellation when the reply wins | `AskSupport.scala:739-741` `result.future.onComplete { _ => try a.stop() finally f.cancel() }` (runs for every completion) | `ask.go` `t.Stop()` once after the Ask settles |
| Late reply after completion | `AskSupport.scala:625-627` `provider.deadLetters ! message` | out of this story (judge S15 covers Pekko) |
| Non-positive timeout | `AskSupport.scala:377,409` `timeout.duration.length <= 0` then `negativeTimeoutException` (`:300-306`, `IllegalArgumentException`) | `ErrAskTimeout` for a deadline not in the future (ADR 0015 D5) |
| Cancellable | `actor/.../actor/Scheduler.scala:525-542` `cancel(): Boolean` ("true if that was successful") | `Timer.Stop() bool` ("reports whether it prevented the call") |
| Manually driven time | `testkit/src/main/scala/org/apache/pekko/testkit/ExplicitlyTriggeredScheduler.scala:44-140`: `timePasses` `:67-81`, runs tasks with `time <= runTo` (`:89,93-107`), its `cancel()` removes the item and returns `before > size` (`:124-129`); starts at time 0, ms resolution (`amount.toMillis`, `:73`) | the caller-written double of `clock_api_test.go:30-93` (no exported manual clock, D3 = 1) |
| Typed ManualTime | `actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed/scaladsl/ManualTime.scala:34-73` wraps the same scheduler | not in this story (M-04) |
| Start time / uptime | `actor/.../actor/ActorSystem.scala:572,577` `System.currentTimeMillis` (no injection) | `Config.Clock` also drives `StartTime`/`Uptime`: Go-only |

## 3. Why the APIs are not equivalent (do not compare them as if they were)

- **Deadline form.** Pekko's `Timeout` is a relative duration armed on the scheduler's own time. Go's Ask takes an absolute `ctx` deadline and arms the remaining time (`deadline - clock.Now()`) on the injected clock. The context's own deadline still runs on the wall clock (`ask.go` `askAwait` selects on `ctx.Done`), which is why the Go tests set the context deadline a day ahead of the wall clock (`clkFar`). A Go test and a Pekko scenario share a property only through the armed delay and the moment it fires.
- **Resolution.** Pekko's manual time is whole milliseconds; the scenarios use whole milliseconds only.
- **Non-positive deadline.** Pekko: `IllegalArgumentException`, no timer (P3). Go: `ErrAskTimeout`, no timer, `build` not called. Same "no timer armed", different failure kind and different `build` behavior (Pekko calls the factory to build the message text, `AskSupport.scala:409-411`). This is a Go-only decision, not a shared property.
- **Where the clock is used.** Pekko's scheduler is a scheduler (periodic, `scheduleOnce`, many users). The Go `Clock` has two methods and one real consumer besides `Now` (Ask). Nothing about schedulers is compared.

## 4. Shared properties and their differential scenarios

Probe: `migration/stage-1/actor-1129-injectable-clock/pekko-repro/ClockRepro.scala`; recorded output: `OBSERVED.txt` next to it. It follows the judge's controlled-mode pattern (`judge/pekko-harness/Judge.scala` `Driver`, `runOne`): `ExplicitlyTriggeredScheduler`, `CallingThreadDispatcher`, scenario body inside the driver's handler after a warm-up round trip. It adds no sleep and no polling (the 100 ms `Thread.sleep` is inside Pekko's own `timePasses`, `ExplicitlyTriggeredScheduler.scala:71`). It reads the scheduler's private `scheduled` map size by reflection (field found by suffix, scalac mangles it) to observe cancellation; that is introspection of the oracle, stated as such.

| Property | Pekko observation (EXECUTED-JVM) | goPekko counterpart (EXECUTED-GO) | Comparison rule |
|---|---|---|---|
| **P1 deadline + fires on advance** | 1 s ask, no reply. Armed: pending=1, future pending. After `timePasses(999ms)`: future pending, pending=1. After `timePasses(1ms)`: `AskTimeoutException`, pending=0. | `TestClockAPI_AskDeadlineFiresOnlyWhenTheClockAdvances` (`clkFiresOnAdvance`, `clock_api_test.go:138-151`): armed delay observed = 1 s; callbacks run after `Advance(999ms)` = 0; after `Advance(1ms)` = 1; `Ask` returns `ErrAskTimeout`. | Same tuple (armed delay, nothing at D-1ms, fired at exactly D, failure is a timeout). Failure kinds are mapped by name (`AskTimeoutException` to `ErrAskTimeout`), not compared as types. Boundary rule agrees: fires when `due <= now`. |
| **P2 cancellation when the reply wins** | Reply before the deadline: armed pending=1; after the reply the future is `success:reply` and pending=0 (the timer was cancelled); after `timePasses(1h)` still `success:reply`, pending=0. | `TestClockAPI_ReplyStopsTheTimerOnce` (`clkReplyWins`, `clock_api_test.go:160-173`): result is the reply; `Timer.Stop` calls = 1; after `Advance(1h)` callbacks run = 0 and Stop calls still 1. | Same tuple (reply is the result; armed timer no longer pending right after the reply; a later advance changes nothing). "Stop called exactly once" is additional Go evidence (AC-R2-2); the JVM pending count does not observe it and it is not attributed to that observation. |
| P3 non-positive timeout (informational) | `IllegalArgumentException`, pending=0. | `clkDueDeadline` (`clock_api_test.go:121-129`): `ErrAskTimeout`, `build` calls = 0. | **Not compared.** Documented non-equivalence (section 3). |

Existing Pekko-side oracle already in the repository for the same ask paths: judge S04 (timeout on `timePasses`) and S15 (late reply dead-lettered) in controlled mode (`judge/pekko-harness/Judge.scala:282-288,399-405`). They were re-run here: `bash judge/run-pekko.sh <copy> <out> controlled 10000 S04 S15` exit 0, then `python3 -I judge/evaluate.py <out> controlled --expect-runs 1 --spec-dir judge/scenarios S04 S15`: both `complete, 1 of 1 runs`, pass 1, fail 0. They are **not** in `judge/go-scope.json` (Go side has no ask scenario), so the Go-vs-Pekko comparison for ask is by the P1/P2 tuple table above, not by the judge comparator. Moving S04/S15 into the Go judge would need a clock-driven judge env (`newJudgeEnv` builds `Config{}`) and is a harness extension outside this burst: **NOT EXECUTED, not proposed for #1129.**

## 5. Criteria -> evidence -> SHA -> pending

Class: **(a)** Go-only injection surface, no applicable Pekko oracle; **(b)** shared property with a differential scenario; **(c)** needs explicit Owner acceptance. A row can carry two classes.

| Criterion | Class | Evidence (EXECUTED-GO unless stated) | SHA of the evidence | Pending |
|---|---|---|---|---|
| R1 seam exists, zero value = wall clock | (a) | `TestSystemClockDefaultsToTheRealClockAndTakesAnInjectedOne` (`clock_internal_test.go:175`); `Config.Clock` in `system.go` | `e786818` (seam), `020611b` (test) | none (Go-only, D13; see section 9) |
| AC-R1-1 short real deadline, no reply: `ErrAskTimeout` | (a) default unchanged; the timeout result itself is shared with Pekko | `TestBoundedMailboxAskAgainstFullMailbox`: Ask with `context.WithTimeout(bg, 50ms)` and no reply (`bounded_mailbox_test.go:198-201`); the timeout machinery on the fake clock: `TestAskTimeout_T1..T10` (`ask_timeout_internal_test.go`, comments cite `AS:721-743`) | `cd55894` (ask timeout tests), `fab8f1d` (ask.go); bounded test `bdae37f` | Weak point: no test is named for "default clock, real short deadline" (the only one is incidental, in the bounded-mailbox suite). Section 8 T1 proposes no new test. Resolved by O3 (accepted, section 9): the existing Ask tests cover AC-R1-1 and no duplicate test is added |
| AC-R1-2 System reads the supplied clock | (a) | `TestClockAPI_SystemReadsTheSuppliedClock`: `StartTime`/`Uptime` follow the clock; a deadline equal to the clock's now gives `ErrAskTimeout` without calling `build` | `e786818` | Go-only (Pekko `ActorSystem.scala:572,577` is not injectable; P3 is a non-equivalence). O1 accepted (section 9) |
| AC-R2-1 deadline fires only when the clock reports it due (999 ms then 1 ms) | (b) P1 | `TestClockAPI_AskDeadlineFiresOnlyWhenTheClockAdvances`; Pekko: `OBSERVED.txt` P1 | `e786818` (Go), this branch (JVM) | O2 accepted (section 9) for the P1 tuple comparison only; not V3, not a judge-comparator run |
| AC-R2-2 (added by scope) reply wins: timer stopped once, later advance fires nothing | (b) P2 for the cancellation, (a) for "Stop exactly once" | `TestClockAPI_ReplyStopsTheTimerOnce`; Pekko: `OBSERVED.txt` P2 | `e786818` / this branch | O2 accepted for the shared part (cancellation, P2); "Stop exactly once" is additional Go evidence, not attributed to the JVM pending-count observation; O1 covers no part of this row |
| AC-R3-1 `Stop` reports whether it prevented the call; no run after `Stop` true | (a) + (c) | Real clock only internally: `TestRealClockFiresOnceAndStopPreventsIt` (`clock_internal_test.go:144`); doubles: `TestFakeClock*`; Pekko analogue READ only (`Scheduler.scala:534`, `ExplicitlyTriggeredScheduler.scala:124-129`) | `77a86d5` (clock), `020611b` (tests) | **The issue text asks for a public test "against the manual clock double and against the real clock"; the real clock is not exported (D3 = 1), so the public half is not met literally.** Resolved by O4 (confirmed, section 9) |
| AC-R4-1 ADR 0011 record per export; `check_deps.sh` | (a) + (c) | Records in the body of PR #1267 (table of three exports); `scripts/check_deps.sh` exit 0 recorded in the PR | `e786818` | The records live in the body of PR #1267; O5 confirmed (section 9) |
| Issue Out of scope (no scheduler/timers/ManualTime) | | nothing was added | | none |

## 6. Executed here (reproducible commands)

- Go, from `pekko-actor` (toolchain `.go-version` = 1.27.2): `go test -count=1 -run 'TestClockAPI|TestRealClock|TestFakeClock|TestSystemClockDefaults' -v ./actor` -> exit 0, 7 tests PASS (3 `TestClockAPI_*`).
- JVM, focused (authorized for this burst): disposable copy `git -C upstream/pekko archive 9a54d1294c3ac7821bbed3c940769c3d5027d1ff | tar -x -C <copy>/pekko` (committed and tagged `v1.7.1` in a throwaway repository, as `migration/stage-1/judge-g5f-pekko/EXECUTION-RECORD.md` did), then `bash judge/run-pekko.sh <copy>/pekko <out> controlled 10000 S04 S15` (exit 0; installs `judge.sbt`), then `cp ClockRepro.scala <copy>/pekko/judge/src/main/scala/` and `JAVA_8_HOME=$HOME/.sdkman/candidates/java/8.0.504+1-zulu sbt -batch "judge/runMain ClockRepro"`.
- Honest run history: the first two attempts of the probe failed because of defects in the probe itself (no `JAVA_8_HOME` exported; a reflection field name; a first draft without the driver pattern whose P2 observation raced the reply). They were fixed, not retried until green; the final program ran 3 times with byte-identical `OBS` lines. Not run on Linux, not under `-race`, not a campaign.
- `upstream/pekko` was only read. Scratch lives in a `mktemp -d` directory outside the repository (`/tmp/b1129.*`), removed by the parent.

## 7. Owner decisions needed (proposed wording; the TL accepts nothing)

D4 of the scope stays open. Proposed text for the Owner to approve, amend or reject, one line each:

- **O1.** "For #1129, `Config.Clock` driving `StartTime`/`Uptime` and the due-deadline precheck (`ErrAskTimeout` without calling `build` when the context deadline is not after the clock's now) are Go-only; ADR 0033 D13 alternative evidence is `TestClockAPI_SystemReadsTheSuppliedClock`; recorded as approved." (Pekko is not injectable at `ActorSystem.scala:572,577` and rejects non-positive timeouts with `IllegalArgumentException`, a documented non-equivalence.)
- **O2.** "The Pekko observations P1 and P2 in `pekko-repro/OBSERVED.txt` (v1.7.1 pin, `ExplicitlyTriggeredScheduler`) are the differential evidence for 'armed on the clock, fires only when due, cancelled when the reply wins'; the Go side is `clkFiresOnAdvance` and `clkReplyWins`; the pair is accepted as the applicable differential scenario for those two properties. Their comparison is by the tuple rule of section 4, not by the judge comparator."
- **O3.** "AC-R1-1 (default wall clock, short real deadline, no reply) is satisfied by the existing Ask tests; no new test is required." (Alternative if refused: the Dev adds one public test with `context.WithTimeout(bg, 20ms)` and a never-replying actor; that test is a real-time wait and is therefore discouraged by this burst's rules.)
- **O4.** "AC-R3-1 is reframed: the real-clock `Stop` contract is proven by the internal test `TestRealClockFiresOnceAndStopPreventsIt`; the public half is the System's single `Stop` call (AC-R2-2). No wall-clock value is exported (D3 = 1). The issue text is not edited by the TL."
- **O5.** "The three EXP-1 / ADR 0011 export records of PR #1267 stand as the AC-R4-1 evidence in the PR body; indexing them is left to #1020."

Documents whose text the Owner approves: this file (`docs/scopes/1129-clock-evidence-closure.md`), the probe `pekko-repro/ClockRepro.scala` and `pekko-repro/OBSERVED.txt`. The scope file `docs/scopes/1129-injectable-clock.md` (sha256 `82e84059dd0eaef1c7ef120de239ef23a89a29f1ccd9efb15e4114c515af1ce3`) and `EXECUTION-RECORD.md` (sha256 `12ce8b7ea0e13e093aa7bd42b89b515b325db230aa88dfa7e5baf56323618b3a`) are unchanged by this branch. The SHA-256 of the three new files is reported in the hand-off, outside the file, because a file cannot contain its own digest.

## 8. Execution plan for the Dev (not executed by the TL)

**New Go tests: none.** Every shared criterion (AC-R2-1, AC-R2-2) already has a public test; AC-R1-1 is O3; AC-R3-1 is O4. If the Owner refuses O3, the Dev adds the single test described there.

Dev steps, in order:
1. Record the Owner's answers to O1-O5 verbatim in a new section 9 of this file (append only; do not edit sections 1-8) and, if accepted, append one "post-merge evidence" paragraph to `migration/stage-1/actor-1129-injectable-clock/EXECUTION-RECORD.md` pointing here (category 3 changes from "pending" to the P1/P2 result; categories 1, 2, 4 unchanged).
2. Re-run the JVM probe once on the Dev's machine with the commands of section 6 and diff against `OBSERVED.txt`; a difference is a finding, not a retry. Commands: section 6, JVM bullet.
3. Gates (AGENTS.md "Go"): `.go-version` toolchain; `scripts/each_module.sh go test -count=1 ./...` is not needed for a docs-only change, run focused `cd pekko-actor && go test -count=1 -run 'TestClockAPI|TestRealClock|TestFakeClock|TestSystemClockDefaults' ./actor`; `scripts/check_deps.sh`; `.github/scripts/check-headers.sh` (the Scala probe has no "Ported from" line, so it is not checked); `git diff --check`. Coverage and diff coverage do not apply (no `.go` change). `bin/golangci-lint` (`.golangci-version`) is not needed (no Go change).
4. Commit with a Conventional Commit (`docs(actor): ...`), `Refs #1129`, no closing keyword, no force-push, no PR from the TL.

**Edit surfaces (exact):**
```
docs/scopes/1129-clock-evidence-closure.md
migration/stage-1/actor-1129-injectable-clock/EXECUTION-RECORD.md
migration/stage-1/actor-1129-injectable-clock/pekko-repro/ClockRepro.scala
migration/stage-1/actor-1129-injectable-clock/pekko-repro/OBSERVED.txt
```
No `.go` file, no `pekko-actor/README.md`, nothing under `pekko-actor/actor/`, `judge/` or `upstream/`.

## 9. Owner answers O1-O5 (authority: mensaje del Owner de 2026-10-29; appended, sections 1-8 otherwise unchanged)

Authority for every row below is the **mensaje del Owner de 2026-10-29** (the current message), not any earlier message. Each acceptance applies to that message only.

| Item | Status | Record |
|---|---|---|
| O1 | **Accepted** | Clock injection on `StartTime`/`Uptime` and the due-deadline precheck have their Go evidence documented in `TestClockAPI_SystemReadsTheSuppliedClock` (ADR 0033 D13 alternative evidence). |
| O2 | **Accepted, exclusively for P1 and P2** | Scope: the properties compared through the section 4 tuples of PR #1272: P1 (armed, fires at the deadline) and P2 (cancelled when the reply wins). The comparison is between the recorded JVM runs (`OBSERVED.txt`) and the Go counterparts `clkFiresOnAdvance` and `clkReplyWins`. It is NOT a run of the judge comparator, does NOT deliver S04/S15 on the Go side, and does NOT establish V3 or general parity. "Stop exactly once" remains additional Go evidence and is not attributed to the JVM pending-count observation. |
| O3 | **Accepted** | AC-R1-1 is covered by the existing Ask tests (matrix of #1272); no duplicate test is added. |
| O4 | **Confirmed** | AC-R3-1: the real-clock contract is verified by `TestRealClockFiresOnceAndStopPreventsIt` (existing internal test); the public part verifies the System's use of `Stop` (AC-R2-2). No other clock is exported. |
| O5 | **Confirmed** | The EXP-1 records of #1267 are the evidence for AC-R4-1; #1020 stays as an index and does not block. |
| #1129 | **Not closed, not merged** | The issue stays open; this PR is not merged by the Dev. |

Dev re-run of the JVM probe (section 6 commands, own scratch outside the repository): exit 0, the `OBS` lines are byte-identical to `OBSERVED.txt` (diff exit 0). Focused Go run of the 7 tests: PASS.

### Propuesta final de cierre de #1129 (proposal only; nothing is closed)

Satisfied, with evidence:
- R1 / AC-R1-2: `TestClockAPI_SystemReadsTheSuppliedClock`, `TestSystemClockDefaultsToTheRealClockAndTakesAnInjectedOne` (`e786818`, `020611b`); O1.
- AC-R1-1: existing Ask tests (`cd55894`, `fab8f1d`, `bdae37f`); O3.
- AC-R2-1 (P1) and AC-R2-2 cancellation (P2): `clkFiresOnAdvance`, `clkReplyWins` (`e786818`) against the JVM runs in `OBSERVED.txt` (pin `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, committed in `5c581aa`); O2, limited as stated above. "Stop exactly once": additional Go evidence.
- AC-R3-1: `TestRealClockFiresOnceAndStopPreventsIt` (`77a86d5`, `020611b`) plus AC-R2-2; O4.
- AC-R4-1: EXP-1 records in PR #1267 (`e786818`); `scripts/check_deps.sh` exit 0; O5.
- Out of scope respected: no scheduler, timer or ManualTime was added.

Accepted by the Owner (message of 2026-10-29): O1, O2 (P1/P2 only), O3, O4, O5.

Remaining before integration and closure: a QA verdict on PR #1272 (this PR is docs only; the QA check of the P1/P2 correspondence to the pin is still the reviewer's), then the Owner's decision to merge #1272 and to close #1129. Not claimed: V3, general parity, S04/S15 on the Go side, a Linux or `-race` run.
