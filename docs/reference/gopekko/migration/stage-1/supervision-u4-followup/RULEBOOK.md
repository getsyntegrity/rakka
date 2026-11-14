# RULEBOOK: U4 follow-up, #459 (stopping branch of Spawn) and #460 (factory failure and Goexit after an accepted stop)

Read-only inside every loop (kit standing rule 1). Rules derive from Apache Pekko v1.7.1 (`upstream/pekko`, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), from the approved goPekko contracts named below, and from the issues. Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `develop` at `a5782c9651bc382020099dfda47b7fdc99dc971b`, branch `feat/go-u4-459-460`.
Status: **PREPARED, NOT APPROVED, NOT EXECUTED.** Amendments are queued for the Owner, never edited by an implementer.

## 0. Common
- C1. Only the surfaces named in the unit prompt may change. Everything else is compared by sha256 with a baseline recorded before the run.
- C2. **No production code changes** to make a test pass. A test that fails on production code derived from the contract shows a candidate defect: stop that item, report the evidence (test, observed, expected, source), and the correction is a separate decision. The only production change contemplated is the test seam of D1, nil in production, in its own commit and only if the Owner approves it.
- C3. Never weaken a comparator, scenario, clause, assert, coverage or threshold. No `//nolint`.
- C4. Timeout, crash, compile failure and `harness-error` are INVALID, never a detection. A mutant is detected only when a named test fails by an assertion or a data race; a test that hangs must fail through its own bounded wait with a message of its own, never through the 30-second `sup1Recv` timeout.
- C5. Any semantic difference from the approved contract, or from Pekko beyond the ones declared in section 2, stops the unit and returns to the Owner: original behavior and source, the Go constraint, the adaptation, the observable difference and its tests.
- C6. Tests are deterministic: controlled executor (`ctlExec`) and explicit synchronization, or the existing seams (`lockPoint`, `userStrategy`, `completionPoint`, `sink`); no sleeps as synchronization. The race test is run with `-race` and `-count=20`.
- C7. Test first: observe RED (the test fails without the effect it protects, or the mutant is detected) before GREEN. If a test is GREEN on the first run, show it fails against its directed mutant.
- C8. Artifacts in English; Conventional Commits; no AI attribution; tests and docs alongside behavior.
- C9. The mutants of section 4 are run on copies outside the repository, with the runner of `judge/mutate_go.py` (`UNIT_MUTATIONS`), and reported per mutant as detected / survived / INVALID, kept apart.

## 1. Traceability matrix (Pekko to Go)

| Item | Pekko source (v1.7.1) | Approved Go contract | Gap class | Acceptance |
|---|---|---|---|---|
| #459 stopping branch of `Spawn` (`spawn.go:64-66`) | No JVM equivalent of the window: `makeChild` throws `IllegalStateException` if the parent is terminating (`dungeon/Children.scala:314`) and `reserve` throws in a terminating container (`dungeon/ChildrenContainer.scala:205-206`); a Pekko parent creates children inside its own turn, so its termination cannot interleave | ADR 0014 (creation transaction; `ErrParentStopping` before the commit); ADR 0012: a stop accepted before the first activation runs the start step once, then `PostStop` once; `Spawn` doc: "if the owner's stop is accepted while Spawn runs, the child is committed already stopping" | missing validation of a Go adaptation | T459 below |
| #460(a) factory failure after an accepted stop | `ActorCell.scala:622-683` and `dungeon/Dispatch.scala:79-111` (constructor failure is an `ActorInitializationException` reported to the parent); `dungeon/FaultHandling.scala:247-257` (`finishTerminate` calls `aroundPostStop` only when an instance exists) | ADR 0012, paragraph "U4 (ADR 0017 S10, #444)": a stop accepted before the first activation and a failed factory (nil, panic, Goexit): no `PreStart`, no `PostStop`, the failure is only an event, **nothing is reported to the parent**, the stop completes | missing validation; **declared difference from Pekko**: in Pekko the `Create` runs before the `Terminate` and the parent still receives `Failed` and decides; in Go the stop is accepted at call time and the failure is not reported (end state with the default decider is the same: the child is gone) | T460a below |
| #460(b) Goexit of the factory inside `terminate` | none (`runtime.Goexit` has no JVM equivalent; no parity is claimed) | ADR 0012 (U4 paragraph) and the Goexit hand-over of `startActor` (`cell.go`): the termination is completed exactly once | missing validation of a Go-only mechanism | T460b below |

## 2. Declared differences (not new decisions)
- D-A. For #460(a), Pekko reports the constructor failure to the parent even when a `Terminate` is queued behind `Create`; Go reports nothing once the stop is accepted (ADR 0012). It is the approved contract; the tests pin the Go contract and the record states the difference.
- D-B. `runtime.Goexit` in a factory: Go-only, contract of ADR 0012 and ADR 0017 S10 (`ErrGoexit`).
- D-C. The window of #459 does not exist in Pekko; the Go contract is the reference.
Anything else observed is C5.

## 3. Test rules

### T459 (needs decision D1)
- T459.1. For each owner type, `*System` (top-level) and `*Context`: reserve a name, close the owner's gate between the reserve and the commit through the seam of D1, run the rest of `Spawn`. Close the gate the way production does (accept the owner's stop), not by writing `closed` directly.
- T459.2. `Spawn` returns the `Ref` and a nil error (after the commit nothing in `Spawn` can fail).
- T459.3. The child is committed, its factory runs exactly once in its first activation, `PreStart` runs once, `PostStop` runs once after `PreStart`, exactly one `stop-accepted` event for the child, `Done` closes, its name is released, and the owner's outstanding set and reservation count return to the baseline (no orphan); the owner finalizes after the child.
- T459.4. The test fails if `c.acceptStop(nil)` is removed from `Spawn` (mutant M459). The guardian path (`guardian.go:118-122`) is outside this unit unless the implementer shows the same gap there; then it is reported, not fixed.

### T460a
- T460a.1. A child under a parent that **declares a strategy** that counts its decider calls (for a top-level child use the `userStrategy` option). The stop is accepted before the child's first activation (controlled executor: `Spawn`, then `Stop`, then run the activation). Two factories: one returns a nil actor, one panics (create the hooked instance first and then panic, so a stray hook would be visible).
- T460a.2. Expected: the decider is never called (no `Failed` reaches the parent); exactly one `stop-accepted`; exactly one `factory-failed` event whose cause is `ErrNilActor` or wraps `ErrFactoryPanic`; no `PreStart` and no `PostStop` of any instance; the child terminates (`Done` closes); the name is released; the parent's outstanding set returns to the baseline.
- T460a.3. A control with a working factory in the same harness shows `PreStart` once and `PostStop` once (ADR 0012), so the absence above is meaningful.
- T460a.4. The test fails if `reportInitFailure` reaches the decider with the stop accepted (mutant M460a).

### T460b
- T460b.1. The factory calls `runtime.Goexit` and the stop was accepted before the first activation. Use the **real executor** (the default one); hold the first activation with the existing `lockPoint` seam until the stop is accepted.
- T460b.2. Expected: the termination completes **exactly once**: `Done` closes once, a watcher receives exactly one `Terminated`, exactly one `stop-accepted`, no `PreStart`/`PostStop`, no decider call, the owner's outstanding set returns to the baseline, no hanging wait (a bounded wait of the test's own, with its own failure message).
- T460b.3. The test fails if the hand-over made by the Goexit handler of `startActor` runs the termination twice (M460b-twice) or never (M460b-none). If a mutant has no observable effect required by the contract, report that with evidence (an equivalent mutant) and do not weaken the claim.

## 4. Directed mutants (anchored on the real production code; each anchor must occur exactly once)
| ID | File | Mutation | Killed by | Contract broken |
|---|---|---|---|---|
| M459 | `spawn.go` | delete the line `c.acceptStop(nil) // before the first Execute: PreStart once, then PostStop once (ADR 0012)` | T459 | a child committed while the gate closed is never stopped |
| M460a | `supervision_failure.go` | `if c.stopAccepted() \|\| c.parent == nil {` becomes `if c.parent == nil {` in `failUpExcept` | T460a | an init failure reaches the decider after a stop |
| M460b-twice | `cell.go` | in the Goexit handler of `startActor`, call `c.handOver()` twice | T460b | termination executed twice |
| M460b-none | `cell.go` | in the same handler, remove `c.handOver()` | T460b | termination never completed |
Cells go in `UNIT_MUTATIONS` of `judge/mutate_go.py` with their selftest cases, like K27/K28/K33. `judge/mutate_go.py` is an input of the Go campaign: adding the cells makes the committed campaign stop being CURRENT until it is renewed (the Owner anticipated this); the two declared exclusions and every comparator, clause and classification stay as they are.

## 5. Byte-identical files (sha256 against the baseline)
`pekko-actor/actor/cell.go`, `stop.go`, `hierarchy.go`, `guardian.go`, `supervision_failure.go`, `supervision_restart.go`, `spawn.go` (unless D1 is approved: then only the lines of the seam, in their own commit); `judge/comparator.py`, `judge/evaluate.py`, `judge/go_scope.py`, `judge/go-scope.json`, `judge/scenarios/`, `judge/pekko-harness/`, `judge/calibrate.py`, `judge/selftest.py`, `scripts/check-judge-matrix.py`; every existing test file.

## 6. Decisions pending for the Owner
- **D1. A test seam in production for #459.** Evidence: between `t.sys.reserve` and `t.sys.commit` in `commitCell` (`spawn.go:79-96`) the code runs no user code and no existing seam (`lockPoint`, `linkPoint`, `recheckPoint`, `completionPoint`, `bootPoint`) is reachable there, so the gate cannot be closed deterministically in that window without touching production. The issue allows "a non-exported test seam if needed"; the brief says not to change production to ease a test. Proposal: one `commitPoint func()` field in `options`/`System`, nil in production (same pattern as `lockPoint`), called once in `commitCell` between `reserve` and `commit`, in its own commit, no behavior change with a nil value. If rejected, #459 stays open and only #460 is delivered.

## 7. Amendments queue
None.
