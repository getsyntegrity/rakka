# Cross-System watch: lock inventory, call-site tables and lock verdicts

**Status: ACCEPTED AS DESIGN by the Owner on 2026-10-25 (work thread); not implemented.** Companion of the [design document](cross-system-watch-design.md) (GOV-15 option B; story `GOV-51`, [#1200](https://github.com/getsyntegrity/gopekko/issues/1200)). It holds the lock evidence that the design document summarizes in its section 4: facts, lock inventory, call-site tables, verdicts P3, P4, P7 and P8, and the executed evidence. The demonstration of carrier B of its section 3.1 (items 1-5, the interleaving table, the retention table, the executor assumptions E1-E10 and U1-U6, what could not be read) is in the next document of the series, `cross-system-watch-carrier-b-demonstration.md` (called the carrier B demonstration below; its sections keep the numbers 4, 5 and 6). Nothing here is implemented or parity-claiming.

Base: commit `3b722fdac9197a0fea9365a253acdddf354db3ff` (the head of the first published draft PR #1199; the `pekko-actor` tree of this commit). Pekko is not touched by this document.

**Owner decisions of 2026-10-25 that frame it** (work thread; **not recorded in a GitHub comment or in an ADR**): carrier B (closure on the subject System's executor) is the Owner's preferred alternative and was **accepted as design on 2026-10-25 with the Q5 defer** after the demonstration of five items (carrier B demonstration, section 4); inline executors are not excluded for now (direction accepted, the design is **not proven deadlock-free**); cross-System removal must progress without waiting for the subject to terminate, under the explicit executor assumptions E1-E10 (carrier B demonstration, section 5.1), which are mandatory implementation criteria of DTH-01 and DTH-02.

## Evidence classes (used in every verdict)

| Class | Meaning here |
|---|---|
| **READ** | The chain was read by hand in the files named; every function on it is listed. "No counterexample found by reading" is the strongest statement this class allows. The absence of a found cycle is not positive evidence of absence. |
| **EXECUTED** | A program was run and its output observed. The only executed evidence is in section 7: the existing same-System suite under `-race`. The cross-System hand-off is not implemented, so nothing of it was executed. A passing run does not prove lock order. No stress run and no reproduction of any interleaving exists. |
| **NOT PROVEN** | Not read, not run, or depends on code outside the reading. Every NOT PROVEN item of the earlier design document is kept. |

**Today's code versus designed code.** Every `file:line` of `stop.go` and `watch.go` below is today's code, in which `acceptStop` calls `leaveWatchees` at `stop.go:56`, `emit` at `stop.go:57` and `Execute(c.drain)` at `stop.go:59`, and nothing hands a release to another System. The **designed** `acceptStop` (not implemented) is, in order: `closeGate`; the `c.mu` section that sets `stopRequested` and takes both the same-System subject IDs and the `(id, owner)` pairs of foreign subjects; unlock; `leaveWatchees` for the same-System subjects (synchronous, unchanged, today's `stop.go:56`); `emit`; `Execute(c.drain)` when `need` (today's `stop.go:59`); and **last**, unconditionally (not inside `if need`), a new step (called `releaseForeign` in this document) that groups the foreign pairs by owner and calls `owner.exec.Execute(closure)` once per owner. The closure must capture the watcher ID `wid` by value, never the watcher's cell. No line number below refers to the designed code.

Paths are relative to `pekko-actor/actor/` unless written `internal/...` (= `pekko-actor/internal/...`). Every `file:line` comes from a file opened in the 2026-10-25 lock pass (full reads: `system.go`, `registry.go`, `executor.go`, `hierarchy.go`, `cell.go`, `stop.go`, `watch.go`, `deadletter.go`, `guardian.go`, `system_shutdown.go`, `names.go`, `mailbox.go`, `supervision_restart.go`, `supervision_failure.go`, `start_failure.go`, `spawn.go`, `port.go`, `ref.go`, `actor.go`, `ask.go` 14-188, `ask_endpoint.go` 14-142, `internal/deadletter/hub.go`, `internal/mailbox/queue.go`; comment-stripped reads: `supervision.go`, `supervision_decider.go`, `supervision_hooks.go`, `supervision_logging.go`, `supervision_budget.go`, `clock.go`, `id.go`; grep-only: `errors.go`, `address.go`, `doc.go`). Tests: only the executor and double definitions listed in section 6 of the carrier B demonstration.

Method note: besides reading every critical section by hand, the lock pass listed mechanically every `Lock` site and every `Execute`, `acceptStop`, `publish`, seam and `sync` use in non-test code. The mechanical list is a cross-check only (its regions end at the first `Unlock`); the evidence is the manual read.

**Contents.** 0 facts; 1 lock inventory and lock-order graph; 2 call-site tables (`acceptStop`, `Execute`, `publish`, cross-System calls, seams); 3 verdicts P3, P4, P7, P8; 7 executed evidence. Sections 4 (carrier B items 1-5, the interleaving table, the retention table, the comparison with carrier A, inline-executor compatibility), 5 (assumptions E1-E10, U1-U6) and 6 (what could not be read) are in the carrier B demonstration; the section numbers of this document are therefore not consecutive, and every "4.x", "5.x" or "6" below that is not preceded by "this document" refers to that document.

---
## 0. Facts the whole argument rests on (all READ)

| # | Fact | Where |
|---|---|---|
| F1 | The only mutexes in non-test code of `pekko-actor` are `System.mu`, `cell.mu` and `Hub.mu`. Other sync: atomics (`idSeq`, `cell.origin`, `cell.guardian`, `node.nextName`, ask endpoint state, `Hub.published`) and `sync.Once` in the cancel of `Subscribe`. `internal/mailbox.Queue` takes no lock. | `system.go:111`; `cell.go:42`; `internal/deadletter/hub.go:32,92`; grep of `sync\.` over non-test files |
| F2 | The only receives/waits on channels in non-test code are in `ask` (`ask.go:112-114, 182-187`, on the asker's goroutine) and `System.Shutdown` (`system_shutdown.go:57-72`, on the caller's goroutine). Nothing in `acceptStop`, `drain`, `finalize`, `terminate`, `register` waits. The only runtime `close(chan)` are `close(c.done)` (`stop.go:119`), the hub's `close(done)` (`hub.go:66`), and the ask endpoint. | grep `<-`, `select`, `make(chan` |
| F3 | The only implementor of `record`, `watcherLink`, `restartable`, `suspendable`, `causeResumer`, `inheritor`, `supervisorLink` in non-test code is `*cell[T]`. No test type implements `record` (grep of `acceptStop(`/`addWatcher(`/`removeWatcher(` definitions over all `*.go`). Test `watcherLink` doubles: `crLink` (`cell_release_internal_test.go:171-172`) and `nameProbe` (`termination_internal_test.go:631-636`). Their `id()` returns a constant; `nameProbe.notifyTerminated` takes `p.sys.mu`, and is called from `notifyWatchers` with no lock held. | `registry.go:9-20`; `watch.go:67-71` |
| F4 | `System.exec` is assigned only in `newSystem` (`system.go:135`, `150-152`); no setter, no close, no reassignment; the interface has one method (`executor.go:17`); production executor is `go f()` (`executor.go:22`). `System.Terminate/Shutdown` never touch it (`system_shutdown.go:30-73`). | grep `exec *=` |
| F5 | `cell.closed = true` is set only at `stop.go:109` (under `c.mu`, in `finalize`). `node.closed = true` only at `hierarchy.go:176` (`closeGate`). `finalize` is scheduled only via `node.fin` (`spawn.go:151`) from `schedule` (`hierarchy.go:194`), which needs `terminating` (set only in `beginTermination`, `hierarchy.go:204`, called only from `terminate`, `stop.go:82`), which runs only inside `drain` after `stopRequested` (`cell.go:120-122`; the only call of `terminate()` in non-test code). So every closed cell passed through `acceptStop` first. | grep `closed = true`, `terminate()`, `.fin` |
| F6 | `byID` entries are deleted only in `retireLocked` (`hierarchy.go:252-256`), reached only from `complete` (`hierarchy.go:240`), which is called only from `signal` (`hierarchy.go:347`) after `finalize`'s `close(done)` (`stop.go:119-120`), and for a non-guardian only (`retirable`, `cell.go:96`). | |
| F7 | Actor IDs come from one process-wide atomic counter and are never reused (`id.go:12-18`; `commit` assigns once, `hierarchy.go:149`). | |
| F8 | `delete` on a nil map is a no-op in Go, so `removeWatcher` (`watch.go:369-373`) on a closed cell (`watchers == nil` after `takeWatchersLocked`, `watch.go:394`) does nothing. | language rule |

---

## 1. Lock inventory

### 1.1 `System.mu` acquisitions (19 sites in non-test code)

"Reaches" = does the critical section, transitively, reach another lock, an `Execute`, user code, a publish, a channel op or a wait. All rows: all callees listed were read.

| # | file:line | function | Held while | Calls under lock (transitive) | Reaches | Verdict |
|---|---|---|---|---|---|---|
| S1 | `registry.go:32` | `lookupID` | `byID[id]` read (defer unlock 33) | none | none | READ |
| S2 | `hierarchy.go:99` | `reserve` | name table writes (defer 100) | `new(token)` | none | READ |
| S3 | `hierarchy.go:119` | `rollback` | map delete; unlock 126; then `schedule` 127 outside | `readyLocked` (pure on `node`) | none | READ |
| S4 | `hierarchy.go:143` | `commit` | whole body (defer 144) | `nextID` (atomic), `r.assignID` (`cell.go:92`, plain field store of a not yet published cell), `r.markOrigin` (atomic store), `r.id()` (`cell.go:91`, field read), **`i.inherit(n.susp)` -> `cell.inherit` takes `c.mu`** (`supervision_failure.go:44-49`, then `c.node.susp = k` after unlock) | **`cell.mu` of the new cell (order `System.mu` then `cell.mu`, the documented order, `hierarchy.go:41-43`)**; nothing else | READ |
| S5 | `hierarchy.go:166` | `detach` | map delete; unlock 170 | `e.rec == r` (interface pointer compare) | none | READ |
| S6 | `hierarchy.go:175` | `closeGate` | `n.closed = true`; unlock 177 | none | none | READ |
| S7 | `hierarchy.go:202` | `beginTermination` | snapshot (defer 203) | none | none | READ |
| S8 | `hierarchy.go:218` | `drained` | unlock 220; `schedule` 221 after | `readyLocked` | none | READ |
| S9 | `hierarchy.go:234` | `complete` | unlock 243; `schedule` 244 and `wake()` 245-247 after | `readyLocked`, `retireLocked` (`r.retirable()` atomic load, `cell.go:96`), `forgetChildLocked` (map ops only) | none | READ |
| S10 | `hierarchy.go:283` | `noteStopID` | map write (defer 284) | none | none | READ |
| S11 | `hierarchy.go:297` | `beginWait` | stores `n.waiter = w` (defer 298); `w` is not called here | none | none | READ |
| S12 | `hierarchy.go:308` | `statsFor` | map (defer 309) | none | none | READ |
| S13 | `hierarchy.go:324` | `survivors` | snapshot (defer 325) | `slices.Sort` on strings | none | READ |
| S14 | `hierarchy.go:353` | `outstanding` | len reads (defer 354) | none | none | READ |
| S15 | `hierarchy.go:360` | `child` | (defer 361) | `e.rec.identity()` (`cell.go:93`, builds `idHandle`, lock-free) | none | READ |
| S16 | `hierarchy.go:370` | `children` | (defer 371) | `identity()` | none | READ |
| S17 | `hierarchy.go:396` | `childByID` | (defer 397) | `e.rec.id()` | none | READ |
| S18 | `supervision_failure.go:63` | `adjustCore` | whole body (defer 64) | **takes `c.mu` at 65** (released at 72 or 84, `System.mu` still held), `holdLocked` -> `convertsLocked`, `hasWorkLocked` -> `sysReadyLocked`/`blockedLocked`/`depthLocked` (pure), then `childRecordsLocked` (`e.rec.id()`), `slices.DeleteFunc` closure (`r.id()`) | **`cell.mu` of `c` (same System)**; no `Execute`: the caller executes after return (`supervision_failure.go:159,180`) | READ |
| S19 | `supervision_restart.go:203` | `snapshotRound` | (defer 204) | `rec.id()`, `slices.Sort` | none | READ |

`System.mu` is never held across: `Execute`, `publish`, `emit`, user code, a channel operation, or another System's lock. The only nested lock is `cell.mu` of a cell of the same System (S4, S18).

### 1.2 `cell.mu` acquisitions (34 sites in non-test code, `lockDrain` included)

| # | file:line | function | Calls under lock (transitive) | Reaches | Verdict |
|---|---|---|---|---|---|
| C1 | `cell.go:111` (via `lockDrain`, `cell.go:107-112`) | `drain` iterations (unlock at 121,127,134,139,144,150,154,160,164) and `endActivation` (unlock at 177,183,189,194,199) | `settleDeathsLocked` (see C32), `takeSysLocked`, `takeFailedLocked`, `blockedLocked`, `takeRuntimeLocked` -> `laneReadyLocked` (builds closures that run AFTER unlock), `depthLocked`, `popLocked` -> `queue.Pop`, `sysReadyLocked`. Seam `lockPoint` runs at `cell.go:108-110`, BEFORE `Lock` (111) | none (C32 releases the lock before it publishes) | READ |
| C2 | `cell.go:267` | `startActor` | `c.actor = a` | none | READ |
| C3 | `cell.go:347` | `endCreateOnGoexit` | flag write | none | READ |
| C4 | `cell.go:355` | `snapshot` (defer) | `depthLocked` | none | READ |
| C5 | `port.go:71` | `postPort` | `occupiedLocked`, lane append; unlock 73 or 82 BEFORE `publish` (74) and `Execute` (84) | none | READ |
| C6 | `port.go:97` | `portOf` (defer) | allocation | none | READ |
| C7 | `ref.go:46` | `Tell` | `occupiedLocked`, `pushLocked` -> `queue.Push`; unlock 50/56/65 BEFORE `publish` (51,57) and `Execute` (67) | none | READ |
| C8 | `start_failure.go:134` | `setResume` | field writes | none | READ |
| C9 | `start_failure.go:148` | `markResume` | field write | none | READ |
| C10 | `stop.go:44` | `acceptStop` | `takeOutgoingLocked` -> `takePortDeadLocked` (append); unlock 46 or 55 | none | READ |
| C11 | `stop.go:76` | `terminate` | `beginStartLocked` | none | READ |
| C12 | `stop.go:104` | `finalize` | `takePendingLocked` -> `queue.TakePending`, `takeWatchersLocked`; unlock 111 | none | READ |
| C13 | `stop.go:130` | `releaseTerminated` | field clears | none | READ |
| C14 | `stop.go:142` | `beginPostStop` (defer) | none | none | READ |
| C15 | `supervision_failure.go:45` | `inherit` | called UNDER `System.mu` (S4) | none | READ |
| C16 | `supervision_failure.go:65` | `adjustCore` | UNDER `System.mu` (S18); pure helpers | none | READ |
| C17 | `supervision_failure.go:105` | `enqueueFailed` | unlock 107 or 115 BEFORE `Execute` (117) | none | READ |
| C18 | `supervision_failure.go:145` | `suspendedCount` (defer) | none | none | READ |
| C19 | `supervision_failure.go:189` | `resumeChildren` | unlock 194 before the loop | none | READ |
| C20 | `supervision_failure.go:236` | `markPerpetrator` (defer) | none | none | READ |
| C21 | `supervision_restart.go:92` | `recreate` | unlock 94 or 102 BEFORE `Execute` (104) | none | READ |
| C22 | `supervision_restart.go:111` | `wakeRestart` | unlock 117 BEFORE `Execute` (119) | none | READ |
| C23 | `supervision_restart.go:165` | `flag` (defer) | none | none | READ |
| C24 | `supervision_restart.go:171` | `setFlag` | none | none | READ |
| C25 | `supervision_restart.go:269` | `beginRestart` | none | none | READ |
| C26 | `supervision_restart.go:424` | `installNew` | `c.actor = a` | none | READ |
| C27 | `supervision_restart.go:446` | `endOwnSuspension` | `perp` write (after `adjustCore` returned) | none | READ |
| C28 | `watch.go:156` | `stopAccepted` (defer) | none | none | READ |
| C29 | `watch.go:201` | `addRegistration` (defer) | `sameMessage` (reflect, `==` on comparable values only: `watch.go:134-140`, no user method) | none | READ |
| C30 | `watch.go:229` | `unregister` | map deletes; unlock 231 or 239 BEFORE `removeWatcher` (241) | none | READ |
| C31 | `watch.go:251` | `notifyTerminated` | lane append; unlock 254 or 263 BEFORE `Execute` (265) | none | READ |
| C32 | `watch.go:282` | `settleDeathsLocked` (re-lock) | entered holding `c.mu` from `drain`/`endActivation`; `processDeathsLocked` -> `admitDeathLocked` -> `terminatedQueuedForLocked`, `queue.Len` (pure); when something was dropped it unlocks (278), calls `c.sys.publish` (280) with NO cell lock, relocks (282) | none | READ |
| C33 | `watch.go:357` | `addWatcher` (defer) | **`w.id()` interface call** (365); production implementor `cell.id` is a field read (`cell.go:91`); test doubles return constants (F3) | none | READ (production + the two test doubles) |
| C34 | `watch.go:370` | `removeWatcher` | `delete` | none | READ |

No function holds two `cell.mu` (no cell-to-cell call is made under a lock: `notifyTerminated`, `addWatcher`, `removeWatcher`, `enqueueFailed`, `recreate`, `suspend` etc. are always called with the caller's lock released).

### 1.3 `Hub.mu` (`internal/deadletter/hub.go`)

| file:line | function | Under lock | Reaches |
|---|---|---|---|
| `hub.go:50-61` | `Publish` | read `subs`, insert `done` in `inflight`; unlock before any subscriber (68-70) | none |
| `hub.go:63-65` | `Publish` deferred cleanup | delete from `inflight`; `close(done)` after unlock (66) | none |
| `hub.go:87-91` | `Subscribe` | copy-on-write append | none |
| `hub.go:95-103` | cancel (inside `once.Do`) | rebuild `subs` | none |
| `hub.go:130-136` | `snapshot` (Barrier) | copy `inflight` keys | none |

`Hub.mu` is a leaf: no lock is held while taking it by any hub function, and no hub function calls out while holding it.

### 1.4 The complete lock-order graph (production code)

Edges (hold X, then acquire Y): `System.mu -> cell.mu` only (S4->C15, S18->C16, both on cells of the SAME System). `Hub.mu` has no edges. There is no `cell.mu -> anything`, no `System.mu -> System.mu`, no `cell.mu -> cell.mu`, no edge between two Systems.

Every cross-System interaction (`notifyTerminated`, `addWatcher`, `removeWatcher`, `lookupID`, `Execute`, `publish`) is made with ZERO runtime locks held by the caller (tables 2.1, 2.2, 2.4). So cross-System watch cannot add an edge. **READ**, under assumptions U1-U4 (carrier B demonstration, section 5).

---

## 2. Call-site tables

### 2.1 Every call of `acceptStop` (non-test code)

"Lock set" = runtime locks (`System.mu`, `cell.mu`, `Hub.mu`) held by the calling goroutine at the call. All sites have lock set = {} (empty). "Reached from" is the chain I read to a function that holds no lock.

| # | file:line | Enclosing function | Reached from (chain read) | Lock set | Stack kind |
|---|---|---|---|---|---|
| A1 | `stop.go:22` | `System.Stop` (after `s.lookup`, lock released, `registry.go:23-35`) | user code; `Terminate` (`system_shutdown.go:31`); `Context.Stop` (`stop.go:31-34`: `noteStop` then `Stop`); `defaultPreRestart` (`supervision_restart.go:356`) | {} | any goroutine / inside an activation / unwinding |
| A2 | `actor.go:53` | `Context.StopSelf` | user handlers; `guardian.OnTerminated` (`guardian.go:29`) via `deliverRuntime` (`watch.go:442-477`) | {} | inside the activation (after `c.mu.Unlock()` at `cell.go:154` for the step) |
| A3 | `stop.go:83` | `terminate` | `drain` after `c.mu.Unlock()` (`cell.go:121-122`); loop over `beginTermination` snapshot (lock released: defer 203) | {} | inside the activation |
| A4 | `spawn.go:120` | `spawn` | after `commitCell` returned (`commit` unlocked, `spawn.go:157,119`) | {} | caller of Spawn (user, handler, or startup) |
| A5 | `guardian.go:124` | `newGuardian` | `startGuardians` (`guardian.go:169-172`) inside `newSystem`; `stopping` is false for fresh nodes (`spawn.go:129-162`, `commit` returns `n.closed`), and `need` is false anyway (`scheduled: true`, `spawn.go:149`) | {} | `NewSystem` caller |
| A6 | `cell.go:340` | `receiveFailed` | `callReceive` (`cell.go:331`, after `protect`), Goexit closure (`cell.go:315`); `callReceive` from `drain` (`cell.go:165`, after unlock 164) and `deliverRuntime` (`watch.go:451`) | {} | inside the activation / Goexit unwinding |
| A7 | `watch.go:487` | `terminatedFailed` | `deliverRuntime` (`watch.go:458,466,476`) from the step closure (`start_failure.go:162`, after unlock at `cell.go:154`) | {} | inside the activation |
| A8 | `port.go:139` | `portFailed` | `deliverPort` (`port.go:117,131`) <- `deliverRuntime` (`watch.go:444`) | {} | inside the activation |
| A9 | `start_failure.go:174` | `startFailed` | `drain` (`cell.go:135`, after unlock 134); `runResume` (`start_failure.go:119,130`) <- `takeRuntimeLocked` closure (`start_failure.go:159`) | {} | inside the activation |
| A10 | `supervision_failure.go:331` | `apply` (Stop) | `carryOut` default branch (`supervision_failure.go:301`) <- `handleFailed` (`:284-286`) <- `drain` (`cell.go:145`, after unlock 144); also the `apply` closure inside `logDecision`'s Goexit handler (`supervision_logging.go`, read) | {} | inside the parent's activation |
| A11 | `supervision_failure.go:343` | `failParent` | `handleFailed` (`:276,280`), `apply` (`:324,333`) | {} | inside the parent's activation |
| A12 | `supervision_restart.go:188` | `applyRestart` (denied) | `carryOut` (`supervision_failure.go:297`) | {} | inside the parent's activation |
| A13 | `supervision_restart.go:263` | `stopRound` | `applyRestartAll` (`:235`), `carryOut` (`supervision_failure.go:299`) | {} | inside the parent's activation |
| A14 | `supervision_restart.go:293` | `faultCreate` | `runRecreate` (`:143`), `applyResume` (`:155`) <- `runSys` (`:124-135`) <- `drain` (`cell.go:140`, after unlock 139) | {} | inside the activation |
| A15 | `supervision_restart.go:497` | `restartFailed` | `finishRecreate` (`:458,464,472`), `preRestartGoexit` (`:343`) | {} | inside the activation / Goexit unwinding |
| A16 | `supervision_restart.go:534` | `reportInitFailure` | `startActor` (`cell.go:259,264`), `callPreStart` (`:289,300`), `finishCreate` (`supervision_restart.go:509,514`) | {} | inside the activation / Goexit unwinding |
| A17 | `registry.go:15` | interface declaration `record.acceptStop` | calls through it are A1, A3, A10, A12, A13, A14 above | n/a | n/a |

Not a call site but relevant: `acceptStop` itself calls `closeGate` (`System.mu`, released at `hierarchy.go:177`) at `stop.go:43`, then `c.mu` (`stop.go:44-55`), and only then, in today's code, `leaveWatchees` (`stop.go:56`), `emit` (`stop.go:57`) and `Execute(c.drain)` (`stop.go:59`). At each of those the lock set is {} (READ: `stop.go:46,55` unlock paths; `closeGate` already returned). The designed foreign hand-off is a later statement of the same function and is made with the same empty lock set (it acquires nothing before it calls `Execute`).

### 2.2 Every `Execute` call (non-test code)

| # | file:line | Function | Lock set at the call | How established | Stack kind |
|---|---|---|---|---|---|
| X1 | `guardian.go:179,181,183` | `startGuardians` | {} | no lock anywhere in `startGuardians` | `NewSystem` caller |
| X2 | `cell.go:178,190,195` | `endActivation` | {} | unlock at 177,189,194 right before | end of an activation |
| X3 | `cell.go:241` | `handOver` | {} | called from Goexit handlers (`cell.go:260,291,316`; `watch.go:467`; `port.go:118`; `start_failure.go:120`; `supervision_failure.go:277`; `supervision_restart.go:337,344,460,465,511`; `supervision_logging.go`), which run during user-code unwinding | Goexit unwinding |
| X4 | `hierarchy.go:194` | `schedule` | {} | callers `rollback` (after 126), `drained` (after 220), `complete` (after 243) | caller of those |
| X5 | `spawn.go:122` | `spawn` | {} | after `commitCell` | caller of Spawn |
| X6 | `port.go:84` | `postPort` | {} | unlock at 82 | any goroutine |
| X7 | `ref.go:67` | `Tell` | {} | unlock at 65 | any goroutine |
| X8 | `stop.go:59` | `acceptStop` | {} | unlock at 55; `closeGate` returned | see 2.1 |
| X9 | `stop.go:162` | `callPostStop` Goexit handler | {} | runs inside `protect`'s deferred call, PostStop (user code) runs with no lock | Goexit unwinding |
| X10 | `watch.go:265` | `notifyTerminated` | {} | unlock at 263; callers `register` (`watch.go:180`, no lock) and `notifyWatchers` (`watch.go:403`, called from `finalize` at `stop.go:117`, no lock) | caller of notify (for a foreign watcher: the SUBJECT's finalize stack) |
| X11 | `supervision_restart.go:104` | `recreate` | {} | unlock at 102 | caller of recreate |
| X12 | `supervision_restart.go:119` | `wakeRestart` | {} | unlock at 117; invoked as `wake()` from `complete` after unlock (`hierarchy.go:243-247`) | caller of `complete` (the finishing child's `signal`) |
| X13 | `supervision_failure.go:117` | `enqueueFailed` | {} | unlock at 115 | child's activation (`failUpExcept` `:230`) |
| X14 | `supervision_failure.go:159` | `suspendWith` | {} | `adjustCore` has returned (deferred `System.mu` release at 64 ran) | any |
| X15 | `supervision_failure.go:180` | `resumeWith` | {} | same | any |
| (designed, not in the code) | proposed `owner.exec.Execute(closure)` in the new last step of `acceptStop` (`releaseForeign`), after `Execute(c.drain)` (today's `stop.go:59`) | `acceptStop` | {} | see 4.1 of the carrier B demonstration | caller of `acceptStop` |

Result: no `Execute` in production code is made with `System.mu`, `cell.mu` or `Hub.mu` held. **READ** (all 19 sites; the callers of the helper functions X4, X10, X12, X13-X15 were read).

### 2.3 `publish` call sites and the hub

| file:line | Function | Lock set at publish | Subscribers run on |
|---|---|---|---|
| `ref.go:51,57` | `Tell` | {} (unlock 50/56) | caller of Tell |
| `port.go:74` | `postPort` | {} (unlock 73) | caller |
| `port.go:109` | `deliverPort` | {} (step after unlock) | activation goroutine |
| `port.go:156` | `publishPortDead` | {} | finalizing goroutine |
| `stop.go:114` | `finalize` (pending messages) | {} (unlock 111, `detach` returned) | finalizing goroutine |
| `watch.go:280` | `settleDeathsLocked` | {} cell lock (released at 278); no `System.mu` | activation goroutine |
| `ask_endpoint.go:68` | `askEndpoint.Tell` | {} | replier |
| `deadletter.go:29` | `publish` -> `hub.go:48` | n/a | n/a |

### 2.4 Cross-System calls, with the caller's lock set (what P1-P8 must keep)

| Call | Made by | Lock set |
|---|---|---|
| `W.notifyTerminated` | `S.finalize` via `notifyWatchers` (`stop.go:117`, `watch.go:401-405`) | {} |
| `A.exec.Execute(W.drain)` | `notifyTerminated` (`watch.go:265`) | {} |
| `S.addWatcher(W, gen)` (after relaxation) | `register` (`watch.go:179`) | {} |
| `S.removeWatcher(W.id)` | `register` recheck (`watch.go:185`), `unregister` (`watch.go:241`), `leaveWatchees` for same-System subjects (`watch.go:410`) | {} |
| `B.lookupID`, `B.lookup` | `resolve` (`watch.go:122`), `leaveWatchees` for same-System subjects (`watch.go:409`) | {} |
| designed `B.exec.Execute(closure)` | `releaseForeign`, the last step of `acceptStop` | {} (carrier B demonstration, 4.1) |

### 2.5 Test seams: lock status at the call site

| Seam | Call site | Lock held at the call? | Evidence |
|---|---|---|---|
| `lockPoint` | `cell.go:108-110` in `lockDrain` | none (it runs before `c.mu.Lock()` at 111; callers `drain` and `endActivation` enter with the lock released) | READ |
| `linkPoint` | `watch.go:176-178` in `register` | none (`addRegistration` returned, deferred unlock at 202) | READ |
| `recheckPoint` | `watch.go:191-195` (`recheck`), called at 183 and 187 | none (`addWatcher` returned; `stopAccepted` returned) | READ |
| `completionPoint` | `hierarchy.go:344` in `signal`; caller `finalize` `stop.go:120` | none | READ |
| `commitPoint` | `spawn.go:154-156` | none (after `reserve` returned, before `commit`) | READ |
| `bootPoint` | `guardian.go:130-134` via `boot(1..7)` (`guardian.go:167,174,175,177,178,180,182`) | none (`startGuardians` takes no lock) | READ |
| `guards.terminated` | `guardian.go:26-28` in `OnTerminated` | none (activation step) | READ |
| `eventSink.Event` | `system.go:171-175` `emit`; all emit sites are in lock-free regions (checked in the functions of 1.2/2.1) | none | READ |
| `afterSnapshot` (hub) | `hub.go:122-124` | none (`snapshot` returned) | READ |

The BODIES of these hooks are test code and were not read (U4).

---

## 3. Verdicts for P3, P4, P7, P8 (three separate evidence classes)

Assumptions behind every READ below: U1-U6 (section 5.2) and the executor assumptions E1-E10 (section 5.1), both in the carrier B demonstration. "Lock order" below means: no cycle in the graph of 1.4 and no runtime lock held at any callout.

### P3 - subject termination (`stop.go:93-121`, including dead-letter subscribers, `completionPoint`, inline executor)

Chain read: `finalize` -> `beginPostStop` (C14) -> `callPostStop` (user PostStop under `protect`, no lock; Goexit handler `Execute(c.finalize)` `stop.go:162`) -> C12 (`takePendingLocked`, `takeWatchersLocked`, `closed = true`) -> `detach` (S5) -> `publish` (hub, 2.3, no lock) -> `publishPortDead` -> `notifyWatchers` -> `W.notifyTerminated` (C31) -> `A.exec.Execute(W.drain)` (X10) -> `releaseTerminated` (C13) -> `close(done)` -> `signal` (seam `completionPoint`, no lock) -> `complete` (S9) -> `schedule`/`wake` (X4/X12).

- **READ**: lock order; no runtime lock held at any callout of this chain (dead-letter subscribers, `OnTerminated` on the watcher's drain under an inline executor, `completionPoint`, `sink`, `Execute`); `Hub.mu` is a leaf and is not held while subscribers run (`hub.go:50-70`). A dead-letter subscriber therefore never runs under a runtime lock. Under an inline `A.exec`, `W.drain` runs on `S`'s finalize stack with no lock held (X10), before `releaseTerminated`, `close(done)` and `signal` (`stop.go:117-120`).
- **EXECUTED**: none for the cross-System hand-off (not implemented). The same-System suite run of section 7 exercises existing behavior only and is not evidence of lock order.
- **NOT PROVEN**: (a) what a subscriber, a seam body or `OnTerminated` does once called (they can block, wait on `Done`, call `Shutdown` -> self-wait under inline executors, exactly the limit of `system_shutdown.go:46-53`); (b) a panicking subscriber escapes `finalize` unprotected (`publish` has a deferred hub cleanup only, `hub.go:62-67`; the call at `stop.go:114` is not under `protect`), so `notifyWatchers`, `close(done)` and `signal` would not run. This is a documented prohibition (`deadletter.go:35-39`), not a lock issue, and exists today without cross-System watch; (c) behavior of the seam bodies (test code).
- No counterexample to lock safety found by reading.

### P4 - stop of the watcher (all `acceptStop` callers, `runSys` bodies)

Chain read: table 2.1 (all 16 call sites, each traced to a function with no lock), and the `runSys` bodies (`supervision_restart.go:124-135`): `continueRestart` (`:305-311`) -> `finishCreate` (`:504-519`) / `finishRecreate` (`:454-482`); `runRecreate` (`:139-148`) -> `faultCreate` (`:289-296`) / `faultRecreate` (`:278-285`) -> `beginRestart`, `runPreRestart` (`:317-327`, user hooks and `defaultPreRestart` `:352-361` under `protect`), `awaitChildren` (`:299-303`, `beginWait` S11, then `continueRestart` or the stored `wakeRestart`); `suspendWith(nil)` (`supervision_failure.go:156-164`); `applyResume` (`:152-161`) -> `faultCreate`/`faultRecreate`/`resumeWith`. Every lock taken in these bodies is taken alone or in the order `System.mu -> cell.mu` (S18); user code (`installNew`'s factory, `postRestart`, `runPreRestart`, `strategy`, deciders) runs with no lock.

- **READ**: at every `acceptStop` call the lock set is {}; therefore the designed foreign hand-off (the last step of `acceptStop`, after today's `stop.go:59`) is made with no runtime lock held on all routes enumerated in 2.1 (no counterexample found by reading); `runSys` bodies hold no lock across any call; `continueRestart` and `runPreRestart` (not read in the earlier pass) are now read.
- **EXECUTED**: none for the cross-System hand-off (not implemented). The same-System suite run of section 7 exercises existing behavior only and is not evidence of lock order.
- **NOT PROVEN**: user locks held by a hook when it calls `Stop`/`StopSelf` (assumption U1); test double bodies; the guard that "`acceptStop` is only called from `drain`-reachable code or from user code" is a statement about the 16 sites above in non-test code of this package only. Other modules that import `actor` were not read.

### P7 - shutdown of either System, end to end

Chain read: `Terminate` (`system_shutdown.go:30-32`) -> `Stop(user)` -> `acceptStop(/user)` -> `terminate` (A3) -> `beginTermination(user.node)` -> children `acceptStop` -> each child `finalize` -> `signal`/`complete(user.node)` (S9) -> `schedule(user.node.fin)` (X4) -> `/user` finalize -> `notifyWatchers`: `/system` is the watcher (`guardian.go:145`) -> `guardian.OnTerminated` (`guardian.go:25-31`: seam `guards.terminated`, `StopSelf`) -> same for `/system` -> root (`guardian.go:146`) -> root finalize -> `close(root.done)` = `WhenTerminated` (`system_shutdown.go:38-41`); `signal(top, root.id)`: `top.fin == nil`, so `readyLocked` is false (`hierarchy.go:184`); guardians are not retired (`retirable` false). Also read in full: `startGuardians` closures (`guardian.go:163-184`), `newGuardian`, `wireGuardians` (single locks only), `names.go` (pure, atomic counter), `mailbox.go` (pure under `c.mu`, `internal/mailbox/queue.go` has no locks), `start_failure.go` (all), `supervision_restart.go` (all), the `terminated` hook (`guardian.go:26-28`, no lock).

Shutdown of A with foreign subjects alive in B: every cell of A passes through `acceptStop` (F5); with the proposal each releases its foreign links via closures on `B.exec` (carrier B demonstration, 4.4); B's executor is never closed (F4). Shutdown of B with foreign watchers in A: `S.finalize` notifies `W` (X10) on `A.exec` (never closed); `W` discards if `stopRequested || closed` (`watch.go:253`).

- **READ**: lock order and "no lock at any callout" for the whole chain above; the claim that shutdown never touches the executor (F4); the claim that after B's termination `byID` holds only guardians, closed, with nil `watchers`, follows from F5/F6 (no counterexample found by reading; `complete` retires a non-guardian exactly when its signal is a member of `n.out`, `hierarchy.go:235-241`).
- **EXECUTED**: none for the cross-System hand-off (not implemented). The same-System suite run of section 7 exercises existing behavior only and is not evidence of lock order.
- **NOT PROVEN**: liveness/termination of the cascade as a whole (this inspection is about locks; ordering of `root` finalization versus the last `complete(root.node, ...)` was read as handled by `readyLocked` re-evaluation at every removal, `hierarchy.go:233-247`, but not exercised); that no cell stays in `byID` after termination (reasoned from F5/F6, not run); user hooks that block (self-wait limit); the earlier design text "guardians stay in `byID` closed with nil watchers" for a guardian that is `stopping` while a `Watch` arrives (`addWatcher` returns false only after `closed`, `watch.go:359-361`) - covered by the register fallback `watch.go:179-181`, read.

### P8 - "no function anywhere takes `cell.mu` and then `System.mu`"

Chain read: all 34 `cell.mu` critical sections (1.2) with every callee, all 19 `System.mu` sections (1.1), `Hub.mu` (1.3); every `*Locked` helper (`settleDeathsLocked`, `processDeathsLocked`, `admitDeathLocked`, `terminatedQueuedForLocked`, `takeOutgoingLocked`, `takePortDeadLocked`, `takeWatchersLocked`, `laneReadyLocked`, `takeRuntimeLocked`, `takeSysLocked`, `takeFailedLocked`, `hasWorkLocked`, `holdLocked`, `convertsLocked`, `sysReadyLocked`, `blockedLocked`, `depthLocked`, `occupiedLocked`, `pushLocked`, `popLocked`, `takePendingLocked`, `beginStartLocked`) was read and none reaches `c.sys`/`s.mu`; the interface calls under `cell.mu` are `w.id()` (C33) only; the interface calls under `System.mu` are `assignID`, `markOrigin`, `id`, `identity`, `retirable`, `inherit` (S4,S9,S15-S17).

- **READ**: in non-test production code of `pekko-actor/actor` and `pekko-actor/internal/{deadletter,mailbox}`: no function takes `System.mu` while holding `cell.mu`; the only nesting is `System.mu -> cell.mu` (S4/C15, S18/C16). Also: no function holds two `cell.mu`, two `System.mu`, or a lock of A and a lock of B. The claim "cross-System watch adds no lock edge" holds for P1-P4 and for the new hand-off.
- **EXECUTED**: none for the cross-System hand-off (not implemented). The same-System suite run of section 7 exercises existing behavior only and is not evidence of lock order.
- **NOT PROVEN**: test code (doubles and seam bodies; the two `watcherLink` doubles and the executors were read, carrier B demonstration section 6); other Go modules; panics inside a non-deferred critical section would leave a lock held (none of the callees under a non-deferred lock can panic except runtime faults; not proven absent); `-race` cleanliness (not run).

---

## 7. Executed evidence (exact statement)

Two commands were run on 2026-10-25 at commit `3b722fdac9197a0fea9365a253acdddf354db3ff` with `go1.27.2 darwin/arm64`, both with the race detector:

1. `go test -race -count=1 -timeout 14m ./...` in `pekko-actor`. The per-package results come from this run: packages `actor`, `internal/deadletter` and `internal/mailbox`: all `ok`.
2. `go test -race -count=1 -v ./actor` in `pekko-actor`. The counts come from this run: 525 top-level tests (1780 including subtests) passed, 0 failed, 0 skipped.

What these runs are: an execution of the existing same-System behavior under the race detector. What they are not: the cross-System hand-off is not implemented, so nothing of it was executed; a passing run does not prove lock order (a deadlock needs a specific interleaving that a run may never produce); no stress run was made; no interleaving of the table in 4.5 of the carrier B demonstration was reproduced. The verdicts of sections 3 and 4 therefore stay in class READ or NOT PROVEN. The raw outputs of the two runs are not committed to the repository, so a reader cannot check them from the PR.
