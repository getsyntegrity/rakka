# RUNBOOK: Ask on a terminated target and a pending Stop (#166)

Status: **prepared, not executed.** The loop does not start until the Owner approves the hashes in the approval request (kit rule 3). Base: `develop` `18d7746` (#165 merged). Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`.

## 1. What changes versus #165 (needs explicit approval)

| Aspect | #165 | #166 |
|---|---|---|
| Behavior | the context is observed, the deadline is mandatory | a target whose termination **completed** fails with `ErrRecipientTerminated` (the request is built, delivered to dead letters, no timer); a target with an **accepted Stop** keeps the Ask pending until deadline or cancel |
| Public surface | `ErrAskCanceled`, `ErrAskNoDeadline` | **one more exported identifier, `ErrRecipientTerminated`** (ADR 0015 D3 lists the name as `[Proposal]`; the condition is `[Owner]`) |
| Writable files | 6, two of them old tests | **4**: `errors.go`, `ask_endpoint.go`, `ask.go`, the new test file. **Every existing test stays read-only** (none asks a stopped target) |
| Permissions | settings `2a44a196...` | new template with the four files; launcher byte-identical to the approved one (`39611d65...`, checked with `cmp`) |
| Judge | campaign renewed | `ask.go` changes again, so the Go judge evidence will be stale again and the campaign must be renewed (see section 6) |

## 2. Scope decisions to confirm

RULEBOOK section 1 lists F1-F6: the terminated path builds, delivers to dead letters and returns the error; a new terminal state `rejected`; termination read as the closed `done` channel of the cell (not an accepted Stop); the window between the check and the send is accepted as in the ADR; `ErrRecipientTerminated` exported now without a cause; and the scope (AK07 adapted from the typed `AskSpec.scala:71`, AK18, AK19a, AK19b, the tail of AK22). The Pekko cases for dead letters, an empty ref and a nonexistent actor have no Go analogue and are omitted with a `PORT-GAP`.

## 3. Model plan, limits, token definition (PROPOSED; same as the approved #165 plan)

| Role | Model | Count |
|---|---|---|
| Architect/TL: brief design review of the RULEBOOK and the behavior specification | `opus` | 1 |
| Implementer (runtime, tests) and fixer (confirmed findings only) | `sonnet` | 1 + 0-1 |
| Architect/TL final review and principal reviewer (with an exact FIXSPEC when it finds something) | `opus` | 1 |
| Second reviewer, read-only | `sonnet` | 1 |
| Review of the fix delta | `opus` | 1 |

Limits: <= 6 loop agents plus one `haiku` agent for the permission probe, <= 60 min wall, US$ 6 per agent as a cap, subscription only, stop if a step would need extra payment. Tokens (limit 600k) are input + output + cache creation per agent JSON, **the Haiku probe included**; cache reads are reported apart. Models and costs are self-reported by the CLI JSON. Reviewer read-only access is controlled by the prompt and by `shasum` of the four writable files before and after each reviewer; the settings do not enforce it. #165 used 487k tokens with six files; with four files and no edits of old tests I expect less, but if a step would pass the limit I stop and report.

## 4. Lessons from #164 and #165 built into the procedure

- The design review (Opus) happens before any code and its guidance goes to the implementer prompt.
- The implementer prompt states the header gate (the `// Ported from` line must end in `.scala`), the shell rule, and the staticcheck SA1032 rule; it requires calls that should return immediately to run through a bounded helper too (T6 of #164 hung the binary).
- Mutations are written against the real code after the implementation; each mutant that survives is reported with the test that closes it. The mutant list includes the ones that matter for this unit: an accepted Stop treated as termination, the order of step 4 relative to the pre-checks, the request not dead-lettered on the rejected path, `build` not called on it, and the check placed in `askTarget` (D6).
- The measured results are kept as measured: detected over total, with the undetected ones named (a goroutine started by `ask` stays undetectable by tests and is verified by reading).
- Evidence reuse is claimed only after comparing the inputs of the **whole** tree the gate reads (including the README), not only `pekko-actor/actor`.

## 5. Procedure after approval

1. Probe the new settings with a `haiku` agent through `run-agent.sh`: writing the four targets allowed (stubs removed afterwards), writing any existing test file, `clock.go`, `stop.go` and `judge/probe_stub.py` denied, `gofmt -w` on a non-target denied, `go test` denied. Record raw results.
2. Record `shasum` of the four writable files and of the existing test files.
3. Opus design review (read-only); a BLOCKER goes to the Owner.
4. Sonnet implements section 2 of the RULEBOOK and scenarios K1-K8; `gofmt`, `go vet`, `go build` in the loop; the orchestrator runs `go test`, including the 500-run `-race` stress.
5. Orchestrator gate and **isolated mutations** (copies of `pekko-actor`, `GOWORK=off`; the delivery tree is only read).
6. Opus final review and Sonnet second review in parallel, with the checks the Owner cares about: completed termination versus accepted Stop, the order of step 4, single winner and cleanup on the rejected path; disagreements settled by a cited line or a measurement; fixer on confirmed findings only; Opus reviews the fix delta.
7. Native review: the engine accepted 735 lines in one range in #165 and refused 1,461 in #164; the sections are the implementation commit, the fix commit, each with its exact base, requested **once**.
8. The orchestrator updates the README paragraph on `ask`, `docs/actor-core-parity.md` and the root README row that mention #166 (manual, recorded) after the loop.
9. Gates: `gofmt`, `go vet`, `go mod tidy -diff`, `go test -race` in the six modules and 500 `-race` runs of all the Ask tests, `golangci-lint` v2.14.0, coverage gate over six profiles and its selftest, `check_deps.sh`, `check-headers.sh`, `check-adr-signatures.sh`, `check-judge-matrix.py`, `modules_selftest.sh`, `go-sdk-update.sh --check`.
10. One `EXECUTION-RECORD.md`, one cost entry, and a PR against `develop`. No merge without the Owner's word.

## 6. Judge campaign and evidence tags (needs the Owner's decision)

Two facts from #165: (a) the Go judge evidence of `develop` is stale (the matrix reports CURRENT 0/13 because the README changed after the campaign), and (b) the gate passes on `develop` only because the Owner authorized the tags `evidence/go-campaign-d182129` and `evidence/go-campaign-1a192c1`, strictly to retain the commits cited by the existing campaigns. This unit changes `ask.go`, so the campaign must be renewed here, which fixes (a); its commit will again be dropped by the squash. The Owner's last authorization covered only the existing campaigns. **Approval of this package asks for a decision**: authorize publishing one more `evidence/*` tag for the commit cited by this unit's renewed report, or accept that the gate fails on `develop` after the squash.

## 7. Known follow-ups carried over (not in this unit)

`afrBuildPanics` calls `ask` without an upper bound; `run-agent.sh` does not escape the worktree root; `TestAskLateReply_Concurrent` drops the `ok` of `dl.Msg.(int)`; the C8 suggestion of #165 (the cancel branch is not required to win under contention); two style suggestions of the #165 delta review.

## 8. Hashes for approval

Listed in the approval request, not repeated here to avoid a self-referential hash.
