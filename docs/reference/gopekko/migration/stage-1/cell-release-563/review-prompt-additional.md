# Task: the one additional independent, read-only review of the T6 candidate (ADR 0024 D5; issue #563; PR #739 on top of PR #738)

You are the only reviewer of this round. You have no Edit or Write. Assume the work is wrong. You did not take part in it.

## Why this review exists, and the commands you may use
A first review (of `dcb9c298b01d985db1390f556d1d6e3287304e29`) found no blocker but was **incomplete** because two of its `Bash` calls were denied: a compound command (`cd`, `;`, a pipe to `grep`, `&&`) and `git -C <path> diff ...`. Do not repeat them. Your working directory is the repository root and your permissions are: `Read` of any file under it; `git status`; `git diff` in any plain form (`git diff <rev> HEAD -- <paths>`, `git diff --stat <rev> HEAD`); `git show <rev>:<path>`; `git log --oneline -5`; `eza`, `wc`; `go vet ./pekko-actor/...`. **Use one plain command per call. Never use `git -C`, `cd`, `;`, `&&`, `|`, redirects or `grep` in a shell call**: use the `Read` tool to read and search files. `go test` is denied, so you cannot run tests or mutants: **mark every conclusion READ or RAN.** If a call is denied, quote the command and the message verbatim in your report and say what that leaves unreviewed. Do not retry the same form.

## What you review (exact revision)
Run `git log --oneline -5` and report the abbreviated HEAD you see. The candidate is the checked-out tree. Reference points: package commit `550bfa16b3d82fe66ff4cf07560e30e190a27837`; first review `dcb9c298b01d985db1390f556d1d6e3287304e29`; `develop` `a2940d416e9ed773a987168ed329aa7ca061c0c8`. The production code (`pekko-actor/actor/stop.go`, `cell.go`) and the two test files are byte-identical to what the first review read, except that the README of `pekko-actor` and ADR 0024 changed (inline below) and the unit's documents were amended. Check that claim: `git diff --stat dcb9c298b01d985db1390f556d1d6e3287304e29 HEAD`.

## Scope: only what the first review left uncovered. Do not redo what it covered.
The first review (read it in `migration/stage-1/cell-release-563/EXECUTION-RECORD.md`, sections 12 and 13) covered, by reading: the finalization order `notifyWatchers`, `releaseTerminated`, `close(done)`, `signal` in `finalize` including after a `Goexit` in `PostStop`; the need for `c.mu`; the M4 correction in T05; the mutant count; the scope. State which of those conclusions still apply to this HEAD (they should, if the production code did not change) and then do **not** re-derive them in full.
1. **Test bodies T01-T04 and T06-T13** (`pekko-actor/actor/cell_release_internal_test.go`, `cell_release_api_test.go`; the table of tests is `PROPOSAL.md` section 8). For each test say what it **actually asserts** (not what its name or comment says), whether that matches the row of PROPOSAL 8 and the invariant it serves, and whether it could pass vacuously (an assertion that is always true, a precondition that may not be reached, a helper that swallows a failure). T05 was covered; skim only.
2. **J1-J8** (`RULEBOOK.md` section 4): for each invariant, name the code line that implements it and the test that pins it, or say that none does.
3. **RED/GREEN and mutant attribution.** `EXECUTION-RECORD.md` sections 3, 5, 10 and 13: check that the table of tests that fail on the base (T01, T02/T05, T03, T07, T08, T11) is consistent with what those tests assert (a test that asserts nothing the release changes cannot fail on the base). Check the attribution of the 24 mutants to tests: in particular **M9**, which the record now attributes to the existing suite and not to T06 (a text search shows `told` and `deadLettered` in no `CellRelease` assertion); and whether the other "killed by" attributions are plausible given what the tests assert. You cannot run mutants: say so.
4. **README and ADR 0024.** The diffs against `develop` are inline below, and you can also read them with `git diff a2940d416e9ed773a987168ed329aa7ca061c0c8 HEAD -- pekko-actor/README.md` (one call) and the same for `docs/adr/0024-actor-registry-retention-after-termination.md`. Check every factual claim against the code and the record. Flag any claim of validation, review, parity or measurement that the evidence does not support.
5. **Reconciliation and later documentary amendments.** `git diff --stat 550bfa16b3d82fe66ff4cf07560e30e190a27837 HEAD`; the two merge commits bring only documentation already in `develop`. Check the amendment of the `PROPOSAL.md` M9 row and of `RUNBOOK.md` section 9 (hashes): the record says only those changed among the hashed files and no launch input changed; verify with `git diff --stat 550bfa16b3d82fe66ff4cf07560e30e190a27837 HEAD -- migration/stage-1/cell-release-563` and say whether the claim holds.

## Constraints on your conclusions (from the Owner)
- Do **not** ask for a new test seam or for a change of production to detect M5 or M11; O1 is "no new seam". M5 (the order of the release against `close(done)`) has no deterministic coverage and rests on inspection: say whether the record states that plainly. M11 (release without `c.mu`): distinguish exclusion by lifecycle from protection by mutex; the lock stays. The survival of M5 or M11 is not by itself a defect.
- Do not read or judge this as approval to merge. Judge whether the candidate is **integrable as a draft chain #738 then #739**, what blocks it if anything, and what remains undemonstrated.

## Output
Findings only, each with **severity** (BLOCKER / DEFECT / INFO), **location** (`path:line`), the **statement**, and the **evidence** (READ or RAN). Then: (a) the **coverage effectively achieved**: which of items 1-5 you completed, and for item 1 which tests you read and which you did not; (b) which conclusions of the first review you confirm; (c) a verdict on integrability of the chain; (d) the HEAD you saw. **At most 2,500 tokens.** If something was denied or you ran out of room, say exactly what is left unreviewed so that it is not recorded as covered. No praise, no restatement of the task.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.

## Inline: diff of `pekko-actor/README.md` against develop `a2940d4`
```diff
diff --git a/pekko-actor/README.md b/pekko-actor/README.md
index 6841208..82364c1 100644
--- a/pekko-actor/README.md
+++ b/pekko-actor/README.md
@@ -251,3 +251,3 @@ Limitations, stated without hiding them:
   handle) gets a closed channel and `ErrUnknownHandle` from `Done` once the actor is retired. Retirement ends the System's reference to the actor; a `Ref` the application keeps still holds the
-  cell, and the fields of a finalized cell are not cleared yet (T6, not implemented). No heap measurement was made.
+  cell. The T6 candidate (draft, unmerged; one independent read-only review found no blocker but was incomplete, because part of its commands were denied; record in `migration/stage-1/cell-release-563/EXECUTION-RECORD.md`) releases the user instance, the factory and the failure and restart state of a finalized cell (`releaseTerminated`, after the watchers are told and before `Done` closes; no test asserts that order against `Done` closing, it rests on the position of the call and on inspection); identity, the hierarchy node, `ctx`, `self`, `owner` and the counters stay, so a held `Ref` still pins that shell and, through `owner`, the parent's node. No heap measurement was made.
 - The default strategy exists for a child of an actor (see "Supervision (units U1 to U5)"); the guardians are actors (units U5a to U5c) and the System can be terminated with `Terminate` and `Shutdown` (U10). The unexported completion seam (`completionPoint`) exists for tests only.
```

## Inline: diff of ADR 0024 against develop `a2940d4`
```diff
diff --git a/docs/adr/0024-actor-registry-retention-after-termination.md b/docs/adr/0024-actor-registry-retention-after-termination.md
index ca96c4b..f4568c4 100644
--- a/docs/adr/0024-actor-registry-retention-after-termination.md
+++ b/docs/adr/0024-actor-registry-retention-after-termination.md
@@ -8,3 +8,4 @@
   Accepted is not Pekko parity. Tasks T2-T5 are implemented in PR #543 (record in
-  `migration/stage-1/byid-retention/EXECUTION-RECORD.md`); T6 is not started. The work is tracked in #541.
+  `migration/stage-1/byid-retention/EXECUTION-RECORD.md`); T6 has a draft, unmerged candidate, independently reviewed once and incompletely
+  (record in `migration/stage-1/cell-release-563/EXECUTION-RECORD.md`, tracked in #563). The work is tracked in #541.
   The first version of this ADR was a proposal with no code (PR #542).
@@ -271,3 +272,3 @@ for any unit that is a port or adaptation, the Code Migration Kit (`migration-ki

-- Every reader of the cell fields that T6 would release (only a grep of `c.actor`, `c.factory`, `c.ctx` was done).
+- Every reader of the cell fields that T6 would release (only a grep of `c.actor`, `c.factory`, `c.ctx` was done). The T6 candidate inventoried them by reading (`migration/stage-1/cell-release-563/PROPOSAL.md` sections 2 and 3); that is a reading, not a proof. One independent read-only review re-derived the readers from the code, also by reading, and found none that touches a released value; that review was incomplete (record section 12).
 - Judge scenarios that touch terminated handles.
```
