# RULEBOOK: Typed `Stopped` and `StoppedWith`, slice A (unit typed-1182-stopped, #1182 TYP-48)

Status: **approved at gate G2 by the Owner on 2026-11-10** for `docs/scopes/1182-typed-stopped.md` (PR #1524, merged). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after the unit starts.

## 1. Unit

- Capability: the constructors `Stopped[T]()` and `StoppedWith[T](postStop func())` and the stop they cause in the interpreter: from a message or signal handler, from a `Setup` (initial or returned by a handler), from the initial behavior and from a restart that rebuilds into `Stopped`. The actor stops through the hierarchy (`Context.StopSelf`), the previous behavior gets the `PostStop` signal and then the callback runs once. A `nil` callback is `Stopped()`.
- Out of this slice (the scope table): the held inputs of a restart becoming dead letters, `Context.System()` and the oracle values (slice B, after #871 and PR #1525); the interceptor case `PORT-GAP(TYP-48-interceptor-978)`; the `Stop` variants of #1184 `PORT-GAP(TYP-48-stop-variants-1063)`; the interruption of a replay (#927).
- Governing decisions: scope D-1 to D-6; ADR 0033 D13; ADR 0035 (`Stopped` is a result, not a failure); ADR 0011 and EXP-1 (each export justified in its doc comment, in the style of `behavior.go`).
- Pekko v1.7.1 (`9a54d12`) is the oracle by reading; no Pekko run exists for slice A, so **not V3**. Evidence classes: `SPEC` (a Pekko spec row, read, not run), `READ` (Pekko code, read). `manifest.tsv` follows the kit runner contract (`source`, `target` first).
- Kit run (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`): port plus adaptation, kit at `cf91c9d`. The Dev is a separate session from QA.
- Layout: everything in `pekko-actor-typed/typed/`; `typed` keeps importing only `pekko-actor`. Slice A touches no file of `pekko-actor/`.

## 2. Allowed files (slice A, one PR)

Paths under `typed/` are in `pekko-actor-typed/`. Anything else is a Stop.

`typed/behavior.go`, `typed/interpreter.go`, `typed/stopped_test.go` (new, `package typed`), `typed/example_test.go` (`ExampleStoppedWith`), `typed/unhandled_internal_test.go` (only the `want` list of exported names in `TestUnhandled_U9_PublicSurface`), `pekko-actor-typed/README.md`. Slice 0 (a separate docs PR, so that slice A stays under 400 authored lines): `migration/stage-1/typed-1182-stopped/*` and one row of `migration/cost-log.tsv`.

## 3. Rules for the Dev

- D-1: two constructors, no sentinel; a nil callback is `Stopped()` and carries `TODO(TYP-48-nil-callback)`. D-2: the callback takes no context. D-3: a sealed variant `stoppedB[T]`; `turn` records the stop and calls `StopSelf`; the previous behavior stays installed so that `PostStop` reaches it. D-4: a stopped start returns a marker that `PreStart` and the restart turn into a stop; no previous behavior, so only the callback runs. D-5: `Stopped` inside `Supervise` and `Resume` follows the same path, no layer decides. D-6: messages after the stop are the runtime's dead letters, unchanged.
- Headers: the files already carry exactly one `// Ported from` line; no file is added that needs one (`stopped_test.go` is a test).
- Tests: go-specs matchers; channels and gates, no `time.Sleep`; no real engine. Coverage at least 85% per module, no `//nolint`, gocyclo at most 10; the README follows the module README contract.

## 4. Mandatory Stops

Report to the TL and stop; never skip, delete or weaken a test:

1. A change in `pekko-actor` is needed.
2. A test fails on the current code where the scope expects it to pass; a failing scenario is run on Pekko before it is classified (`gopekko-failure-triage`).
3. A file outside the allowed list must change, or the PR projects over 400 authored lines.
4. A decision above (D-1 to D-6) must change, or a second export is needed.

## 5. Mutation rule

Each behavior test names a mutation in the "Mutations" table of the EXECUTION-RECORD: R1 the interpreter calls `PostStop` itself before the stop; R2 the callback runs before the signal; R3a `PreStart` stops without keeping the callback; R3b `turn` stops without keeping the callback; R3c `startInto` rejects the stopped marker; R3d the callback also runs at the stop request; R4 `Stopped()` delivers the signal twice; R5 `turn` fails a `Stopped` result; the restart and `Setup`-result paths ignore the marker. Applied by hand in a disposable edit of `interpreter.go`, restored after each run, never committed; the named test must fail by assertion.
