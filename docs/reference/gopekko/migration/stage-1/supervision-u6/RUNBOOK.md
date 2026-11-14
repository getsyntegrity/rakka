# RUNBOOK: Supervision U6, AllForOne (#447)

Status: PROPOSED for the Owner's approval of the package. **Execution is NOT authorized by this file**: it starts only when the Owner approves the hashes of section 9 and says to run. Unit U6 of ADR 0017 (S3, H2), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`). Design and contract: `RULEBOOK.md` (this directory). Base: `develop` at `66ee197c8fad85ccbf8540dafdb9db6120503734` (#468, verified with `git rev-parse origin/develop` on 2026-10-15). Delivery policy of 2026-10-15 and the economical CI policy (`docs/ci-economy.md`) apply: CI runs only the fast check; race, mutants, review and the rest run locally. Procedures not mentioned here are those of `../supervision-u5/u5c/RUNBOOK.md`.

## 0. Revisions 2 and 3 (corrections before authorization)
Revision 2 corrected four findings against the pinned sources. The budget, surfaces, launcher, permission templates and manifest are unchanged. (1) D1/D2: the children set is captured **before** the decider, and a child created by the decider is outside the round and survives it (RULEBOOK 1, 4, AF16). (2) R8: `FailedMessage` is absent for healthy siblings and kept by a sibling with a pending failure (AF4, AF14b). (3) R1/R3/R3a and AF12/AF17: AllForOne takes no action on escalation or decider failure, and the suspension that propagates from the parent's own failure is asserted separately instead of demanding untouched siblings. (4) D3: ascending name is the observed `TreeMap` order of v1.7.1, documented as not a public guarantee (RULEBOOK 5). **Revision 3** corrects the member-that-ends rule: the first revision skipped such a member, which could turn a Pekko denial into a global restart; the snapshot now keeps each member's captured counter and the member still counts (RULEBOOK 4, AF15, D1). The mapping report that called the order unspecified was wrong, and the first draft of RULEBOOK 5 repeated it; both are superseded.

## 1. Scope
Exactly RULEBOOK sections 2 to 6 and 9: `AllForOne` (one exported function), the rules R1 to R11, tests AF1 to AF18 and AFX, the three mutants of 9.3 run in this delivery, and the documentation of section 10. **Not in scope:** U7 (the `PostStop` guard, H4/S11, `docs/judge.md`), U8 (S13/S14/S16), U9 (logging, `kit-logger`), #450 (user strategies, decider context, `Kill`), persistence, remote, cluster, the guardians' strategy, the module-close campaigns of #461. **No native review is requested for this package or for the execution** (decision already taken by the Owner).

## 2. Preparation findings (by reading; no runtime probe was run)
- The failure path has one decision point: `handleFailed` (`supervision_failure.go:248`) calls `apply` or `applyRestart`. `applyRestart` (`supervision_restart.go:176-186`) is the only place that asks a budget. U6 changes `applyRestart` and the `Stop` branch of `apply`; `Resume` and `Escalate` need no change.
- The mechanisms U6 needs exist: per-child counters keyed by incarnation ID (`statsFor`, `supervision_budget.go`), the counted suspension of one child (`suspendable`, `supervision_failure.go:33`), `recreate(cause)`, `noteStopID` + `acceptStop(nil)`, and the name-ordered snapshot `survivors` (`hierarchy.go:289`). **No new seam and no edit of `cell.go` or `hierarchy.go` is expected**; both carry judge anchors and are denied.
- The only placeholders in Go code are `supervision_restart_internal_test.go:29` and `:395-401` (W4, `PORT-GAP(AllForOne-U6)`). `doc.go:15`, `pekko-actor/README.md:247,330-332,355`, `README.md:111-112`, `docs/actor-core-parity.md:67,175` and ADR 0017 (`:656,685,710,718`) say "planned"; the coordinator updates them in step 8.
- Judge anchors: none of the three production files, and no supervision file, is named by `judge/`; `hierarchy.go` and `cell.go` are untouched.
- Premises carried to the run, all verified by reading but not by running: that `survivors` includes children whose stop was requested (RULEBOOK 4), that `suspend()`/`recreate`/`acceptStop` on a finished or terminating member leave no suspension, hold or counter behind and do not block a later restart of P, and the intermediate suspension counts of AF14b (RULEBOOK 9.2). The prompt asks the implementer to report a contradiction as `NEEDS-DECISION` or `BLOCKED(surface)`; neither is expected to need the Owner unless it fires.

## 3. Items carried from U5 (reconciled in #461, not in this package's code)
T7, the V14 escalating-case count and the `bootPoint` doc comment are resolved in `develop` (section 11). U6 adds no work for them.

## 4. Flow, limits and how coordination is counted
| Role | Model | Count |
|---|---|---|
| Implementer, production and tests in **one session**, `run-agent.sh implementer` | `sonnet` | 1 |
| **One** focused review, `run-agent.sh review` (read-only permissions, one-review lock, USD cap) | `opus` | 1 |
| Consultation on one concrete unresolved decision | `opus` | 0-1 |
| Mutants, gates, corrections of confirmed blockers, docs | the **coordinator** | 0 agents |

At most 3 agents. **Time: 45 minutes of active work**; `AGENT_WALL_MIN` implementer 30, review 12.

**What actually limits the review.** U5b's review used 127,642 tokens against a "hard limit" of 100,000 because that limit was an instruction to an Agent-tool subagent and nothing enforced it. Here the review is **not** an Agent-tool subagent; it runs through the same launcher as the implementer and three things bind it:
1. **Mechanical, enforced by the CLI and the script:** `--max-budget-usd` (default 3 for the review role) and the wall clock (12 minutes, `timeout -k 30s`). When either is hit the process ends and the output is kept as `<out>.failed.<N>`. Read-only permissions (no Edit/Write) are enforced by the rendered settings.
2. **Mechanical, one review:** the launcher creates `review.lock` next to the output before launching and never removes it; a second review launch exits 4. Only deleting the file by hand, which is the Owner's decision, allows another.
3. **By instruction only (not enforced):** the final report of at most 1,500 tokens and the read order of the prompt.
USD is a **proxy** for tokens, not a token limit. I have no measurement of Opus's tokens per dollar for this prompt, so the 100,000-token figure below is a **target checked after the fact** from the JSON, not a guarantee; the guarantee is the dollar cap and the clock. If the USD cap truncates the review, its partial output is read from `.failed.1`, and a second review is **not** launched without the Owner.

**This delivery:** maximum **560,000** tokens, evaluation stop **450,000**. No aggregate is authorized. Coordination is a convention, not a measurement: reserve **90,000**, plus **30,000** for the three mutants, plus **30,000** per correction round or repeated full gate cycle; `used = agents measured (input + output + cache creation) + reserve`, stated as an estimate.

| Item | Last observed (U5b / U5c) | U6 expected |
|---|---|---|
| Implementer | 298,504 / 99,350 | 150,000 to 280,000 (about 100 production lines, one public function with a long doc, about 18 scenario tests and an Example) |
| Opus review | 127,642 / 104,415 | 60,000 to 100,000 (target; USD cap 3) |
| **Agents** | 426,146 / 203,765 | **210,000 to 380,000** |
| Reserve (coordination 90,000 + mutants 30,000) | 90,000 | 120,000 |
| **Total used (estimate)** | 546,146 / 293,765 | **330,000 to 500,000**, central about 410,000 |

The upper tail passes the 450,000 stop, so the gates matter. **Forecast gate** before each agent and before each correction: `observed + remaining reserve + forecast`; if it passes 450,000, stop and ask. Concretely, **the review is launched only if the implementer used at most 230,000**; above that the coordinator stops and asks. At 450,000 it stops and asks before any correction round or extra agent. The 560,000 maximum is the absolute ceiling that also absorbs an in-flight agent that overshoots; it is never a target. A run killed by the wall bound counts its whole forecast as spent. Nothing is consumed until the Owner approves and says to run.

## 5. Procedure after the Owner approves
0. **Merge the package PR** by the Owner's word; create `feat/actor-allforone-u6` from the resulting `develop` (CodeGraph-dependent worktrees stay under the user's home, never `/tmp`).
1. **Baseline:** clean branch; `git rev-parse HEAD:pekko-actor/actor` and `shasum -a 256` of every `.go` file of `pekko-actor/actor/`, written outside the repository (create that directory first); submodules at `cf91c9d` (kit) and `9a54d1294c` (upstream); the approved hashes of section 9 written to files outside the repository and compared, **including the RUNBOOK's own hash from `RESUME.md`**. Each command a separate call.
2. **Implementer** (`sonnet`, `AGENT_WALL_MIN=30`), command of section 8.
3. **Gates by the coordinator:** `gofmt -l`, `go vet`, `go mod tidy -diff`, the pinned `golangci-lint` (`gocyclo` at most 10), coverage at least 85% (`scripts/check_coverage.sh`), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/check-adr-signatures.sh`, `scripts/modules_selftest.sh`, `.github/scripts/go-sdk-update.sh --check`, `go test -count=1 ./...` of the module, the judge anchors (`judge/selftest_mutate_go.py`; `hierarchy.go` and `cell.go` byte-identical to the baseline). An anchor that must change stops the work; `judge/` is never edited.
4. **One `go test -race -count=1 ./...`**, and the new AF tests plus the Example repeated 30 times under `-race`.
5. **Mutants MU1, MU2, MU3** (RULEBOOK 9.3), by the coordinator, one at a time, each on a fresh copy of the working tree outside the repository; record per mutant: the patch, the tests that failed, and that the unmodified copy passed. INVALID mutants are redesigned once. A surviving mutant: one correction round of the tests (coordinator), re-run; still surviving: ask the Owner.
6. **The one review** (`opus`, command of section 8) only if the forecast gate allows. Findings cite a RULEBOOK rule or a source line; BLOCKER / DEFECT / INFO; INFO is recorded, not acted on.
7. **Scope control after the fact:** `git diff --stat` and `shasum` against the baseline; any `.go` file outside the surfaces of section 6 is a defect.
8. **Documentation by the coordinator,** once: the execution record (with the kit evidence for #447 criterion 1, the mutant table, the measured tokens), `pekko-actor/README.md`, `README.md`, `actor/doc.go`, ADR 0017 (S3/U6 delivered, gaps kept), `docs/actor-core-parity.md`, `RESUME.md`.
9. **PR** into `develop`, draft, commits of about 700 changed lines at most (production and its core tests first, the remaining tests next, docs last; Conventional Commits, no AI attribution lines in commit messages). No merge without the Owner's word.
10. **#447:** after the merge the coordinator checks the three acceptance criteria against the evidence and closes it **only if completely met and the Owner agrees**; otherwise it lists what is missing. #461 is updated (section 11).

## 6. Allowed edit surfaces (6 files, all under `pekko-actor/actor/`)
Production (3): `supervision.go`, `supervision_failure.go`, `supervision_restart.go`.
New tests (2): `supervision_allforone_internal_test.go`, `supervision_allforone_example_test.go`.
Existing tests (1): `supervision_restart_internal_test.go` (AF7: the header line near 29 and the W4 test near 395-401 only).
Coordinator only, after the run (comments and docs, not the implementer): `pekko-actor/actor/doc.go` (comment), `pekko-actor/README.md`, `README.md`, `docs/adr/0017-supervision.md`, `docs/actor-core-parity.md`, `migration/stage-1/supervision-u6/EXECUTION-RECORD.md`, `RESUME.md`.
Denied by the rendered permissions: `cell.go`, `hierarchy.go`, `stop.go`, `guardian.go`, `names.go`, `system.go`, other `supervision*.go`, `judge/`, workflows, the RULEBOOK.

## 7. What U6 delivers and declares
Delivers `AllForOne` as in RULEBOOK 2 to 6. Declared gaps kept: the stop during a restart (U7), S13/S14/S16 (U8), supervision logging (U9), user-defined strategies and the decider context (#450), `System.Shutdown` and escalation that kills, the Java-API overloads and strategy equality, the observed-not-guaranteed order (RULEBOOK 5), `Ticket669Spec:49`. The U3, U4, U5 and full U6 mutation campaigns, the judge campaign, repeated `-race` and the Pekko surface inventory stay at #461. No parity is claimed.

## 8. Exact execution commands (after approval and the merge of the package PR)
Run from the root of the worktree on `feat/actor-allforone-u6`. `D` is a directory **outside** the repository that exists (create it first); the hash files hold the **approved** values of section 9.
```
D=<directory outside the repository>
printf '%s  RULEBOOK.md\n' 5df5a197faaad448905e7eeb29398888f114beaa2fb78911c1e844caabd5d8b0 > "$D/rb.sha"
printf '%s  settings.implementer.template.json\n' 8938cae3bff7579ff430c67818ee9b0e836d49376c90899165e743d848fb2ab8 > "$D/st-impl.sha"
printf '%s  settings.review.template.json\n' 52fc92140b14da946429011eef62d4eaaa0b93acd3c4ad63aff2a7274ea6a6be > "$D/st-rev.sha"
AGENT_WALL_MIN=30 bash migration/stage-1/supervision-u6/run-agent.sh implementer sonnet migration/stage-1/supervision-u6/implementer-prompt.md "$D/implementer.json" "$D/rb.sha" "$D/st-impl.sha"
```
Review (step 6, only after steps 3 to 5 and the forecast gate):
```
AGENT_WALL_MIN=12 AGENT_BUDGET_USD=3 bash migration/stage-1/supervision-u6/run-agent.sh review opus migration/stage-1/supervision-u6/review-prompt.md "$D/review.json" "$D/rb.sha" "$D/st-rev.sha"
```
The launcher refuses a RULEBOOK or permission template whose hash differs (exit 3, verified), renders the permissions for this worktree, bounds the wall clock, caps spend per agent (6 USD implementer, 3 USD review), refuses to overwrite an output, keeps a failed output as `<out>.failed.<N>` and, for the review, refuses a second launch (exit 4).

## 9. Hashes for approval
| File (in `migration/stage-1/supervision-u6/`) | sha256 |
|---|---|
| `RULEBOOK.md` | `5df5a197faaad448905e7eeb29398888f114beaa2fb78911c1e844caabd5d8b0` |
| `settings.implementer.template.json` | `8938cae3bff7579ff430c67818ee9b0e836d49376c90899165e743d848fb2ab8` |
| `settings.review.template.json` | `52fc92140b14da946429011eef62d4eaaa0b93acd3c4ad63aff2a7274ea6a6be` |
| `run-agent.sh` | `1d9f9af87c42d45d3a403b8c349d0d53adbfd9b82fc90a35aad4865be93b4b82` |
| `manifest.tsv` | `73de7d994f87063aa6ff6edee282cae8e574655f355723cda6b6d2253a0f1783` |
| `implementer-prompt.md` | `5b3446185e198bb600d6fae471e17cd04f770215242fb4932c22e7e001ab41db` |
| `review-prompt.md` | `4a395ae58f56ee067b9fcc24a6d81c72e4f0cbcda76387bb68abb1283f2f7685` |

The hash of this RUNBOOK cannot be inside it: it is given with the package in `RESUME.md`. The launcher mechanically checks only the RULEBOOK and the template of the role; the other hashes are compared by the coordinator in step 1.

## 10. What approving this package authorizes
Approving the hashes authorizes **the execution of U6 only**, within 560,000 tokens and 45 minutes, including the three mutants and at most one review. It authorizes no merge, no native review, no second review, no closing of #447 and no work on U7, U8, U9 or the module-close campaigns.

## 11. Reconciliation of #461 (done with this package)
Resolved in `develop` at `66ee197`, with evidence, and moved out of the pending list:
| Item | Evidence |
|---|---|
| T7 (also rule out `Ref[guardianMsg]`) | `pekko-actor/actor/guardian_goside_internal_test.go:189-190`, delivered in #467 (`d5e936b`) |
| V14 escalating case asserted only the first event | `pekko-actor/actor/supervision_goside_internal_test.go:451-452` now asserts `receive-failed` once and `stop-accepted` zero times (#467) |
| `bootPoint` doc mismatch (`R3-bootpoint-doc-mismatch`) | `pekko-actor/actor/system.go:47-52` now documents steps 1 to 7 |

Kept, with narrower wording: **INFO-6** (`receive-failed` is the *first* event in the V14 case; the counts do not pin the order) stays open. **New, found while verifying:** `pekko-actor/actor/guardian.go:135` still says the `options.bootPoint` doc "predates this layout", which is no longer true (comment only). All other items stay as listed. The `AllForOne` line points to #447 and the U6 package PR.
