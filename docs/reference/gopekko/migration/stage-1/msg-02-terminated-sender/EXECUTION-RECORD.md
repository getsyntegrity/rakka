# Execution record: msg-02-terminated-sender (#814 Part 2, D5b, option B1)

RULEBOOK sha256 approved by the Owner (2026-10-29): `4394a15ce794574f2eb37521ab131dab60bc601131217b2bde6237de036c4878`. Results, parity and V3 are not approved in advance.
Base `0ecfc3cff5304c812ca57686b6b365b2cf4c62ad`. Implementation commit `3f2d08c` (branch `squad-c/814-part2-impl`). Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. Environment: macOS arm64, Go 1.27.2 (`.go-version`), golangci-lint v2.14.0 built with go1.27.2.

## RED on the base (tests added, `watch.go` unchanged)

Failed: T1, T2, T3, T5, T6 (PostStop and PreRestart), T8, T9. Passed on the base (they pin existing behavior): T4, T7, T10; T11 passed only vacuously then (nil sender), and was strengthened to require the observed sender before GREEN.

## Criteria

| Id | Test (`pekko-actor/actor/sender_terminated_test.go` unless noted) | Pekko evidence |
|---|---|---|
| T1 | `TestTerminatedSender/T1` | READ + JVM `observed-T.txt` |
| T2 | `.../T2` | READ + JVM `observed-T.txt` |
| T3 | `.../T3` | READ + JVM `observed-T-extended.txt` |
| T4 | `.../T4` | READ + JVM `observed-T-extended.txt` |
| T5 | `.../T5` | READ + JVM `observed-T-extended.txt` |
| T6 | `.../T6` (PostStop, PreRestart) | READ + JVM `postStop` (`observed-T.txt`); `preRestart` after a death pact: JVM `pekko-repro/observed-T6.txt` (see Evidence limits) |
| T7 | `.../T7` | READ + JVM `observed-T-extended.txt` |
| T8 | `sender_terminated_internal_test.go` `TestTerminatedSenderOfForeignSubjectCarriesItsSystem` | none: mechanism, not observable behavior (Pekko never crosses actor systems); not declared differential |
| T9 | `TestTerminatedSenderIncarnation/T9` | JVM `observed-T-extended.txt` (stop and respawn, `==` by path and uid) |
| T10 | `TestTerminatedSenderUnwatch/T10` | READ `DeathWatch.scala:75` |
| T11 | `.../T11` | n/a; STRUCTURAL evidence of mechanism, not behavioral (see "Evidence limits") |
| T12 | existing suite green, unchanged | n/a |

Deviations from the RULEBOOK, to be judged by the Owner: (1) T8 lives in an internal test file (`package actor`), because actor IDs are unique across Systems and the owning System of a handle is not observable through the public interface; (2) the ADR 0032 wording change was not made: D5a/D5b text is not in this base (it lives in #1273); (3) T11 cannot spy the user handle (not forgeable), so it checks that the sender is not the watched handle.

## Mutation checks (reverted, not committed)

| Mutation | Tests red |
|---|---|
| sender nil | T1, T2, T3, T5, T6, T8, T9, T11 |
| no clear on success | T4 |
| owning System always `c.sys` | T8 |
| no replace in `OnTerminated` (no sender) | T1, T3, T5, T6, T8, T9, T11 |

Not run: "resolve the ID by name" (T9) and "store the user's handle" (T11) as code mutations: the design has no lookup by name to mutate; T9 and T11 are red under the nil-sender mutation only.

## Gates (impl SHA `3f2d08c`, pekko-actor unless noted)

gofmt -l: empty. `go vet ./...`: 0. `go test -count=1 ./...` (pekko-actor): ok; `scripts/each_module.sh` coverage run: 0, all modules >= 85% (pekko-actor 98.62%). golangci-lint v2.14.0 (`bin/`): 0 issues (gocyclo included, no `//nolint`). Diff coverage vs origin/develop: 100% (10 lines). `scripts/check_deps.sh`: 0. `make parity-guard`: 0. `make ci-fast`: 0. `.github/scripts/check-headers.sh`: 1, identical on the base (pekko-persistence* and `pekko-actor/actor/names.go`, none in this change). `go doc -all ./actor` before/after: no difference. No `-race`, no Actions, no network isolation run (`unshare` is Linux only).
PR size: implementation 389 authored lines (`git diff --numstat 0ecfc3c...`), this evidence PR separate.

## Evidence limits (Owner decision 2026-10-29)

- **T11 is accepted as structural evidence of mechanism, not behavioral.** The test checks that the sender is not the same interface value that received `Watch` and that `Done(sender)` gives `ErrUnknownHandle` with the channel closed. It does NOT prove by itself that the runtime did not retain the user's `Handle`. The guarantee is the structural inspection: `watchReg` has only `owner`, `msg`, `hasMsg` and `gen`, so there is no field where a `Handle` could be stored. No parity and no V3 are claimed from T11.
- **`observed-T.txt` is not in this PR.** T1, T2 and the death-pact `postStop` rely on `migration/stage-1/msg-02-terminated-sender/pekko-repro/observed-T.txt`, which lives on branch `squad-c/814-part2-kit` (PR #1277), commit `a02902a45f21a4907f811028bfa4010bd74cb0a9`. Permalink: https://github.com/getsyntegrity/gopekko/blob/a02902a45f21a4907f811028bfa4010bd74cb0a9/migration/stage-1/msg-02-terminated-sender/pekko-repro/observed-T.txt . Checked with `git show a02902a:<path>`: the file exists; sha256 `dee3ced36dab5b0dca3c32c379b6288f0f0c87712ff28c814c77f07de48c605b`.
- **T6 `preRestart` after a death pact: EXECUTED on the JVM (2026-10-29), after the implementation record above.** `pekko-repro/Probe-T6.scala`, `RUN-T6.md`, `observed-T6.txt` (sha256 `60eb3686f18d8d60210524a42f925ffda54b252cb8081c86b349f6b6a19f39aa`), Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, JDK 11.0.32, `sbt -batch "judge/runMain ProbeT6"`, exit 0. Observed: `preRestart` reads `sender()` = the terminated actor, cause `DeathPactException` for that actor, message `Some(Terminated)`; `postRestart` reads `deadLetters`. It matches the source reading; no difference. Limits: one run of one Classic scenario, an explicit Restart strategy (Pekko's default decider Stops on a death pact); the Go side was compared by reading only and no Go test was run or written for this; no parity and no V3 are claimed.
- **SOURCE READING (not execution)**, checked line by line on 2026-10-29 against `upstream/pekko` (`9a54d1294c`), all matching the text: `ActorCell.scala:553` (`currentMessage = messageHandle` in `invoke`), `:559` (`currentMessage = null` after success), `:718-719` (`clearActorFields` sets `currentMessage = null`); `dungeon/FaultHandling.scala:101` (`optionalMessage` read from `currentMessage` in the restart path), `:110` (`clearActorFields(failedActor, recreate = true)`), `:300` (`clearActorFields(actor, recreate = false)`), `:305-306` (`handleFailure` sets `currentMessage = Envelope(f, f.child, system)`); `dungeon/DeathWatch.scala:81-91` (`receivedTerminated` reads `terminatedQueued` and delivers the message), `:110` (`self.tell(Terminated(actor)(...), actor)`, the observed actor is the sender). No citation needed correction.

## Reconciliation of stale text (Owner authorization, 2026-10-29; Refs #814)

Text corrected to match what was decided and integrated: `docs/scopes/814-failure-sender-decider.md` (sequencing: Part 2 no longer waits for Squad A, #1098 wiring is postponed); `docs/adr/0032-local-sender-envelope.md` (B1 versus B2 for D5b: B1 chosen by the Owner on 2026-10-29 and implemented in #1278; candidate map kept, B2 and B3 marked discarded); `docs/scopes/814-terminated-watchwith-sender-d5.md` (status line no longer says implementation has not started). The `RULEBOOK.md` row about the ADR wording describes the plan and is left as written.

Check `git grep -n 'TODO(MSG-02' -- '*.go'` at base `00aecb74c7ad8534ba8946eae5ce20fd102058dc`: the markers are **NOT absent**; three remain, all in `pekko-actor/actor` and none changed here (no Go code is touched by this PR):

- `port.go:114` `TODO(MSG-02-port)`: a posted message carries no sender (a live, separate item).
- `sender_test.go:216` `TODO(MSG-02-resume)`: a Go-only construct awaiting kit classification (a live, separate item).
- `sender_failure_test.go:15` `TODO(MSG-02-D5)` inside a comment: it says the sender of Terminated "stays deferred", which is stale since #1278. Candidate for a later one-line comment fix by the Owner's decision.

Open question for the Owner (not done here): whether a later Go test should pin the cause (`ErrDeathPact`) next to the T6 `preRestart` facts. T6 is EXECUTED on the JVM only; no parity and no V3 are claimed.

## Matriz AC1-AC5 (corrección de evidencia)

Refs #814. The five acceptance criteria are copied literally from the body of issue #814. "Tests" lists case names and `file:line` positions taken from `git grep` on this branch: it is a **reading of names, not a review of every test body**, except where a body is said to have been read. Integrated PRs: #1270 `0ecfc3cff5304c812ca57686b6b365b2cf4c62ad`, #1275 `bc3ed9022c2b77f8afa05ea3966635d127170f03`, #1278 `22a7dcf22080da7f4a0595bf64949c62986547fa`, #1281 `e30715de302b9d8c03f21421f5087681830fe900`, #1282 `7840e8893305b0817b76d7b7f6e7d209afe0b8d7`. Paths are under `pekko-actor/actor/`. Line numbers are labelled "base @ 7840e88" or "rama @ f19602d" (the last commit that changed a test file) and are never mixed.

### AC1: "The sender belongs to the dequeued envelope and never leaks from the previous message."

- Tests (names read): `sender_test.go` TestSender (base @ 7840e88 / rama @ f19602d): S1 84/126, S2 96/138, S3 112/154 (previous sender never leaks), S5 151/191, S6 170/210, S8 187/227, S9 241/281, S10 248/288, S11 205/245, S12 222/262; `sender_failure_test.go` F1-F9 (:136-:221), P1 (:233), P2 (:240), G1-G8, G10, G12, G13 (:415-:591; two different subtests are both named "G6", at :464 and :470, not renamed here); `sender_terminated_test.go` T1-T7 (:73-:158), T9 (:233), T10 (:284), T11 (:173); `sender_terminated_internal_test.go` T8 (:27). G9 and G11 are not present as case names (G10 is the last name inside the "G1-G10" group at :414).
- Covers: the sender travels in the envelope, is absent for `Tell`, is kept across a failed Receive and a resume, is cleared on success and after PreRestart/PostStop, and is the observed actor for Terminated and WatchWith (ADR 0032 D5b, B1).
- Does not cover: calls outside a turn (see AC2); T11 is structural evidence only (see "Evidence limits"); bodies were not re-read for this matrix.
- Integrated: #1270 `0ecfc3cf`, #1275 `bc3ed902`, #1278 `22a7dcf2`, #1281 `e30715de`, #1282 `7840e889`.

#### AC1 by property (rama @ f19602d; assertions copied from the test bodies, label "leído en cuerpo")

All lines are in `pekko-actor/actor/`. The "Mutation" column lists only mutations that were actually run and recorded in this record or in this PR; "none recorded" means no recorded mutation targets that property, not that none would catch it.

| Property | Test (file:line) | Concrete assertion (leído en cuerpo) | Mutation recorded that detects it |
|---|---|---|---|
| P-a: the sender seen in `Receive` is the one of the dequeued envelope | S1 `sender_test.go:126` | `ctx.Expect(o.h == actor.Handle(x)).ToEqual(true)` (:135) and `ctx.Expect(o.h.ID()).ToEqual(x.ID())` (:136) | none recorded for S1 |
| P-a under concurrency | S4 `sender_test.go:167` via `expectEachOnce` (:96, body :96-123) | per message `o.h == actor.Handle(senders[o.msg])` and `o.h.ID()` equal to `senders[o.msg].ID()`, `seen[i] == 1`, `len(p.out) == 0` | shared package variable `lastSender` (S4 fails); duplicated push (S4 fails via `seen[i]`) |
| P-b: nothing leaks from the previous message (X, absent, Y in order) | S3 `sender_test.go:154` | `o1.h == actor.Handle(x)` (:163), `ctx.Expect(o2.ok).ToEqual(false)` (:164), `o3.h == actor.Handle(y)` (:165) | none recorded for S3 |
| P-c: a failure does not leak the retained sender to the next delivery or to the new instance | F4 `sender_failure_test.go:162` | `ctx.Expect(e.ok).ToEqual(false)` (:170), `e.h == nil` (:171) for the new instance's PreStart | none recorded |
| P-c | F7 `sender_failure_test.go:192` | `next.msg == 2`, `ctx.Expect(next.ok).ToEqual(false)` (:203), `isSender(m3, y)` true (:204): the next message replaces the retained sender | none recorded |
| P-c | F5 `sender_failure_test.go:173` | after a successful message, `ctx.Expect(e.ok).ToEqual(false)` for PostStop (:182) | the "no clear on success" mutation is recorded against T4 (the Terminated variant of this property), not against F5 |
| P-c | F9 `sender_failure_test.go:221`, T3 `sender_terminated_test.go:91` | `ctx.Expect(isSender(a.until(ctx, "terminated"), x)).ToEqual(false)` (:231, **negative only**: a nil sender would also pass, so it backs no-leak, not a correct sender); T3 `ctx.Expect(isObserved(e, x)).ToEqual(false)` (:101, **negative only**, same limit) next to the positive `ctx.Expect(isObserved(e, w)).ToEqual(true)` (:100), which does back the correct sender | "no replace in `OnTerminated`" turns T3 red (recorded) |
| P-c (restart paths with failure) | F1 `:136` (`isSender(e, x)` :143), F2 `:146`, F3 `:154` (`isSender(..., x)` :152, :160) | PreRestart/PostStop of the failed message see its sender: the kept-on-failure side of the property | none recorded |
| P-d: messages queued before the turn keep each its own sender | S6 `sender_test.go:210` (receiver parked on a gate, three queued) | `o1.h == actor.Handle(x)` (:223), `ctx.Expect(o2.ok).ToEqual(false)` (:224), `o3.h == actor.Handle(y)` (:225) | none recorded for S6 |
| P-e: a sender that stopped after the send stays retained | S5 `sender_test.go:191` | after `<-done` of the stopper, `ctx.Expect(o.h == actor.Handle(x)).ToEqual(true)` (:208) | none recorded |
| P-f: the decider and PreRestart see the failing child, not the sender of the child's message | F8 `sender_failure_test.go:206` | `ctx.Expect(o.ok && o.h.ID() == kid.ID()).ToEqual(true)` (:218), `... z.ID()).ToEqual(false)` (:219) | none recorded |
| P-f | G2 `:423`, G3 `:433` | `senderIs(e, x)` false (:430), `senderIs(e, kids[0])` true (:431); G3 `isChild(e, kids[0])` true (:443), `senderIs(e, y)` false (:444) | none recorded |
| P-f (PreRestart after escalation) | G4 `:446` | `ctx.Expect(senderIs(e, c)).ToEqual(true)` (:450), `cf.Child.ID()` equals `c.ID()` (:453) | none recorded |
| P-c (new instance after the escalation and restart) | G5 `:456` | `ctx.Expect(e.ok).ToEqual(false)` (:461), `e.h == nil` (:462) | none recorded |
| P-g: Terminated and WatchWith messages see the observed actor | T1 `sender_terminated_test.go:73` | `ctx.Expect(isObserved(e, w)).ToEqual(true)` (:79), `ctx.Expect(e.h.ID()).ToEqual(e.msg)` (:80) | "sender nil" (T1 red), "no replace in `OnTerminated`" (T1 red): recorded |
| P-g | T2 `:82` | `ctx.Expect(e.msg).ToEqual(77)` (:88), `isObserved(e, w)` true (:89) | "sender nil" (T2 red): recorded |
| P-g (failure and death pact) | T5 `:114`, T6 `:128` and `:139` | T5 `isObserved(e, w)` (:121), `term.Who()` equals `w.ID()` (:124), `again.ok` false (:126); T6 `isObserved(a.until(ctx, "poststop"), w)` (:137), cause `ErrDeathPact` (:150) | "sender nil" turns T5/T6 red (recorded) |
| P-h: `Tell` and `TellFrom(nil)` show no sender | S2 `sender_test.go:138` | `ctx.Expect(o.ok).ToEqual(false)` (:147), `o.h == nil` (:148); PreStart `pre.ok` false (:151) | none recorded |
| P-h | S12 `sender_test.go:262` | `ctx.Expect(o.ok).ToEqual(false)` (:278): `Tell` from inside `Receive` passes no sender | none recorded |

Gaps and limits found while reading the bodies:
- **GAP (mutation evidence):** no recorded mutation targets P-b, P-c (except via T-suite), P-d, P-e or P-f; for those the evidence is the assertion alone, observed green, not shown to turn red under a fault.
- **GAP (indirect only):** P-d is exercised with three queued messages and a parked receiver (S6); no test queues a message whose sender was stopped and a message that failed in the same queue.
- P-g's `OnTerminated` sender is accepted as structural evidence for the "not the Handle the user watched" part (T11), as stated in "Evidence limits".
- All checks are single-run on one machine. `-race`: see "Race run" below.

### AC2: "Calls outside a valid actor turn have the documented invalid-context outcome."

- **Withdrawn from the scope by the Owner on 2026-10-29 (#1282). NOT implemented, no test.** `ErrOutsideTurn` is out of scope. The original wording is not marked as implemented anywhere in this record.
- Integrated: #1282 `7840e889` records the decision only.

### AC3: "Concurrent producers cannot overwrite each other sender metadata."

- Test: `sender_test.go` S4 (base @ 7840e88 :125, rama @ f19602d :167), 32 producers released through a start gate and a `WaitGroup`.
- Before this correction S4 only checked `o.h == Handle(senders[o.msg])` and indexed `senders[o.msg]` unchecked, so a lost or duplicated message could pass or panic. Now it checks `o.ok` and `0 <= o.msg < n` before indexing, counts each message exactly once (`seen[i] == 1` for every i), requires `len(p.out) == 0` after the n reads, and compares both the interface value and `ID()` of the sender.
- Local mutation checks of this correction (reverted, not committed): a package-level `lastSender` assigned before the lock is detected by the old and the new S4; a duplicated push when `msg%7 == 0` is detected only by the new S4 (`seen[i]`); a dropped message fails the new S4 with "only 31 of 32 observations within <evWait>; missing message indexes: [14]" (checked with evWait lowered to 1s; the old S4 blocks until the global test timeout).
- Does not cover: a proof of absence of races (a single `-race` run is recorded in "Race run"); ordering between producers (not asserted).
- Integrated: S4 original in #1270 `0ecfc3cf`; the reinforcement is in the PR that carries this section (SHA to be recorded in a later step, not invented here).

### AC4: "Include focused executable positive and negative cases for this result, including the named ordering or race boundary; record the tested candidate SHA and applicable local gates."

- Positive and negative cases: the AC1 and AC3 lists above (for example S1/S2 present versus absent sender, S3 and S6 ordering per message, S12 FIFO, S4 concurrency, T10 negative: nothing delivered after Unwatch).
- Tested candidate SHAs: the "Gates" section above (impl SHA `3f2d08c`) and the integrated SHAs listed at the top of this section.
- Local gates as recorded above: gofmt, `go vet`, `go test -count=1`, coverage per module, golangci-lint v2.14.0, `scripts/check_deps.sh`, `make parity-guard`, `make ci-fast`. No Actions run. `-race` was not part of the Dev gates (see "Race run" for the one run done on the Owner's request). The event waits of the failure and terminated tests are bounded by `evWait` (30 s, only a hang bound, never used to decide absence).

### AC5: "Update the public contract and traceability for this slice; do not claim completion of sibling stories or full actor parity."

- Contract: `docs/adr/0032-local-sender-envelope.md` (D3, D4, D5, D5b). Traceability: `docs/scopes/814-msg-02-current-sender.md`, `docs/scopes/814-failure-sender-decider.md`, `docs/scopes/814-terminated-watchwith-sender-d5.md` and this record.
- Markers still alive (checked with `git grep -n 'TODO(MSG-02' -- '*.go'`): `TODO(MSG-02-port)` at `port.go:114` and `TODO(MSG-02-resume)` at `sender_test.go` base @ 7840e88 :216, rama @ f19602d :256.
- No sibling-story completion and no full parity is claimed.

## Bounded waits (evidence correction)

- `waitEv` (`sender_failure_test.go`) bounds `until` (hookActor, famParent; 2 definitions and 58 uses by `grep -o '\.until(ctx'`: 42 in `sender_failure_test.go`, 16 in `sender_terminated_test.go`), the T10 loop and the T6 `cause` read with one `evWait` timer (30 s) per wait, created once with `defer Stop`. The timer only bounds a hang; it is never used to decide that an event is absent. A **closed channel fails explicitly** (`Fatalf` with the test, the awaited event and the events seen); it never returns a zero event.
- Local negative check for the closed channel (temporary test file, removed, not committed). Command: `cd pekko-actor && go test -count=1 -timeout 60s -run TestZZClosedChannel ./actor`, with a test that sends one non-matching event, closes `hookActor.ev` and calls `until(ctx, "poststop")`. Output (stdout, stderr empty), exit code 1, in 0.00 s of test time: `channel closed before a poststop event; events observed: [{hook:recv h:<nil> ok:false msg:<nil>}]`.
- The earlier mutation checks (reported in the PR #1283 table) lowered `evWait` locally to 1 s and suppressed an event: the tests failed at about 1 s with the same style of message.

## Blocking reads left unbounded (classification, reading only)

Not bounded in this PR: the request was to review them, and bounding them would widen the change. Lines are "rama @ f19602d" (no test file changed after that commit). The list comes from `grep -n '<-'` over the five sender test files (`sender_failure_internal_test.go` has none), keeping reads and the one test-side send; reads inside `select` with the `evWait` timer are bounded and not listed. Each line was classified by reading its context. **The H reads stay unbounded and nothing here promises that they cannot hang.** Classes: **H** = waits for an event that may never arrive if the behavior fails (can hang); **A** = inside an actor, out of scope; **N** = send into a buffer with room, does not block; **S** = already synchronized by a cited earlier step.

| Class | File and lines (rama @ f19602d) | Reason |
|---|---|---|
| H (17) | `sender_test.go` 132, 145, 150, 162, 199, 204, 206, 207, 216, 221, 222, 235, 240, 241, 259, 276, 296 | `<-p.out`, `<-p.pre`, `<-p.entered` (closes at the first `Receive`), `<-done` (stopper terminates), `<-seen` (ResumeWith continuation): all produced from actor goroutines, no earlier synchronization |
| H (13) | `sender_failure_test.go` 215, 217, 237, 247, 391, 410, 518, 520, 560, 568, 574, 576, 587 | kids, `seen`, `ports` (whether `AttachPort` already ran when `Spawn` returns was not verified), `<-done` (child termination) and `<-k.ev` records come from actor callbacks |
| H (3) | `sender_terminated_test.go` 238 (`old := <-p.kid`), 241 (`<-p.seen`), 247 (`<-p.live`) | published from actor callbacks |
| H (2) | `sender_terminated_internal_test.go` 39 (`<-ready`), 40 (`<-fa.seen`) | subject termination and the watcher callback |
| A (3) | `sender_test.go` 52 (`<-p.gate` in the probe's `Receive`); `sender_terminated_test.go` 213 (`<-p.kid` in `tsParent.Receive`), 266 (`<-u.done` in `tsUnwatcher.Receive`) | inside an actor, out of scope |
| N (1) | `sender_terminated_test.go` 239 (`p.kid <- old`) | channel of capacity 2 holding at most one element |
| S (1) | `sender_test.go` 183 (`<-start` in each S4 producer) | S4 closes `start` unconditionally at :187, and `wg.Wait()` is at :188 |

Totals: H 35, A 3, N 1, S 1, counted in lines. `sender_test.go` :162, :222 and :296 hold 3, 3 and 2 reads, so the H lines are 40 reads.

Scope of N: the table lists reads and the one test-side send (:239). Sends made from actors are not listed: `p.pre` (cap 1), `p.out` (256), `a.ev` and `p.ev` (64), `p.kids` (8), `k.ev` (16), `u.ev` (16), `p.kid` (2), `seen`/`live`/`fa.seen` (1). They did not block in the current runs; the capacity versus the number of sends was read, not counted exhaustively. Three sends are `select` with `default` and never block: `sender_failure_test.go:71` (`a.ports`) and `:281` (`p.seen`), and `sender_terminated_test.go:56` (`cause`).

## Race run (executed once at the Owner's explicit request; does not prove absence of races)

The orchestrator ran `go test -race -count=1 -timeout 500s ./...` from `pekko-actor` on c79cdec3bf2f39eb73302c295deb3403af4b9190: exit 0, empty stderr, no "DATA RACE" in the output, go1.27.2 darwin/arm64 (Darwin arm64). Logs are in the orchestrator's scratch, not in this repository. `c79cdec..HEAD` changes no `.go` file, so the result applies to this HEAD. One run on one machine: it does not prove that races are absent.

## Isolated failure (undiagnosed)

One run of `go test` in pekko-actor exited 1 after `develop` was merged into the branch of #1278, and which test failed was not recorded. 20 later full runs (cold cache, GOMAXPROCS 1 and 2, under load) and 200 iterations of the sender tests all passed. This is evidence of non-reproduction only: it is not an explanation and not a fix, and this PR does not attribute the failure to anything.
