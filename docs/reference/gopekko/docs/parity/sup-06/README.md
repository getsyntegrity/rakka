# SUP-06 (#825): traceability of the restart-budget boundary matrix

Story: [#825](https://github.com/getsyntegrity/gopekko/issues/825) (`SUP-06`, inventory row `ACTINV-038`). Depends on [#779](https://github.com/getsyntegrity/gopekko/issues/779) (`SUP-05`). Baseline: goPekko `develop` `108880b0100c11ae838a772c23af952e5a54b827`. Pekko pin: v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`upstream/pekko`, read-only). Contract: [ADR 0017](../../adr/0017-supervision.md) (accepted 2026-10-14), section S4 and decisions A4, A5 and A6.

**This story verifies an implementation that already exists.** The restart budget (`Budget{MaxRetries, Within}`), its window arithmetic and the per-child statistics under `AllForOne` have been in `develop` since the supervision units U2 and U6 (PR #470 and earlier). The ACT2 reconciliation of 2026-10-24 classified the story `CLOSE_CANDIDATE` and left two criteria without a record: the tested candidate SHA with the local gates (AC4) and the traceability update (AC5). This change adds only this document, one pointer in `pekko-actor/README.md` and one row in `migration/cost-log.tsv`. It changes no production code, no test, no contract and no ADR, and it declares no parity beyond what the "Evidence classes" section states.

## What the code does, in one paragraph

Every child has its own `restartStats` (a restart counter and the start of the current window). When a `Restart` is decided, `requestPermission` in `pekko-actor/actor/supervision_budget.go` answers allow or deny exactly as Pekko's `ChildRestartStats.requestRestartPermission` does: a negative `MaxRetries` is "no count", zero always denies, a count without a window is a plain limit, and a window turns the count into "at most N restarts per window", where "inside the window" means `now - windowStart <= window`. The clock is read once per request, and only when there is a window. Under `AllForOne`, `requestRound` in `supervision_restart.go` asks every child of the round in ascending name order with that child's own statistics and stops at the first denial, as Pekko's `children.forall(_.requestRestartPermission(...))` does.

## Acceptance criteria

| AC | Criterion (issue text, abridged) | Class | Evidence |
|---|---|---|---|
| AC1 | Exact-window boundaries with an injected monotonic clock | **PROVEN** | `TestSup2ZeroReading` rows E4 (`supervision_budget_internal_test.go:309`, a reading exactly at `windowStart + window` is inside, so the second request with `MaxRetries` 1 is denied: `AD`) and E5 (`:310`, one nanosecond later is outside, so it is allowed: `AA`). The clock is a function passed to `requestPermission`, so no test sleeps. The same seam is wired through the runtime as `options.nanotime` (`system.go:46-48`, default at `:165-167`) and used by `TestSup2BudgetDirective` (`:267`, cases at `:220-239`, including a crash after the window restarting again, `:231`). |
| AC2 | Finite, zero and unlimited cases match SUP-05, no wall-clock sleeps | **PROVEN** | `TestSup2BudgetMatrix` (`:195`): the 24 cells of ADR 0017 A4 (four `MaxRetries` values by six windows, five requests each; table at `:185-193`). Unlimited over time: `TestSup2BudgetUnlimited` (`:276`, 100 crashes, 100 restarts). Zero budget at runtime: `TestAllForOneBudgetZeroStopsTheChild` (`supervision_allforone_internal_test.go:336`) and the `(0, Inf)` case at `supervision_budget_internal_test.go:221`. Overflow and conversions: `TestSup2Conversions` (`:325`), `TestSup2Int32Wrap` (`:361`). None of these files calls `time.Sleep`; the only `time.After` and `time.Now` in the AllForOne file (`:67`, `:307-309`) are 30-second failure bounds, not part of the window clock. |
| AC3 | `AllForOne` consumption follows the per-child behavior of the source | **PROVEN** (counters). See observation G3 for the window anchor | `TestAllForOneCountersPerChild` (`:444`: counters diverge, the third round is denied by `b` and `c` is not reached), `TestAllForOneDenierFirstAndLast` (`:469`: the denier first, the denier last, children before it are charged and the ones after are not), `TestAllForOneSiblingAtItsLimitDenies` (`:499`: a fresh failing child is denied by a sibling that is at its limit). Statistics that survive restarts and start fresh for a new child of the same name: `TestSup2StatsPerChild` (`supervision_budget_internal_test.go:472`). Only `Restart` consumes: `TestSup2BudgetOnlyRestart` (`:416`), `TestSup2DecisionLeavesCounter` (`:536`). |
| AC4 | Focused positive and negative cases, the named ordering or race boundary, tested candidate SHA and local gates | **PROVEN by this change** | Positive and negative cases: the tests above (allow and deny in every row). The issue names no ordering or race boundary; the reading used here is the order of an `AllForOne` round (ascending name as observed in Pekko, first denial ends it, `TestAllForOneDenierFirstAndLast`) and the single clock reading per request (`reads` column of `sup2Cases`, `:217`). See decision O-2. Candidate SHA and gates: section "Gates" below. |
| AC5 | Public contract and traceability, no claim about sibling stories or full parity | **PROVEN by this change** | Contract: unchanged, ADR 0017 S4/A4/A5/A6 and the `Budget` text of `pekko-actor/README.md`. Traceability: this document, linked from `pekko-actor/README.md`. The row of `SupervisorStrategy` in `INV-A-actor.md` (`ACTINV-038`, line 66, which lists #779 and #825) is a dated snapshot of `develop` `9abfb40` and is left unchanged, as for `ACTINV-056` in `docs/parity/mbx-13/README.md`. Correction 2026-10-28: this document first cited the row as `ACTINV-041`, the identifier written in the bodies of #779 and #825; that identifier does not appear in `INV-A-actor.md`. |

## Criterion to code to test to Pekko source

| Behavior | Go code | Go test | Pekko source (v1.7.1) |
|---|---|---|---|
| `MaxRetries < 1` denies; no window counts plainly; a window with no count allows one per window; no count and no window is unlimited | `supervision_budget.go:42-59` | `TestSup2BudgetMatrix` | `FaultHandling.scala:58-64` |
| Window start stored by the first request; inside is `now - start <= window`; outside resets the count to 1 and the start to `now`, and allows | `supervision_budget.go:63-78` (comparison at `:71`) | `TestSup2ZeroReading` E4, E5; `TestSup2BudgetDirective` | `FaultHandling.scala:66-88` (`now` at `:73`, comparison at `:79`) |
| Start value `0` means "not started", no flag | `supervision_budget.go:33-38`, `:67-70` | `TestSup2ZeroReading` E1, E3 | `FaultHandling.scala:75` |
| Window conversion `toMillis.toInt`, negative or infinite is no window; `Int` wrap | `supervision_budget.go:19-30` | `TestSup2Conversions`, `TestSup2Int32Wrap` | `FaultHandling.scala:303-308`, `:550` |
| One child, one decision | `supervision_restart.go:180-189` | `TestSup2BudgetDirective`, `TestSup2StatsPerChild` | `FaultHandling.scala:673-676` |
| `AllForOne` asks every child, stops at the first denial, then stops all | `supervision_restart.go:232-265` | `TestAllForOneCountersPerChild`, `TestAllForOneDenierFirstAndLast`, `TestAllForOneSiblingAtItsLimitDenies` | `FaultHandling.scala:554-567` (`forall` at `:562`) |

The ascending-name order of the round is the behavior observed in Pekko v1.7.1 (a `TreeMap` by name), not a documented guarantee of Pekko or of this module (`pekko-actor/README.md:379` and `:395`). It is recorded there; this change does not alter it.

## Evidence classes

- **Historical Pekko run, not re-run here.** The 24 cells were produced by calling the real `ChildRestartStats.requestRestartPermission` on 2026-10-14: `migration/stage-1/supervision/h4-repro/a2a4-sbt.out:442-465`. The Go table `sup2Matrix` repeats those verdicts one for one.
- **Source reading and transcription, not a Pekko run.** The exact edge (E4, E5) and the zero reading (E1 to E3) come from `FaultHandling.scala:66-88` and from the line-by-line transcription `migration/stage-1/supervision/h4-repro/a5-edge/oracle.out:1-21`. The real class reads `System.nanoTime` and cannot be driven to a chosen reading, as ADR 0017 states.
- **Current, executed by us on the baseline.** The Go tests, see "Gates".
- No Pekko scenario was executed for this change, so no parity beyond the matrix and the transcription above is claimed.

## Gates

Candidate: `108880b0100c11ae838a772c23af952e5a54b827` (`develop`). This change touches no `.go` file, so the production code and the tests are byte-identical to it; the head of the pull request that carries this document is recorded in the pull request, because a file cannot contain its own commit SHA. Run on 2026-10-27 in the isolated worktree, without `-race` and without a workbench:

| Gate | Command (from the repository root unless noted) | Result | Kind |
|---|---|---|---|
| Focused tests, 13 test functions | `cd pekko-actor && unshare -rn go test -count=1 -v -run 'TestSup2Budget\|TestSup2ZeroReading\|TestSup2Conversions\|TestSup2Int32Wrap\|TestSup2StatsPerChild\|TestSup2DecisionLeavesCounter\|TestAllForOne(CountersPerChild\|DenierFirstAndLast\|SiblingAtItsLimitDenies)\|TestAllForOneBudgetZeroStopsTheChild' ./actor` | `ok`, all PASS | new |
| Unit tests of the module, no network | `cd pekko-actor && unshare -rn go test -count=1 ./...` | `ok` in `actor`, `internal/deadletter`, `internal/mailbox` | new |
| Unit tests of every module, no network | `unshare -rn scripts/each_module.sh go test -count=1 ./...` | every module `ok`, exit 0 (38 packages with tests) | new |
| `gofmt -l pekko-actor`, `go vet ./...` (in `pekko-actor`) | | no output, exit 0 | new |
| Architecture lint | `scripts/check_deps.sh` | exit 0 | new |
| Coverage gate, 85% per module | per-module profile as in `AGENTS.md`, then `scripts/check_coverage.sh <dir> 85` | `pekko-actor` 98.58% `ok`; `supervision_budget.go` and the three restart functions 100% | new |
| Diff coverage and complexity on new lines | | not applicable: no Go line is added or changed | n/a |
| `golangci-lint` v2.14.0 and `gocyclo` | `bin/golangci-lint run --config ../.golangci.yml ./...` | **not run**: the pinned binary is not on this machine, the only copy (`~/go/bin`, v2.13.1, built with Go 1.26) refuses Go 1.27.2, and installing the pinned one downloads from the network, which this execution was not authorized to do. No Go line changed, so the result for these files cannot differ from the one they had when they merged; that is an inference, not a run | not run |
| Local equivalent of the CI job `go` | `unshare -rn make ci-fast` | exit 0 (gofmt, build and the selected fast unit tests of every module) | new |
| Remote CI of the pull request (`go`, `label`) | GitHub Actions | **no verdict**: both jobs were refused in about 1 second with no runner and no steps; the check annotation says "The job was not started because recent account payments have failed or your spending limit needs to be increased". It is a billing condition of the account, not a result about this change. The Owner accepted the local `make ci-fast` run in its place (see "Decisions for the Owner") | not run |

## Observations that are not defects

- **G3. The window anchor under `AllForOne` is not asserted on its own.** The `AllForOne` tests use one fixed clock (1000 ms, window one hour, `supervision_allforone_internal_test.go:73`, `:92-93`), so no test makes one child's window expire while a sibling's is still open. By construction each child has its own `restartStats`, so Go behaves as Pekko does; but a test that advances the clock between two rounds would pin it. It is a possible single test, not a missing capability, and it is not added because the criterion is about counter consumption and the user brief says not to create duplicate tests. See decision O-3.
- **The clock seam is internal.** `options.nanotime` is unexported (`system.go:46-48`), so the injected clock exists for tests only; the public contract has no clock option and this story does not add one (ADR 0017 A5 fixes only the clock source).

## State of #779 (SUP-05), the dependency

Read from GitHub on 2026-10-27: **open**, labelled `status:blocked` since 2026-10-24. Its reconciliation also says `CLOSE_CANDIDATE`; its AC1, AC2 and AC3 are evidenced by the same ADR 0017 A4 matrix and the same tests, and its AC4 and AC5 are not: the Owner has not stated that ADR 0017 A4 is the reviewable ADR the story asks for, and there is no candidate SHA, gate record or traceability for it. The "accepted matrix" that #825 depends on is ADR 0017 S4/A4, accepted on 2026-10-14; that acceptance does not depend on the state of the #779 issue. Nothing in this change closes or modifies #779; it keeps its own Owner decision and its own evidence reconciliation.

## Independent review

AO native review (an isolated reviewer agent, run `c32b3ce5-d3fd-4cec-8323-321ed276bbea`; internal, so an AO approval is not a GitHub approval and does not authorize a merge): **approved, no changes required**, on the pull request head `fe23de26e900976243e6cff34300aa2bb5973a49`, with no inline comment and no open thread. GitHub shows one review of the same commit, in state `COMMENTED`. It checked the Pekko line citations, the Go file and line citations, the 13-test count, the 24 cells against `a2a4-sbt.out:442-465` and E1 to E5 against `oracle.out`.

Its two optional remarks: (1) `oracle.out` has 21 lines and the document cited 1-20. Checked: lines 1-20 hold E1 to E5 and line 21 is `self-check: ok`, so the old range was not wrong for E1 to E5; the citation is now `1-21` (the whole file, including the self-check). (2) Record decision O-4 before closing #825: done below. The commits after `fe23de2` change only this document, so that review stays the independent review of the evidence; they were not reviewed again.

## Remote CI and merge requirements

The remote jobs `go` and `label` of this pull request were refused by GitHub for an account billing condition and give **no verdict**. The local `make ci-fast` run (exit 0, see "Gates") is the accepted evidence in their place. The lint waiver of O-4 does not waive any other merge requirement. If the Owner authorizes the integration, it would happen without a remote CI result.

## Decisions for the Owner

The Owner's decisions on SUP-06 (#825) reached this session as a message of the Owner in the working session, relayed by the orchestrator session `gopekko-1` on 2026-10-28; they are **not** a GitHub comment. The words, as received (English):

> The Owner approves decisions O-1 through O-4 and conditionally approves O-5 for PR #1226.
>
> - O-1 APPROVED: #825 may close independently of #779 because the required ADR 0017 A4 matrix is already accepted. Do not close or modify #779.
> - O-2 APPROVED: AC4's ordering boundary is the AllForOne round order and the single clock reading per request.
> - O-3 APPROVED: no additional window-anchor test is required for the bounded scope of #825.
> - O-4 APPROVED: waive the missing golangci-lint execution exclusively for this documentation-only PR. Preserve the explicit "not run" record.
> - O-5 APPROVED CONDITIONALLY: close #825 as completed only after PR #1226 has been integrated and its final tree and acceptance evidence reconciled.

Earlier in the same session the Owner had also written (Spanish): "Aceptamos la evidencia local de ci-fast por el bloqueo de facturación. Recomendaciones O-1 a O-4 aprobadas; no hagas merge ni cierres #825 todavía."

| ID | Decision, as bounded by the Owner | What it means in this change |
|---|---|---|
| O-1 | #825 may close independently of #779, because the ADR 0017 A4 matrix it requires is already accepted. | #779 is **not closed or modified by this change** and keeps its own Owner decision. Nothing here suggests closing it afterwards or in the same act. |
| O-2 | The ordering boundary of AC4 is the `AllForOne` round order plus the single clock reading per request. | AC4 above uses exactly this reading. |
| O-3 | No additional window-anchor test is required for the bounded scope of #825. | Observation G3 stays an observation; no test is added. |
| O-4 | The missing `golangci-lint` execution is waived exclusively for this documentation-only pull request. | The "not run" record of `golangci-lint` v2.14.0 and `gocyclo` in "Gates" stays. The waiver does not extend to any other change and waives no other merge requirement. |
| O-5 | Conditional: close #825 as completed only after this pull request has been integrated and its final tree and acceptance evidence reconciled. | #825 is **not closed by this change**. It closes only after this pull request is integrated under a separate, explicit merge authorization and the final tree and the evidence are reconciled; only obsolete status labels are removed then. No merge is authorized by this document. |
