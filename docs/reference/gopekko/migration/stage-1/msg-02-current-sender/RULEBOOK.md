# RULEBOOK: Current sender (#814, unit msg-02-current-sender)

Status: **DRAFT, awaiting Owner approval of its sha256.** Read-only inside any loop (kit `CLAUDE.md` rule 1). Authority: `docs/scopes/814-msg-02-current-sender.md` ("Owner authorization", 2026-10-28) and the accepted part of ADR 0032 (D1, D2, D3, D4, D6, D7).

## 0. Unit

- Capability: an actor reads the sender that belongs to the message of the current turn (`Context.Sender`), and a caller attaches one with `TellFrom`. `Tell` and its guarantees do not change.
- Source (Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, read-only; read, not executed):
  - `actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:596-600`: `sender()` returns `currentMessage.sender`, or `deadLetters` when the message or its sender is null.
  - `actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:141, 157`: `tell(msg, sender)`; `noSender` means nobody.
  - `ActorCell.scala:451, 553, 559, 718-719` and `dungeon/FaultHandling.scala:101-111, 247-275, 305-306`: `currentMessage` is replaced at `invoke`, cleared only after a successful invocation and by `clearActorFields`, kept on a failed one and on resume; `preRestart` and `postStop` read the retained value; the parent's `handleFailure` replaces it with the `Failed` envelope whose sender is the failing child.
  - Specs that exercise `sender()` and are the oracle for the scenarios, under `actor-tests/src/test/scala/org/apache/pekko/actor/`: `ActorRefSpec.scala` ("support reply via sender" at line 390, helpers at 37-54 and 83-113), `SupervisorSpec.scala:57` (`sender() != sendTo`), `ActorSystemSpec.scala:205` (`tell("boom", ActorRef.noSender)`).
- Adapted, not ported: Pekko returns `deadLetters` when there is no sender; goPekko returns `(nil, false)` (ADR 0032 D3); `deadLetters` is #896. The `Terminated` and `WatchWith` sender (ADR 0032 D5) is DEFERRED by the Owner and is not part of this unit. The parent decider's sender (Pekko: the failing child) is deferred with D5 and reads `(nil, false)`; it is never the sender of the message the child was processing. A stale `Context` is outside the contract and no detection is promised; `ErrOutsideTurn` is not part of this unit.

## 1. Scenarios

S1 to S6 and S8 to S10 of the scope (section 6) are scenarios; S7 is removed (`ErrOutsideTurn` is not delivered). Section 13.3 of the scope adds F1 to F9 and P1, P2. Each has its test name there and they are not restated here.

Evidence is kept in three separate columns of the execution record, never merged:

- Pekko reading (source read, not executed): the citations above.
- Go evidence: the passing tests and the candidate SHA.
- Pending differential execution: the Classic judge run of F1 to F5 and F7 against Pekko is NOT done; the rows stay `PENDING` until it is run or the Owner approves alternative evidence (ADR 0033 D13).

### 1.1 Current-sender lifetime (follows Pekko `currentMessage`)

- Set when a `Receive` turn starts; cleared when it succeeds; kept when it fails; cleared where the failed message is cleared on restart and on termination, after `PostStop`; never cleared by a resume.
- `PreRestart` and the `PostStop` of a stop after failure or of a restart read the failed message's sender; `PostRestart`, `PreStart` and `PostStop` after a successful turn read absent.
- `ReceivePort`, `OnTerminated` and the `WatchWith` message start with an absent sender (no leak); the `Terminated` and `WatchWith` difference with Pekko stays OPEN (D5).
- `// TODO(MSG-02-clear)` is deleted: the choice it marked is not accepted. New markers: `TODO(MSG-02-resume)`, `TODO(MSG-02-failed)`, `TODO(MSG-02-D5)`, `TODO(MSG-02-port)`.

## 2. Stages

- 2a: internal `envelope[T]{msg, sender}` as the queue element; no new export; existing behavior unchanged.
- 2b: `TellFrom`, `Context.Sender`, `sender_test.go`, README.
- 3: execution record and cost-log row.

## 3. Rules

- The Pekko checkout is read only from `upstream/pekko`; nothing there is edited.
- No scenario is skipped, weakened or marked pending to get green.
- No sleeps and no polling; ordering by channels and gates.
