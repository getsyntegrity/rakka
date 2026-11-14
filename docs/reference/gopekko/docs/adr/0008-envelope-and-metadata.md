# ADR 0008: Envelope and metadata

- Status: proposed. Nothing is accepted here yet; the local watch increment of 2026-10-10 uses runtime deliveries without the envelope (see "Not accepted by the local watch increment").
- Scope change (2026-10-17, Owner): goPekko has **no `pekko-tenancy` module**; tenant identification and authorization belong to
  the consuming application. This **replaces** only the tenancy requirements of this ADR: the `pekko-tenancy` module and its
  `TenantResolver` port, the tenant guard and the "Tenancy guard rules" (rules 4 to 6 as they concern tenants), `tenancy.Owner`,
  `actor.RequireGuards("tenancy")`, "tenancy required" and the tenant-mismatch flows, the trusted tenant key as the example of a
  trusted key, D2 (tenant granularity) and the per-tenant filtering of D5. **Not replaced and not accepted by this note:** metadata
  and its copy and immutability rules, correlation and trace propagation, runtime provenance and origin, generic guards and
  observers, `context.Context` handling, and D8 (a sealed capability for trusted keys), whose only decided consumer was the
  tenant key; whether any trusted key remains is for #194 to decide. The text below is kept as written and any tenancy in it is history.
- Date: 2026-10-08
- Stage: 1

## Feature

How a message carries metadata (tenant identity, trace context, caller data) from sender to receiver, and the ports through which `pekko-tenancy` and `pekko-observability` act on it without `pekko-actor` importing them. Tenancy is fail-closed.

## How Pekko solves it

- Module: `pekko-actor`
- Concept: it does not. The envelope is `Envelope(message, sender)` (`actor/src/main/scala/org/apache/pekko/dispatch/AbstractDispatcher.scala:35`) and has no metadata. Tracing in the Pekko ecosystem is done outside the runtime (bytecode instrumentation). There is no tenancy.
- Consequence: no Pekko oracle exists for this ADR. It is verified by its own specification with `go-specs` (`docs/judge.md` §7).

## How GoAkt solves it

- Package: `reference/goakt/remote`, `reference/goakt/actor`
- Concept: a `ContextPropagator` copies `context.Context` values into HTTP headers for remote calls only (`remote/context_propagator.go:47-56`). Locally, an asked actor's `ReceiveContext.Context()` ends when the asker stops waiting (`actor/receive_context.go:206-221`). There is no trust separation and no tenancy.

## Evidence

- Benchmark: none.
- Judge: the actor part of every scenario runs against Pekko to show that metadata does not change actor behavior (non-interference). The metadata rules below are the specification for the envelope scenarios.
- Complexity: one immutable map per envelope, one capability check per trusted write, the guard and observer calls per delivery.
- Idiomatic in Go: metadata travels in `context.Context`, as `log/slog` and OpenTelemetry already do.

## Alternatives

| | M1: metadata in `context.Context`, inherited on every send | M2: fields in each message type | M3: one actor system per tenant |
|---|---|---|---|
| How | The **registered** metadata of the sender's context is copied into the envelope as an immutable snapshot; the rest of the context (values, cancellation, deadline) is not retained; inside an actor the current message's snapshot is the context | Every protocol carries `TenantID`, `TraceID` | Isolation by process structure |
| Forgetting it | Impossible inside actors; outside, the guard rejects | One missed field leaks or loses identity | Impossible |
| Central fail-closed | Yes, in one guard | No | Yes |
| Shared actors across tenants | Yes | Yes | No |
| Cost | Snapshot copy at enqueue (unmeasured) | None in the core | One runtime per tenant |

**Recommendation: M1.** It is the only option that is both enforceable in one place and keeps shared actors. M3 stays available to deployments that want hard isolation.

## Decision

Proposed, pending the owner.

### Metadata: ownership, copy and immutability

- `actor.Metadata` is an immutable value. `With` returns a new value; nothing mutates a value already in an envelope.
- The core copies the sender's registered metadata into the envelope at enqueue time. The receiver sees that snapshot; later changes on the sender side do not reach it.
- The initial value set is immutable scalar values (strings, booleans and fixed-width numeric values) plus byte sequences copied on write and on read. Arbitrary pointers, maps, slices other than copied bytes, and user objects are not admitted (see "Unsupported values"). A shallow map copy is not an immutability guarantee. Additional value types require an explicit freezing/copy contract and tests before admission.
- Only registered metadata crosses the envelope. Do not retain the caller's arbitrary context value chain: it may contain mutable objects or request resources. Extensions translate supported trace context into registered immutable values and reconstruct receiver context from the snapshot.
- Keys are declared once by the module that owns them: `actor.NewKey[V](name)` for a caller-supplied key, `actor.NewTrustedKey[V](cap, name)` for a trusted one (it needs the capability). Names are unique across both classes.

**Unsupported values.** They are rejected **where metadata is written, never at `Tell`**, which returns nothing (ADR 0010). `V` is constrained to the admitted set (`MetaValue`: fixed-width scalars, strings, booleans and `Bytes`), so `NewKey[*int]` does not compile: `*int does not satisfy MetaValue`, built by the harness in `docs/adr/signatures/`. The typed write `Metadata.With(key, v)` is a generic method (Go 1.27, the repository minimum). For values that arrive untyped (an extension reading HTTP headers or a token), `Metadata.WithUntyped(name, v)` returns `(Metadata, error)` with `ErrUnsupportedValue`, before any envelope exists; the writer reports it, for example `tenancy.Resolve` returns it to the ingress code. Metadata is constructible only through these two, so `Tell` can never receive an unsupported value and has nothing to reject or report.

### Trusted versus caller-supplied

- **Trusted** keys (tenant identity, verified principal) can be **declared and written only with a capability** (owner decision, 2026-10-08, D8). `Capability` is a sealed interface: only the runtime implements it, and it hands one to each extension registered with `actor.WithExtension`, at start-up. `NewTrustedKey(cap, name)` and `Metadata.WithTrusted(cap, key, v)` return `ErrNoCapability` for a nil capability or one from another system. `WithUntyped(name, v)` writes caller-supplied keys only: a trusted name returns `ErrTrustedKey`, an unknown one `ErrUnknownKey`. Application code cannot obtain the capability, implement it (the harness builds this: `capability_forge`), or write a trusted key by name. If application code declares a caller-supplied key with a trusted name before the extension installs, the extension's `NewTrustedKey` fails, `Install` returns the error and `NewSystem` fails: loud, never open.
- **Caller-supplied** keys (baggage, request labels) can be written by anyone and are never used for authorization.
- A caller-supplied key cannot shadow a trusted one: names are unique across both classes.

### Tenant resolution and validation

- **Resolution** happens once, at ingress, outside actors: `pekko-tenancy` implements a `TenantResolver` port (HTTP header, token claim) and writes the trusted tenant into the context through its capability.
- **Validation** happens on every delivery, in a receive-side guard registered by `pekko-tenancy`, before `Receive` runs.
- Inside actors nobody resolves again: the tenant is inherited from the message being processed.

### Fail-closed, concretely

The rules are split so that the core enforces rejection without knowing what a tenant is.

**Core rules** (`pekko-actor`, generic for any guard; enabled with `actor.RequireGuards("tenancy")` at system start):

1. An application delivery that any guard rejects never reaches `Receive`. Runtime deliveries (`Terminated`, the `WatchWith` message, a death pact) are not given to guards (see "Provenance").
2. The rejection is observable: a `Rejected{Envelope, Reason}` event on the event stream (not a dead letter), a call to every observer's `OnRejected`, and an associated runtime `Ask` that completes with `ErrRejected` (ask failure kind `rejected` in the judge) when rejection is decided, using ADR 0010's one-shot correlation endpoint.
3. Guards execute synchronously before delivery, on the cell's serial execution path, and must be bounded, non-blocking and free of external I/O. Errors and recovered panics reject. The runtime supplies a budget context, checks its expiry before and after the call and rejects an expired result. Cancellation is cooperative: it cannot interrupt an arbitrary Go function. Do not launch an unbounded goroutine per delivery to simulate a hard timeout. A guard that never returns stalls the cell and is reported as a stuck runtime/harness failure; a hard execution bound is not promised by this SPI. Tenant resolution needing I/O belongs at ingress.
4. A system that requires a guard which is not registered fails `NewSystem`. A missing extension is never "open".

Guard scope: guards run for application deliveries to `Receive`. System messages (failure notifications, watch registrations, supervision directives, termination) are internal and not guarded.

**Tenancy guard rules** (`pekko-tenancy`, policy of its guard):

5. A missing, malformed or unknown tenant rejects. There is no default tenant.
6. An actor spawned with the attribute `tenancy.Owner` (a generic per-actor attribute, `actor.WithAttribute`, that the core stores and hands to guards without interpreting it) rejects messages from any other tenant.

### Provenance of runtime signals

Owner decision D6 (2026-10-08): internal runtime signals do not pass through the guards that apply to application messages. The provenance is created exclusively by the runtime and cannot be forged through the public API. `WatchWith` keeps that provenance although it delivers a value of the application protocol. A `ReplyTo` or an ask correlation must not let arbitrary messages be marked internal.

Without a rule, a `Watch` made from `PreStart` (system metadata, no tenant) would have its `Terminated` rejected by rule 5, and the watcher would never learn of the death or fail with the death pact, silently, only when tenancy is required.

- Every delivery has an **origin**, `Application` or `Runtime`, a field of the internal envelope. The runtime derives it from the delivery path; it is never a parameter, option, message field or extension capability, and no public function or type can express it (no exported function accepts an `Origin`; `InboundEnvelope` is sealed by an unexported method, so application code cannot implement one; the harness builds this: `envelope_forge`). `Tell`, `Ask`, `AskFrom` and `Forward` always yield `Application`. The capability that extensions use for trusted metadata keys (below) is unrelated and does not include it.
- **Runtime deliveries to an actor are exactly three:** `Terminated` for a plain `Watch`; the `WatchWith` message; and the failure of a death pact. Each goes only to the actor that registered its cause, exactly once per registration unless the registrant unwatches or is terminating (I2 in `docs/judge.md`), with a payload captured **at registration**: `Terminated`, or for `WatchWith` the value the watcher itself passed. No API lets the watched actor or any message change it. A payload that holds pointers shared with the watched actor can still be mutated through them: deep immutability is the registrant's responsibility (E10 uses a value payload).
- **`WatchWith` keeps `Runtime` provenance although its value is of the application protocol.** The value was chosen by the registrant, for the registrant. The same value sent with `Tell` by anyone is an `Application` message and is guarded: equality of value confers nothing.
- **A `ReplyTo` or a correlation never marks anything internal.** The ask completion endpoint (the `ReplyTo` that `Ask` and `AskFrom` create) is not an actor and receives no delivery: a send to it completes that one ask, once, and a later send is dead-lettered (judge I5), not delivered. What the asker's code then receives is application data: the `AskFrom` mapped result is an `Application` message to the asker, guarded, with the asker's metadata, because its content came from another actor. A `ReplyTo` that is an ordinary `Ref` receives an `Application` message. The correlation token sits in the internal envelope, is assigned by the runtime, is copied only by `Forward` for the request being processed, and no message, option or API carries one chosen by the application; a foreign or stale endpoint completes only its own ask (or nothing) and changes no origin.
- Guards are not called for runtime deliveries. Observers see them with `Origin() == Runtime`, and `InboundEnvelope` exposes `Origin()`.
- A runtime delivery carries the metadata of the registrant's context at registration, or system metadata if registered from a hook. In that case the handler runs without a tenant and anything it sends inherits none, so a tenant-scoped target still rejects it. Fail-closed is preserved for everything the application does with the signal.
- Not a bypass: application code cannot create a runtime delivery; the three kinds go to the registrant only, once, with a payload the registrant chose; and no value chosen by another actor is ever delivered with `Runtime` provenance (the aliasing caveat above aside). The residual case is a `WatchWith` handler that acts on the signal without a tenant check, which is the registrant's own code.

### Propagation

The rule: **an envelope inherits the metadata of the context it was sent from.** Inside `Receive`, that context is the current message's.

| Path | Metadata the receiver sees |
|---|---|
| `Tell(ctx, msg)` | The snapshot of `ctx` at enqueue |
| `Ask` from outside | The caller's metadata at enqueue, for the asked message. The reply carries no metadata and is not guarded (see ADR 0010, "Ask completion") |
| `AskFrom` inside `Receive` | The asker's current message; the mapped result is delivered to the asker with that same metadata. From a hook it is refused (`ErrAskFromHook`, ADR 0010 D7) |
| `actor.Forward[U](ctx, target, msg)` | The original sender's metadata, unchanged; preserves runtime ask correlation as defined in ADR 0010 |
| Routers (later stage) | Transparent: each routee gets the original envelope |
| Stash, then unstash (later stage) | The envelope is stored whole; unstash restores its original metadata |
| Dead letter | `DeadLetter` carries the original envelope with its metadata. A late reply to a completed ask is a dead letter of the reply value alone, with no envelope |
| `Terminated` (plain `Watch`) | The metadata of the watcher's context at `Watch` time; system metadata if called from a hook. Origin `Runtime`, no guard |
| `WatchWith` message | The registrant's metadata at registration; system metadata from a hook. Origin `Runtime`, no guard |
| `AskFrom` mapped result | The asker's metadata at the `AskFrom` call. Origin `Application`, guarded |
| Send to an ask `ReplyTo` | The endpoint is not an actor: the send completes the ask once. No delivery, no origin |
| Lifecycle hooks (`PreStart`, `PostRestart`, `PostStop`) | System metadata only: no tenant. Hooks do no tenant-scoped work |
| A `*Context[T]` used after its call returned | System metadata and no ask correlation; other operations do nothing; `ContextMisuse` is published (ADR 0010, X1) |
| `PreRestart` | The failing message's metadata |
| Supervision decision, restart, stop (internal) | System metadata; never a tenant |

### `context.Context`, cancellation and deadlines

- Only the registered immutable metadata snapshot crosses a `Tell`. Caller cancellation, deadlines and arbitrary context values do not. Construct a receiver context from this snapshot and the actor-owned processing context; retaining `context.WithoutCancel(senderCtx)` alone does not meet the ownership contract.
- `Ask` from outside requires a deadline on its context; the deadline bounds the wait for the reply, not the target's processing.
- A stop request, cancellation of in-flight work and completed termination are distinct. When the runtime accepts a stop request, it cancels the actor-owned processing context as a cooperative goPekko extension; the current `Receive` must still return before `PostStop` and termination notifications run. Cancellation does not forcibly interrupt I/O or user code. A handler that ignores it can delay termination indefinitely; shutdown may report its deadline exceeded but must not run hooks concurrently or claim the actor terminated.
- Cancellation caused by a pending stop is not supervised as a new restart. On restart, retire the old instance's processing context after its current handler returns and create a fresh context for the new instance. Caller cancellation never cancels the actor's processing context.
- The cancellation signal is goPekko-specific and uses its own spec oracle; Pekko parity continues to assert lifecycle ordering and non-overlap.

### Ports, in `pekko-actor/spi`

`Metadata`, `Key`, `ID`, the capability and the envelope views live in `pekko-actor/spi`, a leaf package that imports nothing else from goPekko. The root `actor` package imports `spi` and re-exports those types as aliases (`type Metadata = spi.Metadata`), so the ports never import the package that calls them.

```go
// Guard may reject a delivery. Registered by pekko-tenancy.
type Guard interface {
	Name() string
	OnReceive(ctx context.Context, env InboundEnvelope, attrs Attributes) error // non-nil error = reject
}

// Observer sees every lifecycle and message event and cannot change or reject anything.
// Registered by pekko-observability.
type Observer interface {
	OnEnqueue(env InboundEnvelope)
	OnProcessed(env InboundEnvelope, d time.Duration, err error)
	OnLifecycle(actor ID, event LifecycleEvent)
	OnRejected(env InboundEnvelope, reason error)
}
```

### Observer execution

Owner decision D7 (2026-10-08): observers are asynchronous, with a bounded queue, and never block actor processing. Observers can be slow or fail and they run for every message, so they must not be able to stall a sender or a cell.

- **Asynchronous.** Each observer has a bounded queue (default 4096 events) and one runtime-owned goroutine. Calls to one observer are serialized, so an implementation need not be thread-safe. They never run on the sender's goroutine or in the cell's turn, and cannot delay, reject or fail a delivery or an ask. Events produced on one goroutine reach an observer in order; there is no order across goroutines, so no per-actor ordering is promised unless the enqueue happens-before is established by the runtime.
- **Drops.** A full queue drops the **new** event (drop-newest); it never blocks and never evicts older events. Every drop increments the observer's `Dropped` counter.
- **Statistics are authoritative and conserved.** `System.ObserverStats(name)` returns a consistent snapshot, taken under the observer's lock, of monotonic counters: `Enqueued`, `Delivered`, `Dropped`, `Abandoned`, `Panicked`, `Slow`, `Disabled`. Every event offered to an observer ends in exactly one of `Dropped` (queue full, intake closed, or the observer `Disabled`) or `Enqueued`; every enqueued event ends in exactly one of `Delivered` (the call returned), `Panicked` (it panicked) or, at shutdown, `Abandoned` (still queued, or mid-call when the deadline passed). So `Enqueued + Dropped` is the number offered, and once `Shutdown` returns `Enqueued = Delivered + Panicked + Abandoned`. `Slow` describes a finished call and is not an outcome; `Disabled` is 0 or 1. Shutdown takes the queue under the same lock as the drain, so an event goes to one or the other, never both. E11 and E12 compare the counters with the number of events their script offered, not with themselves. `ObserverFault{Observer, Kind, Count, Err}` events (`Kind`: `Dropped`, `Abandoned`, `Panicked`, `Slow` or `Disabled`) on the event stream are a best-effort notification only, aggregated at most once per second per observer and kind; the stream can lose them.
- **Panic isolation.** A panic is recovered per call, in the observer's goroutine; the event is skipped and the panic reaches no actor, sender, other observer or the runtime. After 10 consecutive panics the observer is disabled, counted, and reported.
- **Duration.** Budget per call (default 50 ms), cooperative like the guards': checked after the call, it cannot interrupt. A call over budget counts as `Slow`. An observer that never returns stalls only its own goroutine; its queue fills and drops.
- **Shutdown.** `System.Shutdown(ctx)` first terminates the actors as Pekko's termination does, then stops the observers. Budget: the actors' termination may use until `deadline - drain`, where `drain = min(500 ms, 25% of the time to the deadline)`; with no deadline `Shutdown` has 5 s and the same split. Closing the intake never panics (a producer that races it counts a `Dropped`, nothing is sent on a closed channel). The observer's goroutine drains the queue until the deadline, and then **abandons** what is left. Events still queued are counted as `Abandoned`. An observer that is mid-call and never returns is abandoned too: shutdown does not wait for it past the deadline, its goroutine is leaked and reported, never killed (Go cannot). Events produced after the intake closes, including those of actors still running, are counted as `Dropped`. An observer receives no event after `Shutdown` returns, except from the call that was already running. The final `ObserverStats` stay readable after shutdown. A second `Shutdown` is a no-op.
- **Not promised:** delivery (at most once), completeness, order across observers or relative to guards, any hard time bound, and isolation from what Go cannot isolate: a data race on shared state, a fatal runtime error (for example a concurrent map write), `os.Exit` or an infinite loop that starves the process. `InboundEnvelope` is immutable but the message payload is shared, not copied: observers must treat it as read-only, which is not enforced.
- **Observers are never evidence or policy.** Not for authorization, tenancy, or any audit that must not lose events (that needs a guard or an explicit audit port, out of scope), and not for the judge: its event log comes from a harness-owned recorder with acknowledgements (`docs/judge.md`, "Recording").
- Costs and defaults are unmeasured.

`pekko-actor` defines these interfaces, the key registry, the attributes and the capability, and contains no tenant or tracing logic. `pekko-tenancy` and `pekko-observability` import `pekko-actor`, never the reverse, as `scripts/deps.allow` requires.

## Flow and rejections

```go
// Ingress: resolve once, then ask an actor.
ctx, err := tenancy.Resolve(r.Context(), r) // trusted tenant written by pekko-tenancy
if err != nil {
	http.Error(w, "unknown tenant", http.StatusUnauthorized)
	return
}
ctx, cancel := context.WithTimeout(ctx, 2*time.Second)
defer cancel()
item := r.URL.Query().Get("item")
order, err := actor.Ask(ctx, orders, func(rt actor.Ref[Order]) OrderMsg { return Place{Item: item, ReplyTo: rt} })
if err != nil {
	http.Error(w, err.Error(), http.StatusServiceUnavailable) // ErrRejected, timeout, …
	return
}
json.NewEncoder(w).Encode(order)

// Inside the orders actor: inventory receives the same tenant without any code here.
func (o *Orders) Receive(ctx *actor.Context[OrderMsg], msg OrderMsg) error {
	place := msg.(Place)
	actor.Forward[InventoryMsg](ctx, o.inventory, Reserve{Item: place.Item, ReplyTo: place.ReplyTo}) // preserves ask correlation
	return nil
}
```

| Case | Result |
|---|---|
| A background goroutine sends to a tenant-scoped actor with `context.Background()` | Rejected, reason `tenant missing`; `Rejected` published |
| Caller code declares a caller-supplied key named like the tenant key before the extension installs | The extension's `NewTrustedKey` fails, `Install` returns the error, `NewSystem` fails |
| An actor spawned with `tenancy.Owner` B receives a message with tenant A (if D2 binding is adopted) | Rejected by the tenancy guard, reason `tenant mismatch` |
| The tenancy guard panics | Rejected, reason `guard failed`; the panic value is in the `Rejected` event |
| Tenancy required, guard not registered | `NewSystem` returns an error |

## Not accepted by the local watch increment

The local watch increment (ADR 0010, "Accepted for the local watch increment") delivers `Terminated` and the `WatchWith` message as internal items of the actor's own lane. They have no metadata, no origin and no guards, because `pekko-actor` has no envelope yet. This accepts none of the envelope, the metadata, the tenancy rules, the guards or the provenance rules (D6) of this ADR: they stay proposed. The two requirements below are what this ADR will still have to add when the envelope exists, and the increment does not satisfy them: a `Terminated` that carries the metadata of the watcher's context at `Watch` time, and its exemption from the guards.

## Consequences

- ADR 0010's `Tell(context.Context, T)` signature follows from this ADR.
- The envelope family of the judge has a written oracle: the propagation table and the six fail-closed rules (judge relations T1 to T5 and A1).
- Per-message cost: registered metadata snapshot and copies of byte values per send; guard and observer calls per delivery. These costs are unmeasured.
- Out of scope: remote propagation (a later transport ADR), persistence of metadata, serialization of keys (ADR 0002).

## Decisions for the owner

| # | Decision | Alternatives | Recommendation | Consequence of the recommendation |
|---|---|---|---|---|
| D1 | Propagation model | M1 context; M2 message fields; M3 system per tenant | **M1** | Central enforcement and shared actors; every send takes a context. |
| D2 | Tenant granularity | Per message only; per actor only; both | **Both**: per-message tenant, optional actor binding | Shared actors stay possible; tenant-owned actors reject foreign tenants. |
| D3 | Deadlines across `Tell` | Not propagated; propagated (GoAkt-like for ask) | **Not propagated** | Matches Pekko; a slow caller cannot cancel work already accepted. Handlers that need a deadline get it in the message. |
| D4 | Rejected messages | Separate `Rejected` event; reuse `DeadLetter` | **Separate event** | Operators can tell a security rejection from a delivery failure; dead-letter scenarios stay Pekko-comparable. |
| D5 | Who sees dead letters and rejections | All system-level subscribers; filtered per tenant | **System-level, unfiltered** | Operators see everything; tenant-facing views filter in `pekko-tenancy`. |
| D6 | Guards and runtime provenance | Runtime provenance, guards not called; guards called and each decides on `Origin` | **Decided by the owner (2026-10-08):** internal runtime signals do not pass through the application-message guards; provenance is created only by the runtime and cannot be forged from the public API; `WatchWith` keeps internal provenance with a payload fixed at registration; a `ReplyTo` or an ask correlation cannot mark any message internal | One place enforces it; application code cannot create a runtime delivery, and a value from another actor is never delivered as internal. Cost: the exception lives in the core, which must be reviewed with care. |
| D7 | Observer delivery | Asynchronous bounded queue; synchronous with a budget | **Decided by the owner (2026-10-08): asynchronous, bounded queue, never blocks actors** | Senders and cells cannot be stalled by observability; events may be dropped, drops are counted in statistics, panics are isolated within what Go can isolate, and nothing about delivery is promised. |
| D8 | Writing trusted keys | A mandatory capability in the signature; a runtime check by key name only | **Decided by the owner (2026-10-08): a mandatory, sealed capability** | Application code cannot declare, write or overwrite a trusted key, by name or otherwise; the signatures and the harness enforce it. The tenancy extension is the only holder. |
