# RULEBOOK: Supervision U5, guardians as actors and supervision of top-level actors (#445, unit supervision-u5)

Status: **PROPOSED, for the Owner's approval. This PR changes no code, no test and no other document.** Nothing in `pekko-actor/` or in the ADRs is modified until the Owner approves this file.
Source of truth: `docs/adr/0017-supervision.md` (S1, H1(a) decided by the Owner on 2026-10-14, D-4, the U5 row, the S1 test rows) and Apache Pekko Classic v1.7.1 (`upstream/pekko`, read-only, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, the tag `v1.7.1`). Issue #445.
Base: `develop` at `eaf9cefc38bf84bf3be090abd221c4a31b543577` (U4 merged; #444 closed; test follow-ups #459, #460, grouped with the module close in #461).

## 0. What U5 is, and what it is not

U5 replaces the **interim** "a top-level actor has no parent, so a failure stops it" with the Classic structure: the root guardian, the `/user` guardian and the `/system` guardian are **actors** with their strategies, top-level `Spawn` is child creation under `/user`, and the Watch chain `/user → /system → root` is kept (H1(a): an internal supervisor that omits observable guardian behavior is not an accepted substitute). Today the root is a `node` with no cell (`system.go:70`, `hierarchy.go:18`), a top-level cell has `parent == nil` (`spawn.go:63`) and `failUp` refuses it (`supervision_failure.go:219`).

It is **not** `System.Shutdown`, `whenTerminated`, the termination hooks of the system guardian, `Kill`, a custom guardian `Props`, or the logging of U9. Those are gaps declared in section 7.

## 1. Approvals this repository expressly requires (and nothing more)

| # | Requirement | Where it is written |
|---|---|---|
| G1 | The Owner approves this RULEBOOK before anything in `pekko-actor/` or the ADRs is modified. | `0017` S1 ("ADR 0014 is not edited before" the unit lands), `0017:615`, and the practice of U3 and U4 |
| G2 | **Two amendments of ADR 0014 need the Owner's approval when the RULEBOOK is presented:** (a) "the root owner is not an actor" (`0014:90,110`), and (b) its D4, "top-level names accept any string" (`0014:59,96,122,243,266`), because Pekko validates top-level names with `checkName`. | `#445`, `0017` U5 row (`:685`) and `:615-616` |
| G3 | The unit runs through the Anthropic migration kit `cf91c9d`, with exact upstream sources and tests, and is recorded. | `#445` acceptance |
| G4 | Until the unit lands, top-level actors keep the stop-on-failure divergence, and the end of the system (`/user` failure or escalation, `whenTerminated`, the termination hooks) is a declared gap until `System.Shutdown` (#67/#192/#193). The upstream tests `ActorSystemSpec:328,359` stay `PORT-GAP`. | `#445` acceptance |
| G5 | Per-unit proportional checks; the exhaustive validation (mutants, judge campaign, complete `-race` repeat, integral review) at the module close. | `docs/quality-gates.md`, "Per unit and at module close"; policy of 2026-10-15 |

Not an approval gate of this repository, and not added by this document: a native review needs the Owner's consent per envelope under the review switch (not asked here); and the hashes of the execution package (RUNBOOK, permissions, launcher, prompt) are the kit's sign-off at the point of execution (kit standing rule 3; the launcher refuses a RULEBOOK whose hash was not approved), as in U3 and U4. They are prepared **after** this file is approved, because they bind to its hash and to the allowed surfaces it fixes (section 8).

## 2. Decisions this RULEBOOK asks the Owner to confirm by approving it

| # | Decision | Recommendation and why it needs the Owner |
|---|---|---|
| D1 | **Three guardian cells**, created by `NewSystem`: the root guardian (which reports itself as its own parent, `getParent = this`, but whose own failure has no actor above it: as Pekko's fake parent does at `ActorRefProvider.scala:476-486`, it is stopped directly and the system ends, so the `parent == nil` guard of `failUp` stays for this one cell), `/user` and `/system`. Each is a real cell with a mailbox and an activation. The guardians' message type is an unexported `guardianMsg`; a guardian handles `Terminated` by stopping itself and keeps its children across a restart (a no-op `PreRestart`, `ActorRefProvider.scala:324-325,372-373`). | Recommended. It is the H1(a) decision made concrete; the cost is in the tests (D8). |
| D2 | **Strategies.** Root guardian: `Stop` for any failure (`:518-522`), the log being a gap until U9. `/system`: `DefaultStrategy` (`:532`). `/user`: `DefaultStrategy` unless configured (`:527`, `reference.conf:123`). | The configuration is public API. See D3. |
| D3 | **Guardian strategy configuration: closed, exact public API in section 3.5.** `Config.GuardianStrategy` of the exported struct type `GuardianStrategy`; built by `GuardianOneForOne(Budget, GuardianDecider)` and `StoppingGuardianStrategy()`; the zero value is `DefaultStrategy`. `GuardianDecider` is `func(child Handle, cause error) (Directive, bool)`. **No exported signature mentions `guardianMsg`**, and the signatures compile from an external package (proved by an example in `package actor_test`). | New exported identifiers: `Config.GuardianStrategy`, `GuardianStrategy`, `GuardianDecider`, `GuardianOneForOne`, `StoppingGuardianStrategy`. Needed by `ActorSystemSpec:336,359`. |
| D4 | **Top-level `Spawn` is child creation under `/user`.** `Spawn(sys, …)` keeps its signature and its three synchronous errors. The top-level cell's `parent` is the `/user` handle, so a top-level failure is reported to `/user`'s decider (default: Restart for an ordinary error, unlimited; Stop for an `*InitializationError`, `ErrDeathPact` and `ErrGoexit`). A top-level child of a stopping `/user` gets `ErrParentStopping` (`dungeon/Children.scala:313-314`). **Delivered in U5b** (U5a leaves top-level actors in `s.root`, section 3.3.2). | Observable behavior change (section 4). |
| D5 | **`Context.Parent()` of a top-level actor returns the `/user` handle** (true), as `context.parent` does in Pekko. It is the same kind of handle `Parent()` returns today (an `idHandle`: it has an `ID()` and **no `Tell`**), but **it is not inert**: every operation that resolves a handle by its ID accepts it, including `Stop`, which ends the whole system. The exact consequences are closed in D11 and section 3.6. | Observable API change: `Parent()` was `false` for top-level (`hierarchy.go:368-375`, ADR 0014 table). Alternative: keep `(nil,false)` and declare it a divergence. The Owner chose to return the handle. |
| D6 | **ADR 0014 amendment (a) (G2):** the root owner **is** an actor: the `node` stays the name table and spawn gate of the root guardian's children (`/user`, `/system`), and every top-level actor is a child of `/user`'s node. The root `node` is no longer "not an actor". | Needs approval, see G2. |
| D7 | **ADR 0014 amendment (b) (G2):** **top-level names are validated like child names** (Pekko `checkName`, `dungeon/Children.scala:64-65,270-275`, `ActorPath.validatePathElement`). `Spawn(sys, "", …)`, `"$x"`, `"a b"`, `"a/b"` and `"%zz"` become `ErrInvalidName`. | **Breaking change for callers that use such names.** Needs approval. Consequence: ADR 0014 D4 is replaced; `TestTopLevelNamesAreNotValidated` is rewritten. |
| D8 | **Start-up, IDs and test seams: closed in section 3.4.** `NewSystem` builds, registers, configures and wires all three guardians **before** it hands any activation to the executor, so no guardian ever observes a partial hierarchy, with a deferred or an inline executor. The guardians take the first three IDs (root, `/user`, `/system`). Controlled executors in the tests see three extra activations; the baselines of the tests that read `sys.root` move **in the delivery that moves the root** (section 9). | Cost accepted by H1(a); stated so that nobody is surprised. |
| D9 | **The end of the system is a declared gap until `System.Shutdown`.** When `/user` terminates (a failure that the root guardian stops, or a decider that escalates), the Watch chain stops `/system` and then the root guardian, all top-level actors stop recursively, and `Spawn` through the system fails with `ErrParentStopping`. **The last two effects (top-level actors stop, `Spawn` fails) exist only from U5b**, when top-level actors become children of `/user`; U5a delivers the guardian chain alone (section 3.3.1). What is **not** delivered: `whenTerminated`, a termination future that fails with the cause, `Shutdown`, the termination hooks of `SystemGuardian` (`RegisterTerminationHook`), `Kill` on `/user`. | Matches G4. The chain itself is observable in Go through the `Done` of the top-level actors and the `Spawn` error, so Go-side tests can prove it. |
| D10 | **`System.Stop(h)` of a top-level actor stays direct.** Pekko routes it as `StopChild` to the guardian, which does `context.stop(child)`; the observable result is the same, so no `StopChild` message is added. | Minor; confirmed by approval. |
| D11 | **The `/user` handle from `Parent()` accepts `Stop`, `Watch`, `WatchWith`, `Unwatch` and `System.Done`, and not `Tell` or `Ask`: closed in section 3.6.** Stopping it is the only way, until `System.Shutdown` exists, to end the system from user code. It is allowed (Pekko allows it: `context.stop(parent)`, and `ActorSystem.stop` of a non-child actor is a direct `stop()`, `ActorSystem.scala:945-947`). | **Recommended: allow.** Alternative: make `Stop` of a guardian a no-op until `Shutdown`. Needs the Owner's choice because it is a de facto shutdown. |
| D12 | **Delivery in three coherent parts (U5a, U5b, U5c), each compilable and verifiable, with the tests it breaks adapted in the same part: closed in section 9.** The RULEBOOK is approved once; each part has its own execution package and PR. | Replaces the single-unit assumption of the first version. The per-delivery maximum (560,000) is unchanged; the **aggregate** of up to 1,680,000 for three deliveries is a proposal and is **not authorized** (section 9). |

## 3. Behavior specification

### 3.1 The guardians
| Guardian | Parent | Strategy | Behavior |
|---|---|---|---|
| root | none above it (reports itself as parent) | Stop for any failure of its children; its own failure stops it directly | `Terminated(_)` of what it watches stops it; keeps its children if it restarts |
| `/user` | root | `DefaultStrategy` or `Config.GuardianStrategy` | children are the top-level actors; keeps its children if it restarts |
| `/system` | root | `DefaultStrategy` | watches `/user`; when `/user` terminates, `/system` stops itself (the termination hooks are a gap) |

Wiring, as in `init` (`ActorRefProvider.scala:613-620`): `/system` watches `/user`, the root guardian watches `/system`. A failure of `/user` or `/system` is reported to the root guardian, whose strategy stops the failed guardian (`:518-522`). A failure of the root guardian itself is handled, in Pekko, by a fake parent that is not an actor (`:476-486`: the failed guardian is logged, the termination future fails with the cause and the guardian is stopped); in Go the root guardian stops directly, the log and the future being gaps (D9).

### 3.2 Failures of a top-level actor (all go to `/user`'s decider)
| Cause | Decision by default | Evidence |
|---|---|---|
| error, panic or Goexit in `Receive` | Restart (Goexit: Stop), unlimited budget | `FaultHandling.scala:222`; U3 `defaultDecider` |
| nil actor, panic or Goexit of the factory; failure of `PreStart` | `*InitializationError`: Stop | `ActorCell.scala:642-683`; U4 |
| failure of the new instance after a restart | Stop (`InRestart` true) | U2 |
| a top-level supervisor whose child escalates | the supervisor fails with that cause: Restart by default | `FaultHandlingDocSpec:139` (Escalate step) |
| death pact (`Terminated` unhandled) | Stop | `Actor.scala:658`; U3 |
| `/user`'s own strategy chooses Escalate | `/user` fails; the root guardian Stops it; the chain ends the system (D9) | `ActorSystemSpec:359` |

A top-level restart replaces the instance and keeps the `Ref`, the name, the mailbox and the watchers, with `PreRestart`/`PostRestart` and the finite or unlimited `Budget` of the strategy, as U2 and U3 already do for any child. U5 adds no restart logic: it gives top-level actors a parent.

### 3.3 Termination and DeathWatch
Stopping a top-level actor keeps today's contract (its subtree finishes first, `Done` closes, the name is released, watchers get one `Terminated`). The new part is the chain above `/user`. `Watch` of a top-level actor by another is unchanged. Only the `/user` handle is exposed (through `Parent()`, D5); `/system` and the root guardian have no public handle. The chain, in Go terms: `/user` terminates, then `/system` (which watches it) stops itself, then the root guardian (which watches `/system`) stops itself; each `Done` closes exactly once.

### 3.3.1 The complete termination sequences (corrects the first version, which said that stopping `/system` leaves `/user` alive)
Termination is recursive: the root guardian is the parent of `/user` and `/system`, so **whenever the root guardian terminates, `/user` terminates too**, and `/user` terminates its own children first. There are three entry points; the sequence, the order of the `Done` channels and the notifications are fixed for each. The stops are idempotent (`acceptStop` accepts once, one `stop-accepted` event per guardian), and a `Terminated` is delivered only to a watcher that is still alive when the subject finishes.

| Entry point | Sequence | Order in which the `Done` channels close | Watch notifications |
|---|---|---|---|
| **Stop `/user`** (a request, an escalation, or a failure of `/user` that the root guardian stops) | 1. `/user` closes its gate, stops its children (from U5b: the top-level actors, each after its own subtree) and finalizes. 2. `/system` gets `Terminated(/user)` and stops itself. 3. The root guardian gets `Terminated(/system)` and stops itself; its children `/user` and `/system` are already finished, so it finalizes at once. | `/user`, then `/system`, then the root guardian | `/system` gets one `Terminated` for `/user`; the root guardian gets one for `/system` |
| **Stop `/system`** | 1. `/system` has no children and finalizes. 2. The root guardian gets `Terminated(/system)` and stops itself. 3. Its termination step asks its children to stop: **`/user` is still alive, so it is stopped here** (from U5b it stops the top-level actors first), and the root guardian waits for it. 4. `/user` finalizes, the root guardian finalizes. `/system`'s watch on `/user` is not delivered: `/system` had finished before `/user` did. | `/system`, then `/user`, then the root guardian | the root guardian gets one `Terminated` for `/system`; none for `/user` (nobody alive watches it) |
| **Stop the root guardian** | 1. The root guardian closes its gate and asks `/user` and `/system` to stop, then waits for both. 2. `/user` stops its children and finalizes; `/system` finalizes (and, if `/user` finishes first, its watch delivers `Terminated(/user)` to it, which stops it again: idempotent). 3. The root guardian finalizes. | `/user` and `/system` in either order, then the root guardian last | `/system` gets at most one `Terminated` for `/user`; the root guardian gets one for `/system` if it was still alive |

The invariants of all three: every `Done` closes exactly once, the root guardian's `Done` is the last to close, no guardian stops before its children have finished, nothing waits forever, and there is no second `PostStop`. They are asserted by T13 for every entry point and every start-up boundary. **Amendment (U5a, accepted by the Owner):** with an inline executor the order of the `Done` channels for "Stop `/system`" is `/user`, `/system`, root guardian, because the existing contract notifies watchers before it closes `Done`; the root guardian is still last and every `Done` closes exactly once.

### 3.3.2 What U5a delivers and what waits for U5b (the intermediate behavior)
In U5a the top-level actors still live in the legacy table `s.root` and have no parent, **outside `/user`**. So in U5a:
- Stopping `/user` (reached only through an unexported accessor of the tests, since no public handle exists yet) ends the guardian chain of the table above, **but does not stop any top-level actor and does not close the `Spawn` gate**: `Spawn(sys, …)` keeps working and returns a `Ref`; it does **not** return `ErrParentStopping`.
- A top-level actor failure still stops that actor directly (the interim of U3 and U4).
- `Parent()` of a top-level actor is still `(nil, false)`.

The guarantees "stopping `/user` terminates the top-level actors and `Spawn(sys)` returns `ErrParentStopping`", and the supervision of top-level actors by `/user`, are **delivered and tested in U5b**, together with the re-parenting that makes them true. U5a asserts the intermediate behavior stated here, so that the later change is visible in the tests that it breaks.

### 3.4 Start-up of the guardians (closes the second point of the review)
`NewSystem` runs these steps **in this order, on the calling goroutine, and hands no activation to the executor before step 5 is complete**. A guardian is built exactly as `Spawn` builds any cell: with `scheduled` true, so nothing can run until its first activation is handed over, and with its actor created by a retained factory (the same path as any other cell, so U4's start step applies).

1. **Construct** the three cells (root guardian, `/user`, `/system`) and their nodes. Nothing is registered and nothing runs.
2. **Register** them in the registry, in this order, so the IDs are deterministic: root guardian, `/user`, `/system`. `/user` and `/system` are committed in the root guardian's node (names `user` and `system`), through the existing reserve and commit, so the outstanding set of the root guardian counts them. The root guardian is not committed under any owner.
3. **Install the strategies** before any activation: root guardian, `Stop` for any failure of its children; `/user`, the strategy of the **unexported seam `options.userStrategy Strategy[guardianMsg]`** if set (nil in production, introduced in U5a) and otherwise `DefaultStrategy`; from U5c `Config.GuardianStrategy` fills that same seam (so the public API adds no second path); `/system`, `DefaultStrategy`. A guardian's strategy is a field of its cell, set here and never written afterwards, so no activation can race with it.
4. **Link the Watch chain** by registering the watches directly, under the same lock and with the same registration code as `Context.Watch`: `/system` watches `/user`, the root guardian watches `/system`. (`cell.register` takes the cell lock and the registry lock and needs no running activation; the public comment that `Context.Watch` is valid only inside a hook describes the user-facing method, and the start-up is the one internal caller outside a hook. The calls cannot hit the invalid-handle or conflict panics: the subjects are the guardians just registered.) This is done before any activation, so a termination that happens later always finds the links, and a guardian that terminates before another one has started is delivered as a runtime item in the lane of the watcher, behind its start step.
5. **Mark the system ready** (the last write of construction), then hand the first activations to the executor in this order: root guardian, `/user`, `/system`.

Consequences, both required and tested:
- **Inline executor.** The first activation of the root guardian runs inside step 5, when the hierarchy is complete: it sees `/user` and `/system` committed, the strategies installed and the links wired. The activations of `/user` and `/system` follow in order. No guardian observes a hierarchy in which another guardian is missing, unregistered, without a strategy or unlinked.
- **Deferred executor.** `NewSystem` returns with three activations queued. A `Stop` of a guardian, or a `Spawn` of a top-level actor, may arrive before any guardian activation has run. They are safe because the cells exist from step 2 and the pending activations are `scheduled`: a stop of a guardian before its first activation is the ADR 0012 case (the start step runs once, then the termination). In U5a a top-level `Spawn` is the unchanged legacy one (section 3.3.2); from U5b a `Failed` for `/user` is also queued and decided after its start step.
- **Immediate close during start-up.** A `Stop` of any guardian at any point after step 2 ends in the matching sequence of section 3.3.1, with every `Done` closed exactly once, no deadlock and no duplicated `Terminated`. Before step 2 completes no handle exists, so it cannot be requested. `Spawn(sys, …)` returning `ErrParentStopping` after the chain ended is **only from U5b** (section 3.3.2). The unexported seam `options.bootPoint func(step int)` (nil in production, like `lockPoint`) is called after each of steps 1 to 4 and between the three activations of step 5, with the system, so a test can stop a guardian at every boundary.

### 3.5 Public API of the guardian strategy (D3, exact)
All of this is the **complete** public addition of U5c. It compiles from an external package and names no private type.

```go
// Config configures a System.
type Config struct {
	// Throughput is the number of steps one activation may take before it yields; a value below 1 means 1.
	Throughput int
	// GuardianStrategy decides what happens to a top-level actor that fails. The zero value is
	// DefaultStrategy (restart without limit, stop on an initialization failure, a death pact or a Goexit).
	GuardianStrategy GuardianStrategy
}

// GuardianStrategy is the supervisor strategy of the /user guardian. It is built by GuardianOneForOne or
// StoppingGuardianStrategy; a user cannot implement one. The zero value is DefaultStrategy.
type GuardianStrategy struct {
	kind   uint8 // 0 is the default
	budget Budget
	decide GuardianDecider
}

// GuardianDecider maps the cause of a top-level actor's failure to a directive. child is the identity of the
// failing top-level actor. ok == false means "undefined for this cause" and is Escalate, whatever directive is
// returned. A nil decider is undefined for every cause.
type GuardianDecider func(child Handle, cause error) (d Directive, ok bool)

// GuardianOneForOne decides for the failing top-level actor alone, within budget.
func GuardianOneForOne(budget Budget, decide GuardianDecider) GuardianStrategy

// StoppingGuardianStrategy stops any failing top-level actor (Pekko's StoppingSupervisorStrategy).
func StoppingGuardianStrategy() GuardianStrategy
```

**What the decider receives and may do.** It receives the failing child's `Handle` and the cause (the same causes as any decider: `*PanicError`, `*InitializationError` with `ErrNilActor`/`ErrFactoryPanic`/`ErrGoexit`, `ErrDeathPact`, or the error returned by `Receive`). It receives **no `Context` and no `System`**, because the guardian's context belongs to the guardian: user code must not Spawn, Stop or Watch from inside the guardian's decider. Its only operation is to return a directive: `Resume`, `Restart`, `Stop` or `Escalate`. This is a documented limitation, not a hidden one: the context capabilities of a decider are the gap #450 item 4. The `Handle` may be compared by `ID()`.

**Adaptation to the internal strategy.** `GuardianOneForOne(b, f)` is stored as `GuardianStrategy{kind: 1, budget: b, decide: f}`. At step 3 of the start-up it becomes the internal `Strategy[guardianMsg]` that `/user`'s cell uses: `OneForOne(b, func(_ *Context[guardianMsg], child Handle, cause error) (Directive, bool) { return f(child, cause) })`, with the same `Budget` semantics, the same conversions and the same panic rule as every decider (a panic in `f` is a failure of `/user`, which the root guardian stops: the system ends, D9). `StoppingGuardianStrategy()` is `GuardianOneForOne(Budget{}, func(Handle, error) (Directive, bool) { return Stop, true })`. The zero value takes the `DefaultStrategy[guardianMsg]()` path. `Escalate` makes `/user` fail with that cause; the root guardian stops it and the chain ends the system (`ActorSystemSpec:359`).

**Proof that it is usable from outside.** An `Example` in `package actor_test` builds a system with `Config{GuardianStrategy: actor.GuardianOneForOne(actor.Budget{MaxRetries: 3, Within: time.Minute}, func(child actor.Handle, cause error) (actor.Directive, bool) { ... })}` and with `actor.StoppingGuardianStrategy()`; if any signature needed a private type this file would not compile.

### 3.6 The `/user` handle obtained by `Parent()` (D5 and D11, exact)
**Available from U5b.** Until then `Parent()` of a top-level actor is still `(nil, false)` and no public handle to `/user` exists (section 3.3.2); everything below, including the effect of `Stop(h)` on the top-level actors and on `Spawn`, is delivered and tested in U5b. A top-level actor `w` calls `ctx.Parent()` and gets `(h, true)`, where `h` is an `idHandle` for `/user`: it has `ID()` and no `Tell`. Every existing operation that resolves a handle by its ID accepts it:

| Operation | Effect |
|---|---|
| `ctx.Stop(h)`, `sys.Stop(h)` | Accepts the stop of `/user`. `/user` closes its gate (new `Spawn` through the system returns `ErrParentStopping`), asks every top-level actor to stop (each stops its own subtree first), waits for them, runs its termination, and the chain of section 3.3 follows: `/system`, then the root guardian. **This ends the system** and cannot be undone. It is the only way to end the system from user code until `System.Shutdown` (#67, #193) exists, and the `Done` of every actor closes. There is no `whenTerminated`. The caller `w` is itself a top-level actor, so it is stopped as part of it. |
| `sys.Done(h)` | Returns the channel of `/user`; it closes when `/user` finishes, after every top-level actor. |
| `ctx.Watch(h)`, `ctx.WatchWith(h, msg)` | Accepted. Because every possible watcher is a descendant of `/user` and finishes before it, **no `Terminated` or `WatchWith` message of `/user` can reach a user actor**; the registration is accepted and has no observable delivery. This is stated and tested, not hidden. |
| `ctx.Unwatch(h)` | Accepted, a no-op when there is no registration. |
| `Tell`, `Ask` | Not available: `h` is not a `Ref`. |

Validity: the handle stays valid after the system ended; operations on it are no-ops, like for any finished actor.

## 4. Observable changes with respect to the current (interim) behavior
This table is the **end state of the three deliveries**, not what each part delivers. By part: **U5a changes nothing observable** except three extra IDs and three extra activations (section 3.3.2); **U5b** delivers every row about top-level failures, `Parent()`, the IDs and baselines under `/user`, and `Spawn` after the chain ended; **U5c** delivers the validated top-level names and the public guardian configuration.

| Case | Today | After U5 |
|---|---|---|
| top-level `Receive` returns an error | the actor stops (`stop-accepted`) | it is restarted without limit; no `Terminated` |
| top-level factory fails (U4 T7) | the child stops directly | `*InitializationError` decided by `/user`: Stop (same end result) |
| a top-level supervisor with a child that escalates | stops directly | it fails and `/user` restarts it |
| `Context.Parent()` of a top-level actor | `(nil, false)` | `(/user handle, true)` (D5) |
| top-level names `""`, `"$x"`, `"a b"`, `"a/b"`, `"%zz"` | accepted | `ErrInvalidName` (D7) |
| `Spawn` after the guardian chain ended | not reachable | `ErrParentStopping` (D9) |
| IDs and outstanding-set baselines | root has no IDs | three guardian IDs; top-level actors count under `/user` (D8) |

## 5. Reconciliation (code, tests, documentation); locations verified against `eaf9cef`
| # | Artifact | Where | Today | After U5 | Test or check |
|---|---|---|---|---|---|
| 1 | root and system construction | `system.go:57-105` (`root *node`, `newSystem`) | root `node`, no cell | three guardian cells created and wired | new guardian tests |
| 2 | spawn target of `*System` | `hierarchy.go:18` | `owner: s.root`, no parent | `owner:` `/user`'s node, `parent:` the `/user` handle | top-level `Spawn` tests |
| 3 | `Parent()` | `hierarchy.go:368-375` | false for top-level | true with the `/user` handle (D5) | rewritten test |
| 4 | `failUp` interim | `supervision_failure.go:205-219` | `c.parent == nil` refuses | a top-level cell has a parent; the guard stays for the root guardian | T11 and V14 rewritten |
| 5 | name validation | `spawn.go:53` (`t.parent != nil &&`) | validator only for children | validator for every owner (D7) | `names_test.go` |
| 6 | `Config` | `system.go:50-54` | `Throughput` only | `+ GuardianStrategy` (D3) | `ActorSystemSpec:336` adapted |
| 7 | tests that pin the interim stop | `supervision_default_internal_test.go:336` (T11), `supervision_goside_internal_test.go:431-444` (V14), `supervision_default_goside_internal_test.go:167` (T9), `spawn_supervised_goside_internal_test.go:394` (T7), `spawn_errors_test.go:159` | top-level failure stops directly | restart (or Stop for initialization failures) under `/user` | each rewritten, the purpose kept and the reason commented |
| 8 | tests on the root node | `termination_internal_test.go` (44 refs), `hierarchy_internal_test.go` (26), `completion_internal_test.go` (26), `hierarchy_concurrent_internal_test.go` (8), `names_test.go` (3), `termination_stress_internal_test.go`, `ask_firstreply_internal_test.go` | read `sys.root`, `registered()`, `outstanding(root)` | baselines move to `/user`'s node and +3 IDs; `TestTopLevelNamesAreNotValidated` replaced | adapted, assertions kept in purpose |
| 9 | ADR 0014 | `:59,90,96,110,115,122,243,266` | root not an actor; top-level names unvalidated (D4) | amended per G2, with the history kept | review |
| 10 | ADR 0017 | `:123-147`, `:625`, `:633-636`, `:685` | S1 "F, delivered in U5"; D-4 | S1 delivered; D-4 closed; gaps of D9 kept | review |
| 11 | module README, `doc.go` | `pekko-actor/README.md` (hierarchy and supervision sections), `actor/doc.go` | "a top-level actor stops directly"; `PORT-GAP(top-level-default-U5)` | guardians, `Parent()`, validated top-level names, the gap of D9 | `rg` for the old phrases |
| 12 | parity and first-cut docs | `docs/actor-core-parity.md:66,67,194,217`, `docs/actor-core-first-cut.md` | top-level interim | restated; no parity claimed | review |
| 13 | judge anchors | `judge/mutate_go.py`: `hierarchy.go` (the line ` \|\| len(n.out)+n.reserved != 0 {`, used by two mutants), `ref.go`, `stop.go`, `cell.go`, `mailbox.go` | 22 pairs, each once | **unchanged**; U5 edits `hierarchy.go`, `system.go`, `spawn.go`, `cell.go`: if an anchor must move, the work stops and reports (`judge/` is never edited) | anchor check after the run |

## 6. Upstream sources and test plan (equivalence and Go adaptation kept apart)
Deterministic tests only: channels and a controlled executor where the order matters; no sleeps as coordination; Goexit tests on the real executor.

| Id | Upstream source | Go test |
|---|---|---|
| P1 | `SupervisorMiscSpec.scala:87` ("create named children in its constructor": a top-level actor throws, `preStart` runs twice, `isTerminated` false) | ADAPTED: a top-level actor fails in `Receive`; it is restarted, `PreStart` runs twice, `Done` stays open. The constructor creating `bob` is dropped (`PORT-GAP(factory-context)`, as in U4); a `PreStart` that spawns it is the Go-side variant |
| P2 | `ActorLifeCycleSpec.scala:142` (a top-level default restart; uses `become`) | ADAPTED without `become` (`PORT-GAP(become)`): restart counts of hooks and state reset |
| P3 | `ActorSystemSpec.scala:336` (a configurable guardian strategy: with the stopping strategy a top-level actor that throws is stopped and its watcher gets `Terminated`) | ADAPTED: `Config.GuardianStrategy` set to the stopping strategy; the watcher gets one `Terminated` |
| P4 | `SupervisorSpec.scala:460` (a top-level parent restarted by the guardian, a throwing decider) | ADAPTED where it does not need the mailbox configuration |
| P5 | `SupervisorSpec.scala:510` (a top-level creation failure is stopped; the test fails through `error-mailbox`) | `PORT-GAP(mailbox-config)` for the test; a Go-side test (T2) covers the top-level creation failure |
| P6 | `docs/src/test/scala/docs/actor/FaultHandlingDocSpec.scala:139` (Escalate needs a top-level supervisor restarted by the guardian) | the Escalate step ADAPTED (`PORT-GAP(escalate-kill-U5)` of U2 closes for the restart part; a kill is a gap) |
| P7 | `ActorSystemSpec.scala:328,359` (end of the system on `/user` failure or escalation, `whenTerminated`) | `PORT-GAP(system-shutdown)` until #67/#193, as the issue requires; the chain is covered Go-side by T5 and T6 |
| T1 | S1 | the three guardians exist, are wired by Watch, and have the strategies of D2 (observed through events and `Done`, not through new public API) |
| T2 | S1, U4 | a top-level factory failure: `*InitializationError`, decider of `/user`, Stop, name released at termination |
| T3 | S1 | a top-level `Receive` failure restarts it unlimited: 50 failures never stop it (the budget is unlimited, as T7 of U3) |
| T4 | H1(a) | the guardian keeps its children when it restarts |
| T5 (**U5b**, internal) | D9 | `/user` with an escalating strategy set through the **unexported seam `options.userStrategy`**, not through public API: a top-level failure escalates, `/user` fails with that cause, the root guardian stops it, the chain of section 3.3.1 runs, every top-level actor ends, and `Spawn` returns `ErrParentStopping`. It uses `OneForOne[guardianMsg](Budget{}, escalate)` inside the test package. |
| T5p (**U5c**, public) | D3, D9 | the same scenario through `Config{GuardianStrategy: GuardianOneForOne(…, escalate)}` and a decider that returns `Escalate`; it also runs for a nil decider and for a decider that panics. It is the public-API version of T5 and the adaptation of `ActorSystemSpec:359` without `whenTerminated` |
| T6 (**U5b**) | D9, G4 | `/user` stopped from outside the chain: the same sequence and the end effects (top-level actors stopped, `Spawn` refused), no `whenTerminated` claimed |
| T7 | D5 | `Parent()` of a top-level actor is the `/user` handle |
| T8 | D7 | the five names of D7 give `ErrInvalidName` at top level; valid names are accepted |
| T9 | D8 | IDs and outstanding sets: a top-level actor counts under `/user`, baselines return after create and stop |
| T10 | Goexit | a Goexit in a top-level `Receive` is `ErrGoexit` and Stop by default; a restart does not lose a queued message (the U3 mechanism, now reaching top-level actors) |
| T11 | 3.4, **controlled executor** | `NewSystem` with a deferred executor: right after it returns, the registry holds the three guardians with IDs in the order root, `/user`, `/system`, the names `user` and `system` are committed, the strategies are installed and the Watch links exist, and **no activation has run** (the executor holds exactly three). Running them one by one in order leaves the system quiescent with nothing failed |
| T12 | 3.4, **inline executor** | `NewSystem` with an executor that runs every activation at once (`sup1InlineExec` with inline set): each guardian's first activation, observed from a `bootPoint` probe, sees all three guardians registered, committed under the root guardian, with their strategies and links. Asserted at every probe point, not only at the end |
| T13 (**U5a**) | 3.3.1, 3.4, **immediate close** | for each boundary of `bootPoint` (after steps 2, 3, 4 and between the three activations), with both executors, and for each of the three entry points of section 3.3.1 (stop `/user`; stop `/system`; stop the root guardian): the sequence and the order of the `Done` channels of the table hold, **in the "stop `/system`" case `/user` is stopped by the root guardian's termination and closes after `/system` and before the root guardian**, every `Done` closes exactly once, one `stop-accepted` per guardian, the watch notifications of the table, and nothing deadlocks. Asserted on the guardians only; **in U5a it does not assert any effect on top-level actors or on `Spawn(sys)`** (section 3.3.2) |
| T13b (**U5b**) | 3.3.1, 3.3.2 | the same three entry points, now with live top-level actors: each is stopped before `/user` finalizes (its subtree first), `Spawn(sys, …)` returns `ErrParentStopping` after the gate closed, and a `Spawn` racing the close either commits and is stopped or is refused, never left alive; for both executors |
| T14a (**U5a**) | 3.4, 3.3.2 | with a deferred executor, a legacy top-level `Spawn`, a `Tell` to it and a failure of it arriving before any guardian activation behave exactly as before (the actor stops directly): the guardians neither supervise nor receive them. Asserted to show that U5a changes nothing for top-level actors |
| T14b (**U5b**) | 3.4 | with a deferred executor, a top-level `Spawn`, a `Tell` and a failure of the top-level actor arriving before any guardian activation are not lost and are supervised by `/user`: they are processed after the start steps, in order, and the failure is decided by `/user` (restart) |
| T15 (**U5c**) | 3.5 | the external `Example` of `package actor_test` compiles and runs for `GuardianOneForOne` and `StoppingGuardianStrategy`; the zero value behaves as `DefaultStrategy`; a nil decider escalates; a panicking decider ends the system (D9) |
| T16 (**U5b**) | 3.6, D11 | `Parent()` of a top-level actor returns the `/user` handle; `Done(h)` closes after the actors; `Watch(h)` and `WatchWith(h, msg)` are accepted and deliver nothing; `Stop(h)` from a top-level actor ends the system (every `Done` closed, `Spawn` returns `ErrParentStopping`), and a second `Stop(h)` is a no-op |

Mutants for the module close (not run in U5, policy of 2026-10-15): the default for top-level is Stop; the guardian loses its children on restart; the root guardian Restarts instead of Stopping; the chain does not stop `/system`; `Parent()` still false; top-level names unvalidated; the guardian strategy is ignored.

## 7. Out of scope, gaps and dependencies
`System.Shutdown`, `whenTerminated`, the termination hooks and `Kill` (#67, #192, #193, #450 item 3); a custom guardian `Props` (`ActorSystemSpec:377`); supervision logging (U9, #451: the root guardian's "log and Stop" logs nothing); `AllForOne` (U6, #447); `PostStop` double call (U7); the judge mirrors S13, S14, S16 (U8, #175); user-defined strategies (#450). **No dependency on an unmerged unit.**

## 8. Allowed edit surfaces, by delivery (the exact list is fixed in each RUNBOOK)
Documentation (rows 9 to 12 of section 5) is done by the coordinator, as in U3 and U4. Anchors: section 5, row 13. New file names: `guardian.go` (the guardian cells, the actors and the start-up), `guardian_startup_internal_test.go`, `guardian_internal_test.go`, `guardian_goside_internal_test.go`, `guardian_api_test.go` (the external example).

## 9. Delivery in three parts and budget (D12)
**Two different budget figures, kept apart.** (1) The **maximum per delivery** is unchanged: 560,000 tokens counting agents and coordination, with the evaluation stop at 450,000, as in U3 and U4. (2) The **aggregate for the whole of U5** is a different number: three deliveries at that maximum allow up to **1,680,000** tokens (and an aggregate evaluation stop of 1,350,000), three times the 560,000 that a single unit had. **That aggregate is a proposal and is not authorized by this RULEBOOK or by approving it.** The Owner authorizes each delivery separately when its execution package is presented; the aggregate is reached only if all three are authorized and each spends its maximum. The measured basis is that a delivery with one implementer and one review used about 460,000 agent tokens; all of U5 in one delivery would have meant roughly three times the code and about 12 rewritten test files, which does not fit one cap, hence the split. Each part is a separate PR into `develop`, compilable and verifiable on its own, with the tests that **it** breaks adapted in the same part.

| Part | Content | Production files (expected) | Tests it breaks, adapted in this part | New tests | Behavior change for existing code |
|---|---|---|---|---|---|
| **U5a** guardians and start-up | the three guardian cells, the actors, the strategies of D2 (the `/user` one default, no configuration yet), the Watch chain, the start-up of section 3.4 with the `bootPoint` seam, the stop of `/user` ending the chain. **`s.root` is untouched**: top-level `Spawn` still uses it and has no parent; the root guardian has its own node, so `/user` and `/system` are not names in the legacy top-level table | `guardian.go` (new), `system.go` (build), `hierarchy.go` and `cell.go` only if the construction needs them | only the count-based baselines (`registered()` and the like), by one helper that excludes the three guardian IDs; no behavior assertion changes | T1, T4, T11, T12, T13, T14a (and the intermediate-behavior assertions of section 3.3.2) | none observable except three extra IDs and three extra activations; stopping `/user` (only through a test accessor) ends the guardian chain and **nothing else** |
| **U5b** top-level under `/user` | `spawnTarget` of `*System` uses `/user`'s node and handle as parent; `Parent()` returns the `/user` handle (D5, D11); a top-level failure goes to `/user`'s decider; the legacy `s.root` table is retired | `hierarchy.go`, `spawn.go`, `supervision_failure.go`, `system.go` | the interim tests: T11 and V14 and T9 and T7 and `spawn_errors_test.go:159` (stop directly becomes restart or Stop under `/user`), the `outstanding(root)` baselines of `termination_internal_test.go`, `hierarchy_internal_test.go`, `completion_internal_test.go`, `hierarchy_concurrent_internal_test.go`, `names_test.go`, `termination_stress_internal_test.go`, `ask_firstreply_internal_test.go` | T2, T3, T5 (internal seam), T6, T7, T9, T10, T13b, T14b, T16, P1, P2, P4, P5, P6 | a top-level failure restarts; `Parent()`; **the end of the system reaches top-level actors and `Spawn(sys)` returns `ErrParentStopping`** (moved here from U5a) |
| **U5c** configuration and names | `Config.GuardianStrategy`, `GuardianStrategy`, `GuardianDecider`, `GuardianOneForOne`, `StoppingGuardianStrategy` (section 3.5); top-level names validated (D7) | `system.go`, `spawn.go`, `guardian.go`, `errors.go` (docs) | `names_test.go` (`TestTopLevelNamesAreNotValidated` replaced), and any test that spawns a top-level actor with an invalid name | T8, T5p (public), T15, P3, the external example | `Spawn(sys, …)` with an invalid top-level name returns `ErrInvalidName`; `Config.GuardianStrategy` fills the `options.userStrategy` seam |

Order: U5a, then U5b, then U5c, each merged before the next starts, because b builds on a's cells and c on b's parent. The ADR 0014 amendments are applied with the part that implements them: (a) the root owner is an actor, with U5b; (b) top-level names, with U5c. U5a changes no ADR text. Each part's execution package (RUNBOOK, permissions, launcher, prompt) is prepared after this RULEBOOK is approved and presented for its own hashes; nothing runs without them.

Forecast per part, stated as ranges and not as promises: U5a has the least code and almost no rewritten tests, so it should fit comfortably; U5b carries the largest test migration (the 7 test files and the interim tests) and is the one most likely to reach the evaluation stop; U5c is small in code and in tests. If a part reaches the stop, the coordinator stops and asks, as in U3 and U4.
