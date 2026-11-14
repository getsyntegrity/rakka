# RULEBOOK: Typed `Unstash` and `UnstashAll` (unit typed-927-unstash, #927 TYP-14, includes #1169 TYP-35)

Status: **approved at gate G2 by the Owner on 2026-11-10** for `docs/scopes/927-typed-unstash.md` (PR #1524, merged). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after the unit starts.

## 1. Unit

- Capability: `StashBuffer.UnstashAll(behavior)` and `StashBuffer.Unstash(behavior, n, wrap)`, both `(Behavior[T], error)`: the buffered messages (the first `n`, or all present at the call) go in order, one at a time, to `behavior` and to the behavior each one returns; the result is the final behavior. Four slices, sequential: **A1** the replay engine (U1, U3, U4, U5, U8 and the D-6 test), **A2** `Setup`, `Unhandled`, `Same` and the interpreter link (U2, U6, U7), **B** the failure per strategy (`unstashError`, F1, F2), **C** the interruption by `Stopped` and the dead letters (S1 to S5, closes #1169).
- Out of every slice: Backoff and the `Stop` strategy (`PORT-GAP(TYP-14-backoff-1063)`, `PORT-GAP(TYP-14-stop-strategy-1063)`), the interceptor cases (`PORT-GAP(TYP-14-interceptor-978)`), a message of the Go type `Signal` replayed as a signal (`PORT-GAP(TYP-14-signal-message)`).
- Governing decisions: scope D-1 to D-7; ADR 0033 D13; ADR 0035; ADR 0011 EXP-1 (`ErrUnstashUnhandled` is the only new export, in slice A2).
- Pekko v1.7.1 (`9a54d12`) is the oracle by reading; the sender and recipient of the dead letters (slice C) are the observed values of the oracle in PR #1525. Evidence classes: `SPEC`, `READ`, `ORACLE`. Slices A1 to B are **not V3**.
- Kit run (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`): port plus adaptation, kit at `cf91c9d`. The Dev is a separate session from QA.
- Layout: everything in `pekko-actor-typed/`; `typed` keeps importing only `pekko-actor`. No slice of #927 touches `pekko-actor/`.

## 2. Allowed files

Slice A1 (one PR): `typed/unstash.go` (new), `typed/unstash_test.go` (new, `package typed`), `typed/unhandled_internal_test.go` (only the `want` list of exported names), `pekko-actor-typed/README.md`. The kit package (this directory and one row of `migration/cost-log.tsv`) goes in a separate docs PR so that A1 stays under 400 authored lines. Later slices: the files of the scope table (`typed/behavior.go`, `interpreter.go`, `context.go`, `example_test.go`, `docs/adr/0035-typed-supervision.md` for B). Anything else is a Stop.

## 3. Rules for the Dev

- D-1: eager `(Behavior, error)`, `wrap == nil` is the identity (`TODO(TYP-14-nil-wrap)`). D-2: the engine is a function of the buffer, independent of the interpreter. D-5: only the messages present at the call, `n = min(n, size)`, `Clear` ends the replay, an empty buffer returns the behavior unstarted. D-6: the buffer is used only from the actor's own turn of the incarnation that built it, documented in the README and pinned by a test. A1 starts the behavior and every returned behavior with `startLayers`, and returns the unexported `errUnstashLater` for a `Supervise` layer or a `Stopped` result (`TODO(TYP-14-a2)`, removed by A2 and C).
- Headers: `unstash.go` carries exactly one `// Ported from .../StashBufferImpl.scala` line; the tests carry none.
- Tests: go-specs matchers; no `time.Sleep`; no real engine. Coverage at least 85% per module, no `//nolint`, gocyclo at most 10; the README follows the module README contract.

## 4. Mandatory Stops

Report to the TL and stop; never skip, delete or weaken a test:

1. A change in `pekko-actor` is needed.
2. A test fails on the current code where the scope expects it to pass; a failing scenario is run on Pekko before it is classified (`gopekko-failure-triage`).
3. A file outside the allowed list must change, or the PR projects over 400 authored lines.
4. A decision (D-1 to D-7) must change, or an export other than `ErrUnstashUnhandled` is needed.

## 5. Mutation rule (slice A1)

M1 the replay takes the last message; M2 the head is not removed; M3 the loop runs until the buffer is empty; M4 `wrap` is not applied; M5 `n` is ignored; M6 the replay iterates a snapshot (`Clear` is not seen); M7 an empty buffer starts the behavior; M8 a nil `wrap` is not the identity; M9 `Same` restarts the behavior instead of keeping it; M10 a `Supervise` layer or a `Stopped` result is accepted; M11 the starting behavior is not started. Applied by hand in a disposable edit of `unstash.go`, restored after each run, never committed; the named test must fail by assertion.
