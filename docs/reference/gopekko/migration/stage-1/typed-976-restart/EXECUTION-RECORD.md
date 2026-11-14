# EXECUTION RECORD: Typed restart within its limit (#976, TYP-10)

Scope `docs/scopes/976-typed-restart.md` (PR #1351, amended by #1354, #1356, #1359, #1366; approved at gate G2 by the Owner on 2026-11-05). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`), first base `develop` `bfefdd6`. Delivered in `develop` on 2026-11-05 by the PRs of section 8. All PRs say `Refs #976`, none `Closes`: #976 stays open for the Owner's closure decision (section 11).

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`), one session acting as TL (scope), Dev (slices) and orchestrator of the QA; **50 subagent runs** (counted from the task outputs), all read-only: the clean-context QA reviews and re-reviews of the PRs, each in its own worktree and forbidden to branch, commit or open a PR (the only GitHub write was one review comment), and the three citation-audit reviewers. Tokens and wall clock were not measured (`unknown` in the cost-log row).
2. Inputs: `Supervision.scala`, `SupervisorStrategy.scala`, `ActorAdapter.scala`, `InterceptorImpl.scala`, `Behavior.scala`, `SupervisionSpec.scala` (every one of its 79 tests has a row in `manifest.tsv`), the executed oracles `docs/parity/oracles/s-2-supervision-2026-11-05.md`, `d-a-restart-setup-unbounded-2026-11-04.md` and `d-a-with-children-2026-11-05.md`, ADR 0033 D14/D15, ADR 0035 (S-1 to S-6 and the three amendments of 2026-11-05).
3. Outputs: `supervise.go`, `restart_limit.go`, the restart machinery in `interpreter.go` (loop, children wait, held buffer and replay, limit window, start rules, `giveUp`, `PreRestart` hook, `SupervisorStrategy` of a Typed parent), seven new test files (five of them internal), edits to three existing ones, and `ExampleSupervise`; the kit package (`manifest.tsv` 69 rows, `RULEBOOK.md`, `RUNBOOK.md`); `pekko-actor-typed/README.md`; the amendments of ADR 0035/0036/0037 and five parity documents (citation corrections); the `qa-review` skill change (#1365).
4. Manual changes and deviations: (a) tests use go-specs; internal tests (`package typed`) were needed for the unexported state (supervisor chain, restart window with an injected clock, child wait, `typedActors`), allowed per slice by #1359; (b) `signal_runtime_test.go` TestSig_S6 and `unhandled_internal_test.go` (the exported-surface test) were edited as consequences; (c) 3a-2 carries a hardening of weak assertions of earlier slices (depth proofs, barriers) at the Owner's request.

## 2. RED evidence

Every slice began with its scenarios failing for the right reason before the implementation: slice 1a (the supervised actor never started), 2a/2b (the decider was consulted, `Setup` ran once), 3a-1 (`Setup` reran at once, the child was never stopped), 3b (the child was stopped), 3c-1 (`setup:2` where `prerestart:run1` was expected), 3c-2 (the third `boom` stopped the actor after 3 runs instead of 9 and 6). After each QA round, **mutations of the production code** (not only tests) were run to show that each claimed property fails a test; recorded for #1363, #1364, #1367, #1368 and #1370 in their QA comments; the limit window (#1357) and the start rules (#1360) were reviewed by tracing the code against `Supervision.scala` and no mutation is recorded for them.

## 3. Gates

Per PR, on the pushed head, with the commands of `AGENTS.md` (remote CI runs `ci-fast` only): `gofmt -l`, `go vet`, `go mod tidy -diff`, `go test -covermode=atomic` per module, `golangci-lint` v2.14.0 (gocyclo max 10), `scripts/check_deps.sh`, `diff-cover` 11.0.0 against the base (85%). All 7 exit 0 for every code PR; the full output is in the evidence comment bound to the head SHA (section 8). `check-headers.sh` reports 7 errors in `pekko-persistence*` that predate this work and none in `pekko-actor-typed`. `unshare -rn` is not available on macOS; the unit tests use no network.

## 4. Size (authored lines, additions plus deletions)

See the measured table in the scope ("Measured sizes") and section 8. Every PR is at most 400; the two at the edge are 1a (400) and 3c-2 (391).

## 5. Evidence class and pending approval (ADR 0033 D13)

- **Executed on Pekko 1.7.1 (oracle):** the run counts of S-3 (b) for n = 0, 1, 2, 3 (L, P4a, scenario 3), P4b (9 runs, 2 consultations), P4c (6, 1), P1 (two children after a restart by the Classic parent), C1 (an asynchronous loop with at most one live child).
- **Read against the spec, not run:** R1, R2, R4, R5, R6, R17, R18 (`SupervisionSpec.scala` 512, 214, 224, 239, 261, 528, 550, 575, 662). R3 is the default Stop of #956 and R15 has no spec of its own. **Pending Owner approval as alternative evidence under D13.**
- **No Pekko counterpart:** R10 (100 000 failed `Setup`s in a loop: guards a Go-specific deviation, Pekko recurses), the `typedActors` table (Pekko compares the actor class), the Goexit Stop (Go adaptation).
- **Not claimed:** parity of the module with Pekko Typed, V3, the heap behavior of C1, the oracles' run data beyond the cited code.

## 6. Not delivered (kept out), with owners

c-sync fatal path (#1345, known gap: the loop is iterative and has no fatal path); the bounded restart stash and `Dropped` (#999, `TODO(TYP-26)`); `Resume` (#1183); backoff (#986); the stop-during-wait boundary (#997); innermost-match tests and the equal-matcher rule's own tests (#1184); a `Supervise` capturing the `DeathPactException` of an unhandled `Terminated` (#1379, ADR 0035 contract 10); `ChildFailed` and the death pact of a Typed parent (#1346; #1250 was closed as not planned in favor of it); explicit `Stop()` and `Stopped` (#1182); a Typed spawn and watch (#1170, #1171); timers cancelled on restart (contract 15) and failure logging (contract 13, #1179). `WithStopChildren` moved here from #997 by the Owner.

## 7. QA and integration

Each PR was reviewed from a clean context (a fresh worktree whose `upstream/pekko` submodule is initialized, reading the evidence comment in full and checking its SHA against the head). What the reviews found, in order of weight:
- **A CRITICAL found by the author, not by a review** (#1368). The clean-context review of head `22375ca` approved the PR (one review, one APPROVE comment in the thread). Before merging, the Owner asked in the session (not visible in the PR thread) to verify what happens when the parent is a **Typed** actor. The author ran that case and found that a child of a Typed parent whose limit was exceeded after its start was **restarted** by the runtime's default strategy, because the rethrown cause now reached the runtime and the interpreter declared no strategy. The review had not tested that case: its brief covered the Classic parent only. The merge was stopped, the Owner chose to add M-A-lite (the stop part of mechanism M-A of ADR 0035), the test that exposed the defect became the test that pins the fix, and two further reviews of the new heads (`1ce614c`, `7ea3c53`, both APPROVE) found weaknesses of that fix; the ones that were not closed before the merge (W-1: Stop causes of a killed or death-pact Classic child; S-1: wording; S-2: a trivially true assertion) were carried to slice 4, and W-1 landed in #1370. The lesson is in section 12.
- **REJECT** on #1352: the PR thread shows one REJECT (comment 6663513981, head `d151d74`: the RUNBOOK listed eleven PRs while the scope on `develop` had seven, the dedupe owner contradicted the scope, and the manifest lacked rows for tests of `SupervisionSpec`) and one APPROVE of its own (6663716932, head `ec642d4`). Comment 6663591864 is also an APPROVE on that thread, but it reviews head `e412bb3` of #1354 (its twin is 6663490348 on #1354): a review posted on the wrong thread. The reviewer of the intermediate head `4b523b3` reported a REJECT (more missing `SupervisionSpec` rows) in its hand-back to the author but its comment is **not** in the thread: the id it reported, 6663591864, is the misposted #1354 APPROVE described above. The fix of the missing rows is now a completeness check in the RUNBOOK. REJECT once on #1364 (the reviewer read a rendered summary and missed the evidence: the skill now says to fetch the comment with `gh api`).
- **Assertions that passed for the wrong reason**, found by mutation: a fixed child name made the live-children maximum prove nothing (#1363); `R10` could not tell a loop from a recursion (now a stack-depth proof); several mutations of the M-A-lite decider survived the first review (#1368). Each was fixed with a test that fails under the mutation.
- Two wrong statements of the author in review: a false claim about `--git-common-dir` in a linked worktree (#1365) and a wrong "recovery state" label in a citation correction (#1369).
- The 20 PRs #1351 to #1370 were squash-merged after APPROVE with green CI (this PR is the 21st; #1372 and #1373 follow), under the Owner's standing rule; no PR was merged with an open CRITICAL. Where a head changed after its approval, only comments, tests or wording written to answer that review changed, and the PR says so.

## 8. Candidate versus integrated commit, PRs, size and evidence

| PR | What | Merge commit | Lines | Evidence comment (`gh api repos/getsyntegrity/gopekko/issues/comments/<id>`) |
|---|---|---|---|---|
| #1351, #1354, #1356, #1359, #1366 | scope, amendments | `6f602340`, `620e57e2`, `ce615717`, `d6ef1ba5`, `003e495f` | docs | n/a |
| #1352 | kit package | `00dad56c` | 144 | n/a (docs) |
| #1353 / #1355 | slice 1a / 1b | `576378c0` / `0799db05` | 400 / 155 | 6663320675 / 6663754192 (the merged head `4824fc3` was rebased onto `develop`, tree-identical to the reviewed head `1be716c`, whose own evidence is 6663449258) |
| #1357 | 2a, `WithLimit` | `f53aadd5` | 224 | 6664046713 |
| #1358 | `Matcher.equal` panic fix | `57448ad9` | 31 | 6663989696 |
| #1360 | 2b, first `Setup` under a limit | `a5f6b066` | 174 | 6664238253 |
| #1361 | follow-up tests, doc nits | `44313c52` | 22 | 6664532121 |
| #1362 | 3a-1, children wait | `55359181` | 279 | 6664697898 |
| #1363 | 3a-2, C1 and hardening | `d4417b78` | 180 | 6665117552 |
| #1364 | 3b, `WithStopChildren` | `afb6cb27` | 249 | 6665631146 |
| #1365 | `qa-review` skill | `8dd32cb6` | 4 | n/a (docs) |
| #1367 | 3c-1, `PreRestart` hook | `ecf8d2a0` | 158 | 6666016352 |
| #1368 | 3c-2, rethrow, M-A-lite, W1 | `3883ffe4` | 391 | 6667229718 |
| #1369 | citation audit | `ab761c3e` | 47 | the record in comments 6667768776, 6667769384 |
| #1370 | W-1 test, `ExampleSupervise` | `49298059` | 112 | 6668154104 (+ 6668312321) |
| this PR | README, record, cost-log row | see `git log` | docs | n/a |

Several PRs changed after their first approval to answer a review; the evidence comment above is the one bound to the merged head, and the earlier ones are superseded in the PR thread.

## 9. Verification by head

Executed on the final tree of `develop` at the close of slice 4: `go test -count=2 ./...` in `pekko-actor-typed` (ok), `golangci-lint` (0 issues), `scripts/check_deps.sh` (ok). The module has 20 test files; what was executed on Pekko is only the three oracles of section 5.

## 10. Criteria against the scope

| Criterion (issue "Update" sections) | Result |
|---|---|
| Restart reruns the supervised setup; `PreRestart` to the old behavior only | delivered (1a, 3c-1) |
| Failure matching and retry-window limits with Typed semantics | delivered (1a, 2a, 2b), with the S-2.6 landing of a rethrow (3c-2) |
| Unmatched failures propagate to the outer strategy, then the default Stop | delivered (1a, 1b) |
| S-3 (a), (b) for n = 0 to 3, c-async (C1), iterative restart | delivered (2b, 3a-1, 3a-2, 1b); c-sync fatal path **not** (#1345) |
| P4a, P4b, P4c, initial-start n = 0 to 3 | delivered, matching the oracle counts (2b, 3c-2) |
| `Supervise` / `OnFailure` / `Restart` / `Restart.WithLimit` / `WithStopChildren` | delivered; the Go names are the ones of the scope |
| Restart stash bounded | **not** (#999): unbounded, stated |
| Update the public contract and traceability | this PR (README, record); the scope text is #1373 |

## 11. Proposed closure conclusion (a proposal, not a decision)

Every criterion of #976 is delivered except the two stated gaps (c-sync fatal path, #1345; bounded restart stash, #999), which the issue itself excluded. Proposal: the Owner reviews this record and decides whether to close #976 with those two gaps as the follow-ups. Nothing here closes it.

## 12. Recurring failures and proposed amendments (`AGENTS.md`, kit rule 6)

The same failure appeared at least three times: **a new assertion that passes for the wrong reason, found only after a review had approved** (#1363: a fixed child name; #1364: ordering that let a mutation survive; #1368: three surviving mutations of the new decider). The rule that produced it: a review approves a test because its assertions look right, not because removing the behavior makes it fail. Proposed amendment, for the Owner to accept or reject (it touches `qa-review` and `dev-scope`, so it is not applied here): *a PR that adds a test for a behavior names the mutation of the production code that makes it fail, and the reviewer reproduces it.* A second lesson from the same chain: **a review brief must list the cross-actor cases of the change** (here the kind of parent). The review that approved `22375ca` was right about what it was asked and silent about the Typed parent; the case was found because the Owner asked for it. Reviews of later heads were told to test both parents separately and to try to make a signal reach the parent. Smaller, already applied: the manifest completeness check (RUNBOOK), the submodule initialization and the full evidence comment (`qa-review` skill, #1365), and the estimate ratio (scope, "Measured sizes").
