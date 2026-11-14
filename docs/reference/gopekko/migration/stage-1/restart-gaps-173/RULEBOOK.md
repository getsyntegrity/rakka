# RULEBOOK: unit B (deterministic GMoverlap) and unit S22 (Go gate)

Read-only inside every loop (kit standing rule 1). Rules derive from Apache Pekko v1.7.1 (`upstream/pekko`, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), from the repository's judge documents, and from the Owner decisions of the second brief (recorded in `EXECUTION-RECORD.md`). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: branch `feat/go-restart-gaps-173` at `fdba453`.
Status: **PREPARED, NOT APPROVED, NOT EXECUTED.** Amendments are queued for the Owner, never edited by an implementer.

## 0. Common
- C1. Only the surfaces named in each unit's prompt may change. Everything else is compared by sha256 with the baseline recorded before the run (section 4).
- C2. Never weaken the comparator (`judge/comparator.py`), a scenario (`judge/scenarios/*.json`), a clause, an assert, a threshold or the coverage gate. No `//nolint`.
- C3. Timeout, crash, compile failure and `harness-error` are INVALID, never a detection and never a survival.
- C4. Earlier campaigns and their reports stay as they are. No result is combined with another.
- C5. Any difference from Pekko that is not already approved stops the unit and returns to the Owner with: original behavior and source, the Go constraint, the proposed adaptation, the observable difference and its tests.
- C6. Test first when a deterministic test and a clear expected outcome exist: observe RED, then GREEN. Otherwise state the exception.
- C7. Artifacts in English. Conventional Commits. No AI attribution lines in commits.

## B. Unit B: a deterministic GMoverlap (Owner decision B)

Target defect (unchanged): a second worker takes messages of actor `r` from the same mailbox, so two `Receive` calls of `r` can run at the same time. Designed detection (unchanged): scenario S21, mode `concurrent`, clause `G3.c`. Pekko's guarantee: one message at a time per actor (`upstream/pekko/actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala:263` `processMailbox`: one mailbox run at a time).

Facts established by the investigation (`EXECUTION-RECORD.md`, Update 2): the harness-error is a `-race` report. Site 1: the extra worker (`drainExtra`, mutation text in `judge/mutate_go.py`) can pop a message and read `c.actor` (`cell.go:270`) while `startActor` is still running the factory, before `c.actor = a` at `cell.go:232`; the mutant checks `c.started`, which is set before the factory finishes. Site 2: a write/write on `c.failedMsg` (`cell.go:294`) when two overlapping `Receive` calls both fail.

- B1. The mutation may change only the mutated text of the `GMoverlap` cell (the edits of its `ref.go` anchors) and the code of the runner that classifies it. It must keep two `Receive` calls of `r` genuinely concurrent: no lock, channel or ordering that makes them run one after the other. A mutant that serializes the Receives removes the defect and is rejected.
- B2. The mutation must not reach a runtime field before the runtime has published it. Candidate for site 1: `drainExtra` proceeds only when it can see, under `c.mu`, that the actor is installed (`c.actor != nil`; the write at `cell.go:232` is under `c.mu`). This is edit-text of the mutant, not a production change.
- B3. **Stop on a remaining race (corrected by Addendum 1).** If, after B2, a race report remains, record that **the evaluated design failed** (state which design, the exact sites, the run and the log), and stop unit B with that evidence. Do not claim that no race-free mutant can exist: that is not demonstrated by one failed design. Unit S22 does not start while B is blocked. The implementer must not add locks to production, change S21, change `G3.c`, change the classification, treat the race report as a detection, or try further designs beyond the one the coordinator approved for this run.
- B4. Detection must come from the judge's own guarantee: outcome `failed` with `G3.c` in the fired rules of S21 `concurrent`. A race report is never a detection of GMoverlap in the scenario path (policy of `go_outcome`).
- B5. Verification of the new design (not a search for a clean run): the mutated copy is run with `go test -race -count=1 -run ^TestJudgeScenarios$` 20 times in the background; the unit is accepted only if 20 of 20 exit zero with no `WARNING: DATA RACE` and the comparator reports `failed G3.c`. One failure means the design is not deterministic: stop and report; do not repeat until it passes.
- B6. `judge/selftest_mutate_go.py` gains cases that pin the changed anchors (each anchor occurs exactly once) and the classification rules (a race or harness-error is INVALID). Existing cases are not edited.
- B7. After B, the other 16 designed mutations and the benign mutation are untouched (their definitions hash-compared).

## S. Unit S22: S22 in the Go gate (Owner approved sections 3-5 of `S22-GO-PROPOSAL.md`, candidate `fdba453`)

Pekko behavior (default hooks, a Stop while the restart waits for children): `Actor.scala:625-631` default `preRestart` unwatches and stops the children, then `postStop()` (call 1); `FaultHandling.scala` `terminate()` replaces the reason by `Termination`; when the last child is removed `handleChildTerminated` runs `finishTerminate`, which calls `aroundPostStop` (call 2) on the same instance, then notifies watchers once. No `PostRestart`, no new instance. Contract (unchanged): `judge/scenarios/S22.json`, G5.f with `pre-start` 1, `pre-restart` 1, `post-restart` 0, `post-stop` 2.

- S1. `prerestart-default` is emitted by `defaultPreRestart` in `pekko-actor/actor/supervision_restart.go`, as its first statement, through `c.sys.emit`, so it exists only when the runtime really executed the default hook. Detail value: `nil`. Nothing else in production changes.
- S2. The event must not change order, decisions or behavior: `emit` is a no-op when the sink is nil; no new lock, no allocation on a nil sink, no change of any other event. Evidence: the full test suite unchanged and green, the three byte-identical files below, and a test that runs the default-restart path with and without a sink and compares the observable outcome (events other than the new one, counts, order of `PostStop`, `Terminated` once).
- S3. The harness observes: a sink adapter in `judge_internal_test.go` maps `prerestart-default` of an actor to the judge event `pre-restart` only for nodes with no `PreRestart` method. The harness never emits `pre-restart` for those nodes by itself and never reconstructs or assumes a `PostStop`, `Terminated` or count.
- S4. `post-stop`, `pre-start` and `terminated` keep coming from the node's own `PostStop`, `PreStart` and the watcher's `Terminated`.
- S5. `judge/go-scope.json`: add `"S22"` to `scenarios`, no waiver. `judge/scenarios/S22.json` unchanged.
- S6. Directed mutation **GMdefaultps** in `MUTATIONS` of `judge/mutate_go.py`: restore the pre-U7 body of `defaultPreRestart` (the `PostStop` of the default hook goes through `c.beginPostStop()` again; text in `S22-GO-PROPOSAL.md` section 5), anchor occurring exactly once. Expected cell: `("S22", "concurrent", "G5.f")`. The required detection is `G5.f`: the fired rules must contain `G5.f`, and it must fail because of the observed `PostStop` count of the old instance (1 instead of 2), not because of a harness error. Other clauses (`G5.c`, `REQ`) may fire as consequences (Addendum 2); a run without `G5.f`, or INVALID, is not met.
- S7. Counts after the unit: 15 scenarios in the Go gate, 18 designed mutations (+ 1 benign). Update every statement of those numbers found by `rg` in the surfaces; do not edit history of earlier units.
- S8. `docs/judge.md` replaces the "S22 is Pekko only" paragraph with the Go row and keeps the explanation of why the event was needed. The statement "before U7 Go ran one `PostStop`; the double call is Pekko's" is recorded exactly.
- S9. S22 is not calibrated against Pekko mutations (G5.f has none) and the documents say so.
- S10. New Pekko-vs-Go difference: none expected. Adaptation (Scala/JVM to Go): Pekko's harness records `pre-restart` by overriding `preRestart`; Go's default hook is runtime code, so the event comes from the runtime sink. If implementation shows any other observable difference, C5 applies.

## 4. Byte-identical files (hash-compared before and after each unit)
`pekko-actor/actor/cell.go`, `stop.go`, `hierarchy.go`, `guardian.go`, `supervision_failure.go`; `judge/comparator.py`; `judge/scenarios/` (all); `judge/pekko-harness/Judge.scala`; `judge/calibrate.py`; `judge/selftest.py`. Unit B additionally: all of `pekko-actor/` (production and tests) and `judge/go-scope.json`.

## 5. Amendments queue
Addendum 1 (B3 corrected: the failed design is recorded, impossibility is not claimed). See `ADDENDUM-1.md`. Recurrence rule (kit rule 6): a third occurrence of the same failure stops the instance-fixing and queues a rule change for the Owner.
