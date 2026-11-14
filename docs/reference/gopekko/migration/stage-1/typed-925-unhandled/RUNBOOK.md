# RUNBOOK: Typed Unhandled result and its interpretation (unit typed-925-unhandled)

Status: **prepared.** Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `develop` with the scope integrated (`f682e5f4f8648cf4afa13bd9f949629836e0cff4`). Delivery is draft PRs, each `Refs #925`, never `Closes`, no merge without the Owner.

| PR | Content | State |
|---|---|---|
| 1 | the scope (PR #1258) | integrated |
| 2 | this package | this PR |
| 3 | `Unhandled`, interpreter changes, `unhandled_internal_test.go` (U1-U9) | authorized by the Owner on 2026-10-28, as read (scope 11.1; U-2 (a), U-4, U-7 confirmed, RULEBOOK 4); starts after this package |
| 4 | README, `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row, gate evidence | authorized by the Owner on 2026-10-28, as read (scope 11.1), after PR 3 |

PR 3 and PR 4 run in that order; each is a draft that the Owner merges. The first version of this package made PR 3 wait for the confirmation of the U-2 reading; the confirmation is recorded (dated 2026-10-28, RULEBOOK 4).

## Procedure per PR (3 and 4)

1. TDD: scenario, RED, minimum implementation, inside the allowed files (RULEBOOK 4).
2. Gates (commands in `AGENTS.md`, "Go"), once per final tree, from the repository root unless noted:
   - unit tests with network isolation: `unshare -rn scripts/each_module.sh go test -count=1 ./...` (Linux; elsewhere the container variant `docker run --rm --network none -v "$PWD":/src -w /src <image> ...`, otherwise declare it not run)
   - in `pekko-actor-typed`: `go vet ./...`, `gofmt -l .`, `go mod tidy -diff`
   - complexity: `cd pekko-actor-typed && ../bin/golangci-lint run --config ../.golangci.yml ./...` (golangci-lint v2.14.0, gocyclo at most 10)
   - coverage per module at least 85% (`scripts/check_coverage.sh <profile-dir> 85`) and diff coverage at least 85% (`diff-cover ... --compare-branch=origin/develop --fail-under=85`)
   - `scripts/check_deps.sh` and `make parity-guard` (architecture lint and capability manifest, `AGENTS.md` "Go"), `.github/scripts/check-headers.sh`, `.github/scripts/go-sdk-update.sh --check`
   - no `-race`; no sleeps. CI is not awaited (remote CI is blocked by billing); attribute no remote result.
3. Record each gate with the pushed SHA, command, environment, exit code and output. Every commit after the gates invalidates the evidence.
4. In records keep three things apart: the **reviewed head** (the SHA QA reviewed), any **later head** (commits after that review) and the **integrated commit** (the squash on `develop`). Never present one as another.
5. Independent QA reviews each PR on its final SHA; merge only by the Owner, with `--match-head-commit` and without `--admin`.

PR 2 is documentation only: `gofmt`, `go vet`, `check-headers.sh` and the Go gates do not apply to it.

## Stops

See RULEBOOK 5. Report to the TL; never skip, delete or weaken a test.
