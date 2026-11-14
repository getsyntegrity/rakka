# Restart gaps (#173, part of #461): execution record

Branch `feat/go-restart-gaps-173` from `origin/develop` 0dd0d31. Pekko pin `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1); kit pin `cf91c9d`. `reference/goakt` not initialised, untouched.

## Finding

U7 already implements stop-during-restart and the default restart hooks (`TestSupStopDuringRestart` T1-T3, W25, W11b). Round 1 added evidence only (`git diff origin/develop` then touched only `*_test.go`, `judge/`, `docs/`, `migration/`). **That sentence was true of round 1 and is not true of the candidate:** unit S22 later added one production statement, the diagnostics event `prerestart-default` as the first line of `defaultPreRestart` (`supervision_restart.go`, see Update 5). The rest of the record is amended where it changed (Updates 5-10).

## Traceability

| Item | Pekko source | Go test or cell | Class | Result |
|---|---|---|---|---|
| Ticket669Spec:37 | `Ticket669Spec.scala:37` | `supervision_ticket669_internal_test.go` | missing validation | ported, passes |
| UidClashTest:104 (unwatch in `preRestart`) | `UidClashTest.scala:104` | same file, with a control without `Unwatch` | missing validation | ported, passes; UID half stays `PORT-GAP(uid-clash)` |
| Ticket669Spec:49 | `Ticket669Spec.scala:49-59` | none | language gap | `PORT-GAP(ticket669-postStop)`: a Go `PostStop` has no message |
| K28 | U2 RULEBOOK 5.3 | W16 asserts `n.stats` unchanged after a stale `Failed` | missing assertion | killer added |
| K33 | U2 RULEBOOK 5.3 | `TestSup2DecisionLeavesCounter` | missing assertion | killer added |
| K27 | U2 RULEBOOK 5.3 | `TestSup2ChildFailureDuringInstallNew` | missing interleaving | killer added. `declares()` no longer exists; the mutant is an unlocked `c.actor` read in `enqueueFailed` |
| n3 | native review target 3 (`fa8b54a..0ed334e`) | n/a | review disposition | not a mutant; still `correction_required`, owner decision pending |
| S22 in Go | `Judge.scala:471-479` | none | adaptation needing a decision | round 1: pending. Approved by the Owner afterwards and implemented as unit S22 (Update 5); in the Go gate with GMdefaultps; its cells under GM2 and GMstop are excluded as not applicable (Update 7-8, `EXCLUSIONS.md`) |

K27, K28 and K33 had no script definition; they are now anchored cells in `judge/mutate_go.py` (`UNIT_MUTATIONS`, run only when named).

## Validation actually run (darwin/arm64, go1.27.2)

- `go test -C pekko-actor -race -count=1 ./...`: ok. New tests `-count=20 -race`: ok.
- `python3 -I judge/selftest_mutate_go.py` ok; `judge/selftest.py` 190 checks, 0 failed.
- `go run .github/scripts/ci-fast.go`: ok. Coverage: actor 98.0%, deadletter 100%, mailbox 100%.
- `check-adr-signatures.sh` ok; `check-judge-matrix.py` exit 0.
- Unit mutants (workdir outside the repo): K27 detected 20/20 under `-race`, K28 3/3, K33 3/3; survived 0, INVALID 0 each.
- NOT run: `golangci-lint` (installed 2.13.1, pinned 2.14.0; typecheck fails on the stdlib `testing` export data) and `gocyclo` (not installed). Both remain to run on a machine with the pinned versions.

## Evidence status

- Full Go campaign at `00973f0`, 30 runs, tree clean: 16 of 17 designed mutations detected; benign survived; **GMoverlap `harness-error` (INVALID, not a detection)**. This is the flake documented in `supervision-u8/EXECUTION-RECORD.md` lines 27-29 (data races in `cell.go`, present before U8).
- GMoverlap re-run alone (30 runs): met. This is a separate run, not a replacement of the campaign above.
- CURRENT: no clean single-campaign 17/17 exists for this tree, and `judge/go-mutation-report.tsv` was not replaced. `check-judge-matrix.py` therefore still reports CURRENT 0/17 ("an input of the campaign changed").
- HISTORICAL: tag `evidence/go-campaign-50fe7c2` (U9), 17/17 darwin/arm64, not Linux evidence.
- Raw reports are kept outside the repo in the job directory; publishing them (`judge/go-mutation-report.tsv` plus an `evidence/go-campaign-<sha>` tag) needs the owner's authorization. Prepared step: copy a clean 17/17 report to `judge/go-mutation-report.tsv` on a clean tree, run `python3 -I scripts/check-judge-matrix.py`, then tag.

## Review

Not run. The native review preflight (`gentle-ai review assess`) returned `unassessable`, treated as high; its START needs per-candidate human consent. The repository's one Opus review was not launched because the budget check (observed tokens plus reserve plus 110k against the 560k cap) was not confirmed. The final candidate is therefore **unreviewed**.

## Pending decisions

1. S22 in Go. A judge-visible event for the default restart path changes the harness contract (#175: a scenario enters the gate only with directed mutants that detect it). Proposal: add a `default-restart` event to the Go harness plus a directed mutant on `defaultPreRestart`/`postStopOfRestart`, so the pre-U7 double `PostStop` is detected.
2. n3 `correction_required` has no native way to close a refuted finding.

## Still pending in #173 and #461

Everything listed as pending in #461 other than the rows closed above: #459, #460, INFO-2..6 of U5c, AF7 and AF16-18, the parity campaign tag, G5.f Pekko mutation, calibration of S13/S14/S16, Linux evidence, complete `-race` and stress runs, the integral review, and the Pekko surface inventory. Out of scope and inventoried: restart-hook logs, original cause, Recreate to terminated actors.

## Update 2 (after the owner's second brief): campaign 2 and the GMoverlap investigation

All runs below are darwin/arm64, go1.27.2, 30 runs per mutation, and are kept as separate results. None is combined with another.

| Run | Tree | Result |
|---|---|---|
| Campaign 1 | `00973f0` | 16 of 17 designed detected; benign survived; **GMoverlap `harness-error` (INVALID)** |
| GMoverlap alone (retry) | `00973f0` | met (S21 `G3.c`); a separate result, not part of campaign 1 |
| GMoverlap repeated 20 times (investigation, mutated copy, `go test -race` of `TestJudgeScenarios`) | `00973f0` | 4 of 20 exited non-zero with data-race reports (runs 3, 4, 8, 10); 16 of 20 clean |
| **Campaign 2** | `8a2f4e5` (`pekko-actor/` and `judge/` identical to the candidate; later commits are documentation only) | 16 of 17 designed detected; benign survived; **GMoverlap `harness-error` (INVALID) again** |

Result: no campaign has produced a valid 17 of 17 on this unit. `judge/go-mutation-report.tsv` is not replaced and the judge matrix keeps reporting CURRENT 0/17. HISTORICAL stays `evidence/go-campaign-50fe7c2`.

Investigation (read from the logs, no code changed). The races reported in both campaigns and in the repetitions are always the same sites: a write at `cell.go:232` (`c.actor = a`, under `c.mu`) against the read at `cell.go:270` (`c.actor.Receive`, no lock), and a write/write on `failedMsg` at `cell.go:294` by two goroutines inside `callReceive`/`receiveFailed`. GMoverlap makes a second worker take messages of the same actor, so two `Receive` calls of one actor run at the same time. The unmutated runtime never does that (one drain per actor), and these fields are touched without a lock because of that serial guarantee. So the race is exposed by the mutant, in production lines the mutant does not edit. The designed detection of GMoverlap is the event-log clause `G3.c`, and the policy of `judge/mutate_go.py` and `judge/calibrate.py` (`go_outcome`) is that a non-zero `go test` exit is `harness-error`, never a detection. The same flake is documented in `supervision-u8/EXECUTION-RECORD.md` (1 in 3 campaigns, 1 in 4 branch reruns, 1 in 6 on base `d5b831b`); here it is 2 of 2 campaigns and 4 of 20 repetitions.

Not done, on purpose: no retry until clean, no change of the classification, no change in `cell.go`. The campaign is not repeated until the owner decides.

Owner decision requested (one):
- A. Keep the policy. GMoverlap stays INVALID whenever the race fires; a valid 17 of 17 needs a run in which it does not (about 80% of the repetitions). Cost: evidence depends on luck, and the report cannot be published honestly until one run is clean.
- B. Make the mutant deterministic by changing the mutation so the overlap is serialised for the race detector (a change of `judge/mutate_go.py`, a judge contract change; the detection stays `G3.c`).
- C. Treat a race whose both sites are inside the overlap that the mutant creates as a documented, distinct outcome. This changes the classification policy.
Recommendation: B, because it keeps `harness-error` meaning what it means and removes the dependence on luck.

## n3: pending provider resolution (not closed)

State: native review target 3 (`fa8b54a..0ed334e`, 706 lines) is `correction_required`. It is not approved and this record does not close it. Closing it needs a resolution by the review provider (`gentle-ai`); no annotation here changes its state.

Technical evidence (from `supervision-u2/EXECUTION-RECORD.md` lines 96-132, re-read, not re-run):
- The CRITICAL finding R3-fatal-never-cleared is contradicted by Pekko: `clearFailed` clears only `FailedRef`, and `FailedFatally` is sticky (`FaultHandling.scala:83-86`). Go: `cell.go:63` and `TestSup2FailedFatally` (`supervision_restart_internal_test.go:561`).
- R3-failedmsg-stale mirrors `FaultHandling.scala:306`; R3-restart-untested is obsolete.
- The probe of `gentle-ai review capabilities` found no way to close a `correction_required` that source evidence refutes.

## Update 3: Owner decisions of the third brief and the prepared package (nothing executed)

Decisions recorded: (1) GMoverlap: option B, a deterministic mutation that keeps the defect (overlapping activations detected by the judge's own guarantee `G3.c`), no fabricated detection, no serialization, no weaker comparator; timeouts, crashes and `harness-error` stay INVALID; earlier campaigns stay complete and separate; no search for a clean run by repetition. (2) S22 in Go: sections 3-5 of `S22-GO-PROPOSAL.md` approved with the conditions written at the top of that file. (3) Budget: both rounds count, 661,896 reported tokens plus unmeasured coordination; the excess over the 560,000 cap is recorded (101,896, 18.2 %) and the accounting is not reset; the decision does not raise the ceiling or authorize agents. (4) `odd/tasks/restart-gaps-173.md` stays outside the candidate and is declared excluded in the preflight. (5) Review: one, on the final candidate after B and S22; if native, the consent is the Owner's. (6) Campaign: after the candidate is stable; target 18 of 18; no combination of runs; no report with INVALID published as valid; report and tag prepared with the exact SHA. RG1-RG3 stand; this approval does not turn earlier work into a kit execution.

Prepared in this directory (inline by the coordinator; no agent was launched; the coordinator's own cost is not measured): `BUDGET.md`, `RULEBOOK.md`, `RUNBOOK.md` (replaced), `implementer-prompt.md`, `review-prompt.md`, `manifest.tsv`, `PACKAGE-HASHES.tsv`. Status: waiting for the Owner to approve the package hashes and the incremental budget.

Reading of the earlier GMoverlap races, from the code: the extra worker can read `c.actor` before the factory finished because the mutant checks `c.started`, which is set before `c.actor` is installed (`cell.go:100-105` against `:232`); that is site 1 and is a defect of the mutant text, not of the runtime. Site 2 (two overlapped `Receive` calls that both fail, both writing `failedMsg`) may be inherent to the defect; RULEBOOK B3 defines the stop in that case.

## Update 4: unit B executed (kit package of `361d871` + Addendum 1)

Inputs: package verified before execution (7 hashes matched at `361d871`); Addendum 1 `9680a2f` (tree `8c99e205b963721cc563f774330ecb6ab724bc4b`); baseline sha256 of the byte-identical files recorded before the run (123 files); one Sonnet implementer, prompt = `implementer-prompt.md` unit B, surfaces `judge/mutate_go.py` and `judge/selftest_mutate_go.py`.

Result: accepted, commit `3b604c1`. One token changed in the GMoverlap text (`!c.started` to `c.actor == nil`, evaluated under `c.mu`), so the extra worker proceeds only once the actor is installed; the two `Receive` calls still overlap with no lock or channel between them. Selftest cases 11 added (RED then GREEN reported by the implementer).

Verification, by the implementer and re-checked by the coordinator:
- `python3 -I judge/selftest_mutate_go.py`: ok (coordinator rerun). `python3 -I judge/selftest.py`: 190 checks, 0 failed (implementer).
- B5 (this is the design check of RULEBOOK B5, not a search for a clean run): 20 sequential `go test -race` runs of the mutated copy: exit 0 in 20 of 20, zero `DATA RACE` lines in the 20 logs (coordinator reread `summary.txt` and the logs). Comparator re-run by the coordinator on the 20 kept outputs: S21 `concurrent`, 30 runs, 0 pass, 30 fail, rule `G3.c`, in all 20. Logs: `/Users/pablogore/.claude/jobs/b989ecff/tmp/B-run/b5/` (outside the repository).
- A separate run `mutate_go.py ... GMoverlap --runs 30` (no report; the script refuses a report from a dirty tree): `GMoverlap met`.
- `git diff --stat` of the commit: the two allowed files only (36 insertions, 1 deletion). sha256 check against the baseline: no mismatch (coordinator rerun).
- These runs are kept apart from campaign 1, its retry, the 20 repetitions of the earlier design, and campaign 2. No result is combined with another. The earlier campaigns are not reclassified.
- The earlier design's races are explained by the mutant text (the `c.started` check), which supports the reading in Update 3. Site 2 (two failing overlapped `Receive` calls) was not exercised as a failure by S21, so nothing is claimed about it.

Incidental: `python3 judge/mutate_go.py --help` treats `--help` as a workdir and creates an untracked `--help/` directory in the current directory. It happened twice (once by the coordinator, once by the implementer) and was deleted each time. Not fixed here (outside the approved surfaces); queued as an observation.

### Incremental budget accounting (measured, not forecast)
| Item | Cap | Measured |
|---|---|---|
| Unit B implementer | 120,000 | **131,865** (11,865 over its cap; no launcher enforces the cap, so this is recorded as measured) |
| Incremental total so far | 520,000 ceiling, stop at 416,000 | 131,865 (subagents only; coordination not measured) |
| Cumulative, subagents only | | 661,896 + 131,865 = **793,761** |

Remaining to the ceiling: 388,135; to the evaluation stop: 284,135. If S22 reached its full 200,000 cap the total would be 331,865, and the review gate (`observed + remaining reserve + 110,000 <= 520,000`, with the reserves of `BUDGET.md`) would then be 531,865 and would fail; it is evaluated with real numbers after S22. The cap was exceeded by an unlaunched agent, not by decision; a hard cap needs a launcher, which the package does not have.

## Update 5: unit S22 executed

Inputs: unit B accepted at `3b604c1`; baseline sha256 of 32 byte-identical files recorded at `e3c2e1c` (tree `2ee7774ea4ce3e5d7bb6398c75e513fdaf0cdd3c`); one Sonnet implementer; prompt = `implementer-prompt.md` unit S22; 12 allowed surfaces; Pekko origin `Actor.scala:625-631` and `FaultHandling.scala` (terminate, finishTerminate).

Result: committed as `374f258`. Production change: one statement, `c.sys.emit("prerestart-default", c.cid, nil)` as the first line of `defaultPreRestart` (coordinator read the diff). The harness observes through a sink adapter and maps the event to `pre-restart` only for default-hook nodes; `post-stop`, `terminated` and counts come from the nodes' own hooks. `judge/scenarios/S22.json` and every protected file are byte-identical (32 of 32 OK, coordinator rerun). Rule C5 (new Pekko difference) was not triggered.

Verification:
- Tests first: E1-E4 (`supervision_defaultrestart_internal_test.go`); RED for E2/E3 before the emit, GREEN after (implementer report). E1 compares the default-restart outcome with and without a sink; E4 shows no event with a custom `PreRestart`.
- `go test -C pekko-actor -race -count=1 ./...`: ok (coordinator rerun on `374f258`). Implementer: `-count=20 -race` of the new tests and `TestJudgeScenarios`: ok.
- Judge selftests on `374f258` (coordinator rerun): `selftest.py` 190 checks 0 failed; `selftest_go_scope.py` and `selftest_mutate_go.py` ran (their last-line output was compressed in my terminal; the implementer reported ok for both).
- Go harness S22, unmutated tree (implementer): 300 of 300 runs pass, contract 1/1/0/2 holds. GMdefaultps alone (implementer, workdir outside the repository, no report): `met`; for its 30 runs the fired rules are `G5.f` (message "c: 1 post-stop, expected 2"), and also `G5.c` and `REQ` as side effects. `G5.f` is the expected clause and fired in every run; the other two clauses fired too, which is reported as it is and not as "only G5.f".
- Implementer's lint (pinned 2.14.0): 0 issues; gofmt and vet clean. First lint run flagged `judgeS22` at complexity 13; it was split, not suppressed. Earlier harness and mutant runs used the pre-refactor `judgeS22`; they are superseded and not combined with the reruns.

### Consequence on committed evidence
`python3 -I scripts/check-judge-matrix.py` now exits 1 with 24 FAIL rows, all `GMoverlap`: the `definition_sha256` of the committed HISTORICAL rows (`judge/go-mutation-report.tsv`, tag `evidence/go-campaign-50fe7c2`) no longer matches the new definition. That is the intended effect of unit B (the old definition was replaced), not a regression. The check is not part of the economical CI (`docs/ci-economy.md`, `.github`), so GitHub is not blocked. Until a valid report replaces that file, the matrix check fails locally; the valid replacement requires the 18 of 18 campaign.

### Incremental budget accounting (measured)
| Item | Cap | Measured |
|---|---|---|
| Unit B implementer | 120,000 | 131,865 |
| Unit S22 implementer | 200,000 | **248,725** (48,725 over its cap) |
| Incremental total (subagents) | 520,000, stop at 416,000 | **380,590** |
| Cumulative, subagents only | | 661,896 + 380,590 = **1,042,486** |

Remaining to the ceiling: 139,410; to the evaluation stop: 35,410. Review gate of `BUDGET.md` (`observed + remaining reserve + 110,000 <= 520,000`): observed 380,590 + review 110,000 = 490,590 before any reserve, and 520,590 with the 30,000 correction reserve alone, so the gate is **not confirmed** and the review is not launched. The coordinator's gates and the campaign need no agent. The review needs an explicit Owner decision on the budget (see the report).

## Update 6: Addendum 2 (fifth brief) and campaign 3 in progress

Recorded in `ADDENDUM-2.md`: deviations DV-B (+11,865 tokens over the 120,000 cap, measured 131,865) and DV-S22 (+48,725 over the 200,000 cap, measured 248,725): neither run was done within its cap; the exceptional ceiling of 600,000 for one Opus review (110,000), one correction round (30,000) and coordination/validation (60,000 estimated, not measured), 580,590 in total, with the 416,000 stop crossable only for those; no new implementers or functional extensions; the review runs after the campaign is evaluated and the candidate is stable, over the whole diff (B, S22, first-round tests, mutation classifier); the GMdefaultps clause corrected to "required detection `G5.f` by the observed `PostStop` count, `G5.c` and `REQ` accepted as consequences"; the historical report kept with its commit, definition and tag, the 24 checker rows against the new GMoverlap definition not being a refutation of that run.

Campaign 3 was launched at `3170ef8` (tree `6b89490e0cae091a9df1889cc4c93e2686507363`, darwin/arm64, go1.27.2, 30 runs per mutation, workdir and report outside the repository) before this addendum; its inputs are not modified by it. Its result is recorded in the next update, whatever it is.

## Update 7: campaign 3 result (INVALID cells; not published, not repeated)

Inputs (unmodified during the run): commit `3170ef84940ec71f1ec1a3ece8293223ad9581d4`, `pekko-actor` tree `6b89490e0cae091a9df1889cc4c93e2686507363`, darwin/arm64, go1.27.2, 30 runs per mutation, clean `judge` and `pekko-actor`; the later documentation commits do not touch those inputs. Report (outside the repository): `/Users/pablogore/.claude/jobs/b989ecff/tmp/camp3/go-mutation-report.tsv`, sha256 `8e18e0c6df4f20b69a4d8a5509066e976bad92dd1c04f0d9cc87446467debabd`, 475 rows; runner log sha256 `a6039d58e69e7b426fd24dfa2d73c068a04a64375604db84eb2cf5e6214c6f89`.

Result: 17 of 19 `met` (including GMoverlap with the new definition, GMdefaultps and the benign mutation); **GM2 and GMstop `UNMET`**. Rows by outcome: 71 failed, 402 survived, **2 harness-error** (GM2 S22 concurrent, GMstop S22 concurrent; 30 of 30 runs each). Their designed detections hit (for GM2: S10 controlled `G5.a`,`G5.c`,`G5.d` and S10 concurrent `G5.d`; GMstop `G5.b`). The report contains INVALID rows, so it is **not valid as 18 of 18, is not published, and is not combined with any other run**.

Cause (read from the logs, nothing re-run): the only INVALID cells are the new S22 `concurrent` cells under these two mutations. The S22 harness waits until c ran its first `PostStop` and both held children began theirs (`awaitS22Wait`, `judge_internal_test.go:954`); otherwise it records `gate-timeout`, a name outside the vocabulary, so the evaluator reports `harness-error` (`event 21: not an object with a known 'e' ('gate-timeout')`). This is the design of the scenario and is the same on the Pekko side (`Judge.scala:478`). Under GM2 (`PostStop` skipped in `stop.go`) the held children never run the gated `PostStop`; under GMstop the S22 logs also contain no `post-stop-begin` of a child. The precondition of the scenario (a restart waiting for held children) is not reached, so S22 cannot judge anything under those mutants. `mutate_go.py` calls a mutation met only if every cell is `failed` or `survived`, so one such cell makes the mutation UNMET although its designed detections hit.

Impact: it comes from adding S22 to the Go scope (unit S22), not from the production change and not from unit B. The candidate cannot show 18 of 18 with the current matrix. No review is launched (the candidate may still change) and no campaign is relaunched; the options and the cost are in the report to the Owner.

## Update 8: campaign 4 (valid for the explicit matrix with two declared exclusions)

Inputs: commit `8ab70a7afca59e0381a85cf92c57d48b0fc59609`, `pekko-actor` tree `60e8161660523c63ff4620659e52267f8c95872f`, `judge` clean, comparator sha256 `c9f2e01702901fc9d78d54bb7310363fb8dab159b22ba2004218bffb3285bfd2`, darwin/arm64, go1.27.2, 30 runs per mutation, workdir and report outside the repository, about 20 minutes. It is a single campaign; it is not combined with campaigns 1-3, their retry or the repetitions, which stay as recorded.

Result (checked by the coordinator with the runner's own `row_problems` and `summarize` over the report, and by reading the console log): 19 of 19 mutations `met`; 475 rows: 71 `failed`, 402 `survived`, **2 `excluded`**, 0 `harness-error`, 0 `timeout`, 0 `invalid-campaign`; 0 row problems; 18 of 18 designed mutations `detected`; GMbenign `benign-survived`; the 2 `excluded` rows are exactly `GM2 S22 concurrent` and `GMstop S22 concurrent` (30 of 30 runs each, every one a harness error with the scenario's own `gate-timeout`, per `EXCLUSIONS.md`); GMoverlap (unit B definition) and GMdefaultps `met`. GMdefaultps S22: `failed`, fired `G5.c`,`G5.f`,`REQ`, with the `G5.f` message "c: 1 post-stop, expected 2", i.e. by the observed `PostStop` count; the runner's evaluator on the 30 logs agrees. No INVALID outside the two declared exclusions.

What it proves: only this explicit matrix with its two declared exclusions. It does not prove S22 under GM2 or GMstop, and it is not Linux evidence. The mechanism inside GMstop that prevents the children from beginning `PostStop` was not traced; the exclusion rests on the measured 0 of 30 (this is also given to the reviewer as a point to verify).

Report: `/Users/pablogore/.claude/jobs/b989ecff/tmp/camp4/go-mutation-report.tsv`, sha256 `93676ef5f124483dd5dd5e657c62e122969de53c2428f106d91dd0edd7fb8c4f` (identical to `judge/go-mutation-report.tsv` as committed with this update); runner log sha256 `79a97b332130dea1f7889f47e7aa89c11154a2fa9c5c654d2d9bcf5b257bf320`.

### Checker, CURRENT and HISTORICAL, read separately
With the new report in `judge/go-mutation-report.tsv`, `python3 -I scripts/check-judge-matrix.py` exits 0. Its CURRENT line: detected 18/18, benign 1/1, for the same Go core tree, comparator, campaign script, specs and toolchain as HEAD. Its HISTORICAL line now describes the same 475 rows of campaign 4 (detected 18/18): **replacing the TSV also removed the rows of the earlier historical run from the file**, so the checker no longer evaluates them. That earlier run (`50fe7c2`, 17 mutations, the previous GMoverlap definition) is kept whole in its tag `evidence/go-campaign-50fe7c2` and in git history; it is not refuted by this change. The TSV replacement therefore renews CURRENT, but does not by itself preserve HISTORICAL in the checked file; keeping the earlier rows checkable would need a separate decision on the report layout, not made here.

### Tag (prepared, not pushed)
`evidence/go-campaign-8ab70a7`, local, pointing at the campaign commit `8ab70a7afca59e0381a85cf92c57d48b0fc59609` (tree `76d6dbc854af8e97286c0cb0117bf2ee9b335199`). The push is `git push origin evidence/go-campaign-8ab70a7` and needs the Owner's authorization.

### Budget
No agent in this step. Incremental subagent total stays at 380,590 (ceiling 600,000, 580,590 with the reserves); coordination is estimated, not measured.

### Evidence status after any correction
If the review leads to a correction that changes `pekko-actor/`, `judge/` (script, specs, scope, comparator) or the toolchain, this campaign stops being CURRENT for the corrected tree and has to be validated again before it is cited as current.

## Update 9: the one Opus review of the final candidate (no correction applied yet)

Reviewed candidate: `51596845bedd7ebdb1a9b89176e5b38125a85eec` (tree `e6c9238a5afa233cef858fcf626ba81075864c18`), `origin/develop..HEAD`, 31 files, +2223/-470; one Opus reviewer, read-only by instruction (no launcher enforces it; the coordinator re-checks the tree below). Prompt: `review-prompt.md` plus seven questions (GMstop mechanism, exclusion tightness, unit B, unit S22, Pekko fidelity, first-round tests, records).

**Review coverage, as declared by the reviewer.** Read: `judge/mutate_go.py`, `judge/selftest_mutate_go.py`, `judge/selftest_go_scope.py`, `judge/go-scope.json`, `supervision_restart.go`, `judge_internal_test.go` (diff and parts), `supervision_defaultrestart_internal_test.go`, `supervision_ticket669_internal_test.go`, and the diffs of `supervision_budget_internal_test.go`, `supervision_restart_goside_internal_test.go`, `supervision_restart_internal_test.go`; `review-prompt.md`, `EXCLUSIONS.md`, `EXECUTION-RECORD.md`. Partly read (key-term scan): `README.md`, `docs/judge.md` (read in full at :296-325), `docs/adr/0017-supervision.md`, `pekko-actor/README.md`, `milestone-1.md`, `judge/go-mutation-report.tsv` (key rows). **Not read:** `RULEBOOK.md`, `ADDENDUM-1.md`, `ADDENDUM-2.md`, `S22-GO-PROPOSAL.md`, `BUDGET.md`, `KIT-USAGE.md`, `RUNBOOK.md`, `implementer-prompt.md`, `manifest.tsv`, `PACKAGE-HASHES.tsv`, `migration/cost-log.tsv`. Pekko lines read: `Actor.scala:620-643`, `FaultHandling.scala:92-266, 278-347`, `Ticket669Spec.scala` (hits), `UidClashTest.scala:46-115`; **not read:** `Mailbox.scala processMailbox`, `Children.scala setChildrenTerminationReason`. Not verified by the reviewer: the pre-U7 single-`PostStop` history, and whether every run of the GM2/GMstop cells recorded `gate-timeout`. So the review does **not** cover the whole diff: the package documents and the cost log were not read.

**Findings: 0 BLOCKER, 3 DEFECT, 4 INFO.** The coordinator re-read the code for each DEFECT:
- DEFECT 1, confirmed. `precondition_not_reached` (`judge/mutate_go.py`) checks that every run of the cell was a harness error, but the `gate-timeout` text it looks for is the detail of the first non-passing run only (`judge/evaluate.py` `bad = next(...)`). If run 1 recorded `gate-timeout` and runs 2-30 another harness error, the cell would still be `excluded`. `postStopGate` and `awaitS22Wait` emit the same event name, so the check cannot tell the two apart; the docs say "every run" while the code checks one message. The selftests of section 13 use a uniform message and do not catch it. **For the evidence of campaign 4 the scenario did not occur:** the coordinator counted the 30 runs of each excluded cell in the kept logs: precondition reached 0 of 30 and `gate-timeout` in 30 of 30 for GM2 and for GMstop (GMdefaultps control: 30 of 30 reached, 0 timeouts). The gap is in the code and in what it can promise in the future, not in the data of this campaign.
- DEFECT 2, confirmed. Test E3 (`supervision_defaultrestart_internal_test.go`) compares the sink without `prerestart-default` against the same list with its first occurrence removed, which is true by construction; its title ("every other event keeps its kind and order") is not tested there. E1 covers the observable outcome with and without a sink; `docs/judge.md` cites E3 as evidence.
- DEFECT 3, confirmed. Stale statements: this record, "Finding" (line 7, "No production code changed") and the table (line 20, "S22 in Go ... pending"); `README.md` line 110 (Go gate scenarios list omits S22 and S13/S14/S16); `docs/judge.md` line 314 ("CURRENT ... has to be re-run" although campaign 4 is committed); `milestone-1.md` line 113 ("not committed" although the report is). Update 5 and Update 8 correct the facts in this record; the headers were not amended.
- INFO 1: the GMstop exclusion is justified in substance, but "not applicable" understates that the mutant creates a visible liveness defect in S22 (a Stop sent to a suspended child never takes effect; Pekko does not do that); S22 can report it only as a hang. INFO 2: `drainExtra` skips `blockedLocked`; a future scenario in which `r` restarts or two overlapped `Receive` calls both fail could bring back race reports (already noted in Updates 2-4). INFO 3: the report's `go_commit` is `8ab70a7`, HEAD is `5159684`; the difference is only the report and this record. INFO 4: `S22.json`, comparator, `evaluate.py`, `go_scope.py` and the Pekko harness are byte-identical to `origin/develop`; no `//nolint`, no weakened clause, scenario or threshold.

**GMstop mechanism (the open point), now traced by the reviewer from the code** (and consistent with the kept log): c's children are suspended when c fails; the default `PreRestart` calls `Stop` on them; under GMstop that sets `extStop`, not `stopRequested`; the mutant applies `extStop` only in the idle branch of `drain`, which comes after `blockedLocked`, so a suspended child returns before any stop takes effect and never begins `PostStop`; in the unmutated code `stopRequested` is checked first. The restart waits forever. GM2 is trivial (the mutant skips `PostStop` in finalize). Q1 verdict: justified in substance. Pekko fidelity (Q5): the Go behaviour matches Pekko (two `PostStop` on the old instance, one `Terminated`, no `PostRestart`, no new instance); no undeclared difference was seen.

**Tree check after the review (coordinator):** `git status` is clean apart from the untracked `odd/`; no file was modified by the reviewer.

### Budget: measured, and over its cap
| Item | Cap | Measured |
|---|---|---|
| Opus review | 110,000 | **215,968** (**+105,968**, deviation DV-REV; not within its cap) |
| Incremental total (subagents) | ceiling 600,000 | 380,590 + 215,968 = **596,558** |
| Cumulative, subagents only | | 661,896 + 596,558 = **1,258,454** |

Remaining to the exceptional ceiling: **3,442**. The reserves granted for one correction round (30,000) and for coordination (60,000, an estimate) are not covered any more: `596,558 + 30,000 = 626,558 > 600,000`, so the gate for a correction round is **not satisfied** and no correction was started. The three DEFECTs are confirmed but not corrected. Corrections of DEFECT 1 and DEFECT 2 would change inputs of campaign 4 (`judge/mutate_go.py` and a test under `pekko-actor/`), so its evidence would stop being CURRENT until a new campaign is run and the corrected candidate validated; DEFECT 3 is documentation and changes no campaign input. No second review is authorized.

## Update 10: correction round after the review, and campaign 5

Authorization: the Owner chose option A (correct the three confirmed DEFECTs) and raised the incremental ceiling to 700,000 tokens exclusively for this correction, validation and campaign 5, without new agents and without a second review. The 596,558 measured tokens and every earlier deviation (DV-B, DV-S22, DV-REV) stay. The work was done by the coordinator in this session; its cost is not measured, so no exact total is claimed. Measured subagent total so far: 596,558, with no agent in this round.

### Corrections (correction procedure: RED before, GREEN after, gates, frozen candidate, new campaign)
- **DEFECT 1, exclusion (`0be21a0`, `judge/mutate_go.py`, `judge/selftest_mutate_go.py` section 14).** RED, observed before the change: four mixed cases came out `excluded` ("gate-timeout followed by another harness error", "another harness error in the last run only", "every run a gate-timeout but the children had begun PostStop", "the guard in 29 runs and the other gate in one"; 8 failing checks). The cases "gate-timeout followed by passes / failures / a timeout" were already refused by the cell aggregate; they are kept as regression guards and were not RED. GREEN after: `judge()` now keeps a per-run detail for a cell in which every run was a harness error; `precondition_not_reached` requires, for every run, class harness-error, its own message `gate-timeout`, and a log in which c's first `PostStop` and both children's `post-stop-begin` did not all happen (the same event name is used by the gate of the children's held `PostStop`). The report row `excluded` must also carry the exclusion note. The fixtures of section 13 were moved to the per-run shape; no earlier selftest case was removed.
- **DEFECT 2, test E3 (`6fdff86`).** The tautology (a list compared with itself minus one element) was replaced by three independent observers of one real run, compared stage by stage: the runtime sink, the hook journal and the watcher's `Terminated`, plus the factory calls. Relations required: `prerestart-default` comes before the `stop-accepted` of the child; P1 is done at the first stage and P2 waits for the child (Pekko: `FaultHandling.scala` `terminate`/`finishTerminate`, `Actor.scala:625-631`); one `Terminated`; no second instance. No total order between observers or among runtime events is imposed. Demonstration that it fails when the protected property changes (three altered copies outside the repository, `/Users/pablogore/.claude/jobs/b989ecff/tmp/e3m/`): emit placed after the hook's effect on the child: E3 fails (only E3, where E2 does not); emit twice: E2 and E3 fail; no emit: E2 and E3 fail.
- **DEFECT 3, stale statements (`a0dea21`).** Corrected: the record's "Finding" header and the S22 row, `README.md` (the Go gate runs 15 scenarios), `docs/judge.md` (CURRENT is bound to the inputs; the exclusion rule now says every run; the GMstop liveness defect, citing `Mailbox.scala:223-233` and `:269`, read by the coordinator), `milestone-1.md` (the Go report is committed), and `EXCLUSIONS.md` (addendum: mechanism traced, rule tightened).

### What the review did not cover, and what has not been re-reviewed
The one review (Update 9) did not read `RULEBOOK.md`, `ADDENDUM-1.md`, `ADDENDUM-2.md`, `S22-GO-PROPOSAL.md`, `BUDGET.md`, `KIT-USAGE.md`, `RUNBOOK.md`, `implementer-prompt.md`, `manifest.tsv`, `PACKAGE-HASHES.tsv` or `migration/cost-log.tsv`, and only scanned `README.md`, `docs/judge.md` (read in full at :296-325), ADR 0017, `pekko-actor/README.md` and `milestone-1.md`; it did not read `Mailbox.scala processMailbox` or `Children.scala setChildrenTerminationReason` of Pekko. **The three corrections above, the new section 13/14 selftests, the new E3 and the documents edited after the review (including `EXCLUSIONS.md` and this update) were not reviewed by anyone but their author.** INFO 1-4 of the review were recorded and not acted on, except that INFO 1 is now stated in `docs/judge.md`.

### Gates on the corrected candidate (all by the coordinator, darwin/arm64, go1.27.2)
`gofmt -l pekko-actor` empty; `go vet` clean; `go test -race -count=1 ./...` ok; the default-restart and judge tests `-count=20 -race` ok; `selftest_mutate_go.py`, `selftest_go_scope.py` ok and `selftest.py` 190 checks 0 failed; pinned `golangci-lint` 2.14.0 (go1.27.2, gocyclo inside its config) 0 issues; `ci-fast`, `check_deps.sh`, `check-headers.sh`, `check-adr-signatures.sh` ok; per-module coverage: pekko-actor 98.09%, factory 99.54%, tools 100% (three modules with no executable code skipped); the 32 protected files (S22, comparator, `Judge.scala`, `cell.go`, `stop.go`, `hierarchy.go`, `guardian.go`, `supervision_failure.go`) byte-identical to the baseline.

### Campaign 4 evidence status
Campaign 4 (commit `8ab70a7`, report sha256 `93676ef5f124483dd5dd5e657c62e122969de53c2428f106d91dd0edd7fb8c4f`) is valid for its own commit and tree and was not edited. **It is no longer CURRENT for the corrected candidate**: `judge/mutate_go.py` (a comparator input) and a test under `pekko-actor/` changed after it. Its report stays in git history (`5159684`) and a copy in the job directory; its local tag `evidence/go-campaign-8ab70a7` was not moved, overwritten or pushed.

### Campaign 5 (frozen candidate `a0dea217cca79adf0ac587737c81185580fee242`, `pekko-actor` tree `bc1393ccaf7f221fcff9d53578d10df706855f43`)
Single campaign, darwin/arm64, go1.27.2, 30 runs per mutation, clean `judge` and `pekko-actor`, workdir and report outside the repository; comparator sha256 `65f4dfcf478a0fc5815464f2445499137bfbe55755628108295081599fcc7d66` (it differs from campaign 4's because `judge/mutate_go.py` is one of its inputs). Not combined with campaigns 1-4. Result, checked with the runner's own `row_problems` and `summarize` and by reading the console log: 19 of 19 `met`; 475 rows: 71 `failed`, 402 `survived`, 2 `excluded`, 0 `harness-error`, 0 `timeout`, 0 `invalid-campaign`; 0 row problems; 18 of 18 designed mutations `detected`; GMbenign `benign-survived`; the two `excluded` rows are exactly `GM2 S22 concurrent` and `GMstop S22 concurrent` (30 of 30 runs each, judged by the corrected per-run rule); GMdefaultps S22 `failed`, fired `G5.c`,`G5.f`,`REQ`, expected `G5.f` hit. No INVALID outside the two declared exclusions. It proves only this explicit matrix with its two declared exclusions; it does not prove S22 under GM2 or GMstop and is not Linux evidence.

Report sha256 `8ac896970b3bf94714bfb81e18f6290f62c17e967fa8dc2a08e5f600b6308f90`, installed unchanged in `judge/go-mutation-report.tsv`; runner log sha256 `79a97b332130dea1f7889f47e7aa89c11154a2fa9c5c654d2d9bcf5b257bf320` (the same text as campaign 4's log, which prints no timing).

### Checker, read separately
`python3 -I scripts/check-judge-matrix.py` with this report: first line `OK`; CURRENT on darwin/arm64: detected 18/18, benign 1/1 (same Go core tree, comparator, campaign script, specs and toolchain as the campaign commit). HISTORICAL now describes the same 475 rows of this report. Replacing the TSV renews CURRENT; it does not keep the earlier historical rows in the checked file (campaign 4's, and before it the `50fe7c2` run, remain in git history and in their tags). Not resolved by this change: Linux evidence, Pekko mutation of `G5.f`, calibration of S13/S14/S16/S22.

### Tag (prepared, not pushed)
`evidence/go-campaign-a0dea21` points at the campaign commit `a0dea217cca79adf0ac587737c81185580fee242` (a lightweight tag, like the existing ones). Pushing it, `git push origin evidence/go-campaign-a0dea21`, needs the Owner's authorization. After a squash merge the campaign commit leaves the history and the checker's history check needs that tag, as in U8 and U9.

## Update 11: evidence tag published

Authorization (Owner): publish only `evidence/go-campaign-a0dea21`; no other tag, no further Opus review, no further campaign without a change that justifies it, no functional change; the caveat that the later corrections had no second independent review stays.

Checks before the push, by the coordinator: the local tag resolved to exactly `a0dea217cca79adf0ac587737c81185580fee242`; it did not exist on the remote (`git ls-remote`), so no force was needed or used; `judge/go-mutation-report.tsv` has sha256 `8ac896970b3bf94714bfb81e18f6290f62c17e967fa8dc2a08e5f600b6308f90` and its 475 rows all carry commit `a0dea217cca79adf0ac587737c81185580fee242` and `pekko-actor` tree `bc1393ccaf7f221fcff9d53578d10df706855f43` (71 `failed`, 402 `survived`, 2 `excluded`: `GM2 S22 concurrent` and `GMstop S22 concurrent`); `scripts/check-judge-matrix.py` exit 0, CURRENT on darwin/arm64 detected 18/18 and benign 1/1, HISTORICAL the same 475 rows.

Executed: `git push origin evidence/go-campaign-a0dea21`. Verified afterwards: `git ls-remote --tags origin` lists `refs/tags/evidence/go-campaign-a0dea21` at `a0dea217cca79adf0ac587737c81185580fee242`. `evidence/go-campaign-8ab70a7` (campaign 4) remains local, was not moved and is not on the remote.

PR #480 at the time of the push: draft, open, mergeable, head `6024e7faba06949b4f225880b762fc197e5f255c`, checks `go` and `label` successful. The only difference between that head and the campaign commit is the installed report and this record (documentation and evidence files; `pekko-actor/` and `judge/` scripts are identical to the campaign commit).

### Technical close and what remains, for the merge decision
What the candidate delivers and proves: see Updates 8-10 (campaign 5 proves only its explicit matrix with two declared exclusions, on darwin/arm64). What it does not: S22 under GM2 or GMstop, Linux evidence, Pekko-side mutation of `G5.f`, calibration of S13/S14/S16/S22, the closure of #173 or #461, the module close.
Review limits: one Opus review of `5159684` that did not read the package documents or the cost log; the corrections after it (exclusion rule, E3, stale statements, selftest sections 13-14, `EXCLUSIONS.md` addendum, Updates 9-11) were not independently re-reviewed. Open: n3 (`correction_required`, provider resolution), #459 and #460, the other pending items of #461, the `--help` workdir behavior of `mutate_go.py`, a squash merge dropping the campaign commit from history (the tag now retains it).
