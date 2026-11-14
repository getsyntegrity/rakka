# RUNBOOK: Supervision U8, judge scenarios S13, S14 and S16 in the Go gate (#175)

Status: PROPOSED for the Owner's approval of the package. **Execution is NOT authorized by this file**: it starts only when the Owner approves the hashes of section 9 and says to run. Unit U8 of ADR 0017, kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`). Contract: `RULEBOOK.md` (this directory). Base: `develop` at `b3275d740ad6454b5d8b7b497e4cd8430b01006f` (U7, PR #472; #449 closed). Delivery policy of 2026-10-15 and the economical CI policy (`docs/ci-economy.md`) apply: CI runs only the fast check; race, judge, mutants and review are local. Procedures not mentioned here are those of `../supervision-u7/RUNBOOK.md`.

## 1. Scope
Exactly RULEBOOK sections 5 to 10: the events and nodes of the Go judge, the bodies of S13, S14 and S16, the registration in `go-scope.json`, four directed mutants, the new Go campaign, and the docs of the Go scope. **Not in scope:** the Pekko-side scenarios, specs and mutations; `judge/comparator.py`; the runtime; calibration; S22 in the Go gate (decided against, RULEBOOK 7); U9; #450; `System.Shutdown`; the module-close campaigns of #461. **No native review is requested** (decision already taken by the Owner).

## 2. Preparation findings (by reading and by running the baselines; no change was made)
- **Blocker found in the design, resolved by D4:** Go's `sameMessage` is never true for a type with a `func` field, and the judge's `judgeMsg` has two. A literal port of S16 would make the `I6.b` no-op case a conflict (RULEBOOK 3).
- **Vacuity risk, resolved by D1 and D3:** without `pre-restart`, `decided` and instance numbers in the Go logs, `G7.b` and `I6.b` would pass vacuously (RULEBOOK 2).
- **Whole-gate effect:** adding scenarios to `go-scope.json` obliges every existing Go mutant to be judged on the new cells; the committed Go report (252 rows, Go commit `2427544c98`, darwin/arm64) goes stale. A new campaign is part of the unit.
- **S22:** decided against, with reasons (RULEBOOK 7); a decision for the Owner (D6).
- Baselines measured at `b3275d7`, to be recorded again before the run: `scripts/check-judge-matrix.py` passes (`OK: 38 relations, 65 mutations, 41 scenarios`; designed 35/35; observed Pekko `30/31`, relations `13/14`); `judge/selftest.py` `190 checks, 0 failed`; `selftest_go_scope.py` and `selftest_mutate_go.py` `ok`.
- The environment: Go 1.27.x with `-race`, Python 3, no sbt needed (the Pekko side is not touched).
- Premises carried to the run, **verified by reading and not by running**: that `check-judge-matrix.py` accepts the three scenarios in the Go scope once the milestone table says so; that none of the 14 existing mutants makes a new cell time out; that the four mutant anchors are unique and compile. A contradiction is a `BLOCKED(...)` or `NEEDS-DECISION` of the implementer, or a blocker of the coordinator with evidence.

## 3. Documentation fixes included in this package PR
Done in this PR, text only (README and ADR 0017; no Go touched): `README.md` (the U7 sentence: #449 closed, squash `b3275d7`); `docs/adr/0017-supervision.md` (D-5 row, the U7 row of the unit status table, the U7 row of the test table: delivered in squash `b3275d7`, issue closed, gaps in #461). #461 is updated (section 11).

## 4. Flow, limits and how coordination is counted
| Role | Model | Count |
|---|---|---|
| Implementer, Go harness, python and docs in **one session**, `run-agent.sh implementer` | `sonnet` | 1 |
| **One** focused review, `run-agent.sh review` (read-only, one-review lock, USD cap) | `opus` | 1 |
| Consultation on one concrete unresolved decision | `opus` | 0-1 |
| Gates, mutants, the Go campaign, corrections of confirmed blockers, docs | the **coordinator** | 0 agents |
At most 3 agents. **Time: 45 minutes of active agent time**; `AGENT_WALL_MIN` implementer 30, review 12. The Go campaign (section 5, step 8) is script time, run in the background, and is not agent time.
**What limits the review**, as in U7: mechanically (CLI and script) the USD cap (3) and the wall clock (12 minutes), read-only permissions, and `review.lock`; by instruction only the 1,500-token report. USD is a proxy for tokens: the U7 review cost 59,362 tokens at USD 0.71, the U6 one 111,745 at 1.11.
**What limits the implementer** (new, D7): its template allows one scoped run of the Go judge, `bash judge/run-go.sh /tmp/gopekko-u8-judge-out 1`, so it can see its scenarios run; `go test` stays denied otherwise; the 6 USD cap and 30-minute clock apply.

**This delivery:** maximum **560,000** tokens, evaluation stop **450,000**. Coordination is a convention, not a measurement: `used = agents measured (input + output + cache creation) + reserve`, stated as an estimate. Reserve: **90,000** coordination, **40,000** for the mutants and the campaign, **30,000** for the judge gates, plus **30,000** per correction round.
| Item | Last observed (U7 / U6) | U8 expected |
|---|---|---|
| Implementer | 195,900 / 252,891 | 220,000 to 320,000 (new events and nodes, three concurrent scenario bodies, a second actor type, four mutants, docs; it can run the judge once per iteration) |
| Opus review | 59,362 / 111,745 | 70,000 to 110,000 (USD cap 3) |
| **Agents** | 255,262 / 364,636 | **290,000 to 430,000** |
| Reserve (90,000 + 40,000 + 30,000) | 150,000 / 120,000 | 160,000 |
| **Total used (estimate)** | 465,262 / 544,636 | **450,000 to 590,000**, central about **515,000** |
**The central estimate is above the evaluation stop and the upper tail above the ceiling. I am saying this plainly: approving this package is asking the Owner to accept a central case that crosses 450,000.** The gates make the ceiling hold:
- The review is launched only if **the implementer used at most 250,000 tokens** and `observed + remaining reserve + 110,000 <= 560,000`; the first condition gives 250 + 160 + 110 = 520,000, leaving 40,000 for one correction round. Above 250,000 the coordinator stops and asks.
- A correction round starts only if `observed + remaining reserve + 30,000 <= 560,000`; otherwise it stops and reports.
- At 450,000 the coordinator stops and asks before any correction round or extra agent, unless the Owner's approval of this package said, in words, that crossing 450,000 for the review is authorized.
- 560,000 is the ceiling, never a target; it also absorbs an in-flight overshoot. A run killed by the wall bound counts its whole forecast as spent. Nothing is consumed until the Owner approves and says to run.

## 5. Procedure after the Owner approves
0. **Merge the package PR** by the Owner's word; create `feat/actor-judge-go-u8` from the resulting `develop`.
1. **Baseline:** clean branch; `git rev-parse HEAD:pekko-actor/actor`; `shasum -a 256` of every `.go` file of `pekko-actor/actor/` and of `judge/go-scope.json`, `judge/mutate_go.py`, `judge/selftest_go_scope.py`, `judge/selftest_mutate_go.py`, `judge/comparator.py`, `docs/judge.md`, `migration/stage-1/milestone-1.md`, written outside the repository; submodules at `cf91c9d` and `9a54d1294c`; the approved hashes of section 9 compared (including the RUNBOOK's own from `RESUME.md`); the baselines of section 2 recorded. Each command a separate call.
2. **Implementer** (`sonnet`, `AGENT_WALL_MIN=30`), command of section 8.
3. **Scope control:** `git diff --stat` and `shasum` against the baseline; only the surfaces of section 6 changed; the runtime and the Pekko side byte-identical.
4. **Gates by the coordinator:** `gofmt -l`, `go vet`, `go mod tidy -diff`, the pinned `golangci-lint` v2.14.0 (the installed 2.13.1 fails to load packages), coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/check-adr-signatures.sh`, `scripts/modules_selftest.sh`, `.github/scripts/go-sdk-update.sh --check`, `go test -count=1 ./...`, `go test -race -count=1 ./...`, the three judge self-tests and `scripts/check-judge-matrix.py` against their baselines (design lines hold; the observed lines recorded and reported).
5. **The full Go judge:** `judge/run-go.sh <outdir outside the repo> 300` for the 14 scenarios, exit 0; the new scenarios repeated under `-race` as run-go does. Record per scenario; separate **assertion failures** from **harness errors** from **timeouts**; a timeout or harness error is never a pass or a detection.
6. **Early signal on the four new mutants** (not the report): `python3 -I judge/mutate_go.py <workdir outside> --runs 30 GMrestartnotify GMinitrestart GMwatchoverwrite GMwatchsamefail`. Each must be `failed` with the designed clause; `survived`, `invalid-campaign`, `harness-error` or `timeout` is not a detection and is investigated (mutant redesign once, or a test fix by the coordinator).
7. **The one review** (`opus`, command of section 8) only if the forecast gate allows it. Findings cite a RULEBOOK rule or a source line; BLOCKER / DEFECT / INFO; INFO is recorded, not acted on. Fixes of confirmed findings by the coordinator, test first where they concern the harness.
8. **The final Go campaign:** with `pekko-actor/` **committed and clean** (the report is bound to the Go tree and flags a dirty one), `python3 -I judge/mutate_go.py <workdir outside> --runs 30 --report <outside>/go-mutation-report.tsv` for **all 18 mutants** (14 existing + 4 new), in the background. Compare with the committed report: the 14 existing mutants must keep their designed detections and the benign one must survive; the 4 new ones are met. Copy the report over `judge/go-mutation-report.tsv`, run the matrix check and record the Go lines (design apart from observed). Any change to `pekko-actor/` after this step requires repeating it.
9. **Documentation by the coordinator,** once: the execution record, `pekko-actor/README.md` (lines about S13/S14/S16 out of the Go gate), `docs/actor-core-parity.md` (rows for S16, S13, S14), `README.md`, ADR 0017 (U8 row, the A11 sentence, line about S13/S14/S16 and U7 S22), `RESUME.md`.
10. **PR** into `develop`, draft, commits of about 700 changed lines at most (harness first, mutants and scope next, docs and the report last; Conventional Commits, no AI attribution lines). No merge without the Owner's word.
11. **#175:** after the merge the coordinator checks the five criteria of RULEBOOK 10 and closes it **only if completely met and the Owner agrees**. #461 is updated.

## 6. Allowed edit surfaces (7 files)
Go (1): `pekko-actor/actor/judge_internal_test.go`.
Judge and docs (6): `judge/go-scope.json`, `judge/mutate_go.py`, `judge/selftest_mutate_go.py` and `judge/selftest_go_scope.py` (only if they hardcode the old lists), `docs/judge.md`, `migration/stage-1/milestone-1.md`.
Coordinator only, after the run: `judge/go-mutation-report.tsv` (regenerated), `pekko-actor/README.md`, `README.md`, `docs/actor-core-parity.md`, `docs/adr/0017-supervision.md`, `migration/stage-1/supervision-u8/EXECUTION-RECORD.md`, `RESUME.md`.
Denied by the rendered permissions: the runtime (`pekko-actor/actor/*.go` other than `judge_internal_test.go`), `judge/comparator.py`, `judge/mutate.py`, `judge/scenarios/`, `judge/pekko-harness/`, calibration, `scripts/`, workflows, the RULEBOOK. `go test` and `sbt` are denied to agents except the single scoped `run-go.sh` command of D7.

## 7. What U8 delivers and declares
Delivers S13, S14 and S16 in the Go gate with four directed mutants. Declared gaps kept: S22 not in the Go gate (RULEBOOK 7, D6); Pekko mutations M10, M11, M58 and M13 without a Go analog; no Go calibration row for the three scenarios; U9 logging; #450. The campaign is on darwin/arm64 only (the existing report's platform); there is no Linux evidence. No parity is claimed.

## 8. Exact execution commands (after approval and the merge of the package PR)
Run from the root of the worktree on `feat/actor-judge-go-u8`. `D` is a directory **outside** the repository that exists (create it first); the three `printf` lines carry the approved hashes of section 9.
```
D=<directory outside the repository>
printf '%s  RULEBOOK.md\n' 86205f18a761b25dcb66bc76cea3aa295a35d42b42af48ca25179ac77f3109a9 > "$D/rb.sha"
printf '%s  settings.implementer.template.json\n' cd6a2d0a089e474f2f348139313c162b8ade63d313ddabc0fbb2672770127170 > "$D/st-impl.sha"
printf '%s  settings.review.template.json\n' 70d32b49a66fc17bb7619cb5702c4f1e1dbbab77ebbe4e89d39401950696d844 > "$D/st-rev.sha"
AGENT_WALL_MIN=30 bash migration/stage-1/supervision-u8/run-agent.sh implementer sonnet migration/stage-1/supervision-u8/implementer-prompt.md "$D/implementer.json" "$D/rb.sha" "$D/st-impl.sha"
```
Review (step 7, only after steps 3 to 6 and the forecast gate):
```
AGENT_WALL_MIN=12 AGENT_BUDGET_USD=3 bash migration/stage-1/supervision-u8/run-agent.sh review opus migration/stage-1/supervision-u8/review-prompt.md "$D/review.json" "$D/rb.sha" "$D/st-rev.sha"
```
The launcher refuses a RULEBOOK or permission template whose hash differs (exit 3), renders the permissions for this worktree, bounds the wall clock, caps spend per agent (6 USD implementer, 3 USD review), refuses to overwrite an output, keeps a failed output as `<out>.failed.<N>` and, for the review, refuses a second launch (exit 4).

## 9. Hashes for approval
| File (in `migration/stage-1/supervision-u8/`) | sha256 |
|---|---|
| `RULEBOOK.md` | `86205f18a761b25dcb66bc76cea3aa295a35d42b42af48ca25179ac77f3109a9` |
| `settings.implementer.template.json` | `cd6a2d0a089e474f2f348139313c162b8ade63d313ddabc0fbb2672770127170` |
| `settings.review.template.json` | `70d32b49a66fc17bb7619cb5702c4f1e1dbbab77ebbe4e89d39401950696d844` |
| `run-agent.sh` | `87ff08d49dea1d8f01dbcfcf951c349d2e6a074e97c736b602cc04733b604e48` |
| `manifest.tsv` | `43e991bd5c97fe45a84ca79442dd62ec509fa91a332bb8ccf1bf44aedf135812` |
| `implementer-prompt.md` | `ab58a121c131da047197beae27cd6bcae7e2a41182ccf21e394d6cb050b192f1` |
| `review-prompt.md` | `cb5dbb70133ac1579dfbd004958016f41d3b9f129a42914597e25cb0e0d85746` |

The hash of this RUNBOOK cannot be inside it: it is given with the package in `RESUME.md`. The launcher mechanically checks only the RULEBOOK and the template of the role; the other hashes are compared by the coordinator in step 1.

## 10. What approving this package authorizes
Approving the hashes authorizes **the execution of U8 only**, within 560,000 tokens and 45 minutes of agent time, including the scoped judge run of the implementer (D7), the mutants, the Go campaign and at most one review. It authorizes no merge, no native review, no second review, no closing of #175, no S22 in the Go gate and no work on U9, #450 or the module-close campaigns. **It does not by itself authorize crossing the 450,000 evaluation stop**; the Owner may say so explicitly (section 4).

## 11. Reconciliation of #461 (done with this package)
Updated by editing the issue: the U7 texts that were stale (README, ADR 0017) are marked fixed in this PR; U8 is listed as in preparation with this PR; the Go-side gaps of U8 are added when it is executed (S22 not in the Go gate, Pekko mutations without a Go analog, no Go calibration row, darwin/arm64 only).
