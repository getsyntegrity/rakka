# RUNBOOK: `DeadLetterSuppression`, `SuppressedDeadLetter` and divergence 5a (unit suppressed-dead-letters)

Status: **prepared.** Sources pinned: Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. The scope is `docs/scopes/871-suppressed-dead-letters.md`. Sequential PRs, each `Refs #871`, never `Closes`; no merge without the Owner. Roles: the Dev works in a separate session, QA reviews from a clean read-only context, the TL does not implement.

| PR | Content | Criteria |
|---|---|---|
| 0 | this package (apart from slice 1 so that the 400-line budget holds: slice 1 alone measures 398) | kit run |
| 1 | marker, `SuppressedDeadLetter`, `AllDeadLetters`, classification in `publish`, markers on `PoisonPill` and `Terminated`, the `Terminated` tests corrected (D-7) | S1 to S5 |
| 2 | 5a: the guard in `postLane`, `takePortDeadLocked`; README, ADR 0010 note, inventory C-10b (no V level); S10 and S11 (spill-over from slice 1); `EXECUTION-RECORD.md`; the `migration/cost-log.tsv` row | S6 to S11 |

Budget: at most 400 authored changed lines per PR, kit artifacts included; the real count is reported against the scope estimate.

## Procedure per PR

1. TDD inside the allowed files; the mutations of RULEBOOK section 4, by hand, listed in the PR.
2. Gates on the final head (`AGENTS.md`): gofmt, `go vet`, `go test -race`, 85% coverage per module (`scripts/check_coverage.sh`), diff coverage 85%, gocyclo at most 10 (`bin/golangci-lint`), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/each_module.sh go build ./...`, `make ci-fast`.
3. `scripts/check-judge-matrix.py` on the head; exit 1 is REJECT. The Go campaign after the merge of the code slices is run by the Owner's rule (full campaign on this Mac), not by this unit; until then the Go CURRENT of the checker is stale.
4. Re-open every Pekko line range cited in the PR; evidence as one comment bound to the final SHA with real exit codes. QA reviews the final SHA read-only and reproduces the mutations; any later commit invalidates it.

## Manifest check

```
awk -F'\t' 'NR>1 && (NF!=7 || $1=="" || $2=="")' migration/stage-1/suppressed-dead-letters/manifest.tsv
```

It must print nothing. Stops: RULEBOOK section 3, reported to the TL.
