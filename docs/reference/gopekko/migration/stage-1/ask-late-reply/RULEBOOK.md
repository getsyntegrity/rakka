# RULEBOOK: Ask late reply (#163, unit ask-late-reply)

Status: **DRAFT, awaiting Owner approval of its sha256.** Read-only inside any loop (kit `CLAUDE.md` rule 1). Approval is recorded in `EXECUTION-RECORD.md` of this unit; this header is never edited after approval (lesson of #162).

## 0. Unit

- Capability: a reply that reaches an ask endpoint **after `ask` has returned** does not change the returned result, does not reach any other Ask, is published as `DeadLetter{Msg, To: endpoint ID}` exactly once per late reply, and leaves `System.byID` and the name tables unchanged.
- Source (Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, read-only): `upstream/pekko/actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala` 614-629 (`PromiseActorRef.!`: a message to a completed ask goes to `provider.deadLetters`); typed `upstream/pekko/actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/AskSpec.scala` 132-163 and 164-182, whose dead-letter assertions (message is the late one; recipient is neither the target nor the dead-letters ref) are **adapted**: Pekko's late reply follows a timeout, here it follows a normal return, because the timer belongs to #164.
- Governing decisions (Owner-accepted in ADR 0015): D4 one terminal transition, D8 endpoint outside registries and without `owner()`, AK06.
- Unit of work: **extend** `pekko-actor/actor/ask_firstreply_internal_test.go` with the scenarios of section 4. The runtime (`ask.go`, `ask_endpoint.go`) is expected to need **no change**. If a scenario fails on the unchanged runtime, classify it by root class (`skills/gopekko-failure-triage`) before touching `ask.go`; `ask_endpoint.go` is not writable.
- Only the two files already approved as writable in the #162 settings may be written: `ask.go` and `ask_firstreply_internal_test.go`.

## 1. Out of scope (later slices; do NOT implement or test)

Late reply after a timeout or a cancellation (#164, #165), terminated target (#166), measuring goroutine or timer leaks (AK10, #166; there is no timer yet), an exported `Ask`, any change to `ask_endpoint.go`, `errors.go`, `README`, `judge/` or CI.

## 2. Conventions (unchanged from the #162 RULEBOOK)

- R2.1 Header and citations: keep the file header; each new scenario carries a comment `// AS:625-627 (AK06)` and, for the adapted ones, `// typed AskSpec.scala:<line> (adapted: late after return, not after timeout)`.
- R2.2 Tests: package `actor`, go-specs (`specs.Describe`/`s.It`/`ctx.Expect`) per `go-testing`. Reuse the existing helpers (`bg`, `wait`, `closed`, `prsCollectDeadLetters`, `afrSpawn`, `afrAsk`, `afrTake`, `afrRefOf`, `afrWaitGroup`, `afrReq`, `afrRegistrySizes`, `newSystem`); new helpers use the `afr` prefix; never redeclare an existing one (check with Grep).
- R2.3 Determinism: **no `time.Sleep`, no timers, no polling**. Channels, `sync.WaitGroup`, a start barrier, and `System.deadLetterBarrier` only **after** the sender of each late reply has returned from `Tell` (the barrier covers only publications already started). Every wait bounded by `wait(t, ch)`. Any direct call that can block (such as `ask`) must run in a goroutine and be awaited through a bounded helper.
- R2.4 No `//nolint`; cyclomatic complexity <= 10 per function and literal; `gofmt`-clean.
- R2.5 Unknown behavior: `// PORT-GAP(<id>): <reason>`, never invented behavior.

## 3. Case table

| Upstream case | Disposition | Go scenario |
|---|---|---|
| `AskSupport.scala:625-627` message after completion goes to dead letters | faithful | L1, L2 |
| typed `AskSpec.scala:132` "publish dead-letter if the context.ask has completed on timeout" (message and recipient assertions) | adapted: late after return | L1, L4 |
| typed `AskSpec.scala:164` "publish dead-letter if the AskPattern.ask has completed on timeout" | adapted | L1 |
| after-timeout form of both | **omitted**, `PORT-GAP(late-after-timeout)`, belongs to #164 | none |

## 4. Required scenarios (each with an assertion that fails if the behavior breaks)

- L1 One late reply: `ask` returned `v`; a later `Tell` to the retained endpoint is published once as `DeadLetter{Msg: late, To: endpoint.ID()}`; the returned value and `endpoint.result()` stay `v`.
- L2 Several late replies, sent sequentially from the test goroutine: exactly that many dead letters, in order, result unchanged.
- L3 Cross-Ask: two asks A and B to the same target; B's request is held by the target until the test releases it; A returns; a late reply to A's endpoint is dead-lettered with `To == A.ID` and B stays unsettled; B then completes with its own reply; A's value is unchanged.
- L4 Recipient identity: the dead letter's `To` equals the endpoint's ID and differs from the target's ID and from the other ask's endpoint ID.
- L5 Registries: after the ask returned and after the late replies, `byID` and the name tables have the sizes they had before the ask, `lookupID(endpoint.ID())` finds nothing and the endpoint is not owned (`isOwned`).
- L6 Concurrent late replies: M goroutines released by a start barrier each send one late reply; exactly M dead letters (after the senders finish and the barrier runs), result unchanged, no data race under `-race`.

## 5. Per-file obligations

Only the test file grows; every scenario L1-L6 present with its citation. No new runtime code unless a failing scenario proves a defect (then the finding and the fix are recorded).

## 6. Model plan, limits and token definition (PROPOSED; Owner approves in RUNBOOK)

See `RUNBOOK.md` section 2. A subagent on an untriggered default model is a process violation.

## 7. Deviation log

None approved. Approval and effective authorship are recorded in this unit's `EXECUTION-RECORD.md`, not here.
