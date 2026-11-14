# RULEBOOK: PromiseRefSpec -> Go (Ask reply endpoint, ACT-015)

Status: **DRAFT, complete, NOT approved.** The Owner approves this file before any loop runs (kit `CLAUDE.md` standing rule 3; prompt 04 prerequisite "amended RULEBOOK.md"). Nothing in this repository records an approval, so none is claimed. Authorship note: written by the orchestrating session (self-reported `claude-sonnet-5-5`), not by the largest available model that the kit recommends for rulebook authorship (`CLAUDE.md`, "Model guidance"); the Owner's review is therefore the only check on it.

This is a redesign, not a structure-preserving migration (`CLAUDE.md` of the repo): ADR 0015 and `SCOPE.md` replace the kit's generic rulebook. Modeled on `../watch-test-port/RULEBOOK.md`. Loop agents **cite this file, never edit it**; amendments go to `AMENDMENTS.md` for the Owner.

## 0. Unit of work

One upstream test file becomes one Go test file (`manifest.tsv`):

| Source (Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, read-only) | Target |
|---|---|
| `upstream/pekko/actor-tests/src/test/scala/org/apache/pekko/pattern/PromiseRefSpec.scala` | `pekko-actor/actor/ask_endpoint_promiseref_internal_test.go` |

The runtime (`ask_endpoint.go`, `errors.go`) and the existing tests (`ask_endpoint_internal_test.go`) are **not** units: they exist, are read-only for the loop, and were written manually (see `EXECUTION-RECORD.md`). The loop writes exactly one file: the target.

## 1. Target conventions

- **R1.1** `package actor` (internal): the endpoint is unexported (ADR 0015 D8). Tests use `github.com/getsyntegrity/go-specs` (`specs.Describe`, `s.It`, `ctx.Expect`, `assert`). No network, no `time.Sleep`, no `time.After` as synchronization or as a witness. Wait on channels with the bounded `wait(t, ch)` helper; for concurrent callers use a start gate channel and `sync.WaitGroup`; for time use the `fakeClock` of `clock_internal_test.go`.
- **R1.2** Already defined in package `actor` tests and **not to be redefined**: `bg`, `spawnInt`, `doneOf`, `wait`, `signal`, `newSink`, `wActor`, `fakeClock`, `newFakeClock`, `userRef`, `closed`, `isOwned`, `idsOf`, and every identifier listed by `rg -n '^(func|type|var|const) ' pekko-actor/actor/*_test.go`. New helpers are prefixed `prs`; new tests are named `TestAskEndpointPromiseRefSpec_<short>`.
- **R1.3** Header: the Apache-2.0 header of `pekko-actor/actor/ask_endpoint.go` and exactly one line `// Ported from upstream/pekko/actor-tests/src/test/scala/org/apache/pekko/pattern/PromiseRefSpec.scala` (checked by `.github/scripts/check-headers.sh`).
- **R1.4** Each ported test carries a comment `// PromiseRefSpec.scala:<line>: "<Scala test name>"` and, where it differs, one line saying how (R3).
- **R1.5** Lint and complexity: every function at most 10 cyclomatic complexity (function literals count); no `//nolint`; the file must pass `golangci-lint` at the version in `.golangci-version`.
- **R1.6** Unexported API available to the target: `newAskEndpoint[R](sys)`, `(*askEndpoint[R]).ID()`, `.Tell(ctx, msg)`, `.settled()`, `.result()`, `askTarget(target)`, `System.SubscribeDeadLetters`, `System.deadLetterBarrier`, `System.deadLettersPublished`. There is no exported `Ask`, `PromiseRef` or `StatusReply`; do not invent one.

## 2. Constructs and their mapping

| Scala | Go (this module) |
|---|---|
| `PromiseRef(5.seconds)` / `promiseRef.ref` / `promiseRef.future` | `newAskEndpoint[T](sys)` / the endpoint as `Ref[T]` / `<-e.settled()` then `e.result()` |
| `target ! Request(promiseRef.ref)` and the actor's `replyTo ! Response` | the test calls `e.Tell(bg, Response)` itself: the exported `Ask` and a `Request{ReplyTo}` round trip belong to #162 |
| `system.eventStream.subscribe(deadListener.ref, classOf[DeadLetter])` / `expectMsgType[DeadLetter]` | `sys.SubscribeDeadLetters(fn)` / `sys.deadLetterBarrier(bg)` then the collected `DeadLetter{Msg, To}` |
| `Await.result(f, 5.seconds)` | `wait(t, e.settled())` (bounded; no timeout value is a witness) |
| `ImplicitSender`, `TestProbe` | none: no implicit sender (ADR 0010 D2) |

## 3. Case table

| Upstream case | Line | Disposition | Go target |
|---|---|---|---|
| "complete promise with received message" | 37 | **reduced**: the target actor and `Request{replyTo}` are replaced by a direct `e.Tell`, because there is no exported `Ask` | first reply completes the endpoint |
| "throw IllegalArgumentException on negative timeout" | 48 | **omitted** here: the deadline is validated by the exported Ask (#162, ADR 0015 D2/D5); the endpoint has no timeout | `// PORT-GAP(promiseref-negative-timeout)` |
| "receive only one message" | 54 | **faithful** (adapted): second message becomes a `DeadLetter` | late reply dead-lettered, `To` = endpoint ID |
| "work with explicitly constructed PromiseRef's" (`PromiseRef.wrap`) | 67 | **omitted**: no Go counterpart, no wrapping API is proposed | `// PORT-GAP(promiseref-wrap)` |

## 4. Allowed semantics

Only what ADR 0015 D4 and D8 accept: one terminal transition, losing replies dead-lettered through the owner System, endpoint outside `byID` and the name tables, no `owner()`. Out of scope and forbidden: automatic Watch, `System.Shutdown`, cancellation of the target's processing, deadline or context handling, foreign-ref rejection (D5, D6, D7).

## 5. Escape hatch

`// PORT-GAP(<id>): <reason>` in the target, greppable. Never a skipped, weakened or deleted assertion. A failing test needs triage by root class (`skills/gopekko-failure-triage`), not a looser assertion (kit rule 7).

## 6. Per-file obligations

The implementer writes the target and ends with a status trailer (cases ported, reduced, omitted, `PORT-GAP` ids). Inside the loop the agents may run `gofmt`, `go vet` and `go build` (cheap referees promoted per kit rule 5 / README Step 4); `go test` stays denied and runs at the gate. Reviewers cite a rule of this file or a source line of the spec for every finding.

## 7. Model plan (PROPOSED; the Owner decides, kit `CLAUDE.md` "Model guidance")

| Role | Proposed selector | Reason |
|---|---|---|
| Implementer | `sonnet` (self-reports `claude-sonnet-5-5`) | one small file, mid tier |
| Reviewer 1, Reviewer 2 | `sonnet` | adversarial, separate contexts, read-only |
| Third reviewer (disagreements only) | `sonnet` | defaults to not-confirmed |
| Fixer | `sonnet` | confirmed findings only |

The selector is set explicitly on every agent call; an inherited default is a deviation to log in section 8. A self-reported model id is not an external verification.

## 8. Deviation log

None approved. Each deviation is added here by the Owner as DEV-n, with its scope. In particular, the manual authorship of `ask_endpoint.go`, `errors.go` and `ask_endpoint_internal_test.go` is **not** an approved deviation (see `AMENDMENTS.md` A-1).
