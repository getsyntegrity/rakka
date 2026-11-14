# EXECUTION RECORD: the scheduler, first cut, slice 1 (#821, TIM-02)

Scope `docs/scopes/821-scheduler-first-cut.md` (PR #1523, approved at gate G2 by the Owner on 2026-11-10). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`). This PR is the kit package and the record of slice 1; the code is PR #1544 (`Refs #821`, never `Closes`). Slices 1b and 2 add their own rows to this record when they merge.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`), one Dev session; wall clock, tokens and subagents were not measured and stay `unknown` in the cost-log row.
2. Inputs: `Scheduler.scala:423-436, 479, 525-542`, `LightArrayRevolverScheduler.scala:163-168, 205-224, 372-424`, `AbstractDispatcher.scala:55-63`, `ActorCell.scala:187`, the oracle `docs/parity/oracles/scheduler-lifecycle-2026-11-10.md` (SC1, SC3o).
3. Outputs: `scheduler.go`, `scheduler_once.go` (five new exports; `Context.System` was introduced by #1182 slice B, PR #1545, and this unit only uses it), the tests (`scheduler_internal_test.go`), the README item "Scheduler, first cut", the corrections to the scope, this record and one cost-log row.
4. Deviations from the scope: the tests are one internal file that reuses `fakeClock` and `ctlExec` (the scope listed an API test file too); no other.

## 2. RED evidence

The tests were written first and did not compile against `develop` (`undefined: Cancellable`, `sys.Scheduler undefined`, `undefined: logSchedulerPanic`, `undefined: ErrNilCallback`), then the code made them pass. This is a compile failure, not an assertion failure; the assertion failures are in section 3.

## 3. Mutations (RULEBOOK section 4)

Origin of the figures: declared by the Dev; reproduced by the independent QA of #1544 only where its published comment says so (the QA reports are not in this record). Applied by hand, **declared by the Dev**: by a script outside the repository; restored after each run (the files compare equal to the originals); never committed. Each one fails by assertion (**declared by the Dev**: the test itself takes 0.00s and the 7.5s of each row is the `-race` build):

| Id | Mutation | Failing tests |
|---|---|---|
| T1, T1b | armed at `delay+1`; the task runs twice | `T1` |
| T2, T2b, T2c | `Cancel` never true; true twice; the clock timer is not stopped | `T2` |
| T3, T4 | `IsCancelled` true after the run; the state is swapped after the callback | `T3`; `T4` |
| T5, T5b, T6 | the zero delay waits for the clock; it leaves the executor seam; a 1ns delay is not cancellable | `T5` (three delays); `T6` |
| T7, T8, T15, T16 | the callback runs on the clock's goroutine; the panic is reported twice; a non-atomic swap; no nil check | `T7`; `T8`; `T15`; `T16` (three delays) |
| ACC | a new scheduler per call | `TestSchedulerAccess` (first row) |

The mutation CTX of the first head (`Context.System` returns nil) is gone: that export is #1545's. Fifteen mutations remain (T1, T1b, T2, T2b, T2c, T3, T4, T5, T5b, T6, T7, T8, T15, T16, ACC); rerun on the rebased head, **declared by the Dev**.

## 4. Gates

Origin of the figures: declared by the Dev; reproduced by the independent QA of #1544 only where its published comment says so. On the head of PR #1544 (macOS arm64, go1.27.2): `gofmt`, `go vet`, `go test -race -count=1 ./...` of `pekko-actor` and of `pekko-actor-typed` (exit 0), coverage 98.4% of the module, diff coverage 100% (29 lines, `diff-cover` 11.0.0), `golangci-lint` v2.14.0 (0 issues, gocyclo at most 10), `scripts/check_deps.sh` (0), `make ci-fast` (0), `scripts/check-judge-matrix.py` (exit 0, `anchor_problems('pekko-actor/actor')` empty, goPekko CURRENT stale by the rule). `.github/scripts/check-headers.sh` could not be run from the Dev sandbox; its rule was checked by hand on the new files. Not run: `unshare -rn` (macOS).

## 5. Size (authored lines)

| PR | Content | Estimate | Measured |
|---|---|---|---|
| #1544 | slice 1: code, tests, README, scope corrections (rebased on `b8639c1a`, head `2579ec3859b14adf252df12ab8fbcd242f8adf17`) | ~345 | 393 (378 added, 15 deleted) |
| this PR | kit package, record, cost-log row | n/a | 138 |

## 6. Evidence class and pending approval (ADR 0033 D13)

- **ORACLE, executed on Pekko 1.7.1, Java 1.8.0_504:** the zero and negative delays and the `+1ns` control (SC1), the `Cancel` values (SC3o (a), (b), (c)), as Go tests on the fake clock.
- **Go-only, pending the Owner's approval:** the executor hop (T7), the panic report (T8), the race (T15) and `ErrNilCallback` (T16).
- **Not claimed:** parity of the module, V2 or V3, the close at `Terminate`. The Go campaign after the merge is the Owner's step; until then the Go CURRENT of the checker is stale.

## 7. Not delivered, with owners

`PORT-GAP(TIM-13-close)`: the close, `ErrSchedulerClosed` and the registry of pending tasks (slice 1b, merges right after slice 1). `ScheduleTellOnce` and the oracle SC5once tests (slice 2). Fixed delay and fixed rate (#822, #823), keyed timers (#860), `ReceiveTimeout` (#861), `SchedulerTask.cancelled` (#1077), tick rounding (ADR 0039 O-7, accepted PORT GAP).

## 8. Criteria against the scope

T1 to T8, T15, T16 and the access rows delivered in #1544. T9 to T14, T17 belong to slices 1b and 2.

## 9. Recurring failures (`AGENTS.md`, kit rule 6)

None. Note for the Dev sandbox: a command that names `.github/` or contains a heredoc piped into git-like programs can be refused; run the scripts one at a time.
