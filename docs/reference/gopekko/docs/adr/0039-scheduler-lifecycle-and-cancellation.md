# ADR 0039: Scheduler lifecycle and cancellation of `pekko-actor`

- Status: **Accepted** by the Owner on 2026-11-10: O-1 to O-10 as recommended, with the precisions of the Owner table below (O-2, O-3, O-6, O-7, O-9). Issue #776, `TIM-01`. No Go is produced. The exact values of O-2 and O-3 are pinned by the oracle run `docs/parity/oracles/scheduler-lifecycle-2026-11-10.md` (section "Oracle results").
- Date: 2026-11-09
- Stage: 1
- Principle applied: ADR 0033 D14 (behave exactly as Pekko), D14.3 (a deviation needs its own ADR: the deviations of this ADR are marked "Deviation"; the Owner accepted them on 2026-11-10 (O-7, ACCEPTED PORT GAP)), D13 (Go-only rows need alternative evidence and Owner approval), D5 (ADR impact list).
- Related: ADR 0011 (exports; Owner decision 2 and EXP-1, `docs/parity/typed-owner-decisions-2026-10-27.md:36`), ADR 0033 (D3, D4: timers are mandatory), ADR 0010 (timers listed as "Excluded for now", superseded for timers by ADR 0033 D4), ADR 0038 (precedent for errors instead of exceptions, O-5), `docs/scopes/1129-injectable-clock.md` (the clock seam this ADR builds on).
- Evidence: Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`) source and tests **read**, and, for the values of O-2, O-3, O-6 and O-8, **executed** by the oracle [`scheduler-lifecycle-2026-11-10.md`](../parity/oracles/scheduler-lifecycle-2026-11-10.md) (harness package `scheduler`, 21 scenarios, three runs each, Java 1.8.0_504; PRs #1506 and #1507; see "Oracle results"). **Still read, not run:** the close of the scheduler (P5: pending tasks, rejection after close, `TaskRunOnClose`) and the rounding and tick limits of O-7 (Deviations 1 and 2). "Read" means opened in `upstream/pekko` at the pin. Where Pekko's documentation and its code disagree, both are cited.

## Feature

Who owns the scheduler of a `System`, what the three schedule operations promise, what a `Cancellable` promises, what happens to pending and new tasks when the scheduler closes, and which seams (clock, executor) a Go scheduler uses. The ADR specifies the contract only; the implementations are #821 (one-shot), #822 (fixed delay), #823 (fixed rate), #1131 (close on `System` termination), #860 (keyed actor timer).

## Context

`pekko-actor` has no scheduler. The only timer it arms is the `Ask` deadline, through `Config.Clock` (`Clock{Now, AfterFunc}`, `Timer{Stop() bool}`, #1129; `pekko-actor/actor/clock.go`, `system.go:99-102`, `README.md:298-305`). `README.md:303` states "no scheduler or timer other than Ask's is armed". The default executor runs each activation on its own goroutine (`pekko-actor/actor/executor.go`: `goroutineExecutor.Execute` is `go f()`); the `executor` interface is unexported and replaced only by tests (`system.go:27,110`). `System.Terminate`, `WhenTerminated` and `Shutdown(ctx)` exist (`system_shutdown.go:30,38,54`) and close no scheduler because none exists.

ADR 0033 D4 makes timers mandatory. Five stories wait for a contract: #821, #822, #823, #1131, #860. Each of them cites `Scheduler.scala` by file; the ACT2 reconciliation of #776 adds scheduler close, non-positive intervals and periodic failure as contract points.

## How Pekko solves it

- Module: `pekko-actor`.
- Concept: `Scheduler` is a trait with three schedule families (`Scheduler.scala:61`). The only production implementation is `LightArrayRevolverScheduler` (LARS): one timer thread advances a hashed wheel by ticks (`tick-duration = 10ms`, `ticks-per-wheel = 512`, `reference.conf:900,912`) and **does not run user code**: it hands each due task to the `ExecutionContext` given at schedule time (`LightArrayRevolverScheduler.scala:383-392`). `Cancellable` is a trait (`Scheduler.scala:525`).
- Files: `actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala`, `LightArrayRevolverScheduler.scala`, `dungeon/TimerSchedulerImpl.scala`; tests `actor-tests/src/test/scala/org/apache/pekko/actor/SchedulerSpec.scala`, `TimerSpec.scala`. Paths below are relative to those directories; "S:" is `Scheduler.scala`, "L:" is `LightArrayRevolverScheduler.scala`, "SS:" is `SchedulerSpec.scala`, "TS:" is `TimerSpec.scala`, "TI:" is `TimerSchedulerImpl.scala`.

### P1. The three contracts are separate

| | `scheduleOnce` | `scheduleWithFixedDelay` | `scheduleAtFixedRate` |
|---|---|---|---|
| Runs | once after `delay` (S:428-436, 479) | after `initialDelay`, then `delay` **after the previous run finished** (S:83-105: the next `scheduleOnce(delay, this)` is called after `runnable.run()` returns) | after `initialDelay`, then every `interval`, compensating drift: next delay is `max(interval - drift, 1ns)` (L:134-161, `driftNanos` at L:146) |
| Long run | n/a | frequency slightly below `1/delay` (S:69-73) | frequency exactly `1/interval`; after a stall the missed runs are executed back to back, a burst (S:279-293, 203-218) |
| Overlap | n/a | none | none: the next run is scheduled only after the current one returns (L:146-148); a run longer than the interval is followed immediately by the next |
| Non-positive interval | n/a | `IllegalArgumentException` (L:127-129, `checkPeriod` L:224-228; S:78-79) | same (L:134-136; SS:701-717) |
| Non-positive initial delay | see P3 | passes to `scheduleOnce`, so runs at once (`Duration.Zero` is the idiom, SS:235, S:141-142) | runs at once (`schedule(..., roundUp(initialDelay))`, L:159, then L:206-209) |
| Test of the contract | SS:127-154 | SS:450-461 (125ms + 100ms work keeps 4.4/s), SS:721-737 | SS:422-445 (rate 40/s +-4, long task does not shift it), SS:701-717 |
| Shared tests | SS:216-418 run both periodic variants through the same adapter | | |

The deprecated `schedule` is the fixed-rate implementation (S:229-231, 298-306, 351-364). It is not a contract of this ADR.

### P2. `Cancellable`

- `cancel(): Boolean` "returns true if that was successful. If this cancellable was (concurrently) cancelled already, then this method will return false although `isCancelled` will return true" (S:527-534). `isCancelled` is "true if and only if this Cancellable has been successfully cancelled" (S:536-541). "No strict guarantee that the implementation is thread-safe, but it should be good practice to make it so" (S:520-523).
- **One-shot** (`TaskHolder`, L:372-415): one atomic swap of the task slot. `cancel()` is true only if the swap found a task that had neither run nor been cancelled (L:398-404); otherwise false. A cancelled holder reports `isCancelled` true; an executed one reports false. SS:127-133: first `cancel()` true, then `isCancelled` true, second `cancel()` false. SS:146-154: after execution `cancel()` false and `isCancelled` false, twice. Exactly one of "run" and "cancel" wins the swap, and a task already handed to the executor cannot be recalled.
- **Periodic** (`AtomicCancellable`, S:573-609): the handle owns an atomic reference to the *current* one-shot. `cancel()` calls the current task's `cancel()`, then clears the reference with a compare-and-set; it returns true when **this call cleared the reference** (S:594-605), including when the current holder had already executed (`c.cancel()` false, then `compareAndSet(c, null)` true); and false when `c.cancel()` succeeded but the compare-and-set then fails because a concurrent re-arm replaced `c` (S:599). READ only: no spec in SS pins this return value for periodic tasks (SS:316-346 ignore it). A second `cancel()` returns false; `isCancelled` is true once the reference is null (S:607).
- **Race with the reschedule**: the running task reschedules only if the reference is still non-null (S:92-93; L:147), and `swap` cancels the new task if it finds the reference already null (S:585-592). So a `cancel()` concurrent with a run cannot leave a live repetition, and a callback already running is not interrupted and finishes. SS:316-330 (cancel during the initial delay: 0 runs) and SS:332-346 (cancel after the first run: exactly 1 run).
- **Cancelling does not retract a message already in the mailbox**: TS:224-243 ("replace timer": the next tick was already enqueued and must be discarded by the owner), TS:268-289 (old incarnation).
- `SchedulerTask.cancelled()` (S:511-517, notified through the executor, L:406-412) is #1077's decision, not this ADR's.

### P3. Delay, interval and range rules

- `scheduleOnce` documents `IllegalArgumentException` for a zero or negative delay (S:423-424, repeated at S:441-442, 460-461, 474-475, 485-486). **The code does not do that**: the delay is rounded up to a tick multiple (L:93-97, 165) and `schedule` runs a task with `delay.length <= 0` on the executor at once and returns `NotCancellable` (`cancel()` false, `isCancelled` false; L:205-209, 420-424). SS:527-530 depends on it (`scheduleOnce(Duration.Zero, ...)` succeeds until the scheduler closes). Doc and code disagree; the code is what a run observes.
- Delays whose tick count exceeds `Int.MaxValue` are rejected with `IllegalArgumentException` for one-shot, initial delay and interval (L:230-235; SS:477-484, 743-760). A delay of `tickDuration * Int.MaxValue` is accepted.
- Accuracy: a task never fires early (SS:156-175, "never fire prematurely" with a 20ms delay), may fire up to a tick late, and the one-shot delay is rounded **up** to a whole tick without reading the clock at scheduling time (L:44-48, 93-97). LARS reads its clock (`System.nanoTime`, monotonic; replaceable for tests, L:99-103) not at scheduling time for the one-shot (L:93-97), but the fixed-rate schedule does read it on the caller's thread (L:142) and again on the executor thread (L:146).

### P4. A periodic task that fails stops

- "If the `Runnable` throws an exception the repeated scheduling is aborted, i.e. the function will not be invoked any more" (S:75-76, 120-121, 220-221, 256-257). The reschedule statement sits after `runnable.run()` inside the `try` (S:90-93; L:144-148), so an exception skips it.
- A `SchedulerException` (failed to enqueue because the scheduler is closed, or "timer active for terminated actor", S:164-166) is swallowed: the repetition ends silently (S:96-97 for fixed delay; L:150 for fixed rate and for the deprecated `schedule` forms, which are the fixed-rate path, S:229-231, 351-364). *Corrected by the oracle (2026-11-10, section "Oracle results")*: an `IllegalStateException` caused by a `SchedulerException` is swallowed by fixed delay (S:96-97) but **not** by fixed rate, which catches only `SchedulerException` (L:150) and reports the wrapper. Any other exception propagates out of the task into the executor that runs it; on the system's dispatcher it is reported once (`EventFilter[Exception]("TEST", occurrences = 1)`, SS:270-282; on the first run and on a later run, SS:285-314), on other executors by their own mechanism (oracle SC4).
- Receiving actor stopped: SS:252-267 (no more messages after `system.stop(actor)`).

### P5. Close

- `close()` (L:170-192) flips the `stopped` reference once (L:237-247); the timer thread notices at its next tick, drains the wheel and the queue and completes the promise (L:345-350, 256-264); `close()` blocks up to the scheduler's `shutdown-timeout` (default `5s`, `reference.conf:927` inside `pekko.scheduler`; the `1s` of `reference.conf:587` belongs to the dispatcher and is not this one; L:89, 113, 181). A second `close()` finds nothing (L:246).
- **New tasks are rejected** with `SchedulerException("cannot enqueue after timer shutdown")` (L:206-207, 210-211) **also after the task was queued**: the enqueue re-checks `stopped` and cancels the task it just queued (L:218-220). `scheduleOnce` turns it into `IllegalStateException` (L:163-168); a periodic schedule does the same at creation (S:577-581); a *running* periodic task that fails to re-enqueue ends silently (P4).
- **Pending tasks are dropped, not run and not cancelled**: only a task that is a `Scheduler.TaskRunOnClose` runs, synchronously on the closing thread, with `SchedulerException` ignored and other `NonFatal` logged (L:172-190). A plain pending task is neither run nor given `cancelled()`, and its `isCancelled` stays false (nothing swaps its slot). `TaskRunOnClose` "is not intended to be used by end user applications" (S:561-567); it exists for dispatcher shutdown.
- Tests: SS:661-682 ("reliably reject jobs": every accepted task ran exactly once, and the loop stops at the 201st failure; the test asserts exactly 200 failures (`failure.size == 200`) and that every accepted task ran (`counter == success.size`)), SS:684-697 (a 10s plain task does not run on close, the `TaskRunOnClose` one does), SS:520-548 (a million accepted `TaskRunOnClose` tasks all run on close, none twice, while another thread keeps enqueueing until rejected).
- If the timer thread dies it is restarted unless the scheduler is stopped (L:288-310). Closing on `ActorSystem` termination is #1131.

### P6. Messages to a stopped receiver

`scheduleOnce(delay, receiver, message)` is `receiver ! message` on the executor with no liveness check (S:432-435). The two periodic message forms send, then throw `SchedulerException("timer active for terminated actor")` if `receiver.isTerminated` (S:162-167, 358-363), which P4 swallows. So the last message to a stopped receiver is sent (inferred, not read: a send to a terminated local ref becomes a dead letter) and the repetition then stops without a log.

### P7. Executor

Every schedule method takes the `ExecutionContext` that will run the callback; the LARS thread only enqueues (L:383-392; an exception from `execute` goes to `reportFailure` and the task is lost). Actor timers pass `ctx.dispatcher` (TI:103-107), so a timer message is sent from a dispatcher thread, not from the timer thread. The `delay <= 0` path calls `ec.execute` from the caller's thread (L:208).

### P8. Keyed actor timers (the constraints on #860; not specified here)

`startTimer` first cancels the timer with the same key (TI:88-92), then takes a new generation (TI:93, 65-68) and stores `(key, msg, repeat, generation, task)` (TI:110-111). The scheduled payload carries `(key, generation, owner)` (TI:95-99). On receipt (TI:136-161) a message whose key is absent, whose owner is an old instance (after restart) or whose generation is not the current one is discarded; a single-shot timer is removed when its message is accepted (TI:149-150). `cancel` and `cancelAll` call the handle's `cancel()` and drop the entries without reading its result (TI:124-134). Consequence for this ADR: **the scheduler gives no ordering guarantee between `Cancel` and a delivery already in flight; the filter lives above it** (C8).

## How GoAkt solves it

Not consulted. CLAUDE.md allows GoAkt only from an ADR that shows it solved a feature better; this ADR has no such evidence, so Pekko decides every point.

## Evidence

- Benchmark: none. Judge: no file in `judge/scenarios/` mentions a scheduler (the Pekko harness uses `ExplicitlyTriggeredScheduler` only as its controlled clock, `judge/pekko-harness/Judge.scala:8,199`); the plan row B-02 lists "scheduler cancel" among the missing scenarios. **Every Pekko rule above is READ, not run**; ADR 0033 D13 applies (see O-9).
- Complexity: one timer per task on top of `Clock.AfterFunc` instead of a wheel (see O-7); one atomic reference per periodic handle.
- Idiomatic in Go: a `Timer`-per-task design is idiomatic (`time.AfterFunc`); a hashed wheel is not needed.

## Decision (accepted contract, 2026-11-10; each line follows its O-n as recommended, with the Owner's precisions)

Each point is Pekko's behavior unless it says "Deviation" or "Go form". Go signatures belong to the stories, except where an O-n decides the shape.

- **C1. Three operations, three contracts** exactly as P1. No shared "schedule" that hides the difference. Fixed-rate drift is computed from the System's `Clock.Now` (Go form of `clock()`, L:146) and its next timer is armed with `Clock.AfterFunc` for `max(interval - drift, 1ns)`.
- **C2. One-shot cancel** exactly as P2: first `Cancel` that wins returns true and `IsCancelled` is true; every later call false; after execution both false.
- **C3. Periodic cancel**: stops the repetition; no further run starts; a run in progress finishes; a `Cancel` concurrent with the reschedule cannot leave a live task (P2). Return value: see O-2.
- **C4. Non-positive and out-of-range values** per P3: interval at or below zero is rejected for both periodic forms (O-3); one-shot at or below zero runs at once (O-3b); a zero or negative initial delay runs the first execution at once (O-3c; for fixed rate a negative initial delay is followed by a catch-up burst that the Go scheduler **replicates as Pekko measured it** (Owner, 2026-11-10); #823 carries a test based on the oracle, SC2fr `[019]-[021]`); the `Int.MaxValue`-ticks limit is not reproduced (Deviation 2).
- **C5. Never early**: no callback starts before `delay` has elapsed on the System's clock; late is allowed. Tick rounding is not reproduced (Deviation 1, O-7).
- **C6. Periodic failure**: a callback that panics ends the repetition and is reported once (O-6). A failure to enqueue and a stopped receiver end it silently (P4, P6).
- **C7. Close**: new schedules after close are rejected with a documented error (O-5), including a race with close (the check is repeated after the task is registered, L:218-220); pending tasks never run and their `Cancel` result is not specified beyond "false is never reported as having prevented a run that happened"; a second close is a no-op; `TaskRunOnClose` is not ported (O-5). Closing is idempotent and bounded.
- **C8. No retraction**: `Cancel` never removes a message already enqueued in a mailbox. A keyed timer (#860) discards stale deliveries by generation, as TI:136-161.
- **C9. Callbacks run on the System's executor, not on the timer's goroutine** (O-4).
- **C10. Stopped receiver** per P6 (O-8).

## Decisions for the Owner

Decided by the Owner on 2026-11-10: each "Recommendation" below is the accepted decision, except where an "Owner, 2026-11-10" line adds a precision. "Evidence" cites Pekko.

### O-1. The Go shape of `Cancellable`

- Question: what does a schedule operation return?
- Options: **A** an exported interface `Cancellable { Cancel() bool; IsCancelled() bool }` (Pekko's trait, S:525; the precedent of `Timer` in #1129). **B** a pointer to an exported struct with unexported fields (users cannot implement it; Typed and testkit cannot fake it). **C** only a `func() bool` cancel (loses `isCancelled`, which SS:127-154 asserts).
- Recommendation: **A**, with exactly these two methods and no `SchedulerTask`/`cancelled` (#1077). It lets #860 and the testkit hold the handle through the interface and keeps parity with the trait.
- Evidence: S:525-542; SS:127-154. Needs an EXP-1 record in #821 (each new export is justified in the story that introduces it).

### O-2. What `Cancel` returns for a periodic handle

- Question: P2 shows the periodic `cancel()` returns true whenever it is the call that clears the reference, even if the last run already executed; the plan row C-02 phrases the rule as "true only if it prevented execution" (`docs/actor-typed-delivery-plan.md:273`), which holds for one-shot only.
- Options: **A** reproduce Pekko (true = this call stopped the repetition; false on the second call). **B** true only if a future run was actually prevented (a deviation: differs after an aborted periodic task and while the current holder is executed).
- Recommendation: **A** (ADR 0033 D14), after the Owner approves running one oracle scenario on Pekko for periodic `cancel()` (first and second call, before the first run, during a run, after abort). Until then the contract text stays "true if this call stopped the repetition" and carried the marker `TODO(TIM-01-periodic-cancel-result)`, **closed on 2026-11-10 by the oracle** (section "Oracle results").
- Evidence: S:594-607; SS:316-346 (return value not asserted). READ only.
- **Owner, 2026-11-10:** what rules is Pekko's **code** (`Scheduler.scala:594-607`), not its documentation. The exact values (the result of `Cancel` on a periodic task that already ran, a second call, after an aborted task) are bound to an oracle run on Pekko before #821, on Java 1.8, by the assistant on the Owner's Mac: `docs/parity/oracles/scheduler-lifecycle-2026-11-10.md`. Until that record is merged the marker `TODO(TIM-01-periodic-cancel-result)` stays.

### O-3. Invalid delays: error or panic, and the one-shot zero

- Question (a): periodic interval at or below zero. Pekko throws `IllegalArgumentException` (L:224-228; SS:701-737). Options: **A** return an error (e.g. a sentinel `ErrInvalidInterval`; name for the story), nothing scheduled, as ADR 0038 O-5 does for a nil subscriber. **B** panic. Recommendation: **A**; never a panic inside the scheduler.
- Question (b): one-shot delay at or below zero. Options: **A** run at once, handle with `Cancel` false and `IsCancelled` false (what the code does, L:205-209, 420-424; SS:527). **B** reject, as the documentation says (S:423-424). Recommendation: **A** (the code is the observed behavior; D14.2 puts observation above documentation), but this is exactly a doc/code conflict, so the oracle scenario of O-2 should include `scheduleOnce(Duration.Zero)` and `scheduleOnce(-1.second)`. Marker `TODO(TIM-01-once-nonpositive)` **closed on 2026-11-10 by the oracle** (section "Oracle results").
- **Owner, 2026-11-10 (O-3):** what rules is the code (`LightArrayRevolverScheduler.scala:205-209`, one-shot with a delay at or below zero) and `Scheduler.scala:594-607`, not the documentation. The exact values (the one-shot with a zero or negative delay, intervals at or below zero, a zero initial delay) are bound to the same oracle run before #821.
- Question (c): a negative or zero **initial** delay for the periodic forms: runs at once (read from L:159, 206-209; not in a spec except `Duration.Zero`, SS:235). Recommendation: allow; the oracle run has been made (section "Oracle results"). **Decided (Owner, 2026-11-10):** the oracle showed that, for fixed rate, a *negative* initial delay produces a catch-up burst (observed: runs 2 and 3 came 8 to 10 ms after the first); the Go scheduler **replicates Pekko as the oracle measured it**, with a test based on the oracle in #823 (scenario SC2fr of the record, `[019]-[021]`). The `TODO` marker that tracked this point is resolved and removed.

### O-4. Where callbacks run

- Question: on the timer's own goroutine, on the System's executor, or on an executor the caller supplies?
- Options: **A** the System's internal executor (`executor` seam, default goroutine per activation, `executor.go`), never inline; `Clock.AfterFunc` only wakes the scheduler. Pekko equivalent: the dispatcher given at schedule time (P7). **B** run on the goroutine that `Clock.AfterFunc` provides (`clock.go`: "on its own goroutine"): simplest, but with a manual clock `Advance` would run callbacks inline in the test and a slow callback delays the next one on a shared timer. **C** an exported executor parameter (like Pekko's implicit `ExecutionContext`): new export, no consumer.
- Recommendation: **A**, no exported executor in this slice (C would need its own EXP-1 record and has no consumer).
- Evidence: L:383-392; TI:103-107. Open sub-point: for a message send (`Ref.Tell`) the executor hop adds one goroutine per tick; whether the send may happen directly on the timer's goroutine is a measurement for #821.

### O-5. Ownership, closing and the interplay with #1131

- Question (a), ownership: in Pekko the scheduler belongs to the `ActorSystem` (`system.scheduler`) and the `Scheduler` trait has no `close`; only LARS is `Closeable` and the system closes it. Options: **A** one scheduler per `System`, created with it, reached through the `System`, with no public `Close`. **B** a scheduler value users create and close. **C** free functions that take a `System`. Recommendation: **A** (the shape of the API, method on `System` or a `Scheduler()` accessor, is #821's EXP-1 record).
- Question (b), the closed outcome: Pekko throws `IllegalStateException`/`SchedulerException`. Go form: a sentinel error (e.g. `ErrSchedulerClosed`; name for the story) returned by the schedule call. Recommendation: error.
- Question (c), `TaskRunOnClose`: not ported (S:561-567 says it is not for end users and goPekko has no dispatcher shutdown task). Recommendation: do not port; marker `TODO(TIM-01-run-on-close)`.
- Question (d), order against #1131: if the scheduler (#821) lands before #1131, `System.Terminate` leaks pending timers. Options: **A** #821 includes an internal `close` and `Terminate` calls it (absorbing most of #1131's wiring; #1131 keeps the tests of P5). **B** #821 lands with a documented known gap `TODO(TIM-13-close)` and #1131 follows immediately. **C** #1131 first. Recommendation: **A**.
- Evidence: L:170-247; SS:520-548, 661-697; `system_shutdown.go:30-66`.

### O-6. Does "a periodic task that throws aborts" map to a recovered panic?

- What Pekko does, read at the pin (`Scheduler.scala`, `LightArrayRevolverScheduler.scala`, `AbstractDispatcher.scala`):
  - The repetition is cut **by omission of the re-arm**: in the fixed-delay task, `runnable.run()` and then `if (get != null) swap(scheduleOnce(delay, this))` are inside one `try` (S:88-93); the fixed-rate task has the same structure (L:143-151), as does the deprecated `schedule` (S:229-231, 351-364). An exception from `runnable.run()` skips the re-arm, so the task is never scheduled again. The documentation says the same (S:75-76, 220-221).
  - Only a `SchedulerException`, or an `IllegalStateException` whose cause is one, is swallowed there (S:96-97: "ignore failure to enqueue or terminated target actor"; the closing brace is S:98). Any other exception propagates out of `SchedulerTask.run`.
  - It propagates into the `ExecutionContext` that runs the task (`executionContext.execute(other)`, L:386-387). For a Pekko dispatcher, `TaskInvocation.run` catches `NonFatal(e)` and publishes `Error(e, "TaskInvocation", this.getClass, e.getMessage)` on the event stream (`AbstractDispatcher.scala:58-62`). The `reportFailure` at L:391 is for an exception raised by `execute` itself (a rejection), not by the body of the task.
  - The earlier text of this ADR said "the exception reaches the executor's `reportFailure`"; that is corrected here: the report is `TaskInvocation`'s `Error` on the event stream (for the dispatcher; another `ExecutionContext` reports through its own mechanism), and the repetition ends because the re-arm is never reached.
- Options: **A** the scheduler recovers a callback panic, does **not** re-arm (so the repetition ends exactly as in Pekko), and reports the failure once through `kit-logger` as the Go counterpart of `TaskInvocation`'s `Error` (a goroutine panic is otherwise unrecoverable by the caller and would crash the process). **B** let the panic propagate (crashes the process). **C** deliver the failure to a handler passed by the caller (new export).
- Decision (Owner, 2026-11-10): **A**, written so that it replicates Pekko: no re-arm after a failure of the callback, the failure reported once. Publishing an `Error` on the goPekko event stream instead of (or besides) the log is a difference from `TaskInvocation`, recorded as `TODO(TIM-01-error-report)`. The same recover applies to a one-shot callback. A user callback that returns an error is not Pekko's `Runnable`; whether to offer `func() error` is #821's open choice.
- Evidence: S:75-76, 88-98, 220-221; L:386-392; AbstractDispatcher.scala:55-63; SS:270-314; confirmed by the oracle run SC4 (section "Oracle results").

### O-7. Tick granularity and accuracy

- Question: reproduce the wheel (rounding up to `tick-duration = 10ms`, `ticks-per-wheel = 512`, max `Int.MaxValue` ticks), or time each task on `Clock.AfterFunc`?
- Options: **A** no ticks: each task armed on `Clock.AfterFunc`; promised: never early, no overlap, drift compensation for fixed rate; not promised: tick rounding, a 10ms floor, the `maxFrequency` figure (S:495-499). **B** emulate the tick by rounding delays up to a multiple of a configurable tick.
- Recommendation: **A**. Deviation 1 (no tick rounding) and Deviation 2 (no `Int.MaxValue`-ticks limit) are observable differences, recorded as **ACCEPTED PORT GAP, approved by the Owner on 2026-11-10** (in this ADR and in `docs/parity/actor-inventory.md`, row C-20), justified under D14.3 by Go having no thread-per-wheel and `time.Timer` being exact enough; no test of SS asserts the rounding, only "never early" (SS:156-175) and rates within the tolerance `4.4 +- 0.5`, about 11% (SS:431, 444, 460). Without an oracle for the tolerance, the rate scenarios keep Pekko's tolerance.
- Evidence: L:62-97, 230-235; `reference.conf:900,912`; SS:156-175, 422-461.

### O-8. Delivery of a scheduled message to a stopped actor

- Options: **A** as Pekko: send, the message becomes a dead letter, the periodic repetition ends silently (P6). **B** check liveness first, send nothing, end the repetition (a deviation: no dead letter). **C** surface an error to the caller (no Pekko counterpart).
- Recommendation: **A**. Open: how goPekko detects "terminated" (`Ref.Tell` returns nothing, `ref.go:20`; `System.Done(h)` exists, `system.go:213`) is the story's, with marker `TODO(TIM-01-terminated-detect)`. It is shared with #896 (dead letters).
- Evidence: S:162-167, 358-363, 432-435; SS:252-267.

### O-9. Evidence for Go-only rows (ADR 0033 D13)

- Question: Pekko rules above are read, not run; Go rules (error returns, executor hop, no tick) have no oracle.
- Options: **A** one differential Pekko scenario per row that has an applicable oracle (periodic `cancel()` results, `scheduleOnce(0)`, close with plain and `TaskRunOnClose` tasks, panic in a periodic task), plus Go unit tests with a manual clock for the Go-only rows, which the Owner then approves as alternative evidence. **B** unit tests only.
- Recommendation: **A**; this ADR does not run any scenario.
- **Owner, 2026-11-10:** the rows that have no Pekko oracle (the error returns, the executor hop, the absence of ticks and the other Go-only rows) stay **`pending approval`** under ADR 0033 D13 until the Owner approves them one by one; this ADR does not mark any of them approved, except the two ACCEPTED PORT GAPs of O-7.

### O-10. Accepting this ADR, and the split

- Question: accept as written, accept per O-n, or ask for a split (ownership/close versus the clock seam; the ACT2 reconciliation keeps one ADR unless the Owner asks).
- Recommendation: one ADR; accept O-1, O-3, O-4, O-5, O-6, O-7, O-8 as recommended and O-2 and O-9 conditional on the oracle run (made on 2026-11-10; see "Oracle results").
- **Decision (Owner, 2026-11-10):** accepted as recommended, with the precisions above. This ADR is Accepted.

## Oracle results (2026-11-10)

The oracle `docs/parity/oracles/scheduler-lifecycle-2026-11-10.md` (harness package `scheduler`, 21 scenarios, three runs each, Java 1.8.0_504, Pekko 1.7.1; PRs #1506 and #1507) pins the values that O-2, O-3 and O-6 left to a run. Observed, and now the contract values of this ADR:

- **O-3(b), `scheduleOnce` with a zero or negative delay:** no exception; the handle is the not-cancellable object (`Cancel` false, `IsCancelled` false); the task runs at once (on the caller's thread inside the call when the executor is inline). The documentation of `Scheduler.scala:423` is wrong, as the ADR said. `TODO(TIM-01-once-nonpositive)` is closed.
- **O-3(a), a zero or negative interval** (fixed delay and fixed rate, runnable and message shapes): `IllegalArgumentException` with the message "Task scheduled with [N] seconds delay, which means creating an infinite loop. The expected delay must be greater than 0." (`N` is `toSeconds`, so `-1ns` prints `[0]`); nothing is scheduled.
- **O-3(c), a zero or negative initial delay:** the first run happens at once. **Refinement:** for fixed rate a *negative* initial delay is not the same as zero: observed: the following runs came about 8 to 10 ms after the first (+8 ms and +18 ms in the run), a catch-up burst; the mechanism (the next time computed from `clock() + initialDelay`, L:142) is read from the source, and that the missed time is being made up is inferred; with zero, or with fixed delay, runs are one interval apart. The Go form must state it.
- **O-2, periodic `Cancel`:** true before the first run, while idle, during a run, from inside the callback and after an exception aborted the repetition; false on every later call; `IsCancelled` is false until that first `Cancel`; a running callback finishes. Fixed delay and fixed rate give the same values, **with a positive initial delay** (500 ms in the scenarios): fixed rate with an initial delay of zero returns the not-cancellable object for its first reference (L:205-209) and was not run. The one-shot: true before it runs, then false; false after or during execution, and `IsCancelled` false. `TODO(TIM-01-periodic-cancel-result)` is closed.
- **O-6, a periodic callback that throws:** the repetition stops, in all cases observed. The report depends on the executor: the system's dispatcher publishes one `Error` (`TaskInvocation`), `ExecutionContext.global` prints to stderr and publishes nothing, a plain `fromExecutor` pool goes to the thread's uncaught-exception handler. A Go scheduler that recovers, does not re-arm and logs once matches the dispatcher case in effect (no re-arm, reported once), not in channel (`kit-logger` instead of an `Error` on the event stream, `TODO(TIM-01-error-report)`).
- **O-8, a scheduled message to a stopped actor:** one `DeadLetter` (sender `deadLetters`), then the repetition ends; the schedule call does not throw. The inference of P6 is now observed (the cause that ends the repetition, `SchedulerException` "timer active for terminated actor", stays inferred).
- **Citation corrected:** the swallow of `SchedulerException` in fixed delay is `Scheduler.scala:96-97`, not 96-98.
- **Not covered by the oracle:** scheduler close (P5) and the O-7 rounding and tick limit remain read, not run.

## Exports (ADR 0011, EXP-1)

This ADR exports nothing and approves no export. It names the exports the stories will introduce, each to be justified in the story that adds it (symbol, consumer, why the public API is not enough, what it does not expose, its test): the scheduler accessor, `Cancellable` (O-1), the closed and invalid-interval errors (O-3, O-5). The clock seam is already exported (`Clock`, `Timer`, `Config.Clock`) and unchanged; nothing here asks for a manual clock (a test double, ManualTime is M-04) or an exported executor (O-4 C).

## Consequences

- #821, #822, #823 take their criteria from C1 to C10 once the matching O-n is accepted; #1131 takes P5 and C7; #860 takes P8 and C8; #861 (ReceiveTimeout) and Typed `TimerScheduler` do not.
- The plan rows C-02 to C-06 stay valid; the wording "cancel true only if it prevented execution" (`docs/actor-typed-delivery-plan.md:273`) is exact for one-shot only (O-2).
- Corrections to issue anchors (read at the pin): the abort rule is `Scheduler.scala:75-76` (the reconciliation says :77-78); the IAE note for non-positive intervals is :78-79; `scheduleOnce` documents an IAE for non-positive delays that the code does not throw (P3).
- `README.md` of `pekko-actor` gets a note that this ADR is Accepted and that no scheduler exists yet (see the scope).

## Not covered

`SchedulerTask.cancelled` (#1077); closing the scheduler on `System` termination (#1131: the implementation; the behavior is here); `ReceiveTimeout` (#861); Typed `TimerScheduler`; remote and cluster; `maxFrequency`; the deprecated `schedule`; Java API overloads; stream timers; `ManualTime` (M-04).

## Alternatives rejected

- A hashed wheel with a timer goroutine (Pekko's design): more moving parts for a tick that Go does not need (O-7 A).
- Callbacks on the `Clock.AfterFunc` goroutine: couples user code to the clock's goroutine and breaks deterministic manual clocks (O-4 B).

## Owner table

| Date | Owner decision (literal text) | Effect on this ADR |
|---|---|---|
| 2026-11-10 | "Apruebo O-1 a O-10 como los recomendaste, con estas precisiones": O-2 and O-3 are ruled by the code of Pekko (`Scheduler.scala:594-607`, `LightArrayRevolverScheduler.scala:205-209`) and their exact values are bound to an oracle run before #821; O-6 cites where Pekko cuts the repetition and where it reports; O-7 deviations 1 and 2 are ACCEPTED PORT GAP; O-9 rows without an oracle stay `pending approval` (D13) until approved one by one | The ADR is Accepted; the precisions are applied in O-2, O-3, O-6, O-7, O-9 and O-10 |
| 2026-11-10 | "TODO(TIM-01-fixed-rate-negative-initial) se resuelve replicando a Pekko tal como lo midió el oráculo (ráfagas de recuperación), con un test basado en el oráculo en #823. Sacá el TODO." and "corregí la línea Evidence" | O-3(c) decided: the Go scheduler replicates the catch-up burst of fixed rate for a negative initial delay, with an oracle-based test in #823; the marker is removed; the Evidence line cites the oracle record and says what is still read, not run (P5 close, O-7 ticks) |
