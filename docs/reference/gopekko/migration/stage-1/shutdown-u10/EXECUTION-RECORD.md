# Execution record: ordered minimal termination of the System, U10 (design #192, implementation #193)

Status: code in a draft PR; one focused Opus review done (no BLOCKER, four DEFECTs, all test-quality, fixed by the coordinator); #192 and #193 stay open; no parity claim. Package: PR #478 (squash `1c4cbaa`, approved head `8dea3e5`), all nine hashes matched at execution. Branch `feat/actor-shutdown-u10`. Kit `cf91c9d`, upstream Pekko `9a54d1294c` (v1.7.1). "U10" is this package's own label.

## 1. What was delivered
`(*System).Terminate()`, `WhenTerminated() <-chan struct{}`, `Shutdown(ctx) error` and `ErrSystemStopping` (text `actor: system is stopping`, is-a `ErrParentStopping`). `Terminate` is `Stop(/user)`; `WhenTerminated` is the root guardian's `done`; `Shutdown` calls `Terminate`, returns nil if the channel is closed (also with a cancelled context), else `ctx.Err()` unwrapped. A top-level `Spawn` refused by the closed gate of `/user` returns `ErrSystemStopping`. No helper goroutine in the three functions, no new `System` field, no change to the guardians, `stop.go` or the executor. Commits: `41c633d` (production and tests, 868 lines, above the ~700 planned; not rewritten, by the Owner's instruction) and `2279fcc` (test corrections after the review).

## 2. Results
| Gate | Result |
|---|---|
| `gofmt -l`, `go vet`, `go mod tidy -diff` | clean |
| pinned `golangci-lint` v2.14.0 | 0 issues (after the two fixes below) |
| `go test -race -count=1 ./...` | pass; coverage 98.1% of statements |
| `Shutdown` tests 30 times under `-race` | pass |
| `check_deps.sh`, `check-headers.sh`, `check-adr-signatures.sh`, `modules_selftest.sh`, `go-sdk-update.sh --check` | pass |
| existing tests | pass without edits (no existing test file changed) |
| judge self-tests, `run-go.sh`, `mutate_go.py` | **not run**, by the Owner's decision |

Fixes by the coordinator, mechanical or in tests only: SH2 counted six actors for a five-actor tree (a 30 s timeout); three test functions had cyclomatic complexity above 10 and were split with helpers; `gofmt -w` on the new test file. No production code was changed by the coordinator.

## 3. Mutants (RULEBOOK 8 plus two variants added; copies outside the repository, `GOWORK=off`)
Final pass on the final tests, package timeout 240 s, unmodified copy passes.
| Mutant | Result | Killed by |
|---|---|---|
| MS1a/MS1b `Terminate` stops root or `/system` | killed | SH1, SH3, SH4, SH6, SH8, SH10, SH11, API, SH5 |
| MS2 `WhenTerminated` returns `/user`'s `done` | killed | SH1, SH4, SH10, **the new stepwise test** (alone), SH3 |
| MS3a `Shutdown` ignores `ctx` | **INVALID** (hangs a test: package timeout), see MS3a2 | — |
| MS3a2 same, with a bounded wait | killed | SH6, SH7 |
| MS3c `Shutdown` returns `DeadlineExceeded` instead of `ctx.Err()` | killed | SH6 |
| MS3d `ctx.Err()` checked first (priority inverted; added after the review) | killed | SH6, the inline-executor Shutdown test |
| MS4 `Shutdown` does not call `Terminate` | **INVALID** (hangs a test), see MS4b | — |
| MS4b same, with a bounded wait | killed | API, SH6, SH9 |
| MS5 top-level `Spawn` keeps `ErrParentStopping` | killed | SH5, API |
| MS6 `ErrSystemStopping` does not wrap | killed | SH5, API |
| MS7 `WhenTerminated` new channel per call | killed | SH3, SH6, API |
| MS8 `Shutdown` returns nil when `ctx` ends first | killed | SH6, SH7 |
| MS9 no priority of finalization | killed | SH6 |
**INVALID runs, listed apart from detections:** MS3a and MS4 each hang a test until the package timeout; per the rules a timeout is not a detection, and each was redesigned once (MS3a2, MS4b), which die with a named test. In the first pass (package timeout 900 s) MS3a looked "killed" only because the coordinator stopped the hung process by hand; that result is discarded. **Survivors: none** among valid runs. One finding: MS1b alone does **not** die against the new stepwise test (it is killed by other tests); the stepwise test is the witness of MS2 only. **Declared gap:** a helper goroutine started by the observation calls cannot be killed by a deterministic test (RULEBOOK 8, MS7); it is covered by readback and review.

## 4. The one review (Opus, read-only)
Launched after the forecast gate: implementer measured 152,151 tokens (at most 250,000), and `152,151 + 125,000 reserve + 110,000 = 387,151` is under 450,000. It used 85,509 tokens, USD 0.84. Findings: **no BLOCKER; production code holds**. DEFECT 1: MS2 was caught only by a race (SH2 could not see a root that closed too early) fixed with `TestShutdownStepwiseNeverClosesEarly`, which steps the controlled executor and asserts at every step that the channel is closed only if the root finished and the root only after `/user`, `/system` and the tops; it kills MS2 alone. DEFECT 2: the root's completion position may be absent because the completion seam records signals to an owner and the root has none; the comment says so and R1 for the root is now asserted stepwise. DEFECT 3: SH2's "nothing runs after the close" re-read the journal twice back to back; replaced by the stepwise test (no hook runs when the rest of the queue is drained after the close). DEFECT 4: SH11 did not call `Shutdown`; a test under the inline executor now shows a held `PostStop` holds `Shutdown` before its wait, with a cancelled context, and that it returns nil once released. INFO 1 (SH4 prefix lengths fixed at 0 to 5), INFO 2 (SH7 uses an expired context) and INFO 3 (the `Terminate` doc states the full order without the qualifier of I1) are recorded and **not acted on**. It could not read `guardian.go`, `stop.go`, `ctlExec` and the other helpers, so those are **unreviewed** by it. The fixes were not re-reviewed.

## 5. Denied commands and their differences from the permissions (contrasted, causes not asserted beyond the text)
Implementer (6 denials recorded in its JSON; its allow list had `Bash(gofmt -l pekko-actor/actor)`, `Bash(go vet ./pekko-actor/...)`, `Bash(go build ./pekko-actor/...)` and `Bash(bash migration/stage-1/shutdown-u10/run-shutdown-tests.sh)`):
- `cd <worktree>; cat run-shutdown-tests.sh; cd pekko-actor/actor; cat errors.go stop.go spawn.go`: a compound command with `cd` and `cat`, neither in the allow list.
- a `python3` heredoc rewriting the test file: not in the allow list.
- `gofmt -l .; go vet ./... 2>&1 | tail -30; go build ./... && echo built`: a compound command, run after `cd`, with `.`, `./...` and pipes instead of the exact allowed strings.
- `gofmt -l /abs/path/.../pekko-actor/actor`: an absolute path, where the allowed string is `gofmt -l pekko-actor/actor`.
- `go vet ./...` and `go build ./...`: `./...` instead of `./pekko-actor/...`.
In every recorded case the command text differs from the allowed strings. The implementer's report says that `gofmt`, `go vet`, `go build` "and the wrapper script" were denied; **the wrapper command does not appear among the six recorded denials**, so that part is not contrasted here. The implementer did not run anything and reported `NEEDS-DECISION`. Whether the allowed strings are matched literally, and why a bare `cd` form failed, is **not verified here**; this is a package defect candidate for the next package (state the working directory and give the exact commands, or allow them with `:*`), not a conclusion.
Reviewer (1 denial): `git show --stat ...; git show --stat ...; cd pekko-actor && grep -n ...` a compound with `cd` and `grep`; the review template allows `git show:*` and `git diff:*` but not `cd` or `grep`. A plain `git show --stat` worked afterwards, as it reported.

## 6. Budget (convention, not a measurement)
Agents measured (input + output + cache creation): implementer 152,151 (one run), review 85,509; **237,660**. Reserve used by the coordinator: estimated 70,000 to 90,000. Estimated total about 310,000 to 330,000, under the evaluation stop (450,000) and the ceiling (560,000). The estimate is an estimate.

## 7. The judge and the matrix (not run, recorded honestly)
By the Owner's decision no campaign, `run-go.sh` or judge self-test was run. `scripts/check-judge-matrix.py` was run once, read-only, on the final tree: **exit 0**, but its line `goPekko CURRENT` reads `detected 0/17; benign survived 0/1; none: an input of the campaign changed since it ran (the Go core, ...); run it again before citing these detections as current`. So **the Go CURRENT line no longer applies to this tree** (the campaign report describes the tree before U10). The HISTORICAL line and the Pekko side are unchanged (432 rows, 17/17 detected at the report's tree; Pekko 30/31 and 13/14). No tag was created and no new campaign is proposed here; whether to run one (and the evidence tag, which needs a separate authorization) is the Owner's decision.

## 8. Limitations and open items (also in #461)
- #192 and #193 are open until the Owner evaluates this evidence.
- The review's INFO 1 to 3 are not acted on; the fixes after the review were not re-reviewed; `guardian.go`, `stop.go` and the helpers were not read by the reviewer.
- A helper goroutine is not tested (readback and review only); the claim "starts no goroutine of its own" is about the three functions, not the process (the default executor starts one per activation).
- Under an inline executor (tests only) `Terminate` can be held by a hook and `Shutdown`'s context cannot bound it (documented, SH11).
- Not delivered: `CoordinatedShutdown`, phases, hooks, `registerOnTermination`, cancellation of handlers (ADR 0008), an Ask registry, any promise about journal writes, durability or draining. Behavior of pending Asks when the whole system terminates, and termination before the first activation of the guardians, are *sin verificar* in the pinned Pekko tree.
- The code commit is 868 lines, above the ~700 heuristic; not rewritten.
- Package defect candidate: the implementer's allowed command strings (section 5).
