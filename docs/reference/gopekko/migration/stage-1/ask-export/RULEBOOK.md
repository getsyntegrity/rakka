# RULEBOOK: Export Ask as the usable API (unit ask-export, no issue yet)

Status: **DRAFT, awaiting Owner approval of its sha256.** Read-only inside any loop (kit `CLAUDE.md` rule 1). Approval is recorded in this unit's `EXECUTION-RECORD.md`; this header is never edited after approval.

## 0. Unit

- Capability: the exported `actor.Ask[T, R any](ctx context.Context, target Ref[T], build func(Ref[R]) T) (R, error)`, a thin wrapper over the unexported `ask` of #162-#166. **No change of the integrated semantics**: the same pre-checks, the same single winner among reply, timeout and cancellation, the same cleanup, the same sentinels.
- Source: ADR 0015 D1, whose signature the Owner approved on 2026-10-14 (`[Owner]`, was `[Proposal]`). Pekko `AskSupport.scala` is the origin of the capability (`ask.go` already carries its `// Ported from` line). **AK11 is a Go-specific row and a deliberate divergence from classic Pekko**: `PromiseActorRef.!` turns a `Status.Failure(f)` reply into a failed future (`AskSupport.scala:620-623`); goPekko returns a business error as an ordinary payload `R` and `Ask`'s error stays nil (ADR 0015 D3). `StatusReply` is deferred. **No Pekko parity is claimed and no sbt oracle runs.**
- Governing decisions (ADR 0015): D1 (API, blocking, no implicit sender), D2 (deadline mandatory), D3 (errors: mechanism versus business), D5 (check order), AK11, increment I-6.
- Writable by the loop (exactly these three files): `pekko-actor/actor/ask.go` (add `Ask`, rewrite the doc comments it makes false), and the NEW external-package test files `pekko-actor/actor/ask_api_test.go` and `pekko-actor/actor/ask_example_test.go`. Everything else is read-only, including every existing test, `errors.go`, `ask_endpoint.go`, the README and the ADR (the orchestrator updates the docs). The internal tests keep calling the unexported `ask`; do not rename or remove it.

## 1. Decisions this RULEBOOK asks the Owner to confirm by approving its hash

- **G1 the function.** `func Ask[T, R any](ctx context.Context, target Ref[T], build func(Ref[R]) T) (R, error)` in `ask.go`, whose body is exactly `return ask(ctx, target, build)`. A package function (Go forbids type parameters on interface methods). No new exported identifier besides `Ask`. No behavior in the wrapper: it adds no default timeout, wraps no error, recovers no panic and does not touch `ctx`.
- **G2 documentation.** The doc comment of `Ask` states, in plain prose: it **blocks** the calling goroutine until the first reply, the deadline or cancellation; it **requires a deadline** on `ctx` (a live context without one returns `ErrAskNoDeadline`, nothing is sent); `build` is called at most once on the caller's goroutine and not at all when the Ask fails before sending; the first reply wins and a later one is dead-lettered; the errors it can return (`ErrAskTimeout` satisfies `errors.Is(err, context.DeadlineExceeded)`, `ErrAskCanceled` satisfies `errors.Is(err, context.Canceled)`, `ErrAskNoDeadline`, `ErrRecipientTerminated`, `ErrInvalidHandle`) are **mechanism** failures; a business error travels inside `R`. It also states that **calling `Ask` from `Receive` blocks that actor: it processes no other message while it waits**, the deadline is its only exit if the reply never comes, and asking itself can only end by the deadline or cancellation because the blocked actor cannot receive its own request. The existing "Limits" and "stays unexported" wording of `ask` must be made true again.
- **G3 external tests.** Package `actor_test`, using only exported identifiers (the fake clock is unexported and unavailable). Determinism without it: success paths use a far real deadline (`context.WithTimeout` of one hour, cancelled at the end); the timeout path uses an **already expired** deadline (returns before building); a pending Ask is ended by `cancel()` after a channel witness that the target received the request. The pending timeout by clock is covered by the internal fake-clock suite (#164) and is not repeated here. No `time.Sleep`, no polling.
- **G4 AK11.** A reply type carrying an error (for example `Reply{Value int; Err error}`) comes back as the payload with `Ask`'s error nil and the payload's error intact (identity).
- **G5 executable example.** `ExampleAsk` in `ask_example_test.go` with a `// Output:` block, a counter-style actor that replies through `ReplyTo`, a `context.WithTimeout` deadline and the error check.
- **G6 scope.** Out of scope, with their own fronts: Pekko parity and oracles, `System.Shutdown` (AK17), `AskFrom`, `StatusReply`/`AskStatus`, benchmarks, a Go judge scenario, any change of the integrated semantics.

## 2. Behavior specification (the implementer follows this; it is not open design)

1. `ask.go`: add `Ask` after `ask` (or before it) with the doc of G2 and the body of G1. Keep the exact doc claims true against the code; rewrite any sentence of the existing `ask` comment that says the API is unexported or not yet exported.
2. Add no logic to `Ask`. Complexity is 1.
3. Do not change `ask`, `askTarget`, `askPrecheck`, `askArm`, `askAwait`, the endpoint, the errors, the clock, or any other file.

## 3. Out of scope (do NOT implement or test)

Anything listed in G6; the pending timeout driven by a clock (internal suite); counting goroutines; the README, the ADR, `docs/`, `judge/`, CI; a new sentinel; a change to `Context`.

## 4. Conventions

- R4.1 Header: Apache-2.0 notice exactly as in `ask.go` lines 1-9. The new test files have **no** `// Ported from` line (they are Go-specific); a comment states that AK11 is Go-specific and diverges from classic Pekko. Existing `ask.go` keeps its `// Ported from` line unchanged, ending in `.scala`.
- R4.2 Tests: package `actor_test`; plain `testing` or go-specs per `go-testing`; reuse the external helpers (`bg`, `intLog`, `fnActor` in `helpers_test.go`) only where their message types fit, otherwise declare actors with the prefix `aska`; never redeclare an existing identifier (read `helpers_test.go`, `example_test.go`, `watch_api_test.go` first). Test names `TestAskAPI_E<n>_<short>`; the example is `ExampleAsk`.
- R4.3 Deterministic: no `time.Sleep`, no polling, no `time.Tick`. The internal `wait` helper is not visible from the external package, so declare one helper (prefix `aska`) that receives from a channel with a `select` against a 30 s failsafe `time.NewTimer` (stopped on success) and fails the test on expiry, like the internal `wait`; the timer never decides an outcome, it only turns a hang into a failure. Every `Ask` call that could block runs in a goroutine whose result is received through that helper, including the calls that should return immediately, so a regression fails within the bound instead of hanging the binary.
- R4.4 No `//nolint`; cyclomatic complexity <= 10 per function and literal; `gofmt`-clean; `golangci-lint` v2.14.0 clean (staticcheck SA1032: never `errors.Is(context.DeadlineExceeded, X)`; assert the reverse properties with `errors.Unwrap(...) == nil` and identity).
- R4.5 Unknown behavior: `// PORT-GAP(<id>): <reason>`.
- R4.6 Comments and names must not claim more than is shown (no claim that every reference is released; no Pekko parity).

## 5. Required scenarios (each with an assertion that fails if the behavior breaks)

- E1 first reply (AK01 through the public API): `Ask` returns the value and a nil error; `build` was called exactly once; the request carries a usable `ReplyTo`.
- E2 no deadline: a live `context.Background()` gives `err == actor.ErrAskNoDeadline`, `build` not called, the target received nothing (witness: a FIFO probe message is the first one the target sees).
- E3 expired deadline: a context already past its deadline gives `err == actor.ErrAskTimeout` and `errors.Is(err, context.DeadlineExceeded)`; `build` not called. The reverse property: `errors.Unwrap` of the sentinel itself is not nil only for the two context-backed errors (assert as the internal suite does, never `errors.Is(context.DeadlineExceeded, X)`).
- E4 cancellation of a pending Ask: the target holds the request (it closes `entered` on receipt); `cancel()` gives `err == actor.ErrAskCanceled` and `errors.Is(err, context.Canceled)`; a reply sent afterwards through the captured `ReplyTo` is dead-lettered exactly once (`SubscribeDeadLetters` with a non-blocking witness) and the result does not change.
- E5 completed termination: Stop the target, wait for `Done`; `Ask` gives `err == actor.ErrRecipientTerminated`, `build` called once, exactly one dead letter of the request.
- E6 invalid target: a nil `actor.Ref[T]` gives `actor.ErrInvalidHandle` (`errors.Is`) with `build` not called.
- E7 Ask from `Receive` blocks that actor (G2): actor A receives `m1`, calls `Ask` to actor B inside `Receive`; B holds its reply until released; the test sends `m2` to A, releases B, and A's log shows `m1-done` before `m2` was processed (serial processing). The test states what it shows: the order, not an absence in time.
- E8 AK11 business error as payload (G4): the reply `Reply{Err: errBusiness}` comes back with `Ask`'s error nil and `got.Err == errBusiness`.
- E9 the exported sentinels are distinct values and `Ask` returns them by identity (not wrapped), reached by `errors.Is` as documented.
- E10 `ExampleAsk` (G5): the `// Output:` block must match.

## 6. Model plan, limits and token definition (PROPOSED; Owner approves in RUNBOOK)

See `RUNBOOK.md` section 3.

## 7. Deviation log

None approved. Approval and effective authorship are recorded in this unit's `EXECUTION-RECORD.md`, not here.
