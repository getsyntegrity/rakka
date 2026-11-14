# ADR 0014: implementation decisions taken where the ADR is silent

This file holds the implementation decisions of ADR 0014 that the ADR did not decide: O1-O3 of slice 2 (accepted by the owner) and
P1-P7 of slice 3b (**proposals pending owner acceptance**; the behavior is implemented). It was named for slice 2 until slice 3c
and was renamed because it now covers both.

## Slice 2: O1-O3 (accepted)

- Status: **O1, O2 and O3 accepted by the owner on 2026-10-11, for this increment.** D1-D7 of ADR 0014 are unchanged. These three
  points were taken by the writer of slice 2 (PR #45, candidate `788073f`) where the ADR is silent or only implies the answer.
  Accepting them changes no behavior; it fixes the documented contract. The text below keeps the options and the proposals as
  they were put to the owner.
- Evidence: code at `788073f`, `pekko-actor/actor/hierarchy.go` and `spawn.go`.

## O1. What `Child()` and `Children()` return

Today they return an identity-only handle (`idHandle{id, sys}`): it carries the actor ID and nothing else. `Watch(h)` and
`System.Stop(h)` accept it, because both resolve a handle by `ID()`. `Tell` is not available on it.

| Option | Consequence |
|---|---|
| A. Keep the identity-only handle (current) | No type erasure problem: the parent owns the typed `Ref` it got from `Spawn`. Smallest surface. A parent that lost the `Ref` cannot send to the child. |
| B. Return `Ref[any]` | Defeats the typed `Ref[T]` of ADR 0010 and invites unchecked sends. |
| C. Add a typed lookup later, for example `ChildAs[U](ctx, name) (Ref[U], bool)` | Needs its own ADR (runtime type check, failure mode). Not required by D1-D7. |

Proposal: **A now, C as a separate future decision.** The ADR's own signatures (`Child(name) (Handle, bool)`) already say `Handle`.

**Accepted: A.** Metadata handles carry identity only. `Watch`, `Unwatch` and `System.Stop` resolve a handle by `ID()` alone, so
they accept it; this is documented on `idHandle` and in the README.

## O2. The error of a lost reservation

`commit` reports `ok=false` when the entry under (owner, name) is no longer the caller's reservation; `Spawn` then commits
nothing, runs no actor, and returns `ErrNameTaken`. Every writer of a name-table entry, which is what any path to that state
would need to go through:

| Writer | What it can touch | Can it remove or replace someone else's reservation? |
|---|---|---|
| `reserve` | inserts only if the name is absent | No: a held name returns `ErrNameTaken`, nothing is overwritten |
| `rollback` | deletes only an entry with `rec == nil` **and** its own token | No: tokens are distinct allocations |
| `commit` | sets `rec` only if the stored token is the caller's | No |
| `detach` | deletes only an entry whose `rec` equals the finishing record | No: a reservation has `rec == nil`, so it never matches |

No other non-test code writes a name table (`rg 'names\[|delete\(n.names|names =' pekko-actor/actor` over non-test files lists these four writers and the read-only `child`/`children`). A caller's own
rollback runs only if its commit did not, and runs once. Conclusion: **the state is unreachable in production by
construction.** The only way to reach it is a white-box test that edits the table inside the factory, which is what
`TestALostReservationCommitsNothing` does.

| Option | Consequence |
|---|---|
| A. Keep `ErrNameTaken` (current) | The observable condition is literally "someone else holds this name". No new error. If the invariant ever broke, the message would mislead. |
| B. Panic with an internal-invariant message | Fails loudly if the impossible happens; a library panic on a path no caller can trigger. |
| C. Return an unexported error that wraps nothing public | Distinguishes the case in logs; adds an error value no caller can match. |

Proposal: **A**, with the invariant documented where `commit` is defined, because the branch is unreachable and a new error
or a panic would add surface for a state that cannot occur. Revisit only if a writer of the table is added.

**Accepted: A.** `ErrNameTaken` stays as a defense for the lost reservation. It is not a normal collision: the invariant above makes
it unreachable in production, and it is documented on `System.commit` and at the return in `Spawn`.

## O3. When the ID is allocated and who emits the stop event of a child born stopping

- **ID.** It is allocated inside the commit critical section (`assignID`), together with the `byID` insert. ADR 0014 step 5
  says "allocate the ID only here" (at commit): a failed or rolled-back Spawn issues no ID. This follows the ADR.
- **Event.** A child committed while its owner's gate is closed is born stopping: the Spawn caller calls `acceptStop` on it
  before the first `Execute`, so the child emits its own `stop-accepted` event, exactly as any accepted stop does (the
  accepting call emits it, `stop.go`), and it takes the paired ADR 0012 hooks (`PreStart` once, then `PostStop` once). A
  refused Spawn emits nothing.

| Option | Consequence |
|---|---|
| A. As implemented | One rule for every stop: whoever accepts it emits the event. Observers see the same event sequence for a child stopped normally or born stopping. |
| B. Suppress the event for a child born stopping | A stop that is accepted would be invisible; breaks the rule above and the judge's view of acceptance. |

Proposal: **A.** No code change.

**Accepted: A.** The ID is assigned at the commit, and a child born stopping emits its own `stop-accepted`, once, as any accepted
stop does (ADR 0012). `TestSpawnsInFlightWhileTheRootGateCloses` and `TestChildSpawnInFlightWhileItsParentStops` assert it.

## Concurrency coverage

The deterministic tests force one interleaving each through the controlled executor. `hierarchy_concurrent_internal_test.go`
adds two tests that guarantee the overlap without depending on the scheduler: each Spawn is held inside its factory, so it is
reserved and not committed, while another goroutine closes the root gate or accepts the parent's Stop. The Spawn must then succeed
and be committed already stopping (single `stop-accepted`, `PreStart` and `PostStop` once, nothing left registered), and a Spawn
that starts after the close is refused. Two more tests let the scheduler choose, with real goroutines released by barriers: top-level Spawns race the closing of the root gate, and a parent spawning inside
its own turn races `Stop(parent)`. They check the allowed outcomes (success, or `ErrParentStopping` with the factory not run;
refusals form a suffix; a Spawn after the gate closed is refused), and that after every actor finishes no reservation, name or
unregistered child is left and every actor ran `PreStart` and `PostStop` once. Mutants that fail them: reserve ignoring the
gate, commit not registering the ID, detach doing nothing.

## Slice 3b: P1-P7 (proposals, pending owner acceptance)

The recursive stop and hierarchical finalization follow "Creation and the Spawn/Stop race", "Termination" and the acceptance
table of ADR 0014 as written. These points were not decided by the ADR and were taken by the writer of the slice, **not by the
owner**; none changes D1-D7. The code implements them today. They stay proposals until the owner accepts or amends them. The
last column names the Go tests that exercise each point and says what they do not pin.

| ID | Point | Taken | Pinned by |
|---|---|---|---|
| P1 | Finalization is always handed to the executor, also for a leaf evaluated at the termination step (point (a)) | Yes: one rule, and `PostStop` always runs in a function given to `Execute`. The cost is one extra activation per termination. | `TestALeafFinalizesWithNoSignalEverArriving`, `TestADrainedParentFinalizesWithNoSignalEverArriving` and `TestFinalizationIsScheduledExactlyOnce` (controlled executor, which never runs a function inline: `PostStop` runs once and only after the queue is run). No test counts the extra activation, so the cost is not pinned. |
| P2 | What keeps the second evaluation point from finalizing before the owner's in-flight call ended | A `terminating` flag of the node, set under the hierarchy lock by the termination step (after the in-flight call and the start step). A child that completes earlier only leaves the set. | `TestAChildCompletingBeforeItsParentTerminatesDoesNotFinalizeIt` (the parent does not finalize during its own in-flight turn); `TestADrainedParentFinalizesWithNoSignalEverArriving` (a live parent does not finalize when its set empties). |
| P3 | Where a `PostStop` that ends with `Goexit` resumes | In a fresh activation of the finalization step, not of `drain`; the `postStopBegun` flag of slice 3a still makes `PostStop` run once. | `TestGoexitInPostStopStillCompletesTheFinishStep`, `TestGoexitInPostStopAfterAnEarlyStop` and, at every level of a tree, `TestPostStopPanicAndGoexitAtEveryLevelStillComplete`. They pin that `Done` closes and `PostStop` runs once, not which activation resumes it. |
| P4 | Shape of the completion seam | `completionPoint func(owner *node, id ID) bool` in `options`, nil in production; returning true captures the signal and a test delivers it later with `System.complete`. | `TestACapturedSignalIsReportedToTheSeamWithItsOwnerAndID`; the termination tests that hold and replay signals (`TestADuplicateSignalNeverFinalizesAParentEarly`, `TestAStaleSignalAfterARespawnNeverFinalizesALiveParent`). It is a test seam: its shape is not observable behavior. |
| P5 | Owner-size accessor | `System.outstanding(*node)`, the size of the IDs set plus the unresolved reservations. | `TestReservationAndCommitKeepOneOutstandingItem`, `TestRollbackLeavesTheOutstandingSet`, `TestAReservationHoldsTheParentOpenUntilItCommitsOrRollsBack`. It is unexported and used by tests only. |
| P6 | The test that pinned the interim contract (`TestLiveChildrenOutliveTheirStoppedParentUntilRecursiveStopLands`) | Removed, replaced by the termination tests. | No test pins a removal. The replacement contract is `TestDescendantsFinalizeBeforeTheirAncestors` and `TestDoneOfAParentImpliesDoneOfEveryDescendant`. |
| P7 | The race tests that held children live past the parent's `Done` | Adapted: a parent's `Done` now waits for its children, so they wait for the parent's turn instead. | `TestChildSpawnsRaceTheStopOfTheirParent` and `TestChildSpawnInFlightWhileItsParentStops` (`hierarchy_concurrent_internal_test.go`). |

## The root gate closer

The README limitation "root gate closer" and the tests that use it refer to one thing: the unexported method
`System.closeGate(n *node)` (`pekko-actor/actor/hierarchy.go`), called with the root node (`sys.closeGate(sys.root)`) by
`hierarchy_internal_test.go` and `hierarchy_concurrent_internal_test.go` to close the spawn gate of the whole system. Closing an
actor's own gate is what `acceptStop` does first. There is no public closer and no public `Shutdown`: a user cannot close the root
gate, and a system never terminates as a whole.
