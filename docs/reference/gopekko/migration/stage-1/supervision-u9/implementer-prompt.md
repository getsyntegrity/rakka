# Task: implement delivery U9 (`loggingEnabled` and `logFailure` of the strategies; #451) in pekko-actor/actor

You are the single implementer of this delivery: production code and tests, one session. You run non-interactively: nobody can answer questions. This is **U9 only**. U1 to U8 are merged in `develop` (`ad98f7f`) and are your base. Do NOT touch the judge (`judge/`), the Pekko side, `go.mod`/`go.sum` (the coordinator already seeded the `kit-logger` requirement and the module cache; you work offline), user-defined strategies (#450), the restart-hook logs (`PreRestartException`, `runPreRestart`) or any file outside the surfaces.

## Read first, in this order (do not re-plan or re-map)
1. migration/stage-1/supervision-u9/RULEBOOK.md (approved, read-only): all of it. Sections 2 (Pekko), 3 (Go facts), 4 (D1-D7: read the second paragraph of D2, the order and the `Error()` timing of D3, and the Goexit rule of D4 twice), 5 (scope), 6 (tests L1-L16 and the API test) and 7 (nine mutants) bind you.
2. migration/stage-1/supervision-u9/RUNBOOK.md: sections 1 and 6 (surfaces).
3. Pekko, read-only: `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala` 359-428 (`handleFailure`, `loggingEnabled`, `logFailure`, `publish`), 468-472, 582-586.
4. kit-logger, read-only, in the module cache: `/Users/pablogore/go/pkg/mod/github.com/pablogore/kit-logger@v0.1.2-0.20260912231430-17d1a6eacc85/pkg/logger/interface.go` (the `Logger` interface) and `.../pkg/logger/kitlogtest/mocklogger.go` (`NewMockLogger`, `Entries`, `HasMessage`; `With` returns the same mock, so attributes added with `With` are not recorded: pass attributes as call arguments).
5. The code: supervision_failure.go (`handleFailed`, `carryOut`, `apply`, `failParent`), supervision_restart.go (`applyRestart`, `applyRestartAll`, `stopRound`, the PORT-GAP(logging-U9) comment at `applyRestart`), supervision.go (`Strategy`, constructors, `Supervisor`), guardian.go (`GuardianStrategy`, its conversion to `Strategy[guardianMsg]`), system.go (`Config`, `System`, `newSystem`), and `protect` in cell.go (read only).
6. Test helpers to reuse before writing new ones: `ctlExec`, `sup1Rig`/`sup1NewRig`, `sup1Hooked`, `sup2NewRig`, `sup1Fixed`, `journal`, `sup1Count`, `settle`, and the sink used by the supervision tests (`options.sink`).

## Allowed edit surfaces (the only files you may write), all under pekko-actor/actor/
Production (6):
- supervision_logging.go (new: `(*System).logFailure`, the message function, the protected call)
- supervision_failure.go (the single call site in `handleFailed`, in the order of RULEBOOK 4: snapshot, decider, log, apply; and `strategy()`, which must carry the `noLog` flag when it returns the default for a zero `Strategy`)
- supervision.go (the unexported `noLog` field and `func (s Strategy[T]) WithLogging(enabled bool) Strategy[T]`; the doc comments that say `loggingEnabled` is absent)
- guardian.go (the same flag and `func (g GuardianStrategy) WithLogging(enabled bool) GuardianStrategy`; the conversion `strategy()` and the fallback of `startGuardians` carry the flag)
- system.go (`Config.Logger kitlog.Logger`, `System.logger`, the wiring in `newSystem`; the test-seam fallback at `user.kind == 0` carries the flag)
- supervision_restart.go (comment only: the `PORT-GAP(logging-U9)` at `applyRestart` is resolved; leave the one at `runPreRestart` and say in the comment that it is the restart-hook log, not a strategy log)
New tests (2): supervision_logging_internal_test.go (L1-L12), supervision_logging_api_test.go (`package actor_test`).
Everything else is denied. **If you need a file outside the surfaces, stop and report `BLOCKED(surface): <file and why>`.** In particular, if the import of kit-logger does not compile offline, report the exact error and stop: do not edit `go.mod`.

## What to deliver (RULEBOOK 3-6)
- `kitlog "github.com/pablogore/kit-logger/pkg/logger"`. D1: `Config.Logger kitlog.Logger`, nil = no logs. D2: `WithLogging` on `Strategy[T]` and `GuardianStrategy`, stored as the **negation** so the zero value logs, and **independent of `kind`**: `WithLogging(false)` on a zero `Strategy`, on `DefaultStrategy` and on a zero or default guardian strategy disables the logs of the effective default; the four fallback places listed in RULEBOOK 4 carry the flag. D3: one entry per decision, in `handleFailed` after the decider returned normally and before `carryOut`; Resume `Warn` (no `error` attribute), Restart and Stop `Error` (with `error`), Escalate / `ok == false` / nil decider nothing; a denied Restart still logged; AllForOne once. Message: the cause's text, for `*InitializationError` its underlying `Err` text (a fixed text if nil). D4: the whole function (message and logger call) under `protect`; absorb a panic; a Goexit of the logger still applies the decision **exactly once** (the handler applies it and hands the cell over; the normal path applies it after the log; never both, never neither). `cause.Error()` is evaluated only inside the logged decision, after the decider (RULEBOOK 4). D5: do not touch any `emit`.
- Keep the exported surface to exactly: `Config.Logger`, `Strategy.WithLogging`, `GuardianStrategy.WithLogging`. No other exported identifier. Doc comments in English, no parity claim; list the attribute names in the doc comment of `Config.Logger`.
- Tests L1-L16 and the API test exactly as RULEBOOK 6. Name **every** test and example with `Logging` in its name (the wrapper selects them by that pattern). Read the mock's `Entries` only after the controlled executor drained. The coordinator will run nine mutants against your result (log on Escalate, `loggingEnabled` ignored, levels swapped, an unprotected logger call, AllForOne logging per child, a nil Logger dereferenced, the zero `Strategy` losing the flag, `Error()` computed too early or when disabled, the decision applied before it is logged): write assertions that would fail under each (the level, the count, the message, the attributes), not just "something was logged".
- All existing tests must pass without edits.

## How to verify (you may run exactly these)
`gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...`, `go build ./pekko-actor/...`, and **one scoped test run**: `bash migration/stage-1/supervision-u9/run-logging-tests.sh` (D8: the `Logging` tests with `-race`, once). `go test` is otherwise denied; you cannot run the whole suite, the judge or mutants (the coordinator does).

## Hard constraints
- No git write commands, no commits.
- Do not weaken the invariants of U1-U8. If one concrete decision is unresolved and not decidable from the RULEBOOK, Pekko or the code, report `NEEDS-DECISION: <the single question, the options, your recommendation>`.
- Functions at cyclomatic complexity 10 or less (including literals), no `//nolint`. About 400 authored changed lines per task is a heuristic, not a cap. Time bound: 30 minutes wall. The delivery is capped at 560,000 tokens including coordination: read only the files listed above.

## Final report (your last message), at most 1,200 tokens
1. Files changed (path and one line each). 2. gofmt, vet, build and the scoped test run: results (the run's output, trimmed). 3. Per test id L1-L16 and the API test: the Go test name. 4. The exported identifiers added (exactly three). 5. Confirmation that no file outside the surfaces changed and `go.mod` is untouched. 6. Open doubts and what you could not verify. 7. If blocked: the BLOCKED or NEEDS-DECISION line first.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
