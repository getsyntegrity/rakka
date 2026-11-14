# Scope: Mark suppressible messages and publish `SuppressedDeadLetter` (#871, EVT-05; divergence 5a of ADR 0010)

Status: **approved at G2 by the Owner, 2026-11-10, with D-1 to D-10 as written and the amendments below.** First drafted as a draft for G2. Written on 2026-11-10 against `develop` `8e334700`. #871 is open; its dependency #896 is delivered (`System.DeadLetters()` and the sender in `DeadLetter`, PRs #1517 and #1519) and the oracle is merged ([`suppressed-dead-letters-2026-11-10.md`](../parity/oracles/suppressed-dead-letters-2026-11-10.md), #1508). Order decided by the Owner on 2026-11-10: lane 1 of the new stage. No code is written before the Owner approves this scope.
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: the unit ports `DeadLetterSuppression`, `SuppressedDeadLetter` and `AllDeadLetters` (`ActorRef.scala:579-614`) and the suppression branch of the dead-letter ref (`ActorRef.scala:716-724`, `:736-743`), and adapts them to Go types and to the `Publish` channels of ADR 0038. The Dev runs the migration kit (`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK under `migration/stage-1/suppressed-dead-letters/` (name INFERRED from the sibling kit packages; the Dev may rename it in slice 0 of the run). The Dev is a separate session, QA is independent of both, and the TL does not implement.

## Contracts consumed

[ADR 0010](../adr/0010-actor-api.md) (Amendment 2026-11-04, item 5a: "Un PoisonPill no entregado … debe publicar SuppressedDeadLetter, como Pekko … Dueño: #871. #770 no se cierra hasta que esto esté entregado"), [ADR 0032](../adr/0032-local-sender-envelope.md) D3, D4, D6 (absent sender, `TellFrom`, dead letters keep the sender), [ADR 0038](../adr/0038-event-stream-subscription-semantics.md) D2 (channels by type or interface) and D7 ("`SuppressedDeadLetter` … published through this stream when their stories land"), [ADR 0011](../adr/0011-pekko-actor-layout.md) owner decision 2 (`actor/` stays one package) and EXP-1 (`docs/parity/typed-owner-decisions-2026-10-27.md:36`), ADR 0033 D13/D14.3, the oracle above, the scope [896](896-deadletters-ref-and-sender.md) (sink and sender; its "Out of scope" assigns this unit the marker, the event and 5a) and the issue #871 with its criteria.

## Objective

As a Go actor application developer, I can subscribe to `SuppressedDeadLetter` (and to `AllDeadLetters`) on the event stream and observe that a message that **marks itself suppressible** was not delivered, with the same delivery outcome as before (the marker changes event classification only, #871 AC1). `PoisonPill` and `Terminated` carry the marker; `Kill` does not. This closes divergence 5a: an undelivered `PoisonPill` (stopped actor, stop accepted, full bounded mailbox, queued when the stop is accepted) publishes `SuppressedDeadLetter`, as Pekko does. It is the second criterion's prerequisite of [#770](https://github.com/getsyntegrity/gopekko/issues/770) ("messages behind PoisonPill follow the normal stop/dead-letter contract"); this scope does not touch #770.

## Evidence: Pekko, the oracle, and what goPekko does today

Contract = what the bodies of the specs assert and what the oracle observed; the oracle observations are the contract (Owner, 2026-11-10). `DL8b` is a measured race (oracle section 5, 2 of 11 runs against 9 of 11) and **is not used**: no scenario, test or mutation of this unit depends on it.

| Contract | Pekko 1.7.1 (path:line, read in this revision) | Oracle observation | goPekko today (`develop` `8e334700`) |
|---|---|---|---|
| Marker | `trait DeadLetterSuppression` (`ActorRef.scala:601`); "will not be logged by the default dead-letters listener. Instead they will be wrapped as `SuppressedDeadLetter`" (`:597-600`) | S1 | absent |
| Event | `SuppressedDeadLetter(message: DeadLetterSuppression, sender, recipient)`, sender and recipient non-null (`ActorRef.scala:609-614`); `AllDeadLetters` is the common super-type of `DeadLetter`, `SuppressedDeadLetter` and `Dropped` (`:579-583`, `:592`, `:610-611`, `:622`) | S1, S8 | absent; `DeadLetter{Msg, Sender, To}` only (`deadletter.go:26`) |
| `PoisonPill`, `Terminated` carry it, `Kill` does not | `Actor.scala:46` (`PoisonPill`), `:60` (`Kill` has no marker), `:123-129` (`Terminated`) | S2; `DL3` [005] (`Terminated` suppressed), `DL4` [004] (`Kill` a plain `DeadLetter`) | none |
| A suppressed event is told through `deadLetters`, so its recipient is `deadLetters`, never the original recipient | `specialHandle`: `case m: DeadLetterSuppression => publishSupressedDeadLetter(m, sender); true` (`ActorRef.scala:716-718`); `publishSupressedDeadLetter` builds `SuppressedDeadLetter(msg, if (sender eq noSender) provider.deadLetters else sender, this)` with `this` = the dead-letter ref (`:722-724`); `DeadLetterActorRef.!` routes a `DeadLetter` through `specialHandle(d.message, d.sender)` and a plain message through `specialHandle` first (`:736-743`) | S4; recipient `dls` in `DL1` [006], `DL2` [004], `DL3` [005], `DL5` (a) [004], `DL7` [019]; for a plain `DeadLetter` the recipient is the dead actor (`DL4` [004], `DL7` [010]) | `publish` (`deadletter.go:44-53`) always publishes `DeadLetter` |
| Sender | absent sender becomes `deadLetters` (`ActorRef.scala:723`); an explicit sender is kept | `DL1` [006] sender `dls`; `DL1b` [004] sender `probe`; `DL5` (a) [004] keeps `probe` | `DeadLetter` already does this (#896) |
| Exclusive channels | a `SuppressedDeadLetter` reaches `SuppressedDeadLetter` and `AllDeadLetters` subscribers and not `DeadLetter` subscribers; a `DeadLetter` the reverse | ADR 0038 D7 finding, every scenario | n/a |
| Already stopped actor | `DL1`, `DL1b` | `SuppressedDeadLetter(PoisonPill, sender, deadLetters)`, no `DeadLetter` | `postLane` drops it, publishes nothing (`port.go:80`) |
| Full bounded mailbox | `Mailbox.scala:557-571` | `DL7` [019]; the plain control `PlainMsg` is a `DeadLetter` with recipient `victim` [010] | same drop |
| Queued when the stop is accepted | `Mailbox.cleanUp` (`Mailbox.scala:340-354`, `:400-412`) | `DL8`, `DL8c` [013]: `DeadLetter(m2, deadLetters, victim)` and `SuppressedDeadLetter(PoisonPill, deadLetters, deadLetters)` | `takePortDeadLocked` keeps only `port` and `kill` items (`port.go:159`); a queued `PoisonPill` is dropped silently |
| `Terminated` that finds a full mailbox | dead-lettered with the watched subject as sender (ADR 0032 D5a), and suppressed (marker, `DL3`) | `DL3` [005]: sender `dls` when told by hand with no sender to a stopped actor; with a sender the sender is kept (S4). **The full-mailbox case itself (sender = subject, recipient = `deadLetters`) is not observed: it is inferred from the code** (`DeathWatch.scala:111`, `self.tell(Terminated(...), actor)`, sender = the watched actor, then `ActorRef.scala:716-743`). Complementary evidence: the oracle of PR #1525 (head `e5d2c8c4`, `docs/parity/oracles/unstash-deadletters-*`) observed sender = B and recipient = `deadLetters` for the `Terminated` retained after a stop; the full-mailbox case stays inferred | published as `DeadLetter{Terminated, Sender: subject, To: self}` (`watch.go:308`) |

Spec bodies (read, not the titles): `DeadLetterSupressionSpec.scala:41-72` (stopped actor): a suppressed and a plain message told with the `testActor` as sender; the `DeadLetter` probe gets `DeadLetter(NormalMsg, testActor, deadActor)` and nothing else (`:54-55`); the `SuppressedDeadLetter` probe gets `SuppressedDeadLetter(SuppressedMsg, testActor, system.deadLetters)` and nothing else (`:57-58`); the `AllDeadLetters` probe gets both, in that order, and nothing else (`:60-62`); through an `ActorSelection` the same messages are unwrapped (`:64-70`). `:74-106` (sent to `deadLetters`): the same three probes (`:84-97`, bounded to 200 ms by the spec; the Go port waits on a channel instead of sleeping), and the selection variant (`:99-105`). The line ranges the Owner named, `:46-70` and `:79-104`, are the bodies inside these two tests (subscriptions at `:43-49`, `:76-82`).

## Sites found (what changes in code)

`System.publish` (`deadletter.go:44-53`) is the one funnel of every `DeadLetter`. All publication sites call it: `ref.go:79, 85`, `stop.go:123`, `port.go:81, 118, 168`, `watch.go:308`, `ask_endpoint.go:67`, `deadletters_ref.go:58, 61`. Pekko classifies in one place too (the dead-letter ref). Classification in `publish` therefore touches no call site, no judge anchor and no counter.

The **three situations of divergence 5a** (oracle section 4: already stopped, full bounded mailbox, mailbox clean-up of an accepted stop) map to **two code points** plus the funnel:

| # | Situation | Code point | Change |
|---|---|---|---|
| 5a-1 | stopped actor, accepted stop, full bounded mailbox (`DL1`, `DL1b`, `DL7`) | `postLane`, the `!it.poison` guard (`port.go:80`) | delete the guard: the dead letter is published and `publish` classifies it |
| 5a-2 | queued when the stop is accepted (`DL8`, `DL8c`) | `takePortDeadLocked` (`port.go:157-163`; the condition `it.port || it.kill` is at `:159`, called from `watch.go:420`) and `publishPortDead` (`port.go:166-170`; its publish is at `:168`) | add `it.poison` to the kept items; `publishPortDead` publishes through the funnel |
| 5a-3 | the funnel | `System.publish` (`deadletter.go:44-53`) | if `Msg` implements the marker, publish `SuppressedDeadLetter{Msg, Sender (or deadLetters), To: deadLetters ID}` instead, to the stream only |

`Terminated` (watch.go:308) changes without a code edit there: its type gains the marker (slice 1) and the funnel classifies it.

## Proposed Go API (for the Owner at G2)

```go
// DeadLetterSuppression marks a message whose dead letter is published as SuppressedDeadLetter (ActorRef.scala:601).
type DeadLetterSuppression interface{ DeadLetterSuppression() }

// SuppressedDeadLetter: ActorRef.scala:609-614. Msg is never nil; Sender is never nil; To is the ID of System.DeadLetters().
type SuppressedDeadLetter struct { Msg DeadLetterSuppression; Sender Handle; To ID }

// AllDeadLetters: the common channel of DeadLetter and SuppressedDeadLetter (ActorRef.scala:579-583). Sealed.
type AllDeadLetters interface{ allDeadLetters() }
```

Subscribers use `Subscribe[SuppressedDeadLetter, M]`, `Subscribe[DeadLetter, M]` and `Subscribe[AllDeadLetters, M]` (ADR 0038 D2 already matches interfaces). `PoisonPill`'s type and `Terminated` implement the marker. `Kill` does not.

## Criteria, scenarios, sources and mutations

Level: every scenario below is a unit test, in `package actor_test`; none is an integration test and none uses a real engine.

Tests in `package actor_test` (PAT and PS at once, Owner decision of 2026-11-07), go-specs, spies for doubles. Mutations are named in the PR in a "Mutations" table, reproduced by QA (unmutated green, mutated red).

| # | Criterion | Scenario (test) | Source | Slice |
|---|---|---|---|---|
| S1 | A marked message to a stopped actor is a `SuppressedDeadLetter(msg, testActor, deadLetters)`, reaches the `SuppressedDeadLetter` and `AllDeadLetters` subscribers and not `DeadLetter`; a plain one is `DeadLetter(msg, testActor, deadActor)`; order on `AllDeadLetters` as in the spec | port of `DeadLetterSupressionSpec` first test, selection half is PORT-GAP | spec `:41-62`; oracle Option 1 | 1 |
| S2 | the same told to `System.DeadLetters()` (`DeadLetter(NormalMsg, testActor, deadLetters)`) | port of the second test | spec `:74-97`; `ActorRef.scala:736-743` | 1 |
| S3 | no sender: sender is `DeadLetters()` for both kinds; recipient of the suppressed event is `DeadLetters()` | `DL2`, `DL4` | oracle `DL2` [004], `DL4` [004] | 1 |
| S4 | `Terminated` is suppressed (a full bounded mailbox watcher: `SuppressedDeadLetter(Terminated, subject, deadLetters)`, *inferred from the code, not observed; see the table above*); `Kill` is a plain `DeadLetter` | `DL3` (with the real path of `watch.go:308`, not the hand-built helper), `DL4` | oracle `DL3` [005], `DL4` [004]; ADR 0032 D5a | 1 |
| S5 | `DeadLetter{Msg: marked, Sender: probe}` told to the sink is a `SuppressedDeadLetter` with sender `probe` and recipient `DeadLetters()`; the plain control keeps recipient and sender | `DL5` (a), (c); `DL5` (b) `PoisonPill` told to the sink | oracle `DL5` [004], [011], [018]; `ActorRef.scala:739` | 1 |
| S6 | an undelivered `PoisonPill` to a stopped actor publishes `SuppressedDeadLetter(PoisonPill, DeadLetters() or the explicit sender, DeadLetters())` and no `DeadLetter` | `DL1`, `DL1b` (replaces `TestPoisonPill_UndeliveredPublishesNoDeadLetter`, `poison_pill_test.go:141-143`, which pins today's behavior and carries `TODO(#871)`) | oracle `DL1` [006], `DL1b` [004] | 2 |
| S7 | same for a full bounded mailbox, with the plain control as `DeadLetter` | `DL7` | oracle `DL7` [010], [019] | 2 |
| S8 | same for a `PoisonPill` queued when the stop is accepted, plus the queued `m2` as `DeadLetter`; same with the actor stopping itself | `DL8`, `DL8c` | oracle `DL8` [011]-[014], `DL8c` [011]-[013] | 2 |
| S9 | an actor whose stop was accepted (`Stop`) and then told `PoisonPill` publishes the suppressed event | variant of the existing test "to an actor whose stop was accepted" | ADR 0010 5a list | 2 |
| S10 | the marker changes classification only: admission, counters (`Stats`) and the outcome for unmarked messages are unchanged; `SubscribeDeadLetters` (hub) callbacks and `deadLettersPublished` do not see suppressed events | negative cases in the same files | #871 AC1, AC3; D-4 | 1 |
| S11 | a published `SuppressedDeadLetter` never has a nil `Sender` or empty `To`; a nil `Msg` is refused by construction in `publish` (not exported constructor) | PAT | `ActorRef.scala:612-613` | 1 |

Mutations (names for the PR; the Dev measures them): M1 `publish` ignores the marker (all become `DeadLetter`); M2 recipient kept as the original actor; M3 `Kill` carries the marker; M4 the guard `!it.poison` restored; M5 `takePortDeadLocked` without `poison`; M6 `Terminated` without the marker; M7 suppressed event also sent to the hub. Each is a one-line edit with a unique anchor in `pekko-actor/actor`.

## Slices (one PR each, at most 400 authored lines, sequential)

Estimate (authored lines, counted from the sites above, to be re-measured by the Dev): by slice, from the parts: **slice 1 ~345** (`deadletter.go` types and `publish` branch ~65, markers ~10, port of the spec ~110, `DL2` to `DL5` ~70, update of `bounded_mailbox_terminated_test.go` ~30, API and negative tests S10/S11 ~60); **slice 2 ~215** (`port.go` ~10, `poison_pill.go` comments ~10, `DL1`, `DL1b`, `DL7`, `DL8`, `DL8c` ~100, `poison_pill_test.go` update ~25, README ~40, ADR and inventory ~30). Total ~560: more than 400, so two slices. **What counts for the 400** (`AGENTS.md` "PR budget"): authored lines of Go, tests, docs and the hand-written files of `migration/stage-1/suppressed-dead-letters/**` (manifest, RULEBOOK, RUNBOOK, reports); only generated files (depmap output, `// Code generated`), lockfiles and `.claude/skills/` are excluded, and the PR lists them. The kit artifacts are not part of the estimate above: the Dev adds their size and splits if the sum passes 400. **Pre-agreed spill-over:** if slice 1 passes 400, S10 and S11 (and their tests) move to slice 2; the update of the `Terminated` test stays in slice 1, because the marker on `Terminated` would otherwise break it.

| Slice | Content | Files | Lines (est.) |
|---|---|---|---|
| 1 | marker, `SuppressedDeadLetter`, `AllDeadLetters`, classification in `publish`, marker on `PoisonPill`'s type and on `Terminated`; S1 to S5, S10, S11; `watch.go:308` test adjusted | `deadletter.go`, `poison_pill.go` (marker method only), `watch.go` (marker method only), tests below | ~345 |
| 2 | 5a: the guard, the clean-up, S6 to S9; README; ADR 0010 and inventory edits | `port.go`, `poison_pill.go` (comments), tests, `pekko-actor/README.md`, ADR 0010, inventory | ~215 |

Slice 1 alone changes behavior for `Terminated` only (it becomes suppressed) and for a user type that opts in; the `PoisonPill` drop is untouched until slice 2, so the intermediate state is consistent and each PR is mergeable.

## Documents edited

- **Inventory `docs/parity/actor-inventory.md`, row C-10b** (slice 2): the evidence is recorded (PAT/PS tests of S1 and S2 in the external package with `path:line`; PORT-GAPs: `ActorSelection` halves `spec :64-70, :99-105`, `WrappedMessage`/listener `DL6`, `DL8b`), "Go state" becomes `A` when slice 2 merges, "Missing" lists V2 (a Go judge scenario) and V3. **The V level is assigned by the Owner; this unit does not set it.** C-10a is not edited (#1479's table).
- **ADR 0010** (slice 2): divergence 5a gets an implementation note dated by the merge: delivered, with the three situations and the oracle lines; the Amendment's text is not rewritten; 5b and 5c stay open (#1320, #1321). #770 stays open until its own PR cites this one.
- **ADR 0038 D7**: unchanged by this scope (it already says `SuppressedDeadLetter` is published through the stream when its story lands).

## Exports (EXP-1: symbol, consumer, why the public API is not enough, what it does not expose, test)

| Symbol | Consumer | Why the public API is not enough | Does not expose | Test |
|---|---|---|---|---|
| `DeadLetterSuppression` | application messages; `pekko-actor-typed` later for Typed signals | the spec's `SuppressedMsg` is a user type that opts in; a sealed marker cannot be implemented outside the package | no behavior, only classification | S1, S2 |
| `SuppressedDeadLetter` | event-stream subscribers (observability, tests) | it is the event; there is no existing type to observe it with | no constructor, no `Context` | S1 to S8 |
| `AllDeadLetters` | subscribers that want both kinds (spec `:48-49`) | Go cannot subscribe to two channels with one registration | sealed (no method outside the package) | S1, S2 |
| `Terminated` and `PoisonPill` marker methods | none new (they are the same exported types) | the marker is a method on exported/internal types | nothing more than `DeadLetterSuppression()` | S4, S6 |

No export is added to `Ref`, `Handle` or `Context`.

## Allowed files

| Slice | Paths |
|---|---|
| 1 | `pekko-actor/actor/deadletter.go`, `pekko-actor/actor/poison_pill.go`, `pekko-actor/actor/watch.go`; new tests `pekko-actor/actor/suppressed_deadletter_spec_test.go` (port of the spec) and `pekko-actor/actor/suppressed_deadletter_oracle_test.go` (`DL2` to `DL5`); `pekko-actor/actor/bounded_mailbox_terminated_test.go` if its `Terminated` dead-letter assertion needs the new channel (`:110`); `migration/stage-1/suppressed-dead-letters/**`, `migration/cost-log.tsv` (one row) |
| 2 | `pekko-actor/actor/port.go`, `pekko-actor/actor/poison_pill.go`, `pekko-actor/actor/poison_pill_test.go`, new `pekko-actor/actor/suppressed_deadletter_poison_test.go` (`DL1`, `DL1b`, `DL7`, `DL8`, `DL8c`), `pekko-actor/README.md`, `docs/adr/0010-actor-api.md`, `docs/parity/actor-inventory.md`, kit package and cost-log row |

Go files this unit touches: `deadletter.go`, `poison_pill.go`, `watch.go` (marker method only, one line), `port.go` and the tests above. **It touches `pekko-actor/README.md` (slice 2).** Nothing under `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, another module or `judge/` changes.

**Merge order of `pekko-actor/README.md` (Owner, G2):** #871 merges first; #821 and #1182 slice B then rebase their README hunks onto it.

**Independence from the other lanes (table, not yet confirmed against their scopes):**

| Pair | Shared file | Resolution |
|---|---|---|
| this and lane 3 (#821, scheduler) | `pekko-actor/README.md` only, if the scheduler adds `scheduler*.go` files | Go sources are disjoint (scheduler adds new files; this unit edits `deadletter.go`, `port.go`, `watch.go`, `poison_pill.go`); README: merge in order and rebase the later README hunk |
| this and lane 2 (Typed, #927/#1182) | none (`pekko-actor-typed`) unless the Typed lane reads `DeadLetter` | disjoint; `pekko-actor-typed` and `pekko-persistence` read only `d.Msg` and compile unchanged (to be re-verified by the Dev with grep) |
| this and #1479 (dead-letter log) | `deadletter.go` | #1479 starts after slice 2 and must respect the suppression (`DL6`: a `DeadLetter` whose message wraps a suppressed payload is skipped by Pekko's listener; Go has no `WrappedMessage`, so #1479 decides it) |

## Judge, campaign and gates (this unit touches `pekko-actor/`)

- **Anchors.** `judge/mutate_go.py` has anchors in `ref.go`, `stop.go`, `watch.go`, `cell.go` and others. The edits above touch no anchored line (checked: no anchor contains `takePortDeadLocked`, `postLane`, `poison`, `dropped` or the `publish` function; GM62 anchors `ref.go` lines this unit keeps). The Dev re-checks with `scripts/check-judge-matrix.py` on the head; if an anchor breaks, the same PR updates it with the same contract.
- **QA** runs `scripts/check-judge-matrix.py` on the head: exit 1 is REJECT. Gates on the head: gofmt, vet, `go test -race`, 85 % coverage per module, diff coverage 85 %, gocyclo <= 10 (the `publish` branch must stay under 10; extract a helper otherwise).
- **After the merge** of the code slices: full Go campaign on this Mac (`--runs 30`, `-race`) on a clean `develop`, report as is, PROVENANCE, package in `goPekko-evidence/` (`COPYFILE_DISABLE=1`, `verify_package.py`, `.sha256`, `SHA256SUMS`), tag `evidence/go-campaign-<sha7>`. If two code merges land together, one campaign covers both. Meanwhile the inventory carries the dated note that V2/V3 are not in force. This unit does not run it.
- No Pekko-side campaign: no Pekko judge input changes.

## Decisions for the Owner (G2)

- **D-1. Kit run required** (port plus adaptation). Alternative: waive; not proposed.
- **D-2. Marker form.** Recommended: an exported interface with an exported marker method, `DeadLetterSuppression()`, so application types opt in (the spec requires it). Alternative: sealed (internal types only), which cannot port `SuppressedMsg`; not proposed.
- **D-3. Event shape.** Recommended: `SuppressedDeadLetter{Msg DeadLetterSuppression, Sender Handle, To ID}`, same field names as `DeadLetter`, `To` always the `deadLetters` ID (oracle S4). Alternative: carry the original recipient; **contradicts the oracle**, not proposed.
- **D-4. Hub and counter.** Recommended: a suppressed event goes to the stream only; the deprecated `SubscribeDeadLetters` hub and `deadLettersPublished` keep counting `DeadLetter` only (Pekko's default listener does not see it, S7; the hub is replaced by ADR 0038 D8). Alternative: also deliver to the hub as a `DeadLetter`; would change the meaning of ~42 test files' callbacks.
- **D-5. `AllDeadLetters`.** Recommended: a sealed marker interface for subscription; consumers type-switch on the two structs. Alternative: accessor methods (`Message()`, `Sender()`, `Recipient()`), which clash with the field names; not proposed. `Dropped` joins it in #906.
- **D-6. Classification place (approved).** `System.publish` is a single funnel, the equivalent of Pekko's `DeadLetterActorRef` (every dead letter goes through `deadLetters`, which classifies: `ActorRef.scala:716-724`, `:736-743`). No call site is edited and no judge anchor is touched. Alternative (rejected): classify at each of the ten sites.
- **D-7. `Terminated` to a full mailbox changes from `DeadLetter` to `SuppressedDeadLetter` in slice 1** (it follows from the marker, `DL3`; the existing test at `bounded_mailbox_terminated_test.go:110` changes). The sender stays the watched subject (D5a). Recommended: yes, as Pekko. Alternative: leave `Terminated` unmarked until a later unit; it would keep a known behavioral divergence, which the Owner's rule of 2026-11-04 does not accept. **Approved with this wording:** the edit of `bounded_mailbox_terminated_test.go:110` corrects the expected value to what Pekko does (oracle #1508, `DL3` [005] and `DL7`: a `Terminated`/`PoisonPill` that is dead-lettered is a `SuppressedDeadLetter` with recipient `deadLetters`); it does **not** weaken the test: the assertion `all[0].To == ref.ID()` at `bounded_mailbox_terminated_test.go:113` becomes `All[0].To == sys.DeadLetters().ID()` (Pekko's recipient is `deadLetters`), and the test moves from the hub (`dlRec`) to a subscription to the stream (`SuppressedDeadLetter`); subject (`:110-112`), sender and count assertions are kept. The slice 1 PR body says so.
- **D-8. Slices.** Accept 1 and 2 (about 345 and 215 authored lines, before kit artifacts, with the spill-over rule in "Slices"). Alternative: one PR of ~560 lines; not proposed.
- **D-9. Inventory (approved).** Slice 2 records the evidence in row C-10b (PAT/PS tests of S1 and S2, PORT-GAPs listed) but **does not set the V level: the Owner assigns it.** V2 needs a Go judge scenario with a detecting mutation CURRENT, a later unit.
- **D-10. Not covered, recorded as PORT-GAPs with markers:** the `ActorSelection` halves of the spec (`:64-70`, `:99-105`; no selection in Go), `Identify`/`Watch` on the sink (PORT-GAP(dl-watch-896)), `WrappedMessage` and the listener skip (`DL6`, #1479), `Terminate()` as a suppressed system message (no counterpart), 5b (#1320) and 5c (#1321), and the `DL8b` race.

## Out of scope

#770 (it is unblocked, not edited), #1479 (the INFO "was not delivered" log and the listener), #906 (`Dropped`), #897 (`UnhandledMessage`), #1320, #1321, a Go judge scenario for C-10b, `ActorSelection`, Typed.

## Appetite

Two small PRs, about two working sessions. The risk is slice 2: the clean-up path (`takePortDeadLocked`) runs under `c.mu` and finalization order is guarded by existing tests (stop ordering, ADR 0024 D5).

## Definition of Done

- S1 to S11 have their tests and pass; the PRs carry the Mutations table reproduced by QA; `grep -rn "TODO(#871)\|OPEN DIVERGENCE 5a" pekko-actor` is empty after slice 2.
- `scripts/each_module.sh go test -count=1 ./...`, `scripts/check_deps.sh`, gocyclo <= 10 and the 85 % coverage gate with the commands of `AGENTS.md`; `-race` only as the stage rule above demands; headers (`check-headers.sh`) on ported files.
- README follows the module README contract: implemented behavior, PORT-GAPs, the divergence 5a text rewritten as delivered, a Mermaid diagram of the classification if it clarifies.
- PRs say `Refs #871`, never `Closes`; #871 is closed by hand after slice 2; #770 is not touched.
- Evidence is a comment bound to the final head SHA with real exit codes; QA in a clean context bound to the FINAL head, re-QA after any later change.
