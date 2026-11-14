# Task: implement T6 of ADR 0024 (release of the fields of a terminated cell; issue #563) in pekko-actor/actor

You are the single implementer of this delivery: production code and tests, one session. You run non-interactively: nobody can answer questions. Scope is **T6 only**. The retirement of actors from `System.byID` (T2-T5) is already integrated; do not touch it. The base is `develop` plus this package; `pekko-actor/` is the `develop` recorded in `PROPOSAL.md`.

## Read first, in this order (do not re-plan or re-map)
1. migration/stage-1/cell-release-563/RULEBOOK.md (approved, read-only): all of it. Sections 1 (goal and non-goals), 3 (R1-R4), 4 (invariants J1-J8), 5 (surfaces) and 6 (tests and mutants) bind you.
2. migration/stage-1/cell-release-563/PROPOSAL.md: sections 2 (inventory: the release set, the keep set, the already released set), 3 (release point, order, late callers) and 8 (test matrix T01-T13 and mutants). They are normative.
3. docs/adr/0024-actor-registry-retention-after-termination.md (D5 and the contracts to preserve).
4. The code, read-only, in this order: pekko-actor/actor/cell.go (`cell`), stop.go (`finalize`, `acceptStop`, `terminate`), watch.go (`notifyWatchers`, `addWatcher`, `takeWatchersLocked`), supervision_restart.go (`recreate`, `installNew`, `wakeRestart`), supervision_failure.go (`enqueueFailed`, `failUpExcept`), start_failure.go (`setResume`, `dropResume`), port.go (`postPort`), ref.go (`Tell`), hierarchy.go (`complete`, `readyLocked`).
5. Tests to imitate, not rewrite: registry_characterization_internal_test.go (`rcNewEnv`, `stopped`, `holdSignalOf`, `signalsOf`, `deliver`, `rcObserve`, `rcWant`), registry_retention_internal_test.go, cell_internal_test.go (`ctlExec`, `isClosed`), resume_with_internal_test.go.
6. Pekko, read-only, only to settle a doubt: upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala 247-276; upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala 716-726.

## Allowed edit surfaces (the only files you may write), all under pekko-actor/actor/
Production (2): stop.go, cell.go.
New tests (2): cell_release_internal_test.go (package `actor`), cell_release_api_test.go (package `actor_test`).
Everything else is denied. No existing test may be edited. If you need a file outside the surfaces, stop and report `BLOCKED(surface): <file and why>`. If a test shows the design does not hold in some interleaving, stop and report `BLOCKED(runtime): <the interleaving and the observation>`. Do not edit docs, READMEs, ADRs, `judge/`, `go.mod`, `go.sum`.

## What to deliver
- **Order of work.** (1) Write the characterization tests first: T06, T09, T13 and the restart half of T10, named `TestCellReleaseCharacterization...`. Run the wrapper and report that they passed **before any production change**. (2) Then the production change (R1-R3) and the structural tests T01-T05, T07, T08, T11, T12, named `TestCellRelease...`.
- Production: one method that releases exactly the eight fields of RULEBOOK R1 in one critical section on `c.mu`, called once in `finalize` between `c.notifyWatchers(watchers)` and `close(c.done)`. It runs no user code, takes no other lock, calls no `Execute`, and releases nothing in `acceptStop`, `terminate` or a restart. Update the comments of the released fields in cell.go. Do not release `owner`, `ctx`, `self`, `node`, `sys`, `done`, `origin`, the flags or the counters.
- Do not add a test seam, an option or an exported identifier. Park the finalizer with an in-package fake `watcherLink` and with `completionPoint`, as PROPOSAL 8 says.
- **M5 has no deterministic coverage.** Do not write a test that claims to prove the release happens before `close(done)`; T12 is a `-race` detector and must be described that way in its comment. The order is kept by the position of the call and a comment that says so.
- Tests use `github.com/getsyntegrity/go-specs` like the neighboring files, in-process only, no external service, no `time.Sleep` as evidence of an order, no dependence on GC or finalizers. Each observable effect is asserted (return values, errors, dead letters, events). Every test name starts with `TestCellRelease`.
- Every Go file keeps its header; new files copy the Apache-2.0 header of a neighbor. Functions at cyclomatic complexity 10 or less, no `//nolint`. English comments and identifiers. About 400 authored changed lines is a heuristic, not a cap.

## How to verify (you may run exactly these)
`gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...`, `go build ./pekko-actor/...`, and **`bash migration/stage-1/cell-release-563/run-release-tests.sh`** (the whole pekko-actor module once, then the `CellRelease` tests with `-race`; it fails when no test matches). `go test` is otherwise denied. You cannot run the judge, mutants, the linter or the coverage gate, and you cannot show that the structural tests fail on the base: the coordinator does.

## Hard constraints
- No git write commands, no commits.
- Do not weaken an invariant of U1-U10 or of ADR 0024. If one concrete decision is unresolved and not decidable from the RULEBOOK, the ADR, Pekko or the code, report `NEEDS-DECISION: <the single question, the options, your recommendation>` and continue with the independent work.
- Time bound: 20 minutes wall. The delivery is capped at 450,000 tokens including coordination (a proposal awaiting approval): read only the files listed above.

## Final report (your last message), at most 1,200 tokens
1. Files changed (path and one line each). 2. gofmt, vet, build and the wrapper run: results (trimmed). 3. Per test id T01-T13: the Go test name. 4. Confirmation that the characterization tests passed before any production change. 5. Confirmation that no file outside the surfaces changed, no exported identifier was added and no existing test was edited. 6. Any `BLOCKED` or `NEEDS-DECISION`.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
