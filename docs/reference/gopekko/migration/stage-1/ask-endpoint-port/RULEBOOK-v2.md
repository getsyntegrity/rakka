# RULEBOOK v2: PromiseRefSpec -> Go (Ask reply endpoint, ACT-015)

Status: **version 2, prepared; its text awaits the Owner's approval at a named commit.** The Owner approved the clarifications listed in section 9 on 2026-10-12 (decision on `AMENDMENTS.md` A-10); the approval of this file as a whole is a separate act and is not recorded here.

**Historical approval of version 1.** Version 1 is the file `RULEBOOK.md` in this directory, last changed in commit `02b8e29`, sha256 `2da88e48cde5e6dfadf7c86453381dd5e540159d53661b1ca9a0d7ac882be859`. The Owner approved it on 2026-10-12 (recorded in `EXECUTION-RECORD.md` section 4) and it governed the kit loop executed from 20:44Z to 20:47Z that day. It is kept byte-identical (its own header still reads "DRAFT, complete, NOT approved" so that its approved hash does not change). **This version does not modify version 1 and does not apply retroactively: the executed loop, its reviews, its costs and the commit `59bd95b` remain governed by version 1.** Version 2 applies to any later loop on this unit.

Authorship note: written by the orchestrating session (self-reported `claude-sonnet-5-5`), not by the largest available model that the kit recommends for rulebook authorship (`CLAUDE.md`, "Model guidance"); the Owner's review is the only check on it.

This is a redesign, not a structure-preserving migration (`CLAUDE.md` of the repo): ADR 0015 and `SCOPE.md` replace the kit's generic rulebook. Modeled on `../watch-test-port/RULEBOOK.md`. Loop agents **cite this file, never edit it**; amendments go to `AMENDMENTS.md` for the Owner.

## 0. Unit of work

One upstream test file becomes one Go test file (`manifest.tsv`):

| Source (Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, read-only) | Target |
|---|---|
| `upstream/pekko/actor-tests/src/test/scala/org/apache/pekko/pattern/PromiseRefSpec.scala` | `pekko-actor/actor/ask_endpoint_promiseref_internal_test.go` |

The runtime (`ask_endpoint.go`, `errors.go`) and the existing tests (`ask_endpoint_internal_test.go`) are **not** units: they exist, are read-only for the loop, and were written manually (see `EXECUTION-RECORD.md`). The loop writes exactly one file: the target.

## 1. Target conventions

- **R1.1** `package actor` (internal): the endpoint is unexported (ADR 0015 D8). Tests use `github.com/getsyntegrity/go-specs` (`specs.Describe`, `s.It`, `ctx.Expect`, `assert`). No network, no `time.Sleep`, no `time.After` as synchronization or as a witness. Wait on channels with the bounded `wait(t, ch)` helper; for concurrent callers use a start gate channel and `sync.WaitGroup`; for time use the `fakeClock` of `clock_internal_test.go`.
- **R1.2** *(clarified in v2)* Already defined in the tests of **package `actor`** (the package of the target, files `*_internal_test.go`) and **not to be redefined**: `bg`, `spawnInt`, `doneOf`, `wait`, `signal`, `newSink`, `wActor`, `fakeClock`, `newFakeClock`, `userRef`, `closed`, `isOwned`, `idsOf`, and every identifier listed by `rg -n '^(func|type|var|const) ' pekko-actor/actor/*_internal_test.go`. In particular `bg` is declared at `pekko-actor/actor/cell_internal_test.go:19` and is the one the target uses. A second `var bg` exists at `pekko-actor/actor/helpers_test.go:12` but belongs to the **external** test package `actor_test`; it is a different package, creates no clash, and is neither used nor redefined by the target. New helpers are prefixed `prs`; new tests are named `TestAskEndpointPromiseRefSpec_<short>`.
- **R1.3** *(clarified in v2)* Header, exactly this: copy the first ten lines of `pekko-actor/actor/ask_endpoint.go` (the nine lines of the Apache-2.0 notice, `// Licensed under the Apache License, Version 2.0 (the "License");` through `// limitations under the License.`, followed by one empty comment line `//`), **omit** the eleventh line of that file (its `// Ported from ...AskSupport.scala` line belongs to that file's own source), and write in its place exactly one line:

  `// Ported from upstream/pekko/actor-tests/src/test/scala/org/apache/pekko/pattern/PromiseRefSpec.scala`

  then one blank line and `package actor`. The header must contain exactly one `Ported from` line (checked by `.github/scripts/check-headers.sh`).
- **R1.4** Each ported test carries a comment `// PromiseRefSpec.scala:<line>: "<Scala test name>"` and, where it differs, one line saying how (R3).
- **R1.5** Lint and complexity: every function at most 10 cyclomatic complexity (function literals count); no `//nolint`; the file must pass `golangci-lint` at the version in `.golangci-version`.
- **R1.6** Unexported API available to the target: `newAskEndpoint[R](sys)`, `(*askEndpoint[R]).ID()`, `.Tell(ctx, msg)`, `.settled()`, `.result()`, `askTarget(target)`, `System.SubscribeDeadLetters`, `System.deadLetterBarrier`, `System.deadLettersPublished`. There is no exported `Ask`, `PromiseRef` or `StatusReply`; do not invent one.
- **R1.7** *(new in v2)* Type of `R`: in this target the endpoint is instantiated as `newAskEndpoint[any](sys)`, that is `R` is **`any`**. Reason: the upstream spec sends heterogeneous messages (`Response`, `FirstMessage`, `SecondMessage`) to one endpoint. Messages are unexported `prs`-prefixed struct types; the stored reply is compared as `ToEqual(any(<value>))`. The manual tests in `ask_endpoint_internal_test.go` use `R = int`; that is their own choice and does not bind the target.

## 2. Constructs and their mapping

| Scala | Go (this module) |
|---|---|
| `PromiseRef(5.seconds)` / `promiseRef.ref` / `promiseRef.future` | `newAskEndpoint[any](sys)` (R1.7) / the endpoint as `Ref[any]` / `<-e.settled()` then `e.result()` |
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

The selector is set explicitly on every agent call; an inherited default is a deviation to log in section 8. A self-reported model id is not an external verification. The Owner's approval of the model plan and its limits on 2026-10-12 was given for the executed loop; a later loop under version 2 needs its own confirmation.

## 8. Deviation log

None approved. Each deviation is added here by the Owner as DEV-n, with its scope. In particular, the manual authorship of `ask_endpoint.go`, `errors.go` and `ask_endpoint_internal_test.go` is **not** an approved deviation (see `AMENDMENTS.md` A-1; decision D1 of 2026-10-12 keeps it explicit and does not credit it to the kit).

## 9. Changes from version 1 (all clarifications; no case, disposition or permission changes)

| Where | Version 1 | Version 2 | Source |
|---|---|---|---|
| Header | "DRAFT, complete, NOT approved" | status of v2, plus the historical approval of v1 with hash and commit | `AMENDMENTS.md` A-10 (4) |
| R1.3 | "the Apache-2.0 header of `ask_endpoint.go` and exactly one line `// Ported from ...PromiseRefSpec.scala`" (ambiguous: the header of `ask_endpoint.go` already ends with a `Ported from` line) | first ten lines copied, the eleventh omitted, one exact `Ported from` line written | A-10 (1) |
| R1.7 (new), section 2 | the Go type of `R` was not stated | `R` is `any` | A-10 (2) |
| R1.2 | "`bg` ... in package `actor` tests"; `bg` also exists in `actor_test` | the package and the file:line of the `bg` the target uses; the external-package `bg` named as a different package | A-10 (3) |
| Section 7 | approval implicit in the executed loop | later loops need their own confirmation | n/a |

Conformance of the existing target: `ask_endpoint_promiseref_internal_test.go` at `59bd95b` already has the R1.3 header (ten license lines plus the single `PromiseRefSpec.scala` line) and uses `newAskEndpoint[any]` (R1.7), so version 2 requires no change to it.
