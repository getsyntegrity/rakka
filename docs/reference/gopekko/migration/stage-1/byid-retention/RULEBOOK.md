# RULEBOOK: retirement of terminated actors from `System.byID` (ADR 0024, tasks T2-T5; issue #541)

Status: PROPOSED, awaiting the Owner's approval of the hashes in `RUNBOOK.md` section 9. Nothing is implemented and no agent has run. Base: `docs/byid-retention-design` `7f52bb5` (PR #542, the ADR 0024 text) on `develop` `cd558941`; kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`; upstream Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`).

Evidence tags: **[P]** read in the pinned Pekko tree (path:line); **[G]** read in goPekko at the base (path:line); **[I]** inference; **sin verificar** = not shown. Paths relative to `upstream/pekko`: `A` = `actor/src/main/scala/org/apache/pekko/actor/`, `T` = `actor-tests/src/test/scala/org/apache/pekko/actor/`, `D` = `actor/src/main/scala/org/apache/pekko/dispatch/`.

## 1. Goal and non-goals
**Goal.** Stop `System.byID` from retaining finalized actors, as decisions D1-D5 of ADR 0024 accepted by the Owner, without changing the result of `Tell`, `Ask`, `Done`, `Watch`/`WatchWith`, `Stop` or name reuse on a handle the System issued.
**Non-goals (scope guard).** T6 (releasing the fields of a finalized cell) is **not** in this unit: no field of `cell` is set to nil or zeroed after finalization. No change to the ID format or `idSeq`. No new exported identifier. No mailbox change. No change to Watch between Systems (#443). No judge run, no mutation campaign, no change to `judge/`. No heap or GC measurement is made or claimed.

## 2. What Pekko does (pin)
- **Termination order [P].** `finishTerminate` (`A dungeon/FaultHandling.scala:247-276`): `aroundPostStop`, `stopFunctionRefs`, `dispatcher.detach`, the parent receives `DeathWatchNotification`, the watchers are told, `unwatchWatchedActors`, then `clearActorFields` and `clearFieldsForTermination`. The comment at `:249-252` says the order is crucial.
- **Fields cleared [P].** `A ActorCell.scala:718-726`: `currentMessage`, `behaviorStack`, `_props = terminatedProps`, `_actor = null`. This is the Pekko origin of T6, which this unit does not do.
- **No ID registry [I].** In the files read there is no table of local actors keyed by ID; a terminated actor is reachable through the refs held to it. Eviction of the Go registry is a Go adaptation with no Pekko counterpart.
- **A terminated ref stays usable [P].** `D Mailboxes.scala:54-72`: the dead-letter mailbox turns an enqueue into a `DeadLetter`. `T DeathWatchSpec.scala:145-152`: watching an already dead actor delivers one `Terminated`.
- **Name reuse is a new identity [P].** `T ActorSelectionSpec.scala:124-148`: after the old actor died, the same name gives `a2.path == a1.path` and `a2 != a1`. `T DeathWatchSpec.scala:248-255`: a child with the same name can be watched after the old one died.
- **Not modelled in Go:** Pekko's `existenceConfirmed = false` on a `Watch` of a dead ref (`A dungeon/FaultHandling.scala:263-264` shows the flag on the normal path); `Terminated` here carries only `Who()`.

## 3. What goPekko has at the base [G]
- `System.byID` has one write (`hierarchy.go:132`, `commit`) and no delete; the field comment says "never pruned" (`system.go:101`).
- `complete` (`hierarchy.go:208-222`) runs after `close(done)` and the owner `signal` (`stop.go:99-114`); it removes the child from the owner's `out`, `stopping` and `stats`.
- Registry readers: `System.Done` (`system.go:154-160`), `askTarget` (`ask_endpoint.go:132-141`, `lookup(r) == record(r.c)`), `System.Stop` (`stop.go:21`), `resolve` for Watch/Unwatch (`watch.go:111`), `leaveWatchees` (`watch.go:312`), `failUpExcept` (`supervision_failure.go:223`), `stats` (`registry.go:36-42`). All of them live in `pekko-actor/actor`; other modules only call `sys.Done(ref)` with native refs in tests.
- `idHandle` (`hierarchy.go:68-74`) is `{id, sys}`; `cell.identity()` builds it (`cell.go:80`). A second literal is built in `handleFailed` for `ChildFailure.Child` (`supervision_failure.go:251`).
- A `Done` on a `Ref` of another System returns `(closed, ErrUnknownHandle)` (`behavior_test.go:142-150`); `askEndpoint` and the test handles (`userRef`, `fixedHandle`, `fakeHandle`, `foreignHandle`, `customHandle`, ghost cells `&ref[int]{c: &cell[int]{sys, cid}}`) must stay unknown to `Done`.
- `watchers` is nil after finalization (`watch.go:297`); `Watch` of a subject with no record, or whose `addWatcher` returns false, calls `notifyTerminated` at once (`watch.go:168-170`).
- The three guardians are committed in `startGuardians` (`guardian.go:162-175`); `WhenTerminated` is `Done(s.guards.root)` (`system_shutdown.go`).
- `go test -count=1 ./pekko-actor/...` is green at the base and takes about two seconds.

## 4. Decisions
Accepted by the Owner on 2026-10-20 as recommended in ADR 0024 (an instruction in the work thread against `7f52bb5`, not a GitHub comment). **Only D1-D5 are accepted.** Everything marked *mine* below is an implementation detail that follows from them; if a detail would change an observable result beyond ADR 0024, the implementer reports `NEEDS-DECISION` and does not choose.
- **D1 (accepted).** Retire the `byID` entry when the owner counts the completion: in `System.complete`, in the critical section that removes `id` from the owner's `out`, only when the signal was a member, and only for non-guardians. After `close(done)` and after the owner `signal` path of `finalize`; never before.
- **D2 (accepted).** `Done(h)` for a handle that carries only an ID, for a retired actor, returns `(closedChan, ErrUnknownHandle)`.
- **D3 (accepted).** The doc comment of `ErrUnknownHandle` (`errors.go:35`) says: a handle this System never issued, or one it retired after the actor terminated and that does not carry its own state.
- **D4 (accepted).** The three guardian entries stay registered.
- **D5 (accepted).** Releasing fields is a later independent unit (T6). Not here.
- **M1 (mine) provenance mark.** `cell` gets a field recording the System that committed it, set by `commit` under `System.mu` before any consumer can hold the ref, read without the registry. `askTarget` accepts a native `*ref[T]` only when that mark equals the cell's own `c.sys` (so a hand-built cell, or a cell marked by another System, fails with `ErrInvalidHandle`). The mark is never cleared. A race-free read is required (use an atomic pointer, or prove the happens-before in a comment).
- **M2 (mine) handles carry the terminal state.** `idHandle` gets a `done <-chan struct{}` field; `cell.identity()` fills it. `*ref[T]` already holds its cell. A small unexported interface (owner, done channel, provenance) lets `Done` ask any native handle for that state.
- **M3 (mine) rule of `System.Done(h)`.** In this order: (1) `lookup(h)` finds a record: return `(rec.doneChan(), nil)` as today. (2) Otherwise, if `h` is a native handle whose `owner()` is this System, whose `done` channel is non-nil, and, for a `*ref[T]`, whose provenance mark is this System: return `(done, nil)`. (3) Otherwise `(closedChan, ErrUnknownHandle)`. A `done == nil` idHandle (hand-built in tests) is case (3), never a nil channel. A handle of another System, an `askEndpoint`, a user-implemented `Handle` or a ghost cell never reaches case (2).
- **M4 (mine) `ChildFailure.Child`.** `handleFailed` builds the handle from the child's record when one is known (`rec.identity()`), and otherwise as the bare `idHandle{id, sys}`. The existing order of events (the `failedMsg` is set before the stale-child check) does not change.
- **M5 (mine) guardians.** A guardian cell carries a flag (set in `newGuardian` before its first activation); `complete` skips the delete for a flagged record. The `record` interface gains the minimum methods needed (for example `retirable() bool`).
- **M6 (mine) stats.** `System.stats` keeps its code; after retirement it returns `ErrUnknownHandle`. The tests that read it after termination read the snapshot through the `Ref` instead (RULEBOOK 7).

## 5. Scope and surfaces
Production, under `pekko-actor/actor/` (9): `registry.go`, `hierarchy.go`, `system.go`, `cell.go`, `ref.go`, `ask_endpoint.go`, `guardian.go`, `supervision_failure.go`, `errors.go`.
New tests (3): `registry_characterization_internal_test.go` (package `actor`, must pass on the base **unchanged**), `registry_retention_internal_test.go` (package `actor`), `registry_retention_api_test.go` (package `actor_test`).
Existing tests that may be edited (6), only for the reasons in section 7: `lifecycle_internal_test.go`, `throughput_internal_test.go`, `termination_internal_test.go`, `hierarchy_internal_test.go`, `hierarchy_concurrent_internal_test.go`, `ask_endpoint_internal_test.go`.
Coordinator only, after the run: `pekko-actor/README.md`, `README.md`, ADR 0014, ADR 0015, ADR 0024 status, `docs/actor-core-parity.md` if it names the registry, `migration/stage-1/byid-retention/EXECUTION-RECORD.md`, `RESUME.md`. Comments in production files that say "never pruned" (`system.go:101`, `ask_endpoint.go:131`) are rewritten by the implementer.
If the implementer needs a file outside the surfaces, or a test shows the design does not hold in some interleaving, it stops and reports `BLOCKED(surface)` or `BLOCKED(runtime)` with the interleaving: it does not patch around it.

## 6. Invariants
- **I1.** An entry leaves `byID` only after the cell is `closed`, its watchers were taken and told, its name released and `done` closed; in `finalize` that is `stop.go:99-114`, and retirement is in `complete`, after the `signal` on `stop.go:114`.
- **I2.** An ID is never issued twice and a retired ID never names another actor (`id.go:14-18`).
- **I3.** A parent's entry outlives its children's (`readyLocked`, `hierarchy.go:163-169`).
- **I4.** The three guardian entries are never retired.
- **I5.** An operation on a handle the System issued does not need the registry for state the handle carries.
- **I6.** The retirement takes only `System.mu`, already held in `complete`; the order hierarchy then cell is unchanged; no cell lock is taken in `complete`.
- **I7.** A duplicate, stale or foreign completion signal retires nothing and panics nowhere; it changes nothing but what it changed before.
- **I8.** `Done`, `Tell`, `Ask`, `Watch`, `WatchWith` and `Stop` give the same result on a terminated handle in the state "closed, not yet retired" and in the state "retired", for every handle the System issued. The only differing result is D2 (a handle with only an ID).
- **I9.** Foreign or forged handles gain no authority from knowing an ID: no result of `Done`, `Ask`, `Watch` or `Stop` for a handle that is not native and owned by this System differs from the base, except the D2 case.

## 7. Tests (go-specs, in-process, deterministic; no sleeps as proof of order)
Every new test and example has `Retention` in its name (the wrapper selects them). Synchronization by channels, the existing seams (`completionPoint`, `linkPoint`, `recheckPoint`, `commitPoint`, `lockPoint`) and the controlled executor (`ctlExec`); a `time.Sleep` is never the evidence of an order. Imitate `termination_internal_test.go`, `hierarchy_internal_test.go`, `watch_internal_test.go`, `ask_terminated_internal_test.go`.
**Characterization file (T2).** `registry_characterization_internal_test.go`. It must pass on the unchanged base; the coordinator checks that. Cases:
- C1 `Tell` to a terminated `Ref`: one `DeadLetter`, `To` the ID, nothing queued.
- C2 `Ask` to a terminated `Ref`: `ErrRecipientTerminated`, the request dead-lettered (ADR 0015 D5); `ErrInvalidHandle` for nil, typed nil, a user-implemented `Ref`, a hand-built unregistered cell (AK20).
- C3 `Done(ref)` before and after finalization: the same channel, `nil` error.
- C4 `Done` of an `idHandle` taken from `Child`, `Children` or `Parent` before termination: `(closed, nil)` after.
- C5 `Watch` and `WatchWith` of a terminated handle: exactly one `Terminated`, or one message, never two.
- C6 `Watch` before and during finalization: exactly one.
- C7 `Stop` repeated, concurrent and after termination: no panic, one `stop-accepted`.
- C8 name reuse: same name, different ID; the old `Ref` still dead-letters; the new one receives.
- C9 `Done` of a `Ref` of another System, of a never-issued ID and of a user-implemented handle with an unknown ID: `(closed, ErrUnknownHandle)`. `Done` of a user-implemented handle carrying the ID of a **live** actor: `(done, nil)`.
**Retention file (T3-T5).** `registry_retention_internal_test.go` and `registry_retention_api_test.go`:
- R1 retirement: with the completion signal captured (`completionPoint` returns true), the entry is present; after delivering it through `sys.complete`, it is absent and the owner's outstanding set shrank in the same step.
- R2 state equivalence (I8): run the operation table of C1-C8 in the state "closed, `done` closed, signal captured" and again after the signal is delivered; the results are identical.
- R3 duplicate, stale (respawned name) and foreign-owner signals retire nothing else (I7).
- R4 guardians (D4): after `Shutdown` returns, `WhenTerminated()` is closed, `Done(root)`, `Done(user)` and `Done(system)` give `(closed, nil)`, and the registry holds exactly the three guardians.
- R5 Ask provenance (M1): a retired `Ref` gives `ErrRecipientTerminated`; a hand-built cell, a cell marked by another System, a user `Ref` and an ID coincidence give `ErrInvalidHandle`.
- R6 `Done` (M3): retired native `Ref` and retired `idHandle`: `(closed, nil)`; retired actor with a user-implemented ID-only handle: `(closed, ErrUnknownHandle)` (D2); a `Ref` or `idHandle` of another System: `(closed, ErrUnknownHandle)`; an `idHandle` with nil `done` and no record: `(closed, ErrUnknownHandle)` and the channel is not nil.
- R7 `stats`: `ErrUnknownHandle` for a retired actor; the counters for a live one.
- R8 name reuse after retirement: new ID; `Watch` of the old handle delivers one `Terminated`; the new actor is unaffected.
- R9 races under `-race`: finalization against `Watch`, `WatchWith`, `Stop`, `Done`, `Tell`, `Ask`, each deterministic through the seams, covering the three states of section 6 I8.
- R10 growth: `Shutdown` of a System after rounds of spawn/stop (top-level, nested, with restarts) leaves exactly the three guardians in the registry; with a live parent, after `Done(parent)` none of its children is registered. The comparison discounts guardians and live actors. This is a structural check, not a memory claim.
**Existing tests (adaptation only).** `lifecycle_internal_test.go` (the `stats` reads at about `:132,:333,:390,:395`) and `throughput_internal_test.go` (about `:77,:127,:132,:173`) read the snapshot through the `Ref`'s cell instead of `System.stats` after termination; `termination_internal_test.go:734-736` (the "never pruned" count: now zero), `hierarchy_internal_test.go:341-348` (`issued` after `doneOf`), `hierarchy_concurrent_internal_test.go:400-408` (rewrite so that it does not depend on whether the parent finished), `ask_endpoint_internal_test.go:256` (rename the test and comment; the assertion stays). No other existing test is edited and no existing test is weakened or removed to pass.

## 8. Directed mutants (coordinator, in a scratch copy; not the judge, not committed)
M-a no retirement; M-b retire before the watchers are told (in `finalize`); M-c retire guardians; M-d `askTarget` back to the registry; M-e `Done` fallback without the owner check; M-f `Done` fallback accepting a user-implemented handle; M-g retire on a non-member signal; M-h provenance mark not set. Each must be detected by a named test; a survivor is reported, not hidden.

## 9. Verification
The implementer may run only: `gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...`, `go build ./pekko-actor/...` and `bash migration/stage-1/byid-retention/run-byid-tests.sh` (the whole `pekko-actor` module once, then the `Retention` tests with `-race`, once). The coordinator runs the gates of `docs/quality-gates.md` (gofmt, vet, tidy, the pinned `golangci-lint`, cyclomatic complexity at most 10, coverage at least 85%, `scripts/check_deps.sh`, headers), the focused `-race` runs and the mutants.

## 10. Differences from Pekko (declared)
The registry and its retirement are a Go design (no Pekko counterpart). Identity is the ID, not path plus uid. `Done` per actor does not exist in Pekko. No parity is claimed beyond the cases listed in section 2, which are compared here by reading and, for the Go side, by the tests of section 7.
