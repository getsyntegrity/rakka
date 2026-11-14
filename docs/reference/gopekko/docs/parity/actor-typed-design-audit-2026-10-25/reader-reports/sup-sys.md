# Audit report: group sup-sys (epics #752 SUP, #760 SYS)

Baselines: Pekko v1.7.1 `9a54d12` (submodule), goPekko `origin/develop` `9abfb40` (snapshot dir `tmp/develop/`), issue bodies fetched 2026-10-25. This reader changed no code, ADR, issue or label. Go tests were NOT executed (only located by name). Pekko paths are relative to `upstream/pekko`; `A/` = `actor/src/main/scala/org/apache/pekko/actor/`, `AT/` = `actor-tests/src/test/scala/org/apache/pekko/actor/`, `D/` = `actor/src/main/scala/org/apache/pekko/dispatch/` (not under `A/`). Go paths are relative to `develop/pekko-actor/actor/`.

Depth rule used: DEEP = full body + Pekko source and spec (or documented search) for each AC + the ADR text. Everything else is SHALLOW, with the reason. No story is NOT REVIEWED. Totals: 30 DEEP, 20 SHALLOW, 0 NOT REVIEWED (50 stories: SUP 22, SYS 28).

---
## 1. Coverage inventory

State labels follow the taxonomy: 1 Accepted, 2 Proposed, 3 Contradictory, 4 Not designed, 5 Designed not implemented, 6 Implemented not validated, 7 Validated for scope. Prio and readiness are the issue labels today; `-` means none.

### Epic #752 SUP

| # | Key | Depth | State | Prio | Readiness | Note |
|---|---|---|---|---|---|---|
| 777 | SUP-01 | DEEP | 4 | P2 | blocked | Hook set not designed. ADR 0017 A7 records the gap. |
| 778 | SUP-02 | DEEP | 1 for contract (ADR 0017:151,495); 6 for 3 of 4 classes; 4 for the actor-killed class | P0 | blocked | Only the ActorKilled class is missing in Go. |
| 779 | SUP-05 | DEEP (Go tests not executed) | 6 | - | blocked | Matrix accepted in ADR 0017 A4. Close-candidate, evidence of SHA/gates missing. |
| 780 | SUP-07 | DEEP | 5 | P2 | ready | Rescope required; the gating question is open (F6). |
| 781 | SUP-10 | DEEP | 4 | P2 | ready | AC2 is an undesigned goPekko behavior (F7); no Pekko spec found. |
| 782 | SUP-11 | SHALLOW (Mailbox.cleanUp only skimmed) | 4 | P2 | ready | No Pekko spec found (F5). |
| 783 | SUP-12 | SHALLOW (same) | 4 | P2 | ready | Same. |
| 784 | SUP-13 | DEEP | 4 | P3 | blocked | Pekko order is TreeMap by name. |
| 824 | SUP-03 | DEEP | 4 | P2 | blocked | Needs the hook named. |
| 825 | SUP-06 | DEEP (tests not executed) | 6 | - | blocked | Close-candidate. |
| 826 | SUP-14 | SHALLOW (no dedicated Pekko spec found; ADR 0010 X1 not read in full) | 4 | P3 | blocked | oracle applicability under D13 pending determination (F4). |
| 862 | SUP-08 | DEEP | 3 | P2 | blocked | AC2 contradicts Pekko and ADR 0017 A11 (F2). |
| 863 | SUP-15 | SHALLOW (test at supervision_allforone_internal_test.go:865 located, not read) | 6 | P3 | blocked | Only Spawn remains after the split (F20). |
| 902 | SUP-09 | DEEP | 4 | P3 | blocked | Body dependency on #862 is wrong (F3). |
| 939 | SUP-04 | DEEP | 4 | P2 | blocked | Bundles 3 behaviours. |
| 1112 | SUP-16 | DEEP | 4 | P0 | blocked | AC-R3-1 conflicts with Pekko (F1). |
| 1113 | SUP-17 | DEEP | 4 | P1 | blocked | R3 Pekko basis pending verification (F23). |
| 1114 | SUP-18 | DEEP | 5 (name accepted in ADR 0017 A7, absent in code) | P2 | blocked | Dependency on #777 is unnecessary (F18). |
| 1115 | SUP-19 | DEEP | 6 for decider panic, 4 for hooks | P2 | blocked | Oracle mismatch (F8). |
| 1116 | SUP-20 | SHALLOW | 4 | P2 | blocked | Decision proposal. |
| 1117 | SUP-21 | SHALLOW | 4 | P3 | blocked | No Pekko spec; D13 pending. |
| 1118 | SUP-22 | SHALLOW | 4 | P3 | blocked | Same. |

### Epic #760 SYS

| # | Key | Depth | State | Prio | Readiness | Note |
|---|---|---|---|---|---|---|
| 798 | SYS-01 | DEEP | 4 | P2 | blocked | Pekko caches failed init. |
| 799 | SYS-05 | DEEP | 3 (AC3 vs ADR 0014/0024 WhenTerminated) | P2 | blocked | F10. |
| 800 | SYS-06 | DEEP | 4 | P2 | blocked | AC1 vs Pekko (F11). |
| 801 | SYS-14 | SHALLOW (reconciliation anchors only, deprecation lines checked) | 4 | - | blocked | Owner decision O-6. |
| 802 | SYS-15 | SHALLOW | 4 | - | blocked | Same. |
| 803 | SYS-16 | SHALLOW (CoordinatedShutdown.scala:239-260 only) | 4 | - | blocked | Same. |
| 841 | SYS-02 | DEEP | 4 | P2 | blocked | Concurrency oracle only in the typed registry. |
| 842 | SYS-03 | SHALLOW (PR #1013 and ActorSystem name regex not read) | 3 for AC1 vs ADR 0031 O3a/O3b; 6 for the rest | - | blocked | In flight, stale label noted. |
| 843 | SYS-04 | SHALLOW | 4 | P3 | blocked | Split to #1122. |
| 844 | SYS-07 | DEEP | 4 | P2 | blocked | Dependency on #773 questionable (F12). |
| 882 | SYS-08 | DEEP | 4 | P2 | blocked | Owner of the single-run rule. |
| 883 | SYS-10 | SHALLOW (spec bodies :170-:308 not read) | 4 | P2 | blocked | |
| 918 | SYS-09 | DEEP | 4 | P2 | blocked | Priority inversion with TIM (F12). |
| 919 | SYS-12 | SHALLOW (reason classes not opened) | 4 | P2 | blocked | |
| 932 | SYS-17 | DEEP | 4 | P2 | blocked | AC2 is a Go addition. |
| 950 | SYS-11 | DEEP | 4 | P2 | blocked | |
| 971 | SYS-13 | DEEP | 3 (ADR 0014/0024 contract change) | P2 | blocked | |
| 1055 | SYS-18 | DEEP | 4 | P1 | blocked | |
| 1119 | SYS-19 | DEEP | 4 | P2 | blocked | |
| 1120 | SYS-20 | DEEP | 4 | P2 | blocked | |
| 1121 | SYS-21 | SHALLOW | 4 | P3 | blocked | May have no Go setting (F16). |
| 1122 | SYS-22 | SHALLOW | 4 | P2 | blocked | May reduce to value validation. |
| 1123 | SYS-23 | SHALLOW (Future.sequence :491 not read) | 4 | P2 | blocked | |
| 1124 | SYS-24 | DEEP | 4 | P2 | blocked | |
| 1125 | SYS-25 | SHALLOW | 4 | P3 | ready | Decision proposal; ready is correct. |
| 1126 | SYS-26 | SHALLOW | 4 | P3 | ready | Same. |
| 1127 | SYS-27 | SHALLOW | 4 | P3 | ready | Same. |
| 1128 | SYS-28 | DEEP | 4 | P3 | blocked | Dependency on #1121 unnecessary (F15). |

Epics #752 and #760 themselves were not audited as stories.

---
## 2. Per-capability matrix

| Capability | Pekko contract | Go accepted/proposed contract | State | Go implementation in develop | Evidence | Gap |
|---|---|---|---|---|---|---|
| Overridable SupervisorStrategy hooks (decider, handleChildTerminated, processFailure, handleFailure, loggingEnabled, logFailure) | `A/FaultHandling.scala:332,339,344,366,395,405` | ADR 0017 A7 states "no equivalent" and registers #450 item 1. #777 would be the ADR. | 4 | `Strategy[T]` is a struct built only by `OneForOne`/`AllForOne` (supervision.go:50). `Decider` func exists (supervision.go:48). | Pekko spec: `AT/DeathWatchSpec.scala:216-246` overrides handleFailure and asserts order 1,2,3. `AT/SupervisorMiscSpec.scala:144-162` overrides handleChildTerminated. Go: none. | Hook-by-hook list of new versus existing is missing. Hook failure is outside the ADR scope (#1115). |
| Default decider classification | `A/FaultHandling.scala:210-232`: init, ActorKilled, DeathPact are Stop; other Exception is Restart; Throwable is Escalate | ADR 0017 :151,:495-496 | 1 | `defaultDecider` (supervision_decider.go:57-65): init, DeathPact, ErrGoexit are Stop; everything else Restart; "never escalates". | Go tests per reconciliation (not run). Pekko: `AT/DeathWatchSpec.scala:216-246` exercises Kill, ActorKilledException, then DeathPact. | ActorKilled absent (needs #1112). Throwable-to-Escalate (Pekko `FaultHandling.scala:214-217`) has no Go source (ADR 0017:496 records "no Go source", not an adaptation). |
| ActorKilledException | `A/Actor.scala:172-177` (`private[pekko]` constructor, NoStackTrace); thrown at `A/ActorCell.scala:575` | ADR 0017:495 gap. #1112 proposes the contract. | 4 | Absent (no symbol). | Pekko: `PekkoExceptionSpec.scala:35` (serialization only) and DeathWatchSpec above. | Export form is an Owner decision. |
| Restart budget (MaxRetries/Within) | `A/FaultHandling.scala:55-88`, `:303-308` | ADR 0017 A4/A6, matrix at :224-240 | 6 | supervision_budget.go; doc supervision.go:35-39 | Located tests: TestSup2BudgetMatrix, TestSup2BudgetUnlimited, TestSup2Int32Wrap, TestSup2ZeroReading, TestAllForOneCountersPerChild, TestAllForOneSiblingAtItsLimitDenies. ADR S4 says the 24 cells were measured on the real class (h4-repro/RESULTS.md, not read). | The reconciliation cites no SHA or gate record. No `#779`/`#825` mention in docs. |
| Restart hooks | `A/dungeon/FaultHandling.scala:92-118` (PreRestart failure is `publish(Error(PreRestartException))`, ungated), `:278-303` | ADR 0017 U9/A11/A12 | 5/6 | supervision_restart.go; PORT-GAP(logging-U9) at :316 | Pekko: `AT/SupervisorSpec.scala:419-458` filters `PreRestartException` (1 occurrence) and `PostRestartException` (1 occurrence). Go tests per reconciliation. | PreRestart failure log is missing. |
| PostRestartException original cause | `A/Actor.scala:247-267` (`cause`, `originalCause`, `OriginalRestartException` walks the chain); `A/dungeon/FaultHandling.scala:299-301` | ADR 0017 A12 | 4 | `InitializationError{Err, InRestart}` has no original-cause field (supervision_hooks.go:19-23 PORT-GAP). | `AT/SupervisorSpec.scala:419-458` covers the restart-during-restart flow. No Pekko spec asserts both causes at once (reconciliation, plausible; not independently searched). | |
| Failed / Recreate system-message ordering | `D/Mailbox.scala:289-345`, `A/ActorCell.scala:495-506`, `A/dungeon/Dispatch.scala:158-162` | Not designed; PORT-GAP in code | 4 | supervision_failure.go:101-103 and supervision_restart.go:89-90 mark the gap. | No Pekko spec of Failed/Recreate system-message ordering or disposition found. Searched `actor-tests/src/test` and `testkit/src` for `Recreate(`, `systemDrain`, `sendAllToDeadLetters`: no hits. Related, not covering it: `SystemMessageListSpec.scala:26-109` (list structure), `SupervisorHierarchySpec.scala:870,923` (log filter only). Pekko evidence is code only, not proven by tests. | oracle applicability under ADR 0033 D13 pending determination (Pekko code exists; no spec found). |
| Survivor restart order | `A/dungeon/FaultHandling.scala:278-300`; `A/dungeon/ChildrenContainer.scala:79,120` (TreeMap by name) | Go doc supervision.go:81-90 says ascending name, "not a guaranteed contract" | 4 | supervision_restart.go:291,451-452 | No Pekko spec found (search: `survivors`, restart-children-order in SupervisorHierarchySpec: none). | |
| Logging of failures | `A/FaultHandling.scala:366-424`: logFailure is called unconditionally for Resume/Restart/Stop/Escalate, `loggingEnabled` is checked inside the default | ADR 0017 A11/U9 delivered: gated by `Config.Logger` and `Strategy.WithLogging` | 3 for hook override | `logs()` and `logFailure` (supervision_logging.go:31-70) | | Overridable hook semantics conflict (#862). |
| Directive log level | `A/FaultHandling.scala:118-201` (`resume/restart/stop(logLevel)`, Escalate.logLevel throws); non-Error levels publish without cause (:412-419) | ADR 0017 A2: Directive closed | 4 | `Directive` uint8 enum (supervision.go:20) | SupervisorSpec cited only by name. | How the level travels is an Owner decision. |
| StoppingStrategy | `A/FaultHandling.scala:238-244` | ADR 0017 A7 and stub list :733-735 | 5 | Only `StoppingGuardianStrategy` (guardian.go:70) | Pekko: `AT/ActorSystemSpec.scala:336-356` (guardian config), `TcpListenerSpec.scala:201`. | Not exported as a `Strategy`. |
| Child failure cause at termination | Typed: `adapter/ActorAdapter.scala:83,93-100,238-247,268-272` (cause kept only for watched children, removed at Terminated) | none | 4 | `Terminated{who}` only (watch.go:22-26) | `actor-typed-tests TerminatedSpec`, `WatchSpec` (not read) | |
| Extensions | `A/ActorSystem.scala:723-742,1165-1228`; `A/Extension.scala:76-125` | none; ADR 0010:224 "Nothing else is exported"; ADR 0033 D3 | 4 | Absent (doc.go:21 only) | `AT/ExtensionSpec.scala:70-97`. Typed: `actor-typed-tests/.../ExtensionsSpec.scala:104-119` (concurrent init). | |
| registerOnTermination | `A/ActorSystem.scala:651,663,1303-1335`: reverse order, `RejectedExecutionException("ActorSystem already terminated.")` on late add (only after termination completed; during termination either outcome, spec :272), `terminationFuture` after callbacks | none; Go `WhenTerminated` closes at root guardian finalization (system_shutdown.go:38-44; ADR 0014/0024) | 3 | Absent | `AT/ActorSystemSpec.scala:212` (reverse order), `:234` (whenTerminated after callbacks), `:263` (late registration rejected) | Contract change needs ADR amendment. |
| CoordinatedShutdown | `A/CoordinatedShutdown.scala` (topologicalSort :336-362, knownPhases :395, addCancellableTask :565, run :722-790, totalTimeout :820) | none (doc.go:16, README:352 say planned) | 4 | Absent | `AT/CoordinatedShutdownSpec.scala:69-740` (full list of test names was read) | Whole API undesigned. |
| System guardian spawn (`systemActorOf`) | `A/ActorSystem.scala:779,924-925`; documented order :773-775 | none | 4 | `spawnTarget` returns /user only (hierarchy.go:23-25); `guardianSet` has no system node (guardian.go:108-114). Watch chain /system watches /user exists (guardian.go:141-146). | No dedicated Classic spec (only `ConfiguredLocalRoutingSpec` uses it; not about order). | |
| Terminate / WhenTerminated / Shutdown | `A/ActorSystem.scala:1089-1105` (CoordinatedShutdown.run when enabled, else finalTerminate) | ADR 0014/0024; minimal contract | 6 | system_shutdown.go:30-75 | | Disabled-mode contract is #1124. |
| System name / identity | `A/ActorSystem.scala:840-843` regex | ADR 0031 O3a (accepted): the name comes from a `Config` field; validation "if adopted ... is separate" (not decided). O3b (accepted): empty name means `"default"`, recorded as goPekko's own decision. Contradiction with Pekko: `ActorSystem.scala:840-843` rejects an empty name. Decision not changed here. | 1 | system.go:69-100 | | #842 AC1 contradicts the ADR. |
| TypedActor proxy, DynamicAccess, AbstractActor, JVM hooks | `A/TypedActor.scala:43,118,131` (deprecated), `A/DynamicAccess.scala:33`, `A/CoordinatedShutdown.scala:246,280-296,830` | Dossier D8 closed reason list; Owner O-6 | 4 | none | | Exclusion records pending. |
| Not covered by this group | BackoffSupervisor (`pattern/internal/BackoffOnRestartSupervisor.scala`), LoggingAdapter, Settings/Config | Stories exist in PAT (#909, #941, #968, #981), EVT, GOV-25 (#1074) | n/a | | | Outside group; see section 7. |

---
## 3. Findings

**F1 (high) SUP-16 AC-R3-1 asserts the opposite of Pekko.**
- #1112 AC-R3-1 requires the directive for an actor-killed cause to remain Restart under DefaultStrategy "until SUP-02 lands". Pekko `A/FaultHandling.scala:218-222` maps `ActorKilledException` to Stop, and `AT/DeathWatchSpec.scala:216-246` shows the Kill, ActorKilledException, DeathPact sequence.
- This is a deliberately transitional assertion that #778 must invert. As written it is an anti-parity test that will be committed and then flipped.
- Correction: drop AC-R3-1 or mark it "temporary, replaced by #778 AC1" with a link. Add `DeathWatchSpec.scala:216-246` as the Pekko oracle to #778, #1112 and #1031. #778 cites only "SupervisorSpec".

**F2 (high) SUP-08 (#862) AC2 contradicts Pekko and ADR 0017 A11.**
- Pekko calls `logFailure` unconditionally for all four directives (`A/FaultHandling.scala:372-387`). `loggingEnabled` is checked only inside the default `logFailure` (:406). Go gates before logging via `Config.Logger` and `Strategy.WithLogging` (supervision_logging.go:31-38, delivered in U9).
- Verified; the reconciliation is right.
- Correction: Owner decision needed (section 4, D-SUP-2). Rewrite AC2 after the decision.

**F3 (medium) Dependency edges are inconsistent in the bodies and the GOV edges differ.**
- #902 body still lists `#862 SUP-08` as its dependency. Pekko carries the level on the Directive (`:128-201`), independent of `logFailure`. The reconciliation corrects it, but the body and `## Dependencies` section are unchanged.
- #777 and #902 use `#1020 GOV-02` (an index story) as `depends_on`. #1112, #1113, #798, #799, #800, #1055 use `#1021 GOV-03` (the form). #1112 and #1055 say explicitly that GOV-02 "is only an index". Pick one: the dependency is on the justification form (#1021), and #1020 `enables`.

**F4 (medium) SUP-14 (#826) depends_on SUP-01 (#777) without a Pekko or technical basis.**
- The Decider and the parent `*Context[T]` already exist (supervision.go:48). ADR 0017 A13 (accepted) says the context is valid only during the call. Context expiry does not need a user-implementable Strategy.
- The edge makes #826, then #1116, #1117, #1118, #863, plus consumers #983 (TYP-33) and #900 (TIM-06 lane note), wait for an Owner ADR decision on #777.
- Also: no dedicated Pekko spec was found. Whether an applicable oracle exists under ADR 0033 D13 is pending determination.
- Correction: make #826 independent of #777. Decide scope (decision Context only, or every Context; README:254, ADR 0010 X1). Record the D13 classification (oracle applicability pending determination).

**F5 (high) SUP-11 (#782) and SUP-12 (#783): label inconsistency and no oracle.**
- Both are `status:ready`. Their reconciliation blocks say `status:ready` is applied "only when every prerequisite is delivered", and list #900 (TIM-06, P0 blocked) and #814 (MSG-02, P1 blocked) as `enables` producers.
- The text of #782 calls #900 "a lane constraint, not a contract dependency". The type (`enables`) and the text disagree.
- No Pekko spec of these orderings found (searched `Recreate(`, `systemDrain`, `sendAllToDeadLetters` in `actor-tests/src/test` and `testkit/src`: no hits). Related, not covering it: `SystemMessageListSpec.scala:26-109` (list structure), `SupervisorHierarchySpec.scala:870,923` (log filter only). Pekko evidence is code only, not proven by tests. AC2 "Match selected Pekko system-message order/disposition" can only be sourced from `Mailbox.scala:289-345` and `ActorCell.scala:495-506` (I read only the first lines of each). Whether that Pekko code is an applicable oracle under ADR 0033 D13 is pending determination.
- Correction: record dependency type as `independent` with a lane note, or relabel `blocked`. Add a D13 note "no applicable Pekko spec".

**F6 (medium) SUP-07 (#780) is `ready` but its acceptance criteria need a design decision, and the body is stale.**
- The reconciliation rescopes the story to the PreRestart failure log only. The body still describes PostRestart.
- Pekko publishes the PreRestart failure unconditionally (`A/dungeon/FaultHandling.scala:106-107`). Go logs only when `Config.Logger` is set and the strategy has logging on (ADR 0017 A11). Whether the new log is gated by `WithLogging` is a design decision, not stated.
- A real Pekko oracle exists and is not cited by line: `AT/SupervisorSpec.scala:419-458` asserts `EventFilter[PreRestartException]("Don't wanna!", occurrences = 1)`.
- Correction: add the oracle, state the gating rule, and set `blocked` until the Architect records it.

**F7 (high) SUP-10 (#781) AC2 and the change to `InitializationError` are a public-contract change under a `ready` label.**
- Pekko exposes two causes on `PostRestartException`: `getCause` (the hook failure) and `originalCause` (the failure that caused the restart) (`A/Actor.scala:247-251`); `OriginalRestartException` walks nested originalCause links (:260-267).
- Go's `InitializationError.Unwrap()` returns one error. Returning a multi-error list from `Unwrap() []error` changes `errors.Is`/`errors.As` for every existing decider. For example `MakeRestartDecider(cause)` would now match the original restart cause inside a PostRestart failure.
- This is high-risk item 3 (contracts others consume). The story has no ADR and no Owner decision.
- Correction: `blocked` until an ADR 0017 amendment records which of the two causes `Unwrap` exposes.
**F8 (high) SUP-19 (#1115) oracle does not support its R2.**
- `AT/SupervisorMiscSpec.scala:144-162` calls `system.stop(parent)`. The hook's `actorOf` throws `IllegalStateException` while the parent is terminating, and the spec asserts only the Error log `"handleChildTerminated failed"`.
- The Pekko doc (`A/FaultHandling.scala:335-337`) says exceptions there "do NOT make the actor fail if this happens during termination". The code does call `handleInvokeFailure(Nil, e)` (`A/dungeon/FaultHandling.scala:337-338`). So "parent fails" is source-only for the non-terminating case. Source-only for the non-terminating case; not proven by tests (SupervisorMiscSpec.scala:144-162 asserts only the Error log).
- #1115 also lists only #777 as a dependency. #939 `enables` #1115 and the child-terminated hook does not exist until then.
- Correction: split R2 into "Error entry" (has an oracle) and "parent fails" (source-only; D13 classification pending determination). Add #939 as a dependency of #1115.

**F9 (medium) SUP-04 (#939) has a priority inversion.**
- It is P2 and depends on #895 (IDN-12) which is P3 and blocked. AC3 (stale incarnation) is re-homed there.
- Not verified: how Pekko filters a stale incarnation for `handleChildTerminated` (the `uid` check at `A/dungeon/FaultHandling.scala:306-321` is for `Failed`, not termination).

**F10 (high) SYS-05 (#799): Pekko semantics conflict with the documented Go termination contract.**
- Pekko's `whenTerminated` completes after all callbacks (`A/ActorSystem.scala:1331`, spec `AT/ActorSystemSpec.scala:234`). Go closes `WhenTerminated` at root guardian finalization (system_shutdown.go:38-44).
- Matching Pekko changes a documented contract of ADR 0014/0024.
- The reconciliation is right. Not in the reconciliation: the spec `AT/ActorSystemSpec.scala:272` ("throw RejectedExecutionException or run termination callback when shutting down") accepts either outcome for a registration after `terminate()` (exactly one of: `RejectedExecutionException`, or the callback runs). Scaladoc (`ActorSystem.scala:647,661`) says rejection also applies when termination has been initiated; the code rejects only after termination completes (:1319-1321). The story's "late-registration behavior" AC needs that case.
- Correction: add spec line anchors `:212`, `:234`, `:263`, `:272`. Record the ADR 0014/0024 amendment as a prerequisite.

**F11 (medium) SYS-06 (#800): the reconciliation's "only addTask rejects an unknown phase" is imprecise.**
- `knownPhases = phases.keySet ++ phases.values.flatMap(_.dependsOn)` (`A/CoordinatedShutdown.scala:395`). `addTask` accepts a phase that is only named as a dependency (:565-568, :608).
- `totalTimeout` then calls `timeout(phase)`, which throws `IllegalArgumentException` for such a phase (:810-815, :515-520). That is a Pekko edge relevant to #1128 (code only, no spec: `A/CoordinatedShutdown.scala:395,565-568,608,515-520,810-815`).
- The delivery plan (lines 182, 331) lists #800 as startable with no dependencies, while the issue is `blocked` on #1021. The plan is stale for this row.
- Correction: AC1 should define "unknown" as "neither a key nor a dependency".

**F12 (medium) The SYS-07/SYS-09 chain acquires P0 and P1 blockers that Pekko does not have.**
- #844 depends on #773 (ASK-01, P1 blocked). Pekko tasks are `() => Future[Done]`, not Ask (`A/CoordinatedShutdown.scala:565-606`). #950 explicitly avoids ask (:649-660).
- #918 depends on #821 (TIM-02, P0) and #1129 (TIM-11, P0, "export an injectable clock"). Go already has an internal `clock` seam (clock.go, `options.nanotime`).
- Consequence: P2 shutdown stories wait behind P0 timer exports and an Ask decision. The plan F-04c lists only F-04b.
- Correction: Owner/Architect decision on the task signature in the #800 form record. Use the existing internal clock seam for the timeout test, or accept the export dependency explicitly.

**F13 (medium) The single-ErrSystemStopping-timing decision is duplicated.**
- #1055 (open choice, R4) and #971 (reconciliation: "When ErrSystemStopping starts is unstated") both need it. Pekko: pending verification. Question: until which phase can `actorOf` on `/user` succeed during CoordinatedShutdown? Evidence missing: a Pekko spec or code path showing `actorOf` after `run()` starts (none cited; the comment at `CoordinatedShutdown.scala:654` says spawning is not possible during shutdown). Go: rejects top-level Spawn from the first `Terminate` (errors.go:37).
- Correction: one Owner decision record referenced by both.

**F14 (medium) SYS-18 (#1055): R3/R4 hold, evidence gap.**
- R3 matches Pekko's documented order (`A/ActorSystem.scala:773-775`). I confirmed Go baseline (hierarchy.go:23-25, guardian.go:108-114, wiring guardian.go:141-146).
- No Pekko Classic spec for the /system-after-/user order (searched `systemActorOf` in `actor-tests/src/test`: only `ConfiguredLocalRoutingSpec`, which is not about order). Typed `ActorRefResolverSpec`/`LocalReceptionistSpec` use it. D13: oracle applicability for R3 is pending determination.
- P1 story depends on #1021 (P1) and on Owner decision O-2. It feeds #960, #994, #1056, #1176 (Typed services). Treat as critical path with #1021.

**F15 (medium) SYS-28 (#1128) has an unnecessary dependency and a lane mismatch.**
- It depends on #1121 (per-reason overrides, P3). Pekko's reason-overrides cover only `exit-jvm`, `exit-code`, `terminate-actor-system` (`resources/reference.conf:1280-1292`). They never affect `timeout(phase)` or `totalTimeout()`.
- Lane says "G5 system", other SYS stories say "G6 system".
- Correction: depend on #800 and #844 only; fix the lane label.

**F16 (low) SYS-21 (#1121) may have nothing to override.**
- The overridable Pekko settings are JVM exit (#1127, excluded by default) and `terminate-actor-system` (tied to the #1124 switch). The story itself says "Architect to confirm it is worth keeping".
- Correction: keep blocked, decide after #1124 and #1127, or close as not applicable.

**F17 (medium) Coverage gaps inside this group's areas (no story).**
- ActorSystem name validation (`A/ActorSystem.scala:840-843`): ADR 0031 O3a leaves validation open ("if adopted, ... is separate"). No Owner decision under ADR 0033 D13 classifies the absence of validation as an adaptation; pending. The inventory needs an entry, which the reconciliation of #842 admits has no story key.
- `CoordinatedShutdown.get`/extension registration and the config `phases` parse (spec `:501` "parse phases from config", Go replacement undefined). #800/#1120 do not cover where phases come from in Go (HOCON excluded).
- `ActorSystem.terminate` `aborting` flag (`A/ActorSystem.scala:1089-1103`) not mentioned.
- BackoffSupervisor is in PAT, outside this group (section 7).

**F18 (medium) SUP-18 (#1114): dependency on #777 is weaker than stated; contract name is already accepted.**
- The body says "P2 because a decider can build it today" and "Dependency on SUP-01 may be unnecessary".
- `StoppingStrategy` appears in accepted ADR 0017 A7 and the stub list (:733-735). `StoppingGuardianStrategy` exists (guardian.go:70). The only open decision is the cause set (Pekko: `Exception` only).
- Correction: make it independent of #777 once the Owner decides the cause set. Pekko oracle: `AT/ActorSystemSpec.scala:336-356` (verified, tests guardian config, not the strategy as a child strategy).

**F19 (low) New stories #1112-#1118 have no ACT2 reconciliation block.**
- They carry their own Dependencies, Requirements and ADR impact sections. They are consistent with ADR 0033 D3/D5. No defect found. Listed so a reviewer does not look for the block.

**F20 (medium) SUP-15 (#863) body is stale after the split.**
- The reconciliation narrows it to Spawn inside a decider and moves Stop/Watch to #1117/#1118. The issue body AC1 and AC2 still list Spawn/Stop/Watch/self.
- Go already allows and tests Spawn (supervision.go:84-85, test at supervision_allforone_internal_test.go:865, not read). So what remains is documentation plus a reservation/failure-ordering test.
- Correction: edit the body. Consider closing after the test is confirmed.

**F21 (medium) SUP-20 (#1116) is a decision story (P2) blocked on an implementation story (#826, P3).**
- Inventory and recommendation do not need a delivered context-expiry. Priority inversion plus extra hop (see F4).
- Correction: depend on the #826 decision, not its delivery.

**F22 (low) SUP-01 (#777) AC3 vs SUP-19 (#1115).**
- #777 AC3 asks to "separate hook failure policy from child failure policy; record the required Owner decision". #1115 holds that decision as its open choice and says "Owner decision recorded by SUP-01; not decided". Each story defers to the other. The reconciliation of #777 says "failure of a hook is #1115".
- Correction: state which story records the decision.

**F23 (medium) SUP-17 (#1113): R3 and part of R1 have no Pekko basis.**
- Pekko's typed adapter records the cause (`ActorAdapter.scala:268-272`), only if the parent watches the child, and removes it at Terminated (:96-98). The decision is Stop for typed children and `TypedActorFailedException` (:236-239, 258-259); for other (classic) children it is the classic default decider (:261, 55-58), which can Restart. Whether R3/AC-R3-1 can occur upstream: pending verification. Question: is a stale cause observable after a restart of a watched classic child? Evidence missing: no spec opened (`TerminatedSpec.scala`, `WatchSpec.scala` not read). Evidence so far is code only.
- R4 is verified.
- Spec anchors `TerminatedSpec.scala:28-40`, `WatchSpec` not opened.

**F24 (low) Close-candidate rows #779, #825: evidence is incomplete, not wrong.**
- The reconciliation says ADR 0017 A4 matrix and tests exist. Confirmed by code reading: doc supervision.go:35-39 and Pekko `A/FaultHandling.scala:58-88` agree with ADR A4 text (zero denies, negative means no count, window semantics).
- Missing as stated: tested candidate SHA, full gates run, traceability record, and the Owner statement that ADR 0017 A4 is the "reviewable ADR" for AC4.
- Correction: do not close until those three exist.

---
## 4. Owner decisions needed

Alternatives and consequences are as stated in the stories and the cited evidence. "Recommendation" lines carry no recommendation unless a Pekko source is cited.

**D-SUP-1 Hook-failure policy and hook set for user strategies (#777, #1115).**
- Alternatives: (a) expose all six hooks as an interface; (b) expose only Decider plus child-terminated and keep logging/processFailure internal; (c) no user Strategy, keep Decider only.
- Consequences: (a) widens the public API (ADR 0011 per-export justification each); (c) leaves #824, #939, #862, #1115 void.
- Pekko: the two oracles override `handleFailure` (`DeathWatchSpec.scala:216-246`) and `handleChildTerminated` (`SupervisorMiscSpec.scala:144-162`); the overridable members are `FaultHandling.scala:332,339,344,366,395,405`. Open question: which members goPekko exposes; evidence missing: Owner decision (ADR 0011 per-export justification). No recommendation.

**D-SUP-2 Logging gate for an overridable `logFailure` (#862, also #780).**
- Alternatives: (a) follow Pekko: hook always called, gate inside the default implementation; (b) keep Go gating; that is a difference from the Pekko behavior in (a) and needs an explicit Owner decision; it is not an accepted adaptation.
- Consequence: (a) changes the delivered U9 contract (Config.Logger, WithLogging); (b) deviates from `FaultHandling.scala:372-387,406`.
- Pekko: `logFailure` is called for all four directives (`FaultHandling.scala:375,379,383,387`); `loggingEnabled` is checked inside the default `logFailure` (:406). Code evidence only; no spec cited for an override. goPekko: gated before logging by `Config.Logger` and `Strategy.WithLogging` (ADR 0017 A11/U9, accepted). This differs from Pekko; the accepted decision is not changed here. Open question: whether the hook is called unconditionally; evidence missing: Owner decision. No recommendation.

**D-SUP-3 Export form of ActorKilledException (#1112).**
- Alternatives: sentinel `ErrActorKilled` versus a struct type.
- Pekko: `ActorKilledException` carries a `message` (`Actor.scala:177`), thrown with "Kill" (`ActorCell.scala:575`), constructor `private[pekko]`. Open question: Go export form (sentinel or type); evidence missing: Owner decision (ADR 0011). No recommendation.

**D-SUP-4 How `InitializationError` exposes the original cause (#781).**
- Alternatives: (a) multi-`Unwrap`; (b) single `Unwrap` plus an `OriginalCause()` accessor; (c) a separate type for restart failure.
- Consequence: (a) changes `errors.Is` for existing deciders (F7).
- Pekko: two causes (Actor.scala:247-251). Open question: which cause `errors.Is/As` traverses; evidence missing: Owner decision (ADR 0017 A12). No recommendation.

**D-SUP-5 Where the directive log level travels (#902).** New Directive values versus a strategy option (the reconciliation lists both). Consequence: new values change the exported enum (ADR 0017 A2).

**D-SUP-6 Allowed parent operations inside a decider (#1116) and context-expiry scope (#826).** Allowed or denied per Spawn, Stop, Watch, plus whether a denied operation fails the parent or returns an error. D13 classification for #826: no Pekko counterpart found; oracle applicability pending determination.

**D-SUP-7 Cause set of StoppingStrategy (#1114).** Every cause or exclude Goexit-like causes (Pekko stops `Exception` only).

**D-SUP-8 D13 classification of rows whose oracle applicability is pending determination.** #782/#783 ordering, #1113 R3 (Pekko basis pending verification), #826, #1117, #1118, #1116. A missing dedicated spec does not show that no applicable oracle exists; alternative evidence and approval come into question only for a row where the oracle is shown not to apply.

**D-SYS-1 Individual ADR 0011 justification form (O-2) via #1021.** Blocks #798, #799, #800, #844 (via #800), #1055, #1112, #1113. Alternatives: one ADR per export, or a record in each story.

**D-SYS-2 Extension failure and recursion rules (#798).** Cache failed initialization like Pekko or not; recursive request fails immediately (deviation from Pekko's 20 s creation-timeout then `IllegalStateException`, `A/ActorSystem.scala:1178-1188`) or waits.

**D-SYS-3 WhenTerminated relative to termination callbacks (#799).** Close after callbacks (Pekko) with an ADR 0014/0024 amendment, or keep and record a divergence.

**D-SYS-4 Phase graph rules (#800).** Reject unknown dependency names (Go addition, D13) or accept like Pekko (`:339,:355`).

**D-SYS-5 Task signature and cancellation handle (#844, #883).** Shared with ASK-01/TIM-01 or local to shutdown (F12).

**D-SYS-6 Go additions in #918, #932.** Late task cannot finish a later phase (#918 AC3), reporting skipped phases (#932 AC2). Both have no Pekko counterpart found; oracle applicability under D13 is pending determination.

**D-SYS-7 When ErrSystemStopping starts (#1055, #971).** From first Terminate (today) or at another point (Pekko behavior pending verification, see F13).

**D-SYS-8 Default coordinated-shutdown phase names and timeouts (#1120).** Keep the six non-cluster names (`before-service-unbind`, `service-unbind`, `service-requests-done`, `service-stop`, `before-actor-system-terminate`, `actor-system-terminate`) or fewer; Pekko defaults: `default-phase-timeout = 5 s` (`reference.conf:1248`), `actor-system-terminate` timeout 10 s (:1394-1397). Adopting them in goPekko needs an Owner decision.

**D-SYS-9 Default of the coordinated-shutdown switch (#1124).** Enabled by default changes the documented Terminate contract.

**D-SYS-10 Exclusion records (O-6): #801 TypedActor (jvm-only, deprecated since Akka 2.6.0 per `A/TypedActor.scala:43,118,131`), #1125 DynamicAccess (jvm-only), #1126 AbstractActor (java-api), #803 JVM hooks/JFR (jvm-only), #1127 Go signal/exit helper (owner-excluded or opt-in), #802 construction mapping.**

**D-SYS-11 Approval of narrowed #842 criterion 1 (#1066).** Contradiction with ADR 0031 O3a/O3b.

---
## 5. ADRs needing creation, amendment or acceptance (no ADR was changed)

| ADR | Status line (verified) | Action needed |
|---|---|---|
| 0017 Supervision | accepted 2026-10-14, "does not show implementation" | Amend or add successor for: user-defined Strategy (A7), ActorKilledException (:495), logging gate (A11), Directive log level (A2), decider context capabilities (:630, A13), survivor order, Failed/Recreate dead-letter disposition (S11 area). #777 is the ADR vehicle; none exists as a draft. |
| 0010 Actor API | proposed; supervision part accepted via 0017 | Kill exclusion (:216/:225) and "Nothing else is exported" (:224, Extension/Shutdown exports). Decisions on Kill sit in #1039. |
| 0011 layout | accepted | Per-export individual justifications (owner decision 2) for: ActorKilled, user Strategy, cause accessor, Extension API, System shutdown API, /system spawn. |
| 0014, 0024 termination | 0014 accepted design; 0024 not checked | Amendment if WhenTerminated moves after callbacks (#799) or Terminate runs the phase sequence (#971). |
| 0030 bounded mailbox | accepted | Not touched by this group (checked: no SUP/SYS story needs it). |
| 0031 | accepted (O3a/O3b/O3c) | Records name rules; #842 AC1 must follow it. |
| 0033 | accepted by Owner 2026-10-24 | D3, D5, D8, D11, D13 used as written. Closed exclusion-reason list: `actor-typed-completion-design.md:342` (7 codes: `jvm-only`, `java-api`, `replaced-by-go-stdlib`, `replaced-by-goPekko-module`, `cluster-or-remote`, `network-io`, `owner-excluded`); it is not in ADR 0033 D8. |
| New ADR | none exists | Go CoordinatedShutdown API (phases, tasks, run, timeouts, reasons, disabled mode), Extension API and System extension identity, `registerOnTermination` semantics. Typed supervision is separate (#1063). |
| ADR 0032 | only on PR #1014, not in develop | Not used. #782/#783 sender assertions depend on it (D3, D4, D6) and TellFrom (#814). |

---
## 6. Blocked versus potentially startable

Prerequisites below are those the issue text and my verification support. "Startable" means no open Owner decision is required to begin the work stated.

**Startable now (small):**
- #1125, #1126, #1127: decision documents, labeled ready, no code. Correct as labeled.
- #779/#825: evidence-closure work (record candidate SHA, run gates, add traceability, Owner confirms A4 as the reviewable ADR). No code change.
- #1114: only after the cause-set decision (D-SUP-7); the dependency on #777 is not required (F18).
- #863: only a doc and test confirmation (F20).

**Startable only after a small decision:**
- #780: gating rule (F6).
- #782/#783: label reconciliation (F5) and D13 classification (oracle applicability pending determination; no dedicated Pekko spec found).

**Blocked, with the exact prerequisite:**
- #1112, #778, #1113: #1021 (form, O-2); #778 also #1112; #1031 later.
- #777: #1021 (and #1020 per body, F3), D-SUP-1.
- #824, #939, #862, #1115: #777; #939 also #895; #1115 also #939.
- #902: D-SUP-5 (+ #1021).
- #781: D-SUP-4 (not ready, F7).
- #784: D-SUP-8 after #783 lane.
- #826 → #1116 → #1117, #1118, #863: D-SUP-6 and D-SUP-8; #826's #777 edge should be removed (F4).
- #798 → #841 → (#990, #960, #943, #887 in other epics): #1021, D-SYS-2.
- #800 → #844 → #882 → #919, #932, #918, #971; #1123 feeds #882; #1119, #1120 depend on #800; #883 on #844 + #776. Common roots: #1021, D-SYS-4, D-SYS-5, #773/#821/#1129 as currently declared (F12).
- #799: #1021 and ADR 0014/0024 amendment (D-SYS-3).
- #971: #950, #919, #932, #799, #1120, and D-SYS-7; #1124 follows #971.
- #1055: #1021 (and D-SYS-7).
- #842: #1066 (approval of narrowed AC1); in flight as PR #1013.
- #843 → #1122: #767, #792, #789, #1040, #1041.
- #801, #802, #803: Owner exclusion decisions (O-6).
- #1121, #1128: after #1121/#1124 decisions; #1128 should depend on #800/#844 only (F15).

**No cycle found in the declared edges.** Missing or one-sided edges: #1115 lacks #939; #882 has no edge to #1119 (disabled-phase skip is part of the same `run` loop, `A/CoordinatedShutdown.scala:730-734`); #950 timeout failure uses #918 (declared); no edge from #1124 to #799 although both change Terminate/WhenTerminated.

---
## 7. Pekko references with no spec or limited search; items not read

**No spec found (searches stated):**
- Failed/Recreate system-message ordering and dead-letter disposition (#782, #783): searched `Recreate(`, `systemDrain`, `sendAllToDeadLetters` in `actor-tests/src/test`, `testkit/src`: no hits. Related, not covering it: `SystemMessageListSpec.scala:26-109` (list structure), `SupervisorHierarchySpec.scala:870,923` (log filter only). Pekko evidence is code only, not proven by tests.
- Survivor restart order (#784): no spec; `SupervisorHierarchySpec` was only grepped for "survivors".
- User decider that stops or watches (#1117, #1118): reconciliation says none; I did not grep exhaustively.
- Context expiry (#826): no Pekko counterpart found.
- `/system` after `/user` order (#1055): no Classic spec.
- StoppingStrategy as a child strategy: only guardian config (`ActorSystemSpec.scala:336-356`) and `TcpListenerSpec.scala:201` use it.

**Read but limited:**
- `Mailbox.scala:289-345`, `ActorCell.scala:495-506`, `dungeon/Dispatch.scala:158-162`: first lines only.
- `CoordinatedShutdown.scala`: read lines 127-157 (names only), 239-260, 296-330, 336-362, 436-448, 505-525, 560-660, 722-790, 808-822. Not read: reason classes in detail, `Future.sequence` at :491, `TaskDefinition.cancel` :466, the watcher actor :919-975 (only the error text at :970).
- `CoordinatedShutdownSpec`: read test names for the whole file, bodies for :123-168, :365-500, :535-580, :692-760. Not read: :170-364 (cancel tests), :501-535.
- `ActorSystem.scala`: read :651-663 region docs partially, :770-782, :920-926, :1085-1105, :1165-1228, :1300-1335. Not read: name regex :840-843, `Settings` class, `aborting`.
- `ExtensionSpec.scala:70-97` read; :99-135 (config-listed extensions) not read.
- `ActorAdapter.scala:83-101, 238-272` read.
- `TypedActor.scala`: only deprecation lines grepped. #801/#802/#803 anchors are from the issue.
- Go: supervision_decider.go, supervision_hooks.go, supervision.go (top), supervision_logging.go, system_shutdown.go, guardian.go (:100-180), hierarchy.go, errors.go, watch.go, system.go Config. Test files were located by name only; no `go test` run.

**Outside this group, relevant:**
- BackoffSupervisor / BackoffOnRestartSupervisor (`pattern/internal/BackoffOnRestartSupervisor.scala`) is covered only by PAT stories (#909, #941, #968, #981, #875). Not audited here. It overrides `supervisorStrategy` (so #777 is a real prerequisite for #909, #941).
- LoggingAdapter and Settings/Config: EVT epic and GOV-25 (#1074), GOV-29 (#1078), GOV-30 (#1079).
- ChildRestartStats is internal (no story); its algorithm is covered by #779/#825.
- ADR 0014-implementation-decisions and ADR 0024: not read for this group.
- `h4-repro/RESULTS.md` (ADR 0017's measured matrix) not present in the snapshot, not read.
