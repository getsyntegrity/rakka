# RULEBOOK: Ask cancellation, mandatory deadline and cleanup (#165, unit ask-cancel)

Status: **DRAFT, awaiting Owner approval of its sha256.** Read-only inside any loop (kit `CLAUDE.md` rule 1). Approval is recorded in this unit's `EXECUTION-RECORD.md`; this header is never edited after approval.

## 0. Unit

- Capability: the unexported `ask` observes the context. An already canceled context returns `ErrAskCanceled`; a live context without a deadline returns `ErrAskNoDeadline`; cancellation during `build` or during the wait settles the Ask as canceled; reply, timeout and cancellation compete for one terminal transition; after every outcome nothing is left behind (timer stopped, no registry entry, a losing reply dead-lettered).
- Source: **none in Pekko.** `AskSupport.scala` has no context cancellation (the only `cancel` is the scheduler's, `AskSupport.scala:741`, already ported in #164). These are the Go-specific rows AK12, AK13, AK14, AK15, AK22 (check order) and AK10 (cleanup; Pekko source `AskSupport.scala:739-742`, typed `AskSpec.scala:164`) of ADR 0015, so **no Pekko parity is claimed**: the evidence is Go tests, a mutation campaign and a design review.
- Governing decisions (ADR 0015): D2 mandatory deadline `[Owner, Q1]`; D3 `ErrAskCanceled` and `ErrAskNoDeadline` with `errors.Is(err, context.Canceled)` `[Owner for the condition, names Proposal]`; D4 one terminal transition with `cancelled` `[Proposal]`; D5 steps 1-3, order `[Owner]`; D8 the endpoint is never registered `[Proposal]`.
- Writable by the loop (exactly these six files): `pekko-actor/actor/errors.go`, `ask_endpoint.go`, `ask.go`, `ask_firstreply_internal_test.go` (to give its asks a valid deadline), `ask_timeout_internal_test.go` (to replace T6) and the new `pekko-actor/actor/ask_cancel_internal_test.go`. Nothing else, including the README (the orchestrator updates it).

## 1. Decisions this RULEBOOK asks the Owner to confirm by approving its hash

The ADR leaves these open; this unit implements them as written, and approving the hash approves them.

- **F1 classification.** When `ctx` ends, `ctx.Err()` decides: if `errors.Is(ctx.Err(), context.DeadlineExceeded)` the result is `ErrAskTimeout`, any other end is `ErrAskCanceled`. Both settle through the single CAS, so a clock timer and `ctx.Done` racing still give exactly one result.
- **F2 exports.** `ErrAskCanceled` and `ErrAskNoDeadline` are exported now (the names are `[Proposal]` in D3; the Owner accepted the conditions). `ask` stays unexported.
- **F3 existing tests.** They are edited to carry a valid deadline; no shim inside `ask` (a shim would defeat D2). Test T6 of #164 (`no deadline arms no timer`) asserted the transitional behavior and is replaced.
- **F4 reply after cancel.** A reply that reaches the endpoint after a cancellation is dead-lettered once as `DeadLetter{Msg, To: endpoint ID}` (the unchanged losing branch of `Tell`).
- **F5 AK14 in scope.** The three-way race reply / deadline / cancel has exactly one terminal result per run.
- **F6 "libera su entrada del registro".** D8 says the endpoint is never registered, so the MVP clause is read as: no entry ever exists and the registry sizes are unchanged after every outcome, and the timer is released. No registry is introduced.
- **F7 tie rule.** When the context is both canceled and past its deadline, `ctx.Err()` wins (`ErrAskCanceled` or `ErrAskTimeout` as it reports); the clock-based expiry is judged only if `ctx.Err()` is nil.
- **F8 cancel during `build`.** `ctx` is read again after `build`; if it has ended, nothing is sent and no timer is armed (as for a deadline consumed by `build` in #164), the endpoint settles through the CAS, and the result is classified by F1.

## 2. Behavior specification (the implementer follows this; it is not open design)

1. `errors.go`: add exported `ErrAskCanceled = &askError{msg: "actor: ask canceled", cause: context.Canceled}` and `ErrAskNoDeadline = &askError{msg: "actor: ask without a deadline"}` (no cause), both typed `error`, documented in the sentinel block. Only these two identifiers are exported by this unit.
2. `ask_endpoint.go`: add the terminal state `askCanceled` and a method `cancel()` (CAS from `askPending`; the winner closes `done`), a method `canceled() bool`, and extend `outcome()` to a switch: timed out gives `(zero, ErrAskTimeout)`, canceled gives `(zero, ErrAskCanceled)`, replied gives `(reply, nil)`. `Tell` and `expire` keep their single CAS. Update the comments of the terminal states.
3. `ask.go`, in this order (D5):
   1. `askTarget` (unchanged): `ErrInvalidHandle`.
   2. `if err := ctx.Err(); err != nil` return the classified error (F1) **before** anything else about the deadline (F7); then `deadline, has := ctx.Deadline()`; if `has` and `deadline.Sub(clock.Now()) <= 0` return `ErrAskTimeout` (the #164 check).
   3. If `!has` return `ErrAskNoDeadline`.
   Steps 1-3 create no endpoint, call no `build` and send nothing.
   4. Create the endpoint, call `build(endpoint)` exactly once on the caller's goroutine.
   5. After `build` (F8): if `ctx.Err() != nil` settle the endpoint by the classified error (`cancel()` or `expire()`), send nothing, arm no timer; else measure `rem := deadline.Sub(clock.Now())` (#164, DEV-1): if `rem <= 0` call `expire()`, send nothing; else arm the timer on the injected clock for `rem`, then send.
   6. Wait: `select` on `<-endpoint.settled()` and `<-ctx.Done()`. If `ctx.Done` fires first, settle the endpoint by `ctx.Err()` through the CAS (F1) and then `<-endpoint.settled()` (the CAS may have been lost to a reply, whose `reply` write is ordered by the close of `done`). **No goroutine is started by `ask`.** A context whose `Done()` is nil is never selected.
   7. Stop the timer if armed; return `endpoint.outcome()`.
   Split into unexported helpers as needed: every function and literal must stay at cyclomatic complexity <= 10.
4. A reply after the endpoint left `askPending` is dead-lettered by the unchanged `Tell`.
5. Do not change `Tell`, `askTarget`, the dead-letter path, or anything about termination (#166).

## 3. Out of scope (do NOT implement or test)

A terminated target and `ErrRecipientTerminated` (#166); an exported `Ask`; business-error-as-payload (AK11); counting goroutines (there is no deterministic way; `ask` starts none, which is a structural property stated in a comment and checked by the design review); the message text of the errors; the README (orchestrator); `judge/` and CI.

## 4. Conventions

- R4.1 Header: Apache-2.0 notice exactly as in `ask_endpoint.go`, then a `// Ported from` line for the new test file citing the only Pekko source (`AskSupport.scala` 739-742 for the cleanup) and stating in a comment that the cancellation rows are Go-specific (`// Go-specific: ADR 0015 AK12 ...`; no Pekko case exists). Each scenario carries its AK id.
- R4.2 Tests: package `actor`, go-specs per `go-testing`. Reuse the existing helpers (`bg`, `wait`, `closed`, `prsCollectDeadLetters`, `newFakeClock`, `newSystem(Config{}, options{clock: fc})`, `afrSpawn`, `afrRefOf`, `afrWaitGroup`, `afrTake`, `afrRegistrySizes`, `aftoStart`, `aftoAsk`, `aftoStubClock`, `aftoDeadlineCtx` and the other `afr*`/`afto*`); new helpers use the prefix `afca`; never redeclare an existing identifier (Grep first).
- R4.3 Deterministic: no `time.Sleep`, no real timers, no polling. Time moves only through `fakeClock.Advance`. A deadline context is `aftoDeadlineCtx` (embeds a context, overrides `Deadline()`); to cancel, embed `context.WithCancel(bg)` and keep the `cancel`. Never `context.WithDeadline` or `WithTimeout` in tests that use the fake clock. Existing tests on a real-clock System get a far deadline through ONE helper in `ask_firstreply_internal_test.go` (`afrCtx`: `aftoDeadlineCtx{Context: bg, d: time.Now().Add(time.Hour)}`), changing only `afrAsk`, the two direct `ask(bg, ...)` calls and nothing else in those tests. Every wait is bounded through `wait(t, ch)` or a bounded helper; `ask` always runs in a goroutine awaited through a bounded helper, with the single exception of calls that return before any wait (pre-checks); `deadLetterBarrier` only after every sender of a possible dead letter has returned.
- R4.4 No `//nolint`; cyclomatic complexity <= 10; `gofmt`-clean; the changed files pass `golangci-lint` v2.14.0 (mind staticcheck SA1032: do not write `errors.Is(context.DeadlineExceeded, X)`; assert the reverse through `errors.Unwrap(...) == nil` and identity, as T7 of #164 does).
- R4.5 Unknown behavior: `// PORT-GAP(<id>): <reason>`; never invented behavior. A scenario that fails on the specified runtime is classified by root class (`skills/gopekko-failure-triage`) before any change.

## 5. Required scenarios (each with an assertion that fails if the behavior breaks)

- C1 canceled before `ask`, with a future deadline: `err == ErrAskCanceled`, `errors.Is(err, context.Canceled)`; `build` not called; nothing sent (a probe `afrReq{n:-1}` is the first message the target receives); `fc.Pending()==0`; registry sizes unchanged; zero dead letters.
- C2 canceled before `ask`, with no deadline: `ErrAskCanceled` (not `ErrAskNoDeadline`).
- C3 order of the pre-checks, one case per adjacent pair: invalid target with a canceled context gives `ErrInvalidHandle`; canceled and expired gives `ErrAskCanceled`; expired and live gives `ErrAskTimeout`; live without a deadline gives `ErrAskNoDeadline`; a nil target without a deadline gives `ErrInvalidHandle`. Each: `build` not called, nothing sent, nothing armed.
- C4 no deadline, live context: `err == ErrAskNoDeadline`; not `context.Canceled` nor `DeadlineExceeded`; `build` not called; nothing sent; `Pending()==0`. (This replaces T6 of #164.)
- C5 cancel during the wait: the target captures the endpoint and never replies; after the endpoint is obtained and `fc.Pending()==1`, `cancel()`: `ask` returns `ErrAskCanceled`; `fc.Pending()==0` (the timer was stopped); a later reply to the endpoint is published exactly once as `DeadLetter{Msg, To: endpoint ID}` and the result does not change.
- C6 cancel during `build` (F8): `build` calls `cancel()`; nothing is sent (probe), no timer armed (`aftoArmed==0`), `ErrAskCanceled`; a reply to the leaked endpoint is dead-lettered once.
- C7 classification (F1): a context whose `Done` fires with `Err()==context.DeadlineExceeded` returns `ErrAskTimeout` (identity and `errors.Is(err, context.DeadlineExceeded)`), not `ErrAskCanceled`. Use a small test context type with its own `Done` channel and `Err()`.
- C8 three-way race (AK14): per iteration (fresh system, clock, endpoint) one gate releases `endpoint.Tell(bg, v)`, `fc.Advance` past the deadline and `cancel()` concurrently; join them, take the ask outcome, then the barrier; exactly one of: `(v, nil)` with zero dead letters; `ErrAskTimeout` with exactly one dead letter `{v, endpoint ID}`; `ErrAskCanceled` with exactly one dead letter `{v, endpoint ID}`; `Pending()==0` in all. Count outcomes with `t.Logf` only; never assert that all occur.
- C9 sentinels: `errors.Is(ErrAskCanceled, context.Canceled)` true; `errors.Is(ErrAskCanceled, context.DeadlineExceeded)` false; `errors.Is(ErrAskTimeout, context.Canceled)` false; `ErrAskNoDeadline` has a non-empty text and no cause (`errors.Unwrap == nil`); the three sentinels are pairwise different values; `ask` returns each by identity.
- C10 cleanup (AK10, F6), table-driven over the outcomes reply, timeout, cancel before, cancel during the wait, expired pre-check, no-deadline reject: after `ask` returned, `fc.Pending()==0`; `afrRegistrySizes` equals the size before the ask; for the outcomes that created an endpoint, `lookupID(endpoint.ID())` finds nothing and `isOwned(endpoint)` is false.
- C11 the existing tests of `ask_firstreply_internal_test.go` and `ask_timeout_internal_test.go` keep passing with their asserts unchanged except T6; the loop does not weaken, skip or delete any of them.

## 6. Model plan, limits and token definition (PROPOSED; Owner approves in RUNBOOK)

See `RUNBOOK.md` section 3.

## 7. Deviation log

None approved. Approval and effective authorship are recorded in this unit's `EXECUTION-RECORD.md`, not here.
