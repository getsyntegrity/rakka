# Exclusion of two S22 cells from the Go mutation matrix (Owner option C)

Status: justification recorded **before** any change of `judge/` (condition 1). Authorization: sixth Owner brief, option C, only these two cells: **GM2 x S22 x concurrent** and **GMstop x S22 x concurrent**.

## What is excluded and what is not
- Excluded as "not applicable": the S22 `concurrent` cell under GM2, and the S22 `concurrent` cell under GMstop. Nothing else.
- Kept: every designed detection of GM2 (S10 `G5.a`, `G5.d`) and of GMstop (S19 `G5.b`); every other cell of both mutations; the S22 cell under every other mutation, including GMdefaultps (required `G5.f`); S22 on the unmutated tree (must pass, contract 1/1/0/2).
- Not changed: S22 (`judge/scenarios/S22.json`, the Go harness S22), the comparator, the classification of `harness-error`. The excluded cells are shown in the report as excluded/not applicable, never as passed, survived or detected.
- Campaign 3 stays whole, with its two INVALID rows (`EXECUTION-RECORD.md` Update 7; report sha256 `8e18e0c6df4f20b69a4d8a5509066e976bad92dd1c04f0d9cc87446467debabd`).

## Why: the scenario's precondition is not reached under these two mutations
S22 judges a Stop that arrives while a restart waits for held children. The harness (`awaitS22Wait`, `judge_internal_test.go:954`) waits until c ran its first `PostStop` (P1) and both children began theirs (`post-stop-begin`); otherwise it records `gate-timeout`, which is outside the vocabulary, so the evaluator reports `harness-error`. Pekko's harness has the same guard (`Judge.scala:478`). When the precondition is not reached, S22 has no restart-waiting state to judge.

Evidence, read from the kept logs of campaign 3 (`/Users/pablogore/.claude/jobs/b989ecff/tmp/camp3/work/<mutation>/logs/S22.concurrent.r*.jsonl`, 30 runs each; counting script `/Users/pablogore/.claude/jobs/b989ecff/tmp/precond.sh`, a precondition counts as reached when c has at least one `post-stop` and both children have a `post-stop-begin`):

| Mutation | Runs | Precondition reached | `gate-timeout` |
|---|---|---|---|
| GM2 | 30 | **0** | **30** |
| GMstop | 30 | **0** | **30** |
| GMdefaultps (control) | 30 | 30 | 0 |

- GM2 (`skip PostStop`, `stop.go`: `PostStopper[T]); ok && false {`): the `PostStop` of the termination path never runs, so the held children never record `post-stop-begin`. In the GM2 logs c's P1 is present (the default `PreRestart` path is another code path) and the children show only `pre-start`.
- GMstop (`an external Stop behaves like a poison pill`, `stop.go`, `cell.go`): in the kept logs c's P1 is present and the children show only `pre-start`; they never begin `PostStop`. This is consistent with the mutation deferring the stop of the children behind queued work. The inner mechanism inside the mutated copy was not traced further; the exclusion rests on the measured 0 of 30, not on that explanation.
- The control shows the guard is not a general flake: under GMdefaultps the precondition was reached in 30 of 30 runs with no `gate-timeout`.

## Addendum after the independent review (mechanism traced; rule tightened)
The review of the final candidate traced the GMstop mechanism from the code of the mutated copy and the kept log: when c fails its children are suspended; the default `PreRestart` stops them; under GMstop that sets `extStop`, not `stopRequested`; the mutant applies `extStop` only in the idle branch of `drain`, which comes after `blockedLocked`, so a suspended child returns before any stop takes effect and never begins `PostStop`; unmutated, `stopRequested` is checked first. The exclusion is therefore justified in substance. It is **not** a statement that the mutation is harmless: GMstop creates a visible liveness defect (a Stop to a suspended actor never takes effect, which Pekko does not do: system messages are processed regardless of the suspension, `dispatch/Mailbox.scala:223-233` and `:269`); S22 can report it only as a hang. GMstop is detected by its designed clause (S19 `G5.b`). GM2 is direct: the mutant skips `PostStop` in the termination path, so the children never begin it.

The review also found that the first version of the exclusion decided from the evaluator's message of the first non-passing run only, and could not tell the guard `gate-timeout` from the gate of the children's held `PostStop` (same event name). The rule is now: the cell is excluded only if **every** run is a harness error whose own message is `gate-timeout` and whose log shows that c's first `PostStop` and both children's `post-stop-begin` did not all happen. Mixed results (a different error, a pass, a failure, a timeout, or the other gate) in any run leave the cell as it is. Regressions: `judge/selftest_mutate_go.py` section 14 (RED before the correction: four mixed cases came out `excluded`; GREEN after). For campaign 4, the 30 runs of each excluded cell were counted in the kept logs: precondition reached 0 of 30, `gate-timeout` 30 of 30, for GM2 and for GMstop (control GMdefaultps 30 of 30 reached, 0 timeouts).

## Why this is not a weakening
The two mutations have designed detections elsewhere (GM2: `G5.a`/`G5.d` at S10; GMstop: `G5.b` at S19) and those detections hit in campaign 3. S22 was never designed to detect them: its directed mutant is GMdefaultps. The excluded cell cannot detect or miss anything because the scenario cannot run its own scenario under those mutants. The exclusion removes an INVALID that carries no information; it does not turn any detection or survival into something else. This accredits only the explicit matrix that declares the two exclusions.

## Implementation constraints (checked by selftests)
1. The exclusion list has exactly two entries, `(GM2, S22, concurrent)` and `(GMstop, S22, concurrent)`; a selftest fails if an entry is added, a mutation, scenario or mode differs, or a wildcard is used.
2. An excluded cell must be run and observed as the precondition failure before it is reported excluded: it is excluded only when its run is exactly the INVALID of this document (`harness-error` caused by `gate-timeout`); any other outcome in an excluded cell (a pass, a failure, a different harness error) is reported as such and the mutation is not met.
3. The report shows the cell as `excluded`, with the reason, in its own outcome; the counts of failed, survived, harness-error stay separate; the decision of `met` for a mutation does not treat `excluded` as passed or detected.
4. S22 unmutated must pass; GMdefaultps must fire `G5.f`.
