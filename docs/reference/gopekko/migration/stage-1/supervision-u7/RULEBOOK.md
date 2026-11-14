# RULEBOOK: Supervision U7, Stop during a restart and the scope of G4/G5 (#449)

Status: PROPOSED, for the Owner's approval together with the package (`RUNBOOK.md`). Unit U7 of ADR 0017 (S6, S11, H4, D-5). Base: `develop` at `e0d70bbb85d42f41d9fb6a786de001f6afe02389` (U6 merged, #447 closed; verified with `git rev-parse origin/develop` on 2026-10-16). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`; upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`). Paths are relative to `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/` unless they start with `judge/`, `docs/`, `migration/`, `pekko-actor/` or `scripts/`.

Nothing here reopens a decision already accepted in ADR 0017 (H4: the observed behavior is the target, accepted by the Owner on 2026-10-14). A row marked **NEW** is a decision this unit asks the Owner to accept. The judge's scenario "S11" and the ADR's "S11" are different things (section 6.4). No parity is claimed.

## 1. Sources (read at the pinned commit, not run)
| Fact | Evidence |
|---|---|
| The default `preRestart` stops every child (unwatching it first) and then calls `postStop()` on the old instance. It is user-level code; `aroundPreRestart` is only `preRestart(...)` | `Actor.scala:625-631`, `:574` |
| `faultRecreate`: `aroundPreRestart` runs on the failed actor (skipped if failed fatally), `clearActorFields(actor, recreate = true)` keeps the old actor for later, then `if (!setChildrenTerminationReason(Recreation(cause))) finishRecreate(cause)`; with live children the first is true, so the cell waits | `dungeon/FaultHandling.scala:92-118` (call at `:104`, wait at `:114`) |
| A `Terminate` system message calls `terminate()`; with children still alive `setChildrenTerminationReason(Termination)` returns true and **overwrites** `Recreation(cause)` with `Termination` (`c.copy(reason = reason)`), so the restart can no longer complete | `ActorCell.scala:529`; `dungeon/FaultHandling.scala:180-212` (`:198` `wasTerminating`, `:200` the overwrite); `dungeon/Children.scala:183-189` |
| When the last child terminates, `handleChildTerminated` reads the state change: `Some(Termination) => finishTerminate()`; the `Recreation(cause) => finishRecreate(cause)` branch is never taken | `dungeon/FaultHandling.scala:327-351` (`:345-348`) |
| `finishTerminate` calls `a.aroundPostStop()` (the second `postStop`), detaches, sends `DeathWatchNotification` to the parent and then tells the watchers (the single `Terminated`) | `dungeon/FaultHandling.scala:247-276` |
| There is no guard against a second `postStop`; `Terminated` is sent once because `finishTerminate` runs once | same lines |
| Observed behavior, 3 runs per mode, Pekko v1.7.1, sbt 1.12.14, Java 11.0.32, 2026-10-14. Setup: parent `P` restarts `C`; `C` has children `g1`, `g2` whose `postStop` blocks on a latch; a watcher `W` watches `C`; the driver waits until `C.preRestart` returned and both children started `postStop`, then `context.stop(C)`, then releases the latch | `migration/stage-1/supervision/h4-repro/RESULTS.md:22-105`, `H4Repro.scala:44-52,98-139` |
| Docs of Pekko cited by G5 describe the termination order and do not say "once per instance" | `upstream/pekko/docs/src/main/paradox/actors.md:941-957` |
| ADR 0017: H4 `:68`; S6 `:294-311` (the accepted correction of G4/G5, `:305-308`); S11 `:443-476`; D-5 `:625`; U2 gap `:681`; U7 `:686`; mirror `:726` | `docs/adr/0017-supervision.md` |
| Go today: `postStopBegun` is a per-cell flag (`cell.go:39`); `beginPostStop` (`stop.go:113-124`) returns true once; `finalize` (`stop.go:92-97`) and `defaultPreRestart` (`supervision_restart.go:348-358`) both go through it; `installNew` re-arms it (`:419`) | Go source |
| `c.actor` is replaced only in `installNew` (`supervision_restart.go:418`), so during the wait it is still the old instance and `finalize` reaches it | Go source |
| Pinned interim: `TestSup2PostStopGuard`, second subtest, asserts `[p.pre#1, p.post#1]` after a Stop during the wait; `TestSup2CustomPostStopTwice` (W25) already pins two calls for a custom `PreRestart` | `pekko-actor/actor/supervision_restart_goside_internal_test.go:181-228` |

## 2. The two PostStop calls and Terminated, kept apart
| Item | Who calls it | When | On which instance | Guard in Pekko | Guard in Go today | Go after U7 |
|---|---|---|---|---|---|---|
| **P1, PostStop of the PreRestart** | the user-level `PreRestart`; the **default** one calls it right after stopping the children | at restart time, **before** the children have finished, before any new instance | the old instance | none | `beginPostStop` (consumes the per-cell flag) | none: a direct call, as for a custom hook |
| **P2, PostStop of termination** | the runtime (`finishTerminate`; Go `finalize`) | once, **after every child has finished** | the **current** instance: the old one if the restart was abandoned, the new one otherwise | none needed (runs once) | `beginPostStop` | `beginPostStop`, kept only against the re-entry after a Goexit inside P2 |
| **Terminated** | the runtime, from the termination path | once, after P2, before `Done` closes | to the watchers of the actor | none | none | unchanged; a restart delivers none |
P1 belongs to the restart sequence (G4). P2 and `Terminated` belong to termination (G5). A restart that is abandoned by a Stop still ran P1; it never runs P2 for a new instance because there is none.

## 3. Observed counts and the Go gap
| Mode (Pekko, 3 runs each) | `preRestart` | `postRestart` | new instance | `postStop` of the old instance | `Terminated` |
|---|---|---|---|---|---|
| default hooks, Stop during the wait | 1 | 0 | none | **2** (P1 then P2) | 1 |
| `preRestart` stops the children and does not call `postStop` | 1 | 0 | none | 1 (P2) | 1 |
| plain Stop, no restart | 0 | 0 | n/a | 1 (P2) | 1 |
Order with default hooks (run 1): `preRestart` starts; `g1.postStop-begin`; **`C.postStop` (P1)**; `preRestart` returns; `g2.postStop-begin`; the driver requests the Stop; the latch is released; `g1`, `g2` `postStop-end`; **`C.postStop` (P2)**; `W.Terminated`. Only the begin order of `g1` and `g2` varied between runs.
**Go gap today:** default hooks give 1 `postStop` (the per-cell guard), where Pekko gives 2. The other two modes already match.

## 4. Contract
| Rule | Behavior | Evidence |
|---|---|---|
| R1 Termination path | the runtime calls `PostStop` **once**, at termination, on the current instance, after every child finished and before `Terminated`. The guard `beginPostStop` stays only to stop `finalize` from re-running `PostStop` after a Goexit inside it (`callPostStop`) | `dungeon/FaultHandling.scala:247-276`; Go `stop.go:92-142` |
| R2 Default `PreRestart` | stops (and unwatches) every child, then calls `PostStop` on the old instance **directly**, with no use of the termination guard, and **without waiting** for the children. A custom `PreRestart` is user code and is not guarded (W25 stays) | `Actor.scala:625-631`; D21 of U2 |
| R3 Stop during the wait | the restart is **abandoned**: no `continueRestart`, no `installNew`, no new factory call, no `PostRestart`, no second `PreStart`. The stop proceeds as a termination: children finish, P2 runs on the old instance, `Terminated` once, `Done` closes | `dungeon/FaultHandling.scala:198-206,345-348` |
| R4 Counts | default hooks: `PostStop` 2 on the old instance (P1, P2), `Terminated` 1; hooks that do not call `PostStop`: 1; no restart: 1 | section 3 |
| R5 Order | P1 may run while children are still stopping; P2 runs only after every child's final `PostStop`; `Terminated` after P2; `Done` after `Terminated`; P1 before P2 | section 3 order; Go `stop.go:92-111` |
| R6 Restart that completes | unchanged: P1 on the old instance, wait, `installNew`, `PostRestart` (default `PreStart`), and the new instance's own P2 once at its termination | W11 first subtest |
| R7 Failures of P1 | unchanged: a panic is `prerestart-failed` and the restart goes on; a Goexit in P1 is reported as `poststop-panic`, the rest of `faultRecreate` is done by the handler and a fresh activation takes the restart on (`postStopOfRestart`). **Go-specific consequence (NEW, D4):** if a Stop then arrives, P2 runs too, so the old instance saw P1 (ended by Goexit) and P2 | `supervision_restart.go:360-382` |
| R8 Watchers | a restart delivers no `Terminated`; the abandoned restart still delivers exactly one, from the termination path | `RESULTS.md` |
| R9 Hooks of the parent, mailbox, Watch | unchanged. Messages queued for the abandoned instance are dead-lettered by `finalize` as for any stop | Go `stop.go:98-107` |

## 5. The Go change (exact, minimal)
In `supervision_restart.go`: `defaultPreRestart` stops using `beginPostStop`. After the loop that unwatches and stops the children it calls `postStopOfRestart(stopper)` whenever the old instance is a `PostStopper[T]`, with no guard. `postStopOfRestart` and its Goexit handler are unchanged. The comment of `defaultPreRestart` and of `postStopOfRestart` drop "through the per-cell guard" and say that P1 is user-level and unguarded, citing `Actor.scala:625-631`.
Nothing else changes in production: `stop.go`, `cell.go`, `hierarchy.go` and `guardian.go` stay byte-identical (they carry judge anchors). `installNew` keeps its `c.postStopBegun = false` (harmless: after this change nothing but `finalize` sets the flag, and a cell finalizes once); removing it is **not** part of this unit.
Expected size: under 15 production lines changed, mostly comments.

## 6. Judge and documentation
### 6.1 Text of G4 and G5 in `docs/judge.md` (exact replacement, the Owner-accepted correction of ADR S6)
- **G4:** `Restart that completes: pre-restart(old) → post-stop(old) (the default pre-restart stops the children and calls post-stop without waiting for them) → post-restart(new) → pre-start(new) (the default post-restart). A stop requested while the restart waits for the children abandons it: no new instance, no post-restart, no second pre-start (clause G5.f, S22). The failing message is not received again; queued messages are kept and received by the new instance.`
- **G5:** replace `post-stop runs once per instance` with `the runtime calls post-stop once, at termination, on the current instance (the default pre-restart also calls it on the old instance, so an instance whose restart was abandoned by a stop sees two calls; clause G5.f, S22)`. "every child's final post-stop happens before the parent's final post-stop" is kept and says **final** explicitly. Add: `Clause G5.f: a stop requested while a restart waits for the children gives exactly the hook counts of that case (S22).` Clause G5.d keeps its meaning for every scenario that has no such stop. Sources: keep `docs/actors.md:923-957`; add `dungeon/FaultHandling.scala:180-212,247-276,327-351` and `Actor.scala:625-631`; note that "once per instance" was the judge's wording, not Pekko's.

### 6.2 Rules of `judge/comparator.py` (NEW D3; one new clause, G5.f)
What the comparator can and cannot say today (read at `e0d70bb`):
- `required` entries are **minimums** (`comparator.py:476`, `len(select(...)) < req.get("count", 1)`), and the spec check rejects a `count` below 1 (`:100`). So `required` cannot say "zero" or "exactly N". A scenario that relied on it would pass with a second instance, a `post-restart` or a third `post-stop`.
- `G4.a` (`:221-232`) **requires** `post-restart` and `pre-start(new)`, so it would fail the accepted behavior: S22 does not use it. `G4.c` is about received messages per instance. `G5.d` (`:276-283`) caps `post-stop` at the number of `pre-start`s, so it also fails the accepted behavior: S22 does not use it, **and it is not edited** (it keeps judging every other scenario as today).
- `G5.c` (`:264-273`) compares the **first** `post-stop-begin` of the parent (`pb[0]`) with the **first** `post-stop` of each child (`ce[0]`). With the default hooks the first parent `post-stop-begin` is P1, before the children end, so G5.c would reject the accepted behavior.
Changes:
1. **`r_G5c` compares final with final** (NEW D3b): the parent's **last** `post-stop-begin` (P2 in S22) with the **last** `post-stop` of each child (`ce[-1]`). G5 says "every child's final `post-stop` happens before the parent's final `post-stop`"; `ce[0]` does not check that for a child that stops more than once, and `pb[0]` is wrong when the parent has run P1. For a parent and children with one `post-stop` each (every scenario without a restart, including S10) the result is identical; with a restarted child it is stricter, never weaker. *The instruction that reached this package was garbled at the sentence about `ce[0]`; this reading follows the text of G5 and is a one-line revert if the Owner meant `ce[0]`. It makes no difference to S22, whose children stop once.*
2. **New clause `G5.f`** (exact hook counts of one actor): `{"id": "G5.f", "actor": "C", "counts": {"pre-start": 1, "pre-restart": 1, "post-restart": 0, "post-stop": 2}}`. On a complete log (`done`) every listed event must occur **exactly** the stated number of times (zero is allowed); on a prefix it fails only when a count is already **exceeded**. Registered in `PARAMS` (`counts`: a non-empty object whose keys are hook events and whose values are integers >= 0) and in `RULES`. This is the only way to state the exact absences, since `required` cannot.
3. `Terminated` exactly once uses the **existing** clause `I2.a` (`r_I2a`, `:368-372`: more than one is a violation, none is a violation on a complete log).
No other rule changes. The acceptance conditions for the matrix are in section 8 (criterion 3).

### 6.3 Scenario S22 (Pekko side) and its spec
**Definition (exact):** with the default hooks, a Stop requested while the restart waits for the children gives, for the actor `C`: `pre-start` 1, `pre-restart` 1, `post-restart` 0, `post-stop` 2, and the watcher's `terminated(C)` 1; each of the two children of `C` has `post-stop` at least once; the final `post-stop-begin` of `C` comes after both children's last `post-stop`. The first `post-stop` of `C` comes before the children's end (P1 before, P2 after, `RESULTS.md` order).
`judge/pekko-harness/Judge.scala` gets scenario **S22**, the default-hooks mode of `H4Repro.scala` (`:44-52,98-139`): parent `P` supervises `C` with Restart; `C` has two children whose `post-stop` blocks on a latch; a watcher watches `C`; the driver triggers the failure, waits until `pre-restart` returned and both children began `post-stop`, requests the stop of `C`, then releases the latch. It uses events the harness already emits (`pre-start`, `pre-restart`, `post-restart`, `post-stop-begin`, `post-stop`, `terminated`; `Judge.scala:113-123`).
**Mode: `concurrent` only** (`Set("concurrent")`). In `controlled` mode the harness runs everything on the driver thread (`CallingThreadDispatcher`; `comparator.py:142-146` rejects other threads), so a child blocked in `post-stop` would block the driver before it could request the Stop or release the latch. The existing latch scenarios are concurrent-only for the same reason (S06 `Judge.scala:297`, S20 `:446`; S19 skips its gate when `e.controlled`, `:429-431`). The Go mirror tests use a controlled executor and are not affected.
`judge/scenarios/S22.json` (`"modes": ["concurrent"]`): `required` for presence (minimums): `C` `pre-start`, `pre-restart` and `post-stop` (count 2), the watcher's `terminated`, and `post-stop` of each child; rules `G5.f` (exact counts above), `I2.a` (watcher, subject `C`), `G5.c` (parent `C`, both children). No `G4.a`, no `G5.d`.
The scenario is registered in `migration/stage-1/milestone-1.md` (scenario table with mode `concurrent`, and the matrix rows of G4 and G5), because `scripts/check-judge-matrix.py` requires every scenario to assert a relation and every relation row to list its scenarios.
**Running it is the coordinator's job, not the implementer's** (`sbt` is denied to agents): a disposable `git archive` export of `upstream/pekko` as in `h4-repro/run.sh`, then `judge/run-pekko.sh <copy> <out> concurrent 10000 S22`, **3 runs**, judged with `judge/comparator.py` and compared with `RESULTS.md` (counts and order). The harness build (`judge.sbt`) is unchanged. If the environment cannot run it, the criterion is **not met** and the unit says so.

### 6.4 Numbering
The ADR's "S11" is the H4 reproduction (`RESULTS.md`); the judge's `S11` (`judge/scenarios/S11.json`) is a dead-letter scenario. The new judge scenario is **S22** (the existing ones end at S21). The ADR keeps calling the case S11/H4; `docs/judge.md` and the execution record say "S22 (ADR S11/H4)".

### 6.5 What stays out of the Go judge gate
`judge/go-scope.json` is **not** changed and `S22` is **not** added to the Go gate: the Go judge has no restart scenarios and no restart events yet (that is the Go gate for S13, S14 and S16, U8, #175). The "Go mirror" of #449 is therefore a set of Go unit tests with the event names and counts of `RESULTS.md` (NEW D1), plus the Pekko-side scenario. Judge calibration, `judge/mutate*.py` and the Go mutation reports are untouched.

## 7. Tests
Conventions of U2 RULEBOOK section 4: go-specs only, deterministic (the `sup2NewWait` controlled executor with a held child, `sup2Clock`; no sleeps as coordination), English, no `//nolint`, gocyclo at most 10 including literals, coverage at least 85%. The test-first policy applies, with the coordinator observing it because the implementer cannot run `go test`: the flipped W11b and T1 must be **RED** on the base production code (the same run is MU3) and **GREEN** with the change; the coordinator records both.
### 7.1 Go mirror (`supervision_restart_goside_internal_test.go` edit; new file `supervision_stoprestart_internal_test.go`)
| Id | Asserts | Kills |
|---|---|---|
| W11b (existing, flipped) | default hooks, Stop during the wait: log `[p.pre#1, p.post#1, p.post#1]`; `Done` not closed until the held child is released; closed after. The title loses "U7 interim, not parity"; the header comment is rewritten | MU3 |
| W11a (existing, unchanged) | a restart that completes: `[p.pre#1, p.post#1, p.pre#2]`, then after a Stop `p.post#2` | regression of R6 |
| T1 | mirror of `RESULTS.md` default mode with one watcher: P1 happens **before** the held child's final stop; P2 **after** it; exactly one `Terminated`, delivered after P2; factory called once (no second instance); no `PostRestart`; `Done` closes after `Terminated` | MU1, MU2, MU3 |
| T2 | custom `PreRestart` that stops the children and does not call `PostStop`: `PostStop` 1, `Terminated` 1, no `PostRestart` | regression |
| T3 | plain Stop with a held child, no restart: `PostStop` 1 after the child, `Terminated` 1 | regression |
| T4 | Go-specific (R7): a Goexit inside P1, then a Stop: P1 reported as `poststop-panic`, P2 runs, `Terminated` once, no hang, no second `PostStop` after P2. Marked "no Pekko equivalent" | R7 |
| T5 | W25 (custom `PreRestart` that calls `PostStop`, two calls) stays green with no edit | D21 |
| T6 | the `PostStop` of a normal Stop still runs once when it ends in a Goexit (`stop_goexit_internal_test.go`, no edit) | guard of R1 |
### 7.2 Mutants required by #449 (designed here; run by the coordinator on copies outside the repository)
| Id | Mutation | Killed by |
|---|---|---|
| MU1 "second instance after the Stop" | in `terminate()` (or `drain`), when the cell is `restarting`, call `continueRestart()` so a new instance and `PostRestart` are created | T1 (factory calls, `PostRestart`), W11b |
| MU2 "duplicated Terminated" | `notifyWatchers(watchers)` called twice in `finalize` | T1 (watcher count) |
| MU3 "PostStop(old) with a per-instance guard" | `defaultPreRestart` goes through `beginPostStop` again (today's behavior) | W11b, T1 (count 2) |
Rules as in U6: one at a time, the unmodified copy must pass, a mutant that does not compile or times out is **INVALID** (redesigned once, then reported). The package timeout must be well above the helpers' 30 s waits (use 900 s). The Pekko-side acceptance of the same behavior is the S22 run of 6.3.
### 7.3 In this delivery and deferred
In: W11a/b, T1-T6, S22 on Pekko (3 concurrent runs), MU1-MU3, the new cases J1-J12 of `judge/selftest.py` and the judge mutants MJ1-MJ3 (7.4), all fast gates, one `-race` run, the new tests repeated 30 times under `-race`, the judge self-tests and the matrix check. Deferred (not claimed): mutation and judge campaigns of the module, Go-gate parity, the Go mirror inside the judge (U8), `-race` stress runs, the Pekko surface inventory (#461).

### 7.4 Judge self-tests (`judge/selftest.py`; new cases, written before the comparator change)
Written with the file's own helpers (`case`, `mk`, `ps`, `st`, `ev`, `term`, `tmo`, `run`; the prefix check near `selftest.py:248` is the model for timeout logs). The implementer writes them **first**, runs `python3 -I judge/selftest.py` and sees them fail on the base comparator (`G5.f` unknown, `G5.c` with `pb[0]`/`ce[0]`), then changes the comparator and sees them pass; both runs are reported.
| Id | Clause | Accepts / rejects |
|---|---|---|
| J1 | G5.f | accepts the S22 shape on a complete log: `pre-start` 1, `pre-restart` 1, two `post-stop` (P1, P2), no `post-restart` |
| J2 | G5.f | rejects **P2 absent** (one `post-stop`) on a complete log |
| J3 | G5.f | rejects **an extra `post-stop`** (three), on a complete log **and on a prefix** |
| J4 | G5.f | rejects **a second instance** (`pre-start` with `i=2`, or any `post-restart`), on a complete log **and on a prefix** |
| J5 | G5.f | **prefixes and finalization:** a timeout log (not done) whose counts are still below the targets is not a violation; the same log as a complete log is |
| J6 | G5.f | spec checks: `counts` missing, empty, a negative or non-integer value, or a key that is not a hook event gives `harness-error` |
| J7 | I2.a with the S22 spec | **a duplicated `Terminated`** is rejected (and one is accepted) |
| J8 | G5.c | accepts the accepted case: the parent's first `post-stop-begin` (P1) before a child ends and its last one (P2) after it (the base comparator rejects this) |
| J9 | G5.c | rejects a child whose last `post-stop` comes after the parent's final `post-stop-begin` |
| J10 | G5.c | rejects a child that stops twice with the second stop after the parent's final begin (the base `ce[0]` would accept it) |
| J11 | G5.c | one `post-stop` each: the two existing G5.c cases are unchanged |
| J12 | S22 spec end to end | loads `judge/scenarios/S22.json`; a good concurrent log passes; the logs of J2, J3, J4, J7 and J9 shaped as S22 each fire the expected clause (G5.f, G5.f, G5.f, I2.a, G5.c); evaluated in `controlled` mode it is a `harness-error` (the scenario modes) |
Existing cases (G5.d, G4.a and the rest) are not edited.
**Judge mutants (python, by the coordinator, on a copy of `judge/` outside the repository, one at a time; the unmodified copy must pass `selftest.py`):**
| Id | Mutation | Must be caught by |
|---|---|---|
| MJ1 | `r_G5c` back to the parent's first `post-stop-begin` (`pb[0]`) | J8 (and J12) |
| MJ2 | `r_G5c` back to each child's first `post-stop` (`ce[0]`) | J10 |
| MJ3 | `r_G5f` compares with `>=` (a minimum) instead of equality | J2, J3, J4 |
A mutant not caught by the named case is a defect of the self-tests and is corrected once by the coordinator.

## 8. Acceptance of #449, as observed
1. Executed with kit `cf91c9d` and recorded (`EXECUTION-RECORD.md`; hashes verified).
2. Go rule: the termination path calls `PostStop` once on the current instance; the call of the default `PreRestart` is unguarded user-level code (R1, R2, W11b, T1).
3. `docs/judge.md` G4/G5 corrected (6.1); a Pekko-side scenario (S22, concurrent) and its Go mirror (7.1) that fix the counts and order of `RESULTS.md`, with the S22 run on Pekko judged by the comparator. **Matrix, as a design criterion (demonstrated, not assumed):** `python3 -I scripts/check-judge-matrix.py` passes (`OK: ...`), the designed-coverage line does not drop, and `G5.f` is asserted by S22; `judge/selftest.py` (152 checks, 0 failed at the base), `judge/selftest_go_scope.py` and `judge/selftest_mutate_go.py` stay green, with the new cases J1-J12 added to `judge/selftest.py` (the check count rises and none fails) and the judge mutants MJ1-MJ3 caught. **The observed line** (`relations with every clause observed: 14/14 (Pekko)` at the base) comes only from `judge/mutation-report.tsv`; G5.f has no Pekko mutation campaign yet, so that line may change. It is recorded before and after with the check's own output and reported to the Owner; it is not restored by removing or weakening any clause, mutation or scenario, and a Pekko mutation designed against G5.f is a declared gap in #461.
4. Mutants MU1, MU2, MU3 killed (valid detections).
5. The U2 gap "a Stop during the wait is handled by the per-cell guard" is closed (ADR 0017 `:681`, `:462-467`, D-5 `:625`).
#449 closes **only** if all five are observed and the Owner agrees. A criterion not observed (for example the S22 run) keeps it open.

## 9. Boundaries
| Unit | U7 does not |
|---|---|
| U6 (`AllForOne`) | change its rules; a Stop that overlaps a sibling's restart in an AllForOne round uses the same termination path as any stop |
| U8 (#175) | add S13, S14 or S16, restart events to the Go judge, or S22 to `go-scope.json` |
| U9 (#451) | add logging (`PORT-GAP(logging-U9)` stays) |
| #450 | add user-defined strategies, decider context or `Kill` |
| #461 | run the module-close campaigns; U7 adds its own gaps there |
`stop.go`, `cell.go`, `hierarchy.go` (judge anchors `GM2`, `GMstop`, `GM7`, `GMearly`, `GMnopropagate`) are not edited. If a change would need them: `BLOCKED(surface)` with the evidence.

## 10. Decisions this package asks the Owner to accept
| Id | Decision | Alternative and trade-off |
|---|---|---|
| D1 | the "Go mirror" of #449 is Go unit tests with the counts and order of `RESULTS.md`; the judge's Go gate for restart scenarios stays in U8 | mirror inside the Go judge: needs restart events and a Go scenario runner, which is U8's whole scope |
| D2 | one new Pekko-side scenario, S22 (default hooks, the contract-critical mode), **concurrent only** (a latch in `controlled` mode would block the driver); the other two modes are pinned by T2/T3 and `RESULTS.md` | three scenarios (S22-S24): more judge surface and matrix rows for modes that already match; or a controlled-mode variant without a latch, which would not hold the children |
| D3 | scope G5.c to final-versus-final (parent's last `post-stop-begin`, each child's last `post-stop`) and add the exact-count clause G5.f, both with their own cases in `judge/selftest.py` (7.4); G5.d is not edited | no new clause: `required` minimums cannot say zero or exactly N, so a second instance or an extra `post-stop` would pass S22; or loosen G5.d: weakens every other scenario |
| D3b | the design criterion for the matrix is the `OK:` and designed-coverage lines; the observed line is recorded and reported, not an acceptance gate (6.2, 8.3) | keep "14/14" as a gate: forbids any new clause until a Pekko mutation campaign runs, which U7 does not run |
| D4 | after a Goexit in P1 followed by a Stop, P2 runs (two calls total), documented as Go-specific | suppress P2: would re-introduce a per-cell guard for one case |
| D5 | keep `installNew`'s `postStopBegun = false` | remove it as dead code: a larger diff in a file with no anchors, no behavior gain |
| D6 | the review's permission template allows `git diff <path>` (read-only), because the U6 review lost the AF7, AF16-AF18 and Example hunks when its shell was denied | keep the old template: the same blind spot |
