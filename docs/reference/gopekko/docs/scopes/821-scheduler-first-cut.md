# Scope: the scheduler, first cut: `ScheduleOnce`, `Cancellable` and close (#821, TIM-02)

Status: **approved at G2 by the Owner, 2026-11-10** (decisions D-1 to D-10 of section 9, all resolved; none is open). No Dev has started; the Dev starts a slice only when its predecessor in section 5 allows it.
Written on 2026-11-10 against `develop` `8e334700`. Pekko baseline: v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: it ports Pekko behavior (`Scheduler.scala`, `LightArrayRevolverScheduler.scala`), so the Dev runs the kit with its own manifest, RULEBOOK and RUNBOOK per slice; loading the kit's rules does not count. Rows of `migration/cost-log.tsv`: one per kit run.
**Evidence labels.** READ = Pekko source read at the pin (`path:line` of the body, verified for this scope). ORACLE = observed by `docs/parity/oracles/scheduler-lifecycle-2026-11-10.md` (PRs #1506, #1507; Java 1.8.0_504, three runs per scenario). The Go side was read at the base SHA, not run. No Go code is written by this scope and no parity is claimed.

Contract sources, all accepted: [ADR 0039](../adr/0039-scheduler-lifecycle-and-cancellation.md) (Accepted, Owner 2026-11-10: O-1 to O-10 with precisions), [ADR 0011](../adr/0011-pekko-actor-layout.md) and EXP-1 (`docs/parity/typed-owner-decisions-2026-10-27.md:36`), [ADR 0032](../adr/0032-local-sender-envelope.md) D3/D4 (sender), ADR 0033 D3/D13/D14, the clock seam of [`1129-injectable-clock.md`](1129-injectable-clock.md). Paths below: `S:` = `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala`, `L:` = `.../LightArrayRevolverScheduler.scala`, `SS:` = `upstream/pekko/actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala`.

## 1. Objective

A `System` owns one scheduler. A caller can ask it to run a function, or to send one message to an actor, once after a delay, gets back a `Cancellable`, and can cancel it. The delay is judged on the System's `Clock`, a cancelled task never runs, and the scheduler stops when the System terminates. Nothing else: no fixed delay (#822), no fixed rate (#823), no keyed timers (#860), no `ReceiveTimeout` (#861), no `SchedulerTask.cancelled` (#1077).

## 2. Base state (READ, Go side at `8e334700`)

| Fact | Where |
|---|---|
| No scheduler. The only timer armed is the `Ask` deadline, through `System.clock` (`Clock{Now, AfterFunc}`, `Timer{Stop() bool}`, exported by #1129) | `pekko-actor/actor/clock.go`, `system.go:27-29,110,116`, `README.md:298-306` ("no scheduler or timer other than Ask's is armed") |
| `executor` is unexported, default `goroutineExecutor.Execute = go f()`, replaced only by tests through `options.exec` | `executor.go:15-22`, `system.go:27,110,169` |
| `TellFrom(ctx, target, msg, sender)` exists; a message to a stopped actor becomes a `DeadLetter` that keeps the sender; `System.DeadLetters()` exists (#896) | `ref.go:47-70`, `deadletters_ref.go:47-50` |
| `Config.Logger` is a `kitlog.Logger`, nil means silent | `system.go:69-90` |
| Termination: `Terminate` stops `/user`, the root guardian finalizes last and `WhenTerminated` closes; nothing closes a scheduler | `system_shutdown.go:30-66`, `stop.go:102-124` |
| `judge/mutate_go.py` anchors only `cell.go`, `hierarchy.go`, `mailbox.go`, `ref.go`, `spawn.go`, `stop.go`, `supervision_*.go`, `watch.go` | `grep` on `judge/mutate_go.py` |

## 3. The Pekko contract this slice ports (READ + ORACLE)

| Rule | Pekko (READ) | Observed (ORACLE) |
|---|---|---|
| `scheduleOnce(delay, receiver, message)` is `receiver ! message` run by the executor, with no liveness check; the sender is the implicit argument (default `noSender`) | `S:428-436` (the send is `S:434`); runnable form `S:479` | SC5once: `DeadLetter`, sender `deadLetters`, recipient the stopped actor |
| `Cancellable.cancel()` "returns true if that was successful"; a second call returns false; `isCancelled` "true if and only if ... successfully cancelled" | `S:525-542` | SC3o (a): true, true, false; `isCancelled` true |
| One-shot handle `TaskHolder`: one atomic swap of the slot; `cancel()` is true only when the swap found a task that had neither run nor been cancelled; `executeTask` swaps to `ExecutedTask` before handing the runnable to the executor, so a cancel during execution is false | `L:372-415` (swap `L:375-381`, execute `L:383-392`, cancel `L:398-404`, `isCancelled` `L:414`) | SC3o (b) after run: `cancel()=false`, `isCancelled=false`, again false; (c) during the callback: false, false |
| Delay at or below zero never throws: the task is handed to the executor at once on the caller's thread and the handle is `NotCancellable` (`cancel()` false, `isCancelled` false); the documentation of `S:423` is wrong | `L:163-168`, `L:205-209`, `L:420-424` | SC1: zero, `-1s`, `-1ns`: no exception, class `NotCancellable`, both false; with an inline executor the task ran inside the schedule call |
| A positive delay is a real `TaskHolder`, whatever its size; never early | `L:93-97`, `L:211-222`; `SS:156-175` | SC1 control `+1ns`: `TaskHolder`, ran 23 to 27 ms after the call (tick and dispatcher hop, not a promise) |
| A closed scheduler rejects a new task, also if it was queued during the close (the check is repeated after the enqueue, `L:218-220`); `scheduleOnce` turns it into `IllegalStateException` | `L:163-168`, `L:206-211`, `L:218-220` | not covered by the oracle: READ only (ADR 0039 "Not covered by the oracle") |
| Pending tasks are dropped on close: not run, not given `cancelled`, `isCancelled` stays false; `TaskRunOnClose` is not ported | `L:170-192`; `SS:684-697` | READ only |
| A task whose callback throws: the exception goes to the executor; on the dispatcher it is reported once as an `Error` (`TaskInvocation`) | `AbstractDispatcher.scala:55-63`; `L:386-392` | SC4 (periodic cases; the one-shot is the same path) |

Not ported, with the decision that says so: tick rounding and the `Int.MaxValue`-ticks limit (ADR 0039 O-7, Deviations 1 and 2, ACCEPTED PORT GAP, Owner 2026-11-10); `TaskRunOnClose` (O-5c); the Java overloads, `SchedulerTask`, `maxFrequency`, the deprecated `schedule` (ADR "Not covered").

## 4. Contract proposed (Go form; names are indicative, the shapes are the contract)

Package `actor` of `pekko-actor` (no new package: the scheduler needs the System's executor, clock, logger and dead-letter path, all unexported; a sub-package would force exporting them, which EXP-1 forbids as a convenience).

```go
// System accessor (ADR 0039 O-5 A: one scheduler per System, created with it, no public Close).
func (s *System) Scheduler() *Scheduler

// Cancellable (O-1 A: Pekko's trait, S:525-542, exactly two methods).
type Cancellable interface {
    Cancel() bool      // true only if this call prevented the run (one-shot)
    IsCancelled() bool // true only after a Cancel that returned true
}

var ErrSchedulerClosed error // slice 1b. O-5b: the schedule call returned after the System's scheduler closed
var ErrNilCallback    error  // D-5: returned for a nil callback (slice 1) or a nil target (slice 2); schedules and sends nothing

// Slice 1.
func (sch *Scheduler) ScheduleOnce(delay time.Duration, f func()) (Cancellable, error)
// Slice 2. A function, not a method: Go methods cannot take a type parameter (Ref[T]).
func ScheduleTellOnce[T any](sch *Scheduler, delay time.Duration, target Ref[T], msg T, sender Handle) (Cancellable, error)
```

Behavior (each line cites its source; "Go form" marks the adaptation):

1. **Never early, judged on the System clock (C5).** A task with `delay > 0` is armed with `System.clock.AfterFunc(delay, ...)`; the scheduler reads no clock at scheduling time, as `L:93-97` does not. No tick rounding (Deviation 1). A manual clock fires it exactly when advanced by `delay`.
2. **Zero or negative delay (O-3b, ORACLE SC1).** No error; the function is handed to the executor on the caller's goroutine at once (`executor.Execute(f)` called inside `ScheduleOnce`, `L:205-209`); the handle is a shared not-cancellable value: `Cancel()` false, `IsCancelled()` false. An inline executor runs `f` inside the call.
3. **Positive delay: one atomic state.** pending, then executed or cancelled, by compare-and-swap (`L:375-404`). The timer's goroutine swaps pending to executed and only then hands `f` to the executor (`L:383-392`). `Cancel()` swaps pending to cancelled, stops the clock timer and returns true; in any other state it returns false and changes nothing. `IsCancelled()` is true only in the cancelled state, so after execution it is false (SC3o).
4. **Callbacks run on the System's executor, never on the clock's goroutine (C9, O-4 A).** No exported executor. A panic in `f` is recovered, reported once through `Config.Logger` at Error (Go counterpart of `TaskInvocation`'s `Error`, O-6 A; marker `TODO(TIM-01-error-report)` stays), and does not crash the process; a nil `Logger` reports nothing (D-4).
5. **Message form (slice 2).** `f` is `TellFrom(ctx, target, msg, sender)` with `ctx` = `context.Background()`; no liveness check (`S:432-435`). The sender is explicit; `nil` is `noSender` (ADR 0032 D3/D4, `S:428-436` implicit sender). A stopped target gets the ordinary `DeadLetter` with that sender (ORACLE SC5once); a full bounded mailbox follows ordinary destination semantics (issue #821 AC3).
6. **Close (C7, O-5d A).** The scheduler registers every pending task. An internal `close()`: sets `closed` once (idempotent, a second call is a no-op), stops every pending task's clock timer **without** changing its state (so `IsCancelled()` stays false and `Cancel()` stays whatever the state says, `L:170-192`), and runs nothing. After close, `ScheduleOnce` returns `ErrSchedulerClosed`; the check is repeated after the task is registered and, if the scheduler closed in between, the task's timer is stopped and the error returned (`L:218-220`). `TaskRunOnClose` is not ported. The System calls `close()` when its termination finishes (Pekko `registerOnTermination(stopScheduler())`, `ActorSystem.scala:1062`, `stopScheduler` `:1162-1164`), i.e. after the root guardian finalized and before or at the point `WhenTerminated` closes (exact site: the Dev decides with the kit's RULEBOOK; it must not hold a lock and must not block `Terminate`). Until then scheduling is accepted, even after `Terminate` was called, as in Pekko (`Spawn` is the one that returns `ErrSystemStopping`).
7. **What `Cancel` does not do (C8).** It never removes a message already in a mailbox; the test says so (`TimerSpec` constraint P8 of the ADR).

**Access from inside an actor (Owner notice, 2026-11-10).** Pekko reaches the scheduler with `context.system.scheduler` (`ActorCell.scala:187`, `trait ActorContext`); the Typed lane (#1182 slice B, scope in PR #1524) adds the export `Context.System()` to `pekko-actor`. This scope adds **no** `Context.Scheduler()`, no second way to get the System from a context and no scheduler-specific export on `Context`: an actor calls `ctx.System().Scheduler().ScheduleOnce(...)`. `System.Scheduler()` stays, because it is Pekko's `system.scheduler` for code that holds the System (it is not a second export of the same access). `Context.System()` does not exist at `8e334700` (grep of `pekko-actor/actor`). `typed.Context` (`pekko-actor-typed/typed/context.go:16`) does **not** re-export `Context.System()`: this scope adds nothing to `pekko-actor-typed`; how Typed exposes the System is #1182 slice B's and #964's decision. Who introduces `actor.Context.System()` is decision D-10 (resolved: the first of #821 slice 1 and #1182 slice B to merge after #871). **Fact (2026-11-11): #1182 slice B introduced it (PR #1545, `b8639c1a`, with its EXP-1 record); this scope only uses it and adds no export on `Context`.**

Explicit non-goals of this scope: `ScheduleWithFixedDelay`, `ScheduleAtFixedRate` and their periodic handle (#822, #823; the periodic `Cancel` values of O-2 are theirs), `SchedulerTask.cancelled`, a public `Close`, an exported executor, a manual clock (M-04), the `Int.MaxValue` limit, any change to `Ask`.

## 5. Slices (one PR each, at most 400 authored lines, sequential: 2 after 1)

Estimates are the TL's, to be replaced by the Dev's measured numbers in the PR; if a slice would exceed 400 the Dev stops and reports to the TL (the cut line is the next item of the list).

| Slice | Content | Prod | Tests | Docs | Total (est.) |
|---|---|---|---|---|---|
| **1** | `Scheduler`, `System.Scheduler()`, `Cancellable`, `ErrNilCallback`, `ScheduleOnce` (items 1 to 4), its use of `Context.System()` (introduced by #1182 slice B, PR #1545; D-10), the EXP-1 records of the five new exports, the README paragraph "no scheduler exists" replaced | ~125 (~137 with `Context.System()`) | ~195 (~220 with its in-handler test) | ~25 | **~345; #1545 introduced `Context.System()`, so the ~382 branch does not apply** (Prod + Tests + Docs; the close wiring is only in row 1b) |
| **1b** | `ErrSchedulerClosed`, the registry of pending tasks, internal `close()` and its call at System termination (item 6), T11 to T13. **Merges immediately after slice 1** (nothing else between them) | ~25 | ~35 | 0 | ~60 |
| **2** | `ScheduleTellOnce` (item 5), the oracle-based tests SC5once, the executor-hop measurement (D-6), README | ~40 | ~120 | ~30 | ~190 |

**Recount (TL, 2026-11-10).** The earlier estimate, ~405 with the close wiring, becomes **~442 if slice 1 also introduces `Context.System()`** and ~405 if not. Both exceed or touch 400, so, with the Owner's D-1 A ("slice 1b if needed"), the close wiring (item 6, T11 to T13) is **slice 1b by plan**, merged immediately after slice 1 and before slice 2; between them the scheduler is not closed at `Terminate`, the gap `TODO(TIM-13-close)` of ADR 0039 O-5d option B, which exists only for that interval and is closed by 1b. The Dev replaces these estimates with measured authored lines in each PR; if slice 1 alone passes 400 the Dev stops and reports to the TL.

## 6. Criteria → scenarios (Go unit, real runtime, a caller-written manual clock double, go-specs and spies, channels and gates, no sleeps)

Public tests in a new `package actor_test` file; the manual clock is the one `clock_api_test.go` already writes (copied or shared in a `_test` helper, the Dev's choice). Level: unit for all.

| # | Criterion (issue #821 AC and reconciliation) | Scenario | Pekko source | Evidence |
|---|---|---|---|---|
| T1 | AC1: a non-cancelled task runs exactly once after its due point under an injected clock | advance `delay - 1`: not run (observed through a gate); advance 1: run once; advance far: still once | `SS:156-175` never early | ORACLE-shaped, Go test |
| T2 | AC2: cancel before dispatch prevents the run | `Cancel()` true, `IsCancelled()` true, second `Cancel()` false, `IsCancelled()` true; advance past due: not run | `SS:127-133` | ORACLE SC3o (a) |
| T3 | reconciliation AC: cancel after execution | `Cancel()` false, `IsCancelled()` false, twice | `SS:146-154` | ORACLE SC3o (b) |
| T4 | cancel while the callback runs | `Cancel()` false, `IsCancelled()` false; the callback finishes | `L:383-392` | ORACLE SC3o (c) |
| T5 | zero and negative delay | `0`, `-1s`, `-1ns`: no error, handle with both false, `f` ran; with an inline executor it ran before `ScheduleOnce` returned; the Go executor seam used | `L:205-209`, `SS:527` | ORACLE SC1 |
| T6 | `+1ns` control | a real, cancellable handle (`Cancel()` true before the clock fires) | `L:211-222` | ORACLE SC1 control |
| T7 | callbacks leave the clock's goroutine | the manual clock fires on the test goroutine; `f` runs through the executor seam, not inline in `Advance` | `L:383-392` | Go-only, **pending approval** (D13) |
| T8 | a panicking callback | recovered, one Error entry on a spy `Logger`, the System keeps working; nil `Logger`: no entry, no crash | `AbstractDispatcher.scala:55-63`, SC4 | ORACLE-shaped (reported once), Go form pending approval |
| T9 | AC3 (slice 2): stop or overflow at delivery follows ordinary destination semantics | target stopped before the due point: one `DeadLetter`, sender as given, `deadLetters` when `nil`; bounded full mailbox: the ordinary drop | `S:432-435`, SC5once | ORACLE SC5once |
| T10 | sender explicit (slice 2) | with a sender: the actor reads it with `Context.Sender`; with `nil`: absent | ADR 0032 D3/D4 | Go-only, pending approval |
| T11 | close | `close()` twice is a no-op; a pending task never runs (advance past due after close) and its `IsCancelled()` stays false; `ScheduleOnce` after close returns `ErrSchedulerClosed` and runs nothing | `L:170-192`, `SS:661-697` | READ only, Go form pending approval |
| T12 | close races a schedule | a gate between "registered" and "re-check" (internal seam allowed in an internal test) closes the scheduler: the call returns `ErrSchedulerClosed` and the timer is stopped | `L:218-220` | READ only |
| T13 | System termination closes the scheduler | `Terminate`, wait `WhenTerminated`: `ScheduleOnce` returns `ErrSchedulerClosed`; a pending task armed before did not run | `ActorSystem.scala:1062, 1162-1164` | READ only |
| T14 | `Cancel` does not retract a mailbox message | `ScheduleTellOnce` fires, message in a held mailbox, then `Cancel()` false and the message is still delivered | ADR 0039 C8, P8 | Go-only |
| T15 | race | `Cancel` concurrent with the clock's fire: exactly one of run and cancel wins, over a repeated loop under `-race` | `L:375-404` | READ; stress, Go-only |
| T16 | `ErrNilCallback`, nil callback (slice 1) | `ScheduleOnce(d, nil)` for `d` = 1s, 0 and -1s returns `(nil, ErrNilCallback)`; checked before the delay rule; the manual clock records zero `AfterFunc` calls and the executor seam zero `Execute` calls; the System keeps working | none (a null `Runnable` fails when it runs; marker `TODO(TIM-02-nil-callback)`) | Go-only, **pending approval**; designed mutation: drop the nil check, so the 1s case arms a timer (clock sees an `AfterFunc`) and the 0 case panics in the executor |
| T17 | `ErrNilCallback`, nil target (slice 2) | `ScheduleTellOnce(sch, d, nil, msg, sender)` returns `(nil, ErrNilCallback)`; nothing armed, nothing sent, no dead letter | same | Go-only, **pending approval**; designed mutation: drop the nil-target check, so the call arms a timer and the fire sends to a nil `Ref` |

Each test names a mutation that must fail it (the kit's RULEBOOK requires a designed pair): never-early off by one, `Cancel` after execute returning true, `IsCancelled` true after execute, zero delay returning a cancellable handle, close running the pending tasks, callback run inline on the clock goroutine. The mutation designs are the Dev's in the RULEBOOK; a mutation of `pekko-actor` that is also a judge mutation is updated per section 7.

## 7. Rules that bind this scope because it touches `pekko-actor/` (outside `*.md`)

- **Anchors.** The slices add files and change `system.go` and the termination path; `judge/mutate_go.py` anchors none of `system.go`, `system_shutdown.go`, `executor.go`, `clock.go`, so no anchor is expected to break. The Dev still runs `python3 scripts/check-judge-matrix.py` on the head. If a PR breaks an anchor it updates `judge/mutate_go.py` in the same PR with the same contract. The QA runs `scripts/check-judge-matrix.py` on the head; exit 1 is REJECT.
- **Gates on the head:** `gofmt`, `go vet`, `go test -race`, 85 % coverage for `pekko-actor` (`scripts/check_coverage.sh <dir> 85`), diff coverage 85 %, `gocyclo` ≤ 10 (`golangci-lint`; no `//nolint`). Unit tests run without network. `scripts/check_deps.sh` green: `actor` imports no goPekko module (the scheduler adds none).
- **Ported headers.** A Go file ported from `Scheduler.scala` or `LightArrayRevolverScheduler.scala` carries the Apache-2.0 notice and one `// Ported from upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/<file>.scala` line (`.github/scripts/check-headers.sh`).
- **After each merge:** the full Go campaign on this Mac (`--runs 30`, `-race`) on a clean `develop`, report as is, PROVENANCE, package in `goPekko-evidence/` (`COPYFILE_DISABLE=1`, `verify_package.py`, `.sha256`, `SHA256SUMS`) and tag `evidence/go-campaign-<sha7>`. If slice 1 and #871 merge together, one campaign over both is enough. Meanwhile the inventory carries the dated note that V2/V3 are not in force.
- **`pekko-actor/README.md` is also touched by #871** (and by #1479 later). Merge in the order the Owner sets; the later one rebases and shows that its own delta is identical (a conflict that forces new content means re-QA). Slice 1 replaces `README.md:302` and `README.md:306-307` ("no scheduler or `Cancellable` API exists") and states: implemented (`ScheduleOnce`, `Cancellable`, `Context.System()` if slice 1 introduces it), **not implemented: the close of the scheduler at System termination and `ErrSchedulerClosed` (slice 1b, `TODO(TIM-13-close)`; until then the scheduler is not closed at `Terminate`)**, `ScheduleTellOnce` (until slice 2), fixed delay, fixed rate, keyed timers; the two ACCEPTED PORT GAPs; and that the Go-only rows are pending approval. Slice 1b moves "close" to implemented; slice 2 adds the message form.
- **Inventory.** The row of the scheduler (C-20 in `docs/parity/actor-inventory.md`, the cumulative rule) is updated in slice 2 with what the tests prove; the rows with no applicable oracle stay `pending approval` (ADR 0033 D13, ADR 0039 O-9) until the Owner approves them one by one.

## 8. Exports and EXP-1 records (ADR 0011 decision 2; one per name, in the PR body)

| Export | Consumer (real) | Why the public API is not enough | Does not expose | Test |
|---|---|---|---|---|
| `System.Scheduler()` | the application author and, later, #860 (keyed timers) and Typed `Behaviors.withTimers` in `pekko-actor-typed` | the scheduler is owned by the System and no other door reaches it (O-5 A) | `Close`, the clock, the executor, the registry | T1, T13 |
| `Scheduler` (type) | same | the receiver of `ScheduleOnce` | a constructor: it cannot be created outside a System | T1 |
| `Scheduler.ScheduleOnce` | the application author; `pekko-actor-typed` for `scheduleOnce` (J-04a, #964) | issue #821's user story: no hand-written timer goroutine | tick, executor parameter, `SchedulerTask` | T1 to T8 |
| `Cancellable` | same; #860 holds it by the interface | O-1 A: the trait of Pekko, the testkit can fake it | `SchedulerTask`/`cancelled` (#1077) | T2 to T4 |
| `ErrSchedulerClosed` (**slice 1b**, with T11, T13) | the caller of a schedule call after termination | the caller must tell "closed" from other failures | the cause of the close | T11, T13 |
| `ErrNilCallback` | the caller | a nil callback or target cannot be told from a scheduler failure otherwise | the cause | T16 (slice 1), T17 (slice 2) |
| `ScheduleTellOnce` (slice 2) | the application author; Typed `scheduleOnce` | methods cannot be generic over `Ref[T]` | the executor | T9, T10, T14 |

`Context.System()` and its single EXP-1 record (D-10). **Fact (2026-11-11): the record was carried by #1182 slice B (PR #1545, `b8639c1a`); slice 1 of this scope carries none and its PR body says "uses `Context.System()` introduced by #1545".** The first PR to merge **after #871** between #821 slice 1 and #1182 slice B introduces `func (c *Context[T]) System() *System` and carries the only record; the other rebases and only uses it, and its PR body says "uses `Context.System()` introduced by #<n>" without a record. If #821 slice 1 is the introducer, the record is: symbol `Context.System`; consumer `ctx.System().Scheduler().ScheduleOnce(...)` in this scope and Typed `context.system.scheduler`/`context.system.deadLetters` (#1182 slice B); why the public API is not enough: a handler holds only its `*Context`, and the System is not reachable from it; what it does not expose: the registry, executor, clock and every other System seam (it returns the same `*System` the application already holds); test: T1 run from inside a handler through `ctx.System().Scheduler()`; Pekko: `ActorContext.system`, `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:187` (the `implicit def system` of `trait ActorContext`; `ActorCell.scala:261` is the cell's own `def system`). The scheduler is reached by `system.scheduler`, which is `System.Scheduler()`; there is no `Context.Scheduler()`.

Honesty line: no module in the repository consumes these names yet (`pekko-actor-typed` is a future consumer, not a dependency); EXP-1 allows justifying the export by the story's user and approves no export; the Owner approved the names at G2 (D-2).

## 9. Decisions (approved by the Owner at G2, 2026-11-10; all resolved)

| # | Decision | Resolution (Owner) |
|---|---|---|
| D-1 | Order of the close wiring (ADR 0039 O-5d) and shape of slice 1 | **A**: the close and its call at System termination are part of #821, as **slice 1b** if slice 1 exceeds 400 lines (section 5: planned as 1b) |
| D-2 | Exported names and the "no consumer yet" line (section 8) | Approved: `System.Scheduler`, `Scheduler`, `Scheduler.ScheduleOnce`, `Cancellable`, `ErrSchedulerClosed`, `ErrNilCallback`, `ScheduleTellOnce` |
| D-3 | Signature shape | **A**: callback `func()`, return `(Cancellable, error)` |
| D-4 | Panic report when `Config.Logger` is nil | **A**: silent, documented in the README, with the marker `TODO(TIM-01-error-report)` |
| D-5 | Nil callback or target | **A**: `ErrNilCallback`, marker `TODO(TIM-02-nil-callback)` |
| D-6 | The message form sends through the executor | **A**: always hop; slice 2 records one informational measurement |
| D-7 | Sender argument of `ScheduleTellOnce` | **A**: explicit `sender Handle`, `nil` is absent |
| D-8 | Evidence acceptance (ADR 0033 D13, ADR 0039 O-9) | As written in section 6: ORACLE rows are Pekko evidence; Go-only and READ-only rows (T7, T10, T11 to T15, the error returns) stay `pending approval` one by one |
| D-9 | Order on `pekko-actor/README.md` | **#871 first**; #821 rebases and shows an identical delta |
| D-10 | Who introduces `Context.System()` | The first of #821 slice 1 and #1182 slice B to merge **after #871** introduces it with the single EXP-1 record (section 8); the other rebases and only uses it. So the scheduler does not wait for the Typed lane: slice 1 includes `Context.System()` and its record, conditioned on arriving first. The second PR shows that its own delta is identical after the rebase; a conflict that changes content means re-QA. **Fact (2026-11-11): #1182 slice B arrived first (PR #1545, `b8639c1a`); slice 1 of #821 only uses `Context.System()`** |

## 10. Independence and the Go files

| Lane | Go files (non-test) it touches | Shared with another lane? |
|---|---|---|
| #821 slice 1 (and 1b) | **new** `pekko-actor/actor/scheduler.go`, **new** `pekko-actor/actor/scheduler_once.go`, `pekko-actor/actor/system.go` (field, accessor, init), the System termination path (`pekko-actor/actor/guardian.go` or `stop.go`/`system_shutdown.go`: the Dev picks the one site; if the close goes in `stop.go`, that file carries 6 anchors of `judge/mutate_go.py`: re-grep every anchor of the file and run `scripts/check-judge-matrix.py` on the head before the PR, and update any broken anchor in the same PR with the same contract) | `system.go` and the termination path may be touched by #871 only if #871 adds a System field (to be confirmed from its scope); `README.md` yes (section 7) |
| #821 slice 2 | **new** `pekko-actor/actor/scheduler_tell.go` | no |
| #871 (EventStream lane) | `pekko-actor/actor/eventstream*.go`, `deadletter.go`, `poison_pill.go`, `ref.go`/`watch.go` (suppression marker) | `README.md`; the Go files are disjoint from this scope except a possible `system.go` field |
| #927 (Typed lane) | `pekko-actor-typed/` | none |

**Overlap with the Typed lane (corrects the table above, which said "none"):** the Go files of #1182 slice B include the one that defines `Context` in `pekko-actor/actor/` and adds `Context.System()`. #821 slice 1 touches that file only if it is the first to merge after #871 (D-10); otherwise it only calls the symbol. So the two lanes are **sequenced, not parallel, on that file**: the order is D-10, and the second one rebases and shows its delta unchanged (done: #1545 landed first, and slice 1 of #821 rebased on it without touching `actor.go`). `Context.System()` also gives the Typed `context.system.deadLetters` and `scheduler` accessors one shared door.

Tests (new files, not shared): `pekko-actor/actor/scheduler_api_test.go`, `scheduler_internal_test.go`, `scheduler_tell_api_test.go`.

**Allowed files for the Dev (all slices):** the files above, plus `pekko-actor/actor/actor.go` (where `Context` is defined, `actor.go:47`) **only if slice 1 is the first to merge after #871 and so introduces `Context.System()` (D-10)**, plus `pekko-actor/README.md`, `docs/parity/actor-inventory.md` (slice 2 only), `docs/adr/0039-...md` only for the README-note line that "no scheduler exists yet" if the ADR carries it (it says so in its Consequences). Nothing else may change; `factory/`, `AGENTS.md` and `.gitignore` of the Owner's WIP never enter a PR.

## 11. Appetite

Two PRs, each at most 400 authored lines. Slice 1 is a day of Dev plus the kit run; slice 2 half a day. The QA is independent, in a clean context, bound to the final head; any change after APPROVE, even of wording, needs a new QA.

## 12. Definition of Done

- Every criterion of section 6 has its scenario, passing, and its designed mutation caught.
- `gofmt`, `go vet`, `go test -race`, `scripts/check_coverage.sh` at 85 % for `pekko-actor`, diff coverage 85 %, `gocyclo` ≤ 10 on every new or modified function, `scripts/check_deps.sh`, `python3 scripts/check-judge-matrix.py` (exit 0) and `check-headers.sh`, on the head.
- README and inventory say what is implemented, what is not and what is pending approval, with links to the tests.
- After the merge: the Go campaign of section 7 with its evidence package and tag.
- `gh pr merge` is the Owner's; the PR names the head the QA approved.
