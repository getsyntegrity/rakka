# Scope: Typed restart and the wait for children, what #976 left (#997, TYP-12)

Status: **approved by the Owner at gate G2 on 2026-11-06**, with decisions D-1 to D-4 below as the Owner decided them (D-1: kit run waived; D-2: the
replay after a stop goes to #1182 as an explicit criterion; D-3: agreed, `pekko-actor` coverage verified; D-4: #997 to `status:ready`).
Written on 2026-11-06 against `develop` `04b125eb`.
**Kit run waived (Owner, D-1)**: the unit neither ports nor adapts production code; it pins existing behavior, derived from the oracle
([#1387](https://github.com/getsyntegrity/gopekko/pull/1387)), with tests and docs. **Condition:** if any test fails on the current code and production must
change, the Dev stops and reports to the TL, and the waiver falls (kit run required, with its own manifest, RULEBOOK and RUNBOOK). The Dev is a
separate session, QA is independent of both, and the TL does not implement.

Contract sources: [ADR 0035](../adr/0035-typed-supervision.md) contract 7 (and the restart sequence), [ADR 0034](../adr/0034-typed-behavior-boundary.md),
the scope of [#976](976-typed-restart.md) ("Overlap with #997"), the reconciled criteria in
[typed-supervision-stories-reconciliation-2026-10-26.md](../parity/typed-supervision-stories-reconciliation-2026-10-26.md) (#997 section), and the
new oracle [typed-restart-child-wait-2026-11-06.md](../parity/oracles/typed-restart-child-wait-2026-11-06.md) (W1 to W5; citations `C1` to `C13` below
are its section 1). The "Update (2026-11-05)" of #997 prevails over its original text.

## What #976 already delivered, and what is left

#976 (closed 2026-11-06) delivered `WithStopChildren(bool)` (default `true`) and the default wait (slices 3a-1, 3a-2, 3b: tests R11, R15, R17, R18).
So criteria 1 and 2 of #997 ("`stopChildren` follows the Typed option", "restart waits for stopped children") are **delivered, not re-implemented**.
What stays here is criterion 3 (a stop during the wait cannot publish a new incarnation) and criterion 4 (preserved and stopped child cases beyond
R17 and R18).

**Finding of the oracle and of the probes (the reason this scope is small).** The TL ran the oracle scenarios on `develop` `04b125eb` with throwaway Go
probes (`go test`, not committed). The goPekko result equals Pekko's for the stop during the wait (W1, W2a, W2b, W3a), the indefinite wait (W4),
three children (W5a), a child already stopping (W5b), a Classic child that fails while it is stopped (W5g), a child whose `PostStop` panics
(W5c2), a kept child's `watch` (W5d) and the fixed-name collision with kept children (W5e, W5f). So **#997 expects no production change**: the
interpreter already sets the old behavior to `receiveB{}` before the wait (`interpreter.go:325`), `PostStop` has nothing to deliver, and an internal `stop` drops the held inputs (`interpreter.go:217-221`); after a stop that comes from outside (parent, `PoisonPill`, system) `waiting` stays set, so no `Terminated` is processed and nothing replays. What is missing is the **tests that pin it** and the README sentences. The two differences found are not in this
scope (see "Not carried here").

## What this delivers

Tests that pin, each with a named mutation, the behavior of a Typed restart that waits for its children when the actor is stopped meanwhile, and the
child cases R17 and R18 do not cover; the README that states them. If a test fails on the current code, the Dev stops and
reports to the TL (Mandatory Stop): a fix is then a scope change and the kit-run waiver falls (D-1).

## Not carried here (each with its owner)

| Item | Evidence | Owner | Why it is not in #997 |
|---|---|---|---|
| Restart stash bound, `Dropped` of 1000+ messages, `Dropped(PostStop)` when the stash is full at a stop | oracle W3d, W3e (`C7`, `C8`) | #999 | goPekko holds an unbounded buffer (`interpreter.go:41-45`, `TODO(TYP-26)`), and `Dropped` and the `EventStream` do not exist in `pekko-actor` (ADR 0036). The stop boundary here does not depend on the bound: W1, W2, W3a and W4 are the same with an empty or a full stash |
| Retained messages behind a message that stops the actor during the replay become dead letters | oracle W3c (`C10`, `SB:195`) | #1182 | The only way to stop from a handler is `Stopped`, which does not exist yet; today the replay loop (`interpreter.go:404-425`) would give the remaining inputs to the behavior (probe G3c). It is unreachable through the public API until #1182 and the fix needs a dead-letter route the `typed` package does not have (decision D-2) |
| A Typed child that fails while it is stopped (`ChildFailed` consumed by the wait) | oracle W5c (`C6`) | #1170 and #1346 | needs a Typed spawn and `ChildFailed`; the Classic analogue W5g is carried here (S7) |
| `watchWith` of a child revoked by the restart (`SS:618-660`) and kept by `WithStopChildren(false)` | oracle W5d (`watchWith` half), `C2` | #1171 | needs `WatchWith` from a Typed context |
| A child whose `PostStop` panics does not stop the restart | oracle W5c2; probe G5c2 | none | the behavior belongs to `pekko-actor` (it delivers the `Terminated`); no mutation of the allowed files can make a `typed` test fail (decision D-3) |
| Backoff, a stop during the backoff delay | oracle section 4 | #986 | not run in the oracle |

## Criteria, scenarios, sources and mutations

`S` = `Supervision.scala`, `SS` = `SupervisionSpec.scala`, `AA` = `ActorAdapter.scala` (all under `upstream/pekko`, paths in the oracle). Test files are in `pekko-actor-typed/typed/`. All tests
use go-specs and channels or gates, never sleeps, no `-race`. A mutation is applied to `interpreter.go` in a disposable worktree and never committed
(`qa-review`); the test named in the last column must fail by an assertion.

| # | Criterion | Scenario | Pekko source / oracle | Mutation (what is changed in `interpreter.go`) | Slice |
|---|---|---|---|---|---|
| S1 | If the actor is stopped while the restart waits, `Setup` does not run again and the behavior receives no signal after `PreRestart` (no `PostStop`) | `TestRestartStopDuringWait` rows `Stop` | W1; `S:265-270`, `AA:325-331`, reconciliation criterion 3 | M1a: `PostStop` completes the restart when `i.waiting != nil` (call `prepareRestart`): `Setup` runs. M1b: delete `i.current = receiveB[T]{}` (`:325`): `PostStop` reaches the old behavior | 1 |
| S2 | The same holds when the stop comes from a `PoisonPill` or from the system terminating | the same test, rows `PoisonPill`, `Terminate` | W2a, W2b | M1a, M1b (same test, all rows fail) | 1 |
| S3 | The actor ends only after the awaited children have terminated, and then the watcher gets `Terminated` | the same test, as the order of its log (observation) | W1 (`Terminated(A)` after the last `PostStop`) | none of its own: the order is `pekko-actor` behavior (decision D-3); M1a fails the test through the `Setup` entry in the same log | 1 |
| S4 | Messages and the `Terminated` of a non-awaited actor held during the wait are not delivered when the actor is stopped | `TestRestartStopDuringWait_HeldDiscarded` | W3a; `S:265-270, 280-286` | M4: `PostStop` calls `prepareRestart(ctx, i.waiting.k)` when `i.waiting != nil`: `Setup` runs and the held inputs are replayed to the new incarnation (the same family as M1a; this test fails on the delivered `m1`) | 1 |
| S5 | The wait ends only with the `Terminated` of an awaited child; nothing else ends it and there is no timeout | the existing test `restart_children_test.go:127` (verified, not rewritten) | W4; `S:256-262`, `C11` | M5: `awaited` treats any `Terminated` as the awaited one: the existing test fails on its `done:term` step. The Dev runs and records it | 2 (record only) |
| S6 | With three children the new `Setup` runs only after the last `Terminated`, not after the first or the second | `TestRestartChildren_Several` | W5a; `S:256-262` | M6': `awaited` completes when `len(w.awaiting) <= 1` **and** `childrenToStop` returns nil once (the literal `<= 1` alone is an equivalent mutant: `prepareRestart` re-reads `ctx.Children()` and waits again; verified by Dev and QA of #1390) | 2 |
| S7 | A child that is already stopping when the failure arrives is awaited | `TestRestartChildren_AlreadyStopping` | W5b; `S:362` (`ctx.children` includes it) | M7: `childrenToStop` returns nil (the restart does not wait) | 2 |
| S8 | A Classic child that fails inside a message while the restart is stopping it is not restarted (one start, one `PostStop`), and `Setup` runs after its `PostStop`; the test does not observe the parent's decider (the rig's `causes` sees only the top-level actor), so "not consulted" is inferred from "not restarted" | `TestRestartChildren_ChildFailsWhileStopped` | W5g, W5h (control); `S:421-427` | M8: `waitFor` does not call `ctx.Stop(h)`: the child is not stopped, its failure restarts it (second start observed) | 2 |
| S9 | With `WithStopChildren(false)` a `watch` made by the old incarnation survives the restart: the new incarnation gets `Terminated` when the kept child ends | `TestRestartKeepChildren_WatchSurvives` | W5d (`watch` half); `S:362, 385`, `C1` | M9: the keep path unwatches every child (`ctx.Unwatch`) | 3 |
| S10 | With `WithStopChildren(false)` a `Setup` that spawns a name held by a kept child fails; `Setup` runs 4 times in all under `WithLimit(3, ...)` and the last cause is the spawn error and the parent's decider (answering Stop) is consulted once; with the default (`true`) the name is free and the actor keeps running | `TestRestartKeepChildren_NameHeld` (two rows: `false`, control `true`) | W5e, W5f; `S:362`, `S:335-337` | M10: `childrenToStop` ignores `keepChildren` (stops the children): the `false` row sees a successful second `Setup` | 3 |
| S11 | README states the implemented boundary, the indefinite wait (W4, no timeout), the name collision with kept children (W5), and the known gaps; `README.md` lines 173 and 234 stop listing #997 as pending | review | | none (documentation) | 4 |

## Known gaps stated in the README

| Gap | Closed by | Effect until then |
|---|---|---|
| Restart stash unbounded | #999 | a flood of messages during a wait is kept whole (Pekko: 1000, then `Dropped`; W3d) and the stash is dropped without `Dropped(PostStop)` at a stop (W3e) |
| A retained message that stops the actor during the replay does not stop the replay | #1182 | not reachable through the public API; the messages behind it would reach the behavior instead of dead letters (W3c) |
| Typed child failing during the wait; `watchWith` children | #1170, #1346, #1171 | tested with Classic children only (S8, S9) |

## Slices (one PR each, at most 400 authored lines, sequential)

The raw estimate is the TL's; the second number is the raw one times 1.55, the deviation measured in #976 (slice 1: 590 against ~380; later slices: 279, 249,
391 against re-estimates). Tests count toward the budget. If a slice measures above 400 the Dev splits it before opening the PR.

| Slice | Delivers | Files | Criteria | Est. lines (raw → re-estimated) |
|---|---|---|---|---|
| **1. Stop during the wait** | the shared test helpers (a supervised actor that logs `Setup`, messages and signals; gate children) and the stop tests | `typed/restart_stopwait_test.go` (new) | S1, S2, S3, S4 | ~110 → ~170 |
| **2. Several, stopping and failing children** | S5 verification and the three child tests | `typed/restart_childcases_test.go` (new) | S5, S6, S7, S8 | ~120 → ~185 |
| **3. Kept children** | the `WithStopChildren(false)` cases | `typed/restart_keepchildren_test.go` (new) | S9, S10 | ~95 → ~150 |
| **4. README** | "Supervision and restart": the boundary, the indefinite wait (W4), the fixed-name collision with kept children (W5), known gaps; lines 173 and 234 stop listing #997 as pending | `pekko-actor-typed/README.md` | S11 | ~60 docs (not scaled) |

Order: 1, 2, 3, 4 (four slices; the kit package of the first draft was removed by D-1). Slices 2 and 3 reuse helpers declared in slice 1, so they are sequenced after it; they share no file with each other.

| Pair | Shared file | Resolution |
|---|---|---|
| 1, 2, 3, 4 | none | each slice owns new files; `README.md` only in 4 |
| any slice and `interpreter.go` | `typed/interpreter.go` | **mutation only**; a committed change is Mandatory Stop 1 and goes back to the TL |

Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md` or `AGENTS.md` changes. Internal test files
(`typed/<slice file>_internal_test.go`, `package typed`) are allowed in any slice under the rule of the #976 scope ("Internal test files"), with the same budget.

## Allowed files

Exactly the paths in the slice table, plus `typed/interpreter.go` for mutation only (never committed), plus the internal test files above. Anything
else needs a stop and a report to the TL. This file (`docs/scopes/997-restart-child-wait.md`) is edited only by the planning PR.

## Handoffs after G2 (published as Updates by the TL on 2026-11-06 after approval)

- **#997** (Update): criteria 1 and 2 are delivered by #976; criteria 3 and 4 become S1 to S10 above; `watchWith` children, Typed children and the replay after a stop move to #1171, #1170/#1346 and #1182; `status:blocked` is replaced by `status:ready` (D-4).
- **#999**: add W3d and W3e as evidence (capacity 1000 by default, `Dropped(message, "Stash is full in [RestartSupervisor]", A)` for messages, one more for the `PostStop` signal when the stash is full at a stop; nothing is a dead letter).
- **#1182**: add W3c as a criterion (a retained message that stops the actor ends the replay; the messages behind it are dead letters).

## Decisions (Owner, 2026-11-06)

- **D-1. Kit run: waived.** Reason: the unit neither ports nor adapts production code; it fixes with tests behavior that already exists, derived from the oracle. Condition: if a test fails on the current code and production must change, the Dev stops, reports to the TL, and the waiver falls (kit run required). Slice 0 is removed.
- **D-2. The replay after a stop (W3c) goes to #1182** as an explicit criterion that cites W3 of the oracle (#1387). Not in #997.
- **D-3. No `typed` tests** for a child's throwing `PostStop` or for the order "children end before the watcher's `Terminated`". Verification of `pekko-actor` coverage (TL, 2026-11-06): the order is pinned by `actor/termination_internal_test.go` `TestDescendantsFinalizeBeforeTheirAncestors` (`:94`) and `TestAStoppedParentWithOneChildOutstandingFinalizesOnlyAfterItsSignal` (`:186`); a `PostStop` that panics or ends with `Goexit` still completes the subtree, in order, by `actor/termination_stress_internal_test.go` `TestPostStopPanicAndGoexitAtEveryLevelStillComplete` (`:349`) and `actor/supervision_default_goside_internal_test.go` `TestPostStopPanicEventByPath` (`:360`). No `pekko-actor` test combines a watcher with a panicking `PostStop` (the `Terminated` the watcher receives); this is reported to the Owner and no issue is created.
- **D-4. #997 goes to `status:ready`.**

README (slice 4) must also document, as implemented behavior with its oracle citation: a child that never terminates leaves the restart waiting indefinitely, with no timeout (W4; Pekko defines none, `C11`); and, with `WithStopChildren(false)`, a fixed child name in `Setup` collides in the next `Setup` because the kept child still holds it (W5e, tested by S10).

## Appetite

Four small PRs (three test PRs, one docs PR). No production change is planned; the risk is a probe that passes now and a test that
exposes a race, which the gates (channels, no sleeps) are meant to prevent.

## Definition of Done

- Every criterion S1 to S11 has its test (or, for S5 and S11, its verification and text) and the PR names its mutation in a "Mutations" table, reproduced by QA.
- `scripts/each_module.sh go test -count=1 ./...`, `scripts/check_deps.sh` and the complexity lint green, with the commands in `AGENTS.md`.
- Diff coverage ≥ 85% on new lines (tests only: coverage is not reduced); per-module coverage ≥ 85%; `typed` still imports only `pekko-actor`.
- No committed change to `interpreter.go` or to any other production file.
- Evidence bound to a pushed SHA in each PR, as `dev-scope` requires.
