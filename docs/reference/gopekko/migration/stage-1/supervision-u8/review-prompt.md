# Task: one focused, read-only review of delivery U8 (S13, S14 and S16 in the Go gate; #175)

You are the only reviewer of this delivery. You are read-only: no Edit, no Write. You can run `git status`, `git diff` (also with a path, for example `git diff b3275d7 -- pekko-actor/actor/judge_internal_test.go`), `git diff --stat`, `git show <rev>:<path>`, `go vet`, `eza`, `wc`, and read any file with the Read tool. You cannot run tests, sbt or the judge. You run non-interactively and a **spend cap is enforced by the launcher** (USD and wall clock): when it is hit your session ends with whatever you have. Work in this order and keep your findings as you go:

1. `git diff b3275d7 --stat` (the base of this delivery), then the diff of `pekko-actor/actor/judge_internal_test.go`, `judge/mutate_go.py` and `judge/go-scope.json`. Judge these before anything else. Use `git diff b3275d7 -- <path>` per file, committed or not.
2. migration/stage-1/supervision-u8/RULEBOOK.md sections 2 to 9, to check the diff against the contract, D1-D5 and the mutants.
3. Only then the docs: docs/judge.md, migration/stage-1/milestone-1.md.
4. Pekko only to settle a doubt: `judge/pekko-harness/Judge.scala` 94-145 and 386-420, `judge/scenarios/S13.json`, `S14.json`, `S16.json`, `judge/comparator.py` (`r_G6none`, `r_G7b`, `r_I6a`, `r_I6b`).

## What to look for (assume the work is wrong)
- **Contract fidelity.** The Go logs must satisfy the Pekko specs unchanged. Could any of G6.d, G7.b, I6.a or I6.b still pass **vacuously** (no `pre-restart`, `decided`, `terminated` event possible)? Are `pre-restart`, `post-restart`, `pre-start` instance numbers right (`i` of the old instance for `pre-restart`, of the new one for `post-restart` and `pre-start`)? Does the custom `PreRestart` really replicate the default (unwatch and stop children, then `PostStop`) and does `PostRestart` run `PreStart`?
- **S16.** `w1` and `w2` use a comparable message; the identical repeat of `w2` is a no-op, the conflict of `w1` fails it once, `boss` decides once with `k = IllegalState`, `w2` decides nothing; the `ack` ordering.
- **S14 witness** is two-way (a restart must end the run, never time out). **S13 witness** cannot cut the log before a wrong `Terminated` could arrive.
- **Existing scenarios untouched:** their logs and their instance numbers (`i: 1`); the zero `Strategy` for nodes that are not supervisors.
- **Determinism** in `controlled` (single goroutine, `ctlExec`) and `concurrent`: any wait without a bound or with a sleep, any data race on recorder state, any blocking call inside a `Receive` that could stall the executor.
- **Mutants.** Each anchor occurs exactly once in its file; each edit compiles; each designed cell `(scenario, mode, clause)` matches what the mutant breaks; a mutant that cannot be caught by its cell, or that would make another cell time out, is a DEFECT.
- **No weakening:** `go-scope.json` has no waiver or diagnostic added; no clause, scenario or mutant removed or loosened; S22 not added (RULEBOOK 7).
- Nothing outside the surfaces changed (`git diff b3275d7 --stat`).

## Output
Findings only, classes BLOCKER / DEFECT / INFO, each citing a RULEBOOK rule or a source `path:line`. INFO is recorded, not acted on. **The whole final report is at most 1,500 tokens.** If you could not read something, say exactly what, so it is recorded as unreviewed. No praise, no restatement of the diff, no second pass: this is the only review of this delivery.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
