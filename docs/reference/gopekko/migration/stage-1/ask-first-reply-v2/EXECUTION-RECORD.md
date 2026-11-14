# EXECUTION RECORD: Ask first reply v2 (#162)

Supersedes the closed #429 attempt (nothing continued from it; its branch and commits are untouched). Package approved by the Owner at `4b7d5a2`; one bounded correction (`gofmt` forms in the settings, `3174c5f`) and a role update (Sonnet writes, Opus architect/TL) authorized afterwards; see `RUNBOOK.md` section 7.

## 1. Sources

Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`: `AskSupport.scala` 614-629 (`PromiseActorRef.!`), `AskSpec.scala:34`, `PromiseRefSpec.scala:54`. Contract: ADR 0015 D1, D4, D8 (AK01-AK04). Reused unchanged from #161: `ask_endpoint.go`.

## 2. Who wrote what

| Artifact | Author | Evidence |
|---|---|---|
| `pekko-actor/actor/ask.go` | **kit implementer** (Sonnet) | 1 run, 82 s, 19 turns; unchanged by the fixer |
| `pekko-actor/actor/ask_firstreply_internal_test.go` | **kit implementer**, then **kit fixer** (Sonnet) for confirmed findings | fixer 36 s, 18 turns |
| `pekko-actor/README.md` (one paragraph) | manual (orchestrating session) | |
| package documents, this record, cost entry | manual preparation | prepared for the kit, not kit output |

Change versus earlier units: in #161 and #429 the runtime was written manually and only one test was ported by the kit. Here the kit wrote the runtime and the full test file. The orchestrator wrote no Go code in the loop; it ran gates and one set of mutations (section 5).

## 3. Roles, models, costs (self-reported by the CLI JSON, not independently verified)

| Role | Model | Tokens (in+out+cache creation) | Cache reads | USD | Seconds | Turns |
|---|---|---|---|---|---|---|
| Implementer | claude-sonnet-5-5 | 93,348 | 343,647 | 0.4799 | 82 | 19 |
| Architect/TL, principal reviewer | claude-opus-5-5 | 32,266 | 156,784 | 0.3585 | 55 | 11 |
| Second reviewer | claude-sonnet-5-5 | 22,435 | 57,360 | 0.1196 | 32 | 6 |
| Fixer | claude-sonnet-5-5 | 28,815 | 121,166 | 0.1640 | 36 | 18 |
| Final delta review (Opus) | claude-opus-5-5 | 25,635 | 146,352 | 0.2950 | 59 | 9 |
| Total | | 202,499 | 825,309 | 1.4170 | 264 | 63 |

Not included: an implementer first launched on `opus` by mistake and killed after about two minutes (cost unknown; its 1.8 KB draft was not reused), and `haiku` permission probes (about US$ 0.02 in total, not loop agents). Agents launched: 6 (limit 6). Tokens: 202,499 under the definition above, which is the one used for earlier units; counting cache reads the total is about 1.03 M, above the 600k limit under that definition. The Owner's limit did not say which definition applies. The USD caps were never reached and no extra charge was incurred (subscription).

Reviewer read-only access is controlled by the prompt and by `shasum` comparison of the targets before and after each reviewer; the settings do not enforce it. All comparisons matched.

## 4. Reviews and findings

| Review | Verdict | Outcome |
|---|---|---|
| Opus architect/TL | FINDINGS | F1 BLOCKER (scenario 2 could miss the second dead letter: `deadLetterBarrier` only waits for publications already started, `deadletter.go:44-45`), F2 unbounded waits (R3.3), F3 no test pinned `build` to the caller's goroutine (ADR 0015 D1), F4 suggestion. All confirmed and fixed |
| Sonnet second reviewer | FINDINGS, no blockers | F1 missing citation on scenario 4 (confirmed, fixed), F3 re-read result in scenario 3 (accepted, fixed), F2 duplication of scenarios 2 and 4 (rejected: style) |
| Third reviewer | not needed | the disagreement on F1 was resolved with the cited source line |
| Opus final delta review | PASS | one informational note (O1: the new build-goroutine test has no bound for a mutant that recovers the panic elsewhere); not fixed, outside R3.3's letter, a plain-goroutine mutant still crashes the test |

F1 was not reproduced (2,000 `-race` runs of scenario 2 passed before the fix); it is confirmed from the documented barrier guarantee.

## 5. Gates (observed, darwin/arm64, this worktree)

| Gate | Result |
|---|---|
| `gofmt -l pekko-actor` | clean |
| `go vet ./...` and `go mod tidy -diff` in `pekko-actor` | clean |
| `golangci-lint` v2.14.0 (pinned version, installed outside the repo) | 0 issues. The system `golangci-lint` 2.13.1 failed with a toolchain export-data typecheck error on `internal/mailbox/queue_test.go`, unrelated to this change; not used |
| `go test -race -count=1 ./...` in `pekko-actor` | pass; `actor` 98.4%, `internal/deadletter` 100%, `internal/mailbox` 100% |
| `go test -race -count=300 -run AskFirstReply ./actor` | pass |
| `scripts/check_coverage.sh` on the `pekko-actor` profile | 98.60% (ok). Other modules were not profiled in this run and are not claimed |
| `scripts/check_deps.sh`, `.github/scripts/check-headers.sh` | exit 0 (15 ported files checked) |
| `scripts/check-adr-signatures.sh` | exit 0 |
| `scripts/check-judge-matrix.py` | exit 0; it reports the CURRENT Go campaign as applicable to this tree: 13/13 designed mutations detected, benign survived 1/1. The checker itself decides input equality; no campaign was renewed |
| Orchestrator mutations on `ask.go` | build on another goroutine, no wait for the reply, build called twice: each makes the slice tests fail; `ask.go` restored byte-identical (sha256 `e4d0879b...`) |

Run afterwards on the same code, before merge: `go test -race -count=1 -covermode=atomic` in all six workspace modules (coverage gate over the six profiles, exit 0: factory 99.54%, pekko-actor 98.60%, tools 100%, three modules skipped for having no executable code), `scripts/check_coverage_selftest.sh` (exit 0), `scripts/modules_selftest.sh` (exit 0), `.github/scripts/go-sdk-update.sh --check` (exit 0), and the remote `ci` and `labeler` runs on `ecf0514` (success). Not run: Linux locally. No Pekko oracle (sbt) was run: this PR does **not** claim Pekko parity; the evidence is ported tests plus Go tests. Tests were not run by the agents in the loop (`go test` denied); the orchestrator ran them.

## 6. Limits of this slice

`ask` is unexported, has no deadline, ignores its context while waiting and does not detect termination or clean up. Open: #163-#166.

## 7. Native reviews

- `cc643cd` (134 lines) and `4b7d5a2` (205 lines against the base, 11 informational findings): approved and acknowledged.
- Final full-range review of the branch against `develop` (head `ecf0514`, 10 files, 666 lines, four lenses, lineage `review-5746957043e94aa2`): approved, no blocking findings, 10 informational ones, acknowledged. It covers the settings correction `3174c5f`, which had no review of its own because its START failed while files were being written (probe evidence for it is in `RUNBOOK.md` section 4), the code `81af5dd` and these records `ecf0514`.

## 8. Approval of the RULEBOOK and effective authorship

- **RULEBOOK approval.** The Owner approved `RULEBOOK.md` (sha256 `f0e8f327541594245bb7f6f835a27a1d45e3375c60031c943d0d1990f2f178d3`) as part of the package at commit `4b7d5a2`, together with the RUNBOOK, manifest, settings template and launcher hashes listed in the approval request. The file keeps its original header ("DRAFT, NOT approved") and its deviation log ("None approved") **on purpose**: the RULEBOOK is read-only in the loop and its hash is the approved artifact, so it was not edited to update its status. This section is the record of the approval. Never rely on the RULEBOOK header for approval state.
- **Authorship change (DEV-1, recorded here, not in the RULEBOOK deviation log).** The Owner approved that the kit implements the runtime (`ask.go`) and ports the tests (`ask_firstreply_internal_test.go`), reusing the integrated endpoint; in #161 and #429 the runtime was written manually. The Owner later authorized the bounded `gofmt` correction of the settings (`3174c5f`, `RUNBOOK.md` section 7) and the role update (Sonnet implements and fixes, Opus acts as architect/TL).
- **Effective authorship** is the table in section 2 and the costs in section 3: `ask.go` and the test file were written by the kit (Sonnet implementer, then Sonnet fixer on confirmed findings); the orchestrating session wrote one README paragraph and the package documents, and ran the gates and mutations.
- **Remote CI.** `ci` and `labeler` ran on `ecf0514` and succeeded. The temporary exception on remote CI does not apply to that run, and it is not claimed as parity evidence.
- **Follow-ups, deliberately not done in this slice:** (1) `afrBuildPanics` calls `ask` without an upper bound (review R3-001, note O1 above); (2) `run-agent.sh` substitutes the worktree root into the settings JSON without escaping (review R3-003). Both go to explicit follow-up work.
