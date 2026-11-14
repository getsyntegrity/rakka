# Execution record: Supervision U4, supervised `Spawn` creation failures (#444)

Branch `feat/actor-supervision-u4`, base `develop` at `7f7e292` (U3 merged), package commit `57cdd20`. The design (`RULEBOOK.md`, D1-D8) was approved by the Owner on 2026-10-15; the six hashes of RUNBOOK section 11 were approved by the Owner's instruction to start the run and matched the files before it. Nothing here claims Pekko parity, a current judge matrix or a module-level result.

Status: **implemented, pending close.** The pending items of the unit are in section 8 and are separate from the pending items of the whole module.

## 1. Baseline
Branch clean at `57cdd20`, `develop` unchanged at `7f7e292`, kit at `cf91c9d`, `upstream/pekko` at `9a54d1294c` (the tag `v1.7.1`). Tree of `pekko-actor/actor` at the baseline `973b2e729a8690e400f21af9dfa56f79a3807333`, 69 `.go` files hashed outside the repository. The six approved hashes were compared with the files (all OK) and the launcher refuses to run on a mismatch (checked with a wrong hash: exit 3, no output).

## 2. Agents and tokens
Tokens are input + output + cache creation from the CLI JSON or the agent result. Coordination has no telemetry and is not estimated.

| Agent | Model | Launcher | Tokens | Notes |
|---|---|---|---|---|
| Implementer | sonnet | `run-agent.sh`, `AGENT_WALL_MIN=30`, approved permissions | 303,360 (USD 2.27, 66 turns, 763 s) | Forecast 290,000. It could not run `go test` (denied by configuration); the coordinator ran every test. |
| Final review | opus | Agent tool, read-only | 160,512 (18 tool uses, 271 s) | Forecast 120,000: over by about 40,000. |

Agents total **463,872**, above the evaluation stop of 450,000 and below the cap of 560,000 once coordination is counted only as unmeasured. The coordinator stopped at the evaluation stop, as RUNBOOK section 3 says, and asked; the Owner then authorized the corrections by the coordinator with at most 40,000 more tokens and no new agent. Coordination was not measured, so the compliance with the 560,000 cap cannot be affirmed, only that no agent was launched after the review. Wall clock from the baseline (the start of the run) to the last verification: about 32 minutes by `date`, which includes the stops for the Owner's answers; the implementer ran 13 minutes and the reviewer 5. This is wall clock, not a measure of active work, and it is within the 60 minutes of the plan.

## 3. Verification (final code, commit `3012f47` plus documentation)
- Normal suite `go test -count=1 ./...` of `pekko-actor`: pass.
- **`go test -race -count=1 ./...` of `pekko-actor`, complete, once on the final code: pass** (it takes about 2 s, so it was run whole, not filtered). The two tests changed by F2 also ran 30 times with `-race`: pass.
- `gofmt -l`, `go vet`, `go mod tidy -diff`, `scripts/check_deps.sh`, `check-headers.sh` (29 ported files), `check-adr-signatures.sh`, `modules_selftest.sh`, `go-sdk-update.sh --check`: pass.
- `golangci-lint` v2.14.0 pinned (temporary `GOBIN`): **0 issues**. The implementer's tests had left 3 `gocyclo` findings (`TestChildSpawnInFlightWhileItsParentStops` 11, `TestFactoryFailuresAreSupervisedAndReleaseTheNameAtTermination` 11, `TestACommittedChildHoldsTheParentOpenWhileItsFactoryHasNotRun` 13); the coordinator extracted helpers with the same assertions, no `//nolint` and no threshold change.
- Coverage of `pekko-actor` 98.34% by `scripts/check_coverage.sh` (minimum 85%), run with the profile named as `--profile-name` prints it (with `.out`). The first attempt wrote the profile without `.out` and the gate reported a missing profile: a mistake of the run, not of the module.
- **Judge anchors:** the 22 `(file, anchor)` pairs of `judge/mutate_go.py` each occur once (11 of them live in `cell.go` and `stop.go`, which U4 edited). `judge/` was not edited. The check is of the anchors, not a mutation campaign.
- Scope control after the fact (`git diff --stat` and `shasum` against the baseline): 13 `.go` files, the 12 allowed surfaces plus `completion_internal_test.go`; none outside, no non-Go file changed by the implementer. **This is a post-hoc control, not a test of isolation**; no Haiku probe was run.

## 4. Review (one Opus reviewer) and corrections
No blocker and no major finding. The reviewer found the Goexit handlers (`startActor`, the two restart-factory handlers, `terminate`), the start step, the name lifecycle, the hook accounting and the regressions of U1-U3 clean by reading. Three minor findings:

| Id | Finding | Result |
|---|---|---|
| F1 | Comments referred to `abortGoexit` (removed) and to `callPreStart` as the whole start step (`cell.go:170,173,206`, `stop.go:133`). | Fixed: they name `handOver`, `startActor` and the factory, then `callPreStart`. Comments only. |
| F2 | T2 Restart and P1 sent the `Tell`s after the creation could already have finished, so "no queued message lost" was likely but not forced. | Fixed: the first factory is held on a channel, the test waits for it to enter, queues the messages, then releases it. Both tests passed 30 repeated runs under `-race`. |
| F3 | The `stopping` branch of `Spawn` (`spawn.go:78-80`, `c.acceptStop(nil)` when the gate closes between reserve and commit) lost its end-to-end test: its replacement checks only that `System.commit` returns `stopping=true`. | **Not fixed, registered as a follow-up** (section 8) with an acceptance criterion; pending for the module close. Deleting that line is likely not caught by any current test. The later native review found the same gap independently. |

The corrections were checked by the coordinator and by tests, **not by a second review pass**; this is the author's own check. No native review was requested or granted for the code, and no consent envelope was answered.

## 5. Manual changes and scope extensions
- **Scope extension authorized by the Owner:** `completion_internal_test.go`, a 13th surface. `TestReservationAndCommitKeepOneOutstandingItem` and `TestRollbackLeavesTheOutstandingSet` asserted the old contract (a factory inside `Spawn` sees one outstanding item; `Spawn` returns an error for a nil or panicking factory). They now assert one outstanding item when the child commits and again when the factory runs (the controlled executor holds the activation), and that a failed factory leaves the set at zero once the child terminated. The purpose (one item, not two; nothing left outstanding) is unchanged.
- `spawn_errors_test.go:138`: the assertion `got.Error()` contains the panic text became `errors.Unwrap(got).Error()`, because the decider receives an `*InitializationError` whose own `Error()` is constant; the text is in the error it wraps. `InitializationError.Error()` was **not** changed (a public contract outside the surfaces and the RULEBOOK).
- The rewritten old-contract tests (RULEBOOK rows 3, 4, 5, 7) and each changed assertion are listed in the implementer's report and annotated in the test files.
- **Documentation by the coordinator** (RULEBOOK rows 6-11): ADR 0011 (a note: the sentinels are causes), ADR 0012 (the start step includes the factory; no hook for a missing instance), ADR 0014 (creation transaction, creation steps, the behavior rows for nil, panic, Goexit, stop before and after the commit, stop before the first activation, root gate and root close), `pekko-actor/README.md`, `pekko-actor/actor/doc.go` (comment), `docs/actor-core-parity.md`, `docs/actor-core-first-cut.md` and the root `README.md`. The ADR 0017 reconciliation table and the RULEBOOK describe the old contract as "today" on purpose and were not edited. CURRENT stays stale and the text says so.
- No ACTINV row was touched: the inventory ("Texto pegado(8).txt") is not in the repository (RULEBOOK section 6).

## 6. Pekko equivalence and Go adaptation
Equivalent to Pekko (read at `9a54d1294c`): the constructor runs when `Create` is processed and a null instance or an exception is an `ActorInitializationException` handled by the parent (`ActorCell.scala:622-683`, `dungeon/Dispatch.scala:79-111`); `actorOf` returns synchronously only an invalid name, a name in use and a terminating parent (`dungeon/Children.scala:271-345`); `finishTerminate` calls `aroundPostStop` only when an instance exists (`dungeon/FaultHandling.scala:247-257`). Adapted: `SupervisorHierarchySpec:861-912` (the `workingChild` step is dropped, `PORT-GAP(factory-context)`, because a Go factory has no `Context`); `ActorLifeCycleSpec:165-190` is a `PreStart` failure and only partly adapted. No JVM counterpart: `runtime.Goexit` in the factory (`ErrGoexit`, default Stop), the `factory-failed` event, `FactoryPanicValue`, and the behavior of an inline executor. None is claimed as parity.

## 7. ACTINV
Not updated; see section 5.

## 8. Pending
**Native review of the code (done, approved).** Candidate `1777d37ea3d69bae487623baa294e1837d0665bd..dee0a81b6798fde127ff1b7a84e4eaf1438f4ea4` (base tree `63aa5058a17aa6ae48f11306a16f153d25d9caca`, head tree `ba2689aed7f94001a09546c841f0ef4a256e94e1`), target `sha256:bf424bdf4b4b74b60b6b51058095e2310db0a93c7d749f5a220dc16f32b04ec6`, lineage `review-0e22c2e9921f8c85`, risk medium, 23 files, 1,660 changed lines, one consolidated lens (reliability). The Owner granted the consent envelope; the result was `approved` with no correction opened, and the exact acknowledgement was run (authority burned). An earlier attempt on the larger candidate `7f7e292..55572eb` was rejected by the provider with `lens_context_budget_exceeded` (no review authority was created) and was not retried. The approval is informational: it authorizes no merge. **The commit that adds this section is a later, documentation-only commit; it is not covered by that review.**

**Test gaps found by that review (informative, not a demonstrated defect; both pending for the module close, no test or code was added for them in U4):**
1. **F3 / `R3-stopping-branch-unproved`** (`hierarchy_internal_test.go:553-567`, `spawn.go:78-80`). The `stopping` branch of `Spawn` has no end-to-end test; only `System.commit` returning `stopping=true` is checked. Acceptance criterion: a test that reserves a name under a `*System` or `*Context` owner, closes the gate between the reserve and the commit through a seam, runs the rest of `Spawn`, and asserts that the child is committed, receives its stop, runs its start step once and terminates (no orphan, the outstanding set returns to baseline). It must fail if `c.acceptStop(nil)` is removed from `Spawn`.
2. **`R3-factory-goexit-under-accepted-stop`** (`cell.go:215-235`, `stop.go:79`). `startActor` also runs inside `terminate` when a stop was accepted before the first activation; there it calls `reportInitFailure` unconditionally and, on a Goexit, `handOver` while `terminate` is only half done. The reviewer found no test for two cases; the code reading of the Opus reviewer of this unit supports the intended behavior (`failUp` refuses once the stop is accepted), but no test proves it. Acceptance criteria:
   - **(a) Failed factory after an accepted stop:** a child under a parent that **declares a strategy**, whose stop was accepted before its first activation and whose factory fails (nil actor and panic): no `Failed` reaches the parent's decider, exactly one `stop-accepted` is emitted, no `PreStart` and no `PostStop` run, and the child terminates. It must fail if `reportInitFailure` reaches the decider in that state.
   - **(b) Goexit of the factory inside `terminate`:** the termination completes **exactly once**, with no hanging wait and no duplicated notification (`Done` closes once, a watcher gets one `Terminated`, the owner's outstanding set returns to baseline), on the real executor. It must fail if the hand-over made by the Goexit handler ran the termination twice or not at all.

   T4 covers only a top-level nil factory; `TestACommittedChildHoldsTheParentOpenWhileItsFactoryHasNotRun` uses a parent without a strategy and checks no decider or event counts.
3. The U4 mutants (RUNBOOK section 9) are written but not run.

**Of the whole module (not run in U4):** the two test gaps above (F3 and `R3-factory-goexit-under-accepted-stop`); mutants M1-M15 of U3 and those of U4 on immutable snapshots; the parity judge campaign and its tag; the complete module `-race` repeated; stress runs; the integral review; the mirror of S13, S14 and S16 in the Go gate (U8, #175); the inventory of the Pekko surface; the carried gaps of U2 (#173, K27, K28, K33, the n3 exception) and the informative findings of the U3 native review. The `PORT-GAP(top-level-default-U5)` of a top-level factory failure depends on U5 (guardians) and is isolated.
