# RUNBOOK: Typed default Stop (unit typed-956-default-stop)

Status: **prepared.** Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `develop` `8ebd25a`. Delivery is three draft PRs, each `Refs #956`, never `Closes`, no merge without the Owner. The scope file stays in PR #1233 (G2 SHA `9c716c3`).

| PR | Tasks | Content |
|---|---|---|
| 1 | T1 | this package |
| 2 | T2, T3, T4, T5 | `pekko-actor-typed/typed/` code and tests S1 to S8, S9/S10 of #847 rewritten, README |
| 3 | T6 | `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row, gate evidence bound to the final SHA |

## Procedure per PR

1. TDD: write each scenario, watch it fail, implement the minimum, inside the allowed files.
2. Gates (commands in `AGENTS.md`, "Go"): unit tests of `pekko-actor-typed` with network isolation where the platform allows, `go vet`, `gofmt -l`, `golangci-lint` (gocyclo at most 10), `go mod tidy -diff`, module coverage at least 85%, diff coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `go-sdk-update.sh --check`. Run once per final tree.
3. Record each gate with the pushed SHA, command, environment, exit code and output. Remote CI cannot run (GitHub Actions billing); attribute no remote result.
4. QA reviews each PR on its final SHA; every commit after the gates ran invalidates the evidence.

## Stops

A file outside the allowed list must change; `pekko-actor` needs an export; a scenario fails and is not a Go defect (run it on Pekko first); PR 2 projects over 400 authored lines. Report to the TL; never skip, delete or weaken a test.
