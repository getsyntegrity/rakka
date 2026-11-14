# Execution record: Supervision U9, `loggingEnabled` and `logFailure` (#451)

Status: delivered, integrated in PR #477 (squash `ced37b9`); #451 closed as completed by the Owner; **no independent review of the code** (the focused review was not launched, see section 4); the Go campaign commit is retained by the tag `evidence/go-campaign-50fe7c2`; no parity claim. Package: PR #476 (squash `9219b7f`), hashes in `RUNBOOK.md` section 9 and `RESUME.md`, all matched at execution. Branch `feat/actor-logging-u9`. Kit `cf91c9d`, upstream Pekko `9a54d1294c` (v1.7.1).

## 1. What was delivered
`Config.Logger` (kit-logger `Logger`), `Strategy.WithLogging(bool)` and `GuardianStrategy.WithLogging(bool)`; one entry per Resume (Warn), Restart and Stop (Error, with the cause) decision, written in `handleFailed` between the decider and the application (`supervision_logging.go`, `supervision_failure.go`). The flag is stored negated, is independent of the kind and is carried at the four places that replace a zero value by the default. A nil Logger is silent (declared divergence from Pekko's stdout default). The whole log is protected; a Goexit of the Logger applies the decision exactly once. Commits: seed `10e46d1`, code `1245343` (831 changed lines, above the ~700 planned for one commit; production and tests of one unit).

## 2. Clarification of the GMoverlap policy (Owner, at approval)
`RUNBOOK.md` step 9 prevails over the older sentence of `RULEBOOK.md` section 9: complete campaigns, separate reports, adverse evidence kept, and **no automatic conversion of a GMoverlap failure into `met`**.

## 3. Results
| Gate | Result |
|---|---|
| `gofmt -l`, `go vet`, `go mod tidy -diff` | clean (tidy passes on the final tree) |
| pinned `golangci-lint` v2.14.0 | 0 issues (after the fix below) |
| `go test -count=1 ./...`, `go test -race -count=1 ./...` | pass; coverage 98.1% of statements |
| logging tests 30 times under `-race` | pass |
| `check_deps.sh`, `check-headers.sh`, `check-adr-signatures.sh`, `modules_selftest.sh`, `go-sdk-update.sh --check` | pass (headers after the fix below) |
| judge self-tests, `judge/run-go.sh <out> 300` | `190 checks, 0 failed`; scope and mutate_go self-tests ok; all scenarios 300 of 300 |
| existing tests | pass without edits |

Fixes by the coordinator (mechanical): `TestLoggingL13` had cyclomatic complexity 11, split with a helper (`lgCheckSilentDefault`) without changing what it asserts; the header of `supervision_logging.go` had a suffix the header check rejects. The implementer's one failing test (L7, the factory-panic message is wrapped as `actor: factory panicked: ...`) had been corrected by it without a rerun; the rerun passed.

Mutants (RULEBOOK 7; unmodified copy passes; copies outside the repository, `GOWORK=off`, package timeout 900 s):
| Mutant | Result | Killed by |
|---|---|---|
| ML1 logs on Escalate | killed | L4, L5, L6, L14 |
| ML2 `loggingEnabled` ignored | killed | L6, L11, L13, L14, API (L12 is not among them) |
| ML3 levels swapped | killed | L1, L2, L3, L4, L7, L8, L11, L12, API |
| ML4 unprotected logger call | killed | L9 |
| ML5 AllForOne logs per child | killed | L8, L15, L16 |
| ML6 nil Logger dereferenced | killed | **L14, not L10** (L10 alone does not kill it: the extended protection absorbs the nil dereference and the decision is unchanged) |
| ML7a cell `strategy()` loses the flag | killed | L13 |
| ML7b guardian default loses the flag | killed | L13, L14, L6 |
| ML8a `Error()` before the decider | killed | L9 in the full run; **also L14 with L14 run alone** |
| ML8b `Error()` with logging disabled | killed | L9 in the full run; **also L14 with L14 run alone** |
| ML9 applied before logged | killed | L15, L5, API |
Per the Owner's instruction no test was added only to keep the intended witness of ML6 (L10); the RULEBOOK table said L10. The first mutant pass is kept in `mut-run1` outside the repository; the L14-isolated reruns are the second evidence for ML8a, ML8b and ML6.

## 4. Deviations and limitations, said plainly
- **The first implementer run was blocked by a defect of the package and produced no code.** The permission template only allows reads inside the worktree, and the approved prompt asked it to read kit-logger from the module cache; it ended `BLOCKED(surface)` after 54,495 tokens, USD 0.28, with the tree untouched. The coordinator copied the two kit-logger sources (`interface.go`, `mocklogger.go`, unmodified) into an untracked `_kitlogger-ref/` inside the worktree and launched a second run with the approved prompt plus an addendum pointing to them (`_implementer-prompt-run2.md`); both helpers were removed afterwards and are not in the repository. The RULEBOOK and the permission template hashes were unchanged. Follow-up for the next package: let the template read the module cache.
- **Budget.** Implementer measured tokens (input + output + cache creation): run 1 54,495, run 2 246,332, **both counted: 300,827**, which exceeds the 250,000 gate of the implementer. The Owner ruled that both runs count and granted no exception and no increase of the 560,000 ceiling. **The focused review was therefore not launched, and there is no independent approval of this code**: its only verification is the tests, the gates and the mutants above. Findings an independent reviewer might raise (see section 5) are open.
- Coordination is an estimate, not a measurement (RUNBOOK 4).
- The scoped test wrapper and the whole suite pass on darwin/arm64 only; the Go campaign is darwin/arm64 only.

## 5. Open items (also in #461)
- Independent review of the code (not done).
- ML6's intended witness L10 does not kill it; L14 does. L10 asserts the nil Logger changes nothing but cannot see an absorbed dereference.
- Implementer doubts to be reviewed: for a `PreStart` panic the message is the panic value, not the `PanicError` text; the L16 event count right after the pongs could be flaky in theory (30 runs under `-race` passed); the strength of the last assertion of `L15LoggingDoesNotChangeTheBudgetCharge`.
- Restart-hook logs (`PORT-GAP(logging-U9)` at `runPreRestart`), overridable `logFailure` and user strategies (#450), `EventFilter` log assertions of ported tests, Typed.
- A nil Logger is silent (no Pekko default).
- New dependency in the core module: kit-logger (direct, no tag, pseudo-version `v0.1.2-0.20260912231430-17d1a6eacc85`) and `golang.org/x/time` (indirect).

## 6. Go campaign (D9)
One complete campaign, run once, with `pekko-actor/` and `judge/` committed and clean at `50fe7c259e41fe705327581234dfe772fe664a9f`: `python3 -I judge/mutate_go.py <outside> --runs 30 --report <outside>/go-mutation-report.tsv`, 18 mutants. `unmet: none`, no `harness-error`, **GMoverlap did not fail in this campaign** (it is still a known flake, 1 of 6 on the U8 base code, and nothing here claims it is stable). No second campaign was needed, so the GMoverlap policy of `RUNBOOK.md` step 9 was not exercised. The report was copied unchanged to `judge/go-mutation-report.tsv` (432 rows). `scripts/check-judge-matrix.py` exit 0: Go CURRENT detected 17/17 and benign 1/1 on the darwin/arm64 tree, comparator and HEAD; Go HISTORICAL as before; Pekko side unchanged (30/31, 13/14). Platform: darwin/arm64 only.

## 7. Retention of the campaign commit
The campaign commit `50fe7c2` is in the draft PR's history. After a squash it leaves the history and the matrix fails its history check, as in U8. Retaining it needs a tag `evidence/go-campaign-50fe7c2`, which is **not created or pushed**: it needs the Owner's separate authorization.
