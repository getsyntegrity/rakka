# Decision package: ADR 0034, the Typed `Behavior[T]` boundary (2026-10-27)

> **2026-10-27:** the current decision list and acceptance wordings are in [typed-owner-decisions-2026-10-27.md](typed-owner-decisions-2026-10-27.md); the findings below were applied to the ADR text in commit `99edce8`.

**Status: documentation only.** This package gives the Owner the questions, the Pekko evidence and the acceptance wording needed to
decide ADR 0034. It accepts nothing. ADR 0034 stays **proposed**, and its text is not changed here. The supervision ADR (0035) has its own
package, [typed-supervision-decision-package-2026-10-27.md](typed-supervision-decision-package-2026-10-27.md). The event stream (readiness
decision 5) and the export form (decision 6) are not part of either package.

Baselines:
- goPekko `develop` `77a923783987ea28b959862f4732c136d4c1847a`. ADR 0033 is accepted there; ADR 0034 and ADR 0035 are not on `develop`.
- **Reviewed text:** ADR 0034 as in commit `cd7c61c` (head of draft PR #1212, which carries the block 3 corrections of #1211 on top of
  #1210). This branch is stacked on #1212. Draft PRs #1215 and #1216 (both "block 5", the inventory) also sit on #1212; this package does
  not depend on them.
- Apache Pekko v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.
- No Owner decision on ADR 0034 is recorded after its proposal. Nothing here asks again for an approval already given.

Path prefixes: `T` = `upstream/pekko/actor-typed/src/main/scala/org/apache/pekko/actor/typed`, `AA` = `T/internal/adapter`,
`TT` = `upstream/pekko/actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed`, `A` = `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor`,
`G` = `pekko-actor/actor`.

Evidence levels: **CODE** (source read), **SPEC** (a test body opened and its assertion read), **EXEC** (a recorded run; none exists for
any row here), **INFERENCE**. Default stance, set by the Owner: keep Pekko v1.7.1 observable behavior; a missing spec is not a reason to
change it; a capability goPekko lacks today is not a Go adaptation.

## Answer first

- **Most of ADR 0034 is an equivalence with Pekko.** P1, P3, P4, P6 and P8 reproduce Pekko's observable behavior; X1, X3 and X5 are
  language adaptations or internals with no observable difference. These can be accepted as a contract (B-1).
- **Three items are mislabeled as Go adaptations.** X2 (nil `Tell`) is the open Classic divergence #452, which ADR 0017 refuses to
  authorize. X4 and X6 (`UnhandledMessage`) are capabilities that three Pekko specs assert and goPekko does not have yet. They should be
  recorded as an open divergence and as deferred capabilities, not accepted as permanent differences (B-3, B-4).
- **P5 is incomplete.** It promises `Terminated{Ref}` and `ChildFailed{Ref, Cause}`, but the runtime `Terminated` carries only an `ID`
  (`G/watch.go:22`), and the ADR does not say where the `Ref` and the cause come from (B-2).
- **The adapter ref of block 4 (F1) is a missing capability, not an adaptation** (B-5). Block 4's F2 misstated Pekko; goPekko is closer to
  Pekko than it said.

## 1. Item review

| Item | Pekko behavior | Spec and what it asserts | Go proposal | Observable difference | Classification | Evidence |
|---|---|---|---|---|---|---|
| P1 one interpreter, native ref | `ActorAdapter` is a Classic actor whose `self` is the ref (`AA/ActorAdapter.scala:65-66`) | every Typed spec, indirectly | the Typed ref **is** `actor.Ref[T]` | none (nil `Tell` is X2) | equivalence proposed | CODE |
| P2 sealed shape, `Extend` | `ExtensibleBehavior` is public (`T/Behavior.scala:106-140`) | `TT/BehaviorSpec.scala`, indirectly | sealed `Behavior[T]`, `(Behavior[T], error)` handlers | interceptors, `transformMessages`, `monitor` are not in P2's list | language adaptation; interceptors are a **deferred capability** | CODE |
| P3 start, validation | `preStart` runs `validateAsInitial(Behavior.start(b))` (`AA/ActorAdapter.scala:299-307`); `validateAsInitial` rejects same and unhandled (`T/Behavior.scala:240-243`); the guardian is validated synchronously on the **unstarted** behavior (`T/ActorSystem.scala:303-304`) | `TT/DeferredSpec.scala:51` (setup runs at spawn), `:99-129` (nested, `transformMessages`, `monitor`), `:147` (`setup(setup(same))` gives an initialization error and `Terminated`), `:63` (setup exception, parent gets `Terminated`) | start the `Setup` chain in `PreStart`, then validate; `ErrInvalidInitialBehavior` | error type only | equivalence proposed | SPEC; guardian CODE |
| P4 turn, special behaviors | `next`: unhandled publishes, stopped stops, otherwise canonicalize (`AA/ActorAdapter.scala:164-179`, `T/Behavior.scala:183-192`); `stopped` delivers `PostStop` then the callback (`AA/ActorAdapter.scala:342-361`); `empty` is unhandled, `ignore` is same (`T/Behavior.scala:297-306`). A **returned** null fails with `NullPointerException` at `AA/ActorAdapter.scala:165` | `TT/ActorContextSpec.scala:100` (same, unhandled, new behavior); `TT/scaladsl/StopSpec.scala:43,55,76,95` (callback order, initial `stopped`); `TT/AskSpec.scala:96` (`empty` publishes `UnhandledMessage`); `TT/scaladsl/AdaptationFailureSpec.scala:105` (empty and ignore survive an adapter failure); no spec for null | P4 table; `(nil, nil)` returns `ErrNilBehavior`, a failure | null: cause value differs (`ErrNilBehavior` versus `NullPointerException`); both are supervisable failures | equivalence proposed; the cause value is a form decision (section 3) | SPEC; null CODE |
| P5 signals | `PreRestart`, `PostStop`, `Terminated`, `ChildFailed <: Terminated`, `MessageAdaptionFailure` (`T/MessageAndSignals.scala:42-125`); `ChildFailed` comes from causes the parent records for watched children (`AA/ActorAdapter.scala:94-101, 268-273`) | `TT/ActorContextSpec.scala:136` (`PreRestart`), `:159` (`PostStop`); `TT/WatchSpec.scala:100` (`ChildFailed.cause`) | sealed `Signal`; `Terminated{Ref}` and `ChildFailed{Ref, Cause}` | none intended, but the mechanism is not specified | **open question** (B-2) | SPEC; mechanism CODE |
| P6 death pact | an unhandled `Terminated` (and `ChildFailed`) throws `DeathPactException` (`T/Behavior.scala:270-276`) | `TT/WatchSpec.scala:165` (`DeathPactException` logged, grand-parent sees `Terminated`); `TT/SupervisionSpec.scala:1408` (supervisable) | `OnTerminated` returns an error wrapping `ErrDeathPact` | error type only | equivalence proposed | SPEC |
| P7 / X4 `UnhandledMessage` | Classic `unhandled` publishes `UnhandledMessage(msg, sender, self)` (`A/Actor.scala:656-660`) | `TT/ActorContextSpec.scala:100`, `TT/AskSpec.scala:96`, `TT/scaladsl/ActorContextDelegateSpec.scala:82`: a subscriber receives it | module-local hook; nothing published | subscribers see nothing | **deferred capability** (the ADR files it as an intentional difference) | SPEC |
| P8 failure outcome | the strategy decides (`AA/ActorAdapter.scala:168-172, 232-262`) | in ADR 0035 | the cause only; the strategy is ADR 0035 | none | equivalence proposed | CODE |
| X1 errors, not exceptions | `receive` throws (`T/Behavior.scala:121`) | none needed | `(Behavior, error)` or panic | none | language adaptation (ADR 0017, accepted) | CODE |
| X2 nil `Tell` accepted | `tell(null)` throws `InvalidMessageException` (`AA/ActorRefAdapter.scala:35-36`; Classic `A/ActorRef.scala:685,737`) | `TT/ActorContextSpec.scala:662` (`intercept[InvalidMessageException]`) | not rejected | the sender gets no error and the behavior receives nil | **divergence**, the same as #452; ADR 0017 (`docs/adr/0017-supervision.md:348-352`) "does not authorize accepting nil" | SPEC |
| X3 sealed `Behavior` | users extend `ExtensibleBehavior` | `TT/BehaviorSpec.scala:467,528`, indirectly | `Extend(ExtensibleBehavior[T])` | none for supported extension | language adaptation | CODE |
| X5 no `failed(cause)` | `BehaviorImpl.failed` is `private[pekko]` (`T/internal/BehaviorImpl.scala:69`; the ADR cites `:71`) | none | a returned error | none (internal) | equivalence proposed | CODE |
| X6 no sender in `UnhandledMessage` | Typed `tell` sends without a sender, so the sender is `deadLetters` (`AA/ActorRefAdapter.scala:35-37`, `A/ActorCell.scala:596-600`) | no spec asserts the sender | field omitted | a missing field whose Pekko value is constant for Typed senders | **deferred capability** | CODE; value INFERENCE |
| F1 (block 4) adapter ref | the adapter ref is a `FunctionRef`, watchable, terminated with its owner (`AA/ActorContextAdapter.scala:135-140`, `A/ActorRef.scala:881-908`) | none found | Typed-module `Ref[U]` over `Port`; `Watch` and `Ask` fail on it | watch and ask work in Pekko, fail in Go | **deferred capability** (B-5) | CODE |
| F2 (block 4) DeadLetter recipient | after the owner stops, the `FunctionRef` forwards to `deadLetters` (`A/ActorRef.scala:862-866, 986-990`) | `TT/scaladsl/MessageAdapterSpec.scala:314-333` asserts the message only | `DeadLetter{To: owner ID}` | small; block 4 said the recipient was the adapter ref, which is wrong | equivalence proposed (correction) | CODE |

## 2. Decisions

| Decision ID | Question | Existing authority | Pekko evidence | Go proposal | Observable difference | Alternatives | Architect recommendation | Affected stories | Proposed acceptance wording |
|---|---|---|---|---|---|---|---|---|---|
| B-1 | Is ADR 0034 P1, P2, P3, P4, P6, P8, X1, X3 and X5 the Typed `Behavior[T]` contract? | ADR 0033 D1 (one runtime; boundary "not decided here") | section 1 rows, SPEC for P3, P4, P6 | ADR 0034 at `cd7c61c` | none beyond error types and the null cause value | accept; accept with changes; keep `Behavior[T]` out until supervision is decided | accept, with interceptors recorded as a deferred capability of P2 | #809, #847, then #925, #954, #994 | "I accept ADR 0034 decisions P1, P2, P3, P4, P6 and P8 and differences X1, X3 and X5, as in commit `cd7c61c`, as the Typed `Behavior[T]` contract. P2's interceptor constructors are a deferred capability. P5, P7, X2, X4 and X6 are decided separately. This accepts a contract only: no implementation, no parity claim, no merge." |
| B-2 | How do `Terminated{Ref}` and `ChildFailed{Ref, Cause}` get their `Ref` and cause? | ADR 0033 D1 | `ChildFailed` from causes recorded by the parent for watched children (`AA/ActorAdapter.scala:94-101, 247, 268-273`); `TT/WatchSpec.scala:100` | P5 states the signals, not the mechanism; runtime `Terminated` has only `ID` (`G/watch.go:22`) | none intended | (a) the interpreter implements `actor.Supervisor`, records the cause of every failure of a watched child (as Pekko does for any directive), and maps `ID` to the watched `Ref`; (b) add a cause to the runtime `Terminated` | (a): no `pekko-actor` change; matches Pekko, including a cause recorded for every failure of a watched child | #847, #954, #956 | "P5 of ADR 0034 is accepted with mechanism (a) of decision B-2 of this package: the Typed interpreter records the failure cause of each watched child and resolves the `Ref` from its watch table. This is a contract precision, not an implementation." **Update 2026-10-28:** B-2 was accepted for `Terminated` only, by `ID` (the watch-table wording of this row is not adopted); the cause half stays open (#1250). See ADR 0034, "Amendment (accepted)". |
| B-3 | Does Typed accept a nil message on `Tell` (X2)? | #452 (open); ADR 0017 does not authorize accepting nil | `AA/ActorRefAdapter.scala:35-36`; `TT/ActorContextSpec.scala:662` | X2: not rejected | sender gets no error; behavior receives nil | (a) follow Pekko: reject nil at `Tell`; (b) accept the divergence; (c) leave X2 to the #452 decision, one rule for Classic and Typed | (c): the Typed ref is the native ref (P1), so one rule must cover both; Pekko's rejection is the default outcome of #452 | #452, #847 | "X2 of ADR 0034 is not accepted as a language adaptation. Nil messages follow the decision of #452 for both Classic and Typed. Until #452 is decided, accepting nil is an open divergence." |
| B-4 | Is P7 (publication of `UnhandledMessage`) delivered now, and are X4 and X6 permanent differences? | ADR 0033 Consequences (event stream not pre-approved); readiness decision 5 (separate) | three specs assert `UnhandledMessage` (section 1, P7) | X4: no publication; X6: no sender field | missing event and field | (a) record both as deferred capabilities tied to the event-stream decision; (b) accept as permanent differences | (a) | #925, #897, #870 | "P7 of ADR 0034 is accepted as a module-local hook until the event stream exists. X4 and X6 are deferred capabilities, not intentional differences. They close when the event stream exists; where it lives is decided separately (readiness decision 5)." |
| B-5 | Can an adapter ref be watched and asked (block 4 F1)? | none accepted | `FunctionRef` is watchable and ends with its owner (`A/ActorRef.scala:881-908`, `A/dungeon/FaultHandling.scala:259`) | Typed `Ref[U]` over `Port`; `Watch` fails with `ErrInvalidHandle` | watch and ask fail in Go | (a) deferred capability, owned by #888; the Typed `Context.Watch` maps an adapter ref to its owner meanwhile; (b) accept as a divergence | (a) | #888, #975 | "Watching and asking a message-adapter ref are deferred capabilities owned by #888. Until delivered, the Typed context maps a watch on an adapter ref to its owner." |

## 3. Form decisions inside the contract (Architect, no Owner approval)

- The failure value of a nil returned behavior (`ErrNilBehavior`). It must stay a supervisable failure, as `NullPointerException` is in
  Pekko; whether a matcher can select it like an NPE is open question Q1.
- Error type names (`ErrInvalidInitialBehavior`, `ErrDeathPact`) and the Go names of signals.

## 4. Corrections to the reviewed text (proposed; ADR 0034 is not edited here)

- B12 and P4 cite `InvalidMessageException` (`T/Behavior.scala:285`) for a null returned behavior. That exception needs a null **current**
  behavior; a returned null fails with `NullPointerException` (`AA/ActorAdapter.scala:165`).
- X5 cites `BehaviorImpl.scala:71`; the definition is at `:69`. The ADR's table lists X6 before X5.
- B14 marks the `ChildFailed` variant READ; `TT/WatchSpec.scala:165` exercises it when the failing child is watched by a Typed parent (inference from `AA/ActorAdapter.scala:94-101, 247-273`). B10 says no spec for `ignore`; `TT/scaladsl/ActorContextAskSpec.scala:147` uses it.
- Applied to ADR 0034 on 2026-10-27, together with the labels of section 1.
- Block 4 F2: the DeadLetter recipient is `deadLetters` (or the owner before `stopFunctionRefs`), never the adapter ref.

## 5. Open questions (no evidence yet) and minimal probes (described, not run)

| Id | Question | Probe |
|---|---|---|
| Q1 | Can a matcher select a nil-behavior failure like Pekko's NPE? | Pekko: `receive` returns `null` under `supervise(...).onFailure[NullPointerException](restart)`; record whether it restarts |
| Q2 | Does a watched child that fails at setup give its parent `ChildFailed` with the cause? | Pekko: Typed parent watches a child whose setup throws; record the signal and the cause class |
| Q3 | Watch and ask an adapter ref | Pekko: watch a `messageAdapter` ref, stop the owner, expect `Terminated(adapterRef)`; ask the adapter ref, expect a reply |
| Q4 | Guardian validation | Pekko: `ActorSystem(Behaviors.stopped)` throws synchronously; `ActorSystem(setup(_ => same))` fails at run time |

## 6. Validation and limits

Done in this session: ADR 0034 (`cd7c61c`) read in full; the readiness (block 2) and shared-foundations (block 4) documents read; every
Pekko line cited above opened, by the Architect or by a read-only explorer. Spot-checked by the Architect: `AA/ActorRefAdapter.scala:35-36`,
`AA/ActorAdapter.scala:163-166`, `docs/adr/0017-supervision.md:348-352`. Not done: no Pekko spec and no Go test was run; this is not an
independent review; no parity is declared.
