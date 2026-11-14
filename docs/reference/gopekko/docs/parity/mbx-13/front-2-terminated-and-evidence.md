# MBX-13 (#791), front 2: lane items, `Terminated` placement and evidence classes

Branch `ao/mailbox-verify/dev2-terminated-adr0030`. Baseline: goPekko `develop` `80b947bb396678d2de88890b07ee9070255f3fe0`. Pekko pin: v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`upstream/pekko`, read-only). Nothing here claims parity with Pekko, that remote CI ran, or the completion of a sibling story.

## What this document is

Issue #791 asks us to close the traceability of a bounded mailbox that already exists in `develop` (ADR 0030, accepted by the Owner on 2026-10-22; implementation squash `bdae37f6`, PR #686). This front covers one slice of it: how items of the "lane" are counted against the capacity, and where the `Terminated` message is placed. The "lane" is the internal FIFO next to the user queue that holds `Terminated`, `WatchWith` messages and Port posts (`pekko-actor/actor/watch.go`, `port.go`). This front covers ADR 0030 decisions D3 to D8 and rows E10 to E17. Capacity, overflow and zero timeout (D1, D2, E1 to E9) belong to front 1.

No production code was read as wrong and none was changed. We checked the existing tests against the ADR rows, found four rows or combinations that no test pinned, and added four tests for them (section "Tests added"). Everything else was already covered, so we changed nothing else.

## The ordering boundary of this front

**The placement of the `Terminated` when the death notification is processed (ADR 0030 D8, row E16).** A death notification is a system message. It is processed between two turns of the actor, never inside a handler. At that moment the `Terminated` (or the `WatchWith` message) goes to the tail of the mailbox as it is then, behind everything already there, including messages that arrived after the subject died. It takes a slot only then, or is a dead letter if a bounded mailbox is full. No position or slot is reserved when the notification arrives. Several notifications are processed in arrival order.

The existing tests pin this against messages told before processing (`bounded_mailbox_order_test.go`, capacity 3 and unbounded) and the admission against a full queue (`bounded_mailbox_sysmsg_test.go`). The new test pins the remaining part: with two notifications and room for one, the earlier notification gets the slot and the later one is the dead letter.

## What the Pekko source says (F-10 check)

The reader report `docs/parity/actor-typed-design-audit-2026-10-25/reader-reports/bhv-tim-mbx-dsp.md` finding F-10 disputes a sentence of the issue's second-pass note: "Pekko's capacity does not count 'admitted lane items'; D3/D8 (Terminated placement) are goPekko decisions, not Pekko semantics". We read the source ourselves; the issue text was not edited.

- `DeathWatch.scala:99-111`: `watchedActorTerminated`, when the watcher is not terminating (`:109`), runs `self.tell(Terminated(actor)(...), actor)` (`:110`), then `terminatedQueuedFor` (`:111`). `self.tell` is an ordinary send to the actor's own mailbox. For a bounded mailbox that is `BoundedQueueBasedMessageQueue.enqueue` (`Mailbox.scala:557-568`): a full queue means a `DeadLetter`.
- So in Pekko a `Terminated` is a message of the mailbox that counts against the capacity. The note's claim that Pekko does not count it is contradicted by the source. F-10 is right on this point.
- The statement stays partly true in a narrower form. Pekko has no "lane" and no Port. The lane is a goPekko mechanism, and so is counting Port posts (D6), though the principle behind it (an answer from outside is a message to the `ActorRef`, `SnapshotStore.scala:62,77-116`) is Pekko's. The placement at the processing and the capacity counting of the `Terminated` are Pekko behavior; the lane implementation is not.
- What the source does not make a property: the order between a message told by another goroutine and the processing of the notification when they race. ADR 0030 E16 says it is not guaranteed by Pekko and not reproduced.

## Evidence classes

Three classes, kept apart. **Current**: observed by us on the candidate SHA below. **Historical**: recorded in ADR 0030 or an earlier audit and not re-run. **Pending**: not executed; reason and who decides stated.

| Evidence | Class | Detail |
|---|---|---|
| The local test suite of `pekko-actor` and its gates | **Current** | Section "Executed checks", on tested SHA `52c5bf3e`. |
| The mapping below (rows to tests, Pekko lines) | **Current** (read by us) | Pekko lines were re-read in `upstream/pekko`; test names were read in the files. |
| Pekko JVM scenarios D-1a, D-1b, D-5 to D-11, E1, E2 | **Historical** | Run once on a Pekko build from the pin, outputs quoted in ADR 0030. Not re-run here (no JVM was asked for or run). |
| The independent review of candidate `c0e1e6f4` | **Historical** | A conventional review, not a native one: no native verdict exists (ADR 0030 "Independent review"). |
| `-race` runs, lint, coverage cited in ADR 0030 and `actor-local-fidelity-audit-2026-10-22.md` (revision 10) | **Historical** | Recorded for older heads. The audit's matrix was made at `590e86b7` and states that it was not repeated. |
| Remote CI of #686 at the accepted head (`go`, `label`) | **Pending, never ran** | The jobs did not start (account payments / spending limit). They have no verdict and are **not** green. Nothing here says CI passed. |
| Remote CI of the final SHA of this story | **Pending** | Not requested by this front; the Owner's economy policy (`docs/ci-economy.md`) applies. |
| Module-close campaigns: `go test -race`, mutation, parity judge, stress | **Pending, unexecuted** | Forbidden by default in this request (`-race`, workbench) or not asked for (mutation, judge, stress). The Owner decides when a module closes. |
| Pekko JVM runs for Port posts, the persistence module, `gopekko-testint` | **Pending** | ADR 0030 states Port has no JVM counterpart and persistence was not run on the JVM. |

### State of the linked issues today (`gh issue view`, read-only, 2026-10-27)

| Issue | State | Role here |
|---|---|---|
| #176 (ACT-030) | CLOSED, NOT_PLANNED | Legacy. Closure is supersession, not delivery of this story. |
| #177 (ACT-031) | CLOSED, NOT_PLANNED | Legacy, same. The issue body says "CLOSED"; the tracker reason is NOT_PLANNED. |
| #59 (ACT-E11) | CLOSED, NOT_PLANNED | Legacy. |
| #686 | MERGED | The implementation. |
| #690 / #695 | CLOSED, COMPLETED / MERGED | PostStop after a failed PreStart (E17). |
| #685 | OPEN | Blocking push timeout and flow control: decision needed. Out of this story (D7). |
| #689 | OPEN | Document the answers of `SnapshotResultReceiver` under the bound. Its decision (D6) and tests exist; its documentation obligation is still open. |
| #694 | OPEN | Journal and snapshot-load results are not mailbox messages. |
| #688 | CLOSED, NOT_PLANNED (closed as superseded; successors #749, #754, #763, all open) | Other Pekko bounded variants not migrated. |
| #693 | CLOSED, NOT_PLANNED | A `Terminated` pending when the watcher stops is discarded in Go while Pekko dead-letters it (`Mailbox.scala:340-352`). ADR 0030 listed it as a follow-up. The tracker reason is NOT_PLANNED, but the issue was closed as superseded and its successor #903 (ACT2-DTH-04, open) tracks the divergence, so it is a tracked open divergence, not an accepted one. We did not re-observe it. |
| #677, #678 | CLOSED, NOT_PLANNED (closed as superseded; successors #903 and #871 for #677, #907 and #940 for #678, all open) | Suppressed dead letters and `log-dead-letters`. |

ADR 0030 and the audit still name #688, #693, #677 and #678 as "followed in". The tracker state above is newer than that text. We record the difference and do not edit ADR text.

### Old obligations carried forward as pending

The legacy issues #176 and #177 also asked for a record of the migration-kit execution (models, roles, costs, permissions), a ported test, TCK or judge case, and the CI of the final SHA (ADR 0030 "Do #176 and #177 close?"). None of these exists in the repository for this policy. They are carried forward as **pending**:

1. Kit execution record of this migration: not produced. Decision on whether it is required: the Owner.
2. A ported Pekko test, TCK or judge case: not produced. Pekko's oracles for the zero-timeout policy are `MailboxConfigSpec` and `MailboxSelectorSpec` (front 1). For this front the Pekko test is `DeathWatchSpec.scala:268-291`, not ported as such; goPekko's `UnwatchBeforeTheNotificationIsProcessedTakesNoSlot` is a Go test of the same scenario.
3. CI of the final SHA: not claimed. Remote CI for #686 never ran.

## Mapping of ADR decisions and rows to tests

Kind column: **Pekko property** = behavior fixed by Pekko source (line) and, where one exists, a Pekko test or a recorded JVM run; **Go decision** = ADR decision or adaptation with no Pekko source; **Go-only test** = no Pekko oracle at all. Status: covered / partly / missing. All test names are `TestBoundedMailbox...` unless written otherwise; files are in `pekko-actor/actor/`.

| Row | Rule in short | Kind | Pekko source | Pekko test or run | Go test | Status |
|---|---|---|---|---|---|---|
| D3, E10 | A queued `Terminated` takes a slot; if the mailbox is full it is a dead letter and the watcher never sees it | Pekko property | `DeathWatch.scala:110`, `Mailbox.scala:557-568` | recorded run D-7 (ADR) | `TerminatedDroppedWhenFull`, `TerminatedTakesASlot` (`bounded_mailbox_terminated_test.go`) | covered |
| D3 (lane items together) | `Terminated` and Port post count against **one** capacity | Go decision (Port is not Pekko; the principle follows `SnapshotStore.scala:62,77-116`) | `Mailbox.scala:557-568` (all messages) | none | each alone: above and `PortPostsShareTheCapacityAndKeepOrder`; **together: new** `TerminatedAndPortPostsShareOneCapacity` (`bounded_mailbox_lane_test.go`) | covered (new test) |
| D3 (outside the capacity) | Stop, restart, suspend, resume and child-failure items take no slot (E9) | Pekko property (separate system queue, `Mailbox.scala:289-352`) | `Mailbox.scala:289-352` | none | front 1 (`bounded_mailbox_test.go`) | front 1 |
| E11 | `Unwatch` removes the entry; the queued `Terminated` keeps its slot until taken, then is ignored | Pekko property | `DeathWatch.scala:67-79,81-91` | `DeathWatchSpec.scala:268-291`; run D-1b | `UnwatchKeepsTheSlotAndSuppressesTheDelivery` (`bounded_mailbox_unwatch_test.go`) | covered |
| D4, E12 | After a drop the entry stays; a later `watchWith` delivers the **first** message | Pekko property (the entry is recorded "unless one exists", `DeathWatch.scala:117-119`); D4 chooses to keep it | `DeathWatch.scala:111,117-119` | recorded runs D-1a (first custom message), E1 (dropped plain `Terminated`, then `watchWith`: plain `Terminated`), E2 (unwatch then `watchWith`), historical | D-1a: `ReWatchAfterDropDeliversTheFirstMessage` (`bounded_mailbox_unwatch_test.go`). E1: new `WatchWithAfterADroppedPlainTerminatedDeliversThePlainTerminated`. E2: new `UnwatchThenWatchWithDeliversTheNewMessageOnce` (both in `bounded_mailbox_lane_test.go`; the existing `TestUnwatchThenWatchAgainDeliversOnceNotTwice` in `watch_api_test.go` is the unbounded plain-watch case, not the E2 scenario) | covered (D-1a existing; E1 and E2 new, Go side only; the Pekko side stays historical) |
| E13, D6 | A Port post shares the capacity; a rejected post returns false and publishes one `DeadLetter` | Go decision / adaptation | `SnapshotStore.scala:62,77-116` (principle only) | none (no JVM counterpart, ADR) | `PortPostRejectedWhenFull`, `PortPostsShareTheCapacityAndKeepOrder`, `PortPostWithoutOptionIsUnbounded` (`bounded_mailbox_port_test.go`); `TestSnapshotAnswersUnderBoundedMailbox` (persistence side, not this module) | covered |
| E14 | A second `Terminated` for the same subject holds a slot, finds no entry, is ignored: one notification, the slot is freed when taken | Pekko property | `DeathWatch.scala:81-91,117-119` | run D-6 | `SecondTerminatedHoldsASlotAndIsNotDelivered`, `WithoutTheSecondTerminatedBothAreAdmitted`, `SlotOfTheIgnoredTerminatedIsFreedWhenTaken`, `UnboundedActorSeesOneNotificationWithARewatch` (`bounded_mailbox_rewatch_test.go`) | covered |
| E15 | The notification is a system message: the `Terminated` is enqueued when the handler returns; an `unwatch` before that leaves nothing | Pekko property | `Mailbox.scala:263-289` (`processMailbox`, `processAllSystemMessages`), `DeathWatch.scala:99-111` | `DeathWatchSpec.scala:268-291`; runs D-7, D-8 | `TerminatedIsEnqueuedWhenTheHandlerReturns`, `UnwatchBeforeTheNotificationIsProcessedTakesNoSlot` (`bounded_mailbox_sysmsg_test.go`); `TestBoundedMailboxTerminatedDroppedWhenFull` also asserts no dead letter while the handler runs | covered |
| D8, E16 (tail) | At the processing the `Terminated` goes to the tail as the mailbox is then, bounded and unbounded; no reserved position | Pekko property (with D8 as the Go decision to change ADR 0010 and to apply it without a bound) | `DeathWatch.scala:110` | runs D-9, D-10 | `TerminatedGoesToTheTailWhenItIsProcessed`, `UnboundedTerminatedGoesToTheTailWhenItIsProcessed` (`bounded_mailbox_order_test.go`); `TestWatchWithMessageGoesToTheTailWhenItsNotificationIsProcessed`, `TestPendingRuntimeDeliveriesAreOrderedAgainstUserMessages` (`watch_internal_test.go`, not in this front's files) | covered |
| E16 (several notifications) | Processing order of the notifications is the order of the system queue; under a bound the earlier notification takes the last slot | Split: processing order is read from source (`Mailbox.scala:289-340`, FIFO system queue); the **arrival order** of two notifications from two subjects (the order of the `Watch` calls on already dead subjects) is a Go fact with no Pekko run | `Mailbox.scala:289-340`, `DeathWatch.scala:99-111` | none (D-10 two `WatchWith` is unbounded and tests ordering of placement) | unbounded: `TestTwoWatchWithDeliveriesGoToTheTailInNotificationOrder`; bounded: new `SeveralTerminatedAreProcessedInArrivalOrderAgainstTheCapacity` (with a control) | covered by a **Go-only pin**; no Pekko oracle |
| E17 | PreStart failure with a pending notification leaves nothing; a restarting bounded watcher is told once | Pekko property | `ActorCell.scala:663-666`, `FaultHandling.scala:247-254` | run D-11 | `TestInitFailureWithAPendingDeathNotificationLeavesNothingBehind`, `TestRestartWithAPendingDeathNotificationDeliversTheTerminatedOnce` (`bounded_mailbox_lifecycle_internal_test.go`) | covered |
| D5 | Differences P-1 to P-5 are pre-existing and out of this decision | Go decision (documentation only) | `ActorRef.scala:601-622,709-723` (P11), `Mailbox.scala:340-352` | none | none (no behavior to test) | not a test subject; tracker state in the table above |
| D6 (docs) | `SnapshotResultReceiver` documentation says answers obey the bound | Go decision | n/a | n/a | n/a | **open**: the comment (`snapshot_ops.go:78-91`) does not say it; #689 is OPEN (see Open points) |
| D7 | The blocking push timeout is outside this variant | Go decision | `Mailboxes.scala:257-267` (Pekko itself warns that a non-zero timeout blocks) | none | none by design (a negative case is front 1: zero timeout never blocks) | front 1 / #685 |
| D1 | `WithMailboxCapacity` additive, `Spawn` unchanged without it | Go decision | `Mailboxes.scala:231-235` | none | front 1; the changed `Terminated` placement without an option is `Unbounded...GoesToTheTail...` | front 1 |

A note on counted claims: the `Terminated` of a stopping watcher (a notification pending when the watcher stops) is **not** in this table as covered. Go discards it, Pekko dead-letters it (#693, closed as superseded; successor #903 is open). We did not re-observe that.

### One difference worth keeping in view

In Pekko `Terminated` is `DeadLetterSuppression` (`Actor.scala:124-128`), so a dropped `Terminated` is published as a *suppressed* dead letter (`ActorRef.scala:709-723`). goPekko publishes it as a plain `DeadLetter` (no suppression concept; #677 closed as superseded, successors #903 and #871 open). The tests of this front assert the Go behavior (a `DeadLetter` whose `Msg` is a `Terminated`). That is a recorded pre-existing difference (ADR 0030 P11), not parity.

## Tests added

| Test | File | What it asserts | Why it was missing |
|---|---|---|---|
| `TestBoundedMailboxSeveralTerminatedAreProcessedInArrivalOrderAgainstTheCapacity` (two cases) | `pekko-actor/actor/bounded_mailbox_lane_test.go` | Capacity 2, a handler that watches two dead subjects (`first`, then `second`) and parks; message 2 queued. Positive: log `[1 2 -1]`, i.e. `first`'s `Terminated` took the last slot. Negative: exactly one dead letter, a `Terminated` for `second`. Control, capacity 3: both delivered, `[1 2 -1 -2]`, no dead letters. | E16 says "several notifications in arrival order" but no bounded test fixed which of two competing notifications gets the slot. |
| `TestBoundedMailboxTerminatedAndPortPostsShareOneCapacity` | same | Capacity 2: a `Terminated` waiting in the lane and message 3 fill it, so a Port post returns `false` and is one `DeadLetter("p")`; the rejected answer is never delivered; after the `Terminated` is taken the slot is reusable and `Post("q")` is delivered after message 3. | D3 lists `Terminated` and Port posts against one capacity, but each had only been tested against user messages. |
| `TestBoundedMailboxWatchWithAfterADroppedPlainTerminatedDeliversThePlainTerminated` | same | E1: capacity 2, plain `Watch`, `Terminated` dropped on a full mailbox; a later `WatchWith(subject, 200)` on the dead subject delivers a plain `Terminated` (`[1 2 3 9 -1]`), not 200. | E1 was only a recorded JVM run; no bounded Go test covered a dropped plain `Terminated` followed by `watchWith`. |
| `TestBoundedMailboxUnwatchThenWatchWithDeliversTheNewMessageOnce` | same | E2: capacity 3, `WatchWith(…,100)`, `Terminated` queued; message 2 unwatches and registers `WatchWith(…,200)`; the queued `Terminated` is ignored, 200 is delivered once (`[1 2 200]`), no dead letters. | E2 was only a recorded JVM run; the existing unwatch-then-watch test is unbounded and uses a plain watch. |

All tests of this file use go-specs, no testify and no real resources, and wait only on channels: a `laneEvents` helper (a buffered channel read with a 30 s select, a failure limit and not a witness) replaces polling, so the file adds no `time.Sleep` loop and does not reuse `waitIntLog` or `waitPortLog`. Test and helper names are new (helpers carry the prefix `lane`); no existing file was edited. The Pekko side of E1 and E2 stays historical: we did not run the JVM.

No test was removed. The architecture review round of 2026-10-27 strengthened four of these tests (the two-notification test, the Port rejection test, E1 and E2) with a completion barrier, so that a surplus delivery fails them; their expectations are unchanged. No test was added for D5, D7 or the documentation part of D6 because there is no behavior of this front to assert.

## Executed checks

**Tested SHA:** `52c5bf3e21ab22107d2b867ea151a2d408ec313f` (branch `ao/mailbox-verify/dev2-terminated-adr0030`; test commits `c131606a`, `fe6b8b4f` (helper rename) and `52c5bf3e` (channel instead of polling, E1 and E2 tests)). Every Go command below ran on that commit. Any later commit on this branch is documentation only (this file). Go 1.27.2, 2026-10-27.

The prefix `-run TestBoundedMailbox` does not select all the evidence of this front: some tests lack that prefix (`TestUnbounded...`, the two E17 tests) and some live in `watch_internal_test.go`. So the tested set is listed by exact name. The 24 names are the tests this document cites (E10 to E17, D3, D8), run from `pekko-actor`:

```sh
go test -count=1 -v -run '^(TestBoundedMailboxTerminatedDroppedWhenFull|TestBoundedMailboxTerminatedTakesASlot|TestBoundedMailboxUnwatchKeepsTheSlotAndSuppressesTheDelivery|TestBoundedMailboxReWatchAfterDropDeliversTheFirstMessage|TestBoundedMailboxPortPostRejectedWhenFull|TestBoundedMailboxPortPostsShareTheCapacityAndKeepOrder|TestBoundedMailboxPortPostWithoutOptionIsUnbounded|TestBoundedMailboxSecondTerminatedHoldsASlotAndIsNotDelivered|TestBoundedMailboxWithoutTheSecondTerminatedBothAreAdmitted|TestBoundedMailboxSlotOfTheIgnoredTerminatedIsFreedWhenTaken|TestBoundedMailboxUnboundedActorSeesOneNotificationWithARewatch|TestBoundedMailboxTerminatedIsEnqueuedWhenTheHandlerReturns|TestBoundedMailboxUnwatchBeforeTheNotificationIsProcessedTakesNoSlot|TestBoundedMailboxTerminatedGoesToTheTailWhenItIsProcessed|TestUnboundedTerminatedGoesToTheTailWhenItIsProcessed|TestInitFailureWithAPendingDeathNotificationLeavesNothingBehind|TestRestartWithAPendingDeathNotificationDeliversTheTerminatedOnce|TestBoundedMailboxSeveralTerminatedAreProcessedInArrivalOrderAgainstTheCapacity|TestBoundedMailboxTerminatedAndPortPostsShareOneCapacity|TestWatchWithMessageGoesToTheTailWhenItsNotificationIsProcessed|TestTwoWatchWithDeliveriesGoToTheTailInNotificationOrder|TestPendingRuntimeDeliveriesAreOrderedAgainstUserMessages|TestBoundedMailboxWatchWithAfterADroppedPlainTerminatedDeliversThePlainTerminated|TestBoundedMailboxUnwatchThenWatchWithDeliversTheNewMessageOnce)$' ./actor/
```

New package-level helpers of the added tests carry the prefix `lane` (`laneDeadSubject`, `laneEvents`, `laneTwoSubjectWatcher`, `laneRunTwoNotifications`, `laneWatcher`, `laneRewatcher`, `laneNewRewatcher`, `laneSubject`) so they cannot collide with other fronts. A scratch copy of this tree plus Dev 1's `bounded_mailbox_zero_timeout_test.go` (branch `ao/mailbox-verify/dev1-capacity-overflow`, `cd6bbb8`) passed `go vet ./actor/`; that copy was not committed.

| Command (in `pekko-actor` unless stated) | Result on `52c5bf3e` |
|---|---|
| `gofmt -l .` | empty |
| `go vet ./...` | no output (ok) |
| `golangci-lint run --config ../.golangci.yml ./...` (v2.14.0, pinned, binary kept outside the worktree; includes `gocyclo` at 10) | `0 issues.` |
| the exact-name `-run` above, `-v` | 24 `--- PASS`, 0 `--- FAIL` (one per name) |
| `go test -count=1 ./...` | `ok` for `actor` (1.010s), `internal/deadletter`, `internal/mailbox` |
| in `pekko-persistence`: `go test -count=1 -v -run TestSnapshotAnswersUnderBoundedMailbox ./persistent/` (E13, plain, no `-race`) | `--- PASS`, `ok .../persistent` |

These are local results for the Go side only. The Pekko properties named in the tables were read from `upstream/pekko` source and from the ADR's recorded JVM output; no JVM or Pekko test was executed by us.

Not executed by this front, with the reason: coverage threshold (85%) and `go mod tidy -diff` (the TL runs them at integration); `go test -race` (forbidden unless the request asks for it; it did not); mutation, parity judge, stress/loop runs and workbench (module-close campaigns, not asked for); the Pekko JVM reproducer `docs/adr/0030-pekko-repro/BoundedTerminatedRepro.java` (no JVM requested); remote CI (never ran, not claimed). A check that the new tests fail when the behavior is wrong (mutation of production code) was not run: it is a mutation campaign.

## Open points for the Owner (stated, not resolved)

*These points were stated before the Owner decisions of 2026-10-27. Where [`README.md`](README.md) records a decision they are answered there (see "Open points of the two fronts, and where each stands"); the others remain open as written.*

1. **Pending obligations of #176 and #177.** The kit execution record, a ported test/TCK/judge case and the CI of the final SHA do not exist. Does the Owner accept them as permanently pending, require them, or retire them with the supersession by #791? We do not decide.
2. **#689 is still OPEN and its documentation obligation is unmet today.** ADR 0030 says its consequences list asks that the documentation of `SnapshotResultReceiver` say answers are subject to the bound. The doc comment at `pekko-persistence/persistent/snapshot_ops.go:78-91` says answers are delivered in the actor's turn and are dead letters after a Stop, but it does not say that they are subject to the bounded mailbox. Changing that comment is a production-code edit, out of this story, so we did not edit it. Whether it blocks closing #791 is the Owner's call.
3. **ADR 0030 and the audit list #688, #693, #677 and #678 as follow-ups; the tracker shows them closed NOT_PLANNED, as superseded, with open successors (#749, #754, #763; #903; #903, #871; #907, #940).** Whether the ADR text should be updated to say that, and who does it (ADRs are TL/Owner files), is the Owner's call.
4. **Two sentences in the issue's second-pass note** ("Pekko's capacity does not count admitted lane items" and "D3/D8 are goPekko decisions, not Pekko semantics") are contradicted by `DeathWatch.scala:110` for the capacity counting of `Terminated` and its placement. Whether to correct the issue text is the Owner's or TL's decision; we did not edit it.
5. **One recorded decision still carries a Go-only element:** counting Port posts against the capacity (D6) has no Pekko oracle by construction. The Owner accepted it with ADR 0030; nothing here changes that.
