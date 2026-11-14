# Task: one focused, read-only review of delivery byid-retention (ADR 0024 tasks T2-T5; issue #541)

You are the only reviewer of this delivery. You are read-only: no Edit, no Write. You can run `git status`, `git diff` (also with a path, for example `git diff -- pekko-actor/actor/hierarchy.go`), `git diff --stat`, `git show <rev>:<path>`, `go vet`, `eza`, `wc`, and read any file with the Read tool. Assume the work is wrong.

## Read in this order
1. `git diff --stat` (the work is uncommitted on top of the package commit, which is HEAD), then the diff of the production files: registry.go, hierarchy.go, system.go, cell.go, ref.go, ask_endpoint.go, guardian.go, supervision_failure.go, errors.go. Judge these before anything else.
2. migration/stage-1/byid-retention/RULEBOOK.md sections 1 to 8, to check the diff against D1-D5, M1-M6, I1-I9 and C1-C9, R1-R10.
3. docs/adr/0024-actor-registry-retention-after-termination.md.
4. Only then the tests.

## What to look for
- **Retirement point and order (D1, I1, I6, I7):** the delete is in `complete`, only for a member signal, only for a non-guardian, after `close(done)`; no lock order inversion; nothing deleted twice or for another owner's child.
- **Guardians (D4, M5):** three entries stay; `WhenTerminated` and `Shutdown` still work after termination.
- **Provenance and authority (M1, M3, I9):** a handle gains no authority by knowing an ID; a handle of another System, an `askEndpoint`, a user-implemented handle or a ghost cell never gets `(done, nil)`; a nil `done` never reaches the caller; the mark is race-free and never cleared.
- **State equivalence (I8):** `Tell`, `Ask`, `Done`, `Watch`, `WatchWith`, `Stop` on a terminated handle give the same result before and after retirement, except D2.
- **Races:** finalization against each public operation; a `Watch` that links while the subject is retired; a late `failUp`; `leaveWatchees`.
- **`ChildFailure.Child` (M4)** and every other `idHandle` literal.
- **Scope guard:** no field of a finalized cell is cleared (T6); no exported identifier; no ID format change; no change outside the surfaces; `go.mod`, `go.sum`, `judge/` and docs untouched.
- **Tests assert what they claim:** each new test would fail under the mutants of RULEBOOK 8; no sleep used as evidence of order; existing tests were only adapted for the reasons in RULEBOOK 7 and none was weakened or removed; the characterization file does not depend on the new code.

## Output
Findings only, classes BLOCKER / DEFECT / INFO, each citing a RULEBOOK rule or a source `path:line`. INFO is recorded, not acted on. **The whole final report is at most 1,500 tokens.** If you could not read something, say exactly what, so it is recorded as unreviewed. No praise, no restatement of the task.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
