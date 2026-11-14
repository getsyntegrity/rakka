# Scope: Resume a supervised Typed Behavior (#1183, TYP-49)

Status: **approved by the Owner at gate G2 on 2026-11-06**, with D-1 to D-5 as the Owner decided them (D-1 kit run required; D-2 `Resume() ResumeStrategy`;
D-3 #1183 to `status:ready`; D-4 the death pact under Resume stays in #1379, with R1c2 as an explicit criterion there and in the README known gaps; D-5 R5c to R5e and the
title fix stay in #1184), plus two adjustments the Owner asked for the same day: `UnstashException` under Resume is owned by #927 ("Not carried here"), and slice 2 states the
cause goPekko reports for a failed start ("Slice 2: the cause of a failed start"). Written on 2026-11-06 against `develop` `49a30191`.
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: unlike #997, this scope adds production code that ports
`ResumeSupervisor` (`Supervision.scala:169-178`), so the Dev runs the migration kit (`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK under
`migration/stage-1/typed-1183-resume/`. The Dev is a separate session, QA is independent of both, and the TL does not implement.

Contract sources: [ADR 0035](../adr/0035-typed-supervision.md) (contract 14, S-5, the Resume guarantee, "Supervise"), [ADR 0034](../adr/0034-typed-behavior-boundary.md) (P3, P6, P8),
the reconciled criteria in [typed-supervision-stories-reconciliation-2026-10-26.md](../parity/typed-supervision-stories-reconciliation-2026-10-26.md) (#1183 section), the scope of
[#976](976-typed-restart.md) (the `Supervise` wrapper, the matcher, the layer chain) and the new oracle
[typed-resume-2026-11-06.md](../parity/oracles/typed-resume-2026-11-06.md) (R1 to R6; `C1` to `C9` below are its section 1). The Updates of #1183 prevail over its
original text; the reconciliation corrects its anchors (`SS:163`, `SS:834`, `AC:281`, `SS:1107`; the code comment at `SS:1671` is not evidence).

## What this delivers

`Supervise(b).OnFailure(match, Resume())`: a failure whose cause matches (a failed message, a failed signal handler) leaves the current behavior and its
state as they were, drops only the failed message, runs no `Setup`, delivers neither `PreRestart` nor `PostStop`, and leaves the children and watches
alone. A cause that does not match falls to the next enclosing supervisor and finally to the default Stop. A failure of the initial start is **not**
resumed: the actor fails to start with an initialization failure whose root cause is `ErrInvalidInitialBehavior`, not the user's error (ADR 0035 S-5).

## Proposed Go API (for the Owner at G2)

| Pekko | Go | Note |
|---|---|---|
| `SupervisorStrategy.resume` (`SO:35`) | `typed.Resume() ResumeStrategy` | same shape as `Restart() RestartStrategy`; `ResumeStrategy` implements the sealed `Strategy` |
| `Supervise(b).OnFailure(match, strategy)` | unchanged signature | accepts `ResumeStrategy` (today it returns nil for any strategy but `RestartStrategy`, `supervise.go:117-123`) |

No option on `ResumeStrategy` (logging is contract 13, #1179).

## Evidence: oracle, and what goPekko already does

Pekko result per scenario is in the oracle. The TL measured goPekko on `develop` `49a30191` with a **throwaway prototype** (37 lines of production in
`supervise.go`, `behavior.go`, `interpreter.go` and `restart_limit.go`, in a disposable worktree, never committed; the `restart_limit.go` change is redundant and dropped, so 34 lines are needed) and throwaway probes (`go test`, not committed). The prototype is
the size estimate below, not the implementation.

| Scenario | Without Resume (today) | With the prototype | Equal to Pekko |
|---|---|---|---|
| R1 state kept, no `Setup`, message dropped, subtype matches | `Resume()` does not exist; `OnFailure` returns nil | state 2, 2, 4; `setups=1` | yes |
| R2 no `PreRestart`, no `PostStop` | n/a | the only signal is `PostStop` at the termination | yes |
| R1b unmatched failure | default Stop with `PostStop` (exists, #956) | same, decider not consulted | yes |
| R1c1 failure in a signal handler | n/a | actor alive, state kept | yes |
| R1c2 death pact (`DeathPactException`) under Resume | a `Supervise` does not capture it (#1379) | the actor **stops** | **no** (Pekko resumes; owner #1379) |
| R3 children, `watch` | n/a | children unchanged; `Terminated` reaches the same incarnation | yes (`watchWith`: #1171) |
| R4a failed start under Resume | an error with a layer: the actor stops, the parent is not told | initialization failure to the parent's decider; cause `Is(ErrInvalidInitialBehavior)`, not the user's error | yes (the `ChildFailed` signal itself is #1346) |
| R4c outer `Restart.WithLimit(3)` around the failing start | n/a | no retry, `setups=1`, decider consulted | yes |
| R5a, R5b Resume and Restart nested, both orders; the rebuilt inner Resume stays in force | n/a | as Pekko | yes |
| R5c to R5e equal classes, narrower or broader inner | n/a | as Pekko | yes (tests owned by #1184) |
| R6 what the parent sees | n/a | resumed failure: nothing reaches the decider; unmatched: plain stop; failed start: decider with an initialization failure | yes |

`admit` (`restart_limit.go`) already admits a layer with the zero `RestartStrategy`, so a Resume layer needs no change there.

## Not carried here (each with its owner)

| Item | Evidence | Owner | Why it is not in #1183 |
|---|---|---|---|
| `DeathPactException` resumed (R1c2) | oracle R1c2; README "Not delivered" | #1379 | the capture of the death pact by a `Supervise` is that story; today the interpreter default-stops it before any layer is asked |
| Tests of the innermost-match rule: equal classes, narrower or broader inner Resume (R5c, R5d, R5e), and the fix of the spec title `restart_childcases_test.go:199` ("the decider is not consulted" must say it is inferred) | oracle R5c to R5e; `SS:1270` | #1184 | this scope does not touch `restart_childcases_test.go`; #1184 depends on #1183 for the Resume layer and takes the title fix |
| The failure line that Resume logs at ERROR (contract 13) | `S:172`, `SO:35` | #1179 | no per-actor logger accessor yet |
| `watchWith` children, a Typed child that fails | R3 | #1171, #1170, #1346 | tests use Classic children and `watch` |
| `UnstashException` under Resume (the supervisor continues with the behavior that the exception carries) | `S:174`; the restart paths `S:318, 327, 416`; `StashBufferImpl.scala:206` | #927 (TYP-14, plan item J-02 "failure per strategy (`UnstashException`)") | no Typed stash exists in goPekko yet; #927 adds the criterion (Owner, 2026-11-06) and #1183 does not implement it |
| Restart, limits, backoff | | #976 (delivered), #986 | out of scope of #1183 |

## Criteria, scenarios, sources and mutations

`S` = `Supervision.scala`, `AC` = `ActorContextSpec.scala`, `SS` = `SupervisionSpec.scala` (paths in the oracle). Test files are in `pekko-actor-typed/typed/`; all tests use go-specs,
channels or gates, never sleeps, no `-race`. A mutation is applied to production code in a disposable worktree and never committed (`qa-review`); the named test must fail by assertion.

| # | Criterion | Scenario | Source / oracle | Mutation | Slice |
|---|---|---|---|---|---|
| S1 | A matching failure (including a wrapped cause, the Go subtype) leaves behavior and state unchanged, drops only the failed message, and the next message is handled | `TestResume_KeepsStateAndDropsMessage` | R1; `S:169-178`, `SS:163`, `SS:834` | M1: `fail` restarts a Resume layer (state resets, `Setup` reruns) | 1a |
| S2 | No `Setup` rerun, no `PreRestart`, no `PostStop` until the actor stops | same test, signal log and setup counter | R2; `S:170-177`, `AC:281` | M2: the Resume branch calls `i.preRestart(c)` | 1a |
| S3 | A cause that does not match is not resumed: the actor stops, the behavior gets `PostStop`, the parent's decider is not consulted | `TestResume_UnmatchedStops` | R1b, R6b; `SS:502` adapted | M3: a Resume layer ignores its matcher | 1a |
| S4 | `OnFailure` accepts `ResumeStrategy`; the exported surface lists `Resume` and `ResumeStrategy` | `unhandled_internal_test.go` (U9) | | M4: export one more name from package `typed` (or rename `Resume`): the exported-surface list differs | 1a |
| S5 | A failure in a signal handler under Resume keeps the actor alive with its state | `TestResume_SignalFailure` | R1c1; `S:153-154` | M5: `OnTerminated` default-stops a failed turn instead of asking the layers | 1b |
| S6 | Children, watches and `Children()` are unchanged by a resumed failure; a later `Terminated` reaches the same incarnation | `TestResume_KeepsChildrenAndWatches` | R3; `S:175` | M6: the Resume branch stops the children or unwatches them | 1b |
| S7 | *(dropped)* `runtime.Goexit` in a handler is never resumed | none | #976 R4 pattern; ADR 0035 "Supervise" | M7 (drop the `ErrGoexit` exclusion of `Matcher.matches`) **survives**: `turn()` does not recover a `Goexit`, so `fail` never sees `ErrGoexit` (measured by QA of this scope). The criterion is covered by the Goexit row of `supervise_test.go` (#976 R4, `AnyError()` with `Restart()`); a Resume row would pin nothing new | n/a |
| S8 | A failed initial `Setup` under Resume is not resumed and not retried by an outer Restart: the actor fails to start, the parent's decider is consulted once with an initialization failure whose cause is `Is(ErrInvalidInitialBehavior)` and not the user's error; `Setup` ran once | `TestResume_FailedStart` (two rows: Resume alone, Resume under `Restart().WithLimit(3, ...)`) | R4a, R4c, R6c; `S:151-152`, `B:240-243`, ADR 0035 S-5 | M8: a failed start under a Resume layer stops the actor (`i.stop`) as for a Restart layer; M8b: the outer Restart retries the start | 2 |
| S9 | Resume nested with Restart, in both orders: each failure is taken by the innermost matching layer, and after a restart the rebuilt inner Resume is still in force | `TestResume_NestedWithRestart` (R5a, R5b) | R5a, R5b; `SS:201-211` | M9: a restart drops the rebuilt inner layers | 3 |
| S10 | README states what is implemented, the known gap "until #1379, Resume stops the actor on a death pact and Pekko resumes it (oracle R1c2)", the S-5 start failure, and points the line "No test combines a watcher with a panicking `PostStop`" at #1394 | review | | none (documentation) | 4 |

## Slices (one PR each, at most 400 authored lines, sequential)

The raw estimate is the TL's (the prototype for production); the second number is the raw one times 1.55, the deviation measured in #976. If a slice measures above 400 the Dev splits it before opening the PR.

| Slice | Delivers | Files | Criteria | Est. lines (raw → re-estimated) |
|---|---|---|---|---|
| **0. Kit package** | manifest, RULEBOOK, RUNBOOK | `migration/stage-1/typed-1183-resume/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | kit run | ~140 docs (#976 measured 144) |
| **1a. `Resume()` and the resume branch** | `Resume`, `ResumeStrategy`, `OnFailure` accepts it, the layer flag, the branch in `fail` (no `PreRestart`, behavior kept); the exported-surface list; the core tests | `typed/supervise.go`, `typed/behavior.go`, `typed/interpreter.go`, `typed/resume_test.go` (new), `typed/unhandled_internal_test.go` | S1, S2, S3, S4 | ~115 raw → ~175 (production ~25 → ~40, tests ~85 → ~130, surface list ~5) |
| **1b. Signals and children** | tests only | `typed/resume_children_test.go` (new) | S5, S6 | ~80 → ~125 |
| **2. Failed start** | the start path: a matching Resume layer ends the start with an initialization failure wrapping `ErrInvalidInitialBehavior`, ahead of any outer Restart | `typed/interpreter.go`, `typed/resume_start_test.go` (new) | S8 | ~90 raw → ~140 (production ~20 → ~30, tests ~70 → ~110) |
| **3. Nesting** | tests only | `typed/resume_nested_test.go` (new) | S9 | ~65 → ~100 |
| **4. Docs and record** | README (implemented, gaps, the #1394 pointer), `EXECUTION-RECORD.md`, cost-log row | `pekko-actor-typed/README.md`, `migration/stage-1/typed-1183-resume/EXECUTION-RECORD.md`, `migration/cost-log.tsv` | S10 | ~120 docs |

Order: 0, 1a, 1b, 2, 3, 4. `typed/interpreter.go` is edited by 1a and 2 (sequenced, not parallel); `typed/unhandled_internal_test.go` only by 1a. Internal test files
(`typed/<slice file>_internal_test.go`, `package typed`) are allowed in any slice under the rule of the #976 scope, with the same budget. Nothing under `pekko-actor/`,
`docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md` or `AGENTS.md` changes; a needed `pekko-actor` change is Mandatory Stop 1 and goes back to the TL. A test that fails on the
current code where the prototype passed is also a stop.

| Pair | Shared file | Resolution |
|---|---|---|
| 0, 1b, 3, 4 | none | each owns new files; `README.md` only in 4 |
| 1a, 2 | `typed/interpreter.go` | sequenced (1a first) |

## Slice 2: the cause of a failed start (Owner adjustment, 2026-11-06)

What Pekko reports when a `Setup` fails under Resume: `ActorInitializationException` whose root cause is `IllegalArgumentException("cannot use Same as initial behavior")`,
because `aroundStart` answers `Behaviors.same` (`S:151-152, 169-177`) and `validateAsInitial` rejects it (`B:240-243`); the parent sees it in `ChildFailed` (a Typed parent) or in its
decider (a Classic parent); the user's error is not in the chain (oracle R4a, R4c, R6c; ADR 0035 S-5).

What goPekko reports (measured with the prototype, `Is`/`As` on the cause the parent's decider receives): the start returns an error that **wraps `ErrInvalidInitialBehavior`** (`errors.Is` is true; `behavior.go:25-27`,
the existing Go counterpart of that `IllegalArgumentException`, already used when a `Setup` resolves to `Same`, Behavior.scala:235-243), and the runtime reports a failed `PreStart` as
`*actor.InitializationError` (`supervision_hooks.go:25`, `Unwrap` returns the cause). Result: `errors.As(cause, &*actor.InitializationError)` is true,
`errors.Is(cause, typed.ErrInvalidInitialBehavior)` is true, `errors.Is(cause, <the user's error>)` is false. That is the same shape as Pekko's: an initialization failure whose root cause is the
"not a valid initial behavior" validation, not the user's error. **It is not another cause, so no decision is needed.** Two differences, neither a cause: the message text
(`typed: Same is not a valid initial behavior` against `cannot use Same as initial behavior`; tests assert with `errors.Is`/`errors.As`, never on text), and the delivery to a Typed parent
as `ChildFailed`, which is #1346: until then a Typed parent sees the failure as its decider (M-A) does today, a Classic parent as in R6c. If the Dev finds that the reported cause
differs from this paragraph (for example the user's error in the chain, or a different sentinel), that is a stop and goes back to the TL as a decision.

## Allowed files

Exactly the paths in the slice table, plus the internal test files above. `docs/scopes/1183-typed-resume.md` is edited only by the planning PR.

## Dependencies

- Consumes: the `Supervise` wrapper, the matcher and the layer chain of #976 (delivered), `ErrInvalidInitialBehavior` (behavior.go:27), the default Stop of #956 (delivered), ADR 0035 accepted. The listed blockers of #1183 (#1063, #847, #956) are closed, and #976 is closed.
- **#1184 depends on #1183**: it needs the Resume layer for its tests (R5c to R5e, `SS:208`, `SS:849`, `SS:1270`) and takes the title fix of `restart_childcases_test.go:199`. This scope writes no test of those.

## Handoffs after G2 (published by the TL on 2026-11-06 after approval)

- **#1183** Update: R1c2 moves to #1379; R5c to R5e and the title fix to #1184; `UnstashException` to #927; the start-failure cause follows S-5; `status:blocked` to `status:ready`.
- **#927** Update: add `UnstashException` under a strategy (Resume continues with the exception's behavior, `S:174`; the restart paths, `S:318, 327, 416`) as a criterion.
- **#1184** Update: depends on #1183; owns R5c to R5e (oracle) and the title fix at `restart_childcases_test.go:199`.
- **#1379** Update: add oracle R1c2 (Pekko resumes the death pact under `onFailure[DeathPactException](resume)`; goPekko stops).

## Decisions (Owner, 2026-11-06)

- **D-1. Kit run: required** (production code is added).
- **D-2. API names:** `Resume() ResumeStrategy`.
- **D-3. Readiness:** #1183 to `status:ready`.
- **D-4. The death pact under Resume (R1c2) stays in #1379.** R1c2 becomes an explicit criterion of #1379, with the scenario of the oracle (#1397), and a known gap in the README (slice 4): until #1379, Resume stops the actor on a death pact and Pekko resumes it.
- **D-5. R5c to R5e and the title fix of `restart_childcases_test.go:199` stay in #1184.**
- **Adjustment 1.** `UnstashException` under Resume: the owner is #927 (TYP-14, plan item J-02). Noted above; #927 receives the criterion in an Update.
- **Adjustment 2.** The cause reported for a failed start: see "Slice 2: the cause of a failed start".

## Appetite

Six PRs (one kit package, two code, two test-only, one docs). The risk is the start path (slice 2), where the ordering of Resume and an outer Restart in `startRetryFor` must follow the oracle (R4c).

## Definition of Done

- Every criterion S1 to S6, S8, S9 has its test and S10 its text (S7 is dropped, above), and each behavior PR names its mutation in a "Mutations" table, reproduced by QA.
- `scripts/each_module.sh go test -count=1 ./...`, `scripts/check_deps.sh` and the complexity lint green, with the commands in `AGENTS.md`.
- Diff coverage ≥ 85% on new/modified lines; per-module coverage ≥ 85%; cyclomatic complexity ≤ 10 in every new/modified function; `typed` still imports only `pekko-actor`.
- Every ported file keeps one `Ported from` line; the README states the gaps above and does not claim parity beyond the oracle.
- Evidence bound to a pushed SHA in each PR, as `dev-scope` requires.
