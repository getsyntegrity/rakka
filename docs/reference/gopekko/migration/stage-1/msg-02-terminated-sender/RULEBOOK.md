# RULEBOOK: Sender of `Terminated` and of the `WatchWith` message, option B1 (#814 Part 2, unit msg-02-terminated-sender)

Status: **DRAFT, awaiting Owner approval of its sha256 (the only open decision).** Read-only inside any loop (kit `CLAUDE.md` rule 1). Authority: `docs/scopes/814-terminated-watchwith-sender-d5.md`; ADR 0032 "D5a and D5b" (Owner, 2026-10-28; B1 chosen 2026-10-29). Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, read-only. Base: `origin/develop` `0ecfc3c`.

## 0. Unit

- Capability: `OnTerminated`, the death-pact failure and a `WatchWith` message run with the observed (terminated) actor as `Context.Sender()`, as in Pekko. Nothing else gains a sender.
- Source, under `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/`:
  - `dungeon/DeathWatch.scala:110`: `self.tell(Terminated(actor)(...), actor)`, the observed actor is the sender.
  - `dungeon/DeathWatch.scala:81-91`: `receivedTerminated` delivers only a `Terminated` with a `terminatedQueued` entry; for `WatchWith` it does `currentMessage.copy(message = customTermination)`, so the envelope (and sender) is kept.
  - `ActorCell.scala:553-559`: `invoke` sets `currentMessage` to the delivery, clears it after success; a failure leaves it. `:718-719` `clearActorFields`; `FaultHandling.scala:101-111, 247-275` `preRestart`/`postStop` read it.
- Adapted, not ported: Pekko's sender is an `ActorRef`; goPekko's is an identity `Handle` (ADR 0032 D2). "No sender" stays `(nil, false)`.

## 1. B1: what the sender is

The runtime builds, when the delivery is taken, `idHandle{id: it.who, sys: S}` (`hierarchy.go:75`) where `S` is the owning System of the observed actor: `it.owner` when not nil (subject of another System), else `c.sys`. `done` stays nil.

Invariants (each is an acceptance criterion in section 3):

- I1 Incarnation: the ID is `Terminated.Who()`, fixed when the watch was registered. It is never looked up by name or path afterwards, so a recreated actor under the same name never aliases it.
- I2 System: a foreign subject's handle carries the foreign System; `Watch`, `Unwatch`, `System.Stop` on it resolve there by `ID()` (`watch.go` `resolve`) exactly as for a user handle of that actor; for a retired subject they treat it as "a valid reference to nothing".
- I3 No user value: the `Handle` passed to `Watch`/`WatchWith` is not stored (not in `watchReg`, `termEntry` or `laneItem`). Only `owner *System` is copied, from the existing `watchReg.owner`.
- I4 Retention: the handle is held only in `curSender` (cell.go:35), with the lifetime of Pekko's `currentMessage`: replaced by the next delivery, cleared by success, kept by failure, cleared at the `clearCurrent` sites. It holds an `ID` and a `*System`, never a cell, an actor instance or a done channel.
- I5 Existing operations: `System.Done(sender)` answers as for any `idHandle` without a done channel: `ErrUnknownHandle` and a closed channel once the registry retired the actor (`system.go:205-221`). This is a recorded limitation, not parity.
- I6 `Terminated` and `Terminated.Who()` are unchanged; `ReceivePort`, other hooks (`PreStart`, `PostRestart`, `PostStop` of a normal stop) get no new sender.

## 2. Edit surfaces (exact; the writer touches nothing else)

| Path | Symbols |
|---|---|
| `pekko-actor/actor/watch.go` | `termEntry[T]` (add `owner *System`); `terminatedQueuedForLocked` (set it from `reg.owner`); `laneItem[T]` (add `owner *System`); `laneReadyLocked` (copy `e.owner`); `deliverRuntime` (replace the `curSender.Store(nil)` line and its `TODO(MSG-02-D5)`; clear the sender after `OnTerminated` succeeds; WatchWith: `callReceive(envelope[T]{msg: it.msg, sender: h})`); one new helper `terminatedSender(it laneItem[T]) Handle` |
| `pekko-actor/actor/sender_terminated_test.go` | new file, tests T1-T11 |
| `pekko-actor/README.md` | the "Sender (#814 ...)" paragraph: the `OnTerminated`/`WatchWith` sentence and the D5 bullet (rebase on Part 1's edit of the same paragraph) |
| `docs/adr/0032-local-sender-envelope.md` | only the "B1 versus B2 pending Owner" wording (status line, D5b text, row D5b) becomes "B1, 2026-10-29"; needs Part 1's text of D5a/D5b to exist in `develop` |
| `migration/stage-1/msg-02-terminated-sender/EXECUTION-RECORD.md`, `migration/cost-log.tsv` | evidence bound to the SHA; one row |

## 3. Scenarios and acceptance (test file `sender_terminated_test.go`; go-specs matchers and `mock.Controller`/Spy, public API only)

| Id | Criterion | Test (func) | Pekko evidence |
|---|---|---|---|
| T1 | `OnTerminated` reads the observed actor; `Sender().ID() == Who()` | `TestTerminatedSender_OnTerminatedReadsObserved` | READ + JVM T1 |
| T2 | `WatchWith` message reads it | `..._WatchWithReadsObserved` | READ + JVM T2 |
| T3 | no leak: sender X retained by a failed `Receive` (Resume) is not read | `..._ReplacesRetained` | READ (`:553`); JVM PENDING |
| T4 | success clears: `PostStop` after a successful `OnTerminated` reads absent | `..._SuccessClears` | READ (`:559`); JVM PENDING |
| T5 | failure keeps: `OnTerminated` fails, Restart: `PreRestart` reads it, `FailedMessage` is the `Terminated`; `PostRestart`/new instance absent | `..._FailureKeepsUntilRestart` | READ; JVM PENDING |
| T6 | death pact (no `OnTerminated`): `PostStop`/`PreRestart` read it | `..._DeathPactKeeps` | READ + JVM T3 (`postStop`) |
| T7 | next `Receive` replaces: `Tell` reads absent, `TellFrom(Z)` reads Z | `..._NextReceiveReplaces` | READ |
| T8 | foreign System subject (I2): `ID()` is its ID; `Watch(sender)`, `Stop(sender)` accepted (no `ErrInvalidHandle`) and act on that System | `..._ForeignSystem` | none (local-only construct; READ `resolve`) |
| T9 | recreation (I1): stop child, spawn same name; first sender keeps the old ID; `Stop(sender)` does not stop the new child | `..._RecreationKeepsOldID` | none in JVM (`uid`, READ `ActorRef.scala:183-186`) |
| T10 | unwatch before delivery: nothing delivered, sender unchanged | `..._UnwatchDeliversNothing` | READ `DeathWatch.scala:75` |
| T11 | I3/I5: a user handle is not retained (the `Watch`ed handle is a spy never read by the runtime afterwards); `Done(sender)` after retirement is `ErrUnknownHandle` | `..._NoUserHandle`, `..._DoneAfterRetire` | n/a |
| T12 | regression: `Watch`/`WatchWith`/`Unwatch` conflict, order, capacity (`watch.go:357`): existing tests green; differential vs a base copy in `mktemp -d` | existing suite | n/a |

Mutation checks (each must turn a named test red): keep `curSender.Store(nil)` (T1, T2); no clear on success (T4); `sys` always `c.sys` (T8); resolve the ID by name (T9); store the user's handle (T11); sender taken from the child (T1).

## 4. JVM differential: observed and missing

Observed 2026-10-28 on Pekko v1.7.1 (`pekko-repro/observed-T.txt`, copied from `origin/squad-c/814-kit-and-jvm-evidence` `migration/stage-1/msg-02-failure-sender/pekko-repro/observed.txt`, sha256 `c9b36c69...`; probe `Probe.scala` in the same directory): T1 `Terminated` sender `victim-T1`; T2 `WatchWith` sender `victim-T2`; T3 `postStop` after the death pact `victim-T3`.
Missing (PENDING, not run): T3, T4, T5, T7 of section 3 as JVM observations and incarnation identity (T9). The Dev extends `Probe.scala` for the first four in the kit run if the Owner authorizes another JVM run; otherwise they stay READ + Go evidence, labelled so (ADR 0033 D13). T8 has no Pekko counterpart (Go-only; manifest row `not applicable`).

## 5. Forbidden

- Any export, type, method or conversion (`Terminated.Ref/Handle`, Handle-to-Ref, promises that the sender has `Tell` or can be replied to). Zero public-API diff (`go doc` before/after).
- Storing the user's `Handle`; storing a `*cell`, record or `done` channel in `termEntry`/`laneItem`.
- Editing `cell.go`, `ref.go`, `actor.go`, `port.go`, `supervision_*.go`, `stop.go`, `internal/`, other modules; setting a sender in other hooks.
- Integration with Squad A (#1098, postponed): no head-insert wiring and no reading of its scope. Rebase conflicts in `watch.go` are resolved when #1098 lands, not anticipated.
- Skipping or weakening a test; `//nolint`; sleeps or polling; `-race`; Actions.

## 6. Gates

`AGENTS.md` "Go": gofmt, vet, `scripts/each_module.sh go test -count=1 ./...` (unit tests, no network), coverage 85 and diff coverage 85, golangci-lint v2.14.0 and gocyclo 10 (`deliverRuntime` must not grow: use the helper), `scripts/check_deps.sh`, `make parity-guard`, `make ci-fast`. PR budget 400 authored lines (forecast ~290). SQLite parity: not applicable.

## 7. Real dependencies

Code: none on Part 1 (#1275) or Squad A (#1098); this unit edits only `watch.go` and reads `idHandle`, `callReceive`, `curSender` that are already in `develop`. Documents only: ADR 0032 "D5a and D5b" and the scope `docs/scopes/814-failure-sender-decider.md` (Pekko map `m#`) live in the Part 1 branch; section 2 (ADR row) waits for them. The README paragraph is shared with Part 1 (textual rebase).

## 8. Owner decision

Only the approval of the sha256 of this file. B1 itself is already decided.
