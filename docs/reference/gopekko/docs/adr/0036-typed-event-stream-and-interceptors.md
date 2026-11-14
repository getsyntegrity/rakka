# ADR 0036: Typed: unhandled and dropped messages (EVT-1) and interceptors

- Status: **Accepted** by the Owner on 2026-11-04. **Amended on 2026-11-05** (additive, [EVT-1 oracle](../parity/oracles/evt-1-unhandled-dropped-2026-11-05.md)): the `Dropped` behavior is stated per site and the path of `UnhandledMessage` is closed; see "Amendment (2026-11-05)" at the end. **Amended again on 2026-11-05 (c)** (additive, [oracle of interceptors](../parity/oracles/interceptors-2026-11-05.md) and [audit of untyped routes](../parity/audits/untyped-delivery-paths-2026-11-05.md)): the interceptor table is corrected and the "To be verified" about interceptor order, `transformMessages` and `withMdc`, and about untyped routes, is closed; see "Amendment (2026-11-05, c)" at the end. **Citation corrections of 2026-11-05 (d)**: one Pekko citation corrected in place; no decision changes. **Amended again on 2026-11-06 (e)** (additive): "`Dropped` without sender" is limited to the sites of the table and the row of `MessageBuffer.drop` is added; see "Amendment (2026-11-06, e)" at the end.
- Date: 2026-11-04
- Stage: 1
- Principle applied: ADR 0033 D14 (behave exactly as Pekko).
- Related: ADR 0033 (Consequences, event stream), ADR 0034 (P2, P7, X4, X6), ADR 0035 (contract 5, `Dropped`),
  `docs/parity/typed-owner-decisions-2026-10-27.md` (EVT-1, B-4).
- Placement: EVT-1 had no ADR of its own. This ADR is created with the next free number (0036).

## Context

ADR 0034 left `UnhandledMessage` (X4) and `Dropped` (X6) as deferred capabilities recorded through a module-local hook. It also left the
interceptor constructors (P2) as a deferred capability. Decision EVT-1 placed the stream in `pekko-actor` but did not state that Typed
publishes to it. This ADR closes those points.

## Decision

### EVT-1: publication to the EventStream

1. `UnhandledMessage` and `Dropped` are published to the `EventStream`, as in `pekko-actor`. *(Superseded by Amendment (2026-11-05) as to `Dropped`: what is published depends on the site.)*
2. The `EventStream` is the one in `pekko-actor`. Each export that Typed needs is justified in the story that introduces it (EXP-1).
3. The module-local hook of ADR 0034 P7 is no longer the final destination: once the stream exists, Typed publishes to it. X4 and X6 close at that point.
4. `Dropped` cases already fixed: restart stash overflow (ADR 0035 contract 5), with the actor as recipient and no sender.

Docs: [event bus](https://pekko.apache.org/docs/pekko/current/event-bus.html),
[logging](https://pekko.apache.org/docs/pekko/current/typed/logging.html).

### Interceptors

They are exposed as in Pekko, with the same semantics:

| Pekko | Go equivalent | Semantics kept |
|---|---|---|
| `BehaviorInterceptor` (`aroundStart`, `aroundReceive`, `aroundSignal`, `interceptMessageClass`) | interceptor type with the same three interceptions | a message that is not of the intercepted class reaches the inner behavior untouched; signals always go through `aroundSignal` (`InterceptorImpl.scala:95-104`) |
| `Behaviors.intercept(factory)(behavior)` | constructor with a factory | the factory creates one instance per actor; an interceptor with state is not shared *(qualified by Amendment (2026-11-05, c): a supervisor restart creates it again)* |
| `Behaviors.monitor(ref, behavior)` | constructor | each message of class `T` is also sent to `ref` and continues to the inner behavior *(Superseded by Amendment (2026-11-05, c).)* |
| `Behaviors.logMessages(behavior)` and `logMessages(logOptions, behavior)` | constructors | logs each message before invoking the inner behavior, with `LogOptions` *(Superseded by Amendment (2026-11-05, c).)* |

Nesting: an interceptor returned by the inner behavior is re-applied after each turn. If an `isSame` interceptor is already in the stack it
is not duplicated (`InterceptorImpl.scala:106-122`). *(Superseded by Amendment (2026-11-05, c).)*

**To be verified.** *(Superseded by Amendment (2026-11-05, c).)* The exact order in which nested interceptors are applied (which one sees a message or signal first), and `transformMessages` and
`withMDC`. To be confirmed with `BehaviorSpec`/`InterceptSpec` of `actor-typed-tests` and a differential scenario. Also to be verified: the exact
path through which Pekko Typed publishes `UnhandledMessage`. *(Superseded by Amendment (2026-11-05): the path is known.)*

Docs: [Behaviors API](https://pekko.apache.org/api/pekko/current/org/apache/pekko/actor/typed/scaladsl/Behaviors$.html),
[BehaviorInterceptor API](https://pekko.apache.org/api/pekko/current/org/apache/pekko/actor/typed/BehaviorInterceptor.html).

## Consequences

- The "deferred capability" status of P2 (interceptor constructors), X4 and X6 ends as each is delivered.
- Until the stream exists in `pekko-actor`, Typed keeps using the local hook and does not publish; no parity is claimed.
- A difference from Pekko on any of these points needs its own ADR (ADR 0033 D14.3).
- No change to code. It neither accepts nor modifies other ADRs.

## Amendment (2026-11-05): `Dropped` per site, path of `UnhandledMessage`

- Status: **Accepted** by the Owner on 2026-11-05 (text: "ok", accepting the proposal recorded here). Additive: the text above is unchanged; the phrases marked "Superseded by Amendment (2026-11-05)" are replaced by it.
- Principle applied: ADR 0033 D14. Oracle: [`evt-1-unhandled-dropped-2026-11-05.md`](../parity/oracles/evt-1-unhandled-dropped-2026-11-05.md), Pekko 1.7.1. "A0" below is its section 1 (publication map).

**`Dropped`, per site (D14).** goPekko behaves as Pekko 1.7.1 at each site:

| Site | Pekko 1.7.1 publishes | sender | recipient | Evidence |
|---|---|---|---|---|
| Restart stash overflow | `Dropped` event | noSender | the restarting actor | oracle D1; `Supervision.scala:121-124` |
| Group router with no routees | `Dropped` event | noSender | the router | oracle D3; `GroupRouterImpl.scala:152-153` |
| `Topic.Publish` with no subscribers | sent to `deadLetters`, which publishes a `DeadLetter` wrapping a `Dropped`; **no `Dropped` event** | `DeadLetter`: `deadLetters` (the inner `Dropped`: noSender) | `DeadLetter`: `deadLetters` (the inner `Dropped`: the topic) | oracle D2; `TopicImpl.scala:96`, `ActorRef.scala:736-743` |
| Group router stash full | To be verified (not reproduced: the listing arrived before the stash filled) | - | - | oracle D4; `GroupRouterImpl.scala:97-101` |

- "`Dropped` without sender" holds: it matches `noSender`, confirmed by the oracle (D1, D3; 3-argument constructor, `ActorRef.scala:624-630`). *(Superseded by Amendment (2026-11-06, e) as a general statement: it holds only for the sites of the table above.)*
- The "To be verified" about the publication path of `UnhandledMessage` is closed: see "Amendment (2026-11-05)" of ADR 0034 and A0 (`Behaviors.unhandled` -> `ActorAdapter.next` :166 -> `ActorAdapter.unhandled` :220 -> `Actor.unhandled` -> `eventStream.publish`).
- A message of the wrong class sent to a Typed actor produces a `ClassCastException`, an actor failure that goes through supervision, not an `UnhandledMessage` (oracle, U6). By D14, if goPekko has an untyped delivery route (`any`, adapters), a wrong type is treated as a failure, not as unhandled. Status: **To be verified** (those routes are not identified yet; this amendment does not search for them). *(Superseded by Amendment (2026-11-05, c): closed.)*
- Not covered: the group router stash overflow (above); `MessageBuffer.scala:268` (a Classic utility not used by `actor-typed`) *(covered by Amendment (2026-11-06, e), row `MessageBuffer.drop`)*; `ActorAdapter.scala:187` (`adaptAndHandle`, a message with no registered adapter), not executed.
- Still open from this ADR: the order of nested interceptors, `transformMessages` and `withMDC` (To be verified). *(Superseded by Amendment (2026-11-05, c) except MDC during signals, `BehaviorSignalInterceptor` and the `interceptMessageClass` bypass.)*

## Amendment (2026-11-05, c): interceptor semantics and untyped routes

- Status: **Accepted** by the Owner on 2026-11-05 (text: "si dale", accepting the proposal recorded here). Additive: the text above is unchanged; the phrases marked "Superseded by Amendment (2026-11-05, c)" are replaced by it.
- Principle applied: ADR 0033 D14. Evidence: oracle [`interceptors-2026-11-05.md`](../parity/oracles/interceptors-2026-11-05.md) (scenarios I1 to I7) and audit [`untyped-delivery-paths-2026-11-05.md`](../parity/audits/untyped-delivery-paths-2026-11-05.md). Pekko 1.7.1; `IMPL` = `InterceptorImpl.scala`, `MDC` = `WithMdcBehaviorInterceptor.scala`, `SUP` = `Supervision.scala`, `BI` = `BehaviorInterceptor.scala`.

**Interceptors: corrections to the table.** goPekko behaves as Pekko 1.7.1:

| Topic | Pekko 1.7.1 | Evidence |
|---|---|---|
| `logMessages` | Logs each message **and each signal** ("received signal"), before invoking the inner behavior. Replaces "logs each message". | I2c |
| Deduplication by `isSame` | Also applied **at start**, not only after each turn. The outer duplicate is dropped and the **inner one stays** (its `aroundStart` still runs). `isSame` is per interceptor: instance identity by default, same `actorRef` (`monitor`), equal options (`logMessages`), same throwable class (supervisors), any `withMdc`. | I2a to I2e; `IMPL:106-122, 144-148, 200-203`; `SUP:78-83` |
| Restart by supervision | Interceptors **inside** `supervise` are created again and see `PreRestart`; those **outside** survive and do not see it. The new one is created after the old child's `Terminated` (the restart waits for the child). | I3; `SUP:318-346`; `IMPL:37-41` |
| `monitor` | Sends the message to `ref` **before** the inner handler runs, even if the handler throws or returns unhandled; it does not receive signals. The order is asynchronous: the oracle shows only that the send happens before the handler finishes (`slow`). | I6; `IMPL:138-141` |
| Nesting order | The outer interceptor runs first before the call and last after it, in start, messages and signals. An interceptor that does not override `aroundSignal` forwards the signal. | I1, I3, I7; `IMPL:82-104`; `BI:84-85` |
| `transformMessages` | A message with no match is an `UnhandledMessage` with the **original** message and sender `deadLetters`. Two nested with **different** functions fail the start (`IllegalStateException`); the same instance twice is accepted. | I4a to I4c; `IMPL:226-241` |
| MDC (`withMdc`) | Present only inside the handler and cleared after it. With nesting the **outer** wins. Supervision logs carry it only if they run inside `withMdc`. | I5a to I5d; `MDC:51-82` (`aroundStart`; the loop that removes the nested `WithMdc` is at 60-75), `MDC:85-88` (`isSame`), `MDC:90-96` |

**Names.** `withMdc` is Pekko's name (the ADR wrote `withMDC`). The name in the goPekko API follows Go conventions and is not part of this decision; no ADR defines a naming convention for the Go names of Pekko API members yet.

**Closed "To be verified".** The order of nested interceptors, `transformMessages` and `withMdc` for messages are covered by the interceptors oracle (`interceptors-2026-11-05.md`). **Still To be verified:** MDC during signals (the source puts the static MDC only, `MDC:99-105`; not run), `BehaviorSignalInterceptor` (not run) and the `interceptMessageClass` bypass (not run).

**Untyped routes: closed.** The message type is a generic parameter, and no existing route lets a value of the wrong type reach the handler of a `typed.Behavior`; the compiler rejects it (`cannot use 42 (untyped int constant) as string value in argument to r.Tell`, audit section 3). The rule "a wrong type is a failure" stays in force (EVT-1 amendment above). The "To be verified" of that bullet is closed.
- **Requirement.** A receptionist, routers, `Topic`, message adapters, `transformMessages` and Classic-Typed interop do not exist yet. The PR that introduces any of them must audit its route again with the criterion of the audit `untyped-delivery-paths-2026-11-05.md` and link the result.
- **Outside this ADR.** `Port.Post(any)` and the persistence `ReceivePort` are decided by [ADR 0037](0037-port-as-a-go-adaptation.md) with the evidence of the audit [`port-and-persistence-receiveport-vs-pekko-2026-11-05.md`](../parity/audits/port-and-persistence-receiveport-vs-pekko-2026-11-05.md).

## Amendment (2026-11-05, d): citation corrections (#976 slice 4)

Every Pekko citation of this ADR was re-read from `upstream/pekko` at v1.7.1 / `9a54d12`. One was wrong and is corrected in place; no decision, claim or status changes. In the interceptor table (Amendment (c)), the "MDC (`withMdc`)" row cited `MDC:36-62, 85` for "`aroundStart` removes any nested `WithMdc`, the outermost wins". Lines 36-62 are the tail of the companion and the start of the class; `aroundStart` is `MDC:51-82` (the elimination loop at 60-75) and `isSame` is `MDC:85-88`. The other citations of this ADR matched as written.

## Amendment (2026-11-06, e): `Dropped` without sender only at the sites of the table; the row of `MessageBuffer.drop`

- Status: **Accepted** by the Owner on 2026-11-06 (literal text in the table below). Additive: the text above is unchanged; the sentence marked "Superseded by Amendment (2026-11-06, e)" is limited by it.
- Principle applied: ADR 0033 D14. Evidence: code reading of Pekko 1.7.1 (`ActorRef.scala:618-630`, `:739-743`; `MessageBuffer.scala:267-272`) and the reconciliation of #906. **The new row is not yet executed**: its oracle scenario is a condition of this amendment (point d).
- Why. The bullet "`Dropped` without sender holds" of Amendment (2026-11-05) can be read as a rule of the type. It is a fact about the three sites of the table, which all use the 3-argument constructor. The type has a `sender` field (`ActorRef.scala:622`) whose documentation says that `sender` is `noSender` only "when this message was sent without a sender" (`:618-620`).

**Decisions (D14).**

a. "`Dropped` without sender" holds **only** for the sites of the table above (restart stash overflow, group router with no routees, and the inner `Dropped` of `Topic.Publish`; the group router stash overflow keeps its "To be verified"). It is not a rule of the `Dropped` type and does not apply to a `Dropped` produced elsewhere.

b. New row of the table:

| Site | Pekko 1.7.1 publishes | sender | recipient | Evidence |
|---|---|---|---|---|
| `MessageBuffer.drop(id, reason, deadLetters)`: one `Dropped` per buffered message | sent to `deadLetters` (`deadLetters ! Dropped(msg, reason, ref, noSender)`); `deadLetters` wraps it in a `DeadLetter` (`ActorRef.scala:739-743`); **no `Dropped` event**, as for `Topic.Publish` | `Dropped`: **the sender stored with the buffered entry** (`ref`); `DeadLetter`: `deadLetters` (code reading: `DeadLetterActorRef.!` declares its implicit sender default as `this`, `ActorRef.scala:736`, so the sender is already `deadLetters` on entry; the branch `if (sender eq noSender) deadLetters` at `:742` gives the same value. The oracle scenario of point d observes it) | `Dropped`: `noSender`; `DeadLetter`: `deadLetters` | code reading `MessageBuffer.scala:267-272`, `ActorRef.scala:739-743`; **not executed** (point d) |

The order of the `Dropped` arguments is `(message, reason, sender, recipient)` (`ActorRef.scala:622`); `MessageBuffer.drop` passes the entry's `ref` in the **sender** position and `noSender` in the **recipient** position, the reverse of the three Typed sites.

c. The `Dropped` type of goPekko keeps both fields, `sender` and `recipient`, as Pekko's (accepted by the plan row O-12 and by #906). Each producer sets them as its site says. The sender that a goPekko producer stores is mapped by ADR 0032 D3, D4 and D6 when the producer has a sender (the `MessageBuffer.drop` row); D5 does not apply. `pekko-actor` has no `MessageBuffer` today (plan row U-02, "ABS", reconcile on demand): this row binds a producer only if one is ported.

d. **Condition before the story that implements `Dropped` ([#906](https://github.com/getsyntegrity/gopekko/issues/906)):** the oracle of `EventStreamSpec` (Slice E) contains a scenario that runs `MessageBuffer.drop` on Pekko 1.7.1 with a buffered entry that has a sender and one that has none, and records the `Dropped` and the `DeadLetter` that a subscriber of each receives. Until that oracle is merged the row above is a code reading and no implementation relies on it (ADR 0033 D13).

**What this amendment does not do.** It changes no code and none of the three sites. ADR 0035 contract 5 and ADR 0036 EVT-1.4 stay as written (they are about the restart stash, a site of the table). **Pointer, not a change:** ADR 0030 P-5 says that in v1.7.1 "only `MessageBuffer` emits `Dropped`"; the EVT-1 oracle shows three more producers in `actor-typed` (D1 to D3). That sentence of ADR 0030 is not edited here.

| Date | Owner decision (literal text) | Effect on this ADR |
|---|---|---|
| 2026-11-06 | "si — apruebo acotar 'Dropped sin sender' a los sitios de la tabla y agregar la fila de MessageBuffer.drop" | Points a to d above: the sentence is limited to the sites of the table, the row is added, the oracle scenario is a condition of #906. No code and no other decision changes. |
