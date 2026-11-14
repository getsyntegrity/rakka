# RULEBOOK: Ask first reply (#162, unit ask-first-reply-v2)

Status: **DRAFT, NOT approved.** Read-only inside any loop (kit `CLAUDE.md` rule 1). Replaces the #429 attempt (closed without merge; nothing is continued from it).

## 0. Unit

- Capability: unexported `ask` sends a request carrying an own reply endpoint, returns the **first** reply, keeps that result when a duplicate reply arrives, and publishes the losing reply as `DeadLetter{Msg, To: endpoint ID}`.
- Source (Pekko v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, read-only):
  - `upstream/pekko/actor/src/main/scala/org/apache/pekko/pattern/AskSupport.scala`: `PromiseActorRef` `!` (lines 614-629; `tryComplete` 625, dead-letter send 627).
  - `.../actor-tests/src/test/scala/org/apache/pekko/pattern/AskSpec.scala:34` "send request to actor and wrap the answer in Future".
  - `.../PromiseRefSpec.scala:54` "receive only one message".
- Manifest note: `manifest.tsv` lists one source per target, as in earlier units. The test file also ports `PromiseRefSpec.scala:54`; that co-source is recorded here and cited in the test header, not as a third manifest row.
- Targets (the only two files agents may write): `pekko-actor/actor/ask.go`, `pekko-actor/actor/ask_firstreply_internal_test.go`.
- Governing decisions (Owner-accepted in ADR 0015, `docs/adr/0015-ask-and-reply.md`): D4 single CAS terminal transition; D8 endpoint is an unexported `Ref[R]`, **not** in `System.names`/`System.byID`, no `owner()`; losing `Tell` dead-letters through the owner System.
- Already on develop, reused unchanged: `askEndpoint[R]`, `newAskEndpoint`, `Tell`, `settled()`, `result()`, `askTarget` in `ask_endpoint.go` (#161).

## 1. Out of scope (later slices; do NOT implement)

Deadline and timer, `ErrAsk*` sentinels, pre-send checks (D5 steps 1-3), context cancellation while waiting, completed-termination detection, endpoint cleanup. `ask` therefore waits only on `settled()` and documents that it ignores `ctx` for waiting. No new exported symbol. No change to `ask_endpoint.go`, `errors.go`, `README`, `judge/` or CI.

## 2. Go mapping

| Pekko | Go |
|---|---|
| `ask(target, msg)` creating a `PromiseActorRef` | `ask[T, R any](ctx context.Context, target Ref[T], build func(Ref[R]) T) (R, error)`; `r, err := askTarget(target)` validates the target and returns the native `*ref[T]`; endpoint is `newAskEndpoint[R](r.c.sys)`; the request is `build(endpoint)` sent with `r.Tell(ctx, ...)` |
| `PromiseActorRef.!` first `tryComplete` wins | already `askEndpoint.Tell` (CAS) |
| losing message to `provider.deadLetters` | already `System.publish(DeadLetter)` in `Tell` |
| `Future` result | `<-e.settled()` then `e.result()` |

`build` is called exactly once, on the caller's goroutine, after the target is validated (ADR 0015 D1).

## 3. Conventions

- R3.1 Header: Apache-2.0 notice exactly as in `ask_endpoint.go`, then `// Ported from <source path in upstream/pekko>`. Test file cites `AskSpec.scala:34` and `PromiseRefSpec.scala:54` per scenario.
- R3.2 Tests: package `actor` (internal), go-specs (`specs.Describe`/`s.It`/`ctx.Expect`) per `go-testing`; reuse helpers `bg`, `wait`, `closed`, `prsCollectDeadLetters`, `newSystem(Config{}, options{})`; new helpers prefixed `afr`. Do not redeclare an existing helper (check with `rg`).
- R3.3 Determinism: **no `time.Sleep`, no timers, no polling loops**. Synchronise with channels, `sync.WaitGroup`, a start barrier, `System.deadLetterBarrier`. Every wait bounded by `wait(t, ch)`.
- R3.4 No `//nolint`; `gocyclo` <= 10; `gofmt`-clean; `go vet`/`go build` pass. `go test` is denied in the loop; the orchestrator runs it at the gate.
- R3.5 Unknown or uncovered behavior: `// PORT-GAP(<id>): <reason>` marker (kit rule 8), never invented behavior.

## 4. Required test scenarios (ADR 0015 AK01-AK04 slice, ids in test names)

1. AK01/AK03 immediate reply: target replies inside its turn; `ask` returns it; `build` called once.
2. Two sequential replies: first returned, second dead-lettered, result unchanged.
3. Two concurrent replies (start barrier, N goroutines): exactly one winner, N-1 dead letters, stable result.
4. Stable result + exactly one dead letter whose `Msg` is the losing reply and `To` the endpoint ID.
5. AK02 independent concurrent asks to the same target do not cross.
6. Endpoint absent from `System` registries: `len(byID)`/`len(names)` unchanged after `ask`; endpoint does not implement `owner()`.
7. Port of `AskSpec.scala:34` (request wrapped, answer returned).

## 5. Per-file obligations

`ask.go`: only `ask` (unexported) plus its doc comment stating the limits of section 1. Test file: scenarios 1-7, each citing its upstream line or AK id.

## 6. Model plan (PROPOSED; Owner approves in RUNBOOK)

See `RUNBOOK.md` section 3. Running a subagent on an untriggered default is a process violation (kit `CLAUDE.md`).

## 7. Deviation log

None approved. The authorship change in `RUNBOOK.md` section 2 needs Owner approval before it is recorded here as DEV-1.
