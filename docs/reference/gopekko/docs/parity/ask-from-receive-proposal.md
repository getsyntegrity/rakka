# Ask from inside `Receive`: observed behavior and proposal (issue #675)

Status: **proposal, no decision taken.** No public API is added and the semantics of `Ask` are unchanged.
Base: `origin/develop` `c2c0b8809a223a6577a8d7468ad3e31d7fcad584`. Reference: Apache Pekko `v1.7.1`,
`9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`upstream/pekko`). Paths below are relative to `upstream/pekko/`; `AS` is
`actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala`, `ACI` is
`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/ActorContextImpl.scala`, `AA` is
`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/adapter/ActorAdapter.scala`.

## Behaviors compared

"Returns at once" below means the call does not block the caller; it does not mean the result is complete.

| Behavior | Pekko | goPekko | Class | Evidence |
|---|---|---|---|---|
| Result of the call | `ask` returns a pending `Future` without blocking the caller (`AS:346-347`, `internalAsk` `AS:372-380`). It is already failed only if the recipient had terminated (`AS:373-375`) or the timeout is not positive (`AS:377-378`) | `Ask` blocks the calling goroutine (`ask.go:79-106`, ADR 0015 D1) | Deliberate adaptation (API) | `TestAskAPI_E1` |
| Next message while a reply is pending | The actor is not suspended; the result is usually delivered with `pipeTo`, in typed with `ctx.ask` + `pipeToSelf` (`ACI:228-232,279-281`) | If `Ask` is called in `Receive`, the actor processes nothing until it returns | Observable semantic difference, caused by where the user calls `Ask` | E7; R1, R2 |
| Same, with a goroutine + `Self().Tell` | n/a | The actor keeps processing; the reply is a later message | Reachable with the existing API; no helper | R4; `ExampleAsk_fromReceive` |
| Ask to oneself | The mailbox is free to read the request. The `Future` completes if the actor handles it and replies to the sender; otherwise it fails at the timeout | The blocked actor cannot read its own request; only the end of `ctx` or a reply sent through the reply address ends the wait | Observable semantic difference | R1, R1b |
| Circular Ask (A asks B, B asks A) | Neither is suspended. Each `Future` completes if the other handles the request and replies, otherwise at the timeout | Both stay blocked until a reply is injected or `ctx` ends | Observable semantic difference | R2 |
| Timeout | `scheduleOnce` completes the promise with `AskTimeoutException` (`AS:725-738`); the temp ref is then stopped (`AS:739-742`). Nothing is sent to the recipient | `ErrAskTimeout` from the deadline of `ctx` | Adaptation (deadline from `ctx`, ADR 0015 D2) | `ask_timeout_internal_test.go` T1-T10 |
| Cancelling the wait | The caller cannot cancel a Scala `Future`. The promise is completed by a reply, by the timeout or when the temp ref is stopped (`PromiseActorRef.stop` `AS:653-655` completes it with `ActorKilledException("Stopped")`, `AS:708`) | `ErrAskCanceled` from `ctx` | Go-only addition | C1-C10, R1b, R3 |
| Cancelling the recipient's work | Not done: nothing is sent on timeout or stop (`AS:725-742`) | Not done: a request that was sent stays in the mailbox and is processed later. If `ctx` ends during `build` the request is not sent (`ask.go` `askArm`); Pekko always sends | Equivalent for timeout; the not-sent case is a goPekko difference | C6; R1b does not assert the request |
| Late reply | `DeadLetter` when the temp ref is already stopped (`AS:615-617`), plain dead letter when the promise was already completed (`AS:626-628`) | Dead-lettered by the endpoint | Equivalent | `TestAskLateReply_*` |
| Recipient stopped during the wait | Not detected until the timeout (no watch in classic ask) | Not detected until deadline or `ctx` | Equivalent | K6 |
| Recipient already terminated | Classic: `AskTimeoutException`, request sent to dead letters (`AS:373-375`). Typed: `java.util.concurrent.TimeoutException` (`actor-typed/.../scaladsl/AskPattern.scala:146-151`) | `ErrRecipientTerminated`, request dead-lettered | Open: error identity differs, #674 | K1-K8 |
| Asking actor stopped during the wait | Not blocked, so it terminates normally. With `pipeToSelf`, the callback runs on the thread that completes the `Future` and only sends `AdaptMessage` to `self` (`ACI:279-281`); if the actor is gone it is a dead letter | `Stop` is accepted, but `PostStop` runs only after `Ask` returns (R3); `Done` closes after `PostStop` by the order in `finalize` (`stop.go`), which R3 does not observe | Observable semantic difference | R3 (order of `PostStop`); inspection of `stop.go` (`Done`) |
| Where the response adapter runs | Typed `mapResponse` / `adapt` runs inside the actor, in its message loop (`AA:104-109`) | n/a: no adapter exists. In the README pattern the goroutine only calls `Tell`; the state change happens in `Receive` | Not comparable today; see decision 1 | `ExampleAsk_fromReceive` |
| Resources of the reply mechanism | The temp ref stops when the promise completes (`AS:739-742`) | The endpoint is never registered; no entry to remove; the timer is stopped | Equivalent outcome, different mechanism | C10, `TestAskFirstReply_EndpointIsNotRegistered` |

Not equated: returning a `Future` is not the same as blocking for its result, and a free mailbox does not by itself end a
self-Ask or a cycle: the `Future` completes only through a reply or the timeout. The Pekko-recommended non-blocking pattern
is `ask` + `pipeTo`/`pipeToSelf`; the nearest goPekko equivalent today is the hand-written goroutine in
[`pekko-actor/README.md`](../../pekko-actor/README.md), which the actor does not supervise.

## Classification

- **Defects against the approved contract:** none found. ADR 0015 D1 and the `Ask` doc comment already state the blocking
  behavior, and E7 pins it.
- **Deliberate, documented adaptation:** blocking `Ask`; mandatory deadline; `ErrAskCanceled`.
- **Capability that needs a design decision:** a first-class ask + `pipeToSelf` (issue #675). ADR 0015 D7 lists
  `Context.Ask` / `AskFrom` as deferred.
- **Not part of this proposal:** error identity for a terminated recipient (#674), `StatusReply` (#681), flow control (#685).

## Option C as a proposal (not a decision)

| Option | Shape | Pros | Cons |
|---|---|---|---|
| A. Document only | Keep the README recipe, the example and R4 | No API; no new contract | Every user re-writes the goroutine; no lifecycle tie to the actor |
| B. `PipeToSelf` | A function that runs `fn` in a goroutine and delivers `adapt(result)` to `Self` | Generic; mirrors `pipeToSelf` (`ACI:279-281`); not tied to `Ask` | Starts goroutines; needs rules for stop, restart and panic in `fn` |
| C. `AskFrom` on top of B | `Ask` in a goroutine + `adapt` + delivery to `Self`; mirrors typed `ctx.ask` (`ACI:228-232`) | One call for the common case; deadline stays mandatory | New public API; ADR 0015 D7 must be amended |

Recommendation: C, implemented through B's internals. It is a recommendation only. Until the Owner decides, option A is in force.

Decisions the Owner must take before any implementation:

1. **Where `adapt` runs.** Inside the actor, in the mailbox loop, like Pekko typed (`AA:104-109`): the goroutine delivers
   the raw outcome and `Receive` (or a hook) adapts it, so `adapt` may read actor state. Or in the goroutine: simpler, but
   `adapt` then must not touch actor state and diverges from Pekko. The goroutine option is not decided.
2. **Stop and restart of the asker.** Dead letter for the reply, as in Pekko, or a `ctx` tied to the actor's life that
   cancels the wait on stop. Whether a restart keeps or drops replies of the previous incarnation.
3. **Panic in `fn` or `adapt`.** Recovered and reported as an error message, reported to the supervisor, or left to crash.
4. **Ownership of `ctx`.** Who creates it, whether the actor's stop cancels it, and whether the deadline remains mandatory (D2).
5. **Full bounded mailbox.** The reply is dead-lettered (`Tell` never blocks); say so in the contract (#685).
6. **Concurrency.** A limit on goroutines per actor or none, and whether replies carry a correlation id.
7. **ADR 0015 D7.** Amend it to turn `AskFrom` from deferred into scheduled, keeping the ADR's current policy of registering no automatic Watch on the target. Nothing here is accepted.

## Limits of the evidence

- R3 accepts a `Stop` while `Receive` is inside `Ask`, ends the wait by cancellation (so it does not depend on a reply)
  and checks in the log the order `ask-start`, `ask-return`, `post-stop`. It proves that `PostStop` runs after `Ask` and
  the `Receive` that called it returned. It does not observe `Done`: that `Done` closes after `PostStop` is derived from
  reading `finalize` (`stop.go`), and an early close of `Done` would not be detected by R3. Whether another lifecycle
  test pins that order was not verified.
- R1 and R2 prove the blocking by value: the injected reply is the only way `Ask` can return the exact number, and the
  test logs `release` before replying. They do not prove an absence of progress over time.
- R4 holds its far deadline in a field to keep the test short; the README pattern creates the `ctx` per request and is
  executed by `ExampleAsk_fromReceive`.
- No Linux or CI run was made for this change.
