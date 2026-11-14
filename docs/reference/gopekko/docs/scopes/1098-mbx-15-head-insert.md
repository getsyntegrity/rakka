# Scope: Head insert in the mailbox storage (#1098, MBX-15, partial delivery)

Status: **G2 approved by the Owner on 2026-10-28 for the partial delivery below, and only for it** (real user message relayed by the coordinator;
decisions D-1 to D-4 record it). Written against `develop` `80695c9`. **Delivering this scope does not authorize closing #1098 and does not mark
as met any criterion that cannot be observed through a public surface.** #1098 keeps `status:blocked`; this scope edits no issue and no label.
**Kit run required (ADR 0017, ADR 0024 T6, `AGENTS.md`), no waiver for size (D-4).** The kit package `migration/stage-1/mbx-15-head-insert/`
(manifest, RULEBOOK, RUNBOOK) is prepared with this scope and must be approved before the Dev implements; the Dev writes the execution record.

## Owner authorization (G2), recorded

Approved: an internal `Queue.PushFront`; **no export**; **no cell wiring**; tests of the queue's own order, limits and accounting.
The consumer reverses a sequence when it needs FIFO reinsertion; the primitive does not take that responsibility. The observable reject or
dead-letter policy on overflow is deferred to the first consumer that wires the primitive; the queue does not publish dead letters and this
scope attributes none to it. Pekko pin verified before consuming earlier readings: the `upstream/pekko` gitlink of this branch and the HEAD of the
checkout that was read are both `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.

## What this delivers

```go
// pekko-actor/internal/mailbox/queue.go (shape of the contract, not the implementation)
// PushFront puts msg before every pending message: it is the next one Pop returns.
func (q *Queue[T]) PushFront(msg T)
```

Ordering contract, fixed from Pekko and not assumed: `unstashAll` iterates `theStash.reverseIterator` and calls `enqueueFirst` for each
envelope (`Stash.scala:245-250`; `enqueueFirst` at `:271-277`); the unbounded deque's `enqueueFirst` is `addFirst` (`Mailbox.scala:596`).
So head inserts are **LIFO**: `PushFront(a)` then `PushFront(b)` pops `b, a`. A sequence `s1, s2, s3` is replayed in order only because Pekko inserts
it in reverse (`s3, s2, s1`); inserting it as `s1, s2, s3` yields `s3, s2, s1`. The reversal is the consumer's; the queue only guarantees LIFO for
head inserts and FIFO for the messages already queued.

## What this does not deliver

- No exported symbol, no change under `pekko-actor/actor/` (so no `cell.go`, `context.go`, `ref.go`, `mailbox.go`), no README change (no public fact changes: nothing can
  head-insert an actor's mailbox).
- No reversal of a sequence, no stash (#772, #855, #898, #899, #935, #1025), no bounded deque discipline (#864, #832), no head-inserted `Terminated` (#1051, ADR 0030 D8).
- **No overflow, reject or dead-letter behavior.** The queue is unbounded and does not know the capacity, which `Tell` enforces through `occupiedLocked`.

## Dependencies, as they really are

Verified on 2026-10-28 with `gh issue view` and by reading `develop` `80695c9`.

| Dependency | State today | Effect |
|---|---|---|
| #1021 (GOV-03) | open, `prio:P1`, `status:ready` | Not needed: no export is added. Whether #1098 stays `status:blocked` is the Owner's |
| ADR 0030 (#686) | integrated: `Tell` checks `c.capacity` against `occupiedLocked`, which reads `queue.Len()` (`ref.go:60`, `mailbox.go:27`) | A message in the queue is counted with no change |
| #688 | closed ("variants not migrated by #686"); no bounded deque mailbox exists | historical; live tracker is #864 (open) |
| #772, #855, #898, #899, #935, #1025, #1051 | open | consumers, later than this story |

## Criteria → scenario

Scenarios are in `pekko-actor/internal/mailbox/queue_test.go`, go-specs, no goroutine, no sleep, no polling, no clock, no `-race`.

| # | Criterion | Scenario |
|---|---|---|
| AC-1 | `m1, m2` pending, `PushFront(m0)`: pops are `m0, m1, m2` | Q1 |
| AC-2 | `m1` pending, `PushFront(a)` then `PushFront(b)`: pops are `b, a, m1` | Q2 |
| AC-3 | The primitive does not reverse: `PushFront(s1), (s2), (s3)` before `q1, q2` pops `s3, s2, s1, q1, q2`; a caller that inserts `s3, s2, s1` gets `s1, s2, s3, q1, q2` | Q3 |
| AC-4 | Empty queue and zero value: the message pops alone; `Len` is 1, then 0 | Q4 |
| AC-5 | Wrap-around: with `head` moved by pops, a front insert goes to the slot before `head` (index below zero wraps) and order is kept | Q5 |
| AC-6 | Limits: a front insert into a full buffer grows it as `Push` does (`Cap` doubles, order kept, front-inserted first); `Cap` follows pending depth, not history, after repeated insert and pop | Q6 |
| AC-7 | Accounting: `Len` counts every front-inserted message after each operation | in Q1, Q4, Q6 |
| AC-8 | `TakePending` after front inserts returns head-first and leaves the queue empty with no storage | Q7 |
| AC-9 | A consumed slot is cleared: no reference to a front-inserted message survives `Pop` (same style as `TestQueueConsumedSlots`) | Q8 |
| AC-10 | `Push`, `Pop`, `TakePending`, `Len`, `Cap` unchanged: the existing queue tests and `bounded_mailbox_*` pass unchanged; Q9 interleaves `Push`, `PushFront`, `Pop` against a reference slice | Q9 |
| AC-11 | No export added; the PR says so | `scripts/check_deps.sh`, review |

Each scenario must fail under a wrong implementation: tail insert fails Q1/Q2; FIFO-after-previous-front fails Q2/Q3; a non-wrapping index fails Q5;
a grow that ignores `head` fails Q6. The Dev records, per scenario, the mutation that made it red.

## Issue #1098 criteria: what this delivers and the proposed distribution of the rest

Proposal only; no issue is edited. The Owner decides each destination.

| Issue criterion | After this scope | Remaining, proposed destination |
|---|---|---|
| AC-R1-1 receive order `m0, m1, m2` | **Queue-level evidence only** (Q1). Not met publicly | Public-API ordering test: **#898** (BHV-06, first unstash that wires the primitive; if #855 wires it first, #855) |
| AC-R2-1 `b, a, m1` | Queue-level evidence only (Q2) | Same consumer as above, with the reversal for FIFO replay (#898) |
| AC-R3-1 full bounded mailbox: dead-lettered once, queued messages unchanged, via `SubscribeDeadLetters` | **Nothing.** Deferred by the Owner | The first consumer that wires the primitive (#855 or #898), together with the overflow rule: Pekko's zero-timeout rule (`Mailbox.scala:618-626`) is the reference. Bounded stash capacity: #899 |
| AC-R4-1 existing tests pass | Met at queue level (AC-10) | none |
| AC-R5-1 ADR 0011 justification or "no export" | Met by AC-11 for this PR | The consumer states its own |
| R-lane and counters (hand-off below) | Not delivered | The consumer's cell wiring |

## Hand-off to the consumers (recorded so it is not lost)

- `laneItem.after` is `pushed` when a `Terminated` was admitted and is released when `received >= after` (`watch.go:471`). A head-inserted message pops before
  every pending `Terminated`, so the cell-level insert must raise `after` of each waiting lane item and `pushed` by one, or a `Terminated` arrives one message early.
- `maxDepth` must be updated at the insert as `pushLocked` does (`mailbox.go:33`); `Tell` dead-letters when `stopRequested || closed` (`ref.go:54`) and the head insert needs the same guard.
- At queue level a stop takes whatever is pending with `TakePending` (`stop.go:114`), head-inserted messages first (Q7).

## Allowed files

Exactly these paths. Anything else needs a stop and a report to the TL.

| Path | Edit |
|---|---|
| `docs/scopes/1098-mbx-15-head-insert.md` | this file |
| `migration/stage-1/mbx-15-head-insert/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | kit package, prepared by the TL with this scope |
| `migration/stage-1/mbx-15-head-insert/EXECUTION-RECORD.md`, `migration/cost-log.tsv` (one row) | Dev, after the gates; unknown metrics stay `unknown` |
| `pekko-actor/internal/mailbox/queue.go` | add `PushFront`; keep the single `Ported from` line; reword the type comment only where it says FIFO without the exception |
| `pekko-actor/internal/mailbox/queue_test.go` | Q1 to Q9 |

Independence: squad A (`clock.go`, `ask.go`, `pekko-actor/actor/system*`, README) and squad B (`cell.go`, `context.go`, `ref.go`, README) share no path
with this scope, which does not touch `pekko-actor/actor/`. Later MBX stories share `queue.go` and run after this one.

## ADR impact (ADR 0033 D5), recorded before implementation; no ADR is amended

1. ADR 0030 D2, D3: enforced in `Tell` via `queue.Len()`; `PushFront` changes `size`, so the count stays consistent. The overflow rule of a head insert is deferred (D-2).
2. ADR 0030 D8: the `Terminated` goes to the tail; its head insertion is #1051.
3. ADR 0011, ADR 0033 D3: no export, no individual justification due. An export becoming necessary is a stop.
4. ADR 0010 (proposed) lists stash as excluded and ADR 0033 D4 makes it mandatory; irrelevant to a storage primitive.
5. The issue speaks of "cell mailbox storage" and of counters and depth; those are cell state (`pushed`, `received`, `maxDepth`, `laneItem.after`) and stay out of this scope.

## Evidence and validation

Pekko is read, not run: Go unit evidence derived from source, **not V3**. No upstream spec asserts queue-level ordering, so the ordering contract
stays "pending approval" for any parity claim (ADR 0033 D13). Local gates (commands in `AGENTS.md`, "Go"): unit tests of `pekko-actor` with network
isolation, `go vet`, `gofmt -l`, `golangci-lint` with `gocyclo` at most 10, `go mod tidy -diff`, module coverage at least 85%, diff coverage at least 85%,
`scripts/check_deps.sh`, `.github/scripts/check-headers.sh`. Run once per final tree; remote CI is not used.

## Delivery, size and tasks

Draft PRs, `Refs #1098` (never `Closes`), Conventional Commits, no merge without the Owner. Forecast: `queue.go` about 20 lines, `queue_test.go` about 120,
execution record about 60, under the 400 authored-line budget; over 400 is a stop.

| Task | Result | Check |
|---|---|---|
| T1 | Kit package approved (manifest, RULEBOOK, RUNBOOK) | before any code |
| T2 | Q1 to Q9 red against a stub or wrong implementation, then `PushFront` green | RED and mutation per scenario |
| T3 | Gates and execution record bound to the final SHA | each gate: SHA, command, exit code |

## Decisions (Owner, 2026-10-28)

- **D-1, no export.** Typed is not given the primitive. #1021 is not a prerequisite.
- **D-2, overflow deferred** to the first consumer that wires the primitive; the queue publishes no dead letters.
- **D-3, partial delivery.** Does not close #1098 nor mark publicly-unobservable criteria as met; distribution proposed above.
- **D-4, kit run mandatory, no size waiver**, applied proportionally and before implementation.
- **D-5, reversal is the consumer's.**

## Definition of Done

- Q1 to Q9 pass at the final SHA, each seen red under a wrong implementation; existing mailbox tests pass unchanged.
- Gates green with evidence bound to that SHA; complexity at most 10; no export; `pekko-actor/actor/` and READMEs unchanged.
- The execution record states what was read, not run, and leaves unmeasured metrics as `unknown`.
- A new QA session issues a verdict on the exact head of the PR. #1098 is not closed by it.
