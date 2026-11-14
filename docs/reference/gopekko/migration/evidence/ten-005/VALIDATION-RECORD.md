# VALIDATION RECORD: local gates and review coverage (#393, PR #423)

Purpose: one record of the local validation of a docs-only candidate. It claims nothing the commands below did not show.
**Remote CI did not run** on this branch: GitHub Actions does not start jobs (account credit exhausted); the temporary
exception in `migration/stage-1/ask-endpoint-port/CI-EXCEPTION.md` applies, as authorized by the Owner. Nothing here
replaces remote CI, and it must be validated on `develop` when the credit is renewed.

**No Pekko oracle (sbt) was run and no parity claim is made.** Pekko v1.7.1 has no tenancy; the ADR is an own-extension
design (see the ADR, "Own extension versus upstream Pekko"). The migration kit was not executed (no port).

Candidate: `docs/ten-005-tenant-isolation-adr`, merged with `origin/develop` `91662c9`. Diff against develop: 3 files, all
Markdown (`docs/adr/0016-tenant-identity-and-isolation-model.md`, `migration/evidence/ten-005/*`). Run on darwin/arm64,
2026-10-12. ADR number 0016 is free on develop (it holds 0000, 0008, 0010 to 0012, 0014, 0015 and `signatures/`); there
is no ADR index file in the repository to update.

## 1. Local gates (observed)

| Gate (CI job it mirrors) | Command | Result |
|---|---|---|
| `adr-signatures` | `scripts/check-adr-signatures.sh` | exit 0 (every negative case rejected as expected) |
| `judge-matrix` | `python3 -I scripts/check-judge-matrix.py` | exit 0; Go campaign `detected 13/13`, `benign survived 1/1` |
| dependency direction | `scripts/check_deps.sh` | exit 0 |
| `headers` | `.github/scripts/check-headers.sh` | OK: 13 ported files checked |
| references | every repo path in backticks in the ADR and evidence exists (scripted check) | no missing path |
| upstream citations | each Pekko `file:line` cited was re-read in `upstream/pekko` v1.7.1 `9a54d12` | match; no `tenant`/`multitenant` in the tree |
| ASCII | `rg -P '[^\x00-\x7F]'` over the new files | none |
| RDD assessment | `gentle-ai review assess --base-ref origin/develop --committed-only --json` | risk `passive` (`non_executable_only`), `review_due: false` (`passive`): no native review and no consent envelope were issued |

## 2. Gates that do not apply

`go test -race`, coverage (85%), `golangci-lint`/`gocyclo`, `go vet`, `tidy`, `govulncheck`, `test (min Go)`,
`judge-go` and `judge-selftests` exercise Go code and the judge harness. The diff contains no `.go`, `go.mod`, `judge/` or
script change, so they test nothing this candidate changed. They were not run for this candidate and no result is
claimed for them. The ADR's invariants TI-1 to TI-7 have no scenario or test yet.

## 3. Review

An independent read-only reviewer examined the diff twice: findings on the first draft (21 items, one blocker) were fixed
in `771b802`; the re-review of that text (12 items, none blocking) was fixed in the commit after this record, and that
final text was not reviewed a third time. This is not a native RDD review (the
assessment was passive), and it is not a Pekko parity review.
