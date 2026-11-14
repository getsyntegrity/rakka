# RUNBOOK: the scheduler, first cut (unit scheduler-once-821)

Status: **prepared.** Sources pinned: Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. The scope is `docs/scopes/821-scheduler-first-cut.md`. Sequential PRs, each `Refs #821`, never `Closes`; no merge without the Owner. Roles: the Dev works in a separate session, QA reviews from a clean read-only context, the TL does not implement.

| PR | Content | Criteria |
|---|---|---|
| 0 | this package (apart from slice 1 so that the 400-line budget holds) | kit run |
| 1 | `Scheduler`, `System.Scheduler()`, `Cancellable`, `ScheduleOnce`, `ErrNilCallback`, its use of `Context.System()` (introduced by #1545), README | T1 to T8, T15, T16 |
| 1b | `ErrSchedulerClosed`, the registry of pending tasks, `close()` at System termination; merges right after slice 1 | T11 to T13 |
| 2 | `ScheduleTellOnce`, the oracle tests SC5once, README, inventory row C-20; its rows in `EXECUTION-RECORD.md` | T9, T10, T14, T17 |

The kit PR of slice 1 (PR 0 here) delivers `EXECUTION-RECORD.md` and the `migration/cost-log.tsv` row of the kit run; slices 1b and 2 add their own rows to the record and no second cost-log row.

Budget: at most 400 authored changed lines per PR, kit artifacts included; the real count is reported against the scope estimate.

## Procedure per PR

1. TDD inside the allowed files; the mutations of RULEBOOK section 4, by hand, listed in the PR.
2. Gates on the final head (`AGENTS.md`): gofmt, `go vet`, `go test -race`, 85% coverage per module, diff coverage 85%, gocyclo at most 10, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/each_module.sh go build ./...`, `make ci-fast`.
3. `scripts/check-judge-matrix.py` on the head; exit 1 is REJECT. The Go campaign after the merge of the code slices is run by the Owner's rule (full campaign on this Mac); until then the Go CURRENT of the checker is stale.
4. Re-open every Pekko line range cited in the PR; evidence as one comment bound to the final SHA; QA reviews the final SHA read-only and reproduces the mutations; any later commit invalidates it.

## Manifest check

```
awk -F'\t' 'NR>1 && (NF!=7 || $1=="" || $2=="")' migration/stage-1/scheduler-once-821/manifest.tsv
```

It must print nothing. Stops: RULEBOOK section 3, reported to the TL.
