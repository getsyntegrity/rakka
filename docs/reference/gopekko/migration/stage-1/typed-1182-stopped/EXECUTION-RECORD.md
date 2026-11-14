# EXECUTION RECORD: Typed `Stopped` and `StoppedWith`, slice A (#1182, TYP-48)

Scope `docs/scopes/1182-typed-stopped.md` (PR #1524, approved at gate G2 by the Owner on 2026-11-10). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`). The code is PR #1529 (`Refs #1182`, never `Closes`); this PR is the kit package. Written by the Dev of slice A; the QA reviews are not in it.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`), one Dev session; wall clock and tokens were not measured and stay `unknown` in the cost-log row.
2. Inputs: `Behaviors.scala:76, 90`, `BehaviorImpl.scala:62-65, 99, 105-113`, `ActorAdapter.scala:173-176, 299-305, 320-321, 342-362`, `Behavior.scala:235-243, 248-249`, `StopSpec.scala:33-100`.
3. Outputs: `Stopped`, `StoppedWith`, the stop in the interpreter (`pekko-actor-typed/typed/behavior.go`, `interpreter.go`), `stopped_test.go`, `ExampleStoppedWith`, the README section "Stopped", this kit package and one cost-log row.
4. Deviations from the scope: the kit package is its own PR (the scope listed it under slice A) because slice A measured 396 authored lines without it (the AGENTS.md budget is 400). The scope cites `Behavior.scala:236-244`; the lines are 235-243.

## 2. RED evidence

The tests were written against the finished interpreter and then proven by the mutations below (unmutated green, mutated red), not before it; this is recorded as a process deviation from strict TDD.

## 3. Mutations (RULEBOOK section 5)

Applied by hand in a disposable edit of `interpreter.go`, restored after each run (diff against a backup is empty), never committed.

| Id | Criterion | Mutation | Failing tests |
|---|---|---|---|
| M1 | R1 | `stopWith` calls `PostStop` at once | ChildrenTerminateBeforePostStop, InsideSupervise, NoCallback, SetupResultOfAHandler, SignalThenCallback |
| M2 | R2 | the callback runs before the signal | ChildrenTerminateBeforePostStop, SetupResultOfAHandler, SignalThenCallback |
| M3 | R3a, R3c | `PreStart` stops without keeping the callback | CallbackAfterSetup, CallbackFromInitialBehavior |
| M4 | R3b | `turn` stops without keeping the callback | CallbackFromReceive, CallbackRunsOnce, ChildrenTerminateBeforePostStop, SignalThenCallback |
| M5 | R3c | `startInto` rejects the stopped marker | CallbackAfterSetup, CallbackFromInitialBehavior, RestartRebuildsIntoStopped, ... |
| M6 | R3d | the callback also runs at the stop request | CallbackRunsOnce and seven more |
| M7 | R4 | `Stopped()` delivers the signal twice | InsideSupervise, NoCallback |
| M8 | R5 | `turn` fails a `Stopped` result | CallbackFromReceive, CallbackRunsOnce, InsideSupervise, ... |
| M9 | restart | the restart path ignores the marker | RestartRebuildsIntoStopped |
| M10 | Setup result | the Setup-result path ignores the marker | SetupResultOfAHandler |

## 4. Gates

On the head of PR #1529 (macOS arm64, go1.27.2): `gofmt`, `go vet`, `go test -race`, coverage 99.4% of the module, diff coverage 34/34 changed statement lines (own script, `diff-cover` needs the network), `golangci-lint` v2.14.0 with gocyclo 0 and 3 pre-existing staticcheck findings in other test files, `scripts/check_deps.sh`, `make ci-fast` and `scripts/check-judge-matrix.py` (exit 0, CURRENT 21/21). `check-headers.sh` reports 7 errors in other modules, none in `pekko-actor-typed`. `unshare -rn` is not available on macOS. The full table is in the PR body.

## 5. Size (authored lines)

| PR | Content | Estimate | Measured |
|---|---|---|---|
| #1529 | slice A, code, tests, README | ~330 (with the kit package) | 396 |
| this PR | kit package, record, cost-log row | n/a | 144 |

## 6. Evidence class and pending approval (ADR 0033 D13)

- **SPEC/READ, read against Pekko v1.7.1 and not run:** every row of the manifest. Not V3; no Typed differential run exists for `Stopped`.
- **Alternative evidence, pending the Owner's per-row approval:** the Go scenarios on the shared hierarchy (R1 children before `PostStop`, R5 `Stopped` inside `Supervise`), and the two extra scenarios (Setup result of a handler, restart into `Stopped`).
- **Not claimed:** parity of the module with Pekko Typed; the dead letters behind a `Stopped` (slice B); the behavior of a `nil` callback (a conscious difference, `TODO(TYP-48-nil-callback)`).

## 7. Not delivered, with owners

Slice B of #1182 (held inputs as dead letters, `Context.System()` or its use; after #871 and PR #1525). The interruption of an `unstashAll` replay: #927. `PORT-GAP(TYP-48-interceptor-978)`: #978. `PORT-GAP(TYP-48-stop-variants-1063)`: #1063.

## 8. Criteria against the scope

| Criterion | Result |
|---|---|
| R1 to R5 | delivered (#1529) |
| R3e, R6 | PORT-GAP, with owners |
| R7, R8 | slice B |

## 9. Recurring failures (`AGENTS.md`, kit rule 6)

None in this slice.
