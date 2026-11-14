# Task: one focused, read-only review of delivery U6 (AllForOne, #447)

You are the only reviewer of this delivery. You are read-only (no Edit, no Write; you can run `git status`, `git diff`, `git diff --stat`, `go vet`, `eza`, `wc` and read files). You cannot run tests. You run non-interactively and there is a **spend cap enforced by the launcher** (USD and wall clock): when it is hit your session ends with whatever you have. So work in this order and write your findings as you go in your head, not at the end:

1. `git diff --stat`, then `git diff` of the three production files only (supervision.go, supervision_failure.go, supervision_restart.go). Read nothing else before you have judged these.
2. migration/stage-1/supervision-u6/RULEBOOK.md sections 2 to 6, to check the diff against R1-R11, D1-D3.
3. Only then the tests (supervision_allforone_internal_test.go, supervision_allforone_example_test.go and the AF7 hunk of supervision_restart_internal_test.go).
4. Pekko only to settle a doubt: `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala` 554-567, 58-88 and `dungeon/FaultHandling.scala` 92-118.

## What to look for (assume the work is wrong)
- The exported surface is exactly `AllForOne[T any](Budget, Decider[T]) Strategy[T]` and nothing else; `OneForOne`/`DefaultStrategy` behavior unchanged.
- R6 short-circuit: counters of children not reached are untouched; children before the denier stay charged; every child of the snapshot is stopped on denial; the failing child is never suspended twice and siblings are suspended exactly once per permitted round (R11).
- One snapshot (D1), taken **before** the decider and only for `kindAllForOne`; the same one for both phases; stopping children included; a child created by the decider is outside the round and is not stopped by it (D2); a member that ends before its turn is still evaluated with the counters captured with it, never skipped, and no stats entry is re-created for it (RULEBOOK 4, AF15); order = ascending name, documented as observed and not guaranteed (section 5).
- Resume does not touch siblings. Escalate, ok == false, a nil decider and a panicking decider or Goexit: AllForOne takes **no action** (no counter, no sibling restart/stop/resume), but P's own failure suspends its children except F (R3a); a test that demands untouched siblings in that case is a DEFECT. Stop stops all. `FailedMessage` is absent for healthy siblings and present for a sibling with a pending failure (R8).
- No change to the `PostStop` guard, to logging, or to files outside the surfaces (`git diff --stat`).
- The tests assert what they claim and would fail under the three mutants of RULEBOOK 9.3 (restart only F; do not stop all on denial; count only one child). A test that passes vacuously is a DEFECT.

## Output
Findings only, classes BLOCKER / DEFECT / INFO, each citing a RULEBOOK rule or a source `path:line`. INFO is recorded, not acted on. **The whole final report is at most 1,500 tokens.** No praise, no restatement of the diff, no second pass: this is the only review of this delivery.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
