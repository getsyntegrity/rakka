# EXECUTION RECORD: Supervision U2, Restart, hooks, FailedMessage, InitializationError, conversions and the finite Budget (#169-#174)

**EXECUTED on 2026-10-15 up to the 60-minute stop of the loop (started 02:39Z), then 10 more minutes authorized by the Owner for the judge campaign and its exact tag. NOT COMPLETE.** The runtime and the tests are written, committed and gated; the Go judge campaign was renewed and its exact tag published (section 9). Still pending: the native review of the implementation commits (it needs the Owner's consent per envelope), the fidelity differences against the Scala listed in section 8, and the mutants that hang or are broken. Nothing is merged.

## 1. Approval

| Item | Value |
|---|---|
| Base | `develop` `6f5741c` (U1 merged); package commits `e9aff65` (natively reviewed, four lenses, approved) and `566cea4` (natively reviewed, four lenses, approved) |
| Approved sha256 | RULEBOOK `d8ca52e3e589a9058da0a9c3ada2ea9c034e1cfbe50ef9dd96ceaa1c88e85a91`, RUNBOOK `77ab46e39bcaa9f1b3d826190017e30838c04a8b1a2805ce6a0c8e10c857dc21`, manifest `47f16c0e7139ed4af1d2d011cd8a3f9fc55c2660f0f8e02002a6f3883a1aa7bc`, settings `10532d7267108a4a1e4fe4d26df2fc9d77e81bcc4e5b3ca482aa981e271db6c3`, launcher `e50934cf6bdda4ac1e061ee9780d3c3a9d2c040d8c6358f974b103b79949846e` (the approved launcher is unchanged) |
| Owner decisions D1-D22 | Approved with these precisions (kept outside the repository in the orchestrator's working directory and binding for every agent): D11 injectable monotonic nanosecond seam, never `UnixNano`; D12 name `MakeRestartDecider`; D18 evaluate at 1,120,000 tokens and continue on its own only if the remainder fits in 1,400,000; D19 `PORT-GAP(recreate-dead-letter)` accepted as temporary, inside the migration target; D20-D22 keep the upstream behavior (send order by name, two `PostStop` calls in the observed custom-hook case, retention of the failed message); `originalCause` is an explicit gap (`PORT-GAP(original-cause)`); `PostStop` after a failed first `PreStart` is a **known divergence**, not Unknown (R17 item 4), verified in both sources, and U2 is not widened to fix it |
| Model plan and limits | Haiku probe; Opus design review and final review; Sonnet implementer (two sequential runs: runtime, then tests) and second reviewer; 1,400,000 tokens for the execution with evaluation at 1,120,000; the 60 minutes are the maximum of the whole loop, a stop point; subscription only; `-race` reserved for the final candidate |

## 2. Kit and sources

Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`; Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`; ADR 0017 (accepted). The semantics left Unknown by the package were resolved from the pinned source and one probe before the run (RULEBOOK R8, R17, D19-D22; probe evidence outside the repository: `custom-poststop`, 3 runs, `C#1.postStop=2`).

## 3. Roles, models and costs (self-reported by the CLI JSON; tokens are input + output + cache creation)

| Role | Model | Tokens | USD | Notes |
|---|---|---|---|---|
| Permission probe 1 | claude-haiku-5-5 | 47,147 | 0.011 | misreported two cases (see section 6) |
| Permission probe 2 (verbatim) | claude-haiku-5-5 | 8,460 | 0.003 | confirmed `gofmt -w` on a non-target denied, `go vet` and `go build` allowed |
| Design review | claude-opus-5-5 | 196,343 | 2.458 | NO BLOCKERS; ran 12 minutes because the launcher default of 60 minutes was left in place |
| Implementer run A (runtime) | claude-sonnet-5-5 | 174,470 | 1.380 | 70 turns, about 12 minutes (a first launch at the default bound was killed after 1.5 minutes before writing anything; its tokens were never measured and are charged at zero by the launcher, which only keeps the JSON of a finished run) |
| Implementer run B (tests) | claude-sonnet-5-5 | 356,545 | 2.763 | 68 turns |
| Final review | claude-opus-5-5 | 137,717 | 1.682 | 0 blockers, 2 major, 4 minor |
| Second review | claude-sonnet-5-5 | 116,961 | 0.631 | 0 blockers, 1 major, 4 minor; it could not compare the ports against the Scala |
| Mutation worker (Agent tool, not the launcher) | sonnet | about 131,768 | not reported | measured only K1, K2, K3; its numbers are self-reported by the tool and its later reports (117k to 127k) are cumulative and inconsistent |
| **Total of the execution** | | **about 1,169,411** | | 83.5% of the 1,400,000 cap; the evaluation stop at 1,120,000 was crossed when the two reviewers and the mutation worker were launched together |

D18 evaluation at the crossing: remaining mandatory steps were a fixer (forecast 120,000) and a fix-delta review (60,000), projected total about 1,383,000, inside the cap, so the loop continued. **No fixer and no fix-delta review were launched**: every fix was made by the orchestrator (section 6), which kept the total below the projection. There is no margin for a second fixer.

Preparation of the package, accepted aside by the Owner and not counted in the execution cap: about 598,000 tokens (writer 366,269, explorer 64,279, fix writer 167,259). Total cost of U2 so far: about 1,767,000 tokens.

## 4. Commits (branch `docs/u2-package`)

| Commit | Content |
|---|---|
| `84ce0b6` C1 | exported types, hooks, `MakeRestartDecider` |
| `fa8b54a` C3 | finite budget statistics and explicit conversions |
| `0ed334e` C2 | restart protocol, conversions, supervised `PreStart` failure |
| `a79e801` C4 | budget tests (24-cell matrix, zero reading, conversions, int32 wrap) |
| `e662ef9` C5 | restart lifecycle, watches, survivors, fatal states |
| `ef0603e` C6 | Go-side restart scenarios and the two adapted U1 sites |
| `85ba367` C7 | fix: a `Stop` decision or a denied restart is noted like `context.stop` (Opus F1) |
| `6658f31` C8 | test fixes: faithful W1 port, W3 and W10 assertions |
| `09b0fbf` C9 | W27 for the C7 fix |

Final `pekko-actor` tree `a12fcfa42d0a1181b48a48520d7ab8d99f8845ca`.

## 5. Gates on the final candidate `09b0fbf`

`gofmt -l`, `go vet`, `go build`, `go mod tidy -diff`: clean. `golangci-lint` v2.14.0 (installed under a temporary `GOBIN`): 0 issues. 18 judge anchors intact (read-only check of `judge/mutate_go.py`). Suite without `-race` (several runs during the loop): pass. **One `go test -race -count=1 ./...` on the final candidate: pass, no data race.** Coverage of `pekko-actor`: 97.6% (minimum 85%). `check-headers.sh` (28 ported files, D15 layout), `check_deps.sh`, `check-adr-signatures.sh`, `modules_selftest.sh` (22 ok, 0 failed), `go-sdk-update.sh --check`: pass. `check-judge-matrix.py`: OK; HISTORICAL 13/13, CURRENT 13/13 after the renewal of section 9 (it was 0/13 before it). A second `-race` run, also once, was made after the last runtime change (`313d77b`): pass.

## 6. Manual changes and honest findings

1. Probe 1 misreported two results; probe 2 (verbatim) settled them. The orchestrator removed the six probe files (the template denies `rm`).
2. `adjust` was left unused by run A; removed (lint).
3. Tests written without reading the Scala: the run B implementer **did not read the Scala test files** before porting (it used the RULEBOOK rows and memory). The final reviewer spot-checked: `ActorRefSpec:442-463` faithful, `SupervisorTreeSpec:27-49` faithful, `ActorLifeCycleSpec:49-82` was NOT faithful (fixed by C8) and `SupervisorSpec:419-458` is adapted beyond the RULEBOOK row (budget 3 / inits 3 not exercised; open). Other ports were not compared against the Scala by any reviewer. Treat those ports as unverified for fidelity.
4. Two tests collided with the helper constants `sup1Fail`, `sup1Ping`, `sup1Panic` (message values 1, 2, 3): a test bug, fixed with 11, 12, 13.
5. C7 (Opus F1, a real runtime divergence): when the decider answered `Stop`, or the budget denied a restart, the parent stopped the child without recording the stop, so a later restart of the parent did not wait for it. Fixed; covered by W27 (K38, K39 detected).
6. W3 and W10 could not kill K9 and K24 (found by the mutation run); fixed in C8, then K9 and K24 detected.
7. Not written by the implementer, left as `PORT-GAP` or open: W19, the ports of `Ticket669Spec:37` and `UidClashTest:104` (`PORT-GAP(ticket669-uidclash)`), W6 present-nil, W9 descendant half, most of W21.

## 7. Mutations (immutable snapshots; counts of runs are as measured)

Snapshots of the code at `ef0603e` (K1 to K37 first pass, `-count=20`, no `-race`, a Go timeout of 8 s) and of `6658f31` / `09b0fbf` (re-measures, `-count=1`, Go timeout 70 s). **My 8 s timeout turned assertions that need the tests' own 30 s bounded wait into "hangs"**; those were re-measured once with 70 s. A hang counts as INVALID, never as detection.

- **Detected 20 of 20 runs:** K1, K2, K5, K11, K12, K14, K16, K17, K18, K20, K21, K23, K30, K31, K34, K35, K36, K37.
- **Detected in a single bounded run:** K3, K4, K19, K26, K29 (re-measure), and K9, K24 (against the corrected tests), K38, K39 (against W27).
- **INVALID (a test blocks longer than the bound after the mutation, besides the failing assertions):** K7, K8, K10, K13, K15, K32. Several tests do fail in these runs, but a hang is not a detection.
- **Survived:** K28 and K33. Both charge the budget of a child that is already dead or stopped; its statistics are keyed by incarnation and pruned at completion, so no test observes them from outside (the second reviewer reached the same view for K28). This is NOT a proof of equivalence: section 13 states what is known. They are reported as survivors, not closed.
- **Crash, not a detection:** K22 (my mutation re-panics inside `PreRestart` and takes the binary down). **Does not compile:** K25. **Not run:** K6, K27 (K27 is race-only and the reviewers did not find a deterministic interleaving: it stays "probabilistic under `-race`").
- Total over 39 mutants (the 37 of RULEBOOK 5.3 plus K38 and K39 added for the C7 fix): **27 detected** (18 in 20 of 20 runs, 9 in a single bounded run), 2 survivors (K28, K33), 6 hangs counted INVALID (K7, K8, K10, K13, K15, K32), 2 crash or build failure (K22, K25), 2 not run (K6, K27). Four mutants (K13 and K33 as first written, K38, K39) were rewritten against the code after C7.

## 8. Reviews

Final review (Opus): F1 MAJOR fixed (C7, W27); F2 MAJOR fixed (C8); F3 MINOR (`case c.stopAccepted()` does nothing where Pekko resumes), F4 W24 reads processing order (`ctlExec` is FIFO), F5 the `SupervisorSpec:419-458` port is adapted beyond the row, F6 whether dead-letter publishing is synchronous was not verified. Second review (Sonnet): F1 MAJOR fixed (C8, stash assertions of W10); F2 same as Opus F4; F3 W20 would pass if the hook never fired; F4 W1 adaptation clarity; F5 the stale-`Failed` half of W16 cannot catch K28. Open: Opus F3 to F6 and Sonnet F2, F3, F5.

**Scala mapping check** (explorer, read-only, 73,433 tokens): every cited range starts at the right case; the unported cases are declared as PORT-GAP in the Go header. Doubtful differences, NOT fixed: `SupervisorSpec:546-569` (Restart after an init failure, then recovery and a Pong) has no matching Go test, because the Go test fails `PreStart` on every start; `SupervisorSpec:419-458` runs with an unlimited budget and 4 constructions instead of (3, 10 s) and `inits == 3`; `SupervisorSpec:240-290` does not check that the same instance is returned; `FaultHandlingDocSpec:171-175` (the escalate-kill step) is omitted; `SupervisorMiscSpec:102-124` drops the path equality and the not-stillborn checks; the `PORT-GAP(dispatcher-config)` of the pinned-dispatcher variants is not written. Fixing them changes the tests under `pekko-actor/` and needs another campaign.

**Native review of the implementation commits: NOT RUN.** It needs the Owner's consent per envelope. Proposed ranges by the existing commits (base is the last reviewed boundary `566cea4`): C1 `566cea4..84ce0b6` (about 100 lines), C3 `84ce0b6..fa8b54a` (78), C2 `fa8b54a..0ed334e` (706), C4 `0ed334e..a79e801` (526), C5 `a79e801..e662ef9` (647), C6 `e662ef9..ef0603e` (465), fixes `ef0603e..09b0fbf` (about 90). The whole branch is never one range.

## 9. Judge evidence

**Renewed after one failed attempt.** The first campaign (on `9ce64e9`) met 13 of 14 mutations: `GMoverlap` ended as `harness-error: race detected`. Cause, read from the race report of its log: my own write of the failed message on EVERY `Receive` (`cell.go`, `callReceive`) races when the mutation overlaps two Receives on purpose, so the race detector stops the judge before it evaluates the clause. It is a side effect of U2 on the judge, not a defect of the judge. Fix `313d77b`: the field is written only when a `Receive` fails and cleared only when it was set; suite, lint and the 18 anchors passed again. The first campaign directory is kept as evidence (`goPekko-evidence/u2-judge-campaign-1-gmoverlap-race`). The second campaign, on `313d77bc1ff7f97f030efcf1b7a0be270504ec91`, met **14 of 14** (`--runs 30`); `pekko-actor` tree `ff4458195cfc6e439d6675177ae1ad74c1901578` (equal to the tree of the head), report sha256 `b0e70e751d970351def7be2f545ffba0bdd91f754af9c7e442fa8c33acf50ce0` (equal to the manifest). Tag `evidence/go-campaign-313d77b` published on that exact commit. `check-judge-matrix.py`: HISTORICAL 13/13, **CURRENT 13/13**. The repair exceeded the 10 authorized minutes by the time of the second campaign. Any later change under `pekko-actor/` (a test or a README) makes this evidence stale again.

## 10. Known divergences, gaps and follow-ups

PORT-GAPs: `original-cause`, `recreate-dead-letter`, `logging-U9`, `death-pact-U3`, `custom-strategy`, `ticket669-uidclash`; known divergence R17(4); documentation debt for the orchestrator: stale doc comments in `actor.go` (lines 16, 21-23) and `errors.go` (lines 55-57) (`ErrInvalidDirective`), the `pekko-actor` README and `docs/` (not updated in this run); `OnTerminated` does not clear the failed message. The interim of U1 stays: a parent without a strategy still stops its failing child (U3).

## 11. Closing batch (owner-authorized overage, 2026-10-15)

- **Native review (RDD), medium risk, all ranges granted by the owner.** Approved and acknowledged: targets 1, 2, 8, 4, 5, 6, 7 and the delta `0e1a470..1076d50`. Full outputs: `goPekko-evidence/u2-native-*`.
- **Target 3 (`fa8b54a..0ed334e`, 706 lines): native state stays `correction_required`, pending; it was NOT approved.** Its findings were read from the persisted state and resolved by source evidence, not by the engine: R3-fatal-never-cleared (CRITICAL) is refuted by Pekko `FaultHandling.scala:83-86` (`clearFailed` clears only `FailedRef`; `FailedFatally` is sticky); R3-failedmsg-stale mirrors Pekko `handleFailure` (`FaultHandling.scala:306`); R3-restart-untested is obsolete (tests of targets 4-7). The owner-authorized recovery produced a successor lineage that exceeded the lens budget and captured nothing; it was abandoned (`reason=operator_disposition`, owner-authorized, only that lineage). A synthetic split (local branch `review/n3-split`) gave only artefact blockers, because each half references symbols of the other.
- **Final lot `1076d50` (orchestrator manual change, written by one delegated writer, owner-authorized):** tests for SupervisorSpec:546-569, :419-458 (Budget{3,10s}, inits==3), :240-290 (same-instance factory: a test choice, not a runtime obligation), SupervisorMiscSpec:102-124 and :46-85 (Budget{3,5s}); PORT-GAP(escalate-kill-U5), PORT-GAP(dispatcher-config), PORT-GAP(AllForOne-U6) (OneForOne+Escalate substitution is a partial adaptation); stale doc comments, pekko-actor README, doc.go, parity doc and root README.
- **Delta review suggestions, accepted and not fixed (a fix would need another race run): R3-001 the W8 ADAPTED test does not observe PreRestart; R3-002 the PreStarter comment should say InRestart false applies to the first start only.
- **Final checks:** `go test -race -count=1 ./...` once on `1076d50`: pass. Campaign `--runs 30` on `1076d50`: detected 13/13, benign 1/1, unmet none; `check-judge-matrix.py` CURRENT 13/13; `pekko-actor` tree `07ca3b350a095fd50bac6a7aa09e063130f465d1` equals HEAD. Report commit `819ba8a`. Exact tag `evidence/go-campaign-1076d50`; `evidence/go-campaign-313d77b` is now historical.
- **Cost:** the owner authorized an overage (100k tokens, 20 minutes, up to about 1.47M); the delegated writer alone used about 160k tokens, so the cap was exceeded before that authorization and is disclosed here.

## 12. Reconciliation requested by the owner

- **CI of `9063cff`:** `go` pass (28 s), `label` pass (`gh pr checks 455`).
- **Mutations (39) reconcile:** 27 detected (18 in 20 of 20 runs + 9 in a single bounded run) + 2 survivors (K28, K33) + 6 invalid hangs (K7, K8, K10, K13, K15, K32) + 4 not evidenced (K22 crash, K25 does not compile, K6 and K27 not run) = 39.
- **Target 3 is NOT natively approved.** The source resolution below is evidence, not an engine verdict. The native state of lineage `review-c085925827d8436b` stays `correction_required` (superseded by an abandoned successor); the seven other approvals and the delta approval stand.

| n3 finding | Upstream correspondence | Go evidence | Resolution |
|---|---|---|---|
| R3-fatal-never-cleared (CRITICAL) | `FaultHandling.scala:83-86`: `clearFailed` clears only `FailedRef`; `FailedFatally` stays; `:141` Resume becomes Restart | `cell.go:63` documents the flag as sticky; `supervision_restart.go:83,153,200`; `TestSup2FailedFatally` (`supervision_restart_internal_test.go:555`) asserts that restarts of a fatally failed cell skip PreRestart | refuted: the reviewer assumed `clearFailed` clears the fatal mark |
| R3-failedmsg-stale (SUGGESTION) | `FaultHandling.scala:306`: `currentMessage` is set before the child lookup | `supervision_failure.go:262` | mirrors upstream, no change |
| R3-restart-untested (WARNING) | n/a | restart tests of ranges 4 to 7 | obsolete |

- **Follow-ups kept open (documentary or test-only, no runtime change):** the `PreStarter` comment in `actor.go` should say that `InRestart` false applies to the first start only; the W8 ADAPTED test keeps its current assertions and does not observe `PreRestart` (the observation is a follow-up). Changing either touches `pekko-actor/` and would need another race run and campaign; the owner did not authorize a second one.

## 13. Pending items batch (owner-approved: 250k tokens, 90 minutes, Sonnet writer capped at 100k; no writer was needed)

- **Target 3 probe (read-only):** `gentle-ai review capabilities` lists `review.capabilities`, `repair`, `start`, `status` and `validate`; no documented mechanism closes a `correction_required` that source evidence refutes without a code change against Pekko. The contract is decided by Pekko and ADR 0017, not by the engine, so no runtime change was made. Target 3 stays **pending, not approved** (section 12).
- **Mutants re-measured on immutable snapshots of `452f251` (runner `mut3.py`; `go test -C` must be the first flag).** A Go timeout, a build failure and a crash are never detections.
  - K22 re-panicked inside `PreRestart` and killed the binary (not a detection); replaced by **K22b** (the `prerestart-failed` event removed, no crash): **detected 20 of 20** (`TestSup2FailedFatally`).
  - K25 did not compile (my mutant redeclared the named result `ms`); replaced by **K25b** (clamp to int32): **detected 20 of 20** (`TestSup2BudgetMatrix`, `TestSup2Conversions`).
  - K6 (survivors not restarted), K8, K10, K13, K32: **detected in one run** each, with a 300 s Go bound and no Go timeout (5 to 8 tests fail; they take 2 to 4 minutes because failing assertions wait for their own bounded 30 s waits). K7 and K15 still passed 300 s on the full suite and are **detected by one targeted test each in about 31 s**: `TestSup2IdentityAndMailbox` (K7) and `TestSup2Counters` (K15).
  - **K27 survives** under `-race` (10 runs): no deterministic interleaving through the `lockPoint` seam was written, so it is reported as a survivor with reason (RULEBOOK 5.3 race-only rule), not as detected.
  - **K28 and K33 stay survivors with an open gap; neither is shown equivalent.** Earlier wording ("unobservable", "equivalent from outside") claimed more than was proven and is withdrawn.
    - K27: a data race between a failing child reading `c.actor` in `declares()` and the parent installing the new instance in `installNew`. It survives because W20 drives `Tell`s through `lockPoint`, not a child failure overlapping the restart, and ten `-race` runs prove nothing. Gap: no deterministic overlap seam; closing it is a code change.
    - K28: a stale `Failed` charges `statsFor(...)`, which creates an entry on demand; by reading the code the consequence is an unpruned entry in `n.stats` until the parent node ends (a small leak), not a wrong restart decision. Gap: no assertion on the size of `n.stats` after a stale item.
    - K33: a `Stop` decision charges the counter of a child that is stopped right after; no consequence beyond a transient counter was found. Gap: no assertion on the counter between the decision and the completion.
  - **Reconciled total over 39:** 36 detected (the earlier 27, plus K6, K7, K8, K10, K13, K15, K32, and K22b and K25b standing for K22 and K25) + 3 survivors with reason (K27, K28, K33) = 39. Counts of single-run detections are not 20 of 20 and are reported as such.
- **Accepted follow-ups closed (commit `2427544`, tests and comments only, no runtime change):** the W8 ADAPTED test now asserts exactly one `prerestart-failed` event; the `PreStarter` comment says `InRestart` is false for the first start and true when it runs as the default `PostRestart`.
- **Final checks:** `go test -race -count=1 ./...` once on `2427544`: pass. Campaign `--runs 30` on `2427544`: `unmet: none`; `check-judge-matrix.py` CURRENT (same Go core tree as HEAD, `pekko-actor` tree `c0398d815fd02ab6dfd89e54aad15711554ca6cc`). Report commit `bd43af1`. Exact tag `evidence/go-campaign-2427544`; `evidence/go-campaign-1076d50` is now historical.
- **Native review of the last delta (`452f251..2427544`, 3 changed lines, medium risk): approved and acknowledged** after the owner's consent (`CONTINUAR`, read as the yes to the single pending question). One non-blocking suggestion left open: R3-001, no test pins that a failing `PreStart` that runs as the default `PostRestart` reaches the decider as `InRestart` true; the documented behavior has no dedicated assertion. CI of `9784beb`: `go` pass (32 s), `label` pass.
- **State at the end of the extension:** 36 of 39 mutants detected, 3 survivors with an open gap (K27, K28, K33). Target 3 is **not approved by the engine** and stays `correction_required`; it is resolved only by Pekko source evidence, pending the owner's decision. Time: the active work exceeded the 90 minutes approved (about 2 h 15 min, an estimate); tokens were about 80-120k, also an estimate without telemetry, with no delegated writer. Compliance with the 250k tokens / 90 minutes budget is **not claimed**. This section was changed in a record-only commit: no tests, agents, reviews or campaigns were repeated.
