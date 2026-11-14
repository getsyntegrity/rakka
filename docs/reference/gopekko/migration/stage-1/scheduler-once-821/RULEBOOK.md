# RULEBOOK: the scheduler, first cut (unit scheduler-once-821, #821 TIM-02)

Status: **approved at gate G2 by the Owner on 2026-11-10** for `docs/scopes/821-scheduler-first-cut.md` (D-1 to D-10, PR #1523). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 0.

## 1. Unit

- Capability: a `System` owns one `Scheduler`; `ScheduleOnce(delay, f)` runs `f` once on the System's executor after `delay` on the System's `Clock`, never early, and returns a `Cancellable`; `Context.System()` lets an actor reach it. Not in the unit: the close at `Terminate` and `ErrSchedulerClosed` (slice 1b), `ScheduleTellOnce` (slice 2), fixed delay and fixed rate (#822, #823), keyed timers (#860), `ReceiveTimeout` (#861), `SchedulerTask.cancelled` (#1077), tick rounding (ADR 0039 O-7, ACCEPTED PORT GAP).
- Pekko is the oracle (ADR 0017, ADR 0033 D14): v1.7.1 (`9a54d12`) and `docs/parity/oracles/scheduler-lifecycle-2026-11-10.md` (#1506, #1507). Evidence classes of the manifest: `READ` (source read, not run), `SPEC`, `ORACLE` (scenario executed on Pekko). The `PORT-GAP` rows point here.
- Kit run (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`): port plus adaptation, kit pinned at `cf91c9d`. The Dev is a separate session, QA is independent of both, the TL does not implement.
- Layout: `pekko-actor/actor` (ADR 0011 owner decision 2); no `spi/`; `actor` imports no other goPekko module.
- Writable: exactly the slice table and "Allowed files" of the scope for the PR in hand. Nothing else: not `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, another module.

## 2. Rules for the Dev (G2 decisions)

- D-3: `ScheduleOnce(delay time.Duration, f func()) (Cancellable, error)`. D-4: a recovered panic is written once to `Config.Logger` at Error, nothing without one (`TODO(TIM-01-error-report)`). D-5: a nil `f` is `ErrNilCallback`, checked before the delay rule (`TODO(TIM-02-nil-callback)`).
- O-1 A: `Cancellable` has exactly `Cancel() bool` and `IsCancelled() bool`. O-3 b: a delay at or below zero is handed to the executor at once and the handle is never cancellable (ORACLE SC1). O-4 A: callbacks run on the System's executor, never on the clock's goroutine; no exported executor.
- D-10 and the Owner's order: the first of #821 slice 1 and #1182 slice B to merge after #871 introduces `Context.System()` with its single EXP-1 record; the other rebases and only uses it. **Fact (2026-11-11): #1182 slice B won (PR #1545, `b8639c1a`); this unit only uses `Context.System()` and adds no export on `Context`.** `typed.Context` does not re-export it.
- EXP-1: the new exports are `System.Scheduler`, `Scheduler`, `Scheduler.ScheduleOnce`, `Cancellable` and `ErrNilCallback`; any other is a Stop.

## 3. Mandatory Stops

Report to the TL and stop; never skip, delete or weaken a test:

1. A change outside the allowed files is needed.
2. A test fails on the current code where the scope expects it to pass; a scenario that fails is run on Pekko before it is classified (`gopekko-failure-triage`).
3. A slice projects over 400 authored lines (kit artifacts included): split it first.
4. A contract above (D-3 to D-5, O-1, O-3, O-4, ADR 0039) must change, or another export is needed.
5. `pekko-actor-typed` or `pekko-persistence` stops compiling (`scripts/each_module.sh go build ./...`).
6. A `judge/mutate_go.py` anchor breaks: the same PR updates it with the same contract; `scripts/check-judge-matrix.py` exit 1 is a Stop.

## 4. Mutation rule

Each behavior test names a mutation in the "Mutations" table of its PR; applied by hand in the working tree, never committed; the test must fail by assertion. Slice 1: T1 armed at `delay+1`, T1b the task runs twice, T2 `Cancel` never true, T2b `Cancel` true twice, T2c the clock timer is not stopped, T3 `IsCancelled` true after the run, T4 the state is swapped after the callback, T5 the zero delay waits for the clock, T5b the zero delay leaves the executor seam, T6 a 1ns delay is not cancellable, T7 the callback runs on the clock's goroutine, T8 the panic is reported twice, T15 a non-atomic swap, T16 no nil check, ACC a new scheduler per call (15 mutations; the mutation of `Context.System` is #1545's).

## 5. Conventions

- Header: Apache-2.0 plus one `// Ported from upstream/pekko/<path>.scala` line per file derived from one Pekko file (`.github/scripts/check-headers.sh`).
- Tests: go-specs matchers and spies, the existing `fakeClock` and `ctlExec`, no `time.Sleep`; unit only.
- Coverage at least 85% per module; no `//nolint`; gocyclo at most 10; README per the module README contract.
- Every sentence that says something was observed names the oracle line or the assertion; every Pekko citation is re-opened by the Dev before the PR.
