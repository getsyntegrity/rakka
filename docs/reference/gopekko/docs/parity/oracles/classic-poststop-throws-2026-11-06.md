# Oracle: a watcher of an actor whose `postStop` throws (Pekko Classic v1.7.1)

- Date: 2026-11-06. Evidence only; this document recommends nothing and changes no ADR.
- Pekko `pekko-actor` 1.7.1 (tag `v1.7.1`, `upstream/pekko` at `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), Scala 2.13.18, sbt 1.10.7, JDK Temurin 11.0.32+9, default fork-join dispatcher. The sbt project is outside the repository and is not committed.
- Purpose: the oracle for #1394 ("Actor: watcher receives Terminated when the watched actor's PostStop throws"). Scenario W5c2 of the [restart child-wait oracle](typed-restart-child-wait-2026-11-06.md) saw the effect indirectly (a restart that watches a Typed child whose `PostStop` throws still completes); this oracle runs the direct case with Classic actors.
- Method: read the code (section 1), then run the scenario in its own JVM, three runs; the three runs gave the same lines in the same order after removing the time prefix.

## 1. What the source says (code reading, Pekko 1.7.1)

`F` = `actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala` (first line `/*`).

| # | Claim | Location |
|---|---|---|
| C1 | `finishTerminate` calls `a.aroundPostStop()` inside a `try`; the `catch` at `F:255` uses `handleNonFatalOrInterruptedException`, whose cases are `InterruptedException` and `NonFatal(e)` (`F:353-358`), and publishes an `Error` event (cause, the actor's path, its class, the message) (`F:255-257`) | `F:254-257`, `F:353-358` |
| C2 | The rest of the termination is a chain of nested `finally` blocks, so it runs whether or not `aroundPostStop` threw: `stopFunctionRefs`, `dispatcher.detach`, the `DeathWatchNotification` to the parent, `tellWatchersWeDied()`, `unwatchWatchedActors`, `clearActorFields` | `F:258-275` |
| C3 | The comment above says the order is crucial and that, when a parent is also a watcher, `ChildTerminated` and `Terminated` must be processed in a specific order | `F:249-253` |

## 2. Scenario and result

**PS1.** A Classic actor `T` whose `postStop` logs `T.postStop begins` and then throws `IllegalStateException("postStop-fail")`. A Classic actor `W` watches `T` (`context.watch`) and logs every `Terminated`. A listener subscribed to `Logging.Error` logs the events. The harness sends `PoisonPill` to `T`, waits, sends `"ping"` to `W`, waits and ends.

| Line of the raw log (`ps/PS.1.txt`) | Observed |
|---|---|
| [001] | `-- t is stopped (PoisonPill)` (the harness) |
| [002] | `T.postStop begins` |
| [003] | `EVENT Error cause=IllegalStateException msg=postStop-fail source=pekko://ps/user/t`: one `Error` event, with the cause of `postStop` |
| [004] | `W.Terminated(t) existenceConfirmed`: `W` received `Terminated` for `T`, once |
| [005] | `W.ping`: `W` is alive and handles a later message |

The order [002], [003], [004] is the order of the lines. No second `Terminated` appears in the log.

| Inferred (from the code of section 1; no log line shows it) |
|---|
| `Terminated` is sent by `tellWatchersWeDied()` in a `finally` that runs after the `catch` of C1 (C2). |

## 3. Summary

| Scenario | One line |
|---|---|
| PS1 | When `postStop` throws, an `Error` event with the cause is published and the watcher still receives exactly one `Terminated` for the actor, after `postStop` began; the watcher keeps working |

## 4. Limits of this evidence

- `runtime.Goexit` (a goroutine ending without a panic) has no JVM counterpart: no Pekko scenario exists for it. What a Go runtime does with it is a goPekko decision (ADR 0024) and the existing tests `TestPostStopPanicAndGoexitAtEveryLevelStillComplete` and `TestGoexitInPostStopIsNotAFailure` of `pekko-actor`.
- One watcher, one failing `postStop`; no `Error` thrown by the watcher, no remote watch.
- The raw log is `ps/PS.{1,2,3}.txt` of the harness directory, outside the repository.
