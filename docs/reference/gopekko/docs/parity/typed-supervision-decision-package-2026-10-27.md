# Decision package: ADR 0035, Typed supervision, and `withLimit` under a Classic parent (2026-10-27)

> **2026-10-27:** the current decision list and acceptance wordings are in [typed-owner-decisions-2026-10-27.md](typed-owner-decisions-2026-10-27.md); the findings below were applied to the ADR text in commit `99edce8`.

> **Resolution 2026-11-04:** S-2 to S-5 are decided in the ADR 0035 amendment (S-3 is not accepted as a divergence; it follows the fixed Pekko under ADR 0033 D15 for an actor with no children at restart time, the variant with children being To be verified; D-f is withdrawn).
> See "Resolutions (added 2026-11-04)" in [typed-owner-decisions-2026-10-27.md](typed-owner-decisions-2026-10-27.md). The text below is kept as written.

**Status: documentation only.** This package gives the Owner the questions, the Pekko evidence and the acceptance wording needed to
decide ADR 0035. It accepts nothing. ADR 0035 stays **proposed** and is not edited here. ADR 0034 has its own package
([typed-behavior-decision-package-2026-10-27.md](typed-behavior-decision-package-2026-10-27.md)); accepting one does not accept the other.

Baselines: goPekko `develop` `77a923783987ea28b959862f4732c136d4c1847a` (ADR 0033 and ADR 0017 accepted there); **reviewed text:** ADR
0035 as in commit `cd7c61c` (head of draft PR #1212, stacked on #1211 and #1210); Apache Pekko v1.7.1 `9a54d12`. No Owner decision on ADR
0035 is recorded after its proposal.

Path prefixes: `T` = `upstream/pekko/actor-typed/src/main/scala/org/apache/pekko/actor/typed`, `S` = `T/internal/Supervision.scala`,
`AA` = `T/internal/adapter/ActorAdapter.scala`, `SO` = `T/SupervisorStrategy.scala`, `TT` = `upstream/pekko/actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed`,
`SS` = `TT/SupervisionSpec.scala`, `CSTS` = `TT/coexistence/ClassicSupervisingTypedSpec.scala`, `TSCS` = `TT/coexistence/TypedSupervisingClassicSpec.scala`,
`C` = `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor`, `CFH` = `C/FaultHandling.scala`, `DFH` = `C/dungeon/FaultHandling.scala`,
`G` = `pekko-actor/actor`. Evidence levels: CODE, SPEC (assertion opened), EXEC (none exists here), INFERENCE. Every `SS` test spawns from
the test kit's Typed guardian, so it shows the Typed-parent case (M-A).

Default stance, set by the Owner: keep Pekko v1.7.1 observable behavior; a missing spec does not justify changing it.

## Answer first

- **`withLimit` under a Classic parent: keep Pekko (alternative A).** The behavior ADR 0035 describes is real in Pekko's code, with three
  conditions it omits (section 1). No Go impediment was found: the runtime already restarts an actor through `PreRestarter` and
  `PostRestarter` hooks and restarts surviving children. The earlier recommendation to stop the actor followed Pekko's Scaladoc
  (`SO:263`, "the child actor is stopped"), not its code, and is withdrawn.
- **Most contracts are equivalences with Pekko** (S-1). Two labels are wrong: D-b and D-d are Pekko's own behavior, not Go differences, and
  D-f contradicts an assertion of a Pekko test (`SS:1049-1051`), so it is a divergence (S-4).
- **One divergence has a concrete impediment:** D-a. Pekko ends in a `StackOverflowError` that exits the JVM by default; Go cannot recover
  a stack overflow either, so goPekko stops the actor instead (S-3).
- **The mixed hierarchy matrix needs corrections:** two rows cite tests with custom strategies, and the Typed-parent, Classic-child rows
  hide that Pekko restarts a non-fatal `Error` there, where a Classic parent escalates (section 3).

## 1. `withLimit` under a Classic parent: the chain in Pekko

Case: a Typed actor is spawned by a Classic parent (mechanism M-B) as `Behaviors.supervise(b).onFailure(SupervisorStrategy.restart.withLimit(n, within))`,
and failure n+1 inside the window happens while it handles a message or a signal.

| Step | What happens | Source |
|---|---|---|
| 1 Spawn | The Classic spawn APIs (`context.spawn`, `system.spawn`, `PropsAdapter`, `systemActorOf`) wrap the behavior and use `rethrowTypedFailure = false` | `T/scaladsl/adapter/package.scala:60,74,114`; `T/internal/adapter/ActorSystemAdapter.scala:127-133` |
| 2 Default wrapper | The wrapper is `supervise(behavior).onFailure(SupervisorStrategy.stop)`; with no type argument the class is exactly `Throwable` | `T/Behavior.scala:82-85` |
| 3 User strategy | `onFailure` without a type argument is also `Throwable`; the supervisor is an interceptor created anew on each start | `T/scaladsl/Behaviors.scala:256-258`; `S:45-63`; `T/internal/InterceptorImpl.scala:37-41` |
| 4 Replacement rule | `isSame` is true when both supervisors have the **same** class (equality, not subclass; any strategy kind). At start, the outer interceptor is dropped if an equal one exists anywhere in the started stack, so the user's `Throwable` supervisor replaces the wrapper, at any depth | `S:78-83`; `T/internal/InterceptorImpl.scala:106-122`; `T/Behavior.scala:218-230`; `SS:1283-1307` (innermost kept) |
| 5 Matching | `isAssignableFrom` on the unwrapped cause, only for `NonFatal`; fatal throwables bypass every Typed supervisor | `S:75-76, 146, 163, 297, 333` |
| 6 Exhaustion | Past the limit inside the window, `Restart` rethrows the raw cause without logging (`Backoff` returns `failed` instead). `ActorAdapter` does not catch it, so the cause reaches the Classic parent as `Failed` | `S:332-351, 430-441`; `AA:125-162`; `DFH:215-234` |
| 7 Parent strategy | Default `OneForOneStrategy`, unlimited: `Exception` gives Restart; `ActorInitializationException`, `ActorKilledException`, `DeathPactException` give Stop; any other non-fatal throwable gives Escalate. A custom parent strategy decides anything | `CFH:218-232, 372` |
| 8 Child after Restart | `preRestart` cancels timers and delivers `PreRestart` but does **not** stop children; a fresh adapter runs `setup` again with a new supervisor whose count is 0; surviving children are restarted; mailbox, path and UID are kept; watchers see no `Terminated`; the parent's strategy logs the failure | `AA:309-323`; `DFH:278-303` |

**Conditions that decide the outcome** (default Classic parent unless stated):

| Matcher and nesting | Cause | Phase | Outcome |
|---|---|---|---|
| `Throwable` restart with limit, at any depth | `Exception` | message or signal | **Restart by the parent**; the limit count starts again, so the actor never stops under the default strategy |
| same | `DeathPactException` | unhandled `Terminated` | n Typed restarts, then **Stop** |
| same | non-fatal, not an `Exception` | message or signal | **Escalate** to the grandparent |
| same; parent strategy Stop, Resume or Escalate | any | message or signal | that directive |
| same, inside an outer user supervisor that matches the rethrown cause | that cause | message or signal | the outer user supervisor handles it |
| narrower class (`Exception`, `RuntimeException`, ...) | any non-fatal | any | the wrapper stays and catches the rethrow: **Stop**, plain `Terminated` |
| `Throwable` backoff with `withMaxRestarts` | any | any | `failed`: **Stop** |
| any | any | **start** (`setup` fails) | the wrapper is still outermost (deduplication runs after a successful start): **Stop**, no initialization error reaches the parent |
| Typed parent (M-A), any matcher | non-fatal | message or signal | the parent's adapter decides **Stop**; `ChildFailed` if watched (`AA:240-259`) |

**Evidence, separated.** CODE: every step above. SPEC: `CSTS:94-128` (unlimited match-all restart under a Classic parent: `PreRestart`, no
`Terminated`; it cannot tell replacement from nesting and uses no limit); `SS:224-271` and `SS:528-548` (limit exhausted under a Typed
parent); `SS:1283-1307` (deduplication). No spec covers `Throwable` plus exhausted `withLimit` plus a Classic parent. EXEC: none; the only
recorded Pekko run in the repository (`docs/adr/0030-pekko-repro`) is about a bounded mailbox. INFERENCE: if `setup` keeps failing during
an internal restart, the `PreRestart` of the Classic restart is stashed (`S:252-270`); with the default parent strategy the Typed limit
never ends the actor.

**Alternatives.**

| | A. Keep Pekko | B. Always keep a Stop wrapper | C. Defer, Pekko as the target |
|---|---|---|---|
| Observable result | as in the conditions table | the actor stops when the limit is exhausted, under any parent | the case is excluded from #976 and #1184 for now |
| What goPekko needs | the M-B interpreter drops its default wrapper when an equal `AnyError()` supervisor is in the stack; it returns the rethrown error to the parent decider; it implements `PreRestarter` (deliver `PreRestart`, keep children) and `PostRestarter` (start the initial behavior). The runtime already has the restart path (`G/supervision_restart.go:278-484`, `G/supervision_hooks.go:56-69`) | deduplication skips the default wrapper | nothing now; an inventory row stays pending |
| ADR 0033 D13 | a Pekko oracle applies; a differential scenario is needed before V3 | Go-only divergence: alternative evidence and explicit Owner approval | row pending |
| Impediment found | **none** | none needed | none |

**Recommendation: A.** The concern behind B (the limit is not effective under a default Classic parent) is Pekko's own behavior and is
documented there by its code, not by a test. If the Owner wants it pinned before delivery, C keeps A as the target.

**Minimal probe (described, not run).** A Classic parent with the default strategy spawns `supervise(setup { probe ! "setup"; receive { "boom" => throw; "ping" => probe ! "pong" }.receiveSignal { PreRestart => probe ! "pre"; PostStop => probe ! "post" } }).onFailure(restart.withLimit(2, 10.seconds))`
and the test watches it. Expected under A: `setup`; twice (`boom`, `pre`, `setup`); the third `boom` gives an Error log from the parent,
`pre`, `setup` and no `Terminated`; `ping` gives `pong`. Variants: `onFailure[Exception]` gives `post` and `Terminated`; a stopping parent
gives `Terminated`; a failing initial `setup` gives `Terminated` without `post`; a non-`Exception` cause escalates.

## 2. Item review

| Item | Pekko behavior | Spec and what it asserts | Go proposal | Observable difference | Classification | Evidence |
|---|---|---|---|---|---|---|
| 1 default Stop | M-A: the parent's adapter stops any failed Typed child (`AA:240-259`); M-B: the wrapper stops (`AA:171-172`) | `SS:387` (Error log, `PostStop`, termination); `SS:492`; `SS:502`; `CSTS:74-160` (`PostStop`, `Terminated` to a test watcher) | M-A / M-B | none | equivalence proposed | SPEC |
| 2 Restart | `PreRestart` to the old behavior; start from the initial one (`S:318-328, 392-413`) | `SS:512` (`PreRestart`, then `State(0)`) | layer restart | none | equivalence proposed | SPEC |
| 3 `withLimit` | rethrow past the limit; the window starts at the first restart (`S:335-337, 430-440`) | `SS:224` (two `PreRestart`, rethrow, `PostStop`); `SS:239` (reset after the window); `SS:261` (limit 0); `SS:528` | same | none (under a Classic parent see section 1) | equivalence proposed | SPEC |
| 4 backoff | delay formula `S:185-196`; past `withMaxRestarts` the actor fails (`S:338-340`); reset default `(min+max)/2` (`SO:86`) | `SS:912`; `SS:960`; `SS:1117` | real-time timers | none; manual time is a **deferred capability** | equivalence proposed | SPEC |
| 5 restart stash | stash capacity 1000; overflow publishes `Dropped` (`S:254-286`) | `SS:882` (`Pong(1)`, `Pong(2)`, then `Dropped`) | capacity option; `Dropped` waits for the event stream | `Dropped` not published yet | equivalence for the stash; `Dropped` is a **deferred capability** (the ADR says "adapt") | SPEC |
| 6 start failure | limited restart retries the start; exhausted or non-matching fails (`S:294-311`) | `SS:1090`, `SS:1099`, `SS:1145`, `SS:1157`, `SS:1122`; none for n = 1 | same | none | equivalence proposed | SPEC |
| 7 stop children | stop, wait, stash, revoke `watchWith` (`S:252-270, 362-388, 421-428`) | `SS:583-616`, `SS:618`, `SS:662` | same | none | equivalence proposed | SPEC |
| 8 owner-only signals (D-f) | an unowned `ResetRestartCount` passes through as a signal | `SS:1049-1051` asserts the user behavior receives it and the actor lives | internal posts never reach the user | the user's signal handler does not see it | **divergence** (S-4) | SPEC |
| 9 adapter failure | `MessageAdaptionFailure` is rethrown by default (`AA:197-213`) | `SS:1575` | same | none | equivalence proposed | CODE, partial SPEC |
| 10 death pact supervisable | `T/Behavior.scala:270-276` | `SS:1408` | same | none | equivalence proposed | SPEC |
| 11 strategy on a stopped behavior | — | `SS:1659-1695` (terminated for each strategy) | same | none | equivalence proposed | SPEC |
| 12 / D-c `AbstractBehavior` without `setup` | `S:45-50` | not opened | built inside `Setup` | the check is weaker | language adaptation | CODE |
| 13 logging | supervisor logs restarts; the limit rethrow is logged by the parent (`S:97-118, 337`; `AA:257`) | `SS:528` (three occurrences: two by the supervisor, one by the parent) | `kit-logger` | none if M-A logs in the parent decider | equivalence proposed | SPEC |
| 14 Resume with failing setup | `same` fails initial validation (`S:151-152, 169-177`) | `SS:1107` (both logs) | outcome, not mechanism | the parent's cause is an initialization error wrapping `IllegalArgumentException`, not the user's error | **open question** (S-5) | SPEC |
| 15 timers cancelled | `S:298, 334, 394`; `AA:311, 319` | `TT/TimerSpec.scala:174-197` | same | none; Typed timers are a deferred capability | equivalence proposed | SPEC |
| M-A | `rethrowTypedFailure = true`; the cause is recorded for any failure of a watched child (`AA:232-273`) | `TT/WatchSpec.scala:100,130` | interpreter implements `Supervisor` | none | equivalence proposed | SPEC |
| M-B | as section 1 | `CSTS` | default wrapper | section 1 only | equivalence, subject to S-2 | SPEC |
| D-a | unlimited Restart with a setup that fails during restart recurses into a `StackOverflowError`; with `jvm-exit-on-fatal-error = on` the JVM exits (`S:384-418`; `C/ActorSystem.scala:858-878`); if setup created children it restarts forever instead | none | the actor fails | yes | **divergence** with a concrete impediment (S-3) | CODE, INFERENCE |
| D-b | the code stashes during backoff; only the Scaladoc says "dropped" (`S:278-286`) | `SS:882` | follows the code | none | **not a difference** | SPEC |
| D-d | Pekko logs a failing `PreRestart` or `PostStop` and continues (`S:344-347`; `DFH:104-107, 254-256`) | none | log and continue | none | **not a difference** | CODE |
| D-e | matches the class of the outer exception, not its causes (`S:75-76`) | `SS:214` | `errors.Is`, `errors.As` | a wrapped cause can match in Go | language adaptation | CODE |

## 3. Mixed hierarchy matrix: corrections

All 18 rows of the ADR's matrix were checked. Outcomes hold; these corrections apply:
- **Classic parent, Classic child, processing and setup rows.** The cited `SupervisorSpec.scala:401` and `:546` use custom strategies (the
  second maps initialization errors to Restart). The outcomes come from the default decider (`CFH:218-223`), CODE only.
- **Classic parent, Typed child, processing row.** `CSTS` asserts `Terminated` to a separate test watcher; the parent gets it only if it watches.
- Applied to ADR 0035 on 2026-10-27, together with the labels of section 2.
- **Typed parent, Classic child.** Pekko's decider is `defaultDecider.applyOrElse(_, Restart)` (`AA:55-58`), not the Classic default
  strategy: a non-fatal `Error` is **Restart** (a Classic parent would Escalate), `ActorKilledException` is Stop, and the adapter logs.
  `TSCS:61-71` asserts Restart for an `Exception` only. Do not read the row as "a Classic child under a Typed parent restarts".
- **Missing rows:** Typed parent with a Classic child and a non-fatal `Error` (Restart; goPekko's default decider already does this,
  `G/supervision_decider.go:61-67`); Classic parent with a Classic child and an `Error` (Escalate; an ADR 0017 matter); fatal throwables
  under any parent (not caught; in Go `runtime.Goexit` is the language adaptation).
- **Decision 3 row:** add Escalate for non-`Exception` causes, children kept and restarted, and "a custom parent strategy decides".

## 4. Decisions

| Decision ID | Question | Existing authority | Pekko evidence | Go proposal | Observable difference | Alternatives | Architect recommendation | Affected stories | Proposed acceptance wording |
|---|---|---|---|---|---|---|---|---|---|
| S-1 | Is ADR 0035 alternative (a), with contracts 1 to 7 and 9 to 15, mechanisms M-A and M-B, and D-c and D-e, the Typed supervision contract, as a new ADR with ADR 0017 unchanged? | ADR 0033 D2 (an ADR extending 0017 is required) | section 2, mostly SPEC | ADR 0035 at `cd7c61c` | none, except deferred `Dropped` (contract 5), manual time (4) and Typed timers (15) | (a), (b), (c), (d) of ADR 0035 | accept (a); D-b and D-d recorded as equivalences; `Dropped` recorded as deferred | #956, #1183, #1184, #976, #997, then #986, #999 | "I accept ADR 0035 alternative (a), as in commit `cd7c61c`: contracts 1 to 7 and 9 to 15, mechanisms M-A and M-B, and differences D-c and D-e (language adaptations). D-b and D-d are Pekko's behavior, not differences. Publication of `Dropped` is a deferred capability. Contract 8, D-a, D-f, the cause shape of contract 14 (S-5) and the M-B case of decision S-2 are decided separately. ADR 0017 is unchanged. This accepts a contract only: no implementation, no parity claim, no merge." |
| S-2 | M-B, match-all `Restart.WithLimit`, limit exhausted on a message or signal: Pekko or stop? | none (Pekko behavior, CODE only) | section 1 | readiness section 5, decision 3, recommended stop (ADR 0035 itself only called stop a proposed divergence) | under A: the parent decides (Restart, Stop or Escalate by cause and strategy), count reset, children kept | A Pekko; B always stop (divergence); C defer with Pekko as target | **A** | #976, #1184 (one criterion each) | "In the M-B case of decision S-2 of this package, goPekko follows Pekko: the rethrown cause reaches the parent, whose strategy decides; under a restart the Typed `setup` runs again, the limit count starts again, and the actor's children are kept and restarted. Evidence is CODE only; a differential scenario is required before V3 (ADR 0033 D13)." |
| S-3 | Unlimited Restart with a setup that fails during restart (D-a) | ADR 0033 D13 | `S:384-418`; JVM exit by default | the actor fails (stops) | Pekko exits the process or restarts forever; Go stops the actor | accept the divergence; replicate (not possible: Go cannot recover a stack overflow) | accept as a divergence, including the children case (endless asynchronous restarts in Pekko) | #976 | "D-a of ADR 0035 is accepted as a divergence with a concrete impediment: Pekko ends in a fatal `StackOverflowError` (or restarts forever when setup created children); goPekko stops the actor. Alternative evidence: a Go test of the stop. This is a divergence, not parity." |
| S-4 | Does a user signal handler see an unowned `ResetRestartCount` (contract 8, D-f)? | none | `SS:1049-1051` asserts it does | internal posts never reach the user | the signal is hidden in Go | (a) Pekko: deliver an opaque signal that an unhandled path ignores; (b) accept the divergence | (a), by the Owner's default stance | #986 | "Contract 8 follows Pekko: an unowned `ResetRestartCount` reaches the user behavior as an opaque signal and is ignored when unhandled. D-f is withdrawn." |
| S-5 | Which cause does the parent see when Resume meets a failing setup (contract 14)? | none | `S:151-177`; `SS:1107` | "outcome, not mechanism" | cause type | (a) Pekko: an initialization error wrapping the validation error; (b) the user's error | (a), pending probe Q3 | #1183 | "Contract 14 follows Pekko's cause shape: the parent sees an initialization error that wraps the validation error." |
| S-6 | Is the D2 reading of ADR 0035 accepted, with the Typed-parent, Classic-child decider stated as Pekko's? | ADR 0033 D2 (accepted; not asked again) | `AA:55-58`; `TSCS:61-71` | ADR 0035 "Proposed precision of ADR 0033 D2" | none | accept the precision; decide per story | accept, with the correction of section 3 | #956, #994 | "ADR 0033 D2 is read as stated in ADR 0035 (`cd7c61c`), with this correction: under a Typed parent a Classic child is decided by Pekko's adapter decider (initialization, kill and death-pact errors stop; other non-fatal failures restart). This records how D2 applies and does not change its acceptance." |

**Form decisions inside the contract (Architect, no Owner approval):** the matcher values (`AnyError()`, `ErrorIs`, `ErrorAs`); how the
interpreter recognises its own default wrapper under S-2 (equal constructor value, open question Q2); option names; the clock used by
backoff.

## 5. Open questions and minimal probes (described, not run)

| Id | Question | Probe |
|---|---|---|
| Q1 | S-2 with children and with an `Error` cause | the section 1 probe plus a grandchild spawned in `setup` (expect `PreRestart` and `setup` again in the grandchild) and an `AssertionError` (expect escalation) |
| Q2 | Does `ErrorAs[error]()` count as equal to the default wrapper under S-2? | design question for the Go matcher; no Pekko probe applies (Pekko compares classes) |
| Q3 | Contract 14 cause class | a Typed parent watches `supervise(setup { throw }).onFailure(resume)`; record the `ChildFailed` cause and its nested cause |
| Q4 | Stale `ChildFailed` after a restarted Classic child stops normally | a Typed parent watches a Classic child that throws once (restart) and later stops; record `ChildFailed` versus `Terminated` |
| Q5 | Start failure with `withLimit(1)` | `supervise(setup { throw once }).onFailure(restart.withLimit(1, 1.minute))`: expect no retry; with 2, expect a retry |

## 6. Validation and limits

Done in this session: ADR 0035 (`cd7c61c`) and the block 2 and 3 documents read; every Pekko line above opened by the Architect or by a
read-only explorer. Spot-checked by the Architect: `SS:1040-1052`, `CFH:218-223, 370-373`, `T/internal/InterceptorImpl.scala:106-122`,
`T/scaladsl/adapter/package.scala:60,74,114`, `T/internal/Supervision.scala:70-90, 155-168, 330-340`. Not done: no Pekko spec and no Go
test was run; the conclusions about `withLimit` under a Classic parent are CODE only; this is not an independent review; no parity is
declared.
