# ADR 0024: Retention of terminated actors in `System.byID`

- Status: **accepted for decisions D1-D5 on 2026-10-20**, by the Owner, as recommended below. Provenance: an instruction in the work thread on 2026-10-20
  ("Acepto las decisiones D1-D5 del ADR 0024 presentado en el PR #542, HEAD 7f52bb5fc3d40a04b8f3465c32629dd6ce558e21"); it is not a GitHub comment.
  Only D1-D5 are accepted by that instruction. Any other point of this ADR, and any decision not listed in "Decisions for the Owner", is not accepted by it.
  **Later, on 2026-10-20, the Owner accepted as text of this ADR the implemented details of the owner check and the provenance mark of `Done` and
  `askTarget`** (see "Status of the details" in "Decision"); that is the only acceptance added to D1-D5.
  Accepted is not Pekko parity. Tasks T2-T5 are implemented in PR #543 (record in
  `migration/stage-1/byid-retention/EXECUTION-RECORD.md`); T6 is integrated (#739, squash `a807fed2`; its tests and evidence, #743, squash `d2a85040`; #563 closed with an evidence comment, #461 open), independently reviewed twice (read-only, no blocker; the first incompletely, neither could run tests) and its M5/M11 test once more (PASS_WITH_MINORS at `177ed4c`, record section 17)
  (record in `migration/stage-1/cell-release-563/EXECUTION-RECORD.md`, tracked in #563). The work is tracked in #541.
  The first version of this ADR was a proposal with no code (PR #542).
- Date: 2026-10-20
- Stage: 1 (actor)
- Module: `pekko-actor`
- Issue: [#541](https://github.com/getsyntegrity/gopekko/issues/541). Context: [#461](https://github.com/getsyntegrity/gopekko/issues/461) (section 4, "Recursos"),
  ADR 0014 D5 ("`byID` retention and the `Done` contract for old handles: Separate ADR"), ADR 0015 D6.3.
- Base: `develop` `cd558941dc2b4789a7103c94192af734220b22e0`. Apache Pekko pin: `upstream/pekko` `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1),
  checked against `git ls-tree` of `develop`.

## Feature

Note: the paragraph below describes the base `cd558941`, before the implementation. "Never removes it" and "no `delete` exists in production code" are not true of the branch of PR #543, where `System.complete` retires the entry (see "Decision").

`System.byID` keeps one entry per spawned actor for the life of the System and never removes it (`pekko-actor/actor/system.go:101`; the only write is
`hierarchy.go:132`; no `delete` exists in production code). The entry is the whole `*cell[T]`. A System that creates and stops actors without bound
therefore grows without bound, even when nothing outside the System refers to the stopped actors.

This ADR decides how the System stops retaining terminated actors, which public behavior must not change, and which change needs the Owner.

## Reading convention

"Read" means the author read the code or test at the stated SHA. "Delegated read" (†) means a delegated reader read it and the author did not
re-read it. The analysis sections were written before the implementation and nothing in them was executed: no test, judge, campaign or heap
measurement ran. What the implementation (PR #543) ran, measured and left unverified is in `migration/stage-1/byid-retention/EXECUTION-RECORD.md`.
A statement about reachability is an inference from the code, not a measurement.

## How Pekko solves it

- Module: `actor`, `actor-tests`.
- There is no ID-keyed registry of local actors in the files read (`ChildrenContainer.scala`, `Children.scala`, `ActorCell.scala`). A live child is
  reachable through its parent's children container; a terminated one is reachable only through the `ActorRef` values other code still holds.
  This is a reading of those files, not a search of the whole tree.
- Termination order, `dungeon/FaultHandling.scala:247-276` (read): `aroundPostStop`; `stopFunctionRefs`; `dispatcher.detach` (the mailbox is swapped to the
  dead-letter mailbox and closed†, `AbstractDispatcher.scala:243-245`); the parent receives `DeathWatchNotification`; the watchers are told;
  `unwatchWatchedActors`; then `clearActorFields` and `clearFieldsForTermination`. The comment at `:249-252` says the order is crucial.
- Fields released after termination, `ActorCell.scala:718-726` (read): `currentMessage = null`, `behaviorStack` emptied, `_props = terminatedProps`,
  `_actor = null`. The `LocalActorRef` still holds its cell, path and uid†, so a held ref stays valid and cheap, not free.
- The parent removes a terminated child from its container when it processes the notification†: `ChildrenContainer.scala:125,181`,
  `FaultHandling.scala:327-351`. Until then `reserve` of the same name fails with "is not unique" (`ChildrenContainer.scala:144,209`†).
- Identity: `ActorRef.equals` compares path and uid (`ActorRef.scala:183-186`†); `newUid` is random and non-zero (`ActorCell.scala:392-397`†). A reused name
  gives a new, unequal ref. Tests: `ActorSelectionSpec.scala:123-146`†, `DeathWatchSpec.scala:248-255`†.
- Messages to a terminated ref: the mailbox is the dead-letter mailbox, so the message becomes a `DeadLetter` (`Mailboxes.scala:55-72`†,
  `Dispatcher.scala:72-76`†). No test sends to an already-terminated ref directly (delegated search†).
- Watching a terminated ref delivers `Terminated`: `DeathWatchSpec.scala:145-152` (read: terminate, then watch, then expect the termination).
- Watching a ref of another `ActorSystem` in the same JVM: `DeathWatch.scala:35-49` (read) requires only `InternalActorRef`, and
  `tellWatchersWeDied` filters on `isLocal`†. By reading, it needs no remote. #443 records an executed probe against the pinned Pekko; this ADR did not
  repeat it. This is a separate finding and belongs to #443, not to this ADR (see "Out of scope").

What carries over to Go: a terminated actor is observable through the references held to it, not through a global table. What does not: Pekko has
no ID registry to compare with, so the Go registry is a Go design choice and eviction is a Go adaptation, not a port.

## How GoAkt solves it

Not consulted. `CLAUDE.md` allows GoAkt only from an ADR that shows it solved the feature better; nothing here shows that, and Pekko's model above
already answers the question.

## Evidence

- Benchmark: none. No memory or GC measurement was taken, and none is proposed as a gating proof (see "Validation plan").
- Judge: none run. When this was written no judge scenario had been inspected for calls of `Done`, `Watch` or `Stop` on terminated handles. A delegated
  read-only review made before the implementation found that `judge_internal_test.go` does not read the registry and that the retirement comes after
  the `terminated` and `done` events of `finalize`; that is a reading, not a run, and the judge was not run by the implementation either.
- Complexity: the change touches `registry.go`, `hierarchy.go` (`commit`, `complete`), `ask_endpoint.go`, `system.go` (`Done`), `cell.go` (`identity`), docs. The registry lookups
  below are all of them in production code (`git grep -E "\.lookup\(|\.lookupID\("`, read).
- Idiomatic in Go: yes. A registry that a lifecycle step shrinks, plus handles that carry what they need, needs no finalizers, weak pointers or GC timing.

### What survives finalization today (`cd55894`, read)

| Reference | Where | After `finalize` before T6 (baseline, now superseded by #739 for the cell fields) | After eviction (D1) |
|---|---|---|---|
| `System.byID[id]` -> `*cell[T]` | `system.go:101`, `hierarchy.go:132` | kept forever | removed |
| `cell.actor`, `factory`, `ctx`, `self` | `cell.go:29-31,61` | kept; no assignment clears them (grep of `c.actor`, `c.factory`, `c.ctx`) | released with the cell once no ref holds it |
| mailbox queue | `stop.go:100` (`takePendingLocked`) | pending taken and dead-lettered | same |
| `watchers` | `watch.go:292-299` | set to nil | same |
| `watching`, `lane` | `watch.go:281-289` (at stop acceptance) | set to nil | same |
| `portDead` | `stop.go:101-102` | set to nil | same |
| failure and restart state (`failed`, `sysq`, `failedMsg`, `pendingCause`, `resumeFn`) | `cell.go:56-75` | not cleared by `finalize` (`stop.go:93-115`); other files not searched exhaustively | released with the cell |
| parent's name entry | `hierarchy.go:145-151` (`detach`) | deleted before `close(done)` | same |
| parent's `out`, `stopping`, `stats` entries | `hierarchy.go:208-237` (`complete`, `forgetChildLocked`) | deleted at completion | same |
| `cell.parent` | `cell.go:26` | an `idHandle`: an ID and a `*System` | same |
| `cell.owner` | `cell.go:27` | pointer to the parent's `node`, kept alive by the child | same |
| `ref[T].c` | `ref.go:23` | holds the cell for as long as a consumer holds the ref | same (consumer-owned) |
| Ask endpoint | `ask_endpoint.go:24` | never registered | same |

Two things follow. Releasing the cell's fields frees memory per cell, but `byID` still grows by one entry per actor, so it is not a retention fix by itself.
And the System is not the only holder: a consumer's `Ref` pins the cell, as a `LocalActorRef` pins its cell in Pekko. That consumer-owned retention is out of scope.

### Who needs the registry today (read)

| Operation on a terminated handle | Today | Registry used? | With eviction (D1) |
|---|---|---|---|
| `Ref.Tell` | dead letter, `ref.go:32-40` | no (the ref holds the cell) | unchanged |
| `Ask(ref)` | `ErrRecipientTerminated` (`ask_endpoint.go:113-117`; ADR 0015 D5) | yes: `askTarget` checks `lookup(r) == record(r.c)` (`ask_endpoint.go:137`) | unchanged **only if** the check no longer reads the registry (T3) |
| `Done(ref)` | `(done, nil)` (`system.go:154-160`) | yes | unchanged **only if** a native handle carries its `done` channel (T4) |
| `Done(idHandle)` (from `Child`, `Children`, `Parent`) | `(done, nil)` | yes | unchanged **only if** `idHandle` carries `done` (T4) |
| `Done(h)` for a user-implemented `Handle` with only an ID (`id.go:12`: "validity depends on ID() alone") | `(done, nil)` | yes | **changes**: `(closedChan, ErrUnknownHandle)`; needs D2 |
| `Watch(h)` / `WatchWith` | `Terminated` is delivered at once (`watch.go:168-170`) | yes: a missing record is treated as dead (`watch.go:103-105`) | unchanged: that same path |
| `Unwatch(h)` | no effect on a dead subject | yes | unchanged |
| `System.Stop(h)` / `Context.Stop(h)` | no-op (`stop.go:21`, `acceptStop` returns once closed) | yes | unchanged |
| repeated `Stop` | idempotent (`stop.go:45`) | yes | unchanged |
| `failUp` to the parent | parent record found (`supervision_failure.go:223`; the `stopAccepted` guard is at `:220`) | yes | unchanged: a parent finalizes only after all children completed (`hierarchy.go:164`), so its entry outlives theirs |
| `leaveWatchees` | removes the watcher link from each subject (`watch.go:310-316`) | yes | unchanged: a finished subject has no watchers (`watch.go:297`) |
| `System.stats(h)` | counters of a terminated actor (`registry.go:36-42`) | yes | `ErrUnknownHandle`. Unexported; its callers are three in-package test files (`cell_internal_test.go`, `lifecycle_internal_test.go`, `throughput_internal_test.go`) |
| `WhenTerminated`, `Shutdown` | `Done(s.guards.root)` (`system_shutdown.go`) | yes | unchanged only if the guardians are pinned (D4) |
| name reuse | new ID from the process counter (`id.go:14-18`, monotonic), name freed by `detach` | no | unchanged |
| ownership of a handle by a System | `ref.owner()` and `idHandle.sys` (`ref.go:28`, `hierarchy.go:68-74`) | no | unchanged |

`ErrUnknownHandle` is documented as "returned for a handle this System never issued" (`errors.go:35`). That sentence stops being true for a retired actor when the handle
carries only an ID.

## Alternatives

| | Idea | Frees terminated actors without external refs | Public change | Cost / risk |
|---|---|---|---|---|
| A | Keep every record; release the cell's fields (Pekko's `clearFieldsForTermination`) | No: one record stays per actor, so `byID` grows without bound | none | Smallest. Not enough alone; useful as T6 |
| B | **Retire the entry at completion; native handles carry their terminal state** | Yes, deterministically | `Done` for ID-only handles of retired actors (D2) | Small; touches 5 call sites. Recommended |
| C | Keep a minimal terminal record per ID | Cell yes, record no: still unbounded | none | Exactly preserves `Done`/`stats`, but ADR 0014 already notes tombstones also grow without bound (line 207). Rejected |
| D | Issue IDs from a per-System sequence, so "issued and not present" means "retired" | Yes | **ID format changes** (IDs are unique per process today) | Preserves `Done` for ID-only handles with no set. Changes an observable string and `idSeq` semantics. Offered under D2(b) |
| E | Registry of weak pointers, cleaned by GC | Only when the GC collects | none visible, but timing-dependent | The result of `Done(id)` would depend on GC timing; entries still need cleanup. Rejected |

## Decision (accepted for D1-D5)

Adopt **B**:

1. **Retire at completion.** `System.complete` (`hierarchy.go:208-222`) removes `byID[id]` in the critical section where the owner counts the completion, when the signal is a member of the
   owner's outstanding set (`member`), and only for non-guardians. A duplicate, stale or foreign signal removes nothing, exactly as today.
2. **Native handles carry what they need.** A `Ref` already holds its cell. `idHandle` (`hierarchy.go:68-74`, built by `cell.identity()`) also carries the cell's `done` channel, nothing more.
   `Done` answers from the handle when the registry has no record, and only for a handle this System issued: the handle's owner must be this System and, for a `Ref`, its cell must have been
   registered by this System. For `Done`, a handle of another System, or any user-implemented handle, gains no authority by knowing an ID (implementation detail of D2, added with T4); `Done` does not change. This sentence is about `Done` only; the rules of the other APIs are in their own ADRs: for `Watch`, `WatchWith` and `Unwatch`, ADR 0010, where authority comes from the owning System of the handle, never from the ID.
3. **Ask provenance without the registry.** `commit` marks the cell as registered by its System (set under `System.mu`), and `askTarget` checks that mark instead of
   `lookup(r) == record(r.c)`. The mark records the registering System, and the check requires it to be the cell's own System. AK20 (a native ref whose cell was never registered fails with `ErrInvalidHandle`) is preserved; ADR 0015 D6.3 keeps its meaning, not its mechanism.
   D2 relies on actor IDs being unique across the Systems of one process. That is an existing contractual invariant (`ID` "identifies one actor for the life of the process"); this amendment does not create it, adds no composite key and changes no API. A per-System ID sequence (option D of D2) would break it and would first need a decision of its own.
4. **Guardians stay registered.** The three entries are bounded and `WhenTerminated` depends on `Done(root)`.
5. **Field release (A) is a separate, later unit (T6).** It needs every reader of the released fields proven, and it does not stop `byID` growth.

Status of the details in items 2 and 3 (the owner check of `Done`, the registering-System mark, a nil `done` treated as unknown): they were added to this
text after the Owner accepted D1-D5, and they follow the Owner's implementation instruction of 2026-10-20 (a handle of another System, an invalid one or one
built from an ID alone must not gain authority by knowing that ID). They are implementation details of D2. **The Owner accepted them as text of this ADR on
2026-10-20, after the review of PR #543**, in an instruction in the work thread ("Acepto como texto del ADR 0024 los detalles implementados de chequeo de
dueño y marca de procedencia de Done y askTarget"); it is not a GitHub comment. The acceptance covers the details as implemented in PR #543 and nothing else.

**Amendment 2026-10-25.** The last sentence of item 2 is limited to `Done` and the sentence about the uniqueness of actor IDs across the Systems of one process is added after item 3 (it records an existing invariant, `pekko-actor/actor/id.go:8, 14-18`; it adds no composite key and changes no API). Accepted by the Owner on 2026-10-25 through an explicit message in the working session, not a GitHub comment: "Las enmiendas de ADR 0010, 0015 y 0024 presentadas en el consolidado de #1204, con las correcciones de #1206." The authority of `Watch`, `WatchWith` and `Unwatch` for a handle of another System is defined in ADR 0010, Amendment 2026-10-25, accepted as a design and not yet implemented (implementation note, 2026-10-26: implemented by DTH-01, PR #1207, `cad059c`; `Done` is unchanged; this note accepts nothing new). `System.Stop` takes authority from the ID alone with no owner check (`pekko-actor/actor/stop.go:20-24`) and is a no-op for a foreign handle only because that ID is absent from the caller's registry; this amendment does not change that.

### Invariants

- **I1.** An entry leaves `byID` only after the cell is `closed`, its watchers were taken and told, its name was released, and `done` is closed. In `finalize` that is `stop.go:99-114`;
  eviction is after the `signal` on `stop.go:114`.
- **I2.** An ID is never issued twice (`idSeq` is monotonic, `id.go:14-18`). A retired ID never names another actor.
- **I3.** A parent's entry outlives its children's. The parent's `finalize` is scheduled only when `len(out)+reserved == 0` (`readyLocked`, `hierarchy.go:163-169`), which is evaluated
  in the same `complete` that retires the last child.
- **I4.** The three guardian entries are never retired while the System exists.
- **I5.** An operation on a handle the System issued never needs the registry for state the handle carries: terminal state (`done`), owner (`sys`) and provenance.
- **I6.** `System.mu` is the only lock the retirement takes, in `complete`, where `System.mu` is already held. The order stays hierarchy then cell (`hierarchy.go:38-42`); no cell lock is taken.

### Ordering and synchronization

| Interleaving | Outcome by reading | Test (T5) |
|---|---|---|
| `Watch(h)` before `closed` | the watcher is stored, then told at finalization (`watch.go:168`, `takeWatchersLocked`) | seams `linkPoint`, `recheckPoint` |
| `Watch(h)` after `closed`, before retirement | `addWatcher` returns false, the watcher is told at once | same |
| `Watch(h)` after retirement | no record: told at once (`watch.go:168`) | same |
| `Stop(h)` during or after finalization | `acceptStop` returns early (`stop.go:45`) or no record | repeated and concurrent `Stop` |
| `Done(h)` before / during / after | channel closes at `close(done)`; after retirement it is read from the handle | `Done` with a native handle at each point |
| `Tell` racing the finalization | dead letter once, from the caller (`ref.go:36-40`; ADR 0014) | existing `Tell` race tests, extended |
| Signal duplicated, delayed or after a respawn | not a member: nothing removed, the new incarnation has another ID | seam `completionPoint` (`system.go`, `hierarchy.go:309-314`) |
| Late `failUp` from a finished child | not reachable: the child is closed before its signal, and `failUp` checks `stopAccepted` first (`supervision_failure.go:220`) | covered by a race test |

All rows were read from the code when this ADR was written, and none was run then. The implementation tests the rows through the seams (state
equivalence before and after retirement, the `completionPoint` and `linkPoint` interleavings, the born-stopping child); the tests and their results
are in the execution record.

### Guardians (D4)

`startGuardians` builds `/root`, `/user` and `/system` and stores their handles in `guards` (`guardian.go:162-175`). `WhenTerminated` and `Shutdown` read `Done(s.guards.root)`
(`system_shutdown.go`). Retiring the root after termination would make `WhenTerminated` return `ErrUnknownHandle` and a closed channel instead of its channel.
The recommendation pins the three entries (a constant). The alternative, retiring them once the System terminated, needs `WhenTerminated` to read a channel stored in the System.

## Compatibility and effects on existing ADRs

| Document | Effect if accepted |
|---|---|
| ADR 0014 | D5 ("Separate ADR") is answered by this ADR. Line 207 ("`byID` retention is separate debt") and the statement that the completion signal "removes nothing else" (line 192) are amended by T5: the signal now also retires the entry |
| ADR 0015 | D6.3 and the notes at lines 252, 283 and 307 stop saying the registry is never pruned; AK20 and D5 are unchanged |
| `pekko-actor/README.md:217-219` | the "never pruned" limit is replaced by the new rule |
| `errors.go:35` | the `ErrUnknownHandle` sentence is amended (D3) |
| `system.go:101` comment, `ask_endpoint.go:131` comment, test `ask_endpoint_internal_test.go:256` ("accepts a terminated target: the registry is never pruned") | rewritten by T3/T5 |
| #443 | unchanged; its identity question is independent |

Public API: no new symbol. Unexported: `idHandle` gains a field; `commit` sets a mark; `stats` stops answering for retired actors.

## Pekko-derived and Go-adapted

| Item | Origin |
|---|---|
| A terminated actor stays usable as a ref identity: `Tell` goes to dead letters, `Watch` delivers `Terminated` | Pekko (`DeathWatchSpec.scala:145-152`; `Mailboxes.scala:55-72`†). Comparative validation pending |
| A reused name is a different identity | Pekko (`ActorSelectionSpec.scala:123-146`†, `DeathWatchSpec.scala:248-255`†); in Go identity is the ID, not path plus uid (adaptation) |
| Cleanup order: mailbox closed, parent told, watchers told, then fields cleared | Pekko (`FaultHandling.scala:247-276`, read); the Go order is `stop.go:93-115` |
| Releasing fields of a terminated actor | Pekko (`ActorCell.scala:718-726`, read); Go task T6 |
| The `System.byID` registry and its retirement | **Go adaptation.** Pekko has no counterpart; there is nothing to compare with |
| The terminal state carried by native handles (`done`) | **Go adaptation**; Pekko derives it from the closed mailbox |
| `Done` for an ID-only handle | **Go design.** Pekko has no per-actor `Done` |

Where the comparison rests only on reading: "coincidencia observada por lectura; validación comparativa pendiente".

## Decisions for the Owner (D1-D5 accepted on 2026-10-20 as recommended; see Status)

The "Recommendation" column is the accepted choice for each row.

| | Decision | Options | Recommendation |
|---|---|---|---|
| **D1** | Registry policy | (a) retire at completion, native handles carry state (B); (b) keep records, release fields (A); (c) tombstones (C) | (a) |
| **D2** | `Done` for a handle that carries only an ID, for a retired actor | (a) `(closedChan, ErrUnknownHandle)`; (b) exact result through a per-System ID sequence (D, changes the ID format); (c) keep tombstones (C, unbounded) | (a): the channel is closed, so a waiter still returns; only the error differs, and only for handles the System never issued itself |
| **D3** | Text of `ErrUnknownHandle` | (a) "never issued, or retired after termination and not carried by the handle"; (b) leave | (a) |
| **D4** | Guardians | (a) pinned; (b) retired after the System terminates | (a) |
| **D5** | Field release (T6) | (a) separate unit after T5; (b) with T5; (c) not done | (a) |

## Validation plan

Unit tests with `go-specs` and in-process fakes; no external service. They run in `pekko-actor` only. Infrastructure-backed integration, when it exists, belongs to a dedicated module whose
materialization (`gopekko-testint`) is an open Owner decision; this work needs none.

| Behavior | Case |
|---|---|
| `Tell` to a terminated ref | dead letter once, with `To` = its ID |
| `Ask` to a terminated ref | `ErrRecipientTerminated` and a dead-lettered request; `ErrInvalidHandle` still for nil, typed nil, non-native, never-registered cell (AK20) |
| `Done` before, during, after finalization, and after retirement | native `Ref` and `idHandle`: same channel semantics; ID-only handle: per D2 |
| `Watch`/`WatchWith` before, during and after finalization and retirement | one `Terminated` (or the message), never two, never none |
| repeated and concurrent `Stop` | no panic, one `stop-accepted`, no effect once retired |
| name reuse | same name, different ID; old ref keeps dead-lettering; new ref works |
| races between finalization and `Watch`/`Stop`/`Done`/`Tell`/`Ask` | the seams listed above, under `-race` where the module gate already requires it |
| guardians | `WhenTerminated` channel closes after `Shutdown`; guardian entries present while the System lives |
| System holds no strong reference to a finalized cell | structural: after completion `lookupID(id) == nil` and the registry holds only the guardians and live actors. A GC-based check (`runtime.AddCleanup` or `weak`) may be added as confirmation but is never the proof |
| growth | `len(byID)` returning to the guardian baseline applies **only** to eviction (T5), not to field release (T6) |

No campaign, judge or workflow is run by this ADR. T5 checks `judge/` scenarios for `Done`, `Watch` and `Stop` on terminated handles before it touches code.

## Implementation tasks

| | Task | Depends on | Forecast |
|---|---|---|---|
| T1 | Owner decisions D1-D5 | this ADR | none |
| T2 | Characterization tests of the current contracts above. No production change; they pass on `develop` | T1 | ~300 lines |
| T3 | Provenance mark set in `commit`; `askTarget` without the registry | T2 | ~80 lines |
| T4 | `idHandle` carries `done`; `Done` answers from the handle | T3 | ~150 lines |
| T5 | Retire in `complete`; pin guardians; race and structural tests; adapt the `stats` tests; amend README, ADR 0014/0015, `errors.go`, comments | T4 | ~250 lines |
| T6 | Field release after proving every reader (D5) | T5 | ~150 lines |

Each unit is one PR within the 400-line budget; line counts are forecasts. T3-T5 are Go adaptations with no Pekko source. T6 derives from `clearFieldsForTermination` and is a port:
for any unit that is a port or adaptation, the Code Migration Kit (`migration-kit` pinned at `cf91c9d`) is run with its manifest and execution record; loading its rules does not count.

## Consequences

- Rules out: tombstones for every actor, weak registries, and any change to the ID format in this slice (unless D2(b) is chosen).
- A System with unbounded actor churn stops growing in `byID`; a consumer that keeps a `Ref` still pins that actor's cell (T6 reduces its size).
- The observable change is limited to `Done` on an ID-only handle of a retired actor, and to the unexported `stats`.
- To revisit: whether `Terminated` should carry `existenceConfirmed` as in Pekko (`FaultHandling.scala:263-264`); `Terminated` has only `Who()` here. Not part of this slice.

## Out of scope

- Bounded mailbox and overflow (#59, #176, #177): not a requirement of this slice or established here as a closing criterion of the actor core.
- Watch between two local Systems (#443): by reading it needs no remote in Pekko; goPekko rejects it by local-scope design (`watch.go:108`). That is a separate finding.
- `gopekko-testint`: absent; materialization is an Owner decision.

## Not verified (as of the first version of this ADR; the execution record states what the implementation later checked)

- Every reader of the cell fields that T6 would release (only a grep of `c.actor`, `c.factory`, `c.ctx` was done). The T6 candidate inventoried them by reading (`migration/stage-1/cell-release-563/PROPOSAL.md` sections 2 and 3); that is a reading, not a proof. One independent read-only review re-derived the readers from the code, also by reading, and found none that touches a released value (it was incomplete); a second one read the tests and the invariants and confirmed that the production code had not changed since (record sections 12 and 15).
- Judge scenarios that touch terminated handles.
- That nothing outside `pekko-actor` implements `Handle` and relies on `Done` after termination (a grep of the other modules found only `sys.Done(ref)` with native refs in `pekko-persistence` tests).
- Memory behavior: no measurement.
