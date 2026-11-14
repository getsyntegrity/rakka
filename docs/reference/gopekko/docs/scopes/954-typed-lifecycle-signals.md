# Scope: Typed signals infrastructure and PostStop delivery (#954, TYP-04, Slice A)

Status: **approved by the Owner at gate G2 on 2026-10-28, at SHA `c5e7f192f225f3fbc870ea9fce9e8e7466de4a13`** ([comment on PR #1234](https://github.com/getsyntegrity/gopekko/pull/1234#issuecomment-6469664425)), with D-2, D-3, D-7 and D-8 as this scope recommends them and D-4 (a), D-5 and D-6 accepted. The approval authorizes Dev and an independent QA for Slice A, with local validation (remote CI is blocked by billing and is not awaited). It does not authorize merge or closing #954; B-2, Slice B and the Typed watch source (#1171) stay pending and blocked. Written by the TL on 2026-10-28 against `develop` `3b47251`. The Owner authorized preparing and correcting this
scope (not G2, not Dev, not merge, not closing issues), fixed its six conditions and took D-1, D-4, D-5 and D-6; all are quoted in the Log. **G2 was approved by the Owner (see the status line).** Nothing here is implemented.
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: it ports Pekko behavior, so the Dev runs the kit with its own manifest,
RULEBOOK and RUNBOOK. **Slice B (`Terminated`, death pact) stays blocked and is not specified here.** Delivering A does **not** close #954.

## What this delivers

`pekko-actor-typed` has no signals. Slice A adds the infrastructure and the first delivery: a Go developer attaches a signal handler to a receive
behavior, and when the actor stops the **current started** behavior receives `PostStop` exactly once, never while a message is being handled.
In Pekko: `Behavior.interpretSignal(b, ctx, PostStop)` from `ActorAdapter.postStop` (`ActorAdapter.scala:325-331`) and `receiveSignal`
(ADR 0034 P2, P5). Names are indicative; the shapes are the contract.

```go
b := typed.ReceiveSignal(typed.ReceiveMessage(handle), func(ctx *typed.Context[string], s typed.Signal) (typed.Behavior[string], error) {
    if _, ok := s.(typed.PostStop); ok { release() } // runs once, on the behavior current at stop
    return typed.Same[string](), nil
})
```

## What this does not deliver

- **Delivery** of `PreRestart` (#976), `Terminated` and the death pact (Slice B), `ChildFailed` (open, D-5) and `MessageAdaptionFailure` (#975).
  Slice A only **declares** these types (D-2); declaring one implements none of its contracts and **does not approve B-2**.
- `Stopped`, `StoppedWith`, `Unhandled`, `Empty`, `Ignore` (#1182, #925, #1181), Typed supervision (#956, #976), the Typed watch (#1171) and spawn (#1170).
- Any `pekko-actor` change: it uses `actor.PostStopper` (`actor.go:38`) and `actor.System.Stop`/`Done` (`stop.go:20`, `system.go:204`). EXP-1 needs no record.

## Dependencies, as they really are

| Dependency | State today | Effect on Slice A |
|---|---|---|
| #847 (TYP-02), #1096 (TYP-34) | closed, integrated (#1227, #1228, #1229) | Delivered; A edits the files #847 created |
| #925, #1113, #1063 | open | Not needed: reconciliation `typed-supervision-stories-reconciliation-2026-10-26.md:272-276`, confirmed by the Owner for this slice |
| ADR 0034 B-1 (P2, P3, P4, P6, P8) | accepted 2026-10-27 | Consumed |
| ADR 0034 P5 construction (B-2) | **proposed** | **Not consumed** by A; blocks Slice B |

## Contract produced

In `pekko-actor-typed/typed/` (one package; the sealed `Behavior[T]` needs the interpreter in it). Only these symbols are added; any other export needs a stop.

```go
type Signal interface{ /* sealed: unexported method */ }
type PreRestart struct{}   // declared; delivery: #976
type PostStop struct{}     // delivered by Slice A
type Terminated struct{ /* unexported fields only */ }             // declared, opaque; construction and fields: B-2, Slice B
type ChildFailed struct{ /* unexported fields only */ }            // declared, opaque; construction and fields: B-2, the M-A story (D-5)
type MessageAdaptionFailure struct{ /* unexported fields only */ } // declared, opaque; construction and fields: #975

func ReceiveSignal[T any](base Behavior[T], onSignal func(ctx *Context[T], s Signal) (Behavior[T], error)) Behavior[T]
var ErrSignalBaseNotReceive error // a base that is not Receive/ReceiveMessage/ReceiveSignal fails the start (D-3)
```

**Opaque declarations (D-2).** ADR 0034 P5 names `Terminated{Ref}` and `ChildFailed{Ref, Cause}`, but how the interpreter builds the `Ref` and the cause is B-2, still proposed. So A exports these types with **no exported field, accessor or constructor**: they exist only so the sealed set is complete (B16) and a handler can switch on the type. Nothing in A constructs or delivers them, and no test reads a field. Adding a field later is compatible; removing or retyping an exported one would not, which is why none is published now. Slice B, the M-A story and #975 add their fields and accessors with their own contracts.

The interpreter gains `actor.PostStopper[T]`. It also gets a **no-op `actor.PreRestarter[T]`** (D-4 (a)): it only occupies the hook slot so the runtime's default restart step does not call `PostStop` on the old instance; it delivers **no** `PreRestart` signal (that is #976, which replaces its body). **Temporary mechanism (D-4):** because the hook replaces the runtime's default restart step, the old instance's children are not unwatched or stopped at a restart either; that treatment is restored by #976 and must be in place before #1170 gives a Typed actor children. It still does not implement `TerminatedHandler`. Rules: the signal handler belongs to its
receive behavior, so a returned behavior replaces both (as Pekko); the result of a `PostStop` handler is ignored (its error is edge (a) below: a handler cannot return one to the runtime, so it is raised as a panic); a signal with no handler is dropped (ADR 0034 P6);
`PostStop` is delivered only if the behavior started (B17): a start that failed, or an instance whose restart start failed, gets none and must not panic.

## Criteria → scenario → Pekko source

"Issue criteria" are the reconciled ones (`typed-supervision-stories-reconciliation-2026-10-26.md:263-270`); the issue text stays unchanged. Level: unit, real runtime, no double.

| # | Criterion (Slice A) | Scenario in `pekko-actor-typed/typed/` | Pekko source | Evidence |
|---|---|---|---|---|
| 1 | On a stop by the parent or the system, **with no restart in flight**, the current started behavior receives `PostStop` exactly once | S1 handler logs once; after `System.Stop` and `Done`, one entry and none after | `ActorAdapter.scala:325-331`; `BehaviorSpec.scala:210, 214` | SPEC |
| 1 | After a replacement only the **current** behavior's handler runs | S2 B0 replaced by B1; stop; only B1's handler ran (also after a message following the swap) | `BehaviorSpec.scala:297, 301` | SPEC |
| 1 (order) | `PostStop` completes before the actor's `Done` closes | S3 the handler holds a gate; `Done` stays open until released | `stop.go:160` (finalization) | READ |
| 1 (negative) | No handler: the stop succeeds, nothing is delivered, no failure | S4 plain `Receive`; stop; `Done` closes, no cause reaches the decider | `ActorAdapter.scala:325-331` (`interpretSignal` ignores an unhandled `PostStop`) | READ |
| 1 (B17) | A behavior that never started receives no `PostStop` | S5 the start fails (Setup error); S6 a restart whose `Setup` fails is then stopped; no handler call, no panic | `ActorAdapter.scala:325-331`; `actor.go:29-37` | READ (B17: no spec found) |
| 1 (restart) | A runtime restart delivers no `PostStop`, and a stop that abandons a restart still delivers exactly one per instance | S10 verifies the checkable part. The turn returns an error and the guardian decider of the existing harness (`typed_test.go:52`, `actor.GuardianOneForOne`, directive `actor.Restart`) answers Restart: the old instance's handler is not called at the restart; the new instance, then stopped with `System.Stop` and `Done`, calls its handler once. The Stop that abandons a restart is **not** covered: see "Abandoned restart" | `ActorAdapter.scala:309-314` (`preRestart` sends `PreRestart`, not `PostStop`) | READ |
| 2 (shape) | A base that is not a receive behavior fails the start | S7 `ReceiveSignal(Setup(...))` and `ReceiveSignal(Same())` fail with `ErrSignalBaseNotReceive` as `*actor.InitializationError` | `Behavior.scala:198` (`receiveSignal` exists only on `Receive`) | READ |
| 3 | Signal processing never overlaps message handling on the same cell | S8 a message handler holds a gate; stop is accepted; a shared in-flight counter never exceeds 1 and `PostStop` runs after the handler returns | ADR 0034 B19 | READ; stress, below |
| 3 | same, many | S9 N messages told while a stop is accepted: counter never above 1 across both handlers | same | READ |
| 4 | Positive and negative cases, ordering boundary named, SHA and gates recorded | S1 to S10; ordering boundaries S3 and S8; the SHA and gates go in the PR comments and the execution record | | |
| 5 | Public contract and traceability for this slice; no sibling or parity claim | README, executable example, execution record | | review |

S1 and S2 derive from `BehaviorSpec.scala` and carry its single `Ported from` line; S3 to S10 are Go-only evidence in a file with none. A scenario observes its
result with channels and gates, never a sleep, and `-race` is not used. S9 uses a fixed N (say 200) and an assertion on an invariant, not on timing.
Existing #847 tests (S1 to S11 there) keep passing: A edits the shared interpreter.

**Abandoned restart.** The runtime's default `PreRestart` runs `PostStop` on the old instance and a later Stop of the abandoned restart runs it again, so an instance would see two calls (`supervision_restart.go:378-392`). S1 and S2 claim exactly-once only for stops with no restart in flight. With the no-op `PreRestarter` the restart-time call never happens, so the second call has no first one; S10 pins the restart-time half deterministically. A Stop that races a restart is not given a deterministic scenario here (it needs a hold inside the runtime's restart step, which `pekko-actor` does not expose): it is excluded from the evidence and recorded as an **evidence gap with no coverage claimed**. The README limitations and the execution record state it.

**Edges left open on purpose** (`Unknown is an answer`; leave a greppable `TODO(TYP-04-edge)`): (a) an error returned by a `PostStop` handler: `PostStopper` has no
error return, so the interpreter raises it as a panic and the runtime reports `poststop-panic` and continues (`stop.go:160`); Pekko logs it; no logger accessor exists (#1179).
(b) the abandoned restart above: structural guard only, no scenario.

## Allowed files

Exactly these. Anything else needs a stop and a report to the TL.

| Path | Edit | PR |
|---|---|---|
| `docs/scopes/954-typed-lifecycle-signals.md`, `migration/stage-1/typed-954-lifecycle-signals/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | new; kit package, every Pekko spec row ported, adapted, moved or `PORT-GAP` | 1 |
| `pekko-actor-typed/typed/signal.go` | new, `Ported from .../typed/MessageAndSignals.scala` | 2 |
| `pekko-actor-typed/typed/behavior.go`, `interpreter.go` | edit (keep their `Ported from` line) | 2 |
| `pekko-actor-typed/typed/signal_bhv_test.go` (S1, S2), `signal_runtime_test.go` (S3 to S10) | new | 2 |
| `pekko-actor-typed/typed/example_test.go`, `pekko-actor-typed/README.md` | add a `ReceiveSignal` example; README: implemented versus planned, limitations (D-4, the child treatment skipped at restart, the abandoned-restart evidence gap), link | 3 |
| `migration/stage-1/typed-954-lifecycle-signals/EXECUTION-RECORD.md`, `migration/cost-log.tsv` (one row) | commands, kit `cf91c9d`, models, outputs, manual changes; gates bound to a SHA | 3 |

Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum` or the root `README.md` changes. #956's scope (PR #1233) is not touched.
One scope: the independence table is trivial. **Sequencing:** A shares `behavior.go` and `interpreter.go` with #925, #1182 and #1178, so they run after A or after each other, never in parallel.

## Contradiction: where `ChildFailed` goes (ADR 0033 D5)

Three texts disagree. The issue's reconciliation lists `ChildFailed` in #954. The merged reconciliation moves it to #956 "because it depends on #954". #956's approved scope
(D-2, mechanism M-B only) excludes the Typed parent and says "no owner today". A Typed parent that watches a failing Typed child needs a Typed spawn (#1170), a Typed
watch (#1171), the `Terminated` delivery and B-2. **Accepted by the Owner (D-5, 2026-10-28):** the destination is one **separate story** for M-A / `ChildFailed` ("a Typed parent applies the default stop and receives `ChildFailed`"), **not** #956. It depends on Slice B, #956, #1170 and #1171, and nothing upstream waits on it, so there is no cycle with #954 or #956. It does not exist as an issue yet and no number is assigned; the TL creates no issue without authorization. This supersedes the line of the merged reconciliation that moves `ChildFailed` to #956 (the story's own text and #956's approved scope disagree with it). **This does not approve B-2.** This scope edits no issue.

## Slice B: pending, recorded (not specified, not approved)

| Issue criterion | Pending in B | Blocked by |
|---|---|---|
| `Terminated` delivered at its boundary | delivery of `Terminated{Ref}` to the active handler | B-2; a watch source; a decision on test access |
| Unhandled `Terminated` follows the death pact | cause with `errors.Is(cause, actor.ErrDeathPact)`, raised before the runtime sees it (P6) | the same |
| `PreRestart` delivered | moved to #976, not B | #976 |
| Criteria 4 and 5 for B | its own scenarios, SHA, gates and docs | B |

The Owner has not approved B-2, a watch source or an internal test seam. Their alternatives and implications are presented when B is taken up. #954 stays open, `status:blocked`.

## What #956 consumes

#956 consumes **PostStop of Slice A, integrated into `develop` and verified** (a QA verdict and local gates bound to the merged SHA), not the closure of #954. Its text says
"until #954 is delivered" (`docs/scopes/956-...`, D-1 (a), in PR #1233); the replacement is **D-6** (accepted by the Owner on 2026-10-28). The correction to #956's scope is applied afterwards, in coordination, by the owner of PR #1233; this scope does not touch it. Its S8 needs only this slice.

## Evidence, gates and delivery

**Level, honestly.** Go unit evidence derived from Pekko specs, read and not run. **Not V3** (no Typed Pekko harness, ADR 0033 "Consequences"). Rows marked READ stay alternative evidence, pending approval (ADR 0033 D13).
**Gates** (commands in `AGENTS.md`, "Go"): `pekko-actor-typed` unit tests with network isolation, `go vet`, `gofmt -l`, `golangci-lint` with `gocyclo` at most 10, `go mod tidy -diff`,
module coverage and diff coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, and `go-sdk-update.sh --check`, with the Go version in `.go-version`.
Run once per final tree, each result bound to its SHA. **Remote CI is blocked by billing; local results are not presented as approved CI.**

**Budget.** At most 400 authored lines per PR, additions plus deletions, counting this scope, kit files, code, tests, README and the execution record; excluded are generated code, vendored files, `go.sum` and
`.claude/skills/` (none changes). Three draft PRs (`Refs #954`, never `Closes`, no merge), as for #847:

| Task | Result | Check | PR | Estimate |
|---|---|---|---|---|
| T1 | this scope and the kit package | each spec row has a disposition; RULEBOOK read-only afterwards | 1 | 320 |
| T2 | `signal.go`, `ReceiveSignal`, the sealed set | S7 passes; compiles | 2 | 100 |
| T3 | `PostStop` delivery, B17 guards, no-op `PreRestarter` | S1 to S6 and S10 pass | 2 | 85 |
| T4 | non-overlap evidence | S8, S9 pass | 2 | 130 |
| T5 | example, README, execution record, cost-log | gates bound to the final SHA | 3 | 130 |

**Stop condition:** if the Dev projects a PR over 400, stop and report; the TL moves T4 to PR 3. Other stops: a file outside the list must change; `pekko-actor` needs an export; a scenario
fails and is not a Go defect (run it on Pekko first, per `AGENTS.md`). Never skip, delete or weaken a test.

## Decisions for G2

**G2 requires the Owner's explicit approval of this scope at an exact SHA; silence approves nothing and the Dev does not start without it.** After the QA verdict, the TL presents one G2 request with the open decisions below.

Taken by the Owner on 2026-10-28 (recorded in the Log):

- **D-1, criteria base.** The reconciled criteria are confirmed for Slice A only. #954 is not completed by A.
- **D-4, `PostStop` at a runtime restart.** Option (a): the interpreter gets a **no-op `PreRestarter`**, a temporary mechanism. Why: once the interpreter is a `PostStopper`, the runtime's default `PreRestart` calls `PostStop` on the old instance, and a Stop that abandons the restart calls it again (`supervision_restart.go:378-392`), so two calls on one instance; Pekko Typed sends `PreRestart`, not `PostStop`, at a restart (`ActorAdapter.scala:309-314`). Side effect, to be documented: the hook also skips the default's unwatch and stop of the children. Tracking: **#976** replaces the hook body and restores child handling; **#1170** (Typed spawn) must not give a Typed actor children before that. Evidence: S10 verifies the checkable half; the Stop that abandons a restart stays an evidence gap with no coverage claimed. Rejected: (b) accept two calls; (c) no hook.
- **D-5, `ChildFailed` destination.** A separate M-A / `ChildFailed` story, not #956; see the contradiction section. Not B-2.
- **D-6, #956 prerequisite.** #956 depends on the integrated, verified `PostStop` of Slice A, not on the closure of #954; the correction of #956's scope follows, coordinated.

Open for G2:

- **D-2, declared types.** All five signal types are declared (B16), `Terminated`, `ChildFailed` and `MessageAdaptionFailure` as opaque structs (see "Contract produced"), so no representation of B-2 is frozen. Alternatives: declare only `PostStop` and `PreRestart` now and add the others with their stories (no sealed-set completeness in A); or publish the P5 fields (rejected: it decides B-2). **Recommend the opaque declarations.**
- **D-3, `ReceiveSignal` shape.** A function over a receive base, with `ErrSignalBaseNotReceive` (an added export). Alternatives: a combined constructor with both handlers; a panic on a bad base. **Recommend the function.**
- **D-7, `PostStop` handler error.** Edge (a): panic path with a `TODO(TYP-04-edge)` (**recommended**), or drop it silently.
- **D-8, three PRs.** Because of the 400-line budget and the kit precedent.
- **Labels.** #954 keeps `status:blocked`; this scope changes none.

Still pending and not part of A: B-2, a watch source and test access for Slice B, and the creation of the M-A story. Local validation only: no green CI is expected.

## Definition of Done

- S1 to S10 pass at the final SHA; T1 to T5 each meet their check; no function above complexity 10.
- Gates green with evidence bound to the SHA; the execution record says what was read and not run, and which rows are READ and pending approval.
- README and example match the code and state the evidence gap (a Stop that abandons a restart) and the skipped child treatment; no claim that #954, Slice B or parity is done.
- A new QA session, independent of the TL and Dev, issues a verdict on the exact head of each PR.
- No merge without the Owner. #954 stays open; the Owner closes it only when Slice B and its pending criteria are met.

## Log

L1 (2026-10-28, Owner, verbatim):

> Autorizo preparar el scope de Slice A para G2, con estas condiciones:
>
> 1. Confirmo la reconciliación como base para este recorte: Slice A incorpora la infraestructura de señales, ReceiveSignal y la entrega de PostStop al behavior activo, exactamente una vez, respetando B17 y sin solapamiento con mensajes.
>
> 2. Apruebo la separación secuencial A/B. Distinguir la declaración de los tipos de señal de su entrega efectiva. Declarar PreRestart, Terminated, ChildFailed o MessageAdaptionFailure no implica implementar sus contratos ni aprobar B-2.
>
> 3. Slice B permanece bloqueado. No apruebo todavía B-2, una fuente de watch ni un acceso interno para tests. Presentar alternativas y sus implicaciones cuando se aborde ese slice.
>
> 4. No reasignar ChildFailed automáticamente a #956: su scope aprobado excluye el padre Typed. Resolver explícitamente esa contradicción y proponer el destino correcto, sin introducir un ciclo entre #954 y #956.
>
> 5. La entrega de A no permite cerrar #954. Registrar qué criterios quedan pendientes en B. Para #956, precisar que el prerrequisito consumido es PostStop de Slice A integrado y verificado, en lugar de exigir ambiguamente el cierre completo de #954.
>
> 6. Preparar docs/scopes/954-typed-lifecycle-signals.md con criterios, pruebas, archivos permitidos, presupuesto y evidencia requeridos. El límite de 400 líneas debe incluir todos los cambios computables según la política del repositorio.
>
> Esta autorización permite preparar el scope. No aprueba todavía G2 ni autoriza iniciar Dev, hacer merge o cerrar issues. Presentar el scope y sus decisiones pendientes para revisión.

L2 (2026-10-28, Owner, verbatim):

> Autorizo corregir el scope y ejecutar una revisión QA independiente sobre el SHA resultante. Todavía no iniciar Dev.
>
> Decisiones:
>
> - D-1: confirmo los criterios reconciliados únicamente para Slice A. No se considera completado #954.
> - D-4: acepto la opción (a), PreRestarter sin operación, como mecanismo temporal. Documentar que también evita el tratamiento de hijos del restart y conservar su seguimiento explícito para #976/#1170. S10 debe verificar la parte comprobable; el caso de Stop que abandona un restart permanece como brecha de evidencia, sin afirmar cobertura.
> - D-5: acepto el destino conceptual en una story separada para M-A/ChildFailed, sin introducirlo en #956. Identificar su seguimiento y dependencias; esto no aprueba B-2.
> - D-6: acepto que #956 dependa de PostStop de Slice A integrado y verificado, no del cierre completo de #954. Aplicar después la corrección al scope de #956 de forma coordinada.
>
> Antes de QA, corregir:
>
> 1. Eliminar “No decision blocks the Dev if the Owner is silent”. G2 requiere aprobación explícita.
> 2. Resolver la contradicción entre campos públicos “indicativos” de Terminated/ChildFailed y B-2 pendiente. Proponer una declaración que no congele una decisión de construcción o representación aún no aprobada.
>
> QA debe revisar el scope completo, incluidos D-2, D-3 y D-7, y emitir un dictamen sobre el SHA exacto. Presentar después una única solicitud de G2 con todas las decisiones restantes.
>
> B-2, la fuente de watch y Slice B siguen pendientes. Validación local únicamente: no esperar CI verde ni repetir el diagnóstico de facturación. Sin merge ni cierre de issues. hace spawn de agentes a dev y QA  y TL
