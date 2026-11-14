# RUNBOOK: Supervision U5c, public guardian strategy and top-level names (#445, delivery 3 of 3)

Status: PROPOSED for the Owner's final approval of the package. **Execution is NOT authorized by this file**: it starts only when the Owner approves the hashes of section 9 and says to run. Delivery U5c of unit U5 (ADR 0017 S1, H1(a)), kit `cf91c9d`, upstream `9a54d1294c` (`v1.7.1`). Design: `../RULEBOOK.md` (D1-D12), approved at `981a6ea`. Base: `develop` at `eeac94d8895f64bb9bd0559712b9dda9a548a5f2` (U5b merged by squash, #465; its tree `aec0118b8084a3168c3dd557509064644ea3003e` is identical to the verified head `a3bc639`). Delivery policy of 2026-10-15 applies. Procedures not mentioned here are those of `../u5b/RUNBOOK.md` and `../u5b/EXECUTION-RECORD.md`.

## 1. Scope
Exactly RULEBOOK section 9, row U5c: `Config.GuardianStrategy`, `GuardianStrategy`, `GuardianDecider`, `GuardianOneForOne`, `StoppingGuardianStrategy` (section 3.5, signatures unchanged); the `topLevel` marker of U5b retired and **top-level names validated like child names (D7)**; tests T8, T5p, T15, P3 and the external Example; the ADR 0014 amendment (b) (D4 replaced). Not in scope (G4, D9): `System.Shutdown`, `whenTerminated`, termination hooks, `Kill`, custom guardian `Props`, supervision logging (U9), `AllForOne` (U6). `ActorSystemSpec:328,359` stay `PORT-GAP(system-shutdown)` (P7). The package closes U5; **#445 closes only after the Owner verifies the module-level criteria**, not by this package.

**Breaking change (D7, approved with the RULEBOOK):** `Spawn(sys, …)` with `""`, `"$x"`, `"a b"`, `"a/b"` or `"%zz"` now returns `ErrInvalidName`.

## 2. Preparation findings (measured by a throwaway probe, reverted)
The whole production change was applied on `eeac94d` and the complete suite run:
- **One existing test fails: `TestTopLevelNamesAreNotValidated`** (`names_test.go:86`), as D7 says. No other test spawns a top-level actor with an invalid name, so no other test needs adaptation.
- **Hidden coupling:** the name check must stay `t.parent != nil && !validChildName(name)`: the root guardian is created with name `""` and no parent and must remain exempt. Only the `!t.topLevel` term and the marker are retired.
- **Tests with their own executors:** none is affected (the change adds no activation and no ID).
- **Judge anchors:** only `hierarchy.go` of the files in the surfaces carries one (` || len(n.out)+n.reserved != 0 {`); it is untouched. `cell.go` is **not** a surface (it holds 5 anchors), see section 3.
- The adapter `GuardianStrategy → Strategy[guardianMsg]` compiled in the probe in about 15 lines; whether a panicking decider ends the system as 3.5 states is verified by T5p (if it does not hold the implementer reports `NEEDS-DECISION`).

## 3. INFO items of U5b: which enter U5c, which go to #461 (the Owner's proposal request)
| Item | Decision | Why |
|---|---|---|
| Stale comments in `supervision.go:69`, `supervision_hooks.go:40`, `watch.go:361` | **Enter U5c** (comments only, surfaces) | no judge anchor; they describe the behavior U5c finishes |
| `errors.go:27` root-owner text | **Enters U5c** (comment) | D7 changes what `ErrInvalidName` covers |
| T7 only rules out `Ref[int]`; add `Ref[guardianMsg]` | **Enters U5c** (one assertion, `guardian_goside_internal_test.go`) | pins D5/D11, trivial |
| V14 escalating case asserts only the first event | **Enters U5c** (one assertion, `supervision_goside_internal_test.go`) | matches its own comment |
| RULEBOOK 3.3.1 text for the accepted `[user, system, root]` order | **Coordinator, after the run** (docs step 7) | the RULEBOOK is hashed for the launcher: editing it before the run would be refused |
| Stale comments in `cell.go:26,292` | **#461** | `cell.go` holds judge anchors; comment edits there are cheap to batch with the anchor check at module close |
| `TestSpawnRacingTheCloseIsStoppedOrRefused` depends on scheduling | **#461** | a deterministic version needs a new seam in the hierarchy or cell, a design change, not a U5c need |
| Mutants, judge campaign, repeated `-race`, Pekko surface inventory, S13/S14/S16 mirror, parity | **#461** | module close (policy of 2026-10-15) |

## 4. Flow, limits and how coordination is counted
| Role | Model | Count |
|---|---|---|
| Implementer, production and tests in **one session**, `run-agent.sh` | `sonnet` | 1 |
| One focused review, read-only Agent-tool subagent, **hard limit 100,000 tokens** (U5b's review used 127,642: the brief must say it is a limit, name the files, and ask for the final report within 1,500 tokens) | `opus` | 1 |
| Consultation on one concrete unresolved decision | `opus` | 0-1 |
| Corrections of confirmed blockers and demonstrated defects | the **coordinator** | 0 agents |

At most 3 agents, no Haiku probe. **Time: 45 minutes of active work**; `AGENT_WALL_MIN` implementer 30, review 12.

**This delivery:** maximum **560,000** tokens, evaluation stop **450,000** (RULEBOOK section 9). No aggregate is authorized. Coordination is a convention, not a measurement: reserve **90,000**, **+30,000** per correction round or repeated full gate cycle; `used = agents measured (input + output + cache creation) + 90,000 + 30,000 × extra rounds`, stated as an estimate.

| Item | U5b observed | U5c expected |
|---|---|---|
| Implementer | 298,504 | 150,000 to 230,000 (about 60 production lines, a public API with docs, two new test files, three small test edits) |
| Opus review | 127,642 | 80,000 to 100,000 (limit) |
| **Agents** | 426,146 | **230,000 to 330,000** |
| Reserve | 90,000 | 90,000 |
| **Total used (estimate)** | 546,146 | **320,000 to 420,000** |

Expected to stay under the evaluation stop. **Forecast gate** before each agent: `observed + reserve + forecast`; if it passes the cap, stop and ask. At 450,000 the coordinator stops and asks before any correction round or extra agent. A run killed by the wall bound counts its whole forecast as spent. Nothing is consumed until the Owner approves and says to run.

## 5. Procedure after the Owner approves
0. **Merge the package PR** by the Owner's word; create `feat/actor-supervision-u5c` from the resulting `develop`.
1. **Baseline:** clean branch; `git rev-parse HEAD:pekko-actor/actor` and `shasum -a 256` of every `.go` file of `pekko-actor/actor/`, written outside the repository (create that directory first); submodules at `cf91c9d` (kit) and `9a54d1294c` (upstream); the approved hashes of section 9 written to files outside the repository and compared. Each command a separate call (the tool guard rejects compound git commands).
2. **Implementer** (`sonnet`, `AGENT_WALL_MIN=30`), command of section 8.
3. **Gates by the coordinator:** `gofmt -l`, `go vet`, `go mod tidy -diff`, the pinned `golangci-lint` (`gocyclo` at most 10), coverage at least 85% (`scripts/check_coverage.sh <dir> 85` with the profile named by `--profile-name <module path>`), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/check-adr-signatures.sh`, `scripts/modules_selftest.sh`, `.github/scripts/go-sdk-update.sh --check`, `go test -count=1 ./...`, the judge anchors (`judge/selftest_mutate_go.py` and the `hierarchy.go` anchor once). An anchor that must change stops the work; `judge/` is never edited.
4. **One `go test -race -count=1 ./...`**, and the new public-API tests plus the start-up tests repeated 30 times under `-race`.
5. **Focused review** (`opus`, read-only, hard limit 100,000, `AGENT_WALL_MIN=12`): the exported surface equals RULEBOOK 3.5 and names no private type; the zero value is `DefaultStrategy`; nil and panicking deciders end the system as 3.5 says; `options.userStrategy` precedence; the root guardian stays exempt from name validation; no valid name regressed; T8, T5p, T15, P3 assert what they claim (not vacuous); the three INFO assertions. Findings cite a RULEBOOK rule or a source line; classes BLOCKER / DEFECT / INFO; INFO is recorded, not acted on.
6. **Scope control after the fact:** `git diff --stat` and `shasum` against the baseline; any `.go` file outside the 13 surfaces is a defect.
7. **Documentation by the coordinator,** once: the execution record, `pekko-actor/README.md` and `actor/doc.go` (public guardian strategy; names validated; limits), **ADR 0014 amendment (b)** (D4 replaced, history kept), ADR 0017 (S1 delivered, D-4 closed, gaps of D9 kept), `docs/actor-core-parity.md` and `actor-core-first-cut.md` (restated, no parity claimed), the RULEBOOK 3.3.1 text for the accepted U5a order, RESUME.
8. **PR** into `develop`, draft, CI `go` and `label`, commits of about 700 changed lines at most. No merge without the Owner's word. **No native review is requested** (separate decision of the Owner).
9. **Closing issues:** after the merge the coordinator verifies the acceptance criteria of #445 against the evidence (U5a, U5b, U5c, and the declared gaps G4/D9) and closes it **only if completely met and the Owner agrees**; otherwise it reports what is missing.

## 6. Allowed edit surfaces (13 files, all under `pekko-actor/actor/`)
Production (8): `guardian.go`, `system.go`, `spawn.go`, `hierarchy.go`, `errors.go` (comments), `supervision.go` (comment), `supervision_hooks.go` (comment), `watch.go` (comment).
New tests (2): `guardian_api_test.go`, `guardian_config_internal_test.go`.
Existing tests (3): `names_test.go`, `guardian_goside_internal_test.go`, `supervision_goside_internal_test.go`.
Denied by the rendered permissions: `cell.go`, `stop.go`, `names.go`, other `supervision*.go`, `judge/`, docs, READMEs, ADRs, the RULEBOOK, workflows.

## 7. What U5c delivers and declares
Delivers RULEBOOK section 4 rows about validated names and the public configuration. Declared gaps kept: `Shutdown`, `whenTerminated`, termination hooks, `Kill`, decider without `Context` (#450 item 4), supervision logging (U9). No parity is claimed.

## 8. Exact execution command (after approval and the merge of the package PR)
Run from the root of the worktree on `feat/actor-supervision-u5c`. `D` is a directory **outside** the repository that exists (create it first); the hash files hold the **approved** values of section 9.
```
D=<directory outside the repository>
printf '%s  RULEBOOK.md\n' 5991232cdd05e49f7ee6a2f2f7f1ae4016a9f878fbc76a6f3953e9aa7bc25209 > "$D/rb.sha"
printf '%s  settings.template.json\n' 5a6f72dfab80c223a5e003c4749ee5074cb16a10c83df70d4e54ef29e73bf53b > "$D/st.sha"
AGENT_WALL_MIN=30 bash migration/stage-1/supervision-u5/u5c/run-agent.sh sonnet migration/stage-1/supervision-u5/u5c/implementer-prompt.md "$D/implementer.json" "$D/rb.sha" "$D/st.sha"
```
The launcher refuses a RULEBOOK or template whose hash differs, renders the permissions, bounds the wall clock, caps spend at 6 USD per agent, refuses to overwrite an output and keeps a failed output as `<out>.failed.<N>`.

## 9. Hashes for approval
| File | sha256 |
|---|---|
| `../RULEBOOK.md` (unchanged since `981a6ea`) | `5991232cdd05e49f7ee6a2f2f7f1ae4016a9f878fbc76a6f3953e9aa7bc25209` |
| `settings.template.json` | `5a6f72dfab80c223a5e003c4749ee5074cb16a10c83df70d4e54ef29e73bf53b` |
| `run-agent.sh` | `85ce28a38d5cf569c8a8f64df53725eeac7b1fc751fef923a7ab8d01fd203bab` |
| `manifest.tsv` | `137dc5d308e0847bdedf9d725ba3143d6f2d57fee2b5f27a1fef6b1c40860800` |
| `implementer-prompt.md` | `f1719e7b8fd5eb9f8f7a73ad39016115e1a6e01e62f3f08af6f4c9f13667894e` |

The hash of this RUNBOOK cannot be inside it: it is given with the package and in `RESUME.md`.

## 10. What approving this package authorizes
Approving the hashes authorizes **the execution of U5c only**, within 560,000 tokens and 45 minutes. It authorizes no merge, no native review and no closing of #445.
