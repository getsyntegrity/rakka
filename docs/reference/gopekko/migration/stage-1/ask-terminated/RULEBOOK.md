# RULEBOOK: Ask on a terminated target and a pending Stop (#166, unit ask-terminated)

Status: **DRAFT, awaiting Owner approval of its sha256.** Read-only inside any loop (kit `CLAUDE.md` rule 1). Approval is recorded in this unit's `EXECUTION-RECORD.md`; this header is never edited after approval.

## 0. Unit

- Capability: the unexported `ask` fails with `ErrRecipientTerminated` when the target's termination has **completed** (ADR 0015 D5 step 4), and keeps the Ask **pending until the deadline or a cancellation** when the target merely accepted a Stop whose finish step has not completed (AK19a/b, R1).
- Source (Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, read-only): `upstream/pekko/actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala` 312-319 (`recipientTerminatedException`) and 372-376 (`case ref: InternalActorRef if ref.isTerminated`: the message is still sent, so it dead-letters, and the future fails; no ask ref is created); typed `upstream/pekko/actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala` 71-82 ("fail the future if the actor is already terminated"), **adapted**: Pekko fails with a timeout exception whose text says the recipient had already been terminated, goPekko fails with a distinct sentinel (D3). The cases for dead letters, an `EmptyLocalActorRef` and a nonexistent actor (classic `AskSpec.scala` 41 and 50, typed 109) have no Go analogue because the registry is never pruned: omitted, `PORT-GAP(terminated-nonexistent-target)`. AK18, AK19a, AK19b and the tail of AK22 are Go-specific rows: **no Pekko parity is claimed** for them and no sbt oracle runs.
- Governing decisions (ADR 0015): D5 step 4 `[Owner, Q2, R1]`; D3 `ErrRecipientTerminated` (name `[Proposal]`, the condition "completed before the send, never for a merely accepted Stop" is `[Owner]`); D4 the `rejected` state; D6 a terminated target passes `askTarget`; D7 no automatic Watch, no new contract; AK07, AK18, AK19a, AK19b, AK22.
- Writable by the loop (exactly these four files): `pekko-actor/actor/errors.go`, `ask_endpoint.go`, `ask.go` and the new `pekko-actor/actor/ask_terminated_internal_test.go`. Every existing test file is read-only (none asks a stopped target; the one that touches a stopped target calls `askTarget`, which must keep accepting it). Nothing else, including the README (the orchestrator updates it).

## 1. Decisions this RULEBOOK asks the Owner to confirm by approving its hash

- **F1 position.** The check is step 4 of D5, after the pre-checks 1-3 of #165 and before the normal path. On the terminated path `ask` creates the endpoint, moves it to the terminal state `rejected`, calls `build` exactly once on the caller's goroutine, delivers the request with `r.Tell` (which dead-letters it, as the ADR says), and returns `ErrRecipientTerminated`. No timer is armed, `ctx.Done` is not awaited, nothing waits.
- **F2 the `rejected` state.** A new terminal state `askRejected` of the endpoint, set by the same CAS from `askPending` before `build`; `outcome()` maps it to `(zero, ErrRecipientTerminated)`; a reply that reaches the rejected endpoint is dead-lettered by the unchanged `Tell`.
- **F3 how termination is read.** A non-blocking receive on the target cell's `done` channel (closed by the finish step after `PostStop` and after all children completed, `stop.go` `finalize`). Not `stopRequested`, not `closed` (set earlier than `done`): an accepted Stop is not termination. The check is made in `ask`, **not in `askTarget`** (D6, and the existing test of `askTarget` on a terminated target must keep passing).
- **F4 point-in-time read.** The ADR accepts the window between the check and the send (as Pekko's `isTerminated`). A target that terminates after the check does **not** end the Ask: `Tell` dead-letters or the target never replies, and the Ask ends by deadline or cancellation. No automatic Watch and no goroutine (D7).
- **F5 sentinel.** `ErrRecipientTerminated` is exported now, a plain `*askError` without a cause (like `ErrAskNoDeadline`): it is neither `context.Canceled` nor `context.DeadlineExceeded`. Only this identifier is exported by this unit.
- **F6 scope.** AK07 (adapted from typed `AskSpec.scala:71`), AK18 (the target stops during the wait), AK19a and AK19b (Stop accepted with `Receive` blocked, resolved by the clock or by `cancel`), and the tail of AK22 (the order of the pre-checks with a terminated target).

## 2. Behavior specification (the implementer follows this; it is not open design)

1. `errors.go`: add `ErrRecipientTerminated error = &askError{msg: "actor: recipient terminated"}` in the sentinel block with its documentation.
2. `ask_endpoint.go`: add the terminal state `askRejected`, a method `reject()` (CAS from `askPending`; the winner closes `done`), and extend `outcome()`: rejected gives `(zero, ErrRecipientTerminated)`. Add a method `rejected() bool`. `Tell`, `expire` and `cancel` keep their single CAS.
3. `ask.go`, in this order: `askTarget` (unchanged); `askPrecheck` (unchanged: canceled/expired context, mandatory deadline, deadline in the past); **new step 4**: if the target cell's `done` is closed (non-blocking), then create the endpoint, `reject()` it, call `build(endpoint)` once, `r.Tell(ctx, msg)`, and return `(zero, ErrRecipientTerminated)`; otherwise the existing path (endpoint, `build`, `askArm`, send, `askAwait`, stop the timer, `outcome()`) unchanged. A helper such as `askTerminated(r)` keeps each function at cyclomatic complexity <= 10.
4. `ask` keeps starting no goroutine; no `context.AfterFunc`; no `Watch`.
5. Do not change `askTarget`, `Tell`, `expire`, `cancel`, the clock, or anything in `stop.go`, `cell.go`, `ref.go`, `system.go`.

## 3. Out of scope (do NOT implement or test)

An exported `Ask` and `StatusReply`; business error as payload (AK11); the cross-System rows already covered; counting goroutines; the TOCTOU window itself (declared, not testable without a hook between the check and the send); the error text; the README; `judge/` and CI.

## 4. Conventions

- R4.1 Header: Apache-2.0 notice exactly as in `ask_endpoint.go`, then **exactly one** line `// Ported from upstream/pekko/actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala` — it MUST end in `.scala` with nothing after it (scripts/check-headers.sh rejects a suffix; this was missed in #165) — and a separate comment saying that AK18, AK19a, AK19b and AK22 have no Pekko case. Each scenario carries its AK id and, when adapted, the word ADAPTED.
- R4.2 Tests: package `actor`, go-specs per `go-testing`. Reuse existing helpers (`bg`, `wait`, `closed`, `isOwned`, `prsCollectDeadLetters`, `newFakeClock`, `newSystem`, `afrSpawn`, `afrRefOf`, `afrTake`, `afrWaitGroup`, `afrRegistrySizes`, `aftoNewEnv`, `aftoAsk`, `aftoStart`, `aftoArmed`, `aftoDeadlineCtx`, `afcaCancelable`-style helpers of `ask_cancel_internal_test.go`, `doneOf`); new helpers use the prefix `aftm`; never redeclare an existing identifier (Grep first).
- R4.3 Deterministic: no `time.Sleep`, no real timers, no polling. Time moves only through `fakeClock.Advance`. To hold `Receive` blocked, the target closes an `entered` channel and waits on a `release` channel (`afrSpawn` takes a receive function); accept the Stop with `sys.Stop(target)` after `entered`; assert `Done` is still open by a non-blocking receive; release with `close(release)`. The witness that the request was sent is a dead-letter subscriber that signals a channel (bounded wait), not a sleep. Every wait is bounded through `wait(t, ch)` or a bounded helper; `ask` always runs in a goroutine awaited through a bounded helper, including the calls that should return immediately (a regression must fail within the bound, not hang the binary).
- R4.4 No `//nolint`; cyclomatic complexity <= 10 per function and literal (count `&&`, `||`, `case`); `gofmt`-clean; `golangci-lint` v2.14.0 clean (staticcheck SA1032: never `errors.Is(context.DeadlineExceeded, X)`).
- R4.5 Unknown behavior: `// PORT-GAP(<id>): <reason>`. A scenario that fails on the specified runtime is classified by root class before any change.
- R4.6 Comments and test names must not claim more than is shown: the rejected path shows the error, the single dead letter and the absence of a timer and of a registry entry; it does not show that every reference is released.

## 5. Required scenarios (each with an assertion that fails if the behavior breaks)

- K1 completed termination (AK07, ADAPTED from typed `AskSpec.scala:71`): spawn a target, `Stop` it, wait for `Done`; `ask` returns `err == ErrRecipientTerminated` within the bound; `build` was called exactly once; the request is dead-lettered exactly once as `DeadLetter{Msg: <the request>, To: <target ID>}`; the endpoint (captured in `build`) is `rejected()`; `aftoArmed(fc)==0`; registry sizes unchanged; `lookupID(endpoint.ID())` finds nothing.
- K2 a reply sent to the rejected endpoint after the call is published exactly once as `DeadLetter{Msg, To: endpoint ID}` and the returned error does not change.
- K3 Stop accepted, completion not done (AK19a): `Receive` blocked (`entered`/`release`), `Stop` accepted, `Done` still open; `ask` (in a goroutine, fake-clock deadline) delivers the request: the dead-letter witness fires exactly once; `ask` has **not** returned and its result is not `ErrRecipientTerminated`; after `fc.Advance` past the deadline the result is `ErrAskTimeout`; then `close(release)`: `Done` closes; the result is unchanged; exactly one dead letter in total (the request).
- K4 the same setup resolved by cancellation (AK19b): `cancel()` gives `ErrAskCanceled`, `fc.Pending()==0` afterwards, exactly one dead letter, then release.
- K5 the order of the pre-checks with a terminated target (AK22 tail): for a target whose termination completed, no deadline gives `ErrAskNoDeadline`; a canceled context gives `ErrAskCanceled`; an expired deadline gives `ErrAskTimeout`; an invalid target (a nil `Ref[afrReq]`) gives `ErrInvalidHandle`. In each: `build` not called and zero dead letters.
- K6 the target stops during the wait (AK18): the target receives the request and replies to nothing; `Stop` it and let the termination complete (`Done` closed); `ask` stays pending (a non-blocking check that it has not returned); `fc.Advance` past the deadline gives `ErrAskTimeout`; no `ErrRecipientTerminated`.
- K7 the sentinel: `ErrRecipientTerminated` has a non-empty text, `errors.Unwrap == nil`, is neither `context.Canceled` nor `DeadlineExceeded` (`errors.Is` false for both), differs from the other four Ask sentinels, and `ask` returns it by identity (not wrapped).
- K8 `askTarget` still accepts a terminated target (the existing test stays green; no edit to it).

## 6. Model plan, limits and token definition (PROPOSED; Owner approves in RUNBOOK)

See `RUNBOOK.md` section 3.

## 7. Deviation log

None approved. Approval and effective authorship are recorded in this unit's `EXECUTION-RECORD.md`, not here.
