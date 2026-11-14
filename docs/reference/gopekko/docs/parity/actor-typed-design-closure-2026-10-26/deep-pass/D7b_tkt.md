# D7b Typed testkit stories (TKT-06..13, 16..20): deep second pass

> Verification status (2026-10-26): checked by V6_D7b_D8.md: 62 findings checked in full, 7 partially, 1 not verified. Corrections from the verification are applied in this file. Stories not covered by the verification keep the producer's wording unverified.

Paths: Pekko root `upstream/pekko` (v1.7.1); `TK` = `actor-testkit-typed/src/main/scala/org/apache/pekko/actor/testkit/typed`; `SPEC` = `actor-testkit-typed/src/test/scala/org/apache/pekko/actor/testkit/typed/scaladsl/BehaviorTestKitSpec.scala` (all 678 lines read). ADR 0033 read in full (D3 line 37, D10 line 44, D11 line 45, D13 line 47).

## Common findings (apply to every story below; not repeated)

- **C1 (medium, D13 wording).** The shared D13 sentence in the bodies ("V3 needs a Pekko differential scenario when an oracle exists; otherwise alternative evidence plus explicit Owner approval, recorded `pending approval`") is conditional and compatible with D13. Three ACs use `V3 pending approval` for behavior whose Pekko source exists and has no spec: #1150 AC-R2-1, #1153 AC-R2-1, #1154 AC-R3-1. Replace each with: "Oracle applicability: pending determination; Pekko source `<path:line>` exists; spec: none found (`<search>`)". A missing spec does not show that no oracle exists. Neutral replacement: "V3 requires a differential scenario against Pekko whenever an applicable oracle exists (ADR 0033 D13). Oracle applicability: pending determination; Pekko source `<path:line>` and spec `<path:line or none found>` exist." What counts as an applicable oracle for testkit assertion APIs is an open Owner question (first audit jdg-cls-tkt F1/OD-1/OD-2; those were reader proposals, not Owner decisions).
- **C2 (low).** Boilerplate "ADR 0032 D3/D4/D6 (accepted): sender-bearing assertions use an explicit sender (MSG-02 TellFrom)". ADR 0032 is not in develop (open PR #1014). No story here has a sender-bearing assertion: the TestInbox ref is a `FunctionRef` whose send function ignores the sender (`TK/internal/TestInboxImpl.scala:38`, `StubbedActorContext.scala:39-46`). Neutral text: "ADR 0032 (PR #1014, not integrated) is not exercised by this slice." Do not keep "(accepted)" unqualified.
- **C3 (info).** Label `status:blocked` is consistent with unfinished prerequisites for all 13 stories (no pekko-actor-typed in go.work; #1096 TYP-34 and #847 TYP-02 undelivered).
- **C4 (medium, systemic).** Bodies cite `BehaviorTestKitSpec` line numbers as "ports" of behavior that the spec lines do not assert. Per story below, I state what the spec actually asserts.

---

### #1140 TKT-06 — Capture sent messages with TestInbox
- Depth: DEEP (TestInbox.scala, TestInboxImpl.scala, BehaviorTestKitSpec, SyncTestingExampleSpec:141-166, BehaviorSpec use sites read)
- Labels now: prio:P1, status:blocked
- Verdict: CORRECT
- AC table:

| AC | Pekko contract (path:line) | Spec body that observes it | goPekko state | Match? |
|---|---|---|---|---|
| AC-R1-1 m1,m2 then receiveMessage m1,m2 and hasMessages false | `TestInboxImpl.scala:36` ConcurrentLinkedQueue FIFO, `:38` ref adds, `:41-44`, `:63` | No dedicated TestInbox spec. Indirect: `SPEC:344-346` (`replyTo.receiveMessage()`, `selfInbox().hasMessages should be(true)`), `SPEC:409` (`h.receiveAll() shouldBe Seq("hello")`); docs `SyncTestingExampleSpec.scala:144-146` (`inbox.expectMessage("hello")`). None sends two messages and checks order. Searched: `grep -rn TestInbox` under actor-testkit-typed/src/test and actor-typed-tests (BehaviorSpec:118-131 `receiveAll() should ===(ReceivedSignal(signal) :: Nil)` is single/ordered pair `expected ++ expected` at :131) | none | spec-does-not-observe (order only weakly via BehaviorSpec:131) |
| AC-R2-1 empty receiveMessage fails with 'polling on an empty inbox' | `TestInboxImpl.scala:42` `NoSuchElementException(s"polling on an empty inbox: $path")` | no spec found; searched `polling on an empty` and `NoSuchElement` in actor-testkit-typed/src/test, actor-typed-tests | none | no spec found |
| AC-R3-1 expectMessage equality / `expected: other but received m` / `no messages were received` | `TestInboxImpl.scala:46-51` uses Scala `assert` (message texts `expected message: X but no messages were received`, `expected: X but received Y`); returns `this` | no spec asserts a failure text; success path only (`SyncTestingExampleSpec:146,155,166,232,244`, SPEC:439 via receptionist inbox) | none | success path only |
| AC-R4-1 receiveAll 3 msgs in order, second returns empty | `TestInboxImpl.scala:54-61` (`acc.reverse`) | `SPEC:409` single element; `BehaviorSpec:118,124,131,262` | none | partial (order of 3 and second-empty not observed) |

- Findings:
  - F1 (medium). R2 text "MUST fail naming the inbox path" is correct, but the exception type in Pekko is `NoSuchElementException` while expectMessage failures are `AssertionError` (via `assert`). The ACs do not distinguish the two failure kinds. Replacement: "receiveMessage on an empty inbox throws NoSuchElementException 'polling on an empty inbox: <path>' (TestInboxImpl.scala:42); expectMessage failures are assertion failures with the texts at :48-49; the Go failure surface is the open TKT-01 choice." Add: expectMessage/expectEffect failure texts are prefixed "assertion failed: " (Scala `assert`); expectEffectPF/expectEffectClass throw `AssertionError` without prefix (BehaviorTestKitImpl.scala:130-143).
  - F2 (medium). Evidence line "BehaviorTestKitSpec uses it throughout" does not support AC-R1-1..R4-1: the spec never asserts order, the empty failure, or the expectMessage failure texts (see table). The texts in AC-R2/R3 are source-derived with no spec; state "no spec found".
  - F3 (low). Not covered though in Pekko source: `TestInbox(name)` appends a random uid to the path (`scaladsl/TestInbox.scala:30-32`, `withUid(rnd)`), `expectMessage` consumes the head even when it fails (`poll` at :47), the ref rejects null messages (`StubbedActorContext.scala:45`, `InvalidMessageException`). Scope says default name "inbox": true (`TestInbox.scala:30`). Open question: whether these belong in the AC set (Owner/Architect); missing evidence: none in specs.
  - F4 (info, dependency). Contract "function-backed ref" depends on #847 TYP-02 (stated). ADR 0011 export question is blocking only for the export (OK).
- Dependencies: #847 -> #1140 depends_on: OK (typed ref). Produces for #1141, #1142, #1145; but also consumed by #1150 (stated in its body) and #1151/#1152 via #1142: edges transitive, fine. No cycle.
- Oracle (D13): applicability pending determination. Pekko oracle candidates: TestInboxImpl.scala:36-67 and docs example `SyncTestingExampleSpec:141-166`; no dedicated TestInbox spec. Missing: Owner reading of "applicable oracle" for assertion APIs.

---

### #1141 TKT-07 — Capture replies with ReplyInbox and StatusReplyInbox
- Depth: PARTIAL (the producer marked DEEP; the verifier found that its "no spec found" search stayed inside the testkit and typed-test trees and missed the cluster-sharding-typed doc tests that exercise `runAskWithStatus`, `StatusReplyInbox` and `expectDone`. Read: TestInbox.scala:80-170, TestInboxImpl.scala:85-184, all uses of runAsk in SPEC)
- Labels now: prio:P2, status:blocked
- Verdict: CORRECT
- AC table:

| AC | Pekko contract | Spec body | goPekko | Match? |
|---|---|---|---|---|
| AC-R1-1 receiveReply returns reply; hasReply true before, false after | `TestInboxImpl.scala:89-96` receiveReply sets `underlying = None`; `:117-121` hasReply | `SPEC:401` `runAsk[ActorRef[String]](...).receiveReply()` (returns the reply only); `SPEC:448,461,469` `runAsk(IsTimerActive).expectReply(false/true)`. No assertion on hasReply | none | partial (hasReply not observed) |
| AC-R2-1 expectNoReply passes with none, fails with reply | `:105-115` | no spec found; searched `expectNoReply`, `hasReply` in actor-testkit-typed/src/test, actor-typed-tests, docs | none | no spec found |
| AC-R3-1 receiveValue/receiveError and wrong kind fails | `:141-151` | Success and error paths observed in cluster-sharding-typed doc tests (`AccountExamplePersistenceProbeDocSpec.scala:37,52,56,66`; `AccountExamplePersistenceProbeDocTest.java:45-47,119-121` with `expectErrorMessage("not enough funds to withdraw 110")`); wrong-kind failure and `expectErrorMessage` mismatch: no spec found | none | partial (success/error paths observed outside actor-testkit-typed) |
| AC-R4-1 expectErrorMessage('bad') pass / 'other' fail | `:160-165` compares `getMessage` | pass observed in `AccountExamplePersistenceProbeDocTest.java:119-121` (`expectErrorMessage("not enough funds to withdraw 110")`); mismatch failure: no spec found | none | partial |

- Findings:
  - F1 (high). Incomplete contract: after any `receive*`/`expect*` the inbox is spent. Subsequent `receiveReply`, `expectReply`, `expectNoReply` throw `AssertionError("Reply was already received")` / `"Improper expectation of no reply: reply was already received"` (`TestInboxImpl.scala:95,114,138,176`); `hasReply` becomes false (`:120`). Doc comments say the same (`scaladsl/TestInbox.scala:93,99`). Atomic outcome lists receiveReply/expectReply/expectNoReply/hasReply but no R/AC covers the spent-inbox failure. TKT-16 AC-R1-1 relies on it ("a second receiveReply fails"). Replacement: add requirement text "After the reply is received, further receive/expect/expectNoReply calls fail; hasReply is false (TestInboxImpl.scala:89-96,117-121)". No new design.
  - F2 (medium). R2 "expectNoReply MUST fail if a reply exists": Pekko also consumes the reply to build the message `Expected no reply, but <reply> was received` (`:108`) and returns `this` on success. Not stated.
  - F3 (medium). `expectDone` is listed in the Atomic outcome and Scope ("expectDone") but has no R or AC. Pekko: `expectDone()(implicit ev: T =:= Done)` = `expectValue(Done)` (`scaladsl/TestInbox.scala:162`). Body Open choice "whether expectDone needs a Done type" is correct as open. Observed in `AccountExamplePersistenceProbeDocSpec.scala:37,52,56` (success path only, cluster-sharding-typed docs spec).
  - F4 (low). Traceability cites "BehaviorTestKitSpec.scala:401,411,448 (expectReply use)": 401 uses `receiveReply`, 411 `expectReply(Done)`, 448 `expectReply(false)`; accurate but they observe only the success path.
  - F5 (info). `expectReply` fail text `Expected X but received Y`; `expectValue` same; `expectErrorMessage` text `Expected a throwable with message X, but got Y` (`:164`); AC says "fails" without texts: acceptable.
- Dependencies: #1140 -> #1141 depends_on OK (ReplyInbox wraps TestInboxImpl, `:85`); #774 ASK-07 -> #1141 depends_on OK for StatusReply (`TestInboxImpl.scala:25,143`). ReplyInbox (non-status) needs no ASK-07: the ASK-07 dependency covers only the StatusReply half. Atomicity: SPLIT candidate (ReplyInbox vs StatusReplyInbox have different prerequisites: #774 needed only for the latter); flag, not an Owner-label matter.
- Oracle (D13): applicability pending determination. Missing: no Pekko spec exercises StatusReplyInbox or expectNoReply; Pekko source exists.

---

### #1142 TKT-08 — Drive a Behavior synchronously with BehaviorTestKit core operations
- Depth: DEEP (BehaviorTestKit.scala, BehaviorTestKitImpl.scala fully, Behavior.canonicalize, SPEC 209-260,358,366, BehaviorSpec 259-275)
- Labels now: prio:P1, status:blocked
- Verdict: CORRECT (+ SPLIT flag already in body)
- AC table:

| AC | Pekko contract | Spec body | goPekko | Match? |
|---|---|---|---|---|
| AC-R1-1 behavior counts messages, run twice, second reflects both | `BehaviorTestKitImpl.scala:170-185` run | cited `SPEC:358-363` is a different scenario: `run(SpawnAdapterWithName("adapter"))`, asserts `currentBehavior should not be Behaviors.same` and `returnedBehavior shouldBe Behaviors.same`. No counting behavior in any spec | none | spec-does-not-observe (cited spec observes R2) |
| AC-R2-1 stopped => isAlive false; unhandled => returned=unhandled, current=previous | `:150-152`; `Behavior.scala:184-187` canonicalize returns `current` for Same and Unhandled; `:179` | `SPEC:361-362` only (same). Stop: `BehaviorSpec.scala:268-273` `testkit.run(Stop)` then `Behavior.isAlive(testkit.currentBehavior) should be(false)`. Unhandled: `BehaviorSpec:259-262` uses `Behavior.interpretMessage(testKit.currentBehavior, testKit.context, Miss)` directly, not `run`. No spec asserts `returnedBehavior` is unhandled | none | stopped: yes via BehaviorSpec (not cited); unhandled via run: spec-does-not-observe |
| AC-R3-1 empty selfInbox => runOne fails; self-sent => processed | `:187` `run(selfInbox().receiveMessage())` (empty => NoSuchElementException 'polling on an empty inbox') | success path `SPEC:347,486,510,521` (runOne after adapter/timer send). Failure: no spec found; searched `runOne` in all test trees | none | success yes; failure no spec |
| AC-R4-1 signal delivered; port :366 | `:189-198` signal | `SPEC:366-373` asserts only `noException should be thrownBy testkit.signal(Terminated(other.ref))`. It does not observe that the behavior received the signal (Parent logs `debug("Terminated")` at SPEC:173 but nothing reads it) | none | spec-does-not-observe delivery |
| AC-R5-1 NoEffects when empty, hasEffects false; expectEffect different fails | `:88-91` retrieveEffect, `:119-124` expectEffect, `:200` hasEffects | `SPEC:216-231` (NoEffects by type, expectEffect(NoEffects), hasEffects false/true); `SPEC:233-242` PF no match asserts `ae.getMessage should startWith("expected matching effect but got: ")`; `:244-259` PF match. expectEffect with a different value failing: no spec found | none | yes except "different value fails" (no spec) |

- Findings:
  - F1 (high). AC-R1-1 cites a spec that tests canonicalization (R2), not counting. Current text: "port of BehaviorTestKitSpec 'run behaviors with messages without canonicalization' (…:358)". Replacement: "Pekko spec SPEC:358-363 asserts that after run the currentBehavior is not Behaviors.same while returnedBehavior is Behaviors.same; a counting-behavior scenario has no Pekko spec (Go-side evidence)".
  - F2 (high). AC-R4-1 claims the spec shows the behavior "receives" the signal; SPEC:366-373 only asserts no exception while `context.log` is used. Replacement: "SPEC:366-373 asserts signal(Terminated(ref)) does not throw; delivery to the behavior is not asserted by that spec".
  - F3 (high, missing contract). Failure semantics (first audit F16 now verified from source): `run` and `signal` catch NonFatal, deliver `PostStop` to the current behavior via `Behavior.interpretSignal` + canonicalize (exceptions from that are swallowed), then rethrow the original exception (`BehaviorTestKitImpl.scala:154-161,184,194`). `run` also calls `runAllTasks` after the step (`:183`); `signal` does not (`:189-198`). Initial behavior is started and validated at construction (`:55-62`, `Behavior.validateAsInitial`). No spec asserts any of this (searched `thrownBy`, `intercept` in SPEC: only IllegalStateException of AskInitiated and AssertionError of PF). Open question: whether these belong to TKT-08; missing evidence: spec. Docs say "Supervision is not supported" (`docs/src/main/paradox/typed/testing-sync.md`, limitations list).
  - F4 (medium). `run` passes every message through `BehaviorTestKitImpl.Interceptor` for adapter messages (`:175,212-246`); TKT-20 depends on this inside `run`, but TKT-08's Dependencies and Scope do not mention it. Ownership of the interceptor is unassigned between TKT-08 and TKT-20.
  - F5 (medium). expectEffect on failure still consumes the head effect (`poll`, `:120-122`); R5 does not state it. Fail texts: `expected: X but no effects were recorded` / `expected: X but found Y`; `expectEffectType` texts `:130,132`.
  - F6 (medium, traceability). Scope item "Effect interface, NoEffects" and the Context claim that "NoEffects is owned here": consistent with Effect.scala:307-312. OK. The body of #1144 context also says plan M-02e listed it; no conflict.
  - F7 (low). Missing prerequisite for AC-R4-1 and AC-R5-1 ports: SPEC:366-373 uses `TestInbox` (#1140, listed); PF ports use `Spawned` effects (`SPEC:248`) and `SpawnAnonymous` (`SPEC:212`), which are owned by #1143 (TKT-09), a consumer of TKT-08. Ports at :209,:233,:244 therefore need #1143 constructors/types (or a Go-side stand-in). Dependency direction question: #1143 -> #1142 depends_on is correct for effect infrastructure, but the cited AC evidence in #1142 itself needs spawn effects. Open question for Architect: whether those ports belong to #1142 or to a later story; missing evidence: none.
- Dependencies: #1140 -> #1142 depends_on OK; #847 -> #1142 depends_on OK (redundant/transitive via #1140, body says so). No cycle.
- Oracle (D13): applicability pending determination. Pekko source `BehaviorTestKitImpl.scala:150-198` and SPEC exist; judge cannot compare assertion APIs is a reader proposal only (jdg-cls-tkt OD-2).

---

### #1143 TKT-09 — Observe spawn and stop effects of a Behavior
- Depth: DEEP
- Labels now: prio:P1, status:blocked
- Verdict: CORRECT
- AC table:

| AC | Pekko contract | Spec body | goPekko | Match? |
|---|---|---|---|---|
| AC-R1-1 spawn 'child' no props => expectEffect(spawned(b,'child')) | `EffectfulActorContext.scala:128-132` (records `Spawned(behavior,name,props,ref)` after `super.spawn`); `Effect.scala:119-121` apply default `Props.empty` | `SPEC:284-289`: `run(SpawnChildren(2))`, `retrieveAllEffects()` `should contain.only(Spawned(Child.initial,"child0"), Spawned(Child.initial,"child1",Props.empty))`. Two children, names child0/child1; no spec uses name 'child' with expectEffect(spawned) for a single no-props spawn except `SPEC:488` PF `Spawned(_, "child", _)` | none | yes (2 children, not the single-'child' scenario in AC) |
| AC-R1-2 props carried; ports :291,:300,:307 | `Effect.scala:97-110,128-139` | `SPEC:291-296` Spawned with `props = Props.empty.withDispatcherFromConfig("cat")`; `:300-305` SpawnedAnonymous no props `shouldBe Seq(...)`; `:307-313` SpawnedAnonymous with props | none | yes |
| AC-R2-1 stop => expectEffect(stopped(name)) | `EffectfulActorContext.scala:133-136` records `Stopped(child.path.name)` BEFORE `super.stop` | `SPEC:416-427`: `run(StopChild(child.ref)); expectEffect(Stopped(child.childName))`, then respawn `SpawnChild` and `child.childName shouldBe newChild.childName`; `SPEC:413` `expectEffectType[Stopped]` after KillSession | none | yes |
| AC-R3-1 equal when behavior,name,props equal regardless of ref | `Effect.scala:102-108` (`behavior == o.behavior && childName == && props ==`), anonymous `:133-136` | implicit in `SPEC:288,295,304,312`: expected effects built with `ref = null` (`Effect.scala:121,148`) equal recorded effects with real refs | none | yes (implicitly) |

- Findings:
  - F1 (medium). Context says "Spawned/SpawnedAnonymous equality on behavior, name and props only" (accurate). `behavior == o.behavior` is JVM `equals` on `Behavior` (an abstract class without equals override: Behavior.scala:49), i.e. identity for the specs' `Child.initial` val. The open choice "behavior equality in Go" is a genuine open question; evidence for the Pekko side is identity of the same `val`.
  - F2 (medium, missing contract). Stub stop semantics unmentioned: stop of a non-direct child throws `IllegalArgumentException("Only direct children of an actor can be stopped through the actor context, …")` after the `Stopped` effect has been queued (`EffectfulActorContext.scala:134-135`, `StubbedActorContext.scala:133-143`); stopping removes the child so its name can be reused (`:141`; `SPEC:416-427`); spawning a taken name throws `InvalidActorNameException("actor name X is already taken")` (`:121`). No spec for the two failures. Open question: in or out of R2.
  - F3 (low). Out-of-scope list omits `SpawnedAdapter`/`SpawnedAnonymousAdapter` (`private[pekko]`, `Effect.scala:152-215`); #1154 records the exclusion. Cross-reference only.
  - F4 (low). Anonymous child name is `Helpers.base64(counter)` (`StubbedActorContext.scala:92,114`); a Go test must not depend on it unless Owner-approved.
- Dependencies: #1142 -> #1143 depends_on OK. Missing: effect infrastructure needs childInbox/stub context children which #1145 owns for the *inspection* only; no cycle.
- Oracle (D13): applicability pending determination (SPEC:284-313,416-427 exist).

---

### #1144 TKT-10 — Observe watch, timeout, schedule and ask effects of a Behavior
- Depth: DEEP
- Labels now: prio:P2, status:blocked
- Verdict: CORRECT + DEP + SPLIT (atomicity: five independent outcomes; body already allows splitting AskInitiated)
- AC table:

| AC | Pekko contract | Spec body | goPekko | Match? |
|---|---|---|---|---|
| AC-R1-1 watch, watchWith, unwatch => three effects in order; ports :377,:385 | `EffectfulActorContext.scala:137-148`; `Effect.scala:236-246` | `SPEC:377-383`: `run(SpawnAndWatchUnwatch("hello"))`, `child = testkit.childInbox("hello").ref`, `retrieveAllEffects() should be(Seq(spawned(...), watched(child), unwatched(child)))` (spawn + watch + unwatch; three effects incl. the Spawned). `SPEC:385-392` spawn + `watchedWith(child, msg)`. watch/watchWith/unwatch are never in one run | none | partial (spec observes spawn+watch+unwatch, and spawn+watchWith) |
| AC-R2-1 setReceiveTimeout(d,m) => ReceiveTimeoutSet queued | `EffectfulActorContext.scala:149-156`; `Effect.scala:251-261` | no spec found; searched `ReceiveTimeoutSet`, `ReceiveTimeoutCancelled`, `setReceiveTimeout` in actor-testkit-typed/src/test, actor-typed-tests/src/test (0 hits) | none | no spec found |
| AC-R3-1 schedule => Scheduled queued, target inbox empty | `EffectfulActorContext.scala:157-160`; `StubbedActorContext.scala:160-164` returns Cancellable with `cancel()=false`, `isCancelled=true`, sends nothing | no spec found; searched `Effects.scheduled`, `Scheduled(`, `scheduleOnce` | none | no spec found |
| AC-R4-1 AskInitiated respondWith twice: first delivers, second fails with upstream text; ports :556,:584,:611,:644 | `Effect.scala:55-92`; `EffectfulActorContext.scala:81-101` | `SPEC:556-582`: respondWith, then `selfInbox().hasMessages shouldBe false`, `logEntries.size shouldBe 1`, message `"Got N cookies from distributor"` (delivery observed through the log, synchronously, not via selfInbox). `SPEC:584-609` `effect.timeout()`: log starts with `"Failed to get cookies: Ask timed out on ["`. `SPEC:611-642` second `respondWith`/`timeout` => `an[IllegalStateException] shouldBe thrownBy` (type only, NOT the text); `SPEC:644-676` after timeout, respondWith/timeout throw ISE, `logEntries() shouldBe empty` after clear, `hasEffects() shouldBe false` | none | yes except text 'Can only complete the ask once' (spec-does-not-observe; text is `Effect.scala:83`) |

- Findings:
  - F1 (high). AC-R1-1 text "Given watch, watchWith and unwatch calls, then three effects in order" does not match SPEC:377-392 (spec runs spawn+watch+unwatch in one case and spawn+watchWith in another). Replacement: "SPEC:377-392 asserts the effects [Spawned, Watched, Unwatched] and [Spawned, WatchedWith(ref,message)]".
  - F2 (high, dependency). The ports at :377/:385 call `testkit.childInbox("hello").ref`, and the AskInitiated ports use `TestInbox`, `logEntries()`, `clearLog()` and `selfInbox()` (`SPEC:575-578,603-606,636-641`). #1144 depends only on #1143; the child-inbox accessor is owned by #1145 (TKT-11), the log accessors by #1152 (TKT-18, which itself waits on #1052 EVT-15), the inbox by #1140. Either the ACs must not require those ports, or `depends_on` edges #1145 -> #1144 and #1152 -> #1144 are missing. Evidence: SPEC:380,389,576-578.
  - F3 (high, missing AC). Scope and Atomic outcome promise "can show its adapted message" (adaptResponse, adaptTimeout) and Pekko's ask sends the request to the target immediately (`EffectfulActorContext.scala:87-89`, `target ! askMessage`; `respondWith` runs `AdaptMessage` through `behaviorTestKit.run`, `:95-97`, so the response is processed synchronously, never via selfInbox). R4 mentions none of: target receives the ask message; `adaptResponse`/`adaptTimeout(msg)`; `timeout()` text `Ask timed out on [target] after [N ms]. Message of type [..]. A typical reason…` (`Effect.scala:76-79`); `require` positive timeout (`EffectfulActorContext.scala:51,64-66`); effect equality over the first parameter list only (`Effect.scala:55-57`; SPEC:544). Spec that observes these: `SPEC:532-554` (`shouldEqual Effects.askInitiated(cdInbox.ref, 10.seconds, classOf[CookiesForYou])`, `cdInbox.receiveMessage() shouldBe effect.askMessage`, adaptResponse/adaptTimeout results) which no AC ports.
  - F4 (medium). R3 "Scheduled ... without sending": true (`StubbedActorContext.scala:160-164`), but the Cancellable returned has `cancel()=false`; not stated. No spec.
  - F5 (medium). ReceiveTimeoutCancelled is in the Atomic outcome but has no R/AC. TimerScheduled intentionally excluded (TKT-19).
  - F6 (low). Context mentions "Effects.scala:35,94-121 and Effect.scala:55-92" correct. AC-R2 evidence "upstream Effects.scala:109" is a factory, not a spec.
  - F7 (medium, ADR). The bullet "Ask uses context.Context deadlines in Go (ADR 0033 consequences)" already sits under the body's open choices for the Architect; only its parenthetical should read "(ADR 0033 line 72 records the adaptation of futures to context.Context deadlines)". ADR 0033 line 72 lists "futures replaced by `context.Context` deadlines" as an observable difference to record, not as a decided mechanism for this effect; Pekko's `AskInitiated.responseTimeout` is a FiniteDuration (`Effect.scala:56`). Replacement: "The Go shape of the ask timeout in AskInitiated is an open Architect choice (ADR 0033 line 72 records context deadlines as an adaptation of futures)".
- Dependencies: #1143 -> #1144 depends_on OK (effect queue). Missing: #1145 (childInbox used by ports), #1152 (log accessors used by ports), #1140 (TestInbox in ports; transitive). `#1144 independent #1149` (out of scope text) fine. Typed watch/ask context API is "consumed" without an issue key; note #926 TYP-07 (ask completion) and a watch story are not listed as dependencies although Contracts say "Typed watch/watchWith/ask context API".
- Oracle (D13): applicability pending determination (SPEC:377-392,532-676 exist; ReceiveTimeout/Scheduled have no spec).

---

### #1145 TKT-11 — Inspect children of a Behavior in BehaviorTestKit
- Depth: DEEP
- Labels now: prio:P2, status:blocked
- Verdict: CORRECT
- AC table:

| AC | Pekko contract | Spec body | goPekko | Match? |
|---|---|---|---|---|
| AC-R1-1 behavior spawns 'child' and sends m => childInbox('child').expectMessage(m); port :396 | `BehaviorTestKitImpl.scala:93-97`; `StubbedActorContext.scala:211` | `SPEC:396-414` does not call childInbox(name) and the behavior never sends to a child: it uses `runAsk(SpawnSession)`, `childTestKit(sessionRef)`, `session.run("hello")`, `h.receiveAll() shouldBe Seq("hello")`, `runAsk(KillSession).expectReply(Done)`, `expectEffectType[Stopped]`. The send-to-child case is observed by the docs example `SyncTestingExampleSpec.scala:151-156` (`childInbox[String]("child").expectMessage("hello")`) and `:162-167` (`childInbox(child.ref).expectMessage("hello stranger")`) | none | spec-does-not-observe at the cited line; docs example observes it |
| AC-R2-1 no child 'x' => childInbox('x') fails; BehaviorTestKitImpl.scala:93 | `:94-95` `assert(inbox.isDefined, s"Child not created: $name. Children created: [...]")` => AssertionError | no spec found; searched `Child not created`, `childInbox` in SPEC, actor-typed-tests (usage only: SupervisionSpec, BehaviorSpec not for failure) | none | no spec found |
| AC-R3-1 childTestKit(ref) runs child's behavior; port :416 | `StubbedActorContext.scala:202-206` (`_children(name)` NoSuchElementException if absent; IllegalArgumentException if ref differs) | `SPEC:407-409` (inside :396) `childTestKit(sessionRef).run("hello")` then `h.receiveAll() shouldBe Seq("hello")`. `SPEC:416-427` does not use childTestKit | none | yes at :407, not at :416 |

- Findings:
  - F1 (high). AC labels do not map to requirements: AC-R1-1 tests R3 (message appears in child inbox), AC-R2-1 tests the failure half of R1, AC-R3-1 tests childTestKit which is R2. Replacement: relabel as AC-R1-1 (name lookup + failure), AC-R2-1 (ref lookup, childTestKit), AC-R3-1 (message to child).
  - F2 (high). Citations swapped: childTestKit is exercised at SPEC:407-409 (spec 'allow retrieving and killing'), not at :416; the send-to-child assertion is in the docs example SyncTestingExampleSpec:151-166 (and Java BehaviorTestKitTest.java:384-397 for the ref). Replacement text states the two spec locations.
  - F3 (medium, missing contract). `childInbox(ref)` in BehaviorTestKitImpl is a lookup by `ref.path.name` only (`:99-100`), whereas `StubbedActorContext.childInbox(child)` (not used by the public method) checks identity and throws `IllegalArgumentException(s"$child is not a child of $this")` (`:192-196`); `childTestKit(ref)` uses the identity-checking path (`:202-206`). So R2 "childInbox(ref) MUST return the inbox of that child ref" is true only for the name; a respawned child with the same name is returned for an old ref. Open question: which behavior the Go port must follow (Pekko differential needed if oracle applies; missing evidence: spec).
  - F4 (medium, scope). Body says "childInbox(name), childInbox(ref) and childTestKit(ref) ... fail when no such child exists". Failure kinds differ: AssertionError (name), NoSuchElementException/IllegalArgumentException (testkit).
  - F5 (info). Spawned child behaviors are started immediately at spawn (`StubbedActorContext.scala:114,123` creates a `BehaviorTestKitImpl`, which starts the behavior), so a child's `setup` runs at spawn time; not in ACs.
  - F6 (medium, atomicity). Master M-02f was split; this story says the other three get "proposed new stories TKT-11b/c/d", but #1150/#1151/#1152 now exist (TKT-16/17/18). Replace "proposed new story TKT-11b/c/d" with "#1150, #1151, #1152". Body Risks "three outcomes of master row M-02f need new keys (report)" is stale.
- Dependencies: #1143 -> #1145 depends_on: questionable. childInbox lookup does not need Spawned effects; it needs spawn in the stub context (which the effect infrastructure of #1143 extends). `SyncTestingExampleSpec:162` uses `expectEffectType[SpawnedAnonymous]` to obtain the ref; ref can also come from `runAsk` (SPEC:401). Keep as is; state reason. #1052 -> #1145 `independent`: fine (but the edge is stale: log story is #1152). Missing: #1144's ports need #1145 (see #1144 F2) i.e. proposed #1145 -> #1144 depends_on.
- Oracle (D13): applicability pending determination (SPEC:396-414 and docs example exist; failure case has no spec).

---

### #1146 TKT-12 — Create, shut down and inspect an ActorTestKit
- Depth: DEEP (ActorTestKit.scala 55-100,155-225; TestKitUtils.scala; ActorTestKitSpec 33-190; reference.conf; scrubActorSystemName)
- Labels now: prio:P1, status:blocked
- Verdict: CORRECT
- AC table:

| AC | Pekko contract | Spec body | goPekko | Match? |
|---|---|---|---|---|
| AC-R1-1 name 'my test!' no invalid chars; valid name identical; ports ':65' ':73' | `TestKitUtils.scala:90-91` -> `testkit/.../TestKitUtils.scala:64-70` (drop package prefix `^.*\.`, drop `$..`, replace `[^a-zA-Z_0-9-]` with `_`, drop MultiJvmNode) | `ActorTestKitSpec.scala:65-71` input `"actor~!sys-tem&Name#1%(with*invalid^ch@racter$)\`"` expects `"actor__sys-tem_Name_1__with_invalid_ch_racter___"`; `:73-80` all valid chars identical; `:57-63` `ActorTestKit(classOf[Vector[_]].getName)` => `"Vector"` (package prefix dropped) | none | yes; note rule is more than "invalid chars replaced": the part before the last `.` is dropped |
| AC-R2-1 existing System => kit's system is that System; port ':40' | `ActorTestKit.scala:77-85` creates a guardian `systemActorOf(testKitGuardian, "test"/"test-N")` under the given system | `ActorTestKitSpec.scala:40-48` asserts `testkit2.internalSystem === system` and `testkit2.system === system` | none | yes; the kit spawns an extra system actor (needs `systemActorOf`) |
| AC-R3-1 shutdownTestKit => whenTerminated done; port ':102' | `ActorTestKit.scala:220-225`; `TestKitUtils.scala:93-102` `system.terminate()` + `Await.ready(whenTerminated, timeout)` | `ActorTestKitSpec.scala:102-107` `shutdownTestKit(); system.whenTerminated.futureValue shouldBe a[Done]` | none | yes |
| AC-R4-1 System cannot terminate, short timeout => error naming timeout | `TestKitUtils.scala:96-100`: on TimeoutException message `Failed to stop [name] within [timeout]` + `printTree`; throws `RuntimeException` only if `throwIfShutdownTimesOut`, otherwise `println` | no spec found; searched `Failed to stop`, `throwIfShutdownFails`, `ThrowOnShutdownTimeout`, `throw-on-shutdown-timeout` in actor-testkit-typed/src/test | none | no spec found |

- Findings:
  - F1 (high). R4 says "shutdown MUST fail ... when throwIfShutdownFails is true": correct, but the AC and R4 omit the default: `throw-on-shutdown-timeout=true` in `actor-testkit-typed/src/main/resources/reference.conf:33` and `ActorTestKit.shutdown(system)` / `shutdownTestKit` read it from TestKitSettings (`ActorTestKit.scala:162-165,221-224`), while the static overload default is `throwIfShutdownFails = false` (`:171`). With false, Pekko prints and returns; no failure. The "error naming the timeout" text: message names system and the timeout `Failed to stop [%s] within [%s]` (`TestKitUtils.scala:98`). Replacement: add the false branch (prints, no error) and the settings-vs-static default difference.
  - F2 (medium). Port numbers drifted: body cites ':65', ':73', ':40', ':102' which are the right line starts (65, 73, 40, 102) in the file as read (line numbers confirmed by my read offset 33): correct. Spec lines 50-55 (`ActorTestKit()` derives the name from the call stack, `TestKitUtils.scala:82-83`) and 109-124 (application-test.conf) are listed in Context as an adaptation; also `createTestProbe` unique names (:126) belongs to TKT-01/TKT-14.
  - F3 (medium, missing). "settings and default timeout" are in the Atomic outcome ("exposes system, scheduler, settings and timeout") with no R/AC. Pekko `timeout = testKitSettings.DefaultTimeout` (`ActorTestKit.scala:216`, `default-timeout = 5s`, reference.conf:26) and `DefaultActorSystemShutdownTimeout`. Link to GOV-36 #1085 is `independent` ("fixed defaults until then"); first audit F9 asks whether it should be `depends_on`. Not an Owner label matter; flag as dependency inconsistency: TestProbe stories (#1135..) depend on #1085 differently.
  - F4 (medium, dependency). Kit creation from an existing System uses `system.systemActorOf` (`ActorTestKit.scala:83`); #1055 SYS-18 (system actors) and the ADR 0011 export are not listed in #1146 (only #1096, #994). Evidence: `ActorTestKit.scala:82-83`. Proposed edge: #1055 -> #1146 depends_on (or state it is covered by TYP-29).
  - F5 (low). Spec `:20-38` of ActorTestKitSpec (`ScalaTestWithActorTestKit` naming) is excluded by D10 (JUnit/ScalaTest wrappers): consistent.
  - F6 (info). `ApplicationTestConfig` (HOCON) has no Go meaning: ADR 0033 line 72 "typed Go options instead of HOCON" accepted adaptation. OK.
- Dependencies: #1096 -> #1146 OK; #994 -> #1146 OK; #1085 independent (see F3); missing #1055. #1146 -> #1147/#1148/#1149 OK.
- Oracle (D13): applicability pending determination (ActorTestKitSpec.scala:40-107 exist; R4 has no spec).

---

### #1147 TKT-13 — Spawn and stop actors through ActorTestKit
- Depth: DEEP
- Labels now: prio:P1, status:blocked
- Verdict: CORRECT
- AC table:

| AC | Pekko contract | Spec body | goPekko | Match? |
|---|---|---|---|---|
| AC-R1-1 behavior records start; spawn returns; behavior eventually ran; port 'spawn an actor' | `ActorTestKit.scala:231-257`: `Await.result(guardian.ask(SpawnActorAnonymous|SpawnActor), timeout.duration)`; guardian `TK/internal/TestKitUtils.scala:42-52` (`context.spawn`/`spawnAnonymous`) | `ActorTestKitSpec.scala:82-90` `spawn(Behaviors.setup{ sawMessage.trySuccess(true) ...})`; `sawMessage.future.futureValue should ===(true)` | none | yes |
| AC-R1-2 name 'name' | `:247-257` | `ActorTestKitSpec.scala:92-100` `context.self.path.name` from setup `=== "name"` | none | yes |
| AC-R2-1 stop returns after termination, watching yields Terminated; one ignoring stop => timeout text | `ActorTestKit.scala:264-272`; guardian `StopActor`: `context.watchWith(ref, ActorStopped(reply)); context.stop(ref)` (`TestKitUtils.scala:53-56`); `assert(false, s"timeout ($max) during stop() waiting for actor [${ref.path}] to stop")` | no spec found; searched `testKit.stop`, `\.stop(` over ActorTestKitSpec, TestProbeSpec, docs AsyncTestingExampleSpec (no ActorTestKit.stop assertion) | none | no spec found |

- Findings:
  - F1 (high). AC-R2-1 "one that ignores stop" is not producible via Pekko semantics: `context.stop(ref)` on a direct child of the guardian terminates it regardless of its behavior (a child cannot "ignore" a context stop). The scenario that yields the timeout is stopping an actor that is not a child of the guardian, or one whose termination takes longer than `max`; `ActorContextAdapter.scala:98-103` throws IllegalArgumentException when the ref is not a direct child. Which scenario yields the `timeout (...) during stop()` text (`ActorTestKit.scala:271`) is not shown by any spec: open question. Evidence is incomplete: I did not execute; open question for the Architect: which scenario produces the timeout in Pekko, missing evidence: a spec (none found).
  - F2 (medium). Spawn failure: `handleSpawnException` logs error and replies a stopped anonymous actor (`TestKitUtils.scala:62-70`), so `spawn` returns a ref instead of failing (e.g. duplicate name). Not in R/AC; no spec. Open question whether in scope.
  - F3 (medium). Body risk "upstream asserts through the guardian ask; Go may use direct supervision of the child" is a design proposal; Pekko's observable contract includes: both spawn and stop are asks bounded by `timeout` (default `DefaultTimeout` 5s) so spawn itself can time out with TimeoutException. R1 omits.
  - F4 (low). Out of scope names "Probe actors: #1135"; fine. Dependency table lists only #1146; the Typed guardian spawn needs `context.spawn`/watchWith/stop (TYP-36 #1170 spawn/stop child; TYP-?? watchWith) not listed. Proposed edge #1170 -> #1147 depends_on (evidence: guardian behavior uses `context.spawn`, `spawnAnonymous`, `watchWith`, `stop`, `TestKitUtils.scala:45-55`).
- Dependencies: #1146 -> #1147 OK; add #1170 (child spawn/stop) as above. ADR 0011 note OK.
- Oracle (D13): applicability pending determination (`ActorTestKitSpec.scala:82-100`; stop has no spec).

---

### #1150 TKT-16 — Run asks against a BehaviorTestKit with runAsk and runAskWithStatus
- Depth: PARTIAL (the producer marked DEEP; the verifier found that its "no spec found" search missed the cluster-sharding-typed doc tests that exercise `runAskWithStatus` and `expectDone`)
- Labels now: prio:P2, status:blocked
- Verdict: CORRECT
- AC table:

| AC | Pekko contract | Spec body | goPekko | Match? |
|---|---|---|---|---|
| AC-R1-1 reply Done to reply-to of KillSession; expectReply(Done) passes and second receiveReply fails; port :396-414 | `BehaviorTestKitImpl.scala:67-72` | `SPEC:401` (`runAsk[ActorRef[String]](SpawnSession(_, h.ref)).receiveReply()`), `SPEC:411` (`runAsk(KillSession(sessionRef, _)).expectReply(Done)`), `SPEC:448,461,469`. The second receiveReply failing: no spec found (searched `Reply was already received`) | none | partial (second call failure is source-only: TestInboxImpl.scala:95) |
| AC-R2-1 runAskWithStatus success/error | `:77-82` | runAskWithStatus observed in `AccountExamplePersistenceProbeDocSpec.scala:37,52,56,66` and `AccountExamplePersistenceProbeDocTest.java:45,70,91,119`; not in BehaviorTestKitSpec | none | partial (Pekko oracle: cluster-sharding-typed doc tests; V3 applicability: pending determination) |
| AC-R3-1 behavior spawns anonymous child during ask => expectEffectType[SpawnedAnonymous] | run records effects (`:70`, `EffectfulActorContext.scala:103-107`) | `SPEC:403` after `runAsk(SpawnSession)`: `testkit.expectEffectType[SpawnedAnonymous[_]]` then `s.ref shouldBe sessionRef`. The body cites `:401-409` | none | yes |
| AC-R4-1 two consecutive runAsk calls each own reply-to | `:68` new `TestInboxImpl[Res]("replyTo")` per call | no spec found (no spec calls runAsk twice expecting isolation; `SPEC:448-469` calls it three times but each time with a fresh behavior state and a reply only) | none | spec-does-not-observe |

- Findings:
  - F1 (low). Traceability of the second table row cites `SPEC:448-469 (IsTimerActive via runAsk)` as the verifier of "runAsk 67-72, runAskWithStatus 77-82": it verifies runAsk only. Replacement: "runAskWithStatus observed in `AccountExamplePersistenceProbeDocSpec.scala:37,52,56,66` and `AccountExamplePersistenceProbeDocTest.java:45,70,91,119`; not in BehaviorTestKitSpec".
  - F2 (medium). "TestInboxImpl named replyTo" (Scope): Pekko creates the inbox with `TestInboxImpl[Res]("replyTo")` (`:68`) whose path uses `address / name` WITHOUT uid (`TestInboxImpl.scala:73-76`, different from `TestInbox.apply` which adds a uid, `scaladsl/TestInbox.scala:30-32`). Not an AC; no observable consequence stated.
  - F3 (medium). R3 "Effects ... recorded exactly as for run": true since runAsk calls `run`; the failure semantics of F3 in #1142 (exception rethrown after PostStop) apply to runAsk too: nothing returned if the behavior throws (the ReplyInbox is not created).
  - F4 (info, dependency). The dependency rows marking #1144 and #926 `independent` are supported by `BehaviorTestKitImpl.scala:67-72` (no `context.ask`). The consumed edge to #1153 (TKT-19) is stated as "Produced ... used by #1153" but #1153's dependency table lacks it (see #1153).
- Dependencies: #1141 -> #1150 and #1142 -> #1150 depends_on OK. #1140 consumed transitively via #1141. No cycle (#1150 -> #1153).
- Oracle (D13): applicability pending determination (runAsk: SPEC:401,411,448-469; runAskWithStatus: cluster-sharding-typed doc tests, not BehaviorTestKitSpec).

---

### #1151 TKT-17 — Expose the receptionist inbox of a BehaviorTestKit
- Depth: DEEP
- Labels now: prio:P2, status:blocked
- Verdict: CORRECT
- AC table:

| AC | Pekko contract | Spec body | goPekko | Match? |
|---|---|---|---|---|
| AC-R1-1 fresh kit => receptionistInbox().hasMessages false; port :434-436 | `ActorSystemStub.scala:91,93`; `BehaviorTestKitImpl.scala:208` | `SPEC:434-436` `receptionistInbox().hasMessages should equal(false)` before run | none | yes |
| AC-R2-1 behavior registers ServiceKey 'aladin' with own ref => expectMessage(Register(key, ref)) passes, hasMessages false afterwards | stub `receptionist` returns `receptionistInbox.ref` (`ActorSystemStub.scala:93`) | `SPEC:434-441`: `run(RegisterWithReceptionist("aladin"))`, `hasMessages should equal(true)`, `expectMessage(Receptionist.Register(ServiceKey[Command]("aladin"), testkit.ref))`, `hasMessages should equal(false)` | none | yes (requires `ServiceKey` and `Register` value equality; `Register` is `final case class` with `replyTo: Option` None for the 2-arg apply: `ReceptionistMessages.scala:34`, `Receptionist.scala:127-139`) |
| AC-R3-1 RegisterWithReceptionist, when run, no failure; port :430-433 | n/a | `SPEC:430-433` no assertion except that `run` does not throw | none | yes (no-throw only) |

- Findings:
  - F1 (medium). AC-R2-1 depends on `ServiceKey` equality and `Register` structural equality (key + ref + replyTo). Both are carried by #960 TSD-02 (Register and ServiceKey). Whether `ServiceKey[Command]("aladin")` created in the test equals the one in the behavior is a ServiceKey contract question (id and type) outside this story: open, missing evidence: ServiceKey source (not in `actor-typed/receptionist/ServiceKey.scala`, whose path I could not open; the class is defined elsewhere; not verified).
  - F2 (low). Context "stub system owns a TestInbox at path parent/receptionist" accurate (`ActorSystemStub.scala:91`: `path.parent / "receptionist"`).
  - F3 (medium, scope). The stub system also implements `deadLetters`/`ignoreRef` (a `DebugRef`), `address` ("pekko", "StubbedActorContext"; `SPEC:277-280`), and throws UnsupportedOperationException for scheduler, provider, system actors, extensions (`ActorSystemStub.scala:52-116`). `SPEC:277-280` `return default address` has no story. Report as an unowned stub-system observable (not new design).
  - F4 (info). "A functioning receptionist in ActorTestKit: not in D10" in Out of scope: D10 (line 44) lists the testkit subset; whether a real receptionist in ActorTestKit is excluded is not stated by D10; ADR 0033 D9 (line 43) includes the local Receptionist in Typed services. Replacement: "A real Receptionist runs in an ActorSystem under #960; this story concerns the stub inbox only".
- Dependencies: #1142 -> #1151 OK; #960 -> #1151 depends_on OK for the command/key types; #1145 `independent` supported by `ActorSystemStub.scala:91`. No cycle.
- Oracle (D13): applicability pending determination (SPEC:429-441 exist).

---

### #1152 TKT-18 — Capture log entries of a BehaviorTestKit with the logger fake
- Depth: DEEP
- Labels now: prio:P2, status:blocked
- Verdict: CORRECT (+ ADR)
- AC table:

| AC | Pekko contract | Spec body | goPekko | Match? |
|---|---|---|---|---|
| AC-R1-1 behavior logs 'Hello!' info => one entry level info with message; port :261-266 | `StubbedActorContext.scala:93-94,221-224,240-257`; `CapturedLogEvent.scala:31` | `SPEC:261-266` `logEntries() shouldBe Seq(CapturedLogEvent(Level.INFO, what))` (equality over level, message, cause=None, marker=None) | none (EVT-15 fake not delivered) | yes |
| AC-R2-1 clearLog => empty; port :268-275 | `StubbedActorContext.scala:262-263`; `BehaviorTestKitImpl.scala:206` | `SPEC:268-275` | none | yes |
| AC-R3-1 snapshot: later logging does not change returned list | `StubbedActorContext.scala:240-257` builds a new `List` (`.toList`) on each call | no spec found; searched `logEntries` in SPEC (used only at :265,:272-274,:576-578,:603-606,:636-638,:660-663,:673) | none | no spec found (source-derived) |

- Findings:
  - F1 (high, ADR). ADR 0033 D11 (line 45): "Logging context uses kit-logger `With` per actor ... The test fake must capture `With` fields, honor `SetLevel` and guard `Entries` against concurrent access". The Pekko contract here is SLF4J `SubstituteLoggerFactory` event queue with message formatting through `MessageFormatter.arrayFormat` of `evt.getMessage` and arguments (`StubbedActorContext.scala:93,250`), plus cause and first marker. The kit-logger interface has no marker (body admits). How level filtering is applied in Pekko's substitute logger is determined by SLF4J, which is not in the pinned checkout: not verified. The deviation (no marker, no message-argument formatting?) needs a recorded decision; D11 does not decide it. Open question; missing evidence: kit-logger API surface vs `CapturedLogEvent` fields (level, message, cause, marker).
  - F2 (medium). Scope bullet "Captured event shape: level, formatted message, cause, marker" is correct per `CapturedLogEvent.scala:31`, but the ACs assert only level+message. Marker/cause have no AC and no spec (searched `CapturedLogEvent(` in tests: SPEC:265,272 only the 2-arg form; `BehaviorTestKitTest.java:299-316` also not). Java `getAllLogEntries` irrelevant (excluded).
  - F3 (medium). The ask ports `SPEC:576-581,603-608` use logEntries to observe ask delivery, so #1144's ports depend on this story (see #1144 F2). Also `Terminated` signal handler logs at debug (SPEC:173): the logged debug entry would appear in `logEntries` if the substitute logger records all levels; unobserved.
  - F4 (low). R3 "snapshot" with `go test -race` is Go-side only; Pekko returns an immutable `List` copy; labelled "Go-side scenario" in AC: acceptable but state "Pekko: no spec".
  - F5 (medium). The body says EVT-15 `status:ready, not yet delivered`; consistent with #1052 label `status:ready` (checked). TKT-18 is blocked by #1142 as well. OK.
  - F6 (medium). Blocking dependency "Typed context logger from the context stories" has no issue key: Contracts "Consumed" lists it, dependency table does not. A `depends_on` edge cannot be stated; missing evidence: owning issue (search of issue titles for "context logger" not done by me; flag).
- Dependencies: #1142 -> #1152 OK; #1052 -> #1152 OK; #1145 independent supported (`StubbedActorContext.scala:93`). Missing/unkeyed: context logger story.
- Oracle (D13): applicability pending determination (SPEC:261-275,576-676 exist). ADR D11 constrains the Go fake; the Pekko oracle is SLF4J based.

---

### #1153 TKT-19 — Observe timer effects of a Behavior in BehaviorTestKit
- Depth: DEEP
- Labels now: prio:P2, status:blocked
- Verdict: CORRECT + DEP
- AC table:

| AC | Pekko contract | Spec body | goPekko | Match? |
|---|---|---|---|---|
| AC-R1-1 ScheduleCommand('abc',42s,Single,SpawnChild) => key abc, delay 42s, SingleMode, overriding=false; port :445-459 | `EffectfulActorContext.scala:180-181,206-210`; `Effect.scala:271-279` | `SPEC:450-459` PF on `TimerScheduled("abc", SpawnChild, finiteDuration, SingleMode, false)` and `finiteDuration should equal(42.seconds)` | none | yes |
| AC-R1-2 fixed-rate start => FixedRateModeWithInitialDelay with initial delay = delay; :495-507 | Parent calls `startTimerAtFixedRate(key, cmd, delay, delay)` (SPEC:143); stub `:174-175` => `startTimer(key,msg,interval,FixedRateModeWithInitialDelay(initialDelay))` | `SPEC:498-508` pattern `FixedRateModeWithInitialDelay(\`delay\`)`, `finiteDuration should equal(delay)` | none | yes |
| AC-R2-1 key active, start again => overriding=true | `EffectfulActorContext.scala:207` `activeTimers.keySet(key)` evaluated before insert | no spec found for true (only `false` at SPEC:457,481,505); body says so | none | no spec found |
| AC-R3-1 cancel active => TimerCancelled queued, isTimerActive false; unknown key => nothing queued; ports :460-469,:527 | `:185-189` `if (activeTimers.keySet(key))` | `SPEC:463-469` cancel then `runAsk(IsTimerActive).expectReply(false)`; `SPEC:464-467` `TimerCancelled(key)`; `SPEC:526-527` `expectEffect(TimerCancelled("abc"))`. Unknown key: no spec found | none | active yes; unknown-key no spec |
| AC-R4-1 single timer send twice => one message; fixed rate send twice => two; ports :472-529 | `:193-204` | `SPEC:472-493`: `send()`, `runOne()`, Spawned child, second `send()`, `selfInbox().hasMessages should be(false)`. `SPEC:495-528`: send/runOne twice with a stop in between | none | yes |

- Findings:
  - F1 (high, observed Pekko code, no spec). `cancelAll()` is `activeTimers.foreach(cancel)` (`EffectfulActorContext.scala:191`). `foreach` passes the `(key, effect)` tuple to `cancel(key: Any)`; the tuple is not a key of `activeTimers`, so by source reading no `TimerCancelled` is queued and no timer is deactivated. Not verified by execution (no JVM run); no spec calls `cancelAll` (searched `cancelAll` in actor-testkit-typed/src/test and actor-typed-tests/src/test: 0 hits). Atomic outcome/scope do not mention `cancelAll`. Open question for the Architect/Owner: is `cancelAll` in scope and, if so, which behavior (Pekko as written vs intended) the Go kit follows. A deliberate difference from Pekko needs an explicit Owner decision (D13 does not authorize it). Missing evidence: execution of the Pekko source.
  - F2 (medium). R4 "put its message in the self inbox": `sendAction(key)` reads the message from the CURRENT `activeTimers(key)` at call time (`:193-203`), not from the effect that owns the `send`. After an override, calling the old effect's `send` delivers the new timer's message; after cancel it does nothing; after a single timer fired, it does nothing (observed by SPEC:491-492). No spec for override.
  - F3 (medium). Modes: Pekko maps `startTimerWithFixedDelay(k,m,d)` -> `FixedDelayMode`, 4-arg -> `FixedDelayModeWithInitialDelay(initial)` with delay=period, `startTimerAtFixedRate` 3-arg -> `FixedRateMode`, `startPeriodicTimer` -> `FixedRateMode` (`:165-181`). The Scope lists five modes but the only modes observed by a spec are SingleMode (SPEC:456,481) and FixedRateModeWithInitialDelay (SPEC:504); the Parent used in the spec maps its `FixedDelayMode` and `FixedRateMode` cases to the 4-arg calls (SPEC:140,143), so `FixedDelayMode`, plain `FixedRateMode` and `FixedDelayModeWithInitialDelay` are never asserted (Java `BehaviorTestKitTest.java:427-438` only constructs one; compile test). Replacement: "Modes FixedDelay*, plain FixedRate: source only (EffectfulActorContext.scala:165-181), no spec found".
  - F4 (medium, dependency). The ports of AC-R1-1 and AC-R3-1 use `runAsk(IsTimerActive(...))` (SPEC:448,461,469), i.e. #1150 (TKT-16) and #1141 (ReplyInbox); Contracts lists #1150 as "consumed" for the isTimerActive check, but the Dependencies table has only #1143 and #977. Add `#1150 -> #1153 depends_on` (reason: isTimerActive observed only through runAsk in the cited spec) or state isTimerActive is checked via another route. #1150 itself lists "#1153 (IsTimerActive checks)" as consumer; no cycle.
  - F5 (low). Context "cancel at 184" in Traceability is `EffectfulActorContext.scala:185`; "mkTimer at 162, sendAction 193, startTimer 206" correct. `Effect.scala:281-288` TimerMode is 281-289.
  - F6 (info, correction of plan). Context CORRECTION of plan M-02/K-03 confirmed against `StubbedActorContext.scala:160-164` and `docs testing-sync.md` ("Usage of scheduler is not supported"): the plan statement is wrong for timers. The correction text is accurate.
- Dependencies: #1143 -> #1153 OK; #977 -> #1153 OK; add #1150 -> #1153 depends_on (F4). The Out-of-scope link to #1144 (Scheduled) OK.
- Oracle (D13): applicability pending determination (SPEC:444-529).

---

### #1154 TKT-20 — Observe message-adapter effects of a Behavior in BehaviorTestKit
- Depth: DEEP
- Labels now: prio:P2, status:blocked
- Verdict: CORRECT
- AC table:

| AC | Pekko contract | Spec body | goPekko | Match? |
|---|---|---|---|---|
| AC-R1-1 CreateMessageAdapter(String,f) => expectEffectType[MessageAdapter] passes; port :333-337 | `EffectfulActorContext.scala:118-127`; `Effect.scala:220` | `SPEC:333-337` `run(CreateMessageAdapter(classOf[String], …)); expectEffectType[MessageAdapter[String, Command]]` | none | yes (type-erased: only the class is asserted, not messageClass or function) |
| AC-R2-1 Int adapter n -> SpawnChildren(n); send 2 to adapter ref, runOne => Spawned child0, child1 | `ActorContextImpl.scala:313-333` (adapter ref = stub `internalSpawnMessageAdapter` FunctionRef; sends `AdaptWithRegisteredMessageAdapter` to selfInbox, `StubbedActorContext.scala:172-186`), `BehaviorTestKitImpl.scala:212-238` | `SPEC:339-354`: `adaptorRef ! 2`; `selfInbox().hasMessages should be(true)`; `runOne()`; two `expectEffectPF` Spawned names `child0`, `child1`. (Before them `expectEffectType[MessageAdapter[String, Command]]` at :343 consumes the first effect.) | none | yes |
| AC-R3-1 no adapter for class => run fails 'can't find a message adaptor'; upstream :227 | `BehaviorTestKitImpl.scala:227` `sys.error(s"can't find a message adaptor for $msgToAdapt")` (RuntimeException) | no spec found; searched `can't find a message adaptor`, `AdaptWithRegisteredMessageAdapter` in actor-testkit-typed/src/test, actor-typed-tests/src/test | none | no spec found |

- Findings:
  - F1 (high, AC-R3-1 precondition). The scenario "a message sent to an adapter ref that has no registered adapter for its class" cannot be reached through the public API in the usual flow: the adapter ref only exists after `messageAdapter` registered at least one adapter (`ActorContextImpl.scala:321-331`), and registering replaces the entry only for the same class (`:318-320`). The failing case requires an adapter ref for class A and a message of class B reaching it (type erasure) or a message of an unregistered subclass relation (`clazz.isInstance`, `BehaviorTestKitImpl.scala:225`). AC text "Given no adapter for the class, when a message is run, then the run fails" lacks a reproducible construction. Open question: how the Go port can construct it; missing evidence: spec. Also the failure happens in `run` and follows the `handleException` path (PostStop delivered, then rethrow; #1142 F3).
  - F2 (medium). Scope bullet "Latest adapter for a class wins as in TYP-05": Pekko replaces the adapter for the same (boxed) class (`ActorContextImpl.scala:318-320`) and `collectFirst` picks the first match by `isInstance` in registration-recency order (`BehaviorTestKitImpl.scala:221-226`), so for overlapping classes the most recently registered applicable adapter wins, not the exact class. No spec. TYP-05 #888 owns the semantics (one per class, lazy ref).
  - F3 (medium). The message `Int`: Pekko boxes primitive classes (`BoxedType`, `:318`), which is why `classOf[Int]` adapter matches `Integer 2` in SPEC:342-345; Go has no such issue. Also adapter returning `null` drops the message (`StubbedActorContext.scala:182`): Pekko-observable; no AC.
  - F4 (low). The spec asserts the adapter ref also forwards the original message to an internal child inbox (`StubbedActorContext.scala:183`, `i.selfInbox().ref ! message`), an internal detail not claimed.
  - F5 (info). Exclusion of `SpawnedAdapter`/`SpawnedAnonymousAdapter` is supported: both are `private[pekko]` (`Effect.scala:156-157,178-179,188-189,211-212`; `ActorContext.scala:255-262` INTERNAL API). The spec does test them (`SPEC:316-330`), i.e. exclusion removes assertions that Pekko tests; that exclusion needs an Owner decision (body says "unless the Owner decides otherwise", "Default: no. Owner decision"): consistent; D13 says the exclusion list is closed and limited to the agreed local frontier and documented JVM-only APIs (line 47), and a private[pekko] API is neither: ADR check needed. Open question for the Owner, no recommendation.
- Dependencies: #1143 -> #1154 OK; #888 -> #1154 OK. Missing: the interceptor in `run` belongs to #1142 (F4 there); `runOne` (#1142). No cycle.
- Oracle (D13): applicability pending determination (SPEC:332-355 exist; failure text has none).

---

## Counts
- DEEP: 11; #1141 and #1150 PARTIAL until the repo-wide spec search (cluster-sharding-typed doc tests) is reflected (1140, 1142, 1143, 1144, 1145, 1146, 1147, 1151, 1152, 1153, 1154 DEEP). Caveats inside DEEP stories: #1147 F1 ActorContextAdapter.stop for non-direct children was read only at lines 77-89 and not executed; #1151 F1 ServiceKey equality source not opened; #1152 F1 SLF4J SubstituteLogger semantics are outside the pinned checkout (stated as not verified).

## Stories the Architect should re-read first
#1142 (AC-R1/R4 cite specs that do not assert the claims; failure semantics missing), #1144 (ports need #1145/#1152; five outcomes; AC gaps), #1145 (AC/R mapping and spec citations swapped), #1153 (cancelAll finding, missing #1150 edge), #1147 (timeout scenario), #1141 (spent-inbox contract missing), #1146 (shutdown default), #1154 (AC-R3-1 reachability, exclusion vs D13).

## Key Learnings
1. BehaviorTestKitSpec lines cited as ports often do not assert the claimed behavior: line 358 tests canonicalization not counting, line 366 asserts only no exception, and line 396 never calls childInbox.
2. In Pekko 1.7.1 the AskInitiated second-completion text 'Can only complete the ask once' is never asserted by a spec, which checks only the IllegalStateException type.
3. Per source reading, EffectfulActorContext.cancelAll passes the key-effect tuple as the key, so it cancels nothing; no spec calls it and it was not executed.
4. BehaviorTestKitImpl.childInbox(ref) looks up by path name only, while childTestKit(ref) checks ref identity.
5. The shared D13 sentence in the story bodies is conditional and compatible with ADR 0033, but three ACs (#1150 AC-R2-1, #1153 AC-R2-1, #1154 AC-R3-1) apply `V3 pending approval` where a Pekko source exists without a spec.
