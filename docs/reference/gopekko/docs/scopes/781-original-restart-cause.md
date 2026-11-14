# Scope: keep the original restart cause when the new instance of a restart fails (#781, SUP-10)

Status: **approved** at gate G2 by the Owner in the working session of 2026-10-28 (a message of the Owner; recorded from the work thread, not a GitHub comment). Baseline: goPekko `develop` `1515c55b840975ef9704828ab1e5565506655b0d`; Apache Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (read-only). Authority: issue #781 and its ACT2 reconciliation of 2026-10-24, ADR 0017 (accepted) and the decisions below. Kit run: not required (no Pekko file is ported; a Go field carries a Pekko property).

## Objective

When a restart's new instance fails (factory panic or nil actor, `PostRestart` failure, or a `Goexit` of the factory or of `PostRestart`), the parent's decider receives an `*InitializationError{InRestart: true}` that forgets why the restart happened. Pekko keeps both causes in `PostRestartException(actor, cause, originalCause)` (`Actor.scala:247-251`, built at `dungeon/FaultHandling.scala:299-301`). This scope adds the missing one: a field `OriginalCause`, retiring `PORT-GAP(original-cause)` at `pekko-actor/actor/supervision_hooks.go:20-21`. The runtime already has the value: `finishRecreate(cause)` calls `restartFailed(err, survivors)` and drops it.

## Owner decisions (G2, 2026-10-28), as received (Spanish)

> D1: aprobado añadir `OriginalCause error` a `InitializationError`, sin crear otro tipo. Debe ser nil cuando `InRestart` sea falso.
> D2: aprobado mantener `Unwrap() error` devolviendo exclusivamente `Err`. La causa original se consulta mediante `OriginalCause`. AC2 debe probar explícitamente `errors.Is/As` sobre el contenedor para la rama `Err`, y sobre `OriginalCause` para la otra rama. No se promete que el contenedor recorra ambas.
> D3: aprobados los fallos de factoría, `PostRestart`, nil y Goexit descritos en el scope. Quedan fuera `PreRestartException` y el recorrido de fallos anidados.
> D4: aprobada la enmienda de ADR 0017 A9 y del archivo de firmas para reflejar el campo y su semántica. La forma de acceso mediante errores de Go es una adaptación aprobada por el Owner; no debe presentarse como paridad ejecutada con Pekko.
> D5: aprobado usar el comentario de cierre como registro final de entrega, además de las actualizaciones documentales incluidas en D4.

Consequences for the work. The `Goexit` of a `PreRestart` hook, which Go (not Pekko) also reports through `restartFailed`, gets the same field because it is the same code path. No `PreRestartException` type and no walker of nested failures (Pekko's `OriginalRestartException`) are added; a nested failure is still representable, as an `OriginalCause` that is itself an `*InitializationError`. The new access through Go errors has no Pekko counterpart (Java has one `getCause` chain); the documents must say it is an Owner-approved adaptation and that no Pekko scenario was executed.

## Contract (signature only)

```go
type InitializationError struct {
	Err           error // unchanged
	InRestart     bool  // unchanged
	OriginalCause error // the failure that began the restart whose new instance failed; nil when InRestart is false
}
```
`Unwrap()` returns exactly `Err` and `Error()` is unchanged. The field holds an error value, never the actor instance. It is additive: no unkeyed literal of the type exists in the repository.

## Criteria to scenarios

Unit level, package `actor`, go-specs, no `time.Sleep`; reuse the helpers of `supervision_restart_internal_test.go` (`sup2OnInit`, `isInRestartInit`).

| Criterion | Scenario | Kind |
|---|---|---|
| AC1 | **S1.** `Receive` fails with `E`, then `PostRestart` fails with `H`: the decider sees `InRestart`, `Err == H`, `OriginalCause == E` (the same value) | positive |
| AC1 | **S2.** Same for a factory panic (`ErrFactoryPanic`) and a nil actor (`ErrNilActor`) | positive |
| AC1 | **S3.** Same for a `Goexit` of the factory, of `PostRestart` and of `PreRestart` (`Err == ErrGoexit`) | positive |
| AC1 | **S4.** A failed first start: `InRestart == false`, `OriginalCause == nil` | negative |
| AC1 | **S5.** A restart that succeeds: no initialization failure reaches the decider; the events are unchanged (`receive-failed`, `restart`) | negative |
| AC2 | **S1b.** `Err` branch on the container: `errors.Is(ie, H)` and `errors.As(ie, &typedH)` succeed and `errors.Unwrap(ie) == ie.Err`. `OriginalCause` branch: `errors.Is(ie.OriginalCause, E)` and `errors.As(ie.OriginalCause, &typedE)` succeed (use typed errors). `errors.As(ie, &*InitializationError)` still finds the type, so the default decider still stops. Nothing is asserted about the container reaching `OriginalCause` | explicit contract (D2) |
| AC4 boundary | **S6.** The decider answers `Restart` to the first initialization failure and the second restart fails too: the second error's `OriginalCause` is the first `*InitializationError` (the attempt that failed), not the first `Receive` crash | ordering |
| AC4 boundary | **S7.** A `Stop` accepted while the restart waits for its children abandons the restart: no `InitializationError` is produced (ADR 0017 U7) | race |
| AC3 | **S8.** After `Done` closes, the failed instance is unreachable while the test still holds the `*InitializationError` (a `weak.Pointer` and a bounded GC wait, like the T04b precedent); and no field of the error has the actor's type | retention |

Mutation checks before the review: leave `OriginalCause` unset at each of the four `restartFailed` calls (`supervision_restart.go:374,489,495,503`); fill it with the hook's cause; fill it from something other than the restart's `cause`. Each must fail at least one scenario; record which.

## Allowed files

No other path may change. If another is needed, stop and report to the TL.

| Path | Edit |
|---|---|
| `pekko-actor/actor/supervision_hooks.go` | the field and its comment; remove `PORT-GAP(original-cause)` |
| `pekko-actor/actor/supervision_restart.go` | `restartFailed` takes the restart `cause`; four call sites |
| `pekko-actor/actor/supervision_restart_origcause_internal_test.go` | new, S1 to S8 |
| `pekko-actor/README.md` | one short paragraph in the supervision section |
| `docs/adr/0017-supervision.md` | A9 row and one marked amendment line (documentation only; no other policy) |
| `docs/adr/signatures/supervision.go` | the field in the accepted signature |
| `docs/scopes/781-original-restart-cause.md` | this file |

Not touched: Typed code and documents, #847 and its stack. #1014 also edits `pekko-actor/README.md`, in another section: expect at most a rebase conflict.

## Definition of Done

- Every scenario passes; each mutation check fails at least one; the Dev records which.
- Commands of `AGENTS.md` ("Go"): `gofmt`, `go vet`, unit tests of the module and of every module offline (`unshare -rn`), `scripts/check_deps.sh`, per-module coverage 85% and diff coverage 85%, `golangci-lint` v2.14.0 with gocyclo at most 10, and `make ci-fast`. No `-race` and no workbench unless the Owner asks. Remote CI is refused by GitHub for billing and is never described as passing; the Owner's exceptions so far applied only to earlier documentation-only PRs, so none is assumed here.
- The delivery is one draft PR, `Refs #781`, no closing keyword, no merge, at most 400 authored lines. If that does not fit, report the needed adjustment to the TL before widening the scope or dropping a scenario.
- No claim for SUP-11 to SUP-13 or full actor parity. The closing comment on #781 is the final record (SHA, tree, gates, QA verdict); the Owner closes the issue.
