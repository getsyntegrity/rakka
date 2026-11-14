# Oracle: Typed restart, the wait for children, and a stop during it (Pekko v1.7.1)

- Date: 2026-11-06. Evidence only; this document recommends nothing and changes no ADR.
- Pekko `pekko-actor-typed` 1.7.1 and `pekko-slf4j` 1.7.1 (tag `v1.7.1`, `upstream/pekko` at `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), Scala 2.13.18, sbt 1.10.7, JDK Temurin 11.0.32+9, default fork-join dispatcher, logback-classic 1.3.14 (logging off). The sbt project is outside the repository and is not committed.
- Purpose: the oracle that ADR 0033 D13 asks for before the scope of #997 (TYP-12, children during a Typed restart): what happens when the actor is stopped while a restart waits for its children (W1, W2), what happens to what was retained (W3), a child that never terminates (W4), and the child cases beyond SupervisionSpec's (W5).
- Method: read the code (section 1), then run each scenario in its own JVM, three runs per scenario (the 3 runs gave the same events in the same order wherever order is stated, and the same counters everywhere). The harness is a guardian `P` that spawns the supervised actor `A` (`supervise(...).onFailure[IllegalStateException](strategy)`, strategy `restart` unless a scenario says otherwise), watches it, and can `context.stop(A)`; `A` logs every `Setup`, message and signal; children block in `PostStop` on a latch the harness releases, so the harness decides when a child terminates. `P` subscribes to `DeadLetter`, `Dropped` and `UnhandledMessage`. Timings in the logs below are omitted; the order is the one observed.

## 1. What the source says (code reading, Pekko 1.7.1)

`S` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/Supervision.scala` (first line `/*`); `SS` = `actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala` (first line `/*`); `SO` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/SupervisorStrategy.scala` (first line `/*`); `SB` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala` (first line `/*`); `AA` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala` (first line `/*`); `RC` = `actor-typed/src/main/resources/reference.conf` (first line `# SPDX-License-Identifier: Apache-2.0`).

| # | Claim | Location |
|---|---|---|
| C1 | With `stopChildren` the children to stop are all of `ctx.children`; without it the set is empty | `S:362` |
| C2 | Each child is `unwatch`ed, `watch`ed and `stop`ped (the `unwatch` revokes a `watchWith`) | `S:421-427` |
| C3 | With `Restart` and a non-empty set the supervisor returns `Behaviors.empty` and waits; with an empty set it completes at once | `S:385-388` |
| C4 | While `restartingInProgress`, a signal that is not the `Terminated` of an awaited child is stashed (or published as `Dropped` when the stash is full) | `S:265-270` |
| C5 | While `restartingInProgress`, a message is stashed (or `Dropped` when the stash is full) | `S:280-286` |
| C6 | The `Terminated` of an awaited child is consumed (`Terminated(ref)` is also the extractor of `ChildFailed`); when the last one arrives the restart completes | `S:256-262` |
| C7 | The stash capacity is `strategy.stashCapacity` or `restart-stash-capacity` (default 1000) | `S:365-367`, `RC:27` |
| C8 | `Dropped(message, "Stash is full in [RestartSupervisor]", self)` is published on the classic event stream | `S:119-124` |
| C9 | `ActorAdapter.postStop` gives `PostStop` to the current behavior through `Behavior.interpretSignal`, so it reaches the supervisor's `aroundSignal` | `AA:325-331` |
| C10 | `unstashAll` stops replaying when the behavior is no longer alive or no message is left | `SB:156`, `SB:195` |
| C11 | Nothing in the restart path defines a timeout for the wait; `reference.conf` has no key about it | `S:385-388`, `RC` (only `restart-stash-capacity`, lines 23-27, concerns restart) |
| C12 | `withStopChildren(enabled)`, default `true` | `SO:162, 173-174, 199, 215-216` |
| C13 | The specs cover: two slow children and a stashed `GetState` and `Terminated` (`SS:575-616`), `watchWith` revocation (`SS:618-660`), `stopChildren = false` (`SS:662-689`), a second failure during the replay (`SS:691-734`), a failing first and later `Setup` with children (`SS:736-830`). None of them stops the actor during the wait. A search of `SS` for `PoisonPill`, `terminate()`, `context.stop` and `.stop(` finds only `anotherProbe.stop()` (`SS:604, 646`, stopping a watched actor, not `A`); `Behaviors.stopped` appears in `SS:1661-1690` ("stopped after restart": the new `Setup` returns `stopped`, one restart, no children) and in unrelated specs | `SS:575-831`, `SS:1655-1690` |

## 2. Scenarios and results

`A` has the children named in each scenario; "wait" means the restart has stopped them and some child's `PostStop` is still blocked.

### W1. The parent stops `A` (`context.stop`) while the restart waits

`A` has two children `c1`, `c2` (slow `PostStop`). `m-during-wait` is sent, then `P` calls `context.stop(A)`, then the harness releases `c1`, then `c2`, then sends `after-stop`.

Observed events, in order: `A.signal PreRestart`; `c1.PostStop begin`, `c2.PostStop begin`; `P ctx.stop(A)`; `c1.PostStop end` (c1 released); `c2.PostStop end` (c2 released); `P.signal Terminated(A)`; `DeadLetter(Msg(after-stop))`.

| Question | Answer (3 of 3 runs) |
|---|---|
| Does the new `Setup` run? | No. `Setup` ran once (`setups=1`) |
| `PostStop` to `A`'s behavior? | No `A.signal PostStop` appears (control W1b below shows it when there is no wait) |
| Signals `A`'s behavior got after `PreRestart` | None |
| Does `m-during-wait` reach any behavior? | No; it is not a dead letter either (`deadLetters=1`, the `after-stop`) |
| Do the awaited children terminate? | Yes, each child's `PostStop` ran to the end after its latch was released |
| When does `P` get `Terminated(A)`? | After the last child's `PostStop` ended, not before (not present while `c2` was blocked) |

Control W1b (`withStopChildren(false)`, one child, same `context.stop(A)` after the failure, so there is no wait): `A.signal PreRestart`, `A.setup#2` (children = `c1`), `P ctx.stop(A)`, `c1.PostStop begin/end`, `A.signal PostStop`, `P.signal Terminated(A)`.

### W2. Another route to a stop during the wait

`A` has one slow child. A behavior cannot return `Behaviors.stopped` during the wait (its handler is not called: C3, C5), so the reachable routes are a stop that does not pass through the behavior.

| Run | Action during the wait | `Setup` runs | `A.signal PostStop` | Other observed |
|---|---|---|---|---|
| W2a | classic `PoisonPill` sent to `A` | 1 | no | after the latch is released the child's `PostStop` ends, then `P.signal Terminated(A)`; no dead letter |
| W2b | `ActorSystem.terminate()` | 1 | no | `whenTerminated` was not completed 600 ms after `terminate()` while the child was blocked, and completed after the latch was released |

### W3. Messages and signals that arrive during the wait, then the stop

`A` watches an actor `X` and has one slow child. `m1`, `m2` are sent during the wait and `X` is stopped (so `A` is due a `Terminated(X)`).

| Run | Then | Events after the failure | Counters |
|---|---|---|---|
| W3a | `context.stop(A)`, latch released, `after-stop` sent | `PreRestart`, child `PostStop begin`, `P ctx.stop(A)`, child `PostStop end`, `P.signal Terminated(A)`, `DeadLetter(after-stop)` | `setups=1`, `deadLetters=1` (only `after-stop`), `dropped=0`; `m1`, `m2` and `Terminated(X)` reached no behavior |
| W3b (control) | latch released, no stop | `PreRestart`, child `PostStop begin/end`, `A.setup#2` (children = none), `A.msg m1`, `A.msg m2`, `A.signal Terminated(X)` | `setups=2`; delivered in arrival order, `Terminated(X)` last |
| W3c | `m1`, `StopNow` (handler returns `Behaviors.stopped`), `m-behind-1`, `m-behind-2` sent during the wait; latch released; `after-stop` sent | `A.setup#2`, `A.msg m1`, `A.msg StopNow`, `A.stopped` (the `postStop` callback of `stopped`), `DeadLetter(m-behind-1)`, `DeadLetter(m-behind-2)`, `P.signal Terminated(A)`, `DeadLetter(after-stop)` | `setups=2`, `deadLetters=3`; the messages retained behind the one that stopped the actor were not given to a behavior and were published as dead letters; no `A.signal PostStop` (a stopped behavior does not receive it) |
| W3d | 1500 messages `flood-1..1500` sent during the wait, default capacity; latch released | `Dropped(Msg(flood-1001), "Stash is full in [RestartSupervisor]", A)` ... 500 times (first two shown by the harness) | `dropped=500` before the latch was released; after it `A.setup#2` and `floodDelivered=1000` (`flood-1..1000`); `deadLetters=0` |
| W3e | the same 1500 messages, then `context.stop(A)`, latch released | the same 500 `Dropped`, then one more `Dropped(PostStop, "Stash is full in [RestartSupervisor]", A)` after the child's `PostStop end`, then `P.signal Terminated(A)` | `dropped=501`, `setups=1`, `floodDelivered=0`, `deadLetters=0` |

### W4. A child that never terminates

`A` has one child whose `PostStop` blocks (the latch is released only at the end). A message `m-during-wait` is sent, then the harness observes for 15 s.

Observed: at 5 s, 10 s and 15 s `setups=1`; no timeout, no log line, no failure and no event of any kind during the 15 s. After the latch was released: `child.PostStop end`, `A.setup#2` (children = none), `A.msg m-during-wait`. The check of `reference.conf` is C11.

### W5. Children beyond SupervisionSpec's cases

| Run | Setting | Observed (3 of 3 runs) |
|---|---|---|
| W5a | three children, `restart`; latches released in the order `c2`, `c3`, `c1` | the three `PostStop begin` are logged together after `PreRestart`; `A.setup#2` (children = none) appears only after `c1.PostStop end`, the last one |
| W5b | `A` stops `c1` itself (`ctx.stop(c1)`, `c1`'s `PostStop` blocks), and then fails with `c1` still stopping and `c2` alive | in the failing turn `ctx.children = {c1, c2}` (the stopping child is still a child); `c2.PostStop begin`; `A.setup#2` appears only after both `c1` and `c2` ended |
| W5c | a Typed child blocked inside a message that, once released, throws; `A` had `watch`ed it; `A` fails and the restart stops the child while it is blocked | after the release: `child throws`, `child.PostStop begin/end`, `A.setup#2` (children = none). `A` got no `A.signal` of any kind for the child (the `ChildFailed`/`Terminated` was consumed by the wait); `P` got nothing for the child; `setups=2` |
| W5c2 | a Typed child whose `PostStop` throws after the latch | `child.PostStop end`, then `A.setup#2` (children = none); the restart is not affected |
| W5g | a Classic child (`ctx.actorOf`) blocked inside a message that, once released, throws; the restart stops it while it is blocked | after the release: `classic.throws`, `classic.postStop`, one `DeadLetter(Recreate(RuntimeException: classic-fail))` addressed to the child, `A.setup#2` (children = none), `A.msg after` delivered. The child was not restarted (no `preRestart`, no second `preStart`) |
| W5h | a Classic child that terminates normally during the wait (control of W5g) | `classic.postStop`, `A.setup#2` (children = none), `deadLetters=0` |
| W5d | `withStopChildren(false)`; `A` has `c1` watched with `watch` and `c2` with `watchWith(Msg(watchWith-fired-c2))`; `A` fails; then `c1` and `c2` are stopped from outside | `A.signal PreRestart`, `A.setup#2` (children = `c1`, `c2`, no wait); after the children stopped: `A.msg watchWith-fired-c2` and `A.signal Terminated(c1)`, both delivered to the new incarnation |
| W5e | `withStopChildren(false).withLimit(3, 10 s)`, matcher `Exception`, `Setup` spawns a child named `same` on every run; `A` fails once | `Setup` ran 4 times in all (`A.setup#1..#4`); in `A.setup#2` to `#4` `ctx.children = {same}` at the start of the `Setup` (the first incarnation's child was kept); after the fourth `Setup` `P` got `ChildFailed(A, InvalidActorNameException)` and `A` was stopped; the child `same` ended with `A` |
| W5f | control of W5e with the default `stopChildren = true` | `PreRestart`, child `same` `PostStop begin/end`, `A.setup#2` (children = none), `child same` started again; the actor kept running |

## 3. Summary

| Scenario | One line |
|---|---|
| W1 | A stop (`context.stop`) during the child wait: no new `Setup`, no `PostStop` to `A`'s behavior, retained message not delivered and not a dead letter; the children finish and then `Terminated(A)` reaches the watcher |
| W2 | `PoisonPill` and `ActorSystem.terminate()` during the wait give the same result as W1; `Behaviors.stopped` cannot be returned during the wait |
| W3 | Retained messages and signals are discarded with no event when the actor is stopped during the wait; when a retained message stops the actor during the replay, the messages behind it become dead letters; the stash holds 1000 and publishes `Dropped` beyond it (and `Dropped(PostStop)` if the stash is full at the stop) |
| W4 | No timeout and no event: `A` waits for the child indefinitely; the restart completes when the child finally ends |
| W5 | Several children, a child already stopping, a child failing while stopped, and a throwing `PostStop` all end in the same `Setup` after the last `Terminated`; with `stopChildren = false`, `watch` and `watchWith` of the kept children survive the restart and a fixed child name collides in the next `Setup` |

## 4. Limits of this evidence

- No scenario with `restartWithBackoff` was run; the wait is the same code (`S:224-231` for the scheduled restart) but it is not covered here.
- `Behaviors.stopped` as a way to stop during the wait is not reachable (W2); W3c covers the stop that a retained message causes during the replay.
- W5e used a Typed parent; a Classic parent with a deciding strategy is the subject of the [S-2 oracle](s-2-supervision-2026-11-05.md) (P4b).
- A `Kill` during the wait was not run.
- The harness is not committed (see the header).
