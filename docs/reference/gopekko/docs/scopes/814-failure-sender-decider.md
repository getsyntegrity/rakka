# Scope: Sender of the parent's decider and of PreRestart after an escalation (#814, MSG-02, Part 1)

Status: **PROPOSED by the TL (Squad C), 2026-10-29. D5a decided by the Owner on 2026-10-28: `d5a_accept` (follow Pekko); recorded in ADR 0032 "D5a and D5b". The RULEBOOK of the kit package has NO explicit Owner approval with hash yet and implementation has NOT started (G2 still pending).** Written against `origin/develop` `0ecfc3cff5304c812ca57686b6b365b2cf4c62ad` (`TellFrom` and `Context.Sender` are integrated by #1268 and #1270 and are not reimplemented here).
Pekko baseline: v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (submodule HEAD verified equal to the gitlink). Pekko paths are under `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/`.
Evidence labels: **READ** = source read, not executed; **GO** = goPekko source read at the base above; **PENDING** = differential JVM execution not run. G1, G2, G4 and G6 were run on 2026-10-28: see section 8 and `migration/stage-1/msg-02-failure-sender/pekko-repro/`.
Part 2 (ADR 0032 D5: `Terminated` and `WatchWith`) is a separate, independently reviewable unit: `docs/scopes/814-terminated-watchwith-sender-d5.md`. This file also holds the verified map for both parts (section 2).
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: this unit adapts `FaultHandling.scala` `handleFailure`. Package: `migration/stage-1/msg-02-failure-sender/` (manifest, RULEBOOK, RUNBOOK; RULEBOOK is DRAFT awaiting the Owner's SHA-256).

## 1. Result and whether it is developable now

Result: **code that runs in the parent while it handles the failure of a child (the decider, and whatever runs after it on the same retained delivery, notably `PreRestart` after an escalation) reads the failing child as `Sender()`, as Pekko does. It never reads the sender of the message the child was processing.**

Today (GO): `handleFailed` stores `nil` (`supervision_failure.go:256-258`, marker `TODO(MSG-02-failed)`), so the decider and a `PreRestart` after an escalation read `(nil, false)`; Pekko reads the child (`dungeon/FaultHandling.scala:306`).

**Is it covered by the accepted ADR 0032? NO, not by the accepted text. Decision of the TL, with evidence:**

| Check | Evidence | Result |
|---|---|---|
| Accepted decisions | `docs/adr/0032-local-sender-envelope.md:3-4`: D1, D2, D3, D4, D6, D7 accepted; D5 open | none names the sender of a `Failed` delivery |
| What D1 allows the runtime to set | ADR "Envelope and ownership" table, Sender row: "the caller, through the send function; or the runtime, for `Forward` (P6)" | the runtime setting a sender for a failure is not listed |
| D5 text | ADR table D5: "Sender of runtime deliveries (`Terminated`, the `WatchWith` message)" | names two deliveries; `Failed` is a runtime-originated delivery of the same class but is not named |
| Where the decider was deferred | `pekko-actor/README.md:646` and `RULEBOOK.md` (retrospectively accepted for #1268 only) defer the decider "with D5" | the deferral exists in README/RULEBOOK, not in the ADR, and no Owner decision resolves it |
| D3 (absent is explicit) | accepted | covers only "no sender given", not "the runtime gives one" |

So the part is **not developable on accepted text alone**. It becomes developable with one Owner answer: **D5a** (section 6), a carve-out that decides only the `Failed` delivery. If the Owner reads D5 as already excluding `Failed` (so the Pekko-equal behavior needs no decision), the Owner says so and D5a is not needed; the TL does not assume that. No implementation starts before that answer. The Part 1 work needs **no new export and no new type** (section 7), so the amendment is the only gate.

## 2. Verified map (READ unless marked; both parts)

### 2.1 Pekko: `currentMessage`

| # | Fact | Cite |
|---|---|---|
| m1 | `var currentMessage: Envelope`, null at start | `ActorCell.scala:451` |
| m2 | `sender()` = `currentMessage.sender`; `deadLetters` when the message or its sender is null | `ActorCell.scala:596-600` |
| m3 | `invoke(envelope)`: `currentMessage = messageHandle` (replaces) before processing, also for auto-received messages (`Terminated`, `PoisonPill`) | `ActorCell.scala:553-557` |
| m4 | After a successful invocation: `currentMessage = null` | `ActorCell.scala:559` |
| m5 | On exception: `handleInvokeFailure`, no assignment, so the failed envelope stays current | `ActorCell.scala:560-562` |
| m6 | `clearActorFields` sets `currentMessage = null`; called after `preRestart` (in `finally`), after `postStop`, and before a failing `finishRecreate` reports | `ActorCell.scala:716-720`; `FaultHandling.scala:110, 273, 300` |
| m7 | `faultRecreate` reads `currentMessage` to build the `preRestart` message argument | `FaultHandling.scala:101, 104` |
| m8 | `faultResume` does not touch it | `FaultHandling.scala:136-153` |
| m9 | A `Failed` system message calls `handleFailure(f)` | `ActorCell.scala:521` |
| m10 | `handleFailure`: `currentMessage = Envelope(f, f.child, system)` **before** the child lookup and before the strategy runs; message = the `Failed`, **sender = the failing child**; not cleared afterwards. The assignment happens even if the child is unknown or the `uid` is stale (the failure is then dropped with a Debug entry) | `FaultHandling.scala:305-324` (assignment at 306) |
| m11 | The strategy gets `(context, child, cause, stats, children)`; `Escalate` returns false, then `throw f.cause` | `FaultHandling.scala:313-314` (`throw f.cause` at 314) and `actor/FaultHandling.scala` (the strategy file) `:366-388` |
| m12 | The decider is `decider.applyOrElse(cause, escalateDefault)`: it takes only the cause; user code in it can call the actor's `sender()` | `FaultHandling.scala:372` (strategy file) |
| m13 | `handleInvokeFailure` after the escalation's `throw`: `currentMessage match { case Envelope(Failed(_,_,_), child) if !isFailed => setFailed(child); Set(child) ... }`. The escalating parent uses the **sender of the Failed envelope** to know which child not to suspend again | `FaultHandling.scala:219-225` |
| m14 | `Terminated` is told with the watched actor as sender | `dungeon/DeathWatch.scala:110` |
| m15 | `receivedTerminated`: with a `watchWith` message, `currentMessage = currentMessage.copy(message = customTermination)`: the sender (the watched actor) is kept, the message is replaced | `dungeon/DeathWatch.scala:81-91` (copy at 87) |
| m16 | The `Terminated` is accepted only if `terminatedQueued` holds "the SAME ref which was put in"; `ActorRef` equality is `path.uid == other.path.uid && path == other.path` | `DeathWatch.scala:82-83`; `ActorRef.scala:183-186` |

### 2.2 Pekko: who is the sender, by path

| Path | `sender()` in the code that runs | Cite |
|---|---|---|
| Child fails, parent's decider runs | **the failing child** (never the sender of the child's own message: that envelope lives in the child's cell) | m10, m12 |
| Parent decides Escalate, then the grandparent decides Restart, parent's `preRestart` runs | **the failing child** (the `Failed` envelope stays current through m5, m8, m13, then m7) | m10, m13, m7 |
| The same, `preRestart` receives | `Some(Failed(child, cause, uid))` as message, the same envelope whose sender is read | m7 |
| After the restart (`postRestart`, new instance) | absent (`deadLetters`): cleared by m6 | m6 |
| Parent stopped (grandparent Stop) after the escalation | `postStop` reads the child (cleared after, m6); same retained value | m6, m10 |
| Parent handled a child failure with Resume or Restart of the child, no other message since | `currentMessage` is still the `Failed` envelope: a later `postStop` or the lane of a restart reads the child | m10 (not cleared) |
| Next mailbox message | replaced by that envelope (m3) | m3 |
| A second `Failed` while the parent is suspended | replaced by the second child | m10 |
| `Terminated` delivered to a watcher | **the watched actor** | m14 |
| `WatchWith` message | **the watched actor** (the `Terminated` envelope reused) | m15 |
| `Terminated` not handled (death pact) | the watched actor (the envelope is current when `DeathPactException` is thrown; stays current, so `preRestart` reads it) | m3, m5 |

### 2.3 Pekko: references and identity

- Retention: the `Envelope` holds the `ActorRef` strongly for as long as it is `currentMessage`: until the next `invoke`, a successful invocation (m4) or `clearActorFields` (m6). A parent that handled a failure and then idles keeps the child's ref until its next message (READ). `terminatedQueued` and `watching` hold the watched ref until the death is processed or `unwatch` (`DeathWatch.scala:31, 75, 117-119`).
- Identity under termination and recreation: equality is `path + uid` (`ActorRef.scala:183-186`). A restart (Recreate) keeps the same `ActorRef` and `uid`, emits no `Terminated`, and the `Failed` carries `uid` so a `Failed` from an old incarnation of the same path is dropped (`stats.uid == f.uid`, `FaultHandling.scala:313`; the comment at 308-312 names this case). A stop followed by a new child under the same name has a new `uid`, so its ref is not equal to the old one although the path is equal. `Terminated(actor)` carries the ref of the terminated incarnation.

### 2.4 goPekko (GO, base `0ecfc3c`)

| Fact | Cite |
|---|---|
| `Context.Sender()` reads `cell.curSender` (atomic pointer to a box) | `actor.go:53-71`; `cell.go:33-35` |
| `setSender(h)` stores the box, nil stores nil | `cell.go:309-316` |
| `clearCurrent()` clears `failedMsg` and `curSender`; called after `PreRestart` (`faultRecreate`), at `releaseTerminated` after `PostStop`, and at three restart/start-failure sites | `cell.go:318-323`; `supervision_restart.go:283, 354, 522, 563`; `stop.go:141`; `start_failure.go:171` |
| `callReceive`: sets the sender at turn start; clears on success; keeps on failure | `cell.go:326-358` |
| `handleFailed`: derives `child` (`rec.identity()` for a registered child, `idHandle{id, sys}` for a stale one), stores `curSender = nil` and `failedMsg = ChildFailure{child, cause}`, **then** returns if the child is not registered | `supervision_failure.go:251-262` |
| The decider is called as `st.decide(c.ctx, rec.identity(), f.cause)` | `supervision_failure.go:272` |
| Escalation: `failParent(..., skip=child)` and `markPerpetrator(skip)` (Pekko `setFailed(child)`, m13) | `supervision_failure.go:212-241`, `327` (Escalate) and `343-348` |
| PreRestart message is `c.failedMsg` (the `ChildFailure`) | `supervision_restart.go:318-327` |
| A suspended cell does not run user or lane items; `failed` items are handled even suspended | `cell.go:141-151` (drain) |
| `Handle` identity: `ID` is unique per incarnation (`nextID`, `id.go:17`); a restart keeps the cell and its ID; a new spawn gets a new ID | `id.go`; `hierarchy.go:75-85` |
| `rec.identity()` returns `idHandle{id, sys, done}`: no cell, no actor instance | `cell.go:96` |

Reading of the two models: goPekko's `curSender` + `failedMsg` already mirror m1-m8. The single divergence on this path is the `nil` store at `supervision_failure.go:258`.

## 3. Contract (minimum)

1. While `handleFailed` runs for a failure reported by child `C`, **before** the strategy and decider run, `curSender` becomes the identity of `C` (the same `Handle` value that is stored in `ChildFailure.Child`). This is done also when `C` is not registered (stale), because Pekko replaces `currentMessage` before the lookup (m10).
2. It is **not** cleared when `handleFailed` returns (m10: not cleared). It is replaced at the start of the next `Receive` (`setSender`), of `ReceivePort` and of the Terminated turn (their existing resets, which Part 2 changes for Terminated only), and cleared at the sites that already call `clearCurrent` (m6).
3. `ctx.Sender()` inside the decider returns `(C, true)` with `C.ID()` equal to the `child` argument of the decider. It is never the sender of the message `C` was processing, and never the sender retained from this cell's own earlier failed message.
4. After an escalation, the parent's `PreRestart` reads `Sender()` = `C` and `FailedMessage` = `ChildFailure{C, cause}`; `PostRestart` and the new instance read absent; the `PostStop` of a Stop decision reads `C`.
5. Nothing else changes: `Tell`, `TellFrom`, `Sender` for `Receive`, `PreRestart` after a failed `Receive`, `OnTerminated` and `WatchWith` (Part 2) keep their current behavior.

## 4. Acceptance criteria and evidence

Public API only, go-specs, channels and gates for ordering, no sleeps, no polling, no `-race`. New tests go in `pekko-actor/actor/sender_failure_test.go` (package `actor_test`, as F1-F9). Evidence per row: GO test; READ cite; PENDING differential until the probe of section 8 is run or the Owner approves alternative evidence (ADR 0033 D13).

| Id | Criterion | Scenario | READ |
|---|---|---|---|
| G1 | Decider sees the failing child | child C fails on `TellFrom("boom", X)`; the decider calls `ctx.Sender()` | m10, m12 |
| G2 | Not the child's message sender | same; assert the decider's sender `ID()` is C's and **not** X's | m10 |
| G3 | Not the parent's own retained sender | parent had failed earlier on a message from Y and resumed; then C fails; the decider reads C, not Y | m10, m8 |
| G4 | PreRestart after escalation | `Escalate`, grandparent `Restart`; the parent's `PreRestart` reads `(C, true)` and `FailedMessage` is a `ChildFailure` for C | m13, m7, m10 |
| G5 | New instance after that restart | `PostRestart`/`PreStart` of the new instance read `(nil, false)` | m6 |
| G6 | PostStop after escalation + grandparent Stop | reads `(C, true)`; a later idle PostStop (no failure) reads absent | m6, m10 |
| G7 | Second child failure replaces | two children fail while the parent is suspended; the second decider call reads the second child | m10 |
| G8 | Next `Receive` replaces | after a handled failure, `Tell("next")` then `TellFrom("m", Z)`: `next` reads absent, `m` reads Z, never C | m3 |
| G9 | Stale child | a `Failed` from a child that is no longer registered is dropped (no decider, no counter, no event: unchanged), and `Sender()` in a later `PostStop` reads that child's ID | m10 (assignment before lookup) |
| G10 | AllForOne | the decider of an `AllForOne` strategy reads the failing child, not a sibling | m10 |
| G11 | Regression F8 | existing F8 is rewritten: it asserted only "not the retained sender"; it now asserts "is the failing child" | m10 |
| G12 | Shared-code regression | one test per touched existing behavior: `OneForOne` Restart/Resume/Stop/Escalate decisions, counters and events identical to the base (differential in a `mktemp -d` copy of the base commit: stdout, stderr, exit code) | AGENTS.md test policy |

Mutation checks the Dev reports (each reinstated bug is caught): leave the `nil` store (G1, G4 fail); set the sender after the early return (G9 fails); use the child's message sender (G2 fails); clear in `handleFailed` on return (G4, G6 fail).

## 5. Out of scope

`Terminated`, `WatchWith`, `OnTerminated` (Part 2 and D5b); `Forward` (#854); `DeadLetter.Sender` and `deadLetters` (#896); general validation of `Context`; `ErrOutsideTurn`; stale-`Context` detection; any Handle-to-Ref conversion; stash; remote senders; typed API (`pekko-actor-typed` is not touched); `ChildFailure`'s shape; any change to the decider signature (`Decider`, `DeciderFunc` keep `child Handle`).

## 6. D5a: decided by the Owner, 2026-10-28

**Decided: `d5a_accept` (follow Pekko). The text below is now in ADR 0032 ("D5a and D5b"); the `d5a_absent` and `d5a_defer` rows are historical.** Mechanism criteria (incarnation identity, retention and release, the decider never uses the child's message sender, no contamination between turns) are ADR 0032 "Mechanism acceptance criteria" and will be covered by future Go tests G1-G8 and G11; the current JVM probe does not cover incarnation identity. Original proposal: It is also written as a diff to ADR 0032 in `docs/scopes/814-terminated-watchwith-sender-d5.md` section 5, together with D5b, so both can be answered in one pass; D5a and D5b are independent answers.

> **D5a (proposed).** When a parent handles the failure of a child, the child is the sender of that delivery for as long as it is the cell's current delivery, as in Pekko (`dungeon/FaultHandling.scala:306`). The decider, an escalation and the `PreRestart` that follows it read the failing child as `Sender()`. The sender of the message the child was processing is never read there. The sender is the child's identity `Handle` (the value `ChildFailure.Child` already carries); ADR 0032 rules 1 to 4 and 6 apply to it unchanged, and D3 (absent is explicit) still governs every other step.

Closed single-select for the Owner (answer tokens `d5a_accept`, `d5a_absent`, `d5a_defer`):

| Answer | Effect |
|---|---|
| `d5a_accept` | Part 1 is developable as specified here. |
| `d5a_absent` | "Absent" (today's `nil`) becomes the decided contract for the decider and the escalation `PreRestart`: a classification against Pekko, so ADR 0033 D13 applies (alternative evidence plus explicit Owner approval). Part 1 reduces to documenting and pinning today's behavior. |
| `d5a_defer` | Nothing changes; `TODO(MSG-02-failed)` stays. |

## 7. Surfaces and identities (EXP-1)

No export, no new type, no new field. The child's identity already exists as an unexported-constructor `Handle`: `rec.identity()` for a registered child and `idHandle{id: f.child, sys: c.sys}` for a stale one, both computed in `handleFailed` as `child` before the early return. **EXP-1: nothing to justify; no symbol becomes public.** `ChildFailure.Child` and the decider's `child` argument stay as they are.

### Edit surfaces (Part 1, exact)

| Path | Symbol | Change | Lines (add+del) |
|---|---|---|---|
| `pekko-actor/actor/supervision_failure.go` | `(*cell[T]).handleFailed` | replace `c.curSender.Store(nil)` (line 258) by `c.setSender(child)`; delete the `TODO(MSG-02-failed)` marker and fix the comment; nothing after the early return changes | ~5 |
| `pekko-actor/actor/sender_failure_test.go` | `TestSenderDuringFailures` (F8 rewritten) + new G1-G10 | tests | ~170 |
| `pekko-actor/README.md` | "Sender (#814 ...)" paragraph, lines 633-652 | decider and escalation `PreRestart` now read the failing child; D5 sentence loses the decider | ~12 |
| `migration/stage-1/msg-02-failure-sender/` | manifest, RULEBOOK, RUNBOOK (new, docs); later `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row | kit package | docs, excluded |

Nothing in `cell.go`, `mailbox.go`, `watch.go`, `stop.go`, `port.go`, `ref.go`, `actor.go`. Forecast: **~190 authored changed lines, one PR (budget 400).** Commit: `feat(actor): expose the failing child as sender to the parent's decider (Refs #814)`.

### Shared points with Squad A (#1098, PushFront wiring in the cell)

| File | Part 1 touches it | Squad A likely touches it | Resolution |
|---|---|---|---|
| `cell.go`, `mailbox.go`, `internal/mailbox` | no | yes (cell wiring, head insert) | no overlap |
| `watch.go` (lane, `deliverRuntime`, `laneItem`) | no | possible (head-inserted deliveries, the lane) | none for Part 1; **Part 2 does collide** (see Part 2 scope) |
| `supervision_failure.go` | yes, one line in `handleFailed` | unlikely (supervision) | whoever merges second rebases; a one-line conflict |
| `pekko-actor/README.md`, `sender_failure_test.go` | yes | README possibly | rebase by content |

Sequencing (updated 2026-10-29): Part 1 went first, as proposed. The wiring of Squad A (#1098, head insert) is **postponed**, and Part 2 (the Terminated/WatchWith sender, `watch.go`) is already integrated in `develop` with no code dependency on #1098; the earlier text "Part 2 after Squad A merges" no longer holds.

## 8. Differential execution (run 2026-10-28)

Premise: Pekko's source already decides every row (READ). Execution only turns READ into firm evidence for G1, G2, G4, G6. Proposed minimal command, with a probe in the existing judge harness style (disposable copy, never `upstream/pekko`):

```sh
c=$(mktemp -d) && git -C upstream/pekko archive v1.7.1 | tar -x -C "$c" \
 && judge/run-pekko.sh "$c" "$(mktemp -d)" controlled 10000 <probe-scenario>
```

Probe (Scala, to be added to the copy's `judge` project, **not compiled or run by the TL**): a parent with `OneForOneStrategy() { case _ => println(("decider-sender", sender())); Escalate }` under a grandparent with `Restart`; the parent overrides `preRestart` to print `sender()` and `message`; the child fails on a message told by a named probe actor. Expected by READ: both prints show the child; the child's message sender never appears. Cost: one JVM run of one scenario (sbt cold start dominates). **Run** under the Owner's authorization of 2026-10-28 (differential only): the decider read `child-c`, `preRestart` read `child-c` with `Some(Failed(child-c, boom, uid))`, `postStop` after a Stop read `child-c`; the child's message sender never appeared. Source reading and execution agree. Probe, command, environment, exit code and output: `migration/stage-1/msg-02-failure-sender/pekko-repro/` (`RUN.md`, `Probe.scala`, `observed.txt`).

## 9. Definition of Done

- G1-G12 pass, bound to a tested candidate SHA; `TODO(MSG-02-failed)` deleted.
- Gates (section 10) green; differential check shows no unrequested difference.
- README states: decider and escalation `PreRestart` read the failing child; D5 (Terminated, WatchWith) still open if Part 2 is not delivered.
- No claim of module parity; Part 2 and #896 not claimed.

## 10. Gates for the Dev (`AGENTS.md` "Go")

Toolchain `.go-version` = `1.27.2`; lint `.golangci-version` = `v2.14.0` (build to `bin/`, check `bin/golangci-lint version`; run from `bin/`, not `PATH`).

```sh
scripts/each_module.sh go test -count=1 ./...                  # or: unshare -rn <same> where available
scripts/each_module.sh gofmt -l .   # must print nothing; go vet ./... per module
export ROOT="$PWD" COV="$(mktemp -d)"
scripts/each_module.sh bash -c 'set -euo pipefail; go test -count=1 -covermode=atomic -coverprofile="$COV/$("$ROOT/scripts/check_coverage.sh" --profile-name "$MODULE_PATH")" ./...'
scripts/check_coverage.sh "$COV" 85
GOBIN=$PWD/bin go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@$(<.golangci-version)
(cd pekko-actor && ../bin/golangci-lint run --config ../.golangci.yml ./...)   # gocyclo max 10, no //nolint
scripts/check_deps.sh && make parity-guard
make ci-fast
# diff coverage 85% on changed lines: pipx run --spec diff-cover==11.0.0 diff-cover "$COV"/*.xml --compare-branch=origin/develop --fail-under=85
```

No `-race`, no Actions, no retries, no sleeps. SQLite parity: not applicable (no `pekko-persistence*` change). Commit trailer rules: no `Co-Authored-By`; "Refs #814" only.
