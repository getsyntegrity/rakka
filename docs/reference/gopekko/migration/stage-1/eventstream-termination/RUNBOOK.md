# RUNBOOK: Termination of event stream subscribers (unit eventstream-termination)

Status: **prepared.** Sources pinned: Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. The scope is `docs/scopes/1056-eventstream-termination.md`. Delivery is sequential PRs, each `Refs #1056`, never `Closes`; no merge without the Owner.

| PR | Content | Criteria |
|---|---|---|
| 0 | this package | kit run |
| 0b | oracle addendum EV12 on Pekko 1.7.1 (another Dev): harness, trimmed logs, oracle section | T7 |
| 1a | the stream's internal `watcherLink`, the call at `Subscribe`, the refusal paths | T1 to T4, T4b |
| 1b | unwatch hook, concurrency, `Subscribe` on a stopping System (after EV12) | T5 to T7 |
| 2 | README of `pekko-actor`, `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row | T8, T9 |

Budget: at most 400 authored changed lines per PR; estimates are raw times 1.55. Slice 1a projects at ~400: split it before the PR (watcher, T1 and T2 first; T3, T4, T4b second). `eventstream.go` is edited by 1a then 1b, never in parallel.

## Procedure per PR

1. TDD inside the allowed files: scenario, watch it fail, minimum code.
2. Mutations per RULEBOOK section 5, by hand, listed in the PR "Mutations" table.
3. Gates (`AGENTS.md`), once per final tree:
   - `scripts/each_module.sh go test -count=1 ./...`; `go vet`; `gofmt -l`. On macOS there is no `unshare`: the evidence says "sin aislamiento de red (macOS)"; the Owner runs the container variant (`docker run --rm --network none`) at module close.
   - `GOBIN=$PWD/bin go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@$(<.golangci-version)`, then `cd pekko-actor && ../bin/golangci-lint run --config ../.golangci.yml ./...` (gocyclo at most 10)
   - coverage: `export ROOT="$PWD" COV="$(mktemp -d)"`, per-module profiles (`-covermode=atomic`, named by `scripts/check_coverage.sh --profile-name`), `scripts/check_coverage.sh "$COV" 85`; the Cobertura XML per module (`go run github.com/boumenot/gocover-cobertura@v1.5.0 <"$p" >"${p%.out}.xml"`, commands in `AGENTS.md`); then `pipx run --spec diff-cover==11.0.0 diff-cover "$COV"/*.xml --compare-branch=origin/develop --fail-under=85`
   - `scripts/check_deps.sh`, `.github/scripts/check-headers.sh` (baseline on develop: 7 errors, none new), `.github/scripts/go-sdk-update.sh --check`
   - T6: `go test -count=100` on the concurrent test; no `-race`
4. Before opening the PR, re-open every Pekko line range cited in the manifest, RULEBOOK, RUNBOOK and PR body in `upstream/pekko`, and list them in the Evidence comment as `path:lines -> first line of the range`. Never modify refs of the main checkout.
5. Record each gate in ONE Evidence comment bound to the final pushed SHA (command, environment, real `exit=`, full output in `<details>`). Report real authored lines against the estimate.
6. QA reviews on the final SHA, read-only, and reproduces the mutations; every later commit invalidates the evidence.

## Manifest check

```
awk -F'\t' 'NR>1 && (NF!=7 || $1=="" || $2=="")' migration/stage-1/eventstream-termination/manifest.tsv
```

It must print nothing. Stops: RULEBOOK section 4, reported to the TL.
