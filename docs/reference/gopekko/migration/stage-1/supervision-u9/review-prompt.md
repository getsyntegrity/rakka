# Task: one focused, read-only review of delivery U9 (`loggingEnabled` and `logFailure`; #451)

You are the only reviewer of this delivery. You are read-only: no Edit, no Write. You can run `git status`, `git diff` (also with a path, for example `git diff ad98f7f -- pekko-actor/actor/supervision_failure.go`), `git diff --stat`, `git show <rev>:<path>`, `go vet`, `eza`, `wc`, and read any file with the Read tool. You cannot run tests. You run non-interactively and a **spend cap is enforced by the launcher** (USD and wall clock): when it is hit your session ends with whatever you have. Work in this order and keep your findings as you go:

1. `git diff ad98f7f --stat` (the base of this delivery; the code may be committed or not), then the diff of the production files: supervision_logging.go, supervision_failure.go, supervision.go, guardian.go, system.go, supervision_restart.go. Judge these before anything else.
2. migration/stage-1/supervision-u9/RULEBOOK.md sections 2 to 7, to check the diff against D1-D7 and L1-L16.
3. Only then the tests: supervision_logging_internal_test.go and supervision_logging_api_test.go.
4. Pekko only to settle a doubt: `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala` 359-428.

## What to look for (assume the work is wrong)
- **Levels and silence:** Resume `Warn`, Restart and Stop `Error`, Escalate, `ok == false` and a nil decider nothing; a denied Restart still logged; AllForOne once per decision; stale child, a panicking or Goexit decider and an invalid directive not logged. Is the call really **before** the directive is applied and **after** the decider returned?
- **`loggingEnabled`:** the zero value logs (the flag is stored negated); `WithLogging(false)` suppresses every case, the guardian strategy included, and changes no decision and no event. **The flag survives every place that replaces a zero value by the default:** `strategy()` of the cell, `GuardianStrategy.strategy()`, the fallback of `startGuardians` and the test-seam fallback of `newSystem`; `WithLogging(false)` on a zero `Strategy`, on `DefaultStrategy` and on a zero guardian strategy silences the effective default; explicit constructors and `WithLogging(true)` log.
- **Order and `Error()`:** the AllForOne snapshot before the decider, then the decider, then the log, then the application; `cause.Error()` is evaluated only for a logged decision and after the decider; the log consumes no budget and does not change suspension counts, activation or hand-over.
- **Logger failure:** the message computation and the logger call are both under `protect`; a panic is absorbed; a **Goexit of the logger still applies the decision exactly once** (not zero, not twice) and hands the cell over; no event added.
- **Exported surface is exactly** `Config.Logger`, `Strategy[T].WithLogging`, `GuardianStrategy.WithLogging`; constructors unchanged; `Strategy` still an opaque value (a future #450 is not blocked); the Logger is not stored in `Strategy`.
- **Messages:** an `*InitializationError` shows its underlying error's text; the `error` attribute only on Error entries.
- **Events untouched:** no `emit` changed or added; no log is used as an event or the reverse. The existing tests pass without edits (check `git diff --stat` for edits to existing test files).
- **Tests assert what they claim** and would fail under the nine mutants of RULEBOOK 7 (log on Escalate, `loggingEnabled` ignored, levels swapped, an unprotected logger call, AllForOne per child, nil Logger dereferenced, the zero `Strategy` losing the flag, `Error()` evaluated too early or when disabled, the decision applied before it is logged). A vacuous test is a DEFECT. The mock's `Entries` is read only after the executor drained (a data race otherwise).
- Nothing outside the surfaces changed; `go.mod`, `go.sum` and `judge/` untouched (the coordinator's seed excepted).

## Output
Findings only, classes BLOCKER / DEFECT / INFO, each citing a RULEBOOK rule or a source `path:line`. INFO is recorded, not acted on. **The whole final report is at most 1,500 tokens.** If you could not read something, say exactly what, so it is recorded as unreviewed. No praise, no restatement of the diff, no second pass: this is the only review of this delivery.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
