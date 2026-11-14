# RULEBOOK: Head insert in the mailbox storage (unit mbx-15-head-insert, #1098 MBX-15, partial delivery)

Status: **prepared; the Owner approved G2 for this partial delivery on 2026-10-28.** Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after approval. The scope `docs/scopes/1098-mbx-15-head-insert.md` is the specification (criteria, scenarios Q1 to Q9, allowed files, decisions); this file does not repeat it.

## 1. Unit

- Capability: `mailbox.Queue[T].PushFront(msg)` in `pekko-actor/internal/mailbox/queue.go`; the message is the next one `Pop` returns. Head inserts are LIFO; messages already queued keep FIFO behind them.
- Source: Pekko `Mailbox.scala:576-598` and `Stash.scala:245-250,271-277` at v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), read, not run. Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`.
- Manifest: `manifest.tsv` follows the kit runner contract (`migration-kit/scripts/queue_runner.mjs`): `source` and `target` are the first two columns; the rest is the disposition record. Moved rows point at the scope.
- Writable: exactly the "Allowed files" of the scope. Nothing under `pekko-actor/actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum`, READMEs.

## 2. Upstream search

No Pekko spec asserts the ordering of `addFirst` on the mailbox queue; `ActorWithStashSpec` and `ActorWithBoundedStashSpec` exercise it only through a stash. This is "no queue-level spec found by this search", not proof of absence.

## 3. Decisions

- **READ rows are alternative evidence pending Owner approval (ADR 0033 D13)**: not Pekko parity, not V3.
- No export, no cell wiring (D-1). The queue does not know capacity and publishes no dead letters; the overflow rule is the first consumer's (D-2).
- The queue does not reverse a sequence (D-5). Do not add a batch or reversing method.
- Out of scope: stash and unstash, bounded deque discipline, `Terminated` re-receivability, `laneItem.after`, `pushed`, `maxDepth`.

## 4. Conventions

- Header: `queue.go` keeps its single `// Ported from` line; `queue_test.go` keeps the header it has.
- Scenarios are deterministic: no goroutine, no `time.Sleep`, no polling, no clock, no `-race`.
- No new exported symbol except the method on the internal type; no `//nolint`; cyclomatic complexity at most 10.
- Unknown behavior: `// TODO(MBX-15-edge): <reason>` and keep going (kit rule 8). Comments claim no more than is shown.
