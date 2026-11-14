# RULEBOOK: release of the fields of a terminated cell (ADR 0024 D5, task T6; issue #563)

Status: PROPOSED. **O0 (the Owner's approval of the hashes in `RUNBOOK.md` section 9 and of the budget) is pending.** Nothing is implemented and no agent has run. Base: `develop` `f23a215f9701dd7b5046dada0150ba75dfbe61c1`; kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`; Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1). `PROPOSAL.md` of this directory is part of the contract: its sections 2, 3 and 8 are normative and are hashed with it.

Evidence tags: **[P]** read in the pinned Pekko tree; **[G]** read in goPekko at the base; **[I]** inference. **sin verificar** = not shown.

## 1. Goal and non-goals
**Goal.** After a cell finalizes, drop the references that the cell holds to the user instance, the factory, the failure state and the supervision queues, without changing the result of `Tell`, `Ask`, `Done`, `Watch`/`WatchWith`, `Stop`, identity or the guardians on a handle the System issued.
**Non-goals.** No change to `System.byID`, the ID format, the completion protocol or any exported identifier. No change to the restart path. No release of `ctx`, `self`, `node`, `owner`, `sys`, `done`, `origin`, the flags or the counters. **`owner` is excluded on purpose**: the retention it causes (PROPOSAL 2.4 L2) is a documented limit and a possible later unit. No bounded mailbox (#59), no Watch between Systems (#443). No judge, campaign or stress run. No heap or GC measurement is made or claimed.

## 2. What Pekko does (pin)
`finishTerminate` (`dungeon/FaultHandling.scala:247-276` [P]) runs `aroundPostStop`, `stopFunctionRefs`, `dispatcher.detach`, the parent notification, `tellWatchersWeDied`, `unwatchWatchedActors`, then `clearActorFields` and `clearFieldsForTermination` (`ActorCell.scala:716-726` [P]): `currentMessage`, `behaviorStack`, `_props = terminatedProps`, `_actor = null`. Pekko no longer nulls the context (`ActorLifeCycleSpec.scala:210-213` [P]). The comparison is by reading; no Pekko test was run.

## 3. Decisions
Accepted: **D5** of ADR 0024 (field release is a separate unit after T5). Everything below is a design detail of that unit, proposed and awaiting O0; if a detail would change an observable result beyond ADR 0024, the implementer reports `NEEDS-DECISION` and does not choose.
- **R1.** The release set is `actor`, `factory`, `failedMsg`, `pendingCause`, `startFailure`, `resumeFn`, `resumeReady`, `failed`, `sysq` (PROPOSAL 2.1).
- **R2.** One method, one critical section on `c.mu`, called once in `finalize` after `notifyWatchers` and before `close(c.done)` (PROPOSAL 3.1, 3.2). It runs no user code, takes no other lock, and calls no `Execute`.
- **R3.** The keep set is untouched (PROPOSAL 2.2). `scheduled` stays true after a stop.
- **R4.** No new seam and no new exported identifier (proposed configuration of O1: no new internal test seam). The tests park the finalizer with an in-package `watcherLink` fake and with `completionPoint`. Proposed configuration of O2: no judge and no campaign. Both are preparation choices awaiting O0, not approvals.

## 4. Invariants
- **J1.** Every field of the release set is zero when the owner is signalled, and after.
- **J2.** No field of the release set is zero before `PostStop` returned, and none before the watchers were told.
- **J3.** The keep set is untouched: the operations of ADR 0024 give the results of `rcWant`.
- **J4.** The release takes only `c.mu`, once; no user code, no `Execute`.
- **J5.** Nothing releases outside `finalize`: not `acceptStop`, not `terminate`, not a restart.
- **J6.** After `closed`, nothing refills the release set (`recreate`, `enqueueFailed`, the hold path, `addWatcher`, `Tell`, `Post`, `notifyTerminated`).
- **J7.** A restart leaves `factory` and `actor` non-nil on a live cell.
- **J8.** The guardians' entries stay registered (ADR 0024 D4); their cells are released at shutdown.

## 5. Scope and surfaces
Production under `pekko-actor/actor/` (2): `stop.go` (the call), `cell.go` (the method and the field comments; the method may live in `stop.go` instead).
New tests (2): `cell_release_internal_test.go` (package `actor`), `cell_release_api_test.go` (package `actor_test`).
Existing tests that may be edited: none expected. A test that reads a released field after termination is reported, not weakened.
Coordinator only, after the run: `pekko-actor/README.md`, ADR 0024 status and "Not verified", `docs/actor-core-parity.md` if it names fields, `migration/stage-1/cell-release-563/EXECUTION-RECORD.md`, `RESUME.md`.
A file outside these surfaces, or an interleaving the design does not hold in, stops the run: `BLOCKED(surface)` or `BLOCKED(runtime)`, with the interleaving.

## 6. Tests and mutants
The normative tables are PROPOSAL 8 (T01-T13, M1-M14). Every test name starts with `TestCellRelease`, and the base-passing ones with `TestCellReleaseCharacterization`; the wrapper selects `CellRelease`. Order is proved by channels, `completionPoint`, the fake `watcherLink` and the controlled executor, never by `time.Sleep`. T12 is the only test with a real executor; it is a `-race` detector. **It is not a proof of the order of the release against `close(done)`, and a race build does not guarantee that M5 is caught.** M5 has no deterministic coverage: its order is checked by code inspection, and if the mutant is run and survives it is recorded as surviving. A mutant that survives is reported, never hidden.

## 7. Verification
The implementer may run only `gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...`, `go build ./pekko-actor/...` and `bash migration/stage-1/cell-release-563/run-release-tests.sh` (the module once, then the `CellRelease` tests with `-race`, once; it fails if no test matches). The coordinator runs the gates of PROPOSAL 9.

## 8. Differences from Pekko (declared)
The position of the release follows Pekko (after the watchers are told). The release of `pendingCause`, `startFailure`, `resumeFn`, `failed` and `sysq` is a Go adaptation: Pekko keeps that state in the mailbox, which `detach` cleans up, through `unregister`, into the dead-letter mailbox (`AbstractDispatcher.scala:243-245`, `Mailbox.scala:340-354` [P]). `ctx` and `self` stay, as Pekko stopped nulling the context. No parity is claimed beyond the cases of section 2.
