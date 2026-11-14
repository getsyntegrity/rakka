# Task: implement delivery U8 (judge scenarios S13, S14 and S16 in the Go gate; #175)

You are the single implementer of this delivery: Go judge harness, python judge files and a few docs, one session. You run non-interactively: nobody can answer questions. This is **U8 only**. U1 to U7 are merged in `develop` (`b3275d7`) and are your base. Do NOT touch the Pekko side (`judge/pekko-harness/`, `judge/scenarios/*.json`, `judge/mutate.py`, `judge/comparator.py`), the runtime (`pekko-actor/actor/*.go` other than `judge_internal_test.go`), calibration files or `judge/go-mutation-report.tsv` (the coordinator regenerates it).

## Read first, in this order (do not re-plan or re-map)
1. migration/stage-1/supervision-u8/RULEBOOK.md (approved, read-only): all of it. Sections 2 (the contract), 3 and 4 (Go facts), 5 (D1-D4), 6 (bodies and witnesses), 7 (S22 is NOT added), 8 (mutants) and 9 (registration) bind you.
2. migration/stage-1/supervision-u8/RUNBOOK.md: sections 1 and 6 (surfaces).
3. The Pekko side, read-only: judge/pekko-harness/Judge.scala lines 94-145 (Rec, PostStopGate, Node) and 386-420 (S13, S14, S16); judge/scenarios/S13.json, S14.json, S16.json; judge/comparator.py rules `r_G6none`, `r_G7b`, `r_I6a`, `r_I6b`, `r_G4a`.
4. The Go judge: pekko-actor/actor/judge_internal_test.go, all of it (recorder, `judgeNode`, `judgeEnv`, S18 and S10 as templates, `judgeScenarios`, `runJudge`).
5. The Go runtime pieces you mirror, read-only: supervision_hooks.go (`PreRestarter`, `PostRestarter`, `FailedMessage`), supervision.go (`Supervisor`, `Strategy`, `OneForOne`, `Decider`), supervision_decider.go (`defaultDecider`), watch.go lines 75-125 and 150-205 (`Watch`, `WatchWith`, `sameMessage`, `addRegistration`), errors.go (`ErrWatchConflict`, `InitializationError`).
6. judge/mutate_go.py: the MUTATIONS table structure (lines 52-106) and how an edit is `(file, anchor, replacement)`; judge/go_scope.py; judge/selftest_mutate_go.py and judge/selftest_go_scope.py (do they hardcode the scenario or mutation counts?).

## Allowed edit surfaces (the only files you may write)
- pekko-actor/actor/judge_internal_test.go (instance counter, `PreRestart`/`PostRestart`, failing first start, supervisor/`decided`, the comparable watcher type of D4, bodies judgeS13/judgeS14/judgeS16, entries in `judgeScenarios`, the header comment: remove the stale "no restart / no supervisor" text and say what the Go judge runs now)
- judge/go-scope.json (add "S13", "S14", "S16" to `scenarios`; nothing else)
- judge/mutate_go.py (four new rows in MUTATIONS: GMrestartnotify, GMinitrestart, GMwatchoverwrite, GMwatchsamefail, RULEBOOK 8; each anchor must occur exactly once in its production file; verify with `rg -c`)
- judge/selftest_mutate_go.py and judge/selftest_go_scope.py (ONLY if they hardcode the old scenario or mutation lists; then update them to the new values and say so)
- docs/judge.md (the Go-scope section near line 268: S13, S14, S16 are in the Go gate; S22 is Pekko only and why, RULEBOOK 7)
- migration/stage-1/milestone-1.md (the Go side of the S13, S14, S16 rows; the prose that lists the Go-registered scenarios and the Go observed count; S22 stays "Pekko only" with the reason)

Everything else is denied. **If you need a file outside the surfaces, stop and report `BLOCKED(surface): <file and why>`.** If `python3 -I scripts/check-judge-matrix.py` demands something in another file, report its exact message and stop.

## What to deliver
- RULEBOOK 5 and 6 exactly: the events (`pre-restart {i,m}`, `post-restart {i}`, `decided {subj,k,v}`), instance numbers, the failing first start, the supervisor, the comparable watcher type, and the three bodies with their completion witnesses (S13: a message to `w` after `R`; S14: two-way witness, `terminated` OR `pre-start(child, i=2)`; S16: `ack` at `w1` (after the decision) and at `w2`). The Pekko specs are the contract: your logs must satisfy `S13.json`, `S14.json` and `S16.json` as they are.
- Do not change the logs of the existing scenarios: they have one instance (`i: 1`) and no restart.
- The four mutants of RULEBOOK 8, each with the Pekko analog and the designed `(scenario, mode, clause)` cells.
- Docs as listed. No parity claim. English, neutral register, no `//nolint`, gocyclo at most 10 per function including literals.

## How to verify (you may run exactly these)
`gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...`, `go build ./pekko-actor/...`, the four python checks (`scripts/check-judge-matrix.py`, `judge/selftest.py`, `judge/selftest_go_scope.py`, `judge/selftest_mutate_go.py`), the two `go_scope.py --list` commands, and **one scoped run of the Go judge**: `bash judge/run-go.sh /tmp/gopekko-u8-judge-out 1` (D7: one controlled run and one concurrent run per scenario, with `-race`; it prints the verdict per scenario). Run the baseline of the python checks and note the lines BEFORE you change anything; run the judge once BEFORE your changes only if you need to, and iterate with it AFTER. `go test` and `sbt` are otherwise denied. You cannot run `mutate_go.py` (a campaign); the coordinator does. Expect `run-go.sh` to fail until all three scenarios are right: use its output to correct them.

## Hard constraints
- No git write commands, no commits.
- Do not weaken the existing scenarios, mutants or clauses. If a concrete decision is unresolved and not decidable from the RULEBOOK, the Pekko sources or the Go code, report `NEEDS-DECISION: <the single question, the options, your recommendation>`. If a Pekko behavior cannot be reproduced in Go without changing the runtime, report `BLOCKED(runtime)` with the evidence; do not alter the spec or skip a clause.
- The matrix check, `selftest.py`, and the self-tests must not get worse; the observed Go line changes only through the coordinator's campaign.
- Time bound: 30 minutes wall. About 400 authored changed lines per task is a heuristic, not a cap. The delivery is capped at 560,000 tokens including coordination: read only the files listed above.

## Final report (your last message), at most 1,200 tokens
1. Files changed (path and one line each). 2. gofmt, vet, build, the matrix check and the three self-tests: baseline and after. 3. The result of `run-go.sh ... 1` per scenario (S13, S14, S16 and the existing ones). 4. The exact anchors of the four mutants and the cells designed. 5. Confirmation that no file outside the surfaces changed. 6. Open doubts, anything unverified (the mutants and the 300-run campaign are the coordinator's). 7. If blocked: the BLOCKED or NEEDS-DECISION line first.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
