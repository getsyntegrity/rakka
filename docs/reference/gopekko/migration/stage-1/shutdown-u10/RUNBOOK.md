# RUNBOOK: ordered minimal termination of the system, U10 (design #192, implementation #193)

Status: PROPOSED package. Nothing is implemented, no agent has run, and approving the hashes does not by itself authorize crossing 450,000 tokens. Base `develop` `ced37b9`; kit `cf91c9d`; upstream `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. The contract, evidence tags and decisions are in `RULEBOOK.md`; this file is the flow, budget, procedure, commands and hashes.

## 1. Scope
`Terminate()`, `WhenTerminated() <-chan struct{}` and `Shutdown(ctx) error` on `*System`, plus `ErrSystemStopping` (is-a `ErrParentStopping`); `Shutdown` returns `ctx.Err()` unwrapped; `ErrSystemStopping` reads `actor: system is stopping`; a top-level `Spawn` returns `ErrSystemStopping` once `/user` is stopping. Ordered termination of the `System` only: not `gracefulStop`, not `CoordinatedShutdown`, no phases or hooks, no cancellation of handlers, no Ask registry, no promise about journal writes, durability or draining, no widening to Persistence. No new issue: #192 (design) and #193 (implementation) are reused. No judge scenario and no judge run.

## 2. Preparation findings (by reading, no execution; nothing in the repository changed)
- **The runtime already sustains the design [G].** The three guardians are wired synchronously before any activation (`guardian.go:140-146`), every guardian stops itself on any `Terminated` (`:20-37`), and `finalize` closes `root`'s `done` after its descendants. So `Terminate` is `Stop` of `/user` and `WhenTerminated` is `root`'s `done`; no runtime file needs editing. T5/T6 assert part of the chain. The RULEBOOK's SH4 verifies it for the listed executor orders (FIFO and LIFO prefixes, deferred, inline, real; **not** every interleaving), and the implementer stops with `BLOCKED(runtime)` if some order breaks it. **One claim of the first drafts was found incompatible and corrected before any run:** the cascade is scheduled through the executor (`stop.go:59`), so `Terminate` is "accepts the stop and hands it over", not "non-blocking" in every executor; the executor is a test seam (`options.exec`), so every user-built `System` uses the goroutine executor (RULEBOOK D10).
- **No new dependency**, so there is no seed step (unlike U9).
- **Judge anchors** live in `cell.go`, `stop.go`, `ref.go`, `hierarchy.go`, `mailbox.go`, `watch.go`, `supervision_decider.go` and `supervision_restart.go` (`judge/mutate_go.py`); none is touched. The Go mutation report binds the whole `pekko-actor/` tree, so its CURRENT line will stop applying whichever file changes (RULEBOOK 9); by the Owner's decision no campaign is run here.
- **Lesson of U9 applied.** The implementer's permissions only read inside the worktree, and this package needs nothing outside it (no module cache, no external file). A run blocked by a defect of the package still counts toward the implementer gate.
- **Baseline known [G]:** `scripts/check-judge-matrix.py` exit 0 at `ced37b9` with the tag `evidence/go-campaign-50fe7c2` (Go CURRENT 17/17, benign 1/1).
- Premises carried to the run, **verified by reading and not by running**: that the wiring holds for every executor order; that `Terminate` before the first activation of the guardians is safe (`acceptStop` before the first `Execute`: `PreStart` once, `PostStop` once, ADR 0012); that the existing tests compare `ErrParentStopping` only with `errors.Is`. A contradiction is a `BLOCKED(...)` of the implementer or a coordinator blocker with evidence.

## 3. Documentation included in this package PR
Only the correction of stale U9 text: the U9 rows of ADR 0017 (the issue table and the unit table; they said "code in draft PR; issue open") now state that U9 is integrated (PR #477, squash `ced37b9`) and #451 is closed with no independent review, and the status line of `migration/stage-1/supervision-u9/EXECUTION-RECORD.md`. No code and no other document changes. Issues #192, #193 and #461 are updated by editing or commenting on them, not by files.

## 4. Flow, limits and how coordination is counted
| Role | Model | Count |
|---|---|---|
| Implementer, production and tests in **one session**, `run-agent.sh implementer` | `sonnet` | 1 |
| **One** focused review, `run-agent.sh review` (read-only, one-review lock, USD cap) | `opus` | 1 |
| Consultation on one concrete unresolved decision | `opus` | 0-1 |
| Gates, mutants, docs, corrections of confirmed blockers | the **coordinator** | 0 agents |
At most 3 agents. **Time: 45 minutes of active agent time**; `AGENT_WALL_MIN` implementer 30, review 12. Read-only exploration subagents used to prepare the package are not part of the delivery run.
**What limits the review:** mechanically the USD cap (3), the wall clock (12 minutes), read-only permissions and `review.lock`; by instruction only the 1,500-token report. **What limits the implementer:** the 6 USD cap, the 30-minute clock, the surfaces of section 6 and one scoped test command.

**This delivery:** maximum **560,000** tokens, evaluation stop **450,000**. Coordination is a convention, not a measurement: `used = agents measured (input + output + cache creation, every run, failed or blocked ones included) + reserve`, stated as an estimate. Reserve: **70,000** coordination, **25,000** for the gates and the eight mutants (no seed, no campaign), plus **30,000** per correction round.
| Item | Last observed (U9 / U8) | U10 expected |
|---|---|---|
| Implementer | 300,827 (two runs) / 142,329 | 110,000 to 200,000 (three small production files, two test files of about 500 lines, one scoped test run per iteration) |
| Opus review | not launched / 55,656 | 60,000 to 110,000 (USD cap 3) |
| **Agents** | 300,827 / 197,985 | **170,000 to 310,000** |
| Reserve (70,000 + 25,000 + 30,000) | about 165,000 | 125,000 |
| **Total used (estimate)** | about 466,000 / 422,985 | **295,000 to 435,000**, central about **360,000** |
The whole range is under the evaluation stop. Gates: the review is launched only if **all implementer runs together used at most 250,000 tokens** and `observed + remaining reserve + 110,000 <= 560,000` (at 250,000 that is 250 + 125 + 110 = 485,000); otherwise the coordinator stops and asks, and the review is not launched. A correction round starts only if `observed + remaining reserve + 30,000 <= 560,000`. At 450,000 it stops and asks before any correction round or extra agent unless the Owner said, in words, that crossing is authorized. 560,000 is the ceiling, never a target. A run killed by the wall bound counts its whole forecast as spent. Nothing is consumed until the Owner approves and says to run.

## 5. Procedure after the Owner approves
0. **Merge the package PR** by the Owner's word; create `feat/actor-shutdown-u10` from the resulting `develop`.
1. **Baseline:** clean branch; `git rev-parse HEAD:pekko-actor`; `shasum -a 256` of every `.go` file of `pekko-actor/actor/`, of `go.mod`, `go.sum`, `README.md`, and of `judge/`'s tracked files, written outside the repository; submodules at `cf91c9d` and `9a54d1294c`; the approved hashes of section 9 compared (including the RUNBOOK's own from `RESUME.md`). Each command a separate call.
2. **Implementer** (`sonnet`, `AGENT_WALL_MIN=30`), command of section 8. A `BLOCKED(surface)` that comes from a defect of this package is reported to the Owner before any relaunch (a relaunch counts); `BLOCKED(runtime)` stops the unit with the interleaving as evidence; `NEEDS-DECISION` goes to the Owner.
3. **Scope control:** `git diff --stat` and `shasum` against the baseline; only the surfaces of section 6 changed; no existing test edited; `go.mod`, `go.sum`, `judge/` and the Pekko side byte-identical; the exported surface is exactly the four identifiers.
4. **Gates by the coordinator:** `gofmt -l`, `go vet`, `go mod tidy -diff`, the pinned `golangci-lint` v2.14.0 (the installed 2.13.1 fails to load packages), coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/check-adr-signatures.sh`, `scripts/modules_selftest.sh`, `.github/scripts/go-sdk-update.sh --check`, `go test -count=1 ./...`, `go test -race -count=1 ./...`, and the new tests 30 times under `-race`. The existing tests must pass **without edits**. **Not run: `judge/run-go.sh`, `judge/mutate_go.py`, the judge self-tests.**
5. **Mutants MS1-MS8** (RULEBOOK 8) one at a time on fresh copies outside the repository (`GOWORK=off`, package timeout 900 s); record the patch, the tests that failed, and that the unmodified copy passed; run a mutant also against its designed witness alone if another test killed it first; INVALID runs are listed apart.
6. **The one review** (`opus`, command of section 8) only if the forecast gate allows it; otherwise the execution record says plainly that there is no independent review. BLOCKER / DEFECT / INFO; INFO recorded, not acted on. Fixes of confirmed findings by the coordinator, test first.
7. **Documentation by the coordinator, once:** `pekko-actor/README.md` (the new API, the order, the guarantees and the non-guarantees, `Shutdown` from a handler, the sentences that said "no `Shutdown`" at `:153,194,220,248,400`), `pekko-actor/actor/doc.go`, `README.md`, `docs/actor-core-parity.md` (no parity claim), ADR 0014 (`:109-115,127-128,140`, D7 with `ErrSystemStopping`), ADR 0015 (D7 row and AK17: resolved as "not guaranteed"), ADR 0017 (`:146,725`), the execution record and `RESUME.md`.
8. **No campaign.** Commit everything, then run `scripts/check-judge-matrix.py` once, read-only, and record its exact output (it is expected to stop applying the Go CURRENT line because the tree changed); do not "fix" it, do not run the judge, create no tag. Whether a later campaign is run is the Owner's decision.
9. **PR** into `develop`, draft, commits of about 700 changed lines at most (production and tests, docs; Conventional Commits, no AI attribution lines). No merge without the Owner's word.
10. **#192 and #193:** after the PR, comment the outcome; **closing them needs the Owner's agreement** and the five points of RULEBOOK 10. #461 is updated.

## 6. Allowed edit surfaces
Production (3), under `pekko-actor/actor/`: `system_shutdown.go` (new), `errors.go`, `spawn.go`.
New tests (2): `system_shutdown_internal_test.go`, `system_shutdown_api_test.go`.
Coordinator only, after the run: `pekko-actor/README.md`, `pekko-actor/actor/doc.go`, `README.md`, `docs/actor-core-parity.md`, `docs/adr/0014-actor-children-and-hierarchical-termination.md`, `docs/adr/0015-ask-and-reply.md`, `docs/adr/0017-supervision.md`, `migration/stage-1/shutdown-u10/EXECUTION-RECORD.md`, `RESUME.md`.
Denied by the rendered permissions: every other `.go` file of the runtime and every existing test, `judge/`, `scripts/`, `go.mod`, `go.sum`, workflows, the RULEBOOK. `go test` is denied except through the single wrapper.

## 7. What U10 delivers and declares
Delivers the ordered minimal termination of the `System` with the API and invariants of the RULEBOOK. Declared differences from Pekko (RULEBOOK 12): no `CoordinatedShutdown`, phases, hooks, `registerOnTermination`, `isTerminating` (a scope decision); a channel instead of a `Future[Terminated]`, a typed error instead of `IllegalStateException` and `Shutdown(ctx)` are goPekko decisions; pending Asks not completed by the termination (Pekko: whole-system case sin verificar); I1 is relations, not a total order. Each is marked P or G in RULEBOOK 12. Not delivered: cancellation of handlers (ADR 0008), the end of the system by `/user` failure as a feature (it already works and stays as is), any guarantee about journal writes, durability or draining, the Go campaign. The Go campaign is darwin/arm64 only and is not run. No parity is claimed.

## 8. Exact execution commands (after approval and the merge of the package PR)
Run from the root of the worktree on `feat/actor-shutdown-u10`. `D` is a directory **outside** the repository that exists; the three `printf` lines carry the approved hashes of section 9.
```
D=<directory outside the repository>
printf '%s  RULEBOOK.md\n' 986a86defcb7c57d38da1f229da6c1e450268d6b1025ed2b087fefefdf0e1f6d > "$D/rb.sha"
printf '%s  settings.implementer.template.json\n' 751ff472f33de192daaada82d95ad747f1f0c516a146e0c11e71ee2b692a8e2b > "$D/st-impl.sha"
printf '%s  settings.review.template.json\n' 70d32b49a66fc17bb7619cb5702c4f1e1dbbab77ebbe4e89d39401950696d844 > "$D/st-rev.sha"
AGENT_WALL_MIN=30 bash migration/stage-1/shutdown-u10/run-agent.sh implementer sonnet migration/stage-1/shutdown-u10/implementer-prompt.md "$D/implementer.json" "$D/rb.sha" "$D/st-impl.sha"
```
Review (step 6, only after steps 3 to 5 and the forecast gate):
```
AGENT_WALL_MIN=12 AGENT_BUDGET_USD=3 bash migration/stage-1/shutdown-u10/run-agent.sh review opus migration/stage-1/shutdown-u10/review-prompt.md "$D/review.json" "$D/rb.sha" "$D/st-rev.sha"
```
The launcher refuses a RULEBOOK or permission template whose hash differs (exit 3), renders the permissions for this worktree, bounds the wall clock, caps spend per agent (6 USD implementer, 3 USD review), refuses to overwrite an output, keeps a failed output as `<out>.failed.<N>` and, for the review, refuses a second launch (exit 4).

## 9. Hashes for approval
| File (in `migration/stage-1/shutdown-u10/`) | sha256 |
|---|---|
| `RULEBOOK.md` | `986a86defcb7c57d38da1f229da6c1e450268d6b1025ed2b087fefefdf0e1f6d` |
| `settings.implementer.template.json` | `751ff472f33de192daaada82d95ad747f1f0c516a146e0c11e71ee2b692a8e2b` |
| `settings.review.template.json` | `70d32b49a66fc17bb7619cb5702c4f1e1dbbab77ebbe4e89d39401950696d844` |
| `run-agent.sh` | `f2390e6e82f0366c8bf7ba2e8e148409ec778114e5dbd6e6d0545440600464d4` |
| `run-shutdown-tests.sh` | `75126ed9f69258e29e0faffdacc6d0124d5705f86356e3dd17a5988e970dbe75` |
| `manifest.tsv` | `3a3c3ef4fc658d4d589d7016bdc0bd85a516e3aa1506c44e785b6b1aaf2f0145` |
| `implementer-prompt.md` | `714adb1a71ddc633394d3b44032961aacb698adad4257cf9e41e1e269a5073cc` |
| `review-prompt.md` | `6ba7241508f6b64ba3da0563c57bae467b364fe63ab186fafe75fae9f940c3a8` |

The hash of this RUNBOOK cannot be inside it: it is given with the package in `RESUME.md`. The launcher mechanically checks only the RULEBOOK and the template of the role; the other hashes are compared by the coordinator in step 1.

## 10. What approving this package authorizes
Approving the hashes authorizes **the execution of U10 only**, within 560,000 tokens and 45 minutes of agent time: the scoped test run of the implementer, the mutants, and at most one review. It authorizes no merge, no native review, no second review, no judge run or campaign, no evidence tag, no closing of #192 or #193, and no work on Persistence, `CoordinatedShutdown`, cancellation of handlers or the module-close campaigns of #461. It does not by itself authorize crossing the 450,000 stop.

## 11. Issues
#192 (design) and #193 (implementation) are reused: the design comment of #192 and the pointer of #193 already exist; the package PR is linked in both when it is opened. #461 is updated with the unit. No new issue is opened.
