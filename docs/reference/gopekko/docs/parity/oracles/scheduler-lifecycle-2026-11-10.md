# Oracle: the scheduler lifecycle (Pekko Classic v1.7.1)

- Date: 2026-11-10. Evidence only; this document recommends nothing and changes no ADR.
- Pekko `pekko-actor` 1.7.1 (tag `v1.7.1`, `upstream/pekko` at `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), Scala 2.13.18, sbt 1.10.7, JDK Java 1.8.0_504 (Zulu 8.96.0.205, run through `sbt -java-home`; the banner `[info] welcome to sbt 1.10.7 (Azul Systems, Inc. Java 1.8.0_504)` of the recorded sbt call is in `logs/env.txt`, and every scenario log starts with `java.version=1.8.0_504`, which is the evidence for the JVM that ran each scenario), the `ActorSystem` scheduler (`LightArrayRevolverScheduler`, default config), default fork-join dispatcher, Classic actors only.
- Purpose: the evidence that ADR 0039 (scheduler lifecycle and cancellation, issue #776, PR #1502) asked for in O-2 (what `cancel()` returns for a periodic handle), O-3 (invalid delays and the one-shot zero), O-6 (a periodic task that throws) and, for O-8/P6, what a scheduled message to a terminated actor does. The Owner decided on 2026-11-10 to pin the values before story #821.
- Method: read the code (section 1), run each scenario in its own JVM, three runs each (section 2). The harness is `docs/parity/harness/` package `scheduler` (README: commands, versions). Raw, untrimmed logs are in [`scheduler-lifecycle-2026-11-10.logs/`](scheduler-lifecycle-2026-11-10.logs/README.md): `<SCENARIO>.<run>.txt` is stdout, `<SCENARIO>.<run>.err.txt` is stderr. Citations `[nnn]` are lines of run 1; after masking times, thread numbers and actor uids the three runs of every scenario are identical line by line (63 JVMs, all exit code 0), so "runs 2, 3: same" means that, and only figures in microseconds or milliseconds differ.
- Rule "claims against evidence": a statement in the Observed column names the log line; anything not seen in a log is marked Inferred.

## 1. What the source says (code reading, Pekko 1.7.1)

`S` = `actor/src/main/scala/org/apache/pekko/actor/Scheduler.scala`, `L` = `.../actor/LightArrayRevolverScheduler.scala`, `D` = `.../dispatch/AbstractDispatcher.scala`.

| # | Claim | Location |
|---|---|---|
| K1 | Documentation: `scheduleOnce` throws `IllegalArgumentException` "if the given delays is zero, negative or exceed the maximum reach" (the same text for the periodic forms) | `S:423`, `S:78`, `S:124`, `S:223`, `S:259` |
| K2 | Code: `schedule(ec, r, delay)` with `delay.length <= 0` throws `SchedulerException` only if stopped, otherwise calls `ec.execute(r)` on the caller's thread and returns `NotCancellable` (`cancel()` false, `isCancelled` false) | `L:205-209`, `L:420-424` |
| K3 | `scheduleOnce` (LARS) is `schedule(executor, runnable, roundUp(delay))`, so there is no check of the sign | `L:163-166` |
| K4 | `scheduleWithFixedDelay` and `schedule` (the fixed-rate implementation) call `checkPeriod(delay)` first, which throws `IllegalArgumentException("Task scheduled with [<delay.toSeconds>] seconds delay, which means creating an infinite loop. The expected delay must be greater than 0.")` when `delay.length <= 0`. `scheduleAtFixedRate` and the message forms of both delegate to them | `L:127-130`, `L:134-136`, `L:224-228`; `S:298-306`, `S:154-168` |
| K5 | Periodic handle `AtomicCancellable`: `cancel()` returns false when the reference is null; else if `c.cancel()` succeeds it returns `compareAndSet(c, null)`, otherwise `compareAndSet(c, null) \|\| retry`; `isCancelled` is `get == null` | `S:594-607` |
| K6 | Fixed delay: `runnable.run()` and then `swap(scheduleOnce(delay, this))` inside a `try` that catches only `SchedulerException` and `IllegalStateException` caused by one | `S:88-99` |
| K7 | Fixed rate: `runnable.run()`, drift, `swap(schedule(...))` inside a `try` that catches only `SchedulerException` (no `IllegalStateException` case) | `L:143-152` |
| K8 | The first fixed-rate run time is `clock() + initialDelay.toNanos` and the next delay is `max(delay - drift, 1ns)` with `drift = clock() - getAndAdd(delay)` | `L:139-148` |
| K9 | `TaskInvocation.run` catches `NonFatal(e)` and publishes `Error(e, "TaskInvocation", this.getClass, e.getMessage)` on the event stream | `D:55-63` |
| K10 | The message forms send `receiver ! message` and then throw `SchedulerException("timer active for terminated actor")` if `receiver.isTerminated`; `scheduleOnce(delay, receiver, message)` only sends | `S:160-167`, `S:358-363`, `S:428-436` |

## 2. Scenarios and results

Common method: a latch proves that a callback ran; "nothing else happened" is shown by a window, a one-shot task on the scheduler under test that releases a latch after the stated time (`-- window Nms elapsed`). Two bounded sleeps exist and are named where used (SC3 (b)). Callback executor: the system's default dispatcher unless the row says otherwise. Intervals and delays are 100 ms unless stated.

### SC1 `scheduleOnce` with delay zero and negative (`SC1`; K1, K2, K3)

| Step | Observed (run 1) | Runs 2, 3 |
|---|---|---|
| `scheduleOnce(Duration.Zero, runnable)` | returned, no exception: class `LightArrayRevolverScheduler$$anon$6` (the `NotCancellable` object), `isCancelled=false` [002] | same |
| same, `-1.second` and `Duration(-1, NANOSECONDS)` | returned, same class, `isCancelled=false` [013], [024]; no `IllegalArgumentException` | same |
| `cancel()` / `isCancelled` / `cancel()` again, after the task ran | `cancel()=false isCancelled=false cancel() again=false` [004], [015], [026] | same |
| thread of the task | `sch-pekko.actor.default-dispatcher-4` [003] ([014] ran on `-4`, the next one [025] on `-5`); the task ran in the dispatcher, the caller was `main` | same, dispatcher thread numbers vary |
| EC that runs inline (`sameThread`), delay zero | line [006] `sameThread task ran on main` comes before [007] `... -> returned ... isCancelled=false`; [008] latch count after return `0`: the task ran inside the schedule call, on the caller's thread | same |
| `cancel()` on that handle | `cancel()=false isCancelled=false` [009] (and [020], [031] for `-1s`, `-1ns`) | same |
| `scheduleOnce(Zero, receiver, msg)`, and `-1s`, `-1ns` | returned the same class [010], [021], [032]; the message was delivered [011], [022], [033]; handle after delivery `cancel()=false isCancelled=false` | same |
| control `+1ns` (positive, tiny) | class `LightArrayRevolverScheduler$TaskHolder` [035]; the task ran 23070 us after the call, on a dispatcher thread [036] (run 2: 26736 us, run 3: 24794 us); with the inline EC the latch count after return is `1` [040] and `cancel()=true isCancelled=true` [041] | same, except the figures |

Result: the code is right, the documentation is wrong. Zero and negative delays never throw; the task is handed to the executor at once, from the caller's thread; the handle is an object whose `cancel()` and `isCancelled` are always false. A positive delay, however small, gives a real `TaskHolder` that can be cancelled and ran about 23 to 27 ms later in the three runs.

### SC2 Periodic schedules with zero and negative interval, zero and negative initial delay (`SC2fd`, `SC2fr`; K4, K7, K8)

Rejected values (fixed delay `SC2fd`; fixed rate `SC2fr` gives the same lines at [002]-[010]):

| Call | Observed (run 1) | Runs 2, 3 |
|---|---|---|
| `scheduleWithFixedDelay(100ms, Zero)(runnable)`, the message form, and with initial delay `Zero` | threw `java.lang.IllegalArgumentException: Task scheduled with [0] seconds delay, which means creating an infinite loop. The expected delay must be greater than 0.` [002], [003], [004] | same |
| interval `-1.second` (the three shapes) | threw `IllegalArgumentException: Task scheduled with [-1] seconds delay, ...` [005], [006], [007] | same |
| interval `-1ns` (the three shapes) | threw `IllegalArgumentException: Task scheduled with [0] seconds delay, ...` [008], [009], [010] (the text prints `toSeconds`, which truncates to 0) | same |
| `scheduleAtFixedRate` with the same arguments, the runnable and the message form | the same exception and text, [002]-[010] of `SC2fr.1.txt` | same |
| runs of the rejected schedules after a 400 ms window | `0` [012] (both scenarios; the counter covers the runnable forms: the message forms were rejected too, [003], [006], [009], and had no counter of their own) | same |

Accepted values (run 1; times are microseconds since the call, from the log lines):

| Call | Observed | Runs 2, 3 |
|---|---|---|
| fixed delay, initial delay `Zero`, interval 100 ms | returned `Scheduler$$anon$1`, `isCancelled=false` [013]; run 1 at +3372 us [014] on a dispatcher thread, run 2 at +116341 us, run 3 at +236443 us | JVM run 2: run 1 at +2789 us; JVM run 3: run 1 at +2420 us |
| fixed delay, initial delay `-1.second`, interval 100 ms | run 1 at +44 us [019], run 2 at +113767 us [020], run 3 at +234418 us [021] | run 1 at +108 and +31 us (runs 2, 3) |
| fixed rate, initial delay `Zero` | returned `LightArrayRevolverScheduler$$anon$1` [013]; runs at +2025, +110557, +219631 us [014]-[016] | runs 2, 3: +2417, +113111, +212713 and +3509, +108106, +217873 us |
| fixed rate, initial delay `-1.second`, interval 100 ms | runs at +50, +8622, +18640 us [019]-[021]: runs 8 to 10 ms apart instead of 100 ms (a burst); that the 1 s of missed time is being made up is *Inferred* from K8, not observed | runs 2, 3: +688, +6173, +17389 and +33, +9097, +19116 us |
| interval `1ns` (positive), initial delay 100 ms | accepted. Fixed delay: runs at +117580, +148343, +178980 us [024]-[026] (about 30 ms apart); fixed rate: +120495, +129135, +140536 us [024]-[026] (about 10 ms apart) | same shape in runs 2, 3 |
| `cancel()` after 3 runs | `cancel()=true isCancelled=true` [017], [022], [027] | same |

Result: interval at or below zero throws `IllegalArgumentException` for every shape of both periodic forms; an initial delay at or below zero is accepted and runs the first execution at once (31 us to 3.5 ms after the call in the twelve samples: 2 forms x 2 initial delays x 3 runs). With fixed rate a negative initial delay is not a clamp to zero: the following runs follow back to back (*Inferred* from K8: observed only that runs 2 and 3 came 8 to 10 ms apart) until the delay catches up (K8). A positive interval below the tick is accepted.

### SC3 The `Cancellable` (`SC3fd`, `SC3fr` periodic; `SC3o` one-shot; K5)

Periodic (`SC3fd` fixed delay, `SC3fr` fixed rate: identical values, identical line numbers):

| Case | Observed (run 1, `SC3fd`) | Runs 2, 3 |
|---|---|---|
| (a) before the first run (initial delay 500 ms): fresh handle | `isCancelled=false` [002]; `cancel()=true isCancelled=true` [003]; `cancel() again=false isCancelled=true` [004]; runs after an 800 ms window `0` [006] | same |
| (b) after two runs, task idle (interval 400 ms; a bounded `Thread.sleep(100)` after run 2, so the third run is not due): | `runs=2 isCancelled=false` [009]; `cancel()=true isCancelled=true` [010]; `cancel() again=false isCancelled=true` [011]; runs after an 800 ms window `2` [013] | same |
| (c) `cancel()` from another thread while run 1 is blocked inside the callback | `cancel()=true isCancelled=true` [015], `cancel() again=false` [016]; the callback then finishes [017] (`run 1 ends`); runs after a 500 ms window `1` [019] | same |
| (e) `cancel()` from inside the callback on run 2 | `self cancel()=true isCancelled=true` [021]; runs after the window `2`; `cancel() again=false isCancelled=true` [023] | same |
| (d) repetition ended by an exception on run 1 (`RuntimeException`) | after the abort: `runs=1 isCancelled=false` [026]; `cancel()=true isCancelled=true` [027]; `cancel() again=false isCancelled=true` [028]; runs after the window `1` [030] | same |

Fixed rate (`SC3fr.1.txt`) has the same values on the same lines [002]-[030].

One-shot (`SC3o`):

| Case | Observed (run 1) | Runs 2, 3 |
|---|---|---|
| (a) before running (delay 500 ms) | class `TaskHolder`, `isCancelled=false` [002]; `cancel()=true isCancelled=true` [003]; `cancel() again=false isCancelled=true` [004]; runs `0` [006] | same |
| (b) after it ran | `isCancelled=false` [008]; `cancel()=false isCancelled=false` [009]; `cancel() again=false isCancelled=false` [010] | same |
| (c) while the callback is running | `cancel()=false isCancelled=false` [012]; the callback finishes [013] | same |

Result: a periodic `cancel()` returns true for the call that clears the reference, in every state tried (before the first run, idle after runs, during a run, from inside a run, after the repetition was aborted by an exception), and false for every later call; `isCancelled` is true after that call and not before. After an exception aborted the repetition, the handle still reports `isCancelled=false` and the first `cancel()` returns true (there was nothing left to prevent). A run in progress is not interrupted and finishes. One-shot: `true` only when it prevented the run, `isCancelled=false` after the run.

### SC4 A periodic task whose callback throws on its 2nd run (K6, K7, K9)

All runs: interval 100 ms, initial delay 100 ms, run 2 throws and the log records the number of runs after a 500 ms window (5 intervals after the throw).

| Scenario | Executor | Exception | Observed (run 1) | Runs 2, 3 |
|---|---|---|---|---|
| `SC4fd-disp` | default dispatcher | `RuntimeException("boom-2")` | run 2 [004], throw [005]; `eventstream Error cause=java.lang.RuntimeException: boom-2 logSource=TaskInvocation logClass=org.apache.pekko.dispatch.TaskInvocation message=boom-2` [006]; the default logger printed `[ERROR] ... [TaskInvocation] boom-2` with the stack to stdout; runs after the window `2` [008]; Error events `1` [009]; `isCancelled=false` [010]; `cancel()=true isCancelled=true` [011] | same (`SC4fd-disp.2/3.txt`) |
| `SC4fr-disp` | default dispatcher | same | same values on the same lines (Error event [006], runs `2` [008], Error events `1` [009]) | same |
| `SC4fd-global` | `ExecutionContext.global` | same | run 2 on `scala-execution-context-global-24` [004], throw [005]; no Error event: `Error events on the event stream = 0` [008]; runs `2` [007]; stderr: `java.lang.RuntimeException: boom-2` and its stack (`SC4fd-global.1.err.txt`); stdout has no `[ERROR]` line | same; stderr has the exception in all three runs |
| `SC4fr-global` | `ExecutionContext.global` | same | same: runs `2` [007], Error events `0` [008], stderr `java.lang.RuntimeException: boom-2` | same |
| `SC4fd-plain` | `ExecutionContext.fromExecutor(cachedThreadPool, reporter)` | same | `uncaught exception handler: thread=plain-ec-thread java.lang.RuntimeException: boom-2` [006]; the reporter passed to `fromExecutor` was not called (no `plain reporter` line); Error events `0` [009]; runs `2` [008]; stderr empty (the harness installs a default uncaught handler, so this is the JVM path) | same |
| `SC4fr-plain` | same | same | same: handler line [006], runs `2` [008], Error events `0` [009] | same |
| `SC4sfd` | default dispatcher | `SchedulerException("task-scheduler-exception")` | run 2 [004], throw [005]; no Error event (`Error events = 0` [008]); no `[ERROR]` output; runs `2` [007]; `isCancelled=false` [009]; `cancel()=true isCancelled=true` [010] | same |
| `SC4sfr` | default dispatcher | same | the same values on the same lines | same |
| `SC4ifd` | default dispatcher | `IllegalStateException` caused by `SchedulerException` | swallowed: no Error event (`0` [008]), runs `2` [007], nothing printed | same |
| `SC4ifr` | default dispatcher | same | **not** swallowed: `eventstream Error cause=java.lang.IllegalStateException: task-ise logSource=TaskInvocation ...` [006] with `Caused by: org.apache.pekko.actor.SchedulerException: cause-scheduler-exception` in the stdout stack; Error events `1` [009]; runs `2` [008] | same |

Result: in all ten cases the repetition stops after the throwing run (2 runs in the 500 ms window). The place where an ordinary exception is reported depends on the executor: on the default dispatcher it is published once on the event stream as `Error` with log source `TaskInvocation` and printed by the default logger; on `ExecutionContext.global` there is no event and the exception is printed on stderr; on a plain executor there is no event and the JVM's uncaught-exception handler of the pool thread gets it (the reporter given to `fromExecutor` is not used for a failing `execute`). `SchedulerException` is swallowed silently in both forms; an `IllegalStateException` caused by one is swallowed by fixed delay and is reported by fixed rate.

### SC5 A scheduled message to a terminated actor (K10; `SC5once`, `SC5fd`, `SC5fr`, `SC5fdlive`, `SC5frlive`)

Receiver stopped and `Terminated` seen before scheduling (`victim stopped: isTerminated=true`) in the first three; stopped after two deliveries in the last two. A listener is subscribed to `DeadLetter` and `Logging.Error`.

| Scenario | Observed (run 1) | Runs 2, 3 |
|---|---|---|
| `SC5once`: `scheduleOnce(50ms, victim, "m1")` | returned `TaskHolder`, `isCancelled=false` [004]; one event `DeadLetter message=m1 sender=Actor[pekko://sch/deadLetters] recipient=Actor[pekko://sch/user/victim#...]` [005]; the default logger printed `[INFO] [pekkoDeadLetter] ... Message [java.lang.String] ... was not delivered. [1] dead letters encountered`; summary `victim received=0 deadLetters=1 errorEvents=0` [007]; afterwards `cancel()=false isCancelled=false` [009] | same |
| `SC5fd`: `scheduleWithFixedDelay(Zero, 100ms, victim, "p")` | returned without an exception, `Scheduler$$anon$1 isCancelled=false` [004]; one `DeadLetter message=p` [005] and no more in a 600 ms window: `victim received=0 deadLetters=1 errorEvents=0` [007]; `isCancelled=false` [008]; `cancel()=true isCancelled=true` [009] | same |
| `SC5fr`: `scheduleAtFixedRate(Zero, 100ms, victim, "p")` | returned `LightArrayRevolverScheduler$$anon$1` [004]; one `DeadLetter` [005]; `deadLetters=1 errorEvents=0` [007]; `isCancelled=false` [008]; `cancel()=true isCancelled=true` [009] | same |
| `SC5fdlive`: receiver alive, stopped after 2 messages | victim received `p` twice [003], [004]; stopped [005]-[007]; then one `DeadLetter message=p` [008]; summary `victim received=2 deadLetters=1 errorEvents=0` [010]; `isCancelled=false` [011]; `cancel()=true isCancelled=true` [012] | same |
| `SC5frlive` | the same values on the same lines (`received=2 deadLetters=1 errorEvents=0` [010]) | same |

Result: a message scheduled with `scheduleOnce` to a terminated actor is published as a `DeadLetter` whose sender is `deadLetters` and whose recipient is the stopped actor. A periodic message to a stopped actor produces exactly one dead letter, then the repetition ends: no second dead letter in a 600 ms window (six intervals), no `Error` event and no scheduler log; the only log line is the default listener's `[INFO] [pekkoDeadLetter] ... was not delivered. [1] dead letters encountered` (in every SC5 log); and the schedule call itself never throws. The `SchedulerException("timer active for terminated actor")` itself is not observable (it is swallowed); the end of the repetition is the observed consequence (Inferred: that it is this exception that ends it, from K10 and the silence in SC4s).

## 3. Differences from what the ADR 0039 draft says

Compared with `docs/adr/0039-scheduler-lifecycle-and-cancellation.md` as of commit `145a512a15222641d24ff48a5c11463db30db3d5` of PR #1502 (before its "Oracle results" section).

| # | ADR text | Observed | Verdict |
|---|---|---|---|
| D1 | P3 and O-3(b): `scheduleOnce` with delay at or below zero runs at once, returns the not-cancellable handle (`cancel()` false, `isCancelled` false), the documentation (`S:423`) is wrong | SC1 [002]-[034]: no exception, handle with both false, task runs at once, from the caller's thread inside the call when the EC is inline [006]-[008] | matches |
| D2 | O-3(a): interval at or below zero throws `IllegalArgumentException` | SC2: thrown for both forms, runnable and message shapes, with the initial delay zero or positive | matches. Not in the ADR: the exact message, and that `-1ns` prints `[0] seconds` |
| D3 | O-3(c): negative or zero initial delay runs at once, "read from L:159, 206-209" | SC2 first runs 31 us to 3.5 ms after the call | matches. Imprecise: for fixed rate a negative initial delay is not equivalent to zero; the first run is at once and the following ones are a catch-up burst (+8.6 ms, +18.6 ms) because the next time is computed from `clock() + initialDelay` (K8); with zero the runs are 100 ms apart. C4 and the typed-owner rule "initial delay may be negative" need the burst stated |
| D4 | P2 and O-2: the periodic `cancel()` returns true when the call clears the reference, even if the last run executed; false the second time; `isCancelled` true once the reference is null | SC3 (a)-(e), both forms | matches. The ADR's open points are answered: after an aborted repetition `isCancelled` is false until `cancel()`, which returns true (SC3 (d) [026]-[027]); cancel during a run and from inside a run return true and the callback finishes |
| D5 | The plan row C-02 "true only if it prevented execution" holds for one-shot only | SC3o (b), (c) false after or during execution; SC3 (b), (d) periodic true with nothing to prevent | matches |
| D6 | P4: an exception that is not a `SchedulerException` "propagates out of the task into the executor, which reports it (once ...)"; O-6 option A: reported once | SC4: reported once as `Error` on the event stream only when the executor is the system's dispatcher; with `ExecutionContext.global` it goes to stderr and no event exists; with a plain executor it goes to the pool thread's uncaught handler | imprecise: "the executor reports it" is true for the dispatcher only. A scheduler in Go that recovers and logs once matches the dispatcher case, not the other two |
| D7 | P4: "A `SchedulerException` ... and an `IllegalStateException` caused by one are swallowed" | SC4s: swallowed in both forms. SC4i: swallowed by fixed delay, **reported** by fixed rate (SC4ifr [006], Error events `1`) | wrong for fixed rate (K7 catches only `SchedulerException`; the ADR cites `L:150` for it). Not a difference that a Go port needs unless it models the ISE wrapper |
| D8 | P6: "the last message to a stopped receiver is sent (inferred, not read: a send to a terminated local ref becomes a dead letter) and the repetition then stops without a log" | SC5: confirmed, with exactly one `DeadLetter` (sender `deadLetters`) and no `Error`; the schedule call does not throw | matches; the inference is now observed |
| D9 | O-3 / P3: accuracy "may fire up to a tick late" (O-7) | SC1 control: a `1ns` delay ran 23.1, 26.7 and 24.8 ms after the call; SC2 interval `1ns` gave runs about 30 ms apart (fixed delay) and about 10 ms apart (fixed rate) | the ADR does not claim a figure; these are the first measured latencies, on this machine, with a dispatcher hop; nothing says "one tick" was exceeded by design |

Correction line: ADR 0039 O-6 (draft) cites the swallow of `SchedulerException` as `Scheduler.scala:96-98` (its P4 cites `S:96-97; L:150`); the lines are `S:96-97` (the closing brace of the `catch` is `S:98`). Fixed delay catches the exception at `S:96-97` and fixed rate at `L:150` (a single case). No other citation was found wrong.

## 4. Limits of this evidence

- One machine (Apple silicon, macOS), Java 1.8.0_504, default config (`tick-duration 10ms`, `ticks-per-wheel 512`). Latency figures are samples, not bounds.
- The "no further run" observations use windows of 400 to 800 ms against intervals of 100 to 400 ms; they show the repetition did not continue for that long, not for ever.
- SC3 (b) uses one bounded `Thread.sleep(100)` to be sure the task is idle (interval 400 ms): a latch inside the callback cannot show that the callback already returned. SC3 (d) and SC4 wait on the `Error` event or a latch inside the callback, then a window.
- `SchedulerException` and `ActorRef.isTerminated` are `private[pekko]`; the harness reaches them through `org.apache.pekko.actor.Access` (a harness object in that package, as `eventstream` does with `RecordingRef`).
- The scheduler was never closed: no scenario covers P5 (close, pending tasks, new tasks rejected after close). The ADR's P5, O-5 and O-7 (rounding, `Int.MaxValue` ticks) remain read-only, not run.
- The `Error` events are captured at `Logging.Error`; the default logger's `[ERROR]` lines are in stdout because `pekko.stdout-loglevel = INFO` in the harness config.
- Fixed rate with an initial delay of zero returns the `NotCancellable` object from `scheduledFirst` for the very first reference (K2); that is why a `cancel()` before the first run is not part of SC3 for that case (SC3 used an initial delay of 500 ms). Not run.
