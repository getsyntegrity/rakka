# Rulebook: public ports of ActorSystemSpec:164-186, ActorModelSpec:311 and DeathWatchSpec:195-211 (#1475)

Status: **proposed; the Owner approves it before any loop agent runs** (precedent: `migration/stage-1/watch-test-port/RULEBOOK.md`, approved 2026-10-10). Scope: `docs/scopes/1475-public-ports.md`
(G2 given by the Owner on 2026-11-07). Loop agents **cite it, never edit it**; amendments are queued for the Owner. Pekko pin: `upstream/pekko` v1.7.1 (`9a54d1294c`). goPekko is a redesign: this is a
design document, not a structure-preserving mapping, and it claims no parity of `pekko-actor`.

## 0. Unit of work

Three manifest rows (`manifest.tsv`), three new test files, **tests only**. The runtime (`pekko-actor/actor/*.go` that is not `_test.go`) is read-only for the loop. No existing file is edited.

## 1. Target conventions

- **R1.1** `package actor_test`, public API only (`github.com/getsyntegrity/gopekko/pekko-actor/actor`). Imports: the Go standard library, that package, `github.com/getsyntegrity/go-specs` (test matchers, `AGENTS.md`). No `time.Sleep` as a witness: wait on channels, `sys.Done(h)` or the `await` helper; a negative assertion needs a witness that the
  actor processed a later message queued after the point where the event would have arrived.
- **R1.2** Never redefine an identifier that `rg -n '^(func|type|var|const) ' pekko-actor/actor/*_test.go` lists (`bg`, `fnActor`, `factoryOf`, `await`, `dwSystem`, `dwSpawn`, `dwWatcher`, ...). New helpers are prefixed
  `ds` (P1), `om` (P3), `dr` (P4). Test names: `TestActorSystemSpec_<short>`, `TestActorModelSpec_<short>`, `TestDeathWatchSpec_<short>`.
- **R1.3** Header: the Apache-2.0 header of `pekko-actor/actor/cell.go` and exactly one `// Ported from upstream/pekko/<path of the spec>` line (`.github/scripts/check-headers.sh`).
- **R1.4** Each test carries `// <Spec>.scala:<line>: "<Scala test name>"` and a comment block `PORT-GAPs:` listing every construct of the spec that the test does not reproduce, each with its reason (R3). One public
  test that cites the spec, asserts every observable effect and lists its PORT-GAPs counts as both PAT and PS (Owner, 2026-11-07); the test never assigns a level. **P1 alone does not count as PS of C-10a; the contract of `ActorSystemSpec:164-186` stays incomplete until P2 (#1479)**, and P1 says so in its comment.
- **R1.5** A test is never skipped, weakened or deleted to make it pass (`AGENTS.md`). An assertion the runtime cannot meet stays, fails, and is classified with `gopekko-failure-triage` (R4).

## 2. Constructs and their mapping

| Scala | Go (this module) |
|---|---|
| `TestProbe`, `probe.watch`, `expectTerminated` | `dwWatcher`/`OnTerminated` recording on a channel, awaited with `await` |
| `a.tell("run", probe.ref)`, `sender()` | the message carries the sender explicitly (ADR 0032 D3, D4, D6); the dead letter's sender would be read from `DeadLetter`, which is `{Msg, To}` today: **AC-2 depends on #896** (`PORT-GAP(dl-sender-896)`, Owner 2026-11-07, section 6) |
| `EventFilter.info/warning(pattern, occurrences = 1).intercept` | **not in this run.** The INFO "was not delivered" (`DeadLetterListener.scala:121-137`) is P2, written and delivered in the PR of #1479 together with the runtime log (goPekko has none today). The WARNING "received dead letter from" is formatted by the test-kit `TestEventListener` (`TestEventListener.scala:576-577`), not by the runtime: a PORT-GAP, never asserted |
| `system.eventStream` dead letters | `System.SubscribeDeadLetters` |
| `interceptedDispatcher`, `assertRefDefaultZero(registers, unregisters, msgsReceived, msgsProcessed)` | counters kept by the test's actor (received, processed) and `Done`; the dispatcher id and the registers/unregisters counters are PORT-GAPs (`jvm-only` executor plumbing) |
| `CountDown(latch)`, `Wait(ms)` | a channel closed by the actor; a hold on a channel released by the test (no sleep) |
| `Supervisor(OneForOneStrategy(maxNrOfRetries = 2))`, `terminal ! Kill` | `actor.Spawn` with a supervisor and `Budget{MaxRetries: 2}`; `System.Kill(h)` (PORT-GAP: `Kill` is a message in Pekko) |
| `terminal ? "foo"` | an `int` sent to an actor that closes a channel; "still answers" is the channel closing and `Done` not closed |
| `system.stop(a)` | `sys.Stop(h)`; completion is `sys.Done(h)` |

A construct outside this table is a gap: R3.

## 3. Cases

| Spec line | Pekko test | Go test (file) | Notes |
|---|---|---|---|
| `ActorSystemSpec.scala:164-186` | log dead letters | `P1` only: the dead letter of the message told after the termination, published once, with message and recipient (the sender waits for #896, section 6) (`actorsystem_deadletters_spec_test.go`) | P2 (the INFO, #1479) is **not** part of this run; the WARNING is a testkit PORT-GAP. P1's comment says "P1 alone is not PS of C-10a: the contract stays incomplete until P2 (#1479)". The row C-10a is **not** split |
| `ActorModelSpec.scala:311-329` | process messages one at a time | `P3` (`actormodel_one_at_a_time_spec_test.go`) | counts 1 then 3 received and processed; stop leaves them unchanged and `Done` closed |
| `DeathWatchSpec.scala:195-211` | notify with a Terminated message once when an Actor is stopped but not when restarted | `P4` (`deathwatch_restart_spec_test.go`) | two restarts with no `Terminated`, still answers, third failure stops, exactly one `Terminated`, `Done` closed |

## 4. The sanctioned escape hatch and failures

When neither this rulebook, the ADRs nor the inventory decides a question, write the most conservative test (assert less only where the spec's construct does not exist in Go, never to hide a failure) and leave
`// PORT-GAP(<short-id>): <what is unknown>`. If a ported assertion fails on goPekko, run it against Pekko's contract first (the spec is the oracle) and classify it with `gopekko-failure-triage`; a runtime
finding becomes an issue, the test stays red-by-design only as the Owner decides (the PR does not merge a red test: the Dev reports it and stops).

## 5. Per-file obligations

1. `gofmt -l` prints nothing; the in-loop typecheck passes: `go vet -C <worktree>/pekko-actor ./actor`.
2. Header, per-test comment and `PORT-GAPs:` block present (R1.3, R1.4).
3. No wall-clock witness (R1.1); no identifier of R1.2 redefined; no file other than the three targets created or edited.
4. **Tests are not run by the loop.** The orchestrating session runs `go test -race` at the gate, with the settings file removed.

## 6. Deviation log

| Date | Where | Deviation | Reason |
|---|---|---|---|
| 2026-11-07 | model plan (Owner decision) | Implementer and fixer: `model: "sonnet"`; the two reviewers and the third ruler: `model: "opus"`; explicit on every call | Owner, 2026-11-07, on #1476: skeptic roles get the larger model (kit `README.md`) |
| 2026-11-07 | AC-2 (the sender of the dead letter), Owner decision on #1481 | The assertion of the sender is a `PORT-GAP(dl-sender-896)` in P1 and depends on #896; `DeadLetter` is `{Msg, To}` (`ref.go:47-53`) and ADR 0032 D6 "Keep" is accepted and not implemented: a missing capability, not an accepted divergence. #896 adds the assertion and removes the PORT-GAP | Owner, 2026-11-07; QA escalated (`qa-review` Stage 1: a criterion without its test) |
