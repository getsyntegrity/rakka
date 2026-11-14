# RUNBOOK: Typed Terminated delivery and death pact (unit typed-954-slice-b)

Status: **prepared.** Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `docs/954-slice-b-scope` (PR #1249, not merged). Delivery is stacked draft PRs, each `Refs #954`, never `Closes`, no merge without the Owner.

| PR | Content | State |
|---|---|---|
| B0 | the scope (PR #1249) | G2 approved |
| B0b | ADR 0034 amendment (P5 representation, P6 wording, `0035:205` label) | separate PR [#1251](https://github.com/getsyntegrity/gopekko/pull/1251); its text, and Appendix A of the scope, are **not accepted**: the Owner must review and accept it explicitly |
| B1 | this package | authorized |
| B2 | `OnTerminated`, the accepted `Terminated` representation, tests T1-T11, T15, T16 | **blocked until B0b is accepted** |
| B3 | T12-T14, README, `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row, gate evidence bound to the final SHA | **blocked until B0b is accepted** |

Order: B0b, then B2, then B3. B2 and B3 start only after the Owner accepts the ADR amendment.

## Procedure per PR

1. TDD: scenario, RED, minimum implementation, inside the allowed files (RULEBOOK 4).
2. Gates (commands in `AGENTS.md`, "Go"), once per final tree, from the repository root unless noted:
   - unit tests with network isolation: `unshare -rn scripts/each_module.sh go test -count=1 ./...` (Linux only; on macOS use the container variant `docker run --rm --network none -v "$PWD":/src -w /src <image> ...`)
   - in `pekko-actor-typed`: `go vet ./...`, `gofmt -l .`, `go mod tidy -diff`
   - complexity: `cd pekko-actor-typed && ../bin/golangci-lint run --config ../.golangci.yml ./...` (gocyclo at most 10)
   - coverage per module at least 85% (`scripts/check_coverage.sh <profile-dir> 85`) and diff coverage at least 85% (`diff-cover ... --compare-branch=origin/develop --fail-under=85`)
   - `scripts/check_deps.sh`, `make parity-guard`, `.github/scripts/check-headers.sh`, `.github/scripts/go-sdk-update.sh --check`
   - no `-race`; the unit tests use no sleeps.
3. Record each gate with the pushed SHA, command, environment, exit code and output. Remote CI is blocked by billing; attribute no remote result.
4. Independent QA reviews each PR on its final SHA; every commit after the gates invalidates the evidence.

B1 itself is documentation only: `gofmt`, `check-headers.sh` and the Go gates do not apply to it.

## Stops

See RULEBOOK 5. Report to the TL; never skip, delete or weaken a test.
