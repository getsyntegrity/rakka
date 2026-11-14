# Scope: Expose the local `deadLetters` reference and carry the sender in `DeadLetter` (#896, MSG-08)

Status: **approved at G2 by the Owner, 2026-11-10**, with D-1 to D-10 as written (D-2: an absent sender is `DeadLetters()` only in the dead-letter view, `ActorRef.scala:589`; `Context.Sender` stays with ADR 0032 D3, and revisiting it is another decision). First drafted on 2026-11-09 as a draft for gate G2. Written on 2026-11-09 against `develop` `46ce37ac`. #896 is `prio:P1`, `status:blocked`; its criteria are those of the issue, the ACT2 reconciliation of 2026-10-24 (AC2, the ignore reference, moves to [#1087](https://github.com/getsyntegrity/gopekko/issues/1087)) and the Owner's criterion of 2026-11-07 (#1481). Order decided by the Owner on 2026-11-09: this is the first EventStream-lane unit; [#871](https://github.com/getsyntegrity/gopekko/issues/871) (marker, `SuppressedDeadLetter`, divergence 5a of ADR 0010) follows as its own unit and is not folded in here.
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: the unit ports `DeadLetter` with its sender (`ActorRef.scala:592-595`), the substitution of an absent sender by `provider.deadLetters` (`ActorRef.scala:689, 712, 742`, `dispatch/AbstractDispatcher.scala:43-52`) and `DeadLetterActorRef` (`ActorRef.scala:733-756`); it adapts `ActorRefProvider.deadLetters` (`ActorRefProvider.scala:69`), so the Dev runs the migration kit (`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK under `migration/stage-1/deadletters-ref-and-sender/` (kit-package name: INFERRED from the sibling `migration/stage-1/eventstream-publish/`; the Dev may rename it in slice 0). Ported: `DeadLetter.sender`, the substitution, `DeadLetterActorRef.!` for a non-`DeadLetter` message. Adapted: the reference is a `Ref[any]` of a `System`, not an `InternalActorRef` of a provider; its `ActorPath` (ADR 0031). Pekko-side tests ported: `ActorSystemSpec.scala:164-186` (sender of "boom") and `:188-210` (no sender), `DeadLetterSupressionSpec.scala:87` (a dead letter keeps the explicit sender `testActor`: it backs S4, `TellFrom`, and does not show a no-sender letter), `MailboxConfigSpec.scala:69` (full bounded mailbox, sender `deadLetters`). PORT-GAP: `specialHandle` for `Watch`, `Identify`, `ActorSelectionMessage` (`ActorRef.scala:693-721` (`EmptyLocalActorRef.specialHandle`) and `:746-753` (`DeadLetterActorRef.specialHandle`); no counterpart in Go, D-6), `Dropped` (#906), the INFO/WARNING logs (#1479). The Dev is a separate session, QA is independent of both, and the TL does not implement.

## Contracts consumed

Contract sources: [ADR 0032](../adr/0032-local-sender-envelope.md) (D3 "absent, revisit once #896 defines `deadLetters`", D4 `TellFrom`, D6 "Keep"; rule 2 of the envelope), [ADR 0011](../adr/0011-pekko-actor-layout.md) (owner decision 2: `actor/` stays one cohesive package) and EXP-1 (`docs/parity/typed-owner-decisions-2026-10-27.md:36`: every `pekko-actor` export needed by another module is justified in the story that introduces it), [ADR 0038](../adr/0038-event-stream-subscription-semantics.md), the oracle [eventstream-spec-2026-11-06.md](../parity/oracles/eventstream-spec-2026-11-06.md) (S8, S9; EV8 [006], [008]), [scopes/1475-public-ports.md](1475-public-ports.md) AC-2, the issue #896 and its reconciliation.

## Objective

As a Go actor application developer, I can obtain the local `deadLetters` reference of a `System`, send to it, and read the **sender** of every `DeadLetter` the runtime publishes: the sender of the `TellFrom` that failed, and, when the message was sent without one, the `deadLetters` reference of that `System` (Pekko `DeadLetter(message, sender, recipient)`, `ActorRef.scala:592-595`; "When this message was sent without a sender, `sender` will be `system.deadLetters`", `ActorRef.scala:589`). The Owner's criterion closes the loop on [#1481](https://github.com/getsyntegrity/gopekko/issues/1481): the sender assertion is added to **P1** and `PORT-GAP(dl-sender-896)` is removed.

P1 is the test `TestActorSystemSpec_LogDeadLetters` in `pekko-actor/actor/actorsystem_deadletters_spec_test.go:40-107` (port of `ActorSystemSpec.scala:164`, "log dead letters"; the name comes from `docs/scopes/1475-public-ports.md` AC-2 and the PORT-GAP comment at `:53-59`). The marker is at `:53-59`. The sender of "boom" is `p`, told with `TellFrom` at `:89` (the settle letter's `TellFrom` is `:95`).

## Evidence: Pekko, and what goPekko does today (`develop` `46ce37ac`)

| Case | Pekko 1.7.1 | goPekko |
|---|---|---|
| `DeadLetter` shape | `DeadLetter(message, sender, recipient)`, sender and recipient non-null (`ActorRef.scala:592-595`) | `DeadLetter{Msg any; To ID}` (`deadletter.go:22-25`); no sender |
| Dead letter to a stopped actor or full bounded mailbox | `DeadLetter(handle.message, handle.sender, receiver)` (`Mailbox.scala:434, 567, 615, 625`; `:1031` for a bounded control-aware user queue) | `tell` publishes `DeadLetter{Msg, To}` and drops the sender, though the envelope has it (`ref.go:75, 81`; the comment `ref.go:53` says "until #896") |
| Message queued then lost at stop | `Mailbox.cleanUp` sends each envelope with its sender to `deadLetters` | `stop.go:123` publishes `e.msg`; `e.sender` is available in `envelope[T]` (`mailbox.go:23-26`) and is dropped |
| Send without sender | `Envelope.apply` stores `system.deadLetters` (`AbstractDispatcher.scala:51`); `DeadLetterActorRef.!` and `EmptyLocalActorRef` substitute it again (`ActorRef.scala:689, 712, 742`) | the envelope holds a nil `Handle` (ADR 0032 rule 2, D3); `Context.Sender` returns `(nil, false)` |
| `provider.deadLetters` | public (`ActorRefProvider.scala:69`); `ignoreRef` is `@InternalApi` (`:72`) | none; `README.md:912` says "`deadLetters` are not implemented" |
| Reader of a dead letter with no sender | `DeadLetter(boom, deadLetters, a)` (`MailboxConfigSpec.scala:69`, `AbstractDispatcher.scala:51`; oracle EV8 [006], [008]) | cannot be told apart from "sender dropped" |

## Sites found

**Production, 8 constructions of `DeadLetter` in `pekko-actor/actor`** (all call `System.publish`, `deadletter.go:39`): `ref.go:75` and `:81` (`tell`: stopped or full; the `sender` parameter is in scope), `stop.go:123` (pending envelopes at stop; `e.sender` in scope), `port.go:81` (`deliverPort` without `PortHandler`), `port.go:118` (`postLane`: Kill, PoisonPill-less posted items; no sender in the lane item), `port.go:168` (`publishPortDead`), `watch.go:308` (`Terminated` that finds a full mailbox; Pekko's sender of `Terminated` is the subject, ADR 0032 D5a, `#814`), `ask_endpoint.go:67` (late reply, `Tell` has no sender). No other module constructs a `DeadLetter` (`pekko-actor-typed`, `pekko-persistence`, `gopekko-testint`: they only read `d.Msg`; checked by `grep`).

**Tests that assert or build a `DeadLetter` literal, 12 files named below plus `pekko-persistence/persistent/snapshot_bounded_mailbox_test.go:68`, which returns a zero `DeadLetter{}` and compiles unchanged; ~33 literals, some through the `aftmRequest` helper or as text only** (a nil `Sender` that becomes `deadLetters` breaks every `ToEqual` on a literal): `ask_cancel_internal_test.go:266, 420`; `ask_endpoint_internal_test.go:102`; `ask_endpoint_promiseref_internal_test.go:78`; `ask_firstreply_internal_test.go:167, 219-233, 380, 428`; `ask_terminated_internal_test.go:81, 119, 141, 151, 185, 258`; `ask_timeout_internal_test.go:166, 197, 534`; `deadletter_internal_test.go:21-24` (inputs to `publish`); `eventstream_deadletter_internal_test.go:33, 53` (inputs); `eventstream_deadletter_test.go:65, 92, 146` (inputs told as messages, unaffected); `registry_characterization_internal_test.go:275, 293`; `sender_test.go:238` (`S8`, **asserts `{Msg: 2, To: r.ID()}` after `TellFrom(..., x)`: it pins the dropped sender and must change**); `kill_test.go:241` (text only). `pekko-actor/internal/deadletter/hub_test.go:24` defines its own `DeadLetter` and is unaffected. Docs that say "`{Msg, To}`": `pekko-actor/README.md:755, 912`, comments `ref.go:53`, `ask.go:23`.

## Proposed Go API (for the Owner at G2)

```go
// DeadLetter gains the sender; To (the recipient) is unchanged.
type DeadLetter struct {
	Msg    any
	Sender Handle // never nil in a published DeadLetter: the TellFrom sender, or System.DeadLetters() when none
	To     ID
}

// DeadLetters is the local dead-letter sink of the System (Pekko provider.deadLetters, ActorRefProvider.scala:69).
// Sending to it publishes DeadLetter{Msg, Sender, To: its own ID}; a DeadLetter sent to it is published as is.
func (s *System) DeadLetters() Ref[any]
```

`Sender` is a `Handle` (as `Context.Sender`, ADR 0032 rule 5), so no `Ref[any]` conversion is needed to read it. The substitution is done once, in `System.publish` (`deadletter.go:39`): the eight sites pass the sender they have (or nil) and `publish` replaces nil by `s.DeadLetters()`. `TellFrom` towards `DeadLetters()` needs a runtime-internal path (the `target.(*ref[T])` test at `ref.go:55` would fall to `Tell` and drop the sender): an unexported interface, not an exported method (D-3). Envelope data and `Context.Sender` are **not** changed: ADR 0032 D3 stays "absent" for a receiving actor; only the dead-letter view substitutes (D-2).

## Criteria, scenarios, sources and mutations

Tests: go-specs, channels or gates, no sleeps, no `-race`. A mutation is applied in a disposable worktree and never committed; the named test must fail by assertion.

| # | Criterion | Scenario (test) | Source / oracle | Mutation | Slice |
|---|---|---|---|---|---|
| S1 | **(Owner, #1481)** P1 asserts that the dead letter of "boom" has `Sender == p` (by `ID`), for the first letter, and that the settle letter also carries `p`; `PORT-GAP(dl-sender-896)` and its comment (`actorsystem_deadletters_spec_test.go:53-59`) are removed (`grep dl-sender-896` over the repository is empty) | `TestActorSystemSpec_LogDeadLetters` (edited, same file) | `ActorSystemSpec.scala:164-186`; `Mailbox.scala:434, 567` | `tell` publishes `Sender: nil` | 1a |
| S2 | A message told with `TellFrom` to a stopped actor, and one that overflows a bounded mailbox, publish `DeadLetter{Msg, Sender: the sender, To}`; a message pending at stop keeps its sender | `TestDeadLetterCarriesTheSender` (stopped, full, pending-at-stop; edits `sender_test.go` S8 `:238`) | `Mailbox.scala:434, 567, 615, 625`; EV8 [006] | `stop.go:123` drops `e.sender` | 1a |
| S3 | A message told with `Tell` (no sender) is published with `Sender` equal to `sys.DeadLetters()` (`ID`), not nil; this holds for every one of the eight sites (the table-driven case covers `tell` stopped, `tell` full, pending-at-stop, `postLane`, ask late reply, `Terminated` to a full mailbox, `deliverPort` without a `PortHandler` (`port.go:81`), `publishPortDead` (`port.go:168`)) | `TestDeadLetterWithoutSenderIsDeadLetters` | `ActorRef.scala:589, 712, 742`; `AbstractDispatcher.scala:51`; `ActorSystemSpec.scala:188-210` (log text only: indirect for the sender); `MailboxConfigSpec.scala:69` | `publish` does not substitute | 1b |
| S4 | Public `deadLetters`: `sys.DeadLetters()` is stable (same `ID` on every call), distinct per `System`, and `Tell(msg)` to it publishes `DeadLetter{Msg: msg, Sender: DeadLetters(), To: DeadLetters().ID()}`; `TellFrom(..., x)` to it publishes `Sender: x`; a `DeadLetter` told to it is published unchanged (not wrapped again) | `TestDeadLettersRefPublishes` | `ActorRef.scala:739-743` | wrap a `DeadLetter` again | 1b |
| S5 | AC3 of the issue: the sink is not a normal application actor: it is not in the registry or the children of any actor (`System.Done(DeadLetters())` is `ErrUnknownHandle`), does not count in `Stats`/live counters, and replaces nothing under a user name; its `Receive` is not user code. The `Watch` half of AC3 is not ported here: it is `PORT-GAP(dl-watch-896)` unless D-6 chooses to port `ActorRef.scala:746-753`, so AC3 is partial in this unit | `TestDeadLettersIsNotAnApplicationActor` | issue AC3; `ActorRefProvider.scala:69` | register it in the registry | 1b |
| S6 | The sender substitution does not loop: a `DeadLetter` published to the stream whose recipient is `deadLetters` does not re-publish (the `EVT-03-deadletter-cycle` rule of #870 is unchanged) | `TestDeadLetterCycleRuleUnchanged` (existing `eventstream_deadletter_internal_test.go` re-run with the new field) | scope 870 D-3 | none (regression) | 1b |
| S7 | Negative and race boundary named by the issue: concurrent `TellFrom`s from 64 senders to one stopped actor publish each sender with its own message (no sender swapped between letters); `-count=100` | `TestDeadLetterSenderUnderConcurrency` | `Envelope` is immutable (`AbstractDispatcher.scala:43-52`) | `publish` reuses the sender of the previous letter | 1b |
| S8 | The existing literals are migrated (`Sender: sys.DeadLetters()` or a helper) and the ~33 sites above still pass with the same assertions on `Msg` and `To` | the files listed under "Sites found" | none | none | 1b |
| S9 | README: the limitation line `README.md:912` is rewritten (sender and `deadLetters` implemented; `Forward` #854, ignore reference #1087, `SuppressedDeadLetter` #871 still not); `README.md:755` say `DeadLetter{Msg, Sender, To}`; the ADR 0011 justification of `DeadLetters` and of the new field is recorded in "Open API choices" below; `ref.go:53` and `ask.go:23` comments updated | review | module README contract | none | 2 |

The issue's AC2 (ignore reference) is **not** in this unit: reconciliation of 2026-10-24 moves it to #1087 (`ignoreRef` is `@InternalApi private[pekko]` in Classic, `ActorRefProvider.scala:72`; public only in Typed). AC2 was removed from the issue body by the Owner on 2026-11-10 (D-4). Every PR sentence that says something was observed names the oracle line or the assertion.

## Slices (one PR each, at most 400 authored lines, sequential)

Estimate: the TL wrote no prototype; raw numbers times 1.55 (the deviation measured in #976).

| Slice | Delivers | Files | Criteria | Est. authored lines (raw → x1.55) |
|---|---|---|---|---|
| **0. Kit package** | manifest, RULEBOOK, RUNBOOK | `migration/stage-1/deadletters-ref-and-sender/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | kit run | ~120 docs |
| **1a. The sender is carried** | `DeadLetter.Sender` (nil when absent, no substitution yet); `tell`, `stop.go` pass the sender they hold; P1 asserts it, PORT-GAP removed; S8 case of `sender_test.go` changed | `pekko-actor/actor/deadletter.go`, `ref.go`, `stop.go`, `actorsystem_deadletters_spec_test.go`, `sender_test.go`, a new `deadletter_sender_test.go` | S1, S2 | ~25 + ~110 → ~210 |
| **1b. `deadLetters` and substitution** | `System.DeadLetters()`, the sink type, the substitution in `publish`, the internal `TellFrom` path, migration of the existing literals | `pekko-actor/actor/deadletter.go`, `deadletters_ref.go` (new), `ref.go` (only the `TellFrom` branch), `system.go` (field and constructor), the test files under "Sites found", `deadletters_ref_test.go` (new) | S3 to S8 | ~90 prod + ~120 new tests + ~35 edited literals → ~380 (**at the limit; the Dev splits the literal migration into a 1c if the diff passes 400**) |
| **2. Docs and record** | README, `EXECUTION-RECORD.md`, cost-log row, the record linked from #1020 | `pekko-actor/README.md`, `migration/stage-1/deadletters-ref-and-sender/EXECUTION-RECORD.md`, `migration/cost-log.tsv` | S9 | ~80 docs |

Order: 0, 1a, 1b, 2. Why this order: 1a is useful alone (it closes the Owner's criterion for the explicit-sender part and breaks only the one literal at `sender_test.go:238`); 1b changes the value every no-sender letter carries, which is what touches the 33 literals, so it must be reviewed alone. If the Owner prefers one PR it is ~590 authored lines and does not fit the 400 budget. The README is not touched before slice 2, so after 1a and 1b its limitation line `README.md:912` ("`deadLetters` are not implemented") is stale until slice 2 merges; the alternative is to move that one line into 1b.

## Allowed files

Exactly the paths in the slice table. Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `scripts/deps.allow` or another module changes (`pekko-actor-typed` and `pekko-persistence` only read `d.Msg`, so they compile unchanged; verified by grep, to be re-verified by the Dev with `scripts/each_module.sh go build ./...`).

| Pair | Shared file | Resolution |
|---|---|---|
| 1a, 1b | `deadletter.go`, `ref.go` | sequential; 1b starts from the merged 1a |
| This scope and the #776 scheduler ADR scope | **`pekko-actor/README.md`** | **shared; merge order matters.** Whichever merges second rebases its README hunks. Slice 2 is the only slice here that edits the README, so it should be opened after the #776 scope's README change merges, or rebased on it. No other shared file is known (the #776 scope (PR #1502) touches only the README beyond its own files) |
| #871 (next unit) | `deadletter.go`, `port.go`, `watch.go` | #871 starts after slice 2 of this scope is merged |

**Judge.** `pekko-actor/` changes in 1a and 1b, so the Go mutation report `judge/go-mutation-report.tsv` stops being CURRENT (`docs/judge.md` rule: CURRENT only while the git tree of `pekko-actor/` is unchanged, README included). This unit does **not** rerun it. ONE Go campaign is planned after #896 **and** #871 are both merged, to avoid two campaigns against a tree that is about to change again. Until then the Go CURRENT is stale and the PRs say so.

## Open API choices (EXP-1 of `docs/parity/typed-owner-decisions-2026-10-27.md:36` with ADR 0011 owner decision 2, recorded here; the index #1020 links this section)

| Candidate export | Consumers | Concrete benefit (ADR 0011 criterion) | Alternative considered |
|---|---|---|---|
| `DeadLetter.Sender` (field of an already exported struct) | event-stream subscribers of `DeadLetter` (#1426 migration), `pekko-actor-typed` (#1169 stash dead-letter path uses `deadLetters` as sender), the testkit probes | a subscriber cannot tell who sent a lost message otherwise; ADR 0032 D6 "Keep" | not exported: contradicts D6 |
| `(*System).DeadLetters() Ref[any]` | #1169 (Typed stash, `enables` edge of the reconciliation), #871/#906 (producers of `SuppressedDeadLetter` and `Dropped` need it as default sender), user code sending to the sink | Typed cannot name the sink otherwise; Pekko exposes `system.deadLetters` | `Handle` only: cannot be sent to, which AC1 (the reference "publishes the expected sender/recipient metadata") needs |

## Decisions for the Owner (G2)

- **D-1. Kit run required** (a port plus an adaptation). Alternative: waive; not proposed.
- **D-2. Type and absence of `Sender`.** Recommended: `Sender Handle`, never nil in a published `DeadLetter`; an absent sender is `System.DeadLetters()` **in the dead-letter view only**; `Context.Sender` and the envelope keep ADR 0032 D3 ("absent", nil). This is the revisit D3 asks for, scoped to dead letters (Pekko substitutes at the envelope, `AbstractDispatcher.scala:51`, so a Pekko actor's `sender()` is `deadLetters` too; making `Context.Sender` do the same is a separate, larger decision and is **not** taken here). Alternatives: nil stays absent in `DeadLetter` (no churn on 33 literals, but differs from `ActorRef.scala:589` and from every Pekko assertion cited); or substitute at the envelope (changes `Context.Sender`, #814's contract). The Owner chooses; the substitution inside `System.publish` makes the first alternative a one-line change.
- **D-3. Exports.** Recommended: `DeadLetter.Sender` and `(*System).DeadLetters() Ref[any]` only, justified in "Open API choices"; the sender-aware `TellFrom` path is an unexported interface; the sink type is unexported. Alternative: `DeadLetters() Handle` (smaller, but cannot be sent to).
- **D-4. The ignore reference.** Recommended: not in this unit (reconciliation of 2026-10-24: Classic `ignoreRef` is internal, #1087 owns it). Decided by the Owner on 2026-11-10: not here, and AC2 is removed from the issue body. (Had it been included, it would have added a sink type, an export and ~120 lines and a slice: an "ignore reference" is a `Ref[any]` that consumes every message without running user `Receive` and publishes nothing (`IgnoreActorRef`, `ActorRef.scala:529-541`); UNVERIFIED: its observable behavior was not read beyond the line anchors, and it no longer matters for this unit. The reconciliation of 2026-10-24 is in the body of #896 ("Reconciliation outcome: SPLIT" and "Acceptance-criteria changes required before implementation": AC2 moves to #1087, `GOV-38`, Classic internal versus Typed public; `ActorRefProvider.scala:72` `ignoreRef` is `@InternalApi`). Decided by the Owner on 2026-11-10: AC2 is removed from the issue body.
- **D-5. `Terminated` to a full mailbox (`watch.go:308`).** Its sender is the watched subject in Pekko (ADR 0032 D5a, accepted). Recommended: carry the subject as `Sender` if the 814 delivery already holds it at that site; otherwise `deadLetters` and a TODO(MSG-08-terminated-sender). The Dev reads `watch.go:296-312` and reports.
- **D-6. `Watch` and `Identify` on the sink** (`ActorRef.scala:746-753`, `DeadLetterActorRef.specialHandle`, read in this revision: a `Watch` on the sink whose watchee and watcher are both not the sink makes the sink send `DeathWatchNotification(watchee, existenceConfirmed = false, addressTerminated = false)` to the watcher, which turns it into a `Terminated`, and the `Watch` is consumed; everything else goes to `EmptyLocalActorRef`) are not ported: what `Context.Watch(DeadLetters())` does today follows the existing rule for a handle the registry never issued (UNVERIFIED which result that is; the Dev records it in S5). Alternative: port the Pekko behavior; not proposed (no consumer).
- **D-7. ID and path of the sink.** Unknown: how an ID outside the registry is minted without colliding with `idCounter` (`id.go:20-26`) and what `PathOf` returns (Pekko path `/deadLetters`, `ActorRefProvider.scala:414-417`). The Dev proposes both in slice 0 under the RULEBOOK; a path that cannot be built becomes a PORT-GAP with a marker.
- **D-8. Size.** Accept the split 0, 1a, 1b, 2 (and a 1c if 1b passes 400). Alternative: one PR of ~590 lines; not proposed.
- **D-9. Status.** #896 to `status:ready` when this scope is approved; its dependencies #814, #870 and #793 are delivered (EventStream registration and publish are on `develop`, slice references in the scopes of #836 and #870). The Owner sets the label.
- **D-10. ADR 0038 D7.** `docs/adr/0038-event-stream-subscription-semantics.md:49` says that `DeadLetter` gains its sender in #871/#906; this scope assigns it to #896 (the issue, ADR 0032 D6 and the Owner's order of 2026-11-09). D7 needs an editorial amendment; asked here, not made here.

## Out of scope

`DeadLetterSuppression`, `SuppressedDeadLetter` and divergence 5a (ADR 0010, the undelivered `PoisonPill`) are [#871](https://github.com/getsyntegrity/gopekko/issues/871); `Dropped` is [#906](https://github.com/getsyntegrity/gopekko/issues/906); `UnhandledMessage` is [#897](https://github.com/getsyntegrity/gopekko/issues/897); the ignore reference is [#1087](https://github.com/getsyntegrity/gopekko/issues/1087); `Forward` is #854; the INFO/WARNING logs of dead letters are [#1479](https://github.com/getsyntegrity/gopekko/issues/1479); migration of the callback users and removal of the hub are #1426; `Context.Sender` substitution (D-2 alternative) is not decided here.

## Appetite

Four small PRs (0, 1a, 1b, 2), about three working sessions. The risk is slice 1b: the value of a field that ~33 assertions read changes at once, and `TellFrom` must reach the sink without exporting a method.

## Definition of Done

- S1 to S9 have their tests (or review) and pass; PRs name the mutations in a "Mutations" table reproduced by QA (unmutated green, mutated red); `grep -rn dl-sender-896` is empty.
- `scripts/each_module.sh go test -count=1 ./...`, `scripts/check_deps.sh`, the complexity lint (gocyclo <= 10) and the coverage gate (85 % per module; diff coverage >= 85 %) with the commands of `AGENTS.md`; `-count=100` on S7; no `-race`.
- Every ported file carries Apache-2.0 and one `// Ported from ...` line (`check-headers.sh`); the README follows the module README contract.
- PRs say `Refs #896`, never `Closes`; the issue is closed by hand after slice 2, with a comment that AC2 moved to #1087. The Go judge campaign is not run here (see "Judge").
- Evidence is a comment bound to the final head SHA with real exit codes.
