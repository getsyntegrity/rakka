# RUNBOOK: U4 follow-up, #459 and #460

Status: **PREPARED, NOT APPROVED, NOT EXECUTED.** It authorizes nothing: no agent, no budget, no seam, no campaign, no merge. Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1). Base: `develop` at `a5782c9651bc382020099dfda47b7fdc99dc971b`; branch `feat/go-u4-459-460`.

## 1. Scope
Test-only completion of two informative gaps of U4 (#444, PR #457), recorded in #461: **#459** the `stopping` branch of `Spawn` end to end; **#460** a failed factory (nil, panic) and a `runtime.Goexit` of the factory after an accepted stop. Directed mutants for each. No production change except, if the Owner approves D1, one nil-by-default test seam in its own commit. Not in scope: restart, supervision semantics, the judge scenarios, the other items of #461. Any real defect found is reported with evidence and its correction is a separate decision.

## 2. What already covers the area (checked, to avoid duplicates)
`TestACommitAfterTheGateClosedReportsStopping` (`hierarchy_internal_test.go:559`) checks only that `System.commit` returns `stopping=true`; T4 (`TestSup4T4StopAcceptedBeforeTheFirstActivation`, `spawn_supervised_goside_internal_test.go:279`) covers a top-level nil factory only; `TestSup4T7TopLevelFactoryFailure` (`:392`) and `TestACommittedChildHoldsTheParentOpenWhileItsFactoryHasNotRun` use a parent without a strategy and count no decider calls or events. Nothing covers T459, T460a under a declared strategy, or T460b. The implementer reads these and reuses their helpers.

## 3. Roles, budget and limits (PROPOSED; nothing is approved)
| Role | Model | Count | Mechanical limits (enforced by the launcher) | Advisory tokens (NOT mechanical) |
|---|---|---|---|---|
| Implementer, tests and mutants, edits only the four surfaces | `sonnet` | 1 | USD 6 (`--max-budget-usd`), wall 45 min (`AGENT_WALL_MIN`) | 200,000 |
| One independent review of the whole diff, read-only, `review.lock` | `opus` | 1 | USD 3, wall 12 min | 120,000 |
| Gates, race, mutant re-run, campaign renewal, records, PR | the coordinator | 0 agents | machine time only | 60,000 estimated, not measured |
| One correction round of confirmed findings | the coordinator | 0 agents | none | 30,000 reserve |
**Proposed incremental ceiling: 410,000 tokens** (200,000 + 120,000 + 60,000 + 30,000), **evaluation stop at 330,000**. This is a new ceiling; no earlier ceiling or exception applies to this unit. Earlier consumption of other units is not carried here and is not reset in the records.

**What is and is not mechanical.** `claude -p` accepts a USD cap and a wall bound; the launcher passes both. It has **no token cap**: the token figures above are advisory and are read from the JSON after each run. A run killed by its wall bound usually leaves no output, and the coordinator then counts the whole forecast of that agent as spent. In the previous unit three agents exceeded their token caps (by 11,865, 48,725 and 105,968 tokens) because no mechanical token limit existed; this launcher does not remove that risk, only bounds USD and time. The launcher was copied from `supervision-u8/run-agent.sh` (change: the temporary file name) and **has not been probed in this unit**; the first run is its probe. If `claude -p` cannot run here, the fallback is the in-session agent tool, which has **no** USD, time or token cap, and the Owner is told before using it.

Gates: the review is launched only if `observed + remaining reserve + 120,000 <= 410,000` is confirmed with the numbers written down; a correction round only if `observed + remaining reserve + 30,000 <= 410,000`; at 330,000 the coordinator stops and asks. A measured consumption higher than a forecast is recorded as measured; a cap is never written in place of a measurement.

## 4. Order of work (one writer at a time)
1. **Baseline** (coordinator): clean tree at `a5782c9`; record `git rev-parse HEAD HEAD^{tree}`; sha256 of the protected files (RULEBOOK section 5); submodule pins; `scripts/check-judge-matrix.py` result.
2. **D1 first.** If approved, the coordinator makes the seam commit (`commitPoint`, nil in production), verifies it changes no behavior (`go test -race`, all gates) and updates the baseline; if rejected, T459 is dropped from the run.
3. **Implementer** (`run-agent.sh implementer`): T460a, T460b, T459 if D1, the mutant cells and selftests. The coordinator then re-runs the verification once and checks the diff touches only the allowed surfaces and the protected files are byte-identical. Commit by work unit.
4. **Coordinator gates:** gofmt, vet, `go mod tidy -diff`, pinned `golangci-lint` 2.14.0 with gocyclo (isolated install in the job directory), per-module coverage at least 85 %, `check_deps.sh`, `check-headers.sh`, `check-adr-signatures.sh`, `go test -race -count=1 ./...`, the targeted race (`-count=20`), the three judge selftests, the new mutants on copies outside the repository.
5. **Candidate frozen.** `judge/mutate_go.py` and `pekko-actor/` (if the seam was approved) are campaign inputs: the committed Go campaign report stops being CURRENT. Record that, then run **one** full campaign (`python3 -I judge/mutate_go.py <workdir outside> --runs 30 --report <outside>/go-mutation-report.tsv`) on the frozen candidate. Expected: 18 of 18 designed mutations detected, the benign one surviving, the two approved exclusions declared, no other INVALID. A failure is investigated before any repeat; no repetition looking for luck; no mixing of campaigns.
6. **Review** (`run-agent.sh review`, `review-prompt.md`), once, over the whole diff, after step 5. Corrections of confirmed findings: at most one round; a correction that changes campaign inputs makes the evidence stop being CURRENT until validated again.
7. **Delivery:** draft PR against `develop` (traceability Pekko to Go, real results, review scope, pending), update #461 to the integrated state, prepare the evidence tag with the exact campaign SHA (not pushed). No merge, no automatic closing of issues.

## 5. Allowed edit surfaces (implementer)
`pekko-actor/actor/spawn_stopping_internal_test.go` (only if D1), `pekko-actor/actor/spawn_factory_stop_internal_test.go`, `judge/mutate_go.py`, `judge/selftest_mutate_go.py`. Everything else is denied by the rendered permissions (`settings.implementer.template.json`) and compared by hash afterwards. This is a post-hoc scope control as well as a permission control; the record says which of the two caught what.

## 6. Exact commands (after the Owner approves)
Run from the worktree root. `D` is `/Users/pablogore/.claude/jobs/b989ecff/tmp` (outside the repository). The two hash files are written from the **approved** hashes of `PACKAGE-HASHES.tsv`, never from the files on disk:
```
printf '%s\n' <approved RULEBOOK sha256> > D/rb.sha
printf '%s\n' <approved implementer template sha256> > D/st-impl.sha
AGENT_BUDGET_USD=6 AGENT_WALL_MIN=45 bash migration/stage-1/supervision-u4-followup/run-agent.sh implementer sonnet migration/stage-1/supervision-u4-followup/implementer-prompt.md D/u4f-impl.json D/rb.sha D/st-impl.sha
```
and, for the review, after step 5, the same with `review opus review-prompt.md D/u4f-review.json` and the review template hash.

## 7. Decisions requested of the Owner
1. **Package and budget:** approve, change or reject the roles and models of section 3, the ceiling of 410,000 with the stop at 330,000, the USD and time caps, and the hashes of `PACKAGE-HASHES.tsv`.
2. **D1:** approve or reject the `commitPoint` test seam (RULEBOOK section 6). Without it #459 cannot be proved end to end.
3. **Launcher:** accept the copied launcher with its first run as its probe, or ask for a separate probe first.

## 8. What this file authorizes
Nothing. Approval of the package does not authorize a merge, a second review, a tag push, the closing of #459, #460 or #461, or any production change other than D1 as written.
