# RULEBOOK: Supervision U3, default strategy and failure classification (#448, unit supervision-u3)

Status: PROPOSED. Nothing here runs until the Owner approves the hashes in RUNBOOK section 9. Written for `migration-kit` pinned at `cf91c9d`.
Source of truth: `docs/adr/0017-supervision.md` (S2, S12, A3, A7, A9a/b/c, H5c, the U3 row) and Apache Pekko Classic v1.7.1 (`upstream/pekko`, read-only).
Base: `develop` at the squash of U2 (`0bbc65d`, tree `f9ffbe86`). U2 is merged; its open gaps (#173) are not this unit's.

## 0. Unit

Delivers, per the ADR U3 row: the **default strategy and the default decider** for every parent that is an actor and declares none (or returns a zero `Strategy[T]`), and the **`ErrGoexit` classification** (default Stop). Never Escalate by default.

Pekko evidence (verified by a read-only mapping): `Actor.scala:588` (`supervisorStrategy = defaultStrategy`), `FaultHandling.scala:218-232` (`defaultDecider`: `ActorInitializationException`, `ActorKilledException`, `DeathPactException` -> Stop; any other `Exception` -> Restart; `defaultStrategy = OneForOneStrategy()(defaultDecider)`, `maxNrOfRetries = -1`, `withinTimeRange = Inf`), `Actor.scala:658` (`Terminated` unhandled -> `DeathPactException`).

## 1. Decisions this RULEBOOK asks the Owner to confirm by approving its hash

| # | Decision | Why it needs the Owner |
|---|---|---|
| D1 | `DefaultStrategy[T]()` is exported: `OneForOne`, `Budget{}` unlimited (no invented limit, S2), and the default decider of S12. A parent that does not implement `Supervisor[T]`, or whose `SupervisorStrategy()` returns the zero `Strategy[T]`, behaves exactly as `DefaultStrategy` (A3). | Public API (already listed in the ADR signatures); the zero-strategy rule is defensive, Pekko has none. |
| D2 | The default applies only to parents that are **actors with a parent chain below `/user`-level cells that exist today**. A top-level actor (no parent) keeps the U1/U2 interim "stop" until the guardians (U5). | Keeps U5 out; `failUpExcept` already returns false without a parent. |
| D3 | `ErrDeathPact` becomes a failure **reported to the parent's decider** (default Stop) for a child watcher, instead of the direct `acceptStop` in `watch.go`. A top-level watcher keeps stopping directly. Same for an `OnTerminated` that fails. | Changes the event sequence of the death-pact tests; ADR S12 row and `DeathWatchSpec:216`. |
| D4 | `ErrGoexit` is exported and **replaces the unexported value**, so `errors.Is` keeps working. A `runtime.Goexit` in `Receive`, `PreStart`, `PreRestart`, `PostRestart` or the decider is a failure that reaches the decider with `ErrGoexit`; default Stop; a decider that chooses Restart restarts and **the activation continues in a fresh goroutine without losing a message** (ADR H5c). Goexit in `PostStop` stays `poststop-panic`. **A Goexit inside the decider is final: the failing cell is stopped with `ErrGoexit` and the decider is NOT called again for it** (no re-entry, no loop). | This is the riskiest part: today `abortGoexit` stops the cell directly. See section 2.3. |
| D5 | The **factory** Goexit row is NOT delivered here: it belongs to U4 (`Spawn` reconciliation, #444). Declared `PORT-GAP(factory-goexit-U4)`. | ADR 0017:729, 516. |
| D6 | The judge scenarios S13 and S14 are **not mirrored in the Go gate by this unit** (`judge/`, `judge_internal_test.go` and the stale parity text stay untouched). U3 only supplies the behavior they need and Go-side tests of the same shape. The mirror and the doc fixes belong to U8 (#175). Nothing is claimed as judge coverage until U8 runs them. | Issue #448 says "completed with #175"; the Owner confirms the split. |
| D7 | `SupervisorMiscSpec:87` and `ActorLifeCycleSpec:142` need the top-level guardian: `PORT-GAP(top-level-default-U5)`. `DeathWatchSpec:216` is ported adapted (the stop is observed through `Terminated`; Kill is a `Receive` error; no strategy subclass). | ADR dependency map (716, 724). |
| D8 | Handle-divergence rows (`ErrInvalidHandle` of another System, #443) and the A9c rows keep their interim tests and are not reclassified. | ADR H5b, A9c. |

## 2. Behavior specification (the implementer follows it; it is not open design)

### 2.1 Classification (A9a, A9b) with the default decider

| Go cause | Upstream class | Directive by default | Evidence |
|---|---|---|---|
| error or panic of `Receive` (`panic(nil)` included) | `Exception` | Restart | `cell.go` callReceive; `FaultHandling.scala:222` |
| failure of `PreStart` of the first instance | `ActorInitializationException` | Stop | `ActorCell.scala:642-683` |
| failure of the new instance / `PostRestart` after a restart | `PostRestartException` | Stop | ADR S10; `InitializationError{InRestart: true}` |
| `ErrDeathPact` | `DeathPactException` | Stop | `Actor.scala:658`, `DeathWatchSpec:216` |
| `ErrWatchConflict` | `IllegalStateException` | Restart | `DeathWatch.scala:125-131` |
| nil handle in `Watch` | `IllegalArgumentException` | Restart (in `Receive`); Stop in `PreStart` | `DeathWatch.scala:47-48,63-64,77-78` |
| `ErrGoexit` | none (Go adaptation, H5c) | **Stop** | accepted by the Owner on 2026-10-14; no JVM parity; it authorizes no change to another row |

Never Escalate by default. The default decider has no "undefined" outcome: a cause it does not know is an ordinary failure, Restart. A user decider may choose any directive.

### 2.2 Default strategy
`DefaultStrategy[T]()` = `OneForOne` + unlimited budget + default decider. `handleFailed` must stop treating `Strategy{}` and "no `Supervisor`" as the interim stop for a parent that has a parent chain; it uses the default instead. The check for `declares()` stays for U5 top-levels.

### 2.3 Goexit continuation (the risk to review first)
`runtime.Goexit` ends the goroutine that runs the user hook. Today `abortGoexit` accepts the stop directly. For D4 the cell must (a) report the failure with `ErrGoexit` through the same path as an ordinary failure, (b) let the decider choose, and (c) when the directive is Restart or Resume, continue the activation on a new goroutine so no queued message is lost and the cell's counters (suspension, `sysq`, mailbox) stay balanced. A Goexit raised by the decider itself is never given to the decider again: it ends in Stop. If the design review finds this cannot be done without changing U1/U2 invariants, the implementer **stops and reports**; it does not weaken the invariants. Fallback the Owner would have to approve: Goexit reaches the decider with `ErrGoexit` but a Restart/Resume choice is treated as Stop, declared `PORT-GAP(goexit-continue)`.

### 2.4 Events and observability
Events keep their names and payload from U1/U2. The default path may emit the same `decision` facts a declared strategy emits; no new event names are invented. `loggingEnabled`/`logFailure` stay out (U9).

## 3. Out of scope (do NOT implement or test)
Guardians and top-level defaults (U5); `Spawn` reconciliation and the factory Goexit row (U4); `AllForOne` (U6); `PostStop` double call (U7); judge mirrors and docs of S13/S14/S16 (U8); user-defined strategies (#450); logging (U9); the cross-System handle divergence (#443); any change to `Budget`, the restart protocol, or the U2 hooks beyond what D3/D4 require. No change to `judge/`, hooks or workflows.

## 4. Conventions
Same as U2 (`supervision-u2/RULEBOOK.md` section 4, immutable at its merged hash): module layout, `go-specs` tests only, `kit-logger`, headers on ported files, `errors.Is/As`, no `//nolint`, gocyclo at most 10, coverage at least 85%, gofmt/vet/lint/tidy. Test files are new (`supervision_default_*`) plus the minimal edits to existing ones listed in the allowed surfaces. A PORT-GAP is declared in the file header. Message values avoid the `sup1`/`sup2` constants.

## 5. Required scenarios (each with an assertion that fails if the behavior breaks)

### 5.1 Upstream cases
| Id | Source | Go scenario |
|---|---|---|
| P1 | `DeathWatchSpec.scala:216-250` ("fail a monitor which does not handle Terminated") | ADAPTED: a child watcher without `OnTerminated` fails with `ErrDeathPact`, the default decider stops it, the stop is observed through `Terminated`; Kill is a `Receive` error; the spy and the strategy subclass are dropped and declared. |
| P2 | `SupervisorMiscSpec.scala` (the default-strategy case near :87) | `PORT-GAP(top-level-default-U5)`, header only. |
| P3 | `ActorLifeCycleSpec.scala:142` | `PORT-GAP(top-level-default-U5)` and `become` PORT-GAP, header only. |

### 5.2 Go-side scenarios (derived from the ADR rows and the cited lines; no Pekko run, no parity claimed)
- **T1** (judge S13 shape): a parent that declares nothing; a watched child fails in `Receive`; the child is **restarted** (new instance, same Ref), the watcher gets no `Terminated`, and a later message `R` is processed by the second instance once.
- **T2:** a parent returning `Strategy[T]{}` behaves as T1.
- **T3:** a parent that declares its own strategy is unaffected (a Stop decider still stops).
- **T4** (judge S14 shape): a child whose first `PreStart` fails, under a parent that declares nothing: the child is stopped, the watcher gets `Terminated`, no second `PreStart`.
- **T5:** failure of `PostRestart` / the new instance under the default: Stop (`InitializationError{InRestart: true}` reaches the decider).
- **T6:** `ErrWatchConflict` in `Receive` -> Restart; a nil handle in `Watch` inside `Receive` -> Restart; inside `PreStart` -> Stop (Go-side, inferred rows are labelled).
- **T7:** no invented limit: 50 consecutive failures under the default never stop the child and never open a window.
- **T8:** the default decider never returns Escalate: an unknown cause (`errors.New`) is Restart.
- **T9 (Goexit rows):** `Goexit` in `Receive`, `PreStart`, `PreRestart`, `PostRestart` and the decider each reach the decider with `errors.Is(cause, ErrGoexit)`; default Stop; a decider choosing Restart restarts and **no message is lost** (count messages sent before and after); a decider choosing **Resume** after a Goexit in `Receive` continues with the next message, loses none, and leaves the suspension, `sysq` and mailbox counters at zero; a decider that itself calls `runtime.Goexit` on every call stops the cell after one invocation (the test asserts termination within a bound and that the decider ran once); a test actor calling `t.FailNow` (a Goexit) follows the same rule.
- **T10:** `Goexit` in `PostStop` stays `poststop-panic` (the existing `stop_goexit_internal_test.go` is not edited and must keep passing).
- **T11:** a top-level actor still stops on failure (interim until U5) and says so in a comment.
- **T12:** a parent whose strategy decider chooses Escalate (declared) is unaffected by the default.
- The existing assertions of `TestSup1Interim` (V14) and of the `errGoexit` detail in `lifecycle_internal_test.go` change as listed in the RUNBOOK; each changed assertion is explained in a comment and recorded as a manual change.

### 5.3 Mutant backlog for the module close (NOT run in this unit, policy of 2026-10-15; RUNBOOK 9)
The list below is kept as the backlog the module-close campaign runs on immutable snapshots. U3 itself runs no mutation campaign; only a concrete risk or failure justifies one extra specific check.
M1 default is Escalate. M2 default Stops on an ordinary `Exception`. M3 init failure is Restart. M4 death pact is Restart. M5 `ErrGoexit` default is Restart. M6 zero `Strategy[T]` keeps the interim stop. M7 the default invents a limit (`Budget{3, ...}`). M8 `ErrWatchConflict` is Stop. M9 the default applies to a top-level actor. M10 Goexit does not reach the decider. M11 `errors.Is(err, ErrGoexit)` is false (the value is wrapped away). M12 Goexit in `PostStop` is reclassified as a decider failure. M13 a Restart after Goexit loses the next message. M14 a Resume after Goexit loses the next message or leaves a counter above zero. M15 a Goexit inside the decider re-invokes the decider. A mutant that does not compile, times out, or is a harness error is INVALID, not a detection. Each survivor is reported with the test that closes it; none is called "equivalent" without proof.

## 6. Model plan, limits and token definition (PROPOSED; the Owner approves in RUNBOOK)
See RUNBOOK section 3.

## 7. Deviation log
- **DV1, one reviewer per unit** instead of the kit's two adversarial reviewers (kit standing rule 4). Owner decision of 2026-10-15; the kit (`migration-kit/`) stays unchanged. The integral review moves to the module close.
- **DV2, no permission probe in U3** (weekly quota). The restrictive permissions stay; the modified files are checked after the run with `git diff --stat` and `shasum`. This is a post-hoc scope control and does **not** prove isolation.
- **DV3, mutation campaign, parity judge, evidence tag, complete `-race`, stress and integral review moved to the module close.** U3 runs the normal suite and one targeted `-race` over the Goexit and continuation tests.
- Other entries are written at gates.
