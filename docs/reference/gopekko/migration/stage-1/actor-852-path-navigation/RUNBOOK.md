# RUNBOOK: ActorPath navigation (unit actor-852-path-navigation)

Status: **prepared.** Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `develop` `7840e8893305b0817b76d7b7f6e7d209afe0b8d7`. Scope: `docs/scopes/852-actor-path-navigation.md` (PR #1284, G2 head `76810743150fea3a18be5e72bc39f2a5e627b5f6`). Delivery is three draft PRs, each `Refs #852`, never `Closes`, no merge without the Owner. The PRs are stacked (K, then C, then R); C and R are not independent of K.

Owner instruction given in the working session on 2026-10-29 (not a GitHub comment; recorded here): the Dev may split the work into PRs so that each stays at or under 400 authored changed lines, keeping criteria and traceability. A split that changes the contract requires a new G2; this split keeps the scope contract unchanged.

| PR | Branch (base) | Tasks | Content |
|---|---|---|---|
| K | `docs/852-kit-package` (`develop`) | T1 | this package: `manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` |
| C | `feat/852-actor-path-navigation` (`docs/852-kit-package`) | T2, T3, T4 | `actor_path.go` (`Parent`, `Descendant`), `actor_path_test.go` (S1 to S11), `errors.go` comment, `pekko-actor/README.md` path paragraph |
| R | `docs/852-execution-record` (`feat/852-actor-path-navigation`) | T5 | `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row, gate evidence bound to the SHA of C after QA |

## Procedure per PR

1. TDD: write each scenario, watch it fail (RED output kept), implement the minimum, inside the allowed files.
2. Gates (commands in `AGENTS.md`, "Go"), for the affected module `pekko-actor`: `cd pekko-actor && go test -count=1 ./...` with network isolation where the platform allows (`unshare -rn` needs Linux user namespaces: not available on macOS, state it and run normally), `gofmt -l pekko-actor`, `golangci-lint` (gocyclo at most 10), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, coverage by the `AGENTS.md` per-module recipe over every workspace module then `scripts/check_coverage.sh "$COV" 85`, diff coverage at least 85% (`diff-cover` against `origin/develop`), `go-sdk-update.sh --check`, and `make ci-fast` (what CI runs per `AGENTS.md`; run locally, never remotely). `golangci-lint` is installed with `GOBIN=$PWD/bin go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@$(<.golangci-version)` and run from `bin/`. `make parity-guard` is not applicable (persistence adapters only). Additions from `docs/quality-gates.md`: `go vet ./...` and `go mod tidy -diff` in `pekko-actor`. Differential check for shared code: `Child` shares `checkPathElement` with `Descendant`; compare `Child` before and after on the same inputs against a copy of the base commit in a `mktemp -d` directory (never committed). Run once per final tree.
3. Record each gate with the pushed SHA, command, environment, exit code and output tail. Remote CI is not run; attribute no remote result.
4. QA reviews each PR on its final SHA; every commit after the gates ran invalidates the evidence.
5. PR size: authored changed lines (additions plus deletions, excluding `go.sum` and generated files) at most 400 per PR, stated in the PR body.

## Stops

A file outside the allowed list must change; `pekko-actor` needs another export; a scenario fails and is not a Go defect (run it on Pekko first, in a disposable copy as in PR #1292 `RUN.md`, never inside `upstream/pekko`); a criterion cannot be asserted without a pending contract decision (report it blocked: criterion 2b is #812's); PR C projects over 400 authored lines; a split would change the contract (needs a new G2). Report to the TL; never skip, delete or weaken a test.
