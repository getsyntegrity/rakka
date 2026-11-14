# Scope: Current sender inside `Receive` (#814, MSG-02)

Status: **G2 APPROVED by the Owner (real user message, 2026-10-28), only for the sender part covered by the already-accepted ADR 0032 (D1, D2, D3, D4, D6, D7), with the decisions recorded in "Owner authorization" below.** Anything in this document incompatible with that authorization yields to it. **Amended the same day by Owner option B (section 12), and again on 2026-10-29 (Owner, "Status update" at the start of section 12): `ErrOutsideTurn`, S7 and detection of `Context` use outside a turn are OUT OF SCOPE of #814 (not deferred within #814); sections 3, 4, 5, 6, 7 and 10 read with section 12, which prevails over them.** Originally written by the TL on 2026-10-28 against `develop` `80695c9` (ADR 0032 is in the tree, merged by PR #1014).
Pekko baseline: v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. READ = source read, not executed; nothing below is implemented or run.
Issue #814 is `status:blocked` and stays so; no issue or label is edited by this document.
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: this story ports `sender()` (`ActorCell.scala`) and the `tell(msg, sender)` argument (`ActorRef.scala`), so its Dev runs the kit (`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK.
**Pin check:** the submodule HEAD of the checkout read (`.git/modules/upstream/pekko/HEAD`) is `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, equal to the gitlink in this tree; the readings below were taken from that checkout. The Pekko test oracle (`actor-tests`) exists in the checkout and was not enumerated by the TL: the kit manifest must list the Pekko specs for `sender()`; their absence from this document is not an absence of oracle.

## Owner authorization (real user message, 2026-10-28)

1. **G2 approved only for the part covered by accepted ADR 0032 D1-D4, D6, D7.**
2. **Q-D5 DEFERRED.** This delivery does not change the `Terminated` or `WatchWith` paths. Pekko identifies the watched actor as sender (`DeathWatch.scala:110` and `:81-90`); the difference **stays OPEN**. It is not presented as equivalence with Pekko and not as a newly accepted decision.
3. **Q-INV decided:** within a valid `Context`, outside `PreStart`/`Receive` or another expressly authorized turn, `Sender` must reject invalid use with the identifiable cause `ErrOutsideTurn`, following the validity mechanism that already exists. An invalid context is never confused with the legitimate absence of a sender. If this contradicts an accepted ADR, only that contradiction is presented before that part is implemented; the independent rest proceeds.
4. **DeadLetter:** the `Sender` field is deferred to #896; #814 does not claim the `DeadLetter` contract complete.
5. **ADR 0032:** the exact amendment truly necessary is prepared as a proposal (section 11); the message does not accept any amendment text. No ADR PR for choices that do not change an accepted contract. The independent accepted part may be developed meanwhile.

## 1. Result, and what is accepted

Result: **an actor can read the sender that belongs to the message of the current turn, and a caller can attach a sender to a send.**

| ADR 0032 item | State | Used here |
|---|---|---|
| D1 explicit optional sender (A3), D2 `Handle`, D3 absent is explicit, D4 `TellFrom` package function, D6 dead letters keep the sender, D7 foreign `System` accepted | **accepted** (Owner, 2026-10-24) | yes, as written |
| **D5** sender of runtime deliveries (`Terminated`, the `WatchWith` message) | **OPEN, resolved with #814** | **not decided here**, see Q-D5 |

What MSG-01 (#768, PR #1014) integrated: **the ADR only** (`docs/adr/0032-local-sender-envelope.md`, README paragraph at `pekko-actor/README.md:625-629`). Verified on `develop`: no `TellFrom`, no `Sender`, no envelope type in `pekko-actor/` code; `ref.Tell` pushes the bare `T` (`actor/ref.go:50-72`, `mailbox.go:30`). So MSG-01 has nothing to reimplement and this story builds the code from zero.

## 2. Pekko behavior (READ)

| Fact | Source |
|---|---|
| `tell(msg, sender)`: the sender is an explicit argument, `null`/`noSender` means nobody | `ActorRef.scala:141, 157` |
| `sender()` returns `currentMessage.sender`, or `deadLetters` when the message or its sender is null | `ActorCell.scala:596-600` |
| `Terminated` is told with the watched actor as sender | `dungeon/DeathWatch.scala:110` (`self.tell(Terminated(actor)(...), actor)`) |
| A `watchWith` message is delivered by `currentMessage.copy(message = customTermination)`: the envelope of the `Terminated` is kept, so its sender (the watched actor) is also the sender of the custom message | `dungeon/DeathWatch.scala:81-90` |

## 3. Delivered, limits, not delivered

**Delivered (accepted part of ADR 0032):**
1. `TellFrom[T](ctx, target Ref[T], msg T, sender Handle)` (D4): `Tell` plus an optional sender. `nil` sender is exactly `Tell`.
2. `(*Context[T]).Sender() (Handle, bool)`: inside `Receive` for a message sent with `TellFrom`, it returns the very `Handle` that was passed (ADR 0032 rule 1: same value, equal by `ID()`) and `true`; for a message sent with `Tell` or `TellFrom(..., nil)` it returns `(nil, false)` (D3, absent and explicit).
3. The sender belongs to the dequeued envelope: it never leaks from the previous message, and concurrent producers cannot overwrite each other's sender, because it travels inside the queue element, not in a per-cell field written at send time.
4. `Sender` called outside a valid turn is rejected with the identifiable cause `ErrOutsideTurn` (Owner decision 3), distinct from legitimate absence `(nil, false)`.
5. A `Ref` the runtime did not create (a test double implementing `Ref[T]`) given to `TellFrom` receives the message through `Tell`, without the sender (ADR 0032 "Compatibility plan" 3).

**Limits (pinned by tests or docs):** `Tell` keeps signature and every guarantee (never blocks, never fails, bounded-mailbox overflow dead-letters, FIFO per sender, `Ask` unchanged); `Tell` inside `Receive` does not pass `self`; the sender is not written to metadata, not authenticated, and a sender that already terminated is still retained.

**Not delivered, and not claimed:**
- **D5 (deferred by the Owner)**: the sender of the `Terminated` and `WatchWith` turns. Those paths (`watch.go`, `deliverRuntime`) are **not changed**; whatever `Sender()` returns there is an unchanged-path consequence, **not a decision and not equivalence with Pekko** (Pekko: the watched actor is the sender). No AC, test or README sentence asserts it as the contract; the README states that the difference with Pekko is open.
- `Forward` (#854), `deadLetters` and ignore references (#896; D3 is revisited there), nil validation (MSG-04/05).
- `DeadLetter` carrying the sender (D6 is accepted but "the dead-letter event shape is MSG-08's", ADR 0032 line 154): `DeadLetter{Msg, To}` is unchanged, so dead-lettered messages drop their sender in this story. This is a **declared gap** (the field is deferred to #896 by the Owner), not a parity claim.
- Messages posted to the Port (`ReceivePort`) carry no sender. Remote/cluster senders, Typed API (`pekko-actor-typed` is not touched), sibling stories, module parity.

## 4. Owner questions: status

### Q-D5: DEFERRED (decision 2). Evidence kept for the later story

In Pekko both `Terminated` and the `watchWith` message carry the watched actor as sender (`DeathWatch.scala:110`; `:81-90` reuses the same envelope). goPekko delivers both without an envelope: the lane item has only `who ID` (`watch.go:41-52`, `deliverRuntime` `watch.go:492-510`) and `Terminated` has no `Handle` (`watch.go:20-25`). Alternatives for the later decision: A none, as a decided contract (a classification against Pekko, ADR 0033 D13); B the watched actor as sender (`watch.go` stores the subject `Handle`; touches ADR 0008 provenance); C status quo until a story decides. **Nothing is decided; D5 stays open in ADR 0032.**

### Q-INV: DECIDED (decision 3)

`Sender()` called outside a valid turn of its own actor panics with a typed cause wrapping the new exported sentinel `ErrOutsideTurn` (`errors.Is`). The Dev reuses the validity mechanism that already exists for `Context` ("valid only during the hook", `actor.go:46`; typed-cause panics, `watch.go:103-105`) and does not invent a second one; the `Context` is shared by turns (`cell.c.ctx`), so the check must be race-safe against a stray goroutine. Which hooks are "expressly authorized turns" (`PreStart`, `Receive`, others) is confirmed by the Dev in RED against the existing mechanism; in an authorized turn with no envelope (for example `PreStart`) the answer is legitimate absence `(nil, false)`. **No contradiction with an accepted ADR was found**: ADR 0032 leaves the accessor and its failure mode to this story.

### D6: deferred (decision 4)

`DeadLetter{Msg, To}` is unchanged; the `Sender` field is #896's. #814 does not claim the dead-letter contract.

## 5. Contracts consumed (signatures only, not implemented here)

```go
// Package actor (pekko-actor/actor). Proposal for the Dev; names fixed by ADR 0032 D4.
func TellFrom[T any](ctx context.Context, target Ref[T], msg T, sender Handle)
func (x *Context[T]) Sender() (sender Handle, ok bool)
var ErrOutsideTurn error // Owner decision 3
```

Unexported: the queue element becomes `envelope[T]{msg T; sender Handle}` inside `mailbox.Queue[envelope[T]]`. `internal/mailbox.Queue[T]` is generic and is **not** edited.

**EXP-1 / ADR 0011 justification (this is the introducing story, so it is made here and repeated in the PR):**

| Symbol | Consumer | Why the public API is not enough | What it does not expose | Test |
|---|---|---|---|---|
| `TellFrom` | application code and the sibling stories #854 (`Forward`), #896 | `Ref[T]` cannot gain a method without breaking its implementers (ADR 0032 compat. plan 2); `Tell` has no sender slot | the envelope type, the recipient cell | S1-S3, S9 |
| `Context.Sender` | the actor's `Receive`/hooks | the sender lives in the envelope the runtime dequeues; no other public path reads it | the envelope, the queue, the cell | S1-S6 |
| `ErrOutsideTurn` | callers distinguishing the invalid call | `errors.Is` on the typed panic cause is the only observable | nothing else | S7 |

## 6. Criteria → scenarios → Pekko source

Public API only, go-specs, channels/gates for ordering, **no sleeps, no polling**, no `-race` (as the module's other scopes). Level: unit. Files are under `pekko-actor/actor/`.

| # | Criterion | Scenario (new file `sender_test.go`, package `actor_test`) | Source |
|---|---|---|---|
| S1 | A message sent with `TellFrom(..., h)` shows `h` (equal `ID()`, same value) in `Sender()` and `ok` true | `TestSender_S1_TellFromExposesExactSender` | `ActorCell.scala:596-600` |
| S2 | `Tell` and `TellFrom(..., nil)` show `(nil, false)` | `TestSender_S2_NoSenderIsAbsent` | `ActorRef.scala:141, 157`; D3 |
| S3 | The sender never leaks: message 1 with sender X, message 2 with `Tell`, message 3 with sender Y are observed in order as X, absent, Y | `TestSender_S3_NoLeakFromPreviousMessage` | `ActorCell.scala:596-600` |
| S4 | Concurrent producers: N goroutines each `TellFrom` with its own sender, released together by a gate; every received message's sender equals the producer of that message (the payload carries the producer index) | `TestSender_S4_ConcurrentProducersKeepTheirOwnSender` | issue AC 3 |
| S5 | The sender is captured at the send: the sender stopping after `TellFrom` does not change what is read (a terminated sender is still retained) | `TestSender_S5_SenderCapturedAtAdmission` | ADR 0032 rule 3 |
| S6 | Messages queued before the turn (the actor is blocked on a gate while they arrive) each keep their own sender | `TestSender_S6_QueuedMessagesKeepTheirSender` | ADR 0032 rule 5 |
| S7 | Outside a valid turn (a `Context` kept after `Receive` returns, then called) `Sender` panics with a cause satisfying `errors.Is(_, ErrOutsideTurn)`; inside a turn with no sender it returns `(nil, false)` | `TestSender_S7_OutsideTurn` | issue AC 2 |
| S8 | `Tell` regression: signature, FIFO per sender, never blocks, bounded overflow dead-letters once with `DeadLetter{Msg, To}` unchanged, `Ask` unchanged | existing tests stay green; one new `TestSender_S8_TellUnchanged` per touched behavior | ADR 0032 compat. plan 1, 5 |
| S9 | A foreign `Ref[T]` implementation (a test double) given to `TellFrom` receives the message through its own `Tell`, without sender | `TestSender_S9_ForeignRefReceivesViaTell` | ADR 0032 compat. plan 3 |
| S10 | A `Handle` from another `System` and a hand-built `Handle` are retained exactly | `TestSender_S10_ForeignAndHandBuiltHandle` | ADR 0032 rule 6, D7 |

Differential check (AGENTS test policy): for the shared code (push/pop, the pending drain of `stop.go`, the capacity count at `watch.go:357`) run the existing commands against a base copy in `mktemp -d`; stdout, exit code and counters (`told/received/deadLettered/depth/maxDepth`) must be identical.

## 7. Allowed files

Exactly these paths; anything else needs a stop and a report to the TL.

| Path | Edit | PR |
|---|---|---|
| `docs/scopes/814-msg-02-current-sender.md` | this file | 0 |
| `migration/stage-1/msg-02-current-sender/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | new kit package | 1 |
| `pekko-actor/actor/mailbox.go` | queue element becomes `envelope[T]`; push/pop/pending helpers | 2a |
| `pekko-actor/actor/cell.go` | `queue` field type; the turn carries the dequeued sender (`drain`, `callReceive`) | 2a |
| `pekko-actor/actor/ref.go` | `Tell` wraps in the envelope; `TellFrom` | 2a, 2b |
| `pekko-actor/actor/stop.go`, `pekko-actor/actor/watch.go` | **only** the lines that handle `[]T` or `queue.Len()` of the changed element type (no behavior change; `watch.go:357`) | 2a |
| `pekko-actor/actor/actor.go` | `Context.Sender` (next to `Context`) | 2b |
| `pekko-actor/actor/errors.go` | `ErrOutsideTurn` | 2b |
| `pekko-actor/actor/sender_test.go` | new, S1-S10 | 2b |
| existing `*_test.go` of `pekko-actor/actor/` | only if the element-type change breaks compilation; no assertion weakened | 2a |
| `pekko-actor/README.md` | replace lines 625-629 (sender now partly implemented; D5 unspecified; Forward and deadLetters not implemented); API list | 2b |
| `migration/stage-1/msg-02-current-sender/EXECUTION-RECORD.md`, `migration/cost-log.tsv` (one row) | kit run evidence bound to a SHA | 3 |

Nothing under `pekko-actor/internal/`, `pekko-actor-typed/`, `clock.go`, `ask.go`, system options or other modules changes.

## 8. Collisions and independence

| Other work | Files | Collision with this scope | Resolution |
|---|---|---|---|
| Squad A #1129 (TIM-11, clock) | `clock.go`, `ask.go`, system options, README | README (this scope: lines 625-629 and the API list) | sequence README edits: whoever merges second rebases; this scope touches no `clock.go`, `ask.go` or options |
| Squad C #1098 (MBX-15, head insert) | `internal/mailbox` only | semantic only: head-insert must carry the envelope. This scope keeps `Queue[T]` generic and untouched, so no file overlap | if #1098 changes the `Queue` API, 2a rebases; carrying an envelope on head-insert is then #1098's rebase concern |
| Squad B others (#854 Forward, #896) | `ref.go`, `actor.go`, `cell.go` | yes: they build on `TellFrom` and `Sender` | strictly after PR 2b; never in parallel |
| Typed scopes (#925/#954/#956) | `pekko-actor-typed/` | none | none |

D5 (the `Terminated`/`WatchWith` sender) is a later story that edits `watch.go`; it is not in this scope.

## 9. Size and delivery (budget 400 authored lines)

| PR | Content | Forecast (add+del) |
|---|---|---|
| 0 | this scope | docs |
| 1 | kit package | docs/manifest |
| 2a | internal envelope refactor, no new export, `Tell` characterization green | ~120 |
| 2b | `TellFrom`, `Sender`, `ErrOutsideTurn`, `sender_test.go`, README | ~330 (tests ~230, production ~50, README ~50) |
| 3 | execution record, cost-log | docs |

2a and 2b are split to keep each under 400; if 2b exceeds it, S8-S10 move to a 2c. Excluded from the count: kit and migration docs. Commits: Conventional (`feat(actor): ...`, `refactor(actor): ...`, `docs(...)`).

## 10. Definition of Done

- Every criterion S1-S10 has its scenario, passing, bound to a tested candidate SHA.
- `scripts/each_module.sh go test -count=1 ./...` and `scripts/check_deps.sh` green; per-module coverage at least 85%, diff coverage at least 85%; cyclomatic complexity at most 10 per function (no `//nolint`).
- The kit run is executed with manifest, RULEBOOK and RUNBOOK. No ADR PR gates PR 2a or 2b (section 11).
- README states implemented versus not: no `Forward`, no `deadLetters`, the D5 difference with Pekko open, `DeadLetter` without sender.
- No `Tell`, `Ask`, `Watch` or mailbox behavior changes; the differential check shows no difference.
- No claim of module parity or of completion of #854, #896 or D5; #814 closes for the delivered part only if the Owner agrees (D5 may require it to stay open).

## 11. ADR 0032 amendment: status

**Needed for the independent accepted part: NO.** Checked against the ADR text: D1, D2, D3, D4, D6 and D7 are implemented as written; D5 stays open; D6 keeps the dead-letter shape for MSG-08 (ADR line 154) and "Boundary of this slice" already lists `deadLetters` as not included; the accessor and `ErrOutsideTurn` are left to this story (ADR "Context" and "Compatibility plan" 3). No accepted contract changes, so per the Owner **no ADR PR is opened**.

**Optional bookkeeping wording (not required, not accepted, for Owner review only).** It only stops the ADR from saying D5 is resolved "with #814":

```diff
--- a/docs/adr/0032-local-sender-envelope.md
+++ b/docs/adr/0032-local-sender-envelope.md
@@ line 3
-- Status: **partly accepted**. D1, D2, D3, D4, D6 and D7 are accepted as recommended; **D5 stays open** and is resolved with #814 (MSG-02). Nothing else is accepted.
+- Status: **partly accepted**. D1, D2, D3, D4, D6 and D7 are accepted as recommended; **D5 stays open**; the Owner deferred it out of #814 (MSG-02) on 2026-10-28. Nothing else is accepted.
@@ line 153 (table row D5, last column)
-**open**, resolved with #814
+**open**, deferred out of #814 (Owner, 2026-10-28)
```

If the Owner does not review it, nothing changes and the delivery proceeds.

## 12. Owner option B: `ErrOutsideTurn` (2026-10-28 text was "deferred"; out of scope since 2026-10-29)

**Status update 2026-10-29 (Owner).** `ErrOutsideTurn`, S7 and the detection of `Context` use outside a turn are **OUT OF SCOPE of #814**. They are not a deferred capability that stays pending inside #814, and the original wording of issue AC2 is not marked as implemented. Owner, verbatim (Spanish, as received): "El contrato exige usar Context únicamente durante la llamada autorizada. Retenerlo o usarlo concurrentemente queda fuera del contrato; no se promete detectar ese uso." Issue AC2 reads: "Calls outside a valid actor turn have the documented invalid-context outcome." The issue's own ACT2 reconciliation calls it "a Go-specific rule". Every "deferred" (or "deferred to a later unit") about `ErrOutsideTurn`, S7 or the stale-`Context` check in this document and in the files that quote it is historical text of 2026-10-28, **superseded** by this update and kept unedited below for the record. D5 and `DeadLetter.Sender` deferrals are unaffected.

*Historical text of 2026-10-28 follows (heading was "Owner option B: `ErrOutsideTurn` deferred (real user message, 2026-10-28; garbled in transit, read as option B)").*

**False premise recorded (Q-INV, decision 3).** The scope said `Sender` would follow "the validity mechanism that already exists". The Dev found there is none: `Context[T]` is one `struct{ c *cell[T] }` (`actor.go:47`) created once per cell and shared by every turn (`cell.ctx`, `cell.go:32`; passed unchanged at `cell.go:285` PreStart, `cell.go:312` Receive, `port.go:114` ReceivePort, `start_failure.go:116` ResumeWith). "Valid only during that call" (`actor.go:46`) is documentation only; `watch.go:103-105` panics for unhonourable arguments, not for a stale Context. The Owner does **not** authorize inventing a turn marker.

**Decision (B).** PR 2b delivers `TellFrom` and `Context.Sender()` only. `ErrOutsideTurn`, S7 and its EXP-1 row are ~~**deferred to a later unit**~~ [Superseded 2026-10-29: out of scope of #814, see the status update above]. This is a **recorded limit of the delivery**, not a parity claim. Superseded by this section: item 4 of section 3, the Q-INV text of section 4, the `ErrOutsideTurn` lines of sections 5, 7 (`errors.go` leaves the allowed files) and 6 (S7 removed), and the matching Definition of Done wording. `RULEBOOK.md` is only consumed, not edited.

**`Sender()` contract.** Returns `(nil, false)` when the current delivery has no sender, and `(h, true)` with the exact `Handle` passed to `TellFrom` otherwise. **`(nil, false)` cannot distinguish an invalid or stale `Context` from a legitimately absent sender.** The rule "a Context is valid only during the hook call" stays documented, **not enforced**; the godoc of `Sender` and the README say so and promise no enforcement. Because a stale Context can be read from another goroutine, the Dev must still make the read race-safe (atomic or equivalent) without introducing a validity marker or an error path.

**Hook decision, from Pekko at the pin** (checkout HEAD `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, read-only):
- Pekko sets `currentMessage = messageHandle` in `invoke` before processing (`ActorCell.scala:553`), `sender()` reads it and yields `deadLetters` when it is null (`:596-600`), and it is reset to `null` after a **successful** invocation (`:559`). On a failed invocation `handleInvokeFailure` runs instead (`:561`); `clearActorFields` sets `currentMessage = null` on restart and termination (`:718-719`; callers `dungeon/FaultHandling.scala:110, 273, 300`). `preStart` is not run through `invoke`, so no envelope is current there (READ: `currentMessage` starts `null`, `:451`).

| Where | `Sender()` returns | Basis |
|---|---|---|
| `Receive` | the envelope sender, or `(nil, false)` if none | `ActorCell.scala:553, 596-600` |
| `PreStart` | `(nil, false)` | no `invoke`, `currentMessage` null (`:451`); Pekko reads `deadLetters`, mapped to absent by D3 |
| `ReceivePort` | `(nil, false)` | Go-only mechanism with no Pekko counterpart; most conservative representation; `// TODO(MSG-02-port)` marks it |
| `OnTerminated`, `WatchWith` message | `(nil, false)` as an unchanged-path consequence | Pekko gives the watched actor (`DeathWatch.scala:110, 81-90`); D5 deferred, difference **open** |
| `PostStop`, `ResumeWith` continuation, supervision decider | `(nil, false)` | not turns of a message envelope; conservative |

**Where the sender is cleared.** After every `Receive` turn, on success **and** on failure, before the next step is taken. Pekko clears on success (`:559`) and on restart or termination (`:718-719`) but is silent on a failure that resumes; the Go choice (clear always, so a failed turn's sender never leaks into the next) is the most conservative, marked `// TODO(MSG-02-clear)` for the kit run to classify. No sender is ever carried across turns (S3 pins it).

**Budget.** Removing S7 and `errors.go` lowers 2b to about 300 changed lines; still one PR under 400.

## 13. Sender during failures and in each hook: Pekko pinned, correction rules (Owner, 2026-10-29)

Owner decision (real user message, recovered in full): PR #1270 follows Pekko; the "always clear the sender after a failed `Receive`" choice (`TODO(MSG-02-clear)`) is **not accepted as a divergence**. **Correction of this document:** section 12 said Pekko "is silent on a failure that resumes". That was wrong: Pekko is explicit (d3 and d6 below). The sentence of section 12 "Cleared after every Receive turn, on success and on failure" is **superseded** by this section.

Evidence labels: **PEKKO** = source read, not executed; **GO** = goPekko source read at `develop` `80695c9` and PR #1270 head (`pr1270`); **PENDING** = differential execution not run. Pin check: `.git/modules/upstream/pekko/HEAD` = `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. Pekko paths are under `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/`.

### 13.1 Pekko facts (PEKKO), by question

`sender()` reads `currentMessage` (`ActorCell.scala:596-600`); `currentMessage` starts null (`:451`).

**(d) When `currentMessage` is set, cleared or replaced**

| # | Event | Effect on `currentMessage` | Cite |
|---|---|---|---|
| d1 | `invoke` starts (every mailbox message, including auto-received `Terminated`, `PoisonPill`, `Kill`) | **replaced** by that envelope, before processing | `ActorCell.scala:553` |
| d2 | `invoke` succeeds | **cleared** (`null`) | `:559` |
| d3 | `invoke` fails | **not cleared**: the `catch` runs `handleInvokeFailure` only; the failed envelope stays current | `:560-562` (no assignment) |
| d4 | `systemInvoke` (`Create`, `Recreate`, `Suspend`, `Resume`, `Terminate`, `Watch`, `Unwatch`, `Supervise`, `DeathWatchNotification`) | **untouched** by the dispatcher itself | `:481-531` |
| d5 | `faultRecreate`: `preRestart` runs, then `clearActorFields` in `finally` | readable during `preRestart`, **cleared after** (even if `preRestart` throws) | `dungeon/FaultHandling.scala:101-111`, `ActorCell.scala:718-719` |
| d6 | `faultResume` | **untouched** (the failed envelope stays current until the next `invoke`) | `FaultHandling.scala:136-153` |
| d7 | `finishTerminate`: `postStop`, then `clearActorFields(a, false)` | readable during `postStop`, **cleared after** | `FaultHandling.scala:247-275` (254, 273) |
| d8 | `finishRecreate` failure path | cleared before `handleInvokeFailure` (to avoid a second `preRestart`) | `FaultHandling.scala:298-302` (300) |
| d9 | parent handles a child's `Failed` (`handleFailure`) | **replaced** by `Envelope(f, f.child, system)`: message = `Failed`, **sender = the failing child**; not cleared afterwards | `FaultHandling.scala:305-306` |
| d10 | `watchWith` | the `Terminated` envelope is kept with its message replaced (sender = watched actor) | `DeathWatch.scala:81-90` |

**(a) What `preRestart` observes.** The restart (`Recreate`, a system message) arrives after the failure; by d3 and d4 `currentMessage` is still the envelope that failed, and `faultRecreate` passes `Some(currentMessage.message)` as the `message` argument (`FaultHandling.scala:101, 104`). So `sender()` in `preRestart` is **the sender of the same message that is given to `preRestart`** (the failed one). If the failure came from a child's `Failed` (escalation, d9) that message is the `Failed` and the sender is the child. If the last invocation succeeded (d2) or the actor never processed a message, there is none: absent.

**(b) What `postStop` observes**, by path (all are `finishTerminate`, d7; it is reached from `Terminate`, d4, so nothing replaces `currentMessage` before it):

| Path | Sender in `postStop` | Why |
|---|---|---|
| Normal stop after a successful message (`context.stop`, `PoisonPill`, system stop) | absent | d2 cleared it; `PoisonPill` is itself an `invoke` that succeeds |
| Stop after a failure with the Stop directive | **the sender of the failed message** | d3 kept it; `Terminate` (d4) does not touch it |
| Restart-driven stop of the old instance (default `preRestart` calls `postStop()`, `Actor.scala:625-631`) | **the sender of the failed message** | inside `aroundPreRestart`, before d5's clear |
| Parent stop | absent if the child was idle (d2); **the failed message's sender if the child was left failed** (d3) | no special case in Pekko; same retained value |
| Stop requested while a restart waits for children | not asserted | Pekko's order depends on the children container; not pinned here |

Do not read "keeping `currentMessage`" as identical in every `postStop`: it differs per path as shown.

**(c) The parent's decider.** `handleFailure` sets `currentMessage = Envelope(f, f.child, system)` (d9) before `supervisorStrategy.handleFailure` (`FaultHandling.scala:306, 314`). So `sender()` seen by code running there is **the failing child**, not the sender of the message the child was processing (that envelope lives in the child's own cell). The strategy's decider function itself takes only the cause: it has no sender parameter.

### 13.2 Rules to implement, per goPekko hook

Design (GO, the minimum that follows Pekko): the cell's current sender has the **lifetime of Pekko's `currentMessage`**, which `failedMsg` already mirrors (`cell.go:84, 321-323, 337`; `supervision_restart.go:283`; `stop.go:141`; `start_failure.go:171`). Rule: **set when a `Receive` turn starts (d1); clear on its success (d2); keep on its failure (d3); clear exactly where `failedMsg` is cleared on restart and termination (d5, d7); never clear on resume (d6).** The `defer c.curSender.Store(nil)` of #1270 in `callReceive` is the divergence to remove. Reads stay atomic (a stale `Context` may be read from another goroutine; no detection is promised).

| Hook / path | Sender observed in Pekko (cite) | goPekko rule to implement |
|---|---|---|
| `Receive` | the envelope's sender (`:553, 596-600`) | the envelope sender, or `(nil, false)` |
| `PreStart` (first start) | absent: `currentMessage` null (`:451`) | `(nil, false)` |
| `PreRestart` | sender of the failed message (a) | **the sender of the failed message**, the same value as the `FailedMessage` passed in; `(nil, false)` if the last turn succeeded or the failure had no sender |
| `PostRestart` and the PreStart it runs | absent: cleared by d5 | `(nil, false)` (clear where `supervision_restart.go:283` clears `failedMsg`) |
| `PostStop` | per the (b) table | the same retained value: the failed message's sender after a Stop-directive failure and in the default `PreRestart`'s PostStop; absent after a successful turn; cleared after `PostStop` (`releaseTerminated`, d7) |
| `ResumeWith` continuation | Go-only construct; Pekko resumes with the failed envelope still current (d6) | the retained value (nothing clears it on resume); `// TODO(MSG-02-resume)`: Pekko has no continuation, the kit run classifies |
| Next `Receive` after a resume | replaced at `invoke` (d1) | replaced by the new envelope's sender, or absent for a `Tell`; never the old one (S3 pins it) |
| Parent's decider | **the failing child** (d9), never the child's message sender | **`(nil, false)`, deferred with D5**, `// TODO(MSG-02-failed)`: Pekko's value (the child) is a runtime-delivery sender like D5, not decided here; **never the sender of the message the child was processing**. The Go decider already has `child Handle` as a parameter (`supervision.go:48`); no sender is added |
| `OnTerminated`, `WatchWith` message | the watched actor (d1, d10) | **unchanged paths (D5 deferred, difference OPEN)**; the only edit allowed is the unavoidable reset to absent at the start of the turn, so a retained failed-message sender never leaks into it, `// TODO(MSG-02-D5)` |
| `ReceivePort` | Go-only; shared property: a message from outside the actor told with no sender reads absent (`ActorRef.scala:141, 157`; `ActorCell.scala:596-600`; D3) | **reset to absent at the start of the turn** (no leak of a retained sender), `(nil, false)` during it; keep `TODO(MSG-02-port)` for the Go adaptation |

No accepted contract (ADR 0032 D1-D4, D6, D7) is contradicted: D3 (absent) and D1/D2 (retained exactly, `Handle`) are respected; rule 3 "captured at admission" is untouched (the envelope still carries the sender); retention after a failed turn concerns the reader's current delivery, not the envelope.

Ambiguities flagged for the Owner: (i) the decider row, and `PreRestart` after an escalation, defer Pekko's "sender = the child" together with D5; extend the D5 deferral explicitly or authorize the child as sender; (ii) the reset lines in `deliverRuntime` and `deliverPort` touch `watch.go` and `port.go` minimally.

### 13.3 Scenarios (public API, no sleeps, channels and gates; in `sender_test.go`, package `actor_test`)

Failure setup: a parent with a decider returning the directive; the child's `Receive` returns an error on a marked message; hooks record `ctx.Sender()` into a channel. Evidence for every row: GO test; Pekko reading above; PENDING differential.

| Id | Setup | Assertions |
|---|---|---|
| F1 | `TellFrom(child, "boom", X)`; Restart directive; `PreRestart` records | `PreRestart` saw `(X, true)` and the `FailedMessage` is "boom" |
| F2 | same message; Stop directive; `PostStop` records | `PostStop` saw `(X, true)` |
| F3 | Restart directive; no custom `PreRestart` (the default stops the old instance through `PostStop`) | the old instance's `PostStop` saw `(X, true)` |
| F4 | as F1; the new instance's `PreStart`/`PostRestart` records | saw `(nil, false)` |
| F5 | `TellFrom(child, "ok", X)` succeeds; then `System.Stop(child)`; `PostStop` records | `PostStop` saw `(nil, false)` |
| F6 | idle child; the parent stops; `PostStop` records | `(nil, false)` |
| F7 | `TellFrom("boom", X)`; Resume directive; then `Tell("next")`; then `TellFrom("m3", Y)` | `next` sees absent; `m3` sees `(Y, true)` (the retained value is replaced, never accumulated) |
| F8 | the failing message has sender X; the decider calls `ctx.Sender()` | the result is **not** X (`(nil, false)` today, deferred with D5; the test does not pin "absent" as Pekko equivalence) |
| F9 | `TellFrom("boom", X)`; Resume; the child then receives a `Terminated` for a watched actor | `OnTerminated` does not observe X (asserts `!= X` only; the Terminated sender stays D5) |
| P1 | `Port.Post(msg)` on an idle actor whose `ReceivePort` records | `(nil, false)` |
| P2 | `TellFrom("boom", X)`; Resume; then `Port.Post(msg)` | `ReceivePort` saw `(nil, false)`, not X |

For the record: P1 and P2 test the **Go adaptation** (`Port`); the property shared with Pekko is only "a delivery from outside the actor with no sender reads as absent" (D3). Removed from the unit: `ErrOutsideTurn`, S7 and its EXP-1 row (Owner decision of 2026-10-28, option B); a stale `Context` is **outside the contract, with no detection promise**.

### 13.4 RULEBOOK delta (exact diff for the Dev to apply on the #1270 branch; `RULEBOOK.md` is not edited here)

Against `migration/stage-1/msg-02-current-sender/RULEBOOK.md` at `pr1270` (28 lines). The sha256 approval of the DRAFT must be redone after the delta.

```diff
--- a/migration/stage-1/msg-02-current-sender/RULEBOOK.md
+++ b/migration/stage-1/msg-02-current-sender/RULEBOOK.md
@@ -9,5 +9,7 @@
   - `actor/src/main/scala/org/apache/pekko/actor/ActorCell.scala:596-600`: `sender()` returns `currentMessage.sender`, or `deadLetters` when the message or its sender is null.
   - `actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala:141, 157`: `tell(msg, sender)`; `noSender` means nobody.
+  - `ActorCell.scala:451, 553, 559, 718-719` and `dungeon/FaultHandling.scala:101-111, 247-275, 305-306`: `currentMessage` is replaced at `invoke`, cleared only after a successful invocation and by `clearActorFields`, kept on a failed one and on resume; `preRestart` and `postStop` read the retained value; the parent's `handleFailure` replaces it with the `Failed` envelope whose sender is the failing child.
   - Specs that exercise `sender()` and are the oracle for the scenarios, under `actor-tests/src/test/scala/org/apache/pekko/actor/`: `ActorRefSpec.scala` ("support reply via sender" at line 390, helpers at 37-54 and 83-113), `SupervisorSpec.scala:57` (`sender() != sendTo`), `ActorSystemSpec.scala:205` (`tell("boom", ActorRef.noSender)`).
-- Adapted, not ported: Pekko returns `deadLetters` when there is no sender; goPekko returns `(nil, false)` (ADR 0032 D3); `deadLetters` is #896. The `Terminated` and `WatchWith` sender (ADR 0032 D5) is DEFERRED by the Owner and is not part of this unit.
+- Adapted, not ported: Pekko returns `deadLetters` when there is no sender; goPekko returns `(nil, false)` (ADR 0032 D3); `deadLetters` is #896. The `Terminated` and `WatchWith` sender (ADR 0032 D5) is DEFERRED by the Owner and is not part of this unit. The parent decider's sender (Pekko: the failing child) is deferred with D5 and reads `(nil, false)`; it is never the sender of the message the child was processing. A stale `Context` is outside the contract and no detection is promised; `ErrOutsideTurn` is not part of this unit.
@@ -14,3 +16,16 @@
 ## 1. Scenarios

-S1 to S10 of the scope (section 6) are the scenarios; each has its test name there and they are not restated here.
+S1 to S6 and S8 to S10 of the scope (section 6) are scenarios; S7 is removed (`ErrOutsideTurn` is not delivered). Section 13.3 of the scope adds F1 to F9 and P1, P2. Each has its test name there and they are not restated here.
+
+Evidence is kept in three separate columns of the execution record, never merged:
+
+- Pekko reading (source read, not executed): the citations above.
+- Go evidence: the passing tests and the candidate SHA.
+- Pending differential execution: the Classic judge run of F1 to F5 and F7 against Pekko is NOT done; the rows stay `PENDING` until it is run or the Owner approves alternative evidence (ADR 0033 D13).
+
+### 1.1 Current-sender lifetime (follows Pekko `currentMessage`)
+
+- Set when a `Receive` turn starts; cleared when it succeeds; kept when it fails; cleared where the failed message is cleared on restart and on termination, after `PostStop`; never cleared by a resume.
+- `PreRestart` and the `PostStop` of a stop after failure or of a restart read the failed message's sender; `PostRestart`, `PreStart` and `PostStop` after a successful turn read absent.
+- `ReceivePort`, `OnTerminated` and the `WatchWith` message start with an absent sender (no leak); the `Terminated` and `WatchWith` difference with Pekko stays OPEN (D5).
+- `// TODO(MSG-02-clear)` is deleted: the choice it marked is not accepted. New markers: `TODO(MSG-02-resume)`, `TODO(MSG-02-failed)`, `TODO(MSG-02-D5)`, `TODO(MSG-02-port)`.
@@ -20,3 +35,3 @@
 - 2a: internal `envelope[T]{msg, sender}` as the queue element; no new export; existing behavior unchanged.
-- 2b: `TellFrom`, `Context.Sender`, `ErrOutsideTurn`, `sender_test.go`, README.
+- 2b: `TellFrom`, `Context.Sender`, `sender_test.go`, README.
 - 3: execution record and cost-log row.
```

Apply the hunks by content; the offsets are indicative. The manifest row "use of a Context outside its hook" stays (documented limit, no detection).

### 13.5 Allowed files and budget forecast for the correction (PR #1270)

Files: `pekko-actor/actor/cell.go` (lifetime), `port.go`, `watch.go` (reset line only), `supervision_failure.go` (reset line only in `handleFailed`), `supervision_restart.go` and `stop.go` (clear sites), `start_failure.go` (clear site), `sender_test.go`, `pekko-actor/README.md`, the RULEBOOK per 13.4 (applied by the Dev), `EXECUTION-RECORD.md`. Nothing else; no `ErrOutsideTurn` anywhere.

| Part | Forecast changed lines (add+del) |
|---|---|
| Production: lifetime change in `cell.go` (set at start, clear on success, remove the deferred clear), four clear sites, three reset lines | ~55 |
| Tests F1-F9, P1, P2 (about 25 each) | ~275 |
| README, RULEBOOK delta, execution record | ~55 |
| **Total** | **~385; the removal of S7 and of error-sentinel code adds deletions not counted here, so the total can pass 400** |

**Split proposed before enlarging or trimming any test:** correction **C1** = the production lifetime plus F1, F2, F3, F5, F7 plus the RULEBOOK delta (about 240); correction **C2** = the reset lines plus F4, F6, F8, F9, P1, P2 plus README and execution record (about 160). C2 depends on C1 only through `cell.go`. If the measured `git diff --numstat` of the whole correction is 400 or less, the Dev may keep one commit and says so in the record.
