# EXECUTION RECORD: Supervision U1, the Failed channel, suspension and OneForOne Stop/Resume/Escalate (#168, #446)

**EXECUTED on 2026-10-14** (loop from 20:25Z; the Go campaign and the evidence tag followed). The implementation was natively reviewed in sections (section 7); the generated campaign report was not. Nothing is merged.

## 1. Approval

| Item | Value |
|---|---|
| Base `develop` commit | `9063651ee6817a433307f9b8a3d1ad4ad103bcc5` (squash of #453, ADR 0017 accepted) |
| Package commits | `1643a0f` (package, natively reviewed: four lenses, approved) and `c3c6ad9` (fixes of that review, natively reviewed: four lenses, approved) |
| Approved sha256 | RULEBOOK `fdbd5b0d5452018e49f05ae197d6cb882efc843eb34641d058da76241466dba9`; RUNBOOK `81ed69b4308d8dd26e5784370c1021f2260c779cf3855fec07de98c51fc5e38f`; manifest `4fd2d88047734396e282ca1670bc5856578f2f591fd526dbdc1190f475cd8831`; settings template `670a9c61e738af1a4d5b007e083bb75ad66dee629b5f713e5e78bcab33a51caf`; launcher `ab519c1aeb2e70f181e8faf493b1d5cc553b71e37a48abb62dad7548f358fbbb` |
| Owner decisions D1-D12 | Approved with precisions (binding for every agent, the RULEBOOK stays unedited): D2, D5 and D6 are transitional behaviors of U1, not the final contract and not Classic parity (Restart pending U2, the full default strategy pending U3); the names `ErrInvalidDirective`, `CauseDirective`, `MakeDecider`, `DeciderFunc` are approved; D7 keeps its upstream equivalence Unknown; D11: `ContextMisuse` does not exist, so the case is recorded as `PORT-GAP(context-expiry)` and no expiry capability is added in U1. The text "errors.As" in RULEBOOK D8 is void: `MakeDecider` matches with `errors.Is` only (a `CauseDirective` cannot carry an `errors.As` target; found by the delta review, R2-004) |
| Owner decisions: model plan and limits; judge renewal and `evidence/*` tag | Approved: Sonnet implements and fixes, Opus designs and reviews, with the limits of the package (RUNBOOK section 3); renewal of the Go campaign for this functional change and publication of the `evidence/*` tag of the exact commit, verified against its manifest |

## 2. Kit and sources

| Item | Value |
|---|---|
| Migration kit | `migration-kit/` submodule, commit `cf91c9d5068d9aaf95a36164169f08c3e636c909` (confirmed with `git submodule status`) |
| Pekko | v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` |
| ADR | `docs/adr/0017-supervision.md`, unit U1 |
| Upstream tests ported | `SupervisorHierarchySpec.scala` (815, 832), `SupervisorMiscSpec.scala` (165), adapted as V1-V3 |

## 3. Commands

| # | Command | Result |
|---|---|---|
| 1 | baseline: `git rev-parse HEAD:pekko-actor/actor` | tree `96093f1b8bf06e16b6ac3c5eb36cb0f6945e8b8c`, worktree clean |
| 2 | `run-agent.sh haiku probe-prompt.md probe.json <rulebook hash file> <settings hash file>` | six new files writable; `watch.go`, `spawn.go`, `clock.go` and `judge/` refused; `go test`, `rm`, `curl`, `git push` denied; the Edit of an existing test file (B4) was not applied because the pattern was ambiguous, not by permission. The orchestrator removed the six probe files; the runtime tree equals the baseline again |
| 3 | `run-agent.sh opus design-prompt.md design.json ...` | NO BLOCKERS, design and 15 guidance items |
| 4 | `run-agent.sh sonnet implement-prompt.md implement.json ...` | IMPLEMENTATION DONE; `gofmt`, `go vet`, `go build` clean; no test run (denied) |
| 5 | `go test -race -count=1 -run Sup1 ./actor/` (orchestrator) | one failure, V14 zero strategy (see section 6) |
| 6 | `go test -race -count=20 ./actor/` after the fixes | pass; the existing suite passes unchanged |
| 7 | `run-agent.sh opus review-prompt.md review-opus.json ...` and `... sonnet ... review-sonnet.json ...` | Opus: 0 blockers, 1 major, 4 minor. Sonnet: 0 blockers, 1 major, 7 minor |
| 8 | isolated mutations on a copy (`GOWORK=off`), 21 mutants | 20 detected, 1 survived (M21). M21 was then closed by the deterministic test V5c (row 12): 21 of 21 |
| 9 | `golangci-lint` v2.14.0 (installed under a temp `GOBIN`) | 3 findings before the fixes (ST1023, ST1012, unused helper), 0 after |
| 10 | `judge/mutate_go.py <workdir> --runs 30 --report judge/go-mutation-report.tsv` | 14 of 14 mutations met, `unmet: none` |
| 11 | `scripts/check-judge-matrix.py` | HISTORICAL 13/13, CURRENT 13/13 on darwin/arm64 (it was 0/13 before this campaign) |
| 12 | V5c added to `supervision_goside_internal_test.go`; the same campaign repeated (`mutate_go.py <workdir 2> --runs 30 --report ...`) because a new test file changes the `pekko-actor` tree that CURRENT evidence is bound to | V5c passes on the code and fails under M21 with `expected 1 to equal 0`; campaign 14 of 14 met again; CURRENT 13/13 restored (it had dropped to 0/13 after the test commit) |
| 13 | native review by sections, each with the provider-issued commands and the Owner's explicit consent per envelope (`goPekko-evidence/u1-native-*`) | see section 7 |
| 14 | V9 and V6 checked with concrete mutations, then corrected; V2 corrected; the parity row corrected | see section 6, items 9 to 11 |
| 15 | third Go campaign (`mutate_go.py <workdir 3> --runs 30 --report ...`) on the final committed inputs | 14 of 14 met; CURRENT 13/13 |

## 4. Roles, models and costs (self-reported by the CLI JSON)

Tokens are input + output + cache creation per agent JSON, the Haiku probe included; cache reads are reported apart.

| Role | Model | Tokens | Cache reads | USD | Seconds | Turns |
|---|---|---|---|---|---|---|
| Permission probe | claude-haiku-5-5 | 46,569 | 45,427 | 0.0109 | 21.2 | 22 |
| Design review | claude-opus-5-5 | 101,512 | 180,947 | 1.1258 | 235.4 | 18 |
| Implementer | claude-sonnet-5-5 | 290,544 | 1,933,980 | 1.7820 | 529.0 | 50 |
| Final review | claude-opus-5-5 | 99,914 | 464,516 | 1.2213 | 281.7 | 17 |
| Second reviewer | claude-sonnet-5-5 | 86,207 | 241,655 | 0.5156 | 201.8 | 13 |
| Fixer and fix-delta review | not run | | | | | |
| **Total** | | **624,746** | 2,866,525 | **4.6556** | | |

**The 600,000-token limit was exceeded by 24,746 tokens (4.1%).** The cause was an error of forecasting by the orchestrator: the two reviewers were launched in parallel when the total was already about 439k, which projected past the limit. The Opus review in flight was allowed to finish because it is the principal review of the unit; the fixer and the Opus fix-delta review were NOT launched, and every fix was made by the orchestrator (section 6). No run failed, so no spend was lost to a failed launcher run. The per-agent USD cap (6) and the wall clock (60 min per agent) were not reached by any agent. The orchestrator-side verification workers (mutations, documentation) are not part of the loop and are not counted here.

## 5. Inputs and outputs

| Artifact | Author (kit or manual) | Commit |
|---|---|---|
| `supervision.go`, `supervision_failure.go`, `supervision_decider.go`, edits to `cell.go`, `errors.go`, `hierarchy.go` | kit (Sonnet implementer); manual fixes listed in section 6 | `df7cf4f`, then `93b140c` |
| `supervision_resume_internal_test.go`, `supervision_decider_child_internal_test.go`, `supervision_goside_internal_test.go` | kit (Sonnet implementer); manual fixes listed in section 6 | `ab5890b`, then `764ed50` (V5c), `9d97e43` (V9, V6) and `f8093a4` (V2) |
| `README.md`, `pekko-actor/README.md`, `docs/actor-core-parity.md`, `pekko-actor/actor/doc.go` | manual (orchestrator, through one documentation writer) | `0b767b3` |
| `judge/go-mutation-report.tsv` | tool (`judge/mutate_go.py`) | `0597f39` |

`stop.go`, `mailbox.go` and `actor.go` were not touched. Final `pekko-actor` tree: `44ac7ccf42cfdbd7070a2cd5e0d2cc2eff6d241e`.

## 6. Manual changes

Every orchestrator edit to code, tests or documents, with its reason:

1. `V14` zero-strategy test: the assertion `suspended == 0` was made conditional (only for a parent that declares nothing). A parent that returns a zero `Strategy` is only discovered in the parent's drain (`SupervisorStrategy()` is user code and must not run on the child's goroutine), so the stopped child keeps a count of 1; the RULEBOOK (V14) specifies the event sequence, which both cases still assert. The `Supervisor` doc comment states the same.
2. Review finding Opus F1 (MAJOR): a concurrent `Spawn` could unbalance a counter. Fixed with `adjust` (counter, node mirror and children snapshot in one critical section, lock order `System.mu` then `cell.mu`) and `inherit` (a child committed under an owner with pending suspensions starts with that count); `hierarchy.go` gained `node.susp` and `childRecordsLocked`. New test `V5b` (`TestSup1SpawnDuringSuspension`), verified to fail when the inheritance is disabled.
3. Review finding Sonnet F1 (MAJOR): `V4` could not fail when Resume touched a sibling. Fixed with a sibling suspended from another source whose count must survive the Resume.
4. Review findings (minor): `V10` now holds the grandparent's gate and asserts the counters `[0 1 1]` before it decides; the `D7` test was renamed to what it shows; `PanicError` message is neutral ("actor: panicked"); two comments corrected.
5. Lint: `cell.go` (`cause := returned`), the sentinel error renamed `errSup1Boom`, the unused helper `sup1Run` removed, `hasWorkLocked` kept for the wake-up after a resume.
6. The end of `drain` moved to `endActivation` with the same predicate, keeping the text `if c.stopRequested || c.depthLocked() > 0 || len(c.lane) > 0 {` verbatim: the Go judge's mutation `GMstop` anchors on it, and `judge/` must not change. Found because `mutate_go.py` failed to apply the anchor (`cell.go: anchor occurs 0 times`).
7. A stray directory `--help/` was created in the worktree by an invocation of `mutate_go.py --help` (the tool treats the first argument as a work directory); it was removed before any commit.
8. Mutant M21 survived the first 21-mutant run. The verification worker that was still running built a deterministic closing test with an executor that runs inline only while a flag is set, so the parent decides Resume inside the child's own `enqueueFailed` call. The orchestrator re-read it, merged it into `supervision_goside_internal_test.go` (the RULEBOOK allows exactly three test files) as V5c (the worker had called it V15, an id the RULEBOOK uses for initialization), and checked it itself: it passes on the code and fails under M21. This corrects an earlier statement of this record and of the PR that only a probabilistic stress test could reach M21.
9. V9 and V6 were checked before being changed, as the Owner asked. V9 was confirmed vacuous: under a mutation that makes `drain` ignore the suspension, 194 of 200 runs passed, because the received counter was read without a synchronization point. It was rewritten with the controlled executor and detects the mutation in 50 of 50 runs. V6 was not a flake (the mutation that drops the watcher notification failed 8 of 8) but nothing ordered the watcher's registration before the child failed, so a `registered` signal was added; `Watch` stores the registration and links the watchee before it returns (`watch.go`), so the signal is accurate (commit `9d97e43`).
10. V2 had the same weakness as V9 (185 of 200 runs passed under the same mutation). It was rewritten with the controlled executor: one run on the code passes and one run under the mutation fails (`expected 0 to equal 1`) (commit `f8093a4`).
11. The parity row for the U1 `Receive` failure called all three supervision test files "ported" (native review 3a). `docs/actor-core-parity.md` now separates three upstream cases that are adapted (V1, V2, V3) from the Go extensions in `supervision_goside_internal_test.go` (commit `a865e73`).

## 7. Reviews, mutations, gates, judge evidence, limits and follow-ups

**Reviews of the kit loop.** Opus final review: 0 blockers, 1 major (fixed), 4 minor (2 fixed, 2 noted). Sonnet second review: 0 blockers, 1 major (fixed), 7 minor (4 fixed; F6 dead branch and `failed` retention, F7 a test for a parent that stops with a Failed queued, F8 the `time.After` in `sup1Recv` are follow-ups). No fix-delta review by Opus was run (token limit); the fixes were verified by the tests and the mutations below.

**Mutations (21, on the final code).** First run: 20 detected, 1 survived, M21 (`enqueueFailed` before the self-suspension in `failUp`). It was first reported as reachable only by a probabilistic stress test; that was wrong. A deterministic test, V5c, closes it (manual change 8), verified by the orchestrator: pass on the code, fail under M21. Result: 21 of 21 detected. The worker's full 21-mutant rerun on the final snapshot also reported 21 of 21; Invalid attempts over the whole history, all resolved by a rerun: M2 and M6 (the full suite timed out because the leftover suspension count hung a later test; the bounded rerun of the detecting test fails normally) and M10 (the first attempt, on the earlier snapshot, did not compile: `declared and not used: raw`; the rerun with `_ = raw` is detected). The final state is 21 of 21 detected with no survivor, and the worker also re-ran M21 against the delivery's own V5c to confirm it. The V5c delta itself has no Opus review and no native review.

**Gates run by the orchestrator.** `go test -race -count=20 ./actor/`: pass. Coverage of `pekko-actor` on the final code: 98.3% of statements (the minimum is 85%). `go vet`, `gofmt -l`, `go mod tidy -diff`, `golangci-lint` v2.14.0 (0 issues), `.github/scripts/check-headers.sh`, `scripts/check_deps.sh`, `scripts/check-adr-signatures.sh`, `go-sdk-update.sh --check`, `scripts/modules_selftest.sh`, `scripts/check-judge-matrix.py`: pass. `gocyclo` is not installed; the pinned `golangci-lint` run passed. The repository test suite of the other modules was not re-run (they are not touched).

**Judge evidence.** Three Go campaigns were run (`--runs 30`, 14 of 14 mutations met each). The THIRD and last is the one the committed report records: `go_commit` `a865e73db171aeead6cc79432205c1a92ed9d51e`, `pekko-actor` tree `44ac7ccf42cfdbd7070a2cd5e0d2cc2eff6d241e` (equal to the tree of the final HEAD), report sha256 `a8c2b9d9a86b931f3a8763b6b195368b00ab79c84ab2a70320d8b41874a894f9` (equal to the manifest); tag `evidence/go-campaign-a865e73` published on that commit; `check-judge-matrix.py` reports CURRENT 13/13. It was needed because V9, V6, V2 and the parity row changed the tree after the second campaign. The two earlier tags, `evidence/go-campaign-93b140c` and `evidence/go-campaign-764ed50`, are HISTORICAL: each matches its own manifest and does not cover the final tree; both are kept, not moved or deleted. Detail of the first two campaigns: The FIRST, before V5c: `go_commit` `93b140c9bbb3ef0cc146b0a3f96487caa277a418`, tree `d64391c3f45068774af76640f3158f3c31ba60ba`; tag `evidence/go-campaign-93b140c`. It became stale when V5c changed the `pekko-actor` tree (CURRENT fell to 0/13), so it was repeated. The FINAL, the one the committed report records: `go_commit` `764ed5090fd840ddc201a501fe213792c76f629b`, `pekko-actor` tree `6f5a51962803dc369852a9548c31b6e887eb7e0a` (equal to the tree of the final HEAD), report sha256 `8792c7103464d18f4448108dbf1d95ca62cad122449ae652d91027cbc85840fb` (equal to the manifest); tag `evidence/go-campaign-764ed50` published on that commit. The first tag was left in place (nothing was deleted from the remote); the Owner decides whether to keep it. `check-judge-matrix.py` now reports goPekko CURRENT 13/13 (it was 0/13). The judge has **no U1 scenario**: S13, S14 and S16 stay out of the Go gate and no Pekko parity is claimed beyond the three adapted upstream cases.

**Native review.** The package commits were natively reviewed (four lenses, approved). The implementation was reviewed in sections, each with the Owner's consent for its envelope, a single reliability lens each, and all approved and acknowledged with no blocker: S1 runtime and API `c3c6ad9..df7cf4f`; S2 tests `df7cf4f..ab5890b`; 3a documentation `ab5890b..0b767b3`; 3b `endActivation` `0b767b3..93b140c`; 3d V5c and this record `0597f39..764ed50`; the last delta `4c302cd..a865e73` (V9, V6, V2, parity row). The range 3f `78514c2..4c302cd` was closed by the engine as passive and low risk. The first section 3 (`ab5890b..4c302cd`) exceeded the engine's context budget and was not retried; it was split by the existing commits. **Not natively reviewed:** 3e (`764ed50..78514c2`) and the final report commit, both generated reports validated by `check-judge-matrix.py` and by their hash against the campaign manifest; `4c302cd..9d97e43` was left unanswered on purpose because it was superseded by the consolidated delta; the whole-branch range was never reviewed. A bounded Opus review of V5c (22,362 tokens, 2,362 over the 20k ceiling that the Owner accepted afterwards) found 0 blockers, 0 major and 2 minor. Native warnings and suggestions were not all fixed; they are listed below.

**Follow-ups.** Restart and budgets (U2); the default strategy and `ErrGoexit` (U3); creation failures (U4); guardians (U5); `AllForOne` (U6); stop during restart (U7); judge S13/S14/S16 (U8); logging (U9); the `Failed` to dead letters question (D7); `ContextMisuse` expiry (D11); user-defined strategies and the capabilities of #450; the minor review follow-ups above; the open native findings: S1 R3-escalate-detail (the Escalate payload is the cause, not the raw panic value, and no test pins it), the duplicated end-of-activation predicate (kept because the judge mutation `GMstop` anchors on that text), V5c does not assert that the parent decided inside `enqueueFailed`, and three suggestions of the last review (the exact queue-length checks in V9 and V2 couple them to the scheduler, and the V6 registration signal); the launcher hardening noted by the delta review (a failed run deletes its JSON, the wall bound is per agent, `timeout -k`).
