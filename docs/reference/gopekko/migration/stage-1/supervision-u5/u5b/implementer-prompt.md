# Task: implement delivery U5b (top-level actors under /user) in pekko-actor/actor

You are the single implementer of this delivery: production code and tests, one session. You run non-interactively: nobody can answer questions during the run. This is **U5b only**. U5a (the guardian cells, already merged in `develop` at c87fb0e) is your base. U5c (public `Config.GuardianStrategy` API and top-level name validation) is NOT part of this delivery.

## Read first, in this order (do not re-plan or re-map)
1. migration/stage-1/supervision-u5/RULEBOOK.md: the specification, **approved by the Owner** (D1-D12). Yours: sections 2 (D4, D5, D8, D9, D10, D11), 3.2, 3.3, 3.3.1, 3.3.2, 3.6, 5 (rows 2, 3, 4, 7, 8, 13), 6 (T2, T3, T5, T6, T7, T9, T10, T13b, T14b, T16, P1, P2, P4, P5, P6) and 9 (row U5b). Read-only.
2. migration/stage-1/supervision-u5/u5b/RUNBOOK.md: sections 2, 4 (the probe result) and 7 (surfaces).
3. migration/stage-1/supervision-u5/u5a/EXECUTION-RECORD.md: the accepted deviation of 3.3.1 and the Terminated-count contract; U5a's tests are your model.
4. migration/stage-1/supervision-u2/RULEBOOK.md section 4: conventions (go-specs tests only, kit-logger, headers on ported files, `errors.Is/As`, no `//nolint`, gocyclo at most 10 per function including literals, coverage at least 85%).
5. Pekko, read-only, under `upstream/pekko`: `dungeon/Children.scala` 300-320 (`ErrParentStopping` for a stopping parent), `ActorRefProvider.scala` 515-533 (the guardian strategies), the tests named in the RULEBOOK rows P1, P2, P4, P5, P6.
6. The code: guardian.go (`guardianSet`, `startGuardians`), hierarchy.go (`spawnTarget`, `Spawner`, `node`), spawn.go (`commitCell`, `Spawn`, the name check), system.go (`root`, `newSystem`), supervision_failure.go (`failUp`).

## Allowed edit surfaces (the only files you may write), all under pekko-actor/actor/
Production (5):
- guardian.go
- system.go
- hierarchy.go
- spawn.go
- supervision_failure.go (comments only: the text that says a top-level actor has no parent; the guard `c.parent == nil` in `failUp` stays, it is the root guardian's)

New test (1, create it):
- guardian_goside_internal_test.go

Existing tests (19). Adapt them only as RUNBOOK section 7 says, keeping each test's purpose and assertions, and say in a comment why when behavior changed:
- guardian_internal_test.go, guardian_startup_internal_test.go
- hierarchy_internal_test.go, hierarchy_concurrent_internal_test.go, termination_internal_test.go, termination_stress_internal_test.go, completion_internal_test.go, ask_firstreply_internal_test.go
- ask_endpoint_internal_test.go, lifecycle_internal_test.go, throughput_internal_test.go, watch_internal_test.go, watch_api_test.go
- supervision_default_internal_test.go, supervision_default_goside_internal_test.go, supervision_goside_internal_test.go
- spawn_supervised_internal_test.go, spawn_supervised_goside_internal_test.go, spawn_errors_test.go (the last three: only the stale `PORT-GAP(top-level-default-U5)` comments, and nothing else, unless a test fails)

Everything else is denied: cell.go, stop.go, watch.go, supervision*.go (other than supervision_failure.go), errors.go, names_test.go, judge_internal_test.go, judge/, docs/, READMEs, ADRs, workflows. **If you need a file outside the surfaces, stop and report `BLOCKED(surface): <file and why>`.** The coordinator updates documentation.

## What to deliver (RULEBOOK 3.2, 3.3, 3.6, 9 row U5b)
Top-level actors become children of `/user`. A preparation probe (RUNBOOK section 4) shows the production change is small and which tests it breaks; do not assume more production work than this:
- `guardianSet` gains the node of `/user` (for example `userNode *node`), filled in `startGuardians`.
- `System.spawnTarget()` returns `{sys: s, owner: s.guards.userNode, parent: s.guards.user}` plus a marker field on `spawnTarget` (for example `topLevel bool`, documented) so that **top-level names stay unvalidated in U5b** (D7 and the validation are U5c): the name check in spawn.go today is `t.parent != nil && !validChildName(name)`, and giving top-level actors a parent would validate them without the marker. `TestTopLevelNamesAreNotValidated` (names_test.go) must keep passing unedited.
- Retire `System.root` and its construction in `newSystem`; nothing reads it afterwards. Fix its stale comments (`node`'s doc says the root owner is not an actor; `errors.go` is outside the surfaces, leave it).
- `Context.Parent()` needs no code: a top-level cell's parent is now the `/user` identity, so `Parent()` returns `(handle, true)` (D5). Stopping the `/user` handle (`Stop`, `Done`, `Watch`, `WatchWith`, `Unwatch`) works through the existing ID resolution: prove it with T16, do not add code unless T16 shows a gap.
- A top-level failure goes to `/user`'s decider through the existing `failUp` (no new restart logic). Stopping `/user` stops the top-level actors through the existing node termination; `Spawn(sys, …)` then returns `ErrParentStopping` through the closed gate of `/user`'s node. If any of this does not hold, report it as `NEEDS-DECISION` with the evidence.
- **No exported identifier changes.** `Config` is untouched. The accepted deviation of U5a (3.3.1, "stop /system" with the inline executor closes Done as `[user, system, root]`) stays as U5a's tests pin it; do not change those expectations.

## Tests (RULEBOOK 6; deterministic: channels and controlled executors, no sleeps as coordination; Goexit tests on the real executor)
New in guardian_goside_internal_test.go: T2, T3, T5 (internal seam `options.userStrategy` with `OneForOne[guardianMsg](Budget{}, escalate)`), T6, T7, T9, T10, T13b (the three entry points, both executors, live top-level actors, a racing Spawn either commits and is stopped or is refused), T14b (a deferred executor: Spawn, Tell and a failure before any guardian activation are supervised by `/user` and processed in order after the start steps), T16.
Adapted from upstream in guardian_internal_test.go: P1, P2, P4, P5 (the Go-side part), P6 (rules for ADAPTED / PORT-GAP in the RULEBOOK table apply; P3 and P7 are not yours). Rewrite there the U5a assertions of 3.3.2 that U5b flips (`TestTopLevelActorsStayOutsideUserInU5a`, `Spawn` after the chain ended, `Parent()`).

### Existing tests the probe shows will fail, and what to do (do not touch others unless they fail)
Behavior (16 tests), each keeping its purpose:
- guardian_internal_test.go: `TestTopLevelActorsStayOutsideUserInU5a` (3.3.2 flips).
- guardian_startup_internal_test.go: `TestGuardiansImmediateClose` (live top-level actors now stop with `/user`; Spawn is refused after the chain), `TestLegacyTopLevelActorBeforeAnyGuardianActivation` (T14a becomes the supervised variant, T14b lives in the new file; keep the Terminated counts contract of U5a for the guardians).
- hierarchy_internal_test.go: `TestParentChildAndChildren` ("a top-level actor has no parent" becomes the `/user` handle).
- supervision_default_internal_test.go: `TestDefaultStrategyTopLevelStillStops` (T11); supervision_goside_internal_test.go: `TestSup1Interim` (V14); supervision_default_goside_internal_test.go: `TestGoexitInTheDecider` (T9): top-level failure now restarts (Stop for Goexit, `*InitializationError`, `ErrDeathPact`).
- lifecycle_internal_test.go: `TestPreStartHooks`, `TestReceiveFailures`; throughput_internal_test.go: `TestStopCounters`; watch_internal_test.go: `TestFailuresCarryTypedErrors`, `TestWatchWithRepeatOfAnUncomparableMessageIsAConflict`; watch_api_test.go: `TestConflictingRegistrationFailsOnlyTheWatcher`, `TestInvalidHandlesAreRejectedNotTreatedAsDead`; ask_endpoint_internal_test.go: `TestAskEndpointIsNotAWatchableActorHandle`; completion_internal_test.go: `TestTopLevelActorsSignalTheRootAndItReturnsToBaseline`. Most of these make a top-level actor fail in `Receive`/`PreStart` and wait for its `Done`; under `/user` it restarts instead. Keep each test's purpose: give the actor a failure that still ends it where the test needs that (for example a death pact or a Goexit, or a `Stop` directive), or assert the restart; and say why in a comment. Never weaken an assertion, never delete a test.
Compile (7 files read `sys.root` and will not build once it is retired): hierarchy_internal, hierarchy_concurrent_internal, termination_internal, termination_stress_internal, completion_internal, ask_firstreply_internal, guardian_startup_internal. Replace the reads with `/user`'s node (a small helper in a test file that is yours, for example `userNode(sys)`), keep the assertions; baselines that counted the root's names now count `/user`'s (D8).
Controlled executors: a top-level failure now queues a `Failed` for `/user`; tests that asserted the exact `ex.Len()` after such a failure take one extra activation for it (settle it, do not change what they prove).

## Judge anchors
`judge/mutate_go.py` replaces exact strings that must each occur once. Of the files you may edit only hierarchy.go carries an anchor: ` || len(n.out)+n.reserved != 0 {` (two mutants). Keep that exact text, once. If an anchor must move, stop and report `BLOCKED(anchor): <file and anchor>`.

## Hard constraints
- You CANNOT run `go test` (denied: the coordinator runs every test). Your loop is `gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...` (it compiles the tests) and `go build ./pekko-actor/...`.
- No git write commands, no commits: the coordinator commits.
- Do not weaken the invariants of U1-U5a (suspension counters, `sysq`, mailbox balance, `Budget`, the restart protocol, the Goexit hand-over, the start step, the guardians' `Terminated` contract).
- If one concrete decision stays unresolved and is not decidable from the RULEBOOK or Pekko, stop and report `NEEDS-DECISION: <the single question, the options, your recommendation>` instead of guessing.
- Time bound: 40 minutes wall. Prefer a coherent, compiling state. About 400 authored changed lines per task is a planning heuristic, not a cap.
- Written artifacts (code, comments, test names) are in English, neutral register.

## Final report (your last message)
1. Files changed (path and one line each).
2. Result of gofmt -l, go vet, go build.
3. Per test T2, T3, T5, T6, T7, T9, T10, T13b, T14b, T16, P1, P2, P4, P5, P6: the test name(s) that cover it.
4. Every existing test you adapted and exactly what changed and why.
5. Confirmation that no exported identifier, `Config` or file outside the surfaces changed, and that the hierarchy.go anchor is intact.
6. Open doubts and anything you could not verify without running tests.
7. If blocked: the BLOCKED or NEEDS-DECISION line, first.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
