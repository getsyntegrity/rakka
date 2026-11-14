# RUNBOOK: Supervision U5b, top-level actors under `/user` (#445, delivery 2 of 3)

Status: PROPOSED for the Owner's final approval of the package. **Execution is NOT authorized by this file**: it starts only when the Owner approves the hashes of section 10 and says to run. Delivery U5b of unit U5 (ADR 0017 S1, H1(a)), kit `cf91c9d`, upstream `9a54d1294c` (`v1.7.1`). Design: `../RULEBOOK.md` (D1-D12, three deliveries), approved by the Owner at `981a6ea` (PR #462). Base: `develop` at `c87fb0e81194644efde026e2dce765666ec6a254` (U5a merged, #463). Delivery policy of 2026-10-15 applies (`docs/quality-gates.md`). Procedures not mentioned here are those of `../u5a/RUNBOOK.md` and `../u5a/EXECUTION-RECORD.md`.

## 1. Scope
Exactly RULEBOOK section 9, row U5b: `Spawn(System)` creates children under `/user`; top-level failures go to `/user`'s decider; `Parent()` of a top-level actor returns the `/user` handle; stopping `/user` terminates its actors and closes the admission of `Spawn` (`ErrParentStopping`); the provisional `s.root` table is retired; the affected tests are adapted keeping their purpose and assertions.

**Left to U5c, not in this package:** the public `Config.GuardianStrategy`, `GuardianStrategy`, `GuardianDecider`, `GuardianOneForOne`, `StoppingGuardianStrategy`, the external example, test T5p, T8, T15, P3, and the validation of top-level names (D7). Consequence fixed here: top-level names stay **unvalidated** in U5b (`TestTopLevelNamesAreNotValidated` passes unedited). No ADR text changes in U5b other than what section 6 says. #445 stays open.

## 2. Preparation findings (measured, not inferred)
A throwaway probe (never committed, reverted) applied the whole production change and ran the complete `pekko-actor/actor` suite on it, with `s.root` kept as an alias of `/user`'s node so that only behavior differences show:

- **Production change is about six lines:** a `userNode` in `guardianSet`; `System.spawnTarget()` = `{owner: guards.userNode, parent: guards.user}`; `newSystem` stops creating `root`. `Parent()`, the stop of `/user`, the closed gate and `ErrParentStopping` need **no new code**: they follow from top-level cells having a parent and from the existing node termination (T16 and T13b must prove it; if one does not hold the implementer reports `NEEDS-DECISION`).
- **Hidden coupling found:** `spawn.go:68` validates a name when `t.parent != nil`. Giving top-level actors a parent would validate their names, which is D7 and **U5c**. `spawn.go` is therefore a production surface: a marker field on `spawnTarget` keeps U5b's names unvalidated.
- **16 existing tests fail in behavior** (list in the implementer prompt): the interim-stop pins (T11, V14, T9), `lifecycle_internal_test.go` (`TestPreStartHooks`, `TestReceiveFailures`), `throughput_internal_test.go` (`TestStopCounters`), `watch_internal_test.go` (2), `watch_api_test.go` (2), `ask_endpoint_internal_test.go`, `completion_internal_test.go`, `hierarchy_internal_test.go` (`TestParentChildAndChildren`), and U5a's own `TestTopLevelActorsStayOutsideUserInU5a`, `TestGuardiansImmediateClose`, `TestLegacyTopLevelActorBeforeAnyGuardianActivation`. Many hung for 30 s on a `Done` that no longer closes (a top-level actor that fails now restarts), so the unadapted suite does not finish within the default timeout.
- **7 files do not compile once `System.root` is retired:** `hierarchy_internal`, `hierarchy_concurrent_internal`, `termination_internal`, `termination_stress_internal`, `completion_internal`, `ask_firstreply_internal`, `guardian_startup_internal`.
- **Tests that do not need an edit** (the probe passes them): `spawn_errors_test.go:159`, `spawn_supervised_goside` T7, `stop_goexit`, `supervision_restart_goside`, `judge_internal_test.go`, `names_test.go` and the rest of the suite. Three of them carry a stale `PORT-GAP(top-level-default-U5)` comment (comment-only surfaces). The inventory of the mapping explorer listed some of these as risks; the probe shows they hold, so they are **not** in the surfaces, and the judge scenarios are not touched.
- **Executors:** tests with their own controlled executors that fail a top-level actor gain one queued `Failed` for `/user`; none of them showed a count failure in the probe other than the ones listed.
- **Judge anchors:** of the files U5b edits only `hierarchy.go` carries one (` || len(n.out)+n.reserved != 0 {`, two mutants); it occurs once and the probe did not move it.

## 3. Flow, limits and how coordination is counted
| Role | Model | Count |
|---|---|---|
| Implementer: production and tests in **one session**, through `run-agent.sh` | `sonnet` | 1 |
| Consultation on one concrete unresolved decision | `opus` | 0-1 |
| One focused review, read-only Agent-tool subagent, **limit 100,000 tokens** | `opus` | 1 |
| Corrections of confirmed blockers and demonstrated defects | the **coordinator**, inside the surfaces | 0 agents |

At most 3 agents. No Haiku probe. **Time: 60 minutes of active work**; `AGENT_WALL_MIN` implementer 40, review 12.

**This delivery:** maximum **560,000** tokens, evaluation stop **450,000** (RULEBOOK section 9). The aggregate for U5 is **not** authorized; U5c is not authorized. Coordination is a convention, not a measurement: reserve **90,000** from the start, **+30,000** per correction round or repeated full gate cycle; `used = agents measured + 90,000 + 30,000 × extra rounds`, stated as an estimate.

| Item | U5a observed | U5b expected |
|---|---|---|
| Implementer | 301,272 | 260,000 to 340,000 (about 6 production lines, one new test file of 10+ scenarios, 16 behavior rewrites, 7 compile fixes, 3 comments) |
| Opus review | 108,972 | 90,000 to 100,000 (limit) |
| **Agents** | 410,244 | **350,000 to 440,000** |
| Reserve | 90,000 | 90,000 |
| **Total used (estimate)** | 530,244 after one extra round | **440,000 to 530,000** |

The evaluation stop will most likely be reached at or just after the review; then the coordinator stops and asks before any correction round. **Forecast gate** before each agent: `observed + reserve + forecast`; if it passes the cap, stop and ask. A run killed by the wall bound counts its whole forecast as spent. Nothing is consumed until the Owner approves and says to run.

## 4. Closing rule (Owner's instruction for this delivery)
**One** focused review. Blockers and demonstrated defects are corrected by the coordinator (inside the surfaces, recorded, each round charged). Informational improvements are **recorded in the execution record and not acted on**: no successive review cycles, no extra agents for them. **No native review is requested** (a native envelope needs the Owner's own consent and was not asked for).

## 5. Procedure after the Owner approves
0. **Merge the package.** The package PR is merged into `develop` by the Owner's word; create `feat/actor-supervision-u5b` from that `develop` (the provider takes the merge-base with `develop`).
1. **Baseline:** branch clean; `git rev-parse HEAD:pekko-actor/actor` and `shasum -a 256` of every `.go` file of `pekko-actor/actor/`, written outside the repository; submodules at `cf91c9d` (kit) and `9a54d1294c` (upstream); the approved hashes of section 10 written to files outside the repository.
2. **Implementer** (`sonnet`, `AGENT_WALL_MIN=40`), command of section 9.
3. **Gates by the coordinator:** `gofmt -l`, `go vet`, `go mod tidy -diff`, the pinned `golangci-lint` (`gocyclo` at most 10), coverage at least 85% (`scripts/check_coverage.sh`), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/check-adr-signatures.sh`, `scripts/modules_selftest.sh`, `.github/scripts/go-sdk-update.sh --check`, `go test -count=1 ./...`, and the anchor check (the 22 pairs of `judge/mutate_go.py`, once each). **An anchor that must change stops the work; `judge/` is never edited.** A failing test **outside** the surfaces: the coordinator stops and asks for that one file.
4. **One `go test -race -count=1 ./...`**, and T13b, T14b and T16 plus the start-up tests (`guardian_startup`) repeated 30 times under `-race`. Nothing else is repeated.
5. **Focused review** (`opus`, read-only, limit 100,000 tokens, `AGENT_WALL_MIN=12`): whether `/user`'s decider is the only path for top-level failures; the stop of `/user` reaches every top-level actor before `/user` finalizes, and a racing `Spawn` is either stopped or refused; no `Done` waits forever, no duplicated `Terminated` or `stop-accepted`; `Parent()`/T16 semantics (D11); top-level names still unvalidated; the adapted tests keep their assertions and purpose; U1-U5a regressions. Findings cite a RULEBOOK rule or a source line.
6. **Scope control after the fact:** `git diff --stat` and `shasum` against the baseline; any `.go` file outside the 25 surfaces is a defect.
7. **Documentation by the coordinator,** once, at the end: the execution record, `pekko-actor/README.md` and `actor/doc.go` where U5b changes a statement (top-level failure restarts, `Parent()`, stop of `/user`), ADR 0014 amendment (a) (the root owner is an actor, per RULEBOOK section 9), the RULEBOOK 3.3.1 text for the accepted U5a deviation if the Owner asks, and RESUME. No parity claimed.
8. **PR** into `develop`, draft, CI `go` and `label`; commits of at most about 700 changed lines each; the record with commands, models, observed tokens, the coordination rule and every manual change. No merge without the Owner's word.

## 6. What U5b delivers and declares
Delivers: RULEBOOK section 4 rows 1 to 4, 6 (the IDs/baselines) and 7. Declared gaps kept (G4, D9): `System.Shutdown`, `whenTerminated`, termination hooks, `Kill`, supervision logging (U9), `AllForOne` (U6). `ActorSystemSpec:328,359` stay `PORT-GAP(system-shutdown)`. U5b claims no coverage of S13, S14, S16 and no parity.

## 7. Allowed edit surfaces (25 files, all under `pekko-actor/actor/`)
Production (5): `guardian.go`, `system.go`, `hierarchy.go`, `spawn.go`, `supervision_failure.go` (comments only).
New test (1): `guardian_goside_internal_test.go`.
Existing tests (19): `guardian_internal_test.go`, `guardian_startup_internal_test.go`, `hierarchy_internal_test.go`, `hierarchy_concurrent_internal_test.go`, `termination_internal_test.go`, `termination_stress_internal_test.go`, `completion_internal_test.go`, `ask_firstreply_internal_test.go`, `ask_endpoint_internal_test.go`, `lifecycle_internal_test.go`, `throughput_internal_test.go`, `watch_internal_test.go`, `watch_api_test.go`, `supervision_default_internal_test.go`, `supervision_default_goside_internal_test.go`, `supervision_goside_internal_test.go`, `spawn_supervised_internal_test.go`, `spawn_supervised_goside_internal_test.go`, `spawn_errors_test.go` (the last three: stale comments only).
Denied by the rendered permissions: `cell.go`, `stop.go`, `watch.go`, `supervision*.go` other than `supervision_failure.go`, `errors.go`, `names_test.go`, `judge_internal_test.go`, `judge/`, docs, READMEs, ADRs, workflows.

## 8. Module-close backlog (not run in U5b)
Mutants for the module close: the default for top-level is Stop; `Parent()` still false; the top-level actors do not stop with `/user`; `Spawn` is admitted after `/user` closed; a top-level failure bypasses `/user`; plus the U5a list and everything in `#461`. A mutant that does not compile, times out or is a harness error is INVALID, not a detection.

## 9. Exact execution command (after the Owner's approval and the merge of the package PR)
Run from the root of the worktree, on `feat/actor-supervision-u5b`. `D` is a directory **outside** the repository; the two hash files hold the **approved** values of section 10, never the values read from disk.

```
D=<directory outside the repository>
printf '%s  RULEBOOK.md\n' 5991232cdd05e49f7ee6a2f2f7f1ae4016a9f878fbc76a6f3953e9aa7bc25209 > "$D/rb.sha"
printf '%s  settings.template.json\n' e70f26508063b3bdb9e947fcedfaedcd59605301d5c091567bd04bcdc72319fa > "$D/st.sha"
AGENT_WALL_MIN=40 bash migration/stage-1/supervision-u5/u5b/run-agent.sh sonnet migration/stage-1/supervision-u5/u5b/implementer-prompt.md "$D/implementer.json" "$D/rb.sha" "$D/st.sha"
```
The launcher refuses a RULEBOOK or permission template whose hash differs, renders the permissions for this worktree, bounds the wall clock, caps spend at 6 USD per agent, refuses to overwrite an output and keeps a failed output as `<out>.failed.<N>`.

## 10. Hashes for approval
| File | sha256 |
|---|---|
| `../RULEBOOK.md` (approved design, unchanged since `981a6ea`) | `5991232cdd05e49f7ee6a2f2f7f1ae4016a9f878fbc76a6f3953e9aa7bc25209` |
| `settings.template.json` | `e70f26508063b3bdb9e947fcedfaedcd59605301d5c091567bd04bcdc72319fa` |
| `run-agent.sh` | `8ba870f0bf76926a35dabcb9bdb558de8d0302591779c661b3b4e6ff5db5d19c` |
| `manifest.tsv` | `4d9649a274b7f1ff7292fd7eab8177c7cd001d8f93f1d3a9bd918a6da5ee39ed` |
| `implementer-prompt.md` | `8ff17f0f5e2d5c6e8c20221387b93cbf3ceba815c51ab38a936ee09e26f35dc1` |

The hash of this RUNBOOK cannot be inside it: it is given with the package and in `RESUME.md`.

## 11. What approving this package authorizes
Approving the hashes authorizes **the execution of U5b only**, within the cap of 560,000 tokens and 60 minutes. It authorizes no merge, nothing in U5c, no aggregate budget and no native review.
