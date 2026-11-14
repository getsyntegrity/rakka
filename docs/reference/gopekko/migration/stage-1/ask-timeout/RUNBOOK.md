# RUNBOOK: Ask timeout (#164)

Status: **prepared, not executed.** The loop does not start until the Owner approves the hashes listed in section 6 (kit rule 3). Base: `develop` `fab08e5` (#163 merged). Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`.

## 1. What changes versus #162 and #163 (needs explicit approval)

| Aspect | #162, #163 | #164 |
|---|---|---|
| Runtime | #162: kit wrote `ask.go`; #163: no runtime change | **kit changes the runtime**: `errors.go` (one new exported sentinel `ErrAskTimeout`), `ask_endpoint.go` (state `askTimedOut`, `expire`, `timedOut`), `ask.go` (deadline, timer) |
| Public surface | none (`ask` unexported) | one new **exported** identifier, `actor.ErrAskTimeout`. ADR 0015 D3 lists the name as `[Proposal]`; the Owner accepted `ErrAskTimeout` on the condition that `errors.Is(err, context.DeadlineExceeded)` holds, which this unit implements and tests. No other sentinel is added |
| Writable files | `ask.go` and `ask_firstreply_internal_test.go` | `errors.go`, `ask_endpoint.go`, `ask.go` and the new `ask_timeout_internal_test.go` (four files; `ask_firstreply_internal_test.go` becomes read-only) |
| Permissions | settings `8ffce17a...` | new template (section 6) with the four files; launcher byte-identical to the approved one (`39611d65...`) |
| Behavior without a deadline | n/a | no timer; waits for the reply (unchanged). D2's mandatory deadline is **not** enforced yet |

## 2. Scope decision to confirm: who owns the pre-send checks

ADR 0015 puts the timer (AK05) and the pre-send checks (AK08 expired deadline, AK12 canceled context, AK15 no deadline, AK22 order) in the same increment (I-2) and does not map them to tickets #163 to #166; #162 left them out. This unit takes **only the part that is a timeout**: AK05, AK09 and AK08 (a deadline that is not in the future returns `ErrAskTimeout` without calling `build` or sending, judged by the injected clock, D5 step 2). The context-state checks and the mandatory deadline (AK12, AK15, AK22, `ErrAskCanceled`, `ErrAskNoDeadline`) go to #165 together with cancellation. If the Owner prefers a separate ticket for all pre-send checks, the only change here is to drop T5 and the expired-deadline branch of `ask.go`.

## 3. Model plan, limits, token definition (PROPOSED; same as the approved #163 plan)

| Role | Model | Count |
|---|---|---|
| Architect/TL: brief design review of the RULEBOOK scenarios and the behavior specification | `opus` | 1 |
| Implementer (runtime and tests) and fixer (confirmed findings only) | `sonnet` | 1 + 0-1 |
| Architect/TL final review and principal reviewer (contract, upstream source, diff only) | `opus` | 1 |
| Second reviewer, read-only | `sonnet` | 1 |
| Third reviewer, concrete disagreement only, and any review of the fix delta | `sonnet` or `opus` | within the limit |

Limits: <= 6 agents, <= 60 min wall, US$ 6 per agent as a cap, subscription only, stop if a step would need extra payment. Tokens (limit 600k) are input + output + cache creation per agent JSON; cache reads are reported separately. Models and costs are self-reported by the CLI JSON. Reviewer read-only access is controlled by the prompt and by `shasum` of the four writable files before and after each reviewer; the settings do not enforce it. Because the runtime now changes, the fix-delta review counts as one of the six agents, so the plan keeps no spare: a third reviewer replaces it only if needed.

## 4. Procedure after approval

1. Probe the new settings with a `haiku` agent through `run-agent.sh`: writing `pekko-actor/actor/ask_firstreply_internal_test.go` and `judge/mutate_go.py` must be denied, writing the four targets allowed (stubs removed afterwards), `gofmt -w` on a non-target denied, `go test` denied. Record raw results.
2. Record `shasum` of the four writable files and of `ask_firstreply_internal_test.go`.
3. Opus design review (read-only); its guidance goes to the implementer prompt; a BLOCKER goes to the Owner.
4. Sonnet implements the behavior specification and scenarios T1-T7; `gofmt`, `go vet`, `go build` in the loop; the orchestrator runs `go test`.
5. Orchestrator gate and **isolated mutations** (copies of `pekko-actor` with `GOWORK=off`; the delivery tree is only read): the timeout never fires; fires early; the reply path does not stop the timer; the timeout overwrites a reply (no CAS); an expired deadline still sends; the timeout returns a nil error; `ErrAskTimeout` without its cause; a reply after the timeout is not dead-lettered; `build` called before the expired check. Each must be detected by at least one test, and the control must pass. A mutation that reveals a runtime defect is reported before the scope grows.
6. Opus final review and Sonnet second review in parallel; disagreements settled by a cited source line or a measurement; fixer on confirmed findings only; Opus reviews the fix delta.
7. The orchestrator updates the `pekko-actor` README paragraph on `ask` and removes the now-resolved `PORT-GAP(late-after-timeout)` comment from `ask_firstreply_internal_test.go`; both are manual and recorded as such.
8. Gates: `gofmt`, `go vet`, `go mod tidy -diff`, `go test -race` in the six modules and 500 `-race` runs of the Ask tests, `golangci-lint` v2.14.0, coverage gate over six profiles and its selftest, `check_deps.sh`, `check-headers.sh`, `check-adr-signatures.sh`, `check-judge-matrix.py`, `modules_selftest.sh`, `go-sdk-update.sh --check`. The judge campaign inputs: `go-scope.json` names the `pekko-actor` tree; renew the campaign only if the checker says its inputs changed, and say so. 
9. One `EXECUTION-RECORD.md`, one cost entry, and a PR against `develop`. No merge without the Owner's word.

## 5. Known follow-ups carried over (not in this unit)

Unbounded `ask` call in `afrBuildPanics`; `run-agent.sh` does not escape the root; `TestAskLateReply_Concurrent` drops the `ok` of `dl.Msg.(int)`. The launcher is copied byte for byte, so its known weakness is carried too.

## 6. Hashes for approval

`RULEBOOK.md`, `RUNBOOK.md`, `manifest.tsv`, `settings.template.json` (new) and `run-agent.sh` (identical to the approved `39611d65...`): listed in the approval request, not repeated here to avoid a self-referential hash.
