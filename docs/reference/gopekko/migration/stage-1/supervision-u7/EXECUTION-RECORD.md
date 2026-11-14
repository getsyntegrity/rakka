# U7 execution record (#449)

Scope: U7 only (a Stop during the wait of a restart, and the scope of judge G4/G5). This is not a claim of Pekko parity, and it does not by itself close #449 (the Owner closed #449 as completed afterwards, with the evidence comment of the squash `b3275d7`; the caveats below stand). Package: PR #471 (squash `386a13a`; head `a7abe0f`; the eight approved hashes were verified against the committed files before the merge and again on the code branch). Code: branch `feat/actor-stop-restart-u7` (deleted after the merge), PR #472 merged by squash `b3275d7` (its tree `70fb6e9` equals that of the verified head `f1f61b6`), commits `c266b60` (production and Go tests), `98387bb` (judge and docs), `22b3e83` (review fixes) and the documentation commit. Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`).

## Delivered
- **Production (one function):** `defaultPreRestart` (`pekko-actor/actor/supervision_restart.go`) no longer goes through `beginPostStop`. The PostStop of the default `PreRestart` (P1) is user-level, unguarded code; `finalize` keeps the guard, now only against the re-entry after a Goexit inside the PostStop of termination (P2). With the default hooks a Stop during the wait gives two `PostStop` calls on the old instance, one `Terminated`, no new instance, no `PostRestart`. `stop.go`, `cell.go`, `hierarchy.go`, `guardian.go` are byte-identical to the baseline (judge anchors).
- **Go tests:** W11b (the old interim subtest, flipped to two calls), W11a (kept), W25 (kept), T1 (order of `RESULTS.md`: P1 before the held child's final stop, P2 after it, one `Terminated` after P2, factory called once, no `PostRestart`), T2 (custom `PreRestart` without `PostStop`: one call), T3 (plain Stop: one call), T4 (Goexit in P1, then Stop; Go-specific, no Pekko equivalent).
- **Judge:** `docs/judge.md` G4 and G5 rows (the Owner-accepted correction of ADR S6, "once per instance" was the judge's wording); `judge/comparator.py`: `r_G5c` compares the parent's last `post-stop-begin` with each child's last `post-stop`, and proves the order on a prefix only for a parent that was never restarted; new clause **`G5.f`** (exact hook counts of one actor, zero allowed, only "exceeded" on a prefix; `COUNTS` validator); `r_G5d` and every other rule untouched; scenario **S22** in `judge/pekko-harness/Judge.scala` and `judge/scenarios/S22.json` (concurrent only: a latch in `controlled` mode would block the driver thread); `migration/stage-1/milestone-1.md` registration. `judge/selftest.py`: cases J1-J13.
- **Surfaces:** exactly the nine of RUNBOOK section 6. Documentation by the coordinator: this record, `pekko-actor/README.md`, `README.md`, ADR 0017. `judge/go-scope.json`, `judge/mutate*.py` and calibration files are untouched; S22 is not in the Go gate (that is U8).

## Gates (local, by the coordinator, on the final code)
gofmt, vet, `go mod tidy -diff` clean; pinned `golangci-lint` v2.14.0 0 issues (the installed 2.13.1 fails to load packages); `go test -count=1 ./...` and `go test -race -count=1 ./...` pass; coverage of `pekko-actor` **98.07%** (1,244 statements, threshold 85); deps, headers, ADR signatures, modules selftest and SDK check ok; the three judge self-tests: `selftest.py` **190 checks, 0 failed** (152 at the base), `selftest_go_scope.py` ok, `selftest_mutate_go.py` ok; `scripts/check-judge-matrix.py` exit 0. `stop.go`, `cell.go`, `hierarchy.go`, `guardian.go` and the files the RUNBOOK names as untouched compared by sha256 with the baseline recorded before the run.

## RED / GREEN (W11b and T1)
GREEN with the change. RED on the base production code (the base `supervision_restart.go` with the new tests): `TestSup2PostStopGuard` W11b, `TestSupStopDuringRestart` T1 and `TestSupStopAfterGoexitInP1` T4 fail. This run is also MU3.

## Defects found on the way (all in tests or in the judge, none in the production change)
1. **T4 was non-deterministic** (coordinator, before the review): 13 failures in 40 runs. It delivered the held child's completion by hand before the child had really finished, so the parent could finalize first. Fixed in the test by waiting for the child's `Done` first; then 0 failures in 200 runs and 100 runs under `-race`.
2. **MU2 as designed ("notify the watchers twice") is equivalent** in Go: the watcher drops a second notification for the same subject (`watch.go:231-237`). It is not counted as a detection; it was replaced by MU2b (two sites: notify twice and remove the watcher's de-duplication).
3. **Review DEFECT 1:** `r_G5c` with the last `post-stop-begin` reported a stalled S22 run (cut after P1) as `fail [G5.c]` instead of `timeout`. Test first: the cases J13 failed on the comparator (2 failures, 190 checks), then the fix, then 0 failed.
4. **Review DEFECT 2:** the G4 row said "Restart", RULEBOOK 6.1 says "Restart that completes". Fixed.
5. **Review INFO 1:** the `postStopOfRestart` comment claimed there is never a second `PostStop`; corrected.

## Mutants and runs, valid detections apart from the rest
Copies outside the repository, one at a time, unmodified copy passes.
| Run | Result | Detail |
|---|---|---|
| BASE (Go) | passes | the witnesses pass |
| MU3 per-instance guard back in `defaultPreRestart` | **KILLED** (assertion failures) | W11b, T1, T4 |
| MU1 a second instance after the Stop (`terminate` continues the restart) | **KILLED** (assertion failures) | W11b, T1, T2, T4, W25 |
| MU2 notify the watchers twice | **NOT A DETECTION: equivalent mutant** | passes; the watcher de-duplicates |
| MU2b notify twice and drop the watcher's de-duplication | **KILLED** (assertion failures) | T1, T2, T3, W25 |
| BASE (judge) | passes | 190 checks, 0 failed |
| MJ1 `r_G5c` back to the parent's first begin | **KILLED** | J8, J12 |
| MJ2 `r_G5c` back to the child's first stop | **KILLED** | J10 |
| MJ3 `G5.f` as a minimum | **KILLED** | J3, J4, J5, J12 (J2 does not catch it, as RULEBOOK 7.4 says) |
| MJ4 `G5.c` without the prefix guard (added after the review) | **KILLED** | J13 |
No run was INVALID for compilation and none hit a timeout: the 30 s waits of the Go helpers did not matter here because the failures are assertions, not waits. Harness timeouts: none.

## S22 on Pekko (RULEBOOK 6.3)
Disposable export of `upstream/pekko` at `9a54d1294c` (`git archive`, a throwaway git repository tagged `v1.7.1`, as `h4-repro/run.sh` documents; `upstream/pekko` was not modified). `judge/run-pekko.sh <copy> <out> concurrent 10000 S22`, 3 runs, each log judged with `judge/comparator.py --spec judge/scenarios/S22.json --mode concurrent`: **3 of 3 `pass`, state `complete`**, again with the corrected comparator. Observed in every run, for `c`: `pre-start` 1, `pre-restart` 1, `post-restart` 0, `post-stop` 2, and `terminated(w)` 1. Order in every run: `pre-restart(c)`, `post-stop-begin(c)` and `post-stop(c)` (P1), the begins and ends of both children, `post-stop-begin(c)` and `post-stop(c)` (P2), `terminated(w)`; only the order of the two children varied. This is the order of `RESULTS.md`. S22 in controlled mode is a `harness-error` by design (J12).

## Matrix: design and observed, separately (`scripts/check-judge-matrix.py`)
| Line | Base `e0d70bb`/`386a13a` | After |
|---|---|---|
| `OK:` | 38 relations, 65 mutations, 40 scenarios | 38 relations, 65 mutations, **41** scenarios |
| designed coverage | 35/35 relations with a scenario and a mutation | 35/35 (unchanged) |
| observed on Pekko (mutation report, pinned) | 30/30 clauses, pending 0 | **30/31**, pending 1 (`G5.f`: no mutation is designed to break it) |
| relations with every clause observed | 14/14 | **13/14** |
The observed line dropped because `G5.f` is a new clause that no Pekko mutation campaign has yet exercised. It was **not** restored by removing or weakening any rule, mutation or scenario. The campaign (a mutation designed against `G5.f`, run on Pekko) is pending and recorded in #461. The historical Go report lines (`goPekko HISTORICAL/CURRENT`) are not touched or claimed.

## Review (the one focused review, Opus, `run-agent.sh review`; USD cap 3, 12 minutes)
17 turns, USD 0.71, 140 s; **no BLOCKER, two DEFECTs, four INFO**; it reported that nothing on its priority list was left unread. Prompt: the approved `review-prompt.md` (hash unchanged) plus an addendum kept outside the repository that gives the new base `386a13a`, what the coordinator had already verified and the review priorities. Production change judged correct: only `defaultPreRestart` changed, `finalize` still guards the Goexit re-entry, the `postStopBegun = false` reset in `installNew` is now inert (as RULEBOOK 5 allows). Comparator: `G5.f` is the only new clause, registered in `PARAMS` and `RULES`, only "exceeded" on a prefix; S22 cannot pass with a second instance, a third `post-stop`, a missing P2 or a duplicated `terminated`; `Judge.scala` requests the Stop only after P1 and both children's `post-stop-begin`.
| Finding | Class | Handling |
|---|---|---|
| `r_G5c` reported a stalled S22 run as `fail [G5.c]` | DEFECT 1 | fixed with test first (J13, MJ4) |
| G4 text differs from RULEBOOK 6.1 | DEFECT 2 | fixed |
| stale `postStopOfRestart` comment | INFO 1 | fixed |
| T1-T3 can only catch a duplicated `Terminated` that gets past the watcher's de-duplication | INFO 2 | recorded; that is the observable contract |
| J7, J9, J11 also pass on the base comparator | INFO 3 | recorded; none is vacuous (each fails if G5.c or I2.a is loosened) |
| T1 does not assert `Done` after `Terminated` | INFO 4 | recorded in #461: the order is fixed by `finalize` in `stop.go`, which is unchanged |
The fixes after the review (J13, the comparator guard, G4 text, comment) were not re-reviewed: the review is single by design (lock). They were checked by the self-tests, the mutants and the Go suite.

## Token accounting
| Item | Tokens |
|---|---|
| Implementer (`sonnet`, 74 turns, USD 1.47, 462 s): input 12,430 + output 45,142 + cache creation 138,328 | 195,900 |
| Review (`opus`, 17 turns, USD 0.71, 140 s): input 28 + output 11,991 + cache creation 47,343 | 59,362 |
| Measured agents | 255,262 |
| Reserve by convention (coordination 90,000, mutants 30,000, Pekko runs and judge gates 30,000) | 150,000 |
| Two correction rounds by convention (T4 and the review fixes), 30,000 each | 60,000 |
| **Estimated total** | **465,262** |
The review gate (implementer at most 200,000) was met at 195,900. The evaluation stop (450,000) was **crossed** by the convention-based part once the corrections after the review were counted; the ceiling (560,000) was not (margin **94,738**). Only the two agents are measured; the rest is the declared convention. The documentation pass is not counted as an extra round.

## #449 acceptance, as observed
1. Executed with kit `cf91c9d` and recorded: this record; hashes verified.
2. Go rule: the termination path calls `PostStop` once on the current instance; the call of the default `PreRestart` is unguarded user code (R1, R2, W11b, T1).
3. `docs/judge.md` G4/G5 corrected; S22 on Pekko (3 of 3 pass) and the Go mirror fix the counts and order of `RESULTS.md`; matrix design lines hold, the observed line is reported (13/14); self-tests green.
4. Mutants: second instance (MU1), duplicated `Terminated` (MU2b, MU2 was equivalent) and the per-instance guard (MU3) killed.
5. The U2 gap in ADR 0017 (`:466`, `:681`, D-5) is marked closed.
Observed, not concluded. Status after the merge: #449 was closed by the Owner as completed (squash `b3275d7`), with the limitations below.

## Limitations
S22 exists on Pekko only and in `concurrent` mode; the Go side has no restart events in the judge (U8, #175). Only the default-hooks mode has a judge scenario (RULEBOOK D2). `G5.f` has no Pekko mutation. T4 is Go-specific (RULEBOOK D4). The review did not cover the fixes made after it. No parity is claimed.

## Not done
U8, U9, #450, `System.Shutdown`, the module-close campaigns of #461, native review (not requested), a second review (the lock exists).

## Post-merge verification (status of the evidence, no new run)
- **In the repository:** the production change, the Go tests W11b and T1-T4, `G5.f`, `r_G5c` final-versus-final, scenario S22 and the self-test cases J1-J13, all on `develop`.
- **Only in this record (not committed as artifacts):** the three S22 runs on Pekko v1.7.1 (raw logs), the Go mutants MU1, MU2b and MU3 and the judge mutants MJ1-MJ4 (ad hoc scripts on copies outside the repository), and the RED run. They can be reproduced from the description (`judge/run-pekko.sh` with a disposable export of the pinned Pekko), not re-read.
- **Not covered:** a Pekko mutation against `G5.f` (the observed-clauses line stays 13/14), a re-review of the fixes made after the single review, and S22 in the Go gate (decided against in U8).
