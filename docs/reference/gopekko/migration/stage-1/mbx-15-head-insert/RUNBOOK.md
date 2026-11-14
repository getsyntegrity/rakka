# RUNBOOK: Head insert in the mailbox storage (unit mbx-15-head-insert)

Status: **prepared.** Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `develop` `80695c9` or later. Draft PRs, each `Refs #1098`, never `Closes`; no merge without the Owner; #1098 is not closed by this unit.

| Task | Content |
|---|---|
| T1 | this package, approved before any code |
| T2 | Q1 to Q9 and `PushFront` in `pekko-actor/internal/mailbox/` |
| T3 | `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row (unmeasured values stay `unknown`), gate evidence |

## Procedure

1. TDD: write each scenario, watch it fail (wrong implementation or stub), implement the minimum, inside the allowed files. Note the mutation that turns each scenario red.
2. Gates (commands in `AGENTS.md`, "Go"): unit tests of `pekko-actor` with network isolation where the platform allows, `go vet`, `gofmt -l`, `golangci-lint` (gocyclo at most 10), `go mod tidy -diff`, module coverage at least 85%, diff coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`. Run once per final tree.
3. Record each gate with the pushed SHA, command, environment and exit code. Remote CI is not used; attribute no remote result.
4. QA reviews on the final SHA; any later commit invalidates the evidence.

## Stops

A file outside the allowed list must change; an export or a cell change becomes necessary; a scenario fails and is not a Go defect (read Pekko first); the PR projects over 400 authored lines. Report to the TL; never skip, delete or weaken a test.
