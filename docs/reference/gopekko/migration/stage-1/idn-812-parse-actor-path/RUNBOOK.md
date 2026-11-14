# RUNBOOK: parse local actor paths (unit idn-812-parse-actor-path)

Status: **prepared.** Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `develop` `abc94d55e635704db3ec30c6735227a4c9676048`. Contract: scope #1285 final head `f26fccf30533e6d1ffad134ac700f985a2d49ca6` and proposal #1290 final head `369597b8dd226da16a0e8faa047dc2690fe12aac`, both **not merged**; G2 granted by the Owner on 2026-10-29 (message in `RULEBOOK.md`). Every PR is a draft, `Refs #812`, never `Closes`; no merge, no issue edit, no #1020 index, no remote CI, no `-race`.

## PRs, order and bases

| PR | Branch (base) | Content |
|---|---|---|
| K | `docs/812-kit-package` (`develop`) | this package: `manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` |
| C | `feat/812-parse-actor-path` (`docs/812-kit-package`) | `parse_actor_path.go`, `parse_actor_path_test.go`, `errors.go` (doc comments only), `pekko-actor/README.md` |
| C2 | `feat/812-parse-position-tests` (`feat/812-parse-actor-path`) | `parse_actor_path_position_test.go`: the position rule against an independent grammar (X3, X7-X9), criterion 2b of #852 and AC3. Moved out of C because the tests together were 470 authored lines (338 in C, 132 in C2), over the 400 budget. **C must not merge without C2**: with C alone three mutants survive (system charset admitting `.`, whitespace trimming, a parser calling `nextID()`) |
| R | `docs/812-execution-record` (`feat/812-parse-position-tests`) | `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row |

Order K -> C -> C2 -> R; each later PR is stacked and updated only by a normal merge of its predecessor (no rebase, no force-push). Each PR stays at or under 400 authored changed lines (additions plus deletions, merges and generated files excluded); if C passes it, the tests or the README move to a second PR stacked after it, contract unchanged.

## Procedure per code PR

1. TDD: write the tests first, observe RED (a stub returning the zero `ActorPath` and a wrong error; output kept), implement the minimum, GREEN. Quick own mutants on the parser must be killed by the tests.
2. Gates (`AGENTS.md` "Go"), once per final head: `cd pekko-actor && go test -count=1 ./...` (once also `-count=3` for flakiness, not `-race`; `unshare -rn` does not exist on macOS: record the exit, 127, accepted under G-6), `gofmt -l`, `go vet ./...`, `go mod tidy -diff` (labelled additions from `docs/quality-gates.md`), `scripts/check_deps.sh`, `.github/scripts/go-sdk-update.sh --check`, golangci-lint into the git-ignored `bin/` (`GOBIN=$PWD/bin go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@$(<.golangci-version)`; gocyclo at most 10; never `//nolint`), per-module coverage and `scripts/check_coverage.sh "$COV" 85`, diff-cover 11.0.0 at least 85% against `origin/develop` and against the base branch, `make ci-fast`, `git diff --check`, `make parity-guard` on the final stack, and `.github/scripts/check-headers.sh` byte-compared with the output of the current develop (the 7 pre-existing errors: **stop on any new error**).
3. `git diff --stat` proves only the Allowed files changed and `address.go`, `actor_path.go`, `names.go` are untouched; no differential is needed (no existing function changes).
4. Record each gate with the pushed SHA, command, environment, exit code and tail. Remote CI is not run.

## Accepted exceptions (G-6) and not run

macOS without network isolation (`unshare` exit 127) and the 7 pre-existing `check-headers.sh` errors, only while the comparison with the current develop is identical. Not run: remote CI, `-race`, any new Pekko probe (the probe is **not extended**).

## Stops

As in `RULEBOOK.md` section 7. Report to the Owner; never skip, delete or weaken a test.
