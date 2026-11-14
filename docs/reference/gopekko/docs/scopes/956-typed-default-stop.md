# Scope: Typed default Stop (#956, TYP-09)

Status: **approved by the Owner at gate G2 on 2026-10-28 with the decisions D-1 to D-5 recorded under "Decisions for G2"**, corrected the same
day by D-6 (prerequisite is `PostStop` of Slice A of #954) and by the `ChildFailed` destination decided in the scope of #954; #956 stays
**blocked** in the issue labels until the Owner decides otherwise. **Deltas D-7 (S6 and S10 of Slice A, one more allowed file) and D-8 (PR 2 split in 2a and 2b, four PRs) were
changes after G2; the Owner re-confirmed them on 2026-10-28 at `ea63958` (comment on #1233: "D-7 and D-8 only ... No other change, no new decisions").** Written on 2026-10-28 against `develop` `3b47251dc71498d62b171eaf30a9a1292a80cf76`
and updated the same day against `develop` `a03e38d`, where Slice A of #954 is integrated (the D-6 prerequisite is met, see "Dependencies"). Delivered in
`develop` by #1240, #1243, #1241 and #1242 (candidate to squash mapping: `migration/stage-1/typed-956-default-stop/EXECUTION-RECORD.md`
section 8); the text below is the scope as approved and is not rewritten.
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: this scope ports Pekko behavior, so the Dev runs the migration
kit with its own manifest, RULEBOOK and RUNBOOK. The prerequisite of D-1 (a) as corrected by D-6 (`PostStop` of Slice A of #954, integrated and verified) is met as of `a03e38d`; the Dev
is a separate session assigned by the orchestrator (the TL does not implement), and QA is independent of both. This PR stays a draft
and no issue is edited or relabeled.

## What this delivers

Today a Typed actor whose turn fails is decided by the runtime: `/user` restarts it and `Setup` runs again (`pekko-actor-typed/README.md`,
"Interim failure policy"). That is the Classic default, not the Typed contract. This story makes the Typed default **Stop**
(ADR 0033 D2, ADR 0035 contract 1): a Typed actor that has no matching `supervise` stops when its behavior fails, and the failure is not
handed to the parent's strategy.

The only spawn path that exists is `actor.Spawn(sys-or-classic-ctx, name, typed.NewActor(b))`, which is mechanism **M-B** of ADR 0035: the
interpreter is the default wrapper (`supervise(_).onFailure(stop)` in Pekko, `AA/ActorAdapter.scala:168-172`). A failed turn (a returned
error, a panic, a nil behavior) makes the interpreter call `StopSelf` and return `nil`; the parent's decider is not consulted; a watcher gets
a plain `Terminated`. No exported symbol is added: the change is inside the unexported interpreter.

```go
// pekko-actor-typed/typed/interpreter.go (shape of the contract, not the implementation)
func (i *interpreter[T]) Receive(ctx *actor.Context[T], msg T) error // never returns the failure of a turn; stops the actor instead
```

## What this does not deliver

- **Explicit `Resume`** (the issue's criterion 2): #1183. **Most specific strategy, `Supervise`, `OnFailure`**: #1184. The issue was split
  on 2026-10-24 (ACT2 reconciliation).
- **Mechanism M-A** (a Typed parent) and `ChildFailed`: they need a Typed spawn from a Typed context (#1170), a Typed watch (#1171) and the
  signal types (#954, decision B-2). Nothing to test today. Their destination is a separate story that does not exist as an issue yet, see
  "Traceability of what leaves this story".
- **`PostStop` delivery machinery and the signal types**: Slice A of #954. This story adds the test that the default stop delivers `PostStop`
  (S8), which is **mandatory** for #956 (D-1 (a)) and is written once Slice A is integrated and verified (D-6).
- `Stopped`/`StoppedWith` (#1182), `Unhandled` (#925, #1181), restart, limits, backoff, stash (#976, #997, #986, #999), the Typed System
  guardian (#994), Classic↔Typed adapters (ADR 0033 D12), decision S-2 (match-all `withLimit`, open).
- Logging of the failure at ERROR (ADR 0035 contract 13): **pending**, not delivered and not approximated with a TODO (D-3). No accessor to the
  per-actor logger exists.

## Dependencies, as they really are

| Dependency | State today | Effect on #956 |
|---|---|---|
| #847 (TYP-02) | closed, integrated | Delivered: `typed` package, interpreter, `Setup`/`Receive`/`Same` |
| ADR 0035 S-1 and S-6 | accepted 2026-10-27 | Contract 1, M-A/M-B and the D2 precision are consumable |
| #954 (TYP-04), **Slice A only** | **#954 stays open** (Slice B and B-2 pending). Slice A is **integrated in `develop` `a03e38d`** (#1234 `85d2e76`, #1235 `8d350c5`, #1236 `ab9b44c`, #1237 `a03e38d`); the scope is `docs/scopes/954-typed-lifecycle-signals.md` | **Prerequisite met** (D-1 (a), D-6): #956 consumes **`PostStop` of Slice A integrated into `develop` and verified**, **not the closure of #954**. Available to #956: `Signal`, `ReceiveSignal`, `PostStop` (`typed/signal.go`) and the interpreter's `PostStop` and no-op `PreRestart` (`typed/interpreter.go`). Slice B (`Terminated`, death pact) is not a prerequisite. Verification of the slice: the QA reviews and gate comments on #1236 and #1237, plus the TL's local run on the merged tree (`go test -count=1 -cover ./...` in `pekko-actor-typed`: ok, 100.0% of statements; `go vet`, `gofmt -l`, `scripts/check_deps.sh` clean). Order: Slice A of #954, then #956 |
| #925 (TYP-03), listed in the issue | open | **Not needed**: it concerns `Unhandled`/`Empty`/`Ignore`, not the failed result |
| #1063 (GOV-01), listed in the issue | open | Replaced by the accepted decisions S-1 and S-6 (reconciliation, #1183 row) |
| #1170 (Typed spawn) | open | Not a blocker; it is why M-A is out |

## Contracts consumed

| Contract | Source | State |
|---|---|---|
| Default Stop delivers `PostStop` and terminates (contract 1); M-B wrapper | ADR 0035 | accepted (S-1) |
| Reading of ADR 0033 D2 | ADR 0035 "Proposed precision" | accepted (S-6) |
| Failed turn: error, panic, nil behavior; start failure is an `InitializationError` | ADR 0034 P3, P4, P8 | accepted (B-1) |
| `actor.Context[T].StopSelf` (`actor.go:53`), `actor.PostStopper` (`actor.go:38`), `actor.ErrGoexit`, `*actor.PanicError` | `pekko-actor` | exported today; **no `pekko-actor` change, no EXP-1 record** |
| Classic `DefaultStrategy` and `defaultDecider` | ADR 0017, `supervision.go:103-110` | unchanged; regression-tested (S6) |

## Criteria → scenario → Pekko source

The issue's wording is kept in the issue. After the split its criteria 1 and 3 stay; criterion 2 moves to #1183. Restated:

| # | Criterion (restated) | Scenario in `pekko-actor-typed/typed/` | Pekko source | Evidence |
|---|---|---|---|---|
| 1 | A Typed actor with no `supervise` stops when a turn fails; the parent's decider is not consulted; a watcher gets `Terminated` | S1 handler returns an error; S2 handler panics; S3 handler returns `(nil, nil)` (cause `ErrNilBehavior`). Each: actor terminated, watcher saw `Terminated` once, parent decider call count 0, no restart (setup counter 1) | `SupervisionSpec.scala:387, 492, 502`; `ClassicSupervisingTypedSpec.scala:74-160` (`:105`) | SPEC |
| 1 (ordering) | Mail queued behind the failing message is not handled after the stop | S4 m2 is told while m1 is held on a gate; m1 fails; m2 never reaches a behavior (dead letter) | `ActorAdapter.scala:125-141` | READ |
| 1 (descendants) | The stop stops the actor's children | S5 a Classic child spawned from the setup (internal test, `package typed`, through the wrapped context) terminates with the actor | Classic `context.stop` | READ (no Typed spec asserts it) |
| 1 (negative) | `runtime.Goexit` is never converted into a Typed stop | S7 handler calls `runtime.Goexit`; the runtime's own `ErrGoexit` path decides, as before | ADR 0035 "Supervise"; Pekko never catches fatal throwables (`S:75-76`) | READ |
| 1 (`PostStop`) | The default stop delivers `PostStop` to the behavior, once, before the watcher's `Terminated` | S8, **mandatory for #956**; Slice A is integrated (D-6), so it is written with `ReceiveSignal` over a `Receive`/`ReceiveMessage` base: after a failed turn the handler of the behavior that was current when the turn failed receives `PostStop` exactly once, and the watcher's `Terminated` comes after it | `SupervisionSpec.scala:387-396` | SPEC |
| 3 | The Typed default does not change Classic `DefaultStrategy` | S6 a Classic actor failing under the same parent is still restarted (decider-chosen Restart, cause delivered), while a Typed sibling stops | `TypedSupervisingClassicSpec.scala:61-71` (Classic side) | SPEC |
| (kept) | Start failures are unchanged: a failing `Setup` is an `InitializationError` and stops | existing S5/S6 of #847 pass unchanged | ADR 0034 P3 | SPEC |
| 4 | Positive and negative cases, ordering boundary named, candidate SHA and gates recorded | S1 to S8; ordering boundary S4; SHA and gates in the PR comments and the execution record | | |
| 5 | Public contract and traceability updated; no sibling or parity claim | `pekko-actor-typed/README.md` | | review |

A scenario observes its result with channels and gates, never with a sleep; `-race` is not used. A scenario that fails and is not a Go
defect is run on Pekko before it is classified (`AGENTS.md`).

## Allowed files

Exactly these paths. Anything else needs a stop and a report to the TL.

| Path | Edit | PR |
|---|---|---|
| `docs/scopes/956-typed-default-stop.md` | this file | 1 |
| `migration/stage-1/typed-956-default-stop/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | new kit package; every Pekko spec row ported, adapted, moved or `PORT-GAP` | 1 |
| `pekko-actor-typed/typed/interpreter.go` | the failed-turn path of `Receive`; keeps its single `Ported from` line | 2a |
| `pekko-actor-typed/typed/typed_test.go` | S9/S10 of #847 are rewritten (decision D-4) | 2a |
| `pekko-actor-typed/typed/signal_runtime_test.go` | **delta D-7, one file added after G2 (`9c716c3`)**: only `TestSig_S6_FailedRestartStartGetsNoPostStop` and `TestSig_S10_RestartDeliversNoPostStopOnOldInstance` change; no other test of the file, no production code. Plain `testing`, as the file already is | 2a |
| `pekko-actor-typed/typed/default_stop_test.go` | new, S1 to S8 (S8 is written against `PostStop` of Slice A of #954, integrated and verified); S1 to S3 in PR 2a, S4 to S8 in PR 2b | 2a, 2b |
| `pekko-actor-typed/README.md` | replace the "Interim failure policy" limitation (a failed **turn** stops the actor; a failed **start** and a runtime restart keep the runtime's rules); implemented versus planned; keep the Slice A text of `PostStop`, the no-op `PreRestart` and its evidence gap unchanged; state that failure logging (contract 13) is pending and where each row went | 2a |
| `migration/stage-1/typed-956-default-stop/EXECUTION-RECORD.md`, `migration/cost-log.tsv` (one row) | commands, kit version `cf91c9d`, models, inputs, outputs, manual changes; gate evidence bound to a SHA | 3 |

Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md` or `AGENTS.md` changes. `interpreter.go` is also edited by
#954 (Slice A) and #1183: the order Slice A of #954, #956, #1183 makes the three sequential, so no two scopes run in parallel on it.

## ADR impact and contradictions (ADR 0033 D5)

1. **Issue criterion 2 and title versus the split.** The issue body still says "explicit Resume"; the ACT2 section moves it to #1183. Proposed
   replacement of criterion 1 (the TL does not edit the issue before G2): "An unsupervised failure stops the Typed actor: `PostStop` is
   delivered, the actor terminates, its children stop and the parent's strategy is not consulted (mechanism M-B of ADR 0035); a Classic
   actor under the same parent keeps ADR 0017's Restart."
2. **Typed default Stop (ADR 0033 D2) versus the runtime's Restart (ADR 0017).** Not a contradiction after S-1/S-6: ADR 0035 extends ADR 0017 and
   changes no Classic row. S6 pins that.
3. **#847's interim policy (its D-4) is superseded here by plan**, as that scope said (`docs/scopes/847-typed-behavior-install.md`, D-4 (a)).
4. **Matrix row "Classic / Typed / setup: no initialization failure reaches the parent" is READ, no spec.** Today a failing `Setup` returns an
   `InitializationError` and the default decider stops it (same outcome, different mechanism). Leave it, with
   `// TODO(TYP-09-start): Pekko's wrapper stops without reporting to the parent`, and record it as a gap.
5. **Facts from the integrated Slice A that the Dev must respect (no new decision):**
   - A panic in a handler is recovered by the runtime around `Receive`, which is where the decider is consulted. For the interpreter to stop the
     actor itself (M-B), the failed-turn path must recover the panic inside `Receive`; it must not catch `runtime.Goexit` (S7).
   - The failed turn installs nothing, so `PostStop` reaches the behavior that was current when the turn failed (S8). A start that failed gets no
     `PostStop` (README of Slice A; ADR 0034 B17).
   - **Corrected by D-7.** The first version of this bullet said the no-op `PreRestart` of Slice A (its D-4) was not involved. That was wrong in
     one respect: a failed turn now stops the actor and never reaches a runtime restart, and two merged Slice A tests (S6 and S10 in
     `signal_runtime_test.go`) forced their restart with a failing turn. The restart itself is still reachable without a failing turn (D-7).
     A failed **start** still goes to the runtime's decider, and a custom decider that restarts it still gets no `PostStop`; that stays #954 / #976.
   - The cause of a stopped turn is not recorded anywhere until failure logging (contract 13, pending, D-3) and `ChildFailed` (the M-A story)
     exist. The README says so.
6. No ADR is amended.

## Evidence and validation

**Migration evidence (kit run).** The manifest rows are `SupervisionSpec.scala:387, 492, 502`, `ClassicSupervisingTypedSpec.scala:74-160` and
`TypedSupervisingClassicSpec.scala:61-71`. Rows that need a Typed parent or `supervise` are moved to their stories, not dropped. Pekko is read,
not run. This is Go unit evidence derived from specs, **not V3**; rows marked READ (S4, S5, S7) stay "pending approval" under ADR 0033 D13 and
block any parity claim, not delivery.

**Local gates** (commands in `AGENTS.md`, "Go"): unit tests with network isolation for `pekko-actor-typed`, `go vet`, `gofmt -l`,
`golangci-lint` with `gocyclo` at most 10, `go mod tidy -diff`, module coverage at least 85%, diff coverage at least 85%,
`scripts/check_deps.sh`, `.github/scripts/check-headers.sh`. Run once per final tree. `pekko-actor` is untouched: its tests are not re-run.
No `-race`.

## Delivery, size and tasks

Draft PRs, body starting `Refs #956` (never `Closes`), Conventional Commits, no merge. #956 stays open until the Owner closes it. PR 1 is
the kit package (the scope file ships through PR #1233, which the Owner decides); **PR 2 was projected at 250 to 350 authored lines and measured 528
(additions plus deletions against `develop` `8ebd25a`: `interpreter.go` 37, `default_stop_test.go` 297, `typed_test.go` 60,
`signal_runtime_test.go` 101, README 33)**, so by the stop rule of this section the TL splits it on a behavior boundary (decision D-8): **PR 2a**
is the fix and the tests it changes, **PR 2b** is the remaining test-only scenarios; PR 3 is the execution record (about 100 lines). The order
is stacked: PR 1, PR 2a, PR 2b, PR 3. Each PR stays at or under 400 authored lines on its own, its count is stated in its body, and the suite is
green at each PR alone. If any PR projects over 400, stop and report to the TL.

| Task | Result | Check | PR |
|---|---|---|---|
| T1 | Kit package for the three spec files | every row has a disposition; RULEBOOK read-only afterwards | 1 |
| T2 | Failed-turn path stops the actor (error, panic, nil) | S1 to S3 red, then green | 2a |
| T4 | S9/S10 of #847 and S6/S10 of Slice A (D-7) rewritten, README limitation replaced | existing suite green; every original assertion kept; README meets the module README contract and does not claim S4 to S8 | 2a |
| T3 | Ordering, descendants, Goexit, Classic regression | S4 to S7 | 2b |
| T5 | `PostStop` test S8 (mandatory) | S8 passes against Slice A's `PostStop` | 2b |
| T6 | Execution record, cost-log row, gates bound to the final SHA | each gate comment names SHA, command, environment, exit code; per-scenario RED evidence of PR 2b | 3 |

PR 2b is test-only, so it cannot show red-to-green by itself. Its body says so; the red result that the Dev observed locally before the fix is
recorded per scenario in the execution record, and no RED is invented. S8 is mandatory for #956 and is delivered by PR 2b: #956 is not done
before it is integrated.

Stops: a file outside the allowed list must change; `pekko-actor` needs an export (e.g. a logger accessor); a runtime fact contradicts S4 or
S5; a scenario fails and is not a Go defect.

## Decisions for G2

Decided by the Owner on 2026-10-28. The alternatives the TL presented are in the first commit of this file (`a22dcd3`).

- **D-1, sequencing: (a).** #956 stays blocked until its `PostStop` prerequisite is delivered (precisely defined by D-6); neither Dev nor QA
  starts before. `PostStop` (S8) is mandatory for #956, and the gate evidence for it may be a follow-up bound to a SHA.
- **D-6, the prerequisite is `PostStop` of Slice A of #954 (Owner, 2026-10-28; wording from the scope of #954, PR #1234).** #956 depends on
  `PostStop` of Slice A **integrated into `develop` and verified** (a QA verdict and local gates bound to the merged SHA), not on the closure
  of #954. It replaces the ambiguous "until #954 is delivered" of D-1 (a). S8 needs only that slice and stays mandatory for #956. Slice B of
  #954 and its pending decisions (B-2, a watch source) do not block #956.
- **D-7, Slice A tests S6 and S10 move to a surviving restart trigger (Owner, 2026-10-28; a delta after G2, to be re-confirmed on #1233).**
  With M-B a failed turn stops the actor, so the restart of a *started* Typed actor that those two tests forced by returning an error from a turn
  no longer happens. The Owner authorized a read-only spike and, if it worked, widening the allowed files by exactly the one file above.
  **Spike result (TL, 2026-10-28, scratch copies of `develop` `8ebd25a` and of the Dev's local M-B commit `78be740`, nothing written to the
  repository):** a Classic parent whose strategy is `actor.AllForOne(Budget{MaxRetries:-1, Within:-1}, decider)` and that owns a Classic sibling
  and the Typed child. The sibling returns an error; the decider answers `Restart` once, then `Stop`. AllForOne restarts every child, so the
  *started* Typed child is restarted without failing any turn. Observed, on both trees, three consecutive runs: (S10 analogue) `Setup` ran twice,
  the next message was handled by the new instance, the old instance got no `PostStop`, the new one exactly one, no extra log entry; (S6 analogue)
  the restart's `Setup` failed, the decider stopped the child, `Done` closed and the handler was called 0 times. Removing the no-op `PreRestart`
  makes the S10 analogue fail with `got "i1:poststop", want "i2:ping"`, so the assertion bites. The spike file was kept outside the repository
  (`/tmp/gp956/spike_test.go`, read by the Dev) and is not a deliverable.
  **Rule for the Dev:** rewrite S6 and S10 on that trigger keeping **every** assertion of the originals: nothing weakened, nothing deleted, nothing
  skipped. Plain `testing`; `go.mod` and `go.sum` do not change (a migration of the whole module to go-specs is a separate later PR, not created
  here). Rejected for now: (b) retiring or re-homing S6/S10 to #976/#1183, which would need explicit Owner approval and an evidence-gap record;
  (c) a test-only seam.
- **D-8, four PRs instead of three (TL, under the stop rule of "Delivery, size and tasks"; recorded with D-7 so the Owner re-confirms once).**
  D-5 accepted "three PRs within the 400-line budget"; the budget clause forces one more. Not a change of behavior or of scope. If the Owner
  prefers a different cut, only the PR column changes.
- **D-2, mechanism M-B only.** Criterion 1 is restated to M-B. The Typed-parent rows leave this story with explicit traceability (section
  "Traceability of what leaves this story"), not as a silent drop.
- **D-3, failure logging (contract 13) is pending.** It is proposed as pending in the scope and the README; it is not approximated with a TODO
  and no `pekko-actor` export is added.
- **D-4, #847's S9/S10.** The "turn fails, actor stops" half is kept as Stop. The "previous behavior survives under Resume" half is an
  obligation of #1183, recorded in the traceability table.
- **D-5, three PRs**, within the 400 authored-line budget the repository requires.
- **Labels.** #956 keeps `status:blocked`. This PR stays a draft. This scope changes no label and does not edit any issue.

## Traceability of what leaves this story

Nothing is dropped silently. Except for the `ChildFailed` row, whose destination the Owner accepted, the destinations below are **proposals of
the TL**: creating a story or editing an issue is a separate Owner decision, and this PR edits no issue.

| Row that leaves #956 | Why | Proposed destination | State of that destination |
|---|---|---|---|
| Explicit `Resume` continues without rerunning setup (issue criterion 2) | ACT2 split | #1183 | open, blocked |
| Most specific strategy (`Supervise`, `OnFailure`) | ACT2 split | #1184 | open, blocked |
| **Obligation carried to #1183 (D-4):** prove that under explicit `Resume` a failed turn keeps the previous behavior, the next message is handled by it, and the cause reaches the strategy (today's #847 S10 second half) | the premise of that test is superseded by M-B | #1183 | its criteria must add it; until then the obligation is open and is tracked here |
| Typed parent applies the stop; `ChildFailed` to a watching Typed parent (M-A; `WatchSpec.scala:100`, `:130`) | needs a Typed spawn, a Typed watch and the signal types | **A separate M-A / `ChildFailed` story** ("a Typed parent applies the default stop and receives `ChildFailed`"), **not #956**. Accepted by the Owner on 2026-10-28 (decision D-5 of the scope of #954, PR #1234; not this scope's D-5). It depends on Slice B of #954, #956, #1170 and #1171, and nothing upstream waits on it, so there is no cycle. It supersedes the line of the 2026-10-26 reconciliation that moves `ChildFailed` to #956. This does not approve B-2 | accepted as a destination; **the issue does not exist yet and has no number**, and no issue is created or edited here. The rows are not covered by #956 |
| Failure logged at ERROR by the default stop (contract 13; `SupervisionSpec.scala:387-396`) | no per-actor logger accessor; D-3 | proposed: #1179 (Typed context logger), with the strategy-logging options (`withLoggingEnabled`, `withLogLevel`) to their strategy stories | open; pending, not scheduled |
| Start failure reported to the parent (matrix row READ) | unchanged, recorded as a gap | none; documented in the README and the execution record | gap |

## Definition of Done

- S1 to S8 pass at the final SHA, and each task meets its check. S8 (`PostStop`) is part of #956's Done (D-1 (a)); its gate evidence, bound
  to the SHA, may arrive in a follow-up commit or comment, but it may not be omitted. S8 is written against `PostStop` of Slice A of #954,
  integrated and verified (D-6).
- S6 and S10 of Slice A (`signal_runtime_test.go`) still pass, rewritten on the AllForOne trigger of D-7 with all their original assertions.
- Gates of "Evidence and validation" are green with evidence bound to that SHA; no function above complexity 10.
- The kit execution record states what was read, not run, and which rows are READ and pending approval.
- README matches the code; no sibling story and no parity is claimed.
- A new QA session, independent of the TL and the Dev, issues a verdict on the exact head of each PR.
- No merge without the Owner.
