# Task: implement delivery U7 (a Stop during the wait of a restart, and the scope of G4/G5; #449)

You are the single implementer of this delivery: Go code, Go tests and the judge files, one session. You run non-interactively: nobody can answer questions. This is **U7 only**. U1 to U6 are merged in `develop` (`e0d70bb`) and are your base. Do NOT touch U8 (the Go judge gate for S13/S14/S16), U9 (logging), #450, `go-scope.json`, `judge/mutate*.py`, calibration files or the Go judge runner.

## Read first, in this order (do not re-plan or re-map)
1. migration/stage-1/supervision-u7/RULEBOOK.md (approved, read-only): all of it. Sections 2 (the two PostStop calls and Terminated), 3 (counts), 4 (R1-R9), 5 (the Go change), 6 (judge) and 7 (tests) bind you.
2. migration/stage-1/supervision-u7/RUNBOOK.md: sections 1 and 6 (surfaces).
3. migration/stage-1/supervision/h4-repro/RESULTS.md and H4Repro.scala: the counts and order you mirror.
4. Pekko, read-only, in `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/`: `Actor.scala` 570-631; `dungeon/FaultHandling.scala` 92-118, 180-212, 247-276, 327-351; `dungeon/Children.scala` 183-189.
5. The code: supervision_restart.go (`faultRecreate`, `defaultPreRestart`, `postStopOfRestart`, `installNew`, `awaitChildren`, `continueRestart`), stop.go (`finalize`, `beginPostStop`, `callPostStop`: read only), cell.go (`postStopBegun`, read only).
6. The tests to reuse: supervision_restart_goside_internal_test.go (`sup2NewWait`, `sup2DefaultP`, `sup2CustomP`, `sup2WatchCtl`, `TestSup2PostStopGuard`, `TestSup2CustomPostStopTwice`), stop_goexit_internal_test.go.
7. The judge: docs/judge.md rows G4 and G5 (lines about 68-69); judge/comparator.py `r_G5c`, `r_G5d` and `r_G4a` (read only), `r_I2a`, `PARAMS`, `RULES`, `bad_spec`, `evaluate`, and how `required` counts are read (minimums, never below 1); judge/selftest.py (helpers `case`, `mk`, `ps`, `st`, `ev`, `term`, `tmo`, `run`, and the prefix check near line 248); judge/scenarios/S10.json (a template); judge/pekko-harness/Judge.scala scenarios S10/S11 and the latch scenarios near lines 298, 428, 447 and the event names at 113-123; migration/stage-1/milestone-1.md (scenario table and the rows for G4 and G5).

## Allowed edit surfaces (the only files you may write)
Go, under pekko-actor/actor/ (3):
- supervision_restart.go (production: `defaultPreRestart` stops using `beginPostStop`; its comment and the comment of `postStopOfRestart`)
- supervision_restart_goside_internal_test.go (flip the second subtest of `TestSup2PostStopGuard` to W11b: `[p.pre#1, p.post#1, p.post#1]`; rewrite its header comment and the title; keep W11a and W25 unchanged)
- supervision_stoprestart_internal_test.go (new: T1-T4 of RULEBOOK 7.1, with the event order of RESULTS.md)
Judge and docs (6):
- docs/judge.md (G4 and G5 rows, the exact text of RULEBOOK 6.1)
- judge/comparator.py (`r_G5c` compares the parent's last `post-stop-begin` with each child's last `post-stop`; the new clause `G5.f` of RULEBOOK 6.2, registered in `PARAMS` and `RULES`; `r_G5d` and every other rule are NOT edited)
- judge/selftest.py (the new cases J1-J12 of RULEBOOK 7.4; the existing cases are not edited)
- judge/scenarios/S22.json (new)
- judge/pekko-harness/Judge.scala (scenario S22, declared `Set("concurrent")` because a latch in `controlled` mode would block the driver thread; no other change)
- migration/stage-1/milestone-1.md (register S22 in the scenario table and in the matrix rows of G4 and G5)

Everything else is denied, including stop.go, cell.go, hierarchy.go, guardian.go, judge/go-scope.json, judge/mutate.py, judge/mutate_go.py, calibration files, scripts/, other supervision*.go, docs/ADR files, READMEs and the RULEBOOK. **If you need a file outside the surfaces, stop and report `BLOCKED(surface): <file and why>`.** In particular, if `python3 -I scripts/check-judge-matrix.py` demands a change in another file for S22, report the exact message and stop.

## What to deliver
- The Go change of RULEBOOK 5 and nothing else in production. Keep functions at cyclomatic complexity 10 or less. Comments in English, neutral register, no parity claim.
- The judge changes of RULEBOOK 6: exactly one new clause (`G5.f`), exact on a complete log (zero allowed) and only "exceeded" on a prefix; S22.json with `required` for presence, `G5.f`, `I2.a`, `G5.c`, `modes: ["concurrent"]`, and neither `G4.a` nor `G5.d`. Do not add any other clause or rule.
- The cases J1-J12 of RULEBOOK 7.4 in `judge/selftest.py`, written **first**: run `python3 -I judge/selftest.py`, see the new cases fail on the base comparator (`G5.f` unknown; `G5.c` with `pb[0]` and `ce[0]`), then make the comparator change and see them pass. Report both runs with their check counts. The coordinator will run three judge mutants (`pb[0]` back, `ce[0]` back, `G5.f` as a minimum): write cases that fail under each.
- Go tests T1-T4 and W11b exactly as RULEBOOK 7.1. T1 must pin the order of RESULTS.md (P1 before the held child's final stop, P2 after it, `Terminated` after P2, `Done` after `Terminated`), the factory call count (no second instance) and the absence of `PostRestart`. Deterministic: the controlled executor and the held child of `sup2NewWait`, no sleeps. The coordinator will run three mutants against your result (a second instance after the Stop, a duplicated `Terminated`, the per-instance guard back in `defaultPreRestart`): write assertions that would fail under each.

## Hard constraints
- You CANNOT run `go test` or `sbt` (denied). Your loop is `gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...` (compiles the tests), `go build ./pekko-actor/...`, and the four python checks you are allowed: `python3 -I scripts/check-judge-matrix.py`, `python3 -I judge/selftest.py`, `python3 -I judge/selftest_go_scope.py`, `python3 -I judge/selftest_mutate_go.py`. Run the matrix check and the self-tests **before** you change the judge (note the baseline lines: `OK: ...`, the designed-coverage line, `relations with every clause observed: 14/14 (Pekko)`, `152 checks, 0 failed`) and after. The `OK:` line, the designed-coverage line and the self-tests must not get worse: if one does, revert that edit and report it with the evidence. The observed line may change because `G5.f` has no Pekko mutation campaign; report both values verbatim and do NOT remove or weaken any clause, mutation or scenario to restore it.
- No git write commands, no commits.
- Do not weaken the invariants of U1-U6 or the existing judge scenarios. If one concrete decision is unresolved and not decidable from the RULEBOOK or Pekko, report `NEEDS-DECISION: <the single question, the options, your recommendation>`.
- Time bound: 30 minutes wall. About 400 authored changed lines per task is a heuristic, not a cap. The delivery is capped at 560,000 tokens including coordination: read only the files listed above.
- Written artifacts are in English. No parity claim in comments.

## Final report (your last message), at most 1,200 tokens
1. Files changed (path and one line each). 2. gofmt, go vet, go build, the matrix check and the three self-tests: baseline and after. 3. Per test id (W11b, T1-T4): the Go test name(s). 4. Confirmation that no file outside the surfaces changed and stop.go, cell.go, hierarchy.go are untouched. 5. Open doubts and what could not be verified without running tests or sbt (the S22 run and the Go tests are run by the coordinator). 6. If blocked: the BLOCKED or NEEDS-DECISION line first.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
