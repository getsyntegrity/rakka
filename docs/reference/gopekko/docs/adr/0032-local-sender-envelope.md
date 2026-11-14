# ADR 0032: Local sender envelope contract

- Status: **partly accepted**. D1, D2, D3, D4, D6 and D7 are accepted as recommended; **D5 is split into D5a and D5b and both are decided "follow Pekko" by the Owner on 2026-10-28** (see "D5a and D5b" below). Nothing else is accepted.
  - Accepted by the Owner on 2026-10-24 through an explicit message in the working session (Claude Code), not a GitHub comment. The Owner's words, literally: "D1: A3, D2: Handle, D3: ausencia explícita, D4: TellFrom, D5: abierta para resolver con #814, D6: conservar, D7: aceptar." The decisions were presented against this ADR at head `1fd9469`.
  - D5a and D5b: decided by the Owner on 2026-10-28 in the working session (Claude Code): follow Pekko; the new divergence option is not accepted. The Owner's order also requires the mechanism acceptance criteria below and no new public surface. The Owner's literal D5 wording reached the DEV truncated; this ADR records the faithful reading given in that order. The choice between an identity `Handle` built by the runtime (B1) and the `Handle` the watcher passed (B2) for D5b was made by the Owner on 2026-10-29 in the working session (Claude Code): **B1 chosen**. B1 is implemented (#1278). Candidate map kept for traceability: B1 chosen and implemented; B2 and B3 discarded. The criteria below held for either candidate.
  - Accepted means the contract of the envelope. No production code, test or public API changes with this ADR, and no parity is claimed; the implementation belongs to #814 and #854. D3 is to be reconsidered once #896 defines `deadLetters`.
- Date: 2026-10-24
- Stage: 1 (actor core)
- Module: `pekko-actor`
- Issues: #768 (MSG-01, this slice); epic #748 (MSG); program #746. Siblings that depend on the outcome and are **not** delivered here: #814 (MSG-02, sender inside `Receive`), #854 (MSG-03, `Forward`), #896 (MSG-08, `deadLetters` and ignore references).
- Inventory: `ACTINV-011` (named by #768). The inventory file is not in the repository (`migration/stage-1/supervision-u4/RULEBOOK.md` §6), so no inventory row is edited.
- Sources: Apache Pekko `v1.7.1`, commit `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (the submodule pin). **Normative reference: Pekko Classic.** Paths are relative to `upstream/pekko/actor/src/main/scala/org/apache/pekko`.

## Context

In Pekko Classic every message travels with an optional sender. A receiver reads it with `sender()`, and `forward` re-sends a message keeping the original sender. goPekko today has no sender at all: `Ref[T].Tell(ctx, msg)` (`pekko-actor/actor/ref.go:20`) carries only the payload, and the reply address is a field of the message (ADR 0010 D2, proposed, "no implicit sender").

This ADR decides only **how a local message carries an optional sender**: what the envelope holds, who owns each part, how an explicit reply address differs from the sender, and how the existing `Tell` keeps working. Reading the sender in `Receive`, `Forward`, nil validation, automatic messages and `deadLetters` belong to the sibling stories.

## Pekko sources

| Id | File | Lines | What it fixes |
|---|---|---|---|
| P1 | `actor/ActorRef.scala` | 141 | `tell(msg, sender)`: the sender is an explicit argument; `null` or `noSender` means nobody to reply to |
| P2 | `actor/ActorRef.scala` | 157 | `!(message)(implicit sender = Actor.noSender)`: the sender defaults to `noSender` |
| P3 | `actor/ActorRef.scala` | 143-154 | inside an actor the implicit `self` becomes the sender; outside, no sender is available |
| P4 | `actor/Actor.scala` | 412-416 | `noSender` is `null`, "translated into the receiving system's deadLetters" |
| P5 | `actor/ActorCell.scala` | 596-600 | `sender()` returns `msg.sender`, and `system.deadLetters` when the message is `null` or its sender is `null` |
| P6 | `actor/ActorRef.scala` | 159-164 | `forward(message)` is `tell(message, context.sender())`: it re-sends with the sender of the message being processed |
| P7 | `dispatch/AbstractDispatcher.scala` | 35 | `Envelope(message, sender)`: the sender is envelope data, beside the payload (as cited by ADR 0008) |
| P8 | `actor/dungeon/DeathWatch.scala` | 110 | `self.tell(Terminated(actor)(...), actor)`: the runtime delivers `Terminated` with the watched actor as sender |
| P9 | `dispatch/Mailbox.scala` | 557-571 | an overflowing message is dead-lettered as `DeadLetter(msg, sender, receiver)`, sender included (ADR 0030 P3) |
| P10 | `pattern/AskSupport.scala` | 686-689 | `ask` does `actorRef.tell(message, this)`: the temporary promise ref is the sender of the request |
| P11 | `actor/dungeon/FaultHandling.scala` | 305-306 | `handleFailure(f)`: `currentMessage = Envelope(f, f.child, system)`, the failing child is the sender; the decider and a `preRestart` after an escalation run with it current. Observed on a JVM run of v1.7.1 on 2026-10-28: `migration/stage-1/msg-02-failure-sender/pekko-repro/` |

Pekko does not authenticate the sender: any `ActorRef` may be passed to `tell`. The sender is a convention for replying, not an identity.

## GoAkt

Not consulted. GoAkt is used only when an ADR shows it solved a feature better (`CLAUDE.md`), and nothing here needs it.

## Current state in goPekko

- `Ref[T].Tell(ctx, msg)` has no sender. `ctx` is accepted and not read (`pekko-actor/actor/ref.go:20`).
- `Handle` is `interface{ ID() ID }` (`pekko-actor/actor/id.go:12`): the non-generic identity every ref carries.
- `Ref[T]` is implemented outside the package too (test doubles implement `Tell`), so adding a method to it breaks those implementers.
- ADR 0008 (proposed) defines `Metadata` as immutable scalars and bytes copied from `context.Context`; a `Ref` is not an admissible value. ADR 0010 (proposed) defines `Forward` as a package function and D2 as explicit `ReplyTo`.

## Alternatives

| | A1: implicit sender | A2: sender in `Metadata` (ADR 0008 M1) | A3: explicit optional sender, separate from the payload | A4: only a `ReplyTo` field in each message (status quo) |
|---|---|---|---|---|
| How | The runtime derives the sender from hidden state (goroutine-bound or context-bound) | The sender ref rides in the registered metadata snapshot | The caller passes the sender to a new send function; the runtime stores it in the envelope | The protocol author puts a `Ref[R]` in the message |
| Hidden state | Yes; Go has no safe goroutine-local, a context value would be inherited by every later send | No, but metadata is inherited transitively, so one sender would leak into every send made while handling the message | None: the sender is a value at the call | None |
| Admissible | No | No: `Metadata` admits only immutable scalars and bytes (ADR 0008 "Metadata: ownership"); a ref is a pointer, and sender is not tenancy or trace data | Yes | Yes, but it gives no `sender()` and no sender-preserving `Forward` |
| Pekko `forward` and `sender()` | Reproducible | Not faithfully | Reproducible (MSG-02, MSG-03) | Not reproducible |
| Breaks `Tell` | No | No | No (see "Compatibility plan") | No |

**Recommendation: A3.** A1 is the reason ADR 0010 D2 exists. A2 would put a pointer into a type designed to refuse pointers and would propagate the sender where Pekko does not. A4 stays the right tool for typed request and reply, and A3 does not replace it.

## Decision

Partly accepted by the Owner on 2026-10-24 (see Status). The text below keeps its "proposed" wording as written; the table "Decisions for the Owner" records which choices were accepted. D5 is open.

### Envelope and ownership

A local message, once admitted to a mailbox, is an envelope with exactly these data parts:

| Part | Type | Who sets it | Mutability |
|---|---|---|---|
| Recipient | the target ref (its `ID`) | the runtime, from the ref the send was made on | immutable |
| Payload | `T`, the target's message type | the caller | the value is shared, not copied (as ADR 0008); the runtime never mutates it |
| Sender | optional `Handle` | the caller, through the send function; or the runtime, for `Forward` (P6), and for the runtime deliveries of D5a and D5b | immutable once admitted |
| Metadata, origin, correlation | as ADR 0008 and ADR 0010 | unchanged | unchanged |

Rules:

1. **Retained exactly.** When a sender is present the envelope keeps the reference value that was passed, not a copy of its `ID` and not a lookup by name. Comparing the retained sender with the original by `ID` gives equal.
2. **Absent is a value of its own.** When no sender is given, the envelope holds nothing in that part (a nil `Handle`). It is not an empty `ID` and it is not a placeholder actor. What a reader sees for "absent" is decision D3.
3. **Captured once, at admission.** The sender is read from the send call's argument at the moment of the send. It is never read later from a goroutine, from a `context.Context`, or from the sender's actor state, and the caller changing or stopping afterwards does not change the envelope. A sender that has already terminated is still retained; replying to it is an ordinary dead-letter case.
4. **Not authenticated, not metadata.** The sender is never written into `Metadata`, is not a trusted key, and is not evidence of identity. A guard or observer must not use it for authorization or tenancy, which stay on ADR 0008's trusted metadata. Any ref may be passed as sender, as in P1.
5. **Carried whole.** Whatever stores or re-routes the envelope (dead letter, stash, a router) keeps the sender with it, as P9 does for dead letters. Which of those exist is out of scope here.
6. **No system check.** The runtime does not verify which `System` the sender belongs to, as Pekko does not verify its sender (P1: any `ActorRef` is accepted). A `Handle` from another `System` of the same process is accepted and retained exactly like a local one (rule 1), and gets no special treatment. What happens when the receiver replies to it is the reply's own business: a send through a typed ref of that other system is delivered by that system, and a stopped or foreign target follows the ordinary dead-letter rule. Rejecting foreign senders is the alternative in D7. A `Handle` that is not an actor ref (a hand-built value) is also retained as given; it can never be replied to. Remote and cluster senders are out of scope.

### Explicit reply address versus sender

| | Reply address (`ReplyTo` in the payload) | Sender (envelope) |
|---|---|---|
| Where | A field the protocol author declares in the message | Beside the payload, in the envelope |
| Type | Typed `Ref[R]` | `Handle`, untyped as Pekko's `ActorRef` |
| Who chooses it | The sender of the message, for this protocol | The caller of the send function, or `Forward` |
| Visible to | Only code that reads that message type | The runtime, observers and the receiver through the sibling story's accessor |
| Purpose | Typed request and reply, ask (ADR 0010 D2, ADR 0015) | Classic `sender()`, `forward`, dead-letter reporting |
| Correlation of `Ask` | Created by the runtime, never application-chosen (ADR 0008 "Provenance") | Not involved |

The two never merge: the runtime does not copy the sender into a `ReplyTo`, and does not derive a sender from a `ReplyTo`. `Ask` keeps its explicit endpoint and delivers no sender.

### Compatibility plan

1. `Ref[T].Tell(ctx, msg)` keeps its signature and behavior. It is exactly "send with no sender". Existing callers, tests and `Ref[T]` implementers do not change.
2. The interface `Ref[T]` gets **no new method**. A method would break every type that implements `Ref` today.
3. The sender is attached through a **package-level function**, like `Forward` (ADR 0010), proposed as:

   ```go
   // Proposal, not implemented.
   func TellFrom[T any](ctx context.Context, target Ref[T], msg T, sender Handle)
   ```

   A `nil` sender is the same as `Tell`. Its exact name and the behavior for a `Ref` the runtime did not create (it receives the message without the sender) are fixed by the implementing story, MSG-02, within decision D4.
4. Nothing is inferred at the call site. Inside `Receive`, `Tell` does **not** pass the actor's own ref as sender, unlike Pekko's `!` with implicit `self` (P3). The caller writes the sender explicitly.
5. No behavior of `Tell`, `Ask`, `Watch`, mailbox bounds or ordering changes because of this ADR.

### Observable differences from Pekko

| # | Pekko Classic | This proposal |
|---|---|---|
| 1 | Inside an actor, `!` sends `self` as sender implicitly (P3) | `Tell` sends no sender; the caller must pass one explicitly |
| 2 | No sender reads as the system's `deadLetters` (P4, P5) | Absent is nil in the envelope; the reader's view is D3 |
| 3 | `ask` makes the temporary ask ref the sender of the request (P10) | `Ask` delivers no sender; the reply goes to the explicit `ReplyTo` (ADR 0015) |
| 4 | `Terminated` and the `WatchWith` message are delivered with the watched actor as sender (P8); a parent handles a child's failure with the child as sender (P11) | Same senders (D5a, D5b, decided 2026-10-28). Until implemented, no sender is delivered in those turns (the local watch increment has no envelope, ADR 0008 "Not accepted by the local watch increment") |
| 5 | The sender is an untyped `ActorRef`, usable for `!` directly | The sender is a `Handle`; sending to it needs a typed ref, which MSG-02 defines |

## D5a and D5b (decided by the Owner, 2026-10-28)

Decision: follow Pekko. No new divergence is accepted.

- **D5a.** When a parent handles the failure of a child, the child is the sender of that delivery while it is the cell's current delivery (P11). The decider, an escalation and the `PreRestart` that follows it read the failing child as `Sender()`. The sender of the message the child was processing is never read there.
- **D5b.** The runtime delivers `Terminated` to `OnTerminated` and the `WatchWith` message to `Receive` with the watched actor as sender (P8; `dungeon/DeathWatch.scala:110, 81-91`). The sender's `ID()` equals `Terminated.Who()`. It has the lifetime of the delivery: set when the turn starts, cleared when it succeeds, kept when it fails, cleared where the failed message is cleared, replaced at the start of the next turn.
- Rules 1 to 4 and 6 of "Envelope and ownership" apply unchanged. No metadata, origin or guard is added to these deliveries; ADR 0008 "Not accepted by the local watch increment" is neither accepted nor amended.
- No public surface: no new export, type or conversion (EXP-1: none indispensable).

Mechanism acceptance criteria (each needs a test through the public API before the implementation counts as done):

1. **Incarnation identity observed.** The sender read in the turn has the `ID()` of the incarnation that failed or terminated: after a stop and a new spawn under the same name, the sender of the first delivery keeps the old `ID()` and never equals the new actor's. A restart keeps the same `ID()` (Pekko keeps the `ActorRef` and `uid`). Future Go tests will cover this criterion; the current JVM probe does not cover incarnation identity.
2. **Retention and release.** The sender is retained exactly as long as Pekko's `currentMessage`: until the next turn replaces it, a successful turn clears it, a failed turn keeps it (so `PreRestart` and `PostStop` read it), and `PostRestart` and the new instance read absent. The runtime holds no more than the identity (no cell, no actor instance) beyond that lifetime.
3. **The decider never uses the sender of the child's message.** In the decider the sender is the failing child; the sender of the message the child was processing is never read there, and neither is a sender retained from this cell's own earlier failed message.
4. **No contamination between turns.** A sender retained from a previous turn never leaks into the next one: it is replaced, not reset-then-forgotten. After a handled failure or a `Terminated`, the next `Receive` reads its own sender (absent for `Tell`).

Evidence for the contract: source reading plus a differential JVM run on Pekko v1.7.1 (see P11). Alternatives not decided here and left pending Owner: any Handle-to-Ref conversion. (B1 versus B2 for D5b was decided on 2026-10-29: B1 chosen and implemented in #1278; B2 and B3 discarded.)

## Boundary of this slice

Included: the envelope parts, ownership, the reply-address distinction, the `Tell` compatibility plan, alternatives and unresolved decisions.

Not included, and not claimed: reading the sender in `Receive` (MSG-02), `Forward` (MSG-03), nil validation (MSG-04, MSG-05), `PoisonPill` and `Kill` (MSG-06, MSG-07), `deadLetters` and ignore references (MSG-08), remote or cluster senders, reliable delivery, acknowledgements of `Tell`. No code, test, judge scenario or parity row changes, and no module parity is claimed.

## Traceability

| Item | Where |
|---|---|
| Story | #768 (MSG-01), epic #748, program #746 |
| Inventory | `ACTINV-011` (row not in the repository) |
| Pekko | P1 to P10 above, pin `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` |
| goPekko baseline | `pekko-actor/actor/ref.go:20`, `pekko-actor/actor/id.go:12`; ADR 0008, ADR 0010 D2, ADR 0015, ADR 0030 |
| Public contract | `pekko-actor/README.md`, "Current scope": the sender is not implemented and this ADR is partly accepted (D5 open) |
| Evidence | none executed: this is a decision document. The source citations were read in `upstream/pekko` at the pin |

## Consequences

- With D1, D2 and D4 accepted, MSG-02 and MSG-03 can be specified against fixed envelope data and a fixed compatibility rule, without touching `Tell`.
- If rejected or amended, no code depends on it: nothing was implemented.
- Cost of A3: one more field in the envelope and one more send entry point. Unmeasured.
- ADR 0008 and ADR 0010 stay as written. This ADR neither accepts nor amends them; it only relies on their proposed text.

## Decisions for the Owner

| # | Decision | Alternatives | Recommendation | Consequence of the recommendation | Owner (2026-10-24) |
|---|---|---|---|---|---|
| D1 | Sender model | A1 implicit; A2 in `Metadata`; A3 explicit optional; A4 payload only | **A3** | Reproduces Classic `sender()` and `forward` without hidden state; `Tell` is untouched. | A3 (accepted) |
| D2 | Sender type | Untyped `Handle`; a typed `Ref[R]` per message type | **`Handle`** | One envelope shape for every actor, as Pekko's `ActorRef`; replying needs a typed conversion defined by MSG-02. | Handle (accepted) |
| D3 | What a reader sees when there is no sender | Absent (nil, with a presence flag); the system's `deadLetters` ref as Pekko (needs MSG-08) | **Absent**, revisit once MSG-08 defines `deadLetters` | Honest about "nobody"; differs from P5 until the Owner chooses substitution. | absent, explicit (accepted); revisit with #896 |
| D4 | Send surface | New method on `Ref[T]`; package function `TellFrom`; change `Tell`'s signature | **Package function** | No break for `Ref[T]` implementers or callers; mirrors `Forward`. | `TellFrom` (accepted) |
| D5a | Sender of a failure delivered to a parent (decider, escalation `PreRestart`) | None; the failing child as sender (P11) | The failing child | Reproduces `sender()` in the decider and in `preRestart` after an escalation; no new export. | follow Pekko (decided 2026-10-28) |
| D5b | Sender of `Terminated` and of the `WatchWith` message | None; the watched actor as sender (P8) | The watched actor | Reproduces Pekko; ADR 0008 unchanged; no new export. Which `Handle` carries it: B1, an identity `Handle` built by the runtime (Owner, 2026-10-29; implemented in #1278; B2 and B3 discarded). | follow Pekko (decided 2026-10-28); B1 chosen (2026-10-29), implemented (#1278) |
| D6 | Dead letters keep the sender | Keep (P9); drop | **Keep** | Matches Pekko's `DeadLetter(msg, sender, receiver)`; the dead-letter event shape is MSG-08's. | keep (accepted) |
| D7 | Sender from another `System` | Accept and retain without checks (Pekko, P1); reject in `TellFrom`, which would then report the rejection (an error result, or a dead letter if it stays infallible) | **Accept** | Matches Pekko, adds no new failure to the send and needs no system-membership check on the hot path. Rejecting is also compatible with the unchanged `Tell`, because only `TellFrom` would carry the failure; its cost is a fallible `TellFrom` unlike Pekko's `tell`, and a check that a hand-built `Handle` cannot satisfy. | accept (accepted) |
