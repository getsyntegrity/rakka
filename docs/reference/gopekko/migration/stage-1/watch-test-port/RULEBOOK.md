# Rulebook: port of DeathWatchSpec (Scala, Pekko v1.7.1) to Go tests for the local watch increment

Status: **approved by the owner on 2026-10-10 at commit `89f6abb`** for this scope only: six faithful cases, one reduced (DEV-4, section 3) and four skipped with a stated reason. The approval does not establish parity of `pekko-actor`. It replaces a rulebook and a runbook that were drafted by another session and
lost with its worktree (see section 7). goPekko is a redesign, so this is a design document (kit `README.md`: "If you're
redesigning"), not a structure-preserving mapping. Loop agents **cite it, never edit it**; amendments are queued for the
owner (kit standing rule 1). Contract: ADR 0010, section "Accepted for the local watch increment"; the runtime is
`pekko-actor/actor/watch.go`. Nothing here claims parity of `pekko-actor`.

## 0. Unit of work

One unit: `upstream/pekko/actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala` becomes
`pekko-actor/actor/watch_test.go` (`manifest.tsv`). The runtime is **not** a unit: it exists and is read-only for the loop.

## 1. Target conventions

- **R1.1** `package actor_test`, public API only (`github.com/getsyntegrity/gopekko/pekko-actor/actor`). Imports: the Go
  standard library and that package. No `go-specs`, no other module, no network. No `time.Sleep` as a witness: wait on
  channels, `sys.Done(h)`, or the `await` helper; a negative assertion ("no `Terminated`") needs a witness that the
  watcher processed a later message queued after the point where the event would have arrived.
- **R1.2** Already defined in package `actor_test` and **not to be redefined**: from `helpers_test.go` `bg`, `intLog`,
  `fnActor`, `factoryOf`, `await`; from `watch_api_test.go` and the other test files every identifier that `rg -n '^(func|type|var|const) '
  pekko-actor/actor/*_test.go` lists. New helpers are prefixed `dw`; new tests are named `TestDeathWatchSpec_<short>`.
- **R1.3** Header: the Apache-2.0 header of `pekko-actor/actor/cell.go` and `// Ported from upstream/pekko/actor-tests/src/test/scala/org/apache/pekko/actor/DeathWatchSpec.scala`.
- **R1.4** Each ported test carries a comment `// DeathWatchSpec.scala:<line>: "<Scala test name>"` and, where it differs, one line
  saying how (R3).

## 2. Constructs and their mapping

| Scala | Go (this module) |
|---|---|
| `system.actorOf(Props.empty)` | `actor.Spawn(sys, name, factoryOf(&fnActor{}))`; names are unique per test |
| `context.watch(target)` in the constructor | `ctx.Watch(target)` in `PreStart` (optional `PreStarter`) or in `Receive`. `Spawn` returns before `PreStart` runs: synchronize on a message the actor handles after `PreStart` |
| `case t: Terminated => testActor.forward(WrappedTerminated(t))` | an `OnTerminated(ctx, actor.Terminated) error` method that records `t.Who()` in a channel or a log |
| `watchWith(ref, CustomWatchMsg(ref))` | `ctx.WatchWith(ref, msg)`; the message arrives in `Receive` |
| `unwatch` | `ctx.Unwatch(ref)` |
| `terminal ! PoisonPill`, `system.stop(ref)` | `sys.Stop(ref)`; to know the stop finished, `ch, _ := sys.Done(ref); <-ch` through `await` |
| `testActor`, `expectMsg`, `TestProbe`, `TestLatch` | channels created by the test; a latch is a channel closed by the actor |
| `sender() ! x` | no implicit sender exists: reply on a channel the test passes in the message or captured by the actor |
| `expectNoMessage()` | forbidden as a witness (R1.1); use a settle message |
| `Identify` / `ActorIdentity` liveness probe | send an `int` and wait on a channel closed in `Receive`; "still alive" is `Done` not closed |

Out of scope, never translated: `PoisonPill`, `Kill`, `Supervisor`, `SupervisorStrategy`, `handleFailure`, `ActorKilledException`,
`DeathPactException` as an observed type, `context.actorOf` (children), `become`/`unbecome`, `Stash`, `Deploy`, `filterEvents`,
`DeathWatchNotification` injected by hand, `isTerminated`. A construct outside this table is a gap: R5.

## 3. Cases: seven to port, four to skip

| Line | Scala test | Decision |
|---|---|---|
| 135 | notify with one Terminated message when an Actor is stopped | **port**. The watcher receives `"hallo"` first (its `PreStart` has run), then `Stop(terminal)`; exactly one `Terminated(terminal)` |
| 145 | ... when an Actor is already dead | **port**. `Stop` and `Done` of the terminal first, then spawn the watcher |
| 154 | ... all monitors ... when an Actor is stopped | **port**. Three watchers, three `Terminated`, one each |
| 169 | ... _current_ monitors | **port**. Two watchers stay; a third calls `Watch` then `Unwatch` and gets nothing; a message round trip with the third precedes the `Stop` |
| 216 | fail a monitor which does not handle Terminated() | **port, reduced (DEV-4)**. `failed` and `brother` (watches `failed`, no `OnTerminated`) and an `observer` of `brother`; `Stop(failed)`; assert that `brother` finishes (`Done`) and that `observer` receives `Terminated(brother)`. Not asserted, because goPekko has no supervisor, no `Kill` and no public cause: the supervisor's `Failed` events, `ActorKilledException`, `DeathPactException`, their order |
| 268 | discard Terminated when unwatched between sysmsg and processing | **port**. The watcher is held inside a handler on a channel, the watched actor stops and is `Done`, `Unwatch` is called, the handler is released; the watcher (no `OnTerminated`) must stay alive: it handles two later messages and its `Done` is not closed |
| 293 | watch with custom message | **port**. `WatchWith(subject, msg)`; after `Stop(subject)` the message arrives once |
| 195 | not when restarted | skip: restart does not exist |
| 248 | watch a child with the same name after the old died | skip: children and `become` do not exist |
| 257 | only notify when watching | skip: injects a system message |
| 305 | stash watchWith termination message | skip: `Stash` does not exist |

## 4. Semantics the tests may rely on (ADR 0010, accepted scope)

`Terminated` is queued behind the user messages queued before the notification and after the watched actor's `PostStop`; `Done` of the
watched actor closed implies the notification is already queued; `Unwatch` discards a queued `Terminated`; watching a stopped actor
delivers; watching oneself does nothing; a repeated `WatchWith` with an equal comparable message is a no-op. A `Terminated` without
`OnTerminated` stops the watcher. Nothing else may be assumed: no ordering among several watchers of one actor.

## 5. The sanctioned escape hatch

When neither this rulebook nor ADR 0010 decides a question, write the most conservative test (assert less, never invent behavior) and leave
`// PORT-GAP(<short-id>): <what is unknown>`. The orchestrator collects the markers into the amendment queue. Do not widen the scope and do not edit
anything but the target.

## 6. Per-file obligations

1. `gofmt -l` prints nothing; the in-loop typecheck passes: `go vet -C /Users/pablogore/workspace/pablogore/goPekko-worktrees/local-watch/pekko-actor ./actor` (it compiles the tests against the package).
2. The header and the per-test comment of R1.3 and R1.4 are present.
3. No test depends on the wall clock to decide that something did or did not happen (R1.1).
4. No identifier of R1.2 is redefined; no file other than the target is created or edited.
5. **Tests are not run by the loop.** The owner runs `go test -race` at the gate (kit `settings.README.md`: only the build daemon re-runs tests). Until then every test result is unknown.

## 7. Deviation log

| Date | Where | Deviation | Reason |
|---|---|---|---|
| 2026-10-10 | whole kit flow | Rulebook, runbook and manifest reconstructed from kept sources: kit `prompts/04`, `settings.README.md`, `queue_runner.mjs`; the pilot's `RULEBOOK.md` and `permissions.md` (commit `32237c5`); ADR 0010; the upstream spec | The earlier session's drafts (an ADR 0013 and its RULEBOOK/RUNBOOK/manifest/settings, never committed) were lost with its worktree. Their content is **not** recovered and is not used as an authority |
| 2026-10-10 | kit `README.md` model guidance | One model for every agent (implementer, two reviewers, third ruler, fixer), declared on each call as `model: "sonnet"`; the identity `claude-sonnet-5-5` is what a probe agent reported about itself, not an external verification. Accepted by the owner | Owner's plan of 2026-10-10. The kit advises the largest model for the rulebook and for skeptic reviewers; the rulebook was written by the same model that will run the loop |
| 2026-10-10 | `settings.json` | The deny list is that of `migration/settings.go.json` unchanged; added allows: `gofmt`, `eza`, `Read`/`Edit` of the worktree and the one target file | `go test` stays denied (kit); the target lives in another worktree than the session's directory |
| 2026-10-10 | kit `prompts/04` | Case 216 is ported in a reduced form (section 3); the kit's "four agents per file" applies to the one file | goPekko has no supervisor; the unit is one test file, not a batch of 100 |
| 2026-10-10 | kit `CLAUDE.md` rule 4 / `settings.README.md` | The orchestrating session cannot commit while the deny is installed: the owner removes `.claude/settings.json` after the loop, then the session runs the gate and commits | `git commit` is denied in the file |
| 2026-10-10 | case 268 (`DeathWatchSpec.scala:268`), DEV-5 | **Approved by the owner.** The case is reduced to the clause "`Unwatch` called inside a handler discards a `Terminated` already queued". The Scala schedule (the unwatch arrives as a user message after the watched actor is done) is not reproduced. `PORT-GAP(dw-unwatch-by-message)` in `watch_test.go` is a decided difference, not an unknown | Pekko enqueues `Terminated` when the watcher processes the death notification; goPekko at the moment of the notification (ADR 0010, "Difference from Pekko in where Terminated is placed"). Both are legal interleavings, but only the first lets a later user message overtake it |
| 2026-10-10 | case 216 (`DeathWatchSpec.scala:216`), DEV-6 | **Approved by the owner as a limit of evidence; the missing assertions stay pending.** Not asserted: the cause of `brother`'s stop (`ErrDeathPact` has no public check), that the observer stays alive, the supervisor's `Failed` events, `ActorKilledException` and `DeathPactException` and their order. Closing them needs a public cause or a supervisor | goPekko has no supervisor and no `Kill`. Approving the deviation does not turn the missing assertions into coverage |
| 2026-10-10 | `judge/mutate_go.py` and the Go judge harness, DEV-7 | **Approved by the owner as a record of deviations and limits of evidence, not as coverage.** `GMstop` was re-anchored (same defect, new text) because the reschedule condition of `drain` gained the lane. The S18 gate of the Go harness releases once (`sync.Once`); assertions and detection criterion are unchanged | GMdup duplicates the last chain message; the second `close` panicked inside the watcher and left S18 in timeout. The campaigns that show it (`8b64ad5`, `cc786e4`) are invalid history for this gate |
