# RUNBOOK: Expose `deadLetters` and carry the sender in `DeadLetter` (unit deadletters-ref-and-sender)

Status: **prepared.** Sources pinned: Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. The scope is `docs/scopes/896-deadletters-ref-and-sender.md`. Delivery is sequential PRs, each `Refs #896`, never `Closes`; no merge without the Owner. Roles: the Dev works in a separate session, QA reviews from a clean read-only context, the TL does not implement.

| PR | Content | Criteria |
|---|---|---|
| 0 | this package | kit run |
| 1a | `DeadLetter.Sender`; `tell` and `stop.go` pass the sender; P1 asserts it, `PORT-GAP(dl-sender-896)` removed | S1, S2 |
| 1b | `System.DeadLetters()`, the sink (`deadletters_ref.go`), substitution in `publish`, internal `TellFrom` path, literal migration (1c if over 400) | S3 to S8 |
| 2 | `pekko-actor/README.md`, `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row | S9 |

Budget: at most 400 authored changed lines per PR; estimates are raw times 1.55 (scope: 1a ~210, 1b ~380). 1b starts from the merged 1a; slice 2 edits the shared `pekko-actor/README.md` and is rebased on the #776 scope's README change if that merges first.

## Procedure per PR

1. TDD inside the allowed files: scenario, watch it fail, minimum code. `deadletter.go` and `ref.go` are edited by 1a, then by 1b.
2. Mutations per RULEBOOK section 5, by hand, listed in the PR "Mutations" table.
3. Gates (`AGENTS.md`), once per final tree:
   - `scripts/each_module.sh go test -count=1 ./...`; `go vet`; `gofmt -l`. Network isolation: `unshare -rn scripts/each_module.sh go test -count=1 ./...` on Linux; on macOS the evidence says "sin aislamiento de red (macOS)" and the Owner runs the container variant (`docker run --rm --network none`).
   - `GOBIN=$PWD/bin go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@$(<.golangci-version)`, then `cd pekko-actor && ../bin/golangci-lint run --config ../.golangci.yml ./...` (gocyclo at most 10)
   - coverage: `export ROOT="$PWD" COV="$(mktemp -d)"`, per-module profiles as in `AGENTS.md`, `scripts/check_coverage.sh "$COV" 85`, then `diff-cover ... --compare-branch=origin/develop --fail-under=85`
   - `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/each_module.sh go build ./...` (typed and persistence still compile), `.github/scripts/go-sdk-update.sh --check`
   - S7: `go test -count=100` on the concurrent test; no `-race`
4. Before opening the PR, re-open every Pekko line range cited in the manifest, RULEBOOK, RUNBOOK and PR body in `upstream/pekko`, and list them in the Evidence comment as `path:lines -> first line of the range`. `grep -rn dl-sender-896` must be empty after 1a.
5. Record each gate in ONE Evidence comment bound to the final pushed SHA (command, environment, real `exit=`, full output in `<details>`). Report real authored lines against the estimate. The Go judge campaign is not run here (scope "Judge"); the PRs say the Go CURRENT is stale.
6. QA reviews on the final SHA, read-only, and reproduces the mutations (unmutated green, mutated red); every later commit invalidates the evidence.

## Manifest check

```
awk -F'\t' 'NR>1 && (NF!=7 || $1=="" || $2=="")' migration/stage-1/deadletters-ref-and-sender/manifest.tsv
```

It must print nothing. Stops: RULEBOOK section 4, reported to the TL.
