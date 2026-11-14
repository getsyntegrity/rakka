# RUNBOOK: The innermost matching Typed supervisor decides (unit typed-1184-innermost)

Status: **prepared.** Sources pinned: Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `develop` `4b152df6` (the scope #1406, the oracle #1405 and the ADR 0035 amendment #1408 are merged). Delivery is four sequential PRs, each `Refs #1184`, never `Closes`, no merge without the Owner. The scope is `docs/scopes/1184-innermost-supervision.md`.

| PR | Content | Criteria |
|---|---|---|
| (done) | ADR 0035 amendment (D-5), merged before slice 1a | none |
| 0 | this package | kit run |
| 1a | `dedupeLayers` at the three sites in `typed/interpreter.go`; the two #976 tests adapted (D-2); `typed/nested_replacement_test.go` | T1a, T1b, T1c, T2, T3, T7 |
| 1b | innermost wins (tests only) and the title of `S8` | T4, T5, T6, T8 |
| 2 | README, `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row, the carried wording fixes, gate evidence bound to the final SHA | T9 |

Budget: at most 400 authored changed lines per PR. Re-estimate each at 1.55 times (the deviation measured in #976): 1a about 265, 1b about 170; slices 0 and 2 about 110 each, not scaled. A slice that projects over 400 is split before the PR is opened. Only 1a edits `typed/interpreter.go` and the two #976 test files.

## Procedure per PR

1. TDD: write each scenario, watch it fail, implement the minimum, inside the allowed files.
2. Mutations: for each behavior test, apply its mutation (RULEBOOK section 5) in a disposable worktree, watch the named test fail by assertion, restore. List them in the PR "Mutations" table.
3. Gates (commands in `AGENTS.md`, "Go"), run once per final tree:
   - `unshare -rn scripts/each_module.sh go test -count=1 ./...` (container variant where namespaces are restricted), `go vet`, `gofmt -l`
   - `golangci-lint run --config ../.golangci.yml ./...` in `pekko-actor-typed` (gocyclo at most 10)
   - module coverage at least 85% with `scripts/check_coverage.sh`; diff coverage at least 85% with `diff-cover --compare-branch=origin/develop`
   - `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `.github/scripts/go-sdk-update.sh --check`, `go mod tidy -diff`
4. Record each gate with the final pushed SHA, command, environment, a real exit code and the full output (one PR comment in a `<details>` block); the PR body names that SHA. Report the real authored-line count against the estimate. Every commit after the gates ran invalidates the evidence.
5. QA reviews each PR on its final SHA, read-only, and reproduces the mutations; an unbacked parity claim is CRITICAL (RULEBOOK section 6).

## Manifest check

```
awk -F'\t' 'NR>1 && (NF!=7 || $1=="" || $2=="")' migration/stage-1/typed-1184-innermost/manifest.tsv
node migration-kit/scripts/queue_runner.mjs --manifest migration/stage-1/typed-1184-innermost/manifest.tsv status
```

The first command must print nothing. Every cited Pekko line is re-opened by the Dev before the row is relied on. Test names marked "to be created by slice N" in the manifest exist only after that slice. A worktree whose `upstream/pekko` is empty: run `git submodule update --init --reference <main checkout>/upstream/pekko upstream/pekko` (QA of #1409), or read the main checkout's copy and verify its commit.

## Stops

See RULEBOOK section 4. Report to the TL.
