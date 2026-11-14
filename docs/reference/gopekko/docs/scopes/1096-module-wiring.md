# Scope: wire the `pekko-actor-typed` module (#1096, TYP-34)

Status: **approved** at gate G2 by the Owner in the working session of 2026-10-27 ("Apruebo G2 para #1096 con estos ajustes"; recorded from
the work thread, not a GitHub comment). The adjustments are in this file: the two placeholders (decision A) and `AGENTS.md` among the
allowed files. Authority: issue #1096, ADR 0033 D3 (accepted), and the Owner decisions of 2026-10-27 recorded in
`docs/parity/typed-owner-decisions-2026-10-27.md` (B-1, EXP-1) plus decision A below.

## Objective

Make `pekko-actor-typed` exist as a correctly wired, empty Go module, so that every later Typed story has a home that respects the
dependency direction. This scope adds wiring and documentation only: no behavior, no API, no export, no test, no change to
`pekko-actor`. It does not start #847 (TYP-02).

## Why this scope has two `doc.go` files (a deviation from "no `doc.go`")

The first intent was a module with no Go package at all. We tested it before writing this scope, in a temporary copy of `develop`
`25bbddcc88acfca5add9d931638dfaab6433dd03` outside the repository (the module had `go.mod`, the `go.work` entry, the `deps.allow` line
and a README, and no `.go` file), with go1.27.2 and golangci-lint v2.14.0 built with go1.27.2. Four gates of `AGENTS.md` fail:

| Gate | Result with no package |
|---|---|
| Unit tests (`go test ./...`, also through `scripts/each_module.sh`) | exit 1, `no packages to test` |
| Vet (`go vet ./...`) | exit 1, `no packages to vet` |
| Coverage (`go test -coverprofile ...`) | exit 1, and no profile is written, so `scripts/check_coverage.sh` says `FAIL (no profile ...)` |
| Lint (`golangci-lint run`) | exit 5, `no go files to analyze` |

With two placeholder packages, `spi/doc.go` and `internal/doc.go`, each holding only a package clause and its comment (the pattern
`pekko-observability/spi/doc.go` and `pekko-observability/internal/doc.go` already use), the same gates give exit 0, 0, 0, 0 and the coverage
script prints `skipped (no executable code)`. `list_modules.sh`, `check_deps.sh` and `go-sdk-update.sh --check` pass in both variants.

**Owner decision A (2026-10-27): placeholders in `spi/` and `internal/`, nothing in the module root.** At G2 the Owner authorized
exactly `pekko-actor-typed/spi/doc.go` and `pekko-actor-typed/internal/doc.go`, with only a package clause and a package comment, no
behavior and no export, and said this replaces the earlier choice of a module with no Go file. #1096 lists "any Go package" as out of
scope; these two placeholders are the only exception.

Rejected alternatives, so the choice can be argued:

- A `doc.go` in the module root: the module README contract in `CLAUDE.md` says module roots hold `README.md`, `go.mod` and `go.sum` with
  no `.go` file.
- An empty module and accepting the red gates until #847: it breaks the Definition of Done for this and every following scope.
- An empty module and changing the gates to skip it: outside this scope. It is tracked as its own story, #1224.

## Contracts consumed

- ADR 0033 D3 (accepted): separate module `pekko-actor-typed` depending on `pekko-actor`; the five edits listed under "Allowed files".
- ADR 0034, decision B-1 (accepted 2026-10-27, "partly accepted" ADR): used only to show that TYP-01 (#809) no longer blocks the
  skeleton. Nothing of its behavior is consumed. ADR 0034 and ADR 0035 stay "partly accepted"; nothing here declares them accepted in full.
- EXP-1 (Owner, 2026-10-27): replaces #1021 for the form of the export justification. This scope introduces no export, so it needs no
  per-export record.
- Module README contract and layout per module (`spi/`, `internal/`) in `CLAUDE.md`.

## Criteria → verification

Commands live in `AGENTS.md` ("Go"); this table names the gate, not the command line. "Level" is unit-level or repository-level
(scripts); there is no integration test and no Go test is added.

| # | Criterion | Verification | Level |
|---|---|---|---|
| C1 (AC-R1-1) | `pekko-actor-typed/go.mod` has module path `github.com/getsyntegrity/gopekko/pekko-actor-typed`, `go 1.27.0`, `toolchain go1.27.2` (as `.go-version`), one `replace` of `pekko-actor` to `../pekko-actor`, and no `require`. The module root has no `.go` file and no `go.sum`. | `ls` of the module, `go mod edit -json`, `go mod tidy -diff` (no change), `go-sdk-update.sh --check` | repository |
| C2 (AC-R2-1) | The workspace enumerates the module. | `scripts/list_modules.sh` prints it and exits 0 | repository |
| C3 (AC-R3-1) | `scripts/deps.allow` has exactly one new line: `pekko-actor-typed` may import only `pekko-actor`. The `pekko-actor` line is unchanged, so `actor` does not depend on Typed. | `scripts/check_deps.sh` prints `ok` for the module and exits 0; the diff shows one added line | repository |
| C4 (AC-R4-1) | `pekko-actor-typed/README.md` meets the module README contract: purpose, public usage, observable behavior, package responsibilities and current limitations; it says that no behavior is implemented yet, separates implemented from planned, links ADR 0033, 0034 and 0035 (the last two as "partly accepted") and has a Mermaid diagram of the dependency direction. It names no example, because none exists. | Reviewer checklist against the `CLAUDE.md` contract | review |
| C5 (AC-R5-1) | The dependency-direction bullet of `CLAUDE.md` names `pekko-actor-typed` and its single allowed dependency, `pekko-actor`, and keeps saying that `actor` imports no other goPekko module, Typed included. | Diff of `CLAUDE.md` (that bullet only) | review |
| C6 | `pekko-actor-typed/spi/doc.go` and `pekko-actor-typed/internal/doc.go` contain only the package clause and a package comment; no other `.go` file exists in the module. | `find` of the module's `.go` files; diff | repository |
| C7 | The gates treat the module correctly: unit tests exit 0 with no network, vet exits 0, the per-module coverage gate reports `skipped (no executable code)` for it (no profile is invented and no gate is changed), and `golangci-lint` from `bin/` exits 0. | The "Unit tests", "Coverage" (module row) and "Complexity" commands of `AGENTS.md`, `gofmt -l` | repository |
| C8 | The root `README.md` is consistent: the diagram has the edge `pekko-actor-typed` → `pekko-actor`; the Modules sentence says **eleven** entries and **eight** product modules; the Modules table has a row for the module (state: scaffold only, no behavior); the repository tree lists it; the Infrastructure row says eleven entries. No other statement is changed. | Diff of `README.md` limited to those five places | review |
| C9 | Nothing else changes: no path under `pekko-actor/`, `docs/adr/`, `.github/` or `scripts/` other than `scripts/deps.allow`, and no file outside the allowed list. | `git diff --name-only` against the allowed list | repository |
| C10 | `AGENTS.md` ("Modules" paragraph only) lists `pekko-actor-typed` among the product libraries and says it depends only on `pekko-actor`; nothing else in that file changes. | Diff of `AGENTS.md` | review |

## Allowed files

Exactly these paths; anything else needs a stop and a report to the TL.

| Path | Edit |
|---|---|
| `pekko-actor-typed/go.mod` | new |
| `pekko-actor-typed/README.md` | new |
| `pekko-actor-typed/spi/doc.go` | new, package comment only |
| `pekko-actor-typed/internal/doc.go` | new, package comment only |
| `go.work` | one `use ./pekko-actor-typed` line |
| `scripts/deps.allow` | one line (C3) |
| `CLAUDE.md` | the "Dependency direction" bullet only |
| `README.md` (root) | the five places of C8 only |
| `AGENTS.md` | the "Modules" paragraph only (C10), authorized by the Owner at G2 |
| `docs/scopes/1096-module-wiring.md` | this file; it ships with the PR |

There is one scope, so the independence table is trivial: no other scope shares these files. `go.sum` is not created (there is no
`require`).

Known drift that is **not** fixed here (the Owner kept them out at G2) and goes into the PR as a note: `.github/labeler.yml` and
`.github/dependabot.yml` list four modules and already omit three product modules.

## Toolchain and evidence rules

- The Dev works with go1.27.2, as `.go-version` requires. If the machine has only go1.27.1, use
  `go install golang.org/dl/go1.27.2@latest && go1.27.2 download`, or `GOTOOLCHAIN=go1.27.2` when there is network.
- `golangci-lint` is rebuilt into `bin/` (untracked, `/bin/` is in `.gitignore`) with go1.27.2 at the tag in `.golangci-version`, per
  `AGENTS.md`. Evidence produced with go1.27.1 or with another lint binary does not count for this scope.
- Unit tests run with no network (`unshare -rn`, per `AGENTS.md`). No `-race`, no workbench, no campaign, no Pekko scenario.
- Each gate's **full output** goes in a PR comment, in a `<details>` block whose heading states the pushed SHA, the command, the
  environment (OS, `go version`, `golangci-lint version`) and the exit code. The PR description summary does not replace it.
- Remote CI cannot run (GitHub Actions billing). The PR says so and attributes no remote result.
- Diff coverage is not applicable: the diff adds no Go statement. The PR says so instead of a number.
- **Proportional checks, no repetition.** The gates that visit the whole workspace (`list_modules.sh`, `check_deps.sh`,
  `go-sdk-update.sh --check`) run once on the final tree. The module gates (unit tests, vet, coverage row, lint, `gofmt`, `go mod tidy
  -diff`) run for `pekko-actor-typed` only: no other module's code, `go.mod` or `go.sum` changes, so their results are not re-run. A
  gate whose inputs are identical to a run already published (same tree hash, same command, same environment) is not repeated: the
  comment cites that run's SHA and tree hash. A later commit that only touches documentation re-runs only what its files can change.
  No artificial test is added to make a number appear.

## Delivery rules

- The Dev starts from the commit that carries this scope: branch `ao/gopekko-execution-focus/typed-34-wiring` in the shared local
  repository (create the Dev's own branch from it; do not rewrite that commit).
- One **draft** PR, body starting with `Refs #1096` (not `Closes`), listing the files excluded from the line budget (none expected).
  Authored lines are about 260 (this scope is 151 of them), under the 400 budget.
- Conventional Commits, for example `chore(actor-typed): wire the pekko-actor-typed module skeleton`. No `Co-Authored-By` line.
  Normal push, never a force-push. No merge. #1096 stays open.
- Not in this scope: #847, #1129, #1098, #1112, persistence or CQRS work, #1014, closing #809 or #1021, relabeling other stories.

## Stop conditions

The Dev stops and reports instead of improvising when: a gate fails with the placeholders; a file outside the allowed list must change;
go1.27.2 cannot be obtained; or the work would pass the line budget.

## ADR impact

Recorded after gate G2 and before implementation: **no ADR is amended.**

| ADR | Effect of this scope |
|---|---|
| 0033 (accepted) | D3 is applied as written (module, `go.work`, one `deps.allow` line, `go.mod` with `replace`, README, `CLAUDE.md` text). Nothing in it changes. |
| 0034 (partly accepted) | Not amended. B-1 is cited only to show that TYP-01 no longer blocks the skeleton. It stays partly accepted. |
| 0035 (partly accepted) | Not amended and not consumed. It stays partly accepted. |
| 0011 | Untouched: no export is added, so no per-export record is needed (EXP-1). |
| 0010, 0017 and the rest | Untouched. |

Documentation outside the ADRs that changes: `CLAUDE.md` (dependency direction), `README.md`, `AGENTS.md` ("Modules"), the new module README and
this scope, all within the allowed list.

## Appetite

One Dev session, small. A second session or more than 400 authored lines means the scope was wrong: stop and report.

## Definition of Done

- C1 to C10 each have evidence bound to the final pushed SHA, as PR comments in the format above.
- The gates of `AGENTS.md` that apply give the results in C7; `scripts/check_deps.sh` prints `ok` for the module; no function is added, so
  the complexity limit (10) is met trivially.
- A QA session, new and independent of the TL and the Dev, issues a verdict on the exact PR HEAD, or rejects it naming the missing entry.
- The PR is a draft, ready for the Owner's review. It is not merged.
