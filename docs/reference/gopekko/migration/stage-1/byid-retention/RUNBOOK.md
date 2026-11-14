# RUNBOOK: retirement of terminated actors from `System.byID` (ADR 0024, T2-T5; issue #541)

Status: PROPOSED for the Owner's approval of the package. **Execution is NOT authorized by this file**: it starts only when the Owner approves the hashes of section 9 and says to run. Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`). Contract: `RULEBOOK.md` (this directory). Base: branch `feat/actor-byid-retention` (ADR 0024 with D1-D5 accepted, from PR #542 head `7f52bb5`) on `develop` `cd558941`. The economical CI policy (`docs/ci-economy.md`) applies: CI runs only the fast check; race, mutants and review are local.

## 1. Scope
Exactly RULEBOOK sections 4 to 7: T2 (characterization tests), T3 (Ask provenance mark), T4 (native handles carry their terminal state), T5 (retirement in `complete`, guardians kept, tests, comments, adapted tests). **Not in scope:** T6 (field release), any change to the ID format, the judge and its campaign, the Pekko side, `go.mod`/`go.sum`, #443, the mailbox.

## 2. Preparation findings (by reading and by running the baseline; no code was changed)
- Baseline: `go test -count=1 ./pekko-actor/...` is green (`actor` 1.6 s, `internal/deadletter`, `internal/mailbox`), go1.27.2. Not measured: race, coverage, lint (the coordinator measures them at step 1).
- The consumers of `byID` were reviewed by reading (a delegated read-only review, not re-read in full by the coordinator). Findings that shaped the RULEBOOK: `Done` must check the owner of the handle (`behavior_test.go:142-150` expects `(closed, ErrUnknownHandle)` for a `Ref` of another System); hand-built handles and ghost cells have a nil `done` and must stay unknown; `ChildFailure.Child` is built outside `identity()` (`supervision_failure.go:251`); `complete` also receives the three guardians' signals; 8 `stats` reads after termination and 3 `len(byID)` assertions need adapting (RULEBOOK 7).
- The Go judge (`judge_internal_test.go`) was read by that review: it does not read the registry; its `Watch`/`Done`/`Stop` on terminated handles go through native refs, and retirement happens after the `terminated` and `done` events of `finalize`. By reading, no scenario changes. **The judge is not run** in this unit; this conclusion is a reading, not a measurement.
- Premises carried to the run, **verified by reading and not by running**: that retirement in `complete` does not reorder any event observed by the tests; that no consumer outside `pekko-actor/actor` reads the registry; that a nil `done` can only come from tests. A contradiction is a `BLOCKED(...)` or `NEEDS-DECISION` of the implementer, or a blocker of the coordinator with evidence.
- Environment: Go 1.27.2 with `-race`, `claude` CLI, `gtimeout`, python3. `gocyclo` is not installed locally; cyclomatic complexity is checked with the pinned `golangci-lint` v2.14.0 if its config enables it, otherwise by reading (to be stated in the record).

## 3. Documentation fixes included in this package commit
Text only: ADR 0024 status and decisions (D1-D5 accepted, with provenance) and the D5 row of ADR 0014. No Go touched.

## 4. Flow, limits and how coordination is counted
| Role | Model | Count |
|---|---|---|
| Implementer, production, tests and comments in **one session**, `run-agent.sh implementer` | `sonnet` | 1 |
| **One** focused review, `run-agent.sh review` (read-only, one-review lock, USD cap) | `opus` | 1 |
| Consultation on one concrete unresolved decision | `opus` | 0-1 |
| Gates, mutants, corrections of confirmed blockers, docs | the **coordinator** | 0 agents |
At most 3 agents. **Time: 45 minutes of active agent time**; `AGENT_WALL_MIN` implementer 30, review 12.
**What limits the review**, mechanically (CLI and script): the USD cap (3), the wall clock (12 minutes), read-only permissions, `review.lock`; by instruction only the 1,500-token report.
**What limits the implementer:** its template allows one wrapper run (`run-byid-tests.sh`: the whole module, then the `Retention` tests with `-race`); `go test` is otherwise denied; 6 USD cap and 30-minute clock.
**This delivery:** maximum **560,000** tokens, evaluation stop **450,000**. Coordination is a convention, not a measurement: `used = agents measured (input + output + cache creation) + reserve`, stated as an estimate. Reserve: **90,000** coordination, **40,000** for the mutants, **30,000** for the gates, plus **30,000** per correction round. Expected: implementer 200,000 to 300,000 (about 800 changed lines of code and tests, nine production files); review 70,000 to 110,000; central total about **470,000**. **The central estimate is above the evaluation stop (450,000).** The gates make the ceiling hold:
- The review is launched only if the implementer used at most 250,000 tokens and `observed + remaining reserve + 110,000 <= 560,000`.
- A correction round starts only if `observed + remaining reserve + 30,000 <= 560,000`; otherwise the coordinator stops and reports.
- At 450,000 the coordinator stops and asks before any correction round or extra agent, unless the Owner's approval said, in words, that crossing 450,000 is authorized.
- A run killed by the wall bound counts its whole forecast as spent. Nothing is consumed until the Owner approves and says to run.

## 5. Procedure after the Owner approves
1. **Baseline:** clean tree at the package commit; record `git rev-parse HEAD`, `git rev-parse HEAD:pekko-actor/actor`, `shasum -a 256` of every `.go` file of `pekko-actor/actor/` (outside the repository); submodules at `cf91c9d` and `9a54d1294c`; the nine hashes of section 9 compared (the RUNBOOK's own from `RESUME.md`); the baselines of section 2 recorded, plus `go test -race -count=1 ./pekko-actor/...`, coverage and lint at the base. Each command a separate call.
2. **Implementer** (`sonnet`, `AGENT_WALL_MIN=30`), command of section 8.
3. **Scope control:** `git diff --stat` and `shasum` against the baseline; only the surfaces of section 6 changed; no `judge/`, docs or module files.
4. **T2 on the base:** copy `registry_characterization_internal_test.go` into a scratch worktree of the package commit (outside the repository), run it there: it must pass without the production change. Record the result as the RED/GREEN evidence of the characterization (it is GREEN-before, GREEN-after by design; the retention tests are the ones that fail on the base).
5. **Gates by the coordinator:** `gofmt -l`, `go vet`, `go mod tidy -diff`, `golangci-lint` v2.14.0 (`.golangci-version`), coverage at least 85% (`scripts/check_coverage.sh`), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `go test -count=1 ./pekko-actor/... ./pekko-persistence/...`, then `go test -race` focused on the new tests and on the Watch, Stop, Done, Ask, Spawn, hierarchy and termination tests. The full module `-race` runs once if the focused run is clean and its cost is small (the module suite takes about two seconds without race); it is reported either way.
6. **Directed mutants** of RULEBOOK 8 in a scratch copy (outside the repository, never committed). Each must be detected by a named test.
7. **The one review** (`opus`, command of section 8) only if the forecast gate allows it. Findings cite a RULEBOOK rule or a source line; BLOCKER / DEFECT / INFO; INFO is recorded, not acted on. Fixes of confirmed findings by the coordinator, test first.
8. **Documentation by the coordinator,** once: `pekko-actor/README.md` (the limit at `:217-219`), `README.md` if it names the registry, ADR 0014 (lines 192, 207, 256, D5) and ADR 0015 (D6.3 and the notes at 252, 283, 307), ADR 0024 status, the execution record and `RESUME.md`.
9. **Commits** (Conventional Commits, no AI attribution lines): T2, T3, T4, T5, docs, each with its tests. **PR** in draft against `docs/byid-retention-design` (so it stacks on #542), linked to #541 and #542. No merge without the Owner's word. Authored size above 400 lines is stated in the PR with the per-commit sizes.
10. **#541 and #461** are updated by comment/body; nothing is closed.

## 6. Allowed edit surfaces (18 files)
Go production (9): `pekko-actor/actor/{registry,hierarchy,system,cell,ref,ask_endpoint,guardian,supervision_failure,errors}.go`.
New tests (3): `registry_characterization_internal_test.go`, `registry_retention_internal_test.go`, `registry_retention_api_test.go`, all under `pekko-actor/actor/`.
Existing tests (6, adaptation only): `lifecycle_internal_test.go`, `throughput_internal_test.go`, `termination_internal_test.go`, `hierarchy_internal_test.go`, `hierarchy_concurrent_internal_test.go`, `ask_endpoint_internal_test.go`.
Everything else is outside the surfaces. The permission template enforces these paths for Edit and Write.

## 7. Traceability
`manifest.tsv` maps each Pekko source and test, and ADR 0024/0015, to its Go target. RULEBOOK 2 states what was read in the pin and what is inference. The execution record (written by the coordinator after the run) states what the kit executed, with the measured costs, and which parts are Go adaptation with no Pekko source.

## 8. Exact execution commands (after approval)
Run from the root of the worktree on `feat/actor-byid-retention`, at the package commit. `D` is a directory **outside** the repository that exists; the three `printf` lines carry the approved hashes of section 9.
```
D=<directory outside the repository>
printf '%s  RULEBOOK.md\n' 8bf440bbdbc28efd3956e0f2564178db23c0297a58ae1ca744fe1e7f57590830 > "$D/rb.sha"
printf '%s  settings.implementer.template.json\n' d3a6f7526f649ab1dc1b8726cbe9298145d802609aaa5cdbe83144faa91e6af0 > "$D/st-impl.sha"
printf '%s  settings.review.template.json\n' 3e2da2a0e5b969642e9f6c5c4bbe62ebd90202409b870cf717da4bb41eb77214 > "$D/st-rev.sha"
AGENT_WALL_MIN=30 bash migration/stage-1/byid-retention/run-agent.sh implementer sonnet migration/stage-1/byid-retention/implementer-prompt.md "$D/implementer.json" "$D/rb.sha" "$D/st-impl.sha"
```
Review (step 7, only after steps 3 to 6 and the forecast gate):
```
AGENT_WALL_MIN=12 AGENT_BUDGET_USD=3 bash migration/stage-1/byid-retention/run-agent.sh review opus migration/stage-1/byid-retention/review-prompt.md "$D/review.json" "$D/rb.sha" "$D/st-rev.sha"
```
The launcher refuses a RULEBOOK or permission template whose hash differs (exit 3), renders the permissions for this worktree, bounds the wall clock, caps spend per agent (6 USD implementer, 3 USD review), refuses to overwrite an output, keeps a failed output as `<out>.failed.<N>`, and for the review creates `review.lock` before launching.

## 9. Hashes for approval
| File (in `migration/stage-1/byid-retention/`) | sha256 |
|---|---|
| `RULEBOOK.md` | `8bf440bbdbc28efd3956e0f2564178db23c0297a58ae1ca744fe1e7f57590830` |
| `settings.implementer.template.json` | `d3a6f7526f649ab1dc1b8726cbe9298145d802609aaa5cdbe83144faa91e6af0` |
| `settings.review.template.json` | `3e2da2a0e5b969642e9f6c5c4bbe62ebd90202409b870cf717da4bb41eb77214` |
| `run-agent.sh` | `19bc6fe9b79b7266104d7045fbde62396bd8c260923507d41fd4b0a737cc7690` |
| `run-byid-tests.sh` | `a987172d58c94316e249bae3285fa82b8c317c87070cfe535d4a8502da84b5f9` |
| `manifest.tsv` | `29e184b4f06a4d2563cd6902d4230fc36fec9a48a5314eda2a10b956ed5b0370` |
| `implementer-prompt.md` | `a2207229f18706102733e4b026a752ad036d0a05c17d9338a31e50320d46a242` |
| `review-prompt.md` | `5e8e21095d141d0019345aefec22aa4f50574d7a39ba2ebec0963a3c8d2997ef` |

The hash of this RUNBOOK cannot be inside it: it is given with the package in `RESUME.md`. The launcher mechanically checks only the RULEBOOK and the template of the role; the other hashes are compared by the coordinator in step 1. `run-agent.sh` is a byte copy of the one in `shutdown-u10` except the prefix of its temporary file name.

## 10. What approving this package authorizes
Approving the hashes authorizes **the execution of this unit only**, within 560,000 tokens and 45 minutes of agent time: the implementer run, the gates, the mutants and at most one review. It authorizes no merge, no native review, no second review, no judge run or campaign, no evidence tag, no closing of #541, no T6.

## 11. Issues
#541 is the tracking issue; PR #542 holds the ADR text. #461 is updated with the unit. No new issue is opened.
