# VALIDATION RECORD: local gates and review coverage (#161, PR #424)

Purpose: one consolidated record of the local validation and of what each native review covers. It claims nothing the commands below did not show. **CI has not run on any commit of this branch after `c4d3905`** because GitHub Actions does not start jobs (account billing, `AMENDMENTS.md` A-9); nothing here replaces it.

Validated tree: `pekko-actor` subtree `9124dc5ef9039a20a16cbea498f21e53203a4527`. It is identical at `8a252e9`, `1bc8ee3` and `de15c1f` (the later commits touch only `judge/` and `migration/`). Run on darwin/arm64, Go 1.27.2 unless noted, on 2026-10-12.

## 1. Local gates (observed)

| Gate (CI job it mirrors) | Command | Result |
|---|---|---|
| format, vet, tidy (`format`, `tidy`) | `gofmt -l`, `go vet ./...`, `go mod tidy -diff` in `pekko-actor` | clean |
| `go` (race tests, every module) | `go test -race -covermode=atomic -count=1 ./...` in each of the 5 modules | all `ok`; the 3 scaffold modules have no test files |
| coverage (minimum 85%) | `scripts/check_coverage.sh <profiles> 85` over the 5 module profiles | `pekko-actor` 98.58% (563 statements), `tools` 100.00% (37); 3 modules `skipped (no executable code)` |
| coverage gate selftest | `scripts/check_coverage_selftest.sh` | ok |
| `lint` (incl. gocyclo at most 10) | `golangci-lint` v2.14.0 (the pin in `.golangci-version`), `run --config ../.golangci.yml ./...` in each of the 5 modules | 0 issues in each |
| `headers` | `.github/scripts/check-headers.sh` | OK: 13 ported files checked |
| `go-version` | `.github/scripts/go-sdk-update.sh --check` | OK: 5 modules and `go.work` on go1.27.2 |
| dependency direction | `scripts/check_deps.sh` | ok |
| `adr-signatures` | `scripts/check-adr-signatures.sh` | ok |
| `modules` selftest | `scripts/modules_selftest.sh` | ok |
| `test (min Go)` | `GOTOOLCHAIN=go1.27.0`: `go build`, `go vet`, `go test -count=1` in `pekko-actor` and `tools`; build and vet in the 3 scaffold modules | all ok with Go 1.27.0 (`go.work` minimum) |
| `vuln` | `govulncheck@latest ./...` in `pekko-actor` | No vulnerabilities found (the only module with code; not run in the others) |
| `migration-kit-ext` | `python3 -I migration-kit-ext/test_depmap_scala.py` | OK |
| `judge-matrix` | `python3 -I scripts/check-judge-matrix.py` | exit 0; current Go campaign `detected 13/13`, `benign survived 1/1` |
| `judge-selftests` | the 10 commands of the CI job: `selftest`, `selftest_runner`, `selftest_backfill`, `selftest_calibrate`, `selftest_go_report`, `selftest_witness`, `selftest_pekko_source`, `calibrate.py selftest`, `selftest_go_scope`, `selftest_diagnose_go`, `selftest_mutate_go` | all pass (152, 95, 84, 262, 15 and 10 checks, 0 failed; the others print `ok`) |
| `judge-go` | `judge/mutate_go.py --runs 30` at `8a252e9` | 53 failed, 199 survived; `unmet: none`; no harness error or timeout |

Not run locally: the Pekko-side sbt oracle (so no parity claim), the nightly strict `govulncheck`, Linux. Profiles were named by the module-path convention of `check_coverage.sh --profile-name`; the first attempt with basename-named profiles failed closed, as designed.

## 2. Review coverage of HEAD

| Range | Commits | Native review | Result |
|---|---|---|---|
| `77a86d5..f90165a` | `4fc2748`, `f90165a` | `review-8dab1b644bea1a1d` (`sha256:de1005ac…`) | approved, acknowledged |
| `fcdf56b..c4d3905` | `ff1b525`, `c4d3905` | `review-a08b1337b5bb22bc` (`sha256:198a965f…`) | approved, acknowledged |
| `c4d3905..02b8e29` | `774e0cb`, `02b8e29` | `review-183e5dd85b4bb283` (`sha256:2bff5625…`) | approved, acknowledged |
| `21b041f..78502b5` | `78502b5` (approval package) | `review-e15bab03a0ba1e96` (`sha256:a63750c3…`) | passive approval (low risk, no lenses), acknowledged |
| `78502b5..8a252e9` | `7ef74f5`, `59bd95b`, `8a252e9` (kit port and docs) | `review-e37ef866843c6ee4` (`sha256:e531c576…`) | approved, acknowledged; 2 informational suggestions (A-11) |
| `8a252e9..de15c1f` | `1bc8ee3` (generated report), `de15c1f` (A-11 record) | none | `1bc8ee3` is campaign evidence only (its `pekko-actor` tree equals the validated one and `check-judge-matrix.py` passes); `de15c1f` is passive docs (2 files: the A-11 row of `AMENDMENTS.md` and one `EXECUTION-RECORD.md` cell) and has no native review of its own; `review-e37ef866843c6ee4` covers `78502b5..8a252e9` and does not cover it; its only check was a structural readback of the range, the trees and the lineage marker. For that lineage: the terminal-consumption marker for target `sha256:e531c576…` records `review-e37ef866843c6ee4` as acknowledged, which a native review reaches only after an approved result; `AMENDMENTS.md` A-11 records that the Owner granted consent. A different lineage id, `review-8b34fb2485e53b78`, has been cited for the same target; it appears in no repository file and in no local review state, so its correspondence with `review-e37ef866843c6ee4` is not established here and needs separate evidence. R3-001/R3-002 are recorded in A-11 and cannot be re-read from the burned state |
| `1b2fb2e..de1d89a` (PR #427, base `develop` before the merge, squash `051b41fe75736e1d38da9f22c50e2e7706ca0c0a`, head `de1d89aa010d8af895821d4c0be827f45ebea87f`) | the `de15c1f` A-11 record, as merged | `review-2e18d09c0d43992a` (target `sha256:dd24f298427133485a8cf4dac25a24ab56a89114bd014e7a0c0817d8100993fe`) | **post-merge review of a passive change; it did not gate the merge.** Risk low, reason `non_executable_only`, passive, no lenses, 1 file, 4 lines; approved and acknowledged. Source: the figures were reported by the native review of this lineage; they are recorded here as reported, and the review state is burned and cannot be re-read from the repository |
| `fcdf56b`, `21b041f` | generated reports | none | campaign evidence only, no semantic approval |
| whole range `77a86d5..` at `21b041f`, `78502b5`, `1bc8ee3`, `de15c1f` | all | candidates `665d09db…`, `3d2214c6…`, `6f557e99…`, `3a6a1f6e…` | **declined by the Owner**; no approval attributed to the whole range |

## 3. Delta of this record

This record and the PR description update are one docs commit after `de15c1f`; no native review of `de15c1f` is claimed here; any review of this docs commit is requested separately (see the PR). The `pekko-actor` tree does not change in it, so the Go campaign stays valid.
