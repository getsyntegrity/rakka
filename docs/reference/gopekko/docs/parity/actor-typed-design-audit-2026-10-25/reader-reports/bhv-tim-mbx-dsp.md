# Audit report: bhv-tim-mbx-dsp (epics #749 BHV, #751 TIM, #754 MBX, #755 DSP)

Baselines: Pekko v1.7.1 `9a54d12` (submodule); goPekko `origin/develop` `9abfb40` via snapshot `develop:`. Pekko paths relative to `upstream/pekko`; Go paths relative to `develop/pekko-actor/`. ADR 0032 is not in develop and is treated as not part of it. Nothing was edited. Live check (GET only): `gh` listed legacy #160 `[ACT-014] clock real y controlado I-1` CLOSED and merged PR #382 "unexported clock seam beside the executor (ADR 0015 I-1)".

Depth markers. DEEP = story body + reconciliation read, Pekko source and the cited specs opened, ADR text read. "DEEP*" = same, but one named spec body was not opened (stated in the note). SHALLOW = references only. Nothing is NOT REVIEWED at story level; some contract sub-points are listed in section 7.

## 1. Coverage inventory

State label legend: 1 Accepted, 2 Proposed, 3 Contradictory, 4 Not designed, 5 Designed not implemented, 6 Implemented not validated, 7 Validated. Where the story is a decision slice, the state is that of the contract it would produce. Key = stable key. All stories OPEN. "Ready" = label `status:ready`; none of the 46 except #791 is ready.

### BHV (epic #749, 9 stories)

| # | Key | Prio | Ready | Depth | State | Note |
|---|---|---|---|---|---|---|
| 771 | BHV-01 | P1 | blocked | DEEP | 4 (API shape) + exclusion decision 2 (GOV-04 #1035) | HotSwapSpec fully read (HotSwapSpec.scala:26-134) |
| 817 | BHV-02 | P1 | blocked | DEEP | 3 for AC3 (story says "restores", Pekko resets); rest 4 | GOV-49 #1196 pending |
| 897 | BHV-03 | P1 | blocked | DEEP | 4 | Go "unhandled" signal undefined (F-6) |
| 772 | BHV-04 | P1 | blocked | DEEP | scope 1 (ADR 0033 D4); contract 4 | decision story |
| 855 | BHV-05 | P0 | blocked | DEEP | 4 | AC1 wording wrong vs Pekko (F-3) |
| 898 | BHV-06 | P0 | blocked | DEEP | 4 | head insert owner conflict (F-2) |
| 899 | BHV-07 | P0 | blocked | DEEP | 4 | capacity location corrected by reconciliation, verified |
| 935 | BHV-08 | P0 | blocked | DEEP | 4 | split verified; mis-cite ActorWithStashSpec:149 (F-17) |
| 1025 | BHV-09 | P0 | blocked | DEEP | 4 | depends on ADR 0032 D3/D4/D6 (not in develop); F-4, F-12 |

### TIM (epic #751, 17 stories)

| # | Key | Prio | Ready | Depth | State | Note |
|---|---|---|---|---|---|---|
| 776 | TIM-01 | P0 | blocked | DEEP | 4 | ADR to produce; F-8, F-13 |
| 821 | TIM-02 | P0 | blocked | DEEP | 4 | non-positive delay rule missing (F-14) |
| 822 | TIM-03 | P0 | blocked | DEEP | 4 | |
| 823 | TIM-04 | P0 | blocked | DEEP | 4 | |
| 860 | TIM-05 | P0 | blocked | DEEP | 4 | wrapper/marker design issue (F-5) |
| 861 | TIM-09 | P1 | blocked | DEEP | 4 + exclusion decision 2 (GOV-06 #1037) | applies also to auto-received messages (F-15) |
| 900 | TIM-06 | P0 | blocked | DEEP* | 4 | TimerSpec 260-300 read; sequencing claim with SUP-11..14 not verified |
| 901 | TIM-10 | P1 | blocked | DEEP | 4 | |
| 937 | TIM-07 | P1 | blocked | DEEP | 4 | |
| 938 | TIM-08 | P1 | blocked | DEEP | 4 | |
| 1129 | TIM-11 | P0 | blocked | DEEP | internal seam 6 (Ask only); export 4 | F-9 |
| 1130 | TIM-12 | P2 | blocked | DEEP | 4 | |
| 1131 | TIM-13 | P1 | blocked | DEEP | 4 | |
| 1132 | TIM-14 | P2 | blocked | DEEP | 4 | |
| 1133 | TIM-15 | P1 | blocked | DEEP | 4 | |
| 1134 | TIM-16 | P2 | blocked | DEEP | 4 | contradicts TIM-05 premise (F-5) |
| 1191 | TIM-17 | P2 | blocked | DEEP | 4 | depends on GOV-07/08 and SUP-16 |

### MBX (epic #754, 15 stories)

| # | Key | Prio | Ready | Depth | State | Note |
|---|---|---|---|---|---|---|
| 789 | MBX-01 | P3 | blocked | DEEP | 4 | |
| 790 | MBX-03 | P3 | blocked | DEEP | 4 | negative timeout undecided |
| 791 | MBX-13 | P2 | **ready** | DEEP | 6 (ADR 0030 accepted = 1; Go tests present; JVM scenarios recorded in ADR 0030; not re-run by me) | F-10 |
| 829 | MBX-02 | P3 | blocked | DEEP* | 4 | ActorMailboxSpec not opened |
| 830 | MBX-04 | P3 | blocked | DEEP | 4 | |
| 831 | MBX-05 | P3 | blocked | DEEP | 4 | |
| 832 | MBX-06 | P3 | blocked | DEEP | 4 | RESCOPE verified |
| 833 | MBX-08 | P3 | blocked | DEEP | 4 | |
| 834 | MBX-11 | P3 | blocked | DEEP | 4 | |
| 864 | MBX-07 | P3 | blocked | DEEP | 4 | |
| 865 | MBX-09 | P3 | blocked | DEEP* | 4 | StablePriority queue class (Java/Scala util) not opened |
| 866 | MBX-10 | P3 | blocked | DEEP | 4 | no Pekko oracle for overflow (F-11) |
| 867 | MBX-12 | P3 | blocked | DEEP | 4 | AC3 vs ADR 0030 D3 (F-10b) |
| 931 | MBX-14 | P3 | blocked | DEEP | 4 | |
| 1098 | MBX-15 | P0 | blocked | DEEP | 4 | lane-ordering hazard (F-1) |

### DSP (epic #755, 5 stories)

| # | Key | Prio | Ready | Depth | State | Note |
|---|---|---|---|---|---|---|
| 792 | DSP-01 | P3 | blocked | DEEP | 4 | |
| 835 | DSP-02 | P3 | blocked | DEEP | 4 | |
| 868 | DSP-03 | P3 | blocked | DEEP | global part 6, per-executor 4 | default differs from Pekko (F-16) |
| 869 | DSP-05 | none | blocked | DEEP | 4 (EXCLUDE_PROPOSAL, state 2) | no prio label |
| 904 | DSP-04 | P3 | blocked | DEEP | 4 | |

Counts: 46 stories (BHV 9, TIM 17, MBX 15, DSP 5). DEEP 43, DEEP* 3 (#900, #829, #865), SHALLOW 0, NOT REVIEWED 0.

## 2. Capability matrix

| Capability | Pekko contract (path:line) | Go accepted/proposed contract | Tax. | Implementation in develop/pekko-actor | Evidence | Gap |
|---|---|---|---|---|---|---|
| become(discardOld=true) | `actor/ActorCell.scala:602-603` (`behavior :: (if discardOld && nonEmpty tail else stack)`); HotSwapSpec.scala:26-43 (become in constructor, last wins), :70-83 | #771; ADR 0010:225 excludes; ADR 0033 D4 individual; GOV-04 #1035 proposed | 4 | absent (`actor.go:19` `Actor[T]` is a struct with `Receive`; no grep hit for become) | Pekko oracle exists (HotSwapSpec) | AC covers only "after current delivery"; become in factory/PreStart not covered; invalid-replacement AC has no Pekko basis (Pekko does not validate: `ActorCell.scala:602-603`; the effect of a null behavior is not verified) |
| become(discardOld=false)/unbecome | `ActorCell.scala:602-615`; unbecome on single-element stack reinstalls `actor.receive` (:613); HotSwapSpec.scala:44-68 (self-unbecome inside the pushed behavior), :85-107 | #817 | 3 (AC3) / 4 | absent | HotSwapSpec | AC3 "restores" has no Pekko basis; Pekko resets on restart (`ActorCell.scala:718-721`, HotSwapSpec.scala:109-134) |
| unhandled / UnhandledMessage | `Actor.scala:297,656-663` (Terminated -> DeathPactException, else publish `UnhandledMessage(message, sender(), self)`) | #897; GOV-05 #1036 | 4 | DeathPact half adapted: `errors.go:54` `ErrDeathPact`, `actor.go:40` `TerminatedHandler`; no publish | oracle partial: LoggingReceiveSpec.scala:77-125 asserts message+recipient only (`_` for sender); LocalActorRefProviderSpec.scala:69 | no Go definition of "unhandled" from `Receive(...) error`; sender assertion has no oracle |
| Stash (classic) | `Stash.scala:145` private Vector[Envelope]; :149-151 capacity from `Mailboxes.stashCapacity` (`Mailboxes.scala:295-305`); :160-175 requires deque mailbox else ActorInitializationException; :182-190 stash(); :245-252 unstashAll | #772 (decision), #855, #898, #899; ADR 0033 D4 stash mandatory | 4 (scope 1) | absent; `internal/mailbox/queue.go:20-86` has Push/Pop/TakePending/Len/Cap only; `mailbox.go:20-47` | ActorWithStashSpec.scala:121-186; ActorWithBoundedStashSpec.scala:27-176 | see F-1..F-4 |
| stash double-stash rule | `Stash.scala:183-185` throws IllegalStateException only if `currMsg eq theStash.last` (reference equality with the LAST element only); spec ActorWithStashSpec.scala:141-147 | #855 AC1 "appended once", reconciliation "classified error" | 4 | absent | spec exists | F-3 |
| unstashAll order | reverse iteration + `enqueueFirst` (`Stash.scala:245-250`); `Mailbox.scala:594-598` addFirst; bounded `:604-640` offerFirst with pushTimeOut else DeadLetter(handle.message, handle.sender, receiver) | #898 AC | 4 | absent | no spec pins order vs already queued messages (reconciliation says so; I confirmed: ActorWithStashSpec "support protocols" asserts only completion) | oracle must be written |
| stash restart/stop | `Stash.scala:89-92` preRestart unstashAll, :100-102 postStop unstashAll; spec ActorWithStashSpec.scala:149 is the RESTART case; stop->dead letters only in ActorWithBoundedStashSpec.scala:126-133 | #935 (restart), #1025 (stop) | 4 | stop path: `stop.go:107-114` dead-letters `takePendingLocked()` messages with `DeadLetter{Msg, To}` (no sender) | Go test of existing stop path exists; stash path none | F-4, F-12, F-17 |
| Typed StashBuffer relation | `actor-typed/.../internal/StashBufferImpl.scala:156-252`: unstash interprets messages inline (no mailbox, no enqueueFirst); rest goes to dead letters only when the actor stops DURING unstash (:246-252) | BHV-09 "Produced: ... consumed by the Typed StashBuffer"; MBX-15 "consumed by stash replay" | 3 | n/a | Pekko source | F-4 |
| Scheduler once/delay/rate/Cancellable | `Scheduler.scala:64-82,154-168,197-228,298-368,428-436,505-534`; close `LightArrayRevolverScheduler.scala:170-190`, reject after shutdown :206-215; interval<=0 IllegalArgumentException (SchedulerSpec.scala:701-772); throw aborts repetition (SchedulerSpec.scala:269-307) | #776 ADR to write; #821-#823 | 4 | absent (no scheduler; `clock.go:8-26` unexported, used by Ask `ask.go:58,149,166`) | SchedulerSpec (TimingTest-tagged) | F-13, F-14 |
| Keyed actor timers | `dungeon/TimerSchedulerImpl.scala:25-165` (generation filter :138-161, single removed :149, isTimerActive = `timers.contains` :114); `Timers.scala:41-48` cancelAll on preRestart/postStop; `Timers.scala:50-66` unwrap + AutoReceivedMessage path | #860, #900, #937, #938, #1130, #1134, #1191 | 4 (scope 1) | absent | TimerSpec.scala:200-318, TimersAndStashSpec :319-352 | F-5 |
| ReceiveTimeout | `dungeon/ReceiveTimeout.scala:28-82`; `ActorCell.scala:547-565` (cancel before, re-arm after, for BOTH user and auto-received messages), `:654` checkReceiveTimeout after preStart; `FaultHandling.scala:289` after postRestart; `Actor.scala:164` marker | #861, #901, #1133; GOV-06 #1037 | 4 | absent | ReceiveTimeoutSpec.scala:82-320 | F-15 |
| Injectable clock | n/a (Pekko uses `ExplicitlyTriggeredScheduler` in tests; judge uses it, `judge/pekko-harness/Judge.scala:8,199,250`) | ADR 0015 D9/I-1 (unexported); #1129 export | 6 (internal) / 4 (export) | `clock.go:8-26`; `system.go:102,153-154` field + default realClock; injection only via unexported `options` | Ask tests | F-9 |
| Bounded mailbox, zero timeout | `Mailbox.scala:557-569`; `Mailboxes.scala:230-235`; `DeathWatch.scala:110` Terminated is a `self.tell` (counts); MailboxConfigSpec.scala:56-90,142-192 | ADR 0030 D1-D8 accepted | 6 | `ref.go:54`, `mailbox.go:27`, `port.go:72`, `spawn.go:41-57`; 9 `bounded_mailbox_*_test.go` | Go tests; ADR 0030 records JVM runs and `0030-pekko-repro/` (not CI) | no judge-harness differential found for it |
| Positive/negative push timeout | `Mailbox.scala:557-569` (>=0 offer with timeout, <0 `queue.put` blocks forever) | ADR 0030 D2/D7 exclude; #790, #830 | 4 | absent | oracle: MailboxConfigSpec.scala:56-75, :89-91 | negative case undecided |
| NonBlockingBounded | `Mailbox.scala:421-433,690-700`, `AbstractBoundedNodeQueue.java:39-41` (capacity 0 allowed) | #831 | 4 | absent | oracle MailboxConfigSpec.scala:274,287 | |
| Deque / head insert | `Mailbox.scala:576-640,837-868` | #1098, #832, #864 | 4 | absent | | F-1, F-2 |
| Priority / stable priority | `Mailbox.scala:732-770,780-826` | #833, #865, #866, #931 | 4 | absent | PriorityDispatcherSpec.scala:58-100 (bounded variant uses capacity 1000, pushTimeOut 10s: no overflow case), StablePriorityDispatcherSpec.scala:60-104 | F-11 |
| ControlAware | `Mailbox.scala:873-904,944-1010` | #834, #867 | 4 | absent | ControlAwareDispatcherSpec.scala:37-60 | F-10b |
| Dispatcher throughput | `Mailbox.scala:263-280` (`max(throughput,1)`; counts user messages only; system messages processed between via `processAllSystemMessages`; deadline checked after each message); `AbstractDispatcher.scala:333`; `reference.conf:591-594` default throughput 5, deadline 0 | #868, #904 | global 6 / per-executor 4 | `system.go:75-77,136` `max(cfg.Throughput,1)`; `cell.go:116-168` loop `for range throughput` counts start, start-failure, sys, failed and runtime steps as steps | `behavior_test.go:46`, `cell_internal_test.go:116-162` | F-16 |
| Executor / dispatcher selection | `Dispatcher.scala:42-48,125-128`, `Dispatchers.scala:69,112` | #792, #835, #869; GOV-10 #1040 | 4 | `executor.go:17-22` unexported, no per-actor choice | DispatcherShutdownSpec.scala:30-46, ForkJoinPoolStarvationSpec.scala:64 | |

## 3. Findings

Severity scale: blocker, high, medium, low.

**F-1 (high). Head insert (MBX-15/BHV-06) can break the existing lane ordering that implements ADR 0030 D8.**
Evidence (Go, code analysis, not executed): `cell.go:57-61` documents `pushed` and `lane`; `port.go:77` records `after: c.pushed`; `watch.go:421-426` `laneReadyLocked` delivers a lane item (Terminated, WatchWith, Port post) when `after <= received`. A head-inserted message is popped before older queued messages; if `pushed` is incremented and `after` of existing lane items is not shifted, a pending Terminated is delivered one message early per inserted message, contradicting ADR 0030 D8 ("Terminated at the tail", the `m3 m4 m5 T` example). In Pekko the same case is trivial because Terminated is a normal envelope (`DeathWatch.scala:110`) and `enqueueFirst` puts stashed ones ahead of it.
Missing from #1098 AC (R1-R5 cover order and bounded overflow only) and #898. Correction: add an AC to #1098: "Given a pending Terminated/WatchWith/Port item in the lane and a head insert, the lane item is still delivered after every message that was queued before it". Add a Go mutation test. Also state in #1098 how `pushed`, `maxDepth` and `received` counters treat a head insert.

**F-2 (medium). Two issues claim the head-insert primitive; ownership is unresolved and the dependency graph encodes both.**
#772 and #898 reconciliation: "#832 duplicate or sibling of #1098, to be reconciled by the Owner". #832 body AC1-AC2 are head-insert/deque-order ACs while the title says selectable discipline; #832 reconciliation = RESCOPE. #898 body still lists `#832` in Dependencies while its reconciled table lists `#1098` as `depends_on` and `#832` as `independent`. #1098 itself lists only GOV-03 (#1021) as prerequisite and no GOV-11 (#1041), although #772 depends on GOV-11. Correction: rewrite #898 body dependency to #1098 (not #832); apply the RESCOPE of #832 before anyone starts it; add to #1098 a note that the primitive is internal (no export) unless Owner decides otherwise.

**F-3 (medium). BHV-05 AC1/AC2 mis-describe the Pekko rule.**
`Stash.scala:183-185` throws IllegalStateException only when `currMsg eq theStash.last` (identity with the most recent stashed element). Spec: ActorWithStashSpec.scala:48-59 (StashingTwiceActor) and :141-147. The story body says "appended once"; reconciliation says "double-stash throws". Neither states the Go failure channel (a Go `Receive` returns `error`, Pekko throws). Also not addressed: stash during a WatchWith/Port/lane delivery (Pekko explicitly rewrites `currentMessage` so stashing a custom WatchWith message works, `DeathWatch.scala:86-87`; Timers does the same at `Timers.scala:56-60`), and Pekko's `ActorWithStashSpec.scala:186` "re-receive unstashed Terminated". In Go, Terminated is a hook (`actor.go:40`), not a `Receive` message, so that spec has no direct Go counterpart. Correction: AC1 = "stashing the same delivery twice in a row returns/raises <typed error>"; AC3 must define whether a WatchWith message and a Port post are valid stash targets; Pekko: a stashed `Terminated` is re-received (ActorWithStashSpec.scala:83-98,186; Stash.scala:271-277; spec and code). goPekko: Terminated is a hook (`actor.go:40`), not a message; this is a difference with no Owner decision under ADR 0033 D13, pending.

**F-4 (high). BHV-09 and MBX-15 claim the Typed StashBuffer consumes head insert and stop-time dead-lettering; Pekko Typed does neither.**
`actor-typed/.../StashBufferImpl.scala:156-252`: `unstash` interprets stashed messages inline through the behavior interpreter; there is no `enqueueFirst`. The "rest to dead letters" path (:246-252) runs only when the behavior stops while unstashing, with sender = dead letters ref. Nothing dead-letters a non-empty typed stash at actor stop (grep `unstashRestToDeadLetters|clearStash` in actor-typed/src/main: only StashBufferImpl). So "Stop-time stash disposition contract consumed by the Typed StashBuffer" (#1025 Contracts) and "Head-insert ... consumed by stash replay" (#1098) are Classic-only; the Typed contract needs its own stories. Correction: reword #1025 and #1098 "Produced/consumers" to Classic stash only, and list Typed StashBuffer as a separate (TYP) contract with the inline-replay semantics; this also affects the J-02a plan row.

**F-5 (medium). TIM-16 open choice contradicts TIM-05/TIM-06 premises about the timer wrapper.**
#1134 says "Go Tell delivers T directly, so the problem [wrapper vs payload] may not exist". But #860 AC2 and #900 AC2 require that an already-queued message of an old generation, or of a pre-restart incarnation, is discarded at delivery (Pekko `TimerSchedulerImpl.scala:138-161`, TimerSpec.scala:224-237, :260-300). That requires either a tagged item in the mailbox or a lane item, because `cell.queue` is `Queue[T]` (`queue.go:29`) and `Receive` gets a plain T. The existing lane (`cell.go:61`, `watch.go`) is the natural place, which in turn ties TIM-05 to F-1 (positions in the queue). Pekko: stale timer messages are discarded by generation and owner (TimerSchedulerImpl.scala:136-161). Open question: how a Go timer delivery is represented so an already queued stale one is discarded; evidence missing: Owner/ADR decision. No recommendation.

**F-6 (medium). BHV-03 has no Go definition of "unhandled".**
Go `Receive(ctx, msg) error` (`actor.go:19`) has no "partial function not defined" result. Pekko publishes UnhandledMessage when the `Receive` partial function is not defined at the message (`Actor.scala:656-663`). AC "An unhandled ordinary message produces one event" cannot be executed without a signal (sentinel error? separate interface?). Also the UnhandledMessage sender component has no oracle (LoggingReceiveSpec.scala:125 uses `_`). Open question: what in a Go `Receive(...) error` counts as "not defined"; evidence missing: Owner decision (GOV-05). Whether Pekko has an applicable oracle for the event sender, and its classification under ADR 0033 D13, are pending determination.

**F-7 (medium). BHV-02 AC3 contradicts Pekko; BHV-01 AC3 has no Pekko basis.**
Pekko resets the behavior stack on restart (`ActorCell.scala:718-721`, `:625`, `:632`; HotSwapSpec.scala:109-134 "revert to initial state on restart"). AC3 "resets or restores" must drop "restores" or justify it by ADR (GOV-49 #1196 proposal exists). BHV-01 AC3 "invalid replacement ... without corrupting the stack": Pekko does not validate; the failure path is Go-only (e.g. nil behavior). Also uncovered upstream cases in HotSwapSpec: become in constructor (:26-43, last wins), stacking + self-unbecome inside the pushed behavior (:44-68). In Go the equivalents are factory and PreStart. Correction: add ACs for those, the classification of AC3 of #771 under ADR 0033 D13 is pending determination (Pekko does not validate the replacement; whether an oracle applies is open).

**F-8 (medium). TIM-01 / TIM-11 dependency direction is self-contradictory.**
#776 reconciliation: `#1129 -> #776 depends_on` ("TIM-01 specifies the clock seam that TIM-11 exports; record the seam decision first or state that TIM-01 delivers it"). #776 AC3 itself says "Specify clock/executor seams". #1129's own table lists only GOV-03 and its Design leaves the exported form as an Owner decision with no ADR assigned. Result: a P0 export story (#1129) with an open Owner decision and no ADR owner, and a P0 ADR story (#776) that is declared to depend on it. Plan row: `docs/actor-typed-delivery-plan.md:272-273` C-01 (clock) before C-02 (TIM-01). Reader proposal, not an Owner decision; no Pekko basis. Correction: make #776 deliver (and own) the seam decision, direction `#776 -> #1129` (spec before export), or move the seam decision to a dedicated ADR story that both depend on; update plan C-01/C-02 accordingly.

**F-9 (medium). TIM-11 state and evidence.**
Verified: `clock.go:8-26` is unexported with `Now`/`AfterFunc`, `realClock` default (`system.go:153-154`), only injection path is the unexported `options` struct (`system.go:26`; `newSystem(cfg, options{...})`), used only by Ask (`ask.go:58,149,166`). The ACT-014 seam is therefore implemented for Ask only (PR #382, #160 closed). AC-R1-2 "System created with a manual clock" requires a public creation path that does not exist; AC-R3-1 requires a public Timer contract. These are the open choices listed in the story. D13: the Go seam has no direct Pekko counterpart; whether an applicable oracle exists, and how the story's evidence is classified, are pending determination; the story acknowledges the tension. Pekko-side manual time exists (ExplicitlyTriggeredScheduler, used by `Judge.scala:8,199,250`), so timer/scheduler stories (TIM-02..TIM-13) DO have an applicable oracle and need a differential scenario (D13), not only "ported intent" Go tests. Correction: state per story whether evidence is V3 differential (judge extension B-02 in plan) or "ported spec"; no TIM story mentions the judge extension.

**F-10 (medium). MBX-13 (#791) audit statement is wrong.**
Second-pass text: "Pekko's capacity does not count 'admitted lane items'; D3/D8 (Terminated placement) are goPekko decisions, not Pekko semantics". `DeathWatch.scala:110` shows `self.tell(Terminated(...), actor)` through the normal mailbox, so a bounded Pekko mailbox does count Terminated; ADR 0030 D3 and D8 reproduce that. Correction: edit the audit note in #791; keep the `status:ready` label (genuinely independent, ADR 0030 accepted, tests present in `bounded_mailbox_*_test.go`) but note its ACs ask for evidence "not in the repository" (process-only, not a Pekko behavior).
**F-10b (medium). MBX-12 AC3 vs ADR 0030 D3.** "Internal runtime messages remain admissible independently of user occupancy" conflicts with D3, which counts Terminated deliveries and Port posts against capacity (`mailbox.go:27`, `port.go:72`). Only Stop/Restart/Suspend/Resume/child-failure items are outside. Correction: reword to "system messages (stop, restart, suspend, resume, failure) stay outside; lane items follow ADR 0030 D3".

**F-11 (medium). Priority/bounded-priority/control mailboxes: oracles cover ordering only.**
PriorityDispatcherSpec.scala:58-100 uses a bounded variant with capacity 1000 and pushTimeOut 10s (no overflow). No Pekko spec for: overflow with a priority queue, "processing message does not consume a slot" (#866 AC2), "overflow does not reorder" (#866 AC3), stable-sequence allocation on rejection (#931 AC2), lane sharing of bounded control-aware (#867). No dedicated Pekko spec was found for these ACs; whether an applicable oracle exists under ADR 0033 D13 is pending determination. Reconciliation of #865/#931 cites StablePriorityDispatcherSpec for all; only the ordering AC is covered (StablePriorityDispatcherSpec.scala:60-104). Also priority comparators in Pekko receive system-generated items (Terminated is a tell); the Go comparator is over T only, a Go adaptation that no story records.

**F-12 (medium). BHV-09 R3 oracle uses a positive push timeout; Go ADR 0030 has timeout zero.**
ActorWithBoundedStashSpec.scala:27-94: `Bounded10 extends BoundedDequeBasedMailbox(10, 500.millis)`. Pekko's `enqueueFirst` during `unstashAll` therefore waits up to 500ms on the actor's own thread when full (`Mailbox.scala:604-640`), a self-wait that only ends by timeout; the observable dead letter is `hello1` (the OLDEST stashed message, because replay inserts in reverse order, ActorWithBoundedStashSpec.scala:119-133). Whether the dead-lettered message is the same under timeout 0 (ADR 0030 D2, zero timeout, accepted) is not verified; the spec observes timeout 500 ms only; ADR 0030 D7 forbids a blocking variant. #1025 R3 and #1098 R3 cite this spec as V1 ported evidence without recording the timeout difference. Correction: record "Pekko oracle uses 500ms push timeout; Go uses 0; outcome compared, timing not" as an observable difference.

**F-13 (medium). TIM-01 omissions from `Cancellable`/close semantics.**
Not covered by any story: `Cancellable.isCancelled` semantics (`Scheduler.scala:505-540`: false after execution; cancel returns false when already cancelled; SchedulerSpec.scala:127-155), `TaskRunOnClose` run-on-close (`Scheduler.scala:562-567`, SchedulerSpec.scala:684-698; the story lists it only as an Owner open choice in #1131), maximum delay rejection (`LightArrayRevolverScheduler.scala:230`, SchedulerSpec.scala:477,743-760), tick-granularity round-up (:93). Open question: classification of each (ported, adapted, excluded with a reason from the closed list); evidence missing: Owner decision per capability. The specs at `SchedulerSpec.scala:565-660` test the wheel implementation.

**F-14 (low). TIM-02 omits delay <= 0.**
`LightArrayRevolverScheduler.scala:206-209`: a non-positive delay runs the task immediately on the executor and returns `NotCancellable` (cancel is false); after close it throws (:207). Code evidence only; not proven by tests. Add AC to #821.

**F-15 (low). TIM-09 omits that ReceiveTimeout wraps auto-received messages and the post-preStart arm.**
`ActorCell.scala:547-565` calls `cancelReceiveTimeoutIfNeeded`/`checkReceiveTimeoutIfNeeded` around BOTH `autoReceiveMessage` and `receiveMessage`; `:654` arms after `aroundPreStart`. ReceiveTimeout itself influences (not marked NotInfluence), so it re-arms and repeats. Also the first branch of `checkReceiveTimeout` (`ReceiveTimeout.scala:44-58`) arms the task even for NotInfluence messages when none is scheduled yet (spec ReceiveTimeoutSpec.scala:215-300). Add ACs for repeat, preStart arm and the "set during NotInfluence handler" cases.

**F-16 (medium). DSP-03: Go default and step semantics differ from Pekko and are not recorded as adaptations.**
Pekko default `throughput = 5` (`reference.conf:591`), counts user messages only, drains system messages between each (`Mailbox.scala:263-280`). Go default is 1 (`system.go:136` `max(Throughput,1)`; zero value = 1), and counts start, start-failure, system, failed-child and runtime steps as steps (`cell.go:116-168`). `Dispatcher.scala:38-40` scaladoc says "zero or negative -> always continues until empty" but code floors at 1 (`Mailbox.scala:264`); the Go doc matches the code. #868 AC1 asks to document step accounting (good) but not the default (5 vs 1) nor the stale scaladoc. No recommendation. The difference between Pekko (evidence above) and current goPekko (`system.go:136`, `cell.go:116-168`) is recorded; its classification needs an Owner decision (ADR 0033 D13).

**F-17 (low). Citation drift.**
(a) #935 reconciliation cites `ActorWithStashSpec:149` for stop->dead letters; line 149 is "process stashed messages after restart"; the stop case is only in ActorWithBoundedStashSpec.scala:126-133 and no unbounded-stash stop spec exists (searched ActorWithStashSpec.scala fully). (b) Plan rows D-06 still merge restart and stop (`actor-typed-delivery-plan.md:293`) while the issue was split into #935/#1025. (c) #935 says it `enables` #1025, but #1025's own dependency table lists only #898. (d) Plan C-09 cites `Actor.scala:158`; marker is `Actor.scala:164` (reconciliation already notes; plan not corrected). (e) Verified correct: TIM-17 cite `TimerSpec.scala:308-314`, TIM-10 cite `ReceiveTimeoutSpec.scala:191`, TIM-04 cites `SchedulerSpec.scala:252,269,701-719`.

**F-18 (medium). Priority/readiness inversions.**
P0 consumers of P1 prerequisites: #855, #898, #899, #935, #1025 (P0) depend on #772 (P1, decision slice) and, via #772, on GOV-11 #1041 (P1) and #1098 (P0). #1129 and #1098 (P0) depend on GOV-03 #1021 (P1). TIM-17 P2 depends on GOV-07/08 (P1) fine. Because every P0 BHV story is blocked behind a P1 decision, P0 labels do not drive order. Reader proposal, not an Owner decision; no Pekko basis. Correction: raise #772, #1021 to P0 or lower the consumers; make the plan's Slice A explicit.

**F-19 (low). DSP-05 has no priority label** (`labels=['status:blocked']`), by design (EXCLUDE_PROPOSAL, under GOV-10). Its reasoning (goroutine-per-activation, `executor.go:19-22`) is verified: one blocked Receive blocks only its own goroutine. Note that Go's default executor is unbounded goroutines, so Pekko's ForkJoinPoolStarvationSpec scenario (starvation of a bounded pool) does not exist for the default executor.

**F-20 (low). MBX-01 `enables` direction.** #1098 `enables` #789 ("queue port must include head insert") while #789 is blocked by GOV-11 and #1098 is not; if the port ADR is accepted after the primitive lands, the primitive may need rework. Reader proposal, not an Owner decision; no Pekko basis. Correction: #1098 should state it is an internal primitive designed to fit the port, or depend on a draft of #789.

**F-21 (low). Missing dependency: TIM-05 on lane ordering.** F-5 + F-1: TIM-05 and TIM-06 delivery representation depends on #1098 only if a lane/queue-position representation is chosen. Not currently declared.

## 4. Owner decisions needed

| # | Decision | Alternatives and consequences | Pekko evidence / recommendation |
|---|---|---|---|
| O-A | Who owns head insert (#1098 vs #832) | (a) #1098 internal primitive, #832 later selectable discipline (reconciliation proposal); (b) merge into one story. (a) keeps P0 work small; (b) removes a P3 duplicate | No recommendation. The difference between Pekko and current goPekko is recorded; its classification needs an Owner decision (ADR 0033 D13). |
| O-B | Timer delivery representation (wrapper vs lane item) | wrapper type changes `Queue[T]`; lane item reuses existing mechanism but must follow F-1 | No recommendation. Pekko: stale timer messages are discarded by generation and owner (TimerSchedulerImpl.scala:136-161). Evidence missing: Owner/ADR decision. |
| O-C | Go "unhandled" signal | sentinel error; optional interface; separate return | No recommendation. Pekko: a message the `Receive` partial function does not define goes to `unhandled` (`Actor.scala:656-663`). Evidence missing: Owner decision (GOV-05). |
| O-D | become/unbecome API shape and restart behavior (GOV-04, GOV-49) | restore vs reset on restart | Pekko observed: a restart reverts to the initial behavior (`HotSwapSpec.scala:109`, spec and code); no recommendation |
| O-E | ReceiveTimeout, PoisonPill, Kill classification (GOV-06/07/08) | each required/excluded | not needed to start TIM-01..08 |
| O-F | Clock/scheduler export form (Config field vs option; Clock/Timer names; manual clock in actor or testkit) | per #1129 open choices | No recommendation. The difference between Pekko and current goPekko is recorded; its classification needs an Owner decision (ADR 0033 D13). |
| O-G | Push-timeout variants (negative, positive) | blocking variants need new ADR (ADR 0030 D7) | No recommendation. Pekko has both non-negative and negative push timeouts (`Mailbox.scala:560-568`; `MailboxConfigSpec.scala:82,86,90`). Classification needs an Owner decision (ADR 0033 D13). |
| O-H | Throughput default (5 vs 1) and step accounting | align default to Pekko 5; keep 1; count user messages only | No recommendation. The difference between Pekko (`reference.conf:591`, `Mailbox.scala:263-280`) and current goPekko (`system.go:136`, `cell.go:116-168`) is recorded; its classification needs an Owner decision (ADR 0033 D13). |
| O-I | Dispatcher/mailbox config classification (GOV-10, GOV-11) | include per capability or exclude | blocks MBX-01..14 and DSP |

## 5. ADRs needing creation or amendment

| ADR | Action | Why |
|---|---|---|
| ADR 0010 (status: proposed, partly accepted) | amendment line :225 (exclusions) per capability after GOV decisions | become, stash, timers, receive timeout, dispatcher/mailbox config |
| ADR 0030 (accepted D1-D8) | amendment for stash replay into a bounded mailbox, head-insert overflow and lane positions (F-1, F-12); not accepted by anyone yet | stash x bounded |
| New: scheduler/timers/clock ADR (TIM-01) | create (proposed) | Cancellable, close, TaskRunOnClose, max delay, clock export (F-8, F-13) |
| New: stash ADR (BHV-04) | create | ownership, capacity, double-stash, WatchWith/Port stash, restart/stop (F-3, F-4) |
| New: Typed StashBuffer contract | create under TYP, separate from Classic | F-4 |
| New: mailbox queue port ADR (MBX-01) + named selection (MBX-02) | create after GOV-11 | |
| New: executor ownership ADR (DSP-01) | create after GOV-10 | |
| ADR 0012 (accepted for the bounded case) | impact only | stop before first activation + stash at stop: `stop.go:107-114` |
| ADR 0033 | no change; D4/D13 applied | |
| ADR 0032 (PR #1014, not in develop) | referenced by #1025, #855, #831, #1098 for sender assertions (D3/D4/D6) | those assertions are not startable until it is accepted/merged |

## 6. Blocked vs startable

Every story is `status:blocked` except #791 (MBX-13).
- Startable now: #791 (no prerequisites; evidence-closure; ADR 0030 accepted; F-10 correction first). Decision/ADR drafting (not implementation) can begin for #776 (TIM-01), #772 (BHV-04), #789 (MBX-01), #792 (DSP-01) because they only produce proposals; acceptance is Owner work.
- Potentially startable once one prerequisite lands: #1098 (needs GOV-03 #1021 form; plus F-1 AC); #1129 (GOV-03 + export-form decision, see F-8).
- Chain TIM: GOV-02 #1020 + #1129 -> #776 -> #821 -> {#822, #823, #860, #861} -> #900 -> {#937, #938, #1130, #1133}; #901 after #861 and #860; #1131 after #776/#1129; #1132 after #822/#823; #1134 after #860 and #898; #1191 after #860, #770, #816, #1112, GOV-07/08.
- Chain BHV: GOV-11 #1041 -> #772; #772 + #814 -> #855 -> {#898 (also #1098), #899} -> #935 -> #1025 (also ADR 0032). #771 after GOV-04 #1035 and GOV-02; #817 after #771 and GOV-49; #897 after GOV-05, #814 and #870 (or #871).
- MBX: all after GOV-11; #832 after #1098 and #789; #864 after #832 and #772; #833/#834/#830/#831 after #789; #865 after #833; #866/#931/#867 after their siblings and a decision on exposing pushTimeOut.
- DSP: all after GOV-10 #1040; #835 also after #792 and #767; #868 after #835; #904 after #868; #869 pending exclusion.

## 7. No spec found, limits of search, items not read

- No Pekko spec found for: relative order of unstashed vs already queued messages (searched ActorWithStashSpec.scala, ActorWithBoundedStashSpec.scala, StashSpec in actor-typed-tests not opened); empty `unstashAll` no-op; stash capacity <= 0 meaning unbounded (`Stash.scala:186`); `unhandled` event sender; bounded/stable priority overflow; BoundedControlAware lane sharing; `Cancellable.isCancelled` after periodic cancel not read in detail; DSP: no spec for `throughput` count or `throughputDeadlineTime` (searched actor-tests/dispatch directory names only; DispatcherShutdownSpec.scala:30-46 and ForkJoinPoolStarvationSpec.scala:64 opened); no spec for Kill by timer (TIM-17 itself says so, SupervisorMiscSpec.scala:46-85 is the analogue).
- Read by name/line only, bodies not opened: `ActorMailboxSpec.scala` (cited for #829, #789), `PropsAdapter.scala:43-45`, `MailboxSelectorSpec.scala:75-95`, `Mailboxes.scala:230-267` (only 295-305 opened), `Mailbox.scala:1000-1010` end of enqueueWithTimeout, `actor-typed-tests` StashSpec/TimerSpec (typed), `FSM.scala` timers, `ActorCell.scala:110-137` thread-safety doc of become, `Mailbox.scala:827-868` bodies (class headers only).
- Not verified: sequencing claims with SUP-11..14 (#782-#784, #826) in #900; ADR 0014 lock-order statements for DSP; ADR 0017 restart interplay; ADR 0012 beyond the dead-letter-at-stop path read in `stop.go`.
- Go implementation checks were by reading code (grep and file reads of snapshot), no tests were run.
