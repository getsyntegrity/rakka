# ADR 0012: A stop accepted before the first activation (divergence D2)

- Status: accepted for the bounded case below (option 1); the amendment "Correction (#690)" (no `PostStop` for an instance
  whose first `PreStart` failed) accepted by the Owner on 2026-10-22, see the provenance in that section
- Date: 2026-10-09 (proposed); 2026-10-10 (approved and implemented); 2026-10-22 (amendment of #690 accepted)
- Stage: 1

> **Approval recorded.** The owner approved option 1 on 2026-10-10, **only** for the case "a stop accepted after a
> successful `Spawn` and before the first activation". The approval does not define restart, system shutdown or a
> cancellation API. The sections "How Pekko solves it", "How GoAkt solves it" and "Evidence" describe the divergence as it
> was before the fix and are kept as history.
> Naming note: this D2 is the divergence in `actor-core-parity.md`, not decision D2 (reply address) of ADR 0010.
>
> **Correction of 2026-10-22 (#690), accepted by the Owner on 2026-10-22** (provenance in the section "Correction (#690)").
> The text below said that a failed `PreStart`
> still runs `PostStop`, and presented the lifecycle as "always paired". Pekko Classic does not do that: see the section
> "Correction (#690)" at the end for the sources, the previous and the new behavior. The approval above covers only the
> case "a stop accepted before the first activation"; it never decided `PostStop` after a failed `PreStart`, which the text
> described as "the existing contract". The corrected lines are marked **[corrected, #690]**.

## Feature

What `Spawn` followed by an accepted stop must do when the stop is accepted before the actor's first activation ran:
call `PreStart` and `PostStop` (paired when `PreStart` succeeded **[corrected, #690]**: an instance whose `PreStart` failed
gets no `PostStop`) or neither (the stop wins before the actor ever started).

## How Pekko solves it

- Module: `pekko-actor` (Pekko's `actor` module).
- Concept: the start of an actor is a system message. `ActorCell.init` enqueues `Create` in the system queue before
  the reference is handed out, so a `Terminate` sent afterwards is processed after it: `preStart` runs, then
  `postStop` **[corrected, #690]: only if `preStart` succeeded**. If `preStart` throws, `create` runs `failActor()`, which
  leaves the actor null (`ActorCell.scala:643-647`, `:663-666`), and `finishTerminate` calls `postStop` only `if (a ne null)`
  (`dungeon/FaultHandling.scala:247-254`), so no `postStop` runs. A stop never overtakes the start. Messages queued before the
  stop are dead-lettered, never received.
- Files: `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/dungeon/Dispatch.scala:79-111` (`init` enqueues
  `Create`), `:167-169` (`stop` sends `Terminate`); `.../actor/ActorCell.scala:642-653` (`create` calls
  `aroundPreStart`); `.../actor/dungeon/FaultHandling.scala:180`, `:247-254` (`terminate`, `finishTerminate` call
  `aroundPostStop`). `upstream/pekko/docs/src/main/paradox/actors.md:923-957` documents `stop`.

## How GoAkt solves it

- Package: not consulted. Pekko wins when all else is equal, and no evidence shows GoAkt solved this better.
- Concept: none recorded.

## Evidence

- Benchmark: none. This is a lifecycle contract, not a performance question.
- Judge: scenario S20, clause G5.e, [judge.md](../judge.md) section 3. Pekko `v1.7.1`: 300 of 300 concurrent runs pass
  (`pre-start`, `post-stop` once each; the queued message is dead-lettered). goPekko at the base of this change: all
  runs fail with exactly G5.e (neither hook runs; the queued message is dead-lettered). The stop is accepted before
  the activation by construction on both sides (a one-thread dispatcher held by a blocker actor in Pekko, a held
  executor in Go) and checked (a log with an event of the actor before the release is a harness error). Mutation M65
  reproduces the Go behavior inside Pekko and is caught by G5.e.
- Complexity: the Go behavior comes from two lines: `cell.go` `drain` checks `stopRequested` before the start step, and
  `stop.go` `finish` calls `PostStop` only if `started`. Changing it touches the same two places.
- Idiomatic in Go: both are expressible. Skipping the hooks avoids running user code for an actor that is already
  being stopped; pairing them keeps one rule (a started resource is released) that does not depend on timing.

## Decision

Option 1, for the bounded case only. The options below it are kept as the record of what was considered.

**Approved contract.**

- `PreStart` is invoked once.
- `PostStop` is invoked once, after `PreStart` has returned.
- No `Receive` runs after the stop is accepted.
- The existing contracts of dead letters (messages that waited are dead-lettered once, published in order), name release and
  `Done` (closes after `PostStop` and the dead-letter publication) are unchanged.
- Errors and panics: a failure of the first `PreStart` emits `prestart-failed` (no second `stop-accepted`) and **no `PostStop`
  runs for that instance [corrected, #690; the text said "`PostStop` still runs"]** (a restart's new instance is a different
  case, see "Correction (#690)"); a `PostStop` panic (after a successful
  `PreStart`) emits `poststop-panic` and cleanup continues; a `PreStart` that ends with `runtime.Goexit` is a
  `prestart-failed`, no `PostStop` runs and the stop completes.
- **U4 (ADR 0017 S10, #444):** the start step now includes the factory, which runs in the first activation before
  `PreStart`. If a stop was accepted before that activation and the factory fails (a nil actor, a panic or a
  `runtime.Goexit`), no instance exists: no `PreStart` and no `PostStop` run, the failure is only an event
  (`factory-failed`, the stop is already accepted so nothing is reported to the parent) and the stop completes. If the
  factory works, `PreStart` runs once and then `PostStop`, as above.
- The "once" guarantee is **scoped to this case**. It says nothing about restarts, which do not exist in this runtime (a
  failure stops the actor), and the decision does not define restart or system shutdown.

**Mechanism.** One transition, `beginStartLocked`, decides who runs the start step: it reports true to exactly one caller.
`drain` takes it on the first activation; `finish` takes it when the stop was accepted before that activation, then runs
`PostStop`. Both run inside `drain`, so the transition is serial, and an activation handed over after a `Goexit` finds the flag
already set and does not run `PreStart` again. `PostStop` no longer depends on whether the start step began, because `finish`
has always taken it by then. **[corrected, #690]** It does depend on whether the start failed: `finalize` skips `PostStop`
when `initFailed` is set (the start of the instance failed and was reported), the Go counterpart of Pekko's `_actor == null`.

**Context obligation (not implemented).** ADR 0008 (K1) says that accepting a stop cancels the actor-owned processing context.
The runtime has no such context and this change adds no cancellation API. When it exists, `PreStart` of this case receives a
context that is already cancelled and must be written to return promptly. This is a future obligation linked to ADR 0008; it is
**not implemented** and nothing in this change claims it.

**Judge.** S20 moves from `diagnostics` to `scenarios` in `judge/go-scope.json` and is judged with all its clauses. The
historical evidence of the divergence (300 of 300 Go failures with exactly G5.e at `b1863f1`, and the Pekko mutation M65
caught by G5.e) is kept. S20 is **not** calibrated on the Go side: no Go mutant is designed for it yet.

### Options considered

1. **Pair the hooks (Pekko behavior). Chosen.** `finish` runs the start step first if it has not run, then `PostStop` (only if
   the start succeeded **[corrected, #690]**).
2. **Keep the Go behavior and record the divergence as intentional.** The stop wins before the start; no hook runs. Rejected:
   code that expects `PostStop` for every spawned actor would differ from Pekko.
3. **Change `Spawn` so the first activation cannot lose a race with a stop.** Rejected: it changes the meaning of `Stop` and
   would need its own ADR.

## Consequences

- D2 is resolved for the bounded case. S20 passes in 300 of 300 Go runs and is part of the approved-parity gate.
  `diagnose_go.py` no longer reports S20 (`diagnostics` is empty); its validation is the gate's.
- The test "a Stop accepted before the start step runs no hook" fixed the old behavior and was inverted on purpose.
- Pekko differs in mechanism only: `Create` is a system message queued ahead of `Terminate`, while here `finish` takes the start
  step. The observable order (`pre-start`, `post-stop`, when `PreStart` succeeds) and the dead-letter contract are the same;
  the event `stop-accepted` precedes `pre-start` in the Go log.
- Not resolved here: restart, system shutdown, the cancelled context of ADR 0008 for this case, and the calibration of S20.

## Correction (#690)

Status: **accepted by the Owner on 2026-10-22**, for the behavior demonstrated in #695. It amends statements of this ADR, not
the approved bounded case.

**Provenance of the acceptance.** It comes from the Owner's written instruction of 2026-10-22 in the working session of #695,
which says (in Spanish): "Acepto la enmienda del ADR 0012 para el comportamiento demostrado en #695." ("I accept the amendment of
ADR 0012 for the behavior demonstrated in #695."). It is not the outcome of a review. The scope is that behavior only: no
`PostStop` for an instance whose first `PreStart` failed (and for the `Create` that follows an initialization failure), with
the cases listed under "What does not change" kept as they are. It does not accept anything else: not the `PORT-GAP` markers
(#691, #692), not a change to restarts, and not a change to `OrdinaryFailure`.

**Record of how #695 was integrated, so that nobody reads more into it.**

- The native `high` review of the candidate was authorized by the Owner and **did not start**: the preflight returned the terminal
  stop `managed_assets_outdated`, whose only continuation is `gentle-ai sync`, which changes global configuration and was not
  authorized. There is no START, no lineage and **no verdict**. The review is neither approved nor replaced; the Owner
  authorized integrating without it.
- The remote checks `go` and `label` did not run: their annotation says "The job was not started because recent account
  payments have failed or your spending limit needs to be increased". They are not green. The Owner authorized using the local
  validation instead (`go vet`, `gofmt`, `golangci-lint` v2.14.0, `go test -race`, `make ci-fast`, `scripts/check_deps.sh`,
  `go mod tidy -diff` and per-module coverage).
- The diff is 465 authored changed lines, over the 400-line budget of the PR rules; the Owner authorized that exception for the
  correction and its directly related tests.

**What was wrong.** The "Decision" section said that a failed `PreStart` emits `prestart-failed` "and `PostStop` still runs",
and the "Feature" section called the lifecycle "always paired". Both were stated as the existing contract and attributed to
the Pekko pairing. Pekko Classic does not run `postStop` for an instance whose `preStart` failed. The sources below show it;
the rows of this ADR that cite `Dispatch.scala:79-111` and `FaultHandling.scala:247-254` prove only that `postStop` follows a
successful `preStart`.

**Sources** (Apache Pekko `v1.7.1`, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, under
`upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/`):

| Contract | Source |
|---|---|
| A failed `preStart` runs `failActor()`, which clears the fields and sets `_actor = null` | `ActorCell.scala:642-683` (`create`: `created.aroundPreStart()` at `:653`; the `NonFatal` branch at `:663-666`; `failActor` at `:643-647`) |
| `newActor()` assigns `_actor` only on success of the constructor | `ActorCell.scala:622-640` (`_actor = instance` at `:633`) |
| `postStop` runs only if the actor is not null | `dungeon/FaultHandling.scala:247-254` (`finishTerminate`: `val a = actor`, `if (a ne null) a.aroundPostStop()`) |
| After the supervisor decides a restart of a cell that failed to initialize, a new `create` runs | `dungeon/FaultHandling.scala:158-178` (`faultCreate`, `finishCreate`) |
| A failure while a **restart** builds the new instance does not null `_actor` | `dungeon/FaultHandling.scala:278-303` (`finishRecreate`: the catch calls `clearActorFields(actor, recreate = false)`, which does not set `_actor = null`, `ActorCell.scala:718-721`) |
| The test | `actor-tests/src/test/scala/org/apache/pekko/actor/ActorLifeCycleSpec.scala:165-188`: with a `preStart` that always throws and `maxNrOfRetries = 3`, it expects four `preStart` reports and then `expectNoMessage()`, while `LifeCycleTestActor.postStop` reports (`:36`) |

**Previous behavior.** `finalize` called `PostStop` on the instance whenever one existed, so after a failed `PreStart` (an error,
a panic or a `runtime.Goexit`) the stop that followed ran `PostStop` once. Tests fixed it: `lifecycle_internal_test.go`
(`["pre","post"]` after an early stop, `["post"]` after a failed `PreStart`).

**New behavior.** `finalize` skips `PostStop` when `initFailed` is set. That flag is set by `reportInitFailure` (the failure of the
first start and of a `Create` after a supervised init failure) and cleared by `finishCreate`, so it is the Go counterpart of
Pekko's `_actor == null`. Nothing else changes: termination waits for the children and runs their `PostStop`, the name is
released, the queued messages are dead-lettered once in order, the watchers are told once and `Done` closes.

**What does not change (and the test that pins it).**

- `PostStop` after a successful `PreStart`, also when the stop was accepted before the first activation
  (`TestPreStartHooks`, `TestPostStopHooks`).
- `OrdinaryFailure`/`ResumeWith`: the start step counts as completed, so the instance gets its `PostStop`
  (`TestOrdinaryFailureKeepsPostStop`). It is an adaptation API of this port (`start_failure.go`), not a Pekko API.
- A failure while a restart builds the new instance (a panicking or failing factory aside, an error in `PostRestart` or in the
  `PreStart` it runs): `initFailed` is not set, the instance exists, and the stop that follows runs its `PostStop`, as in Pekko
  (`TestAFailedPostRestartKeepsPostStop`).
- A failing or nil factory: no instance, no `PostStop` (already the case, U4 above).
- The supervised restart path: the init failure is decided by the parent and each new attempt runs `PreStart` only, now with no
  `PostStop` at the final stop either, which is what `ActorLifeCycleSpec.scala:165-188` expects (`TestSup2InitializationRestart`).

**Consumers reviewed.** The only `PostStop` in the consumer modules is the persistent actor's
(`pekko-persistence/persistent/persistent.go`, `PostStop`), which releases the Context of `RecoveryCompleted` (`hold`). The hold is
set inside `completeRecovery`. Its failures that come back as a value go through `ordinary`, that is, as an `OrdinaryFailure`, so
the instance counts as started and its `PostStop` still runs. The failures that reach the runtime as an initialization failure
before the hold exists (an empty persistence id, a failed snapshot load, a failed replay) acquire nothing that `PostStop` releases.

**One dependency was found by the final review of #695.** The earlier text of this section said that every failure of
`completeRecovery` is an `OrdinaryFailure`. That was too strong. When the write of what `RecoveryCompleted` persisted fails and the
`OnPersistFailure` observer panics or calls `runtime.Goexit`, the panic leaves `PreStart` (`writeFailed`, `settle`), the runtime
sees a failed initialization, and with this change `PostStop` no longer runs, so the hold stayed unreleased and the retained Context
stayed live after the actor stopped (instead of panicking with `ErrContextExpired`). Verified: the assertion added to
`TestAnObserverThatPanicsOrExitsStillStopsTheActorAfterAFailedRecoveryCompletedWrite` is green with the previous behavior of
`finalize`, red with the guard and without the fix, and green with the fix. The fix applies the rule of this correction to the
consumer: `PreStart` of the persistent actor releases the hold itself when it leaves by a panic or a `Goexit`
(`pekko-persistence/persistent/persistent.go`). A normal return, an error included, keeps the hold as before (`ResumeWith`
still needs it). The change is in the persistence core module; the SQLite and PostgreSQL adapters have no persistent-actor layer,
so there is nothing to mirror in `pekko-persistence-sqlite` (CLAUDE.md, SQLite parity). The unit suites of every module that
imports the actor pass with `-race`.

**Consequence for users (a change of a public lifecycle contract).** A resource acquired in `PreStart` before it fails is not
released by `PostStop`, exactly as in Pekko. `PreStart` must release what it acquired **on its own failure path** (for example
with a `defer` that runs when it returns an error or panics), or acquire resources only after the last point at which it can
fail. Code that relied on `PostStop` to release the resources of a `PreStart` that failed leaks them after this change. The
guard of the rule is covered by `TestAHealthyStartAfterAnInitFailureKeepsItsPostStop`: an instance that starts correctly
after a failed first start keeps its `PostStop` (the flag is cleared when the new instance is created); without that reset the
rule would suppress the `PostStop` of healthy instances, and no other test detected its removal.

**Scope of the rule (precision added in the final review of #695).** It applies to the first start of an actor and to the
start of a `Create` (the supervisor's decision after an initialization failure), which are the cases in which Pekko's `_actor`
is null. It does not apply to the new instance of a **restart**: if its `PostRestart`, or the `PreStart` that the default
`PostRestart` runs, fails, the instance exists (`ActorCell.scala:633`, `finishRecreate`, `dungeon/FaultHandling.scala:286-288`)
and its `PostStop` runs when the supervisor stops it (`TestARestartWhoseDefaultPostRestartFailsInPreStartKeepsPostStop`,
`TestAFailedPostRestartKeepsPostStop`).

**Traceability.** #690 (this correction), the audit that found it (#687, "PostStop after a failed PreStart"), and the tests
changed: `lifecycle_internal_test.go`, `supervision_restart_internal_test.go` (`TestSup2InitializationRestart`) and
`prestart_failure_internal_test.go` (new; the two tests of the final review pin the reset of `initFailed` and the restart
case), and `pekko-persistence/persistent/write_failure_stop_test.go` (the hold of the persistent actor). The corrected statements
are marked **[corrected, #690]** above; the rest of the ADR is unchanged.

**What the final review of #695 changed** (head `f810e827` reviewed; follow-up commit on the same branch): the guard of the new
rule had no test (removing the reset of `initFailed` in `finishCreate` broke nothing; a mutation run confirmed it), the public
wording over-generalized to restarts, and the persistent actor depended on `PostStop` for one failure path. The three are
corrected here and in the PR; the behavior for the first start, `OrdinaryFailure` and the restart cases did not change.

**Tests added after that review, with no functional change** (a conventional review of the candidate `4dc9ccc3`, the native
review being blocked and without a verdict): the retirement of the registry entries of a failed actor and of its child
(`TestAFailedPreStartKeepsTerminationButSkipsPostStop`); a `Goexit` inside the `RecoveryCompleted` callback, which `recover()`
does not catch and which is therefore released only by the deferred release of `PreStart`
(`TestAGoexitInRecoveryCompletedExpiresItsContextWithoutPostStop`); and the factory that fails while a restart builds the new
instance, whose old instance gets `PostStop` twice, as in Pekko and as before this correction
(`TestAFactoryThatFailsDuringARestartKeepsThePostStopOfTheOldInstance`; this is not a divergence).
