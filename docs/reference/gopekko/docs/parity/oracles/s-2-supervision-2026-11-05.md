# Oracle: S-2, `withLimit` with a match-all matcher under a Classic parent (Pekko v1.7.1)

- Date: 2026-11-05. Evidence only; this document recommends nothing and changes no ADR or "To be verified" state.
- Pekko `pekko-actor-typed` and `pekko-slf4j` 1.7.1 (tag `v1.7.1`), Scala 2.13.18, sbt 1.10.7, JDK Temurin 11.0.32+9, default fork-join dispatcher, logback-classic 1.3.14, `pekko.actor.debug.lifecycle = on`, log level DEBUG. The sbt project is outside the repository and is not committed. Two runs per scenario; the two runs of every scenario produced the same sequence of events (only actor uids differ).
- Text under test: ADR 0035, "Amendment (accepted): decisions S-2 to S-5", section S-2 (in [ADR 0035](../../adr/0035-typed-supervision.md)). `S` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala`; `AA` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/`; `FH` = `actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala`.

## 1. Claims of S-2

Quoted from ADR 0035, S-2: "When the limit is exceeded Pekko rethrows the cause (`S:335-337`). The rethrown cause reaches the Classic parent, whose strategy decides: under the default strategy an `Exception` restarts the actor (`setup` runs again, the restart count starts again, the children are kept and restarted), a `DeathPactException` stops it, and any other non-fatal cause escalates. A start failure always stops it."

| Id | Claim | Support | Scenario |
|---|---|---|---|
| S-2.1 | When the limit is exceeded, Pekko rethrows the cause | `S:335-337` (`case _: Restart => throw t`) | P1, P2, P3 |
| S-2.2 | The rethrown cause reaches the Classic parent, whose strategy decides | `S:335-337`; the supervisor is the user's match-all `supervise` (class exactly `Throwable`) | P1 to P3 |
| S-2.3 | Default strategy and an `Exception`: the actor restarts | classic `defaultDecider` (`FH:218-223`) | P1 |
| S-2.3a | ... `setup` runs again | `S:386, 404` | P1 |
| S-2.3b | ... the restart count starts again | a new instance has a new `RestartSupervisor` with `restartCount = 0` (`S:211`) | P1 |
| S-2.3c | ... the children are kept and restarted | classic restart keeps and restarts children | P1 |
| S-2.4 | A `DeathPactException` stops the actor | classic `defaultDecider` (`FH:218-223`) | P2 |
| S-2.5 | Any other non-fatal cause escalates | classic `defaultDecider` | P3 |
| S-2.6 | A start failure always stops it | `S:294-309` | P4a, P4b |
| (matrix) | Contrast: under a Typed parent the child is stopped (ADR 0035, contract matrix M-A) | `AA/ActorAdapter.scala:232-259` | P5 |

All are observable from outside (restarts, signals, the parent's decider, lifecycle logs); none is unobservable.

## 2. Harness (relevant parts)

```scala
def child(scn: String) = Behaviors.supervise(Behaviors.setup[String] { ctx =>
  val n = setupRuns.incrementAndGet(); p(s"CHILD setup run=$n")
  if (scn == "P4a" || (scn == "P4b" && n > 3)) throw new IllegalStateException("setup-boom")
  ctx.spawnAnonymous(grandchild)                       // a child of the supervised actor
  Behaviors.receiveMessage[String] {
    case "boom"      => throw new IllegalStateException("boom")
    case "deathpact" => throw DeathPactException(ctx.self.toClassic)
    case "error"     => throw new AssertionError("err")
    case o           => p(s"CHILD processed $o (setupRun=$n)"); Behaviors.same
  }.receiveSignal { case (_, s) => p(s"CHILD signal $s (setupRun=$n)"); Behaviors.same }
}).onFailure[Throwable](SupervisorStrategy.restart.withLimit(2, 10.seconds))   // match-all, limit 2

class Parent(scn: String) extends classic.Actor {           // Classic parent, default decisions
  val kid = context.spawn(child(scn), "child"); context.watch(kid.toClassic)
  override val supervisorStrategy = OneForOneStrategy() { case t =>
    val d = SupervisorStrategy.defaultDecider.applyOrElse(t, (_: Throwable) => SupervisorStrategy.Escalate)
    p(s"PARENT decider saw ${t.getClass.getName}: ${t.getMessage} -> $d"); d }   // prints, then applies the default
  def receive = { case Terminated(r) => p(s"PARENT got Terminated($r)"); case m: String => kid ! m }
}
```

The parent's decider prints the cause and returns exactly what the default decider returns, so the decisions are the default strategy's. Scenarios: P1 `"boom"` x3, then `"hello"`, then `"boom"` x3, then `"hello2"`; P2 `"deathpact"` x3; P3 `"error"` x3; P4a `setup` always throws; P4b `"boom"` x3 and the `setup` fails from its fourth run on; P5 the same child under a Typed parent that watches it.

## 3. Results

| Scenario | Restarts and `setup` runs | Signals to the behavior | Parent | Final state of the actor and the system | Supervision logs |
|---|---|---|---|---|---|
| P1 Exception, Classic parent | `setup` ran 7 times: failures 1 and 2 of each cycle are restarts of the Typed supervisor (runs 2, 3 and 5, 6); failure 3 is rethrown and the parent decides Restart (runs 4 and 7); the count starts again (2 more restarts, then the next rethrow) | `PreRestart` 6 times (at each failure that restarts, rethrow included); `PostStop` only at the end | decider saw `IllegalStateException: boom -> Restart` twice (one per cycle); no `Terminated` | the actor processes `hello` and `hello2` after each cycle; alive; system alive | `ERROR Behavior$ - Supervisor RestartSupervisor saw failure: boom` for each Typed restart (4 times), `ERROR OneForOneStrategy - boom` for each rethrown failure (2 times) |
| P2 `DeathPactException` after the limit | 3 `setup` runs (two restarts, then the rethrow) | `PreRestart` 2 times, `PostStop` once | decider saw `DeathPactException -> Stop`; the parent receives `Terminated(child)` (no `ChildFailed`: a Classic parent) | the actor is stopped; the system is alive | the two Typed restarts are logged by `RestartSupervisor`; the rethrown one by the parent |
| P3 non-fatal `Error` (`AssertionError`) after the limit | 4 `setup` runs | `PreRestart` 2 times, `PostStop` twice | decider saw `AssertionError -> Escalate`; the parent fails; its own parent (the Typed guardian) restarts the parent, which creates a new child (`setup` run 4) | after the escalation the new instance of the parent and its child are alive; system alive | as P1 for the first two failures; the escalation is logged by the supervisor chain |
| P4a `setup` fails from the first start (limit 2) | 3 `setup` runs (the start is retried twice, `S:294-309`) | none (the behavior never started) | the parent's decider is **not** consulted; `Terminated(child)` | the actor is stopped; system alive | `RestartSupervisor saw failure: setup-boom` twice, then `Supervisor StopSupervisor saw failure: setup-boom` (the default stop wrapper is still outermost at start) |
| P4b limit exceeded, then the new instance's `setup` fails | 9 `setup` runs: runs 4, 5, 6 fail (two retries, then a rethrown failure that the parent decides `Restart`), runs 7, 8, 9 fail again and the actor stops | `PreRestart` 3 times (none for the failing starts); no `PostStop` (the behavior never started) | decider saw `IllegalStateException: boom -> Restart` and `IllegalStateException: setup-boom -> Restart`; then `Terminated(child)` | the actor is stopped after 9 runs; system alive | `RestartSupervisor saw failure: setup-boom` (4 times) and `OneForOneStrategy - setup-boom` |
| P5 Typed parent (contrast) | 3 `setup` runs (two restarts, then the rethrow) | `PreRestart` 2 times, `PostStop` once | `ChildFailed(child, IllegalStateException)`; no restart | the actor is stopped; system alive | `ERROR SupervisorStrategy - boom` for the rethrown failure |

Children in P1: at the first rethrow, the actor's existing child is **restarted** (it logs `started` again with its old name) and the new `setup` creates another child, so two children exist after the parent's restart; at the next restart both are stopped (two `PostStop` lines). At each restart of the Typed supervisor the child is stopped and a new one is created (one `PostStop`, one new `started`).

## 4. Findings vs ADR 0035

| Claim | Result | Evidence |
|---|---|---|
| S-2.1 rethrown at the limit | **Confirmed** | P1, P2, P3, P4b: the parent's decider receives the cause after two Typed restarts |
| S-2.2 reaches the Classic parent, which decides | **Confirmed** | the parent's decider output in P1 to P4b |
| S-2.3 `Exception`: restart | **Confirmed** | P1: decider `-> Restart`, then a new instance |
| S-2.3a `setup` runs again | **Confirmed** | P1: `setup` run 4 after the rethrow |
| S-2.3b the restart count starts again | **Confirmed** | P1: two more Typed restarts after each parent restart, then the next rethrow |
| S-2.3c children kept and restarted | **Confirmed, with a qualification the ADR does not state**: the old child is restarted and the new `setup` creates another one, so the actor has two children after the parent's restart | P1 lifecycle lines |
| S-2.4 `DeathPactException`: stop | **Confirmed** | P2: `-> Stop`, `Terminated` to the parent |
| S-2.5 other non-fatal cause: escalate | **Confirmed** for the decision (`-> Escalate`); the ADR does not say what happens next: here the parent was restarted by the Typed guardian and its child came up again | P3 |
| S-2.6 a start failure always stops it | **Refuted as stated** (it does not stop at the first failure): an initial start failure is retried up to the limit, then the actor stops without consulting the parent (P4a); a start failure after the parent's restart is retried, reaches the parent's decider (`Restart`) once more, and the actor stops after 9 runs (P4b). The actor does end stopped in both. | P4a, P4b; `S:294-309` |
| (contrast) Typed parent stops the child | **Confirmed** (ADR 0035 matrix) | P5: `ChildFailed`, no restart |

Not covered: a limit with a time window that expires (only the count was used), fatal `Error` subclasses (`VirtualMachineError`), and parents with a non-default strategy.

## 5. Limits

Two runs per scenario on one OS and JDK; the sequence was the same in both. The parent's decider wrapper prints and then applies the default decider; it was not compared with a parent that has no `supervisorStrategy` override.

## 6. Addendum (2026-11-05, later): initial start with small limits, and the trace of P4b

Evidence only; it recommends nothing and changes no ADR. Same harness as section 2, with the limit as a parameter (`-Dlim`: `withLimit(Integer.getInteger("lim", 2), 10.seconds)`) and a variant without the grandchild (`-Dnokid`: `setup` skips `ctx.spawnAnonymous(grandchild)`); scenarios L and LT are P4a and P5 with a `setup` that always fails; two runs per configuration; the two runs gave the same sequence.

### 6.1 First `setup` always fails, `restart.withLimit(n, 10.seconds)`

| Scenario | n | `setup` runs | Retry | Signal to the parent | Final state |
|---|---|---|---|---|---|
| L (Classic parent) | 0 | 1 | none | `Terminated(child)`; the parent's decider is not consulted | stopped; system alive |
| L (Classic parent) | 1 | 1 | none | same | same |
| L (Classic parent) | 2 | 3 | 2 restarts | same | same (this is P4a) |
| L (Classic parent) | 3 | 4 | 3 restarts | same | same (the 4 runs of scenario 3 of the first oracle) |
| LT (Typed parent that watches) | 0 | 1 | none | `ChildFailed(child, ActorInitializationException)` | stopped; system alive |
| LT (Typed parent that watches) | 1 | 1 | none | same | same |
| LT (Typed parent that watches) | 2 | 3 | 2 restarts | same | same |

So with n ≤ 1 the initial start is not retried and with n ≥ 2 it runs n + 1 times. This agrees with `S:302` (`(restartCount + 1) >= strategy.maxRestarts` throws on the first failure when n ≤ 1).

### 6.2 Why P4b has no third consultation of the parent's decider (source trace)

Where a rethrow lands depends on whether it happens while the supervisor's `aroundStart` is still running. During the start the default stop wrapper is still outermost; it is dropped only when the start returns (`deduplicate`, `InterceptorImpl.scala:82-85, 106-122`; `isSame` by throwable class, `S:78-83`). An exception that escapes the restart supervisor during the start is caught by it: `StopSupervisor` logs it and returns `failed(t)` (`S:162-166`), so the actor stops and the parent is not consulted. An exception that escapes later, from `aroundSignal` or `aroundReceive`, goes to the actor and to the parent's decider. The stack of each log line shows which case it was.

| Run | What happened (P4b, 2 runs) | Source |
|---|---|---|
| 1 to 3 | Normal; `boom` x3: two Typed restarts, the third failure is rethrown from `aroundReceive`; the decider says Restart | `S:333-337` |
| 4 | New instance (Classic restart): `postRestart` starts a new `RestartSupervisor` with `restartCount = 0`; the start fails; `0 + 1 >= 2` is false, so `prepareRestart`. The kept child of the old instance still exists, so the restart waits for its `Terminated` and the start returns `Behaviors.empty`; the outer stop wrapper is dropped | `ActorAdapter.scala:317-322`; `S:302-309`, `S:385-388` |
| 5 | The `Terminated` arrives; `restartCompleted` runs `setup` again (stack: `aroundSignal`, `Supervision.scala:259`); it fails; count 1 < 2, so another restart; no children left | `S:256-259, 333-349, 414` |
| 6 | `setup` fails in the synchronous restart; count 2 >= 2: rethrown **from `aroundSignal`** (after the start). The decider receives `setup-boom` and says Restart | `S:335-337`; log "OneForOneStrategy - setup-boom" |
| 7 | New instance, `restartCount = 0`; the start fails; `prepareRestart`; **no children** (stopped by `prepareRestart` in run 4; their `Terminated` was handled in run 5; `Supervision.scala:355-370`), so `restartCompleted` runs inside the start | `S:385-386` |
| 8 | Fails (stack: `aroundStart`, `Supervision.scala:88`, via `handleExceptionOnStart`, `S:306`); count 1 < 2: another synchronous restart | `S:333-349` |
| 9 | Fails; count 2 >= 2: rethrown, but still **inside `aroundStart`**: `StopSupervisor` catches it ("Supervisor StopSupervisor saw failure: setup-boom") and the actor stops. The decider is not consulted; the parent gets `Terminated(child)` | `S:162-166`, `S:335-337` |

Control P4c (same as P4b but `setup` spawns no child; 2 runs): `setup` ran 6 times; the decider was consulted once (`boom`, run 3); runs 4 to 6 happened inside the start (no children to wait for), the rethrow at run 6 was caught by `StopSupervisor`, and the parent received `Terminated(child)`. This is what the trace predicts. P4b re-run twice for this addendum: 9 runs, two decider consultations, same as section 3.

Not covered: a time window that expires during these sequences; `restartWithBackoff`; parents with a non-default strategy.

