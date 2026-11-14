# Task: implement unit U4 (supervised `Spawn` creation failures) in pekko-actor/actor

You are the single implementer of this unit: production code and tests, one session. You run non-interactively: nobody can answer questions during the run.

## Read first, in this order (do not re-plan or re-map)
1. migration/stage-1/supervision-u4/RULEBOOK.md: the specification, **approved by the Owner on 2026-10-15** (decisions D1-D8, behavior of section 2, reconciliation of section 3, test plan of section 4). It is read-only for you.
2. migration/stage-1/supervision-u4/RUNBOOK.md: sections 3, 4, 7 and 8 only (limits, lessons, allowed surfaces, anchors).
3. migration/stage-1/supervision-u2/RULEBOOK.md section 4: conventions (go-specs tests only, kit-logger, headers on ported files, `errors.Is/As`, no `//nolint`, gocyclo at most 10 per function, coverage at least 85%).
4. docs/adr/0017-supervision.md sections S10 and H7, and the sources named in migration/stage-1/supervision-u4/manifest.tsv (Pekko, read-only, `upstream/pekko`).
5. The code you will change or reuse: spawn.go, cell.go (`drain`, `callPreStart`, `protect`, `handOver`, `abortGoexit`), stop.go (the start step taken by `terminate`, lines around `beginStartLocked`), supervision_restart.go (`installNew`, `finishCreate`, `faultCreate`, `reportInitFailure`, `guarded`), errors.go, hierarchy.go (`reserve`, `commit`, `rollback`, read-only for you).

## Allowed edit surfaces (the only files you may write), all under pekko-actor/actor/
Production:
- spawn.go
- errors.go
- cell.go
- stop.go
- supervision_restart.go

Existing tests that assert the old contract:
- spawn_errors_test.go
- hierarchy_internal_test.go
- hierarchy_concurrent_internal_test.go
- termination_internal_test.go
- example_test.go

New tests (create them):
- spawn_supervised_internal_test.go (the upstream-derived cases: P1, P2)
- spawn_supervised_goside_internal_test.go (the Go-side scenarios T1-T11)

Everything else is denied: hierarchy.go, system.go, watch.go, supervision_failure.go, supervision_hooks.go, doc.go, every other test, judge/, docs/, READMEs, ADRs, workflows. The coordinator updates ADR 0011/0012/0014, the READMEs, doc.go and the parity docs after your run (rows 6, 8-11 of the reconciliation). Do not edit them.

## What to deliver
Implement decisions D1-D8 and the behavior of RULEBOOK section 2, and write the tests of section 4 (P1, P2, T1-T10; T11 only if no existing U2 test already asserts it: look for it with the Read tool in supervision_restart*_test.go and cite what you found in your report). Every test has an assertion that fails if the behavior breaks. Rewrite the old-contract tests of the five existing test files to the supervised contract (RULEBOOK rows 3, 4, 5, 7 and the race of `hierarchy_concurrent_internal_test.go` around the factory held while the root gate closes); each changed assertion gets a comment explaining the change, and you list every one in your report. Ported or adapted test files carry the header convention of the U3 test files, with `PORT-GAP(factory-context)` for the dropped `workingChild` step and `PORT-GAP(top-level-default-U5)` for the top-level case.

Facts you must respect (verified, do not re-derive):
- `Spawn` has exactly three synchronous errors: `ErrInvalidName`, `ErrNameTaken`, `ErrParentStopping`. After the commit nothing in `Spawn` can fail. `Spawn` does not call the factory; the first activation does, through the executor, so with an inline executor the factory runs inside `Spawn`, and then at the return the child may already be terminated. Never write a test, a comment or a doc comment that says the factory runs "on the child's goroutine" as a guarantee, or that the child is "in its start step" at the return.
- Goexit in the factory: with the default executor `Spawn` still returns normally; with an inline executor the Goexit ends the goroutine that called `Spawn`. Goexit tests use the real executor; an inline Goexit case runs `Spawn` on a separate goroutine and observes the cell.
- Reuse the U3 mechanisms; do not duplicate them. A failed factory is an `*InitializationError{InRestart: false}` through `reportInitFailure`; a Restart or Resume choice goes through the existing `finishCreate`. No `PostStop` and no `PreRestart` for an instance that never existed. In the Goexit paths a nested `protect` fires **both** enclosing Goexit handlers, so a flag that says "already handled" is needed when a handler sits inside another guard (this is how U3 solved the PostStop of a restart); do not hand over twice.
- `callPreStart` reads `c.actor`; with a nil actor its type assertion is false and it would report success, so the factory step must run before it and must stop the start step when it fails.

## Judge anchors: do not move or duplicate them
`judge/mutate_go.py` replaces exact strings that must each occur **once** in the file it names. If one changes or appears twice, the work stops. The coordinator checks them after your run. In `cell.go` keep exactly as they are (do not reformat, reorder or re-indent): `if c.stopRequested { // checked`, the two lines `\tps, ok := c.actor.(PreStarter[T])` and `\tif !ok {` at the top of `callPreStart`, the `stopRequested  bool             // the acceptance point of a stop` field line, `if c.depthLocked() == 0 { // idle decision and emptiness check in one critical section`, and `if c.stopRequested || c.depthLocked() > 0 || len(c.lane) > 0 {`. In `stop.go` keep: `PostStopper[T]); ok {` (it must stay unique in the file), `c.sys.publish(DeadLetter{Msg: msg, To: c.cid})`, `func (s *System) Stop(h Handle) {`, `r.acceptStop(nil)`, `c.stopRequested = true` followed by `need := !c.scheduled`, and the `k.acceptStop(nil) // no recursion under a lock...` line. Put the new factory step in a **new function** that `drain` and the start path of `terminate` call before `callPreStart`; do not edit the body of `callPreStart` above its first two lines. Read the full list in `judge/mutate_go.py` (the `MUTATIONS` table) before you edit cell.go or stop.go.

## Hard constraints
- You CANNOT run `go test` (denied by configuration: the coordinator runs every test). Do not try to route around it. Your loop is `gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...` (it also compiles the tests) and `go build ./pekko-actor/...`. Write the tests carefully from the existing helpers (`sup1NewRig`, `defBoss`, `defProbe`, `ctlExec`, `sup1InlineExec`, `holdExec`) and patterns; prove "not processed" with a controlled executor or an explicit channel, never by reading a counter right after a `Tell`; no sleeps as coordination.
- No git write commands, no commits: the coordinator commits.
- Do not weaken the invariants of U1-U3 (suspension counters, `sysq`, mailbox balance, `Budget`, the restart protocol, the Goexit hand-over). Restart and Resume are never converted into Stop.
- If one concrete design decision stays unresolved and is not decidable from the RULEBOOK, the ADR or the Pekko source, stop and report `NEEDS-DECISION: <the single question, the options, your recommendation>` instead of guessing. If a fix needs a file outside the surfaces, stop and report `BLOCKED(surface): <file and why>`; do not work around it.
- If an anchor of the judge would have to change, stop and report `BLOCKED(anchor): <file and anchor>`.
- Time bound: 30 minutes wall. Prefer a coherent, compiling state over polish.
- About 400 authored changed lines per task is a planning heuristic, not a cap; do not split or compress artificially.
- Written artifacts (code, comments, test names) are in English, neutral register.

## Final report (your last message)
1. Files changed (path and one line each) and the behavior each delivers.
2. Result of gofmt -l, go vet, go build.
3. Per scenario P1, P2, T1-T11: the test name(s) that cover it, or the PORT-GAP, or why T11 was not added.
4. Every changed assertion of an existing test and why.
5. Each judge anchor you had to be careful with, and confirmation that you did not change it.
6. Open doubts and anything you could not verify without running tests.
7. If blocked: the BLOCKED or NEEDS-DECISION line, first.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
