# Execution record: Supervision U3 (#448)

Branch `docs/u3-package`, base `0bbc65d` (U2 merged), package commit `305a3e8`. Run under the reduced flow of RUNBOOK section 3 with the Owner's authorization of 2026-10-15. Nothing here claims Pekko parity, a current judge matrix, or a module-level result.

## 1. Baseline

Branch, clean tree and `origin/develop` (`0bbc65d`, already in HEAD) checked before starting. The five approved hashes (RULEBOOK, settings template, run-agent, manifest, RUNBOOK) matched the files and were written outside the repository. Kit at `cf91c9d`. Tree of `pekko-actor/actor` at the baseline `33e1f8bd79b298fa916cbe81efea4da4dc26c423`, 67 `.go` files hashed.

## 2. Agents and tokens

Tokens are input + output + cache creation, from the CLI JSON or the agent result. Coordination has no telemetry and is not estimated here.

| Agent | Model | Launcher | Tokens | Notes |
|---|---|---|---|---|
| Implementer | sonnet | `run-agent.sh`, `AGENT_WALL_MIN=30`, approved permissions | 290,172 (USD 1.88, 62 turns, 519 s) | Forecast 220,000: over by about 70,000. It could not run `go test` (denied by configuration); the coordinator ran every test. |
| Final review | opus | Agent tool, read-only | 170,685 (25 tool uses, 323 s) | Forecast 90,000: over by about 80,000. Not run through `run-agent.sh` because its approved permissions do not allow `git diff <range>`. |

Agents total 460,857, above the evaluation stop of 450,000. The Owner then authorized the corrections by the coordinator with 40,000 more tokens of coordination and no new agents. The corrections were not resumed through the implementer because `run-agent.sh` uses `--no-session-persistence`, and a fresh agent would reload about 200,000 tokens. Active time from the implementer launch to this record: about 29 minutes.

## 3. Verification

- Normal suite `go test -count=1 ./...` of `pekko-actor`: pass (final code).
- Targeted `-race` over `Goexit|TestDefault|TestOnTerminated|TestSup2InterimPaths|TestDeathWatchSpec`, `-count=1`: pass on the final code. Earlier runs on the first commit and after the review corrections also passed; the last one was repeated on the final code of the follow-up (it added `TestGoexitInPostStopOfARestartLeavesNoState` and a test-only refactor), with the regex widened to every `TestSup2*` and `TestSup1Interim` case. No `-count=20`.
- `gofmt -l`, `go vet`, `go mod tidy -diff`, `scripts/check_deps.sh`, `check-headers.sh` (29 ported files), `check-adr-signatures.sh`, `modules_selftest.sh`, `go-sdk-update.sh --check`: pass.
- Coverage of `pekko-actor` 98.60% (minimum 85%). The other modules of `check_coverage.sh` show FAIL only because no profile was generated for them in this run.
- Judge anchors: the 22 `(file, anchor)` pairs of `judge/mutate_go.py` each occur once; `judge/` was not edited.
- **`golangci-lint` v2.14.0 (pinned, installed under a temporary `GOBIN`).** On the first closing run it reported one finding, `gocyclo` 11 in `TestSup2FailedFatally` (`supervision_restart_internal_test.go:555`); U3 had not touched that file and the same lint on a `git archive` of the base `0bbc65d` reported the same finding, so it predated U3. The Owner then authorized fixing it without `//nolint` and without changing the threshold (section 5). After that change: **0 issues** on the final code.

## 4. Review (one Opus reviewer) and corrections

No blocker. The reviewer found the Goexit handover, suspension balance, decider-Goexit finality, routing and the U1/U2 regressions clean. Findings and what was done, by the coordinator, with focused re-runs and without a second Opus pass:

| Id | Finding | Result |
|---|---|---|
| F1 major | A Goexit in the `PostStop` of the default `PreRestart` was treated as a `PreRestart` failure and stopped the child. | Fixed: `PostStop` runs under its own guard (`postStopOfRestart`), a Goexit or panic is `poststop-panic`, and the restart continues. The Goexit keeps unwinding through the outer guard, so both handlers fired in the first attempt (`prerestart-failed` was emitted and a race made the test time out); the cell flag `postStopGoexit` now tells the outer handler it was already handled. Test `TestGoexitInPostStopOfARestart`. |
| F2 major | Nothing observed that a death pact or an `OnTerminated` failure reaches the decider. | Fixed in tests: P1 asserts the decider saw `ErrDeathPact`; `TestOnTerminatedFailureReachesTheDecider` covers an error (default Restart) and a Goexit (decider sees `ErrGoexit`, Restart loses nothing). |
| F3 minor | The restart-time branch of `endCreateOnGoexit` was never executed. | Test `TestGoexitInPreStartOfARecreate` (PreStart error, then Goexit in the re-Create, then a later failure restarts again). |
| F4 minor | PreRestart after an `OnTerminated` failure got an older message. | Verified against Pekko (`ActorCell.autoReceiveMessage`: `currentMessage` is the `Terminated`). `terminatedFailed` now sets the `Terminated` as the failed message; covered by the F2 test. |
| F5 minor | Stale comment and a V14 case that duplicated another. | Comment fixed; the V14 case is renamed to what it checks (its assertions are unchanged). It was renamed, not removed. |

The corrected findings were verified by tests and by the coordinator, not by a second review pass of the reviewer (budget); the Opus verification step of the plan ran afterwards, on the Owner's request, as a focused read-only pass over `git diff 6689859 ac76b20` (section 4b). For F1 the coordinator additionally checked that `postStopGoexit` does not leak into later restarts: by structure, its only writer (`postStopOfRestart`) is reachable only through `defaultPreRestart`, which is called only inside the guard of `runPreRestart`, whose handler `preRestartGoexit` clears it on the same goroutine; and by behavior, `TestGoexitInPostStopOfARestartLeavesNoState` (a handled PostStop Goexit, then a later restart whose custom PreRestart ends its goroutine, is still `prerestart-failed` with the decider asked with `ErrGoexit`, and the flag is false at the end). No native review was requested or granted for any of this.

## 4b. Final checks on `ac76b20` and focused Opus verification

- `go test -race -count=1 ./...` of `pekko-actor`, no `-run` filter: pass in `actor`, `internal/deadletter` and `internal/mailbox`. CI of the PR head (`go`, `label`): pass.
- Focused Opus verification (read-only, same reviewer role, tokens 130,229, 15 tool uses, 181 s). **It exceeded the 100,000-token limit the Owner set for it by about 30%**; it was not interrupted and the overrun is recorded here. Agents total is now 591,086 tokens (460,857 + 130,229). It could not run tests.
- Verdicts: **F2, F3, F4, F5 FIXED. F1 FIXED WITH A NEW MINOR PROBLEM.** For F1 it confirmed, by reading: one `handOver` only, made by `preRestartGoexit`; the flag has a single writer on a path that always reaches its clearing handler before any decider or handover; the `beginWait`/`waitDone` handler matches the original `faultRecreate` remainder with suspension, `sysq` and `restarting` balanced; and the two F1 tests fail if the guard, the flag or its clearing are removed. It also checked that the F2 and F3 tests fail if their fix is reverted. It could not verify that the tests pass or are race-clean (it did not run them; section 4b above did), nor the handover when a child finishes inside the `beginWait` to `handOver` window (only the static ordering).
- **New minor finding, not fixed (parity of an event kind only):** a panic in the `PostStop` of the *default* `PreRestart` now emits `poststop-panic`; at `6689859` it emitted `prerestart-failed`, which is what Pekko reports (a `PreRestartException`, because `Actor.preRestart` calls `postStop()` unguarded, `FaultHandling.scala:106`). The restart behaves the same either way, and no existing test asserts either kind for this case. The F1 fix only needed to change the Goexit case. Options for the Owner: keep the guard and emit `prerestart-failed` for the panic path, or accept the new kind as a recorded deviation. **Closed afterwards by the Owner's authorization (commit `713a73b`, section 4c).**
- An edge the reviewer noted and I keep as is: a `defer` inside a user `PostStop` that panics during a Goexit unwind is classified as a PreRestart failure, as before the F1 fix and as `callPostStop` does for a stop.
- `supervision_restart_internal_test.go:24` still names `PORT-GAP(death-pact-U3)` for `SupervisorMiscSpec.scala:144-163` (a custom-strategy case not ported); the death pact itself is closed by U3, so the tag there is now misleading. It is a U2 header and was not edited.

## 4c. Event-kind fix (`713a73b`)

A panic in the `PostStop` that the default `PreRestart` calls is now reported once as `prerestart-failed` (the guard absorbs it, so `runPreRestart` does not report it again) and the restart continues. The `PostStop` of a normal stop keeps `poststop-panic`; the Goexit handling and `postStopGoexit` are unchanged. New test `TestPostStopPanicEventByPath` separates both paths and checks the event, the single report, the continuation (instance 2 processes both queued messages, no `stop-accepted`, suspension and `sysq` at zero). Negative control on a copy outside the worktree: with the old event name the restart-path case fails (`prerestart-failed` count 0, expected 1).

Verified on `713a73b` by reading and specific tests, not by a new review pass (as the Owner asked; the focused Opus verification of section 4b is kept and covered `ac76b20`): targeted tests (including the Goexit PostStop tests), `gofmt`, `go vet`, pinned `golangci-lint` 0 issues, and `go test -race -count=1 ./...` of `pekko-actor`, once on the final code: pass. No new agents, campaigns or native review. CI of the published head is recorded in the PR.

## 4d. Native review of the accumulated range (granted by the Owner)

The Owner granted, by his own answer to the consent envelope, a native review of the accumulated range `0bbc65d..063d99a` (the provider offered no per-range start, so R1-R5 could not be reviewed separately). Target `sha256:5e061db3e6cdd05dc5eb03583f23cc8327584bf317a16880b4b1c59a00bc8b6b`, lineage `review-433d46e4d9663901`, risk high, 26 files, 1,794 changed lines, four lenses (risk, resilience, readability, reliability), no correction opened.

- **Outcome: approved.** The last admitted capture closed it as `approved`; the exact acknowledgement ran and reported authority `burned` (`gentle-ai.review-acknowledged/v1`). The approval is informational: it does not authorize a merge, and no merge was done.
- **Advisory findings (11, all non-blocking, none opened a correction):** risk none; resilience R4-001, R4-002; readability seven (R2-*); reliability two (R3-*). They are separate later work and are not a reason to re-run review on this candidate. Their treatment follows.

| Finding | Treatment |
|---|---|
| R2-readme-otherwise-stops, R3-readme-stale-otherwise-stops (`pekko-actor/README.md:20`) | Fixed in the documentation commit that follows: the sentence now says the parent's decider decides, with `DefaultStrategy` when it declares none, and only a top-level actor stops directly. |
| R2-docgo-finite-budget (`doc.go:7`) | Fixed (comment only): the Budget is the strategy's, and the default one is unlimited, without backoff. |
| R4-002 (unlimited restarts, no backoff, no logging) | Documented in the module README limitations and linked to logging-U9, issue #451; `PORT-GAP(logging-U9)` stays open. No behavior change. |
| R4-001 (a child stays suspended until the parent handles `Failed`, so a parent blocked in `Ask` to its own child waits for the deadline) | Documented in the module README limitations as a behavior change of U3. No code change; whether it needs a design decision is open for the Owner. |
| R2-run-agent-token-cap (`run-agent.sh:22` says 900,000 / 720,000, RUNBOOK section 3 says 560,000 / 450,000) | **Not edited**: the file is under an approved hash. Follow-up: correct the header in the next package, and read the cap from RUNBOOK, not from the script. The token cap was never mechanical in the launcher (it says so), so no run used 900,000 as a limit. |
| R2-rulebook-hash-section (RULEBOOK cites RUNBOOK "section 9", the hashes are in section 11) | **Not edited**: hashed file. Follow-up in the next package. |
| R2-resume-stale-status (`RESUME.md` says execution is NOT authorized) | Follow-up: `RESUME.md` is outside the hashes but is a package file; this record is the current state, and `RESUME.md` is superseded by it. Not edited here to keep this change to what the Owner listed. |
| R2-quality-gates-where-column (`docs/quality-gates.md` table lists CI jobs that no longer run) | Follow-up: align the table with `docs/ci-economy.md`. Not edited here. |
| R2-stale-death-pact-portgap (`supervision_restart_internal_test.go:24`) | Follow-up: a U2 test header, and this change edits no test. |
| R3-weak-decider-goexit-assertion (`supervision_default_goside_internal_test.go:181`) | Follow-up: strengthen the assertion to filter by the parent's id and check `ErrGoexit`. No test was changed here. |

Documentation-only delta after the review: `pekko-actor/README.md`, `pekko-actor/actor/doc.go` (comment), this record and the PR body. No Go code, test or approved-hash file changed. The delta is not itself natively reviewed. If the provider offers a new start or consent envelope for it, it is relayed to the Owner and not answered.

## 5. Manual changes

- **Scope extension authorized by the Owner:** the W21 case of `TestSup2InterimPaths` in `supervision_restart_goside_internal_test.go` (a 14th surface). The decider now restarts on the original failure and stops on `ErrGoexit`; it keeps checking that the cell terminates after exactly one `prerestart-failed`. The case does not replace the U3 continuation tests for Restart and Resume (`TestGoexitReachesTheDecider`, `TestGoexitResume`). No assertion was weakened and the runtime was not changed to make it pass.
- **Second scope extension authorized by the Owner (#456 follow-up):** `supervision_restart_internal_test.go`, only `TestSup2FailedFatally`. The four occurrences of `errors.As(err, &ie) && ie.InRestart` became a call to the new helper `isInRestartInit(err)`, which holds the same condition; the scenarios and every assertion are unchanged and no `//nolint` or threshold change was made. That brought `gocyclo` of the function under 10 and lint to 0 issues.
- **`supervision_hooks.go` (Owner-authorized, comment only):** the stale `PORT-GAP(death-pact-U3)` text on `FailedMessage` now says that a death pact or an `OnTerminated` failure gives the `Terminated` as the failed message and that a top-level actor stops directly. The README sentence that called it stale was removed.
- Changed assertions of `TestSup1Interim` (V14): each case now fails a top-level actor, explained in comments.
- Documentation by the coordinator, once at close: `pekko-actor/README.md`, `pekko-actor/actor/doc.go` (comment only), `docs/actor-core-parity.md`, root `README.md`. CURRENT stays stale and the text says so; the S13/S14 "out of scope" rows stay until U8.
- Scope control after the fact (`git diff --stat` and `shasum` against the baseline): 9 modified and 2 new `.go` files, all within the surfaces plus W21; no other `.go` file of `pekko-actor/actor` changed before the documentation commit. This is a post-hoc control, **not a test of isolation**: it shows what changed, not that the permissions would have blocked another write. No Haiku probe was run.

## 6. Declared gaps and pending

- Pending until the module closes: mutants M1-M15, judge campaign and its tag, complete `go test -race`, stress runs, integral review, mirror of S13/S14/S16 (U8, #175), inventory of the Pekko surface.
- Carried from U2: #173, mutants K27, K28, K33 and the open n3 exception.
- New in U3: `PORT-GAP(factory-goexit-U4)`, `PORT-GAP(top-level-default-U5)`; a Goexit in the factory of a restart leaves `restarting` set on a terminating cell (harmless, tracked by the factory gap). Closed after the PR was opened: the inherited `gocyclo` finding and the stale `PORT-GAP(death-pact-U3)` comment (section 5). `supervision_restart_internal_test.go:24` still names `PORT-GAP(death-pact-U3)` for `SupervisorMiscSpec.scala:144-163`, a custom-strategy case not ported; that U2 header was not edited.
- The event-kind finding of section 4b is closed (section 4c); the delta after the Opus pass was verified by tests only. Pending, not run: the module-close campaigns and everything in the first bullet. U3 evidence is the tests and gates of section 3 on the final code; no CURRENT or parity claim.
- No native review was requested: the Owner did not authorize an accumulated one, and no consent envelope was answered or assumed.
