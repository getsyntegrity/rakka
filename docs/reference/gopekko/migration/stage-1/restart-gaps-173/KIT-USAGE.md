# Restart gaps (#173): real use of the migration kit

Owner requirement: a matching pin does not replace an execution record, and no evidence may be reconstructed. This file states only what was verified in the repository on 2026-10-16 (branch `feat/go-restart-gaps-173`, draft PR #480). Where a fact could not be verified it says so.

## 1. Pins verified

| Item | Expected | Verified with | Result |
|---|---|---|---|
| `migration-kit/` | `cf91c9d5068d9aaf95a36164169f08c3e636c909` | `git submodule status`, `git rev-parse HEAD:migration-kit` | match |
| `upstream/pekko` | `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1) | `git submodule status`, `git rev-parse HEAD:upstream/pekko` | match |
| `reference/goakt` | n/a | `git submodule status` | not initialised (leading `-`), untouched |

A matching pin only shows which kit text is checked out. It does not show that the kit was used. Section 2 says what was used.

## 2. What was actually done in round 1

Commits of round 1, in order: `7496a99`, `27fac43`, `de2df2c`, `00973f0`, `8dfbc61` (all dated 2026-10-16 between 16:21 and 16:33 -0300; `git log`).

Facts:
- The kit's `CLAUDE.md` was loaded as project instructions (the repository `CLAUDE.md` imports `@migration-kit/CLAUDE.md`). Loading it is not the same as running its process.
- The work was done by direct delegation from the session: one read-only explorer (about 159,786 subagent tokens) and one writer (about 238,433 subagent tokens). These two numbers are the only measured costs. They were reported to the writer of this file by the parent; this file did not re-measure them.
- Parent-session cost: **not measured**.

Negative claims, each checked on 2026-10-16:
- No kit prompt (`migration-kit/prompts/*.md`) was run for this unit. Evidence: `migration/stage-1/restart-gaps-173/` held only `EXECUTION-RECORD.md` before this round of documentation (no prompt outputs, no `RULEBOOK.md`); `git log --all -- migration/stage-1/restart-gaps-173` shows only the record commit `8dfbc61` before this documentation. Absence of a trace is the evidence; there is no log of a prompt run.
- No queue runner or manifest was used. `migration-kit/scripts/queue_runner.mjs` and `make_manifest.py` produce or consume a manifest; none existed for this unit before this documentation.
- No `RULEBOOK.md` or `RUNBOOK.md` was written before the work. Earlier units (for example `supervision-u7`, `supervision-u8`, `shutdown-u10`) have `RULEBOOK.md`, `RUNBOOK.md`, `manifest.tsv`, `implementer-prompt.md`, `review-prompt.md`, two settings templates, `run-agent.sh`, `RESUME.md` and `EXECUTION-RECORD.md`. This unit has none of them except the execution record.
- No `run-agent.sh` launcher, hash-checked permission template, USD cap or `review.lock` applied. The delegation used the session's ordinary subagent mechanism.
- No independent adversarial reviewer ran, let alone two. `EXECUTION-RECORD.md` (section Review) says the review was not run. The native review preflight returned `unassessable`.
- The final candidate is unreviewed.

What round 1 did produce, by its own record (`EXECUTION-RECORD.md`): ports of Ticket669Spec:37 and the unwatch half of UidClashTest:104, killers for K27, K28 and K33 with anchored mutation cells, ADR 0017 text, and a Go campaign (16 of 17 designed mutations detected, GMoverlap `harness-error`, which is not a detection).

Also present in the unit directory, written after round 1: `S22-GO-PROPOSAL.md`, a proposal that awaits the owner's decision. It is not an executed artifact.

## 3. Deviation entry

Format of the kit's `templates/RULEBOOK.md` section 7 (ID, date, deviation, sanctioned by) and of earlier units (DV1 to DV3 in `supervision-u3/RULEBOOK.md`). Earlier RULEBOOKs are not edited.

| ID | Date | Deviation | Sanctioned by |
|---|---|---|---|
| RG1 | 2026-10-16 | Round 1 ran without a unit RULEBOOK, RUNBOOK, manifest, implementer prompt, review prompt, permission templates or launcher. Kit standing rules 1 (rulebook read-only inside loops, none existed), 2 (queues on disk, none existed) and 5 (banned operations by configuration, none configured) were not applied by mechanism. | not sanctioned; recorded here by the writer of this file, pending the owner |
| RG2 | 2026-10-16 | No reviewer of any kind ran in round 1 (kit standing rule 4 asks for two separate adversarial reviewers; earlier units ran one under DV1, the owner decision of 2026-10-15, which does not cover zero). | not sanctioned |
| RG3 | 2026-10-16 | Model plan: the kit's CLAUDE.md model guidance says a plan chosen at the feasibility gate binds subagent tiers. The models of the two round-1 subagents are not recorded in a verifiable source, so conformance cannot be checked. | not sanctioned |

Why it matters:
- The pin and the loaded `CLAUDE.md` could be read as "the kit was applied". They only show the kit was available. The changes of round 1 are therefore verified by tests and mutants run by the session, not by the kit's separation of implementer and reviewers.
- Without a rulebook and manifest, scope, allowed edit surface and budget were not fixed before the work, and the cost record is partial.
- The tests and mutants are real evidence of behavior; the absence of an independent review is the open gap.

## 4. Remaining work and the kit process

The kit has no prompt for the final review or for a campaign; those follow the project's own per-unit procedure (`shutdown-u10/RUNBOOK.md` is the latest complete example; the request named `supervision-u10`, which does not exist, so `shutdown-u10` was used). The kit prompts that exist are `00-feasibility`, `00b-judge-setup`, `01-dependency-map`, `02-gap-inventory`, `03-stress-test`, `04-translation-kickoff`, `05-survey-build`, `06-post-parity`.

| Remaining work | Governing process | Artifacts required before it starts | Present now |
|---|---|---|---|
| S22 in Go (the `default-restart` event and a directed mutant). Awaits owner approval of the judge contract change (#175). | Step 3 implementation, prompt `04-translation-kickoff.md`, with `00b-judge-setup.md` for the judge side; executed in the per-unit form of `supervision-u8` | `RULEBOOK.md`, `implementer-prompt.md`, `review-prompt.md`, settings templates, `run-agent.sh`, `manifest.tsv`, owner-approved hashes | No. **Deliberately not prepared**: nothing is written until the owner approves. Only `S22-GO-PROPOSAL.md` exists. |
| Final candidate review | Kit standing rule 4 (adversarial, separate, read-only) as reduced by DV1 to one Opus review; budget and lock rules of `shutdown-u10/RUNBOOK.md` section 4 | `RUNBOOK.md` with the gates, `review-prompt.md`, review settings template, `run-agent.sh` | `RUNBOOK.md` prepared in this directory (not executed). Prompt, template and launcher are **not** prepared. |
| Full Go campaign | `judge/mutate_go.py` and `judge/run-go.sh`; evidence tagging as in `EXECUTION-RECORD.md` Evidence status | clean committed tree, owner authorization to publish a report and tag | `RUNBOOK.md` section 4 prepared (not executed). Existing 30-run campaign at `00973f0` is not a clean 17/17. |

Prepared, not executed: `RUNBOOK.md` and `manifest.tsv` in this directory. `manifest.tsv` has two rows (final review and campaign, status `pending`). Earlier units use two columns (source, target) with no status; the third `status` column here is a deviation from that format requested for this unit, and the S22 row is intentionally absent. Rows are execution gates (record, report), not source-to-target translations, so they are not the output of `make_manifest.py`. No agent, review, campaign or command listed in them has been run. They carry no approved hashes; hashes are computed only when the owner approves a package.

How it will be followed: the review and campaign only start after the RUNBOOK gates are confirmed in writing (observed tokens, reserve, one review, `review.lock`), and each result is written in an execution record with the command and its observed output. A task that cannot be confirmed is recorded as not run.

## 5. Cost record

`migration/cost-log.tsv` columns are `step, timestamp, wall_clock_min, tokens, subagents, model`. It can hold the two observed runs if unknown fields are written as `unknown`, as earlier rows already do. Two lines were appended:

| step | tokens | subagents | unknown fields |
|---|---|---|---|
| `restart-gaps-173-r1-explorer` | 159786 | 1 | timestamp, wall_clock_min, model |
| `restart-gaps-173-r1-writer` | 238433 | 1 | timestamp, wall_clock_min, model |

Token figures are the values reported by the runtime for each subagent (about 159,786 and 238,433); they are the figures the runtime reported for each subagent, not re-measured. The parent-session cost is not measured and has no row. Total of the two subagent runs: about 398.2k, which is an addition of reported numbers, not a measured total.
