# RUNBOOK: Classified event subscriptions on the EventStream (unit eventstream-registration)

Status: **prepared.** Sources pinned: Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. The scope is `docs/scopes/836-eventstream-registration.md`. Delivery is sequential PRs, each `Refs #836`, never `Closes`; no merge without the Owner.

| PR | Content | Criteria |
|---|---|---|
| 0 | this package | kit run |
| 0b | oracle addendum EV4f on Pekko 1.7.1: harness, trimmed logs, oracle section | X7b |
| 1a | `Index[K, V]`: `AddKey`, `AddValue`, `FindValues`, `Subclassification[K]` | X1, X2, X4a, X12 |
| 1b | removal by key and by value (same files as 1a, after it) | X3, X4b |
| 2 | relation on `reflect.Type` (`channel.go`) | X5 |
| 3a | `ErrNilSubscriber`, `ErrChannelNotAccepted`, `Subscribe` checks, `eventStream` shell, one field in `system.go` (after the ADR 0038 amendment of D-2) | X8a, X9, X10 |
| 3b | index wiring under the lock, `deliver` closure, concurrency | X6, X7a, X7b, X8b, X13 |
| 4 | README of `pekko-actor`, `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row | X11 |

Budget: at most 400 authored changed lines per PR; scope estimates are raw, re-estimate at 1.55 (the deviation measured in #976). A slice that projects over 400 is split before the PR is opened.

## Procedure per PR

1. TDD inside the allowed files: scenario, watch it fail, minimum code. Slices 1a/1b and 3a/3b touch the same files: never in parallel.
2. Mutations per RULEBOOK section 5, listed in the PR "Mutations" table.
3. Gates (`AGENTS.md`), once per final tree:
   - `unshare -rn scripts/each_module.sh go test -count=1 ./...` (the container variant `docker run --rm --network none` where namespaces are restricted); `go vet`; `gofmt -l`
   - `GOBIN=$PWD/bin go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@$(<.golangci-version)`, then `cd pekko-actor && ../bin/golangci-lint run --config ../.golangci.yml ./...` (gocyclo at most 10)
   - coverage: per-module profiles, then `scripts/check_coverage.sh "$COV" 85`; diff coverage `diff-cover --compare-branch=origin/develop --fail-under=85`
   - `scripts/check_deps.sh`, `.github/scripts/check-headers.sh` (baseline on develop: 7 errors in `pekko-actor/names.go` and persistence snapshot files), `.github/scripts/go-sdk-update.sh --check`
   - X13: `go test -count=100` on the concurrent test; no `-race`
4. Record each gate in a PR comment bound to the pushed SHA (command, environment, exit code, full output in `<details>`). Report real authored lines against the estimate.
5. QA reviews on the final SHA, read-only, and reproduces the mutations; every later commit invalidates the evidence.

## Manifest check

```
awk -F'\t' 'NR>1 && (NF!=7 || $1=="" || $2=="")' migration/stage-1/eventstream-registration/manifest.tsv
```

It must print nothing. Stops: RULEBOOK section 4, reported to the TL.
