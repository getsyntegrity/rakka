# RULEBOOK: Ask timeout (#164, unit ask-timeout)

Status: **DRAFT, awaiting Owner approval of its sha256.** Read-only inside any loop (kit `CLAUDE.md` rule 1). Approval is recorded in this unit's `EXECUTION-RECORD.md`; this header is never edited after approval.

## 0. Unit

- Capability: when the context carries a deadline, the unexported `ask` arms a timer on the System's injected `clock` and, if the deadline is reached before a reply, returns `ErrAskTimeout`. Reply and timeout compete for one terminal transition (exactly one wins); a reply that loses is dead-lettered. A deadline that has already passed when `ask` starts returns `ErrAskTimeout` without calling `build` and without sending.
- Source (Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, read-only): `upstream/pekko/actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala` 721-743 (scheduler call at 725, `result.tryComplete(Failure(timeout))` at 726-734, `f.cancel()` at 739-742) and 614-629 (a message after completion goes to dead letters); typed `actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala` 96 (fail if the actor does not reply in time), 132 and 164 (late reply after a timeout, adapted), and classic `AskSpec.scala` 72 and 83 (zero and negative timeout, adapted to a deadline that is not in the future).
- Governing decisions (ADR 0015): D4 one terminal transition with `timed_out`, the winner stops the timer; D9 the timer duration is `deadline - clock.Now()`; D3 `ErrAskTimeout` satisfies `errors.Is(err, context.DeadlineExceeded)` (the one condition the Owner attached to accepting the name); D5 step 2 (an expired deadline is judged with the injected clock; `build` is not called and nothing is sent).
- Writable by the loop (exactly these four files): `pekko-actor/actor/errors.go`, `pekko-actor/actor/ask_endpoint.go`, `pekko-actor/actor/ask.go`, and the new `pekko-actor/actor/ask_timeout_internal_test.go`. Nothing else, including the README (the orchestrator updates it).

## 1. Behavior specification (the implementer follows this; it is not open design)

1. `errors.go`: add an unexported `askError{msg string; cause error}` with `Error()` returning `msg` and `Unwrap()` returning `cause`, and the exported sentinel `ErrAskTimeout` (type `error`, value `&askError{"actor: ask timed out", context.DeadlineExceeded}`), documented in the sentinel block. `errors.Is(err, ErrAskTimeout)` holds by identity; `errors.Is(err, context.DeadlineExceeded)` holds through `Unwrap`. `ask` returns `ErrAskTimeout` itself, not a wrapper. Only this one sentinel is added.
2. `ask_endpoint.go`: add a terminal state `askTimedOut` beside `askPending` and `askReplied`, a method `expire()` that does `CompareAndSwap(askPending, askTimedOut)` and, only if it won, closes `done`, and a method `timedOut() bool`. `Tell` is unchanged: its single CAS already makes any reply after the endpoint left `askPending` a dead letter, so a reply after a timeout is dead-lettered as `DeadLetter{Msg, To: endpoint ID}`. `result()` is meaningful only when the endpoint replied.
3. `ask.go`, in this order: `askTarget` (unchanged); read `deadline, has := ctx.Deadline()`; if `has`, `rem := deadline.Sub(r.c.sys.clock.Now())` and if `rem <= 0` return the zero `R` and `ErrAskTimeout` **before** creating the endpoint, calling `build` or sending; create the endpoint; call `build(endpoint)` exactly once on the caller's goroutine; if `has`, arm `t := r.c.sys.clock.AfterFunc(rem, endpoint.expire)` (after `build`, before the send); send with `r.Tell(ctx, msg)`; wait on `<-endpoint.settled()`; if `t != nil`, call `t.Stop()`; if `endpoint.timedOut()` return the zero `R` and `ErrAskTimeout`, else return `endpoint.result(), nil`.
4. A context **without** a deadline arms no timer and `ask` waits only for the reply, exactly as in #162 and #163. This is a deliberate, documented transitional behavior: the mandatory-deadline rule (ADR 0015 D2, `ErrAskNoDeadline`) belongs to the pre-send checks, not to this unit.
5. `ctx.Done()` is not observed (cancellation is #165).

## 2. Out of scope (do NOT implement or test)

Cancellation during the wait and `ErrAskCanceled` (#165); the mandatory deadline and `ErrAskNoDeadline`, canceled-context pre-send checks and their order (AK12, AK15, AK22); a terminated target (#166); the full AK10 cleanup (registry and endpoint retention after every outcome; only the timer stop is in scope here); the message-content assertions of Pekko's classic timeout tests (`AskSpec.scala` 94, 103, 111, 120: target, timeout, sender and message class in the error text, `PORT-GAP(timeout-message-info)`, because the error text is a `[Proposal]` of D3); an exported `Ask`; `judge/`, `README`, CI.

## 3. Conventions

- R3.1 Header: Apache-2.0 notice exactly as in `ask_endpoint.go`, then `// Ported from <path in upstream/pekko>`; each scenario carries `// AS:<lines>` and, for adapted ones, the typed or classic AskSpec line with the word ADAPTED.
- R3.2 Tests: package `actor`, go-specs per `go-testing`. Reuse the existing helpers (`bg`, `wait`, `closed`, `prsCollectDeadLetters`, `newFakeClock`, `newSystem(Config{}, options{clock: fc})`, `afrSpawn`, `afrRefOf`, `afrWaitGroup`, `afrTake` and the other `afr*` helpers); new helpers use the prefix `afto`; never redeclare an existing one (Grep first).
- R3.3 Deterministic time: **no `time.Sleep`, no real timers, no polling**. Time moves only through `fakeClock.Advance`. A deadline context is a tiny test type embedding `context.Context` and overriding `Deadline()` with a value derived from the fake clock's `Now()`; do not use `context.WithDeadline` or `WithTimeout` (they arm wall-clock timers). The sync point "the timer is armed" is the moment the test receives the endpoint from the target (the timer is armed before the send), which `fakeClock.Pending()` confirms. Every wait goes through `wait(t, ch)` or a bounded `afto`/`afr` helper; `ask` always runs in a goroutine awaited through a bounded helper. `deadLetterBarrier` only after every sender of a possible dead letter has returned.
- R3.4 No `//nolint`; cyclomatic complexity <= 10 per function and literal; `gofmt`-clean; the new test file passes `golangci-lint` v2.14.0.
- R3.5 Unknown behavior: `// PORT-GAP(<id>): <reason>`; never invented behavior. A scenario that fails on the specified runtime is classified by root class (`skills/gopekko-failure-triage`) before any change.

## 4. Case table

| Upstream case | Disposition | Go scenario |
|---|---|---|
| typed `AskSpec.scala:96` fail if the actor does not reply in time | faithful (fake clock instead of 10 ms) | T1, T2 |
| `AskSupport.scala:725-734` timer completes with a timeout | faithful | T1 |
| `AskSupport.scala:739-742` completion cancels the timer | faithful | T3 (reply first) |
| `AskSupport.scala:614-629` and typed `:132, :164` late reply after a timeout is dead-lettered | adapted (Go has no `context.ask` or `AskPattern`) | T3 (timeout first), T4 |
| classic `AskSpec.scala:72, :83` zero and negative timeout | adapted: a deadline that is not in the future (D5 step 2); no claim of parity | T5 |
| classic `AskSpec.scala:94, :103, :111, :120` error text content | omitted, `PORT-GAP(timeout-message-info)` | none |
| typed `:109` and classic `:41, :50` | not timeout (termination, #166) | none |

## 5. Required scenarios (each with an assertion that fails if the behavior breaks)

- T1 timeout without reply: deadline `fc.Now()+D`; the target captures the endpoint and never replies; after `fc.Advance(D)` `ask` returns the zero value and `ErrAskTimeout`; `errors.Is` holds for `ErrAskTimeout` and `context.DeadlineExceeded`; `fc.Pending()` is 0.
- T2 not early: with the endpoint obtained and `fc.Pending()==1`, `Advance(D-1ns)` leaves the endpoint unsettled and `Pending()==1`; one more `Advance(1ns)` settles it as timed out.
- T3 controlled interleavings: (a) the reply is sent first, then `fc.Advance` past the deadline: `ask` returns the reply with a nil error, `Pending()==0` right after `ask` returns (the timer was stopped), and no dead letter; (b) `fc.Advance` first, then the reply: `ask` returns `ErrAskTimeout`, the reply is published exactly once as `DeadLetter{Msg, To: endpoint ID}` and the returned error does not change.
- T4 race: for K iterations a start gate releases at the same instant a reply `Tell` and `fc.Advance` past the deadline; each iteration has exactly one terminal outcome: either (the reply, nil) with no dead letter, or (zero, `ErrAskTimeout`) with exactly one dead letter addressed to that endpoint; never both, never neither. K iterations use fresh systems, fake clocks and endpoints.
- T5 deadline not in the future: deadline `fc.Now()` and deadline in the past: `ask` returns `ErrAskTimeout`, `build` is not called, the target received nothing, no timer was armed (`Pending()==0`), `deadLettersPublished()==0`.
- T6 no deadline: with a context that has no deadline, `ask` arms no timer (`Pending()==0` while it waits) and returns the reply when it arrives.
- T7 the sentinel: `errors.Is(ErrAskTimeout, context.DeadlineExceeded)` is true, `errors.Is(context.DeadlineExceeded, ErrAskTimeout)` is false, `ErrAskTimeout` is not the value `context.DeadlineExceeded`, and its text is non-empty.
- The existing `PORT-GAP(late-after-timeout)` comment in `ask_firstreply_internal_test.go` is resolved by T3(b) and T4; the loop does not edit that file (the orchestrator removes the comment).

## 6. Model plan, limits and token definition (PROPOSED; Owner approves in RUNBOOK)

See `RUNBOOK.md` section 2.

## 7. Deviation log

None approved. Approval and effective authorship are recorded in this unit's `EXECUTION-RECORD.md`, not here.
