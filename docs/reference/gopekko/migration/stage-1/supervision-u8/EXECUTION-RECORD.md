# U8 execution record (#175)

Scope: U8 only (judge scenarios S13, S14 and S16 in the Go gate). This is not a claim of Pekko parity and it does not close #175. Package: PR #473 (squash `d5b831b`; head `c22c5a7`; the eight approved hashes were verified against the committed files before the merge and again on the code branch). Code: branch `feat/actor-judge-go-u8` (deleted after the merge), PR #474 merged by squash `3054971` (its tree `770fd3b` equals that of the verified head `c922a06`), commits `94ed1c6` (harness, scope, mutants, judge docs), `64bfbaa` (module README), `d14d7d9` (review fix), `36e5f49` (S14 probe, superseded), `e543c6a` (S14 factory failure), `784fb0c` (Go mutation report) and the documentation commit. Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.
**Order of the procedure changed by the Owner:** all of `pekko-actor/` (its README included) and `judge/` are committed and clean before the final campaign, because the report is bound to the `pekko-actor/` tree and marks `+dirty`; the RUNBOOK (hash-approved) was not edited.

## Delivered
- **Go judge** (`pekko-actor/actor/judge_internal_test.go`): per-name instance numbers; `PreRestart` (records `pre-restart`, stops the children, calls `PostStop`) and `PostRestart` (records `post-restart`, runs `PreStart`) on `judgeNode`; a supervisor node whose decider records `decided {subj, k, v}`; a second actor type `judgeWatcher` with a comparable message (Go compares `WatchWith` messages with `==`, and `judgeMsg` holds funcs); bodies `judgeS13`, `judgeS14`, `judgeS16` in both modes; header rewritten. The logs of the existing scenarios do not change (one instance, `i: 1`).
- **Scope and mutants:** S13, S14, S16 in `judge/go-scope.json` (no waiver, no diagnostic). Four directed mutants in `judge/mutate_go.py`: GMrestartnotify (M12, S13 `G6.d`), GMinitrestart (M14, S14 `G7.b`), GMwatchoverwrite (M43, S16 `I6.a`), GMwatchsamefail (M59, S16 `I6.b`).
- **Docs:** `docs/judge.md` (Go scope, the S22 decision, the S14 difference), milestone table, `pekko-actor/README.md`, `README.md`, `docs/actor-core-parity.md`, ADR 0017.
- **Surfaces:** only the seven approved plus the coordinator's list; the runtime (`stop.go`, `cell.go`, `hierarchy.go`, the rest of `pekko-actor/actor/`), `judge/comparator.py` and the Pekko side are untouched.

## Gates (local, by the coordinator, on the final code)
gofmt, vet clean; pinned `golangci-lint` v2.14.0 0 issues (the installed 2.13.1 fails to load packages); `go test -count=1 ./...` passes; `judge/run-go.sh <out> 300` exit 0 (14 scenarios, 300 concurrent runs with `-race`, controlled once, S13/S14/S16 300 of 300); `judge/selftest.py` 190 checks, 0 failed; `selftest_go_scope.py` and `selftest_mutate_go.py` ok; `scripts/check-judge-matrix.py` exit 0.

## Deviation from RULEBOOK 6 and D2 (S14), with reason and evidence
RULEBOOK 6 gave S14 a failing `PreStart` and a two-way witness (`terminated` or `pre-start(child, i=2)`). The first campaign (all 18 mutants over the new scope) showed that it cannot hold for every existing mutant:
1. **GMnostart** ("never call PreStart") became a `harness-error` for the whole mutant: the child never fails its start, stays alive and silent, and the controlled run is `quiescent before its completion witness`.
2. The Owner approved a **probe message** to the child as a third witness. It fixed GMnostart (met) but made **GM7** ("keep processing queued messages after a stop is accepted") a deterministic `harness-error`: a message queued at a failing, suspended child cannot be finalized under GM7, so no `Terminated` arrives (reproduced in isolation, every run).
3. The Owner then approved the **factory failure**: the first instance fails in the factory (a panic, an `InitializationError` decided by the parent, same default decision as a failing `PreStart`) and no message is sent. Evidence: `run-go.sh` 300 runs exit 0, S14 `terminated(w, child)` in 301 of 301 runs, no `pre-restart`, no second instance; GMnostart, GM7 and GMinitrestart are `met`. The cost is that the Go judge no longer exercises a failing `PreStart`; the Go unit tests do: `TestDefaultStrategyInitializationStops` (T4 of U3), `TestSup1Initialization`, `TestSup4PreStartFailuresUnderAFiniteBudget`. The Pekko body is unchanged (`preStart` throws). The probe commit stays in the history and is superseded.

## Mutants: the three campaigns, kept apart
Reports of every campaign are kept outside the repository (`tmp/u8run/`); only the last one is committed.
| Run | Result | Counted |
|---|---|---|
| early signal of the 4 new mutants, 30 runs, no report | all `met` | signal only |
| campaign 1 (probe absent, PreStart failing), 18 mutants | 17 met, **GMnostart `harness-error`** | invalid result |
| campaign 2 (probe), 18 mutants | GMnostart met; **GM7 `harness-error`** (deterministic, the probe); **GMoverlap `harness-error`** (3 data races in `cell.go`) | invalid result |
| isolated reruns on the branch: GM7 once, GMoverlap four times | GM7 unmet; GMoverlap 3 of 4 met | evidence |
| isolated reruns on the BASE `d5b831b`: GMoverlap six times | 5 of 6 met | evidence: the flake predates U8 |
| campaign 3 (factory failure), 18 mutants, commit `e543c6a`, tree `1f9a486` | **`unmet: none`**, 432 rows | the committed report |
Campaign 3: the 4 new mutants `failed` with their designed clause in both modes; the 13 existing mutants keep their designed detections; **GMbenign survives in all 24 cells**; no cell `timeout` or `harness-error`. Timeouts and harness errors were never counted as detections.

## GMoverlap is intermittent (investigation requested by the Owner)
GMoverlap overlaps two activations of one actor on purpose (clause `G3.c` of S21). Run with `-race`, the overlap exposes unsynchronized accesses **inside the runtime**: in the failed campaign the three data races were between the activation that writes `c.actor` at start (`cell.go:232`) and an overlapped `Receive` that reads it without the lock (`cell.go:269-270`), plus neighboring `ctx` and `failedMsg` accesses (`cell.go:279-294`). When the detector sees one the whole `go test` fails, so the mutant is a `harness-error` although `G3.c` would fire on the logs; when it does not, the mutant is `met`. Observed: failed 1 of 3 campaigns (campaign 2), 1 of 4 isolated runs on the branch, 1 of 6 on the base code. The incidental races are therefore enough to invalidate the run, and removing them needs a runtime change (`cell.go` carries judge anchors and is outside the surfaces of U8) or a change to how the campaign treats a race report; neither was done. **No claim that all mutants are stable is made:** the committed report is a valid complete campaign in which GMoverlap happened to be met.

## Matrix: design and observed, separately (`scripts/check-judge-matrix.py`)
| Line | Before U8 | After |
|---|---|---|
| `OK:` | 38 relations, 65 mutations, 41 scenarios | unchanged |
| designed coverage (Pekko) | 35/35 | 35/35 |
| observed on Pekko / relations with every clause | 30/31, 13/14 | unchanged (U8 does not touch the Pekko side) |
| Go design | 13 mutations with designed detections + 1 benign, 11 scenarios in the Go gate | **17** mutations + 1 benign, **14** scenarios |
| Go HISTORICAL (report, darwin/arm64 only) | 252 rows | **432 rows**: detected 17/17 designed, benign survived 1/1, 0 invalid, 0 incomplete |
| Go CURRENT (same Go core tree, comparator, scope, specs and toolchain as HEAD) | detected 0/13 (the old report no longer matched) | detected **17/17**, benign survived 1/1 |
There is no Linux Go report. Nothing was restored by weakening a rule, mutant or scenario.

## Review (the one focused review, Opus, `run-agent.sh review`; USD cap 3, 12 minutes)
14 turns, USD 0.63, 141 s; **no BLOCKER, one DEFECT, four INFO**. Prompt: the approved `review-prompt.md` (hash unchanged) plus an addendum kept outside the repository (the new base `d5b831b`, what the coordinator had verified, priorities). Judged sound: scope and no weakening, instance numbers, the supervisor, the four mutant anchors and cells, the S14 two-way witness, S16, races of the new hooks, the scenario table, the milestone.
| Finding | Class | Handling |
|---|---|---|
| `docs/judge.md` said `G7.b` "does not pass for lack of events"; its `pre-restart` half cannot fire for a child whose start failed (`faultCreate` runs no `PreRestart`, on either side); detection is by the missing `terminated` | DEFECT D-1 | fixed in `docs/judge.md` (`d14d7d9`); RULEBOOK 2 (hash-approved) keeps the old wording, corrected here |
| `fin` of S13 is sent as from `c`, not `main` | INFO 1 | recorded, not acted on |
| the S13 witness cannot catch a `Terminated` sent after `R`'s `received-end` (D5) | INFO 2 | recorded |
| GMinitrestart relies on operator precedence | INFO 3 | recorded |
| the custom `PreRestart` calls `PostStop` without the runtime guard (the reason S22 stays out) | INFO 4 | recorded |
**Not reviewed by the reviewer:** the `pekko-actor/README.md` diff and `Judge.scala`. **The fixes after the review were not re-reviewed** (the review is single by design): the S14 probe and its replacement by the factory failure, and the docs. They were checked by `run-go.sh` with 300 runs, by the three campaigns and by the self-tests. No second review was requested.

## Token accounting
| Item | Tokens |
|---|---|
| Implementer (`sonnet`, 71 turns, USD 1.01, 316 s): input 1,482 + output 34,495 + cache creation 106,352 | 142,329 |
| Review (`opus`, 14 turns, USD 0.63, 141 s): input 1,750 + output 10,060 + cache creation 43,846 | 55,656 |
| Measured agents | 197,985 |
| Reserve by convention (coordination 90,000, mutants and campaign 40,000, judge gates 30,000) | 160,000 |
| Correction round: the S14 probe (authorized 30,000) | 30,000 |
| Correction round: the S14 factory failure (authorized 35,000) | 35,000 |
| **Estimated total** | **422,985** |
Margin against the 560,000 ceiling: 137,015; the evaluation stop (450,000) was not crossed. Only the two agents are measured; the rest is the declared convention. The review gate (implementer at most 250,000) was met at 142,329. The Go campaigns are script time, not tokens or agent time.

## #175 acceptance, as observed
1. Events and scenarios implemented, documented in the header of `judge_internal_test.go`, stale text removed: **observed**.
2. Directed mutants GMrestartnotify, GMinitrestart, GMwatchoverwrite, GMwatchsamefail met, valid detections only: **observed** (campaign 3, early signal).
3. S13, S14, S16 in `go-scope.json` and passing: `run-go.sh` exit 0 with 300 concurrent runs: **observed**.
4. The 14 existing mutants still judged and valid over the whole new scope: **observed in campaign 3**, after the S14 change; **with the caveat that GMoverlap is intermittent** (above), which predates U8.
5. Docs corrected, the S22 decision recorded: **observed**.
#175 stays open until the Owner verifies its closure.

## Limitations
The Go campaign is darwin/arm64 only. S13, S14 and S16 are not calibrated (no Go row in `calibrate.py`). Pekko mutations without a Go analog: M10, M11, M58 (S13 `G6.d`) and M13 (S14 `G7.b`). S22 is Pekko only (RULEBOOK 7). The Go S14 does not exercise a failing `PreStart`. No parity is claimed.

## Not done
U9, #450, the module-close campaigns of #461, native review (not requested), a second review (the lock exists).

## Post-merge verification (status of the evidence, no new run)
- **The matrix check failed on `develop` until the evidence tag existed** (`scripts/check-judge-matrix.py`, exit 1, 432 rows): `go_commit e543c6a2e1 is neither an ancestor of HEAD nor a retained evidence commit`. The squash removed the campaign commit `e543c6a` from the history of `develop`, and the remote branch `feat/actor-judge-go-u8` was deleted, so the commit is no longer on any remote ref (it exists in the coordinator's local clone). The data are consistent: the report's tree `1f9a486` equals `develop:pekko-actor`, the self-tests are 190/0 and the Go tests pass.
- **What retains it in this repository:** a tag `evidence/go-campaign-<sha>`, as for the earlier Go campaigns (for example `evidence/go-campaign-2427544`). With a temporary local tag on `e543c6a` the check passed (exit 0, Go CURRENT 17/17 designed detections, benign survived). The Owner authorized it after the report was verified against that commit (432 rows with that exact `go_commit`, the same tree `1f9a486`, unchanged campaign inputs): the tag `evidence/go-campaign-e543c6a` (lightweight, pointing at `e543c6a2e1bdbff73cca4a202a41c3f2223aa37f`) was created and pushed without force, and the remote target was read with `git ls-remote`. The matrix check on `develop` (`3054971`) now passes: exit 0, 0 FAIL, Go HISTORICAL and CURRENT 17/17 designed detections, benign survived 1/1, the Pekko lines unchanged (30/31 clauses, 13/14 relations). #175 was then closed as completed, with the caveats of this record.
- **CI:** the heads of the PRs were green (`go`, `label`) and their trees equal the squash commits; the merge commits themselves have no check runs (the CI of this repository runs on pull requests).
