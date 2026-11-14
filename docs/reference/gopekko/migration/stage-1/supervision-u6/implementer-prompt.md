# Task: implement delivery U6 (AllForOne, #447) in pekko-actor/actor

You are the single implementer of this delivery: production code and tests, one session. You run non-interactively: nobody can answer questions during the run. This is **U6 only**. U1 to U5 are merged in `develop` (`66ee197`) and are your base. Do NOT touch U7 (stop during a restart, the `PostStop` guard), U8 (judge S13/S14/S16), U9 (logging), user-defined strategies (#450), the guardians' strategy or persistence.

## Read first, in this order (do not re-plan or re-map)
1. migration/stage-1/supervision-u6/RULEBOOK.md (approved, read-only): all of it. Sections 1 (upstream evidence), 2 (API), 3 (R1-R11 and R3a), 4 (D1, D2), 5 (D3, order), 9 (tests AF1-AF18, AFX) bind you.
2. migration/stage-1/supervision-u6/RUNBOOK.md: sections 1 and 6 (surfaces).
3. migration/stage-1/supervision-u2/RULEBOOK.md section 4: conventions (go-specs tests only, headers on ported files, `errors.Is/As`, no `//nolint`, gocyclo at most 10 per function including literals, coverage at least 85%).
4. Pekko, read-only, in `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/`: `FaultHandling.scala` 49-89, 366-390, 435-452, 468-568; `dungeon/FaultHandling.scala` 92-153, 215-245, 278-325; `dungeon/ChildrenContainer.scala` 134-139, 196-202. Tests in `upstream/pekko/actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala` 144-230, 292-410 and `SupervisorHierarchySpec.scala` 767-788.
5. The code: supervision.go (`Strategy`, `OneForOne`, `kindOneForOne`), supervision_failure.go (`handleFailed`, `apply`, `failParent`, `suspendable`), supervision_restart.go (`applyRestart`, `recreate`), supervision_budget.go (`restartStats`, `requestPermission`), hierarchy.go (`survivors`, `childByID`, `statsFor`, `noteStopID`).
6. Test helpers to reuse before writing new ones: `sup1Rig`/`sup1NewRig`, `sup1Hooked`, `sup1InlineExec`, `sup2Clock`, `hookActor`, `fnActor`, `factoryOf`, `journal`, `settle`.

## Allowed edit surfaces (the only files you may write), all under pekko-actor/actor/
Production (3):
- supervision.go (`kindAllForOne`, `AllForOne`, its doc comment with the rules of RULEBOOK 3-5 including the order adaptation and the decider-spawn note)
- supervision_failure.go (`apply`: Stop for `kindAllForOne`; `handleFailed` only if the dispatch needs the strategy kind)
- supervision_restart.go (`applyRestart`: the all-children permission loop and application; the comment of `applyRestart`)

New tests (2, create them):
- supervision_allforone_internal_test.go (AF1-AF6, AF8-AF18)
- supervision_allforone_example_test.go (`package actor_test`: `ExampleAllForOne`, AFX, with an `// Output:` block)

Existing tests (1):
- supervision_restart_internal_test.go: AF7 only (the `PORT-GAP(AllForOne-U6)` header line near 29, and the W4 test at about 395-401: the manager becomes `AllForOne(Budget{MaxRetries: -1, Within: -1}, nil)`; remove the gap wording from the comment and the test title; keep the assertions).

Everything else is denied, including cell.go and hierarchy.go (judge anchors), stop.go, guardian.go, other supervision*.go, judge/, docs/, READMEs, ADRs, the RULEBOOK and doc.go. **If you need a file outside the surfaces, stop and report `BLOCKED(surface): <file and why>`.** In particular, if a sibling that ends between the snapshot and its restart request breaks the counters and the fix is in cell.go, report that with the failing scenario; do not edit cell.go.

## What to deliver (RULEBOOK 2 to 6)
- Exactly `func AllForOne[T any](budget Budget, decide Decider[T]) Strategy[T]`. No other exported identifier. `OneForOne`, `DefaultStrategy`, the zero `Strategy` and the guardian constructors keep their behavior byte for byte.
- R1-R11. Reuse the existing mechanisms: `statsFor(...).requestPermission(budget, nanotime)`, `suspend()` through `suspendable`, `recreate(cause)`, `noteStopID` + `acceptStop(nil)`, `emit("restart-denied", ...)`. Take **one** snapshot of the children **before the decider runs** (in `handleFailed`, after `strategy()` and before `st.decide`, only when the kind is `kindAllForOne`; `survivors`-style, ascending name, stopping children included) and use it for both phases; the snapshot records each member's record AND its `*restartStats` pointer in one critical section under `System.mu` (helper inside supervision_restart.go; do not call `statsFor` after the snapshot and do not edit hierarchy.go), and a member that ends before its turn still counts: its captured counter is evaluated and may deny the round (RULEBOOK 4). A child created by the decider is outside the round and survives it (D2). Evaluate permissions in that order with short-circuit; restart requests in the same order after all are granted; F is not suspended again, every other child is suspended first.
- A denial emits one `restart-denied` naming the denier, then stops every child of the snapshot. Stop stops every child. Resume and Escalate behave as `OneForOne`. When P fails (Escalate, ok == false, nil decider, decider panic or Goexit) AllForOne takes no action, but P's own failure suspends its children except F (RULEBOOK R3a): do not add code to prevent that and do not assert "siblings untouched". `FailedMessage` is whatever the cell holds (R8): do not set or clear it for siblings. The doc comment of `AllForOne` must state the order as "ascending name, as observed in Pekko v1.7.1, not a guaranteed contract".
- Keep functions at cyclomatic complexity 10 or less: split helpers instead of nesting.

## Tests (RULEBOOK 9; deterministic: channels and controlled executors, no sleeps as coordination)
Write AF1-AF6, AF8-AF18 and AFX exactly as RULEBOOK 9.1 and 9.2 describe, with the scenario of AF8 as written (children `a`, `b`, `c` named so the order is fixed). Each test's comment cites its upstream case or the RULEBOOK rule. The three mutants of RULEBOOK 9.3 will be run by the coordinator against your result: write assertions that would fail under each (counters of every child, which children stopped, which restarted), not just "something restarted".

## Judge anchors
None of your files carries a `judge/mutate_go.py` anchor. Do not edit cell.go or hierarchy.go.

## Hard constraints
- You CANNOT run `go test` (denied). Your loop is `gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...` (compiles the tests) and `go build ./pekko-actor/...`.
- No git write commands, no commits.
- Do not weaken the invariants of U1 to U5. If one concrete decision is unresolved and not decidable from the RULEBOOK or Pekko, report `NEEDS-DECISION: <the single question, the options, your recommendation>`.
- Time bound: 30 minutes wall. About 400 authored changed lines per task is a heuristic, not a cap. The delivery is capped at 560,000 tokens including coordination; do not read files outside the list above without need.
- Written artifacts (code, comments, test names) are in English, neutral register. No parity claim in comments.

## Final report (your last message)
1. Files changed (path and one line each). 2. gofmt -l, go vet, go build results. 3. Per AF test id: the Go test name(s). 4. Confirmation that the exported surface equals RULEBOOK 2, no file outside the surfaces changed, and cell.go/hierarchy.go are untouched. 5. Open doubts, anything unverifiable without running tests, whether `survivors` includes children whose stop was requested, and any contradiction of RULEBOOK R8 or of the AF14b counts (report `NEEDS-DECISION` with the observed counts; do not bend the assertion). 6. If blocked: the BLOCKED or NEEDS-DECISION line first.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
