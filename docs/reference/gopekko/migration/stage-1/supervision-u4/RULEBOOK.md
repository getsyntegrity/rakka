# RULEBOOK: Supervision U4, `Spawn` creation failures supervised by the parent (#444, unit supervision-u4)

Status: **PROPOSED, for the Owner's approval. This PR changes no code, no test and no other document.** Nothing in `pekko-actor/` or in the ADRs is modified until the Owner approves this file (ADR 0017 S10: "nothing is modified before U4's RULEBOOK is approved"; #444 acceptance: the behavior list and the reconciliation table are approved before any file is modified).
Source of truth: `docs/adr/0017-supervision.md` (S10, H7 direction approved by the Owner on 2026-10-14, H5c, A8, A11, the U4 row) and Apache Pekko Classic v1.7.1 (`upstream/pekko`, read-only, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, which is the tag `v1.7.1`).
Base: `develop` at `7f7e292` (squash of #456, U3 merged). U1, U2 and U3 are in `develop`.

## 0. What U4 is, and what it is not

U4 is the **contract change of `Spawn`** decided in H7: the factory runs in the child's start step, and a nil actor, a panic or a `runtime.Goexit` of the factory is an `*InitializationError` decided by the parent. It is **not** a rollback fix. The rollback of the reservation on a failed factory is already implemented and tested (section 2.6); U4 replaces that contract, it does not add it. Where the task description and the ADR differ, this RULEBOOK follows the ADR and #444.

Infrastructure already in `develop` that U4 reuses (no change proposed): the retained factory and `installNew` (`supervision_restart.go`), `finishCreate` and `faultCreate` (R12a, the re-Create of a never-initialized cell), `reportInitFailure`, `defaultDecider` (Stop for `*InitializationError` and `ErrGoexit`), the Goexit hand-over `handOver`, and the guard `protect`.

## 1. Decisions this RULEBOOK asks the Owner to confirm by approving it

| # | Decision | Why it needs the Owner |
|---|---|---|
| D1 | `Spawn` returns once the name is validated, the owner is checked and the child is **committed** and the first activation is handed to the executor. `Spawn` does not call the factory; the executor does, through the activation. With an inline executor the activation, and so the factory, runs inside that hand-over, on the caller's goroutine, before `Spawn` returns (section 2.1). The synchronous errors of `Spawn` are exactly three: `ErrInvalidName`, `ErrNameTaken`, `ErrParentStopping` (verified in `spawn.go:44,76` and `hierarchy.go:79,82`; `ErrNameTaken` also has an unreachable defensive return at commit). **There is no other error** (the ADR text "plus the others it already has" is imprecise: no other exists today). The child (ID, `Ref`, reserved name, `Done`) exists from the commit, which precedes the first activation. What state the child is in at the return is **not** promised (section 2.1). | Public behavior; H7 direction, details here. |
| D2 | `Spawn` **stops returning** `ErrNilActor` and an error wrapping `ErrFactoryPanic`. The exported identifiers `ErrNilActor`, `ErrFactoryPanic`, `FactoryPanicValue` and their `errors.Is/As` behavior **stay**; they describe the **cause** of the `*InitializationError` that a decider or an event receives. Signature of `Spawn` unchanged. | "No silent API change" (ADR 0017 S10). Callers that tested `errors.Is(err, ErrNilActor)` on the `Spawn` result break; they must observe `Done`, `Watch` or the decider. |
| D3 | The factory runs **once**, in the child's start step, in the child's first activation, outside every lock, **before** `PreStart`. `Spawn` does not call it: with the default executor it runs on the activation's own goroutine, and with an inline executor the activation, and the factory with it, runs on the goroutine that called `Spawn`, inside the call. | Concurrency and data-race implications for factory closures; ADR 0014 transaction text changes (row 6). |
| D4 | Failure classification: nil actor, panic, or Goexit of the factory is an `*InitializationError{InRestart: false}` whose `Err` is `ErrNilActor`, the error wrapping `ErrFactoryPanic` (with `FactoryPanicValue` reachable), or `ErrGoexit`. It reaches the parent's decider exactly like a `PreStart` failure; default Stop. A Restart or Resume choice re-creates through the existing `finishCreate` (the factory runs again, same `Ref`, no `Terminated`, queued messages kept). | A9a/A9b classification; the factory Goexit row of H5c. |
| D5 | The **factory of a restart** that ends with Goexit is classified too (`InRestart: true`, cause `ErrGoexit`), replacing `abortGoexit` for that site and closing `PORT-GAP(factory-goexit-U4)`. | Declared in U3 as U4's row; it changes the U2 interim. |
| D6 | No `PostStop` and no `PreRestart` for an instance that never existed (a failed first factory, nil actor). `PostStop` stays paired with a `PreStart` that ran, as ADR 0012 requires. | Hooks accounting; upstream `finishTerminate` calls `aroundPostStop` only if an instance exists. |
| D7 | Top-level `Spawn` (owner is a `*System`) has no decider until U5: a factory failure stops the child directly with the cause (the existing stop-on-failure interim), `Done` closes, the name is released then. Declared `PORT-GAP(top-level-default-U5)`. | Keeps U5 out. |
| D8 | A new event `factory-failed` (name already listed in ADR 0017 A11) is emitted with the cause as detail, before the failure is reported to the parent; no other event name changes. Logging stays with U9 (#451). | Observability; events are an additional channel, not a replacement for logs. |

## 2. Behavior specification

### 2.1 Order of `Spawn`, and what it promises at the return
Order inside `Spawn` (today's code, kept except that the factory step moves out): (1) validate the child name (`ErrInvalidName`, through a `*Context` only); (2) reserve the name, which also checks the owner's gate (`ErrParentStopping`, `ErrNameTaken`); (3) build the cell, `Ref` and `Context`; (4) **commit**: the child is registered, with its ID, before anything can run; (5) if the owner's stop was accepted in between, accept the child's stop; (6) hand the first activation to the executor; (7) return the `Ref`. After step 4 nothing in `Spawn` can fail.

Two statements are kept apart, because they were mixed up:
- **What the contract says `Spawn` does:** it does **not** invoke the factory. The factory is invoked by the child's first activation, and the executor decides where that activation runs. That is a statement about who calls the factory, not about which goroutine: an inline executor runs the activation on the caller's goroutine, inside `Spawn`.
- **What can be observed when `Spawn` returns:** the default executor starts a goroutine per activation (`executor.go:19-22`, never inline), so the first activation may not have started, may be running the factory, or may have finished; a test executor may run the activation **inline inside step 6** (`sup1InlineExec` does; the executor contract says "it may run f inline or later", `executor.go:15`), and then the factory, `PreStart`, a default Stop after a factory failure and even `Done` can complete before `Spawn` returns. So `Spawn` promises **no** relation between its return and the factory having run or not.

What is promised is only this: the child **was committed** before the factory could run, so it was given an ID, a `Ref`, a reserved name and a mailbox, and `Spawn` returns that `Ref` with a nil error. Nothing about its state at the return is promised. With an inline executor it may already have started, failed in the factory, been stopped by the default decider and **terminated**: then `Done` is already closed, the name is already released and reusable, the owner's children set no longer includes it, watchers already got their `Terminated` (or a later `Watch` of the finished `Ref` follows the existing rule for a terminated actor), and `Tell` is dead-lettered. `Done` closes only when it terminates. Tests that need an order use a controlled executor or hold the factory on a channel; none relies on timing or on the child "being in its start step" when `Spawn` returns.

### 2.2 The start step
First activation: run the factory (guarded: error-less panic and Goexit classified), install the actor, then `PreStart`. Any failure of the factory skips `PreStart`. The start step takes one step even with no `PreStart`, as today.

### 2.3 Failure, supervision and the name
| Case | Result |
|---|---|
| Factory returns nil | `*InitializationError{Err: ErrNilActor}`, `factory-failed`, parent decider; default Stop. |
| Factory panics | `*InitializationError{Err: error wrapping ErrFactoryPanic}`, `FactoryPanicValue(cause)` returns the value. |
| Factory Goexit | the same path with `ErrGoexit`; a Restart choice continues the activation on a fresh goroutine (the U3 hand-over), no queued message lost. |
| Decider Stop (default) | the child terminates with no `PostStop`; `Done` closes; watchers get one `Terminated`; the name is released only then and can be reused (a new ID). |
| Decider Restart or Resume | `finishCreate` runs the factory again; the new instance gets `PreStart`; same `Ref`; watchers get no `Terminated`; messages queued behind the start are processed by the new instance. |
| Top-level owner | D7. |

### 2.4 Interaction with stop, restart and DeathWatch
- **Stop accepted before the first activation** (ADR 0012): the start step still runs once (factory, then `PreStart`), then `PostStop` once. If that factory fails, no instance exists: no hook runs and the stop completes. The failure is an event only, because a parent that is stopping or a child that is stopping refuses `failUp` (as in U3).
- **Parent stops or restarts while the child is initializing:** the child is committed, so the owner's stop or restart stops it through the normal recursive stop; a parent does not finalize until it completes.
- **DeathWatch:** a watcher registered on the returned `Ref` gets exactly one `Terminated` when the child terminates, whether it failed in the factory or not; shape of judge S14, which U8 (#175) mirrors, not U4.
- **A factory that never returns:** blocks the child's start and completion, and therefore its parent's finalization. It does not block `Spawn` with the default executor, which runs the activation on its own goroutine. **With an inline executor it does block `Spawn`**, because the activation, and the factory, run inside the call. Nothing in this RULEBOOK promises that `Spawn` returns while the factory runs.

### 2.5 Pekko equivalence and Go adaptation (kept apart)
- **Equivalent to Pekko:** the constructor runs when `Create` is processed, not called by `actorOf` itself (in Go an inline executor runs the activation on the caller's goroutine, which is a property of the executor and not a call made by `Spawn`; the default executor never does it; `dungeon/Dispatch.scala:79-111`, `ActorCell.scala:642-683`); a null instance or an exception becomes an `ActorInitializationException` handled by the parent (default Stop); `actorOf` returns synchronously only an invalid name, a name in use and a terminating parent (`dungeon/Children.scala:271-345`, `reserveChild` at `:316`, `unreserveChild` only for failures of `provider.actorOf`); `finishTerminate` calls `aroundPostStop` only if an instance exists (`dungeon/FaultHandling.scala:247-257`).
- **Go adaptation, no JVM counterpart:** `runtime.Goexit` in the factory (`ErrGoexit`, default Stop, H5c); the `factory-failed` event; `FactoryPanicValue`; nil detection by `isNilActor` (typed nil pointers). None of these is claimed as parity.

### 2.6 What is already covered (so it is not redone)
Rollback of the reservation on nil, panic and Goexit of the caller-side factory, no ID, no event, name reusable, the owner held open by a reservation: `spawn_errors_test.go`, `hierarchy_internal_test.go:265-295`, `termination_internal_test.go:310-330`. U4 moves these assertions to the start step (section 3, rows 3, 4 and 7); the **reservation rollback** itself disappears with the contract, because the commit now happens at `Spawn`.

## 3. Reconciliation (every row; U4 may not leave the old text next to the new contract)
Locations were re-checked against `7f7e292`.

| # | Artifact | Where (today) | Old text | New text | Test or check |
|---|---|---|---|---|---|
| 1 | `Spawn` and `callFactory` | `spawn.go:5-17,29-84` | caller-side factory, failures returned, "creates nothing" | factory in the start step; doc lists the synchronous errors | `spawn_errors_test.go` rewritten |
| 2 | sentinels and the docs of the `Spawn` errors | `errors.go:11-14,68-98` and `errors.go:15-17,24-28` | `ErrNilActor`/`ErrFactoryPanic` "returned by Spawn"; `ErrNameTaken` held by "a running factory"; `ErrParentStopping` "before the factory ran" | the two sentinels are the cause of an `*InitializationError`, identifiers kept; `ErrNameTaken` is held by a live, stopping or initializing child; `ErrParentStopping` is about the owner's gate at `Spawn` | `errors.Is(cause, ErrFactoryPanic)`, `FactoryPanicValue(cause)` on the decider's cause; review of the three docs |
| 3 | Spawn error cases | `spawn_errors_test.go` (cases at `:29,40,42,53,80,84`) | failing factory returns and frees the name | child terminates (default Stop), `Done` closes, `Terminated` delivered, name freed only then; Goexit is `ErrGoexit` | new tests from `SupervisorHierarchySpec:861`, `ActorLifeCycleSpec:165` |
| 4 | internal tests | `hierarchy_internal_test.go:265-295`, `termination_internal_test.go:310-330` | returned `ErrNilActor`/`ErrFactoryPanic`; rollback | supervised contract | same files, adapted |
| 5 | example | `example_test.go:75-80` | `Spawn` returns the panic | the value comes from the decider's cause | example output |
| 6 | ADR 0014 transaction | `0014:104,115-118,139-145` | factory outside the lock before commit | commit at `Spawn`; factory in the start step | hierarchy tests that hold the factory on a channel |
| 7 | ADR 0014 behavior table | `0014:218-224,233,238-239` | rows nil, panic, Goexit, stop during factory, root gate | one rewritten row each | one test per row |
| 8 | ADR 0011 | `0011:56,78,118,164` | sentinels listed as errors of `Spawn` | listed as causes | review |
| 9 | ADR 0012 | paired-hooks rule (`0012:63-67`) | stop first runs `PreStart` then `PostStop` | adds the failing-factory case: no instance, no hook, stop completes | test: stop accepted, then a failing factory |
| 10 | module README | `pekko-actor/README.md:109-111,129-141,154,169,202,231,259-263,303,380` | factory failures returned; "reservations whose factory has not returned"; `PORT-GAP(factory-goexit-U4)` | every statement updated (module README contract) | `rg` for the old phrases |
| 11 | parity and first-cut docs | `docs/actor-core-parity.md:66,67,217`, `docs/actor-core-first-cut.md:13,26` | "different contracts", "Spawn creation failures not implemented" | Classic-aligned, Go unit tests only, no parity until the judge compares it | review |
| 12 | **found here, not in the ADR table** | `supervision_restart.go` (`installNew` comment, `abortGoexit`), `cell.go:204-212`, `errors.go` `ErrGoexit` doc | `PORT-GAP(factory-goexit-U4)`: a restart factory Goexit stops the cell | classified, `InRestart: true` (D5) | test: Goexit in the factory of a restart, Restart and Stop |

## 4. Test plan and upstream sources
Deterministic tests only: channels and a controlled executor where the order matters; no sleeps as coordination; `runtime.Goexit` tests on the real executor (a controlled one runs the activation on the test goroutine).

| Id | Upstream source | Go test |
|---|---|---|
| P1 | `SupervisorHierarchySpec.scala:861-912` (creation fails 5 times under a decider alternating Resume and Restart; 6th works; `createAttempt == 6`, `preStart == 1`, `postRestart == 0`) | ADAPTED: factory fails attempts 1-5 under a Resume/Restart decider; the 6th starts; `createAttempt == 6`, one `PreStart`, no `PostRestart`. **`workingChild`: decided here, not by the implementer.** In Pekko the constructor runs with `context` in scope and attempts 3 and 6 spawn a child named `workingChild` from it; the Go factory is `func() Actor[T]` (A8) with no `Context`, so that step has **no Go equivalent and is dropped**, declared `PORT-GAP(factory-context)` in the test header. The implementer must not give the factory a `Context`, and must not simulate the child from the test. What the step proves upstream (the children of a discarded creation attempt are stopped and their name is free for the next attempt) is covered separately by T11 |
| T11 | `SupervisorHierarchySpec.scala:861-912` (the `workingChild` step), R12a | A `PreStart` that spawns `workingChild` and then fails, under a decider that chooses Restart: the re-Create stops that child first and the next `PreStart` spawns the same name without `ErrNameTaken`. The implementer first looks for an existing U2 test that already asserts this (`faultCreate` stops every child, `supervision_restart.go`) with `rg` and cites it in the unit record; only if none does, T11 is added. It is a Go-side scenario, not a port, and is labelled so |
| P2 | `ActorLifeCycleSpec.scala:165-190` (**a `PreStart` failure** with Restart `maxNrOfRetries=3`, not a factory failure) | PARTIAL adaptation, labelled as such: `PreStart` failure sequence under Restart with a finite `Budget`; the factory-failure shape is T1 |
| T1 | ADR 0017 S10, `ActorCell.scala:642-683` | nil, panic, on a controlled executor and with the factory held on a channel: the child was committed (ID, `Ref`, reserved name) before the factory ran, and the name stays reserved while it initializes; after the default Stop `Done` closes, `Terminated` is delivered once, no `PostStop` runs, and the name is free only then and reusable. Separate case with an inline executor: the same sequence completes inside `Spawn`, so at the return the child may already be terminated, its name released and absent from the children set; the test asserts the final state and does not assert presence at the return |
| T2 | H5c | factory Goexit reaches the decider with `ErrGoexit`; Stop; Restart continues with no queued message lost |
| T3 | S10, D1 | `Spawn` returns exactly the three errors of D1 and no other: each of the three has a case, and a nil or panicking factory returns a `Ref` and a nil error (`ErrNilActor`/`ErrFactoryPanic` never come from `Spawn`). **Goexit is a different case:** with the default executor a Goexit in the factory ends the activation's own goroutine and `Spawn` still returns the `Ref` normally; with an inline executor the Goexit ends the goroutine that called `Spawn`, from inside the call, so `Spawn` does **not** return at all (the cell is still handed over and finishes with `ErrGoexit`). The RULEBOOK promises a normal return only for the default executor; the Goexit tests use the real executor, as every Goexit test of U3 does, and an inline-executor Goexit case runs the `Spawn` call on a separate goroutine and observes the cell, not the return |
| T4 | ADR 0012 | stop accepted, then a failing factory: no hook, stop completes; stop accepted, then a working factory: `PreStart` once, `PostStop` once |
| T5 | ADR 0014 | parent stopped or restarted while the factory is held on a channel: the child is stopped, the parent finalizes only after it completes |
| T6 | A9a | the decider receives `*InitializationError{InRestart: false}`; a factory failure during a restart is `InRestart: true` (existing), a restart-factory Goexit is `InRestart: true` (D5) |
| T7 | D7 | top-level factory failure stops the child directly; `PORT-GAP(top-level-default-U5)` |
| T8 | DeathWatch | a watcher gets exactly one `Terminated` for a child whose factory failed; none for a Restart |
| T9 | D8 | `factory-failed` is emitted once with the cause |

Mutants and the parity judge are **not** part of U4's per-unit flow (policy of 2026-10-15); a backlog for the module close is written when the unit runs.

## 5. Out of scope and dependencies
Guardians and the default for top-level actors (U5); `AllForOne` (U6); `PostStop` double call (U7); judge mirrors S13/S14/S16 (U8, #175); logging (U9, #451); user-defined strategies; any change to `Budget`, the restart protocol or U1-U3 beyond D5. **Dependency on U5** is limited to row T7 and is isolated: everything else in U4 is independent of it.

## 6. Inventory identifiers (ACTINV-028, ACTINV-067)
The Owner relates U4 to ACTINV-028 and ACTINV-067. Those identifiers come from the inventory file "Texto pegado(8).txt", which is **not in the repository** (no `ACTINV` occurrence in any file, branch or PR). Their content is not reconstructed here, and no ACTINV row is updated by this PR. When the inventory is added to the repository, the rows that map to this RULEBOOK are updated in the unit's execution, with files, tests and the exact commit.

## 7. What approving this file authorizes, and the next step
Approving this RULEBOOK authorizes **only its content** (the decisions D1-D8, the behavior of section 2 and the reconciliation of section 3). It authorizes no code, no agent run and no merge. After approval the RUNBOOK, manifest, permission template and launcher of U4 are prepared in a separate PR for their own hashes. The U3 launcher and package files are under approved hashes and need an **amendment before they are reused** for another execution (its header states a token cap of 900,000/720,000 where the RUNBOOK of U3 says 560,000/450,000; the RULEBOOK of U3 cites "RUNBOOK section 9" for the hashes in section 11). Allowed edit surfaces will be derived then from section 3 (about 16 files under `pekko-actor/` and `docs/`), not widened.
