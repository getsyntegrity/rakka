# Oracle: Typed `Resume` supervision (Pekko v1.7.1)

- Date: 2026-11-06. Evidence only; this document recommends nothing and changes no ADR.
- Pekko `pekko-actor-typed` 1.7.1 and `pekko-slf4j` 1.7.1 (tag `v1.7.1`, `upstream/pekko` at `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), Scala 2.13.18, sbt 1.10.7, JDK Temurin 11.0.32+9, default fork-join dispatcher, logback-classic 1.3.14 (logging off). The sbt project is outside the repository and is not committed.
- Purpose: the oracle that ADR 0033 D13 asks for before the scope of #1183 (TYP-49, Resume): what Resume keeps and does not do (R1, R2, R3), a failure of the initial start (R4), Resume nested with Restart (R5) and what the parent sees (R6).
- Method: read the code (section 1), then run each scenario in its own JVM, three runs per scenario. Every scenario gave the same set of events in the three runs, and the same order, except that the path hash after `#` in an actor path (`Actor[.../A/k1#<hash>]`) differs between runs and, in R3, the order of two lines that belong to different actors is not fixed (stated there). The harness is a Typed guardian `P` that spawns the supervised actor `A` and watches it (for R6, a Classic parent whose `supervisorStrategy` logs every failure it is asked to decide and answers `Stop`, and which watches `A`). `A` logs every `Setup`, message and signal. Failure types: `E1` extends `RuntimeException`, `E2` extends `E1`, `E3` extends `RuntimeException`. The state is a counter in a `var` created inside `Setup`.

## 1. What the source says (code reading, Pekko 1.7.1)

`S` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala` (first line `/*`); `SO` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala` (first line `/*`); `B` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/Behavior.scala` (first line `/*`); `SS` = `actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala` (first line `/*`); `AC` = `actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/ActorContextSpec.scala` (first line `/*`).

| # | Claim | Location |
|---|---|---|
| C1 | `SupervisorStrategy.resume` is `Resume(loggingEnabled = true, logLevel = ERROR)`; `Resume` is internal | `SO:35`, `SO:125` |
| C2 | `Supervisor.apply` builds a `ResumeSupervisor` for a `Resume` strategy | `S:56-58` |
| C3 | A supervisor catches a throwable only if its class is assignable from the unwrapped exception's class (`UnstashException.unwrap`) | `S:75-76` |
| C4 | Two supervisors are the same (the enclosing one disappears) when their throwable classes are equal | `S:78-83` |
| C5 | `aroundStart` catches with `handleExceptionOnStart`; `aroundSignal` with `handleSignalException`; `aroundReceive` with `handleReceiveException` | `S:85-88`, `S:91-94`, `S:139-143` |
| C6 | For `SimpleSupervisor` (which `ResumeSupervisor` extends) the start, signal and receive handlers are all `handleException` | `S:151-156` |
| C7 | `ResumeSupervisor.handleException` catches a non-fatal throwable of the class, logs it, and returns the `UnstashException`'s behavior or `Behaviors.same`; it never calls `PreRestart`, never touches children | `S:169-178` |
| C8 | A start that returns `Same` or `Unhandled` fails with `IllegalArgumentException("cannot use <behavior> as initial behavior")` | `B:240-243` |
| C9 | The specs for Resume: `SS:163` (BehaviorTestKit), `SS:834` (actor), nested/flatten with restart `SS:201-211`, `SS:849`, a failing deferred factory `SS:1107`, the strategy matrix `SS:1654-1690` (the "stopped after restart" case is skipped for Resume, `SS:1671`), and `AC:281` "not reset behavior upon resume" (a counter created in `setup` keeps counting) | as listed |

## 2. Scenarios and results

Each run ends when the harness terminates the system, which gives `A` its `PostStop` signal if it is still alive.

### R1. Resume keeps the state, does not rerun `Setup`, drops the failed message

`supervise(target).onFailure[E1](resume)`; script: `Inc, Inc, Get`; `Throw(E2)` (a subtype), `Get`; `Inc, Throw(E1), Inc, Get`.

Observed: `A.setup#1`; `Get` reported `state=2 setups=1`; after `Throw(E2)` `Get` reported `state=2 setups=1`; after `Inc, Throw(E1), Inc` `Get` reported `state=4 setups=1`. `setups=1` at the end. The message that threw was not redelivered.

### R2. No `PreRestart`, no `PostStop`

Same run as R1: no `A.signal` line appears between the failures. The only `A.signal` is `PostStop`, logged once, after the script, when the system terminated.

### R1b. A failure that does not match is not resumed

`onFailure[E1](resume)`; `Inc, Get`; `Throw(E3)`; `Get`.

Observed: `A.throws E3`, then `A.signal PostStop`, then `P.signal ChildFailed(cause=E3, msg=e3, no cause.cause)`. The second `Get` was not handled. `setups=1`.

### R1c1. A failure in a signal handler is resumed

`onFailure[E1](resume)`; `A` has a child `k1`, watches it, and its signal handler throws `E1` on `Terminated`. Script: `Inc, Kid, Watch, Get`; stop `k1`; (wait) `Inc, Get`.

Observed: `kid[k1] stopping`, `kid[k1].PostStop`, `A.signal Terminated(k1) -> throws E1`, then `A.get state=2 setups=1 children=List()`: the actor lives and the state was kept. No `PreRestart`, no `PostStop` for `A` before the termination of the system.

### R1c2. The death pact under Resume

`onFailure[DeathPactException](resume)`; `A` has no signal handler, watches `k1`. Script: `Inc, Kid, Watch, Get`; stop `k1`; (wait 500 ms) `Inc, Get`; `Get`.

Observed (3 of 3): `A.get state=1 setups=1 children=List(k1)`; `kid[k1] stopping`; `kid[k1].PostStop`; then, after the child had ended, `A.get state=2 setups=1 children=List()` twice. So after the child's termination `A` answered, its counter had the value it had before plus the later `Inc`, `Setup` had run once, and nothing reached `P`. `A` has no signal handler and logging is off, so the unhandled `Terminated` itself is not in the log.

Control R1c2ctl (3 of 3), the same script without any `supervise`: after `kid[k1].PostStop`, `P.signal ChildFailed(cause=DeathPactException, "death pact with Actor[.../A/k1#<hash>] was triggered", no cause.cause)`; the later `Inc, Get` produced no `A.get`; `setups=1`.

### R3. Resume with children and watches

`onFailure[E1](resume)`; `A` has children `k1`, `k2`; `k1` watched with `watch`, `k2` with `watchWith(Pong(k2))`. `Throw(E1)`, `Get`; then `k1`, `k2` are stopped from outside, `Get`.

Observed: after the resumed failure `Get` reported `state=1 setups=1 children=List(k1, k2)`; neither child logged `PostStop` at the failure. After the children stopped: `kid[k1].PostStop`, `kid[k2].PostStop`, `A.signal Terminated(k1)` and `A.watchWith-message k2`, both delivered to the same incarnation (`setups=1`). The order of these two lines, and of the `kid[k1]`/`kid[k2]` lines before them, differs between runs (run 1: `Terminated(k1)` then the message; runs 2 and 3: the message then `Terminated(k1)`).

### R4. Failure during the initial start

| Run | Configuration | Observed (3 of 3) |
|---|---|---|
| R4a | `onFailure[E1](resume)`, `Setup` throws `E1` | `A.setup#1` once; `P.signal ChildFailed(cause=ActorInitializationException, "exception during creation, root cause message: [cannot use Same as initial behavior]", cause.cause=IllegalArgumentException: cannot use Same as initial behavior)`. The user's `E1` is not in the chain. `setups=1`; `A` did not run |
| R4c | `supervise(supervise(target).onFailure[E1](resume)).onFailure[Exception](restart.withLimit(3, 10 s))`, `Setup` throws `E1` | the same as R4a: `setups=1`, the outer `restart` did not retry, `P` got the same `ChildFailed(ActorInitializationException ... IllegalArgumentException: cannot use Same as initial behavior)` |

### R5. Resume nested with Restart

`E3` extends `RuntimeException`, `E2` extends `E1`. `Get` prints the counter; "restart" prints `A.signal PreRestart` and `A.setup#2`.

| Run | Supervisors (outer ← inner) | Script and observed (3 of 3) |
|---|---|---|
| R5a | `restart[E3]` ← `resume[E2]` | `Inc`; `E2`: state kept (1); `E3`: `PreRestart`, `setup#2`, state 0; `Inc, E2`: state kept (1); `E1` (matches neither): `PostStop`, `P.signal ChildFailed(cause=E1)`. After the restart the inner Resume is still in force |
| R5b | `resume[E3]` ← `restart[E2]` | `Inc`; `E3`: state kept (1); `E2`: `PreRestart`, `setup#2`, state 0; `E1`: `PostStop`, `ChildFailed(E1)` |
| R5c | `restart[E1]` ← `resume[E1]` (equal classes) | `Inc`; `E1`: state kept (1); `E2`: state kept (1); no `PreRestart` at any point; `setups=1` |
| R5d | `restart[E1]` ← `resume[E2]` | `Inc`; `E2`: state kept (1); `E1`: `PreRestart`, `setup#2`, state 0 |
| R5e | `restart[E2]` ← `resume[E1]` | `Inc`; `E2`: state kept (1); `E1`: state kept (1); `setups=1` (the inner, broader Resume handles `E2` although the outer Restart names it) |

### R6. What the parent sees

Classic parent `CP` with a decider that logs and answers `Stop`, watching `A` (spawned by `context.spawn`).

| Run | Case | Observed (3 of 3) |
|---|---|---|
| R6a | `resume[E1]`, `Inc`, `Throw(E1)`, `Get` | `A.throws E1`, `A.get state=1 setups=1`; the decider was not consulted; no `Terminated` |
| R6b | `resume[E1]`, `Throw(E3)` | `A.throws E3`, `A.signal PostStop`, `CP.Terminated(A)`; the decider was not consulted |
| R6c | `resume[E1]`, `Setup` throws `E1` | `A.setup#1`; `CP.decider consulted: ActorInitializationException msg=".../A: exception during creation, root cause message: [cannot use Same as initial behavior]" cause=IllegalArgumentException: cannot use Same as initial behavior`; then `CP.Terminated(A)` |

With a Typed parent the same three cases appear in R1, R1b and R4a: a resumed failure reaches the parent with nothing (R1); an unmatched failure gives `ChildFailed(cause=<the user's exception>)` after `PostStop` (R1b); a failed start gives `ChildFailed(ActorInitializationException)` whose root cause is the `IllegalArgumentException` (R4a).

## 3. Summary

| Scenario | One line |
|---|---|
| R1 | A matching failure (including a subtype) leaves the behavior and state as they were, drops only the failed message and does not rerun `Setup` |
| R2 | A resume sends no `PreRestart` and no `PostStop` |
| R1c2 | Under `resume` for `DeathPactException` an actor whose watched child ends keeps answering with its state; the same actor without `supervise` is stopped with `ChildFailed(DeathPactException)` |
| R3 | Resume does not touch children, `watch` or `watchWith`; a later `Terminated` and `watchWith` message reach the same incarnation (in either order) |
| R4 | A failing initial `Setup` under Resume is not resumed and is not retried by an outer Restart either: the actor fails to start with `ActorInitializationException` whose root cause is `IllegalArgumentException("cannot use Same as initial behavior")`, not the user's error |
| R5 | The innermost matching supervisor decides; after a restart the rebuilt inner Resume is in force; equal classes make the outer disappear; a broader inner Resume takes a failure that a narrower outer Restart names |
| R6 | A resumed failure is not reported to the parent (no decider call, no signal); an unmatched failure is a plain stop; a failed start is an initialization failure the parent decides |

## 4. Limits of this evidence

- The death pact in R1c2 is inferred from the effect (the actor answers afterwards; the control without `supervise` is stopped); it is not logged, because the actor has no signal handler and logging is off. The failure of a signal handler was run with a thrown `E1` (R1c1). The logged-failure line that Resume writes (`log(ctx, t)`, `S:172`, ERROR by default) was not captured (logging is off in the harness).
- `UnstashException` under Resume (`S:174`) was not run.
- The harness is not committed (see the header).
