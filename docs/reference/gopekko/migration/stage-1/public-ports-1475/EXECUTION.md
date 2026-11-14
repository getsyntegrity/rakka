# Execution record: public ports of #1475

Run on 2026-11-07, following `RULEBOOK.md` and `RUNBOOK.md`. Base: `develop` `359d4a73`. Loop: three implementers (`sonnet`), two reviewers over all three targets (`opus`), one fixer (`sonnet`); settings installed from `settings.publicports.json` and removed after `queue_runner.mjs verify` (`3/3 done`). Deviations: the reviewers each read all three targets instead of one pair per target (three small files); the PR is split in two for the 400-line budget (P3 alone; P1 + P4 + this record).

## Review findings applied (confirmed by the reviewers)

| Finding | Applied |
|---|---|
| P1 settle witness proved nothing (the settle went to the live probe) | the settle goes to the dead actor; exactly two dead letters `[boom, settle]`; synchronous publication named as the witness (`ref.go:71-76`) |
| P3 could stay green under an overlapping Receive (a microsecond window) | a second phase of 300 hold/second-message/release cycles (`omOverlapCycles`); `PORT-GAP(om-overlap-window)` |
| P3 restarts effect of `assertRefDefaultZero` not asserted | `PreRestart`/`PostRestart` counter asserted 0 |
| P4 "no Terminated on restart" could pass on a runtime that notifies on restart (`watch.go:333-336` fixes the place at the next drain) | a second round trip before the assertion |
| P4 comment: Scala's decider escalates (not stops) on a non-Exception Throwable (`FaultHandling.scala:260-262`) | corrected |

## Gate (settings removed; worktree at the head of this branch)

- `gofmt -l pekko-actor`: empty. `go vet ./...` and `go test -count=1 ./...` in `pekko-actor`: ok. The three tests: `go test -race -count=100`: ok.
- Mutations, each on a copy of `pekko-actor`, the named test run under `-race`; every one fails **by an assertion** (no compile error):

| Test | Mutation | Result |
|---|---|---|
| P1 | no dead letter published for a message told to a stopped actor (`ref.go`) | `actorsystem_deadletters_spec_test.go:96`: expected 0 to equal 2 |
| P1 | the dead letter published twice | `:96`: expected 4 to equal 2 |
| P3 | GMoverlap with its condition `c.name == "r"` replaced by `true` (any actor), `-count=30` | `actormodel_one_at_a_time_spec_test.go:166`: expected 3 to equal 2, and the 300-cycle subtest fails |
| P4 | GMrestartnotify (a restart tells the watchers) | `deathwatch_restart_spec_test.go:114`: expected 1 to equal 0 |
| P4 | GMtwice (each watcher told twice, without the deregistration) | `:125`: expected 1 to equal 0 |

## Open: AC-2 (the sender of the dead letter)

`DeadLetter` is `{Msg, To}`; the runtime drops the sender (`ref.go:47-53`, "until #896"; ADR 0032 D6 "Keep" is accepted). The assertion cannot be written on the public API, so P1 declares `PORT-GAP(dl-sender-896)`. This is a **runtime gap, not a test gap**; **Owner decision (2026-11-07): option a, AC-2 depends on #896**; scope, RULEBOOK sections 2, 3 and 6 and the P1 comment are amended; #896 carries the acceptance criterion. No test was weakened: the assertion does not exist to weaken.

## PORT-GAPs

P1: INFO log (P2, #1479); WARNING (testkit, `TestEventListener.scala:576-577`); `TestProbe`/`EventFilter`; system settings; `dl-sender-896`. P3: dispatcher interceptors, dispatcher id and registers/unregisters (`jvm-only`), suspensions/resumes, enqueue-time `msgsReceived`, latch timeouts, `RepointableRef.isStarted`, `om-processed-collapsed`, `om-overlap-window`, `om-point-in-time-absence`. P4: `Kill` as message, `OneForOneStrategy` exception list, `filterException`, `ask`, `startWatching`, `isTerminated`; the shared `dwExactlyOne` has the same settle weakness and is outside this unit.
