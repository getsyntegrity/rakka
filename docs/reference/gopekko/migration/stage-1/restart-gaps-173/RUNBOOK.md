# RUNBOOK: unit B + unit S22 + final review + campaign

Status: **PREPARED, NOT APPROVED, NOT EXECUTED.** It authorizes nothing. It replaces the earlier prepared RUNBOOK of this directory (which covered only review and campaign gating) after the Owner decisions of the second brief. Modeled on `../shutdown-u10/RUNBOOK.md` and `../supervision-u8/RUNBOOK.md`. Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, base branch `feat/go-restart-gaps-173` at `fdba453`.

## 1. Package
`RULEBOOK.md`, `RUNBOOK.md`, `BUDGET.md`, `implementer-prompt.md`, `review-prompt.md`, `manifest.tsv`, `PACKAGE-HASHES.tsv` (sha256 of every file above plus `S22-GO-PROPOSAL.md`). The Owner approves the package by those hashes. Not prepared, and not reused without the Owner saying so: the launcher `run-agent.sh` and the settings templates of earlier units. Without a launcher the edit surfaces and byte-identical files are enforced by the coordinator, who compares the diff and the hashes after each run and rejects the run on any difference.

## 2. Order of work (one writer at a time)
1. **Baseline** (coordinator): clean tree; record `git rev-parse HEAD HEAD^{tree}`; sha256 of the RULEBOOK section 4 files; submodule pins; `python3 -I scripts/check-judge-matrix.py` exit and lines. Each command a separate call.
2. **Unit B** (implementer 1, `implementer-prompt.md` unit B). Coordinator then reruns the verification itself once and checks the diff touches only `judge/mutate_go.py` and `judge/selftest_mutate_go.py`. Commit: `test(judge): ...`. If B3 fires, stop and report to the Owner.
3. **Unit S22** (implementer 2, unit S22), only after step 2 is accepted. Coordinator verifies, then commits by work unit (production event with its test; harness and scope; mutation; docs).
4. **Coordinator gates on the final candidate:** gofmt, vet, `go mod tidy -diff`, pinned golangci-lint 2.14.0 and gocyclo (isolated install in the job directory), coverage at least 85 % per module, `check_deps.sh`, `check-headers.sh`, `check-adr-signatures.sh`, `go test -count=1 ./...`, `go test -race -count=1 ./...`, the three judge selftests, `check-judge-matrix.py`.
5. **Candidate frozen.** From here any change to `pekko-actor/` or `judge/` restarts steps 4-6.
6. **Full Go campaign** (coordinator, background, workdir and report outside the repository, `--runs 30`, tree clean). Target **18 of 18** designed + benign survived. A report containing any INVALID is not valid: it is kept, not published, not combined; investigate before any repeat (no repetition looking for a clean run).
7. **Review of the final candidate** (after step 6 on the frozen candidate, once), per `BUDGET.md` gate. If native: `gentle-ai review status ... --base-ref origin/develop --committed-only` with the untracked declaration below, then the START consent envelope is presented to the Owner verbatim and answered by the Owner only. If the opus reviewer: `review-prompt.md`, one launch, `review.lock`.
8. **Corrections** of confirmed BLOCKER/DEFECT findings: at most one round, test first; if the candidate changes, steps 4-6 are repeated (the budget gate decides whether that is affordable; if not, the Owner is asked).
9. **Publication package** (prepared, not executed): the valid report copied to `judge/go-mutation-report.tsv` on a clean tree, `python3 -I scripts/check-judge-matrix.py`, and the exact tag command `git tag evidence/go-campaign-<full sha>` with the SHA, tree, platform and toolchain recorded. Pushing the tag needs the Owner's authorization.

## 3. Untracked file declaration (preflight)
`odd/tasks/restart-gaps-173.md` is **not part of the candidate**. In the native preflight it is declared explicitly as excluded with `--untracked-scope=exclude --expected-untracked-inventory=<the sha256 the preflight returns for the exact tree>` (current value `sha256:aca07feedf451c4f972511a0930ca7d13027f03f0de04c89b9fd8e7554f69ee6`; it must be re-read at the time, never reused from this file). The file stays on disk and is not deleted, not forced into a commit. Everything a reviewer needs is in the versioned documents of this directory.

## 4. Limits
Budget, roles, gates and stop criteria: `BUDGET.md`. No merge, no tag push, no closing of #173 or #461, no change of submodules, PR #480 stays draft.

## 5. Execution record
Every command with its observed result, including failed, skipped and pending checks; the incremental token accounting against `BUDGET.md`; the RG1-RG3 deviations stay as recorded: the approval of this package does not turn the earlier work into a kit execution.
