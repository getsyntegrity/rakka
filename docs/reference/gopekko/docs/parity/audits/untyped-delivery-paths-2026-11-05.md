# Audit: untyped message delivery paths in gopekko

- Date: 2026-11-05. Evidence only; this document recommends nothing and changes no ADR or "To be verified" state.
- Code state: `develop` at `53cb678` (modules `pekko-actor`, `pekko-actor-typed`, `pekko-serialization`, `pekko-persistence`, `pekko-projection`, `pekko-observability`). Method: code reading with `grep` for `any`, `interface{}`, type assertions, type switches, `reflect` and `unsafe`, plus one compile snippet in a scratch module outside the repository (nothing committed).
- Question (ADR 0036, amendment of 2026-11-05, "To be verified"): through which untyped routes can a value of a type other than the behavior's message type reach that behavior? In Pekko 1.7.1 a wrong-class message is a failure (`ClassCastException`), not an `UnhandledMessage` (oracle `evt-1-unhandled-dropped-2026-11-05.md`, scenario U6).

## 1. Summary

The message type is a Go type parameter: `Ref[T]`, `envelope[T]`, `Context[T]`, `Receive(ctx, msg T)`. A value of another type cannot be sent through `Tell`, `TellFrom`, `Ask`, `WatchWith` or a reply ref; the compiler rejects it (snippet in section 3). The only untyped routes in the repository are the `Port` of a cell (`any`, by design), the persistence payload types (`any`, as Pekko's `Any`), and read-only information (`DeadLetter.Msg`, `FailedMessage`). None of them delivers to a `typed.Behavior`'s message handler. Several Pekko-side features that create untyped routes do not exist in gopekko yet.

## 2. Routes

`A` = `pekko-actor/actor/`, `T` = `pekko-actor-typed/typed/`, `P` = `pekko-persistence/persistent/`.

| # | Route | File:line | Can a wrong type reach a behavior? | What happens today |
|---|---|---|---|---|
| 1 | `Ref[T].Tell(ctx, msg T)`, `TellFrom[T](ctx, target Ref[T], msg T, sender Handle)` | `A/ref.go:20, 54-60, 65` | No: the parameter is `T`. | The wrong type does not compile. |
| 2 | Mailbox `envelope[T]` (`msg T`, `sender Handle`) | `A/mailbox.go:23-26` | No. | n/a. |
| 3 | `Ask[T, R]`, `build func(Ref[R]) T`, `askEndpoint[R]`, `askTarget[T]` (`target.(*ref[T])`) | `A/ask.go:52, 104`; `A/ask_endpoint.go:51, 132-134` | No: the reply ref is `Ref[R]`. The assertion `target.(*ref[T])` is on the handle kind (a user-implemented `Ref` or a test double fails), not on a message. | A target that is not a native ref is rejected with `ErrInvalidHandle`. |
| 4 | `WatchWith(h, msg T)`: `laneItem[T].msg T`; `sameMessage(a, b any)` compares two registrations with `reflect` | `A/watch.go:48, 121, 161-170` | No: the message is `T`. The `any` comparison is only to detect a conflicting registration. | A conflict returns the registration outcome (`regConflict`); no delivery of another type. |
| 5 | `Port.Post(msg any)` -> `PortHandler[T].ReceivePort(ctx, msg any)` (`laneItem.payload any`) | `A/port.go:40, 61, 67-72, 115-123`; `A/watch.go:56, 509` | Depends. The `Port` is obtained only by the actor through `AttachPort`; whoever holds it can post any value. It is not the behavior's `T`: `ReceivePort` is a separate method. `typed.interpreter` implements neither `AttachPort` nor `ReceivePort` (`T/interpreter.go`), so a Typed behavior cannot receive from a Port. | If the current instance does not implement `PortHandler`, the message is published as a `DeadLetter` once (`A/port.go:116-119`). Code reading. |
| 6 | `persistent` implementation of `ReceivePort`: `msg.(SnapshotResult)` | `P/snapshot_ops.go:141-146` | Depends: any value posted to the port. | A value that is not a `SnapshotResult` is ignored: `return nil` (no failure, no dead letter, no unhandled). Code reading. |
| 7 | `Persist(event any)`, `PersistAll([]any, handler func(event any))`, `ReceiveRecover(event any)`, `ReceiveRecoverContext(ctx, event any)`, `ReceiveSnapshot(ctx, metadata, state any)` | `P/persistent.go:78, 140, 148, 324-330, 544-551, 565-575, 638-645` | Not a message route. Events and snapshot state are `any` as in Pekko's `Any`; the type is whatever the serializer returns, and the behavior chooses with its own type switch. | A panic in the handler is a failure of that event (`applyRecovered`, `P/persistent.go:638-645`); an error is returned. No unhandled event. Code reading. |
| 8 | Serialization: `Registry.Serialize(v any)`, `Deserialize(s) (any, error)`, `SerializerFor(v any)`, `spi.Serializer` (`ToBinary(v any)`, `FromBinary(...) (any, error)`) | `pekko-serialization/serialization/registry.go:35, 59-73, 76, 104`; `.../spi/serializer.go:44-46` | Not a message route: nothing in the repository sends a deserialized value to an actor mailbox. | A failure is a `*ReadError`. |
| 9 | `DeadLetter{Msg any}`; `FailedMessage.Value() (any, bool)`; `PanicError.Value any` | `A/deadletter.go:23`; `A/supervision_hooks.go:47-52, 76`; `A/supervision.go:118` | No: output and diagnostic data only. | n/a. |
| 10 | Internal assertions and switches on actor roles (`PreStarter[T]`, `PostStopper[T]`, `Supervisor[T]`, `TerminatedHandler[T]`, `PortHandler[T]`, `restartable`, `suspendable`) and on behavior kinds (`switch v := b.(type)`) | `A/supervision_restart.go:181-508`; `A/supervision_failure.go:95-330`; `T/interpreter.go:126, 170`; `T/signal.go:66` | No: they select a capability of an actor or of a behavior, not a message. | n/a. |
| 11 | `reflect` | `A/spawn.go:30-31`; `A/watch.go:153-170` | No: nil detection of a handle and equality of `WatchWith` messages. | n/a. |
| 12 | `unsafe` | no use in these modules | n/a | n/a |

Not present in the repository (so no route exists): a receptionist, routers, `Topic`, an `EventStream`, message adapters, `transformMessages`, Classic-to-Typed adapters or `unsafe` upcasts (ADR 0033 D12 decides them one by one; none is implemented), remoting and cluster. `typed.Behavior[T]` handlers are typed (`func(*Context[T], T)`).

## 3. Check run outside the repository

A scratch module outside the repository (`replace` to the local `pekko-actor`) with:

```go
var r actor.Ref[string]
r.Tell(context.Background(), 42) // wrong type
```

`go build` fails: `cannot use 42 (untyped int constant) as string value in argument to r.Tell`. No other route was run: routes 5 and 6 need a `Port`, which only the actor obtains, and no existing test posts a foreign type to it (read).

## 4. Findings vs ADR 0036

ADR 0036 (amendment of 2026-11-05): "a wrong type is treated as a failure, not as unhandled" for any untyped delivery route; status To be verified.

| Route | Wrong type possible | Current behavior | Pekko (failure, not unhandled) |
|---|---|---|---|
| 1 `Tell`/`TellFrom` | no (compile error) | not applicable | not applicable (the error is earlier) |
| 2 mailbox | no | not applicable | not applicable |
| 3 `Ask`, reply ref | no | not applicable | not applicable |
| 4 `WatchWith` | no | not applicable | not applicable |
| 5 `Port.Post` / `ReceivePort` | depends (any value posted by the holder of the Port) | to a non-`PortHandler` instance: `DeadLetter`; to a `PortHandler`: the handler gets `any` and decides | does not match a failure in the non-`PortHandler` case (dead letter); not a behavior route |
| 6 persistent `ReceivePort` | depends | a foreign type is ignored (`return nil`) | does not match a failure; not a behavior route |
| 7 events, snapshot state | not a message route | the behavior's own switch; a panic fails that event | not applicable (Pekko's `Any`) |
| 8 serialization | not a message route | `*ReadError` | not applicable |
| 9, 10, 11 | no | not applicable | not applicable |

Summary of the comparison: no route lets a wrong-typed value reach a `typed.Behavior` handler, so the "failure, not unhandled" rule has no case to apply to today. The two places that take `any` (routes 5 and 6) are runtime ports, not behaviors; in both, a foreign type is not a failure (dead letter, or ignored).

## 5. Limits

Code reading at one commit; routes added later (receptionist, routers, `Topic`, message adapters, `transformMessages`, adapters, remoting) are not covered because they do not exist yet. Routes 5 and 6 were not run.
