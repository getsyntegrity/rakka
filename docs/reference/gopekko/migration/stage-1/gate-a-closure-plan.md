# Gate A: verdict, causes of the open gaps, and the first Go↔Pekko comparison

Status (current, see `gate-a-calibration.md` sections 10-12): **detection complete, calibration measured on macOS and on GitHub
Actions; no technical blocker found; Gate A is proposed for human approval and is not approved.** The text below this line is the
state at the time each section was written (`2d4bceb`, 26 of 28), kept as history; where it says 'pending', section 8 and the
calibration document supersede it.

Original status line: **Gate A is pending: detection complete, calibration pending.** Sections 1-7 were written on `origin/develop` at
`2d4bceb` (26/28 observed); section 1 shows the state after the stacked judge PR (28/28). Pekko pin `9a54d1294c`. Green CI is
not approval: `ci` proves the self-tests and the checker pass, not that a calibration exists.

## 1. What approval needs, and what exists

Contract: `docs/judge.md` (sections 3, 5, 6). Gate A needs every G/I clause observed on Pekko by a designed mutation in a
valid campaign, and a justified concurrent run count `N` (`judge/calibration.tsv`).

| Fact (kept apart, as `docs/judge.md` section 6 requires) | Value | Evidence |
|---|---|---|
| Designed coverage | 35 / 35 relations | `python3 -I scripts/check-judge-matrix.py --clauses` |
| Observed detection on Pekko | 28 / 28 clauses (was 26 / 28 before `S17`, `M37`, `M60`) | same, over `judge/mutation-report.tsv` (54 rows) |
| Pending clauses | none | same |
| Execution reliability | 8 of 54 rows are `timeout`, no clause depends only on them | `migration/stage-1/judge-partial-delivery.md` section 4 |
| Concurrent calibration | all rows `pending` when written; now 14 measured: Linux in `judge/calibration.tsv`, macOS in `judge/calibration.macos.tsv` | `gate-a-calibration.md` |
| goPekko-side observation (parity Go↔Pekko) | 0 relations | no Go harness yet |

The three facts in the first rows and the last two are different questions and none stands in for another. **Does the
contract make Go observation a condition of Gate A?** `docs/judge.md` sections 5 and 6 define Gate A over the Pekko side
(clauses observed, calibration); goPekko-side mutations are 'applied once the runtime exists' (section 6, item 3), and
the partial-delivery report calls Gate A 'open' without naming Go observation as a condition. So Go's 0 relations is a
later milestone, not a reason to reject Gate A. What keeps it pending is the calibration.
Self-tests after the stacked PR: `judge/selftest.py` 131, `judge/selftest_runner.py` 95, `judge/selftest_backfill.py` 56 checks, 0 failed.

## 2. Cause of each gap

| Gap | Cause class | Demonstrated cause | Fix |
|---|---|---|---|
| Campaign not reproducible from the repo | **Execution** | `judge/run-pekko.sh` runs `sbt judge/runMain`, but no file in the repo defined the `judge` sbt project. A fresh Pekko copy fails with `not found: value actor` (a naive project), then with `pekko-actor_2.12 not found` (default Scala version). | **Fixed in #24**: `judge/pekko-harness/judge.sbt`, installed by the script. |
| `G8.a` (the ask result is the first reply) | **Scenario and mutation** | `S03` has the target send **one** reply (`Judge.scala`, `"S03"`), so a second reply never exists to be ignored. `M37` was marked unimplementable because `PromiseActorRef.!` uses `result.tryComplete`, which is first-wins (`AskSupport.scala`, `!` method). The mutation must therefore change what the *asker observes*, not the promise. | **Fixed in the stacked PR**: `S17` and `M37`. Section 8. |
| `I3.b` (never both received and dead-lettered, never twice) | **Mutation** | `r_I3b` in `judge/comparator.py` fires only on a `dead-letter` event for a message also `received`, or two dead letters for one message. No designed mutation produces either (`judge-partial-delivery.md` section 6.1). Scenarios `S10` and `S11` already assert it. | **Fixed in the stacked PR**: `M60`. Section 8. |
| Calibration | **Execution/calibration** | Not run. It needs mutant campaigns (200 runs each, per scenario and side). | Method in section 4. |

No assertion, exclusion or criterion was relaxed. The comparator and the report are unchanged.

## 3. Acceptance criteria of the next judge PR (closes `G8.a` and `I3.b`)

Each item is a clause that must be observed from a valid campaign, never typed in a document.

1. **`G8.a`**: a scenario (new `S17`, so the existing `S03` evidence stays valid) where the asked actor replies `pong`
   and then a second reply; rule `G8.a` expects `pong`. `M37` is implemented as a patch of the pinned Pekko that makes the
   later reply the value the asker observes. It needs: baseline passes (controlled and concurrent), `M37` fails with
   `G8.a` in a campaign where every requested run is judgeable, and the matrix, `mutate.py` and `milestone-1.md` rows are
   updated together. The second reply is dead-lettered (I5) and must be an allowed event of `S17`.
2. **`I3.b`**: a mutation (new `M60`) that lets a queued message be received and also dead-lettered, or publishes it
   twice, expected to break `I3.b`; same criteria. `M7` stays amended to `G5.b`.
3. Evidence kind: a campaign run directly by `mutate.py run` is `run_evidence=executed`, which the checker accepts when
   `evaluation=run`. `judge/backfill.tsv` proves the execution of **earlier logs that are re-evaluated**; it is required only
   for `evaluation=reevaluation` rows (`scripts/check-judge-matrix.py`, 'a re-evaluation cannot be `executed`'). The
   contract does not require a ledger entry for a direct run.

## 4. Calibration method (run in the calibration PR: `gate-a-calibration.md`)

From `docs/judge.md` section 5, restated with the assumptions:

- Per concurrent scenario and side, run one intermittent mutant `n = 200` times and count detections `d`.
- `q_low` is the one-sided 95% Clopper-Pearson lower bound of the detection probability; `N = max(300, ceil(ln 0.01 / ln(1 - q_low)))`.
- Assumptions: runs are independent; the bound holds for that mutant class only; it depends on CPU count, recorder and
  runtime, so it must be redone when any of them changes. `d = 0` is a harness error. PRs run `min(N, 5000)`.
- Cost: measured in the calibration PR, not estimated: 2200 runs, under two minutes per campaign on 16 CPUs.
- Result (`gate-a-calibration.md`): 8 of 13 rows measured, 5 pending. `S07` cannot be calibrated: 0 detections in 200 fresh JVMs
  (a harness error by the contract) and, in one JVM, a detection rate that depends on the run position. `S08`, `S11`, `S14`
  end on the event their mutant removes (completion witness); `S03` needs a coverage decision. Gate A stays pending.
- Update (harness-witness PR): all 14 concurrent scenarios are now measured with one harness (`gate-a-calibration.md` section 10),
  `S07` through `M8` (`I1.a`) and the new `S18` through `M9` (`I1.b`); `S08`, `S11`, `S14` got completion witnesses and `S03`
  keeps its own mutant (`M61`). Every `N` is the contract's floor. This does not approve Gate A: the independence assumption is
  checked in one way only, the evidence is on one machine, and the approval is the owner's decision.
- Go calibration: **not a condition of Gate A**. This plan defines the gate over the Pekko side (section 1), `docs/judge.md`
  applies goPekko-side mutations once the runtime exists, and the file is per side; the Go calibration belongs to the later
  acceptance of the Go runtime. The Pekko-side calibration above is measured; the approval is recorded in
  `gate-a-calibration.md` section 13.

## 5. First Go↔Pekko slice

Mapped from the current Go code (`pekko-actor/actor`), not from the plan. Go exposes `NewSystem`, `Spawn`, `Ref.Tell`,
`System.Stop`, `System.Done`, `Context.StopSelf`, `SubscribeDeadLetters` and the optional `PreStart`/`PostStop` hooks.

| Behavior | Observable contract | Pekko scenario | Go entry point | Go evidence today | Gap |
|---|---|---|---|---|---|
| Spawn and refs | a spawned actor receives messages; a reused name gets a new identity | S01 (partly) | `Spawn`, `Ref.Tell` | `behavior_test.go:117`, `stop_race_test.go:99` | no children, so `G2` is not expressible |
| FIFO, serial processing | one sender → one receiver in order; no overlapping `Receive` | S02 (`G1.a`), S01 (`G3`) | `Tell` | `behavior_test.go:22`, `:45` | no judge-vocabulary recorder |
| PreStart, PostStop | `pre-start` → `received`* → `post-stop`, once | S10 (flat part, `G3`, `G5.a`) | hooks | `lifecycle_internal_test.go:82`, `:321` | no judge-vocabulary recorder |
| Stop accepted vs completed | accepted: later messages not received; completed: after the in-flight `Receive` and `post-stop` | S10 (`G5.a`, `G5.b`) | `Stop`, `Done` | `lifecycle_internal_test.go:290`, `stop_race_test.go:75` | Pekko has no `Done`; completion is observed by watch, absent in Go |
| Pending and post-Stop messages | queued at stop → dead-lettered once; sent after stop → dead-lettered once | S10, S11 (`I3.a`, `I3.b`) | `Tell`, `SubscribeDeadLetters` | `deadletter_test.go:42`, `:70`, `stop_race_test.go:28` | `docs/judge.md` section 2 asks for a runtime `DeadLettersPublished` counter; section 9 evaluates whether `SubscribeDeadLetters` and `Done` are enough first |
| Dead-letter order limits | no order claim beyond FIFO per receiver | S10, S11 | same | `deadletter_test.go:42` | mapping `DeadLetter.To` (an ID) to the scenario name |

Expressible with the current API: **S02, S11**, and the flat parts of S01, S06 and S10. Everything else needs children,
watch, restart or ask, which the task excludes.

**Semantic differences found, not hidden** (read from the code, to be proved by the harness):

1. A `Receive` error or panic stops the actor in Go (`cell.go:173-176`); Pekko's default decider restarts (`G7`).
2. After `Stop` is accepted, a `Tell` is dead-lettered by the caller at once (`ref.go:33-37`). In Pekko the message is
   enqueued and either dead-lettered at cleanup or received; only the controlled-mode outcome is fixed by `I3`.
3. A stop accepted before the first activation runs neither `PreStart` nor `PostStop` in Go
   (`lifecycle_internal_test.go:135`); Pekko always runs `postStop` once the instance exists. Unverified for Pekko.
4. Go publishes dead letters synchronously on the publisher's goroutine; Pekko publishes through the event stream.

## 6. Next work, with acceptance criteria

1. **PR A, judge** (section 3): `S17` + `M37`, `M60`. Done: 28/28 observed (section 8).
2. **PR B, Go harness slice** (after PR A, or in parallel if its scope stays small): a Go recorder with the append-only,
   per-actor design of `docs/judge.md` section 2, running `S02` and `S11` on the Go core, reading the existing scenario
   JSON and the comparator. Whether it needs any core change is decided by section 9, not assumed.
3. **PR C, calibration** (section 4), run as its own campaign. Gate A cannot be approved before it.

## 7. Verified in this PR

A fresh disposable Pekko copy (`cp -R upstream/pekko`, not modified) built with the new `judge.sbt` and ran `S03` and
`S11` in controlled mode through `judge/run-pekko.sh`; `judge/evaluate.py --expect-runs 1` reports both as `complete`,
1 pass, 0 fail. This proves the harness is reproducible from the repo. It does not move any clause count.

## 8. Update: `G8.a` and `I3.b` (stacked judge PR)

`S17`, `M37` and `M60` close the two pending clauses: `check-judge-matrix.py --clauses` reports **28/28 clauses observed on
Pekko** (54 rows) and 26/26 implemented mutations detected by their expected rules. Rows: `M37/S17` and `M60/S10`, both
modes, run directly by `mutate.py run` (`run_evidence=executed`) at harness commit `3effc02`, Pekko pin `9a54d1294c`,
Java 11.0.32, 1 controlled run and 30 concurrent runs each. Baseline of `S17` on unmutated Pekko: controlled 1/1 pass,
concurrent 30/30 pass.

Known limits, stated rather than hidden:

- `M37` wraps the `Future` that the ask returns and changes only `value`. `ready` and `result` still report the first
  reply, and `S17` reads `value` after completion. It is a faithful violation of 'what the asker observes' for that read
  path, not for every way of reading the future.
- `M60` dead-letters the last message the actor received at termination. It proves `I3.b` can fire; it does not model how
  a real defect would double-publish.
- `S17` waits on the ask future and on a promise completed after both sends, each with a fixed 5 s bound inside a 10 s
  scenario deadline. A slower machine would surface as a harness error, never as a pass.
- The campaign logs are not in the repository. They are preserved outside it (see the PR that records them) with their
  SHA-256 sums, so that the two direct rows can be re-checked.

Gate A stays pending: detection is complete, the calibration is not.

## 9. Proposal: observing the Go core for S02 and S11 (no Go code in this PR)

Derived from the current API (`pekko-actor/actor`); the harness lives outside the core and uses only exported symbols.

- **Do not export `DeadLettersPublished` first.** The completeness witness exists without it: `Stop` is accepted at once,
  and `Done(h)` closes only after `PostStop`, the name release and the publication of every queued message
  (`stop.go:51-68`); a `Tell` after the stop is published synchronously on the caller's goroutine (`ref.go:33-37`).
  So once `<-Done` returned and every `Tell` returned, every dead letter of that actor has been delivered to the
  subscriber. The counter becomes necessary only to cross-check a subscriber against other publishers, which S02 and S11
  do not have. Export it later, with its own justification. This is a reading of `stop.go` and `ref.go`: the Go harness must
  prove it with a test that fails when a callback is still running after `Done`, before relying on it.
- **Identity.** The harness gives every message its own logical id (the payload type carries it) and maps `Handle.ID()`
  to the scenario name at spawn; `DeadLetter.To` is an `ID`, so it never needs a runtime name.
- **Recorder.** Append-only, per-actor buffers as in `docs/judge.md` section 2; the dead-letter subscriber is registered
  before the first `Tell`. Subscriber callbacks run on the publisher's goroutine, so several can run concurrently: the
  subscriber appends to its own buffer and takes no lock shared with actors.
- **Real limits.** `Stop` returning is acceptance, not completion. A message sent after acceptance is dead-lettered by its
  sender and never enters the queue (Pekko may still receive it in a race; Go decides at `Tell`). Pending messages are
  published in FIFO order after `PostStop`. None of this is hidden by the recorder.
- **Scope.** `S11` (controlled-style, single sender) and `S02` (two sender goroutines, concurrent only). `S10` needs
  children for `G5.c/d`; only its flat part (`G3`, `G5.a`, `G5.b`, `I3`) is expressible and must be labelled as a Go variant,
  not as `S10`.
- **Acceptance.** Scenarios are read from `judge/scenarios/*.json` and judged by `judge/comparator.py`, not re-implemented;
  baseline passes; at least one Go-side mutation per asserted clause fails with that clause; runs use `go test -race`;
  an invalid run is a harness error, never a pass; no sleep proves order or completion.
