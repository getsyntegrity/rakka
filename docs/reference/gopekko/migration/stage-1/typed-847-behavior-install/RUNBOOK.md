# RUNBOOK: install and replace a Typed receive Behavior (unit typed-847-behavior-install)

Status: **prepared.** Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `develop` `108880b`. Delivery is three stacked draft PRs, each `Refs #847`, never `Closes`, no merge without the Owner.

| PR | Tasks | Content |
|---|---|---|
| 1 | T1 | this package and the scope file |
| 2 | T2, T3, T4 | `pekko-actor-typed/typed/` code, tests S1 to S11, example, READMEs; if projected over 400 authored lines, T4 moves to PR 3 |
| 3 | T5 | `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row, gate evidence bound to the final SHA |

## Procedure per PR

1. TDD: write each scenario, watch it fail, implement the minimum, inside the allowed files.
2. Gates (commands in `AGENTS.md`, "Go"): `unshare -rn` unit tests of `pekko-actor-typed`, `go vet`, `gofmt -l`, `golangci-lint` (gocyclo at most 10, rebuilt in `bin/` at `.golangci-version` with go1.27.2), `go mod tidy -diff`, module coverage at least 85%, diff coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `go-sdk-update.sh --check`. Run once per final tree.
3. Publish each gate as a PR comment: one `<details>` block with the pushed SHA, command, environment (OS, `go version`, lint version), exit code and full output. An identical tree, command and environment is cited, not repeated. Remote CI cannot run (GitHub Actions billing); say so and attribute no remote result.
4. QA reviews each PR on its final SHA; every commit after the gates invalidates the evidence.

## Stops

A file outside the allowed list must change; `pekko-actor` needs an export; a scenario fails and is not a Go defect (run it on Pekko first); the line budget would be exceeded; go1.27.2 is unavailable. Report to the TL (session `gopekko-15`); never skip, delete or weaken a test.
