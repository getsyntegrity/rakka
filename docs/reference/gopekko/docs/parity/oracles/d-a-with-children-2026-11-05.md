# Oracle: D-a with children (unbounded `restart`, `setup` failing on every restart) (Pekko v1.7.1)

- Date: 2026-11-05. Evidence only; this document recommends nothing and changes no ADR state.
- Pekko `pekko-actor-typed` and `pekko-slf4j` 1.7.1 (tag `v1.7.1`); for two combinations also the fixed `1.8.x` build (HEAD `59d8040885ea17675ad63aebc9f746e687306934`, version `1.8.0-M0+1-59d80408-SNAPSHOT`, includes the fix of apache/pekko#3620). Scala 2.13.18, sbt 1.10.7, JDK Temurin 11.0.32+9, default fork-join dispatcher, logback-classic 1.3.14, macOS. The sbt project is outside the repository and is not committed.
- Harness differences from the first D-a oracle (`d-a-restart-setup-unbounded-2026-11-04.md`, in this directory): the JVM ran with `-Xmx512m` (so that a runaway loop cannot take the machine's memory), a sampler thread prints one line per second, and a run is cut after 60 s.
- Purpose: the "To be verified" variant of ADR 0035 S-3 case (c): the actor has children at restart time. The code reading was: with the default `stopChildren = true` the restart waits for the children and the loop is asynchronous (no stack overflow); with `withStopChildren(false)` the recursion stays synchronous (`Supervision.scala:362`, `:384-388`).

## 1. Scenarios

Same shape as the first D-a oracle: the first `setup` works, a message `"bad"` fails and triggers the restart, every later `setup` throws; `restart` without a limit. A guardian watches the supervised actor. Children count themselves in `liveChildren`/`startedChildren`.

```scala
val idle = Behaviors.setup[String] { _ =>                       // a child
  liveChildren.incrementAndGet(); startedChildren.incrementAndGet()
  Behaviors.receiveMessage[String](_ => Behaviors.same).receiveSignal { case (_, PostStop) => liveChildren.decrementAndGet(); Behaviors.same }
}
def boom() = new IllegalStateException("boom")
def child(scn: String) = scn match {
  case "C1" => // setup creates a child, then throws on every restart; stopChildren = true (default)
    supervise(setup { ctx => val n = setupRuns.incrementAndGet(); ctx.spawnAnonymous(idle); if (n > 1) throw boom()
      receiveMessage { case "bad" => throw new IllegalStateException("first-failure"); case _ => same } })
      .onFailure[IllegalStateException](SupervisorStrategy.restart)
  case "C2" => // the child is created OUTSIDE the supervise; withStopChildren(false); the inner setup fails on every restart
    setup { ctx => ctx.spawn(idle, "outer-child")
      supervise(setup { _ => if (setupRuns.incrementAndGet() > 1) throw boom()
        receiveMessage { case "bad" => throw new IllegalStateException("first-failure"); case _ => same } })
        .onFailure[IllegalStateException](SupervisorStrategy.restart.withStopChildren(false)) }
  case "C3" => // setup has no children; the actor creates one while processing the failing message; stopChildren = true
    supervise(setup { ctx => if (setupRuns.incrementAndGet() > 1) throw boom()
      receiveMessage { case "bad" => ctx.spawnAnonymous(idle); throw new IllegalStateException("first-failure"); case _ => same } })
      .onFailure[IllegalStateException](SupervisorStrategy.restart)
  case "CTRL" => // the first D-a oracle's scenario 5: no children
    supervise(setup { _ => if (setupRuns.incrementAndGet() > 1) throw boom()
      receiveMessage { case "bad" => throw new IllegalStateException("first-failure"); case _ => same } })
      .onFailure[IllegalStateException](SupervisorStrategy.restart)
}
```

The harness prints hook, `registerOnTermination` and `whenTerminated` markers (as in the first oracle), a `SAMPLE` line per second (`setupRuns`, `liveChildren`, `startedChildren`, used heap without a forced GC), and, after 60 s with no `StackOverflowError` and no termination, a `CUTOFF` line and `Runtime.halt(3)` (exit code 3 means "did not end in 60 s").

## 2. Results

Three runs per combination. "SOE" is `StackOverflowError`. Exit code 3 is the 60 s cut; 255 is `System.exit(-1)` of the fatal-error handler; 0 is the harness exiting after the ActorSystem terminated.

| Scenario | Version | `jvm-exit-on-fatal-error` | SOE | Nested `setup` runs | Ends | Exit | CoordinatedShutdown | `registerOnTermination` | Guardian signal |
|---|---|---|---|---|---|---|---|---|---|
| CTRL (no children) | 1.7.1 | on | 3 of 3 | 2071 to 2072 | under 1 s | 255 | ran | ran | none |
| C1 (`setup` creates a child, `stopChildren = true`) | 1.7.1 | on | **0 of 3** | 5.05 to 5.11 million `setup` runs in 60 s | **no end in 60 s** (3 of 3) | 3 (cut) | not started | no | none |
| C1 | 1.7.1 | off | **0 of 3** | 5.12 to 5.19 million in 60 s | **no end in 60 s** (3 of 3) | 3 (cut) | not started | no | none |
| C2 (`withStopChildren(false)`, child outside `supervise`) | 1.7.1 | on | 3 of 3 | 2071 to 2088 | under 1 s | 255 | ran | ran | none |
| C2 | 1.7.1 | off | 3 of 3 | 2048 to 2087 | the system **did not terminate in 60 s** (3 of 3) | 3 (cut) | started; `actor-system-terminate` timed out after 10 s | no | none |
| C2 | fixed 1.8.x | off | 3 of 3 | 2073 to 2090 | terminated within 1 s (3 of 3) | 0 | ran, no timeout | ran | none |
| C3 (child created in the failing message, `stopChildren = true`) | 1.7.1 | on | 3 of 3 | 2085 to 2087 | under 1 s | 255 | ran | ran | none |
| C3 | 1.7.1 | off | 3 of 3 | 2081 to 2087 | the system **did not terminate in 60 s** (3 of 3) | 3 (cut) | started; `actor-system-terminate` timed out after 10 s | no | none |
| C3 | fixed 1.8.x | off | 3 of 3 | 2070 to 2090 | terminated within 1 s (3 of 3) | 0 | ran, no timeout | ran | none |

Notes: the "whenTerminated" marker was printed in only some of the `on` runs (it races with `System.exit`), as in the first oracle; the exit code 255 is the reliable signal. In every run the guardian received neither `ChildFailed` nor `Terminated` before the end.

### 2.1 C1, where no `StackOverflowError` occurred

| Run | Restarts/s, first 10 s | Restarts/s, last 10 s | Used heap at start / end (MB, no forced GC) | `liveChildren` at start / end / maximum | `startedChildren` at the end |
|---|---|---|---|---|---|
| on 1 | 78,219 | 85,744 | 18 / 41 | 1 / 0 / 1 | 5,101,613 |
| on 2 | 81,479 | 85,911 | 18 / 246 | 1 / 1 / 1 | 5,021,390 |
| on 3 | 80,194 | 84,874 | 19 / 103 | 1 / 1 / 1 | 5,046,143 |
| off 1 | 82,259 | 85,982 | 19 / 303 | 1 / 0 / 1 | 5,118,297 |
| off 2 | 83,560 | 86,540 | 19 / 155 | 1 / 1 / 1 | 5,086,159 |
| off 3 | 83,456 | 87,174 | 18 / 288 | 1 / 1 / 1 | 5,186,818 |

The rate is steady (about 80,000 to 87,000 restarts per second). The `setup` runs of section 2 and `startedChildren` here are two counters: `setupRuns` is incremented in `setup` before the child is spawned, `startedChildren` when the child itself starts (asynchronously), so at the cut-off they differ slightly. `liveChildren` never exceeds 1 and `startedChildren` tracks `setupRuns`, so each restart stops the previous child before creating the next one; the number of live children does not grow. The used heap oscillates between about 18 MB and a few hundred MB under `-Xmx512m` (no forced GC, so the end value is a sampling point, not a leak measure).

### 2.2 Lifecycle (one separate run per scenario, `pekko.actor.debug.lifecycle = on`, `off`, first 400 matching lines, 25 s)

- C1 (first 400 matching lines): 136 child `started` lines and 130 `stopped` lines, i.e. each restart creates and stops a child; the frame `RestartSupervisor.restartCompleted(Supervision.scala:404)` appears 129 times and `:414` 0 times: no nesting of `restartCompleted` frames.
- C2 and C3 (first 400 matching lines): the failure logs carry the alternating frames `restartCompleted(Supervision.scala:404)` and `:414` (C2: 28 and 361; C3: 28 and 360), the nesting of the first oracle, until the `StackOverflowError`. Only 6 `started` lines appear (guardian, supervised actor, children) and no child restart cycle.

## 3. Comparison with the code reading (`Supervision.scala:362`, `:384-388`, 1.7.1)

| Scenario | Reading | Observed | Result |
|---|---|---|---|
| C1 | `stopChildren = true` with children to stop: `Behaviors.empty // wait for termination of children` (`:384-388`); asynchronous loop, no stack overflow | no SOE, no end in 60 s, steady rate, no growth of live children | Matches |
| C2 | `withStopChildren(false)`: `childrenToStop` is empty (`:362`), so `restartCompleted` is called directly (`:385-386`); recursion stays synchronous | SOE (3 of 3) with the same ~2000 nested runs and the same `on`/`off` outcomes as CTRL | Matches |
| C3 | the first restart waits for the child; later restarts have no children and recurse | SOE (3 of 3), same outcomes as CTRL | Matches |
| CTRL | control of the first oracle | identical to the first oracle (SOE, 255 with `on`) | Harness unchanged |

## 4. One-line conclusions

- C1: with a `setup` that creates a child on each restart and the default `stopChildren = true`, no stack overflow and no termination in 60 s: an asynchronous loop of about 85,000 restarts per second.
- C2: with `withStopChildren(false)` and the child outside `supervise`, the recursion is synchronous, same as without children (exit 255 with `on`; with `off` the 1.7.1 hang and, with the fixed build, termination).
- C3: a child created while processing the failing message is stopped by the first restart; the later restarts recurse as in the control (exit 255 with `on`; with `off` the 1.7.1 hang and, with the fixed build, termination).

## 5. Limits

Three runs per combination, one OS and one JDK, `-Xmx512m`, a 60 s cut. The used heap is sampled without a forced GC. Other ways of creating children (timers, message adapters) were not run.
