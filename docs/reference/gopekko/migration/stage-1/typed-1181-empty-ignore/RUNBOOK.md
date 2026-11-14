# RUNBOOK: Empty and Ignore (unit typed-1181-empty-ignore)

Status: **executed in PR #1264, integrated into `develop` as `595595cfa87ed6d9ffcd143043707f54f6437763`** (final candidate `32e7870`); the unit was accepted as an exception to scope/G2 (RULEBOOK). Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base of that PR (history): `develop` `bc727c7`. PRs for this unit are `Refs #1181`, never `Closes`, and are merged only by the Owner's decision.

## Procedure

1. TDD: write each scenario, watch it fail, implement the minimum.
2. Gates (commands in `AGENTS.md`, "Go"): unit tests of `pekko-actor-typed`, `go vet`, `gofmt -l`, `golangci-lint` (gocyclo at most 10), `go mod tidy -diff`, unit tests with network isolation (container variant where `unshare` is missing), module coverage at least 85%, diff coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `.github/scripts/go-sdk-update.sh --check`. Run once per final tree; a docs-only commit inherits the rows of the last code tree, with the equivalence shown. A failed gate is recorded as failed with the exception the Owner must decide, never as green.
3. Record each gate with the pushed SHA and result. Remote CI is not attributed.
4. QA reviews on the final SHA; every commit after the gates ran invalidates the evidence.

## Stops

A file outside the writable list must change; `pekko-actor` needs an export; a scenario fails and is not a Go defect (run it on Pekko first). Never skip, delete or weaken a test.
