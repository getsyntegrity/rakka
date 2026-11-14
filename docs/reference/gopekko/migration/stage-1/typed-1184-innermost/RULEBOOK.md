# RULEBOOK: The innermost matching Typed supervisor decides (unit typed-1184-innermost, #1184 TYP-50)

Status: **approved at gate G2 by the Owner on 2026-11-06** for `docs/scopes/1184-innermost-supervision.md` (D-1 to D-6 as recorded in the scope; the ADR 0035 amendment of 2026-11-06 is merged, #1408). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 0.

## 1. Unit

- Capability: with several `Supervise(...).OnFailure(match, strategy)` layers, the **innermost layer that matches** decides, an exhausted inner limit is rethrown to the next outer layer, and a layer whose matcher equals an enclosing one's replaces it once the start has succeeded and whenever a handler returns a new supervised behavior (ADR 0035; Pekko `InterceptorImpl.deduplicate`). The defect: `joinLayers` never removes an equal layer among the layers just started and `PreStart` does not dedupe (scope, "The defect the oracle found"). The dedupe applies at three sites (start, turn, restart rebuild), never while the start is failing (N8).
- Sources, read at v1.7.1 and listed in `manifest.tsv`: `InterceptorImpl.scala:82-85, 90-98, 101-103, 106-122`; `Behavior.scala:76-86, 218-232`; `Supervision.scala:75-76, 78-83, 294-306, 335-337`; `SupervisionSpec.scala:201-206, 208-212, 448-453, 455-460, 462-468, 470-475, 477-483, 485-490, 849-880, 1270-1307`; oracles `docs/parity/oracles/typed-nested-supervision-2026-11-06.md` (N1 to N10, executed) and `typed-resume-2026-11-06.md` (R5c to R5e, executed).
- Governing decisions: ADR 0035 and its Amendment (2026-11-06); ADR 0033 D13; ADR 0017, ADR 0024 T6; Owner decisions D-1 to D-6 of the scope.
- Manifest: `manifest.tsv` follows the kit runner contract (`migration-kit/scripts/queue_runner.mjs`): `source` and `target` are the only columns it reads. Evidence classes: `SPEC` (a Pekko spec row, read, not run), `READ` (Pekko production code, read), `ORACLE` (a scenario executed on Pekko). Moved and `PORT-GAP` rows point at this RULEBOOK.
- Writable: exactly the slice table and "Allowed files" of the scope for the PR in hand. Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum`, the root `README.md`, nor the scope file.

## 2. Criteria and slices

| Slice | Criteria | Allowed files |
|---|---|---|
| 0 | kit package | `migration/stage-1/typed-1184-innermost/{manifest.tsv,RULEBOOK.md,RUNBOOK.md}` |
| 1a | T1a, T1b, T1c, T2, T3, T7 | `typed/interpreter.go`, `typed/nested_replacement_test.go` (new), `typed/restart_stopchildren_test.go`, `typed/restart_limit_test.go` |
| 1b | T4, T5, T6, T8 | `typed/nested_innermost_test.go` (new), `typed/restart_childcases_test.go` (the title of `S8` only) |
| 2 | T9 | `pekko-actor-typed/README.md`, `migration/stage-1/typed-1184-innermost/EXECUTION-RECORD.md`, `migration/stage-1/typed-1183-resume/EXECUTION-RECORD.md` (one sentence), `migration/cost-log.tsv` |

Paths under `typed/` are in `pekko-actor-typed/`. Raw estimates: 1a about 170, 1b about 110; re-estimated at 1.55 (the deviation measured in #976): 1a about 265, 1b about 170. Documentation slices are not scaled. Internal test files (`typed/<slice file>_internal_test.go`, `package typed`) are allowed in any slice with the same budget.

| Criterion | Test | Oracle |
|---|---|---|
| T1a | `TestNested_EqualMatcherReplacesAfterStart`, `TestNested_EqualMatcherInnerRethrowStops` | N1, N7, N6c |
| T1b | `TestNested_ReturnedStackDedupes` | N9 |
| T1c | `TestNested_RebuiltStackDedupes` | N10 |
| T2 | `TestNested_ReturnedSuperviseReplacesEqual` | N3 |
| T3 | `TestRestartStopChildren_Nested` case 3 and `TestRestartLimit` "a rethrown failure reaches the enclosing supervisor", adapted | N6d, N7d |
| T4 | `TestNested_ReturnedSuperviseKeepsDifferent` | N4 |
| T5 | `TestNested_InnermostMatchWins` (three rows) | R5c, R5d, R5e |
| T6 | `TestNested_ExhaustedLimitReachesOuter` (two rows) | N2, N7d |
| T7 | `TestRestartLimit_InitialStart`, unchanged | N8, N8d |
| T8 | the title of `S8` in `restart_childcases_test.go` | none |
| T9 | README and execution record | none |

## 3. Decisions and known gaps

- **Explicit `Stop()`: #1182** (D-4); **`ChildFailed(cause)` in a Typed parent: #1346**; **`SupervisionSpec.scala:1637`/`:1644`: #975**; **backoff nested in other layers: #986**; **a flattened `.OnFailure().OnFailure()` form: none, Owner D-3** (recorded in the README, no API); the rethrow through a match-all layer to a Classic parent is `TestRethrowParentKinds` (#976 slice 3c-2), only referred to.
- The deduplication is **inferred** from its effects in the oracle (N1, N3, N6c, N7, N9, N10); no log line shows the stack of supervisors. N7d and N6d on goPekko are inferred until slice 1a runs the adapted tests.
- This is Go unit evidence derived from Pekko specs and an oracle, **not V3**. Delivering #1184 does not close #1182, #1346, #975 or #986.

## 4. Mandatory Stops

Report to the TL and stop; never skip, delete or weaken a test:

1. A change in `pekko-actor` is needed.
2. A test other than the two named in D-2 (`TestRestartStopChildren_Nested` case 3, `TestRestartLimit` "a rethrown failure reaches the enclosing supervisor") fails after the change.
3. The assertions of those two tests would change. Only one matcher of one layer may change in each; each change is explained in the PR with its oracle scenario (N6c/N6d, N7/N7d).
4. The dedupe is applied while the start is failing: N8 and `TestRestartLimit_InitialStart` must stay green.
5. A file outside the allowed list must change, or a slice projects over 400 authored lines (split it first).

## 5. Mutation rule

Each behavior test names a mutation in a "Mutations" table of its PR (behavior, mutation, commit, failing test). The mutation is applied to production code in a disposable worktree, never committed, and the named test must fail by assertion; a test that survives its mutation is fixed. Slice 0 is docs only: no mutation applies.

| Id | Mutation | Test that must fail | Slice |
|---|---|---|---|
| M1a | `PreStart` does not dedupe the layers of the start | T1a rows | 1a |
| M1b | the turn site does not dedupe the returned layers | T1b | 1a |
| M1c | the restart site does not dedupe the rebuilt layers | T1c | 1a |
| M2 | `joinLayers` appends without replacing | T2 | 1a |
| M3 | a returned layer replaces every enclosing layer | T4 | 1b |
| M4 | `supervisorFor` picks the outermost matching layer | T5 | 1b |
| M5 | an exhausted limit stops the actor instead of searching outward (`selectRestart` answers -1) | T6 | 1b |
| M6 | dedupe while collecting the layers in `startInto` | `TestRestartLimit_InitialStart` (existing) | 1a |

## 6. Claims against evidence (rule of 2026-11-06)

A sentence that says something was observed names the log line or the assertion that shows it. What is only reasoned is marked **inferred**. QA treats an unbacked parity claim as CRITICAL.

## 7. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>` line per file derived from one Pekko file; files that mix several specs carry none. Checked by `.github/scripts/check-headers.sh`.
- Scenarios observe results through channels and gates; no `time.Sleep`, no polling, no `-race`. go-specs for assertions.
- No `//nolint`; cyclomatic complexity at most 10.
- Unknown behavior: `// TODO(TYP-50-edge): <reason>` and keep going (kit rule 8).
- Comments and names claim no more than is shown: no Typed parity.
