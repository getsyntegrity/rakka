# Report jdg-cls-tkt: epics #1017 JDG, #1018 CLS, #1019 TKT (read-only audit)

Baselines: Pekko v1.7.1 `9a54d12` (`upstream/pekko`), goPekko `origin/develop` `9abfb40` (snapshot `tmp/develop/`). Nothing edited anywhere. Paths: Pekko paths relative to `upstream/pekko`; judge paths relative to `tmp/develop/judge` (copies of `comparator.py`, `mutate_go.py`, `run-pekko.sh`, `judge.sbt`, `go-scope.json` in `tmp/cmp/`).

Method notes
- The bundles contain NO "ACT2 reconciliation" block (grep count 0 in epic1017/1018/1019 bundles and in `issues_all.json` except #746). Dependencies below are taken from each story's in-body "Dependencies" table; #1094/#1095 carry inline "(added; master listed only JDG-02)" remarks. Not verifiable here: the Architect's separate reconciliation text.
- Taxonomy for these stories: the stories are documentation/verification/tooling/test-support work, so "state" = decision state of the contract the story depends on (1-7) plus implementation state. Scope of testkit subset is state 1 (ADR 0033 D10); Go API of each testkit piece is state 4 (not designed) with Architect leanings (state 2 where the body states a lean).
- Dependency-table priority/readiness inversions were computed mechanically from labels in `issues_all.json`.

## 1. Coverage inventory

Depth rule applied honestly: DEEP = story body read fully + every cited Pekko line I could check opened + ADR 0033 text + plan/judge.md. Where I opened Pekko source but not the cited spec body, that is stated in the matrix.

| # | Key | Prio | Ready label | Depth | State (taxonomy) | Note |
|---|---|---|---|---|---|---|
| 1026 | JDG-01 | P1 | ready | DEEP | contract 1 (D7/D13) + plan rows are Architect proposal (2); impl 5 | readiness questioned, F4/F14 |
| 1027 | JDG-02 | P1 | blocked | DEEP | 4 (event kinds/clauses/mutants not designed) | cited lines verified (Stash.scala:245, ReceiveTimeout.scala:37/40, Timers.scala:41-48, FaultHandling.scala:218); spec bodies not opened |
| 1028 | JDG-05 | P1 | blocked | DEEP | 4 | clock/scheduler facts verified in ExplicitlyTriggeredScheduler.scala |
| 1029 | JDG-06 | P1 | blocked | SHALLOW+ (body scope/risks/deps read, Stash.scala:243-252 verified, spec bodies not opened) | 4 | |
| 1030 | JDG-07 | P1 | blocked | SHALLOW+ (as above; ReceiveTimeoutSpec TimingTest count checked) | 4 (ADR 0010 exclusion unclassified, GOV-06 #1037) | |
| 1032 | JDG-08 | P1 | blocked | SHALLOW+ (ActorCell.scala:575-576 and FaultHandling.scala:218-221 verified) | 4 (GOV-07/08) | |
| 1034 | JDG-04 | P1 | blocked | DEEP | 4; approval states 2 | F13 |
| 1046 | JDG-03 | P1 | blocked | DEEP | 4 | build.sbt:481/579/584, Behavior.scala:184/240/270, Supervision.scala:44/198/210 verified |
| 1047 | JDG-11 | P1 | blocked | DEEP | 4 (design pending) | |
| 1048 | JDG-12 | P1 | blocked | DEEP | 4 (Typed supervision ADR #1063 not accepted) | Pekko Supervision.scala:434-437, SupervisionSpec:508-531 read |
| 1094 | JDG-09 | P2 | blocked | SHALLOW+ (CircuitBreaker nanoTime lines and BackoffOnRestartSupervisor scheduler lines verified) | 4 | |
| 1095 | JDG-10 | P2 | blocked | SHALLOW+ (RoundRobin.scala:83, RoundRobinSpec:61 verified) | 4 (GOV-12 #1042 open) | |
| 1097 | JDG-13 | P1 | blocked | SHALLOW+ (Atomic/Requirements/Deps read; mutate_go.py:7,462 verified) | 4 | F2 |
| 1033 | CLS-01 | P2 | blocked | DEEP | 5 | F3, F12 |
| 1045 | CLS-02 | P2 | blocked | DEEP | 5 (needs GOV-19) | F3 |
| 1049 | CLS-03 | P2 | blocked | DEEP | 4 (tooling gap, stale) | F2 |
| 1050 | CLS-04 | P2 | blocked | DEEP | 4 | |
| 1135 | TKT-01 | P1 | blocked | DEEP (TestProbe.scala, TestProbeImpl.scala read) | scope 1, Go API 4 | F7 |
| 1136 | TKT-02 | P1 | blocked | DEEP (TestProbeImpl:172-225) | same | F9 |
| 1137 | TKT-03 | P1 | blocked | DEEP (within_internal 136-157) | same | F9, F12 |
| 1138 | TKT-04 | P2 | blocked | DEEP (fishForMessage_internal) | same | |
| 1139 | TKT-05 | P1 | blocked | DEEP (expectTerminated_internal) | same | |
| 1140 | TKT-06 | P1 | blocked | SHALLOW+ (TestInbox.scala API list checked, impl body not) | same | |
| 1141 | TKT-07 | P2 | blocked | SHALLOW+ (ReplyInbox/StatusReplyInbox API list checked) | same | |
| 1142 | TKT-08 | P1 | blocked | SHALLOW+ (BehaviorTestKit trait members checked) | same | F16 |
| 1143 | TKT-09 | P1 | blocked | SHALLOW+ (Effect.scala Spawned/Stopped checked) | same | |
| 1144 | TKT-10 | P2 | blocked | SHALLOW+ (Effect list checked) | same | |
| 1145 | TKT-11 | P2 | blocked | SHALLOW | same | |
| 1146 | TKT-12 | P1 | blocked | SHALLOW+ (ActorTestKitSpec:38-73 read) | same | F17 |
| 1147 | TKT-13 | P1 | blocked | SHALLOW | same | |
| 1148 | TKT-14 | P1 | blocked | DEEP (ActorTestKit:305-325, ActorAdapter.scala:220-229) | same | F8 |
| 1149 | TKT-15 | P1 | blocked | DEEP (ManualTime.scala, ExplicitlyTriggeredScheduler.scala read) | same | F10 |
| 1150 | TKT-16 | P2 | blocked | SHALLOW | same | |
| 1151 | TKT-17 | P2 | blocked | SHALLOW | same | |
| 1152 | TKT-18 | P2 | blocked | SHALLOW | same | |
| 1153 | TKT-19 | P2 | blocked | SHALLOW+ (Effect.scala:271-302 and docs testing-sync.md:8-20 checked) | same | |
| 1154 | TKT-20 | P2 | blocked | SHALLOW+ (Effect.scala:179-220 checked) | same | |

Counts (37 stories): DEEP 18 (1026,1027,1028,1034,1046,1047,1048; 1033,1045,1049,1050; 1135-1139,1148,1149), SHALLOW 19 of which 14 marked SHALLOW+ (1029,1030,1032,1094,1095,1097,1140-1144,1146,1153,1154) and 5 plain SHALLOW (1145,1147,1150,1151,1152), NOT REVIEWED 0. Epic bodies #1017/#1018/#1019 read for scope only.
SHALLOW+ means: body Atomic/Requirements/Dependencies/Traceability read, key Pekko symbols checked in source, spec bodies NOT opened. No story above had every acceptance criterion checked against spec bodies.

## 2. Capability matrix (judge/oracle/testkit)

### 2.1 Judge today vs ADR 0033 D13 / V3

| Capability | Pekko contract | Go contract | State | Implementation in develop | Evidence | Gap |
|---|---|---|---|---|---|---|
| Existing Classic judge (S01-S22 Pekko, 15 on Go) | docs/judge.md; `Judge.scala` 572 lines (CallingThreadDispatcher+ExplicitlyTriggeredScheduler at :8,25,199,250) | docs/judge.md, comparator.py | 7 for its scope (lifecycle, supervision directives OneForOne, deathwatch, FIFO, dead letters, ask; calibrated only S19, S21) | `judge/` present; `go-scope.json` lists 15 scenarios (S03 S04 S05 S06 S12 S15 S17 Pekko-only) | differential + mutation reports in repo | 7 scenarios Pekko-only: these rows are not Go-V3. Closed vocabulary `EVENTS` 16 kinds (`comparator.py:61-65`), 31 clauses (`PARAMS` 65-72, `RULES` 455-459) |
| Judge for Typed (D13 consequence "new Typed judge family") | no typed harness; `judge.sbt` depends only on `LocalProject("actor")`, `testkit` | JDG-03 design, JDG-11, JDG-12 | 4 | absent (`ls`, go.work) | none | see F1, F15; sbt LocalProject name for actor-typed not verified |
| Timers/scheduler | Timers.scala:41-48, dungeon/TimerSchedulerImpl.scala:103-107; Scheduler.scala:90-100: fixed-delay task is rescheduled after each successful `run()`; a `SchedulerException` (or ISE caused by one) is ignored without rescheduling; any other exception leaves `run()` without rescheduling | JDG-02/05 | 4 | absent in Go | Pekko classic TimerSpec: 7 of 10 cases are `TimingTest` (untagged :301, :308, :346); typed TimerSpec has cases without `TimingTest` (:256, :283, :308, :327, :372) | controlled mode possible (scheduler abstraction), but ExplicitlyTriggeredScheduler.timePasses has `Thread.sleep(100)` (:67-72) |
| Classic stash | Stash.scala:243-252 | JDG-06 | 4 | absent | | concurrent-sender replay left N-tier unless JDG-02 decides |
| ReceiveTimeout | dungeon/ReceiveTimeout.scala:33-62 | JDG-07 | 4 (GOV-06 open) | absent | ReceiveTimeoutSpec 11 of 13 cases `TimingTest` (grep) | JDG-07 modelled on spec lines 82/97/115 (TimingTest) and 313 (not TimingTest) |
| PoisonPill / Kill | ActorCell.scala:575-576; FaultHandling.scala:218-221 | JDG-08 | 4 (GOV-07/08 open) | absent | ActorLifeCycleSpec uses Kill | |
| Backoff / CircuitBreaker | BackoffOnRestartSupervisor.scala:74,100 (scheduler); CircuitBreaker.scala:724,740,756,864,1079,1106 `System.nanoTime` | JDG-09 | 4 | absent | | breaker concurrent-only (story states it, verified) |
| Classic routing | RoundRobin.scala:83; RoundRobinSpec:61 | JDG-10 | 4 (GOV-12 open) | absent | | N1 order across senders not asserted (ok) |
| Typed restart within limit, StashBuffer, timers | Supervision.scala:44,198,210,434-437; StashBuffer.scala:45; TimerSchedulerImpl.scala:86,93 | JDG-12 | 4 | absent | SupervisionSpec:285, StashBufferSpec:25, TimerSpec:34 (classes verified) | `withLimit` window uses `Deadline.now` (Supervision.scala:434-437) and spec uses real 500 ms `resetTimeout` (SupervisionSpec:529-531): wall-clock, concurrent-only, as docs/judge.md:179 states for Classic. JDG-12 does not state the mode. |
| Typed mutation tooling | n/a | JDG-13 #1097 | 4 | `mutate_go.py:7,462` copies `pekko-actor/` only | | F2 |

### 2.2 Oracle availability per capability (my assessment; JDG-04 is the story that must record it)

| Group | Pekko oracle exists as spec/source | Drivable by the event-log judge today | Proposed class (not a decision) |
|---|---|---|---|
| Lifecycle, supervision (OneForOne), deathwatch, dead letters, ask | yes | yes (S01-S22) | pekko-differential |
| Classic timers, scheduler cancel, stash, ReceiveTimeout, PoisonPill, Kill, backoff, breaker, round-robin | yes | after JDG-02 + harness change | pekko-differential, V3 needs JDG-05..10 |
| Typed restart-limit, stash, timers | yes | after JDG-11 | pekko-differential (wall-clock caveat) |
| become/unbecome/unhandled, ActorSelection/Identify, FSM (15 stories), mailbox variants (MBX 16), dispatcher options, EventStream/EventBus/Dropped/UnhandledMessage, CoordinatedShutdown, Extensions, Props/Deploy, serialization primitives, Address/ActorPath/StatusReply value semantics, Typed Behavior core, interceptors, adapters, SpawnProtocol, Receptionist, Routers, Topic, Delivery | yes (source and specs) | NO story creates a judge scenario | UNCLASSIFIABLE with a binary scheme: not `go-only` (Pekko source/specs exist; whether that is an 'applicable oracle' under D13 is undecided), not `pekko-differential` (no vehicle). See F1/F13 |
| Typed testkit (TestProbe, TestInbox, BehaviorTestKit, ActorTestKit, ManualTime) | yes (testkit specs) | no: judge compares lifecycle event logs, testkit is an assertion API | needs Owner definition of V3 for testkit |
| Port, OrdinaryFailure, ResumeWith, Config seams, kit-logger mapping, Go idioms replacing futures, S-tier relations | no | n/a | classified go-only by plan 5.2; oracle applicability pending determination per row, alternative evidence and approval under ADR 0033 D13 only where the oracle is shown not to apply |

## 3. Findings

**F1 (high) D13 V3 has no vehicle for most oracle-bearing rows.** ADR 0033 D13: V3 "requires a differential scenario against Pekko whenever an applicable oracle exists". Judge-execution stories cover only Classic timers/scheduler (JDG-05), stash (06), ReceiveTimeout (07), PoisonPill/Kill (08), backoff/breaker (09), round-robin/pool (10) and Typed restart-limit/stash/timers (12). JDG-12 itself says "Typed backoff (#986), restart waiting for children, restart stash overflow: later stories" and no such stories exist. Rows with Pekko oracles and no judge story: become/unbecome/unhandled, ActorSelection, FSM, mailbox variants, EventStream/Dropped/UnhandledMessage, CoordinatedShutdown, Extensions, Typed Behavior core/interceptors/adapters/ask, SpawnProtocol, Receptionist, Routers, Topic, Delivery, Terminated sender rows. JDG-04 R1 forces `pekko-differential` or `go-only` per row, so those rows either claim a scenario that does not exist or are mislabelled `go-only` (which would generate mass Owner approvals for rows that do have an oracle). Open question (Owner): what 'applicable oracle' means in D13 for rows that have Pekko source/specs but no judge scenario story (list above). Plan 5.2 rows 2-3 record some as 'V3 needs B-02/B-03 first'. Missing evidence: an Owner interpretation. No recommendation. Evidence: ADR 0033 D13, plan 5.2 row 2-3 ("V3 needs B-02 first"), JDG-12 Out of scope.

**F2 (high) Stale "no owner for typed Go mutation tooling"; #1097 not wired.** #1049 (CLS-03) "Blocking gap: no story owns typed Go mutation tooling" and #1047 (JDG-11) Risks say the same, but #1097 (JDG-13) exists to build it (`finding:mutate-typed`). #1049 depends only on #1048; #1048 R3 says Go mutants run "through `judge/mutate_go.py`" which copies only `pekko-actor/` (`mutate_go.py:7`, `:462`), so R3 cannot be executed for typed Go mutants without #1097. #1097 is not listed as a dependency of #1048 or #1049 (its only consumers named nowhere). Also #1097 depends on #1096 TYP-34 (module skeleton) and #1046, correct. Correction: add `depends_on` #1097 -> #1048 (Go mutants) and #1097 -> #1049; rewrite the "no story owns" sentences in #1047/#1049 as "owned by #1097". Mind-map of #1017 shows JDG-13 under JDG-03 only.

**F3 (high) Closure protocol contradiction: README edit stales the Go mutation report.** docs/judge.md:319: the committed `judge/go-mutation-report.tsv` is CURRENT only while the git tree of `pekko-actor/` "and also its README.md, go.mod and go.sum" is unchanged. CLS-01 AC-R2-1 requires `go-mutation-report.tsv` with CURRENT provenance at the recorded SHA; CLS-02 (depends_on CLS-01) then edits `pekko-actor/README.md` (local-frontier section). After CLS-02 merges, the report is stale at the closing commit; CLS-02 notes the risk as "non-blocking" and says only "run order matters". Same for CLS-04 if it touches shared tooling. Correction: define the closure SHA protocol: either README (CLS-02 part) lands before CLS-01 and CLS-01's SHA is the closing SHA, or the audit states explicitly that the report is of SHA X and the README change is documentation-only and re-run is not required (Owner decision). Evidence: judge.md:319, CLS-01 AC-R2-1, CLS-02 Risks.

**F4 (high) Three key spaces, no crosswalk; JDG-01 freezes plan gaps.** (a) #746 body ACTINV-001..126 is the legacy inventory (126 rows, classic+typed only; no judge, closure or testkit rows). (b) Plan 1.2 rows (C-xx, D-xx, E-xx, P-xx, R-xx, S-01, TY-xx, K-01..K-07, ...). (c) Story "Inventory row" fields: `plan row B-01`, `P-03a`, `testkit:M-01a..e/M-02a..f/M-03a..c/M-04/timer-effects/adapter-effects`, `finding:mutate-typed`, `ACTINV-nnn`, `C-14`, `gap:g5-36`... JDG-01 R1 requires "exactly the rows of plan 1.2/1.3", yet testkit stories cite `testkit:M-0x` (plan slice M ids), which are not plan 1.2 row ids (1.2 has K-01..K-04). My count (title-matched `[ACT2-xx-nn]` stories): 421; 243 referenced from the ACTINV table; 91 stories outside GOV/JDG/CLS/TKT are not referenced from it (TYP 18, TSD 14, SYS 11, TIM 7, SUP 7, PAT 7, IDN 7, RTE 5, FSM 5, EVT 4, MSG 2, MBX/DTH/BHV/ASK 1 each). `ACTINV-011` is a dangling reference (design.md:66, ADR 0032:10). `docs/parity/*` contains no ACTINV ids (capabilities.json is the persistence bootstrap, `"capabilities": []`). Observed: three key spaces without crosswalk (counts above). Open question (Owner): how JDG-01 R1 'exactly the rows of plan 1.2/1.3' treats rows outside the plan. No recommendation.

**F5 (medium) Inventory coverage gaps vs the real Pekko surface** (checked by listing public files under `actor/src/main/scala/.../{actor,event,pattern,routing,io,serialization,util,dispatch,japi}`, `actor-typed/.../typed/**`, `actor-testkit-typed`, `testkit`; filename found in no issue body and no `develop/docs`: 56 actor files, 3 typed). Real gaps (not just implementation classes):
- Classic `testkit` (K-08 in dossier 4.5, `completion-design.md:218`: "OD; only what ported specs need", 4,301 lines: `TestKit`, `TestActorRef`, `TestFSMRef`, `TestLatch`, `TestBarrier`, `EventFilter`, `TestActors`, `CallingThreadDispatcher`, `ExplicitlyTriggeredScheduler`, `SocketUtil`) is absent from plan 1.2 (K-01..K-07 only) and has no GOV story or exclusion row; D10 names only the Typed subset.
- `TestException` (typed testkit; ~100 uses in `actor-typed-tests`): no story, no exclusion.
- `util.TypedMultiMap` is used by `LocalReceptionist.scala` and `ServiceKey.scala` (typed/internal/receptionist) but U-02 ("on demand") lists WildcardIndex, SubclassifiedIndex, MessageBuffer, TokenBucket, BoundedBlockingQueue, not TypedMultiMap. U-02 `R on demand` is an unclassified scope (state 4) with no owner.
- `AbstractActor`, `AbstractFSM`, `AbstractProps` (Java API in package `actor`), `ClassicActorSystemProvider`, `event/jul`, `event/japi`, `dispatch` JVM executors (`ForkJoinPoolConstants`, `ThreadPoolBuilder`, `VirtualizedExecutorService`, `affinity/AffinityPool`), `serialization/WireConfig`: only covered by group rows (J-01, D-04, TY-23). Group-level exclusion is arguably fine, but GOV-19 (#1044) scope is "ADR 0033 D8, D9, D12, plan 2.3 and GOV-04..14", so plan-1.2 X rows (J-01, U-01, U-03, T-00, D-04, TY-22, TY-23, K-05, K-07) are not in the register scope, while D13/D8 require every exclusion with source and reason.
- `pattern.extended` (ExplicitAskSupport), `routing.Listeners`, `event.AddressTerminatedTopic`, `event.LoggerMailbox`, `dispatch.Future`/`CompletionStages`: no mention anywhere.
Observed: three key spaces without crosswalk (counts above). Open question (Owner): how JDG-01 R1 'exactly the rows of plan 1.2/1.3' treats rows outside the plan. No recommendation.

**F6 (medium) JDG-01 `status:ready` has internal tension.** Scope says oracle availability is out of scope (JDG-04), yet AC-R2-1 requires rows in the plan-5.2 "no oracle" group to be labelled `pending approval`; R2 also demands "owning story key" per row, i.e. the traceability matrix, plus per-row alternative evidence/approval reference which D13 says the inventory carries while JDG-04 puts them in a separate register (two documents, same row set; JDG-01 Risks admits drift). Correction: one owner for the `pending approval` column (Owner decision; no recommendation), or state JDG-04 register is a section of the same document.

**F7 (medium) TKT-01 dependencies incomplete.** Pekko probe is a system actor named `name-N` created with `system.systemActorOf(...)` (`TestProbeImpl.scala:99`, counter shared with classic TestKit `:84`), behavior built with `Behaviors.receive/receiveSignal` (`:56-75`). TKT-01 depends only on #1096 (skeleton); it names "system-actor spawn export needs ADR 0011" in Risks but not #1055 SYS-18 (system actors under the guardian) nor #847 TYP-02 (Behavior core, which TKT-06/08 do list). Correction: add `depends_on` #847 and #1055 (and GOV-02/#1020 index entry).

**F8 (medium) TKT-14 misses the UnhandledMessage publisher.** Pekko typed `unhandled` falls to classic `super.unhandled` (`ActorAdapter.scala:220-229`) which publishes `UnhandledMessage`. TKT-14 lists #871 (suppressed dead letters) and #906 (Dropped) as "Dropped and Unhandled event publication", but neither publishes UnhandledMessage; the publisher is #897 BHV-03 (and Typed #925 TYP-03 for the `unhandled` result). AC-R2 "unhandled probe receives UnhandledMessage" is unreachable without them. Also TKT-14 (P1) depends on TKT-04 (P2). Correction: add #897 and #925.

**F9 (medium) GOV-36 dependency inconsistency.** TestProbe reads `TestKitSettings` (`TestProbeImpl.scala:86`): default `expectNoMessage` timeout (:217, 100 ms per story), `dilated` time factor, single-expect default, shutdown timeout (`TestKitSettings.scala:60-100`). TKT-03 `depends_on` #1085 (P2, blocked), TKT-01/TKT-12 mark it `independent` ("fixed defaults until then"), TKT-02 (uses the default) lists no GOV-36 relation, TKT-15 states "no dilation". Observed: TKT-03 `depends_on` #1085, TKT-01/TKT-12 mark it `independent`, TKT-02 lists no relation, TKT-15 states 'no dilation'. Pekko: the probe reads `TestKitSettings` (`TestProbeImpl.scala:86`) and dilates `within`/`expectMessage` (:125,162). Open question (Owner): how GOV-36 (#1085) relates to these stories. No recommendation.

**F10 (medium) TKT-15/ManualTime semantics underspecified.** `ExplicitlyTriggeredScheduler.timePasses` does `Thread.sleep(100)` first "to give dispatchers time to clear" (:67-72), runs tasks inline in time order (`executeTasks` :93-107), `Try` swallows a runnable failure and then does NOT reschedule an interval task, `cancel()` is true only if removed (:124-130), a zero/negative initial delay executes at once (:117-119). TKT-15 R1 "run every job due within it" is true but says nothing about whether effects on actor mailboxes are complete on return (the source comment at `ExplicitlyTriggeredScheduler.scala:68-71` says 'Give dispatchers time to clear'; Pekko states no completion guarantee). Pekko test sources: classic `ExplicitlyTriggeredSchedulerSpec.scala` (8 cases) and the docs examples `ManualTimerExampleSpec.scala` / `ManualTimerExampleTest.java`; no ManualTime use in actor-typed-tests (0 files). Pending verification: whether these count as an applicable oracle for TKT-15 under ADR 0033 D13. Missing evidence: an Owner reading of 'applicable oracle' (D13 does not define it). Correction: add acceptance criteria for failing job, repeated job and cancel return value; state the Go guarantee; record the oracle class.

**F11 (medium) D10 rationale vs real use; LoggingTestKit gap.** Dossier D-07 table says the subset "matches what actor-typed-tests imports". Verified: `LoggingTestKit` appears in 24 files of `actor-typed-tests` (e.g. SupervisionSpec:521 `LoggingTestKit.error[Exc2].expect`, ActorContextSpec, WatchSpec, LogMessagesSpec, TransformMessagesSpec) while `ManualTime` appears in 0 (docs only, dossier 4.5 K-04 admits it). LoggingTestKit is excluded by D10 and replaced by the logger fake (D11, EVT-15 #1052), but only TKT-18 maps the fake to `BehaviorTestKit.logEntries`; no story provides an `ActorTestKit`-level log expectation. V1 ports of SupervisionSpec/WatchSpec/LogMessagesSpec need it. Open question (Owner, GOV-36 #1085): what replaces `LoggingTestKit` expectations (D10 excludes logback tools; D11 specifies a logger fake). Pekko: `LoggingTestKit` used in 24 files of actor-typed-tests.

**F12 (medium) Priority/readiness inversions** (computed from labels; `depends_on` producer lower priority than consumer): JDG-12 P1 <- TYP-15 #977 P2; JDG-10 P2 <- RTE-04 #912, RTE-07 #878 P3; TKT-03 P1 <- GOV-36 P2; TKT-14 P1 <- TKT-04 P2. CLS-01..04 are P2 although plan 8.1 places P-03 at the end of the critical path (JDG-05..08 P1, plan 7.4 table assigns no priority to Slice P). Plan 7.4 lists only B-01..B-04 and M-01..M-04 as P1. All 37 stories except JDG-01 are `status:blocked`, consistent with their dependency tables; JDG-01 `ready` consistent only if F6 is fixed.

**F13 (medium) JDG-04 binary classification.** R1 "exactly one oracle class" per row conflicts with its own risk "a row may mix an oracle part and a Go-only part; the register must split it", and with AC-R1-1 "same row set" as the inventory. Plan 5.2 itself has a third state ("yes in Pekko; judge harness does not yet drive them"). Observed: JDG-04 R1 'exactly one oracle class' vs its own risk 'a row may mix an oracle part and a Go-only part'. Open question (story author/Owner): which wins. No recommendation.

**F14 (low) JDG-01 vs D13 wording.** D13: "The inventory carries, per such row, the alternative evidence and the approval reference". JDG-01 puts them as columns (ok) but R2 makes the label rule only; no AC checks the alternative-evidence column content (that is JDG-04). Covered under F6.

**F15 (medium) Judge feasibility limits not stated for Typed/Classic additions.** (a) Typed `withLimit` uses wall-clock `Deadline.now` (`Supervision.scala:435`); `resetBackoffAfter` is scheduled with `ctx.scheduleOnce` (`Supervision.scala:399`), not wall-clock. JDG-12 does not state the mode. Not drivable by a manual scheduler, so JDG-12's restart-limit scenario is concurrent-only; story silent. (b) No Pekko typed spec runs on `CallingThreadDispatcher` (grep `CallingThreadDispatcher|calling-thread` in actor-typed, actor-typed-tests, actor-testkit-typed: 0 hits; only `testkit/src/main/resources/reference.conf:31`), so the Typed controlled mode of JDG-03 has no spec precedent; JDG-03 says controlled/concurrent split is TBD, fine, but the claim "typed testkit ManualTime/BehaviorTestKit do not replace the judge" is design only. (c) Typed restart sends only `PreRestart` (plan 5.3); existing G4 clauses require `pre-restart -> post-stop -> post-restart -> pre-start` (judge.md G4), so Typed needs a new clause family; `HOOKS` in comparator is a closed tuple (`comparator.py:32`), `G5.f` counts keyed by those hook names. JDG-03 R3 says clauses are defined; ok but dependent on GOV-01 #1063 (state 2/4). (d) `ExplicitlyTriggeredScheduler.timePasses` Thread.sleep(100) per call makes controlled timer scenarios slow, relevant to the 300-run cost.

**F16 (medium) BehaviorTestKit failure semantics not covered.** TKT-08 R1-R5 describe run/runOne/signal/effects; no AC covers what happens when the behavior throws or a `Terminated` is unhandled (DeathPactException) inside `run`, which differs from the supervised runtime (docs testing-sync.md:8-20: "Supervision is not supported"). Not verified against BehaviorTestKitSpec bodies. Also plan 1.1 K-03 and plan M-02 text say the kit "does not support timers"; TKT-19 correctly reads `Effect.TimerScheduled`/`TimerCancelled` (`Effect.scala:271-302`, `EffectfulActorContext.scala:162-206`) so plan wording (M-02 "no timer or supervision support (as upstream)") is partly wrong: scheduler (`StubbedActorContext.scheduleOnce` cancel false) and supervision are unsupported, keyed timers are recorded.

**F17 (low) TKT-12 JVM-only name behaviour.** `ActorTestKit()` derives the name from the call stack and a class name (`ActorTestKitSpec:50-73`); TKT-12 R1 covers the sanitising rule only. The call-stack default and `ApplicationTestConfig` have no Go meaning; only the config part is recorded as adaptation. Also `ScalaTestWithActorTestKit`/`ActorTestKitBase` (excluded) leave the Go fixture pattern (t.Cleanup, failer interface) to TKT-01 open choice.

**F18 (low) TKT-02 R4 wording.** "expectNoMessage MUST consume nothing": in Pekko the poll removes a message before failing with "Received unexpected message" (`TestProbeImpl.scala:201-225`). TKT-03 R4 omits the nested assertion "required min time ... not possible, only ... left" (`:140-141`). TKT-03 traceability says 'no dedicated within spec'; Pekko has two success-path uses (`TestProbeSpec.scala:42`, `TestProbeTest.java:136-137`).

**F19 (low) CLS-04/CLS-01 dependency gaps.** CLS-04 audits "testkit-subset rows" but depends on no TKT story; CLS-03 on no TKT story. Harmless (audit can conclude not closable) but the testkit rows would then always read open. CLS-01 excludes JDG-09/10 (Owner choice recorded); consequence: backoff/breaker/routing rows stay without judge result, so CLS-02 R4 "no row absent..." cannot hold for them unless excluded or pending approval (stated in story).

## 4. Owner decisions needed

| # | Decision | Alternatives and consequences | Relevant evidence (ADR/Pekko) |
|---|---|---|---|
| OD-1 | Meaning of "applicable oracle" in D13 (F1) | (a) judge-drivable only: rows without a scenario story need alt evidence + approval, or new judge stories; (b) any Pekko spec counts, V3 = ported spec + race + campaign, judge only where event-log fits | none (Owner decision between the listed alternatives) |
| OD-2 | V3 for testkit rows (TKT) | judge cannot compare assertion APIs: (a) V1 ports of testkit specs + Go tests, mark `go-only`/`pending approval`; (b) design a differential harness for the testkit | none (Owner decision between the listed alternatives) |
| OD-3 | Testkit location and failure surface (open in TKT-01, repeated in 19 stories) | package of `pekko-actor-typed` vs module `pekko-actor-typed-testkit`; failer interface vs error vs panic | body says Architect leans to package; keep as proposal |
| OD-4 | Classic testkit (K-08) disposition | include a Go classic probe vs rely on inline test helpers vs exclude | none given here |
| OD-5 | Closure SHA protocol (F3) | README-before-campaign vs documented re-run exemption | none (Owner decision between the listed alternatives) |
| OD-6 | What replaces LoggingTestKit expectations (F11, GOV-36 #1085) | logger fake only (TKT-18) vs an ActorTestKit-level expectation helper | none |
| OD-7 | Whether JDG-09/JDG-10 precede the close (CLS-01 design) | rows stay open or excluded | per body: Owner |
| OD-8 | U-02 util classes and TestException disposition | port on demand vs record exclusions now | ADR 0033 D8: 'Every exclusion is recorded with its source and reason'; timing (now vs on demand) is an Owner decision. |

## 5. ADRs needing creation/amendment/acceptance (not changed)

- Typed supervision ADR (#1063 GOV-01, not accepted): gates JDG-12 scenario content (ADR 0033 D2).
- Typed Behavior boundary ADR (TYP-01 #809): gates #847 and every Typed harness/testkit story.
- ADR 0011 individual export justifications (GOV-02 #1020 index, GOV-03 #1021 form): needed by system-actor spawn (TKT-01), injectable clock (TKT-15 via TIM-11 #1129), function-backed ActorRef (TKT-06), event probes.
- A note/amendment to ADR 0033 D13 or the inventory doc on the "applicable oracle" interpretation (OD-1). Not proposing an edit, only that the question needs a recorded Owner answer.
- ADR 0033 D10 rationale paragraph in the dossier (D-07) cites actor-typed-tests imports inaccurately (F11); dossier is accepted-by-reference, so any correction is by the delivery plan, not editing the ADR.
- ADR 0010 exclusions (ReceiveTimeout, PoisonPill, Kill, routers): GOV-06/07/08/12 must be Owner-classified before JDG-07/08/10 can start.

## 6. Blocked vs potentially startable

- Startable now (prerequisites satisfied): JDG-01 (#1026) only, subject to F4/F6. After it: JDG-02, JDG-03, JDG-04 (all depend only on #1026, parallel, docs).
- JDG-05 needs JDG-02 + TIM-05 #860 + TIM-06 #900 (+ TIM-01, TIM-11 transitively; A-03/GOV-02 export index). JDG-06 needs JDG-02 + BHV-06 #898 + BHV-08 #935 (+ GOV-04 decisions, SPLIT_REQUIRED on #935). JDG-07 needs JDG-02 + TIM-09 #861 + GOV-06. JDG-08 needs JDG-02 + MSG-06 #770, MSG-07 #816, MSG-09 #1031 (+ GOV-07/08). JDG-09 needs JDG-02 + PAT-09 #795, PAT-06 #909. JDG-10 needs JDG-02 + GOV-12 + RTE-04/07.
- JDG-11 needs JDG-03 + TYP-02 #847 (+ TYP-34 #1096, + decision on Go harness location). JDG-12 needs JDG-11 + TYP-10 #976 + TYP-13 #889 + TYP-15 #977 + (missing) JDG-13 + GOV-01. JDG-13 needs JDG-03 + TYP-34 (the tool can be written against a fixture module: marked `independent` of JDG-11).
- CLS-01 needs JDG-05..08; CLS-02 needs CLS-01 + JDG-04 + GOV-19; CLS-03 needs JDG-12 (+ JDG-13); CLS-04 needs CLS-03 + JDG-04 + GOV-19.
- TKT critical chain: TYP-34 -> TKT-01 -> TKT-02 -> (TKT-04, TKT-05[TYP-04], TKT-15[TKT-12, TIM-11]); TYP-02 -> TKT-06 -> TKT-08 -> TKT-09 -> TKT-10/11/19/20; TKT-12 needs TYP-29 #994. TKT-17 needs TSD-02 #960; TKT-18 needs EVT-15 #1052 (P1, ready: the one TKT prerequisite already startable); TKT-07 needs ASK-07 #774.
- Nothing in TKT can start before the module skeleton TYP-34 #1096 (P0, blocked by A-01/A-03/ADR) except TKT-18's prerequisite EVT-15.

## 7. References with no spec found / items not read

- ExplicitlyTriggeredScheduler: spec exists, `testkit/src/test/scala/org/apache/pekko/testkit/ExplicitlyTriggeredSchedulerSpec.scala` (8 cases, success paths and cancel). Not found: a spec for a failing runnable or for the `Thread.sleep(100)` in `timePasses`. Typed `ManualTime`: only docs examples (`ManualTimerExampleSpec.scala`, `ManualTimerExampleTest.java`).
- `within`: `grep within` in `actor-testkit-typed/src/test` finds 2 uses, `TestProbeSpec.scala:42` and `TestProbeTest.java:136-137`, both the success path only. No test of the failure assertions at `TestProbeImpl.scala:139,152,154` was found (pending verification: searched only the word `within`; not searched `required min time` / `exceeding`). `TimerSpec.scala:121` in actor-typed-tests also uses it.
- Typed on CallingThreadDispatcher: no spec found (searched actor-typed, actor-typed-tests, actor-testkit-typed for `CallingThreadDispatcher|calling-thread`).
- ManualTime in actor-typed-tests: none (docs examples only).
- Read only as source, spec body not opened: ReceiveTimeoutSpec cases 82/97/115/313 (only TimingTest tagging counted), ActorWithStashSpec 141/149, ActorWithBoundedStashSpec 126-133, ActorLifeCycleSpec 50-99, RoundRobinSpec, BehaviorTestKitSpec (all TKT-06..11,16..20 acceptance criteria), TestProbeSpec cases, ActorTestKitSpec cases beyond 38-73, TimerSpec/StashBufferSpec/SupervisionSpec bodies (class declarations and listed cases only).
- Not read: `Judge.scala` scenario bodies, `judge_internal_test.go`, `calibrate.py`, `evaluate.py`, `go_scope.py`, `scripts/check-judge-matrix.py` (statements about them are from docs/judge.md and story bodies). Not verified: sbt project id for `actor-typed` / `actor-testkit-typed` (`pekkoModule(...)`), SupervisionSpec line 550 and `:512,528` mapping used by JDG-12/I-02.
- Not checked: the 56 unmatched actor files individually for Go meaning; I judged by package group.
