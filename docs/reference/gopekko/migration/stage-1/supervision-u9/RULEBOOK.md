# RULEBOOK: Supervision U9, `loggingEnabled` and `logFailure` of the strategies (#451)

Status: PROPOSED, for the Owner's approval together with the package (`RUNBOOK.md`). Unit U9 of ADR 0017 (S14, A11). Base: `develop` at `ad98f7fc3e7a529d5ebc156daf1eda0407ff774d` (verified with `git rev-parse origin/develop` on 2026-10-16). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`; upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`). Paths are relative to the repository root; `FH` is `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/FaultHandling.scala`.

Nothing here reopens a decision accepted in ADR 0017; in particular the sink is `github.com/pablogore/kit-logger` (owner, 2026-10-14) and events stay a separate channel (A11). A row marked **NEW** is a decision this unit asks the Owner to accept. The message that asked for this package reached me partly unreadable; section 11 says how I read it. No parity is claimed.

## 1. Goal and what #451 requires
Preserve `loggingEnabled` and `logFailure` of Pekko Classic. #451 acceptance: (1) executed with kit `cf91c9d`, recorded; (2) `loggingEnabled = false` suppresses the log, Warning for Resume, Error for Restart and Stop, Escalate does not log, a logger failure does not break failure handling; (3) the existing events keep their meaning and are not used as a substitute, with tests that check both channels separately; (4) until then the divergence is declared in the parity doc. #451 also asks this RULEBOOK to resolve three design questions (section 4): how the `Logger` is injected, the level mapping, and how `loggingEnabled` is exposed without silently changing the API.

## 2. The contract to preserve (Pekko v1.7.1, read at the pinned commit)
| Fact | Evidence |
|---|---|
| `loggingEnabled` is `protected def ... = true`; `OneForOneStrategy` and `AllForOneStrategy` take it as a constructor parameter, default `true` | `FH:395`; `:468-472`, `:582-586` (overloads `:483-484,493-494,535-536`, `:595-596,605-606,644-645`) |
| `handleFailure` calls `logFailure` **before** applying the directive, in every branch (Resume, Restart, Stop, Escalate) | `FH:366-390` |
| `logFailure`: only if `loggingEnabled`; the message is the cause's message, and for an `ActorInitializationException` with a cause it is that cause's message (unwrapping an `InvocationTargetException`); **Escalate logs nothing**; **Resume logs at Warning** (`Logging.WarningLevel`), **Restart and Stop at Error** and the Error event **carries the throwable**; the source is `child.path.toString`, the class is the strategy's `getClass` | `FH:405-423`, levels `:136,148,156,162` |
| `publish` is wrapped in `try ... catch { case NonFatal(_) => }`: "logging is not the main purpose, and if it fails there's nothing we can do". The computation of the message is **not** inside that `try` | `FH:425-428` |
| The default strategy and the stopping strategy keep `loggingEnabled = true` | `FH:230-243` |
| `logFailure` is a public overridable `def`; `loggingEnabled` an overridable `protected def` | `FH:323-424` (this is #450 item 1, not U9) |
| AllForOne's `processFailure` runs after **one** `logFailure` for the failing child | `FH:554-567` |
| Only `UidClashTest:84` and `BackoffOnRestartSupervisorSpec:170` use `loggingEnabled = false`, and only to silence; no upstream test asserts it. Many specs assert supervision logs through `EventFilter` (for example `SupervisorMiscSpec:47,94,160,174`, `ActorRefSpec:156-278`) | `actor-tests/src/test/scala/org/apache/pekko/` |

## 3. The Go runtime today (read at `ad98f7f`)
| Need | Go fact | Evidence |
|---|---|---|
| decision point | `handleFailed` decides one failed item; a stale child returns early; the decider runs inside `protect`; a panic goes to `failParent`; otherwise `carryOut` applies Restart (`applyRestart`, `applyRestartAll`), Stop (`stopRound` under AllForOne) or `apply` (Resume, Stop, Escalate, invalid directive) | `pekko-actor/actor/supervision_failure.go:248-334`; `supervision_restart.go:179-256` |
| Escalate | `ok == false`, a nil decider and `Escalate` all call `failParent` | `supervision_failure.go:310-324` |
| the declared gap | `PORT-GAP(logging-U9)` at `applyRestart` (`supervision_restart.go:178`); the same tag at `runPreRestart` (`:315`) is a **different** log (Pekko's `PreRestartException`), not a strategy log | the file |
| strategy API | `Strategy[T]` has unexported fields, so a new unexported field is not an API change; constructors `OneForOne`, `AllForOne`, `DefaultStrategy`; `GuardianStrategy` converts to `Strategy[guardianMsg]` | `supervision.go:51-105`, `guardian.go:41-67` |
| System configuration | `Config` has `Throughput` and `GuardianStrategy` only; the event sink is the unexported test seam `options.sink`; a cell reaches the system as `c.sys` | `system.go:59-123` |
| events to keep apart | `receive-failed`, `restart-denied`, `restart`, `restart-failed`, `prerestart-failed`, `poststop-panic`, `stop-accepted`, `prestart-failed` | `supervision*.go`, `stop.go` |

## 4. Design (NEW decisions D1-D7)
**D1. Injection: `Config.Logger kitlog.Logger`** (`kitlog` = `github.com/pablogore/kit-logger/pkg/logger`). `NewSystem(Config{Logger: l})`. A `nil` Logger means **no supervision logs** (the library is silent unless configured). This differs from Pekko, whose default logger prints to standard output through the event stream; it is declared as a divergence (D1 alternative below). The `System` stores the logger; a cell reads `c.sys.logger`. The Logger is **not** in `Strategy`, so strategies stay values and #450 is not blocked.
**D2. `loggingEnabled`: a method, not new constructors.** `func (s Strategy[T]) WithLogging(enabled bool) Strategy[T]` and `func (g GuardianStrategy) WithLogging(enabled bool) GuardianStrategy` return a copy. The unexported field is the **negation** (`noLog`), so the zero value logs (Pekko's default `true`) and the zero `Strategy` (which means `DefaultStrategy`) logs. `OneForOne`, `AllForOne`, `DefaultStrategy`, `GuardianOneForOne` and `StoppingGuardianStrategy` keep their signatures; this is an addition. **The flag is independent of `kind`.** `WithLogging(false)` on the **zero** `Strategy` (which means "the default strategy applies") still means "no strategy declared" and therefore disables the logs of the **effective default strategy**; `DefaultStrategy[T]().WithLogging(false)` and a zero or default `GuardianStrategy` with `WithLogging(false)` do the same; `WithLogging(true)` and every explicit constructor (`OneForOne`, `AllForOne`, `GuardianOneForOne`, `StoppingGuardianStrategy`) leave logging on. Four places replace a zero value by the default and each must **carry the flag over**, otherwise `WithLogging(false)` on a zero value would be lost: `strategy()` of a cell (`supervision_failure.go:302-306`, a zero `Strategy` returned by `SupervisorStrategy()`), `GuardianStrategy.strategy()` (`guardian.go:67-69`), the fallback of `startGuardians` (`guardian.go:146-148`) and the test-seam fallback of `newSystem` (`system.go:118-120`). An actor that does not implement `Supervisor` gets `DefaultStrategy` with logging on (to silence the default, implement `Supervisor` and return `Strategy[T]{}.WithLogging(false)`). The `/system` and root guardians keep logging on when a Logger is configured.
**D3. What is logged (single place, `logFailure`).** One entry per **decision**, produced in `handleFailed` after the decider returned normally and **before** the directive is applied (as `FH:366-390`):
| Decision | Level (`kitlog.Logger`) | Entry |
|---|---|---|
| Resume | `Warn` | message = the cause's message; attributes `child` (the child's ID) and `directive` ("resume") |
| Restart | `Error` | same message; attributes `child`, `directive` ("restart") and `error` (the cause) |
| Stop | `Error` | same; `directive` ("stop"), `error` |
| Escalate, `ok == false`, nil decider | nothing | Pekko: "don't log here" |
**Order, in `handleFailed`, exactly:** (1) under AllForOne the **snapshot** of the children, with their restart statistics, is taken **before** the decider (unchanged from U6); (2) the decider runs under `protect`; (3) if it panicked or ended with Goexit, the parent fails and **nothing is logged**; (4) otherwise, if the decision is Resume, Restart or Stop, and logging is enabled for the effective strategy, and a Logger is configured, the decision is **logged**; (5) only then is it **applied** (`carryOut`: the budget is asked, the suspensions are added, the children are restarted or stopped). The log has no access to and no effect on the budget (`requestPermission` is called only in step 5, so a denied Restart is logged but charged as before), on the counted suspensions, on the snapshot, on the activation or on the hand-over of the cell.
**When `cause.Error()` is evaluated.** Only in step 4, after the decider returned and only for a decision that is logged: never before the decider, never when logging is disabled, when no Logger is configured, or for Escalate, `ok == false`, a nil decider, a stale child, a decider failure or an invalid directive. For a `*InitializationError` the text is its underlying `Err`'s `Error()`. An `Error()` that panics is absorbed (D4) and the entry is skipped or sent with a fixed fallback text (the implementer chooses one and tests it).
Not logged either (no `logFailure` in Pekko): a stale child (dropped before the decider), a decider that panics or ends with Goexit (the parent fails; the existing `receive-failed` event covers it), an invalid directive. A **denied** Restart is still logged as Error (Pekko logs the decision before `processFailure`). AllForOne logs **once** per decision with the failing child as subject, not once per child; the same holds for its Stop and for the denial that stops the whole round.
**Message.** The cause's `Error()`; for a `*InitializationError` the message of its underlying `Err` (Pekko: the `ActorInitializationException`'s cause), so a factory panic or a `PreStart` error shows its own text; an `InitializationError` with a nil `Err` uses a fixed text. Warning carries no `error` attribute (Pekko's Warning has no throwable); Error does. The attribute names are fixed by the implementer in one place and listed in the doc comment of `Config.Logger`.
**D4. A logger failure never breaks failure handling.** The whole `logFailure` (the message computation **and** the logger call) runs under the existing `protect`: a panic is absorbed, a `runtime.Goexit` of the logger is handled like the one of the decider is (a handler passed to `protect` plus the hand-over of the activation) with one difference: the decision **is still applied, exactly once**, and the item is not lost or applied twice. Concretely, the Goexit handler applies the decision (`carryOut` with the same arguments, and the same AllForOne snapshot) and then hands the cell over; the normal path applies it after the log returns; only one of the two runs. The order "decide, log, apply" and the suspension and activation accounting are the same on both paths. This is **stricter** than Pekko, whose `try` covers only `publish`; declared. No event is added or changed for a logger failure.
**D5. Events are a separate channel.** `emit` calls are untouched. The logger is never used to emit events and the sink is never used to log. A differential test (L6) shows that the events and the decisions are identical with logging on, off and with a nil Logger.
**D6. `logFailure` stays internal.** The single function `(*System).logFailure(...)` is the hook point that #450 item 1 will route to a user-supplied strategy; U9 adds no exported override.
**D7. Level mapping:** Pekko Warning → `Warn`, Pekko Error → `Error`. Pekko's custom `Directive.logLevel` is `private[pekko]` and out of scope.
**Alternatives rejected.** (a) A port in `spi/` with an adapter in `pekko-observability`: keeps the core free of kit-logger but moves the sink away from where the ADR and the Owner put it (`pekko-actor/go.mod`) and the real footprint is small (section 8). (b) `nil` Logger = a default kit-logger: prints by default, a surprise for a library and non-deterministic for tests. (c) A `Logger` inside `Strategy`: couples strategies to the sink and blocks #450.

## 5. Scope and boundaries
| In U9 | Not in U9 |
|---|---|
| `logFailure` for Resume, Restart, Stop and the silence for Escalate; `loggingEnabled` as `WithLogging`; `Config.Logger`; the guardian strategy (the failures of top-level actors are decided by `/user`'s strategy and go through the same path) | the `PreRestartException` / `PostRestart` failure logs (`PORT-GAP(logging-U9)` at `runPreRestart` stays; renamed in the docs as "restart-hook logging" and listed in #461); user-defined strategies and an overridable `logFailure` (#450); Typed logging; Pekko's custom `Directive.logLevel` |
| the ported upstream tests keep asserting events; the new tests assert logs | porting the `EventFilter` log assertions of the upstream specs (a follow-up in #461) |
The judge, `judge/` and the Pekko side are untouched. **Consequence on the matrix:** U9 edits `pekko-actor/`, whose tree the Go mutation report is bound to, so the Go **CURRENT** line stops applying until a campaign is re-run (section 9, D9).

## 6. Tests (new files; conventions of U2 RULEBOOK section 4: go-specs only, deterministic, no sleeps as coordination, English, no `//nolint`, gocyclo at most 10, coverage at least 85%)
Logs are asserted with `kitlogtest.NewMockLogger()`; its `Entries` is read **only after the executor drained** (it is not synchronized for concurrent readers), through a controlled executor.
| Id | Asserts | Kills |
|---|---|---|
| L1 | Resume gives exactly one `Warn` entry with the cause's message, `child` and no `error` attribute | ML3 |
| L2 | Restart gives one `Error` entry with the `error` attribute; also when the budget denies (`Budget{}`): still one entry | ML3 |
| L3 | Stop gives one `Error` entry | ML3 |
| L4 | Escalate, `ok == false` and a nil decider give **no** entry at the escalating parent, and the grandparent that decides gives one entry whose `child` is the parent | ML1 |
| L5 | a decider that panics or ends with Goexit, an invalid directive and a stale child give no entry; the `receive-failed` event is still emitted | — |
| L6 | **differential:** with `WithLogging(false)`, no entries for the four directives, and the sequence of events and the final state (restarts, stops, counters) equal those with logging on and with a nil Logger | ML2 |
| L7 | `*InitializationError`: the entry's message is the underlying error's text (a `PreStart` error and a factory panic) | — |
| L8 | AllForOne: exactly one entry per decision (permitted Restart, denied Restart, Stop), subject the failing child | ML5 |
| L9 | a Logger whose `Error`/`Warn` panics and a cause whose `Error()` panics: the decision is applied identically, no stall, no extra event | ML4 |
| L10 | `nil` Logger: no panic, no entry anywhere, behavior unchanged | ML6 |
| L11 | guardian strategy: a top-level failure is logged through `Config.Logger`; `GuardianOneForOne(...).WithLogging(false)` suppresses; `StoppingGuardianStrategy` logs Error | — |
| L12 | the zero `Strategy` and `DefaultStrategy` log (logging is on by default); `WithLogging(true)` after `WithLogging(false)` logs again | ML2 |
| L13 | **the flag survives the defaults:** a zero `Strategy` returned by `SupervisorStrategy()` with `WithLogging(false)`, `DefaultStrategy().WithLogging(false)`, a zero `GuardianStrategy` and `StoppingGuardianStrategy()` / `GuardianOneForOne(...)` with `WithLogging(false)`: no entry for Resume, Restart and Stop, with the decisions (default decider: Restart; Stop for an initialization failure) unchanged; the same values without `WithLogging(false)`, with `WithLogging(true)` and the explicit constructors: logged | ML7 |
| L14 | **`Error()` timing:** a cause that counts the calls to its `Error()`: zero calls when logging is disabled, when the Logger is nil, for Escalate / `ok == false` / nil decider, for a stale child and for a failed decider; exactly one call per logged decision; and when the decider runs the counter is still zero (the call comes after the decider) | ML8 |
| L15 | **order and accounting:** the logger observes, at the moment of the call, that the decision is not applied yet (the failed child is still registered and its suspension count is unchanged; under AllForOne no sibling is restarted or stopped yet); a denied Restart is logged and still charges the budget exactly once as without logging; under AllForOne the snapshot and the counters are equal with logging on and off | ML9 |
| L16 | **Goexit of the logger** (real executor): the decision is applied exactly once (one restart, one stop), not zero and not twice, the cell is handed over and keeps processing, and no event is added | ML4 |
| API | an external test (`package actor_test`) builds a system with `Config.Logger` and a strategy with `WithLogging(false)` using only exported API | — |
All the existing tests must pass **without edits**, which is the evidence that the events did not change meaning.

## 7. Directed mutants (run by the coordinator on copies outside the repository; not the judge)
| Id | Mutation | Must be caught by |
|---|---|---|
| ML1 | `logFailure` also logs on Escalate | L4 |
| ML2 | `loggingEnabled` is ignored (logs although `WithLogging(false)`) | L6, L12 |
| ML3 | levels swapped (Stop at `Warn`, Resume at `Error`) | L1, L2, L3 |
| ML4 | the logger call is not protected (a panic reaches the activation) | L9 |
| ML5 | AllForOne logs once per child of the round | L8 |
| ML6 | a nil Logger is dereferenced | L10 |
| ML7 | the zero `Strategy` loses the flag when it is replaced by `DefaultStrategy` (the `strategy()` of the cell, or the guardian conversion) | L13 |
| ML8 | `cause.Error()` is computed before the decider, or when logging is disabled | L14 |
| ML9 | the decision is applied before it is logged | L15 |
Rules as in U6/U7: one at a time, the unmodified copy passes, a mutant that does not compile or times out is INVALID (redesigned once, then reported), package timeout well above the helpers' 30 s waits (900 s).

## 8. Dependency and the `go.mod` seed
Static analysis of the pinned pseudo-version `v0.1.2-0.20260912231430-17d1a6eacc85` (the newest in the local module cache; the module has no tag): `pkg/logger` imports one third-party package, `golang.org/x/time/rate`, and `kitlogtest` nothing more. So `pekko-actor/go.mod` gains one direct requirement (`kit-logger`) and one indirect (`golang.org/x/time v0.14.0`, absent from the local cache). A throwaway probe resolved kit-logger offline from the cache and then stopped at `x/time` (`module lookup disabled by GOPROXY=off`): **building needs network once**, from the public Go proxy, for `x/time`; kit-logger itself comes from the cache (`GOPRIVATE=github.com/pablogore/*`). kit-logger declares `go 1.26.0`, below the module's `go 1.27.0`. `scripts/deps.allow` restricts only goPekko-internal imports, and `.golangci.yml` has no `depguard`.
**Seed (coordinator, before the implementer):** `go get github.com/pablogore/kit-logger@v0.1.2-0.20260912231430-17d1a6eacc85` and `go mod download` in `pekko-actor`, committed as a seed (`go.mod`, `go.sum`, and `go.work.sum` if it changes). The implementer never edits `go.mod` or `go.sum` and works offline. `go mod tidy -diff` is a gate on the **final** tree (the import then exists); at the seed commit it would remove the requirement.

## 9. Judge and matrix (D9)
The judge's Go side builds systems with `Config{}`, so a nil Logger keeps its logs unchanged. No judge file, scenario, mutation or report is edited. Because `pekko-actor/`'s tree changes, `scripts/check-judge-matrix.py` will report the Go **HISTORICAL** line as before and drop the Go **CURRENT** line (no match with the HEAD tree) until a campaign is re-run. **D9 (NEW):** re-run the Go campaign over the 18 mutants at the end of U9 (script time, not agent time; the GMoverlap policy of U8 applies: one rerun on `harness-error`, both runs kept), regenerate `judge/go-mutation-report.tsv` and retain its commit with an evidence tag (needs the Owner's authorization to push). Alternative: leave CURRENT not applicable after U9 and say so; cheaper, but the campaign is also the guard that logging did not change the judged behavior.

## 10. Acceptance of #451, as observed
1. Executed with kit `cf91c9d` and recorded (`EXECUTION-RECORD.md`; hashes verified).
2. L1-L16 and the API test pass; levels Warn/Error, Escalate silent, `WithLogging(false)` silent with unchanged decisions and events, a failing Logger absorbed.
3. The existing tests pass unchanged; the differential L6 shows both channels apart.
4. Mutants ML1-ML9 killed (valid detections only).
5. The parity doc, READMEs, `doc.go`, ADR 0017 (S14, A11, U9) and the PORT-GAP comment of `applyRestart` updated: the divergence "events but no logs" is closed for strategies; the restart-hook logs remain a declared gap.
#451 closes **only** if all five are observed and the Owner agrees.

## 11. How I read the request, and what I did not decide
The request was partly unreadable. I read it as: prepare the RULEBOOK and execution package for U9 (#451) from `origin/develop` (known `ad98f7f`); contrast ADR 0017 S14/A11, #451 and the pinned Pekko; fix Resume = Warning, Restart and Stop = Error, Escalate = no level, `loggingEnabled = false` = no logs and no effect on decisions, a logger failure not breaking failure handling, the `InitializationError` message from its cause; use `github.com/pablogore/kit-logger`, verify its API and propose how it is injected in `System`, without public changes that block the extensibility of strategies (#450), noting that `logFailure` stays overridable only in Pekko; include upstream sources, surfaces, dependencies, manifest, permissions and budget (stop 450,000, ceiling 560,000 including coordination); keep the matrix state in view; do not run anything; a draft PR; decisions and pendings in #461; no parity. The Owner corrects it at approval if wrong.
Not decided here: the exact attribute names (fixed by the implementer, documented), the version of kit-logger beyond the pseudo-version above, a user-supplied `logFailure`.

**Second round (corrections before authorization).** The request to correct this package was also partly unreadable. I read it as: (1) fix that `WithLogging(false)` on the zero value disables the logs of the effective default strategy, and the same for `DefaultStrategy` and the guardian strategy, with the explicit constructors and `WithLogging(true)` enabled (D2, L13, ML7); (2) specify when `Error()` is evaluated (after the decider), that AllForOne keeps its snapshot and budget and that the log does not alter suspension, activation or hand-over, and the order "decide, log, apply" (D3, D4, L14-L16, ML8-ML9); (3) the GMoverlap policy (RUNBOOK step 9): a failure is a known flake and is never converted automatically into `met`. Production is untouched; no agent ran.

## 12. Decisions this package asks the Owner to accept
| Id | Decision | Alternative and trade-off |
|---|---|---|
| D1 | `Config.Logger kitlog.Logger`; `nil` = silent (divergence from Pekko's default stdout log) | nil = a default kit-logger (prints by default); or a `spi` port with an adapter in `pekko-observability` (moves the dependency out of the ADR's place) |
| D2 | `WithLogging(bool)` methods on `Strategy[T]` and `GuardianStrategy`; the flag is independent of `kind`, so it also disables the effective default strategy (zero value, `DefaultStrategy`, zero or default guardian strategy); the zero value logs; four fallback places carry it | new constructors or an options parameter (breaks signatures, ADR 0017 S3/S7) |
| D3 | one entry per decision, in the order decide, log, apply, with `Error()` evaluated only for a logged decision after the decider; Warn / Error / none; AllForOne once; a denied Restart still logged; attributes `child`, `directive`, `error` (Error only) | per-child logs under AllForOne; logging after applying |
| D4 | the whole `logFailure` absorbed (message computation included); a Goexit of the logger still applies the decision exactly once; stricter than Pekko | guard only the logger call, as Pekko |
| D5 | events untouched, a differential test | none |
| D6 | no exported `logFailure` override (#450) | an exported hook now: pre-empts #450 |
| D7 | Warning → `Warn`, Error → `Error` | none |
| D8 | the implementer template allows exactly one scoped test command, `bash migration/stage-1/supervision-u9/run-logging-tests.sh` (the new tests with `-race`, one run), because these are deterministic unit tests it would otherwise write blind | no run: as U6/U7 |
| D9 | re-run the Go campaign at the end so the matrix keeps its Go CURRENT line (needs the evidence tag authorization then) | leave CURRENT not applicable |
