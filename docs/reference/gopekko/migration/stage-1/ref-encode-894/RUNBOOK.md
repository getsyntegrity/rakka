# RUNBOOK: encode an exact local reference string (unit ref-encode-894)

Status: **prepared.** Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909` (verified: `git -C migration-kit rev-parse HEAD`). Base: `develop` `899d93e0f13f2e7673652b2a060329d9ee8f6b51`. Contract: scope #1288 final head `3ad7e7230ad14d5f00c5090443255ceab4cf4e18` and proposal #1291 final head `33d16d863aabb8abf1323aeff45c8cb337eb266d`, both **not merged**; G2 granted on `d65884dbbb5aa5c17b9efa197c43558b0385a6c7` and `5ac4ae89ecc47a1acbb989fec0d632ac4be04255` (Owner message of 2026-10-29, quoted in the scope). Every PR is a draft, `Refs #894`, never `Closes`; no merge, no issue edit, no #1020 index, no remote CI, no `-race`.

## PRs, order and bases

| PR | Branch (base) | Content |
|---|---|---|
| K | `docs/894-kit-package` (`develop`) | this package: `manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` |
| C1 | `feat/894-id-counter` (`docs/894-kit-package`) | `id.go` (guarded counter, `mint`, `idMint`), `ask_endpoint.go` (the `nextAskID` line), `spawn.go` (the reservation release only), `id_counter_internal_test.go` (C1-C8) |
| C2 | `feat/894-exact-ref-string` (`feat/894-id-counter`) | `ref_string.go`, `ref_string_test.go` (E1-E9), `pekko-actor/README.md` |
| R | `docs/894-execution-record` (`feat/894-exact-ref-string`) | `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row |

Order K -> C1 -> C2 -> R; each later PR is stacked and updated only by a normal merge of its predecessor (no rebase, no force-push). Each PR stays at or under 400 authored changed lines (additions plus deletions, merges and generated files excluded); if C1 or C2 passes it, the README, comments or an independent test file move to a follow-up PR stacked after it, contract unchanged.

## Procedure per code PR

1. TDD: write the tests first, observe RED (stub or unfixed variant, output kept), implement the minimum, GREEN. C7 is also run against the unfixed `commitCell` (a mutant) and must fail there.
2. Gates (commands of `AGENTS.md` "Go"), once per final head: `cd pekko-actor && go test -count=1 ./...` (and `scripts/each_module.sh go test -count=1 ./...` for the final stack; `unshare -rn` does not exist on macOS: record the exit, 127, accepted under G-6), `gofmt -l`, `go vet ./...`, `go mod tidy -diff` (labelled additions from `docs/quality-gates.md`), `scripts/check_deps.sh`, `.github/scripts/go-sdk-update.sh --check`, golangci-lint installed per `AGENTS.md` into the git-ignored `bin/` (`GOBIN=$PWD/bin go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@$(<.golangci-version)`; gocyclo at most 10), per-module coverage and `scripts/check_coverage.sh "$COV" 85`, diff-cover 11.0.0 at least 85% against `origin/develop` and, for a stacked PR, against its base branch, `make ci-fast`, `git diff --check`, `make parity-guard` on the final stack, and `.github/scripts/check-headers.sh` byte-compared with the output of the current develop (the 7 pre-existing errors: **stop on any new error**).
3. Differential on shared code: Spawn, SpawnAnonymous and Ask ID text and order, base versus change, on a scratch copy outside the repository; a micro-benchmark of `nextID` and `nextAskID` before and after (the CAS cost is a declared risk).
4. Quick own mutants on the counter (wrap instead of panic, add-then-check, two counters, release removed); QA runs the independent mutation round.
5. Record each gate with the pushed SHA, command, environment, exit code and tail. Remote CI is not run.

## Accepted exceptions (G-6) and not run

macOS without network isolation (`unshare` exit 127) and the 7 pre-existing `check-headers.sh` errors, only while the comparison with the current base is identical. Not run: remote CI, `-race`, the optional Classic serialize-after-stop scenario (E6: **not run, proposal only**; the Owner did not accept it).

## Stops

As in `RULEBOOK.md` section 6. Report to the Owner; never skip, delete or weaken a test.
