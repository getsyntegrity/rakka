# Task: one focused, read-only review of delivery U7 (Stop during a restart, scope of G4/G5; #449)

You are the only reviewer of this delivery. You are read-only: no Edit, no Write. You can run `git status`, `git diff` (also with a path, for example `git diff e0d70bb -- pekko-actor/actor/supervision_restart.go`), `git diff --stat`, `git show <rev>:<path>`, `go vet`, `eza`, `wc`, and read any file with the Read tool. You cannot run tests or sbt. You run non-interactively and a **spend cap is enforced by the launcher** (USD and wall clock): when it is hit your session ends with whatever you have. Work in this order and keep your findings as you go:

1. `git diff e0d70bb --stat` (the base of this delivery; use `git diff e0d70bb -- <path>` per file, whether or not the work is committed), then the diff of the production file (pekko-actor/actor/supervision_restart.go) and of the judge files (judge/comparator.py, judge/selftest.py, judge/scenarios/S22.json, the S22 hunk of judge/pekko-harness/Judge.scala, docs/judge.md, migration/stage-1/milestone-1.md). Judge these before anything else.
2. migration/stage-1/supervision-u7/RULEBOOK.md sections 2 to 7, to check the diff against R1-R9, 6.1, 6.2 and 7.1.
3. Only then the tests: supervision_stoprestart_internal_test.go and the W11 hunk of supervision_restart_goside_internal_test.go. Read each test file with the Read tool; do not skip any hunk.
4. Pekko only to settle a doubt: `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala` 92-118, 180-212, 247-276, 327-351 and `Actor.scala` 625-631.

## What to look for (assume the work is wrong)
- The Go change is minimal: only `defaultPreRestart` stops going through `beginPostStop`; `stop.go`, `cell.go`, `hierarchy.go`, `guardian.go` untouched; `finalize` still guards the Goexit re-entry (R1, R2).
- A Stop during the wait: no `continueRestart`, no factory call, no `PostRestart`, `PostStop` twice with default hooks and once otherwise, `Terminated` once, P1 before the held child's final stop, P2 after it (R3-R5). A restart that completes is unchanged (R6).
- T4 (Goexit in P1 then Stop) asserts the Go-specific outcome of R7 and does not hang.
- The comparator edit: `r_G5c` compares the parent's last `post-stop-begin` with each child's last `post-stop` and is identical when each has one (check S10, S18 and every scenario that uses G5.c); the **only** new clause is `G5.f` (exact counts on a complete log, zero allowed; only "exceeded" on a prefix; registered in `PARAMS` and `RULES`; bad parameters rejected by `bad_spec`); `r_G5d` and every other rule are untouched. A rule that now accepts something an existing scenario was designed to reject is a DEFECT.
- S22 is `concurrent` only, asserts the counts and order of RESULTS.md (`required` minimums, `G5.f`, `I2.a`, `G5.c`), uses neither `G4.a` nor `G5.d`; the text of G4 and G5 equals RULEBOOK 6.1; the milestone registration is consistent. Could a second instance, a `post-restart` or a third `post-stop` still pass S22? If so, it is a DEFECT.
- The new cases J1-J12 of `judge/selftest.py` fail on the base comparator and pass on the new one, and cover P2 absent or extra, a second instance, a duplicated `Terminated`, prefixes and the final-versus-final `G5.c`; a case that cannot fail is a DEFECT; the existing cases are untouched.
- The tests assert what they claim and would fail under the three mutants of RULEBOOK 7.2 (a second instance after the Stop, a duplicated `Terminated`, the per-instance guard back in `defaultPreRestart`). A vacuous test is a DEFECT.
- Nothing outside the surfaces changed (`git diff e0d70bb --stat`); no change to U6 or to the Go judge gate.

## Output
Findings only, classes BLOCKER / DEFECT / INFO, each citing a RULEBOOK rule or a source `path:line`. INFO is recorded, not acted on. **The whole final report is at most 1,500 tokens.** If you could not read something, say exactly what, so it is recorded as unreviewed. No praise, no restatement of the diff, no second pass: this is the only review of this delivery.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
