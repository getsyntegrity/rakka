# RUNBOOK: Supervision U7, Stop during a restart and the scope of G4/G5 (#449)

Status: PROPOSED for the Owner's approval of the package. **Execution is NOT authorized by this file**: it starts only when the Owner approves the hashes of section 9 and says to run. Unit U7 of ADR 0017 (S6, S11, H4, D-5), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`). Contract: `RULEBOOK.md` (this directory). Base: `develop` at `e0d70bbb85d42f41d9fb6a786de001f6afe02389` (U6, PR #470; #447 closed). Delivery policy of 2026-10-15 and the economical CI policy (`docs/ci-economy.md`) apply: CI runs only the fast check; race, mutants, review, the judge and the Pekko run are local. Procedures not mentioned here are those of `../supervision-u6/RUNBOOK.md`.

**Revision 2 (after the Owner's technical review of the first package).** Verified against `judge/comparator.py` and `Judge.scala`: (1) `required.count` is a minimum and cannot be below 1, so S22 gets a new exact-count clause `G5.f` and `G5.d` is no longer edited; (2) `G5.c` compares the parent's last `post-stop-begin` with each child's last `post-stop`; (3) S22 is `concurrent` only, because a latch in `controlled` mode (everything on the driver thread) would block the driver before the Stop; (4) the matrix criterion is design coverage, demonstrated by running the check; the observed line is recorded and reported, not a gate. Budget, launcher and run procedure are unchanged.

**Revision 3 (Owner's review of revision 2).** Accepted: `ce[-1]` in `G5.c`, S22 `concurrent` only, and a matrix criterion that does not require the observed line to stay 14/14. Added: `judge/selftest.py` as a **ninth surface** with the cases J1-J12 of RULEBOOK 7.4 (P2 absent or extra, a second instance, a duplicated `Terminated`, prefixes, the `G5.c` correction) and three judge mutants MJ1-MJ3; the implementer template, the manifest, both prompts and the budget change. The pending Pekko campaign for `G5.f` is recorded in #461.

## 1. Scope
Exactly RULEBOOK sections 4 to 7: the Go change of `defaultPreRestart`, the text of G4/G5, the comparator changes (`r_G5c` final-versus-final and the new clause `G5.f`) with their cases J1-J12 in `judge/selftest.py`, the Pekko-side scenario S22 (concurrent) with its spec, the Go mirror tests W11b and T1-T4, the three mutants of 7.2 and the Pekko run of S22. **Not in scope:** U8 (restart scenarios or events in the Go judge, `go-scope.json`), U9 (logging), #450, `System.Shutdown`, the module-close campaigns of #461, and any edit of `stop.go`, `cell.go`, `hierarchy.go` or `guardian.go`. **No native review is requested** (decision already taken by the Owner).

## 2. Preparation findings (by reading and by running the baselines; no change was made)
- The whole production change is in `defaultPreRestart` (`supervision_restart.go:348-358`): it is the only place where P1 goes through `beginPostStop`. `finalize` (`stop.go:92-97`) keeps the guard. `c.actor` is replaced only in `installNew` (`:418`), so P2 reaches the old instance. **No edit of an anchor-bearing file is expected**; `supervision_restart.go` carries no judge anchor.
- Three existing comparator behaviors cannot judge the accepted case: `r_G5c` (first `post-stop-begin` of the parent against the first `post-stop` of each child), `G5.d` (`post-stop` count above `pre-start` count) and `G4.a` (requires `post-restart`). `r_G5c` is changed to final-versus-final (identical for one `post-stop` each); `G5.d` and `G4.a` are not used by S22 and not edited; `required` counts are minimums of at least 1, so the exact counts come from the new clause `G5.f` (RULEBOOK 6.2).
- Baselines measured at `e0d70bb`: `python3 -I scripts/check-judge-matrix.py` passes and prints `OK: ...`, a designed-coverage line (`designed 35/35 ...`) and `relations with every clause observed: 14/14 (Pekko)`; `judge/selftest.py` `152 checks, 0 failed`; `judge/selftest_go_scope.py` and `judge/selftest_mutate_go.py` print `ok`. Afterwards: the `OK:` line and the self-tests must hold and the designed-coverage line must not drop; the observed line is recorded before and after with the check's own output and reported (it can change because `G5.f` has no Pekko mutation campaign), and it is never restored by removing or weakening a clause, mutation or scenario.
- The environment can run the Pekko side: `sbt` and Java 11.0.32 are installed (sdkman); `judge/run-pekko.sh` and `migration/stage-1/supervision/h4-repro/run.sh` show the recipe.
- Pre-existing and not caused by U7: the matrix check prints `goPekko CURRENT ... detected 0/13`; it is not touched or claimed.
- Premises carried to the run, **verified by reading and not by running**: that `scripts/check-judge-matrix.py` accepts a new clause asserted by one scenario and needs only `milestone-1.md` for a new scenario (if it demands more, `BLOCKED(surface)` with its message); that the held child of `sup2NewWait` lets T1 observe P1 before and P2 after the child's final stop. The prompt asks the implementer to report a contradiction as `NEEDS-DECISION` or `BLOCKED(surface)`.

## 3. Documentation fixes included in the package PR
Done in this PR, comment and documentation only, verified with `gofmt` and `go vet`: `README.md` (U6 row: #447 closed, PR #470); `docs/adr/0017-supervision.md` (rows of U6 in the status table, in the unit table and in the `SupervisorHierarchySpec:767` row: delivered, issue closed, gaps in #461); `pekko-actor/actor/guardian.go:135` (the comment no longer says the `options.bootPoint` doc "predates this layout"). #461 is reconciled by editing the issue (section 11).

## 4. Flow, limits and how coordination is counted
| Role | Model | Count |
|---|---|---|
| Implementer, Go and judge files in **one session**, `run-agent.sh implementer` | `sonnet` | 1 |
| **One** focused review, `run-agent.sh review` (read-only permissions, one-review lock, USD cap) | `opus` | 1 |
| Consultation on one concrete unresolved decision | `opus` | 0-1 |
| Mutants, gates, the S22 run on Pekko, corrections of confirmed blockers, docs | the **coordinator** | 0 agents |

At most 3 agents. **Time: 45 minutes of active agent time** plus the coordinator's runs; `AGENT_WALL_MIN` implementer 30, review 12.

**What actually limits the review.** Mechanical, enforced by the CLI and the script: `--max-budget-usd` (default 3 for the review role) and the wall clock (12 minutes, `timeout -k 30s`); read-only permissions by the rendered settings; and `review.lock`, created before the launch and never removed, so a second launch exits 4. By instruction only (not enforced): the 1,500-token report and the read order. USD is a **proxy** for tokens, not a token limit: the U6 review cost 111,745 tokens at USD 1.11. **Change from U6:** the review template now allows `git diff <rev> -- <path>` and `git show`, because the U6 reviewer lost three hunks when its shell was denied; the prompt also tells it to report anything it could not read. If the cap truncates the review, its output is read from `.failed.1` and recorded as incomplete; no second review without the Owner.

**This delivery:** maximum **560,000** tokens, evaluation stop **450,000**. Coordination is a convention, not a measurement: `used = agents measured (input + output + cache creation) + reserve`, stated as an estimate. Reserve: **90,000** coordination, **30,000** for the three mutants, **30,000** for the S22 runs and the judge gates, plus **30,000** per correction round.
| Item | Last observed (U6 / U5c) | U7 expected |
|---|---|---|
| Implementer | 252,891 / 99,350 | 160,000 to 270,000 (under 15 production lines; two Go test files of about 350 lines; a Scala scenario, a JSON spec, a rule edit and a new clause in the comparator, about 120 lines of python self-test cases, two docs) |
| Opus review | 111,745 / 104,415 | 80,000 to 115,000 (USD cap 3) |
| **Agents** | 364,636 / 203,765 | **240,000 to 385,000** |
| Reserve (coordination 90,000, mutants 30,000, Pekko runs and judge gates 30,000) | 120,000 / 90,000 | 150,000 |
| **Total used (estimate)** | 544,636 (two correction rounds) / 293,765 | **390,000 to 535,000**, central about **455,000**; with one correction round up to about 565,000, above the ceiling, so a correction round starts only if `observed + remaining reserve + 30,000 <= 560,000` |
**The central estimate sits on the evaluation stop and the upper tail on the ceiling.** So: **the review is launched only if the implementer used at most 200,000 tokens** and `observed + remaining reserve + 115,000 <= 560,000`; above that the coordinator stops and asks. At 450,000 it stops and asks before any correction round or extra agent. The 560,000 maximum also absorbs an in-flight overshoot and is never a target. A run killed by the wall bound counts its whole forecast as spent. Nothing is consumed until the Owner approves and says to run.

## 5. Procedure after the Owner approves
0. **Merge the package PR** by the Owner's word; create `feat/actor-stop-restart-u7` from the resulting `develop`.
1. **Baseline:** clean branch; `git rev-parse HEAD:pekko-actor/actor` and `shasum -a 256` of every `.go` file of `pekko-actor/actor/`, plus `judge/comparator.py`, `judge/pekko-harness/Judge.scala`, `docs/judge.md` and `migration/stage-1/milestone-1.md`, written outside the repository; submodules at `cf91c9d` and `9a54d1294c`; the approved hashes of section 9 written to files outside the repository and compared, **including the RUNBOOK's own hash from `RESUME.md`**; the baselines of section 2 recorded. Each command a separate call.
2. **Implementer** (`sonnet`, `AGENT_WALL_MIN=30`), command of section 8.
3. **Gates by the coordinator:** `gofmt -l`, `go vet`, `go mod tidy -diff`, the pinned `golangci-lint` v2.14.0 (the installed 2.13.1 fails to load packages), coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/check-adr-signatures.sh`, `scripts/modules_selftest.sh`, `.github/scripts/go-sdk-update.sh --check`, `go test -count=1 ./...`, the three judge self-tests and `scripts/check-judge-matrix.py` against their baselines (design lines hold; the observed line recorded and reported); `stop.go`, `cell.go`, `hierarchy.go`, `guardian.go` byte-identical; `judge/go-scope.json` unchanged. An anchor that must change stops the work.
4. **RED/GREEN:** W11b and T1 against the base `defaultPreRestart` (this is also MU3): they must be RED; with the change GREEN. Recorded.
5. **One `go test -race -count=1 ./...`**, and W11, T1-T4 repeated 30 times under `-race`.
6. **Pekko side:** a disposable export (`git -C upstream/pekko archive --format=tar v1.7.1`, unpacked outside the repository and committed and tagged `v1.7.1` in a throwaway git repository, as `h4-repro/run.sh` documents), then `judge/run-pekko.sh <copy> <out> concurrent 10000 S22`, 3 runs; judge each log with `judge/comparator.py --spec judge/scenarios/S22.json --mode concurrent` and compare the counts and the order with `RESULTS.md` section H4. `upstream/pekko` is never modified. If the run cannot be made, #449 criterion 3 stays unmet and the unit says so.
7. **Mutants MU1, MU2, MU3** (RULEBOOK 7.2) and **MJ1, MJ2, MJ3** (RULEBOOK 7.4, python, on a copy of `judge/` outside the repository), one at a time on fresh copies outside the repository with a 900 s package timeout; record the patch, the tests that failed, and that the unmodified copy passed. INVALID runs are listed apart from detections and redesigned once.
8. **The one review** (`opus`, command of section 8) only if the forecast gate allows it.
9. **Scope control after the fact:** `git diff --stat` and `shasum` against the baseline; any file outside the surfaces of section 6 is a defect.
10. **Documentation by the coordinator,** once: the execution record, `pekko-actor/README.md`, `README.md`, ADR 0017 (S6/S11/U2 gap/D-5/U7 delivered), `docs/actor-core-parity.md`, `RESUME.md`.
11. **PR** into `develop`, draft, commits of about 700 changed lines at most (production and Go tests first, judge files next, docs last; Conventional Commits, no AI attribution lines). No merge without the Owner's word.
12. **#449:** after the merge the coordinator checks the five criteria of RULEBOOK 8 against the evidence and closes it **only if completely met and the Owner agrees**; otherwise it lists what is missing. #461 is updated.

## 6. Allowed edit surfaces (9 files)
Go, under `pekko-actor/actor/` (3): `supervision_restart.go` (production), `supervision_restart_goside_internal_test.go` (existing test), `supervision_stoprestart_internal_test.go` (new test).
Judge and docs (6): `docs/judge.md`, `judge/comparator.py`, `judge/selftest.py` (new cases), `judge/scenarios/S22.json` (new), `judge/pekko-harness/Judge.scala`, `migration/stage-1/milestone-1.md`.
Coordinator only, after the run: `pekko-actor/README.md`, `README.md`, `docs/adr/0017-supervision.md`, `docs/actor-core-parity.md`, `migration/stage-1/supervision-u7/EXECUTION-RECORD.md`, `RESUME.md`.
Denied by the rendered permissions: `stop.go`, `cell.go`, `hierarchy.go`, `guardian.go`, other `supervision*.go`, `judge/go-scope.json`, `judge/mutate.py`, `judge/mutate_go.py`, calibration files, `scripts/`, workflows, the RULEBOOK. `go test` and `sbt` are denied to agents.

## 7. What U7 delivers and declares
Delivers the behavior accepted in ADR 0017 H4/S11 and the corrected scope of G4/G5. Declared gaps kept: the Go judge gate for restart scenarios (U8), logging (U9), user-defined strategies (#450), the module-close campaigns (#461); only the default-hooks mode has a Pekko-side scenario (RULEBOOK D2); `G5.f` has no designed Pekko mutation and no campaign (the observed-clauses line is not claimed; the pending Pekko campaign is recorded in #461); S22 runs on Pekko in `concurrent` mode only. No parity is claimed.

## 8. Exact execution commands (after approval and the merge of the package PR)
Run from the root of the worktree on `feat/actor-stop-restart-u7`. `D` is a directory **outside** the repository that exists (create it first); the hash files hold the **approved** values of section 9.
```
D=<directory outside the repository>
printf '%s  RULEBOOK.md\n' ad6de852b61590cb815ed9be1c4b17e36c5cd6f2fe18281e6636f5b60733e61b > "$D/rb.sha"
printf '%s  settings.implementer.template.json\n' 5f7cb873342aa60b993294026895a6de8df9225784ec463efa8d175a78ca7767 > "$D/st-impl.sha"
printf '%s  settings.review.template.json\n' 70d32b49a66fc17bb7619cb5702c4f1e1dbbab77ebbe4e89d39401950696d844 > "$D/st-rev.sha"
AGENT_WALL_MIN=30 bash migration/stage-1/supervision-u7/run-agent.sh implementer sonnet migration/stage-1/supervision-u7/implementer-prompt.md "$D/implementer.json" "$D/rb.sha" "$D/st-impl.sha"
```
Review (step 8, only after steps 3 to 7 and the forecast gate):
```
AGENT_WALL_MIN=12 AGENT_BUDGET_USD=3 bash migration/stage-1/supervision-u7/run-agent.sh review opus migration/stage-1/supervision-u7/review-prompt.md "$D/review.json" "$D/rb.sha" "$D/st-rev.sha"
```
The launcher refuses a RULEBOOK or permission template whose hash differs (exit 3), renders the permissions for this worktree, bounds the wall clock, caps spend per agent (6 USD implementer, 3 USD review), refuses to overwrite an output, keeps a failed output as `<out>.failed.<N>` and, for the review, refuses a second launch (exit 4). The three `printf` lines carry the literal approved hashes of section 9.

## 9. Hashes for approval
| File (in `migration/stage-1/supervision-u7/`) | sha256 |
|---|---|
| `RULEBOOK.md` | `ad6de852b61590cb815ed9be1c4b17e36c5cd6f2fe18281e6636f5b60733e61b` |
| `settings.implementer.template.json` | `5f7cb873342aa60b993294026895a6de8df9225784ec463efa8d175a78ca7767` |
| `settings.review.template.json` | `70d32b49a66fc17bb7619cb5702c4f1e1dbbab77ebbe4e89d39401950696d844` |
| `run-agent.sh` | `a63903e8d4ceb954b5dc47731896b1f67b3d21dea345d29f41aca82d5e52f225` |
| `manifest.tsv` | `54a0b8430dcca4dbf9fe93f0ca3de1ce226167eea80ab3ca3d78df9be86f99a2` |
| `implementer-prompt.md` | `4536972259a9b452cff0114148da6798ad49408c53df0fb55b5fde9242ff5a33` |
| `review-prompt.md` | `9b0f3d95b89683109139d55da7564c310dcda5c57d4a21c564974d035edce9ed` |

The hash of this RUNBOOK cannot be inside it: it is given with the package in `RESUME.md`. The launcher mechanically checks only the RULEBOOK and the template of the role; the other hashes are compared by the coordinator in step 1.

## 10. What approving this package authorizes
Approving the hashes authorizes **the execution of U7 only**, within 560,000 tokens and 45 minutes of agent time, including the S22 run on Pekko, the three mutants and at most one review. It authorizes no merge, no native review, no second review, no closing of #449 and no work on U8, U9, #450 or the module-close campaigns.

## 11. Reconciliation of #461 (done with this package)
Updated by editing the issue: the U6 block is marked delivered with the squash `e0d70bb`, the residue of `guardian.go:135` is marked fixed in this PR, the stale `README.md` and ADR 0017 texts are marked fixed in this PR, and the remaining U6 gaps stay (independent review of AF7, AF16-AF18 and the Example; the full U6 mutation set and the AF6 witness; the observed-order caveat; the Example's hang-only failure mode; the missing "hold" accessor). The U7 gaps are added when U7 is executed.
