# RULEBOOK: Sender of the parent's decider and of PreRestart after an escalation (#814, unit msg-02-failure-sender)

Status: **DRAFT, awaiting Owner approval of its sha256 and of ADR 0032 D5a.** Read-only inside any loop (kit `CLAUDE.md` rule 1). Authority: `docs/scopes/814-failure-sender-decider.md`. **Implementation does not start until the Owner answers D5a** (scope section 6): the accepted ADR 0032 (D1, D2, D3, D4, D6, D7) does not name the sender of a `Failed` delivery.

## 0. Unit

- Capability: while a parent handles the failure of a child, `Context.Sender()` returns the failing child's identity `Handle`, as Pekko's `sender()` does there. It is never the sender of the message the child was processing.
- Source (Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, read-only; read, not executed), paths under `actor/src/main/scala/org/apache/pekko/actor/`:
  - `dungeon/FaultHandling.scala:305-306`: `handleFailure` sets `currentMessage = Envelope(f, f.child, system)` before the child lookup and before the strategy runs.
  - `dungeon/FaultHandling.scala:313-314` and `:219-225`: `Escalate` throws the cause; `handleInvokeFailure` reads the sender of the `Failed` envelope to choose the child it does not suspend again.
  - `dungeon/FaultHandling.scala:101-111, 247-275` and `ActorCell.scala:553-559, 596-600, 718-719`: `preRestart` and `postStop` read the retained value; it is cleared after them and on success; the next `invoke` replaces it.
  - `FaultHandling.scala:366-388` (strategy file `actor/FaultHandling.scala`): the decider takes only the cause.
  - Oracle spec that reads `sender()` in a decider: `actor-tests/src/test/scala/org/apache/pekko/actor/SupervisorSpec.scala:84-92`.
- Adapted, not ported: Pekko returns an `ActorRef`; goPekko returns an identity `Handle` (ADR 0032 D2) built from the child's ID (the value `ChildFailure.Child` already carries). Pekko's `deadLetters` for "no sender" stays `(nil, false)` (D3, #896).
- Not part of this unit: `Terminated`, `WatchWith`, `OnTerminated` (D5b, other unit); `Forward`; `DeadLetter.Sender`; `ErrOutsideTurn`; stale-`Context` detection; any Handle-to-Ref conversion.

## 1. Scenarios

G1 to G12 of the scope (section 4); each has its test name there. Evidence is kept in three separate columns of the execution record, never merged: Pekko reading (READ), Go evidence (passing tests and the candidate SHA), pending differential execution (PENDING until the probe of scope section 8 is run or the Owner approves alternative evidence, ADR 0033 D13).

### 1.1 Lifetime (unchanged from the msg-02-current-sender RULEBOOK 1.1, restated for this path)

- Set in `handleFailed` before the strategy and decider run, also for a stale child; not cleared when `handleFailed` returns; replaced at the start of the next turn; cleared at the existing `clearCurrent` sites.
- `// TODO(MSG-02-failed)` is deleted by this unit. No new marker is introduced.

## 2. Stages

- 1: this package (docs). 2: `supervision_failure.go` (one line), tests, README. 3: execution record and cost-log row.

## 3. Rules

- The Pekko checkout is read only from `upstream/pekko`; nothing there is edited.
- No scenario is skipped, weakened or marked pending to get green; `F8` is rewritten, not removed.
- No sleeps and no polling; ordering by channels and gates. No `-race`, no Actions.
