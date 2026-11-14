# Scope: The innermost matching Typed supervisor decides (#1184, TYP-50)

Status: **approved by the Owner at gate G2 on 2026-11-06** with D-1 to D-6 as decided under "Decisions" (D-1 kit run required; D-2 the two #976 tests adapted under three conditions;
D-3 no flattened form; D-4 no explicit `Stop` yet; D-5 the ADR 0035 amendment in a separate PR that **merges before slice 1a**; D-6 #1184 to `status:ready`). Written on 2026-11-06 against
`develop` `e114ea95` (still valid at `e4c5dc22`: `interpreter.go` and the cited tests did not change in between).
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: this scope changes production code (a defect found by the oracle, below), so the Dev runs the
migration kit (`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK under `migration/stage-1/typed-1184-innermost/`. The Dev is a separate session, QA is independent of both,
and the TL does not implement.

Contract sources: [ADR 0035](../adr/0035-typed-supervision.md) ("Additional guarantees": the innermost first, the equal class makes the enclosing one disappear), the reconciled
criteria in [typed-supervision-stories-reconciliation-2026-10-26.md](../parity/typed-supervision-stories-reconciliation-2026-10-26.md) (#1184 section), the scopes of
[#976](976-typed-restart.md) and [#1183](1183-typed-resume.md), the oracles [typed-resume-2026-11-06.md](../parity/oracles/typed-resume-2026-11-06.md) (R5a to R5e) and the new
[typed-nested-supervision-2026-11-06.md](../parity/oracles/typed-nested-supervision-2026-11-06.md) (N1 to N10; `C1` to `C8` below are its section 1). The Updates of #1184 prevail over its
original text; the reconciliation corrects its criteria (no `Stop` is declared in `SS:208`, `SS:849`; `SS:1637`/`SS:1644` go to #975).

## What this delivers

When several `Supervise(...).OnFailure(match, strategy)` layers are declared (Go has the nested form only), a failure is handled by the **innermost layer that matches**, an exhausted
inner limit is rethrown to the next outer layer, and a layer whose matcher **equals** an enclosing one's replaces it once the start has succeeded and whenever a handler returns a new
supervised behavior (ADR 0035, Pekko `InterceptorImpl.deduplicate`, `I:106-122`). The rule is pinned by tests with overlapping matchers, the part #1183's rows could not show.

## The defect the oracle found

Pekko's `preStart` runs `aroundStart` and **then** `deduplicate` (`I:82-85`), and `deduplicate` also runs after every message and every signal (`I:90-98`, `I:101-103`, `I:106-122`): during a
failing start every supervisor is consulted (oracle N8: 4 runs, equal classes or not), and once a stack has started an equal-class inner supervisor replaces the outer one (N1, N6c, N7:
with an inner Restart whose limit is used and an outer layer of the same class, the actor **stops**). That holds for the initial start (N1), for a stack that a handler returns (N9)
and for a stack that a restart rebuilds (N10). goPekko's `joinLayers` removes from the layers already in force those that an added layer equals (`interpreter.go:269` for a handler's result,
`:361` for a restart), but it never removes an equal layer **among the layers that were just started**, and the initial start (`PreStart`, `interpreter.go:74`) does not dedupe at all: the
outer layer stays and the rethrown limit reaches it.

| Scenario (oracle) | Pekko | goPekko `develop` `e114ea95` (throwaway probes, except where marked) |
|---|---|---|
| R5c, R5d, R5e overlapping and equal classes, Resume and Restart | the innermost matching layer decides | equal |
| N2 inner narrower Restart, limit used, outer broader Resume | the outer Resume keeps the state | equal |
| N3 a handler returns a supervise of an equal class | the enclosing Restart is replaced | equal |
| N4 a handler returns a supervise of a different class | both kept | equal |
| N8 failing first `Setup`, nested layers | 4 runs | equal (existing `TestRestartLimit_InitialStart`) |
| **N1** (start) inner `Restart.WithLimit(1)`, outer `Resume`, equal classes | **stops** at the limit | **keeps running** (resumed) |
| **N7** (start) inner `Restart.WithLimit(1)`, outer `Restart`, equal classes | **stops** at the limit | **the outer restarts** (shown by the existing test `TestRestartLimit`, not by a probe) |
| **N6c** (start) inner `WithStopChildren(false).WithLimit(0)`, outer `Restart`, equal classes | **stops** | **the outer restarts** and stops the children (shown by the existing test `TestRestartStopChildren_Nested` case 3) |
| **N9** (turn) a handler returns a stack that holds an equal pair | **stops** at the limit | **keeps running** |
| **N10** (restart) a restart rebuilds a stack that holds an equal pair | the second `E1` restarts (`setup#4`) | the second `E1` is resumed (3 `Setup` runs, `keeps running`) |
| N7d, N6d the same with different classes | the outer takes the rethrow | *inferred* equal (reasoned from `selectRestart`; the adapted tests of T3 pin it, they are run by the Dev) |
| N5a flattened against nested | the same events | no flattened form exists in Go (`OnFailure` returns a `Behavior`) |

A throwaway prototype (never committed, a disposable worktree, 15 lines of production in `interpreter.go`: a `dedupeLayers` applied at the **three** sites) makes N1 and N9 stop and N10 reach
`setup#4` as Pekko does, keeps N8 at 4 runs, and fails exactly **two existing tests**, both written in #976 with two equal `AnyError()` layers. Removing one site at a time showed that each
site is necessary and is seen by exactly one row: without the `PreStart` site N1 keeps running; without the turn site N9 keeps running; without the restart site N10 reaches only `setup#3` and keeps running.

- `TestRestartStopChildren_Nested`, case 3 ("the inner layer (keep, limit 0) rethrows: the outer layer (default) restarts and stops the children"), `restart_stopchildren_test.go`;
- `TestRestartLimit`, "a rethrown failure reaches the enclosing supervisor, which restarts everything it wraps", `restart_limit_test.go:77`.

Both encode a behavior Pekko does not have for equal classes (N6c, N7); with different classes (N6d, N7d) Pekko behaves as they say. A naive version that dedupes while *collecting* the
layers (in `startInto`) also breaks `TestRestartLimit_InitialStart` (N8), which is why the dedupe belongs after a stack has started.

## Not carried here (each with its owner)

| Item | Owner | Why not here |
|---|---|---|
| An explicit `Stop()` strategy (the issue's AC-R1-1 and AC-R2-1 say "stop for the rest" and "stop for RuntimeException") | #1182 | not delivered; the criteria use `Resume`, `Restart` and the default Stop |
| `ChildFailed(cause)` observed by a parent in the stop rows | #1346 | the rig has no Typed-parent `ChildFailed` yet |
| The pair `SS:1637`/`SS:1644` (adapter-function failures through nested supervisors) | #975 | reconciliation, criterion 3 |
| A flattened `.OnFailure().OnFailure()` form | Owner (D-3) | Go's `OnFailure` returns a `Behavior`; ADR 0035 fixes `Supervise(b).OnFailure(match, strategy)` |
| Backoff nested in other layers | #986 | not delivered |
| The rethrow of an exhausted `withLimit` through a match-all layer to a Classic parent (AC-R4-3) | delivered by #976 slice 3c-2 | `TestRethrowParentKinds` (W1, `restart_rethrow_test.go`); this scope only refers to it |

## Criteria, scenarios, sources and mutations

`I` = `InterceptorImpl.scala`, `S` = `Supervision.scala`, `SS` = `SupervisionSpec.scala` (paths in the oracle). Test files are in `pekko-actor-typed/typed/`; go-specs, channels or gates,
never sleeps, no `-race`. A mutation is applied to production code in a disposable worktree and never committed (`qa-review`); the named test must fail by assertion.

| # | Criterion | Scenario | Source / oracle | Mutation | Slice |
|---|---|---|---|---|---|
| T1a | Initial start: once the start has succeeded, an inner layer whose matcher equals an enclosing one's replaces it: an inner `Restart.WithLimit(1)` with an outer `Resume` (N1) or `Restart` (N7) of the same matcher stops the actor when the limit is used; the equal form of `TestRestartStopChildren_Nested` case 3 as its own test (N6c) | `TestNested_EqualMatcherReplacesAfterStart` (rows N1, N7) and a separate test function `TestNested_EqualMatcherInnerRethrowStops` (N6c; not a row of the loop of `TestRestartStopChildren_Nested`, whose loop asserts `setup:2` and a child state for every row, so the existing assertions stay untouched) | N1, N6c, N7; `I:82-85`, `I:106-122`, `S:78-83`, `S:335-337` | M1a: `PreStart` does not dedupe the layers of the start | 1a |
| T1b | Turn: a stack that a handler returns and that itself holds an equal pair is deduplicated when it starts (N9: stops at the limit) | `TestNested_ReturnedStackDedupes` | N9; `I:90-98`, `I:106-122` | M1b: the turn site does not dedupe the returned layers | 1a |
| T1c | Restart: a stack that a restart rebuilds and that holds an equal pair is deduplicated (N10: the second `E1` restarts, `setup#4`) | `TestNested_RebuiltStackDedupes` | N10; `I:106-122` | M1c: the restart site does not dedupe the rebuilt layers | 1a |
| T2 | A supervised behavior returned by a handler replaces an enclosing one of an equal matcher (N3) | `TestNested_ReturnedSuperviseReplacesEqual` | N3; `I:90-98`, `SS:1270` | M2: `joinLayers` appends without replacing | 1a |
| T3 | The two existing tests keep their **assertions unchanged** and change only the matcher of one layer, so that the two layers differ (as N6d and N7d); the equal-matcher forms that do what Pekko does are T1a (N6c, N7). The PR explains each change with its oracle scenario | `TestRestartStopChildren_Nested` case 3 and `TestRestartLimit` "a rethrown failure reaches the enclosing supervisor", adapted | N6c/N6d, N7/N7d | none of its own: M1a fails the T1a rows | 1a |
| T4 | A returned supervised behavior of a different matcher is added: both are kept, the narrower inner handles its class, the outer handles the rest and its restart rebuilds from its own initial behavior | `TestNested_ReturnedSuperviseKeepsDifferent` | N4; `SS:1270` (the surviving supervisor), `I:113-118` | M3: a returned layer replaces every enclosing layer | 1b |
| T5 | With overlapping matchers the innermost matching layer decides: outer `Restart` for E1 with inner `Resume` for E2 (R5d), outer `Restart` for E2 with inner `Resume` for E1 (R5e), equal classes (R5c) | `TestNested_InnermostMatchWins` (three rows) | R5c, R5d, R5e of the Resume oracle (the specs `SS:201-211` and `SS:849` use disjoint classes and back R5a and R5b only) | M4: `supervisorFor` picks the **outermost** matching layer (the mutation that #1403's QA showed #1183's disjoint rows cannot kill) | 1b |
| T6 | An exhausted inner limit is rethrown to the next outer layer of a different matcher, which handles it by its own strategy: `Resume` (N2) and `Restart` (N7d) | `TestNested_ExhaustedLimitReachesOuter` (two rows) | N2, N7d; `S:335-337` | M5: an exhausted limit stops the actor instead of searching outward (`selectRestart` answers -1) | 1b |
| T7 | A failing first `Setup` is consulted by every layer in turn, equal matchers or not (4 runs under an outer `WithLimit(3)`) | the existing `TestRestartLimit_InitialStart` ("a failing first Setup in a nested Supervise is rethrown outward"), unchanged; a regression guard for T1a | N8; `S:294-306` | M6: dedupe while collecting the layers in `startInto`: the existing test fails | 1a (verification) |
| T8 | The title of `S8` of `restart_childcases_test.go:199` says that the decider's non-consultation is inferred | the spec title | #1403/#1390 QA | none (title) | 1b |
| T9 | README and execution record state the rule, the oracle, the gaps and the carried wording fixes (below) | review | | none (documentation) | 2 |

**Carried wording fixes** (the QA warnings of #1404, Owner instruction of 2026-11-06 to fix them in the next PR that touches the README): the README Status line also names #1183 (TYP-49) and its
sentence about "three executed Pekko oracles" counts the Resume and nested-supervision oracles; "Resume stops the actor on a death pact" is marked as inferred, `TODO(#1379)`; and the record of #1183
says that the #1400 delta review reproduced M1, M2, M3 and M5 (M4 was not re-run).

## Slices (one PR each, at most 400 authored lines, sequential)

The raw estimate is the TL's (the prototype for production); the second number is the raw one times 1.55, the deviation measured in #976. The documentation slices (0 and 2) are not scaled, as in #976 and
#1183 (measured sizes: 144 for slice 0 of #976, 111 for slice 0 of #1183, 120 for slice 4 of #1183, which is 113 added plus 7 deleted). If a slice measures above 400 the Dev splits it first.

| Slice | Delivers | Files | Criteria | Est. lines (raw → re-estimated) |
|---|---|---|---|---|
| **0. Kit package** | manifest, RULEBOOK, RUNBOOK | `migration/stage-1/typed-1184-innermost/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | kit run | ~111 docs (#1183 measured 111) |
| **1a. Dedupe at the three sites** | `dedupeLayers` applied to the layers of a stack that has started: at `PreStart`, at the turn and at the restart rebuild; the two #976 tests adapted (D-2); the tests T1a, T1b, T1c, T2 | `typed/interpreter.go`, `typed/nested_replacement_test.go` (new), `typed/restart_stopchildren_test.go`, `typed/restart_limit_test.go` | T1a, T1b, T1c, T2, T3, T7 | production ~15 → ~25; tests ~140 → ~215; adapting the two tests ~15 → ~25; ~170 raw → ~265 |
| **1b. Innermost wins** | tests only, and the title | `typed/nested_innermost_test.go` (new), `typed/restart_childcases_test.go` (the title only) | T4, T5, T6, T8 | ~110 → ~170 |
| **2. Docs and record** | README, `EXECUTION-RECORD.md`, cost-log row, the carried wording fixes (three fixes across the README and the #1183 record) | `pekko-actor-typed/README.md`, `migration/stage-1/typed-1184-innermost/EXECUTION-RECORD.md`, `migration/stage-1/typed-1183-resume/EXECUTION-RECORD.md` (one sentence), `migration/cost-log.tsv` | T9 | ~110 docs |

Order: **the ADR 0035 amendment (D-5, its own PR) merges first**, then 0, 1a, 1b, 2. `typed/interpreter.go` is edited by 1a only; the two #976 test files are edited by 1a only. Internal test files
(`typed/<slice file>_internal_test.go`, `package typed`) are allowed in any slice under the rule of the #976 scope, with the same budget. Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`,
`.github/`, `CLAUDE.md` or `AGENTS.md` changes in the four slices; a needed `pekko-actor` change is Mandatory Stop 1. A test other than the two named in D-2 that fails after the change is a stop and goes
back to the TL.

| Pair | Shared file | Resolution |
|---|---|---|
| 0, 1a, 1b, 2 | none | 1b owns new files and one title line; `README.md` only in 2 |
| amendment, 1a | none (the amendment edits `docs/adr/0035-typed-supervision.md` and the Owner table) | sequenced: the amendment first |

## Allowed files

Exactly the paths in the slice table, plus the internal test files above. `docs/scopes/1184-innermost-supervision.md` is edited only by the planning PR.

## Dependencies

- Consumes: the `Supervise` wrapper and the chain of #976, `Resume()` of #1183 (closed), the default Stop of #956. All delivered.
- **Precondition of slice 1a (D-5):** the amendment of ADR 0035 ("the deduplication happens after a stack has started, not while the start is failing") is merged. It is a separate docs PR, outside the four slices.
- **Nothing depends on #1184 in this unit**; #1182 (explicit `Stop`) will reuse the rows with `Stop()` once it exists.

## Handoffs (published by the TL on 2026-11-06 after approval)

- **#1184** Update: criteria T1a to T9 replace AC-R1-1 to AC-R4-3 (the reconciled criteria); the defect and the two adapted tests; `status:blocked` to `status:ready`.
- **#1182** Update: when `Stop()` exists, the rows of `TestNested_InnermostMatchWins` get a `Stop` variant (the issue's AC-R1-1 and AC-R2-1).

## Decisions (Owner, 2026-11-06)

- **D-1. Kit run: required** (production code).
- **D-2. The two #976 tests are adapted** (`TestRestartStopChildren_Nested` case 3 and `TestRestartLimit` "a rethrown failure reaches the enclosing supervisor"): they assert, with two equal `AnyError()` layers, something that the oracle (N1, N7, N6c) shows Pekko does not do; this is not weakening a test. **Conditions:** the scenario with different matchers (N6d, N7d) is kept with its assertions unchanged, the equal-matcher form that does what Pekko does is added (T1a), and the PR explains each assertion change with its oracle scenario.
- **D-3. No flattened form.** Go's `OnFailure` returns a `Behavior`, and ADR 0035 fixes `Supervise(b).OnFailure(match, strategy)`; "flattened equals nested" (AC-R3-1) is recorded in the README and the issue, no API is added.
- **D-4. No explicit `Stop` yet** (#1182): the criteria use `Resume`, `Restart` and the default Stop.
- **D-5. An amendment of ADR 0035 in its own PR**, with the usual pattern (an Amendment section, a "Superseded" mark next to "at start and after every turn" in "Additional guarantees", and a line of the Owner table quoting the accepted text literally); it merges **before** slice 1a.
- **D-6. #1184 to `status:ready`** (done).

## Appetite

Four PRs plus the amendment (a kit package, one code PR, one test PR, one docs PR). The risk is slice 1a: the dedupe must not apply while the start is failing (N8), it must apply at the three sites, and the two adapted tests must keep their assertions.

## Definition of Done

- Every criterion T1a to T8 has its test and T9 its text; each behavior PR names its mutation in a "Mutations" table, reproduced by QA.
- `scripts/each_module.sh go test -count=1 ./...`, `scripts/check_deps.sh` and the complexity lint green, with the commands in `AGENTS.md`.
- Diff coverage ≥ 85% on new/modified lines; per-module coverage ≥ 85%; cyclomatic complexity ≤ 10 in every new/modified function; `typed` still imports only `pekko-actor`.
- No existing test changes except the two of D-2 (their assertions unchanged, only a matcher; each change explained with its oracle scenario in the PR) and the title of `S8` (T8); every Pekko citation is verified by the Dev; the evidence of each PR is a comment bound to its final head SHA with real exit codes (RUNBOOK step 4); a sentence that says something is observed names the log line or the assertion (rule of 2026-11-06).
