# ADR 0014: Actor children and hierarchical termination

- Status: accepted design. Owner decisions D1-D7 are accepted (D1-D6 on 2026-10-10 after review of `eab4b7f`; D7 on
  2026-10-11). Accepted design is not an implementation and not parity: nothing in this ADR is implemented until its
  slices land, and no Pekko parity is claimed until the judge shows it.
- Amendment (a), unit U5b of #445, approved by the Owner with the U5 RULEBOOK (`981a6ea`): **the root owner is now an actor**.
  The root `node` described below stays as the name table and spawn gate of the root guardian's children (`/user`,
  `/system`), and every top-level actor is a child of `/user`. The text below is kept as the history of the decision
  before U5b; where it says the root owner is not an actor or that top-level `Spawn` uses a system root table, read
  the guardian structure of `migration/stage-1/supervision-u5/RULEBOOK.md`. Amendment (b) is **applied in U5c**: top-level
  names are validated like child names (Pekko `checkName`), so D4 below is replaced and `Spawn(sys, "")`, `"$x"`,
  `"a b"`, `"a/b"` and `"%zz"` now return `ErrInvalidName`.
- Date: 2026-10-10
- Stage: 1
- Module: pekko-actor
- Base: `origin/develop` at `193fa63` (includes ADR 0012, #38, #39 and Watch, #41). The local `develop` ref in the main
  checkout is stale (`b1863f1`); do not review against it.
- Number: reconfirmed on 2026-10-10. Ask is ADR 0015 (`docs/adr/0015-ask-and-reply.md`, branch
  `origin/docs/adr-0015-ask-and-reply`; renumbered from 0013, which is no longer used by Ask); it is not in
  `origin/develop`. 0014 exists only in this branch. Recheck the inventory (`git log --all -- 'docs/adr/001*'`)
  before merging; renumber on collision.

## Feature

Parent-owned children, name scoping per parent, and a subtree stop: when an actor stops, its children stop first and
the actor finalizes only after all of them. This is the mechanism that the supervision/restart slice needs; it does
not implement restart.

Supersedes divergence D1 ("intentional: no hierarchy", `docs/actor-core-parity.md`) and the waiver of S10 G5.c in
`judge/go-scope.json` once implemented. Until then both stay true.

## Evidence

Pekko `v1.7.1`, paths under `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/`:

- `dungeon/Children.scala:161-164` `reserveChild`; `:313-344` `makeChild`: the check `isTerminating` and the
  reservation are separate steps there; `:269-277` `checkName`; `:246-263` `removeChildAndGetStateChange`.
- `dungeon/ChildrenContainer.scala:205-211`: `reserve` in a terminating container throws `IllegalStateException`;
  `:208-209` a name still held by a child that is stopping is "not unique" (the name is free only after removal).
- `dungeon/FaultHandling.scala:180-212` `terminate`: unwatch, stop all children, suspend normal processing, and
  `finishTerminate` only when no children remain; `:247-276` `finishTerminate` order: `postStop`, detach, **notify the
  parent (`DeathWatchNotification`)**, then notify watchers; `:327-351` `handleChildTerminated` removes the child and,
  when the container's state changes, continues the suspended action.
- `ActorPath.scala:77-148`: valid element = ASCII letters/digits, `-_.*$+:@&=,!~';`, or `%XX`; empty and a leading `$`
  are invalid.
- `dungeon/Children.scala:133-145`: `context.stop(ref)` marks a child as dying, then stops any ref.
- `ActorSystem.scala:933-934`: top-level `actorOf` is `guardian.underlying.attachChild`, i.e. the same `makeChild` gate
  as any child (see "Root owner"). `ActorRefProvider.scala:59,64`: `guardian` and `systemGuardian` are `LocalActorRef`s.
- ADR 0010 (`Spawner`, `Context.Stop`, X1), ADR 0012, and Watch, merged in `origin/develop` at `193fa63` (#41; the former candidate branch
  `feat/actor-local-watch` is superseded).

Current code (`pekko-actor/actor/`): `Spawn(sys *System, ...)` (`spawn.go:36`); flat `System.names` with a `nil`
reservation (`registry.go:16-47`); `byID` never pruned at the time (`system.go:44`; ADR 0024 retires entries since); `finish` order is PostStop, take pending,
`closed`, release name, publish dead letters, `close(done)` (`stop.go`); `Tell` after an accepted stop dead-letters in
the caller (`ref.go:30-34`). No parent/child state exists.

Not measured: no benchmark, runtime test or judge result backs this ADR. GoAkt was not consulted: no evidence that it
solves anything better (project rule: Pekko wins).

## Deviations found in the original proposal (corrected below)

| # | Proposal said | Finding | Correction |
|---|---|---|---|
| 1 | Add the optional alias `Context.Spawn` | ADR 0010 already makes `*Context[T]` a `Spawner` and fixes `Spawn(Spawner, name, factory)`; the alias is a second creation path | Dropped. Only the function. |
| 2 | `Spawn` keeps its signature | Code has `Spawn(sys *System, ...)`; ADR 0010 and `signatures/api.go` say `Spawner`. Widening is source compatible for direct calls with a `*System` argument **only**. Checked on go1.27.2: a stored or assigned function value typed on the old signature, `var f func(*System, string, func() Actor[int]) (Ref[int], error) = Spawn[int]`, does not compile (`cannot use Spawn[int] (value of type func(Spawner, string, ...)) as func(*System, string, ...)`). Search recorded 2026-10-10 at `origin/develop` `44c1b62`, run from the repository root (modules present: `pekko-actor`, `pekko-observability`, `pekko-serialization`, `pekko-tenancy`; there is no `pekko-examples` module yet): (a) `rg -nP --glob '*.go' --glob '!upstream/**' --glob '!reference/**' --glob '!migration-kit/**' '\bSpawn\b(?![(\[])' .` found 17 matches, all on comment lines or inside string literals (test names, error text), none an identifier used as a value; (b) `rg -nP --glob '*.go' --glob '!upstream/**' --glob '!reference/**' --glob '!migration-kit/**' 'func\(\s*\*?(actor\.)?System\s*,\s*string' .` found no function type with the old first parameter. Direct calls (for example `judge_internal_test.go:217`) pass `*System`. Result: no non-direct use in this repository. External users are unknown | Source-breaking for function values, a contract change accepted in D1. The slice must re-run both commands on its exact base, including `pekko-examples` once it exists, and record the command, scope and result in its PR; a new match blocks the widening until resolved |
| 3 | Validate names with the exact Pekko element validator, only for children | Top-level names accept any string today; two rules in one namespace is incoherent | Open decision D4 |
| 4 | Misuse of an expired context publishes `ContextMisuse` (X1) | `Context[T]` is `struct{ c *cell }`; there is no per-call expiry in code | This slice enforces only the stop gate (`ErrParentStopping`). X1 conformance waits for the context rework; no claim here |
| 5 | Close the spawn gate "atomically with the stop state" under lock order hierarchy → cell | Stop state lives in `cell.mu`; atomicity across two locks is avoidable | Keep the gate in the hierarchy node under the hierarchy lock; `acceptStop` sets it first, then takes `cell.mu` |
| 6 | Hook `Goexit` in `PostStop` "needs a lifecycle fix" | `callPostStop` recovers only panics (`stop.go`): a Goexit there skips the rest of `finish`, so the name is never released and `Done` never closes. It is an existing defect, and with children it would hang the whole subtree | Prerequisite fix in the same slice, with a test |
| 7 | Parent finalization is triggered by the child's completion | Order versus `close(done)` unspecified | The child signals its parent last, after `close(done)`: `Done(parent)` closed implies every descendant's `Done` closed |
| 8 | Child name release before watcher notification | **Corrected after owner review.** An earlier draft claimed Pekko's watchers are told before the parent's bookkeeping. That is wrong on send order: `finishTerminate` sends the parent's `DeathWatchNotification` (`FaultHandling.scala:263-264`) before `tellWatchersWeDied()` (`:266`). Three things must be kept apart: (a) **send order** by the dying child: parent first, then watchers; (b) **processing by the parent**: asynchronous, in the parent's own system-message turn, where `watchedActorTerminated` enqueues `Terminated` to itself and then calls `handleChildTerminated`, which removes the child and frees the name (`DeathWatch.scala:109-114`); (c) **delivery to a non-parent watcher**: through that watcher's own queue. Nothing read orders (b) against (c) across two different actors, so no Pekko guarantee about "name free before a watcher sees `Terminated`" is established, and no divergence is claimed | goPekko's own contract (owner decision D2): the name is released before watchers are notified. Not presented as parity, not as divergence; the judge decides if a scenario ever observes it |
| 9 | Watch is reconciled "before implementation" | Pekko unwatches at the **start** of terminate (`FaultHandling.scala:185`). Watch as merged at `193fa63` withdrew registrations at the **end** of `finish` (`leaveWatchees`); the first increment of the hierarchy front moves it to the stop acceptance, so a stopping parent waiting for children could still receive runtime `Terminated` deliveries | Dependency, see below |
| 10 | Receive failures stop the actor | Consistent with develop, but differs from ADR 0010 D3 (default restart). Interim only | Stated; ends with the supervision slice |
| 11 | Per-mailbox FIFO dead letters | `stop.go` documents no order across sources | Keep the claim per mailbox only; no cross-level order |

## Decision (proposed)

### API

```go
// Existing in ADR 0010 / signatures/api.go: Spawn[T any](Spawner, string, func() Actor[T]) (Ref[T], error)
// Spawner is sealed; *System and *Context[T] implement it.
func (*Context[T]) Parent() (Handle, bool)
func (*Context[T]) Child(name string) (Handle, bool)
func (*Context[T]) Children() []Handle // detached snapshot, includes stopping children, no order
var ErrNameTaken, ErrParentStopping, ErrInvalidName error
```

Validated on go1.27.2 with `scripts/check-adr-signatures.sh` (9 `ok`, exit 0) through
[`signatures/hierarchy.go`](signatures/hierarchy.go): `Spawn` accepts a `*System` and a `*Context[Parent]`, infers a
child protocol different from the parent's, and the metadata methods build. This shows the signatures are well formed,
not any behavior. `ctx.Stop(h)` (ADR 0010) is needed by the slice for a parent to stop a child; it may stop any
handle, as Pekko's `context.stop` does (`Children.scala:133-145`). `Parent()` returns false for top-level actors.

### Identity and names

- Key = (owner incarnation ID, exact name). The system has an internal root owner that is not an actor (see "Root
  owner" below).
- Same name under the same parent fails with `ErrNameTaken`, including a reservation and a stopping child; the same
  name under different parents is allowed. A recreated name is a new ID; old refs keep targeting the old cell and
  dead-letter. Do not key by interface equality or by `Handle`.
- Logical path `/user/<parent>/<child>` is metadata, never a delivery identity, and it is **not guaranteed unique**:
  top-level names may contain `/` (D4), so `/user/a/b` can denote a top-level actor named `a/b` or a child `b` of
  `a`. Nothing may key, compare or route by it. No selection, no remote address.
- Child names: exact Pekko validator above, no trimming or decoding. Empty is invalid, not "anonymous".

### Root owner

Grounding. *Current runtime:* `System` had no shutdown (history: U10, #192/#193, added `Terminate`, `WhenTerminated` and `Shutdown`, see the note after D7). Its methods were `Stop(h)`, `Done(h)`, `SubscribeDeadLetters`
and unexported helpers (`registry.go`, `stop.go`, `system.go`); top-level `Spawn` only reserves a name in the flat
`names` map and could fail with `ErrNilActor`, a factory panic or `ErrNameTaken` (history: since U4, `ErrNilActor` and a factory panic are no longer errors of `Spawn`, see the note below). Nothing today can make a top-level
`Spawn` fail because the system is stopping. A `Shutdown` was listed in ADR 0010's milestone scope and was not
implemented at that time (it is since U10). *Upstream:* in Pekko the top-level `actorOf` is a child creation under the user guardian
(`ActorSystem.scala:933-934`, `guardian.underlying.attachChild`), so it passes the same `makeChild` gate: while the
guardian is terminating, creation throws `IllegalStateException("cannot create children while terminating or
terminated")` (`Children.scala:313-314`); a name held by a dying child is still "not unique"
(`ChildrenContainer.scala:205-211`). The guardian is a real actor; goPekko's root owner is not (this ADR), which is a
difference of mechanism, not of contract.

Decision proposed:

- The root owner is a hierarchy node with the same parts as any owner: a spawn gate, the reservation set and the
  outstanding-completion set. Top-level `Spawn` is the same creation transaction as a child `Spawn` (reserve under the
  hierarchy lock, commit, hand the first activation to the executor; since U4 the factory is not part of the
  transaction, see "Creation and the Spawn/Stop race"), with the owner being the root. Top-level `System.Stop(h)`
  uses the same subtree termination and the same completion signal to the root, so the root's set returns to baseline.
- **This slice adds no way to close the root gate** and no public `Shutdown`. The gate is closed only by a future
  system shutdown (its own ADR and slice; delivered later as U10) and, in tests, by an unexported seam. With the gate open, top-level behavior
  is exactly today's, including D4: any string is accepted as a name (empty, leading `$`, spaces, `/`), `ErrNameTaken`
  for a name held by a live or stopping top-level actor, and no `ErrInvalidName`.
- **When the root gate is closed:** `Spawn` through `*System` fails with the same error as a stopping parent,
  `ErrParentStopping`, before the factory runs and with no reservation, ID or event (decision D7, accepted). In this
  ADR "parent" includes the system's root owner (the counterpart of Pekko's user guardian); the error message is
  neutral ("owner is stopping"). A distinct `ErrSystemStopping`, and its relation to `ErrParentStopping` under
  `errors.Is`, were left to the shutdown ADR and are not decided here (U10 delivered them: see the note after D7).
- **Concurrent top-level `Spawn` against the close:** same ordering point as for a parent. The reservation is the
  linearization: gate closed first, the call fails before the factory; reservation first, the factory runs, the child
  commits already stopping, receives Stop, takes the paired ADR 0012 hooks and counts toward the root's completion set.
  There is no outcome in which `Spawn` succeeds and the actor is left outside the subtree.
- What shutdown waits for, in what order, and what it reports are not decided here.

This does not change D1-D6. D7 is accepted.

### Creation and the Spawn/Stop race

1. Validate owner and name. 2. Under the hierarchy lock: check the owner's gate, reserve with a unique token.
3. Commit the record and link before the first `Execute`; allocate the ID only here. 4. Hand the first activation to
the executor and return the `Ref`. **Changed by U4 (ADR 0017 S10, #444):** the factory is no longer run by `Spawn` and
there is no window in which a reservation is unresolved. The child's first activation runs the factory, outside every
lock, before `PreStart`; a nil actor, a panic or a `runtime.Goexit` is an `*InitializationError` decided by the
parent (default Stop), so the child terminates and its name is released then. Whether that activation runs on the
caller's goroutine depends on the executor: an inline one runs it inside `Spawn`.

If the gate closes before the reservation, `Spawn` fails with `ErrParentStopping` before anything is committed. If the
reservation wins, the child is committed; a stop accepted afterwards stops it through the owner's termination and it
takes the paired ADR 0012 hooks when it has an instance; it is never an orphan. Parent finalization waits for children
(a reservation now exists only between the reserve and the commit of one `Spawn` call). A factory that never returns
blocks that child's start and completion, and so its parent's finalization; with an inline executor it also blocks
`Spawn`. That is documented, not solved.

**Reservation to child (one critical section).** Under the hierarchy lock, in a single step, the reservation token is
replaced by the registered child record under the same (owner ID, name) key and the owner's pending count is adjusted
once. There is no instant at which neither the reservation nor the child is present, so a concurrent `Spawn` of the
same name always sees `ErrNameTaken` and a concurrent finalization check always sees a nonzero count. The step is valid
only if the stored token is still the caller's; otherwise nothing is committed, so a stale caller cannot overwrite a
newer entry. If the owner's gate is closed at that moment (the parent is stopping), the child is recorded in the owner's
wait set and a stop request for it is queued in the same step, to be dispatched after the lock is released; the child
takes the ADR 0012 paired hooks and its completion counts toward the parent. **Rollback** (nil, panic, Goexit or a
failed commit) deletes only the entry whose token is the caller's and, if the owner is waiting, re-evaluates
finalization. Finalization eligibility is one counter: outstanding completions plus reservations. It is the size of a set that holds
incarnation IDs (committed children) and reservation tokens, so "one decrement per incarnation" is membership removal
and is idempotent: a signal whose ID is not in the set (duplicate, stale, unknown, or from another owner's child) is
ignored and changes nothing. A child adds one when it is
committed; it is decremented only by that incarnation's post-`close(done)` completion signal, and a reservation only by
its commit-to-child step (which swaps one outstanding item for the other) or by its rollback. Detach alone, which
removes the record and the name before `close(done)`, never decrements it. Finalization is scheduled exactly once, when
the counter is zero and the gate is closed. That condition is evaluated at **two** points, never only on a transition:
(a) at the start step after the gate closed, once the child stops were dispatched, so an owner with no children and no
reservations (a leaf, or one whose children all completed before its stop) finalizes at once; and (b) after every
removal from the set (completion signal or rollback). Whichever point sees the condition first schedules; a guard
makes the second a no-op.

### Termination

Stop acceptance closes the gate and returns; it does not wait. After the in-flight call and the start step, the parent
requests the stop of every committed child through the executor (no recursion under a lock) and gives up its
activation. Children report completion through a private control path, not the mailbox. The zero check of "Reservation
to child" runs here too, so an owner with nothing outstanding finalizes without waiting for an event. The last
completion or reservation rollback schedules the parent's finalization exactly once. Required partial order: descendant
finalization before ancestor PostStop; PostStop never overlaps the in-flight Receive; pending messages dead-lettered
once per mailbox; name release and dead-letter publication before `close(done)`; a child signals its parent after
`close(done)`. Detach is one step: the name key and the child record leave the parent's registry together, at name
release, before `close(done)`; the later signal only counts the completion of that incarnation ID toward the parent's
finalization and, since ADR 0024, retires that ID from `System.byID` in the same step; it removes nothing else (a stale
or duplicate signal changes nothing). Sibling order unspecified. A blocked handler blocks completion and `Done` does not pretend otherwise.

Locks: hierarchy then cell, never the reverse; no factory, hook, executor call, stop propagation or subscriber under
either.

### Watch, supervision and retention

- Child bookkeeping is mandatory and independent of Watch. A parent that also watches a child sees the structural
  removal first. Pekko's comment at `FaultHandling.scala:249-252` requires `ChildTerminated` and `Terminated` to be
  *processed* in that order; `watchedActorTerminated` enqueues `Terminated` and removes the child in the same turn
  (`DeathWatch.scala:109-114`). Send order, processing and delivery are different things (deviation 8).
- Supervision seam only: parent identity and an internal failure channel. No restart here, and never Stop followed by
  Spawn: restart keeps ID, name reservation and `Ref`.
- `byID` retention was separate debt when this slice was written; it is decided by ADR 0024 (entries retire at
  completion, no tombstones, as lightweight tombstones also grow without bound) and implemented under #541. This slice
  itself guaranteed structural cleanup only.

## Dependencies

- **Watch** (merged in `origin/develop` at `193fa63`, #41). The hierarchy builds on it. Required changes in the
  hierarchy slice: withdraw the actor's **outgoing** registrations (what it watches) when the stop is accepted (Pekko
  `unwatchWatchedActors`), keeping the **incoming** watchers, who must still receive its termination; and order the
  structural child completion before `notifyWatchers`.
- **Context expiry (ADR 0010 X1)**: not required; this slice does not claim it.
- **Supervision/restart**: builds on this; it owns restart ordering against Stop.

## Minimum implementation scope (slice 2, not started)

Files under `pekko-actor/actor/` only: `Spawner` widening of `Spawn`, hierarchy registry replacing `names`, child
creation, `ctx.Stop`, subtree termination, `PostStop` Goexit fix, README, `docs/actor-core-parity.md` (D1), and
`judge/go-scope.json` (G5.c waiver removed **only** with judge evidence; this ADR retires nothing and declares no scenario covered). No new module, no empty `spi/`. Also in scope: the root owner node (gate, reservations, outstanding set) without a public closer or `Shutdown`; the unexported completion and gate seams the acceptance cases need; and the recorded `rg` search of deviation 2 re-run on the slice's exact base.

## Acceptance cases

| Case | Required observation |
|---|---|
| Parent and child with different protocols | Compiles; sends are typed |
| Two parents each spawn `worker` | Both succeed; the same name under one parent fails |
| Invalid child name (empty, leading `$`, bad `%XX`, `/`, space) | `ErrInvalidName`; no reservation, no ID, no factory; `%41` is accepted verbatim, no trimming or decoding |
| `Parent()`, `Child(name)`, `Children()` | `Parent()` false for top-level; `Children()` is a detached snapshot that includes stopping children and excludes reservations; `Child(name)` resolves a stopping child until its detach |
| `ctx.Stop` on a non-child, a sibling, or the parent itself | Accepted as for any handle. For a child that stops its own parent P: the child's `Done` closes before P's `PostStop` runs, P's `Done` closes exactly once, P's outstanding-set size returns to zero, and no completion is counted twice or for a stale incarnation. The same assertions hold when P then also stops that child. Stopping a sibling only affects the sibling's own subtree and completion |
| Factory nil, panic (U4) | `Spawn` returns the `Ref` and a nil error. The child's first activation reports an `*InitializationError` (cause `ErrNilActor`, or an error wrapping `ErrFactoryPanic`) with the event `factory-failed`; the parent's decider decides (default Stop; a top-level child has no decider until U5 and stops directly). The name is released when the child terminates, not at the failure; no `PreStart`, no `PostStop` |
| Factory ends its goroutine with Goexit (U4) | With the default executor `Spawn` still returns the `Ref` (only the activation's goroutine ends) and the child fails with `ErrGoexit`, default Stop; a Restart choice continues the activation on a fresh goroutine and loses no queued message. With an inline executor the Goexit ends the goroutine that called `Spawn`, which then does not return (the cell is still handed over and finishes with `ErrGoexit`) |
| Stop before reservation | `ErrParentStopping`; nothing committed, factory not invoked |
| Stop after the commit and before or during the child's factory (U4) | The child is stopped by the owner's termination; its start step runs once; if the factory fails no hook runs and the stop completes, and if it works `PreStart` then `PostStop` run; no orphan |
| Parent, child, grandchild | Descendants finalize before ancestor PostStop and `Done` |
| D3: `Done(parent)` closed implies `Done` closed for all its descendants | Parent, child, grandchild; the grandchild's finalization (its `PostStop`) is held on a channel. Stop the parent and assert, with channels only, that neither `Done(child)` nor `Done(parent)` is closed while the grandchild is held. Release it, wait for `Done(parent)`, and assert immediately after it is observed closed that `Done(child)` and `Done(grandchild)` are already closed. A mutation that signals the parent before `close(done)`, or finalizes the parent early, must fail this case |
| D2: a watcher recreates the name from its termination handler | Parent P owns child `worker` (ID1); a watcher W watches it. W's handling of the termination does not hold P's context (that would be context misuse, ADR 0010 X1): it sends P a message, and P, in its own `Receive`, spawns `worker` again and obtains ID2 different from ID1 without `ErrNameTaken`. The hook records `name-released(ID1)` and `notify-sent(W, ID1)`; the controlled executor holds W's delivery, so the recorded order (release first) is a guarantee of the code path and not a favorable race. Run it with delivery immediate and with delivery delayed; a mutation that notifies before releasing the name must fail the held-delivery run deterministically. This fixes goPekko's own contract (D2) and claims nothing about Pekko's cross-actor order (deviation 8) |
| Reservation to child, no gap | A concurrent `Spawn` of the same name while the first commits sees only `ErrNameTaken` (or success after detach); a finalization check never sees zero while a reservation or its child exists; a stale token cannot overwrite the replacement; rollback removes only its own entry |
| Reservation commits while the parent is stopping | The child is in the wait set and receives Stop; the parent's `PostStop` runs after the child's `Done` closes |
| A child handler held on a channel | Parent `Done` stays open; unrelated actors run |
| One-thread executor | Subtree stop completes; no activation blocked |
| Child stops and the name is respawned | Name busy until detach; new ID; old `Ref` dead-letters |
| Stop before first activation | ADR 0012 holds, with the U4 start step: the factory, then `PreStart`, run once, then `PostStop`. If the factory fails no instance exists, no hook runs and the stop completes. A `Spawn` from that `PreStart` returns `ErrParentStopping` and its factory is not invoked |
| Parent also watches the child | Structural removal precedes `Terminated` |
| Childless owner and drained owner finalize | (a) A leaf actor with no children and no reservations is stopped: `PostStop` runs, `Done` closes, with no completion signal ever arriving. (b) A parent whose only child completed and was delivered before the parent's stop (set already empty) is then stopped: same result. (c) A parent stopped while one child is outstanding finalizes only after that child's signal. A transition-only implementation (finalize only when a removal reaches zero) must fail (a) and (b) by never closing `Done` |
| Stale and duplicate child completion | Injection is deterministic: an unexported seam on the owner's completion path (nil in production, like `lockPoint`) lets the test capture a child's completion signal instead of delivering it, then deliver it later, twice, or after a respawn; the controlled executor sequences everything, no sleeps. An unexported accessor reads the owner's outstanding-set size. (1) *Duplicate:* parent P with children A and B, gate closed, size 2. Deliver A's signal: size 1. Deliver it again: size 1, P not finalized (`PostStop` not run, `Done(P)` open). Deliver B's: P finalizes exactly once. (2) *Stale after respawn, with P live (gate open):* P owns `worker` as A (ID1), size 1. `ctx.Stop(A)`; A finalizes and detaches, and its signal is captured, not delivered. Deliver it once: this is the legitimate first delivery, ID1 leaves the set, size 0, P stays live (the gate is open, so zero does not finalize it). Respawn `worker` under P as A' (ID2): succeeds, size 1. Replay ID1's signal: size stays 1, A' stays registered and findable by name, P live. Then stop P: the gate closes, A' receives Stop, and delivering A' signal takes the size to 0 and P finalizes exactly once and does not hang. (3) *Unknown or foreign:* a signal for an ID never issued, or for another owner's child, changes nothing. In every sub-case the assertions are: the injected stale, duplicate or foreign signal leaves the size unchanged (only the first legitimate delivery of an ID changes it), no early termination, and termination still happens when the real last signal arrives. A mutation that decrements by count instead of by ID must fail (1) and (2) |
| Top-level `Spawn` keeps its name contract (inverse of the child validator, D4) | Through `*System`: names `""`, `"$x"`, `"a b"`, `"a/b"` and `"%zz"` are accepted exactly as today and `ErrInvalidName` is never returned; the same name twice yields `ErrNameTaken`. The same strings through a `*Context` owner are rejected by the child validator. A mutation that runs the child validator for every `Spawner` must fail this case |
| Root gate closed (test seam; no public closer) | `Spawn` through `*System` returns `ErrParentStopping` (D7); a stopping parent's gate is tested separately; no reservation, no ID, no event, no factory. With the gate open, behavior equals today's |
| Top-level `Spawn` races the root close | Gate first: fails with `ErrParentStopping` before anything is committed. Reservation first: the actor commits (the factory has not run, U4), receives Stop, runs its start step once, takes the paired hooks when it has an instance, counts toward the root's set; no actor survives outside the subtree. Ordering is forced by holding the factory on a channel, not by timing |
| Repeated top-level create/stop | The root's outstanding set returns to baseline; `System.Stop(h)` on a top-level actor sends its completion signal to the root |
| Pending messages at every level | Each dead-lettered once, FIFO per mailbox |
| PostStop panic and Goexit | Cleanup completes, `Done` closes |
| Repeated create/stop | Registries return to baseline; `byID` growth reported separately (it returns to baseline too since ADR 0024) |

Use channels and the controlled executor, no sleeps. Judge: map to the inventory before assigning IDs (S10 G5.c is
the existing clause); add mutations for missing recursive stop, early parent finalization and path-based identity.
Pekko v1.7.1 is the oracle; Go tests alone do not establish parity.

## Decisions (owner)

Stated by the owner on 2026-10-10 and recorded as **accepted** after the native review of `eab4b7f` closed without
blocking findings (two informational findings; the detach order one is resolved in "Termination"). They accept the Go contract. They do not prove a divergence from Pekko
and do not authorize any parity claim.

| ID | Decision | Owner-stated |
|---|---|---|
| D1 | `Spawn(Spawner, name, factory)`, no `Context.Spawn` alias; function-value uses break (deviation 2) | Accepted |
| D2 | goPekko releases the name before notifying watchers (its own contract; Pekko's cross-actor order is not established, deviation 8) | Keep |
| D3 | Structural completion is signalled to the parent after `close(done)` | Accepted |
| D4 | Validate child names only; top-level names preserved | Replaced by amendment (b), U5c: top-level names are validated |
| D5 | `byID` retention and the `Done` contract for old handles | Separate ADR: [ADR 0024](0024-actor-registry-retention-after-termination.md) (decisions D1-D5 accepted 2026-10-20; implementation in progress, #541) |
| D6 | Fix `Goexit` in `PostStop`, with tests, inside the slice | Accepted |

Added 2026-10-11, owner-stated and accepted:

| ID | Decision | Owner-stated |
|---|---|---|
| D7 | A `Spawn` refused by its owner's gate returns `ErrParentStopping`, for a stopping parent and for the root owner alike; "parent" includes the root guardian; neutral message "owner is stopping"; both gates are tested without a public shutdown API. `ErrSystemStopping` and its `errors.Is` relation are for the shutdown ADR | Accepted; see the U10 note below |

**U10 note (#192, #193).** `ErrSystemStopping` now exists: a top-level `Spawn` refused because `/user` is stopping returns it, its text is `actor: system is stopping`, and `errors.Is(err, ErrParentStopping)` is true, so every check written for D7 keeps working; a `Spawn` from a `Context` still returns `ErrParentStopping`. `System` gained `Terminate()`, `WhenTerminated()` and `Shutdown(ctx)` (see `pekko-actor/README.md`, "Termination of the System"). Their contract, differences from Pekko and the open items are in `migration/stage-1/shutdown-u10/RULEBOOK.md` and its execution record. No behavior of D1 to D6 changed.

Basis: Pekko routes the top-level `actorOf` through the guardian's `makeChild`, so one condition covers both (`Children.scala:313-314`, `ActorSystem.scala:933-934`); this slice cannot produce a user-visible system shutdown.

## Consequences

Hierarchy appears in the public contract; divergence D1 of `actor-core-parity.md` and the G5.c waiver are retired
only when implemented and backed by judge evidence; `Spawn` widens; the
README gains a children section. Delivery order: this ADR, then slice 2 (hierarchy runtime with its tests), then judge
coverage and Watch/restart reconciliation. Benchmarks and pekko-examples stay at the end of the migration plan.
