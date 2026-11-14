# RUNBOOK: Export Ask as the usable API (unit ask-export)

Status: **prepared, not executed.** The loop does not start until the Owner approves the hashes in the approval request (kit rule 3). Base: `develop` `81906d2` (#166 merged). Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Branch: `feat/actor-ask-export`. There is no GitHub issue for this unit yet (see section 6).

## 1. What changes versus #166 (needs explicit approval)

| Aspect | #166 | ask-export |
|---|---|---|
| Behavior | `ask` fails with `ErrRecipientTerminated` for a completed termination | **none**: the exported `Ask` is a thin wrapper, the integrated semantics are unchanged |
| Public surface | `ErrRecipientTerminated` | **one exported function, `Ask[T, R any](ctx, target Ref[T], build func(Ref[R]) T) (R, error)`** (ADR 0015 D1 signature approved by the Owner 2026-10-14) |
| Writable files | 4 | **3**: `ask.go`, and the new `ask_api_test.go` and `ask_example_test.go` (package `actor_test`). Every existing file is read-only |
| Tests | internal, fake clock | **external package**, exported identifiers only; deterministic without the fake clock (section 2 of the RULEBOOK, G3) |
| Docs | manual after the loop | manual after the loop, longer: README usage section, ADR 0015, parity doc, ADR signature stub (section 5) |
| Judge | campaign renewed | `ask.go` changes, so the campaign is renewed again (section 6) |
| Permissions | settings `772ae729...` | new template with the three files; launcher byte-identical to the approved one (`39611d65...`, checked with `cmp`) |

## 2. Scope decisions to confirm

RULEBOOK section 1 lists G1-G6: the wrapper and its body, the documentation that `Ask` blocks, requires a deadline, and blocks the calling actor when used from `Receive`; the external-package tests; AK11 (a divergence from classic Pekko, no parity claimed); `ExampleAsk`; and the out-of-scope list (parity, Shutdown, `AskFrom`, `StatusReply`/`AskStatus`, benchmarks, judge scenarios).

Honest limit of the external suite: the clock is unexported, so the pending timeout driven by a clock is **not** repeated at the public level; it stays covered by the internal fake-clock suite. The external timeout test uses an already expired deadline.

## 3. Model plan, limits, token definition (PROPOSED; same as the approved #166 plan)

| Role | Model | Count |
|---|---|---|
| Architect/TL: brief design review of the RULEBOOK and the scenarios | `opus` | 1 |
| Implementer (wrapper, doc, tests, example) and fixer (confirmed findings only) | `sonnet` | 1 + 0-1 |
| Architect/TL final review and principal reviewer | `opus` | 1 |
| Second reviewer, read-only | `sonnet` | 1 |
| Review of the fix delta | `opus` | 1 |

Limits: <= 6 loop agents plus one `haiku` agent for the permission probe, <= 60 min wall, US$ 6 per agent as a cap, subscription only, stop if a step would need extra payment. Tokens (limit 600k) are input + output + cache creation per agent JSON, **the Haiku probe included**; cache reads are reported apart. Models and costs are self-reported by the CLI JSON. Reviewer read-only access is controlled by the prompt and by `shasum` of the writable files before and after each reviewer; the settings do not enforce it. #166 used 305,908 tokens with four of six agents; this unit is smaller in code and larger in tests and docs, and I expect a similar figure. If a step would pass the limit I stop and report.

## 4. Lessons from #164-#166 built into the procedure

- The design review (Opus) happens before any code and its guidance goes into the implementer prompt.
- The implementer prompt states the header rule (no `// Ported from` line in the new files; `ask.go` keeps its own), the shell rule (no `cd`, no chaining, no pipes), the staticcheck SA1032 rule, the rule that calls that should return immediately also run through a bounded helper, and the rule that external tests may not see the fake clock.
- Mutations are written against the real code after the implementation, in isolated copies (`GOWORK=off`). The list for this unit: the wrapper adds a default timeout when there is no deadline (E2); the wrapper replaces `ctx` with `context.Background()`; the wrapper ignores `build`; the wrapper wraps the error (E3, E4, E9 by identity); the wrapper swallows `ErrRecipientTerminated`; the wrapper swaps its arguments; `Ask` panics-recovers into a nil error; the wrapper starts a goroutine (declared undetectable, verified by reading). Results are kept as measured, detected over total, with survivors named.
- Evidence reuse is claimed only after comparing the inputs of the **whole** tree the gate reads (README included). The orchestrator runs `-race` once on the final code and reuses it when only comments or docs change afterwards; the 500-run stress runs on the implementation, not by routine.

## 5. Procedure after approval

1. Probe the new settings with a `haiku` agent through `run-agent.sh`: writing the three targets allowed (stubs removed afterwards), writing any existing test file, `errors.go`, `ask_endpoint.go` and `judge/probe_stub.py` denied, `gofmt -w` on a non-target denied, `go test` denied. Record raw results.
2. Record `shasum` of the writable files and of the files that must stay unchanged.
3. Opus design review (read-only); a BLOCKER goes to the Owner.
4. Sonnet implements section 2 of the RULEBOOK and scenarios E1-E10; `gofmt`, `go vet`, `go build` in the loop; the orchestrator runs `go test`.
5. Orchestrator gate and isolated mutations.
6. Opus final review and Sonnet second review in parallel; disagreements settled by a cited line or a measurement; fixer on confirmed findings only; Opus reviews the fix delta if code changed.
7. Manual docs after the loop (orchestrator, recorded): `pekko-actor/README.md` (a usage section with the example and the three documented facts, the Layout paragraph rewritten to say `Ask` is exported, the "five executable examples" count), the root `README.md` row, `docs/actor-core-parity.md` row, `docs/adr/0015-ask-and-reply.md` (D1 marked `[Owner, 2026-10-14]`, AK11 and increment I-6 marked done, nothing else), and `docs/adr/signatures/api.go` (type parameters renamed `T, R` to match D1; `check-adr-signatures.sh` must stay green).
8. Native review: the sections are the package commit, the implementation commit, the fix commit if any, each with its exact base, requested **once** and presented for consent only after the gates.
9. Gates: `gofmt`, `go vet`, `go mod tidy -diff`, `go test -race` in the six modules, `golangci-lint` v2.14.0, coverage gate over six profiles and its selftest, `check_deps.sh`, `check-headers.sh`, `check-adr-signatures.sh`, `check-judge-matrix.py`, `modules_selftest.sh`, `go-sdk-update.sh --check`.
10. One `EXECUTION-RECORD.md`, one cost entry, and a PR against `develop`. No merge without the Owner's word.

## 6. Judge campaign, evidence tag and the missing issue (needs the Owner's decision)

- `ask.go` changes, so the Go judge campaign must be renewed (`judge/mutate_go.py --runs 30`), and its commit is dropped by the squash. **Approval of this package asks for one decision**: authorize publishing one more `evidence/*` tag for exactly the commit cited by this unit's renewed report, checked against its manifest (as for `evidence/go-campaign-2b3628a`), or accept that HISTORICAL evidence is not retained after the squash. The tag does not make a stale CURRENT result current.
- There is no GitHub issue for this unit: #167 is the supervision ADR and the Ask epic #54 is closed. I will not create one without your authorization; the PR will say so. If you want a ticket, say so and I create it with the scope of RULEBOOK section 0.

## 7. Known follow-ups carried over (not in this unit)

`aftmLetter` goroutine parked if the wait fails (R3-001); `afrBuildPanics` calls `ask` without an upper bound; `run-agent.sh` does not escape the worktree root; `TestAskLateReply_Concurrent` drops the `ok` of `dl.Msg.(int)`; the C8 cancel branch is not required to win; the `ask` doc describes the expired-deadline check before the no-deadline check while `askPrecheck` does the reverse; ADR 0015 cites `finish` where the code is `finalize`.

## 8. Hashes for approval

Listed in the approval request, not repeated here to avoid a self-referential hash.
