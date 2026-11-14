# RUNBOOK: Typed `Stopped` and `StoppedWith`, slice A (unit typed-1182-stopped)

Status: **slices A and B executed** (B: PR #1545 and `EXECUTION-RECORD-B.md`). Sources pinned: Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. The scope is `docs/scopes/1182-typed-stopped.md`. Delivery is one PR per slice, each `Refs #1182`, never `Closes`; no merge without the Owner. Roles: Dev in a separate session, QA from a clean read-only context.

| PR | Content | Criteria |
|---|---|---|
| 0 | this package, `EXECUTION-RECORD.md`, cost-log row (separate docs PR, to keep A under 400 lines) | kit run |
| A | `typed/behavior.go`, `typed/interpreter.go`, `typed/stopped_test.go`, `ExampleStoppedWith`, exported-names list, README section (PR #1529) | R1 to R5 (R3e and R6 are PORT-GAP) |
| B | `Context.System()` (or its use), held inputs become dead letters (code PR #1545; record `EXECUTION-RECORD-B.md`, cost-log row and manifest rows in this docs PR) | R7, R8; started after #871 and PR #1525 were merged |

Budget: at most 400 authored changed lines per PR; the estimate of slice A is about 330 (the scope table).

## Procedure

1. TDD inside the allowed files: scenario, watch it fail, minimum code.
2. Mutations per RULEBOOK section 5, by hand, listed in the EXECUTION-RECORD "Mutations" table.
3. Gates (`AGENTS.md`), once per final tree: `gofmt -l`, `go vet`, `go test -race`, `scripts/each_module.sh go test -count=1 ./...`, `golangci-lint` with `../.golangci.yml` (gocyclo at most 10), coverage per module at least 85% (`scripts/check_coverage.sh`) and diff coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `make ci-fast`, and `scripts/check-judge-matrix.py` (exit 0).
4. Before the PR, re-open every Pekko line range cited in the manifest and the PR body in `upstream/pekko`.
5. Record each gate in ONE Evidence comment bound to the final pushed SHA. Report real authored lines against the estimate.
6. QA reviews on the final SHA, read-only, and reproduces the mutations (unmutated green, mutated red); every later commit invalidates the evidence.

## Manifest check

```
awk -F'\t' 'NR>1 && (NF!=7 || $1=="" || $2=="")' migration/stage-1/typed-1182-stopped/manifest.tsv
```

It must print nothing. Stops: RULEBOOK section 4, reported to the TL.
