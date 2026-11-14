# ADR 0010: Actor API style and the Pekko semantics it keeps

- Status: proposed. Partly accepted: the local watch increment of 2026-10-10 (section "Accepted for the local watch increment"), and, for **supervision only**, the Classic model (owner answer H2 of 2026-10-14, recorded in [ADR 0017](0017-supervision.md): semantics (b) and D3 below); everything else stays proposed.
- Date: 2026-10-08
- Stage: 1

## Feature

The public Go API for defining, spawning, messaging, supervising, watching and stopping actors. This ADR separates two decisions that the feasibility survey had merged (assumption A1):

- **(a) API style:** what an actor, a reference and a message look like in Go.
- **(b) Semantics:** which Pekko behavior (classic or typed) goPekko keeps, and therefore which Pekko API is the judge's oracle.

Surveying the classic API (`migration/stage-1/feasibility.md` §1) did not approve it for goPekko.

## How Pekko solves it

Pekko ships two APIs over one runtime.

| | Classic (`org.apache.pekko.actor`) | Typed (`org.apache.pekko.actor.typed`) |
|---|---|---|
| Actor | Class with `receive: PartialFunction[Any, Unit]` | `Behavior[T]`, a function returning the next behavior |
| Reference | `ActorRef`, untyped | `ActorRef[T]` |
| Reply address | Implicit `sender()`; `forward` keeps it (`actor/…/actor/ActorRef.scala:164`) | Explicit `replyTo: ActorRef[R]` in the message |
| Default supervision | Restart on `Exception`; Stop on init failure, kill, death pact (`actor/…/actor/FaultHandling.scala:210-223`) | Stop (`docs/typed/fault-tolerance.md:5-11`) |
| Supervisor | Parent's `supervisorStrategy`, `OneForOne` / `AllForOne`, `Escalate` | `Behaviors.supervise` wraps the child's behavior |
| Lifecycle | `preStart`, `preRestart`, `postRestart`, `postStop` methods | `PreRestart`, `PostStop` signals |
| Watch | `context.watch`; `Terminated` as a message (`docs/actors.md:411-443`) | `context.watch`; `Terminated` as a signal; `watchWith` maps it to a `T` |

## How GoAkt solves it

- Package: `reference/goakt/actor`
- Concept: an `Actor` interface with `PreStart(*Context) error`, `Receive(*ReceiveContext)` and `PostStop(*Context) error` (`actor/actor.go:49-97`). Messages are `any`, references are `*PID`, the receive context exposes `Sender()` and `Forward()`.
- Differences that matter for parity:
  - `ReceiveContext.Ask` blocks the actor until the reply (`actor/receive_context.go:389`).
  - Some stop paths run `PostStop` concurrently with `Receive` and drop queued messages instead of dead-lettering them (`actor/actor.go:73-83`). Pekko's G3, G5 and I3 (`docs/judge.md`) would fail.
  - The default supervisor stops on panic (`supervisor/supervisor.go:294`).

## Evidence

- Benchmark: none. No runtime exists.
- Judge: none yet. The judge contract (`docs/judge.md`) is proposed; the API choice decides which Pekko API the Pekko harness drives.
- Complexity: estimated from the sketches below, not measured.
- Idiomatic in Go: discussed per alternative.

## Alternatives for (a), API style

The same counter in each style: `Increment` adds one, `Get` replies with the count.

- **A1, classic-shaped (untyped).** `Receive(ctx actor.Context, msg any)` with a type switch; the reply goes to an implicit sender (`ctx.Reply(c.n)`); `ref.Tell(Increment{}, actor.NoSender)`; `actor.Ask(ctx, ref, Get{})` returns `any`.
- **A2, typed behaviors (functional, as Pekko typed).** `func counter(n int) actor.Behavior[CounterMsg]` returns `actor.Receive(func(ctx, msg) actor.Behavior[CounterMsg] { … })`; `Increment` returns `counter(n + 1)`, `Get` replies through `m.ReplyTo` and returns `actor.Same[CounterMsg]()`.
- **A3, typed references with struct actors (recommended).** Shown in full in the next section; its signatures are built by the harness.

| | A1 classic-shaped | A2 typed behaviors | A3 typed refs, struct actors |
|---|---|---|---|
| Type safety | None at the send site; `any` everywhere | Compile-time per protocol | Compile-time per protocol |
| Ergonomics in Go | Familiar to Akka users; type switches on `any` | Closures returning closures; allocation per message; unfamiliar | Plain struct with methods and a sealed message interface |
| Semantic fit with the oracle | Direct with classic | Direct with typed | Either; set by (b) |
| Extensibility (envelope, metadata, interceptors) | Envelope fields on the context | Behavior wrappers | Envelope fields on the context; hooks as optional interfaces |
| Runtime complexity | Lowest | Highest: behavior swapping and signal plumbing | Middle: generics on `Ref[T]` and `Context[T]` |

## The recommended API, by operation (A3)

**References.** `Ref[T]` is the typed handle used to send. Every `Ref[T]` is also an `actor.Handle`, the non-generic identity used by operations that accept any actor (`Stop`, `Watch`, `WatchWith`). `Handle` is a design choice, not a language requirement: methods of concrete and generic types may declare their own type parameters (Go 1.27, checked below), so `Stop` could be generic. It is chosen because it needs no type argument at the call site and works as a map key. What the language does forbid is a type parameter on an **interface** method, so `Ref[T]` and `Spawner`, both interfaces, cannot offer generic operations: `Spawn`, `Ask`, `AskFrom` and `Forward` are package functions.

```go
type Handle interface{ ID() ID }
type Ref[T any] interface {
	Handle
	Tell(ctx context.Context, msg T)
}
```

**Definition and construction.** A protocol is a sealed interface; an actor is a struct with `Receive`. A returned error or a recovered panic is a failure for the supervisor.

```go
type CounterMsg interface{ counterMsg() }
type Increment struct{}
type Get struct{ ReplyTo actor.Ref[int] }

func (Increment) counterMsg() {}
func (Get) counterMsg()       {}

type Counter struct{ n int }

func (c *Counter) Receive(ctx *actor.Context[CounterMsg], msg CounterMsg) error {
	switch m := msg.(type) {
	case Increment:
		c.n++
	case Get:
		m.ReplyTo.Tell(ctx, c.n)
	}
	return nil
}
```

**Spawn, Tell, Ask.** `Spawn` is a function, not a method, so it can be generic; its first argument is an `actor.Spawner`, implemented by the system (top-level actors) and by every `*Context[T]` (children). The factory runs again on every restart, so state never survives one. Every send takes a `context.Context`, which carries the envelope metadata (ADR 0008); inside an actor, `*actor.Context[T]` is that context. It is created per delivery, is valid only during the call it was passed to, and has no exported fields. After the call returns, a send made with it carries system metadata and no ask correlation (so a tenant-scoped target rejects it), its other operations do nothing, and each use is published as a `ContextMisuse` event (owner decision, 2026-10-08, X1). A zero `Context[T]` is expired. `Ask` reads the context's deadline and arms it on the runtime's clock, so the judge's controlled mode can fire it.

```go
counter, err := actor.Spawn(sys, "counter", func() actor.Actor[CounterMsg] { return &Counter{} })
counter.Tell(ctx, Increment{})

// Outside an actor: blocks until the reply or the context's deadline (required).
n, err := actor.Ask(ctx, counter, func(r actor.Ref[int]) CounterMsg { return Get{ReplyTo: r} })

// Inside Receive only: never blocks; the reply or the failure comes back as a message.
// Called from a hook it panics with ErrAskFromHook, which fails the actor (an initialization failure from PreStart).
actor.AskFrom(ctx, counter, func(r actor.Ref[int]) CounterMsg { return Get{ReplyTo: r} },
	2*time.Second, func(n int, err error) ShopMsg { return CountResult{N: n, Err: err} })
```

**Ask completion.** Each runtime ask creates a one-shot completion endpoint and a request correlation token, both owned by the runtime and carried in the internal envelope, independently of the payload's `ReplyTo`. Timeout, cancellation, reply and rejection race through one atomic terminal transition. A guard rejection completes the ask with `ErrRejected`. **A reply that arrives after the ask completed is published as a dead letter**, as Pekko does (`AskSupport.scala:614-628`, judge I5); it neither changes the result nor reaches any actor. The `ReplyTo` that `Ask` and `AskFrom` hand out **is** the endpoint: a send to it only completes that ask, once. It is not a delivery to an actor, and neither a `ReplyTo` nor the correlation can make any message internal (ADR 0008, decision D6): the `AskFrom` mapped result is a guarded application message to the asker, with the asker's metadata; a reply sent to a `ReplyTo` that is an ordinary `Ref` is an application message. The endpoint is a `Ref` and so a `Handle`, but `Watch`, `WatchWith` and `Stop` on it do nothing and are published as `ContextMisuse`. The ask reply carries no metadata and is not guarded; a responder that serves several tenants must not answer a stored `ReplyTo` while processing another tenant's message. That risk is documented, tested (E13) and not enforced by the runtime.

**Forward.** Forwarding the request being processed preserves its reply address (it travels in the message) and its ask correlation. **The correlation is transferred once**: the first `Forward` made while processing an asked message carries it, even when the forwarded message differs from the received one (the usual case, a continuation for another protocol). A second `Forward` in the same turn, a `Forward` after the turn, and a plain `Tell` carry none, inherit only the metadata, and their rejection does not complete or fail the ask (judge F1, F2). `Forward` is not available through a captured or expired context.

```go
func Forward[U, T any](ctx *Context[T], to Ref[U], msg U)
```

`U`, the protocol of the target, comes first so a caller can name it alone. Go does not infer `U` when the message value is narrower than the target's protocol: a value narrowed by a type switch, or a literal of a concrete message type, gives `type Get of m does not match inferred type CounterMsg for U`. Write `Forward[CounterMsg](ctx, target, m)`, or pass the received `msg` variable itself, which infers.

There is no implicit sender: the reply address travels in the message. (Note, 2026-10-24: [ADR 0032](0032-local-sender-envelope.md), partly accepted by the Owner, decides an *explicit, optional* sender in the envelope for Classic `sender()` and `forward`. It keeps this rule: nothing is implicit, `Tell` is unchanged, and `ReplyTo` stays the typed reply address. It does not amend or accept anything here.)

```go
case Relay:
	actor.Forward[CounterMsg](ctx, c.replica, m.Inner) // first Forward of the turn: carries the ask correlation
```

**Lifecycle, supervision, restart and stop.** Hooks are optional interfaces. When an actor does not implement one, the runtime does what Pekko's default does: `PreRestart` stops the children and calls `PostStop`, and `PostRestart` calls `PreStart` (`docs/actors.md:463-492`). Supervision belongs to the parent, as in classic. **The supervision signatures in the sketch below (`PreRestart(..., failed *WorkMsg)`, a non-generic `actor.Strategy`, `OneForOne(func(err error) actor.Directive)`) are superseded by the proposed signatures of [ADR 0017](0017-supervision.md) (`FailedMessage`, `Strategy[T]`, a decider with the parent's context and the failing child); nothing else of the sketch is accepted by this note.**

```go
func (w *Worker) PreStart(ctx *actor.Context[WorkMsg]) error { return w.open() }

// failed is nil when the restart was not caused by a message (e.g. a sibling's failure).
func (w *Worker) PreRestart(ctx *actor.Context[WorkMsg], cause error, failed *WorkMsg) { w.close() }
func (w *Worker) PostRestart(ctx *actor.Context[WorkMsg], cause error) error         { return w.open() }
func (w *Worker) PostStop(ctx *actor.Context[WorkMsg])                               { w.close() }

func (p *Pool) SupervisorStrategy() actor.Strategy {
	return actor.OneForOne(func(err error) actor.Directive {
		if errors.Is(err, ErrBadInput) {
			return actor.Resume
		}
		return actor.Restart
	})
}

ctx.Stop(worker) // worker is a Ref[WorkMsg], accepted as a Handle; or ctx.StopSelf()
```

**Watch and termination.** `Terminated` arrives through an optional handler, or mapped into the protocol with `WatchWith`. `Terminated` from a plain `Watch`, and the `WatchWith` message, are runtime deliveries with an internal provenance that application code cannot create (ADR 0008, "Provenance", decision D6). They reach the watcher even when it called `Watch` or `WatchWith` from `PreStart` and tenancy is required. The `WatchWith` message keeps that provenance although its value belongs to the application protocol: the value is captured at registration, goes only to the registrant, exactly once unless the watcher unwatches or is terminating, and the same value sent with `Tell` is an ordinary, guarded application message. A watched actor's termination with neither is a failure of the watcher (death pact), as in both Pekko APIs. **Registering again for the same subject with a different termination message fails the watcher** (`ErrWatchConflict`, kind `IllegalState`), as Pekko does (`DeathWatch.scala:125-131`, judge I6); the same message is a no-op. `Terminated` has unexported fields and no constructor: application code cannot build one, and a protocol interface that does not declare it cannot carry it.

```go
ctx.Watch(worker)
func (p *Pool) OnTerminated(ctx *actor.Context[PoolMsg], t actor.Terminated) error { /* … */ }

ctx.WatchWith(worker, WorkerGone{ID: id}) // delivered as a PoolMsg
```

## Accepted for the local watch increment

Approved by the owner on 2026-10-10 for `Watch`, `WatchWith`, `Unwatch` and `Terminated` between actors of one System, as implemented in `pekko-actor/actor/watch.go`. This is a bounded acceptance: it does not accept the rest of this ADR (style A3 as a whole, supervision, ask, restart, hierarchy), and it is not a claim of parity with Pekko.

**Accepted**

- `Context.Watch(h)`, `Context.WatchWith(h, msg)` and `Context.Unwatch(h)` have no return value. A call that cannot be honoured panics with a typed cause; the existing `protect` turns it into the failure and stop of the watcher, so it never escapes the actor nor stops the System.
- `Terminated` has unexported fields and no constructor (D9). The optional handler is `TerminatedHandler[T]` (`OnTerminated(ctx, Terminated) error`). `WatchWith` delivers its `T` to `Receive`.
- A plain `Terminated` with no `OnTerminated` is a death pact: the watcher stops with a cause that wraps `ErrDeathPact`.
- D6, with the typed cause `ErrWatchConflict`: a second registration for the same subject with a different termination message, or a plain `Watch` over a `WatchWith` and the reverse, fails the watcher. A repeat with the same message is a no-op. **Equality of `WatchWith` messages:** `==` when both values can be compared without a panic (so `reflect.Value.Comparable` is true for both, dynamic types included); otherwise they are never the same message. A `T` holding a slice, map or func therefore cannot repeat a registration: call `Unwatch` first. Pekko compares with Scala `equals`; Go has no equivalent for every `T`, and failing loudly is the conservative choice.
- **Handles.** **Superseded on 2026-10-25 for a handle of another System, as a design: see "Amendment 2026-10-25" below. The code behaves as this bullet says until DTH-01 ([#785](https://github.com/getsyntegrity/gopekko/issues/785)) is implemented.** (Implementation note, 2026-10-26: DTH-01 is integrated by PR #1207, `cad059c`; for a handle of another System the code follows the amendment. See its Implementation note.) A handle of this System whose actor has terminated is a normal subject. A valid reference of this System that has no actor behind it is treated as an actor that is already dead: `Terminated` is delivered (Pekko answers a watch of a dead reference with `existenceConfirmed = false`; goPekko's `Terminated` carries no such flag). A nil handle, a handle of another System, a zero ID or a `Handle` that is not an actor reference is rejected with `ErrInvalidHandle` and is never confused with a dead actor.
- **Identity of a registration.** Each registration has a generation, unique in the watcher. A notification for a registration that was withdrawn (`Unwatch`) or replaced by a newer one, even for the same actor, is discarded; so is one for a watcher that is stopping. `Unwatch` also discards a `Terminated` already queued for the subject.
- **Delivery.** `Terminated` and the `WatchWith` message are internal items of a FIFO lane next to the user queue, not a new system queue. **Placement: amended on 2026-10-22, see "Amendment 2026-10-22" below; the original sentence ("An item is handled after every user message queued before the notification and before any queued after it") is superseded.** It never runs synchronously inside `Watch`.
- **Difference from Pekko in where Terminated is placed.** **Superseded on 2026-10-22 by the amendment below** (the text is kept there, verbatim). The part that is not superseded: the rulebook amendment (DEV-5, `migration/stage-1/watch-test-port/AMENDMENTS.md`) was approved by the owner as a record of the deviation and of the limit of evidence at that date.
- **Order on termination.** `PostStop`, then in one critical section `closed`, the pending messages and the watchers' snapshot, then the dead letters of the pending messages, then the notification of each watcher, then the withdrawal of the actor's own registrations, then `Done`. So `PostStop` precedes any `Terminated`, and `Done` closed implies every notification is already queued (not delivered).
- **Locks.** No cell lock is held while another cell is locked or while user code runs.
- **No dead letters.** A runtime item that is never delivered (the watcher stopped first, or `Unwatch`) is dropped silently. Pekko publishes it as a suppressed dead letter; goPekko has no such event.
- A watcher that stops is removed from the registrations of the actors it watched.
- Self-watch, and any registration call after a stop was accepted, are no-ops.

**Amendment 2026-10-22: where `Terminated` is placed (ADR 0030 D8)**

- **Authorization.** Given by the Owner through an explicit message in the working session (Claude Code), not a GitHub comment, on 2026-10-22. The words, literally: "Autorizo modificar la colocación anterior documentada en ADR 0010 para migrar el comportamiento de Pekko. Esta autorización cubre implementación, pruebas, actualización documental, commit y push a los PRs existentes. No autoriza merge ni aceptación global del ADR 0030." Nothing else is amended or accepted by that sentence. (At that moment ADR 0030 stayed proposed; the Owner accepted it later the same day, D1 to D8 including D8, see its Status.)
- **What it replaces, and only that.** Two statements of the list above: the placement sentence of "Delivery" and the whole bullet "Difference from Pekko in where Terminated is placed". The text replaced, verbatim: "An item is handled after every user message queued before the notification and before any queued after it." and "Pekko enqueues `Terminated` at the tail of the watcher's user mailbox when the watcher *processes* the death notification, so a user message that arrived after the death but before that processing is handled before it. goPekko places it at the moment of the notification (see Delivery): the same message is handled after it. Both are legal interleavings of an asynchronous protocol, but a test that builds the first one on purpose (DeathWatchSpec, \"discard Terminated when unwatched between sysmsg and processing\") cannot be reproduced; its port keeps the clause \"Unwatch called inside a handler discards a queued Terminated\"."
- **New decision.** The death notification is a system message, as `DeathWatchNotification` in Pekko (`Mailbox.scala` `processMailbox`: `invoke`, then `processAllSystemMessages`; `DeathWatch.scala:99-115`). (1) A pending notification takes no capacity of a bounded mailbox and reserves no position. (2) The actor processes it between two turns, never inside a handler: before the next step and when an activation ends. (3) On processing, a notification whose registration was withdrawn (`Unwatch`) or replaced is dropped; otherwise the `Terminated`, or the `WatchWith` message, is enqueued at the **tail** of the mailbox as it is at that moment, behind every user message and runtime delivery already there, or, if a bounded mailbox is full, goes to dead letters. (4) A message that arrived after the death but before the processing is therefore handled **before** the `Terminated`. The unbounded mailbox follows the same rule.
- **Evidence.** Pekko built from the pin (`docs/adr/0030-pekko-repro/BoundedTerminatedRepro.java`, scenarios D-9 and D-10; outputs in ADR 0030 E16). The scenario that DEV-5 called not reproducible, "discard Terminated when unwatched between sysmsg and processing", is now reproduced: ADR 0030 scenario D-8 (`UnwatchBeforeTheNotificationIsProcessedTakesNoSlot`).
- **Contracts and tests changed.** The tests that fixed the superseded placement now expect Pekko's order, with the Pekko output quoted in each: `TestPendingRuntimeDeliveriesAreOrderedAgainstUserMessages` (`[m3 m4 T m5]` to `[m3 m4 m5 T]`), `TestWatchWithMessageGoesToTheTailWhenItsNotificationIsProcessed` (`[3 99 5]` to `[3 5 99]`), `TestTwoWatchWithDeliveriesGoToTheTailInNotificationOrder` (`[3 98 4 99 6]` to `[3 4 6 98 99]`) and `TestAfterAResumeTheContinuationComesBeforeTheLaneAndTheCommands` (`[cont terminated port:p msg]` to `[cont port:p msg terminated]`). The `barrier` helper of `watch_api_test.go` (one message told right after `Done`) and two tests (`TestMessagesSentBeforeDeathAreHandledBeforeTerminated`, `TestWatchingAgainWhileATerminatedIsQueuedCoalescesIntoOne`) assumed the superseded placement without saying so; they were made deterministic, because the order between a message told by another goroutine and the processing of a system message is not defined by Pekko.
- **Status.** The code of this amendment is in PR #686, accepted with ADR 0030 D8 and integrated with it; `develop` has the new placement from that integration. The rulebook record DEV-5 is not edited here (the rulebook is read-only inside a loop); its limit of evidence no longer applies to this scenario.

**Amendment 2026-10-25: watching an actor of another local System (GOV-15 option (b))**

- **Authorization.** Given by the Owner through an explicit message in the working session (Claude Code), not a GitHub comment, on 2026-10-25. The words, literally: "Apruebo como diseño: - Las enmiendas de ADR 0010, 0015 y 0024 presentadas en el consolidado de #1204, con las correcciones de #1206. - La enmienda explícita de ADR 0017, conservando el texto histórico y las referencias GOV-15 #1065 y DTH-01 #785. - Actualizar el README únicamente al entregar DTH-01. - El portador B con el defer definido en Q5, conservando los supuestos de ejecución y las pruebas como criterios obligatorios de implementación. - El diseño de GOV-51. Esta aprobación no certifica código implementado ni pruebas futuras."
- **What it replaces, and only that.** The whole bullet "Handles" of the list above (it stays in place, marked superseded). The text replaced, verbatim: "**Handles.** A handle of this System whose actor has terminated is a normal subject. A valid reference of this System that has no actor behind it is treated as an actor that is already dead: `Terminated` is delivered (Pekko answers a watch of a dead reference with `existenceConfirmed = false`; goPekko's `Terminated` carries no such flag). A nil handle, a handle of another System, a zero ID or a `Handle` that is not an actor reference is rejected with `ErrInvalidHandle` and is never confused with a dead actor." The bullet "Identity of a registration" and the other bullets are not replaced; the bullet "A watcher that stops is removed from the registrations of the actors it watched" is extended by the second new bullet.
- **New decision, bullet "Handles" (replaces the bullet above).** **Handles.** `Watch`, `WatchWith` and `Unwatch` accept a native handle of this System or of another System of the same process. Three operations are distinguished, and none of the first two retains anything. (1) Obtaining the owning System from a native handle: it reads the System that the handle already holds, whether that System is running or already terminated; it consults no registry, creates no reference that the handle does not already hold and registers nothing, so it adds no retention. (2) Asking that owning System whether the actor is still active: a read-only lookup by ID in the registry of that System, made under that System's own lock, which stores nothing and adds no retention; an ID that the System does not hold means "no active actor", never "invalid handle". (3) Treating a valid handle of a retired actor as dead: `Watch` and `WatchWith` deliver `Terminated` (or the `WatchWith` message), and `Unwatch` is silent, with no notification and no error. This third operation stores no more than any watch stores: the registration, which for a subject of another System also holds the owning System until the registration ends, and the queued-termination entry until the `Terminated` is taken; both belong to the watcher and are removed when the notification is processed and taken. The owning System of a handle is resolvable if and only if the handle is non-nil, is not a typed nil, has a non-empty `ID()`, is a handle issued by a System of this package (a `Ref`, or a handle returned by `Child`, `Children`, `Parent` or `ChildFailure.Child`), and reports a non-nil owning System. The owning System is the System that issued the handle, whether it is running or already terminated. A handle whose actor has terminated is a normal subject. A valid handle that has no active actor behind it is, for `Watch` and `WatchWith`, treated as an actor that is already dead (Pekko answers a watch of a dead reference with `existenceConfirmed = false`; goPekko's `Terminated` carries no such flag). A nil handle, a zero ID, a `Handle` that is not a native handle of a System of this package, a handle whose owning System cannot be resolved, and the reply endpoint of an Ask (not an actor reference, ADR 0015 D8) are rejected with `ErrInvalidHandle` and are never confused with a dead actor. No exported API and no registry of Systems is added.
- **New decision, new bullet after "Handles".** **Subjects owned by another local System.** The rules of this section for the identity of a registration, the discard of stale notifications and the order on termination apply unchanged to a subject owned by another System of the same process. A watcher that stops is removed from the registrations of the actors it watched. For a subject of the same System this is done before `acceptStop` returns, as today. For a subject of another System the removal is handed to the executor of the subject's System as the last step of accepting the stop, after the watcher's own activation has been handed over to its own executor when one was needed. When the stop is accepted the removal has been handed off, not completed, and it may be issued after the watcher has finished. The removal does not wait for the subject to terminate: it progresses provided the executor of the subject's System runs the work it is given (assumptions E1 and E4 of the design document `cross-system-watch-design.md`, section 3.1: the executor eventually runs every function it is given, and the call to `Execute` happens before that function starts); the termination of the subject is another path that clears the same link. What is retained, by whom and for how long: a registration of a subject of another System keeps the owning System reference until the registration ends, that is, until `Unwatch`, the processing of the subject's death, or the stop of the watcher; the subject's entry for the watcher keeps the watcher's cell, and through it the watcher's System, until the first of the removal handed to the executor of the subject's System, the recheck made by `Watch` after it linked, `Unwatch`, and the termination of the subject. Between the acceptance of the stop and that first event the subject may still try to notify the watcher; a notification that a subject sends to a stopping or finished watcher is discarded (Pekko `DeathWatch.scala:109`). This covers the termination of the subject, the termination of the watcher and the shutdown of either System. `Done`, `Ask` and `Stop` do not change. Remoting and cluster are not part of this decision. This extends the acceptance of 2026-10-10; it does not replace it.
- **Evidence.** Pekko `watch` and `watchWith` accept any `InternalActorRef` (`DeathWatch.scala:35-65`); a watch of a dead or empty reference gets `existenceConfirmed = false` and `Unwatch` of one is ignored (`ActorRef.scala:694-699, 745-751`); a terminating watcher sends no `Terminated` (`DeathWatch.scala:109`); release by `Unwatch` is asynchronous (`DeathWatch.scala:165-180`). No Pekko spec that watches across Systems was found. The design, the lock inventory and verdicts and the demonstration of the carrier are in `docs/parity/cross-system-watch-*.md` (draft PRs #1201 to #1206), with each claim marked READ, EXECUTED or NOT PROVEN. The only executed evidence is the existing same-System test suite under the race detector and a language-semantics check of `defer` outside the repository; the Pekko probe `migration/stage-1/watch-cross-system-probe/` was written, reviewed natively and is not compiled and not executed.
- **Accepted with it (Owner, same message).** Carrier B for the eventual release with the `defer` of Q5 (the foreign hand-off is registered with `defer` right after the `c.mu` section of `acceptStop` and stays the last thing that runs), keeping the executor assumptions E1 to E10 and the tests as mandatory criteria of the implementation: the two Q4 tests, the five Q5 tests (T-Q5a to T-Q5e) and the ID-uniqueness test across Systems.
- **Contracts and tests changed (at implementation, not now).** `pekko-actor/actor/watch_api_test.go:544` and `pekko-actor/actor/watch_internal_test.go:345` (case "foreign system", currently expecting `ErrInvalidHandle`); `pekko-actor/README.md` lines 65-67, only in the delivery of DTH-01; the markers of ADR 0017 and the sentence of ADR 0015 point here. `Done`, `Ask` and `Stop` do not change.
- **Status.** Accepted by the Owner on 2026-10-25 **as a design**. **Not implemented**: no code, no test of the cross-System behavior and no run of the Pekko probe exist. The implementation is DTH-01 ([#785](https://github.com/getsyntegrity/gopekko/issues/785)) and DTH-02 ([#827](https://github.com/getsyntegrity/gopekko/issues/827)), which stay blocked until the documents are merged and their own readiness criteria are checked. This amendment does not accept any other ADR and does not claim parity.
- **Implementation note (2026-10-26, reconciliation; not a new acceptance).** The "Not implemented" statement above describes 2026-10-25. DTH-01 was integrated by PR #1207 as `cad059c177b37f62c27b8a2e89a2826972817f18`: `Watch`, `WatchWith` and `Unwatch` resolve a native handle of another System in its owning System, the release of a stopping watcher at foreign subjects is handed off with the Q5 `defer`, the two "foreign system" test cases now expect acceptance, and `pekko-actor/README.md` states the new rule. The Q4, Q5 (T-Q5a to T-Q5e) and ID-uniqueness tests exist. The tree of `cad059c` equals the candidate tree `ab6b53741e1e71d67054b6a8ccce0aee6ebeeefa` that was reviewed natively (one lens) and on which the author ran the local gates; the earlier three-lens review and the independent verifier cover the tree of `71b3bab`. Lock order rests on reading and sampling, not a detector. Still pending: DTH-02 ([#827](https://github.com/getsyntegrity/gopekko/issues/827)) evidence (tests T1 to T7) and the Pekko probe, which is neither compiled nor executed. No parity is claimed. Matrix: `docs/parity/gov-48-dth-02-reconciliation-2026-10-26.md`, section 11.

**Deferred (still proposed)**

- The envelope, metadata, tenancy, guards and the provenance rules of ADR 0008: the lane items carry none of that.
- Supervision, restart, children and the parent's special notification (S13, S14, `ChildFailed`), and a suspended watcher: no system queue exists, so the stash rules of Pekko's `Watch`/`Unwatch` messages have no equivalent.
- Remote and cluster watchers, `addressTerminated`, `existenceConfirmed`, a bounded mailbox policy for `Terminated`, and the E-scenarios of milestone 1.
- Replacing the lane by a general system queue: a later ADR must decide it with restart and supervision.

**Amendment 2026-11-04: `Kill` and `PoisonPill` are included (Owner decision)**

- **Authorization.** Given by the Owner on 2026-11-04 (author: Owner, pablogore). The words, literally:
  > Principio rector: "siempre pekko como mirada funcional, solo si en Go algo se debe hacer de tal o cual forma pero funcionalmente respetamos el comportamiento de apache pekko". Toda decisión respeta funcionalmente a Apache Pekko. Solo se aceptan diferencias de forma propias de Go. Ninguna diferencia de comportamiento observable se acepta, ni siquiera como provisional.
  >
  > 1. Kill y PoisonPill quedan INCLUIDOS en goPekko, como adaptación Go de los mensajes auto-recibidos de Pekko Classic. La línea "Excluded for now: … PoisonPill/Kill" de ADR 0010 deja de aplicarles. Esto ratifica lo ya entregado para Kill (#1112, #778, #816, #1031) y lo decidido para PoisonPill el 2026-11-02 (D3-1).
  > 2. Kill: se procesa en su posición del mailbox. Hace fallar al actor con ErrActorKilled por el camino normal de supervisión. El default decider lo detiene.
  > 3. PoisonPill: se procesa en su posición del mailbox. Todo lo encolado antes se procesa; después el actor se detiene. No llega a Receive y no es una falla. Lo encolado después sigue el contrato normal de stop/dead letters. Es distinto del Stop inmediato de System.Stop/Context.Stop (D3-2).
  > 4. Forma Go aceptada: ambos se envían con funciones de System y Context por el lane ordenado existente. Ref[T] y Handle no ganan métodos. Cada export se justifica en su PR (ADR 0011, EXP-1) (D3-6).
  > 5. Divergencias funcionales vigentes. NINGUNA queda aceptada; cada una se corrige y tiene dueño:
  >    a. Un PoisonPill no entregado (actor detenido, stop aceptado o mailbox acotado lleno) debe publicar SuppressedDeadLetter, como Pekko. Hoy no publica nada. Dueño: #871. #770 no se cierra hasta que esto esté entregado. Esto reemplaza D3-7 (2026-11-02), que lo aceptaba como gap provisional.
  >    b. Tell(actor.Kill) y Tell(actor.PoisonPill) sobre un Ref[any] deben comportarse como ref ! Kill y ref ! PoisonPill. Hoy entregan el marcador como mensaje común. Se abre una issue nueva.
  >    c. Los dead letters de Kill y PoisonPill encolados al detenerse deben respetar el orden del mailbox, como Pekko.
  > 6. Esto no declara paridad completa de Kill ni de PoisonPill. La evidencia sigue las reglas de ADR 0033 D13.
- **What it replaces, and only that.** In "Consequences", the item "Excluded for now" no longer applies to `PoisonPill`/`Kill`. The original text stays in place, marked superseded for those two names; every other exclusion in it is unchanged. It also **replaces D3-7 (2026-11-02)**, which accepted an undelivered `PoisonPill` as a provisional port gap; that acceptance is withdrawn.
- **Open divergences, none accepted.** (a) An undelivered `PoisonPill` (stopped actor, accepted stop, or full bounded mailbox) publishes nothing today; it must publish `SuppressedDeadLetter` as Pekko does. Owner: #871. **#770 is not closed until this is delivered.** (b) `Tell(actor.Kill)` / `Tell(actor.PoisonPill)` on a `Ref[any]` delivers the marker as an ordinary message; it must act as `ref ! Kill` / `ref ! PoisonPill`. Tracked by a new issue. (c) Dead letters of `Kill` and `PoisonPill` queued at stop must keep mailbox order.
- **Implementation note (2026-11-11, #871 slice 2; not a new acceptance): divergence (a) is delivered.** The text above is not rewritten. An undelivered `PoisonPill` now publishes `SuppressedDeadLetter(PoisonPill, sender, deadLetters)` to the event stream, and no `DeadLetter`, in the three situations the oracle observed ([`suppressed-dead-letters-2026-11-10.md`](../parity/oracles/suppressed-dead-letters-2026-11-10.md)): an actor already stopped (DL1 [006], DL1b [004]), a full bounded mailbox (DL7 [019]), and a `PoisonPill` still queued when the stop is accepted (DL8 [013], DL8c [013]). A `PoisonPill` told after the stop was accepted is covered by a Go test only (scope scenario S9), not by the oracle. The classification is one funnel, `System.publish` (slice 1, PR #1528); slice 2 removes the guard of `postLane` and keeps the queued `PoisonPill` in `takePortDeadLocked`. Tests: `suppressed_deadletter_poison_test.go`, `poison_pill_test.go`; scope `docs/scopes/871-suppressed-dead-letters.md`. (b) (#1320) and (c) (#1321) stay open. **#770 stays open until its own PR cites this one**; no parity claim is made (ADR 0033 D13).
- **Status.** `Kill`: delivered (#816; end-to-end evidence #1031). `PoisonPill`: in delivery (#770, PR #1317).
- **No full parity claim** for `Kill` or `PoisonPill`. Evidence follows ADR 0033 D13.

**Amendment 2026-11-04 (b): `ActorSelection` and `Identify` are included (Owner decision)**

- **Authorization.** Given by the Owner on 2026-11-04 (author: Owner, pablogore), in the message that also granted G2 of #1286 and #1289. The words that concern this ADR, literally (the message continues between the principle and point 3, marked `[…]`; the inner quote marks of the principle are normalized):
  > Principio: "toda decisión funcional debe ser tal cual apache pekko". Solo se aceptan diferencias de forma propias de Go.
  >
  > […]
  >
  > 3. ActorSelection e Identify (GOV-09, #1022) quedan INCLUIDOS en goPekko, con el comportamiento de Pekko v1.7.1. La línea "Excluded for now: … actor selection by path" de ADR 0010 deja de aplicarse a ellos. #893 y #1092 se destraban por esta decisión.
  >
  > 4. Esto no declara paridad. La evidencia sigue ADR 0033 D13.
- **What it replaces, and only that.** In "Consequences", the item "Excluded for now" no longer applies to actor selection by path (`ActorSelection`). `Identify` was not named in that item; the Owner decision includes it as well. The original text stays in place, marked superseded for the selection item; every other exclusion in it is unchanged.
- **Not decided here:** the Go form of the API (names, receivers, exports), the lookup contract (the Owner approved K1 to K5 of the scope in PR #1289, `docs/scopes/1092-resolve-exact-ref-string.md` on that branch, as the contract #1092 consumes from #893; that approval is recorded in #1289, not here), remote and cluster selection (ADR 0033 D7 and D8 limit "100%" and the `actor` scope to local capabilities; remoting and cluster are out of scope, `docs/adr/0033-actor-typed-scope-and-architecture.md:41-42, 74`), and the delivery order. Each export is justified in its own PR (ADR 0011, EXP-1).
- **Status.** Not implemented. Stories that were waiting for this decision: #893 (exact local selection path), #1092 (resolve exact reference strings), #936 and #1024 (ask and pipe through a selection). #893 still depends on #852 and #1089.
- **No parity claim.** Evidence follows ADR 0033 D13.

## Signatures checked

The signatures and examples of this ADR are stubs in [`docs/adr/signatures/`](signatures/), built by `scripts/check-adr-signatures.sh` with the minimum Go of `go.work` (1.27.0). Results on go1.27.0:

| Claim | Result |
|---|---|
| `Spawn`, `Ask`, `AskFrom`, `Forward`, `Ref`, `Context`, `Spawner` and the examples above are well formed | builds, `go vet` clean |
| `Forward(ctx, target, m)` with `m` narrowed by a type switch | **rejected**: `does not match inferred type` |
| `Forward[CounterMsg](ctx, target, m)` and `Forward(ctx, target, msg)` | build |
| `Terminated{who: …}` or `Context[T]{ctx: …}` written outside the package | **rejected**: unexported field |
| A method of a concrete or of a generic type with its own type parameter | builds; under language 1.26 rejected: `generic method requires go1.27 or later` |
| A type parameter on an interface method | rejected: `interface method must have no type parameters` |

This shows the signatures are well formed and the language claims hold. It shows **no behavior**: the bodies are empty and no runtime exists. CI runs the script as the job `adr-signatures` (PR #9).

## Decision

Proposed, pending the owner (see Decisions for the owner):

1. **(a) Style A3:** `Ref[T]` with `Tell(context.Context, T)`, struct actors with `Receive(*Context[T], T) error`, optional hook interfaces, explicit `ReplyTo`, `Ask` with a required deadline outside actors and a non-blocking `AskFrom` inside them.
2. **(b) Semantics:** classic supervision (parent-owned strategy, `Resume`/`Restart`/`Stop`/`Escalate`, the classic default decider) and classic lifecycle hooks, with typed messaging (no implicit sender). **Accepted for supervision by the owner on 2026-10-14 (ADR 0017, H2); the rest of this ADR stays proposed.** The judge's oracle is the classic API for lifecycle, supervision, deathwatch, FIFO and dead letters; reply scenarios are written with an explicit reply address on both sides.

Pekko is the source for both halves; GoAkt is not adopted.

## Consequences

- **Runtime requirements:** an injectable executor and clock, with ask deadlines armed on the clock (the judge's controlled mode, `docs/judge.md` §5); recovery of panics into failures; an event stream that publishes dead letters, with a monotonic `DeadLettersPublished` counter so the judge can prove it received every one (`docs/judge.md`, "Recording").
- **Judge and ADR 0008:** events carry no sender (`docs/judge.md` §2); the envelope is built on `Context[T]`, not on a sender field.
- **Initial scope (milestone 1):** system, `Spawn`, `Tell`, `Forward`, `Ask`, `AskFrom`, the four hooks (`PreStart`, `PreRestart`, `PostRestart`, `PostStop`) with Pekko's defaults, supervision as in [ADR 0017](0017-supervision.md) (**updated 2026-10-14**: the target is the four Classic directives `Resume`, `Restart`, `Stop` and `Escalate` and both strategies, one-for-one and all-for-one, delivered in units; every capability not yet delivered is listed there as an explicit gap, none is replaced by another behavior), `Watch`, `Unwatch`, `WatchWith`, `Stop`, dead-letter subscription and `DeadLettersPublished`; plus what ADR 0008 and the judge require (metadata keys and capability, `Guard` and `Observer` ports, `ObserverStats`, `Shutdown`, `RequireGuards`, `WithExtension`, `WithAttribute`, `ContextMisuse`). Nothing else is exported.
- **Excluded for now:** `become`/`unbecome` (state lives in the struct), stash, timers and receive timeout, routers, actor selection by path (**superseded by the amendment 2026-11-04 (b)**), `PoisonPill`/`Kill`, dispatcher and mailbox configuration (ADR 0001). **Superseded for `PoisonPill` and `Kill` by the amendment 2026-11-04** (they are included); the rest of this item stands.
- **Revisit** after milestone 1 if generics on `Context[T]` make child spawning or adapters awkward in real code.

## Decisions for the owner

| # | Decision | Alternatives | Recommendation | Consequence of the recommendation |
|---|---|---|---|---|
| D1 | API style | A1 classic-shaped; A2 typed behaviors; A3 typed refs with struct actors | **A3** | Compile-time protocol safety with ordinary Go structs; the runtime carries generics in `Ref[T]` and `Context[T]`. |
| D2 | Reply address | Implicit sender (classic); explicit `ReplyTo` (typed) | **Explicit `ReplyTo`** | No hidden state in the context; forwarding preserves ReplyTo and explicit runtime correlation; the judge compares no sender field. With the implicit sender, every `Tell` needs a sender argument or goroutine-bound state, which Go cannot provide safely. |
| D3 | Default supervision | Restart (classic); Stop (typed) | **Restart (classic)**. **Decided by the owner for supervision on 2026-10-14 (ADR 0017, H2): Classic.** | The classic oracle and the surveyed test census apply unchanged. Choosing Stop is safer against restart loops but moves the supervision oracle to `actor-typed`, outside the surveyed scope, and the census must be redone. |
| D4 | Ask inside an actor | Blocking (as GoAkt); non-blocking `AskFrom` only | **Non-blocking only** | Prevents A-asks-B-asks-A deadlocks and keeps one-message-at-a-time throughput. Code that wants to wait must model the wait as a state. |
| D5 | Late reply to a completed ask | Dead letter (Pekko); silently ignored | **Decided by the owner (2026-10-08): dead letter** | Same observable behavior as Pekko; the judge compares it (I5). |
| D6 | `watch` or `watchWith` again with another message | Fail the watcher (Pekko); the last one wins | **Decided: fail the watcher** | Same as Pekko (I6); a silent overwrite hides a conflict. |
| D7 | `AskFrom` from a lifecycle hook | Refuse loudly; define its delivery | **Decided: refuse** (`ErrAskFromHook`, fails the actor) | A hook has no message whose metadata the result could inherit; under tenancy the result would be rejected and lost. Compile-time exclusion through a separate hook context type is a possible refinement. |
| D8 | Ask correlation on `Forward` | Only the first `Forward` of the turn; any message | **Decided: once, for the request being processed** | A fan-out or an audit message cannot fail the ask. |
| D9 | `Terminated` constructibility | Unexported fields, no constructor (Pekko `private[pekko]`); public | **Decided: not constructible** | A spoofed death notification cannot be built. |
