# Scope: Public ports of three Pekko specs (#1475)

Status: **G2 given by the Owner on 2026-11-07**, with these decisions: the log half of `ActorSystemSpec:164-186` (P2) moves to the PR of #1479 together with the runtime log (the missing capability), and the WARNING stays a testkit PORT-GAP, C-10a stays `unassigned` and the row is not split; the dispatcher-interceptor PORT-GAPs of `ActorModelSpec.scala:311` are listed. The kit `RULEBOOK.md` of `migration/stage-1/public-ports-1475/` was approved by the Owner on 2026-11-07 with changes (P1 alone in this run, P2 goes to #1479; sonnet for implementer and fixer, opus for reviewers and ruler). Written against `develop` `656edf1e`. No Dev is assigned and nothing is implemented. **Kit run required
(ADR 0017, ADR 0024 T6, `AGENTS.md`)**: these are ports of Pekko specs; the kit package `migration/stage-1/public-ports-1475/` (manifest, RULEBOOK, RUNBOOK) is
prepared with the approval of this scope and the Dev writes the execution record. Pekko pin: `upstream/pekko` `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1).

## Objective

Give three inventory sub-rows the public test they lack (PAT and PS under the Owner decision of 2026-11-07: a test in `package actor_test`, citing the spec with `path:line`,
asserting every observable effect, listing its PORT-GAPs): D-01a and C-08c fully; C-10a only P1 (PAT; PS only with P2, #1479). **Tests only; no production change; no README change; no level assigned.**

## Contracts consumed (public API, no change)

`actor.NewSystem`, `System.SubscribeDeadLetters`, `actor.DeadLetter`, `System.Stop`, `System.Kill`, `System.Done`, `actor.Spawn` with a supervisor and `Budget`
(`pekko-actor/actor/deadletter.go`, `kill.go`, `stop.go`, `system.go`, `spawn.go`). Helpers of the existing external test package (`fnActor`, `factoryOf`, `await`, `dwSystem`,
`dwSpawn`, `dwWatcher` in `deadletter_test.go` and `watch_test.go`) may be reused; they are test files and are not edited.

## Criteria → scenarios

All scenarios are go-specs in `package actor_test`, unit level, no real resource. Each cites its Pekko test with `path:line` and carries a "PORT-GAPs" comment.

| # | Criterion | Pekko contract | Scenario (file) |
|---|---|---|---|
| AC-1 | After an actor has terminated, a message told to it is published once as a `DeadLetter` carrying the message and the recipient | `ActorSystemSpec.scala:164-186` ("log dead letters"): the dead letter of `a.tell("boom", probe.ref)` after `probe.expectTerminated(a)` | `P1` in `pekko-actor/actor/actorsystem_deadletters_spec_test.go` |
| AC-2 | **Depends on #896 (Owner, 2026-11-07).** The sender of that dead letter is the sender of the `Tell` (ADR 0032 D3, D4, D6). Today the assertion is `PORT-GAP(dl-sender-896)` in P1: `DeadLetter` is `{Msg, To}` and the runtime drops the sender (`ref.go:47-53`). ADR 0032 D6 "Keep" is accepted and not implemented: a missing capability, not an accepted divergence. #896 adds the assertion to `actorsystem_deadletters_spec_test.go` (P1) and removes the PORT-GAP | same | `P1`, in #896. **Fulfilled by #896 (2026-11-10):** P1 asserts the sender and `PORT-GAP(dl-sender-896)` is gone from the code (#1513, #1517) |
| AC-3 | **Moved out of this run (Owner, 2026-11-07):** the INFO "was not delivered" (`DeadLetterListener.scala:121-137`) is P2, written and delivered in the PR of #1479 with the runtime log; the WARNING is a testkit PORT-GAP (`TestEventListener.scala:576-577`). P1 alone is not PS of C-10a; the contract stays incomplete until P2 | same | `P2`, in #1479 |
| AC-4 | A message processed by an actor is processed one at a time: while the first `Receive` is held, a second message is not received; after release it is, and the counts of received and processed messages are 1, then 3 (Pekko: `registers = 1`, `msgsReceived`, `msgsProcessed`) | `ActorModelSpec.scala:311-329` ("process messages one at a time") | `P3` in `pekko-actor/actor/actormodel_one_at_a_time_spec_test.go` |
| AC-5 | Stopping the actor after the counts leaves them unchanged and the actor deregistered (`Done` closes) | `ActorModelSpec.scala:326-328` | `P3` |
| AC-6 | A watched actor that is restarted twice sends **no** `Terminated`, still answers afterwards, and sends exactly one when it finally stops, after which it is terminated | `DeathWatchSpec.scala:195-211` ("notify with a Terminated message once when an Actor is stopped but not when restarted") | `P4` in `pekko-actor/actor/deathwatch_restart_spec_test.go` |
| AC-7 | `Terminated` carries the subject's identity and `Done` is closed after it | same | `P4` |

**PORT-GAPs expected** (to be written in the comments, not hidden): `interceptedDispatcher` counters and the dispatcher id (JVM executor plumbing, `jvm-only`); `TestProbe`/`EventFilter`
(replaced by a subscription; the log assertions are P2, #1479); `Kill` as a message vs `System.Kill` (as in `watch_test.go`); `OneForOneStrategy(maxNrOfRetries = 2)` vs `Budget{MaxRetries: 2}`.

Each scenario must fail under a wrong implementation, and the Dev records, per scenario, the mutation that made it red (kit rule): `P1` red if the dead letter is not published or is published
twice; `P3` red if the executor runs two `Receive` calls of one actor at once (the GMoverlap defect of the judge); `P4` red if a restart notifies the watchers (GMrestartnotify) or the final stop notifies twice.

## Allowed files

Exactly three new files, nothing else changes:

- `pekko-actor/actor/actorsystem_deadletters_spec_test.go`
- `pekko-actor/actor/actormodel_one_at_a_time_spec_test.go`
- `pekko-actor/actor/deathwatch_restart_spec_test.go`

plus the kit package `migration/stage-1/public-ports-1475/` and its execution record. No `.go` outside `*_test.go`; no edit of an existing test, of the judge or of the inventory (the inventory update is a later docs PR).

## Independence

| Scope | Allowed files | Shares a file with |
|---|---|---|
| this one | the three test files above, the kit package | none |
| #1474 (harness change for the absent `Terminated`) | `pekko-actor/actor/judge_internal_test.go`, `judge/mutate_go.py`, its docs | none |

The two scopes are independent but **both change the `pekko-actor/` tree**, so both invalidate the Go CURRENT. Their merges are coordinated: one Go campaign (`--runs 30`, `-race`) is run after both,
and the checker must be at CURRENT for all designed mutations before the inventory is updated. If the PR touches `pekko-actor/`, it says so and that a new campaign is needed before merging.

## Appetite

One small batch: three test files, at most 300 authored lines in total (budget 400).

## Definition of Done

- Every criterion of this run (AC-1, AC-4 to AC-7) has its scenario, passing; AC-2 depends on #896 and AC-3 belongs to #1479
- `gofmt`, `go vet`, `unshare -rn scripts/each_module.sh go test -count=1 ./...` green; coverage and complexity unchanged (tests only); `scripts/check_deps.sh` green
- The execution record of the kit run is written; each scenario names the mutation that made it red
- The PR states that `pekko-actor/` changed (tests) and that CURRENT falls until the joint campaign
- `qa-review` in a clean context, bound to the head SHA, before merge
