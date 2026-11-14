# RUNBOOK: Supervision U9, `loggingEnabled` and `logFailure` (#451)

Status: PROPOSED for the Owner's approval of the package. **Execution is NOT authorized by this file**: it starts only when the Owner approves the hashes of section 9 and says to run. Unit U9 of ADR 0017 (S14, A11), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`). Contract: `RULEBOOK.md` (this directory). Base: `develop` at `ad98f7fc3e7a529d5ebc156daf1eda0407ff774d` (U7 and U8 records merged). Delivery policy of 2026-10-15 and the economical CI policy (`docs/ci-economy.md`) apply: CI runs only the fast check; race, judge, mutants and review are local. Procedures not mentioned here are those of `../supervision-u8/RUNBOOK.md`.

## 1. Scope
Exactly RULEBOOK sections 3 to 10: `Config.Logger`, `Strategy.WithLogging`, `GuardianStrategy.WithLogging`, the single `logFailure`, tests L1-L16 and the API test, nine directed mutants, the dependency seed, the docs and the final Go campaign. **Not in scope:** user-defined strategies and an overridable `logFailure` (#450), the restart-hook logs (`PreRestartException`), Typed, the Pekko side, the judge files, U6/U7/U8 behavior, `System.Shutdown`, the module-close campaigns of #461. **No native review is requested** (decision already taken by the Owner).

## 2. Preparation findings (by reading and by an offline probe; nothing in the repository was changed)
- **Dependency footprint:** static analysis of `pkg/logger` of the pinned kit-logger pseudo-version shows one third-party import, `golang.org/x/time/rate`; `kitlogtest` adds none. An offline probe (a throwaway module outside the repository, `GOPROXY=off`) resolved kit-logger from the module cache and then failed at `x/time` (`module lookup disabled by GOPROXY=off`): **building needs network once** (the public Go proxy, for `x/time`); kit-logger itself is in the cache. kit-logger has no tag; its newest cached pseudo-version is `v0.1.2-0.20260912231430-17d1a6eacc85` and declares `go 1.26.0` (the module is `go 1.27.0`).
- **Decision points are single:** every Resume, Restart and Stop goes through `handleFailed` (guardian cells included), so one call site covers the strategies and the guardian strategy.
- **The Go campaign report is bound to the `pekko-actor/` tree** (README included) and to `judge/`; U9 changes that tree. After U9 the Go **CURRENT** line of the matrix stops applying until a campaign is re-run (RULEBOOK 9, D9). The history binding needs an evidence tag at the end (the lesson of U8).
- Baselines measured at `ad98f7f`, to be recorded again before the run: `scripts/check-judge-matrix.py` exit 0 with the `evidence/go-campaign-e543c6a` tag (designed 35/35; Pekko observed 30/31 and 13/14; Go HISTORICAL and CURRENT 17/17); `judge/selftest.py` `190 checks, 0 failed`; `selftest_go_scope.py` and `selftest_mutate_go.py` `ok`.
- Premises carried to the run, **verified by reading and not by running**: that the import of kit-logger compiles offline once the coordinator's seed and `go mod download` are done; that `protect` can wrap the whole `logFailure` including a Goexit; that the mock's `Entries` can be read safely after the controlled executor drained. A contradiction is a `BLOCKED(...)` or `NEEDS-DECISION` of the implementer, or a blocker of the coordinator with evidence.

## 3. Documentation included in this package PR
None beyond this directory. #461 is reconciled by editing the issue (section 11).

## 4. Flow, limits and how coordination is counted
| Role | Model | Count |
|---|---|---|
| Implementer, production and tests in **one session**, `run-agent.sh implementer` | `sonnet` | 1 |
| **One** focused review, `run-agent.sh review` (read-only, one-review lock, USD cap) | `opus` | 1 |
| Consultation on one concrete unresolved decision | `opus` | 0-1 |
| Seed, gates, mutants, the Go campaign, corrections of confirmed blockers, docs | the **coordinator** | 0 agents |
At most 3 agents. **Time: 45 minutes of active agent time**; `AGENT_WALL_MIN` implementer 30, review 12. The Go campaign and the `go mod download` are script time.
**What limits the review**, as in U7/U8: mechanically (CLI and script) the USD cap (3), the wall clock (12 minutes), read-only permissions and `review.lock`; by instruction only the 1,500-token report. USD is a proxy for tokens (U8's review cost 55,656 tokens at USD 0.63; U6's 111,745 at 1.11).
**What limits the implementer** (D8): the 6 USD cap, the 30-minute clock, the surfaces of section 6 and one scoped test command; the seed leaves it offline.

**This delivery:** maximum **560,000** tokens, evaluation stop **450,000**. Coordination is a convention, not a measurement: `used = agents measured (input + output + cache creation) + reserve`, stated as an estimate. Reserve: **90,000** coordination, **45,000** for the seed, the nine mutants and the Go campaign, **30,000** for the gates, plus **30,000** per correction round.
| Item | Last observed (U8 / U7) | U9 expected |
|---|---|---|
| Implementer | 142,329 / 195,900 | 160,000 to 270,000 (about 140 production lines, three exported identifiers with docs, four places that carry the flag, a Goexit path, two test files of about 600 lines; it can run the logging tests once per iteration) |
| Opus review | 55,656 / 59,362 | 60,000 to 110,000 (USD cap 3) |
| **Agents** | 197,985 / 255,262 | **220,000 to 380,000** |
| Reserve (90,000 + 45,000 + 30,000) | 160,000 / 150,000 | 165,000 |
| **Total used (estimate)** | 422,985 / 465,262 | **385,000 to 545,000**, central about **450,000** |
**The central case sits on the evaluation stop and the upper tail is above it; approving this package does not by itself authorize crossing 450,000.** Gates: the review is launched only if **the implementer used at most 250,000 tokens** and `observed + remaining reserve + 110,000 <= 560,000` (at 250,000 that is 250 + 165 + 110 = 525,000); above that the coordinator stops and asks. A correction round starts only if `observed + remaining reserve + 30,000 <= 560,000`. At 450,000 it stops and asks before any correction round or extra agent unless the Owner said, in words, that crossing is authorized. 560,000 is the ceiling, never a target. A run killed by the wall bound counts its whole forecast as spent. Nothing is consumed until the Owner approves and says to run.

## 5. Procedure after the Owner approves
0. **Merge the package PR** by the Owner's word; create `feat/actor-logging-u9` from the resulting `develop`.
1. **Baseline:** clean branch; `git rev-parse HEAD:pekko-actor`; `shasum -a 256` of every `.go` file of `pekko-actor/actor/`, of `pekko-actor/go.mod`, `go.sum`, `README.md`, and of `judge/`'s tracked files (or the tree id of `judge`), written outside the repository; submodules at `cf91c9d` and `9a54d1294c`; the approved hashes of section 9 compared (including the RUNBOOK's own from `RESUME.md`); the baselines of section 2 recorded. Each command a separate call.
2. **Seed (coordinator, network once, public Go proxy for `golang.org/x/time` only):** in `pekko-actor`, `go get github.com/pablogore/kit-logger@v0.1.2-0.20260912231430-17d1a6eacc85` and `go mod download`; record the resulting `go.mod`/`go.sum` lines (expected: kit-logger direct, `x/time` indirect, plus the `go.sum` lines for both) and compare with the footprint of section 2; commit as a seed (`chore(actor): require kit-logger for the supervision logs (#451)`). If the footprint is larger than one indirect module, stop and report it. `go mod tidy -diff` is expected to fail at the seed and pass at the end.
3. **Implementer** (`sonnet`, `AGENT_WALL_MIN=30`), command of section 8.
4. **Scope control:** `git diff --stat` and `shasum` against the baseline; only the surfaces of section 6 changed; `go.mod`/`go.sum` equal to the seed; `judge/` and the Pekko side byte-identical.
5. **Gates by the coordinator:** `gofmt -l`, `go vet`, `go mod tidy -diff` (must pass now), the pinned `golangci-lint` v2.14.0 (the installed 2.13.1 fails to load packages), coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/check-adr-signatures.sh`, `scripts/modules_selftest.sh`, `.github/scripts/go-sdk-update.sh --check`, `go test -count=1 ./...`, `go test -race -count=1 ./...`, the new logging tests 30 times under `-race`, the three judge self-tests, and `judge/run-go.sh <outdir outside the repo> 300` (the existing Go gate: the logging must not change judged behavior). The existing tests must pass **without edits**.
6. **Mutants ML1-ML9** (RULEBOOK 7) one at a time on fresh copies outside the repository (`GOWORK=off`, package timeout 900 s); record the patch, the tests that failed, and that the unmodified copy passed; INVALID runs are listed apart from detections.
7. **The one review** (`opus`, command of section 8) only if the forecast gate allows it. BLOCKER / DEFECT / INFO; INFO recorded, not acted on. Fixes of confirmed findings by the coordinator, test first.
8. **Documentation by the coordinator, once, BEFORE the final campaign** (all of `pekko-actor/` including its README, and `judge/`, must be committed and clean before it): `pekko-actor/README.md` (the logging behavior, the attribute names, the divergences: nil Logger silent, the stricter guard, the restart-hook logs still absent), `pekko-actor/actor/doc.go` (comment), `README.md`, `docs/actor-core-parity.md` (the divergence "events but no logs" closed for strategies; no parity claim), ADR 0017 (U9 row, A11, S14, the gap lines), the execution record and `RESUME.md`.
9. **Final Go campaign (D9):** with `pekko-actor/` and `judge/` committed and clean, `python3 -I judge/mutate_go.py <workdir outside> --runs 30 --report <outside>/go-mutation-report.tsv` for all 18 mutants, in the background; **GMoverlap policy.** GMoverlap is a known flake (data races in `cell.go` raised by the mutant's own overlap; 1 of 6 on the U8 base code). If it ends in `harness-error` the campaign is **not** a valid complete campaign and is **never converted automatically into `met`**: its report is kept as it is, marked invalid with the race sites, and **no row of it is mixed into another report**. One further **complete** campaign may be run (script time); its result stands on its own, it is recorded as the second run with the flake that preceded it named, and both reports are kept and linked in the execution record. If the second one also fails, the coordinator stops and reports. No document claims GMoverlap is stable; whether a campaign that passed after a flake is acceptable evidence is the Owner's decision at closure. Only a complete, valid campaign is copied to `judge/go-mutation-report.tsv`; then run the matrix check (design apart from observed) and commit it. Retaining the campaign commit needs an evidence tag `evidence/go-campaign-<sha7>` pushed **only with the Owner's authorization**; until then the matrix fails its history check after the squash, as in U8.
10. **PR** into `develop`, draft, commits of about 700 changed lines at most (seed, production and tests, docs, report; Conventional Commits, no AI attribution lines). No merge without the Owner's word.
11. **#451:** after the merge the coordinator checks the five criteria of RULEBOOK 10 and closes it **only if completely met and the Owner agrees**. #461 is updated.

## 6. Allowed edit surfaces
Production (6), all under `pekko-actor/actor/`: `supervision_logging.go` (new), `supervision_failure.go`, `supervision.go`, `guardian.go`, `system.go`, `supervision_restart.go` (comment only).
New tests (2): `supervision_logging_internal_test.go`, `supervision_logging_api_test.go`.
Coordinator only: `pekko-actor/go.mod`, `go.sum` (and `go.work.sum` if it changes) at the seed; after the run `judge/go-mutation-report.tsv`, `pekko-actor/README.md`, `pekko-actor/actor/doc.go`, `README.md`, `docs/actor-core-parity.md`, `docs/adr/0017-supervision.md`, `migration/stage-1/supervision-u9/EXECUTION-RECORD.md`, `RESUME.md`.
Denied by the rendered permissions: every other `.go` file of the runtime and every existing test, `judge/`, `scripts/`, `go.mod`, `go.sum`, workflows, the RULEBOOK. `go test` is denied except through the single wrapper of D8.

## 7. What U9 delivers and declares
Delivers the strategy logs of Pekko (`loggingEnabled`, `logFailure` default behavior) on `kit-logger`. Declared gaps kept: the restart-hook logs (`PreRestartException` and similar), an overridable `logFailure` and user-defined strategies (#450), the `EventFilter` log assertions of the ported upstream tests, Pekko's default stdout log (a nil Logger is silent), Typed. The Go campaign is darwin/arm64 only. No parity is claimed.

## 8. Exact execution commands (after approval, the merge of the package PR and the seed of step 2)
Run from the root of the worktree on `feat/actor-logging-u9`. `D` is a directory **outside** the repository that exists; the three `printf` lines carry the approved hashes of section 9.
```
D=<directory outside the repository>
printf '%s  RULEBOOK.md\n' 1ce42d8664a7792b66970ad16d9fb43a45c63d7f16fae5ac3f4fd4951ab75729 > "$D/rb.sha"
printf '%s  settings.implementer.template.json\n' fc37fd2ee93a5e2f1eaa1140f6184d26f480fa9193e7f65ef472ec9c4c1a42a3 > "$D/st-impl.sha"
printf '%s  settings.review.template.json\n' 70d32b49a66fc17bb7619cb5702c4f1e1dbbab77ebbe4e89d39401950696d844 > "$D/st-rev.sha"
AGENT_WALL_MIN=30 bash migration/stage-1/supervision-u9/run-agent.sh implementer sonnet migration/stage-1/supervision-u9/implementer-prompt.md "$D/implementer.json" "$D/rb.sha" "$D/st-impl.sha"
```
Review (step 7, only after steps 4 to 6 and the forecast gate):
```
AGENT_WALL_MIN=12 AGENT_BUDGET_USD=3 bash migration/stage-1/supervision-u9/run-agent.sh review opus migration/stage-1/supervision-u9/review-prompt.md "$D/review.json" "$D/rb.sha" "$D/st-rev.sha"
```
The launcher refuses a RULEBOOK or permission template whose hash differs (exit 3), renders the permissions for this worktree, bounds the wall clock, caps spend per agent (6 USD implementer, 3 USD review), refuses to overwrite an output, keeps a failed output as `<out>.failed.<N>` and, for the review, refuses a second launch (exit 4).

## 9. Hashes for approval
| File (in `migration/stage-1/supervision-u9/`) | sha256 |
|---|---|
| `RULEBOOK.md` | `1ce42d8664a7792b66970ad16d9fb43a45c63d7f16fae5ac3f4fd4951ab75729` |
| `settings.implementer.template.json` | `fc37fd2ee93a5e2f1eaa1140f6184d26f480fa9193e7f65ef472ec9c4c1a42a3` |
| `settings.review.template.json` | `70d32b49a66fc17bb7619cb5702c4f1e1dbbab77ebbe4e89d39401950696d844` |
| `run-agent.sh` | `50949c95900d206320cb45f76bae69852ca871c1d30ba8d6017e19636ad838b0` |
| `run-logging-tests.sh` | `f118db1d31ee746d90261201c6b1c4c9172da4fdd54d94d2cda88a77751b453d` |
| `manifest.tsv` | `e4c9cdcdba0a12938ca49499103b650a6745283dde30673378047d9a9a63f248` |
| `implementer-prompt.md` | `998fc489e450dcc91cad75d5797666e17c8257d555b435ea8808acf2400d3909` |
| `review-prompt.md` | `20186da53996d72f725899a360f7e1f07fb3a9b33304865009644930be9f2c37` |

The hash of this RUNBOOK cannot be inside it: it is given with the package in `RESUME.md`. The launcher mechanically checks only the RULEBOOK and the template of the role; the other hashes are compared by the coordinator in step 1.

## 10. What approving this package authorizes
Approving the hashes authorizes **the execution of U9 only**, within 560,000 tokens and 45 minutes of agent time, including the network fetch of the dependency seed (step 2: the public Go proxy for `golang.org/x/time`; kit-logger from the local cache), the scoped test run of the implementer (D8), the mutants, the Go campaign and at most one review. It authorizes no merge, no native review, no second review, no push of an evidence tag (a separate authorization at step 9), no closing of #451 and no work on #450 or the module-close campaigns. It does not by itself authorize crossing the 450,000 stop.

## 11. Reconciliation of #461 (done with this package)
Updated by editing the issue: U9 listed as in preparation with this PR; the decisions D1-D9 that leave pendings (the nil-Logger divergence, the restart-hook logs, the `EventFilter` assertions of the ported tests, the Go CURRENT line after the tree changes, the evidence tag) are listed to be confirmed on execution.
