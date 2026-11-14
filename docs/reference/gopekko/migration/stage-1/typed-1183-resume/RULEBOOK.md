# RULEBOOK: Typed Resume of a supervised behavior (unit typed-1183-resume, #1183 TYP-49)

Status: **approved at gate G2 by the Owner on 2026-11-06** for `docs/scopes/1183-typed-resume.md` (merged into `develop` `fa5b4bfb`; D-1 to D-5 and two adjustments as recorded in the scope). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 0.

## 1. Unit

- Capability: `Supervise(b).OnFailure(match, Resume())`. A failure whose cause matches (a failed message, a failed signal handler) leaves the behavior and its state as they were, drops only the failed message, runs no `Setup`, sends neither `PreRestart` nor `PostStop`, and leaves children and watches alone. A cause that does not match falls to the next enclosing supervisor and finally to the default Stop. A failed initial start is not resumed: the actor fails to start with an initialization failure whose root cause is `ErrInvalidInitialBehavior`, not the user's error (ADR 0035 S-5). Source: Pekko `Supervision.scala:56-58, 139-156, 169-178`, `SupervisorStrategy.scala:35, 125`, `Behavior.scala:240-243` at v1.7.1, read, and the oracle `docs/parity/oracles/typed-resume-2026-11-06.md` (R1 to R6, executed).
- Governing decisions: ADR 0035 (contract 14, S-5, the Resume guarantee, "Supervise"); ADR 0034 P3, P6, P8; ADR 0033 D13; Owner decisions D-1 to D-5 of the scope.
- Manifest: `manifest.tsv` follows the kit runner contract (`migration-kit/scripts/queue_runner.mjs`): `source` and `target` are the first two columns and the only ones it reads. Evidence classes: `SPEC` (a Pekko spec row, read, not run), `READ` (Pekko production code, read), `ORACLE` (a scenario executed on Pekko). Moved and `PORT-GAP` rows point at this RULEBOOK, whose recorded disposition is their output.
- Writable: exactly the slice table and "Allowed files" of the scope for the PR in hand. Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum`, the root `README.md`, nor the scope file.

## 2. Criteria and slices

| Slice | Criteria | Allowed files |
|---|---|---|
| 0 | kit package | `migration/stage-1/typed-1183-resume/{manifest.tsv,RULEBOOK.md,RUNBOOK.md}` |
| 1a | S1, S2, S3, S4 | `typed/supervise.go`, `typed/behavior.go`, `typed/interpreter.go`, `typed/resume_test.go`, `typed/unhandled_internal_test.go` |
| 1b | S5, S6 (tests only) | `typed/resume_children_test.go` |
| 2 | S8 | `typed/interpreter.go`, `typed/resume_start_test.go` |
| 3 | S9 (tests only) | `typed/resume_nested_test.go` |
| 4 | S10 | `pekko-actor-typed/README.md`, `migration/stage-1/typed-1183-resume/EXECUTION-RECORD.md`, `migration/cost-log.tsv` |

Paths under `typed/` are in `pekko-actor-typed/`. S7 (`runtime.Goexit` never resumed) is dropped: mutation M7 survives because `turn()` does not recover a `Goexit`; the Goexit row of `supervise_test.go` (#976 R4) covers it. Internal test files (`typed/<slice file>_internal_test.go`, `package typed`) are allowed in any slice with the same budget.

## 3. Decisions and known gaps

- **Death pact under Resume (R1c2): #1379** (D-4). Until then Resume stops the actor and Pekko resumes it; stated in the README (S10).
- **R5c to R5e and the title fix of `restart_childcases_test.go:199`: #1184** (D-5).
- **`UnstashException` under Resume: #927** (no Typed stash yet). **Failure log at ERROR: #1179.** **`watchWith` and Typed children, `ChildFailed` to a Typed parent: #1171, #1170, #1346.** Restart, limits, backoff: #976 (delivered), #986.
- **Failed start.** The reported cause is an `*actor.InitializationError` wrapping `ErrInvalidInitialBehavior`; the user's error is not in the chain. Tests assert with `errors.Is`/`errors.As`, never on message text.
- This is Go unit evidence derived from Pekko specs and an oracle, **not V3**. Delivering #1183 does not close #1379, #1184, #927, #1179, #1171, #1170 or #1346.

## 4. Mandatory Stops

Report to the TL and stop; never skip, delete or weaken a test:

1. A change in `pekko-actor` is needed.
2. The cause of a failed start differs from the scope's paragraph "Slice 2: the cause of a failed start" (for example the user's error in the chain, or another sentinel).
3. A test fails on the current code where the TL's prototype passed.
4. A file outside the allowed list must change, or a slice projects over 400 authored lines (split it first).

## 5. Mutation rule

Each behavior test names a mutation in a "Mutations" table of its PR (behavior, mutation, commit, failing test), taken from M1 to M9 of the scope. The mutation is applied to production code in a disposable worktree, never committed, and the named test must fail by assertion; a test that survives its mutation is fixed. Slice 0 is docs only: no mutation applies.

## 6. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>` line per file derived from one Pekko file; files that mix several specs carry none. Checked by `.github/scripts/check-headers.sh`.
- Scenarios observe results through channels and gates; no `time.Sleep`, no polling, no `-race`. go-specs for assertions.
- No `//nolint`; cyclomatic complexity at most 10.
- Unknown behavior: `// TODO(TYP-49-edge): <reason>` and keep going (kit rule 8).
- Comments and names claim no more than is shown: no Typed parity.
