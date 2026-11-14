# Task: implement delivery U5a (guardian cells and their start-up) in pekko-actor/actor

You are the single implementer of this delivery: production code and tests, one session. You run non-interactively: nobody can answer questions during the run. This is **U5a only**. U5b (top-level actors under `/user`) and U5c (public configuration and name validation) are NOT part of it.

## Read first, in this order (do not re-plan or re-map)
1. migration/stage-1/supervision-u5/RULEBOOK.md: the specification, **approved by the Owner** (decisions D1-D12). Sections 2, 3.1, 3.3.1, 3.3.2, 3.4, 5 (row 1, 13), 6 (rows T1, T4, T11, T12, T13, T14a) and 9 (row U5a) are yours. It is read-only.
2. migration/stage-1/supervision-u5/u5a/RUNBOOK.md: sections 3, 4, 7 and 8 only (limits, lessons, surfaces, anchors).
3. migration/stage-1/supervision-u2/RULEBOOK.md section 4: conventions (go-specs tests only, kit-logger, headers on ported files, `errors.Is/As`, no `//nolint`, gocyclo at most 10 per function including function literals, coverage at least 85%).
4. Pekko, read-only, under `upstream/pekko`: `actor/src/main/scala/org/apache/pekko/actor/ActorRefProvider.scala` lines 310-376 (the guardians), 476-486, 515-533, 538-620 (creation, strategies, wiring).
5. The code you will change or reuse: spawn.go (`Spawn`, `callFactory`), system.go (`System`, `options`, `newSystem`), hierarchy.go (`node`, `reserve`, `commit`, `Spawner`, `spawnTarget`), cell.go (`cell`, `drain`, `startActor`, `identity`), stop.go (`acceptStop`), watch.go (`register`, `TerminatedHandler`), supervision.go and supervision_failure.go (`Strategy`, `strategy()`, `Supervisor`), supervision_hooks.go (`PreRestarter`). Read-only except for the surfaces below.

## Allowed edit surfaces (the only files you may write), all under pekko-actor/actor/
Production:
- guardian.go (new)
- system.go
- spawn.go

New tests (create them):
- guardian_startup_internal_test.go
- guardian_internal_test.go

Existing tests, to be adapted ONLY for the three extra IDs and the three extra activations (a baseline offset or a settle-after-construction call; no behavior assertion changes):
- hierarchy_internal_test.go
- hierarchy_concurrent_internal_test.go
- names_test.go
- termination_internal_test.go
- throughput_internal_test.go
- supervision_goside_internal_test.go
- supervision_restart_goside_internal_test.go
- spawn_supervised_goside_internal_test.go
- ask_endpoint_internal_test.go
- ask_firstreply_internal_test.go

Everything else is denied, including cell.go, stop.go, hierarchy.go, watch.go, supervision*.go, errors.go, judge/, docs/, READMEs, ADRs and workflows. **If you need to edit a file outside the surfaces, stop and report `BLOCKED(surface): <file and why>`.** The coordinator updates documentation after your run.

## What to deliver (RULEBOOK sections 3.1, 3.3.1, 3.3.2, 3.4)
Three guardian cells created by `NewSystem`: the root guardian, `/user` and `/system`. Behavior:
- Each is a real cell built exactly like any other (reuse the machinery of `Spawn`): a mailbox, an activation, a retained factory that returns the guardian actor. The guardian actor's message type is an unexported `guardianMsg`; it does nothing in `Receive`. It implements `TerminatedHandler[guardianMsg]`: `OnTerminated` stops itself (`Terminated(_) => context.stop(self)`). It implements `PreRestarter[guardianMsg]` with an **empty** `PreRestart` (so the default, which stops the children, never runs: "a guardian MUST NOT lose its children during restart"). It implements `Supervisor[guardianMsg]`, so its strategy is fixed when the factory closure is built and no activation can race with it: root guardian, `OneForOne` with a decider that answers `Stop` for any cause; `/user`, `options.userStrategy` if it is non-nil (an unexported field of `options`, nil in production), else `DefaultStrategy[guardianMsg]()`; `/system`, `DefaultStrategy[guardianMsg]()`.
- **Hierarchy.** The root guardian has no parent (`parent == nil`, so `failUp` refuses it and its own failure stops it directly) and lives in a new hidden `node` of `System` that nothing else uses. `/user` and `/system` are committed under the **root guardian's own node** with the names `user` and `system`. **Do not touch `s.root` or `System.spawnTarget`**: top-level `Spawn(sys, …)` keeps using the legacy `s.root` and has no parent in this delivery (RULEBOOK 3.3.2).
- **How to build without running.** Split `Spawn` in spawn.go into two parts without changing its public behavior: an unexported `commitCell[T]` that does the validation, reservation, construction and `commit` and returns the cell and whether the gate closed meanwhile, **without** handing over the first activation; and `Spawn`, which calls it, does `acceptStop(nil)` if the gate had closed, and then `Execute(c.drain)` and returns the `Ref`. Guardians use an unexported `Spawner` for each owner (the hidden node for the root guardian, the root guardian's node and identity for the other two) and `commitCell`.
- **Start-up order in `newSystem`** (RULEBOOK 3.4), on the calling goroutine, with **no** `Execute` before step 5: (1) construct; (2) register in the order root guardian, `/user`, `/system` (commit them in that order, so their IDs increase in that order; IDs are global, so compare order and never literal values); (3) strategies are in the cells through the factory closures; (4) link the Watch chain by calling `register("Watch", handle, false, guardianMsg{})` on the cells: `/system` watches `/user`, the root guardian watches `/system` (`cell.register` needs no running activation and cannot panic here); (5) call `options.bootPoint`, then `Execute` the first activations in the order root guardian, `/user`, `/system`, calling `bootPoint` between them.
- **Test seams (unexported, nil in production):** `options.bootPoint func(s *System, step int)`, called after step 1, after step 2, after step 3, after step 4, then before the root guardian's activation (step 5), between the root guardian's and `/user`'s (step 6) and between `/user`'s and `/system`'s (step 7); `options.userStrategy Strategy[guardianMsg]`. Store the three handles in an unexported field of `System` for the tests (for example `guards struct{ root, user, system Handle }`); there is no public accessor in this delivery.
- **No public API changes.** Do not add or change any exported identifier. `Config` is untouched. `Context.Parent()` is untouched.

## Tests to write (RULEBOOK section 6; each with an assertion that fails if the behavior breaks)
Use the existing helpers (`ctlExec`, `sup1InlineExec`, `recSink`, `doneOf`, `closedDone`). Deterministic: channels and a controlled executor, no sleeps as coordination. Add one small helper in your new test file to **settle** a freshly built system (run the pending boot activations) and a **baseline helper** that counts only non-guardian IDs; use them in the existing tests you adapt.
- **T1** the three guardians exist, are wired by Watch, and have the strategies of D2, observed through events and `Done`, not through new public API.
- **T4** a guardian keeps its children when it restarts: assert that the guardian actors implement `PreRestarter` with a no-op (an interface assertion plus a call on a guardian that has a child; the child's `Done` stays open).
- **T11** deferred (controlled) executor: right after `NewSystem`, the registry holds the three guardians with increasing IDs root, `/user`, `/system`, the names `user` and `system` are committed, the Watch links exist, and the executor holds exactly three activations and none has run; running them in order leaves the system quiescent.
- **T12** inline executor: each guardian's first activation, observed from `bootPoint`, sees all three registered, committed, with strategies and links; asserted at every probe point.
- **T13** immediate close: for **each** `bootPoint` step from 2 to 7, with **both** executors, and for **each** of the three entry points of RULEBOOK 3.3.1 (stop `/user`; stop `/system`; stop the root guardian) issued from the probe: the sequence and the order of the three `Done` channels of the table hold (**stop `/system`: `/user` is stopped by the root guardian's termination and closes after `/system` and before the root guardian**), every `Done` closes exactly once, one `stop-accepted` per guardian, the watch notifications of the table, nothing deadlocks (use a timeout only as a test guard). Assert on the guardians only: **no effect on top-level actors and none on `Spawn(sys)`** (RULEBOOK 3.3.2).
- **T14a** deferred executor: a legacy top-level `Spawn`, a `Tell` and a failure of that actor before any guardian activation behave exactly as before (the actor stops directly); the guardians neither supervise nor receive them.
- **3.3.2** assertions: after `/user` ended the chain, `Spawn(sys, …)` still returns a `Ref` and not `ErrParentStopping`; a top-level actor failure stops it directly; `Parent()` of a top-level actor is `(nil, false)`.
Existing tests that count the executor's queue (`ex.Len()`) or the registry (`registered()`, `byID`) are listed in the surfaces: after constructing the system with a controlled executor, settle the boot activations; count only non-guardian IDs. Do not change what they assert.

## Judge anchors
`judge/mutate_go.py` replaces exact strings that must each occur once in the file it names. U5a does not edit cell.go, stop.go, hierarchy.go, ref.go or mailbox.go, so no anchor should move. The coordinator checks the 22 pairs after your run. If you believe one must change, stop and report `BLOCKED(anchor): <file and anchor>`.

## Hard constraints
- You CANNOT run `go test` (denied by configuration: the coordinator runs every test). Do not route around it. Your loop is `gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...` (it also compiles the tests) and `go build ./pekko-actor/...`.
- No git write commands, no commits: the coordinator commits.
- Do not weaken the invariants of U1-U4 (suspension counters, `sysq`, mailbox balance, `Budget`, the restart protocol, the Goexit hand-over, the start step). `Spawn` keeps its behavior and its three synchronous errors.
- If one concrete design decision stays unresolved and is not decidable from the RULEBOOK or the Pekko source, stop and report `NEEDS-DECISION: <the single question, the options, your recommendation>` instead of guessing.
- Time bound: 30 minutes wall. Prefer a coherent, compiling state over polish. About 400 authored changed lines per task is a planning heuristic, not a cap.
- Written artifacts (code, comments, test names) are in English, neutral register. Ported or adapted test files carry the header convention of the U3 and U4 test files.

## Final report (your last message)
1. Files changed (path and one line each) and the behavior each delivers.
2. Result of gofmt -l, go vet, go build.
3. Per test T1, T4, T11, T12, T13, T14a and the 3.3.2 assertions: the test name(s) that cover it.
4. Every existing test you adapted, and exactly what changed (a baseline offset or a settle call).
5. Confirmation that you did not touch `s.root`, `spawnTarget` of `*System`, `Config` or any exported identifier.
6. Open doubts and anything you could not verify without running tests.
7. If blocked: the BLOCKED or NEEDS-DECISION line, first.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
