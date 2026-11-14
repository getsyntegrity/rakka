# RUNBOOK: release of the fields of a terminated cell (ADR 0024 D5, task T6; issue #563)

Status: PROPOSED. The package is complete (section 3), but **execution is NOT authorized by this file: O0, the Owner's approval of the hashes of section 9 and of the budget of section 4, is pending.** Nothing in this directory records a run. Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1). Contract: `RULEBOOK.md` and `PROPOSAL.md` (this directory). Base of the analysis: `develop` `f23a215f9701dd7b5046dada0150ba75dfbe61c1`. The economical CI policy (`docs/ci-economy.md`) applies: CI runs only the fast check; race, mutants and review are local.

## 1. Scope
Exactly RULEBOOK sections 3 to 6 (R1-R4). **Not in scope:** releasing `owner` (a possible later unit), any change to `System.byID`, IDs, the restart path, exported API, the mailbox, #443, the judge and its campaign, `go.mod`/`go.sum`, ADR and README text (the coordinator updates docs after a run, RULEBOOK 5).

## 2. Preparation findings (by reading; nothing was run)
- The retention inventory, the release point and the late-caller table are in `PROPOSAL.md` sections 2 and 3, each row with a path and line.
- A grep of `*_test.go` for the candidate fields lists 14 files; the matches are live-cell reads, `sysq == 0` assertions, the guardian boot tests and `resume_with_internal_test.go:66`, which already asserts that a stopped cell releases its continuation. The run must re-check this by running the suite, not by the grep.
- Four existing tests contain the word `Release` in their name; the unit's tests use the prefix `TestCellRelease` and the wrapper selects `CellRelease`.
- Premises carried to the run, **verified by reading and not by running**: that no activation runs after `terminate` returns (PROPOSAL 3.3); that no late caller reads the release set (PROPOSAL 3.4). A contradiction is a `BLOCKED(...)` or `NEEDS-DECISION`.
- Not measured: race, coverage and lint at the base; the coordinator measures them at step 1 of section 5.

## 3. Package completeness (honest state)
| Protocol stage | Artifact | State |
|---|---|---|
| source and test | `manifest.tsv`; `PROPOSAL.md` section 1 | present |
| contract | `RULEBOOK.md`, `PROPOSAL.md` | present |
| unit and manifest | this file, `manifest.tsv` | present |
| launch artifacts | `implementer-prompt.md`, `review-prompt.md`, `settings.implementer.template.json`, `settings.review.template.json`, `run-agent.sh`, `run-release-tests.sh` | present, **checked for consistency, never run** |
| execution | `EXECUTION-RECORD.md` | **does not exist.** No kit run was made; the coordinator writes it after a run |
| evidence | gates, mutant outputs, review | none exists |

`run-agent.sh` is a byte copy of `migration/stage-1/byid-retention/run-agent.sh` except the prefix of its temporary file name. **It must be the copy in this directory.** The launcher resolves `RULEBOOK.md` and `settings.<role>.template.json` from its own directory; the `byid-retention` copy would hash that unit's rulebook and use that unit's permissions. `settings.review.template.json` is a byte copy of the one in `byid-retention` (read-only, no Edit or Write, no `go test`).

## 4. Flow, limits and how coordination is counted
| Role | Model | Count |
|---|---|---|
| Implementer, production, tests and comments in one session, `run-agent.sh implementer` | `sonnet` | 1 |
| One focused review, `run-agent.sh review` (read-only, one-review lock, USD cap) | `opus` | 1 |
| Consultation on one concrete unresolved decision | `opus` | 0-1 |
| Gates, mutants, corrections of confirmed blockers, docs | the coordinator | 0 agents |
**All of the following is a proposal awaiting O0, not an approved budget.** USD caps: implementer 4 (`AGENT_BUDGET_USD=4`; the launcher default is 6) and 20 minutes (`AGENT_WALL_MIN=20`; default 45); review 2 USD (`AGENT_BUDGET_USD=2`; default 3) and 10 minutes. Tokens: evaluation stop 350,000, ceiling 450,000; reserve 90,000 coordination, 30,000 gates, 30,000 mutants, 30,000 per correction round. These are an estimate scaled from `byid-retention/EXECUTION-RECORD.md` section 2 (PROPOSAL 9), not a measurement; the Owner may change them at O0. The launcher enforces the USD cap and the wall clock; the token figures are controlled by the coordinator reading the JSON after each agent. The model tiers are the same proposal.
- The review launches only if the implementer used at most 150,000 tokens and `observed + remaining reserve + 60,000 <= 450,000`.
- A correction round starts only if `observed + remaining reserve + 30,000 <= 450,000`; otherwise the coordinator stops and reports.
- At 350,000 the coordinator stops and asks before any correction round or extra agent, unless the approval said, in words, that crossing it is authorized.
- A run killed by the wall bound counts its whole forecast as spent. Nothing is consumed until the Owner approves and says to run.

## 5. Procedure after the Owner approves
Prerequisites on the machine: `claude` CLI, `timeout` or `gtimeout`, `python3`, `shasum`, Go as in `.go-version`, submodules initialized (`upstream/pekko` at `9a54d1294c`, `migration-kit` at `cf91c9d`).
1. **Baseline:** clean tree at the package commit rebased on current `develop` (re-hash if the rebase touched a hashed file); record `git rev-parse HEAD`, `git rev-parse HEAD:pekko-actor/actor`, `shasum -a 256` of every `.go` file of `pekko-actor/actor/` (outside the repository); compare **every** hash of section 9 by hand (the launcher checks two, section 9); `go test -count=1 ./pekko-actor/...`, `go test -race -count=1 ./pekko-actor/...`, coverage and lint at the base. Each command a separate call.
2. **Implementer**, command of section 8.
3. **Scope control:** `git status` and `shasum` against the baseline; only the four surfaces of section 6 changed; no existing test edited.
4. **RED/GREEN:** copy the two new test files into a scratch worktree of the base (outside the repository). `-run CellReleaseCharacterization` must pass there; the other `CellRelease` tests must fail there. Record both.
5. **Gates by the coordinator:** PROPOSAL 9, one command per call.
6. **Directed mutants** of PROPOSAL 8 in a scratch copy, never committed. Each is detected by a named assertion. M5 has no deterministic coverage: it is checked by inspection of the call position, and recorded as surviving if it is run and survives.
7. **The one review** only if the forecast gate allows it, command of section 8. Findings cite a RULEBOOK rule or a source line; BLOCKER / DEFECT / INFO; INFO is recorded, not acted on. Fixes of confirmed findings by the coordinator, test first.
8. **Documentation by the coordinator,** once: RULEBOOK 5 list, the execution record, `RESUME.md`.
9. **Commits** (Conventional Commits, no AI attribution lines): S1, S2, S3, S4. **PR** in draft against `develop`, linked to #563 with a non-closing reference. Authored size above 400 lines is stated in the PR with the per-commit sizes. No merge without the Owner's word.
10. **#563 and #461** are updated by comment; nothing is closed.

## 6. Allowed edit surfaces
Production (`pekko-actor/actor/`): `stop.go`, `cell.go`.
New tests: `cell_release_internal_test.go`, `cell_release_api_test.go`.
Everything else is outside the surfaces. `settings.implementer.template.json` grants Edit and Write on exactly these four paths (plus `gofmt -d` and `gofmt -w` on each), reads on the worktree, `gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...`, `go build ./pekko-actor/...` and `bash migration/stage-1/cell-release-563/run-release-tests.sh`; it denies `go test`, git write commands, `make`, `rm`, `mv`, `cp`, `curl` and `gh`. The same four paths are named in `implementer-prompt.md` and `RULEBOOK.md` section 5.

## 7. Traceability
`manifest.tsv` maps each Pekko source and test, and the ADRs, to its Go target. PROPOSAL 1 states what was read in the pin, what is port, what is adaptation and what is pending. The execution record states what the kit executed with measured costs.

## 8. Exact execution commands (after O0; read from `run-agent.sh`, not executed)
The launcher takes exactly six arguments: `<implementer|review> <model> <prompt-file> <out-json> <rulebook-sha256-file> <settings-sha256-file>`. Run from the root of the worktree at the package commit. `D` is a fresh directory **outside** the repository: the launcher refuses to overwrite `$D/implementer.json`, and for the review it creates `$D/review.lock` and never removes it (a second review needs the Owner to delete it).
```
cd "$(git rev-parse --show-toplevel)"   # the worktree whose HEAD is the package commit
D=$(mktemp -d)
printf '%s  RULEBOOK.md\n' d8fb1d848960f20c2329d7133f2526c6c2410a9dcb87a828a7f58c72ccccba5e > "$D/rb.sha"
printf '%s  settings.implementer.template.json\n' bde2f140b73110025a15371a294cd8a14463d12f8eee02f6a37168e1ea1d7f0e > "$D/st-impl.sha"
printf '%s  settings.review.template.json\n' 3e2da2a0e5b969642e9f6c5c4bbe62ebd90202409b870cf717da4bb41eb77214 > "$D/st-rev.sha"
AGENT_WALL_MIN=20 AGENT_BUDGET_USD=4 bash migration/stage-1/cell-release-563/run-agent.sh implementer sonnet migration/stage-1/cell-release-563/implementer-prompt.md "$D/implementer.json" "$D/rb.sha" "$D/st-impl.sha"
```
Review (step 7, only after steps 3 to 6 and the forecast gate):
```
AGENT_WALL_MIN=10 AGENT_BUDGET_USD=2 bash migration/stage-1/cell-release-563/run-agent.sh review opus migration/stage-1/cell-release-563/review-prompt.md "$D/review.json" "$D/rb.sha" "$D/st-rev.sha"
```
Exit codes: 2 bad usage or missing file; 3 hash mismatch; 4 review lock exists; 1 claude failed or exceeded a bound (the output is kept as `<out>.failed.<N>`).

## 9. Hashes
Algorithm `shasum -a 256`, over the file bytes.

| File (in `migration/stage-1/cell-release-563/`) | sha256 | Checked by the launcher |
|---|---|---|
| `RULEBOOK.md` | `d8fb1d848960f20c2329d7133f2526c6c2410a9dcb87a828a7f58c72ccccba5e` | **yes**, against `$D/rb.sha`, for both roles |
| `settings.implementer.template.json` | `bde2f140b73110025a15371a294cd8a14463d12f8eee02f6a37168e1ea1d7f0e` | **yes**, against `$D/st-impl.sha`, for `implementer` |
| `settings.review.template.json` | `3e2da2a0e5b969642e9f6c5c4bbe62ebd90202409b870cf717da4bb41eb77214` | **yes**, against `$D/st-rev.sha`, for `review` |
| `PROPOSAL.md` | `e3f2a619d7b986bfe419c91269ae8d3c84e3795d4e7944c6a3f62321ab750226` | no: the coordinator, step 1 |
| `manifest.tsv` | `092ce2803558f58b63edf0d559670f08b2bbf5072c46ed3459af570582337008` | no: the coordinator, step 1 |
| `implementer-prompt.md` | `e90048e578681e9197c390d34235f538ae69abc3e97ff3aebc2f53658d377d57` | no: the coordinator, step 1 |
| `review-prompt.md` | `51e39a3c075754d5a02847adbb1f59879432de747ad52da0c51f166ac0b26774` | no: the coordinator, step 1 |
| `run-agent.sh` | `d40c5de71ceff21f123859123dcc9fed9e69161c49a5005b71b58761887ff026` | no: the coordinator, step 1 |
| `run-release-tests.sh` | `9d1b9dfe3f3b11a61d78bb013227792f545bc9c15b68f2c13dd513dab6123524` | no: the coordinator, step 1 |

What the launcher really verifies: only the two files it reads from its own directory (the RULEBOOK and the template of the role). It does not hash the prompt, `PROPOSAL.md`, the wrapper, the manifest, its own script or the other role's template. The hash of this RUNBOOK cannot be inside it: it is given in `RESUME.md`, which is not hashed.

**Amendment after the run (not an approved change).** The hashes above were approved at `550bfa1` with `PROPOSAL.md` `48799cbe19f86b213c6034193fe77a1700920302fdf7f2c210974de83abfd444`. After the execution, the row of mutant M9 in `PROPOSAL.md` section 8 was corrected, because it said that T06 detects M9 and T06 asserts no counter (M9 was detected by the existing suite; see `EXECUTION-RECORD.md` sections 12.3 and 13). That is the only change to `PROPOSAL.md`; the hash in the table is the amended one. The three files the launcher checks (`RULEBOOK.md` and the two templates) and the prompts, the wrapper, the launcher copy and the manifest are byte-identical to what was approved, so no launch input changed. This section 9 and the RUNBOOK hash in `RESUME.md` changed with it. The earlier executions (the implementer run and the review) used the approved files; nothing here is evidence about a rerun.

## 10. What approving the package authorizes
Approving the hashes authorizes **the execution of this unit only**, within the budget of section 4 as the Owner sets it: the implementer run, the gates, the mutants and at most one review. It authorizes no merge, no native review, no second review, no judge run or campaign, no evidence tag, no closing of #563 or #461.

## 11. Issues
#563 is the tracking issue; ADR 0024 D5 is the accepted contract; #461 is updated with the unit. No new issue is opened.
