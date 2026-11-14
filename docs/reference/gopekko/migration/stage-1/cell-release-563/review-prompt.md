# Task: one focused, read-only review of delivery cell-release-563 (ADR 0024 task T6; issue #563)

You are the only reviewer of this delivery. You are read-only: no Edit, no Write. You can run `git status`, `git diff` (also with a path, for example `git diff -- pekko-actor/actor/stop.go`), `git diff --stat`, `git show <rev>:<path>`, `go vet`, `eza`, `wc`, and read any file with the Read tool. Assume the work is wrong. You did not take part in the implementation and you receive no conclusion of it beyond the diff.

## Read in this order
1. `git diff --stat` (the work is uncommitted on top of the package commit, which is HEAD), then the diff of the production files: stop.go and cell.go. Judge these before anything else.
2. migration/stage-1/cell-release-563/RULEBOOK.md sections 1 to 8, to check the diff against R1-R4 and J1-J8; then PROPOSAL.md sections 2, 3 and 8.
3. docs/adr/0024-actor-registry-retention-after-termination.md.
4. Only then the tests.

## What to look for
- **Release set (R1):** exactly `actor`, `factory`, `failedMsg`, `pendingCause`, `startFailure`, `resumeFn`, `resumeReady`, `failed`, `sysq`; no field of the keep set (`ctx`, `self`, `node`, `owner`, `sys`, `done`, `origin`, the flags, the counters) is cleared.
- **Position and synchronization (R2, J2, J4, J5):** one call in `finalize`, after `notifyWatchers` and before `close(c.done)`; one critical section on `c.mu`; no user code, no other lock, no `Execute`; nothing released in `acceptStop`, `terminate` or a restart. Check the order **by reading the code**: no test covers the order against `close(done)` (mutant M5).
- **Late readers (J3, J6):** find any reader of the release set that can run after the release (`recreate`, `enqueueFailed`, the hold path, `takeSysLocked`, `takeFailedLocked`, `hasWorkLocked`, `runPreRestart`, `deliverRuntime`, `strategy`, `deliverPort`, a retained `Context`). A reader that dereferences a released field is a BLOCKER.
- **Activation ownership:** PROPOSAL 3.3 claims no activation runs after `terminate` returns, so the lock-free fields are safe to write. Try to refute it, including the `Goexit` hand-over paths.
- **Restart and guardians (J7, J8):** `factory` and `actor` stay set across restarts; the guardians' entries stay registered.
- **Scope guard:** `owner` is not released; no exported identifier; no seam or option added; no existing test edited; nothing outside stop.go, cell.go and the two new test files; `go.mod`, `go.sum`, `judge/`, docs untouched.
- **Tests assert what they claim:** each structural test fails under the mutants of PROPOSAL 8 it is assigned to; no sleep as evidence of an order; no test presents T12 or a race build as proof of the order against `close(done)`; the characterization tests do not depend on the new code.

## Output
Findings only, classes BLOCKER / DEFECT / INFO, each citing a RULEBOOK rule or a source `path:line`. INFO is recorded, not acted on. **The whole final report is at most 1,500 tokens.** If you could not read something, say exactly what, so it is recorded as unreviewed. No praise, no restatement of the task.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
