# RULEBOOK: Empty and Ignore (unit typed-1181-empty-ignore, #1181 TYP-47)

Status: **accepted by the Owner as an exception, for this unit only** (decision on PR #1264 at `043ffaa`, 2026-10-28; see "Owner decision" below); PR #1264 was later integrated as `595595c`. It was **proposed, not approved,** before that decision (history). No scope file exists for #1181 and no G2 approval for this unit was found. The issue body is the specification. This package was written **after** the implementation, as the kit trace that review of PR #1264 asked for: it records what was done, it is not evidence that a kit run happened beforehand. The Owner accepted it as an exception to scope/G2 (below); that does not make it evidence of earlier steps.

Authorization search (2026-10-28, head `d08b600`): issue #1181 and PR #1264 have no comments or reviews; `docs/scopes/` has no file for #1181. The only Owner-accepted text that names #1181 is the "Correction (accepted, 2026-10-28)" section of `docs/adr/0034-typed-behavior-boundary.md`, which assigns `empty`/`ignore` to #1181 ("ADR 0034: #925; #1181 empty/ignore. aceptada"); it does not authorize a scope, a G2 gate or a kit run for this unit. `docs/scopes/954-slice-b-terminated-death-pact.md` (D-B4) and `docs/scopes/925-typed-unhandled-marker.md` (U-6) name #1181 as a follow-up only.

## 1. Unit

- Capability: `Empty[T]()` answers every message and signal with the unhandled result; `Ignore[T]()` answers with Same. An unhandled Terminated is the death pact (default Stop of #956); Ignore survives it. Both are valid initial behaviors, are zero-size special behaviors resolved by the interpreter (ADR 0034 P2), and `ReceiveSignal` rejects both as a base (`ErrSignalBaseNotReceive`). Source: Pekko `Behavior.scala:248-249,270-275,297-306`, `BehaviorImpl.scala:67-69,78-84` at v1.7.1 (`9a54d12`), read, not run.
- Governing decisions: ADR 0033 D1, D2, D13; ADR 0034 P2; #925 Slice A (Unhandled and its local record); #956 (default Stop).
- Exports added: `Empty`, `Ignore` (the U9 public-surface pin is updated).
- Writable: `pekko-actor-typed/` (code, tests, README, `go.mod`/`go.sum` for go-specs) and this package. Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`.

## 2. Upstream search

No Typed spec states the difference between `empty` and `ignore`; they appear only as no-op behaviors (`ActorRefResolverSpec.scala:31`, `LogMessagesSpec.scala:36`). This is "no spec found by these searches", not a proof of absence. All rows are READ. The oracle exists: Pekko's `Behavior.scala:297-306` defines both behaviors. What does not exist yet is an executed Typed scenario (PORT-GAP below), so "no oracle" must not be read from a missing spec or a missing run.

## 3. Decisions

- **READ rows are alternative evidence pending Owner approval (ADR 0033 D13).** Not Pekko parity, not V3, not an executed Pekko scenario.
- **AC-R3-1 differential scenario** is a PORT-GAP until the Typed harness (#1046) exists.
- **AC-R4-1 (aliveness) stays a requirement of #1181.** The AC asks for a public-API Go test of aliveness. `pekko-actor-typed` has no public aliveness accessor, so that test cannot be written without a new export, which this unit does not add. The tests give alternative evidence only: both behaviors keep the actor running after messages, and Ignore after Terminated. It is **READ, pending Owner approval (ADR 0033 D13)**. No destination story is named; the Owner decides whether the alternative evidence is accepted.
- Observing "unhandled" uses the unexported record hook of #925 (U-2 (a)); nothing is published.
- Out of scope: the unhandled marker publication (#925 Slice B, #897), `Stopped` (#1182), death-pact plumbing (#954).

## 4. Conventions

- Tests use go-specs (Owner reminder in this session); an observable barrier proves each input was processed, never a sleep as a witness.
- No `//nolint`; cyclomatic complexity at most 10; Unknown behavior is `// TODO(TYP-47-edge): <reason>`.
- Comments and names claim no more than is shown: no Typed parity.

## Owner decision (PR #1264, head `043ffaa`, 2026-10-28)

Recorded from the Owner's message, which reached the Dev partly truncated; only the legible parts are recorded:

1. Diff coverage 83% (`behavior.go`, the two zero-statement marker methods) is accepted **exceptionally**; the 85% threshold is not to be met with padding tests.
2. This kit package is accepted for this unit as an exception to the scope/G2 requirement. The package was written after the implementation; the acceptance does not make it evidence that earlier kit steps took place.
3. The READ rows and AC-R4-1 remain **pending** under ADR 0033 D13. The decision is bound to `043ffaa`; later code changes need a new decision.

Not decided, so not claimed: Pekko parity, V3, closing #1181 (it stays open with its labels).
