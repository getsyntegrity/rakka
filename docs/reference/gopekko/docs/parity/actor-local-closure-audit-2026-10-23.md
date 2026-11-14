# Local actors vs Apache Pekko: closure audit (lifecycle, hierarchy, supervision, restart, deathwatch)

- **Audited (original cut, kept as historical):** `develop` `a807fed22327dfd3a02587528ce13a85661ddbab` (#739), 2026-10-23. The matrix, the verdicts and "Execution" describe that cut; where a row changed afterwards it says so, and the later state is in "Update after #743 and #744" below.
- **Pekko:** tag `v1.7.1`, `upstream/pekko` at `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. Paths below are relative to `actor/src/main/scala/org/apache/pekko/actor/`.
- **Scope:** local actors in one System. Not audited: Ask (its Ask-from-`Receive` evidence is #744, outside this audit), bounded mailbox, `CoordinatedShutdown`, remoting, clustering, stash.
- **Method:** three read-only inspections of the Go code, its tests and the upstream source, followed by spot checks of the claims that drove a change. Tests were executed (see "Execution"); no remote CI ran.
- **Reading rule:** *equivalent in scope* means the observable local behavior matches for the cases the named tests pin; it is not a parity claim for the capability. A test that exists was not re-read line by line unless stated.

## Matrix

| Capability | Pekko | Go | Evidence (tests) | Status |
|---|---|---|---|---|
| Start is part of the first activation | `ActorCell.scala:642-653`, `Dispatch.scala:79-111` | `cell.go` `startActor`, `beginStartLocked` | `TestPreStartHooks` | equivalent in scope; mechanism differs (ADR 0012) |
| `PreStart` failure: reported to the parent, no `PostStop` for that instance | `dungeon/FaultHandling.scala:247-254`, `ActorLifeCycleSpec.scala:165-188` | `cell.go` `callPreStart`; `stop.go` `finalize` (`initFailed`) | `TestAFailedPreStartKeepsTerminationButSkipsPostStop`, `TestAHealthyStartAfterAnInitFailureKeepsItsPostStop` | equivalent in scope |
| Stop before the first activation | `Create` queued ahead of `Terminate` | `stop.go` `terminate` runs the start step first | `TestPreStartHooks`, judge S20 | equivalent in scope; mechanism differs (ADR 0012, accepted) |
| `PostStop` failure | logged, cleanup continues (`FaultHandling.scala:247-262`) | `stop.go` `callPostStop` | `TestPostStopHooks` | equivalent in scope; the log entry is an internal event, not an event-stream `Error` |
| Children stop before the parent's `PostStop` | `FaultHandling.scala:180-212` | `stop.go` `terminate`/`finalize`, `hierarchy.go` `readyLocked` | `TestDescendantsFinalizeBeforeTheirAncestors`, judge S10 G5.c (two children) | equivalent in scope for ordering; `Done(parent)` implies `Done(descendants)` is a Go contract (ADR 0014 D3) |
| Child names: validity, duplicates, release timing | `Children.scala:269-276`, `ChildrenContainer.scala:142-146` | `names.go`, `hierarchy.go` `reserve`, `stop.go` `detach` | `TestInvalidChildNamesAreRejectedBeforeReservation`, `TestTheNameIsReleasedBeforeTheWatchersAreTold` | equivalent for validity and uniqueness; **adaptation:** Go frees the name before the watchers and the parent are told, Pekko when the parent processes `ChildTerminated` (ADR 0014 D2); no auto-named children |
| Spawn while terminating | `Children.scala:313-314` throws | `hierarchy.go` `reserve` returns `ErrParentStopping`; a racing spawn is committed already stopping | `TestChildSpawnsRaceTheStopOfTheirParent` | adaptation (ADR 0014 D7) |
| Registry retention and cell release | no equivalent registry; `ActorCell.scala:716-726` | `hierarchy.go` `complete`; `stop.go` `releaseTerminated` | `TestRetention*`, `TestCellRelease*` | adaptation. At the original cut (`a807fed`) the release before `Done` closes was not pinned by a test (mutant M5 survived) and there was no heap measurement; both changed after it (#743, see "Update") |
| `OneForOne` / `AllForOne`, directives | `FaultHandling.scala:545-695` | `supervision.go`, `supervision_failure.go`, `supervision_restart.go` | `supervision_*_internal_test.go`, `TestAllForOne*` | equivalent in scope for the built-in strategies; user-defined strategies and `Kill` are **missing** (#450) |
| Default decider | `FaultHandling.scala:218-224` | `supervision_decider.go` `defaultDecider` | `TestDefaultDecider`, `TestDefaultStrategyWatchFailures` | adaptation: no `ActorKilledException` case (a `Kill`-like failure restarts), never escalates, `ErrGoexit` stops |
| Restart budget | `FaultHandling.scala:56-88` | `supervision_budget.go` | `TestSup2BudgetMatrix`, `TestSup2Int32Wrap` | equivalent in scope; the Go zero `Budget{}` denies every restart, Pekko's default is unlimited |
| Restart hooks, new instance, children | `FaultHandling.scala:92-118,278-303`, `Actor.scala:625-643` | `supervision_restart.go` | `TestSup2LifecycleDefaults`, `TestSup2CustomHooks`, `TestSupStopDuringRestart` | equivalent in scope; `PostStop(old)` can run twice when `Stop` arrives during the children wait (ADR 0017 S11); survivor restart order is the Go choice (ascending name), **not validated against Pekko** |
| Messages during restart | mailbox suspended | `blockedLocked`, `sysq` | `TestSup2WaitAndStash`, `TestSup2IdentityAndMailbox` | equivalent in scope; user-level `Stash` is **missing** (#62, #182, #183) |
| Stale `Failed` from an old incarnation | `FaultHandling.scala:305-324` | `supervision_failure.go` `handleFailed` | `TestLoggingL5NothingForAFailedDeciderAnInvalidDirectiveOrAStaleChild`, `TestUidClashOldChildTerminatedNotDelivered` | equivalent in effect (the decider is not called); Pekko's Debug event is not emitted |
| Guardians | `ActorRefProvider.scala:324,518-522` | `guardian.go` | `TestGuardiansStrategiesAndChain`, `TestEscalationOfUserEndsTheSystem` | adaptation; the "guardian failed" log is **missing** |
| `Watch` / `Terminated` | `dungeon/DeathWatch.scala:35-49,99-115,133-152` | `watch.go` `register`, `notifyTerminated`, `processDeathsLocked` | `TestWatchLiveActorDeliversTerminatedOnce`, `TestTerminatedFollowsEarlierMessagesAndPostStop` | equivalent in scope |
| `WatchWith` | `DeathWatch.scala:51-65,81-93` | `watch.go` `WatchWith`, `sameMessage` | `TestWatchWithDeliversTheMessageNotTerminated` | adaptation: repeat detection uses `==`; stash case not ported |
| `Unwatch`, including after `Terminated` is queued | `DeathWatch.scala:67-79` | `watch.go` `unregister`, ignored lane item | `TestUnwatchCancelsAQueuedTerminated` | equivalent in scope |
| Watching an already stopped actor | `ActorRef.scala:690-697` | `watch.go` `register` | `TestWatchOfStoppedActorDelivers` | adaptation: `existenceConfirmed`/`addressTerminated` are absent (#683 open) |
| Watch conflict, self-watch | `DeathWatch.scala:38,125-131` | `addRegistration`, `ErrWatchConflict` | `TestConflictingRegistrationFailsOnlyTheWatcher`, `TestSelfWatchIsANoOp` | adaptation: typed panic; with the default decider the watcher is **restarted**, not stopped |
| Death pact | `Actor.scala:652-658`, `FaultHandling.scala:221` | `watch.go` `deliverRuntime`, `ErrDeathPact` | `TestDeathPactWithoutHandlerStopsTheWatcher` | equivalent in scope (reduced upstream body) |
| Watcher restart keeps watches | `DeathWatchSpec.scala:195` | registrations live on the cell | `TestSup2Watches` | equivalent in scope (adapted test) |
| `Terminated` held while the watcher is suspended | `DeathWatchNotification` is a system message (`ActorCell.scala:522`); it enqueues `Terminated` as a user message (`DeathWatch.scala:99-115`), which waits behind the suspension | `blockedLocked` | `TestTerminatedIsHeldWhileTheWatcherIsSuspendedAndDeliveredOnceAfterTheResume` (added by this audit) | equivalent in scope; before this audit only the restart case had a test |
| `Terminated` pending when the watcher stops | dead-lettered by `Mailbox.cleanUp` (`dispatch/Mailbox.scala:340-353`) | discarded, never dead-lettered (`watch.go`) | none | **divergence, Owner decision open (#693)** |

## Verdict per capability

| Capability | Verdict |
|---|---|
| Lifecycle | Equivalent in scope for start, stop, hooks and their failures. Mechanism differs from Pekko; not total parity. |
| Hierarchy | Equivalent in scope for creation, uniqueness, stop order. Deliberate adaptations: earlier name release, spawn gate at stop acceptance, `Done` implies descendants. Cell release is integrated. At the original cut its ordering against `Done` was unpinned; since #743 test T04b pins it (see "Update"), and the adjacency of `releaseTerminated()` and `close(c.done)` rests on inspection. |
| Supervision | Equivalent in scope for the built-in strategies, directives and budget. **Open:** user-defined strategies and `Kill` (#450), default-decider classification (#448), root "guardian failed" log. |
| Restart | Equivalent in scope. **Open:** `Stash`; survivor order not validated against Pekko; `Recreate` to a terminated child is not dead-lettered (`PORT-GAP(recreate-dead-letter)`). |
| Deathwatch | Equivalent in scope for local `Watch`, `WatchWith`, `Unwatch`, already-dead subjects and death pact. **Open:** `existenceConfirmed`/`addressTerminated` (#683), pending `Terminated` at watcher stop (#693), cross-System watch (#443). |

None of these is declared as complete Pekko parity.

## Execution

Original run (at the branch head of 2026-10-23, from `pekko-actor`; not re-run for the head that carries the update):

- `go vet ./...`: clean.
- `go test ./... -count=1`: `actor`, `internal/deadletter`, `internal/mailbox` pass.
- `go test ./actor -race -count=3`: pass.
- The new test was checked against a deliberate mutation (`blockedLocked` ignoring `suspended`): it fails, and the file was restored.

Limits at that cut: no heap measurement, no mutation campaign beyond the one above, no remote CI, and the upstream tests that need a JVM-only facility (system-message injection, stash) were not run against Pekko in this audit.

## Update after #743 and #744 (2026-10-24)

This section does not rewrite the original audit. It records what changed in the state it described and what was checked for this update.

**What this update reviewed.** `develop` `ece0b7ad72e43a2f6594c5e44e17f227287d43a4` (it contains #739, #741, #743 and #744) merged into the audit branch; the mutation check below ran on a scratch copy of the merge commit `a083d940aadc459417b83f83bdd7dc2dca972fc3`, and the full list of gates ran on the head of this branch at the time of the update (its SHA is in the pull request body). The commits after the merge change documentation only, so the Go content of both is the same (`git diff` of `*.go`, `go.mod`, `go.sum` and `go.work` between them is empty).

**State of T6 and #563, as recorded elsewhere (not re-measured here).**

- The T6 implementation was integrated by #739 (squash `a807fed2`). Its additional tests and evidence were integrated by #743 (squash `d2a85040`), recorded in sections 16 to 18 of `migration/stage-1/cell-release-563/EXECUTION-RECORD.md`.
- #563 is closed with an evidence comment. #461 stays open for the consolidated actor closure.
- Mutants: the historical author-run campaign detected 22 of 24, and M5 and M11 survived it. M5 and M11 were later detected, each alone, by one test (T04b), failing 3 of 3 and 4 of 4 runs in the author's scratch copy and again in the independent reviewer's (record 16.2, 17.3). There was no new full campaign and no 24 of 24 is claimed.
- The independent review of that test is PASS_WITH_MINORS at `177ed4cac1c61e8aa8e400d992462dacf75a78dd`. The final gates ran on `23771e91cd48de8baa83a63f270d976effb1f4ad` (record section 18). Checked with git for this update, not taken from the record: the tree of the #743 squash `d2a8504` is identical to the tree of the authorized head `52ab6f67abfa5600e40ebc33dea4494f5675c8e0`, and `git diff` between `23771e9` and `52ab6f6` shows no change in `*.go`, `go.mod`, `go.sum`, `go.work` or `go.work.sum`.
- T04b polls with `time.Sleep(1ms)` as cadence; its 10 s limit only bounds a failure. That `releaseTerminated()` sits immediately before `close(c.done)` rests on inspection of `stop.go`, not on a test.
- One heap measurement exists, for one scenario: 200 stopped actors of 1 MiB with every `Ref` held, reachable heap after GC about 0.4 to 0.8 MiB with the release against about 200 MiB before #739 (record 16.3, 17.4). It measures reachability after GC, not memory returned to the operating system, and says nothing general.
- Not established: the primary source of the Owner's approval of O1 was not found, so it is not presented as approved. `-race` on Linux was not run, and the GitHub Actions jobs of #743 did not start because of billing, so there is no CI result. The additional validations the record recommends are not requirements.

**What changed in this audit's own statements.** The matrix row for registry retention and the Hierarchy verdict no longer say that the release ordering is unpinned or that no heap measurement exists, and "Issues" no longer lists #563 as open. The original text is kept in the original cut above; nothing in the sections of the execution record was rewritten.

**#744 (Ask from `Receive`).** Integrated in `develop` as `ece0b7ad72e43a2f6594c5e44e17f227287d43a4`; Ask stays outside this audit. `AskFrom` and `PipeToSelf` have no production implementation. The options A, B and C and the seven design decisions of its proposal are pending, and #675, #674 and #745 (the `TestAskTerminated_K1` flake) are open. Test R3 shows the order `ask-start`, `ask-return`, `post-stop`; it waits for `Done` only to complete the log and does not show when `Done` closes. No parity of Ask is claimed.

**New checks for this update, run on `a083d940aadc459417b83f83bdd7dc2dca972fc3`** (darwin/arm64, go1.27.2, `golangci-lint` v2.14.0 built with go1.27.2). They are separate from the original run above.

- The new test `TestTerminatedIsHeldWhileTheWatcherIsSuspendedAndDeliveredOnceAfterTheResume` was run against a scratch copy outside the repository where `blockedLocked` ignores `suspended` (`return c.restarting`). It failed at its first expectation (`watch_suspended_internal_test.go:55`: expected nothing to be taken while suspended, got `[msg terminated]`), not by timeout or compilation; the unmutated copy passed. The mutation was not committed, and the copy was removed.
- The command list and results of the gates are in the pull request body.

Not run: `-race` on Linux, remote CI (jobs blocked by billing; that is not a green CI and not a functional failure), mutation campaigns, the judge campaigns, heap measurements, containers.

## Statements corrected by this audit

- `docs/actor-core-parity.md`: D2 is retired (ADR 0012 is accepted); the "no system queue" difference; `System.Shutdown` and supervision logging are no longer pending; `finish` is now `finalize`.
- `pekko-actor/README.md`: a watch conflict or invalid handle fails the watcher and the parent's decider chooses (the default restarts); T6 is integrated.
- `docs/adr/0024-actor-registry-retention-after-termination.md`: T6 is integrated; the retention table is labelled as the pre-T6 baseline.
- `pekko-actor/actor/names.go`: comment only; top-level names are validated.
- Update of 2026-10-24: after merging `develop`, the T6 status text of `pekko-actor/README.md` and of ADR 0024 is the one integrated by #741 (T6 integrated, #563 closed, #461 open); this audit keeps only its own corrections to those files (the watch-conflict wording and the retention-table label).
- Not changed: ADR 0012 still says `finish` and "restarts do not exist in this runtime"; it is an accepted decision record and is left as history.

## Issues

Stay open after this PR: #461 (inventory not finished), #450, #448, #683, #693, #443, #692, #691.

#563 was open at the original cut with M5 and M11 surviving and no heap measurement; it has since been closed with an evidence comment (https://github.com/getsyntegrity/gopekko/issues/563#issuecomment-6377285344), and this PR does not reopen or close it.
