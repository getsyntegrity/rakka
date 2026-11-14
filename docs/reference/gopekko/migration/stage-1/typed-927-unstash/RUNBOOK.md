# RUNBOOK: Typed `Unstash` and `UnstashAll` (unit typed-927-unstash)

Status: **slices A1, A2, B and C executed.** Sources pinned: Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. The scope is `docs/scopes/927-typed-unstash.md`. Delivery is one PR per slice, each `Refs #927`, never `Closes` (slice C also `Refs #1169`); no merge without the Owner. Roles: Dev in a separate session, QA from a clean read-only context.

| PR | Content | Criteria |
|---|---|---|
| 0 | this package, `EXECUTION-RECORD.md`, cost-log row (separate docs PR, to keep A1 under 400 lines) | kit run |
| A1 | `typed/unstash.go`, `typed/unstash_test.go`, exported-names list, README section "Replay" | U1, U3, U4, U5, U8, D-6 |
| A2 | `Setup`/`Unhandled`/`Same` as the start, the record of an unhandled message, the interpreter link, `ErrUnstashUnhandled` | U2, U6, U7 |
| B | `unstashError`, supervision per strategy, ADR 0035 amendment | F1, F2 (F3, F4 PORT-GAP) |
| C | `Stopped` interrupts the replay, dead letters (#1169); code, tests, README and this package in one PR; started after #871, PR #1525, PR #1548 and slice B of #1182 | S1 to S5 |

Budget: at most 400 authored changed lines per PR; the scope estimate of A1 is about 220 to 250 (the engine alone), A1 measured 399 with the README and the tests; A2 measured 400; B measured 343 (PR #1542); C measured in the body of its PR, the package included.

## Procedure

1. TDD inside the allowed files: scenario, watch it fail, minimum code.
2. Mutations per RULEBOOK section 5, by hand, listed in the EXECUTION-RECORD "Mutations" table.
3. Gates (`AGENTS.md`), once per final tree: `gofmt -l`, `go vet`, `go test -race`, `golangci-lint` with `../.golangci.yml` (gocyclo at most 10), coverage per module at least 85% and diff coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `make ci-fast`, and `scripts/check-judge-matrix.py` (exit 0).
4. Before the PR, re-open every Pekko line range cited in the manifest and the PR body in `upstream/pekko`.
5. QA reviews on the final SHA, read-only, and reproduces the mutations (unmutated green, mutated red); every later commit invalidates the evidence.

## Manifest check

```
awk -F'\t' 'NR>1 && (NF!=7 || $1=="" || $2=="")' migration/stage-1/typed-927-unstash/manifest.tsv
```

It must print nothing. Stops: RULEBOOK section 4, reported to the TL.
