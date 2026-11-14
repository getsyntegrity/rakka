# Ask reply endpoint (ACT-015, #161): scope, sources and mapping

Status: scope document. The Owner approved the unit RULEBOOK, model plan, permissions and decision D1 on 2026-10-12 (`EXECUTION-RECORD.md` section 4). Nothing here is an approval, a signature or a receipt by itself.
Authority: ADR 0015 (`docs/adr/0015-ask-and-reply.md`, status `proposed`; implementation of I-1 to I-6 authorized by the Owner on 2026-10-11), index #48, protocol #123, #322 to #324.
Source: Apache Pekko v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`upstream/pekko`, read-only). Kit: `migration-kit` at `cf91c9d5068d9aaf95a36164169f08c3e636c909` (read-only).

## 1. Interpretation of #161 against the ADR

| Ticket wording | Reading compatible with the Owner decisions | Why |
|---|---|---|
| "Registro temporal de Ask" | The per-request **reply endpoint**, the Go counterpart of Pekko's temporary ask ref (`PromiseActorRef` under `/temp`). It is **not registered**. | D8: the endpoint is outside `System.names` and `System.byID` (`byID` is never pruned, so a registry would leak). "Registro temporal" does not authorize a permanent entry or a global registry. |
| "Una identidad por solicitud" | `ask-<n>` drawn from the process-wide `idSeq`; never reused. | D8. |
| "Handle inválido/terminado sigue contrato" | Target validation by identity (D6) returns `ErrInvalidHandle`; a terminated target passes validation and is handled later (D5 step 4, `ErrRecipientTerminated`, #166/I-4). | D5, D6. |
| "No reutilizar correlación" | No correlation table exists; each request owns its endpoint. | D8. |
| MVP text "liberado al resolver y al detener el sistema" | Release on resolve is structural (nothing is registered). "Al detener el sistema" needs `System.Shutdown`, which does not exist. | D7: AK17 blocked. Not implemented here. |

## 2. What #161 delivers and what stays in other tickets

| Piece | Where | Ticket |
|---|---|---|
| Unexported `askEndpoint[R]`: identity, single terminal transition (CAS), losing `Tell` dead-lettered through the owner System, not an `owner()` handle | `pekko-actor/actor/ask_endpoint.go` | #161 (this slice) |
| Unexported `askTarget`: native ref, owner derived from the target, verified by cell pointer identity | same | #161 |
| `ErrInvalidHandle` reused, text neutralized, Watch comments updated | `errors.go` | #161 |
| Scenarios AK20, AK21, AK23 plus identity, isolation, no-registration | `ask_endpoint_internal_test.go` | #161 |
| Exported `Ask`, `build`, check order D5 steps 2 to 3, deadline timer, `ErrAskTimeout`, `ErrAskCanceled`, `ErrAskNoDeadline`, first-reply completion wired to the caller | `ask.go` | #162 to #165 (I-2/I-3 split; the ticket-to-increment mapping is not in the ADR, see AMENDMENTS A-3) |
| `ErrRecipientTerminated`, AK19a/b, accepted Stop is not termination | `ask.go` | #166 (I-4) |
| `System.Shutdown`, AK17, `AskFrom`, `StatusReply`, foreign-ref rejection, cross-system | deferred | D7, D6 |

No exported Ask is published: a partially working public API would be a contract the later tickets would have to keep.

## 3. Pekko -> Go -> tests

| Upstream fact (verified by reading) | Go adaptation | Test | State |
|---|---|---|---|
| One `PromiseActorRef` per ask, unique temp path (`AskSupport.scala:667` path is created lazily; `ActorRefProvider.scala:428-441, 603-611`) | `ask-<n>` ID from `idSeq`; no path | `TestAskEndpointIdentity` | Go test passes. Parity with a Pekko oracle: **not verified**. |
| `tryComplete` is a single atomic transition (`AskSupport.scala:725-738`) | `atomic.Uint32` CAS pending -> replied | `TestAskEndpointCompletesOnceAndDeadLettersTheRest` | Go test passes. Pekko oracle: **not verified**. |
| A second message to a completed ask ref goes to dead letters (`AskSupport.scala:625-627`; `PromiseRefSpec.scala:54-64`) | `DeadLetter{Msg, To: id}` through the owner System | same | Ported by the kit loop in `ask_endpoint_promiseref_internal_test.go` (reduced / faithful-adapted); the Go test passes; the Pekko-side oracle was **not** run. |
| Ask ref is not registered until its path is read (`AskSupport.scala:667`) | Never registered | `TestAskEndpointIdentity` (no `byID` growth) | Go test only. |
| `Unsupported recipient type` for a non-local ref (`AskSupport.scala:383`) | `ErrInvalidHandle` for nil, typed nil, user ref, endpoint, unregistered cell, ID coincidence | `TestAskTargetValidation` (AK20) | Go-only behavior (G); Pekko has no equivalent. |
| No origin identity for a caller | A native ref of another System is accepted and owned by its own System | `TestAskTargetValidation` (AK21) | Go-only; documented limitation of D6. |
| Watch/Unwatch/Stop/Done against the endpoint | See section 4 | `TestAskEndpointIsNotAWatchableActorHandle` (AK23) | Go-only. |

## 4. Contradiction found and resolved: ADR 0015 D7 versus the Owner-accepted Watch contract

| Source | What it says about `Unwatch` on a handle that is not an actor handle | Status |
|---|---|---|
| ADR 0010 `docs/adr/0010-actor-api.md:162-166` | Owner approved on 2026-10-10 `Watch`, `WatchWith`, `Unwatch` and `Terminated` "as implemented in `pekko-actor/actor/watch.go`"; a call that cannot be honoured panics with a typed cause that fails and stops the watcher | **Accepted** (bounded to the local watch increment) |
| Code `pekko-actor/actor/watch.go:204-208` (`unregister`) and `:100-103` (`resolve`) | after the stop-accepted check, `resolve` panics with `ErrInvalidHandle` for nil, foreign, or any handle without `owner()` | implementation of the accepted contract |
| Test `pekko-actor/actor/watch_internal_test.go:282-291` | calls `Unwatch(foreignHandle{})` after `StopSelf` and states it "would otherwise fail the actor with `ErrInvalidHandle`" | pins the post-stop no-op only |
| ADR 0015 D7 (row "Watch on the reply endpoint") and AK23, as first written | `Watch`/`WatchWith` fail the watcher; `Unwatch(askRef)` is a no-op "(the endpoint is not registered: unknown ID)" | `proposed`; contradicted the row above it, which claimed alignment with the accepted contract |
| Test `pekko-actor/actor/ask_endpoint_internal_test.go` (`TestAskEndpointIsNotAWatchableActorHandle`, subtest `Unwatch`) | `Unwatch(askRef)` from a live watcher stops it with `ErrInvalidHandle` | pins the accepted behavior for the endpoint |

Why the ADR 0015 rationale does not hold: the endpoint is rejected by the `owner()` check, which runs **before** any registry lookup, so "unknown ID" is never reached for `Unwatch`.

Resolution (no contract change, no code change): the existing Owner decision (ADR 0010, 2026-10-10) already settles it. D7 and AK23 were corrected to say `Unwatch(askRef)` fails a live watcher with `ErrInvalidHandle`, and is a no-op only after the watcher's own stop was accepted. `System.Stop`, `Context.Stop` and `System.Done` keep their D7 behavior. The alternative (make `Unwatch` a no-op for non-actor handles) would amend the accepted Watch contract, including the existing `Unwatch(nil)` behavior, and was **not** taken. The Owner may still confirm or reverse this when reviewing the ADR text edit.

## 5. Verifiable criteria of this slice

1. Every endpoint has a distinct `ask-<n>` ID; none equals an actor ID.
2. An endpoint never appears in `System.byID` or in a name table; `lookup` returns nil.
3. N concurrent endpoints receive only their own reply; no dead letter is produced when each is replied once.
4. Of many concurrent replies to one endpoint exactly one wins; the rest are dead-lettered with `To` = the endpoint ID.
5. The endpoint does not implement `owner()`.
6. `askTarget` returns `ErrInvalidHandle` (and not `ErrUnknownHandle`) for every invalid target in AK20; accepts a terminated target.
7. `ErrInvalidHandle.Error()` no longer mentions Watch; `errors.Is` is unchanged for Watch callers.
8. No sleep is used as synchronization (gate channel and wait group).

## 6. Not verified

Parity with Pekko is not claimed. No Pekko-side oracle was run for the endpoint. The Go judge has no Ask scenario yet (the judge exercises exported behavior; the first one belongs to #162). AK scenario ids here are the ADR's proposed ids.

Shared-file entries (cost log row, milestone AK table, parity note) live in `entries/` and are merged into the shared files by the coordinator, not by this PR.
