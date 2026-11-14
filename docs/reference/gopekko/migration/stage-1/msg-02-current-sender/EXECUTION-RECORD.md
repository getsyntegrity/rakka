# EXECUTION RECORD: Current sender, stage 2b (#814, PR #1270)

Written after the code. It simulates no prior execution and records no approval that was not given. Sources: Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (read, not run); timestamps from `git log --format='%H %aI %s'`.

## 1. What existed before the code, and what did not

| Fact | Evidence |
|---|---|
| Scope authorization for 2b existed: Owner option B (`ErrOutsideTurn` deferred), recorded in scope section 12 | TL commit `feb62008b720fba043b62170143f9861ee7a8eed`, cherry-picked here as `46f892587b1873233f97339d812a5f766445cb82` (2026-10-28T22:50:19-03:00) |
| Owner accepted the retrospective record **for unit #1268 only**, against RULEBOOK SHA-256 `7b116f3522c310e33c11fe7328448e584cc404385f666530f3fd13ee5794bfd2` | `5b76b0c3bfa9d055e6483c34b3cffcc4a3f08157` (2026-10-28T22:47:29-03:00) |
| 2b code was written **before** these kit rows and this record existed | `43dabd714e51b4ce6eea6153f91d1235c4abf5ca` (22:55:48-03:00) and `4b816bbd5152c453d5accd61d98cf06c75c56190` (22:57:07-03:00); the 2b manifest rows and this record are added by the later docs commit of this PR |
| No Owner sign-off of a RULEBOOK delta or of the 2b manifest rows existed before the code | none in the repository or in the messages relayed to the Dev |

The Owner's acceptance of the RULEBOOK covers unit #1268; it is **not** claimed to cover 2b.

## 2. Does the RULEBOOK need a delta for 2b?

Yes, a small one; RULEBOOK.md is **not edited** (SHA-256 unchanged). Delta, for the Owner to approve or reject:

1. Section 2, stage 2b line: drop `ErrOutsideTurn` (deferred by scope section 12).
2. Section 1: scenarios are S1-S6 and S8-S12; S7 is removed (scope section 12), S11 (sender cleared after a failed Receive) and S12 (Tell keeps its behavior and passes no sender from inside Receive) were added by the Dev, S12 on the Owner's request for a Tell regression test.
3. Section 0, "Adapted, not ported": add the three rows of section 4 below (ReceivePort, clearing after a failure, stale Context) as recorded adaptations or limits.

Nothing else in the RULEBOOK is affected.

## 3. Retrospective exception proposal (2b)

What was missing: the Owner's approval of the RULEBOOK delta above and of the 2b manifest rows **before** the 2b code (kit rule 3). The code commits `43dabd7` and `4b816bb` precede this record.

Content pending acceptance:

| Item | Path | Commit | SHA-256 |
|---|---|---|---|
| RULEBOOK (unchanged; delta of section 2 not applied) | `migration/stage-1/msg-02-current-sender/RULEBOOK.md` | `c6de3e34d28d23951cc086eca68e50c0786c5589` | `7b116f3522c310e33c11fe7328448e584cc404385f666530f3fd13ee5794bfd2` |
| Manifest with the 2b rows | `migration/stage-1/msg-02-current-sender/manifest.tsv` | the docs commit that adds this record (see `git log` of #1270) | `93c7f1b94aa48622f89562acc2d55bfeac7fd4bcbd9b5c51fbacbf1b2c9d80ae` |
| This record | `migration/stage-1/msg-02-current-sender/EXECUTION-RECORD.md` | the same commit | not hashed here |

Correspondence:

| RULEBOOK clause | Implementation | Executed evidence |
|---|---|---|
| 2b: `TellFrom` | `pekko-actor/actor/ref.go:54` (`TellFrom`), `:68` (`tell` puts the sender in the envelope) | S1, S2, S3, S4, S5, S6, S8, S9, S10, S12 in `pekko-actor/actor/sender_test.go`; `go test -count=1 ./...` (pekko-actor) exit 0 |
| 2b: `Context.Sender` (**SUPERSEDED by scope section 13 and section 7 below: the `defer` clear described in the implementation cell is the original behavior, kept as history; the shipped code no longer has it**) | `pekko-actor/actor/actor.go:62`; `cell.go:33-35` (atomic `curSender`), `:318-323` (set per Receive, cleared by `defer`); `mailbox.go` (`senderBox`) | same tests; S11 for the clear after a failure; mutation (clearing `defer` removed) made S3, S6, S11 fail |
| 2b: `ErrOutsideTurn`, S7 | not implemented (deferred, scope section 12) | not applicable |
| 2b: README | `pekko-actor/README.md` "Current scope" | read back; no executed check |
| 3: execution record, cost-log row | this file; `migration/cost-log.tsv` row `814-msg-02-sender-api` | pending Owner decision |
| Pekko differential | none written | pending (section 4) |

Gates already run at `4b816bb` (Go tree unchanged by this docs commit): gofmt, vet, tests, golangci-lint with gocyclo, tidy, check_deps all exit 0; coverage of all modules and `check_coverage.sh 85` exit 0 (pekko-actor 98.60%); diff-cover 11.0.0 100% against the stacked base and against `origin/develop`; `check-headers.sh` exit 1 from pre-existing errors in files this PR does not touch (separate debt, not approved).

**Owner decision (message of 2026-10-29): exceptional acceptance of this retrospective record of the ORIGINAL 2b implementation, limited to #1270.** The code (`43dabd7`, `4b816bb`) preceded the kit delta. No prior approval and no prior execution is reconstructed. The acceptance does not certify parity, V3 or any result that does not exist yet. (This replaces the earlier line "Exception pending Owner decision; not granted".)

### 3.1 RULEBOOK consumed by the correction (fixed before implementing it)

Per scope section 13 the Owner authorized the RULEBOOK delta of 13.4, applied unchanged by the commit `90a6a0aad50402b6b7bd33c858ba588e222be492` (2026-10-28T23:15:32-03:00) **before any Go change of the correction**. The RULEBOOK that the correction implementation consumes is `migration/stage-1/msg-02-current-sender/RULEBOOK.md` with SHA-256 `4e9020899d985632cee69987595f02b237904a7ff9a72f973e923184e1f473a7` (the earlier `7b116f35...` is superseded for 2b). The manifest at that commit has SHA-256 `93c7f1b94aa48622f89562acc2d55bfeac7fd4bcbd9b5c51fbacbf1b2c9d80ae` (unchanged). The Owner's approval of this new RULEBOOK hash is the delta authorization above; nothing more is claimed.

## 4. The three rows

### 4a. ReceivePort

The `Port`, `AttachPort` and `ReceivePort` are a Go construct (`port.go:11-17` calls it an adaptation of work done outside an actor handing its result back). The property shared with Pekko is the delivery of a message that does not come from another actor: in Pekko it is a `!` or `pipeTo` with the implicit sender defaulting to `noSender`, and `sender()` then yields `deadLetters`.

| Property | Applicable Pekko oracle | Executed in Go | Owner acceptance pending |
|---|---|---|---|
| A message from outside the actor reaches the handler with no sender | `ActorRef.scala:141, 157` (`noSender` default); `PipeToSupport.scala:31, 57`; `ActorCell.scala:596-600` (`deadLetters`, mapped to `(nil, false)` by ADR 0032 D3) | **None.** `Post` has no sender argument and `ReceivePort` never sets `curSender`, so `(nil, false)` follows from the code (`port.go:114` marker), but no test pins it. S2 covers only `Tell` into `Receive` | yes: the Go adaptation that a Port post cannot carry a sender; and the missing test (a gap, not a defect) |

Pending differential scenario: Pekko, a future completes and `pipeTo(self)` with no sender, `sender()` read in `receive`; Go, `Post` then `ctx.Sender()` in `ReceivePort`.

### 4b. Sender clearing after a failure

Pekko (ActorCell.scala, FaultHandling.scala): `invoke` sets `currentMessage` (`:553`) and resets it to `null` **only after a successful invocation** (`:559`); on an exception it goes to `handleInvokeFailure` (`:561`; `FaultHandling.scala:215`), which does not touch it. So after a failed message `currentMessage` keeps the failed envelope until (i) the next `invoke` replaces it (resume), (ii) `faultRecreate` runs `aroundPreRestart` reading `currentMessage` (`FaultHandling.scala:101`) and then `clearActorFields` sets it `null` (`:110`; `ActorCell.scala:719`), or (iii) termination runs `aroundPostStop` (`:254`) and then `clearActorFields` (`:273`). The supervisor side is different: `handleFailure` sets `currentMessage = Envelope(f, f.child)` (`:306`), so `sender()` in the parent's strategy is the failed child.

**SUPERSEDED by scope section 13 and section 7 below (original behavior kept as history):** Go: `cell.go:323` clears `curSender` with a `defer` when the Receive call ends, on success, error, panic and Goexit, before the failure is decided.

| Step after a failed Receive | Pekko `sender()` | Go `Sender()` | Pekko-observable difference |
|---|---|---|---|
| success (no failure) | reset to null at `:559` | cleared | none |
| PreRestart | the failed message's sender (`:101` before `:110`) | `(nil, false)` | **yes**, if the hook reads the sender |
| PostRestart | `deadLetters` (cleared at `:110`) | `(nil, false)` | none |
| PostStop after a Stop directive | the failed message's sender (`:254` before `:273`) | `(nil, false)` | **yes**, if the hook reads the sender |
| Resume, next message | replaced by the next `invoke` | cleared, replaced by the next envelope | none observable through a hook |
| parent's strategy/decider | the failing child (`:306`) | `(nil, false)` (the child is a separate parameter) | **yes**, if the decider reads the sender |
| `ResumeWith` continuation | no counterpart | `(nil, false)` | Go-exclusive, no oracle |

So an oracle applies; "no oracle" is not claimed for PreRestart, PostStop or the decider. The Go choice (`TODO(MSG-02-clear)`, scope section 12) is the conservative one and is **not** parity. No Go test exercises PreRestart or PostStop `Sender`; S11 covers the `ResumeWith` continuation only.

| Property | Pekko oracle | Executed in Go | Owner acceptance pending |
|---|---|---|---|
| **SUPERSEDED (see section 7):** Sender cleared after a failed Receive | `ActorCell.scala:553-561, 718-719`; `FaultHandling.scala:101, 110, 254, 273, 306` | S11 (`ResumeWith` continuation sees none); mutation check on the `defer` | yes: keep clear-always, or keep the sender through PreRestart and PostStop like Pekko |

Pending differential scenarios: (P1) fail on a message from X, read `sender()` in `preRestart`: Pekko X, Go `(nil, false)`. (P2) the same with a Stop directive, in `postStop`. (P3) read `sender()` in the parent's decider. This is a recorded difference and no code was changed.

### 4c. Stale Context

Using a `Context` outside its hook is use **outside the contract**. It is not a delivered capability and not a detection guarantee; `(nil, false)` from a stale Context is indistinguishable from an absent sender, and no enforcement is promised (godoc of `Sender`, README "Current scope", scope section 12). It is a documented limit. No Pekko differential is requested: the operation is Go-exclusive.

| Property | Pekko oracle | Executed in Go | Owner acceptance pending |
|---|---|---|---|
| Stale Context use | none (Go-exclusive) | none; the read is an atomic load, race-safe by construction, `-race` not run | none: a limit, not a deliverable |

## 5. Wording check

The godoc of `Sender` and the README paragraph make no claim of detection or of parity, and list `(nil, false)` for the other steps as "carry no sender here". They do not list PreRestart or PostStop, whose Pekko behavior differs (section 4b); that is an omission recorded here, not a contradiction, so no wording was changed.

## 6. Roles and costs

One Dev agent wrote the code, tests and docs. Tokens, wall-clock and cost were not measured: the cost-log row is `unknown`.

## 7. Correction following scope section 13 (supersedes the "Go" column of section 4b)

Section 4b described the original implementation (clear always, `TODO(MSG-02-clear)`), which the Owner did not accept. The correction makes the sender follow Pekko's `currentMessage`. It consumes the RULEBOOK of section 3.1 (`4e902089...73a7`).

| Property (scope 13.3) | Pekko reading (`ActorCell.scala`, `FaultHandling.scala`) | Go evidence (`sender_test.go`, `sender_failure_test.go`) | Pending differential |
|---|---|---|---|
| PreRestart sees the failed message's sender (F1); PostStop after Stop directive (F2) and in the default PreRestart (F3) | `:101-111`, `:247-275` | F1, F2, F3 pass | P1 to P3 of section 4b, Classic judge, not run |
| New instance after a restart sees none (F4); PostStop after success (F5) or idle (F6) sees none | `:559`, `:718-719` | F4, F5, F6 pass | not run |
| A resumed failure keeps the sender until the next message replaces it (F7); ResumeWith continuation (S11, Go-only, `TODO(MSG-02-resume)`) | `:136-153`, `:553` | F7, S11 pass | F7 not run |
| Parent's decider, OnTerminated/WatchWith, ReceivePort start without the retained sender (F8, F9, P1, P2) | decider: the failing child (`:306`), OnTerminated: the watched actor (D5): both DEFERRED and open | F8, F9 assert only "not the retained sender"; P1, P2 assert absent for the Go Port | not run; decider and D5 need an Owner decision |

Mutation checks (each reinstated bug is caught): clear at turn end again (F1, F2, F3, S11 fail), no clear on success (F5), no clear after PreRestart/PostStop/restart (F4), no reset in `port.go` (P2), in `watch.go` (F9), in `supervision_failure.go` (F8). RED-first was not strictly observed for these tests (code and tests were written in the same step); the mutation checks are the substitute and are recorded as such.

Not delivered: the parent decider/PreRestart-after-escalation sender (`TODO(MSG-02-failed)`), D5, DeadLetter.Sender, `ErrOutsideTurn`.

### 7.1 QA finding W1 (recorded, no test)

The clearing in `releaseTerminated` (`pekko-actor/actor/stop.go:141`, `clearCurrent`) is not detected by any test: after the release no valid `Context` can observe it, and use of a stale `Context` is outside the contract. It is faithful to Pekko (`clearActorFields` after `postStop`, `FaultHandling.scala:273`) and is left without a test.
