# Scope: Restart a supervised Typed Behavior within its limit (#976, TYP-10)

Status: **approved by the Owner at gate G2 on 2026-11-05** (decisions D-1 to D-4 below were accepted as written; D-4 as the two restarts it
records). Written the same day against `develop` `bfefdd6`. #976 is `status:ready` (Owner decision, 2026-11-05). QA review of this file: APPROVE at
`8d848e3` ([comment](https://github.com/getsyntegrity/gopekko/pull/1351#issuecomment-6663028124)); the later commit changes only this status text.
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: this scope ports Pekko behavior, so the Dev runs the migration kit
(`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK under `migration/stage-1/typed-976-restart/`. The Dev is a separate session, QA is
independent of both, and the TL does not implement.

Contract sources, all in `develop`: [ADR 0035](../adr/0035-typed-supervision.md) (S-1 to S-6; contracts 2, 3, 6, 15; S-3 cases (a), (b), (c-async);
S-2.6; "Restart sequence"), [ADR 0033](../adr/0033-actor-typed-scope-and-architecture.md) D14 and D15, [ADR 0034](../adr/0034-typed-behavior-boundary.md),
the oracles [S-2](../parity/oracles/s-2-supervision-2026-11-05.md), [S-3 / D-a](../parity/oracles/d-a-restart-setup-unbounded-2026-11-04.md) and
[D-a with children](../parity/oracles/d-a-with-children-2026-11-05.md). The issue's "Update (2026-11-05)" and "Update (2026-11-05, b)" prevail over
its original text.

## What this delivers

A Typed actor whose behavior is wrapped in `Supervise(b).OnFailure(match, Restart())` (or `RestartWithLimit(n, within)`) restarts when a turn fails with a
matching cause: `PreRestart` goes to the old behavior only, the supervised behavior is rebuilt from its initial value (`Setup` runs again, state is
reset), and the cell, ID, mailbox and watches are kept (ADR 0035 "Restart lives in the layer"). Within a limit it restarts; past the limit, or when
the initial start is not retried, the cause is rethrown with the S-2.6 consequences. An unmatched cause falls to the next enclosing supervisor and
finally to the default Stop of #956.

## What this does not deliver

- **c-sync fatal path (D-a, ADR 0035 S-3 case c-sync): out of scope, explicitly.** Closed by #1345. #976 does not depend on #1345. Until then an
  unbounded `Restart()` whose `Setup` fails on every restart, with no children to wait for (or `WithStopChildren(false)`), loops without end and
  without a fatal path. This is a known, temporary deviation (issue "Update (2026-11-05)"). The restart loop is **iterative** (slice 1a; the 100 000-restart test is in 1b) so it never
  reaches Go's stack overflow; it adds no counter, no log, no CoordinatedShutdown and no `os.Exit`.
- **Explicit `Resume`** (#1183), **innermost-match selection and the equal-matcher dedupe** (#1184: the tests of the nested rule; #976 only walks the
  chain so that "unmatched propagates" holds), **backoff** (#986), the rest of #997 (the stop-during-child-wait boundary: "a stop during the wait cannot publish a new incarnation"), **bounded restart
  stash, capacity and `Dropped`** (#999), explicit `Stop` strategy and `Stopped`/`StoppedWith` (#1182), timers cancelled on restart (contract 15, Typed
  timers story), failure logging (contract 13, #1179), Typed spawn/watch from a Typed context (#1170, #1171), mechanism M-A (`ChildFailed` to a Typed
  parent; its stop part, M-A-lite, was delivered in 3c-2 on the Owner's decision of 2026-11-05), the Typed System guardian (#994).

## Known gaps (temporary, stated in the README and in an Update of #976)

| Gap | Closed by | Effect until then |
|---|---|---|
| c-sync fatal path (D-a) | #1345 | unbounded `Restart()` with a `Setup` that fails on every restart and no children to wait for loops without end, iteratively |
| Restart stash unbounded | #999 | messages held while waiting for children are not bounded (Pekko: `restart-stash-capacity`; overflow publishes `Dropped`, ADR 0036) |

## Proposed Go API (for the Owner at G2; names settled in the implementation PR only if the Owner changes them)

Convention found in the repo: a Pekko `Behaviors.x` / `SupervisorStrategy.x` becomes a package-level PascalCase function of `typed`
(`Setup`, `Receive`, `ReceiveMessage`, `Same`, `Unhandled`, `Empty`, `Ignore`, `ReceiveSignal`), and the builder shape follows ADR 0035 "Supervise"
(`Supervise(b).OnFailure(match, strategy)`). There is no written naming rule beyond the ADR.

| Pekko | Go | One-line justification |
|---|---|---|
| `Behaviors.supervise(b)` | `typed.Supervise[T](b Behavior[T]) Supervised[T]` | Same name as Pekko, ADR 0035 spelling; returns a builder, not a `Behavior` |
| `.onFailure[E](strategy)` | `Supervised[T].OnFailure(match Matcher, s Strategy) Behavior[T]` | ADR 0035 fixes the shape; the matcher is a comparable value because Go has no class hierarchy and #1184 needs matcher equality |
| `onFailure[Throwable]` / `[E]` | `typed.AnyError() Matcher`, `typed.ErrorIs(target error) Matcher`, `typed.ErrorAs[E error]() Matcher` | ADR 0035 names them; `errors.Is`/`errors.As` semantics, D-e |
| `SupervisorStrategy.restart` | `typed.Restart() RestartStrategy` | Pekko name; unlimited |
| `restart.withLimit(n, t)` | `RestartStrategy.WithLimit(n int, within time.Duration) RestartStrategy` | Pekko method name kept; **method on the value, so `Restart().WithLimit(...)` reads as `restart.withLimit(...)` and the issue's "Restart.WithLimit" is that method** |
| `.withStopChildren(enabled)` | `RestartStrategy.WithStopChildren(enabled bool) RestartStrategy`, default `true` | Pekko name and default (`SupervisorStrategy.scala:162, 173-174, 199, 215-216`); ADR 0035 names it in the c-sync definition, so it is delivered here (slice 3b). See "Overlap with #997" |
| `SupervisorStrategy` (sealed trait) | `typed.Strategy` (sealed interface, only `RestartStrategy` in #976) | #1183 adds Resume and #1182/#1184 Stop without changing the signature of `OnFailure` |
| `PreRestart` signal | existing `typed.PreRestart` (`signal.go:28`), delivered by this scope | Already declared; nothing delivers it today |

`Stop()` and `Resume()` appear **only** as follow-up constructors of `Strategy` (#1182/#1184 and #1183); this scope adds neither. If the Owner wants
explicit `Stop()` here, it is one line in `supervise.go` and a case in slice 1a; say so at G2 (decision D-1).

## Decisions for G2 (proposed by the TL, accepted by the Owner on 2026-11-05)

- **D-1. Explicit `Stop()` strategy.** Proposed: not in #976 (see above). Alternative: add it in slice 1a.
- **D-2. Restart stash during the wait for children (slice 3a-1).** C1 needs the interpreter to hold messages while the children stop. #999 bounds it
  (capacity, `Dropped`). Proposed: slice 3a-1 holds them in an **unbounded** layer-owned buffer, marked `// TODO(TYP-26)`, and #999 replaces the buffer.
  Alternative: move C1 to a story after #999. **Recorded as a known gap, next to c-sync: "restart stash is unbounded until #999 (Pekko:
  `restart-stash-capacity`, overflow publishes `Dropped`, ADR 0036)".** It goes into the README (slice 4) and into an Update of #976 (2026-11-05, c).
- **D-3. Chain walk.** Proposed: #976 walks the supervisor chain innermost-first so an unmatched cause reaches the outer one (criterion 3). The
  equal-matcher dedupe and its tests stay in #1184.
- **D-4 (settled by the Owner on 2026-11-05 as two different restarts; recorded here).** The TODO at `interpreter.go:168-170` ("#976 replaces this
  body and restores child handling") is about the **runtime `PreRestart` hook**, that is, the **restart by a Classic parent** (mechanism M-B), not
  the Typed restart. Both exist and they treat children differently:
  - **Restart by a Classic parent (S-2.3c, ADR 0035).** The hook mirrors `ActorAdapter.preRestart` (`ActorAdapter.scala:309-314`): cancel timers,
    deliver `PreRestart` to the behavior, replace it by stopped. It **overrides** Classic's `Actor.preRestart`, which is the one that unwatches and stops
    every child (`Actor.scala:625-631`). So the children are **kept** and restarted, and the new `Setup` creates its own: two children after the
    restart (oracle P1). The TODO's "restores child handling" is therefore wrong for this path: **the TODO is corrected, not implemented.** The hook
    delivers `PreRestart` and still does not stop children. Slice 3c-1 replaces the no-op body and the comment; scenario R16 pins that a child survives.
  - **Typed restart (`Restart()`, what #976 implements).** With `stopChildren = true`, the default (`SupervisorStrategy.scala:162, 199`), the supervisor
    computes the children to stop (`Supervision.scala:362`), unwatches, watches and stops each one (`Supervision.scala:363, 421-427`), and if there are
    any it returns `Behaviors.empty` and waits for their `Terminated` (`Supervision.scala:385-388`; the wait itself at `:226`, `:256`); otherwise it
    completes the restart at once (`:385`). C1 depends on that wait. This is slice 3a-1 (3a-2 adds C1); `WithStopChildren(false)` (empty set, no wait) is slice 3b.

### Overlap with #997 (for the Owner; #997 is not edited by this PR)

#997 (TYP-12) lists "Typed stopChildren option and child-wait integration". On the Owner's instruction of 2026-11-05 the option and the default
wait are delivered here because C1 and ADR 0035's c-sync definition depend on them. What stays in #997: the boundary "a stop during the child
wait cannot publish a new incarnation", plus the preserved/stopped child cases beyond R17 and R18. #997's body needs a matching update when this
scope is approved.

## Slices (one PR each, each at most 400 authored lines, as `AGENTS.md` says; all sequential; 1a and 1b merge together)

Estimates are the TL's. **Amendment of 2026-11-05 (slice 1 measured):** the first estimate for slice 1 was ~380 lines and the Dev measured **590**
(+55 %): 284 of production code (not ~200) and 297 of tests. Every later estimate below is the earlier one times 1.55, and any slice that comes out
above 400 is split now. Slice 1 became **1a** (exactly 400 measured, PR #1353) and **1b**. The Dev still reports the real count and splits further if a
slice would pass 400; a margin of about 100 lines under the budget is the target. Test lines count toward the budget; generated files and `go.sum` do not.

| Slice | Delivers | Files | Criteria covered | Est. lines (old → re-estimated) |
|---|---|---|---|---|
| **0. Kit package** | manifest, RULEBOOK, RUNBOOK | `migration/stage-1/typed-976-restart/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | kit run (AGENTS.md) | ~150 docs → **144 measured** (PR #1352) |
| **1a. Wrapper and restart of case (a), unlimited** | `Supervise`/`OnFailure`/`Matcher`/`Strategy`/`Restart()`; the interpreter keeps a supervisor chain (`append`); a failed turn that matches restarts in a loop: `PreRestart` to the old behavior only, rebuild from the initial behavior; unmatched falls outward then to default Stop | `typed/supervise.go` (new), `typed/behavior.go`, `typed/interpreter.go`, `typed/supervise_test.go` (new), **`typed/unhandled_internal_test.go`** (exported-surface list U9) | R1, R2, R3 (default Stop), R4 (matching), R15 (ordering), nil strategy, a `Setup` that panics or fails with an unmatched cause during the restart stops the actor | ~380 → **400 measured** (PR #1353) |
| **1b. Equal-matcher replacement and the chain tests** | `joinLayers`/`Matcher.equal` (an equal matcher replaces the enclosing layer, so a handler that returns a fresh `Supervise` does not grow the chain); tests for nested supervisors, the restart loop and `PreRestart` failures. Merge together with 1a | `typed/interpreter.go`, `typed/supervise.go`, `typed/supervise_test.go`, `typed/supervise_internal_test.go` (new: the chain is unexported state) | R3b (inner unmatched, outer matched), R10 (100 000 consecutive restarts, bounded stack), a `PreRestart` handler that errors or panics does not stop the restart, chain-growth guard | n/a → ~190 |
| **2a. `WithLimit` window** | `RestartStrategy.WithLimit(n, within)`: the window is counted from the first failure, reset on expiry, rethrow past the limit; a clock seam local to the module for the window tests | `typed/supervise.go`, `typed/restart_limit.go` (new), `typed/interpreter.go`, `typed/restart_limit_test.go` (new), `typed/restart_limit_internal_test.go` (new: the window with an injected clock), **`typed/unhandled_internal_test.go`** | S-3 (a) with a limit: R5, R6 | (half of 360) → ~280 |
| **2b. Initial start under a limit** | the start failure rules of S-3 (b) as corrected on 2026-11-05: unlimited does not retry; `WithLimit(n)` with n ≤ 1 runs `Setup` once, n = 2 three times (P4a), n = 3 four times; the actor ends stopped; S-2.6 case 1 | `typed/interpreter.go`, `typed/restart_limit.go` (`retryStart`), `typed/restart_limit_test.go` | R7, R8, R9 (Classic parent not consulted, `Terminated(child)`) | (half of 360) → ~280 |
| **3a-1. Wait for children, default `stopChildren = true`** | the Typed restart stops the children (`Unwatch`, `Watch`, `Stop`), waits for their `Terminated` on the wrapped context and holds the messages meanwhile in an unbounded buffer (D-2, known gap until #999) | `typed/interpreter.go` (the wait and the held buffer stayed there), `typed/restart_children_test.go` (new) | R17 (default waits), R15 during the wait | (part of 340) → ~310 |
| **3a-2. C1** | the asynchronous loop of ADR 0035 sub-case C1: `Setup` creates a child and fails, the restart waits and loops, no fatal error, at most one live child; `Terminated` of actors that are not awaited during the wait | `typed/interpreter.go`, `typed/restart_children_test.go` | R11 (C1); c-sync only as a loop, no fatal path (#1345) | (part of 340) → ~220 |
| **3b. `WithStopChildren(bool)`** | the option on `RestartStrategy`; `false` has an empty set of children to stop, so no wait and the children are kept | `typed/supervise.go`, `typed/interpreter.go`, `typed/restart_stopchildren_test.go` (new), **`typed/unhandled_internal_test.go`** | ADR 0035 c-sync definition; R17 default `true` waits, R18 `false` keeps the children and restarts at once | ~150 → ~230 |
| **3c-1. Runtime `PreRestart` hook (D-4)** | body of the runtime hook: delivers `PreRestart`, does **not** stop children; the TODO at `interpreter.go:168-170` corrected | `typed/interpreter.go`, `typed/restart_rethrow_test.go` (new), `typed/signal_runtime_test.go` (TestSig_S6: the old instance now gets `PreRestart`) | D-4; R16 a restart by a Classic parent keeps the child and `Setup` creates a second one | (part of 290) → ~170 |
| **3c-2. Rethrow after the start (P4b/P4c)** | S-2.6 case 2: a rethrow after the start returned goes to the Classic parent's decider and the new instance starts with count 0; **added on 2026-11-05 on the Owner's decision (the Owner's instruction in the session of 2026-11-05, "Opción 2, con estos límites", which is not on GitHub; the Update of #1346 records it only as "Owner decision, 2026-11-05"). The author, checking the Typed-parent case that the Owner had asked for, found that a child of a Typed parent was restarted by the runtime's default strategy once its rethrown cause reached the runtime: a CRITICAL introduced by this slice. One clean-context review had approved the head without testing that case. The Owner chose M-A-lite (the stop part of M-A: a Typed parent stops a Typed child) and the W1 fix (a match-all layer anywhere in the chain replaces the default wrapper)** | `typed/interpreter.go`, `typed/restart_rethrow_test.go` | P4b (9 runs of `Setup`, 2 consultations), P4c (6 runs, 1), S-2.6 general rule; LT limited by #954/#1170 | (part of 290) → ~280 |
| **4. Docs, record and the citation audit** | README (implemented vs planned, c-sync gap, D-2 buffer), `EXECUTION-RECORD.md`, cost-log row; the re-verification of Pekko citations below (splits per "Size" below if it passes 400 lines) | `pekko-actor-typed/README.md`, `migration/stage-1/typed-976-restart/EXECUTION-RECORD.md`, `migration/cost-log.tsv`, and only where the audit finds a mismatch: the document that carries the wrong citation | "update the public contract and traceability"; SHA and gates; every Pekko citation of the documents listed under "Citation audit of slice 4" verified or corrected | ~120 docs (not scaled) plus the corrections |

Order: 0, 1a, 1b, 2a, 2b, 3a-1, 3a-2, 3b, 3c-1, 3c-2, 4 (eleven slices; the PRs that delivered them are listed in the execution record). **1a and 1b merge together** (1a alone lets the chain grow). Slices 1a to 3c-2
are planned to edit `typed/interpreter.go`, so they are **sequential**, never parallel (the independence rule is met by sequencing, as in #956); `typed/unhandled_internal_test.go`
is edited by 1a, 2a and 3b only, because those are the PRs that add exported symbols (U9 pins the exported surface). In slices 0 to 3c-2 nothing under `pekko-actor/`,
`docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md` or `AGENTS.md` changes (slice 4 corrected citations in ADRs 0035, 0036 and 0037 and in `docs/parity/`, #1369); **no `pekko-actor` change is expected** (the wrapped context already has `Stop`,
`Watch`, `OnTerminated`, `Children`, `StopSelf`). If the Dev finds one is needed, that is Mandatory Stop 1 and goes back to the TL.

### Measured sizes (authored lines = additions plus deletions of each PR, from `gh pr view <n> --json additions,deletions`)

| Slice | PR | Estimate | Measured |
|---|---|---|---|
| 0 kit package | #1352 | ~150 | 144 (docs) |
| 1a / 1b | #1353 / #1355 | ~380 for the unsplit slice | 400 / 155 |
| 2a / 2b | #1357 / #1360 | ~360 for both, re-estimated ~280 each | 224 / 174 |
| 3a-1 / 3a-2 | #1362 / #1363 | ~340 for both, re-estimated ~310 / ~220 | 279 / 180 |
| 3b | #1364 | ~150, re-estimated ~230 | 249 |
| 3c-1 / 3c-2 | #1367 / #1368 | ~290 for both, re-estimated ~170 / ~280 | 158 / 391 |
| fixes found on the way | #1358, #1361 | n/a | 31, 22 |
| 4 | #1369, #1370 | ~120 (docs) | 47, 112 |

The estimates of the slices that shared one number are written once for the pair. The README and the record of slice 4 (#1371) and the later follow-ups are in the execution record.

## Criteria → scenario → Pekko source

| Criterion (issue "Update" sections) | Scenario (`pekko-actor-typed/typed/`) | Pekko source / oracle | Slice | Level |
|---|---|---|---|---|
| Restart reruns the supervised setup; state reset | R1 a failing turn restarts, `Setup` count 2, state back to initial | `SS:512`; `S:392-404` | 1a | unit |
| `PreRestart` to the old behavior only, no `PostStop` | R2 the old behavior's handler sees `PreRestart` once, never `PostStop`; the new one sees neither | `S:318-319, 327`; `TT/ActorContextSpec.scala:255` | 1a | unit |
| Unmatched failure propagates outward | R3 non-matching cause under an outer matching supervisor restarts through it; with none, default Stop and the parent's decider is not consulted | `SS:214`; ADR 0035 matrix | 1a (R3b nested: 1b) | unit |
| Failure matching with Typed semantics | R4 `ErrorIs`, `ErrorAs`, panic as `*actor.PanicError`, `runtime.Goexit` never caught | ADR 0035 "Supervise", D-e | 1a | unit |
| Limit: window and rethrow | R5 n failures inside the window restart, the n+1th is rethrown (with a match-all layer after the start it reaches the parent's decider, slice 3c-2; the actor stops with a narrower matcher, inside the start, or when the parent's decider answers Stop); R6 an expired window resets the count | `SS:528, 550`; `S:335-337, 430-440` | 2a | unit |
| S-3 (b): first `Setup` fails, unlimited | R7 `Setup` ran once, no retry | S-3 oracle 1, 2; `S:302-304` | 2b | unit |
| S-3 (b): first `Setup` fails, `WithLimit(n)` n = 0–3 | R8 runs of `Setup`: 1, 1, 3 (P4a), 4 (S-3 oracle 3); always stopped, system alive | S-2 oracle 6.1 (L); S-3 oracle 3; `S:302` | 2b | unit |
| L/LT: Classic parent not consulted; Typed parent watching | R9 Classic parent decider count 0 and `Terminated(child)`; the Typed-parent side is limited by what #954 and #1170 provide (see slice 3c-2) | S-2 oracle L, LT | 2b, 3c-2 | unit |
| Classic-parent restart keeps children (D-4) | R16 a restart by a Classic parent delivers `PreRestart`, does not stop the child, and `Setup` creates a second one | S-2 oracle P1; `ActorAdapter.scala:309-314`; `Actor.scala:625-631` | 3c-1 | unit |
| `WithStopChildren` default `true` | R17 the restart stops the child and waits for its `Terminated` before `Setup` runs again (this is C1's wait) | `Supervision.scala:362-363, 385-388, 421-427` | 3a-1, 3b | unit |
| `WithStopChildren(false)` | R18 the child is not stopped, no wait, the restart completes at once | `Supervision.scala:362, 385` | 3b | unit |
| Iterative restart | R10 100 000 consecutive restarts with no children complete with bounded stack | ADR 0035 S-3 amendment | 1b | unit |
| c-async C1 | R11 `Setup` creates a child then fails: the restart waits, the loop continues, the system stays alive, at most one live child | D-a with children oracle, C1 | 3a-2 | unit |
| P4b | R12 9 runs of `Setup`, decider consulted twice (`boom` → Restart, `setup-boom` → Restart), actor ends stopped | S-2 oracle P4b, 6.2 | 3c-2 | unit |
| P4c | R13 6 runs, 1 consultation | S-2 oracle 6.2 | 3c-2 | unit |
| General rule S-2.6 | R14 decider consulted only if the rethrow happens after the start returned | ADR 0035 S-2.6 | 3c-2 | unit |
| Ordering/race boundary | R15 a message queued behind the failing one is handled by the new incarnation, in order, not before the restart completes | `S:254-286`; `SS:882` | 1a, 3a-1 | unit |
| Public contract and traceability | README and execution record | | 4 | review |

All scenarios use go-specs and channels or gates, never sleeps, and no `-race` unless the user asks. A scenario that fails and is not a Go defect is
run on Pekko before it is classified (`AGENTS.md`). The runtime is the real one: these are unit tests in the module, with no real resource and no driver.

## Citation audit of slice 4 (Owner decision, 2026-11-05)

Several reviews of the earlier slices and of the ADR chain ran in a worktree whose `upstream/pekko` submodule was empty, so a Pekko citation may have been
judged without being opened. Slice 4 re-verifies, **from the main checkout at v1.7.1 / `9a54d12`** (or from a worktree whose submodule was initialized with
the command in the `qa-review` skill), every Pekko citation (`file:line` or `file:line-line`, and the spec names they point to) of:

- [ADR 0035](../adr/0035-typed-supervision.md), [ADR 0036](../adr/0036-typed-event-stream-and-interceptors.md) and [ADR 0037](../adr/0037-port-as-a-go-adaptation.md);
- the oracles and audits under `docs/parity/oracles/` and `docs/parity/audits/`.

The slice reports the citations that do not match (file, line, what the text claims, what the line says). The ADRs also cite Pekko in short forms
(`S:198-203`, `SS:1060`, `SO:82`, `AA/...`, `T/...`; `S`, `SS`, `SO` are defined in the header of ADR 0035 and `T`, `AA`, `TT` in the header of ADR 0034): the audit resolves those too. Each mismatch is corrected in this same
slice, in the document that carries it, and recorded where that kind of document records changes:

| The document | A citation fix that leaves the claim unchanged | A correction that changes the text of an accepted decision |
|---|---|---|
| ADR 0035 | corrected in place; one dated line in its `## Revision log` | a dated `## Amendment (...)` section, **and** one dated line in a new `## Resolutions (added <date>)` table of `docs/parity/typed-owner-decisions-2026-10-27.md` (the Owner's table, where the Owner decisions of the ADR are recorded) |
| ADR 0036 | corrected in place; a short dated `## Amendment (...): citation corrections` section (the ADR has no revision log) | a dated `## Amendment (...)` section and the Owner-table line, as above |
| ADR 0037 | corrected in place; a short dated `## Amendment (...): citation corrections` section (the ADR has neither a revision log nor amendments yet) | the same |
| an oracle or an audit under `docs/parity/` (evidence, not a decision) | corrected in place; a final line `Correction (<date>): <old> -> <new>` | the same line, plus an Amendment of the ADR that relies on it if the conclusion changes |

The audit changes no behavior, no code and no accepted decision on its own: a correction that would change what an accepted ADR *decides*, not only what it cites,
is a finding that stops the slice and goes to the Owner. **Size:** the 400-line budget of `AGENTS.md` applies to slice 4. If the corrections, with the README and the
record, would pass it, slice 4 splits: 4a for the README, the record and the cost-log row, then one slice per document family (ADR 0035, ADR 0036 and 0037, oracles and
audits). The citations of this scope, its kit package and `pekko-actor-typed/` were already re-verified on 2026-11-05 (about 94 occurrences, no mismatch) and are
not repeated.

## Allowed files

Exactly the paths in the slice table (including `typed/unhandled_internal_test.go` in 1a, 2a and 3b, `typed/signal_runtime_test.go` in 3c-1 for the one test its change made stale, and, in slice 4, the document that carries a wrong citation).
**Internal test files (amendment of 2026-11-05).** Any slice may also add one `typed/<slice file>_internal_test.go` (`package typed`) next to
its test file, when the behavior it proves lives in unexported state that the exported API cannot observe (the supervisor chain, the restart
window and its clock, the child-wait state). It is a test file only, it never touches production code, and its lines count toward the same
**400-line** budget. Reason: slices 1b and 2a each needed one (`supervise_internal_test.go`, `restart_limit_internal_test.go`); listing them one by one
kept producing the same QA deviation. Anything else needs a stop and a report to the TL. Also edited by this scope: `docs/scopes/976-typed-restart.md`
(this file, PR 0 of the planning).

## Appetite

Eleven slices; the PRs that delivered them, with follow-up fixes and amendments, are listed in the execution record. 1a and 1b merged together; 3a-1 was the riskiest (waiting for children, D-2). The kit
run adds the manifest of slice 0 up front.

## Definition of Done

- Every criterion above has its scenario, passing; the c-sync gap is stated in the README and not claimed as parity.
- `scripts/each_module.sh go test -count=1 ./...`, `scripts/check_deps.sh` and the complexity lint green, with the commands in `AGENTS.md`.
- Diff coverage ≥ 85% on new/modified lines; per-module coverage ≥ 85%; cyclomatic complexity ≤ 10 in every new/modified function.
- `typed` still imports no goPekko module besides `pekko-actor`.
- Every ported file keeps one `Ported from` line; new files ported from `Supervision.scala` / `SupervisorStrategy.scala` carry their header.
- Evidence bound to a pushed SHA in the PR, as `dev-scope` requires.
