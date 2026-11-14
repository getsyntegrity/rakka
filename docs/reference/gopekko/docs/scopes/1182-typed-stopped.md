# Scope: Typed `Stopped` and `Stopped(postStop)` (#1182, TYP-48)

Status: **approved at G2 by the Owner, 2026-11-10** ([comment on #1524](https://github.com/getsyntegrity/gopekko/pull/1524#issuecomment-6782903754)). Written on 2026-11-10 against `develop` `8e334700`. It is the first scope of the Typed lane of the stage of 2026-11-10 (#927 `unstashAll` follows, scope `927-typed-unstash.md`). **Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: the unit ports the `StoppedBehavior` branch of the interpreter and the stop of the adapter, so the Dev runs the migration kit (`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK under `migration/stage-1/typed-1182-stopped/` (name INFERRED from the neighbours). The path of the Pekko sources below is `upstream/pekko/actor-typed/src/main/scala/org/apache/pekko/actor/typed/` (`BEH` = `Behavior.scala`, `BI` = `internal/BehaviorImpl.scala`, `AA` = `internal/adapter/ActorAdapter.scala`, `BHS` = `scaladsl/Behaviors.scala`, `CTX` = `scaladsl/ActorContext.scala`); specs under `upstream/pekko/actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/` (`StopSpec.scala`).

## Why #1182 is a real blocker of #927 (verified on `develop` `8e334700`)

- `pekko-actor-typed/typed` has **no `Stopped`**: `behavior.go` lists `Setup`, `Receive`, `Same`, `Unhandled`, `Empty`, `Ignore`, `Supervise` and nothing that stops the actor from a result (`behavior.go:15` still says "Stopped ... belong to sibling stories"). `rg -i "stopped\(|StoppedBehavior" --glob '*.go' pekko-actor-typed` finds only test helpers named `swStopped` and similar, no constructor.
- #927's replay loop is defined by `Behavior.isAlive(b2)` (`StashBufferImpl.scala:195, 218, 238`); `isAlive` is false for `StoppedBehavior` and `FailedBehavior` only (`BEH:248-249`). A Go behavior can never be `Failed` (a failure is an `error`, ADR 0035), so **the interruption of the replay and the dead letters behind it cannot be written or tested without `Stopped`**. Two criteria of #927 and all of #1169 depend on it.
- #1182 itself carries two more criteria from the Owner (Updates of 2026-11-06 on the issue): the replay after a stop of the restart wait (oracle W3c, `docs/parity/oracles/typed-restart-child-wait-2026-11-06.md:66`) and the `Stop` variants of `TestNested_InnermostMatchWins` (#1184).
- Its own blocker #847 is closed; #1181 (`Empty`/`Ignore`) is closed; #978 (interceptor) is open and only blocks one StopSpec case, which stays a PORT-GAP.
- **#1169 is not a blocker of #927**: it is the dead-letter publication of the same replay, which only exists inside `unstash`. It is delivered by slice C of #927 (see that scope).

## What this delivers

A behavior that returns `Stopped()` (or `StoppedWith(postStop func())`) ends the actor through the hierarchy that exists today (`Context.StopSelf`, children first, then `PostStop`): the behavior that was current sees the `PostStop` signal first and then the optional callback runs exactly once (`AA:173-176, 342-362`; `BHS:66-90`). It also holds when the behavior stops from `Setup`, as the initial behavior, and (when #978 exists) under an interceptor. Slice B makes the inputs that a restart held behind a stopping input dead letters instead of dropping them.

## Proposed Go shape (decisions for G2; each with the Pekko line it follows)

- **D-1 (approved). Two constructors, no sentinel value.** `Stopped[T any]() Behavior[T]` and `StoppedWith[T any](postStop func()) Behavior[T]`. Pekko: `def stopped[T]: Behavior[T] = BehaviorImpl.stopped` (`BHS:76`, body read) and `def stopped[T](postStop: () => Unit): Behavior[T] = BehaviorImpl.stopped(postStop)` (`BHS:90`, body read), which build `StoppedBehavior` with `OptionVal.None` or `Some(callback)` (`BI:62-65, 99, 105-113`). Go has no overloading, so the callback form has its own name (same reason as `ReceiveMessage`). **A `nil` callback is `Stopped()`** (Owner, 2026-11-10); Pekko's `Some(null)` would fail when the callback is invoked (`BI:108-111`), so this is a conscious difference, recorded in the README and with `TODO(TYP-48-nil-callback)` as the greppable marker.
- **D-2. The callback takes no context.** Pekko's `postStop: () => Unit` (`BI:64`) receives none; the internal `onPostStop(ctx)` (`BI:108-113`) is only the call. Same for Go.
- **D-3. A new sealed variant `stoppedB[T]{postStop func()}`**, in `behavior.go`; `turn` (`interpreter.go:244`) gets a case that records the stop and calls the interpreter's `stop` (`ctx.StopSelf()`), as `next` does (`AA:173-176`: `behavior = ComposedStoppingBehavior(behavior, stopped); context.stop(self)`). The previous behavior stays installed so that `PostStop` reaches it (`AA:344-362`: signal to `lastBehavior` unless deferred, then the callback). `interpreter.PostStop` (`interpreter.go:481`) runs the previous behavior's signal handler first and then the callback, once.
- **D-4. Initial and setup stop.** `startInto` accepts `stoppedB` and returns a marker that `PreStart` turns into a stop (`AA:299-305`: `preStart` at 299, `validateAsInitial` at 302, `if (!Behavior.isAlive(behavior)) context.stop(self)` at 305; the restart does the same at 320-321; `validateAsInitial` allows a stopped start, `BEH:236-244`). A `Setup` returning `Stopped` is the same. `PostStop` then reaches no signal handler (there was no previous behavior) and only the callback runs, once (StopSpec `:33, :95`).
- **D-5. `Stopped` as a result inside `Supervise` and `Resume` layers** follows the same path: it is not a failure, so no layer decides (Pekko's supervision intercepts exceptions, not results). No change to the layer code.
- **D-6. Messages arriving after the stop** are the runtime's dead letters (Classic), unchanged.
- **D-7 (approved). Ordering rule of the stage (Owner, 2026-11-10): the first of this slice B and slice 1 of #821 to arrive after the merge of #871 introduces `Context.System()` with the only EXP-1 record; the other rebases and only uses it (if #821 slice 1 arrived first, slice B adds no export and only uses it). `pekko-actor` gains one export, `func (x *Context[T]) System() *System`** (Owner, 2026-11-10, replacing the proposed `Context.DeadLetters()`), the analogue of `context.system` (typed: `CTX:76`, `implicit def system: ActorSystem[Nothing]`, body read; Classic: `ActorCell.scala:187, 261`). Consumers: this scope (`system.deadLetters`, `StashBufferImpl.scala:248-251`) and #821 (`context.system.scheduler`). Today the `typed` package reaches no `System` because `actor.Context` has no accessor (`pekko-actor/actor/actor.go:50-75`). EXP-1 justification goes in the doc comment, in the style of `behavior.go:88-98` (Symbol, Consumer, why the public API is not enough, what it exposes). **Analysis the Owner asked for (what a `*System` opens from inside an actor):**

  | Method of `System` today | Pekko via `context.system` | Verdict |
  |---|---|---|
  | `DeadLetters()`, `SubscribeDeadLetters`, `Name()`, `Address()`, `StartTime()`, `Uptime()` | `deadLetters`, `name`, `address`, `startTime`, `uptime` (`ActorSystem.scala` typed `:50, 73, 78, 172, 217`; classic `ActorSystem.scala:537, 599`) | allowed |
  | `Terminate()`, `WhenTerminated()` | `terminate()`, `whenTerminated` (typed `ActorSystem.scala:126, 136`; classic `:681, 711`) | allowed; Go's `Terminate` is documented valid from a handler (`system_shutdown.go:27`) |
  | `Shutdown(ctx)` (blocking) | none; the Pekko analogue is blocking on `whenTerminated`, which Pekko permits but discourages | allowed; from inside a handler it can only end by `ctx` (`system_shutdown.go:50-51`), documented; no deadlock beyond the caller's own `ctx` |
  | `Kill(h)`, `PoisonPill(h)`, `Stop(h)`, `Done(h)` | typed `ActorSystem` has none; Classic `context.system.stop(actor)` exists for top-level actors (`ActorSystem.scala:939`); `Context` already has the Go equivalents `Context.Kill/PoisonPill/Stop` (`kill.go:50`, `poison_pill.go:55`, `stop.go:31`) | **not an operation that Pekko forbids from an actor (Classic allows `system.stop`) and `Context` already exposes the same power**; noted so the Owner can veto |
  | `Spawn`-family and `Subscribe`/`Publish` are functions of the package, not methods | `actorOf`/`eventStream` | unchanged |

  Nothing in this list is an operation that Pekko denies to an actor, so the scope goes on; the `typed.Context` wrapper does **not** re-export `System()` in this unit (only the `typed` package's own code uses it); a public `typed.Context.System()` is a later story with its own EXP-1 justification. If the Owner reads the `Kill/PoisonPill/Stop/Done` rows as a stop condition, slice B waits for a narrower accessor, which is a one-line change in D-7.
- **D-8 (approved; contract is the oracle). Held inputs behind a stopping input.** `interpreter.stop` today does `i.held = nil` (`interpreter.go:235-238`). After D-3 a held input is published as a dead letter, in order, once (oracle W3c: `deadLetters=3`; `StashBufferImpl.scala:246-251`; `Supervision.scala:210, 375, 407`). **The sender and recipient of that dead letter, and whether a held `Terminated` is published and as what, are not decided by this text: they are the observed values of the Pekko oracle that another agent runs in its own PR (`docs/parity/oracles/`), and they are the contract.** No inference remains in this scope.

## Criteria → scenario → source → mutation

| # | Criterion | Scenario (test) | Pekko source | Detecting mutation | Slice |
|---|---|---|---|---|---|
| R1 | A stop through the hierarchy: children terminate before the actor's `PostStop` | `TestStopped_ChildrenTerminateBeforePostStop` | `AA:173-176`; ADR 0014; Classic rule (alternative evidence: Go scenario on the shared hierarchy, ADR 0033 D13, no Typed oracle) | the interpreter calls `PostStop` without waiting (skip `StopSelf`, close the cell directly) | A |
| R2 | `PostStop` signal before the callback | `TestStopped_SignalThenCallback` (port of `StopSpec.scala:55`) | `AA:354-361` | run the callback first | A |
| R3a | callback after setup | port of `StopSpec.scala:33` | `AA:299-305` | no callback on a stopped start | A |
| R3b | callback from receive | port of `StopSpec.scala:43` | `AA:173-176` | callback never run | A |
| R3c | callback from the initial behavior | port of `StopSpec.scala:95` | `BEH:236-244`, `AA:299-305` | `startInto` rejects `stoppedB` | A |
| R3d | callback once (two stop paths) | `TestStopped_CallbackRunsOnce` (stop plus `PostStop` re-entry: `Stopped` returned, then `StopSelf` too) | `AA:354-361` | callback run by `turn` and by `PostStop` | A |
| R3e | wrapped by an interceptor | `StopSpec.scala:76` | `PORT-GAP(TYP-48-interceptor-978)` (#978) | | none |
| R4 | `Stopped()` without callback signals only `PostStop` | `TestStopped_NoCallback` | `BI:108-113` | `Stopped()` runs a callback | A |
| R5 | `Stopped` is not a failure: no layer decides, the parent sees a plain `Terminated` | `TestStopped_InsideSupervise` (Restart and Resume layers do not restart) | `AA:173-176` (not an exception) | `turn` returns `failed` for `stoppedB` | A |
| R6 | the `Stop` variants of `TestNested_InnermostMatchWins` (#1184 AC-R1-1, AC-R2-1) | **out of scope**: they need the `Stop` strategy (#1063), not `Stopped` | `Supervision.scala` | `PORT-GAP(TYP-48-stop-variants-1063)` | none |
| R7 | held inputs behind the stopping input are dead letters, in order, once; the message sent after the stop is a dead letter too | `TestRestartWait_StoppedDuringReplay_HeldAreDeadLetters` (oracle W3c: `setups=2`, `deadLetters=3`) | `StashBufferImpl.scala:195, 221, 246-251`; oracle `typed-restart-child-wait-2026-11-06.md:66` | `stop()` drops `held` again | B |
| R8 | the dead letter's sender and recipient (and a retained `Terminated`) are the values the Pekko oracle observed | `TestDeadLetters_SenderAndRecipient`, asserting the oracle's table | the oracle PR in `docs/parity/oracles/` (another agent); `StashBufferImpl.scala:249-251` | sender or recipient differ from the oracle | B |

All tests use go-specs, channels or gates (no sleeps); a mutation is applied in a disposable worktree and never committed.

## Slices (one PR each, at most 400 authored lines, sequential)

| Slice | Content | Files | Estimate (authored) |
|---|---|---|---|
| **A. `Stopped`** | D-1 to D-6, R1-R5 (R6 is out of scope), README section, kit manifest | `pekko-actor-typed/typed/behavior.go`, `interpreter.go`, `stopped_test.go` (new), `example_test.go`, `README.md`; `migration/stage-1/typed-1182-stopped/*` | production ~70, tests ~200, docs ~60: **~330** |
| **B. Held inputs are dead letters** (starts after the merge of #871 and of the oracle PR) | D-7, D-8, R7, R8 | `pekko-actor/actor/actor.go` (the export `Context.System`, ~12 lines) and `actor_test.go` or a new `context_deadletters_test.go`; `pekko-actor-typed/typed/interpreter.go`, `restart_stopped_test.go` (new); `pekko-actor/README.md`; `migration/cost-log.tsv` | production ~40, tests ~150, docs ~30: **~220** |

B touches `pekko-actor/`: the Dev checks `judge/mutate_go.py` for anchors in `actor.go` (adding a method must not shift or break one; update it in the same PR with the same contract), runs `scripts/check-judge-matrix.py` (QA does too; exit 1 is REJECT), the full gates (gofmt, vet, `go test -race`, 85 % per module, diff coverage 85 %, gocyclo ≤ 10), and after the merge the full Go campaign on this Mac (`--runs 30`, `-race`) on a clean `develop`, with the inventory note that V2/V3 have no validity in the meantime. `pekko-actor/README.md` can also be touched by #871 and #821: merge in order and rebase. Slice A touches no file of `pekko-actor/`.

Independence from the other lanes: slice A and B edit `behavior.go`, `interpreter.go` (typed) and, in B, `actor.go` and `deadletters_ref`-adjacent files of `pekko-actor`. Lane 1 (#871) edits `eventstream.go`/`deadletter.go` of `pekko-actor` and lane 3 (#821) adds scheduler files; **B and #871 share the package `actor` (not a file except `README.md`)**. **B and #821 slice 1 share `pekko-actor/actor/actor.go`, where `Context` lives and where `Context.System()` is added (D-7)**: they are sequenced, not parallel: the first to arrive after #871 adds the export, the other rebases and shows that its own delta is identical to what it had before the rebase (the export is not duplicated). Confirmed by `git grep` before the code starts (the Dev repeats it).

## ADR impact and contradictions (ADR 0033 D5)

- ADR 0033 D1/D2 (Typed on the `actor` runtime, no Pekko actor-system copy): no contradiction.
- ADR 0035 (supervision): `Stopped` is a result, not a failure; no amendment. The ADR 0010 / event stream text is not touched.
- ADR 0011 EXP-1: one export (`Context.System()`, slice B or #821 slice 1, whichever arrives first), justified in D-7; `Stopped` and `StoppedWith` justified in slice A in the style of `behavior.go:88-98` (Symbol, Consumer, why not the public API, what it exposes).
- `docs/parity` inventory: row of ACTINV-104 updated in slice A; V levels per the cumulative rule.

## Decisions of the Owner (G2, 2026-11-10)

1. `Stopped` / `StoppedWith`, `nil` callback = `Stopped()`: approved (D-1).
2. Export `Context.System()`, not a narrower dead-letters accessor: approved; analysis in D-7 (no operation forbidden by Pekko was found).
3. Ordering of `Context.System()` with #821 slice 1: see D-7. Sender, recipient and the retained `Terminated`: resolved by a Pekko oracle in its own PR; **slice B waits for the merge of #871 and of that oracle PR**, and the oracle's observed values are the contract. Not an open decision.
4. The `Stop` variants of #1184: **out of scope**; `PORT-GAP(TYP-48-stop-variants-1063)` tied to #1063 (greppable marker in the test file and the README).
5. Sequencing #1182 A, then #1182 B (after #871 and the oracle PR), then #927: approved.

What waits for the Owner: nothing in this scope.


## Not carried here

`empty`/`ignore` (#1181, done); the interceptor case (#978, PORT-GAP R3e); supervision-driven stops and failed behaviors (#956, #1063); messages after the stop (Classic dead letters).

## Definition of Done

Every criterion above has its test (or its PORT-GAP row); mutations reproduced by QA (unmutated green, mutated red); the README separates what exists from #927, #978, #1063; kit manifest and `cost-log.tsv` row per kit run; QA independent on the final head; for B the judge matrix, the gates and the campaign.
