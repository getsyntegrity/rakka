# Oracle: EVT-1, `UnhandledMessage` and `Dropped` in Pekko Typed (Pekko v1.7.1)

- Date: 2026-11-05. Evidence only; this document recommends nothing and changes no ADR.
- Pekko `pekko-actor-typed` 1.7.1 and `pekko-slf4j` 1.7.1 (tag `v1.7.1`, `upstream/pekko`), Scala 2.13.18, sbt 1.10.7, JDK Temurin 11.0.32+9, default fork-join dispatcher, logback-classic 1.3.14. The sbt project is outside the repository and is not committed.
- Purpose: answer ADR 0034 X6 (what the sender of `UnhandledMessage` carries) and the "To be verified" of ADR 0036 EVT-1 (the publication route of `UnhandledMessage`; the fields of `Dropped`).
- Method: read the code (section 1), then run each reachable site with a Typed and a Classic subscriber to `UnhandledMessage`, `Dropped` and `DeadLetter`, 2 runs per scenario (`pekko.actor.debug.unhandled` = `on` in run 1, `off` in run 2).

## 1. Publication map (code reading, Pekko 1.7.1)

`A` = `actor/src/main/scala/org/apache/pekko/`, `T` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/`.

| Site | What triggers it | Reachable from the public Typed API |
|---|---|---|
| `A/actor/Actor.scala:297` | `final case class UnhandledMessage(message, sender, recipient)` | n/a |
| `A/actor/Actor.scala:655-660` `Actor.unhandled`: `case Terminated(dead) => throw DeathPactException`; `case _ => context.system.eventStream.publish(UnhandledMessage(message, sender(), self))` | default `unhandled` of the classic actor | yes, through the Typed route below |
| `T/internal/adapter/ActorAdapter.scala:166-167` (`next`, case `UnhandledBehavior` calls `unhandled(msg)`) | a Typed behavior returns `Behaviors.unhandled` (`T/scaladsl/Behaviors.scala:65` -> `T/internal/BehaviorImpl.scala:60`) while handling a message | yes (U1, U2, U3) |
| `T/internal/adapter/ActorAdapter.scala:220-229` (`override def unhandled`): `Terminated` -> `DeathPactException`; `case _: Signal =>` nothing ("that's ok"); `case other => super.unhandled(other)` | the adapter's override of the classic hook | yes; unhandled signals publish nothing (U4) |
| `T/internal/adapter/ActorAdapter.scala:187` (`adaptAndHandle`, case `Nil`) | a message of a class with no registered message adapter reaches `AdaptWithRegisteredMessageAdapter` | only through a message-adapter ref whose adapter is gone; not executed |
| `A/event/DeadLetterListener.scala:42-43, 126-132` | subscribes to `UnhandledMessage` and `Dropped` and logs them ("was unhandled", "was dropped") | yes (log) |
| `A/event/Logging.scala:154-163`, `A/actor/ActorSystem.scala:460` | with `pekko.actor.debug.unhandled = on`, a forwarder logs "unhandled message from <sender>: <msg>" at DEBUG | yes (log) |
| `A/actor/ActorRef.scala:622` `Dropped(message, reason, sender, recipient)`; `:624-630` companion `apply(message, reason, recipient)` = `Dropped(message, reason, ActorRef.noSender, recipient)` | n/a | n/a |
| `T/internal/Supervision.scala:121-124` `dropped`: `eventStream.publish(Dropped(signalOrMessage, "Stash is full in [<class>]", ctx.self.toClassic))` (3-argument form) | restart/backoff stash full | yes (D1) |
| `T/internal/pubsub/TopicImpl.scala:96`: `context.system.deadLetters ! Dropped(message, "No topic subscribers known", context.self.toClassic)` (3-argument form, sent to `deadLetters`, not published directly) | `Topic.Publish` with no subscribers | yes (D2) |
| `T/internal/routing/GroupRouterImpl.scala:97-101`: `eventStream ! EventStream.Publish(Dropped(msg, "Stash is full in group router for [<key>]", self))` (3-argument form) | initial group router (before its first listing) with a full stash (hard-coded capacity 10000, `:85`) | yes in theory (D4) |
| `T/internal/routing/GroupRouterImpl.scala:152-153`: `eventStream ! Publish(Dropped(msg, "No routees in group router for [<key>]", self))` (3-argument form) | active group router with no routees | yes (D3) |
| `A/util/MessageBuffer.scala:268`: `deadLetters ! Dropped(msg, reason, ref, ActorRef.noSender)` (4-argument form) | classic `MessageBuffer` | no (not used by `actor-typed`) |
| `A/actor/ActorRef.scala:736-743` `DeadLetterActorRef.!`: `case d: DeadLetter => publish(d)`; `case _ => publish(DeadLetter(message, <sender or deadLetters>, this))` | anything sent to `deadLetters` that is not a `DeadLetter`, `Dropped` included | explains D2 |

Typed route of an unhandled message: `Behaviors.unhandled` (`Behaviors.scala:65`) -> `BehaviorImpl.UnhandledBehavior` (`BehaviorImpl.scala:60, 86`) -> `ActorAdapter.next` (`:166-167`) -> `ActorAdapter.unhandled` (`:220-229`) -> `Actor.unhandled` (`Actor.scala:655-660`) -> `context.system.eventStream.publish(UnhandledMessage(message, sender(), self))`. `Actor.sender()` is the classic sender of the current envelope. `ActorRefAdapter.tell` sends `classicRef ! msg` with no sender (`T/internal/adapter/ActorRefAdapter.scala:35-38`).

## 2. Harness (relevant parts)

Typed subscribers (one per class) and a Classic subscriber, all printing the event class, `message`, `sender`, `recipient`, `reason` (for `Dropped`), the thread and `toString`:

```scala
def sub[E <: AnyRef](cls: Class[E], name: String)(implicit ct: ClassTag[E]): Unit = {
  val ref = ctx.spawn(Behaviors.receiveMessage[E] { m => show("typed", m); Behaviors.same }, name)
  ctx.system.eventStream ! EventStream.Subscribe[E](ref)
}
sub[UnhandledMessage](...); sub[Dropped](...); sub[DeadLetter](...)
val classicSubRef = ctx.actorOf(Props(new Actor { def receive = { case m => show("classic", m) } }), "classicSub")
// classic eventStream.subscribe(classicSubRef, classOf[UnhandledMessage | Dropped | DeadLetter])
```

Scenarios:

```scala
U1: target = receiveMessage[String](_ => Behaviors.unhandled); another Typed actor does `target ! "u1-msg"`
U2: same target; `target ! "u2-msg"` from the main thread
U3: target = receiveMessage[AskMsg](_ => Behaviors.unhandled); `target.ask(r => AskMsg(r))` (AskPattern, 500 ms timeout)
U4: receiveMessage(_ => Behaviors.stopped).receiveSignal { case (_, s) => Behaviors.unhandled }   // PostStop reaches the handler
U5: classic actor with receive { case "known" => }; a classic sender does `tgt.tell("classic-msg", self)`
U6: target.unsafeUpcast[Any] ! Integer.valueOf(42)                       // wrong message class
D1: supervise(setup { spawn(slowChild); receiveMessage { case "fail" => throw ..; case m => .. } })
      .onFailure[RuntimeException](restart.withStashCapacity(2)); slowChild sleeps 1500 ms in PostStop;
      send "fail", wait 400 ms, then m1..m5
D2: Topic[String]("t1"); wait 300 ms; `topic ! Topic.Publish("topic-msg")`
D3: Routers.group(ServiceKey[String]("k")); wait 800 ms; `router ! "r1"`
D4: Routers.group(...); immediately send 20000 messages in a loop
```

## 3. Results

Two runs per scenario. Both subscribers (Typed and Classic) received every event listed; the order between them varied. A DeadLetter column of "-" means no `DeadLetter` event was received.

### 3.1 Unhandled messages

| Scenario | Event received (both subscribers) | `message` | `sender` | `recipient` | `DeadLetter` event | Extra log |
|---|---|---|---|---|---|---|
| U1 Typed tell from a Typed actor | `UnhandledMessage` | `u1-msg` | `deadLetters` (`pekko://evt/deadLetters`) | `pekko://evt/user/target` | - | INFO "Message [String] to Actor[.../target] was unhandled" (both runs) |
| U2 tell from outside the system (main thread) | `UnhandledMessage` | `u2-msg` | `deadLetters` | `pekko://evt/user/target` | - | same INFO line |
| U3 ask | `UnhandledMessage` | `AskMsg(Actor[pekko://evt/temp/target$a#0])` | `deadLetters` (not the temp ask actor) | `pekko://evt/user/target` | - | same INFO line; the ask ends in `TimeoutException` |
| U4 unhandled signal (`PostStop`) | none | - | - | - | - | none (the handler ran: "SIGNAL handler got PostStop") |
| U5 Classic `unhandled` with an explicit sender | `UnhandledMessage` | `classic-msg` | `pekko://evt/user/csender` | `pekko://evt/user/ctarget` | - | same INFO line (with "from Actor[...csender]") |
| U6 wrong message class sent to a Typed actor | none | - | - | - | - | `ClassCastException` logged by the supervisor, no `UnhandledMessage` |

`pekko.actor.debug.unhandled`: the INFO line "was unhandled" appears with `on` and `off` (it comes from `DeadLetterListener`). With `on` and the log level at DEBUG, one more line appears: `unhandled message from Actor[pekko://evt/deadLetters]: u1-msg`; with `off` it does not (checked with `U1` at DEBUG level). The event is published in both modes.

### 3.2 Dropped messages

| Scenario | Event received | `message` | `reason` | `sender` | `recipient` | `DeadLetter` event | Extra log |
|---|---|---|---|---|---|---|---|
| D1 restart stash overflow (`withStashCapacity(2)`; 5 messages while the slow child stops) | `Dropped` (3 events, m3 to m5), both subscribers | `m3`, `m4`, `m5` | `Stash is full in [RestartSupervisor]` | null (`noSender`) | the restarting actor, `pekko://evt/user/parent` | - | INFO "was dropped. Stash is full in [RestartSupervisor]" x3; `m1` and `m2` processed after the restart |
| D2 `Topic.Publish`, no subscribers | **`DeadLetter`** only; **no `Dropped` event** | `Dropped(topic-msg,No topic subscribers known,null,Actor[pekko://evt/user/topic#...])` (a `Dropped` wrapped in the dead letter) | inside the wrapped `Dropped`: `No topic subscribers known` | `deadLetters` (of the `DeadLetter`); `null` inside the wrapped `Dropped` | `deadLetters`; inside the wrapped `Dropped`: the topic actor | yes (that is the event) | INFO "Message [String] wrapped in [Dropped] to Actor[.../deadLetters] was not delivered" |
| D3 group router with no routees | `Dropped` | `r1` | `No routees in group router for [ServiceKey[java.lang.String](k)]` | null (`noSender`) | the router, `pekko://evt/user/router` | - | INFO "was dropped. No routees ..." |
| D4 group router, initial stash full | **not reproduced**: 20000 `Dropped` events (both runs) with reason `No routees ...`; 0 with `Stash is full in group router` | `f0` to `f19999` | `No routees in group router ...` | null | the router | - | the first listing arrived before the first message, so the 10000-message stash was never filled |

## 4. Findings vs ADRs

Compared with the current text of ADR 0034 (X4, X6, P7) and ADR 0036 (EVT-1, "To be verified"). No recommendation.

| # | ADR text | Observed | Result |
|---|---|---|---|
| 1 | ADR 0034 X6: "`UnhandledMessage` has no sender field until a sender exists in the runtime envelope" | Pekko's `UnhandledMessage` has a `sender` field. From a Typed `tell`, from outside, and from an ask, its value is the system's `deadLetters` ref (not null, not the Typed sender, not the ask temp actor). With a Classic explicit sender it is that sender (U5). | Does not match the premise "no sender field": Pekko has the field, filled with `deadLetters` for Typed messages. |
| 2 | ADR 0034 P7: "Pekko's also carries the sender, `Actor.scala:659`" | Confirmed: `Actor.scala:659` publishes `UnhandledMessage(message, sender(), self)`. | Matches. |
| 3 | ADR 0036 "To be verified": the route by which Pekko Typed publishes `UnhandledMessage` | Section 1: `Behaviors.unhandled` -> `ActorAdapter.next` -> `ActorAdapter.unhandled` -> `Actor.unhandled` -> `eventStream.publish`. | Answered (no longer unknown). |
| 4 | ADR 0034 P7: "It is never sent as a `DeadLetter`" | U1 to U3: no `DeadLetter` event; the stream carries only `UnhandledMessage`. (`DeadLetterListener` logs it as a dead-letter-style line, which is a log, not an event.) | Matches for the event. |
| 5 | ADR 0034 (P6, line 139): "Other unhandled signals are dropped" | U4: an unhandled `PostStop` publishes nothing. (`ActorAdapter.scala:226`: `case _: Signal =>` does nothing.) | Matches. |
| 6 | ADR 0034 X4 cites `TT/ActorContextSpec.scala:100`, `TT/AskSpec.scala:96` asserting the publication | U1 to U3 publish the event; the ask case (U3) publishes with `AskMsg(<temp actor>)` as message. The cited specs were not run here. | Matches the behavior; spec citation not re-checked. |
| 7 | ADR 0036 EVT-1 item 1: "`UnhandledMessage` and `Dropped` are published to the `EventStream`" | `UnhandledMessage` and `Dropped` events are published for D1, D3 (and D4 shape). For D2 (Topic) subscribers of `Dropped` receive nothing; the event is a `DeadLetter` that wraps a `Dropped`. | Partly matches: D2 is not covered by the text. |
| 8 | ADR 0036 EVT-1 item 4: "`Dropped` ... restart stash overflow, with the actor as recipient and no sender" | D1: recipient = the restarting actor, sender = null, reason `Stash is full in [RestartSupervisor]`. | Matches. |
| 9 | ADR 0036 does not mention the other Typed `Dropped` sites | D2 (Topic, wrapped in `DeadLetter`), D3 (no routees), D4 (router stash, not reproduced). All use the 3-argument constructor: sender null. | Not covered by the ADR. |

## 5. Limits

One OS, one JDK, two runs per scenario. U5 and U6 are controls. The `adaptAndHandle` site (`ActorAdapter.scala:187`) and the group router stash overflow (D4) were not triggered. The cited specs of ADR 0034 were not executed.

Correction (2026-11-05, #976 slice 4): `A/event/DeadLetterListener.scala:43, 126-130` -> `42-43, 126-132` (`Dropped` is subscribed at line 42 and `UnhandledMessage` at 43; "was dropped" is line 128, `case _: UnhandledMessage` is 130 and "was unhandled" is 132). Every other citation was re-read and matched.
