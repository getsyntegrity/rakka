# S22 in the Go gate: proposal for owner approval

Status: **APPROVED by the Owner for sections 3-5 (third brief, candidate `fdba453`), with conditions: the event comes from the really executed `defaultPreRestart`; the harness observes and never assumes or rebuilds calls; the contract 1/1/0/2 stays; GMdefaultps is detected by `G5.f` (Addendum 2: `G5.c` and `REQ` may also fire as consequences); the event must not alter order, decisions or behavior; implementation through the kit package (`RULEBOOK.md` unit S); any new difference from Pekko returns to the Owner. Not implemented yet: it waits for the approval of `BUDGET.md`.** Original status: PROPOSAL, not implemented. It changes the judge contract (`docs/judge.md`, `judge/go-scope.json`, `judge/mutate_go.py`) and adds one internal runtime event, so it needs approval before any code is written. Source of truth: Apache Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`).

## 1. Behavior derived from Pekko (default hooks, Stop while a restart waits for its children)

| Step | Pekko source | Effect on the old instance |
|---|---|---|
| a. failure, Restart decided | `FaultHandling.scala:92-114` `faultRecreate` | `aroundPreRestart` runs unless failed fatally |
| b. default `preRestart` | `Actor.scala:625-631` | unwatch and stop every child, then **`postStop()` (call 1)** |
| c. children still alive | `Children.scala:183-189` `setChildrenTerminationReason(Recreation)` | restart suspended, waiting |
| d. Stop arrives | `FaultHandling.scala:180-212` `terminate()` | reason replaced by `Termination`; no new instance will be built |
| e. last child removed | `FaultHandling.scala:327-346` `handleChildTerminated` | `finishTerminate` instead of `finishRecreate` |
| f. `finishTerminate` | `FaultHandling.scala:~230-262` | **`aroundPostStop` (call 2)** on the same old instance, then `Terminated` to watchers once |

Result: **two `PostStop` calls on the old instance, in the order b then f**, one `Terminated`, no `PostRestart`, no new instance (`pre-start` stays 1). This is the Pekko behavior (ADR 0017 S11, `h4-repro/RESULTS.md`, `judge/scenarios/S22.json`: `post-stop` of `c` with count 2), not a defect.

Number and order in Go today (U7, `supervision_restart.go:346-358`): `defaultPreRestart` calls `PostStop` without the termination guard, so the old instance sees two calls in the same order. Tests: W11b, W25, T1 (order), T2 (custom `PreRestart` without `PostStop`: one call), T3 (plain Stop: one call).

### Wording correction
"The double `PostStop` existed before U7" is wrong. **Before U7 Go ran ONE `PostStop`** here (the default `PreRestart` went through `beginPostStop`, which made termination skip its own call: `git show b3275d7 -- pekko-actor/actor/supervision_restart.go`). The **double call is Pekko's** and Go has it since U7. The text of `docs/judge.md:292-297` is accurate (it speaks of a *custom* hook, which was unguarded both before and after U7) and needs no correction; the error was only in a chat summary.

## 2. Why S22 is not in the Go gate today
`pre-restart` can only be recorded by a custom `PreRestart`; with a custom hook the second call exists with or without U7, so no directed mutant of U7 is detectable. The default path, which U7 changed, emits nothing the harness can see (`docs/judge.md:295`).

## 3. Proposed observation (real path, not scripted)
Add **one** event to the existing unexported diagnostics sink (`options{sink: ...}` / `System.emit`, `system.go:141`), already used by internal tests and not part of the public API:

```go
// defaultPreRestart, first statement
c.sys.emit("prerestart-default", c.cid, nil)
```

- It is emitted **by the runtime, inside the default hook**, and only when the restart pipeline actually reached `Actor.preRestart`'s default. The harness never emits it and never assumes the outcome.
- The Go harness installs a sink and maps `prerestart-default` of an actor to the judge event `pre-restart` (`{a, i}`) **only for nodes with no `PreRestart` method** (no double counting with the custom-hook path, which keeps recording from the hook).
- `post-stop`, `pre-start`, `terminated` keep coming from the node's own `PostStop`/`PreStart` and the watcher's `Terminated`, i.e. from calls made by the runtime.
- Risk to verify first: tests that assert the exact event list of the sink (`rg "acceptedBy|sup1Count"`) must stay unchanged; the event adds a new kind and no existing kind changes.

## 4. Exact contract change requested
1. `pekko-actor/actor/supervision_restart.go`: the one-line emit above (production, +1 line).
2. `pekko-actor/actor/judge_internal_test.go`: a sink adapter; a `judgeDefaultNode` (no `PreRestart`/`PostRestart`, records `pre-start`, `post-stop`); an S22 builder: parent `c` with children `child-1`, `child-2`, watcher `w`, a held child so the Stop arrives during the wait, in `concurrent` mode.
3. `judge/go-scope.json`: add `"S22"` to `scenarios`. No waiver. `judge/scenarios/S22.json` is **unchanged** (same required events, rules G5.f with counts pre-start 1, pre-restart 1, post-restart 0, post-stop 2, I2.a, G5.c).
4. `judge/mutate_go.py`: directed mutation **GMdefaultps** (see §5), analog = the U7 change. Added to `MUTATIONS` so the gate must detect it.
5. `docs/judge.md`: replace the "S22 is Pekko only" paragraph by the S22 Go row, keep the explanation of why it needed the event; update `judge/selftest_go_scope.py`, `judge/selftest_mutate_go.py`, `scripts/check-judge-matrix.py` expectations (14 to 15 scenarios, 17 to 18 designed mutations). `migration/stage-1/milestone-1.md` registration.
6. The Go campaign becomes 18 mutations and its CURRENT evidence must be re-run after the change.

## 5. Directed mutant GMdefaultps (derived from the guarantee, no weakening)
Guarantee: with default hooks, Stop during the restart wait gives **exactly two** `PostStop` on the old instance (G5.f `post-stop` = 2).
Mutation: restore the pre-U7 body of `defaultPreRestart`:

```go
// before (current)
if stopper, isStopper := c.actor.(PostStopper[T]); isStopper {
    c.postStopOfRestart(stopper)
}
// after (mutant = pre-U7)
if c.beginPostStop() {
    if stopper, isStopper := c.actor.(PostStopper[T]); isStopper {
        c.postStopOfRestart(stopper)
    }
}
```

Expected: the old instance sees **one** `PostStop`, so G5.f fails (`post-stop` 1 instead of 2). Corrected by Addendum 2: `G5.c` and `REQ` also fire as consequences when the second `PostStop` is missing (measured in unit S22); they are accepted, the required detection is `G5.f` by the observed count. The anchor must occur exactly once. Timeout, compile error and harness-error are INVALID, never a detection. S22 itself is not relaxed: counts stay 1/1/0/2.

## 6. Differences with Pekko
None in semantics. Adaptation (Scala/JVM to Go): Pekko's harness records `pre-restart` by overriding `preRestart`; Go's default hook is runtime code, so the event comes from the runtime sink instead. Observable difference: none at the judge level (the same `pre-restart` event, same count).

## 7. Decision requested
Approve or reject sections 3-5 as written. If approved, implementation follows the kit process with its own RULEBOOK, RUNBOOK, manifest and a reviewer separate from the writer.
