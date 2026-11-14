# PROPOSAL: release the fields of a terminated cell (ADR 0024 D5, task T6; issue #563)

**Verdict.** Release eight fields of `cell[T]` (`actor`, `factory`, `failedMsg`, `pendingCause`, `startFailure`, `resumeFn` with `resumeReady`, `failed`, `sysq`) in one critical section of `finalize`, after the watchers are told and before `Done` closes. Keep every other field: identity, the hierarchy node, the context and the counters are read by operations that stay valid after termination. No contract of ADR 0024 changes, so no contract decision is asked of the Owner; only the authorization to run the unit (section 6).

Status: **PROPOSED. Documentation and launch artifacts only.** No production code, no functional test, no agent and no kit run was touched or made by this unit. **O0 (authorization and budget) is pending: nothing here is approved.** T6 is not implemented and #563 stays open. The first version of this package was PR #738 at `b916d0f`; this revision completes it (section 11).

| Item | Value |
|---|---|
| Base | `origin/develop` `f23a215f9701dd7b5046dada0150ba75dfbe61c1` (2026-10-23, #737), fetched for this unit |
| Contract | ADR 0024 D5 (accepted 2026-10-20), `docs/adr/0024-actor-registry-retention-after-termination.md`; T2-T5 integrated by #543 `a081e479f7cf26617f87611dad9554a07fb227b3` |
| Pekko pin | `upstream/pekko` `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1); the gitlink at the base matches it |
| Kit pin | `migration-kit` `cf91c9d5068d9aaf95a36164169f08c3e636c909` (not executed here) |
| Duplicates | Observed 2026-10-23T20:37Z, after #738 existed: the only branch for T6 on `origin` is this PR's `docs/t6-cell-field-release-563`; the only open PR is #738; the merged #542 and #543 are the T1-T5 work; #563 is open, has 0 comments, last updated 2026-10-21. A later change is not covered by this line |

Evidence tags: **[P]** read in the pinned Pekko tree; **[G]** read in goPekko at the base; **[I]** inference from reading, not run. Nothing below was executed. Paths of Pekko are relative to `upstream/pekko`: `A` = `actor/src/main/scala/org/apache/pekko/actor/`, `T` = `actor-tests/src/test/scala/org/apache/pekko/actor/`.

## 1. Source revalidated (Pekko pin)

Both references of #563 hold exactly: `A ActorCell.scala:718-726` and `A dungeon/FaultHandling.scala:247-276`.

| Symbol | Where [P] | What it does |
|---|---|---|
| `clearActorFields(actorInstance, recreate)` | `A ActorCell.scala:716-721` | `currentMessage = null`, `behaviorStack = emptyBehaviorStack`; both parameters unused |
| `clearFieldsForTermination()` | `A ActorCell.scala:722-726` | `unstashAll()`, `_props = ActorCell.terminatedProps`, `_actor = null` |
| `ActorCell.terminatedProps` | `A ActorCell.scala:388` | a `Props` whose creator throws "This Actor has been terminated" |
| `finishTerminate()` | `A dungeon/FaultHandling.scala:247-276` | `aroundPostStop`, `stopFunctionRefs`, `dispatcher.detach`, `DeathWatchNotification` to the parent, `tellWatchersWeDied`, `unwatchWatchedActors`, then the two clears (`:273-274`). The comment at `:249-252` says the order is crucial |
| other callers of `clearActorFields` | `FaultHandling.scala:110` (restart), `:300` (failed recreate), `ActorCell.scala:645` (failed create) | the restart path clears the same two fields; T6 does not touch it |
| `detach` and `unregister` | `A.../dispatch/AbstractDispatcher.scala:170`, `:240-246` | the mailbox is swapped for the dead-letter mailbox, closed and cleaned up |
| `tellWatchersWeDied`, `unwatchWatchedActors`, `stopFunctionRefs` | `DeathWatch.scala:133,165`, `Children.scala:116` | the steps named above |

Upstream tests that touch these fields (the only ones found: `grep` of `actor-tests` for `terminatedProps`, `clearFieldsForTermination`, `clearActorFields`):

| Test [P] | Relevance |
|---|---|
| `T LocalActorRefProviderSpec.scala:109-129` "not retain its original Props when terminated" | the direct counterpart of releasing `factory`. Its comment (`:122-123`) says the fields are cleared after the `Terminated` message is sent, so it polls |
| `T ActorLifeCycleSpec.scala:193-229` "have a non null context after termination" | the comment at `:210-213` says the context used to be nulled and "this isn't done any more". This is the upstream reason `ctx` and `self` stay |
| `T ActorLifeCycleSpec.scala:142-163` "clear the behavior stack upon restart" | the restart use of `clearActorFields` |
| `T DeathWatchSpec.scala:145-152` | watching a dead actor delivers `Terminated` (already in ADR 0024) |

No upstream test asserts `_actor == null` or `currentMessage == null` directly. The comparison with Pekko is by reading: *coincidencia observada por lectura; validación comparativa pendiente*.

| Class | Items |
|---|---|
| **Port** (Pekko-derived) | release of `actor` (`_actor = null`), `factory` (`_props = terminatedProps`) and `failedMsg` (`currentMessage = null`); the position after the watchers are told (`:266` before `:273`) |
| **Go adaptation** (no Pekko source) | release of `pendingCause`, `startFailure`, `resumeFn`/`resumeReady`, `failed`, `sysq`: Go-only supervision and restart state. Pekko keeps its system messages in the mailbox, not in cell fields, and `detach` (`AbstractDispatcher.scala:170-172`) goes through `unregister`, which calls `cleanUp` on it (`:243-245`), which moves the pending system messages and the queue to the dead-letter mailbox (`dispatch/Mailbox.scala:340-354`) [P]. The lock is `c.mu` |
| **Kept on purpose** | `ctx`, `self` (upstream precedent above); `behaviorStack` and stash have no counterpart in `cell` (grep: `stash` appears only in comments) |
| **Pending (not in this unit)** | releasing `owner` (L2 below, a possible later unit); anything about heap |

## 2. Retention inventory [G], at the base

Every field of `cell[T]` (`pekko-actor/actor/cell.go:23-87`). "Readers" include late callbacks, handles and concurrent paths; all line numbers are in `pekko-actor/actor/`.

### 2.1 Release set (eight fields)

| Field | Retains | Writers | Readers | Needed until | Sync today |
|---|---|---|---|---|---|
| `actor` (`cell.go:30`) | the user instance and whatever it captured | `startActor` `cell.go:265-267`, `installNew` `supervision_restart.go:424-427` | `Receive` `cell.go:310`; `PreStart` `:277`; `ReceivePort` `port.go:107`; `PostStop` `stop.go:100`; strategy `supervision_failure.go:309`; hooks `supervision_restart.go:320,358,433,436`; `OnTerminated` `watch.go:455` | PostStop returns (`stop.go:100-102`) | writes under `c.mu`; reads in the activation |
| `factory` (`cell.go:72`) | the closure given to `Spawn` (deps captured by it) | `commitCell` `spawn.go:127`, once | `startActor` `cell.go:250`; `installNew` `supervision_restart.go:409,415` | the last start step, which is `startActor` inside `terminate` (`stop.go:80`) | none needed: set once |
| `failedMsg` (`cell.go:82`) | the last failed message (a `T`, a `ChildFailure` or a `Terminated`) | the activation, no lock: `cell.go:320,335`; `port.go:122,136`; `start_failure.go:171`; `supervision_failure.go:256`; `supervision_restart.go:283,375,490,531`; `watch.go:484` | `runPreRestart` `supervision_restart.go:318` | the end of the last activation | activation-owned |
| `pendingCause` (`cell.go:80`) | the cause of the restart in progress (an error, possibly a wrapped panic value) | `beginRestart` `supervision_restart.go:273` | `continueRestart` `:307,310` | `continueRestart` returns. It is never cleared today, also on a live cell (section 2.4) | activation-owned |
| `startFailure` (`cell.go:84`) | an `OrdinaryFailure` of a start step | `cell.go:293`; `supervision_restart.go:469` | `cell.go:129-131,186`; `supervision_failure.go:137` | the next drain step reports it. If the stop was accepted first, `terminate` runs the start (`stop.go:80`) and nothing ever consumes it | activation-owned |
| `resumeFn`, `resumeReady` (`cell.go:85-86`) | the continuation closure of a `ResumeWith` | `setResume` `start_failure.go:133-137` (under `c.mu`); callers `cell.go:294,328`, `port.go:130`, `start_failure.go:129` | `takeRuntimeLocked` `start_failure.go:155-159` | `terminate` calls `dropResume` (`stop.go:75`), but `startActor` inside it can set it again (`cell.go:294`), so it can outlive finalization | `c.mu` |
| `failed` (`cell.go:69`) | queued `failedItem{child, cause, detail}`: errors and panic values of children | `enqueueFailed` `supervision_failure.go:110`, which refuses once `stopRequested \|\| closed` (`:106`) | `takeFailedLocked` `:124-131`; `len` in `hasWorkLocked` `:140` and `endActivation` `cell.go:186` | never after the stop is accepted (`drain` goes to `terminate`, `cell.go:118-122`). Items queued before it and not handled stay for the life of the cell | `c.mu` |
| `sysq` (`cell.go:73`) | queued `sysItem{kind, cause}` | `recreate` `supervision_restart.go:97`, which refuses only `closed` (`:93`), so it can append between the stop being accepted and `closed`; the hold path `supervision_failure.go:67` refuses once the stop is accepted (`holdLocked` `supervision_restart.go:76`) | `takeSysLocked` `:57-69` | same as `failed` | `c.mu` |

### 2.2 Kept (cannot be released)

| Field | Late reader that needs it |
|---|---|
| `sys`, `cid` | `Tell` publishes the dead letter with them (`ref.go:51,57`); `ID()` and `owner()` (`ref.go:25,28`); `Done` and `askTarget` (`system.go:163`, `ask_endpoint.go:135-138`) |
| `done`, `origin`, `guardian` | `terminal()` (`ref.go:33-39`), `identity()` (`cell.go:91`), `retirable()`: the state that lets `Done` and `Ask` work after retirement (ADR 0024 M1-M2) |
| `node` | `acceptStop` calls `closeGate(c.node)` before it looks at `closed` (`stop.go:43-45`), so a `Stop` that resolved the record just before retirement still reaches it. A retained `Context` reads it in `Child`, `Children` and `Spawn` (`hierarchy.go:25,413,417`). `adjustCore` writes `c.node.susp` (`supervision_failure.go:79`) |
| `parent` (an `idHandle`, no cell) | `Context.Parent()` (`hierarchy.go:404-409`); `failUpExcept` reads it only after the `stopAccepted` check (`supervision_failure.go:220`) |
| `self`, `ctx` | `Context.Self()` (`actor.go:50`) returns `c.self`; a nil would be a typed-nil `Ref`. `Context` is one pointer, so releasing it saves nothing; Pekko dropped nulling the context (`ActorLifeCycleSpec.scala:210-213`) |
| `mu` and the flags `stopRequested`, `closed`, `started`, `postStopBegun`, `scheduled`, `restarting`, `waitDone`, `initFailed`, `fatal`, `pendingCreate`, `postStopGoexit` | read by `Tell`, `Watch`, `Stop`, `Post` after termination (`ref.go:48`, `watch.go:203,253`, `port.go:72`). `scheduled` must stay true (section 3.3) |
| counters `told`, `received`, `deadLettered`, `maxDepth`, `capacity`, `pushed`, `watchGen`, `suspended`, `perp`, `name` | a late `Tell` still increments `told` and `deadLettered` (`ref.go:47-50`); `snapshot()` reads them (`cell.go:352-362`) |
| `port` | handed to the instance at `attachPort`; `Post` after termination dead-letters (`port.go:70-75`). A `portRef` is one pointer |

### 2.3 Already released today (no change; pinned by a structural test)

`queue` (`takePendingLocked` `stop.go:105` → `TakePending` leaves a nil buffer, `internal/mailbox/queue.go:81-86`); `portDead` (`stop.go:106-107`); `watchers` (`watch.go:394`); `watching`, `lane`, `termEntry`, `pendingDeaths` (nil since the stop was accepted, `watch.go:384`; every later writer refuses, `watch.go:203,253,331`, `port.go:72`).

### 2.4 Retention that T6 does not remove (limits, [I])

- **L1.** A `Ref` held by the application still holds the cell shell: the counters, `node`, `ctx`, `self`, `port`.
- **L2.** The shell is not alone. `owner` is the parent's `*node` and `node.fin` is a method value bound to the parent's cell (`spawn.go:132`), so a held `Ref` of a terminated child reaches the parent's cell, and through its own `owner` every ancestor's. With T6 the dead ancestors lose their actor and factory but not their shells. Releasing `owner` after `signal` would cut the link (its only readers are in `finalize`, `stop.go:112,119`), but it is **excluded from this unit**: it is a possible later unit, and this one documents the retention as a limit.
- **L3.** `sys` keeps the whole `System` reachable from any held `Ref`; `Tell` needs it to publish dead letters.
- **L4.** `pendingCause` stays on a live cell after a restart completes. It is outside the termination path; not changed here, offered as a follow-up.
- No heap or GC measurement exists. "References released" is a structural statement about fields, not a measured saving.

## 3. Design

### 3.1 The release point

In `finalize` (`stop.go:93-120`), between `notifyWatchers` (`:117`) and `close(c.done)` (`:118`), one call:

```go
// sketch for the implementation unit; not applied by this proposal
func (c *cell[T]) releaseTerminated() {
	c.mu.Lock()
	c.actor, c.factory = nil, nil
	c.failedMsg, c.pendingCause, c.startFailure = FailedMessage{}, nil, nil
	c.resumeFn, c.resumeReady = nil, false
	c.failed, c.sysq = nil, nil
	c.mu.Unlock()
}
```

It takes only `c.mu`, once, and runs no user code and no `Execute`. It reuses nothing from `System.mu`, so the lock order of `hierarchy.go:38-42` is untouched.

### 3.2 Order against each step

| Step | Release is | Why |
|---|---|---|
| `PostStop` (`stop.go:100-102`, `callPostStop` `:143`) | after | `PostStop` reads `actor` and `ctx`. After a Goexit in `PostStop` the next frame skips it (`beginPostStop` `:125-133`) and reaches the release once |
| finalization of children | after | the parent's finalize is scheduled only when `len(out)+reserved == 0` (`readyLocked` `hierarchy.go:178-184`), so every descendant is released before the ancestor's `PostStop` |
| `closed`, pending and watchers taken (`stop.go:104-111`) | after | `closed` is the flag that stops every late writer of the release set (`recreate` `supervision_restart.go:93`, `enqueueFailed` `supervision_failure.go:106`, `addWatcher` `watch.go:359`, `Tell` `ref.go:48`). Release before it could be refilled |
| name released, dead letters, port dead letters (`:112-116`) | after | they use the locals `pending`, `portDead`, `c.cid`, `c.sys` |
| `notifyWatchers` (`:117`) | after | Pekko tells the watchers (`:266`) before it clears (`:273-274`). It reads `c.cid` and the snapshot only |
| `close(c.done)` (`:118`) | before | a waiter on `Done` then finds the cell released. A choice, not a contract; see M5 |
| `signal` → `complete` (`:119`, `hierarchy.go:228-243`) | before | the owner counts a cell only once it is released |
| retirement from `System.byID` | before | I1 of ADR 0024 puts retirement after the signal; release stays upstream of it. A registry reader that still finds the record (closed, not retired) reads none of the released fields (section 3.4) |

Like Pekko, a watcher can see `Terminated` before the release (`LocalActorRefProviderSpec.scala:122-123`). The guarantees are: released when the owner counts the cell, and (by reading) when `Done` closes.

### 3.3 Why the activation-owned fields are safe to write [I]

Four fields are written by the activation with no lock. The release writes them under `c.mu`, which does not exclude an activation, so correctness rests on this: **no activation runs after `terminate` returns.**

- `scheduled` is reset only at `cell.go:147,157,180,196`. Each is reached only after the `stopRequested` check in the same critical section (`cell.go:118`; `:179` tests `!c.stopRequested`; `:191` returns first), so once the stop is accepted `scheduled` stays true.
- Every other `Execute(c.drain)` is guarded by `!c.scheduled` under `c.mu` (`ref.go:61-67`, `supervision_restart.go:98-105,113-120`, `watch.go:259-266`, `port.go:78-85`, `supervision_failure.go:111-118`, `stop.go:50-52`) or runs inside an activation (`cell.go:176,188,193`, `handOver` `:239`).
- `terminate` is the last thing a drain does after a stop (`cell.go:119-122`). The finalize is scheduled by `readyLocked` only after `beginTermination` set `terminating` under `System.mu` (`hierarchy.go:196-217`), so the last activation write reaches the finalize through `System.mu` and the executor: a happens-before path.
- The exception class is a `runtime.Goexit` hand-over (`handOver` `cell.go:239`, `callPostStop` `stop.go:143-150`). The dying goroutine starts the next frame while it unwinds, and nothing deferred above touches cell state (comments `cell.go:235-239`, `stop.go:139-142`).

This is a reading. The unit pins it with a `-race` test on a real executor and states that such a test detects, it does not prove.

### 3.4 Late callers against the release [G]

| Operation after or during termination | Reads from the release set? |
|---|---|
| `Tell`, `Ask`, `Port.Post` | no: `stopRequested \|\| closed` first, then `sys`, `cid` |
| `Done` | no: `done`, `origin`, `owner()` |
| `Watch`, `WatchWith`, `Unwatch` on it | no: `addWatcher` is false once closed (`watch.go:359`) → one `Terminated` at once |
| `Stop`, repeated or concurrent | no: `closeGate(c.node)`, then the early return on `stopRequested \|\| closed` |
| `ID()`, equality, name reuse | no |
| retained `Context`: `Self`, `Parent`, `Child`, `Children`, `Spawn`, `Watch`, `Stop` | no: `self`, `parent`, `node`, `sys` |
| `recreate`, `enqueueFailed`, `suspend`, `resume`, `notifyTerminated` from a parent or a watcher | no append after `closed`; `suspend` reads counters and `node` only; `wake` needs `!scheduled`, which is false |
| `failUp` from a closed cell | no: `stopAccepted` returns false first (`supervision_failure.go:220`) |
| `handleFailed` of a parent for a child that finished | no: `childByID` on the parent's own node (`:251`) |
| guardians | same release at shutdown; their entries stay (D4); `WhenTerminated` reads `done` |

### 3.5 Races and interleavings

- **Activation / restart.** A stop accepted during a restart wait is taken by `terminate` before any system step (`cell.go:118` precedes `takeSysLocked`). `wakeRestart` on a closed cell schedules nothing (`!c.closed`, `supervision_restart.go:113`); `forgetChildLocked` hands out no continuation while `terminating` (`hierarchy.go:260`).
- **Supervision.** A child failure queued before the stop is dropped with `failed`; a failure arriving after it is refused (`:106`). A late `Recreate` is refused after `closed` and, between the stop and `closed`, is cleared by the release.
- **Late sends.** `Tell` decides under `c.mu` on `stopRequested \|\| closed`; the release changes none of those fields.
- **Observing completion.** `Done` closes after the release; the owner is signalled after it; retirement follows the signal.
- **Cannot be released.** Section 2.2, with the reason per row. Where none of the Pekko behavior applies, nothing is claimed.

## 4. Alternatives

| | Idea | Verdict |
|---|---|---|
| A | release inside the critical section that sets `closed` (no window where the cell is closed but unreleased) | **Fallback.** Safe, and the window disappears. It moves the clear before the dead letters and the watchers, against the order that ADR 0024 records ("then fields cleared") and Pekko `:266` → `:273`; it would need the Owner to accept that declared deviation |
| B | release after `close(done)` | rejected: `Done` would not imply release |
| C | release after `signal` (end of finalize) | rejected: the owner would count the cell before it is released, and the release would race with retirement |
| D | release in `acceptStop` or `terminate` | rejected: `PostStop` needs `actor`; with a nil one the type assertion at `stop.go:100` fails silently and `PostStop` would not run (mutant M2) |
| E | also nil `ctx`, `self`, `node`, `sys` | rejected: Pekko stopped nulling the context; `acceptStop` needs `node`; a typed-nil `Self()` |
| F | tombstone, weak pointers, finalizers, GC | rejected by ADR 0024 (alternatives C and E): timing-dependent, and not a proof |
| G | reset the struct (`*c = cell[T]{...}`) | rejected: copies the mutex, loses identity, races every reader |
| H | release only `actor` | rejected: leaves the factory closure and the causes in `failed`, `sysq`, `pendingCause` |
| I | also release `owner` after `signal` (former step S5) | **excluded from this unit.** It touches a different reference chain (the parent's node), needs its own mutants and review, and leaves the T6 core unchanged. Residual retention is documented in 2.4 L2; it is a candidate for a later unit |

## 5. Contract preserved

Tell, Ask, Done, Watch/WatchWith, Stop, identity and guardians keep the results recorded in ADR 0024 ("Who needs the registry today"). The unit reuses `rcObserve` and `rcWant` of `registry_characterization_internal_test.go` as the expected values, so the base is the oracle. The only observable difference is that the user's `Actor` and `factory` stop being reachable from the cell after termination.

## 6. Decisions

| | Decision | Needs the Owner | Recommendation |
|---|---|---|---|
| O0 | Authorize the execution: approve the hashes of `RUNBOOK.md` section 9 and the budget of section 9 below, and say to run (the protocol of `byid-retention`, ADR 0024 "Consequences") | **Yes**, always. **Pending: not granted** | to be decided by the Owner after reading this PR |
| O1 | Whether to add an internal test seam, nil in production, between the watchers being told and `Done` closing (precedent: `commitPoint`, #481 D1, approved case by case) | Only if the Owner wants M5 covered | **Proposed configuration: no new internal test seam.** The tests use the fake `watcherLink` and `completionPoint`. M5 then has no deterministic coverage (section 8). This is a preparation choice, **not** an approval |
| O2 | Whether to run the Go judge or any campaign in this unit | Yes (cost) | **Proposed configuration: no judge and no campaign.** They stay module-close gates (#461). This is a preparation choice, **not** an approval |

Not Owner decisions, with the reason: the eight-field release set (no contract changes; every reader is listed in 2.1 and 3.4); keeping `ctx`, `self` and `node` (upstream precedent and a late reader); the position of the release (aligned with ADR 0024 and Pekko); the exclusion of `owner` (no contract changes either way). If the Owner prefers alternative A, that is a change to the order the ADR records and goes back to them.

## 7. Atomic steps of the implementation unit

| | Step | Kind | Evidence |
|---|---|---|---|
| S0 | O0 given; rebase on `develop`; record the baseline hashes (RUNBOOK 5) | process | hashes |
| S1 | `test(actor)`: characterization of a terminated `Ref`, `Context` and `Port` (T06, T09, T10, T13), named `TestCellReleaseCharacterization...` | test only, no production change | GREEN on the base: copied into a scratch worktree of the base and run with `-run CellReleaseCharacterization` |
| S2 | `feat(actor)`: `releaseTerminated` and its call in `finalize`; field comments; the structural tests T01-T05, T07, T08, named `TestCellRelease...` | production ≈ 30 lines + tests | the structural tests fail on the base (scratch worktree) and pass with S2 |
| S3 | `test(actor)`: the guardian test T11 and the race test T12 | test only | `-race`; T12 is a detector, not a proof of order (M5) |
| S4 | docs by the coordinator: `pekko-actor/README.md` (the retention limit), ADR 0024 status and "Not verified" (the reader proof), `docs/actor-core-parity.md` if it names fields, the execution record | docs | diff |
| S5 | gates (section 9), mutants (section 8), one independent review, draft PR | process | record |

Forecast: about 30 lines of production in `stop.go` and `cell.go`, about 400 of tests, about 60 of docs. That exceeds 400 authored lines, so the PR states the size of each commit, as the previous unit did; each commit stays under 400.

## 8. Validation matrix (deterministic, in-process, go-specs, no GC, no sleeps, no external service)

**Names.** Every new test is `TestCellRelease...`; the ones that must pass on the base without any production change are `TestCellReleaseCharacterization...` (T06, T09, T13 and the restart half of T10). The wrapper selects `CellRelease` (`run-release-tests.sh`), and the pattern is specific on purpose: four existing tests contain the word `Release` (for example `TestRestartAndStopReleaseTheContinuation`) and a bare `Release` would select them.

Fixtures to reuse [G]: `rcNewEnv` (controlled executor `ctlExec` plus recording sink plus dead-letter log), `e.stopped`, `e.holdSignalOf`, `e.signalsOf`, `e.deliver`, `rcObserve`, `rcWant`, `isClosed` (`registry_characterization_internal_test.go:24-155`, `cell_internal_test.go:23,89`), the seam `completionPoint`, and an in-package fake `watcherLink` (`watch.go:68-71`) added to a cell with `addWatcher`: its `notifyTerminated` runs inside `notifyWatchers` (`stop.go:117`), which parks the finalizer after the dead letters and before the release, with no new production seam.

| ID | Test | Assertion |
|---|---|---|
| T01 | classification guard: walk `reflect.TypeOf((*cell[int])(nil)).Elem()` | every field is classified as keep, release or already-released; a new unclassified field fails. After termination the release and already-released sets are zero, the keep set `{sys, cid, node, done, origin, self, ctx, parent}` is not |
| T02 | per field: seed a sentinel in each of the eight fields, stop, hold the signal | at the captured signal each field is zero, `Done` is closed and the entry is still registered |
| T03 | reachable states: Receive failure with a Stop decision; restart, then Stop; stop accepted before the first activation with an ordinary start failure and a continuation; a child failure queued while the parent is suspended, then Stop; a `Recreate` queued between the stop and finalization | the field the state fills is zero after finalization |
| T04 | no premature release: park in the fake watcher | `PostStop` ran once, `closed` is true, `Done` is open, all eight fields are intact; late `Tell` dead-letters, `Stop` is a no-op, `addWatcher` is false |
| T05 | released before the owner counts: `completionPoint` | zero at capture; after `deliver` the entry is gone and the fields are still zero |
| T06 | external references after termination: `rcObserve` on a held `Ref` | equals `rcWant`: `Tell` one dead letter, `Ask` `ErrRecipientTerminated`, `Done` `(closed, nil)`, `Watch`/`WatchWith` one `Terminated`, `Stop` one `stop-accepted`, name reuse a new ID |
| T07 | `PostStop` contract | runs once with a usable instance and context; after a Goexit the second frame does not rerun it and the release happens once |
| T08 | hierarchy: a held child cell read inside the parent's `PostStop` | the child is already released; the parent is not |
| T09 | late internal callers on a released cell: `recreate`, `enqueueFailed`, `suspend`, `resume`, `notifyTerminated`, `postPort`, `failUp` | no panic, `len(sysq) == 0`, `len(failed) == 0`, nothing handed to the executor |
| T10 | restart unchanged: restart twice, then terminate | characterization half: `factory` and `actor` are non-nil between restarts (passes on the base). The "released only at termination" half is covered by T02 and T03 |
| T11 | guardians | after `Shutdown` the three entries are registered (D4), the guardian cells are released, `WhenTerminated` is closed, `Done` of root, user and system is `(closed, nil)` |
| T12 | real executor, `-race`: `Tell`, `Watch`, `Stop`, `Done` concurrent with termination | one `Terminated`, one `stop-accepted`, dead-letter counts, and no race report. It detects unsynchronized access to the release set. It does **not** prove the order of the release against `close(done)`, and a race build does not guarantee that M5 is caught |
| T13 | a retained `Context` and `Port` | `Self` is non-nil; `Parent`, `Child`, `Children` answer; `Spawn` returns `ErrParentStopping`; `Post` is false and dead-letters |

Directed mutants (a scratch copy outside the repository, never committed; each must be killed by a named assertion, a survivor is reported):

| ID | Mutation | Killed by | Assertion |
|---|---|---|---|
| M1.a-h | omit the release of one field (eight mutants) | T02, T01 | that field is zero at the captured signal |
| M2 | release in `acceptStop` or `terminate` | T04, T07 | `PostStop` ran exactly once; fields intact while parked |
| M3 | release before the children finish | T08 | the parent's `PostStop` sees its own fields |
| M4 | release after `signal` | T05 | zero at the captured signal |
| M5 | release after `close(done)` | **none** (no deterministic coverage) | The order against `close(done)` is checked by **code inspection** (the call sits between `notifyWatchers` and `close(c.done)` in `finalize`), by the reviewer and the coordinator. If the mutant is run and survives, it is **recorded as surviving**; if it happens to be caught by T12, that is incidental and not claimed. Not asserted by any test |
| M6 | also release `ctx` or `self` | T13, T01 | `Self()` non-nil; classification |
| M7 | also release `node` | T04, T06, T13 | `Stop`, `Spawn`, `Children` on a held handle do not dereference nil |
| M8 | also release `done`, `origin` or `sys` | T06 | `Done` `(closed, nil)`; `Ask` `ErrRecipientTerminated`, not `ErrInvalidHandle` |
| M9 | reset the counters | **no `CellRelease` test** (amended after the run, see `RUNBOOK.md` section 9: this row said T06, which asserts no counter); detected by the existing suite, `TestPortMessagesStayOutOfTheTMessageCounters` and `TestPreStartHooks` | the counters of those tests |
| M10 | release on restart | T10 | `factory` non-nil after a restart |
| M11 | release without `c.mu` | T12 | a race report. Probabilistic: a miss is reported, not counted as detection |
| M12 | `recreate` stops checking `closed` | T09 | `len(sysq) == 0` after a late `recreate` |
| M13 | `enqueueFailed` stops checking | T09 | `len(failed) == 0` |
| M14 | clear `resumeFn`, keep `resumeReady` | T01, T02 | `resumeReady` false |

**Update note (after integration; the text and the tables above are the historical proposal and are unchanged).** Where this section says that M5 has no deterministic coverage and that M11 is only probabilistic, that was true of the proposal and of the first run. A later test, `TestCellReleaseDoneClosesOnlyAfterTheReleaseTookTheLock` (see `EXECUTION-RECORD.md` section 16), detects M5 and M11 when each is applied alone to a scratch copy, with no new seam (O1 unchanged). It takes `c.mu` from a watcher notification and checks that the release parks on the lock with `Done` still open and nothing released. Limits still in force: the verdict depends on the finalizer goroutine reaching the lock within a 10 s bound and on the goroutine dump naming `releaseTerminated` and a mutex frame; and that `releaseTerminated()` is the statement immediately before `close(c.done)` has no observable point under O1 and rests on inspection of `stop.go`. The 24-mutant campaign was not re-run and no new total is claimed. The test was written by the author; An independent review of the test at HEAD `177ed4c` (PASS_WITH_MINORS, no blocker) is recorded in `EXECUTION-RECORD.md` section 17; the independent run confirmed the detection of M5 and M11 alone and the limits above, and did not verify the approval of O1. Later commits on that branch are documentation or comments only.

## 9. Commands, gates and budget

**This preparation (executed; documentation and launch artifacts only):**

| Check | What it establishes | What it does not |
|---|---|---|
| diff whitespace check (`diff --check` on the staged change) | no whitespace errors | nothing about content |
| JSON parse of `settings.implementer.template.json`, raw and rendered with a sample root | valid JSON; every `__ROOT__` is substituted | that the permissions behave as intended in a live run |
| `bash -n` on `run-agent.sh` and `run-release-tests.sh` | shell syntax | behavior |
| a script that compares the permission template with `RULEBOOK.md` section 5 and with `implementer-prompt.md` | every writable path is a surface and every surface is writable; the prompt, wrapper and launcher name the same paths | that a future agent obeys them |
| a script that lists every `file:line` of the package with the cited line, read by a person for the references this revision touched, then a range check | the references exist; the touched ones were read | semantic correctness of unchanged references beyond the first revision's reading |
| `shasum -a 256` of the hashed files, recomputed after the last edit and compared with `RUNBOOK.md` section 9 and `RESUME.md` | the hashes in the documents are the hashes of the files | what the launcher verifies (`RUNBOOK.md` section 9) |
| `CI_BASE_SHA=origin/develop make ci-fast` | the fast path finds no Go package to build or test. It is **not applicable**, not a functional validation | any behavior of T6 |

Not run, by design: the launcher, any agent, `run-release-tests.sh`, Go gates (no Go file changed), race, coverage, judge, mutants. The functional tests of T6 do not exist yet.

**Implementation unit (after O0).** Policy: `docs/quality-gates.md`, `docs/ci-economy.md`. GitHub Actions runs only the `go` fast check; everything else is local.

| Gate | Command |
|---|---|
| format, vet, tidy | `gofmt -l pekko-actor/actor`; `go vet ./pekko-actor/...`; `(cd pekko-actor && go mod tidy -diff)` |
| lint and complexity ≤ 10 | `golangci-lint` v2.14.0 (`.golangci-version`) `run --config ../.golangci.yml ./...` in `pekko-actor` |
| coverage ≥ 85% per module | `scripts/check_coverage.sh` |
| dependencies, headers | `scripts/check_deps.sh`; `.github/scripts/check-headers.sh` (6 errors exist at the previous base, all in `pekko-persistence`) |
| tests | `go test -count=1 ./pekko-actor/... ./pekko-persistence/...`; `go test -race -count=1` focused on the new tests and on the watch, stop, done, ask, spawn, hierarchy and termination tests; the full `pekko-actor` module under `-race` once |
| mutants | section 8, scratch copy |
| not run | the Go judge, any campaign, stress runs, evidence tags |

**Budget (a proposal for O0; an estimate scaled from `byid-retention/EXECUTION-RECORD.md` section 2, not a measurement).** That unit measured 232,455 tokens, 1.92 USD and 9.1 minutes for one Sonnet implementer on about 800 changed lines across nine production files, and 76,340 tokens, 0.91 USD and 2.2 minutes for one Opus review. This unit has about 30 production lines in one file and about 400 test lines.

| | Cap |
|---|---|
| Implementer, one Sonnet run | 4 USD, 20 minutes |
| Review, one Opus run | 2 USD, 10 minutes |
| Consultation | 0, unless a concrete blocker appears |
| Tokens | evaluation stop 350,000; ceiling 450,000; reserve 90,000 coordination, 30,000 gates, 30,000 mutants, 30,000 per correction round |
| CI | the existing fast check, one run per pushed head; no paid retry |

The review launches only if the implementer used at most 150,000 tokens and `observed + reserve + 60,000 <= 450,000`.

## 10. What this proposal does not claim

T6 is not implemented and #563 is not closed. O0 is pending and none of the proposed configurations (O1, O2, the budget) is an approval. No kit run, agent, judge, campaign or independent review was made. The launch artifacts were checked for consistency, not run. M5 has no deterministic coverage. No heap or GC measurement exists and none is proposed as a proof. Pekko parity is not claimed beyond the cases in section 1, compared by reading. A green fast CI establishes none of it.

## 11. Revision log

| Revision | Change |
|---|---|
| `b916d0f` (PR #738, first version) | analysis, design, contract, runbook and manifest. Launch artifacts missing |
| this revision | (1) `owner` excluded from the unit: former step S5, R5, M15 and M16 removed; the residual retention stays as limit L2 and alternative I. (2) O1 proposed as "no new internal test seam", O2 as "no judge and no campaign"; both are preparation choices, not approvals. (3) M5 declared without deterministic coverage; T12 is no longer presented as evidence of order. (4) Duplicates observation re-dated. (5) Launch artifacts added. (6) **Correction:** the first runbook pointed at `byid-retention/run-agent.sh`, which reads `RULEBOOK.md` and the permission template from **its own directory**; used that way it would have verified the old unit's rulebook. The unit now carries its own byte copy of the launcher. (7) The wrapper pattern is `CellRelease`, not `Release` |
