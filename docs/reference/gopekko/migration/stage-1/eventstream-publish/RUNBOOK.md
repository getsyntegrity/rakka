# RUNBOOK: Publish events through the subscribed actors' mailboxes (unit eventstream-publish)

Status: **prepared.** Sources pinned: Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. The scope is `docs/scopes/870-eventstream-publish.md`. Delivery is sequential PRs, each `Refs #870`, never `Closes`; no merge without the Owner.

| PR | Content | Criteria |
|---|---|---|
| 0 | this package | kit run |
| 0b | oracle addendum EV11 on Pekko 1.7.1: harness, trimmed logs, oracle section | P9 |
| 1a | `Publish`, `ErrNilEvent`, the cache and its update in `Subscribe` | P1 to P4 |
| 1b | delivery tests: slow subscriber, full or stopped subscriber, concurrency | P5 to P7 |
| 2 | `DeadLetter` events on the stream, cycle rule, `Deprecated:` paragraph | P8 to P10 |
| 3 | README of `pekko-actor`, `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row | P11 |

Budget: at most 400 authored changed lines per PR; estimates are raw times 1.55. A slice that projects over 400 is split before the PR is opened. Slice 2 starts only after EV11 (0b) is observed.

## Procedure per PR

1. TDD inside the allowed files: scenario, watch it fail, minimum code. `eventstream.go` is edited by 1a only, `deadletter.go` by 2 only.
2. Mutations per RULEBOOK section 5, by hand, listed in the PR "Mutations" table.
3. Gates (`AGENTS.md`), once per final tree:
   - `scripts/each_module.sh go test -count=1 ./...`; `go vet`; `gofmt -l`. On macOS there is no `unshare`: the evidence says "sin aislamiento de red (macOS)"; the Owner runs the container variant (`docker run --rm --network none`) at module close.
   - `GOBIN=$PWD/bin go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@$(<.golangci-version)`, then `cd pekko-actor && ../bin/golangci-lint run --config ../.golangci.yml ./...` (gocyclo at most 10)
   - coverage: per-module profiles, then `scripts/check_coverage.sh "$COV" 85`; `diff-cover --compare-branch=origin/develop --fail-under=85`
   - `scripts/check_deps.sh`, `.github/scripts/check-headers.sh` (baseline on develop: 7 errors, none new), `.github/scripts/go-sdk-update.sh --check`
   - P7: `go test -count=100` on the concurrent test; no `-race`
4. Before opening the PR, re-open every Pekko line range cited in the manifest, RULEBOOK, RUNBOOK and PR body in `upstream/pekko`, and list them in the Evidence comment as `path:lines -> first line of the range`.
5. Record each gate in ONE Evidence comment bound to the final pushed SHA (command, environment, real `exit=`, full output in `<details>`). Report real authored lines against the estimate.
6. QA reviews on the final SHA, read-only, and reproduces the mutations; every later commit invalidates the evidence.

## Manifest check

```
awk -F'\t' 'NR>1 && (NF!=7 || $1=="" || $2=="")' migration/stage-1/eventstream-publish/manifest.tsv
```

It must print nothing. Stops: RULEBOOK section 4, reported to the TL.
