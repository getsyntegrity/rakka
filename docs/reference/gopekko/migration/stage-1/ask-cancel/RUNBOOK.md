# RUNBOOK: Ask cancellation, mandatory deadline and cleanup (#165)

Status: **prepared, not executed.** The loop does not start until the Owner approves the hashes in the approval request (kit rule 3). Base: `develop` `9317a59` (#164 merged). Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`.

## 1. What changes versus #164 (needs explicit approval)

| Aspect | #164 | #165 |
|---|---|---|
| Behavior | the deadline is optional; `ctx.Done` is not observed | **the deadline is mandatory** (`ErrAskNoDeadline`); cancellation is observed before, during `build` and during the wait; the unexported `ask` changes behavior for every caller (there is no caller outside the package tests) |
| Public surface | one exported identifier, `ErrAskTimeout` | two more, **`ErrAskCanceled` and `ErrAskNoDeadline`** (ADR 0015 D3 lists the names as `[Proposal]`; the Owner accepted the conditions) |
| Writable files | 4: `errors.go`, `ask_endpoint.go`, `ask.go`, the new timeout test | **6**: the same three runtime files, the new `ask_cancel_internal_test.go`, and **two existing test files that were read-only before**: `ask_firstreply_internal_test.go` (to give its asks a deadline) and `ask_timeout_internal_test.go` (to replace T6) |
| Permissions | settings `3d9bbe5b...` | new template with the six files (section 5); launcher byte-identical to the approved one (`39611d65...`) |
| Existing tests | read-only | edited **only** as RULEBOOK R4.3 says: one helper `afrCtx`, `afrAsk`, two direct calls, and T6 replaced by C4. Any other change to them is a process violation |

## 2. Scope decisions to confirm

RULEBOOK section 1 lists F1-F8 (classification by `ctx.Err()`, exports, editing the existing tests instead of a shim, a reply after cancel is dead-lettered, AK14 in scope, the reading of "libera su entrada del registro" as "no entry ever exists and the sizes do not change", the tie rule, cancel during `build`). They are proposals; approving the RULEBOOK hash approves them. They are the only places where the ADR is silent or `[Proposal]`.

## 3. Model plan, limits, token definition (PROPOSED; same as the approved #164 plan)

| Role | Model | Count |
|---|---|---|
| Architect/TL: brief design review of the RULEBOOK and the behavior specification | `opus` | 1 |
| Implementer (runtime, tests) and fixer (confirmed findings only) | `sonnet` | 1 + 0-1 |
| Architect/TL final review and principal reviewer (with an exact FIXSPEC when it finds something) | `opus` | 1 |
| Second reviewer, read-only | `sonnet` | 1 |
| Review of the fix delta | `opus` | 1 |

Limits: <= 6 loop agents plus one `haiku` agent for the permission probe, <= 60 min wall, US$ 6 per agent as a cap, subscription only, stop if a step would need extra payment. Tokens (limit 600k) are input + output + cache creation per agent JSON; cache reads are reported separately. Models and costs are self-reported by the CLI JSON. Reviewer read-only access is controlled by the prompt and by `shasum` of the six writable files before and after each reviewer; the settings do not enforce it. **Risk declared:** #164 used 320k tokens with four files; this unit has six files and a larger behavior surface, so the 600k limit is closer. If a step would pass it I stop and report instead of continuing.

## 4. Lessons from #164 built into the procedure

- The design review (Opus) happens before any code, and its guidance goes to the implementer prompt.
- Any claim about order ("armed before the send", "nothing sent") is proved by a deterministic probe, not by chance; the prompt carries the mechanisms that worked: the target cell's `told` counter inside a stub clock, a FIFO probe message, `fc.Pending()`.
- The implementer prompt forbids the forms that staticcheck rejects (SA1032) and states the shell rule (no `cd`, no chaining, no pipes).
- The mutation list is written against the real code after the implementation (patterns must match), including the mutants that survived last time (order of the pre-checks, a timer not stopped on cancel).
- The spec is a contract, not a suggestion: if the implementer cannot meet it, it reports instead of working around it.

## 5. Procedure after approval

1. Probe the new settings with a `haiku` agent through `run-agent.sh` (the seventh agent, for the probe only): writing the six targets allowed (stubs removed afterwards), writing `judge/probe_stub.py` and `pekko-actor/actor/ask_late_probe.go` denied, `gofmt -w` on a non-target denied, `go test` denied. Record raw results.
2. Record `shasum` of the six writable files.
3. Opus design review (read-only); a BLOCKER goes to the Owner.
4. Sonnet implements section 2 of the RULEBOOK and scenarios C1-C10; `gofmt`, `go vet`, `go build` in the loop; the orchestrator runs `go test`, including the 500-run `-race` stress.
5. Orchestrator gate and **isolated mutations** (copies of `pekko-actor`, `GOWORK=off`; the delivery tree is only read): cancel never observed; cancel classified as timeout; cancel returns a nil error; a canceled context still builds or sends (pre-check dropped); the order of the pre-checks swapped (no-deadline before cancel; expired before canceled); `DeadlineExceeded` from `ctx.Done` classified as canceled; the timer not stopped on cancel; a reply after cancel not dead-lettered; `outcome()` ignoring the canceled state; the wrong cause of `ErrAskCanceled`; the mandatory-deadline check dropped; a cancel during `build` still sends; a goroutine started by `ask` is not tested (declared limit). Each must be detected by at least one test and the control must pass. A mutation that reveals a runtime defect is reported before the scope grows.
6. Opus final review and Sonnet second review in parallel; disagreements settled by a cited source line or a measurement; fixer on confirmed findings only; Opus reviews the fix delta. Intermediate review envelopes are left unanswered; the native review is requested **once**, by sections sized under the engine's budget (the whole-branch review failed for #164 with `lens_context_budget_exceeded`): the plan is the implementation commit, the fix commit, and the test-edit commit as separate ranges, each with its exact base.
7. The orchestrator updates the `pekko-actor` README paragraph on `ask` (manual, recorded).
8. Gates: `gofmt`, `go vet`, `go mod tidy -diff`, `go test -race` in the six modules and 500 `-race` runs of all the Ask tests, `golangci-lint` v2.14.0, coverage gate over six profiles and its selftest, `check_deps.sh`, `check-headers.sh`, `check-adr-signatures.sh`, `check-judge-matrix.py`, `modules_selftest.sh`, `go-sdk-update.sh --check`. If the matrix checker reports the Go judge evidence as stale, renew the campaign with `judge/mutate_go.py --runs 30` (as in #164) and say so.
9. One `EXECUTION-RECORD.md`, one cost entry, and a PR against `develop`. No merge without the Owner's word.

## 6. Known follow-ups carried over (not in this unit)

`afrBuildPanics` calls `ask` without an upper bound (R3-001 of #162; this unit touches its call only to add the deadline, not to bound it); `run-agent.sh` does not escape the worktree root; `TestAskLateReply_Concurrent` drops the `ok` of `dl.Msg.(int)`. The launcher is copied byte for byte, so its known weakness is carried too.

## 7. Hashes for approval

Listed in the approval request, not repeated here to avoid a self-referential hash.
