# Scope: Sender of `Terminated` and of the `WatchWith` message, ADR 0032 D5b (#814, MSG-02, Part 2)

Status: **PROPOSED by the TL (Squad C), 2026-10-29; D5b decided by the Owner (2026-10-28, follow Pekko) and mechanism B1 chosen (2026-10-29). Update 2026-10-29: the RULEBOOK sha256 was approved and the implementation is integrated in `develop` (#1278). T6 `preRestart` after a death pact was later executed on the JVM (see the Part 2 EXECUTION-RECORD). No parity and no V3 are claimed.** Written against `origin/develop` `0ecfc3cff5304c812ca57686b6b365b2cf4c62ad`; Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. Refs #814.
**Kit run required (ADR 0017, ADR 0024 T6, `AGENTS.md`)**: kit package `migration/stage-1/msg-02-terminated-sender/` (RULEBOOK, RUNBOOK, manifest). That RULEBOOK is the normative text (surfaces, invariants, scenarios, gates); this scope states only the result and what changed since the earlier draft.
Part 1 (`docs/scopes/814-failure-sender-decider.md`, the parent's decider) shares no production line with this part.

## 1. Result

`OnTerminated`, the death-pact failure and a `WatchWith` message run with the observed actor as `Sender()`, as in Pekko (`dungeon/DeathWatch.scala:110, 81-91`). Today `deliverRuntime` stores `nil` (`watch.go:504`, `TODO(MSG-02-D5)`), recorded in `README.md` as a difference, not a decision.

Lifetime, as Pekko's `currentMessage` (`ActorCell.scala:553-559`): a new delivery replaces the sender, success clears it, a failure keeps it until the existing `clearCurrent` sites. Other hooks gain no sender.

## 2. Mechanism B1 (chosen)

The runtime builds the sender from the observed incarnation: `idHandle{id: Terminated.Who(), sys: owning System}`, where the owning System is the one `watchReg.owner` already records (nil means this cell's System). The user's `Handle` is not retained. No export, type or conversion is added; the scope does not promise that the sender has `Tell` or that it allows replying without a typed `Ref`. B2 (retain the user's `Handle`) and B3 (public conversion) are not adopted.
Guarantees: the ID of the original incarnation survives a recreation of the name; a foreign subject keeps its own System, so `Watch`, `Stop` and `System.Done` behave on it as on any `idHandle`; the runtime holds an ID and a pointer, nothing more.

## 3. Dependencies

Code: none. The earlier draft sequenced this part after Squad A (#1098, head-insert wiring); that wiring is **postponed** and this scope no longer depends on or anticipates it. Part 1 (#1275) is not needed by the code (`watch.go` only; `callReceive`, `curSender`, `idHandle` are in `develop`). Documentary only: ADR 0032 "D5a and D5b" lives in the Part 1 branch, so the ADR wording change ("B1") and the shared README paragraph wait for it.

## 4. Acceptance, edit surfaces, forecast

T1-T12 with their test names, mutation checks, the Pekko differential (T1, T2 and the death-pact `postStop` observed on the JVM; the rest READ, JVM pending) and the exact edit surfaces are in the kit RULEBOOK sections 2 to 4. Forecast ~290 authored lines for the implementation PR (budget 400).

## 5. Out of scope

Part 1; `Forward`; `DeadLetter.Sender` and `deadLetters`; stash; any Handle-to-Ref conversion; `existenceConfirmed`/`addressTerminated`; remote watchers; typed `Terminated`; sender in other hooks.

## 6. Gates

`AGENTS.md` "Go" (see RULEBOOK section 6); `deliverRuntime` stays at gocyclo 10 or less via a helper. SQLite parity: not applicable.
