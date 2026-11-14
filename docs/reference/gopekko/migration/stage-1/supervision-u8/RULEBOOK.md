# RULEBOOK: Supervision U8, judge scenarios S13, S14 and S16 in the Go gate (#175)

Status: PROPOSED, for the Owner's approval together with the package (`RUNBOOK.md`). Unit U8 of ADR 0017 (A11, U3 D6, U4). Base: `develop` at `b3275d740ad6454b5d8b7b497e4cd8430b01006f` (U7 merged, #449 closed; verified with `git rev-parse origin/develop` on 2026-10-16). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`; upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`). Paths are relative to the repository root.

Nothing here reopens a decision already accepted in ADR 0017. A row marked **NEW** is a decision this unit asks the Owner to accept. The message that asked for this package reached me partly unreadable; section 11 lists how I read it. No parity is claimed.

## 1. Goal and what #175 requires
#175 (ACT-029): "Implementar eventos/scenarios exigidos y mutantes dirigidos antes de incluirlos en gate"; MVP acceptance: "Detectores judge S13, S14 y S16 habilitados con evidencia". So the order is: the Go side emits the events the scenarios need; the three scenarios are implemented; **directed mutants** show that each one can fail; only then are they added to `judge/go-scope.json`.
The Pekko side already has the three scenarios and their specs (`judge/pekko-harness/Judge.scala`, `judge/scenarios/S13.json`, `S14.json`, `S16.json`); this unit does **not** change them. The Go side must produce logs that the same comparator judges against the same specs.

## 2. The contract to mirror (read at the pinned commit)
| Scenario | Pekko body (`Judge.scala`) | Spec (required / rules) | Contract |
|---|---|---|---|
| S13 `:386-392` | `w` watches `c`; `c` receives `boom` (throws `RuntimeException`), then `R` | `received c R count 1 i 2`; `G6.d {watcher w, subject c}` | a restart yields **no** `terminated` at the watcher, and the new instance (`i` = 2) receives `R` |
| S14 `:396-398` | `w` spawns `child` (its `preStart` throws on instance 1 only) and watches it | `terminated a=w subj=child`; `G7.b {actor child, watcher w, subject child}` | an initialization failure under the default decider **stops** the child (no `pre-restart`) and the watcher gets `terminated` |
| S16 `:410-420` | `boss` (decider records `decided`, returns Restart) spawns `w1`, `w2`, `c`; `w1`: `watchWith(c, X)` then `watch(c)` then `ack`; `w2`: `watchWith(c, X)` twice then `ack` | `received w1 ack`, `received w2 ack`; `I6.a {watcher w1}`, `I6.b {watcher w2}` | a second registration with another message fails the watcher (`IllegalState`) and its supervisor decides once; the identical repeat is a no-op and decides nothing |
Clauses: `G6.d` and `I6.b` forbid an event (`terminated`, `decided`); `G7.b` forbids `pre-restart` of the child; `I6.a` requires exactly one `decided` with `k = IllegalState` for `w1` (`judge/comparator.py`). **Consequence:** a Go log without `pre-restart` and `decided` events would pass `G7.b`, `I6.a` (no) and `I6.b` **vacuously**. The events below are therefore part of the contract, not decoration.
Pekko semantics: the default decider stops on `ActorInitializationException` and restarts on any other `Exception` (`FaultHandling.scala:216-221`); a conflicting watch throws `IllegalStateException` (`dungeon/DeathWatch.scala:125-131`); a restart sends no `Terminated`.

## 3. The Go runtime after U1 to U7 (read at `b3275d7`)
| Need | Go fact | Evidence |
|---|---|---|
| a restart keeps the Ref, mailbox, watchers; no `Terminated` | U2 | ADR 0017 U2; `supervision_restart.go` |
| a new instance on each restart | the factory is called at each (re)creation | `supervision_restart.go` (`installNew`), U4 |
| `PreRestart(ctx, cause, FailedMessage)`, `PostRestart(ctx, cause) error` | optional interfaces; a custom `PreRestart` **replaces** the default one | `supervision_hooks.go`; W25 |
| default decider | `Stop` for `*InitializationError`, `ErrDeathPact`, `ErrGoexit`; `Restart` otherwise (so `ErrWatchConflict` restarts) | `supervision_decider.go:57`; ADR 0017 `:491` |
| a declaring parent | `Supervisor[T]`, `SupervisorStrategy() Strategy[T]`; the zero `Strategy` means the default | `supervision.go` |
| a conflicting `WatchWith`/`Watch` fails the actor with `ErrWatchConflict`; an identical repeat is a no-op | `addRegistration` | `watch.go:183-202` |
| **"identical" is `==` on a comparable value; a type with a `func`, slice or map field is never identical to itself** | `sameMessage` uses `reflect.Value.Comparable()` | `watch.go:113-123` |
The last row is a blocker for a literal port. The judge's message type `judgeMsg` has `run` and `after` func fields (`judge_internal_test.go:49-53`), so a repeated `WatchWith(c, judgeMsg{id:"X"})` would be a **conflict** in Go, and S16's `w2` (the no-op case, clause `I6.b`) could not pass.

## 4. What the Go judge lacks today (read at `b3275d7`)
`judge_internal_test.go`: `judgeNode` emits only `pre-start`, `received`, `received-end`, `terminated`, `post-stop-begin`, `post-stop`, all with `i: 1`; it has no `PreRestart`, no `PostRestart`, no instance counter, no failing `PreStart`, no supervisor and no `decided`. The header (`:13-17`) says "goPekko has no restart" and that S16 is not run for lack of a supervisor: both are stale since U1.
The table of scenarios is `judgeScenarios` (`:627-639`), the scope list is `judge/go-scope.json` (`S01 S02 S07 S08 S09 S10 S11 S18 S19 S20 S21`).

## 5. Design of the Go side (NEW decisions D1-D4)
All in `pekko-actor/actor/judge_internal_test.go`. English, no `//nolint`, gocyclo at most 10, no parity claim.
**D1. Instance numbers and restart events.** A per-name instance counter, incremented in the factory (the Pekko `Node` constructor does `rec.nextInst`), gives `i` to every event of an instance. `judgeNode` gets `PreRestart` and `PostRestart`:
- `PreRestart` emits `pre-restart {i, m}` (`m` = the id of the failing message, if it is a `judgeMsg`, as `Judge.scala:119`), then does what Pekko's `super.preRestart` does: unwatch and stop every child, then call its own `PostStop` (which emits `post-stop-begin`/`post-stop` for the old instance).
- `PostRestart` emits `post-restart {i}` and then runs `PreStart` (Pekko's default `postRestart` calls `preStart`), which emits `pre-start` of the new instance.
Order per instance: `pre-restart(1)` → `post-stop(1)` → `post-restart(2)` → `pre-start(2)`, the order `G4.a` expects. Existing scenarios have a single instance, so their logs do not change.
**D2. Failing first start.** A flag on the node makes the **first** instance's `PreStart` return an error after emitting `pre-start` (Pekko: `failStart && inst == 1`). The default strategy then stops the child (`*InitializationError`).
**D3. `decided`.** A node can be a supervisor: `SupervisorStrategy()` returns a strategy whose decider emits `decided {subj, k, v:"Restart"}` and returns `Restart`; `subj` is the logical name of the failing child (the recorder's `names` map), `k` is `IllegalState` when `errors.Is(cause, ErrWatchConflict)` and the Go error type name otherwise. A node without the flag returns the zero `Strategy` (the default), so existing scenarios keep the default decider.
**D4. Comparable watch message for S16 (the blocker of section 3).** The watchers `w1` and `w2` of S16 use a second, small actor type whose message is a **comparable** struct without func fields (an id and an operation code), recording the same events (`pre-start`, `received`, `received-end`, `pre-restart`, `post-restart`, `terminated`, `post-stop`...) through the same recorder; their `WatchWith` message is a comparable value. `c` stays a `judgeNode`; a `Handle` is not typed, so a watcher of another message type can watch it. The alternative, changing `judgeMsg` for every scenario, touches all existing scenarios and is rejected. The new type and its events are documented in the header comment.

## 6. Scenario bodies (Go) and their completion witnesses
Each scenario must end by a witness in both modes: in `controlled` mode `e.await` runs the executor and a quiescent queue before the witness is a harness error; in `concurrent` it is bounded by the 10 s deadline.
- **S13.** `w` watches `c` (registration witnessed as S18 does); `c` receives a `boom` message whose `run` returns an error (`c` restarts); then `R`. Witness: `R` received by instance 2. **Completeness (NEW, stronger than Pekko's `got("c","R")`):** after `R`, a message to `w` whose `after` closes a channel is awaited; a `Terminated` that a wrong implementation delivered at the restart was queued at `w` before that message and is handled before it, so `G6.d` cannot pass because the log was cut early. The Pekko body and spec are unchanged; this strengthens only the completeness witness of the Go body (D5).
- **S14.** `w` spawns `child` with the failing-first-start flag and watches it. Witness, **two-way as in Pekko** (`Judge.scala:393-396`): `terminated(w, child)` **or** `pre-start(child, i=2)`. Without the second branch a mutant that restarts the child would make the run time out, and a timeout is never a detection.
- **S16.** `boss` (supervisor with the `decided` decider) spawns `w1`, `w2` and `c`; messages as in the table of section 2, with the comparable type for `w1`/`w2`. Witness: `ack` received by `w1` (instance 2, after the decision) and by `w2`. `w1` is restarted by `boss`; `w2` is untouched.
All three are `controlled` and `concurrent`, like their Pekko specs, and are added to the `judgeScenarios` table.

## 7. S22 in the Go gate: decision and reasoning (NEW D6)
S22 (stop during a restart, U7) is **not** added to the Go gate in this unit. This is a decision, not an omission:
1. S22 asserts `G5.f`: `pre-restart` 1, `pre-start` 1, `post-restart` 0, `post-stop` 2. In Go the `pre-restart` event can only come from a custom `PreRestart` (D1). A custom hook is user code: its `PostStop` call is never guarded (W25, `supervision_restart_goside_internal_test.go`), so a Go S22 built on it gives two `PostStop` calls **with and without** U7's change. It would pass on the pre-U7 code (mutant MU3 of U7 undetected), which violates #175's own rule that a scenario enters the gate only with directed mutants that it detects.
2. The code U7 changed, `defaultPreRestart`, is reached only when the actor has **no** custom `PreRestart`; then the Go runtime emits no judge-visible `pre-restart`. Mapping the runtime's internal `restart` sink event to `pre-restart` would be a new contract and needs an ADR.
3. Concurrent-only and held children (`spawnGated`, `postStopGate`) exist in the Go judge, so the mechanics are not the obstacle.
4. The behavior has deterministic witnesses already merged: W11b and T1-T4 (`supervision_restart_goside_internal_test.go`, `supervision_stoprestart_internal_test.go`), validated by mutants MU1, MU2b and MU3 in U7.
Recorded in `docs/judge.md`, the milestone table (S22 stays "Pekko only") and #461: revisit if the runtime exposes an observable event of the default restart path.

## 8. Directed mutants (the criterion of #175)
Four Go mutants, one per asserted clause, analogs of existing Pekko mutations (`judge/mutations/M12.patch`, `M14.patch`, `M43.patch`, `M59.patch`), added to `MUTATIONS` in `judge/mutate_go.py` (each anchor must occur exactly once in the named production file, else the mutation is stale):
| Id | Analog | Edit (intent; the implementer fixes the exact anchor) | Must be caught by |
|---|---|---|---|
| GMrestartnotify | M12 | the restart tells the watchers `Terminated` (in `supervision_restart.go`, at the start of `finishRecreate`, take the watchers and `notifyWatchers`) | S13 `G6.d` (controlled and concurrent) |
| GMinitrestart | M14 | the default decider restarts on `*InitializationError` (`supervision_decider.go`) | S14 `G7.b` |
| GMwatchoverwrite | M43 | a conflicting registration is silently ignored instead of failing the watcher (`watch.go`, `addRegistration`: `regConflict` becomes `regIgnored`) | S16 `I6.a` |
| GMwatchsamefail | M59 | an identical `WatchWith` also fails the watcher (`watch.go`, `addRegistration`: the equality case no longer returns `regIgnored`) | S16 `I6.b` |
Rules (as in `mutate_go.py`): a mutant is **met** when every designed `(scenario, mode, clause)` is `failed` with that clause fired; a build error, a harness error, an invalid campaign or a **timeout is never a detection**; the unmodified copy must pass everything. **Whole-gate effect:** adding S13, S14 and S16 to `go-scope.json` makes every one of the 14 existing mutants (13 + the benign one) be judged on the new cells too, and the committed Go report (`judge/go-mutation-report.tsv`, 252 rows, Go commit `2427544c98`, darwin/arm64) no longer matches the Go core or the scope. A new campaign is part of this unit (RUNBOOK 5). If an existing mutant becomes `timeout` or `harness-error` in a new cell, it is **unmet because of the new cells**: the work stops with the evidence; no mutant, scenario or clause is weakened to restore it.
Pekko mutations without a Go analog stay a declared gap (#461): M10, M11, M58 (S13 `G6.d`), M13 (S14 `G7.b`).

## 9. Registration and evidence
- `judge/go-scope.json`: S13, S14, S16 added to `scenarios`; `waivers` and `diagnostics` stay `{}`. No clause is waived.
- `migration/stage-1/milestone-1.md`: the "Go" column of the S13, S14 and S16 rows, the prose that lists the Go-registered scenarios and the Go observed count; S22 stays "Pekko only" with the reason of section 7.
- `scripts/check-judge-matrix.py` must keep passing. **Design and observed are reported separately** (design: Go registration and designed mutation pairs; observed: only from a committed campaign report). The observed Go line is whatever the new campaign gives; it is never restored by weakening anything.
- Calibration: S13, S14 and S16 get **no** Go calibration row (as S01 to S18): `run-go.sh` keeps 300 concurrent runs; calibrating them (`GO_CALIBRATION` in `judge/calibrate.py`) is a gap recorded in #461.
- Go gate command: `judge/run-go.sh <outdir> 300` passes for the 14 scenarios (concurrent 300 runs, controlled once).

## 10. Acceptance of #175, as observed
1. Events and scenarios implemented (D1-D4, section 6), documented in the header of `judge_internal_test.go`; the stale header text removed.
2. Directed mutants GMrestartnotify, GMinitrestart, GMwatchoverwrite, GMwatchsamefail met (valid detections only).
3. S13, S14, S16 in `go-scope.json` and passing: `judge/run-go.sh` exit 0 with the new scope; controlled once, concurrent 300 runs.
4. Existing mutants still judged and valid on the whole new scope (or the blocker reported with evidence, section 8).
5. Docs corrected: `docs/judge.md` Go scope, milestone table, the README and parity lines that say S13/S14/S16 are out of the Go gate, ADR 0017 U8 row; the S22 decision recorded.
#175 closes **only** if all five are observed and the Owner agrees.

## 11. How I read the request, and what I did not decide
The request for this package was partly unreadable. I read it as: prepare the RULEBOOK and execution package for U8 from `origin/develop`; mirror S13, S14 and S16 in the Go gate with the same observable contract as the Pekko side; contrast Pekko with the runtime integrated through U7; say which events and synchronization exist; decide S22 explicitly and justify it; include the #175 mutants and the witnesses of the criterion; give surfaces, manifest, permissions, budget and coordination (stop 450,000, ceiling 560,000, forecast gate, correction); correct the stale README and ADR 0017 references of U7/#449 in the same PR; do not weaken anything and report a blocker with evidence; do not execute; update #461. If any of this is wrong, the Owner corrects it at approval.
Not decided here: any change to the Pekko-side scenarios, the Pekko mutation set, `calibrate.py`, or a Go sink-to-event mapping.

## 12. Decisions this package asks the Owner to accept
| Id | Decision | Alternative and trade-off |
|---|---|---|
| D1 | emit `pre-restart` and `post-restart` from a custom hook that replicates Pekko's default `preRestart`/`postRestart`; per-name instance counter | emit nothing: `G7.b` and `I6.b` become vacuous |
| D2 | failing first start through a node flag | a separate failing actor type: more code, same behavior |
| D3 | `judgeNode` can be a supervisor whose decider emits `decided`; zero `Strategy` otherwise | a dedicated boss type: duplicates the node |
| D4 | S16's watchers use a small actor type with a comparable message | make `judgeMsg` comparable for all scenarios: touches every scenario; or skip `I6.b` in Go: weakens the contract |
| D5 | stronger completeness witness for S13 (a message to `w` after `R`) | the Pekko witness only: a late `Terminated` could be missed |
| D6 | S22 is not in the Go gate (section 7), recorded in docs and #461 | include it with a custom hook: passes on pre-U7 code, no directed mutant can detect it |
| D7 | the implementer template allows exactly one scoped judge command, `bash judge/run-go.sh /tmp/gopekko-u8-judge-out 1` (one concurrent run, `-race`), so it can see the three scenarios run; `go test` stays denied otherwise | no run: the implementer writes concurrent harness code blind (U7's T4 was found flaky only by the coordinator) |
| D8 | four Go mutants, one per clause; the other Pekko mutations stay a gap | all eight analogs: more anchors, more campaign time |
