# GOV-48 decision and reconciliation of GOV-48 and DTH-02 against `develop` (2026-10-26)

**Status: documentation only. Nothing is implemented by this document.** It records an Owner decision, reconciles GOV-48 ([#1195](https://github.com/getsyntegrity/gopekko/issues/1195)) and DTH-02 ([#827](https://github.com/getsyntegrity/gopekko/issues/827)) with `develop` at `cad059c177b37f62c27b8a2e89a2826972817f18` (which contains PR #1207, DTH-01), and gives the residual scope, tests and prerequisites of DTH-02. It claims no parity with Pekko, accepts no ADR, changes no label, and does not approve any proposal other than the decision quoted below.

**Update 2026-10-26 (audit #004, block 1).** Sections 11 and 12 were added: a verifiable delivery matrix for the actor items integrated up to `develop` `13fb9953879462adba4218114d6615dc3d3156b9`, and a concrete proposed wording of the DTH-02 criteria. Sections 1 to 10 keep their original baseline (`cad059c`) and are not rewritten; where a state they describe has changed, section 11 says so.

Pekko baseline: v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. Paths of Pekko are relative to `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor`. Paths of goPekko are relative to the repository and describe `cad059c`. "Read" means the code was read; "executed" means a program was run and its output observed.

## 1. The Owner decision

Provenance: an explicit message of the Owner in the working session (Claude Code) of 2026-10-26, not a GitHub comment. The words, literally (Spanish, as given):

> Para GOV-48 decido:
>
> 1. Mantener la identidad del actor y reutilizar la generación existente de Go para descartar notificaciones de registros retirados o reemplazados. Es un mecanismo interno de Go; no lo atribuyas a Pekko.
>
> 2. En el retiro por stop del watcher, su ID único es suficiente: un watcher detenido no puede volver a registrar watches. No agregues generaciones a esa ruta sin una necesidad concreta.
>
> 3. Unwatch conserva el contrato vigente. Un retiro atrasado no debe borrar un registro posterior; verificá si esa ventana existe antes de introducir mecanismos nuevos.
>
> Apruebo como plan de evidencia alternativa tests Go deterministas para esas ventanas, limpieza eventual y terminaciones concurrentes, con -race. Esto no declara paridad ejecutada con Pekko. Usá un oráculo diferencial cuando exista uno aplicable.

The same message asked for this reconciliation and the residual scope, ordered "No implementes todavía", and kept PR #1208 in draft. It states that the decision does not approve the other proposals of #1208, does not authorize bulk changes to the backlog and does not modify other contracts.

**What the decision settles, in the Architect's reading (translation and reading are not the Owner's words):**

| Point | Reading |
|---|---|
| 1 | The identity rule of GOV-48 is the actor identity, plus the existing per-registration generation of goPekko, used to discard notifications of withdrawn or replaced registrations. The generation is a goPekko-internal mechanism and is not attributed to Pekko |
| 2 | Releasing a stopping watcher's link needs only the watcher's unique ID. No generation is added on that path unless a concrete need appears |
| 3 | `Unwatch` keeps its current contract. No new mechanism is introduced for a late withdrawal before it is checked whether such a window exists (section 5) |
| Evidence | Deterministic Go tests with `-race` for those windows, the eventual cleanup and concurrent terminations are approved **as an alternative-evidence plan**. The Owner approves the plan, not any result, and states that the plan does not declare executed parity with Pekko (consistent with the 2026-10-25 statement recorded in #827). A differential Pekko scenario is to be used when an applicable oracle exists |

What it does not settle: the wording of DTH-02's acceptance criteria (section 6), the labels of any issue, and anything in other ADRs.

## 2. Baseline

| Item | State |
|---|---|
| `develop` | `cad059c177b37f62c27b8a2e89a2826972817f18`, 2026-10-26 |
| PR #1207 (DTH-01, code) | merged in `cad059c` |
| PRs #1201 to #1206 (design, ADR 0010, 0015, 0017 and 0024 amendments, probe) | merged |
| #1200 (GOV-51) | closed |
| #785 (DTH-01) | open, `status:ready`, its code is in `develop` |
| #827 (DTH-02) | open, `status:blocked` |
| #1195 (GOV-48) | open, `status:blocked`; no comments on the three issues |
| Open PRs (checked again on 2026-10-26, after `13fb995`) | #1013 (system start time, SYS-03, draft), #1014 (ADR 0032), #1198 and #1208 (audit and closure records, documentation), and this one |
| PR #1011 (`ActorPath` value, IDN-14) | open when this document was first written; merged afterwards into `develop` as `13fb995`. It changes `actor_path.go`, `errors.go`, `names.go`, `pekko-actor/README.md` and tests, and does not touch `watch.go`, `stop.go` or `id.go` |

**In `develop` versus only in an open PR.** The cross-System watch registration, the withdrawal of a stopping watcher and their tests are in `develop`. **No open PR contains code that touches `watch.go`, `stop.go` or the tests of this area.** The code of #1013 is only in that PR and is unrelated to this reconciliation. #1013 and #1014 both edit `pekko-actor/README.md` (and #1014 edits ADR 0010), so the README line numbers cited below can move when they merge. Every line of goPekko cited in this document was re-read at `13fb995`; none of the cited lines of `watch.go`, `stop.go`, `id.go` or of the README (lines 63-68) moved between `cad059c` and `13fb995`.

## 3. What DTH-01 delivered (`develop` at `cad059c`; unchanged at `13fb995` for the files cited)

Criteria of #785, with the evidence that exists. "Test" names are in `pekko-actor/actor/`.

| DTH-01 criterion | Delivered in `develop` | Evidence |
|---|---|---|
| AC1 `Watch` and `WatchWith` accept native references from another System (and, by GOV-15 option B, `Unwatch`) | Yes. `resolve` finds the record in the System that issued the handle (`watch.go:123-143`); `Unwatch` uses the same path (`watch.go:242-264`) | Read. Tests: `TestWatchOfAnActorOfAnotherSystemDeliversTerminated`, `TestWatchWithOfAnActorOfAnotherSystemDeliversTheMessage`, `TestUnwatchOfAnActorOfAnotherSystem` (`watch_cross_system_test.go:50, 95, 127`) |
| AC2 an already stopped registration produces the documented notification | Yes. A subject with no record, or a finished one, makes `register` call `notifyTerminated` at once (`watch.go:199-202`) | Tests at `watch_cross_system_test.go:68, 112, 144` and `TestAGhostRefOfAnotherSystemIsAnUnconfirmedTermination` (`watch_cross_system_internal_test.go:297`) |
| AC3 registration and unregistration do not nest System locks in opposite orders | By design and by reading: every cross-System call is made with no runtime lock held (`docs/parity/cross-system-watch-lock-analysis.md`, verdicts marked READ). Unwatch and registration take one lock at a time (`watch.go:250-262`, `:222-239`) | **Read, not proven.** The lock analysis marks end-to-end shutdown, the dead-letter subscribers and the body of the `completionPoint` seam as NOT PROVEN (the `runSys` bodies are READ there). No committed test drives opposite-direction cross-System registrations concurrently. The verifier of PR #1207 reports, in its body, a scratch run (not committed, not reproducible from the tree) of concurrent `Tell`, `Stop(watcher)` and `Stop(subject)` across two Systems and of the shutdown of either System under `-race`; that run is not evidence recorded in the repository |
| AC4 focused executable cases, tested SHA, local gates | Partly: the cross-System tests exist and pass (this document, section 8). The local gates of PR #1207 are not re-run here; its body reports them for the tested SHAs `584123f2` (verifier) and `e8bc26b` (author-run) | Executed here: 16 test functions under `-race`, listed in section 8 |
| AC5 public contract and traceability | `pekko-actor/README.md` lines 63-68 state that `Unwatch` discards a `Terminated` already queued, that a handle of another System is accepted for `Watch`, `WatchWith` and `Unwatch` (line 67), and that each registration has its own generation | Read |

Also delivered by the same PR and relevant to DTH-02, not to DTH-01's own criteria: the withdrawal of a stopping watcher from subjects of other Systems (`acceptStop`, `stop.go:50-70`; `releaseForeign`, `watch.go:453-463`), the post-link re-check (`watch.go:204-206`), the Q5 tests T-Q5a to T-Q5e and the Q4 reentrancy test (`watch_cross_system_internal_test.go:104-246`), the ghost-reference test (`:297`), the post-link re-check test (`:337`) and the ID-uniqueness test (`watch_cross_system_test.go:163`).

The issue #785 is still open with `status:ready` although its code is merged. Closing it, or changing its labels, is the Owner's decision; this document changes neither. (Superseded on 2026-10-26: the Owner authorized the closure in audit #004, block 1; see section 11.)

## 4. GOV-48: Pekko evidence and goPekko state

Pekko (code, read):
- The identity of a reference is `path.uid == other.path.uid && path == other.path` (`ActorRef.scala:183-186`). Pekko has no per-registration generation counter.
- A `DeathWatchNotification` is a no-op unless the subject is still in `watching` (`dungeon/DeathWatch.scala:103-104`); the entry is removed before the `Terminated` is enqueued (`:106-111`); `receivedTerminated` delivers a queued `Terminated` once (`:81-93`).
- `unwatch` removes the `watching` entry and clears `terminatedQueued` for the subject (`:67-79`, the clearing at `:75`).

Spec that exercises it: `actor-tests/.../DeathWatchSpec.scala:268-291` ("discard Terminated when unwatched between sysmsg and processing"). It observes that a watcher which unwatches after the notification was queued does not receive the `Terminated` (a watcher without a handler for it would die; the spec checks that it stays alive). It uses one `ActorSystem`.

goPekko (`develop`, read):
- Each registration has a generation, unique in the watcher: `watchGen++` at `watch.go:236-238`.
- A notification carries the generation of the registration it was made for, and is discarded on arrival if the registration is gone or replaced (`watch.go:271-275`) and again when processed (`watch.go:350-353`).
- A stopping watcher is removed from the subject by ID: `removeWatcher(id)` deletes `watchers[id]` and ignores the generation (`watch.go:390-394`).
- IDs are drawn from one process-wide counter and are never reused (`id.go:14-18`).

Under the decision (section 1), this is the accepted rule. The generation is not claimed to be Pekko behavior. The cross-System discard of a stale notification has no executed test today: the generation tests are `TestStaleNotificationOfAnEarlierRegistrationIsDiscarded` (`watch_internal_test.go:87`) and `TestUnwatchThenWatchAgainDeliversOnceNotTwice` (`watch_api_test.go:386`), both for one System.

## 5. The `Unwatch` window of point 3

Question: can a late withdrawal erase a later registration? Answer by reading, `cad059c` (re-read at `13fb995`; `watch.go`, `stop.go` and `id.go` are unchanged).

Every code path that removes a link from a subject by the watcher's ID (the callers of `removeWatcher` are exactly `watch.go:205`, `:262`, `:441` through `leaveWatchees` and `:458` through `releaseForeign`):

| Path | Where | When it runs | Can a registration of the same watcher exist after it? |
|---|---|---|---|
| `Unwatch` | `watch.go:262` | Inline in the watcher's own activation, before `Unwatch` returns | Yes, if the watcher registers again, but only after `Unwatch` returned; the withdrawal already happened (no deferred work) |
| Withdrawal from subjects of the same System at stop | `leaveWatchees`, `watch.go:438-444`, called at `stop.go:65` | Synchronously in `acceptStop`, after `stopRequested` was set under `c.mu` (`stop.go:57`) | No: after `stopRequested`, `register` and `addRegistration` refuse (`watch.go:182`, `:224-226`) |
| Withdrawal from subjects of other Systems at stop | `releaseForeign`, `watch.go:453-463`, deferred at `stop.go:64` | **Asynchronously**, on the subject System's executor, only if a stop was accepted | No, for the same reason: it is issued by `acceptStop` and the stopped watcher cannot register again |
| Post-link re-check in `register` | `watch.go:204-206` | In the watcher's own activation, only if a stop was accepted meanwhile | No: the watcher is stopping |

Other removals, which are not keyed by the watcher's ID or are watcher-side:

| Path | Where | What it does |
|---|---|---|
| `takeWatchersLocked` | `watch.go:419-426`, called once at `stop.go:119` | At the subject's finalization it takes the whole map of watchers in the same critical section that sets `closed` (`stop.go:118-119`), so no later `addWatcher` can succeed (`watch.go:380`) |
| `admitDeathLocked` | `watch.go:355` | Watcher-side: deletes the watcher's own registration after the subject died |
| `unregister` map delete | `watch.go:255` | Watcher-side step of `Unwatch`, before `removeWatcher` |

Findings, all by reading:
- The only asynchronous withdrawal (`releaseForeign`) exists only after the stop was accepted, and a stopped watcher cannot create a later registration. `Unwatch` has no asynchronous part.
- `removeWatcher(wid)` deletes only the key `wid`, and IDs are never reused, so a late release cannot remove the link of another watcher.
- A restart of the watcher does not clear `watching` and does not set `stopRequested`, so it triggers no withdrawal (`watching` is assigned only at `watch.go:234, 414`).
- Therefore the window "a late withdrawal erases a later registration" **was not found in the code**. This is a reading. It is not proven by a test, it assumes the executor of each System eventually runs what it is given (assumptions E1 to E10 of `docs/parity/cross-system-watch-design.md`), and the stop-accepted guard is a property of the code that a future change could break. Tests T3 and T4 of section 7 pin it.
- The argument also assumes that the `Context` calls of one watcher are serialized by its activation (`watch.go:103-106`: valid only inside a hook or `Receive`). `Unwatch` is two steps (the map delete under `c.mu` at `:255`, the link removal at `:262`), so a call made from a foreign goroutine of the same watcher is outside the contract.
- Difference from Pekko, already recorded in the merged design: Pekko's `unwatch` clears `watching` and `terminatedQueued` locally (`dungeon/DeathWatch.scala:72, 75`) but removes the watcher at the subject asynchronously, through an `Unwatch` system message (`:70`, handled by `remWatcher`, `:199-215`). goPekko removes the link synchronously (`watch.go:262`). `docs/parity/cross-system-watch-design.md`, row G6, records this as a timing difference. The synchronous removal is the reason the late-withdrawal window cannot occur in Go; in Pekko the analogous ordering is guaranteed by system-message FIFO.
- Consequence for the decision: no new mechanism is introduced; point 3 is satisfied by the current contract plus these tests.

Where the generation is needed (point 1): when the subject finalizes, it takes a snapshot of its watchers (`takeWatchersLocked`, `watch.go:419-426`) and notifies each one later. If the watcher withdrew and registered again in between, the snapshot entry carries the old generation and is discarded (`watch.go:274, 352`); the new registration is notified separately because `addWatcher` fails on a closed subject (`watch.go:380-382`, then `:200`). That path is why the generation stays.

## 6. DTH-02: criteria, what is delivered and the residual scope

Original criteria of #827, unchanged in the issue:

1. Stopping either side cleans only the matching registration generation.
2. `Unwatch` cancels a queued cross-System notification under the documented rule.
3. Concurrent System terminations deliver no duplicate and leave no retained cross-System link.

| Criterion | Delivered in `develop` | Residual (what DTH-02 still has to deliver) |
|---|---|---|
| 1, watcher stops | Yes: `acceptStop` withdraws from same-System subjects synchronously and from other Systems through `releaseForeign`; re-check at `watch.go:204-206` | Nothing to implement. By decision point 2 the path uses the watcher's ID. Residual **wording**: "matching registration generation" does not describe this path (needs the Owner's confirmation, section 9). Residual **evidence**: T2 |
| 1, subject stops | Yes, the generation discards stale notifications (`watch.go:274, 352`) | Residual **evidence**: a cross-System test of the discard (T1). No code change is indicated |
| 2 | Yes, in code: `Unwatch` deletes the registration and the queued entry (`watch.go:255-259`), and a queued lane item is ignored when taken (`watch.go:477-483, 497-499`). The code does not depend on where the subject lives | Residual **evidence**: no cross-System test of `Unwatch` over an already queued notification (T5). The rule is the one in the README line "discards a `Terminated` already queued" |
| 3 | Partly: a watch of a subject whose System had **already** terminated (`watch_cross_system_test.go:80`: `Terminate` and `WhenTerminated` run before the watcher registers) and the Q5 and Q4 tests. **No committed test terminates a System while a foreign watcher is linked to a live subject** | Residual **evidence**: termination of a System under a linked foreign watcher, concurrent termination of both Systems, and the absence of a retained link after the eventual release (T6, T7). End-to-end shutdown remains NOT PROVEN by the lock analysis |

**Residual scope of DTH-02, in short.** DTH-02 is no longer an implementation of the release path: that path is in `develop`. What remains is (a) the deterministic evidence for the windows and the concurrent terminations that the Owner approved as an alternative-evidence plan, (b) the restatement of criterion 1 so that it matches decision points 1 and 2, and (c) the README and traceability line of the criteria. If a test in section 7 fails, a code correction becomes part of DTH-02 at that point; none is indicated by the reading.

The part of DTH-02's text that mentions "queued cross-System notification" should read, in Pekko terms, that the queue is watcher-local (`terminatedQueued`, `dungeon/DeathWatch.scala:75`); there is no cross-System queue in Pekko. That wording correction is part of the proposal in section 9.

## 7. Deterministic tests of the residual scope

All with the race detector (`go test -race`), no sleeps as proof, controlled executors and the existing seams (`linkPoint`, `recheckPoint`, `completionPoint`, the controlled executor `sup1InlineExec` used by `watch_cross_system_internal_test.go`), as the existing tests do. They are proposals: none exists yet.

| Id | Criterion | Scenario | Assertion | Oracle status (ADR 0033 D13) |
|---|---|---|---|---|
| T1 | 1, subject stops | Watcher `W` in System A registers at `S` in System B (generation 1), calls `Unwatch`, registers again (generation 2). `S` finalizes with its notification to generation 1 delivered after the second registration | Exactly one `Terminated` for generation 2; none for generation 1 | Mechanism: call `notifyTerminated(S, gen1)` directly after the second registration (as `watch_internal_test.go:87` does), or park `S`'s finalization between `takeWatchersLocked` (`stop.go:119`) and `notifyWatchers` (`stop.go:126`) with a blocking dead-letter subscriber. Oracle: the generation is goPekko-internal, but the observable (one `Terminated` after `Unwatch` plus `Watch`) may admit a one-System Pekko scenario; applicability pending determination. Alternative evidence under the Owner-approved plan meanwhile |
| T2 | 1, watcher stops | Two watchers in different Systems watch the same `S`; one stops and its release runs | The link of the other watcher stays; when `S` stops, the other is notified exactly once | Applicability pending determination; Pekko shows the observable result for one watcher only (probe case e, not executed) |
| T3 | `Unwatch` window | Foreign `Unwatch(S)` followed by `Watch(S)`, with the executor of `S` controlled | After `Unwatch` the executor of `S` has no queued work (the withdrawal was synchronous); after the second `Watch` the link is the second registration; `S` stopping delivers once | Pekko `unwatch` clears `watching` and `terminatedQueued` locally but removes the watcher at the subject asynchronously through an `Unwatch` system message (`dungeon/DeathWatch.scala:67-79`, `:70`; `remWatcher` `:199-215`). Go removes synchronously: timing difference already recorded as G6 (`cross-system-watch-design.md:64`). Applicability pending determination |
| T4 | Late release vs later registration | A release from a stopped watcher is queued on `S`'s executor, not yet run; a third watcher registers at `S`; the release runs | The third watcher's link remains; `S` stopping notifies it once | Applicability pending determination (Pekko removes the watcher by key in `remWatcher`, `dungeon/DeathWatch.scala:199-215`); alternative evidence meanwhile |
| T5 | 2 | `S` stops, `W`'s lane holds the queued `Terminated`; `W` calls `Unwatch(S)` before taking it | Nothing is delivered to `W`; the slot is freed; a later registration starts clean | Applicable oracle candidate: `DeathWatchSpec.scala:268-291` for the watcher-local rule in one System. Same-System Go counterparts already committed: `watch_test.go:306`, `watch_api_test.go:354`, `bounded_mailbox_unwatch_test.go:74`. Whether a cross-System Pekko scenario is needed is pending determination |
| T6 | 3 | Systems A and B terminate in a named, finite set of interleavings (listed in the test) of: subject finalization, release run, notification run (controlled executors); watchers in both directions. Sub-case: the owner System has already terminated when the release is issued, with the production executor (advisory from PR #1207) | No duplicate `Terminated` per registration; no panic or deadlock; each listed interleaving ends with the same observable result | No Pekko spec for concurrent terminations of two Systems found (`DeathWatchSpec` has none). Applicability pending determination; the H5b probe does not cover it |
| T7 | 3 | After the eventual release or the subject's finalization, inspect the state | The subject holds no link to the stopped watcher; the watcher holds no registration and no reference to the subject's System (`takeOutgoingLocked` sets `watching` to `nil`, `watch.go:414`) | Go-internal state; alternative evidence. The absence is asserted only after the subject System's executor ran the release or the subject finalized, never at `Done(W)` (accepted design) |

Oracle: the only Pekko material that exists for a two-System differential scenario is the probe `migration/stage-1/watch-cross-system-probe/H5bCrossWatch.scala`. It is **not compiled and not executed** (its README), so it is not an oracle today. It covers cases a to e of DTH-01 and does not cover the cases T1 to T7 above. Whether to extend and run it is a separate decision.

## 8. Executed evidence of this reconciliation

On 2026-10-26 the following was executed in a worktree whose `pekko-actor` directory was checked identical to `cad059c` (the comparison against `cad059c` was empty; the worktree differed only by documentation files):

- `go test -race -count=1 -v -run '<14 patterns>' ./actor/` in `pekko-actor`: 16 test functions passed, `ok ... 2.053s`. They are: `StopOfAWatcherHandsTheForeignReleaseToTheSubjectExecutor`, `AcceptStopIssuesTheForeignHandOff` (3 subtests: `HookEndsTheStopperGoroutine`, `EventSinkPanics`, `AfterTheWatchersOwnHandOver`), `AcceptStopTwiceIssuesOneForeignHandOff`, `StopOfAWatcherReleasesTheForeignLinkWithTheProductionExecutor`, `CrossSystemInlineExecutorSubjectFinalizeRunsWatcherCallbackAndReenters`, `AGhostRefOfAnotherSystemIsAnUnconfirmedTermination`, `ARegistrationLinkedAfterTheStopIsAcceptedIsRemovedByTheRecheckAtAForeignSubject`, `StaleNotificationOfAnEarlierRegistrationIsDiscarded`, `UnwatchCancelsAQueuedTerminated`, `UnwatchThenWatchAgainDeliversOnceNotTwice`, `WatchOfAnActorOfAnotherSystemDeliversTerminated`, `WatchWithOfAnActorOfAnotherSystemDeliversTheMessage`, `UnwatchOfAnActorOfAnotherSystem`, `ActorIDsAreUniqueAcrossSystems`.
- The independent verifier's own run at `13fb995` (17 functions) also passed, `ok 1.834s`.
- Go version `go1.27.2`, as in `.go-version`.

This is an execution of existing tests, not the full suite and not the local gates. It is not evidence for T1 to T7, none of which exists, and it declares no parity with Pekko.

## 9. Prerequisites that remain for DTH-02

| Prerequisite | State |
|---|---|
| DTH-01 (#785) delivered | Code delivered in `develop` (#1207). The issue is still open |
| GOV-51 (#1200) delivered | Yes, closed |
| GOV-15 option B | Decided; amendments of ADR 0010, 0015, 0017 and 0024 integrated |
| GOV-48 (#1195) | Decided (section 1) |
| Wording of the criteria of #827 | **Open, one Owner confirmation.** Original criterion 1 says "the matching registration generation" for both sides, but the decision applies the generation to the discard of stale notifications and the watcher's ID to the release at stop. The Architect's proposal is to restate criterion 1 as two statements and to restate criterion 2 in watcher-local terms (section 12; the original criteria are in section 6). The original text stays in the issue |
| Mandatory test criteria carried by the accepted design (controlled-executor removal test, Q5 tests, executor assumptions E1 to E10) | The removal test and T-Q5a to T-Q5e exist; E1 to E10 remain assumptions of the design |
| Labels | `status:blocked` is unchanged. Whether it changes is the Owner's decision |

If the Owner confirms the restatement, no further prerequisite was found for DTH-02 to be implemented as tests and, if a test fails, the correction it shows. The size of T1 to T7 against the 400-line heuristic is not estimated here. This document makes no claim about slices other than DTH-02.

## 10. Not verified

- The behavior of Pekko was read; no Pekko program was run. `DeathWatchSpec.scala:268-291` was read, not executed.
- The window analysis of section 5 is a reading of `cad059c`.
- The full test suite and the local gates (`golangci-lint`, coverage) were not run.
- The tested tree was a documentation branch whose `pekko-actor` equals `cad059c`; no commit of Go code other than `cad059c` is claimed.
- T1 to T7 are proposals; their oracle status is pending determination except where stated.

## 11. Delivery matrix against `develop` `13fb995` (audit #004, block 1, 2026-10-26)

This section separates what was **decided**, what is **integrated** in `develop`, which **evidence** exists and on which SHA, and what is **left**. Closing an issue means its own outcome was delivered; it does not mean parity with Pekko or a migrated module.

Baselines: goPekko `develop` `13fb9953879462adba4218114d6615dc3d3156b9` (checked against `origin/develop` on 2026-10-26); Apache Pekko v1.7.1, submodule pin `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (checked with `git ls-tree`).

Evidence sources are kept apart:

- **Author:** results reported in the PR body by whoever wrote the change.
- **Independent:** a review or verifier other than the author (native review lenses, the read-only verifier, static review).
- **This session:** run by the Architect of this block on 2026-10-26.
- Remote CI gave no result on any of these PRs: the `go` and `label` jobs were not started (GitHub billing annotation), so no green CI is claimed.

| Item | Accepted contract | Integrated delivery/SHA | Evidence and tested SHA | Remaining scope | Disposition |
|---|---|---|---|---|---|
| IDN-14 #1008, `ActorPath` value | ADR 0031 P-4 to P-8, O4, O5 (accepted 2026-10-24) | PR #1011, squash `13fb9953879462adba4218114d6615dc3d3156b9`, tree `31f051e39809cd111425efe76475e11a530d46b5` | **Author:** full-module `go test -race`, coverage 98.5%, gofmt, vet, `golangci-lint` v2.14.0, tidy, `ci-fast`, `check_deps.sh`, on head `2596066` and on the combined tree `31f051e` (head merged with `cad059c`); five mutants on `43aff37` only. **Independent:** static review of `2596066`, PASS_WITH_MINORS, no test run. **This session:** the tree of `13fb995` equals the author's combined tree `31f051e`; focused tests pass (see below) | Navigation #852, parsing #812, incarnation #853, reference strings #894. No `ActorPath` is attached to a `Ref` or a `System` | Closed as completed (2026-10-26) |
| DTH-01 #785, cross-System `Watch`, `WatchWith`, `Unwatch` | ADR 0010 Amendment 2026-10-25 with ADR 0015 R2, ADR 0024 D2 item 2 and the ADR 0017 markers; GOV-51 design (accepted as design 2026-10-25) | PR #1207, squash `cad059c177b37f62c27b8a2e89a2826972817f18`, tree `ab6b53741e1e71d67054b6a8ccce0aee6ebeeefa` | **Author:** gofmt, vet, `go test -race ./...`, coverage 98.5%, `golangci-lint`, `check_deps.sh`, tidy, `ci-fast` on `e8bc26b` (tree `ab6b537`); four mutants on `71b3bab`, one more on `e8bc26b`. **Independent:** native review of the committed range at `e8bc26b`, one lens, approved (candidate tree `ab6b537`); three-lens native review and a read-only verifier on tree `584123f2` of `71b3bab`. **This session:** the tree of `cad059c` equals `ab6b537`; focused tests pass at `13fb995` | DTH-02 evidence (T1 to T7, section 7); lock order is read and sampled, not checked by a detector; executor assumptions E1 to E10 stay assumptions; the H5b probe is not compiled or executed | Closed as completed (2026-10-26) |
| GOV-15 #1065, decision on cross-System watch | Owner decision option (b), recorded in #1065 (2026-10-25) | ADR 0010, 0015, 0017 and 0024 amendments `febd7308616f3f35ca411d48f858070b1e27e1e0` (#1206); design `5255f38` (#1201), lock analysis `7fab5aa` (#1202), carrier B `ee2ea26` (#1203) | Documentation only; the Owner decision and the design acceptance are quoted in the ADRs and in #1065. The five-section proposal the issue asked for was overtaken by the decision itself: its sections are in the issue body and in `cross-system-watch-design.md` | None of its own. Implementation is DTH-01 (delivered) and DTH-02 | Closed as completed (2026-10-26) |
| GOV-51 #1200, ownership, cleanup and lock order | Accepted as design 2026-10-25 (acceptance record in `docs/parity/cross-system-watch-ownership-story.md`) | `ac94de41b52e00a3c1a2d277fabcd8cd21472789` (#1204) with #1201 to #1203 | Documentation; verdicts marked READ / EXECUTED / NOT PROVEN in the lock analysis | End-to-end shutdown NOT PROVEN (carried by DTH-02 T6) | Already closed as completed before this block; not changed |
| GOV-48 #1195, identity rule for cleaning | Owner decision of 2026-10-26, quoted in section 1 (actor identity plus the existing generation for stale notifications; watcher ID for release at stop; `Unwatch` unchanged; Go tests with `-race` approved as an evidence **plan**) | **Not integrated.** The decision record is this document, only in draft PR #1209 | Decision quoted in #1195 and in section 1. **This session:** none beyond reading | Integration of this document | Kept open: its documentary deliverable exists only in an open PR |
| DTH-02 #827, cleanup of cross-System watches at stop | GOV-48 decision; GOV-51 design; ADR 0010 Amendment 2026-10-25 | Release path delivered by #1207 (`cad059c`): `acceptStop` plus `releaseForeign` (`stop.go:50-70`, `watch.go:438-463`), post-link re-check (`watch.go:204-206`), generation discard (`watch.go:274, 352`) | Same-System generation tests and the cross-System Q4, Q5, ghost and re-check tests (section 3). Section 8 lists the earlier `-race` run of 16 functions at a `cad059c`-identical `pekko-actor` | Tests T1 to T7 (section 7), the restated criteria (section 12), the README and traceability line, oracle determination per ADR 0033 D13 | Open, not closed by this block; `status:blocked` unchanged |
| ADR 0031 | Accepted 2026-10-24 (O1 to O5) | Address `cff473f` (#1010), system name and address `a3aa35d` (#1012), `ActorPath` `13fb995` (#1011) | Per PR, as above | #852, #812, #853, #894 | Dated implementation note added; decisions and quotes unchanged |
| ADR 0010, 0015, 0017, 0024 (cross-System amendments) | Accepted as design 2026-10-25 | Implemented by `cad059c` | As DTH-01 | DTH-02 evidence | Dated implementation notes added; "not yet implemented" kept as the 2026-10-25 statement |
| Program #746 | Not a delivery | n/a | n/a | Its rows `ACTINV-003` ("Not implemented yet") and `ACTINV-049` ("Planned"), and the "in flight" note on #1008, are superseded | Reconciliation comment on #746; body unchanged |
| Audit records #1198 and #1208 | Not decisions | n/a (draft or open PRs) | n/a | #1198 is a snapshot of `9abfb40`; #1208 already records that #1011 merged and that GOV-48 was decided (its `block-1-reconciliation.md`). Its statements that #1008 and #785 are open are superseded by this matrix | Not edited by this block |

**Focused run of this session.** On a clean checkout of `13fb9953879462adba4218114d6615dc3d3156b9`, in `pekko-actor`, with `go1.27.2`:

```
go test -count=1 -v -run 'ActorPath|Elements|Child|RootPath|PathElement|ChildName|GeneratedName|AnotherSystem|ForeignHandOff|ForeignRelease|ForeignLink|CrossSystem|AcrossSystems|ARegistrationLinkedAfterTheStop|StaleNotification|UnwatchCancelsAQueued|UnwatchThenWatchAgain' ./actor/
```

Result: `ok`, 69 top-level test functions passed. Scope: the existing tests that match those names, without `-race`. It is not the full suite and not the local gates, and it adds no evidence for T1 to T7.

## 12. DTH-02: proposed wording of the criteria (confirmed by the Owner on 2026-10-27)

> Confirmed on 2026-10-27 as the acceptance criteria of #827 (decision DTH-02, `docs/parity/typed-owner-decisions-2026-10-27.md`
> section 0). The confirmation approves no test result, changes no label and declares no parity. The criteria below are unchanged.

The original criteria of #827 stay in the issue unchanged. This wording applies the GOV-48 decision (section 1). **It was a proposal when written; the Owner confirmed it on 2026-10-27 (note above).**

1. **Watcher stops.** When a watcher stops, each link it holds at a subject of another System of the same process is removed by the watcher's unique ID. The removal is handed to the executor of the subject's System and finishes when that executor runs it (assumptions E1 to E10), without waiting for the subject to terminate. A watcher whose stop was accepted creates no new link, and no generation is used on this path.
2. **Subject stops.** When a subject of another System stops, a notification made for a registration that the watcher has since withdrawn or replaced (`Unwatch`, or `Unwatch` then `Watch`) is discarded through goPekko's per-registration generation. The current registration is notified exactly once. The generation is an internal goPekko mechanism and is not attributed to Pekko.
3. **`Unwatch` of a queued notification.** If the watcher calls `Unwatch` for a subject of another System after that subject's `Terminated` (or `WatchWith` message) was queued in the watcher, but before the watcher processed it, the notification is not delivered. The queue is the watcher's own (the counterpart of Pekko's `terminatedQueued`); there is no cross-System queue. `Unwatch` keeps its current contract and removes the link at the subject synchronously.
4. **Concurrent terminations.** When Systems terminate concurrently, in each named interleaving of the test, every registration is notified at most once, and nothing panics or deadlocks. Once the subject System's executor has run the release, or the subject has finalized, neither side holds a link. That absence is checked at that point, never at `Done(watcher)`.

| Proposed criterion | Tests (section 7) | Oracle status (ADR 0033 D13) |
|---|---|---|
| 1 | T2, T4, T7 | Applicability pending determination; Go deterministic tests are the Owner-approved alternative-evidence **plan**, with no result approved |
| 2 | T1 | Applicability pending determination (a one-System Pekko scenario may apply to the observable result) |
| 3 | T3, T5 | Candidate oracle `DeathWatchSpec.scala:268-291` (one System, read, not executed) |
| 4 | T6, T7 | No Pekko spec found for two concurrent System terminations; "not found" is not "no oracle". The H5b probe does not cover it and is not compiled or executed |

Out of these criteria: no production change is proposed. A test that fails would bring its correction into DTH-02 at that point.

## 13. DTH-02 evidence delivered (2026-10-27)

T1 to T7 of section 7 are implemented in `pekko-actor/actor/watch_cross_system_dth02_internal_test.go`. No production code changed: no
test revealed a defect. The executed run, its commit and the gates are recorded in the pull request of #827. Kinds of evidence: **executed**
(the Go tests, run in this delivery), **read** (Pekko specs opened, none executed), **invariant** (goPekko-internal state asserted directly).

| Test | Criterion | Go test | Evidence | Pekko oracle applicability (ADR 0033 D13) |
|---|---|---|---|---|
| T1 | 2 | `TestDTH02StaleNotificationOfAnEarlierCrossSystemRegistrationIsDiscarded` | executed; the generation is an invariant | No two-System spec. The observable (only the current registration is notified) projects onto one System: `DeathWatchSpec.scala:169`, `:257` (read). Alternative evidence |
| T2 | 1 | `TestDTH02ReleaseOfOneWatcherKeepsTheLinkOfAWatcherOfAnotherSystem`; the "no new link after the stop was accepted" part of criterion 1 is the existing `TestARegistrationLinkedAfterTheStopIsAcceptedIsRemovedByTheRecheckAtAForeignSubject` (DTH-01) | executed | No two-System spec; one-System projection `DeathWatchSpec.scala:154`, `:169` (read). Alternative evidence |
| T3 | 3 | `TestDTH02UnwatchOfAForeignSubjectIsSynchronousAndARewatchIsNotifiedOnce` | executed; the synchronous removal is an invariant | Not applicable to the timing: Pekko removes asynchronously (recorded difference G6). Notification once: `DeathWatchSpec.scala:169` (read) |
| T4 | 1 | `TestDTH02ALateReleaseDoesNotRemoveALaterRegistrationOfAnotherWatcher` | executed; invariant | None: no Pekko spec covers a late release; goPekko-internal |
| T5 | 3 | `TestDTH02UnwatchDiscardsATerminatedAlreadyQueuedForAForeignSubject` | executed | Applicable for one System: `DeathWatchSpec.scala:268-291` (read, not executed). No two-System spec |
| T6 | 1, 4 | `TestDTH02ConcurrentTerminationOfTwoSystemsThatWatchEachOther` (six named interleavings, each with its exact count of Terminated), `TestDTH02TerminationOfTheWatchersSystemWhileTheForeignSubjectStaysAlive`, `TestDTH02AReleaseIssuedAfterTheSubjectsSystemTerminatedIsHarmless` | executed | None: no Pekko spec terminates two actor systems that watch each other. Alternative evidence |
| T7 | 4 | asserted in the T6 tests and in `TestDTH02AfterTheForeignSubjectFinalizesNeitherSideHoldsALink` | executed; invariant | None: goPekko-internal state |

Criterion numbers here follow section 12; section 7, written before the confirmation, numbers them after the original three criteria of
#827 (its T5 is criterion 3 here, its T6 and T7 criterion 4). Section 7's "each listed interleaving ends with the same observable result"
is replaced by the confirmed criterion 4 (at most once per registration): the interleavings deliver different, exact counts.

Each test was checked against a deliberately broken runtime (generation checks removed, `removeWatcher` clearing every link,
`Unwatch` keeping the queued entry, the foreign release removed, the stop keeping its registrations): at least one test failed for each.
Not done: `go test -race` (the plan of section 7 names it; it was not run in this delivery and needs the Owner's explicit request), no
Pekko scenario was executed, so every row stays alternative evidence and no parity is declared.
