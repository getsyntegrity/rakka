# ADR 0030: Bounded mailbox of the local actor (Pekko Classic `BoundedMailbox` with a zero push timeout)

- Status: **accepted** (decisions D1 to D8, in the version of #686 at head `c3d3d3041736ed05ce5b43f2f1e84d7023672243`).
  - Accepted by the Owner on 2026-10-22 through an explicit message in the working session (Claude Code), not a GitHub comment. The Owner's words, literally: "Acepto el ADR 0030, decisiones D1–D8, según el texto completo preparado en #686 para el HEAD: c3d3d3041736ed05ce5b43f2f1e84d7023672243." and "Esta aceptación conserva el comportamiento de Apache Pekko Classic v1.7.1, pin: 9a54d1294c3ac7821bbed3c940769c3d5027d1ff."
  - The same message names what it includes, literally: "Mailbox acotado con descarte a dead letters y timeout de encolado cero, sin bloqueo ni backpressure."; "Capacidad compartida con Terminated y Port."; "Conservación del primer WatchWith tras descarte."; "Encolado de Terminated/WatchWith al final al procesar la notificación, también sin límite."; "Enmienda correspondiente del ADR 0010."; "Diferencias preexistentes y variantes adicionales fuera del alcance documentado."
  - And the limits of evidence that it keeps, literally: "persistence no se ejecutó en la JVM; no hubo revisión nativa ni veredicto nativo. assess informó review_due:false." Nothing beyond that is accepted: this acceptance does not verify behavior outside the local unit tests and the JVM runs of scenarios D-1a to D-11 (and E1, E2 of the independent review) on the `actor` module built from the pin, and it does not replace a native review.
  - **Remote CI.** The two checks of #686 at the accepted head (`go`, `label`) did not start: "The job was not started because recent account payments have failed or your spending limit needs to be increased". They have no verdict and are **not** green. The Owner authorized integration on the local validation (the project's fast path `go run .github/scripts/ci-fast.go`, `scripts/check_deps.sh`, and the race, lint and coverage runs recorded in #686), as an exception the Owner can grant: the branch protection of `develop` lists no required status check. `docs/ci-economy.md` names `go` as the required check; the exception is recorded here and in the comments that close the issues.
- Date: 2026-10-22
- Stage: 1 (actor core)
- Module: `pekko-actor`
- Issues: #176 (ACT-030, overflow policy), #177 (ACT-031, bounded mailbox), #59 (ACT-E11); #182 (ACT-036, stash) out of scope; #685 (flow control and blocking push timeout); #688 (other bounded variants); #689 (Port posts under the bound, implemented here); #693 (pending `Terminated` at stop); #694 (journal and snapshot-load results are not mailbox messages); #677 and #678 (suppressed dead letters, `log-dead-letters`).
- Sources: Apache Pekko `v1.7.1`, commit `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (the submodule pin, verified). **Normative reference: Pekko Classic**, the `BoundedMailbox(capacity, pushTimeOut)` class with `pushTimeOut = Duration.Zero` (`dispatch/Mailbox.scala:705-726`, `Mailboxes.scala:231-235`, the mailbox id `bounded-capacity:N`) and the death-watch and dead-letter machinery of the Classic `ActorCell`. Typed's `MailboxSelector.bounded(n)` is a secondary reference: it only builds that same Classic mailbox (`PropsAdapter.scala:41-45`), so it adds no behavior of its own.

## Context

Pekko has no backpressure in the local `tell`. A bounded mailbox only limits how many messages wait and defines what happens to the excess. Demand-driven flow control belongs to Pekko Streams, not to actors. This ADR migrates the bounded mailbox and nothing else.

## Pekko sources

Paths are relative to `upstream/pekko`.

| Id | File | Lines | What it fixes |
|---|---|---|---|
| P1 | `actor-typed/src/main/scala/org/apache/pekko/actor/typed/Props.scala` | 221-224 | `MailboxSelector.bounded(capacity)`: "new messages are dropped (passed to deadletters)" |
| P2 | `actor/src/main/scala/org/apache/pekko/dispatch/Mailboxes.scala` | 231-235 | `bounded-capacity:N` builds `new BoundedMailbox(capacity, Duration.Zero)` |
| P3 | `actor/.../dispatch/Mailbox.scala` | 557-571 | `BoundedQueueBasedMessageQueue.enqueue`: `offer` with the push timeout; on failure `deadLetters.tell(DeadLetter(msg, sender, receiver))`; negative timeout means `put` (blocks) |
| P4 | same | 705-726 | `BoundedMailbox` checks `capacity < 0` (`:713`); the queue is a JDK `LinkedBlockingQueue(capacity)`, which also refuses 0. Rejecting 0 is therefore JDK behavior, not a Pekko rule (noted in #687); `ErrInvalidMailboxCapacity` for `n < 1` is an adaptation, and Pekko's `NonBlockingBoundedMailbox` accepts 0 (out of scope, #688) |
| P5 | same | 421-440, 690-700 | `NonBlockingBoundedMailbox` / `BoundedNodeMessageQueue` |
| P6 | same | 289-352 | system messages travel in a separate system queue; `cleanUp` sends what is left to dead letters |
| P7 | `actor/.../actor/dungeon/DeathWatch.scala` | 31, 67-93, 99-119 | `watchedActorTerminated` consumes the registration, does `self.tell(Terminated)` (through the user mailbox) and `terminatedQueuedFor` (records an entry **unless one exists**); `receivedTerminated` delivers only if the entry exists; `unwatch` removes the entry |
| P8 | `actor-typed-tests/.../typed/MailboxSelectorSpec.scala` | "set capacity on a bounded mailbox" | capacity 2, actor busy with `one`: `two`, `three` queued, `four` dead-lettered |
| P9 | `actor/src/main/resources/reference.conf` | 642-647 | `mailbox-capacity`, `mailbox-push-timeout-time` |
| P10 | `actor/.../actor/dungeon/FaultHandling.scala` | `faultRecreate` | restart suspends and keeps the mailbox; pending messages are not cleared |
| P11 | `actor/.../actor/ActorRef.scala` | 601-612, 622, 709-723 | `Dropped` exists but only `util/MessageBuffer.scala:268` emits it; a `DeadLetterSuppression` message (`Terminated` is one, `Actor.scala:128`) is published as `SuppressedDeadLetter`, not `DeadLetter` |
| P12 | `persistence/.../snapshot/SnapshotStore.scala` | 62, 77-84, 95, 116 | the snapshot store answers with `senderPersistentActor() ! evt` and `pipeTo(self)(senderPersistentActor())`: a message to the persistent actor's `ActorRef`, through its mailbox |
| P13 | `actor-tests/.../actor/DeathWatchSpec.scala` | 268-291 | "discard Terminated when unwatched between sysmsg and processing" |
| P14 | `actor/.../actor/Stash.scala` | 266-277 | `enqueueFirst` of a `Terminated` calls `terminatedQueuedFor(ref, None)` |

### Reproduction against the pin

`docs/adr/0030-pekko-repro/BoundedTerminatedRepro.java` runs the scenarios D-1a, D-1b, D-5, D-6 (twice: with and without the re-watch), D-7, D-8, D-9 (bounded and unbounded), D-10 (three orderings) and D-11 (a failed PreStart) on the real Pekko classes, with the mailbox id `bounded-capacity:2` that Typed's `bounded(2)` builds; `README.md` there says how. Recorded output (2026-10-22):

| Scenario | Output |
|---|---|
| D-1a: `Terminated` dropped (full), then `watchWith(subject, "B")` on the dead subject | received `[m1, m2, rewatch, A]`; dead letters `[SuppressedDeadLetter: Terminated]`. The first message `A` is delivered, not `B`. |
| D-1b: `unwatch` while a `Terminated` waits, then two `tell` | received `[x1]`; dead letters `[x2]`; no `Terminated`. The slot is kept until the `Terminated` is taken. |
| D-5: a `Terminated` waits when the watcher stops | received `[m1]`; dead letters `[SuppressedDeadLetter: Terminated]`. |

Provenance. The first run used the `actor/target` build in the project checkout (compiled 2026-10-08, origin not proven). It was then repeated on a build made for this purpose: a copy of the submodule at the pin (`git submodule status` clean at `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, copied without `.git`), compiled with `sbt -Dsbt.offline=true actor/compile` under JDK 8 (`JAVA_8_HOME`, Zulu 8.0.504; Scala 2.13.18, sbt 1.12.14). The two class trees were compared with `diff -rq`: **every class is byte-identical except `Version$.class`**, whose version string differs because the copy has no `.git` (`HEAD+20261022-1703` instead of `1.7.1`). The scenarios gave the same outputs on the fresh build (D-6 was added and run only on it). Remaining limits: only the `actor` module was built (the `persistence` classes were not run on the JVM); `pekko.version` in the reproducer's `version.conf` is set by hand to the string of the build, because sbt generates it; the build is a local, offline one and was not compared with a release artifact.

## 1. Equivalences demonstrated

| # | Pekko contract | goPekko | Evidence |
|---|---|---|---|
| E1 | Classic `BoundedMailbox(n, Duration.Zero)`, the mailbox `bounded-capacity:n` (P2, P3); Typed `bounded(n)` is the same mailbox (P1) | `Spawn(..., WithMailboxCapacity(n))` | `bounded_mailbox_test.go` |
| E2 | Full: `DeadLetter` to the receiver, no block (P3) | `Tell` publishes `DeadLetter{Msg, To}` and returns | `OverflowToDeadLetters`, `TellNeverBlocks` |
| E3 | The message under processing is not in the queue (P8) | Depth counts waiting messages only | same |
| E4 | Capacity 2: two queued, later ones dead-lettered, in order (P8) | Same | same |
| E5 | Admission by multiple producers (P5, P6) | Admission under the cell mutex; exactly `capacity` admitted, per-producer order kept | `ConcurrentProducers` |
| E6 | `tell` returns nothing and never blocks; `ask` whose request is dropped ends by timeout | `Tell` unchanged; the request is dead-lettered; `Ask` ends with `ErrAskTimeout` or `ErrAskCanceled` (cancel during the wait); an already-ended context sends nothing (ADR 0015) | `AskAgainstFullMailbox` |
| E7 | `Stop`: queued messages to dead letters, FIFO (P6) | Finalization dead-letters the queue in FIFO order, after the overflow already published | `StopDeadLettersInFIFO` |
| E8 | Restart keeps the mailbox: capacity, pending messages in order, failed message not redelivered (P10) | Queue and capacity live in the cell | `SurvivesRestart` |
| E9 | System messages (stop, restart, suspend, resume, child failure) use a separate queue and are not refused (P6) | Those items travel outside the user queue | by structure; `SurvivesRestart`, `StopDeadLettersInFIFO` |
| E10 | `Terminated` is told through the user mailbox: it takes a slot; if full it is a dead letter and the watcher never sees it (P7) | `notifyTerminated` consumes the registration, then takes a slot or dead-letters it | `TerminatedDroppedWhenFull`, `TerminatedTakesASlot` |
| E11 | `unwatch` removes the entry; the queued `Terminated` keeps its slot until taken, then is ignored (P7, P13) | `Unwatch` deletes the entry only; the item stays in the lane and is ignored when taken | `UnwatchKeepsTheSlotAndSuppressesTheDelivery`; Pekko run D-1b |
| E12 | After a drop the entry stays; a later `watchWith` on the dead subject delivers the **first** message (P7) | `notifyTerminated` records the entry unless one exists | `ReWatchAfterDropDeliversTheFirstMessage`; Pekko run D-1a |
| E13 | Answers that arrive outside the actor's turn are messages to the `ActorRef` and take a mailbox slot (P12) | Port posts share the capacity; a rejected `Post` returns `false` and publishes one `DeadLetter` (the documented contract of a rejected post) | `PortPostRejectedWhenFull`, `PortPostsShareTheCapacityAndKeepOrder`, `TestSnapshotAnswersUnderBoundedMailbox` |
| E14 | A second `Terminated` for a subject whose first one still waits is a second item of the mailbox: it holds a slot until it is taken, finds no `terminatedQueued` entry then (the first took it) and is ignored, so the actor is notified **once** (P7). Delivery and occupancy are separate contracts. | `notifyTerminated` queues the second item; `laneReadyLocked` gives the entry to the first one taken and marks the other ignored | `SecondTerminatedHoldsASlotAndIsNotDelivered` (one `OnTerminated`, the extra `Tell` dead-lettered), control `WithoutTheSecondTerminatedBothAreAdmitted`, `SlotOfTheIgnoredTerminatedIsFreedWhenTaken`, `UnboundedActorSeesOneNotificationWithARewatch`; Pekko run D-6 |
| E15 | The death notification is a system message: the `Terminated` is enqueued when the current message returns, not when `watch` is called inside the handler, and takes its slot only then; an `unwatch` before that leaves nothing to enqueue (P7, `Mailbox.scala` `processMailbox`, `DeathWatchSpec:268`) | `notifyTerminated` queues a pending lane item with no slot; `settleDeathsLocked` admits or drops it between steps (start of every `drain` step and at the end of the activation) and revalidates the registration | `TerminatedIsEnqueuedWhenTheHandlerReturns`, `UnwatchBeforeTheNotificationIsProcessedTakesNoSlot`; Pekko runs D-7 and D-8 (section E15 below) |
| E16 | At the processing the `Terminated` (or the `WatchWith` message) goes to the **tail** of the mailbox as it is then, behind what arrived after the death; no position is reserved when the notification arrives; several notifications are processed in arrival order (P7, `self.tell` at the processing) | `processDeathsLocked` moves the admitted items behind the lane with the count of user messages at the processing; ADR 0010 placement amended | `TerminatedGoesToTheTailWhenItIsProcessed` (capacity 3), `UnboundedTerminatedGoesToTheTailWhenItIsProcessed`, the four ordering tests of ADR 0010's amendment; Pekko runs D-9 and D-10 (section E16 below) |
| E17 | An instance whose `PreStart` failed gets no `PostStop` (#690, integrated by #695 in `develop` `dca0ce92`); a death notification that is pending then is processed while the parent has not decided yet, so with a bounded mailbox that already holds a message the `Terminated` finds it full and is dead-lettered before the queued message; nothing is delivered. A bounded watcher that restarts keeps its registration and mailbox and is told once after the restart (`ActorCell.scala:663-666`, `FaultHandling.scala:247-254`, P7) | `finalize` skips `PostStop` when `initFailed` (#695); `takeOutgoingLocked` clears `lane`, `pendingDeaths` and `termEntry` at the stop; `settleDeathsLocked` runs before the blocked check, so a suspended or restarting cell processes the notification | `InitFailureWithAPendingDeathNotificationLeavesNothingBehind`, `RestartWithAPendingDeathNotificationDeliversTheTerminatedOnce` (controlled executor, 50 runs under `-race`); Pekko run D-11: received `[]`, no `PostStop`, dead letters `[Terminated (suppressed), m]` |

Pekko and goPekko were run on the same sequence with capacity 2 (P7, `BoundedTerminatedRepro` scenario D-6 and `bounded_mailbox_rewatch_test.go`): the first `Terminated` is waiting, the actor watches the dead subject again, the second `Terminated` is enqueued, the first is delivered and the actor parks in it, then two `Tell` arrive.

| | Pekko (pin build) | goPekko |
|---|---|---|
| with the re-watch | received `[m1, TERMINATED, y1]`, dead letters `[y2]` | received `[1, 2, -1, 3]`, dead letters `[4]` |
| control, no re-watch | received `[m1, TERMINATED, y1, y2]`, dead letters `[]` | received `[1, 2, -1, 3, 4]`, dead letters `[]` |

Both show one notification and one extra drop only when the second item exists. The earlier deduplication of the second item (a develop behavior, with its test) imposed a different contract: it made one `Terminated` and one slot where Pekko has one `Terminated` and two slots. That test also fixed an order Pekko does not fix (the second item against a message told concurrently with the re-watch), and failed in 155 of 300 runs under `-race` once the item was kept; it now tells the later message after the re-watch and passes 300 of 300. A difference in when the second item is enqueued was found afterwards and is **not** only timing: it changed what is admitted and whether the actor is notified. It is E15 and was corrected.

### E15. The death notification is a system message: the `Terminated` is enqueued when the handler returns

In Pekko `watch` on a dead subject, and the termination of a live one, deliver a `DeathWatchNotification` **system message**. It is processed after the current message returns (`Mailbox.scala` `processMailbox`: `invoke` then `processAllSystemMessages`), never inside the handler, and only then does `watchedActorTerminated` do `self.tell(Terminated)` (P7). goPekko used to enqueue the item when `Watch` was called inside the handler, so a `Tell` completed meanwhile met a different mailbox. Scenario D-7 (capacity 2): the handler of `m1` watches a dead subject and stops inside; `x1`, `x2`, `x3` are told while it is stopped.

| | Pekko (pin build) | goPekko before E15 | goPekko now |
|---|---|---|---|
| D-7 received by the actor | `[m1, x1, x2]` | `[1, -1, 2]` | `[1, 2, 3]` |
| D-7 dead letters | `x3` and the `Terminated` (suppressed there) | `x2`, `x3` | `4` and the `Terminated` |
| D-8: the handler also unwatches before it stops | received `[m1, x1, x2]`, dead letters `[x3]`, no `Terminated` | not run | received `[1, 2, 3]`, dead letters `[4]`, no `Terminated` |

Before E15 the actor was notified and one message less was admitted; in Pekko it is never notified and two are admitted, because the `Terminated` finds the mailbox full when the notification is processed. That is a contract difference, not a timing detail. The classification, with what each part rests on:

- **Order Pekko guarantees (and goPekko now keeps).** System messages are processed after the current message and before the next one, so the `Terminated` is enqueued at the tail of whatever the mailbox holds at that moment, and it takes a slot only then. A `Tell` that completed before the handler returned is ahead of it; the admission of the `Terminated` depends on those, not on when `Watch` was called. An `Unwatch` before the processing leaves the notification without a registration, so nothing is enqueued, nothing is dead-lettered and no slot is used (`DeathWatchSpec:268`, D-8).
- **Order Pekko does not guarantee (not reproduced and not required).** The order between messages of different senders that arrive at the same time, and whether a `Tell` from another goroutine lands just before or just after the system message is processed when the mailbox is idle: that depends on the dispatcher's thread timing. The goPekko tests therefore tell only while the handler is stopped on a channel, where the order is fixed.
- **Differences derived from system message processing.** (1) *Resolved by E16.* The first version of E15 kept the place that the notification had among the runtime deliveries, so the `Terminated` went **ahead** of a message told while the handler was stopped, which is the opposite of what an earlier text of this ADR said about Port posts, and not Pekko's order: with capacity 3 Pekko receives `[m1, x1, x2, TERMINATED]` and that version received `[1, -1, 2, 3]` (scenario D-9). E16 moves the `Terminated` to the tail at the processing and amends the placement of ADR 0010. (2) A notification for a registration that is gone or replaced is discarded when it arrives, as before, and checked again when processed; Pekko only checks at the processing. The outcome is the same (nothing is enqueued, dead-lettered or counted); the early discard only avoids keeping a notification that cannot matter.

### E16. The `Terminated` goes to the tail of the mailbox as it is when the notification is processed (D8, amends ADR 0010)

Pekko's `watchedActorTerminated` does `self.tell(Terminated)` when the system message is processed, so the `Terminated` is behind **everything that is in the mailbox at that moment**, including what arrived after the death and before the processing. goPekko used to reserve its place when the notification arrived (ADR 0010: "handled after every user message queued before the notification and before any queued after it"). That is replaced, by the Owner's authorization quoted in ADR 0010 ("Amendment 2026-10-22"). The contract, as implemented in `processDeathsLocked`:

1. A pending death notification takes no capacity and reserves no position.
2. It is processed between turns of the actor: before every step of `drain` and when an activation ends, never inside a handler.
3. On processing it is dropped if its registration was withdrawn or replaced; otherwise the `Terminated` (or the `WatchWith` message) is enqueued at the tail, behind every user message and runtime delivery already there, or goes to dead letters if a bounded mailbox is full. Several notifications are processed in arrival order.
4. No position is kept from the arrival of the notification: the item is enqueued with the number of user messages queued at the processing.

Scenario D-9 (the handler of `m1` watches a dead subject and stops inside; `x1` and `x2` are told while it is stopped), run on the Pekko pin build and on goPekko, bounded with capacity 3 and unbounded, and scenario D-10 (the handler is stopped, one message is told before the death and others after it):

| Scenario | Pekko (pin build) | goPekko before E16 | goPekko now |
|---|---|---|---|
| D-9, capacity 3 | received `[m1, x1, x2, TERMINATED]`, dead letters `[]` | `[1, -1, 2, 3]`, `[]` | `[1, 2, 3, -1]`, `[]` |
| D-9, unbounded | received `[m1, x1, x2, TERMINATED]`, dead letters `[]` | `[1, -1, 2, 3]` | `[1, 2, 3, -1]`, `[]` |
| D-10, plain Watch, `m3 m4 \| death \| m5` | `[m3, m4, m5, T]` | `[m3, m4, T, m5]` | `[m3, m4, m5, T]` |
| D-10, one WatchWith, `m3 \| death \| m5` | `[m3, m5, W0]` | `[3, 99, 5]` | `[3, 5, 99]` |
| D-10, two WatchWith, `m3 \| death 0 \| m4 \| death 1 \| m6` | `[m3, m4, m6, W0, W1]` | `[3, 98, 4, 99, 6]` | `[3, 4, 6, 98, 99]` |

The "before E16" column of D-9 was measured on `77ccdabc`; those of D-10 are what the corresponding tests expected, and passed with, at `77ccdabc` before they were changed. D-6, D-7 and D-8 are unchanged and still pass.

- **Equivalences demonstrated by E16:** the position of the `Terminated` and of the `WatchWith` message against messages told before the processing, bounded and unbounded; the order of several notifications; the admission of the `Terminated` against what is in the mailbox then (D-7); the single notification (D-6); the withdrawal by `Unwatch` before the processing (D-8).
- **Not guaranteed by Pekko, not reproduced:** the order between a message told by another goroutine and the processing of the notification when it happens at the same time (dispatcher thread timing). The `barrier` helper of `watch_api_test.go` (one message told right after `Done`) and two tests asserted that a message told concurrently comes after the `Terminated`; they relied on the superseded placement and now wait for the `Terminated` or use a second barrier (see ADR 0010, amendment).
- **Changes to earlier contracts:** the superseded sentences of ADR 0010 and four ordering tests (named in the amendment). The unbounded actor changes too: a message told after the death but before the processing is now handled before the `Terminated`, as in Pekko.
- **Remaining difference, not a contract of this ADR:** the early discard of a stale notification (see E15).

### E17. Combined with #695 (develop `dca0ce92`)

`develop` moved to `dca0ce9208f0d1679baf0f8c46c1164551fe15a3` while this PR was open: #695 fixes #690 (no `PostStop` for an instance whose first `PreStart` failed, `ActorCell.scala:663-666`, `FaultHandling.scala:247-254`; it touches `stop.go` `finalize`, `actor.go`, `persistent.go`, the README and ADR 0012). The branch was brought up to date by a **merge commit** (`origin/develop` into `feat/bounded-mailbox`, no force push), without conflicts and without editing the files of #695. The two changes do not share code: #695 decides whether `PostStop` runs, this PR decides when the `Terminated` is enqueued and what it costs. The combinations that could interact were checked:

- *Failed `PreStart` with a pending death notification.* `finalize` runs no `PostStop` (#695) and the stop (`takeOutgoingLocked`) clears the lane, the pending count and the entries, so nothing is left to hold a slot or to fire later. The notification, being a system message, is processed after the failed start and before the parent's decision, as in Pekko (run D-11), so a bounded mailbox that already holds a message dead-letters the `Terminated`, then the message. Test `InitFailureWithAPendingDeathNotificationLeavesNothingBehind`.
- *Restart.* A restart keeps the registration, the mailbox and the entries (`SurvivesRestart`, D-1a). A notification that arrives while the cell restarts is processed (`settleDeathsLocked` runs before the blocked check, as a system message is processed while the mailbox is suspended) and the actor is told once after the restart. Test `RestartWithAPendingDeathNotificationDeliversTheTerminatedOnce`; also 1500 rounds of a throwaway stress with a restart and a death at the same time under `-race`, not committed.
- *Failure of the instance a restart builds* (`PostRestart` or its `PreStart`): #695 keeps its `PostStop`; it does not touch the lane.
- *`Unwatch`, `WatchWith`, Port and capacity* do not reach `finalize`; D-1a to D-10 pass unchanged on the merged tree.

Port in detail (#689): the only producer is the snapshot worker (`snapshot_worker.go`, six Save/Delete answers) and the only consumer is `ReceivePort` (`snapshot_ops.go`). The recovery `Load` does not use Port. No operation waits for a Port answer, so a rejected answer cannot leave anything pending; the worker ignores the result of `Post`. Under saturation a Save/Delete answer can be lost, as in Pekko; the snapshot operation itself is not undone, and the actor and the worker of the cell go on (persistence test with a held store call). Order is arrival order with the user messages.
  *See ADR 0037, Decision 1 (2026-11-05, additive): `Port` is a Go adaptation; admission, order and capacity stay as this paragraph and D6 state. See also Decisions 3 and 5 for what `ReceivePort` does with a foreign type and with an unhandled snapshot result.*

## 2. Pekko variants outside this scope

- `BoundedMailbox` with a positive push timeout (blocks the producer up to the timeout, P3, P4) and the negative-timeout `put` (blocks forever): #685. Blocking the caller of `Tell` contradicts #176 ("core does not block an unknown goroutine by default") and the contract "`Tell` never blocks".
- `NonBlockingBoundedMailbox` (P5), including capacity 0; `BoundedPriorityMailbox`, `BoundedStablePriorityMailbox`, `BoundedDequeBasedMailbox`, `BoundedControlAwareMailbox`: #688. Bounded stash: #182.
- `MailboxSelector.fromConfig` and the mailbox configuration keys (P9). The capacity contract they express is expressible with `WithMailboxCapacity`, so their absence is not a functional divergence; a name-based lookup is a separate feature (#688).
- Dead-letter logging and throttling (`log-dead-letters`).

## 3. Differences introduced or exposed by this PR (decided by the acceptance)

- **D-1a. Decided: keep Pekko's stale entry (D4, accepted).** E12 is implemented as Pekko does, and the Pekko run shows it is the real behavior of v1.7.1: after a dropped `Terminated`, `terminatedQueued` keeps the first custom message and a re-watch with another message delivers the first. Evidence on intent is mixed: `terminatedQueuedFor` has no comment on the non-overwrite; `DeathWatchSpec` (P13) covers unwatch but not drop-then-rewatch; `Stash.scala` (P14) shows the entry is the "this Terminated is in flight" marker; the drop case is not tested or documented in the original. It is therefore not concluded to be accidental. **Recommendation:** keep it as implemented (follows the migrated behavior and the project rule that Pekko wins); if the Owner prefers the message of the latest `WatchWith`, that is a deliberate divergence to be decided, documented and tested, and `terminatedQueuedForLocked` is the only place to change. The Owner accepted the recommendation (D4).

- **D-6, resolved as an equivalence (E14); its enqueue difference resolved as E15 and E16.** Pekko run and goPekko run agree: the second `Terminated` holds a slot and is not delivered, the `Terminated` is enqueued when the handler returns (E15) and at the tail of what the mailbox holds then (E16). The existing coalescing test was imposing a different contract and was made deterministic (see E14, E16). No difference in the position or the moment of the `Terminated` is left open; what remains is the early discard of a stale notification (E15), which has the same outcome.

## 4. Differences that predate this PR

Not introduced here, not changed here; listed so that none is read as equivalence.

- **P-1. `DeadLetter` carries no sender.** Pekko's `DeadLetter(message, sender, recipient)` does; goPekko's has `Msg` and `To`. Follow-up: accepted adaptation, ADR 0010 D2 (no implicit sender; the reply address travels in the message).
- **P-2. No suppressed dead letters.** Pekko publishes a dropped `Terminated` as `SuppressedDeadLetter` (P11), which the default listener does not log; goPekko has one `DeadLetter` channel, so a subscriber sees it as an ordinary `DeadLetter` whose `Msg` is a `Terminated`. Follow-up: #677 (and #678 for `log-dead-letters`).
- **P-3. A `Terminated` still waiting when the watcher stops is discarded without a dead letter** (`watch.go`, `takeOutgoingLocked`: "not messages and never dead-lettered"; it predates this PR). Pekko's `cleanUp` dead-letters it (Pekko run D-5, as `SuppressedDeadLetter`). Applies to unbounded actors too. Follow-up: #693.
- **P-4. The journal write and replay results and the recovery snapshot `Load` do not travel through the mailbox** (`journal.Write` and `journal.Replay` are synchronous steps of the turn and `loadQueued` waits on its own channel); in Pekko they are messages to the persistent actor (`Eventsourced.scala:367, 687`; `SnapshotStore.scala:62`) and a bounded mailbox can dead-letter them. Not run on the JVM. Follow-up: #694. Only the snapshot Save/Delete answers, which do return through the Port, are bounded (E13, #689).
- **P-5. `Dropped`.** In v1.7.1 only `MessageBuffer` emits it (P11); the bounded mailbox emits `DeadLetter`. The audit comment on #177 that links `Dropped` to this work refers to `MessageBuffer`.

Go adaptations that are not divergences: admission under a mutex instead of a lock-free CAS loop (same observable result); `Spawn` returns `ErrInvalidMailboxCapacity` for a capacity below 1 (Pekko fails when the mailbox is created; a Go function reports it at the call and reserves no name); the overflow is published on the sender's goroutine, as the dead letters of a stopped actor already are; a `Port.Post` made from `AttachPort` can now be rejected if the mailbox is full.

## Decisions (accepted)

- **D1.** Additive variadic option `WithMailboxCapacity(n)` on `Spawn`. Without it the capacity stays **unbounded**, and `Spawn` still compiles and returns the same three errors for the same calls. **This is a statement about capacity only, not about all behavior:** the position of the `Terminated` and of the `WatchWith` message changes for every watcher, also one without a capacity, by D8. `Spawn` documented "exactly three errors"; `ErrInvalidMailboxCapacity` is the fourth, reachable only through the new option.
- **D2.** Policy is Classic's `BoundedMailbox` with an enqueue timeout of zero: when `capacity` messages wait, the new message is not queued; it is sent to the dead letters (`DeadLetter(message, sender, recipient)`, `Mailbox.scala:560-568`) and the sender is not blocked (`offer(handle, 0, unit)` returns at once). Typed's `bounded(n)` builds exactly this mailbox. Not adopted: the default `mailbox-push-timeout-time = 10s` of `reference.conf` and the negative timeout, which block the sender (section 2, #685). No error, acknowledgement or demand signal reaches the sender. This is not backpressure.
- **D3.** Capacity counts the user messages waiting plus the admitted lane items: `Terminated` deliveries (E10, including a second one that will be ignored, E14) and Port posts (E13). A death notification that has not been processed yet is a system message and takes no slot (E15). Stop, restart, suspend, resume and child-failure items are outside it (E9). An item keeps its slot until it is taken, also after `Unwatch` (E11).
- **D4.** The stale `terminatedQueued` entry of a dropped `Terminated` is kept as in Pekko v1.7.1: a later `WatchWith` on the dead subject delivers the **first** message (E12, section 3). Keeping the first is the option faithful to Pekko; choosing the latest would be a new divergence, which this ADR does not take.
- **D5.** The differences P-1 to P-5 are recorded as pre-existing and out of this decision; they are followed in ADR 0010 D2, #677, #693 and #694.
- **D6.** Port (#689). **The implementation is done in this PR**: Port posts share the capacity (E13). What is accepted is the decision that they do, and that the answers of the snapshot store can be lost under saturation as in Pekko. No implementation work is pending for Port; a rejection of D6 would be a change to remove it.
  *See ADR 0037, Decision 1 (2026-11-05, additive): capacity per D6 is kept; `Port` is recorded as a Go adaptation.*
- **D7.** The blocking enqueue timeout (#685) is **outside this variant**. This ADR adopts a zero timeout (D2) and does not make the mailbox of #686 blocking, now or automatically if #685 is decided; a blocking variant would be a separate option with its own ADR.
- **D8.** Placement of the `Terminated` (E16). **The implementation is done in this PR, the Owner authorized the change of the placement of ADR 0010** (quoted in the amendment of ADR 0010, 2026-10-22) **and then accepted this decision** (see Status). What is accepted: that the `Terminated` and the `WatchWith` message are enqueued at the tail when the death notification is processed, **also for unbounded actors and without `WithMailboxCapacity`** (the one observable change that D1 does not cover), with the changes to the tests listed in the amendment. Example, an unbounded watcher: handler stopped on message 2, messages 3 and 4 queued, the subject dies, message 5 is told, the handler returns: `develop` gives `m3 m4 T m5`, this PR and Pekko give `m3 m4 m5 T`. `develop` changes with the integration of #686.

## Independent review of the candidate (`c0e1e6f4`)

A reviewer separate from the implementer, with its own context and read-only, was asked to refute the candidate `c0e1e6f48d338609a3fbf5f90c828dadd933a445` against base `dca0ce9208f0d1679baf0f8c46c1164551fe15a3` (27 files, +2392 -77) and the Pekko pin. This is a **conventional review, not a native one**: `gentle-ai review assess` reported risk `medium` with `review_due: false` (`under_budget`), no native cycle (STATUS, START, consent) was run, and there is no native verdict. It does not replace one.

- **Result:** no blocking finding. Capacity accounting, lane compaction, ordering (`after` monotonic), wakeups, the publication of dead letters outside the lock, `Unwatch`, the non-overwritten entry, the ignored item, stop and restart, `Port` and the `Spawn` compatibility were checked by reading the code and found correct. The suites passed `-race` (`pekko-actor`; `pekko-persistence` and `pekko-projection` without `FAIL`) and the pinned lint reported 0 issues.
- **Own JVM scenarios** (Pekko `actor` built from the pin; the Go side in a scratch copy of `pekko-actor` under `-race`, 3 runs): **E1**, capacity 2, a plain `Watch` in `PreStart`, a full mailbox, the subject dies, then `watchWith` on the dead subject: Pekko `[m1, m2, rewatch, TERMINATED]` with the `Terminated` dead-lettered; Go `[m1 m2 rewatch TERMINATED]`. It shows the stale entry after a dropped **plain** `Terminated` (D-1a covered a dropped `WatchWith`). **E2**, capacity 3, `watchWith("A")`, `x` queued, the subject dies, `x` runs `unwatch` then `watchWith("B")`: Pekko `[x, B]`, no dead letters; Go `[x B]`. The generation check of goPekko and Pekko's consume-then-ignore give the same single delivery.
- **Findings, all non-blocking:** (F1) D1 and its acceptance text said that callers without the option are unchanged, which is false for the position of the `Terminated`; corrected in D1 and D8. (F2) four tests poll with `time.Sleep(1ms)` under a 30 s limit (`bounded_mailbox_terminated_test.go:71,79`, `bounded_mailbox_port_test.go:75`, `bounded_mailbox_unwatch_test.go:22`); no unsound negative assertion was found and no defect was shown, so they were **not changed**. (F3) the documentary signature of `Spawn` in `docs/adr/signatures/api.go` lacked the option; corrected.
- **Not verified by the reviewer:** the interaction with #695 (covered by the repository's own scenario D-11 and tests, not re-run by the reviewer), `Port` (no JVM counterpart: read from the source and tested only in goPekko), the repository's D-1a to D-11 output (not re-run), mutation of the tests, and `gopekko-testint`. The `persistence` module was **not executed on the JVM**.

## Not backpressure

The requested backpressure is not provided by a mailbox in Pekko. It maps to Pekko Streams (epic #105) or to application protocols (Ask with a deadline, explicit credit/ack messages). Tracked in #685.

## Do #176 and #177 close?

Not by this PR: it carries no closing keyword. The issues are evaluated after the integration, one by one against their own checklists, and each is closed only if every criterion is met; what is not met is written on the issue.

- #176 asks to choose reject/drop/backpressure with an observable error and not to block an unknown goroutine by default. The choice is made and accepted (D2): drop to dead letters, the `DeadLetter` as the observable signal, no blocking, and the "observable error" is a dead letter, not an error returned to the sender.
- #177 asks for capacity under concurrency, FIFO of the promised scope, and Stop that drains or discards per contract; those are tested (`ConcurrentProducers`, `StopDeadLettersInFIFO`).
- Both issues also carry a checklist of the migration alignment (a record of the kit execution with models, roles, costs and permissions, and a ported test, TCK or judge case) and "CI of the final SHA". The CI checks never started (see Status); no judge was run.
- #689: the decision (D6) and the deterministic tests exist; its consequences list also asks that the documentation of `SnapshotResultReceiver` say that the answers are subject to the bound.

## Consequences

- `docs/actor-core-first-cut.md` "No bounded mailboxes" and ADR 0010 "dispatcher and mailbox configuration" and "a bounded mailbox policy for `Terminated`" stay true for the unbounded default and are superseded, for the opt-in bounded mailbox, by this ADR now that it is accepted.
- **ADR 0010 is amended for one decision only** (the placement of the `Terminated`, "Amendment 2026-10-22" in that ADR), by the Owner's explicit authorization, now confirmed by the acceptance of D8; the rest of ADR 0010 is untouched. The behavior of `develop` changes with the integration of #686.
- No dependency changes; no SQLite or PostgreSQL impact (`pekko-actor` imports no other goPekko module; the only user of Port is the snapshot worker, whose tests use the in-memory store).
