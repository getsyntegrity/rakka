# Oracle: D-a, unbounded `restart` with a failing `setup` (Pekko v1.7.1)

- Date: 2026-11-04
- Purpose: confirm by execution (not by reading code) what Pekko Typed does in case D-a of ADR 0035.
- Pekko artifacts: `pekko-actor-typed` 1.7.1 and `pekko-slf4j` 1.7.1 (the pin of this repository, `upstream/pekko` tag `v1.7.1`), logback-classic 1.3.14.
- Scala 2.13.18 (sbt required `scalaVersion` to be at least 2.13.18, because the Pekko 1.7.1 artifacts bring `scala-library` 2.13.18), sbt 1.10.7, JDK Temurin 11.0.32+9, macOS (Apple silicon), default thread stack size.
- The sbt project lives outside the repository and is not committed; its full source is in this document.

## 1. Result against the hypothesis

The hypothesis was: unbounded `restart` + `setup` that always fails gives a recursive restart that ends in `StackOverflowError`, which is fatal;
with `jvm-exit-on-fatal-error = on` the JVM exits, with `off` the ActorSystem terminates.

| Part of the hypothesis | Observed |
|---|---|
| The case as specified (`setup` throws on the first start, scenarios 1 and 2) ends in `StackOverflowError` | **Not confirmed.** No `StackOverflowError`. The start failure is *not* restarted by an unbounded `restart`: `setup` runs once, the child fails with `ActorInitializationException`, the guardian gets `ChildFailed`, the system keeps running. This matches `Supervision.scala:297-304` ("if unlimited restarts then don't restart if starting fails"). |
| D-a proper (`setup` succeeds once, a message failure triggers a restart, every later `setup` throws; scenarios 5 and 6) ends in `StackOverflowError` | **Confirmed**: after about 2080 nested `setup` runs in under 1 s the thread throws `StackOverflowError`. |
| With `on` the JVM exits | **Confirmed** (scenario 5): exit code 255, about 0.6 s after the start. |
| With `off` the ActorSystem terminates | **Not confirmed** (scenario 6): the shutdown starts (`shutting down ActorSystem`, CoordinatedShutdown runs) but the system does **not** terminate. See section 4. |
| Outside `setup`, unbounded `restart` is indefinite (scenario 4) | Confirmed: 5 failures, 5 restarts, the valid message is processed. |

Conclusion: the case the task text describes is not D-a; D-a is the restart-time variant (scenarios 5 and 6). Scenario 5 confirms the hypothesis for `on`; scenario 6 contradicts it for `off`.

**Later sections supersede the `off` row.** The hang with `off` is the upstream bug apache/pekko#3620, not designed behavior (section 6), and with a build that includes the fix the ActorSystem terminates in about 0.63 s (section 7).

## 2. Test program

`build.sbt`:

```scala
scalaVersion := "2.13.18"
val pv = "1.7.1"
libraryDependencies ++= Seq(
  "org.apache.pekko" %% "pekko-actor-typed" % pv,
  "org.apache.pekko" %% "pekko-slf4j" % pv,
  "ch.qos.logback" % "logback-classic" % "1.3.14")
fork := true
```

`src/main/scala/Oracle.scala` (scenarios 5 and 6 were added after scenarios 1 to 4 showed that the start failure is not restarted):

```scala
import org.apache.pekko.actor.typed._
import org.apache.pekko.actor.typed.scaladsl._
import scala.concurrent.duration._
import java.util.concurrent.atomic.AtomicInteger

object Oracle {
  val setupRuns = new AtomicInteger(0)
  val msgFailures = new AtomicInteger(0)
  def p(s: String): Unit = { println(s"MARK ${System.currentTimeMillis() % 1000000} $s"); System.out.flush() }

  def child(scn: Int): Behavior[String] = scn match {
    case 1 | 2 =>
      Behaviors.supervise(Behaviors.setup[String] { _ => setupRuns.incrementAndGet(); throw new IllegalStateException("boom") })
        .onFailure[IllegalStateException](SupervisorStrategy.restart)
    case 5 | 6 =>
      Behaviors.supervise(Behaviors.setup[String] { _ =>
        if (setupRuns.incrementAndGet() > 1) throw new IllegalStateException("boom")
        Behaviors.receiveMessage[String] { m => throw new IllegalStateException("first-failure") }
      }).onFailure[IllegalStateException](SupervisorStrategy.restart)
    case 3 =>
      Behaviors.supervise(Behaviors.setup[String] { _ => setupRuns.incrementAndGet(); throw new IllegalStateException("boom") })
        .onFailure[IllegalStateException](SupervisorStrategy.restart.withLimit(3, 10.seconds))
    case 4 =>
      Behaviors.supervise(Behaviors.receiveMessage[String] {
        case "bad" => msgFailures.incrementAndGet(); throw new IllegalStateException("boom")
        case m => p(s"child processed valid message: $m"); Behaviors.same
      }).onFailure[IllegalStateException](SupervisorStrategy.restart)
  }

  def guardian(scn: Int): Behavior[String] = Behaviors.setup { ctx =>
    val c = ctx.spawn(child(scn), "child")
    ctx.watch(c)
    if (scn == 5 || scn == 6) c ! "bad"
    if (scn == 4) { (1 to 5).foreach(_ => c ! "bad"); c ! "good" }
    Behaviors.receiveMessage[String] { m => Behaviors.same }
      .receiveSignal {
        case (_, ChildFailed(ref, cause)) => p(s"guardian got ChildFailed($ref, $cause)"); Behaviors.same
        case (_, Terminated(ref))         => p(s"guardian got Terminated($ref)"); Behaviors.same
      }
  }

  def main(args: Array[String]): Unit = {
    val scn = args(0).toInt
    val t0 = System.nanoTime()
    Runtime.getRuntime.addShutdownHook(new Thread(() => p(s"JVM shutdown hook ran; setupRuns=${setupRuns.get} msgFailures=${msgFailures.get} t=${(System.nanoTime()-t0)/1000000}ms")))
    p(s"scenario $scn start")
    val sys = ActorSystem(guardian(scn), "oracle")
    sys.whenTerminated.onComplete(r => p(s"whenTerminated: $r"))(scala.concurrent.ExecutionContext.global)
    sys.classicSystem.registerOnTermination(p("registerOnTermination ran"))
    org.apache.pekko.actor.CoordinatedShutdown(sys.classicSystem).addTask(org.apache.pekko.actor.CoordinatedShutdown.PhaseBeforeServiceUnbind, "marker") { () =>
      p("CoordinatedShutdown task ran"); scala.concurrent.Future.successful(org.apache.pekko.Done) }
    Thread.sleep(20000)
    p(s"main woke after 20s, still alive; setupRuns=${setupRuns.get} msgFailures=${msgFailures.get}; terminating")
    sys.terminate()
    scala.concurrent.Await.result(sys.whenTerminated, 10.seconds)
    p("main done")
  }
}
```

`logback.xml`: console appender, root level INFO. Scenarios 5 and 6 were rerun with `%ex{3}` (three stack frames per exception) because the full stack traces made one hung run write a log of about 234 MB.
Each scenario is its own JVM: `java -cp <sbt runtime classpath> -Dpekko.jvm-exit-on-fatal-error=<on|off> Oracle <n>`.
`on` is Pekko's default (`reference.conf`); scenario 1 passes it explicitly.

## 3. Scenarios and results

| # | Child | `jvm-exit-on-fatal-error` | `StackOverflowError` | Process exit | `registerOnTermination` | JVM shutdown hook | CoordinatedShutdown | Guardian signal | Restarts seen | Time to end |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `setup` always throws, unbounded `restart` | on | no | exit 0 after the test's own 20 s sleep and `terminate()` | yes, at the test's own terminate | yes | yes, at the test's own terminate | `ChildFailed(ActorInitializationException(boom))` | 0 (`setup` ran once) | none; ended at 20 s by the test |
| 2 | same | off | no | same as 1 | same as 1 | same as 1 | same as 1 | `ChildFailed(ActorInitializationException(boom))` | 0 | none; 20 s |
| 3 | `setup` always throws, `withLimit(3, 10.seconds)` | on | no | exit 0 at the test's own end | yes | yes | yes | `ChildFailed(ActorInitializationException(boom))` | 3 (`setup` ran 4 times) | limit reached in under 1 s |
| 4 | message handler throws on `"bad"`, unbounded `restart`; 5 `"bad"` then `"good"` | on | no | exit 0 at the test's own end | yes | yes | yes | none | 5 (one per bad message) | `"good"` processed immediately |
| 5 | `setup` succeeds once, fails on every restart (D-a), unbounded `restart` | on | **yes**, thread `default-dispatcher-5` | **255** | yes (after the hook, during the JVM exit) | yes (`setupRuns=2087`, t=600 ms) | yes, reason `JvmExitReason` | none | about 2087 `setup` runs | about 0.6 s |
| 6 | same as 5 | off | **yes**, thread `default-dispatcher-5` | **the process did not exit**: the main thread timed out waiting 10 s for `whenTerminated` and the JVM stayed alive until killed after more than 5 minutes | **no** | **no** (the JVM never exited) | started (reason `ActorSystemTerminateReason`); phase `actor-system-terminate` timed out after 10 s | none | about 2080 `setup` runs | the system never terminated |

"Guardian signal" is what the guardian's `receiveSignal` logged for the watched child.

## 4. Relevant output

Scenario 1/2 (start failure, no restart):

```
ERROR o.a.p.a.SupervisorStrategy - boom
org.apache.pekko.actor.ActorInitializationException: pekko://oracle/user/child: exception during creation, root cause message: [boom]
MARK guardian got ChildFailed(Actor[pekko://oracle/user/child#...], ...ActorInitializationException: ... [boom])
MARK main woke after 20s, still alive; setupRuns=1 msgFailures=0; terminating
```

Scenario 5 (`on`):

```
MARK scenario 5 start
Uncaught error from thread [oracle-pekko.actor.default-dispatcher-5]: null, shutting down JVM since 'pekko.jvm-exit-on-fatal-error' is enabled for ActorSystem[oracle]
java.lang.StackOverflowError
MARK JVM shutdown hook ran; setupRuns=2087 msgFailures=0 t=600ms
ERROR o.a.p.a.ActorSystemImpl - Uncaught error from thread [...dispatcher-5]: null, shutting down JVM since 'pekko.jvm-exit-on-fatal-error' is enabled
INFO  o.a.p.a.CoordinatedShutdown - Running CoordinatedShutdown with reason [JvmExitReason]
MARK CoordinatedShutdown task ran
MARK registerOnTermination ran
(exit code 255)
```

Scenario 6 (`off`):

```
MARK scenario 6 start
Uncaught error from thread [oracle-pekko.actor.default-dispatcher-5]: null, shutting down ActorSystem[oracle]
java.lang.StackOverflowError
INFO  o.a.p.a.CoordinatedShutdown - Running CoordinatedShutdown with reason [ActorSystemTerminateReason]
MARK CoordinatedShutdown task ran
WARN  o.a.p.a.CoordinatedShutdown - Coordinated shutdown phase [actor-system-terminate] timed out after 10000 milliseconds
MARK main woke after 20s, still alive; setupRuns=2080 msgFailures=0; terminating
Exception in thread "main" java.util.concurrent.TimeoutException: Future timed out after [10 seconds]
```

A `jstack` taken 12 s into scenario 6 shows only the main thread (sleeping), the scheduler and the dispatcher threads idle (`TIMED_WAITING`/`WAITING`); no thread is running the restart.

## 5. Limits of this evidence

- One run per scenario, one JVM and one OS. Restart counts are approximate: they come from the `setupRuns` counter (2080 to 2087) and depend on the stack size.
- Why the system does not terminate in scenario 6 was not investigated; only the observation is recorded.
- Pekko 1.7.1 runs on Scala 2.13 here; the Pekko code under test is the same artifact version as the `upstream/pekko` pin.

## 6. Investigation: `off` hang

Date: 2026-11-04. Same environment as above (Pekko 1.7.1, Scala 2.13.18, JDK Temurin 11.0.32, sbt 1.10.7). JDK 17 and 21 are not installed on the machine, so the JDK-independence check was not run.

**Conclusion: deterministic (7 of 7 runs); likely incidental (a dispatcher defect, fixed after 1.7.1), not a designed behavior.**

### 6.1 Repetitions

Scenario 6 (D-a, `off`), five JVMs, observed for 40 s each, then killed by the harness (the `hookT` time in the logs is that kill, not a termination):

| Run | `StackOverflowError` | Nested `setup` runs | Phase `actor-system-terminate` timed out (10 s) | `whenTerminated` completed | Process alive at 40 s |
|---|---|---|---|---|---|
| 1 | yes | 2046 | yes | no | yes |
| 2 | yes | 2087 | yes | no | yes |
| 3 | yes | 2087 | yes | no | yes |
| 4 | yes | 2090 | yes | no | yes |
| 5 | yes | 2063 | yes | no | yes |

No run terminated, so there is no log that distinguishes a terminating run. Together with the run of section 3 and the short-log rerun, the hang is 7 of 7 (the diagnostic runs of section 6.2 hung too).

Scenario 5 (D-a, `on`), control, three more JVMs. With the run of section 3 and the short-log rerun, scenario 5 has 5 runs on 1.7.1, all with exit code 255:

| Run | `StackOverflowError` | Nested `setup` runs | Exit code | Time to hook |
|---|---|---|---|---|
| 1 | yes | 2087 | 255 | 612 ms |
| 2 | yes | 2038 | 255 | 585 ms |
| 3 | yes | 2086 | 255 | 602 ms |

The `on` case is stable. The nesting depth varies by about 3 percent between runs.

### 6.2 What `actor-system-terminate` waits for

Config used for the dumps: scenario 6 as in section 3 (`logback-short.xml`). Two `jstack` dumps, the first about 30 s after the start (15 s after the error) and the second 10 s later.

Findings from the thread dumps:

- No thread runs actor code and no thread is blocked inside CoordinatedShutdown. The phase is future-based, not a blocked thread: `terminate-system` calls `system.finalTerminate()` and returns `system.whenTerminated` (`CoordinatedShutdown.scala:243-270`, `ActorSystem.scala:1112-1117`).
- All three `default-dispatcher` threads, including `default-dispatcher-5`, where the `StackOverflowError` happened, are alive and idle in `ForkJoinPool.runWorker` (parked). The thread is not dead and not replaced.
- Both dumps are identical apart from timing; the first also shows `main` waiting in `Await.result` for `whenTerminated`, the second shows it gone (the `TimeoutException` of section 4).

```
"oracle-pekko.actor.default-dispatcher-5" TIMED_WAITING (parking)
	at java.util.concurrent.locks.LockSupport.parkUntil(...)
	at java.util.concurrent.ForkJoinPool.runWorker(ForkJoinPool.java:1619)
"oracle-pekko.actor.default-dispatcher-6" WAITING (parking)
	at java.util.concurrent.ForkJoinPool.runWorker(ForkJoinPool.java:1628)
```

Findings from a separate run with `loglevel = DEBUG` and `pekko.actor.debug.lifecycle = on` (log trimmed to the lines after the error):

```
Uncaught error from thread [...default-dispatcher-6]: null, shutting down ActorSystem[oracle]
INFO  CoordinatedShutdown - Running CoordinatedShutdown with reason [ActorSystemTerminateReason]
DEBUG CoordinatedShutdown - Performing phase [actor-system-terminate] with [1] tasks.
DEBUG CoordinatedShutdown - Performing task [terminate-system] in CoordinatedShutdown phase [actor-system-terminate]
DEBUG DeadLetterListener - stopped
DEBUG ActorAdapter - stopping                    <- the user guardian starts terminating
WARN  CoordinatedShutdown - Coordinated shutdown phase [actor-system-terminate] timed out after 10000 milliseconds
```

Only the guardian logs `stopping`; the failing child `/user/child` never logs `stopping` or `stopped`. The guardian waits for its child, the child never processes the `Terminate` system message, so `whenTerminated` never completes.

**Hypothesis (one).** The mailbox of the failing child is permanently dead. `Mailbox` is a `ForkJoinTask` reused for the whole life of the actor, and `Mailbox.exec()` rethrows every `Throwable` that escapes `run()` after calling the uncaught-exception handler (`upstream/pekko/actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:244-258`; `run()` at `:230-240`). The `StackOverflowError` escapes `run()`; `ForkJoinTask.doExec` records an exceptional completion and every later submission of the same object returns without calling `run()`. The child's mailbox then never runs again, and system messages (`Terminate`, `Watch`) go through the same mailbox, so the child cannot be stopped and its parent waits forever.

Supporting evidence:

- Control experiment: scenario 6 with `-Dpekko.actor.default-dispatcher.executor=thread-pool-executor` (and the same for `internal-dispatcher`), which does not run mailboxes through `exec()`. The same `StackOverflowError` happens and the ActorSystem **terminates** in about 0.6 s (`whenTerminated: Success(Done)`, `registerOnTermination` ran).
- The upstream issue describes exactly this mechanism (section 6.3).

What this does not prove: the `Terminate` message never being processed was inferred from the missing `stopping` log, not observed with a debugger.

### 6.3 Is it reported?

| Source | Status | Reading |
|---|---|---|
| [apache/pekko#3620](https://github.com/apache/pekko/issues/3620): "a fatal error or a rejected execution permanently bricks an actor's mailbox, because the reused Mailbox ForkJoinTask completes exceptionally" | closed (completed, 2026-10-13), label `bug`, fixed by [#3621](https://github.com/apache/pekko/pull/3621) | The maintainers treat it as a bug. It states that `Watch`, `Terminate` and `PoisonPill` are dropped too. Fix on `main` (milestone 2.0.0-M5) and backported to 1.8.x by [#3627](https://github.com/apache/pekko/pull/3627). Not in 1.7.1 (the code at `Mailbox.scala:244-258` above is the pre-fix code). No release with the fix exists yet on Maven Central (latest: 1.7.1, 2.0.0-M4). |
| [apache/pekko#3613](https://github.com/apache/pekko/issues/3613): rejected execution strands a mailbox; the reporting path recurses into `StackOverflowError` | open, label `bug` | Related (another `StackOverflowError` plus `jvm-exit-on-fatal-error`); different trigger. Its first defect was fixed by #3621. |
| [ben-manes/caffeine#2021](https://github.com/ben-manes/caffeine/issues/2021) | referenced by the two issues above | Same anti-pattern in another project (a reused `ForkJoinTask`). |
| [Akka user list, "Handling of Fatal errors", Feb 2015](https://groups.google.com/g/akka-user/c/TyWlxjNaBiQ) | no issue link seen | Same symptom with `akka.jvm-exit-on-fatal-error=off`: the failed actor and its parents get no `postStop`, `awaitTermination` blocks. The maintainer replied that no guarantees can be made after a fatal error, recommended forcibly shutting the JVM down, and agreed the documentation should improve. Not labelled bug or expected. The page was truncated, later replies may exist. |
| [discuss.akka.io, StackOverflowError in a remote dispatcher](https://discuss.akka.io/t/error-akka-actor-actorsystemimpl-uncaught-error-from-thread-mpe-akka-remote-default-remote-dispatcher-6-null-shutting-down-jvm-since-akka-jvm-exit-on-fatal-error-is-enabled-for-actorsystem-mpe-java-lang-stackoverflowerror-null/3446) | n/a | Not about a hang. The maintainer says the overflow comes from application code. |

Not found: an `akka/akka` GitHub issue (the repository could not be searched with `gh`), a Pekko mailing-list thread, or any statement that the hang is designed behavior.

### 6.4 One-line conclusion

Deterministic (7 of 7 runs); likely incidental (mailbox reuse defect of the fork-join dispatcher, bug #3620, fixed after 1.7.1).

## 7. Oracle against fixed build (1.8.x)

Date: 2026-11-04. Purpose: observe D-a on a Pekko build that includes the fix of [apache/pekko#3620](https://github.com/apache/pekko/issues/3620) ([#3621](https://github.com/apache/pekko/pull/3621) on `main`, [#3627](https://github.com/apache/pekko/pull/3627) on `1.8.x`).

**Conclusion: with the fix, scenario 5 is unchanged and scenario 6 terminates the ActorSystem in about 0.63 s (5 of 5 runs); the 1.7.1 hang is the #3620 defect.**

### 7.1 Build

- Source: `apache/pekko`, branch `1.8.x`, HEAD `59d8040885ea17675ad63aebc9f746e687306934` (`git describe`: `v1.8.0-M0-1-g59d8040`). The merge of #3627 is in its history: `5b1b0e5 fix: backport mailbox task reuse fix to 1.8.x (#3621, #3628) (#3627)`. `Mailbox.scala` no longer extends `ForkJoinTask`.
- Built outside this repository with `sbt actor/publishLocal actor-typed/publishLocal slf4j/publishLocal`, JDK 11.0.32 (Temurin) running sbt with JDK 8 (Zulu 8.0.504) as `JAVA_8_HOME`, which the Pekko build requires.
- Version produced: `1.8.0-M0+1-59d80408-SNAPSHOT` (Scala 2.13).
- Oracle program: the one of section 2, only the Pekko version changed. Same JDK (Temurin 11.0.32), default fork-join dispatcher, same scenario code.
- Not a release: no published artifact contains the fix (latest on Maven Central: 1.7.1 and 2.0.0-M4).

### 7.2 Runs

Scenario 5 (`on`), `Oracle 5`, three JVMs, plus one with lifecycle debug (`pekko.actor.debug.lifecycle = on`, `loglevel = DEBUG`):

| Run | `StackOverflowError` | Nested `setup` runs | Exit code | Order of events |
|---|---|---|---|---|
| 1 | yes | 2063 | 255 | error log "shutting down JVM since ... is enabled", JVM shutdown hook, CoordinatedShutdown (`JvmExitReason`), `registerOnTermination`, `whenTerminated` |
| 2 | yes | 2087 | 255 | same |
| 3 | yes | 2089 | 255 | same; `whenTerminated` not printed before the exit (race with `System.exit`) |
| debug | yes | 2037 | 255 | same; phases performed with tasks: `before-service-unbind` (1), `actor-system-terminate` (1); the guardian logs `stopping`, then two `stopped` (the child and the guardian) |

Scenario 6 (`off`), `Oracle 6`, five JVMs, plus one with lifecycle debug. In every run the harness `main` thread sleeps 20 s before the process can exit, so the process exit (code 0) comes at about 20 s; the relevant time is `whenTerminated`:

| Run | `StackOverflowError` | Nested `setup` runs | CoordinatedShutdown | Phase timeout | `registerOnTermination` | ActorSystem terminated after | Process exit |
|---|---|---|---|---|---|---|---|
| 1 | yes | 2078 | ran | none | yes | 631 ms | 0 (after the harness sleep) |
| 2 | yes | 2079 | ran | none | yes | 628 ms | 0 |
| 3 | yes | 2087 | ran | none | yes | 629 ms | 0 |
| 4 | yes | 2071 | ran | none | yes | 629 ms | 0 |
| 5 | yes | 2072 | ran | none | yes | 629 ms | 0 |
| debug | yes | 2046 | ran | none | yes | under 15 ms after the error log | 0 |

Order of events in scenario 6 (all runs): error log "Uncaught error ... shutting down ActorSystem", CoordinatedShutdown with reason `ActorSystemTerminateReason`, `before-service-unbind` task, ..., `actor-system-terminate` task, guardian `stopping`, child `stopped`, guardian `stopped`, `registerOnTermination`, `whenTerminated: Success(Done)`. The JVM shutdown hook runs at the end of the harness's own exit. In no run does the guardian receive `ChildFailed` or `Terminated` for the child before the system ends (same as 1.7.1).

### 7.3 Comparison, 1.7.1 vs fixed build

| Point | 5 (`on`), 1.7.1 | 5 (`on`), 1.8.x fixed | 6 (`off`), 1.7.1 | 6 (`off`), 1.8.x fixed |
|---|---|---|---|---|
| `StackOverflowError`, nested runs | yes, ~2038 to 2087 | yes, ~2037 to 2089 | yes, ~2046 to 2090 | yes, ~2046 to 2087 |
| Main error log | "Uncaught error from thread [...dispatcher-N]: null, shutting down JVM since 'pekko.jvm-exit-on-fatal-error' is enabled for ActorSystem[oracle]" | identical | "Uncaught error from thread [...dispatcher-N]: null, shutting down ActorSystem[oracle]" | identical |
| JVM shutdown hook | ran first, ~0.6 s after start | identical | not before the harness exit | at the harness exit (the system had already ended) |
| CoordinatedShutdown | ran, reason `JvmExitReason` | identical | ran, reason `ActorSystemTerminateReason`; `actor-system-terminate` **timed out** after 10 s | ran; **no phase timed out** |
| `registerOnTermination` | ran | ran | **did not run** | ran |
| ActorSystem terminates | n/a (the JVM exits) | n/a | **never** (7 of 7) | **yes, ~0.63 s** (5 of 5) |
| Process exit | 255 (5 of 5) | 255 (4 of 4) | stayed alive until killed | 0 after the harness sleep |
| Child lifecycle | not observed (no lifecycle run on 1.7.1 for scenario 5) | guardian `stopping`, then two `stopped` | child never logs `stopping`/`stopped` | child and guardian `stopped` |
| Guardian `ChildFailed`/`Terminated` | none | none | none | none |

Differences in scenario 5: none observed in behavior or event order. (The hypothesis that scenario 5 does not change holds.) The only change is in scenario 6.

### 7.4 Limits

One OS, one JDK (11), one build commit. Restart counts are approximate and depend on the stack size. The `on` runs end by `System.exit`, so the last events (`whenTerminated`) race with the exit.

Correction (2026-11-05, #976 slice 4): `Supervision.scala:297-300` -> `297-304` (the quoted comment "if unlimited restarts then don't restart if starting fails" is line 301, the condition 302, the rethrow 304; 297-300 only open the `case`); `CoordinatedShutdown.scala:243-266` -> `243-270` (`finalTerminate()` is line 266, the returned `whenTerminated.map` lines 267-270). The `ForkJoinPool.java:1619` and `:1628` lines are frames of a JDK 11 `jstack` dump, not citations into `upstream/pekko`; they are quoted data and are not changed. Every other citation was re-read and matched.
