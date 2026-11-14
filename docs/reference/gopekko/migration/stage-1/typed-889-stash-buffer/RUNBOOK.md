# RUNBOOK: Typed StashBuffer and `WithStash` (unit typed-889-stash-buffer)

Status: **prepared.** Sources pinned: Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. The scope is `docs/scopes/889-typed-stash-buffer.md` (PR #1505). Delivery is sequential PRs, each `Refs #889`, never `Closes`; no merge without the Owner. Roles: Dev in a separate session, QA from a clean read-only context, the TL does not implement.

| PR | Content | Criteria |
|---|---|---|
| 0 | this package | kit run |
| A | `typed/stash.go` (buffer, errors, `Contains`), `typed/stash_test.go`, `typed/unhandled_internal_test.go` (exported-names list only) | S1 to S7c |
| B | `WithStash`, `stash_withstash_test.go`, `unhandled_internal_test.go` (exported-names list only), `ExampleWithStash`, README section, `EXECUTION-RECORD.md`, cost-log row | S8, S9, S10 |

Budget: at most 400 authored changed lines per PR; estimates are raw times 1.55 (scope: A about 340, B about 330). B starts from the merged A.

## Procedure per PR

1. TDD inside the allowed files: scenario, watch it fail, minimum code. A test that fails on the current code where the implementation passes is a Stop.
2. Mutations per RULEBOOK section 5, by hand, listed in the PR "Mutations" table.
3. Gates (`AGENTS.md`), once per final tree:
   - `scripts/each_module.sh go test -count=1 ./...`, `go vet`, `gofmt -l`. Network isolation: `unshare -rn ...` on Linux; on macOS the evidence says so and the Owner runs the container variant.
   - `cd pekko-actor-typed && ../bin/golangci-lint run --config ../.golangci.yml ./...` (gocyclo at most 10)
   - coverage per module at least 85% (`scripts/check_coverage.sh`) and diff coverage at least 85% (`diff-cover --compare-branch=origin/develop`)
   - `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/each_module.sh go build ./...`, `.github/scripts/go-sdk-update.sh --check`
4. Before the PR, re-open every Pekko line range cited in the manifest, RULEBOOK, RUNBOOK and PR body in `upstream/pekko`; list them in the Evidence comment as `path:lines -> first line of the range`.
5. Record each gate in ONE Evidence comment bound to the final pushed SHA (command, environment, real `exit=`, output in `<details>`). Report real authored lines against the estimate. The Go judge is unaffected (no `pekko-actor/` file changes).
6. QA reviews on the final SHA, read-only, and reproduces the mutations (unmutated green, mutated red); every later commit invalidates the evidence.

## Manifest check

```
awk -F'\t' 'NR>1 && (NF!=7 || $1=="" || $2=="")' migration/stage-1/typed-889-stash-buffer/manifest.tsv
```

It must print nothing. Stops: RULEBOOK section 4, reported to the TL.
