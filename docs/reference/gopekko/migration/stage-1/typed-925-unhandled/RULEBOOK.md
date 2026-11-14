# RULEBOOK: Typed Unhandled result and its interpretation (unit typed-925-unhandled, #925 TYP-03 Slice A)

Status: **prepared (PR 2, documentation only).** Scope: `docs/scopes/925-typed-unhandled-marker.md` (integrated as `f682e5f4f8648cf4afa13bd9f949629836e0cff4`, PR #1258). The G2 decisions U-1 to U-7 are recorded there (section 11). **Update 2026-10-28 (dated, appended; the text of PR 2 as first reviewed at `818516899e40705b508a97997efcdbcb9cc682b5` is otherwise unchanged):** the Owner confirmed U-2 (a), U-4 and U-7 in a second message that reached the TL partly corrupted (the fragments are quoted in the scope Log, L3); what is confirmed is written below, and no point is widened beyond the fragments. Read-only inside any loop (kit `CLAUDE.md` rule 1).

## 1. Unit

- Capability: the `Unhandled` result of a Typed behavior and its interpretation in `pekko-actor-typed/typed/`, with a module-local record. Source: Pekko `Behavior.scala`, `BehaviorImpl.scala`, `Behaviors.scala` and `ActorAdapter.scala` at v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), read, not run. Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`.
- Manifest: `manifest.tsv` follows the kit runner contract (`source` and `target` are the first two columns, the only ones it reads; the other five are the disposition record). Moved and `PORT-GAP` rows point at this RULEBOOK.
- Writable per PR: exactly the "Allowed files" of the scope (section 6) for that PR. Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum`.

## 2. Dispositions

- Go targets: `behavior.go`, `interpreter.go`, `signal.go` (production) and `unhandled_internal_test.go` (U1-U7, U9; no `Ported from` line, it derives from no single Pekko file). U8 is the existing tests, run unchanged.
- **moved**: publication of `UnhandledMessage` (`ActorAdapter.scala:228-229`, `Actor.scala:656-660`, the probe parts of `ActorContextSpec`, `AskSpec`, `ActorContextDelegateSpec`) goes to Slice B / #897; `Empty` and `Ignore` (`Behavior.scala:297-306`) go to #1181.
- **PORT-GAP**: `BehaviorSpec.scala:256-262` (BehaviorTestKit and the internal predicate `isUnhandled`; Go has neither). U1 covers the observable behavior as READ; no coverage of the predicate is claimed.

## 3. Evidence labels

- Nothing was executed against Pekko. There is no Typed oracle (not V3). No parity claim. **The evidence level of every row is READ** (alternative evidence **pending Owner approval, ADR 0033 D13**). Two kinds of row are told apart: Go-only assertions with no Pekko oracle (U3, U7: READ, no equivalence claimed) and the contract check of the public API (U9: READ, a diff of the exported surface, not a behavior row). The specs publish through an event stream that Go does not have, so Go-test equivalence with a spec is not claimed.

## 4. Rules for PR 3 (the later Dev)

- **Decisions confirmed by the Owner on 2026-10-28 (dated; replaces the start block of the first version of this package).** **U-2 (a):** the local record is a per-instance injection point with **no new public API**, whose default is a no-op that stores nothing and publishes nothing; it is the mechanism of ADR 0034 P7, and D-B3's rejection of a production injection point does not apply to this case (the phrase "D-B3 vigencia" is **read** that way, scope section 11.1). **U-4:** `Unhandled` returned for `Terminated` takes the death pact without consulting the Classic decider; this decides neither `ChildFailed` nor `MessageAdaptionFailure`. **U-7:** B-4, X4, X6, EVT-1 and the publication (#897) stay pending; the wording is "not published yet", never "deferred capability" or "intentional difference". The hook is still a production injection point (a seam): it is unexported, has the no-op default, and is removed when the event stream exists. PR 3 and PR 4 are authorized by the Owner in that order, **as read** from a partly corrupted message (scope section 11.1); no PR merges without the Owner.
- Internal tests in `package typed`. TDD: each scenario, observe RED, minimum implementation. No `time.Sleep`, no polling, no `-race`; channels and gates only. Cyclomatic complexity at most 10, no `//nolint`.
- **One new export:** `Unhandled[T any]() Behavior[T]`, a sealed value like `Same`. The PR carries its EXP-1 record: symbol, consumer, why the public API is not enough, what it does not expose, its test (U9). No other export; nothing from `package actor`; the hook is unexported.
- **Signals (U-4 as read):** `Unhandled` for `Terminated` takes the existing death-pact path (default Stop, `StopSelf`; the Classic decider is not consulted, count 0, even if a custom decider answers Restart; no Typed-supervision hook or attach point). Any other signal: dropped, nothing recorded. This does not decide `MessageAdaptionFailure` or `ChildFailed`.
- **Initial behavior (U-5):** reuse `ErrInvalidInitialBehavior` without changing its text or adding a sentinel; `errors.Is` must hold. A Setup resolving to `Unhandled` fails the same way.
- **Code points that change:** (i) `turn()` becomes tri-state (handled, unhandled, failed), (ii) `start()` gets an `Unhandled` case before `default`, so it does not report `ErrNilBehavior`, (iii) a `Setup` that resolves to `Unhandled` goes through `start()` inside `turn()`, (iv) `OnTerminated` distinguishes the handler's `Unhandled` from `Same`. Completes the `TODO(TYP-04-edge)` marker left by Slice B.
- A message turn answering `Unhandled` succeeds, keeps the behavior and its signal handler, restarts nothing, is never a `DeadLetter`, and calls the hook once with the message before the next turn. The default hook stores nothing and publishes nothing. Reuse existing state helpers; add no goroutine or lock (B19).
- `Same` is unchanged (still handled for `Terminated`). Update the `ReceiveSignal` comment; the README change is PR 4.
- Unknown behavior: `// TODO(TYP-04-edge): <reason>` and keep going (kit rule 8).
- Headers: keep the `Ported from` lines of edited files; check with `.github/scripts/check-headers.sh`.
- PR budget: at most 400 authored changed lines (additions plus deletions, generated files excluded). Forecast for PR 3 about 330; if projected over, move tests to PR 4.
- Out of scope: Slice B (publication), #1181, #1182, #976, #1250, #1171, the sender (X6).

## 5. Stop conditions

A file outside the allowed list must change; `pekko-actor` needs an export; a scenario fails and is not a Go defect (reason it on Pekko first); the budget would be exceeded. Report to the TL; never skip, delete or weaken a test.
