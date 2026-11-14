# Oracle: interceptors in Pekko Typed (Pekko v1.7.1)

- Date: 2026-11-05. Evidence only; this document recommends nothing and changes no ADR or "To be verified" state.
- Pekko `pekko-actor-typed` and `pekko-slf4j` 1.7.1 (tag `v1.7.1`), Scala 2.13.18, sbt 1.10.7, JDK Temurin 11.0.32+9, default fork-join dispatcher, logback-classic 1.3.14 with a pattern that prints the MDC keys `s`, `m`, `k`, `b`. The sbt project is outside the repository and is not committed. Two runs per scenario; the sequence of events was the same in both (only actor uids differ).
- Text under test: ADR 0036, section "Interceptors" ([ADR 0036](../../adr/0036-typed-event-stream-and-interceptors.md)): "They are exposed as in Pekko" with a table (`BehaviorInterceptor`, `intercept`, `monitor`, `logMessages`), the nesting paragraph and the "To be verified" about the order of nested interceptors, `transformMessages` and `withMDC`.
- `IMPL` = `actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/InterceptorImpl.scala`; `MDC` = `.../internal/WithMdcBehaviorInterceptor.scala`; `SUP` = `.../internal/Supervision.scala`; `BI` = `.../BehaviorInterceptor.scala`. In Pekko the MDC decorator is `Behaviors.withMdc` (ADR 0036 writes `withMDC`).

## 1. Harness (relevant parts)

Every line is printed with a global sequence number; actor handlers and interceptors print through the same counter.

```scala
class Tr(name: String, signals: Boolean = true) extends BehaviorInterceptor[String, String](classOf[String]) {
  override def aroundStart(ctx, target)   = { p(s"$name aroundStart before"); val r = target.start(ctx); p(s"$name aroundStart after"); r }
  override def aroundReceive(ctx, m, tg)  = { p(s"$name receive before '$m' ${mdc()}"); val r = tg(ctx, m); p(s"$name receive after '$m' ${mdc()}"); r }
  override def aroundSignal(ctx, sig, tg) = if (signals) { p(s"$name signal before $sig"); val r = tg(ctx, sig); p(s"$name signal after $sig"); r } else tg(ctx, sig)
}
def target() = Behaviors.setup[String] { ctx => val kid = ctx.spawnAnonymous(quietKid); ctx.watch(kid)   // "kid" stops it: a Terminated signal
  Behaviors.receiveMessage[String] { case "fail" => throw new IllegalStateException("boom"); case "stop" => Behaviors.stopped
    case "u" => Behaviors.unhandled; case "slow" => Thread.sleep(400); Behaviors.same; case m => p(s"TARGET message '$m' ${mdc()}"); Behaviors.same
  }.receiveSignal { case (_, s) => p(s"TARGET signal $s"); Behaviors.same } }
// I1  intercept(A)(intercept(B)(intercept(C)(target())))
// I2a the same interceptor instance twice; I2b monitor(ref, monitor(ref, target())); I2c logMessages(logMessages(target()));
//     I2d supervise(supervise(target()).onFailure[Throwable](restart)).onFailure[Throwable](stop); I2e a recursive behavior that returns monitor(ref, ...) every turn
// I3  intercept(OUT)(supervise(intercept(IN)(target())).onFailure[IllegalStateException](restart))
// I4a target().transformMessages[String] { case "ok" => ... }; I4b two transformMessages with different partial functions; I4c the same partial function instance twice
// I5a intercept(OUT)(withMdc(Map(s -> static), m => Map(m -> m))(target())); I5b withMdc(outer)(withMdc(inner)(target()));
//     I5c withMdc(supervise(target())); I5d supervise(withMdc(target()))
// I6  monitor(ref, target()) with "m1", "slow", "u" (unhandled), "fail"
// I7  intercept(A: signals)(intercept(B: default aroundSignal)(intercept(C: signals)(target())))
```

## 2. What the source expects (written before running)

| Scenario | Expectation from the 1.7.1 source |
|---|---|
| I1 | `preStart` calls `aroundStart`, which starts the nested behavior (`IMPL:82-85`); `receive` calls `aroundReceive` with a target that interprets the nested behavior (`IMPL:90-99`): outer `before` first, inner `after` first. `receiveSignal` always calls `aroundSignal` (`IMPL:101-104`); the default `aroundSignal` forwards (`BI:84-85`): every interceptor sees every signal. |
| I2 | After the start and after every turn, `deduplicate` replaces an interceptor whose `isSame` matches one already in the nested stack, keeping the inner one (`IMPL:106-122`). `isSame` is `eq` by default (`BI:92`); `MonitorInterceptor`: same `actorRef` (`IMPL:144-148`); `LogMessagesInterceptor`: equal `opts` (`IMPL:200-203`); supervisors: same throwable class (`SUP:78-83`). |
| I3 | `RestartSupervisor` restarts the `initial` behavior, which contains the inner interceptor's factory call (`IMPL:37-41`), so IN is created again; OUT is outside and survives. `PreRestart` is sent through `signalRestart` to the supervised (nested) behavior (`SUP:318-346`), so IN sees it and OUT does not. |
| I4 | `TransformMessagesInterceptor.aroundReceive` returns `Behaviors.unhandled` when the partial function does not match (`IMPL:238-241`), which the adapter turns into an `UnhandledMessage` (see the EVT-1 oracle). A second `transformMessages` with a different partial function throws `IllegalStateException` in `isSame` (`IMPL:226-236`); the same instance is allowed. |
| I5 | `MDC:aroundStart` removes any nested `WithMdc` (its `isSame` is true for every `WithMdc`): the outermost wins (`MDC:51-82`, the loop at 60-75; `isSame` at `MDC:85-88`). `aroundReceive` puts the static and per-message MDC, calls the target and clears the MDC in `finally` (`MDC:90-96`); `aroundSignal` puts the static MDC only (`MDC:99-105`). Supervision logs carry the MDC only when they run inside `withMdc`. |
| I6 | `MonitorInterceptor.aroundReceive` sends the message to the monitor and then calls the target (`IMPL:138-141`): before the handler, whatever the handler does. It has no `aroundSignal`. |
| I7 | As I1: the default `aroundSignal` forwards, so an interceptor that does not override it neither prints nor blocks anything (`BI:84-85`). |

## 3. Results

Excerpts of the observed sequences (`T` = the target behavior).

| Scenario | Observed | Matches the source? |
|---|---|---|
| I1 | Start: `A before, B before, C before, T setup, C after, B after, A after`. Message `m1`: `A before, B before, C before, T message, C after, B after, A after`. Signals (`Terminated` of the watched child, then `PostStop`): `A before, B before, C before, T signal, C after, B after, A after`. | Yes |
| I2a | The same instance twice: `aroundStart` ran twice (outer and inner); after the start there is one interceptor: each message and the `PostStop` signal show one `before` and one `after`. | Yes |
| I2b | `monitor(ref, monitor(ref, ...))`: the monitor received `m1`, `m2` and `stop` once each. | Yes |
| I2c / I2c1 | `logMessages(logMessages(...))` and a single `logMessages`: the same lines, one log line per message and one for the signal `PostStop` ("actor [...] received signal: PostStop"). | Yes |
| I2d | Inner `restart`, outer `stop`, class `Throwable` both: the failure restarted the actor (`PreRestart`, `RestartSupervisor saw failure: boom`, `setup` again, `m2` processed); the outer `stop` did not apply. | Yes |
| I2e | A recursive behavior that returns the monitor every turn: the monitor received `m1`, `m2`, `m3` once each. | Yes |
| I3 | `OUT` and `IN` saw `fail`; the target threw; `IN` saw `PreRestart`, `OUT` did not; `OUT receive after 'fail'` was printed (the failure did not reach `OUT`); `ERROR RestartSupervisor saw failure: boom`; `IN` was created again (`IN instance created`, `aroundStart`, `T setup`) while `OUT` was not, and the re-creation happened inside `OUT`'s handling of the `Terminated` signal of the old child (the restart waits for the child); `m2` passed `OUT` and the new `IN`. | Yes |
| I4a | `ok` reached the target as the transformed value; `bad` (no match) produced an `UnhandledMessage` with `message='bad'` (the original), `sender=deadLetters`, `recipient=the actor`, plus the INFO "was unhandled"; no failure and the actor kept running. | Yes |
| I4b | Two `transformMessages` with different partial functions: the actor did not start: `ERROR ... transformMessages can only be used one time in the same behavior stack. One defined in ..., and another in ...`; the later message `a` became a dead letter ("was not delivered"). | Yes |
| I4c | The same partial function instance twice: accepted; `ok` transformed, `bad` produced the same `UnhandledMessage`. | Yes |
| I5a | In the handler: `MDC[s=static m=hello]`, then `m=again` for the next message. `OUT` (outside `withMdc`) printed `MDC[null]` before and after the call: the MDC is set only inside and cleared afterwards. | Yes |
| I5b | `withMdc(outer: s=outer, k=outer-k)(withMdc(inner: b=inner, k=inner-k)(...))`: the handler saw `s=outer k=outer-k` and no `b`: the outer replaced the inner. | Yes |
| I5c | `withMdc(...)(supervise(target))`: the supervision log line `RestartSupervisor saw failure: boom` carried `s=static`. | Yes |
| I5d | `supervise(withMdc(...)(target))`: the same supervision log line had no MDC (`s=`); after the restart the handler again saw `s=static`. | Yes |
| I6 | `monitor` received `m1`, `slow`, `u` (handler returned unhandled) and `fail` (handler threw). For `slow` the monitor printed `got 'slow'` before `TARGET handler end` (the handler slept 400 ms). The monitor did not receive any signal (it received the message `stop` of I2b, not `PostStop`). | Yes |
| I7 | Signals (`Terminated`, `PostStop`): `A before, C before, T signal, C after, A after`; `B` (default `aroundSignal`) printed nothing, and the signal reached the target. Messages passed through `A, B, C`. | Yes |

## 4. Findings vs ADR 0036

| ADR 0036 text | Observed | Result |
|---|---|---|
| Table: `BehaviorInterceptor` with `aroundStart`, `aroundReceive`, `aroundSignal`, `interceptMessageClass`; "signals always go through `aroundSignal`" | I1, I7: every interceptor's `aroundSignal` runs for every signal; one that does not override it forwards | Matches |
| Table: `Behaviors.intercept(factory)`: "the factory creates one instance per actor" | I1: one instance per actor; I3: the factory is called again when a supervisor restarts the behavior that contains it | Matches; the restart re-creation is not stated |
| Table: `monitor`: "each message of class `T` is also sent to `ref` and continues to the inner behavior" | I6: sent before the handler runs, also when the handler throws or returns unhandled; signals are not sent | Matches; the timing and the failure cases are not stated |
| Table: `logMessages`: "logs each message before invoking the inner behavior, with `LogOptions`" | I2c: it also logs each signal ("received signal"), before invoking the inner behavior | Partly covered: signals are not mentioned |
| Nesting: "an interceptor returned by the inner behavior is re-applied after each turn. If an `isSame` interceptor is already in the stack it is not duplicated" | I2a to I2e: the duplicate is removed at the start as well as after a turn; the outer duplicate is dropped and the inner kept (its `aroundStart` still ran); `isSame` differs per interceptor (instance, `actorRef`, `opts`, throwable class, any `withMdc`) | Matches; the start-time removal and the per-interceptor `isSame` are not stated |
| To be verified: order of nested interceptors | I1, I3, I7: the outer one runs first before the call and last after it, for the start, messages and signals | Answered |
| To be verified: `transformMessages` | I4a, I4b, I4c: unmatched message is unhandled (`UnhandledMessage` with the original message, sender `deadLetters`); a second one with a different function fails the actor at start; the same function instance is accepted | Answered |
| To be verified: `withMDC` (Pekko: `withMdc`) | I5a to I5d: static and per-message MDC during the handler, cleared after; the outer `withMdc` wins; supervision logs carry the MDC only inside `withMdc` | Answered for messages |
| (not in the ADR) MDC during signals | Source: static MDC only (`MDC:99-105`); not run (the target did not print the MDC in its signal handler) | Not covered |

## 5. Limits

Two runs per scenario on one OS and JDK. `Behaviors.monitor` ordering between the monitor and the handler is asynchronous; I6 shows only that the monitor is sent the message before the handler returns. `BehaviorSignalInterceptor` was not run. The ADR text on `interceptMessageClass` bypass (a message of another class passing untouched) was not run.

Correction (2026-11-05, #976 slice 4): `MDC:36-62, 85` -> `MDC:51-82` (`aroundStart`; the loop that removes the nested `WithMdc` at 60-75) and `MDC:85-88` (`isSame`); line 85 was right, 36-62 were the tail of the companion and the class header. Every other citation of this document was re-read and matched.
