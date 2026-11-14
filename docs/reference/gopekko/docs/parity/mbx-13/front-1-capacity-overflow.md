# MBX-13, front 1: capacity and overflow with a zero push timeout

Story: [#791](https://github.com/getsyntegrity/gopekko/issues/791). Squad 01, Dev 1. Contract: [ADR 0030](../../adr/0030-bounded-mailbox.md) D1, D2 and rows E1 to E9. Pekko baseline: v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, `upstream/pekko`).

## What this front covers, in plain words

An actor spawned with `WithMailboxCapacity(n)` keeps at most `n` waiting messages. The message the actor is currently processing is not waiting, so it does not count (ADR 0030 E3). When `n` messages wait, the next `Tell` is not queued: it is published as one `DeadLetter` addressed to the receiver, and `Tell` returns at once. Nothing tells the sender (no error, no acknowledgement). That is the behavior of Pekko's classic `BoundedMailbox(capacity, Duration.Zero)`; Typed's `MailboxSelector.bounded(n)` builds that same mailbox.

This front does not cover where a `Terminated` goes in the queue, whether Port posts count against capacity, or the ordering of death notifications. Those belong to front 2 (`front-2-terminated-and-evidence.md`). The blocking variants (positive or negative push timeout) are out of the story (ADR 0030 D7; #685, #790, #830).

This work verified an existing implementation. No production code was changed.

## Evidence classes used below

- **current**: observed by us, re-run on the candidate SHA named in the last section.
- **historical**: recorded in ADR 0030 or an earlier audit, not re-run here.
- **pending**: not executed; the reason and the decider are named.

Property labels: **Pekko property** (source line plus Pekko test), **Go decision** (an ADR decision, an adaptation of Pekko), **Go-only** (a Go test with no Pekko oracle).

## Pekko cases and whether they apply

| Pekko case | Source | Applies to Go? | Why |
|---|---|---|---|
| "create a bounded mailbox with 10 capacity and with push timeout": 10 enqueued, the 11th is a `DeadLetter` for the receiver | `MailboxConfigSpec.scala:56-75` | Partly | The capacity and dead-letter half applies. The 10 ms push timeout is a blocking variant, out of scope (D7). Pekko's check goes through `numberOfMessages`/`dequeue` on the raw queue; Go exposes no queue, so the same state is observed through the actor (one message parked in `Receive`, the queue empty). |
| "dequeue what was enqueued properly for bounded mailboxes with 0 pushTimeout" (`BoundedMailbox(10, 0ms)`, 20 enqueued, 10 taken, exactly `enqueueN - dequeueN` dead letters) | `MailboxConfigSpec.scala:85-87`, helper `testEnqueueDequeue:142-192` | Yes | Zero timeout is the adopted policy (D2). Go analogue: offer twice the capacity, expect exactly `capacity` dead letters and each admitted message delivered once. |
| same helper with `BoundedMailbox(10000, -1ms)` and with positive timeouts | `MailboxConfigSpec.scala:81-83, 89-91` | No | Negative timeout is `queue.put` (blocks forever); positive timeout blocks up to the timeout. Both are the blocking variant, out of scope (D7). |
| "set capacity on a bounded mailbox" (Typed `bounded(2)`: `one` parks, `two`, `three` fit, `four` is a dead letter) | `MailboxSelectorSpec.scala:75-95` | Yes | Typed `bounded(n)` is the zero-timeout classic mailbox (`PropsAdapter.scala:43-45`, `Mailboxes.scala:230-235`). Existing Go test `OverflowToDeadLetters` is the same shape. |
| "create a bounded mailbox ... capacity 0" | no Pekko test | n/a | Capacity 0 is refused by the JDK `LinkedBlockingQueue`, not by a Pekko rule (ADR 0030 P4). Go refuses `n < 1` with `ErrInvalidMailboxCapacity`: **Go decision**, adaptation. |

## What was checked: inventory of existing evidence

Files: `pekko-actor/actor/bounded_mailbox_test.go` and `bounded_mailbox_restart_test.go` (existing, unchanged), and the new `bounded_mailbox_zero_timeout_test.go`. Status `covered`, `partly` or `missing` is stated against the row before the new tests existed.

| ADR row | Claim | Existing Go test | Pekko line / test | Kind | Status before | Class |
|---|---|---|---|---|---|---|
| D1, E1 | `WithMailboxCapacity(n)` is additive; without it the mailbox is unbounded | `TestBoundedMailboxUnboundedByDefault` (5000 `Tell`, zero dead letters) | `Mailboxes.scala:231-235` | Go decision (option shape) over a Pekko property (unbounded default) | covered | current |
| D1 | capacity below 1 is refused, no name held | `TestBoundedMailboxRefusesInvalidCapacity` (0, -1; then the same name spawns) | `Mailbox.scala:713` checks `< 0` only; zero fails in the JDK queue (P4) | Go decision (adaptation) | covered | current |
| D2, E2 | full: the new message is a `DeadLetter` for the receiver; `Tell` does not block | `TestBoundedMailboxOverflowToDeadLetters` (receiver id checked), `TestBoundedMailboxTellNeverBlocks` | `Mailbox.scala:557-568`; `MailboxSelectorSpec.scala:75-95` | Pekko property | covered | current |
| E3 | the message being processed is not counted | same two tests (capacity 2 admits 2 and 3 while 1 is being processed) | `MailboxSelectorSpec.scala:75-95` | Pekko property | covered | current |
| E4 | capacity 2: later messages dead-lettered in order | `OverflowToDeadLetters` (`[4,5]`) | `MailboxSelectorSpec.scala:75-95` | Pekko property | covered | current |
| E4 | the **exact** boundary: capacity messages give no dead letter, the capacity+1th gives exactly one | none; only capacity 2 with four offers | `MailboxConfigSpec.scala:56-75` (10 fit, 11th is a dead letter) | Pekko property | **partly** | current (new test) |
| E4 / Pekko `testEnqueueDequeue` | offered twice the capacity: exactly `capacity` dead letters; no admitted message delivered twice or lost; no overflow message delivered | none (existing tests never read what the actor received in `OverflowToDeadLetters`) | `MailboxConfigSpec.scala:85-87, 142-192` | Pekko property | **missing** | current (new test) |
| E4 | FIFO delivery of the admitted messages | only indirectly in `SurvivesRestart` and `StopDeadLettersInFIFO` | none directly in the Pekko specs above | Go-only assertion of Pekko's queue FIFO | **partly** | current (new test) |
| E4 | capacity 1 and capacity 10 (a different capacity from the single existing 2, 3, 8) | `TellNeverBlocks` (1, no assertion on counts), `ConcurrentProducers` (8) | `MailboxConfigSpec.scala:56-75` uses 10 | Pekko property | **partly** | current (new test) |
| E5 | concurrent producers: exactly `capacity` admitted, per-producer order | `TestBoundedMailboxConcurrentProducers` (8 slots, 16 producers x 50) | `Mailbox.scala:705-726` uses a thread-safe queue; `testEnqueueDequeue(BoundedMailbox(10, 0ms), 20, 10, false)` uses one producer, awaited before the consumers start; no Pekko case saturates a zero-timeout mailbox concurrently | Capacity and accounting: **Pekko property shared with goPekko** (`BoundedMailbox` is a `LinkedBlockingQueue(capacity)` with a zero-timeout `offer`, `Mailbox.scala:557-568, 720-725`): an oracle applies, no existing upstream test saturates it concurrently, and the differential scenario is **pending** (ADR 0033 D13). Admission under the cell mutex is a Go adaptation (E5). *Corrected on 2026-10-27; this row said Go-only before.* | covered, with a gap: it counts dead letters and the union but not that each message appears exactly once, and does not check the order of the dead letters | current |
| E5 | each message accounted for exactly once, delivered or dead-lettered, never both; per-producer order also in the dead letters | none | `MailboxConfigSpec.scala:142-192` (`distinct`, `diff` checks) | Pekko property for "each message once"; the order within one producer's dead letters is "checked in Go and supported by a reading of the Pekko code for a single local subscriber; no differential execution" (Go publishes the overflow on the sender's goroutine, ADR 0030 section 4 "Go adaptations"; Pekko publishes on the producer's thread too, `EventStream.scala:51-54`; no Pekko spec asserts it). *Corrected on 2026-10-27, refined on 2026-10-28* | **missing** | current (new test) |
| E6 | `Ask` against a full mailbox: request dead-lettered, ends with `ErrAskTimeout` or `ErrAskCanceled`, does not block | `TestBoundedMailboxAskAgainstFullMailbox` | `AskTimeoutException` (Pekko ask semantics) | Pekko property for the timeout; Go decision for the cancel branch (ADR 0015) | covered | current |
| E7 | `Stop` on a full mailbox: the queue is dead-lettered in FIFO order after the overflow already published | `TestBoundedMailboxStopDeadLettersInFIFO` (`[5,2,3,4]`) | `Mailbox.scala:289-352` `cleanUp` | Pekko property | covered | current |
| E8 | restart keeps capacity and pending messages in order; the failed message is not redelivered | `TestBoundedMailboxSurvivesRestart` | `FaultHandling.scala` `faultRecreate` (P10) | Pekko property | covered | current |
| E9 | system messages use a separate queue and are not refused | no front-1 assertion; `SurvivesRestart` and `StopDeadLettersInFIFO` show that restart and stop succeed on a full mailbox | `Mailbox.scala:289-352` | Pekko property, shown by structure | covered by structure; direct tests belong to front 2 (`bounded_mailbox_sysmsg_test.go`) | current |
| none | recovery: capacity is freed, later messages are admitted again | `TestBoundedMailboxRecoversWhenCapacityIsFreed` | implied by `LinkedBlockingQueue` | Go-only | covered | current |
| none | no goroutine left after stop | `TestBoundedMailboxNoGoroutineLeft` | none | Go-only | covered | current |

One limit on the Pekko comparison: Pekko's `DeadLetter` carries `(message, sender, receiver)`; goPekko's carries `Msg` and `To` only (ADR 0030 P-1, an accepted adaptation of ADR 0010 D2). The test checks `To` and `Msg`, not a sender.

## Tests added

One new file, `pekko-actor/actor/bounded_mailbox_zero_timeout_test.go`, two tests, 4 specs, no existing test edited, no production file touched:

1. `TestBoundedMailboxZeroTimeoutBoundary`, run for capacity 1, 2 and 10. Parks the first message in `Receive`, then:
   - offers exactly `capacity` more messages and asserts there are **no** dead letters (positive case, boundary);
   - offers one more and asserts exactly that message is the only dead letter (negative case, capacity+1);
   - offers until twice the capacity has been offered (the `20 enqueued / 10 taken` shape of `MailboxConfigSpec.scala:85-87`) and asserts exactly `capacity` dead letters, in `Tell` order, all addressed to the receiver;
   - releases the actor and asserts it receives the parked message and the admitted ones, in FIFO order, and nothing else; after `Stop` it asserts nothing more was delivered and the dead letters did not change (no overflow message was also delivered).
2. `TestBoundedMailboxConcurrentAdmissionIsExact`: 16 producers released together by a barrier each send 50 messages to an actor with capacity 8. Asserts that 792 were dead-lettered, that exactly 8 (plus the parked one) were delivered (nothing more arrives after `Stop`), that each of the 800 messages is accounted for exactly once, and that within each producer the order is preserved both among the delivered and among the dead-lettered messages. It uses the parameters of the existing `TestBoundedMailboxConcurrentProducers` on purpose: it supersedes that test's weaker assertions, and the old test stays untouched.

Determinism: no `time.Sleep`, no wall-clock assertion; synchronization is by channels (the parked actor, the start barrier) and a 30 s timeout that only fires on failure, as in the existing tests.

## The race boundary of this front

**Concurrent producers are admitted up to capacity, and each producer's messages keep their order.** Admission happens under the cell mutex, so however the producers interleave, the number admitted is exactly `capacity` (never more, never fewer, while the actor is parked), and two messages from the same producer are never reordered. What is deliberately not asserted: the order *between* different producers (it depends on scheduling), and which producer's messages are the ones admitted. No **existing** Pekko test covers this boundary: the only saturated zero-timeout case, `MailboxConfigSpec.scala:85-87`, has one producer awaited before the consumers start (`parallel = false`, `:142-192`), and the 20-producer concurrent runs are the unsaturated default cases (capacity 10000). That is not the absence of an oracle: Pekko's `BoundedMailbox` is a `LinkedBlockingQueue(capacity)` with a zero-timeout `offer` (`Mailbox.scala:557-568, 720-725`), so an applicable oracle exists and a differential scenario can be written and run on it. Capacity and accounting under concurrent producers with a parked consumer is therefore a **Pekko property shared with goPekko**, with the differential scenario **pending** (ADR 0033 D13; not written or run here). The per-producer order of the dead letters is "checked in Go and supported by a reading of the Pekko code for a single local subscriber; no differential execution" (no Pekko spec asserts it; that is not a semantic difference and not a guarantee exclusive to Go). Under ADR 0033 D13 the differential scenario is pending; no validation level is assigned here and nothing is declared equivalent. *This paragraph was corrected on 2026-10-27 (README, "Architecture review correction"; it said before that there was no Pekko oracle and that E5 was Go-only) and refined on 2026-10-28 (the order of the dead letters).* The exact count when the consumer is parked follows from the admission rule of D2/E5.

## Executed and unexecuted checks

Candidate: branch `ao/mailbox-verify/dev1-capacity-overflow`, Go code at `a3738b5d6f467c8c35a9f2f6501de72916fcc938` (first delivery `b55d767`, review follow-up `a3738b5`; base `80b947bb396678d2de88890b07ee9070255f3fe0`). The gate results below (gofmt, vet, lint, `go test -count=1 ./...`, coverage) were run on `a3738b5`; the Go code is unchanged in every later commit, which touch only this file. The `-count=20` row is the one exception, see its label. Any later commit touches this file only. Run in `pekko-actor`:

| Command | Result | Class |
|---|---|---|
| `gofmt -l .` | empty | current |
| `go vet ./...` | no output | current |
| `golangci-lint run ./...`, pinned v2.14.0 (`.golangci-version`), built locally with the repo toolchain because the one in `~/go/bin` was built with go1.26 and refused the repo's go 1.27.2 | `0 issues.` (the repo configuration includes the `gocyclo` limit) | current |
| `go test -count=1 ./...` | `ok` for `actor`, `internal/deadletter`, `internal/mailbox` | current |
| `go test -count=20 -run 'TestBoundedMailboxZeroTimeout\|TestBoundedMailboxConcurrentAdmission' ./actor/` on `b55d767`: `ok` (a repeat to look for flakiness, not a stress campaign) | author-reported, outside the brief's gates, not re-run by QA or TL, not counted as evidence | author-reported |
| `go test -count=1 -coverprofile ./...`, total, on `a3738b5` | 98.6% statements (the gate is 85% per module; the TL reruns it at integration) | current |

Unexecuted, and why:

- `go test -race`: forbidden by the Owner's standing rule unless the request asks for it; this one did not. The concurrent test is therefore **not** race-checked locally by us. ADR 0030 records `-race` runs of the older tests on #686 (**historical**).
- Mutation campaign, the parity judge, stress runs, workbench: module-close campaigns (`docs/quality-gates.md`) and forbidden or excluded by the brief. No claim is made that the new tests kill any mutant.
- The Pekko JVM reproducer: not requested. No Pekko scenario was run by us; every "Pekko property" above is read from source and the Pekko tests' text, not executed. ADR 0030's D-1a to D-11 runs are **historical**. None of D-1a to D-11 concerns the plain-capacity overflow of this front.
- `go mod tidy -diff`, the module coverage gate and the combined candidate: the TL runs them at integration.
- Remote CI: it did not run for ADR 0030 (spending limit) and has no verdict; it is not evidence here and nothing in this document says CI passed.

## Open points for the Owner (stated, not resolved)

*These points were stated before the Owner decisions of 2026-10-27. Where [`README.md`](README.md) records a decision they are answered there (see "Open points of the two fronts, and where each stands"); the others remain open as written.*

1. **Whether "the exact boundary" test claim is enough as "Pekko parity" for E4.** Pekko's tests observe the raw queue (`numberOfMessages`, `dequeue`); goPekko's tests observe the actor. They agree on the numbers (capacity admitted, the next one a dead letter), but no executed Pekko scenario of ours compares the two on this exact sequence. The Owner decides if a JVM scenario for it is wanted (ADR 0033 D13 validation level) or if the Pekko source plus its two specs suffice.
2. **Per-producer order among dead letters** is asserted as "checked in Go and supported by a reading of the Pekko code for a single local subscriber; no differential execution" (the overflow is published on the sender's goroutine). Pekko's `deadLetters.tell` publishes synchronously to the event stream on the producer's thread (`ActorRef.scala:736-743`) and the stream tells each subscriber with `subscriber ! event` (`EventStream.scala:51-54`), so the order at one subscriber follows by construction from program order and the subscriber's FIFO mailbox (read, not executed); no Pekko test or documentation states it. The Owner decides if that assertion should stay as a pinned test.
3. **Four older tests that poll with `time.Sleep`** (ADR 0030 finding F2) are untouched and fall in front 2's files; mentioned only so the count is not read as zero.
