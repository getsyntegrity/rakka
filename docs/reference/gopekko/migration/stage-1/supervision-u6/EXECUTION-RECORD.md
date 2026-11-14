# U6 execution record (#447)

Scope: U6 only (`AllForOne`). This is not a claim of Pekko parity, and it does not close #447. Package: PR #469 (squash `56a95aa`; head `3efe963`, eight hashes verified against the committed files before the merge and again on the code branch). Code: branch `feat/actor-allforone-u6`, commits `43ac197` (production and W4), `37383e0` (tests), `21b65a9` (documentation) and `494831f` (AF15 test delta after the review). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.

## Delivered
- `AllForOne[T](Budget, Decider[T]) Strategy[T]`, the only new exported identifier. Rules R1-R11 of the RULEBOOK; the children are captured once, with their restart statistics, before the decider runs (`snapshotRound`, `supervision_restart.go`); permissions are asked in ascending name order with short-circuit; a denial stops every captured child with one `restart-denied` naming the denier; Stop stops all; Resume and Escalate are as in `OneForOne`.
- Tests AF1-AF6 (from `SupervisorSpec:292,318,323,354,364,401`), AF7 (W4, `SupervisorHierarchySpec:767-788`, the gap wording removed), AF8-AF18 and `ExampleAllForOne`.
- Files changed: exactly the six surfaces of RUNBOOK section 6 (`supervision.go`, `supervision_failure.go`, `supervision_restart.go`, the two new test files, `supervision_restart_internal_test.go`); `cell.go`, `hierarchy.go`, `stop.go` and `guardian.go` are byte-identical to the baseline. Documentation by the coordinator: this record, `pekko-actor/README.md`, `README.md`, `actor/doc.go`, ADR 0017, `docs/actor-core-parity.md`.

## Gates (local, by the coordinator)
gofmt, vet, `go mod tidy -diff` clean; `go test -count=1 ./...` passes; **pinned `golangci-lint` v2.14.0: 0 issues** (the installed 2.13.1 failed to load packages, so the pinned binary was used); coverage of `pekko-actor` **98.07%** (1,245 statements, threshold 85); deps, headers, ADR signatures, modules selftest, SDK check and judge selftest ok; `go test -race -count=1 ./...` passes; the AF tests, `TestSup2HierarchyEscalation` and the Example **30 times under `-race`** pass. The implementer could not run `go test` (denied) and reasoned the expectations; the AF14b suspension counts and the AF15 cases it derived held on the first run (the AF15 cases were later strengthened, see below).

## Correction rounds (coordinator, no agents)
1. **AF13** asserted a counter of 1 for the restarted child under `Budget{-1,-1}`; Pekko's `None` case allows without counting (`FaultHandling.scala:58-64`), so the counter is 0 and the siblings have no entry (-1). Expectation fixed; no production change.
2. **AF15, after the review** (commit `494831f`, `supervision_allforone_internal_test.go` only, +83/-5). The review found three DEFECTs in the AF15 tests (below). Fixes: the completion signal of the member that ends inside the decider is captured with `options.completionPoint` and delivered by hand **inside the decider, before the permission phase**, so its statistics entry is gone when the round asks; AF15a and AF15b assert that the entry is gone before the round and that it does not come back (`statsOf(b) == nil`); the "stop requested before the snapshot" case reads b's suspension while it is held in `PostStop` (1) and, after release, the suspension of a, c and the parent (0); AF15a asserts the suspensions of a, b and the parent (0); AF15a, AF15b and the stop-requested case restart the parent afterwards (`afFailBoss`) and check that its default `PreRestart` completes, the new instance answers and no statistics entry, stop record or waiter is left. One adjustment of my own expectation: the failing child c keeps the +1 of its own failure after a denial (it is stopped without a Resume), so AF15a asserts `[0,0,1,0]` for a, b, c, p. This is not a production problem.

## Review (the one focused review, Opus, `run-agent.sh review`, USD cap 3, 12 minutes; 17 turns, USD 1.11, 127 s; no BLOCKER)
Prompt: the approved `review-prompt.md` plus an addendum with the Owner's priorities (kept outside the repository; the approved file and its hash are unchanged). Production code judged correct: exported surface, snapshot before the decider and once, stopping and finished children, short-circuit (R6, R7), suspensions (R11), Stop and denial, Resume/Escalate/panic/Goexit, logging and the `PostStop` guard untouched; AF14b judged sound; AF12, AF13, AF8, AF9, AF9b confirmed.
| Finding | Class | Handling |
|---|---|---|
| AF15 "no stats entry re-created" can pass without proving it: `close(done)` precedes the owner's `forgetChildLocked` (`stop.go:109-110`, `hierarchy.go:230`), so the round usually ran while b's entry still existed | DEFECT 1 | fixed in `494831f` with `completionPoint` |
| "a later restart of P is not blocked" not asserted in AF15 | DEFECT 2 | fixed: AF15a, AF15b and the stop-requested case restart P |
| "no suspension or hold left" only partly asserted (AF15a no P; stop-requested case never read b) | DEFECT 3 | fixed (see above). A "hold" has no accessor in the tests: it is covered only through the suspension counts and the completion of P's restart |
| `snapshotRound` creates zero-count entries for every child on any handled failure | INFO 1 | accepted by RULEBOOK 4; AF11 and AF12 rely on it |
| RULEBOOK 9.3 lists AF7 as a MU1 witness; it is not | INFO 2 | recorded: AF7's path is Escalate. Valid MU1 witnesses: AF4, AF5. AF6 not run as a witness |
| MU2 kills come from a 30 s wait inside the tests | INFO 3 | recorded: valid only with the 900 s rerun |
**Review incomplete, not approved in those parts.** The reviewer's shell was denied mid-run, so it did not read the AF7 hunk, AF16-AF18 or `ExampleAllForOne`. The coordinator then read them (no agent): AF7 is a faithful port whose decider is empty, so the strategy's own restart rule is not exercised there (the comment says so); AF16 (four cases), AF17 (panic and Goexit) and AF18 (permitted, denied with `a` as the denier) assert what they claim; the Example waits on channels (a regression would show as a hang, as INFO-5 of U5c). This is the coordinator's reading, not a second independent review.

## Mutants (RULEBOOK 9.3), on independent copies outside the repository
The unmodified copy passes. Valid detections and INVALID runs are kept apart:
| Run | Result | Detail |
|---|---|---|
| first run, 180 s package timeout | **INVALID** for MU1 and MU2 | the helpers wait 30 s before failing, so several failing tests exceed the package timeout; MU3 was a valid kill in it |
| MU1 restart only the failing child (witnesses, 900 s) | **KILLED** | `TestAllForOneRestartsAllChildren`, `TestAllForOneMultipleChildrenCallKillCall`. AF7 passes under MU1 and is **not** a witness; AF6 was not run as a witness |
| MU2 do not stop all on denial (witnesses, 900 s) | **KILLED** | `TestAllForOneBudgetZeroStopsTheChild`, `TestAllForOneCountersPerChild`, `TestAllForOneDenierFirstAndLast`; kills come from a 30 s wait inside the tests, valid only because the package timeout was wide |
| MU3 count only one child | **KILLED** | ten tests, among them `TestAllForOneCountersPerChild`, `TestAllForOneSiblingAtItsLimitDenies`, `TestAllForOneDenierFirstAndLast` |
| MU4 (diagnostic, not part of #447): `statsFor` per member after the snapshot (the regression RULEBOOK 4 warns about) | killed by **both** the old AF15 (`21b65a9`) and the new one | with the new tests: AF15a through a 30 s wait inside the test (the denial is lost because a fresh entry has count 1), AF15b by a direct assertion (the captured counter of b stays 0). With the old tests the kill was also real on this run, so the review's DEFECT 1 is a determinism and race-dependence concern, not a demonstrated surviving mutant |
The MU4 run with the new tests: package timeout 600 s, not reached.

## Gates after `494831f` (local)
gofmt, vet clean; pinned `golangci-lint` v2.14.0: 0 issues; `go test -count=1 ./...` and `go test -race -count=1 ./...` pass; the AF tests, `TestSup2HierarchyEscalation` and the Example 30 times under `-race` pass; coverage of `pekko-actor` 98.07% (1,245 statements); repository gates and judge selftest ok; `cell.go`, `hierarchy.go`, `stop.go`, `guardian.go` unchanged.

## Token accounting
| Item | Tokens |
|---|---|
| Implementer (`sonnet`, 51 turns, USD 1.78, 619 s): input 58 + output 86,544 + cache creation 166,289 | 252,891 |
| Review (`opus`, 17 turns, USD 1.11, 127 s): input 26 + output 10,713 + cache creation 101,006 | 111,745 |
| Measured agents | 364,636 |
| Reserve by convention (coordination 90,000, mutants 30,000) | 120,000 |
| Correction round 1 (AF13) | 30,000 |
| Correction round 2 (AF15 after the review, authorized by the Owner at +30,000) | 30,000 |
| **Estimated total** | **544,636** |
Margin against the 560,000 cap: **15,364**. The evaluation stop (450,000) was passed with the Owner's explicit authorization for the review. Only the two agents are measured; the rest is the declared convention. The documentation pass and the MU4 diagnostic are not counted as extra rounds.

## #447 acceptance, as observed
1. Kit `cf91c9d`, exact sources and tests, recorded: this record.
2. Tests derived from the listed upstream cases, finite budget per child: AF1-AF6, AF8, AF9, AF9b (pass).
3. Mutants: MU1, MU2, MU3 killed (valid detections, table above).
Not observed: a complete independent review of AF7, AF16-AF18 and the Example, the module-close campaigns (#461), parity. **#447 stays open** until the Owner verifies its closure.

## Not done
U7, U8, U9 (their contracts are untouched), the module-close campaigns of #461, `System.Shutdown`, user-defined strategies (#450), native review (not requested), a second review (the lock `review.lock` exists).
