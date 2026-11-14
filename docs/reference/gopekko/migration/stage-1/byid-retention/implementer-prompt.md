# Task: implement tasks T2-T5 of ADR 0024 (retirement of terminated actors from `System.byID`; issue #541) in pekko-actor/actor

You are the single implementer of this delivery: production code and tests, one session. You run non-interactively: nobody can answer questions. Scope is **T2-T5 only**. T6 (releasing the fields of a finalized cell) is NOT part of this task. The base is branch `feat/actor-byid-retention` (it holds the ADR text and this package); `pekko-actor/` is `develop` `cd558941`.

## Read first, in this order (do not re-plan or re-map)
1. migration/stage-1/byid-retention/RULEBOOK.md (approved, read-only): all of it. Sections 1 (goal and non-goals), 4 (decisions D1-D5 and details M1-M6), 5 (surfaces), 6 (invariants I1-I9), 7 (tests C1-C9, R1-R10 and the adaptations of existing tests) bind you.
2. docs/adr/0024-actor-registry-retention-after-termination.md (the design and the accepted decisions).
3. The code, read-only, in this order: pekko-actor/actor/registry.go, hierarchy.go (`commit`, `complete`, `idHandle`, `node`), system.go (`System`, `Done`, `options`, seams), stop.go (`finalize`, `acceptStop`), cell.go (`cell`, `identity`, `snapshot`), ref.go, ask_endpoint.go (`askTarget`), guardian.go (`newGuardian`, `startGuardians`), supervision_failure.go (`handleFailed`), errors.go, watch.go (`resolve`, `register`), system_shutdown.go.
4. Tests to imitate, not rewrite: termination_internal_test.go, hierarchy_internal_test.go, hierarchy_concurrent_internal_test.go, watch_internal_test.go, ask_terminated_internal_test.go, ask_endpoint_internal_test.go, lifecycle_internal_test.go, throughput_internal_test.go, cell_internal_test.go (`ctlExec`), behavior_test.go (foreign System `Done`).
5. Pekko, read-only, only to settle a doubt: upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala 247-276; upstream/pekko/actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala 145-152 and 248-255.

## Allowed edit surfaces (the only files you may write), all under pekko-actor/actor/
Production (9): registry.go, hierarchy.go, system.go, cell.go, ref.go, ask_endpoint.go, guardian.go, supervision_failure.go, errors.go.
New tests (3): registry_characterization_internal_test.go, registry_retention_internal_test.go, registry_retention_api_test.go (`package actor_test`).
Existing tests, only for the adaptations of RULEBOOK 7 (6): lifecycle_internal_test.go, throughput_internal_test.go, termination_internal_test.go, hierarchy_internal_test.go, hierarchy_concurrent_internal_test.go, ask_endpoint_internal_test.go.
Everything else is denied. If you need a file outside the surfaces, stop and report `BLOCKED(surface): <file and why>`. If a test shows the design does not hold in some interleaving, stop and report `BLOCKED(runtime): <the interleaving and the observation>`. Do not edit docs, READMEs, ADRs, `judge/`, `go.mod`, `go.sum`.

## What to deliver
- T2: `registry_characterization_internal_test.go` with C1-C9. It must pass on the base without any production change: write and run it FIRST, before touching production code, and report that it passed.
- T3: the provenance mark (M1) and `askTarget` without the registry; AK20 and `ErrRecipientTerminated` for a terminated `Ref` preserved.
- T4: `idHandle.done`, the native-handle interface and `System.Done` per M3; `ChildFailure.Child` per M4.
- T5: retirement in `complete` (D1, I1, I3, I4, I6, I7), guardians flagged (M5); R1-R10 in the two retention test files; the adaptations of existing tests (RULEBOOK 7); production comments that said "never pruned" rewritten; the `ErrUnknownHandle` doc per D3.
- Do not set any field of a finalized cell to nil (T6). Do not add an exported identifier. Do not change the ID format.
- Tests use `github.com/getsyntegrity/go-specs` like the neighboring files, in-process only, no external service, no `time.Sleep` as evidence of an order. Name every new test `Retention...` so the wrapper selects it. Each observable effect of a rule is asserted (return values, errors, dead letters, events), through the public interface where the file is `actor_test`.
- Every Go file keeps its header; new files copy the Apache-2.0 header of a neighbor. Functions at cyclomatic complexity 10 or less, no `//nolint`. English comments and identifiers. About 400 authored changed lines per task is a heuristic, not a cap.

## How to verify (you may run exactly these)
`gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...`, `go build ./pekko-actor/...`, and **`bash migration/stage-1/byid-retention/run-byid-tests.sh`** (the whole pekko-actor module once, then the `Retention` tests with `-race`). `go test` is otherwise denied. You cannot run the judge, mutants, the linter or the coverage gate: the coordinator does.

## Hard constraints
- No git write commands, no commits.
- Do not weaken an invariant of U1-U10. If one concrete decision is unresolved and not decidable from the RULEBOOK, the ADR, Pekko or the code, report `NEEDS-DECISION: <the single question, the options, your recommendation>` and continue with the independent work.
- Time bound: 30 minutes wall. The delivery is capped at 560,000 tokens including coordination: read only the files listed above.

## Final report (your last message), at most 1,200 tokens
1. Files changed (path and one line each). 2. gofmt, vet, build and the wrapper run: results (trimmed). 3. Per test id C1-C9 and R1-R10: the Go test name. 4. Confirmation that C1-C9 passed before any production change. 5. Confirmation that no file outside the surfaces changed and no exported identifier was added. 6. Any `BLOCKED` or `NEEDS-DECISION`.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
