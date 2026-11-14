# RUNBOOK: Typed Resume of a supervised behavior (unit typed-1183-resume)

Status: **prepared.** Sources pinned: Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `develop` `fa5b4bfb` (the scope, #1398). Delivery is six sequential PRs, each `Refs #1183`, never `Closes`, no merge without the Owner. The scope is `docs/scopes/1183-typed-resume.md`.

| PR | Content | Criteria |
|---|---|---|
| 0 | this package | kit run |
| 1a | `Resume()`, `ResumeStrategy`, `OnFailure` accepts it, the resume branch of `fail`, exported-surface list | S1 to S4 |
| 1b | signals and children (tests only) | S5, S6 |
| 2 | failed start under Resume (`typed/interpreter.go`, sequenced after 1a) | S8 |
| 3 | nesting with Restart (tests only) | S9 |
| 4 | README, `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row, gate evidence bound to the final SHA | S10 |

Budget: at most 400 authored changed lines per PR. The scope estimates are raw; re-estimate each at 1.55 times (the deviation measured in #976): 1a about 175, 1b about 125, 2 about 140, 3 about 100. A slice that projects over 400 is split before the PR is opened.

## Procedure per PR

1. TDD: write each scenario, watch it fail, implement the minimum, inside the allowed files. Slices 1a and 2 both edit `typed/interpreter.go`: never in parallel.
2. Mutations: for each behavior test, apply its mutation (RULEBOOK section 5) in a disposable worktree, watch the named test fail by assertion, restore. List them in the PR "Mutations" table.
3. Gates (commands in `AGENTS.md`, "Go"), run once per final tree:
   - `unshare -rn scripts/each_module.sh go test -count=1 ./...` (container variant where namespaces are restricted), `go vet`, `gofmt -l`
   - `golangci-lint run --config ../.golangci.yml ./...` in `pekko-actor-typed` (gocyclo at most 10)
   - module coverage at least 85% with `scripts/check_coverage.sh`; diff coverage at least 85% with `diff-cover --compare-branch=origin/develop`
   - `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `.github/scripts/go-sdk-update.sh --check`, `go mod tidy -diff`
4. Record each gate with the pushed SHA, command, environment, exit code and output (PR comment in a `<details>` block). Report the real authored-line count against the estimate.
5. QA reviews each PR on its final SHA, read-only, and reproduces the mutations; every commit after the gates ran invalidates the evidence.

## Manifest check

```
awk -F'\t' 'NR>1 && (NF!=7 || $1=="" || $2=="")' migration/stage-1/typed-1183-resume/manifest.tsv
```

It must print nothing. Every cited Pekko line is re-opened by the Dev before the row is relied on.

## Stops

See RULEBOOK section 4. Report to the TL.
