# RUNBOOK: Supervision U5a, guardian cells and their start-up (#445, delivery 1 of 3)

Status: PROPOSED for the Owner's final approval of the package. **Execution is NOT authorized by this file**: it starts only when the Owner approves the hashes of section 11 and says to run. Delivery U5a of unit U5 (ADR 0017 S1, H1(a)), kit `cf91c9d`, upstream `9a54d1294c` (`v1.7.1`). The design (`../RULEBOOK.md`, decisions D1-D12, three deliveries) was approved by the Owner at commit `981a6ea18ee1084e3c8b77d806b0a6651b79ada0` (PR #462). Delivery policy of 2026-10-15 applies (`docs/quality-gates.md`, "Per unit and at module close"). Procedures not mentioned here are those of `migration/stage-1/supervision-u4/RUNBOOK.md` as merged, with the lessons of `supervision-u4/EXECUTION-RECORD.md`.

## 1. What this plan is
The reduced flow of U3 and U4 for the first of the three deliveries of U5: one Sonnet implementer for production and tests in one session, one Opus review, the coordinator runs every test and gate. U5a is almost purely additive: it builds the three guardian cells and their start-up and changes **nothing observable** for existing actors except three extra IDs and three extra activations (RULEBOOK 3.3.2). U5b (top-level actors move under `/user`, `Parent()`) and U5c (public configuration, name validation) are **not** part of this package, have no package yet and are not authorized.

## 2. Scope
Exactly RULEBOOK section 9, row U5a, with the intermediate behavior of 3.3.2: guardian cells and actors; strategies of D2 (the `/user` one default, through the unexported seam `options.userStrategy`); the Watch chain; the start-up of 3.4 with the `options.bootPoint` seam; the three termination sequences of 3.3.1 observed on the guardians. `s.root`, `System.spawnTarget`, `Config`, `Context.Parent()` and every exported identifier are untouched. The ACTINV identifiers are not in the repository (U4 RULEBOOK section 6): no ACTINV row is touched.

## 3. Flow, limits, token definition and how coordination is counted

| Role | Model | Count |
|---|---|---|
| Implementer: production and tests in **one session**, through `run-agent.sh` | `sonnet` | 1 |
| Consultation on one concrete unresolved decision | `opus` | 0-1 |
| Final review, read-only Agent-tool subagent | `opus` | 1 |
| Corrections of confirmed findings | the **coordinator**, inside the surfaces | 0 agents |

At most 3 agents. No Haiku probe (section 8). **Time: 60 minutes of active work**; `AGENT_WALL_MIN` implementer 30, review 12.

**Two figures, kept apart.** *This delivery:* maximum **560,000** tokens, evaluation stop **450,000**. *U5 as a whole:* a proposed aggregate of 1,680,000 (stop 1,350,000) that is **not authorized** (RULEBOOK section 9), and the executions of U5b and U5c are **not authorized** either. Approving this package authorizes this delivery only.

**How coordination is counted (a convention, not a measurement).** Agent tokens are measured: input + output + cache creation from the CLI JSON or the agent result. The coordinator's own consumption has no telemetry, so it is charged by rule:
- a **fixed reserve of 90,000** is counted from the start (the same figure U3 used as its estimate);
- **+30,000** is added for each correction round the coordinator performs after the review, and for each additional full gate cycle it has to repeat;
- `used = agents measured + 90,000 + 30,000 × extra rounds`.
The record states the final `used` as an estimate with this rule and never as a measured figure. In effect the agents have about **360,000** before the evaluation stop (450,000 − 90,000) and about **470,000** before the cap.

| Item | U4 observed | U5a expected |
|---|---|---|
| Implementer | 303,360 | 250,000 to 300,000 (one new production file, `spawn.go` split, two new test files, ten existing test files adapted by a baseline or a settle call) |
| Opus review | 160,512 | 110,000 to 160,000 |
| **Agents** | 463,872 | **360,000 to 460,000** |
| Coordination (reserve) | not measured | 90,000 |
| **Total used (estimate)** | above 450,000 | **450,000 to 550,000** |

**What to expect, stated plainly:** the evaluation stop of 450,000 will most likely be reached at or just after the review, and then the coordinator stops and asks before any correction round and before any extra agent. The margin to the cap of 560,000 is small. Levers the Owner may choose when authorizing: a tighter review brief (the plan already caps it at 12 minutes and gives the file list), or skipping the Opus review. **Forecast gate:** before each agent the coordinator writes `observed + reserve + forecast`; if it passes the cap it stops and asks. A run killed by the wall bound normally leaves no output, so its whole forecast counts as spent. Nothing is consumed until the Owner approves and says to run.

## 4. Lessons carried in
- The implementer cannot run `go test` (denied by configuration): the coordinator runs the suite. The surfaces already list the existing tests the extra boot activations and IDs affect; a failing test **outside** the list makes the coordinator stop and ask the Owner for that one file, as W21 did in U3.
- **Preparation finding, covered by RULEBOOK D8 and not a deviation:** RULEBOOK section 9 says that U5a breaks "only the count-based baselines". A static scan found that tests with a controlled executor that assert `ex.Len()` (the queue of activations) are also affected, besides the ID counts (`registered()`, `byID`): `throughput_internal_test.go`, `supervision_goside_internal_test.go`, `supervision_restart_goside_internal_test.go`, `termination_internal_test.go`, `hierarchy_internal_test.go`, `spawn_supervised_goside_internal_test.go`. D8 already says that "controlled executors see three extra activations" and that the schedule counts change. The fix in each is a settle call after construction, with the same assertion.
- `judge_internal_test.go` (a `holdExec` that holds activations) and other tests that build a system with a custom executor are **not** in the surfaces. If one fails because of the extra activations, the coordinator stops and asks; the judge scenarios' controlled schedules may be perturbed, and CURRENT is stale anyway.
- A review that sees only part of a change reports missing symbols as blockers: review the whole work unit.
- IDs are global (`actor-N`), not per system: tests compare the **order** of the guardians' IDs, never literal values.
- Tests prove "not processed" with a controlled or inline executor or a channel, never by reading a counter right after a `Tell`. Goexit-related tests use the real executor.
- The tool guard rejects compound commands and heredocs: scripts go to files outside the repository and run with `python3 -I`.
- `check_coverage.sh` needs the profile named as `--profile-name` prints it (with `.out`); the other modules show FAIL for lack of a profile, which is not a defect.
- The pinned `golangci-lint` (`.golangci-version`) is installed under a temporary `GOBIN`.
- **The provider takes the merge-base with `develop`.** A squash-merged package PR that is not in the ancestry of the code branch kept the base at the old commit in U4 and the candidate was rejected for size. So the package PR (#462) is merged into `develop` **before** the run, and the code branch is created from that `develop`.
- A native review is a separate decision with the Owner's own consent per envelope; a candidate of about 1,700 lines fit in U4, one of 2,121 did not.

## 5. Procedure after the Owner approves
0. **Merge the package.** PR #462 (the RULEBOOK and this package) is squash-merged into `develop` by the Owner's word; `develop` then holds the six files with the hashes of section 11 (the squash keeps the tree). Create the branch `feat/actor-supervision-u5a` from that `develop`.
1. **Baseline:** branch clean; `git rev-parse HEAD:pekko-actor/actor` and `shasum -a 256` of every `.go` file of `pekko-actor/actor/`, written outside the repository; `git submodule status` shows the kit at `cf91c9d` and `upstream/pekko` at `9a54d1294c`; the approved hashes of section 11 written to files outside the repository and compared with the files.
2. **Implementer** (`sonnet`, one session, `AGENT_WALL_MIN=30`), with the command of section 10.
3. **Gates by the coordinator:** `gofmt -l`, `go vet`, `go mod tidy -diff`, the pinned `golangci-lint` (`gocyclo` at most 10), coverage at least 85% of `pekko-actor` (`scripts/check_coverage.sh`), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/check-adr-signatures.sh`, `scripts/modules_selftest.sh`, `.github/scripts/go-sdk-update.sh --check`, the normal suite `go test -count=1 ./...`, and the anchor check (every `(file, anchor)` of `judge/mutate_go.py` occurs once in the file it names; U5a edits none of those files, so none should move). **If an anchor must change the work stops and reports; `judge/` is never edited.** Failures of the suite are fixed by the coordinator inside the surfaces, each recorded, each round charged as in section 3.
4. **One `go test -race -count=1 ./...`** of the module on the final code (it takes seconds), and the start-up and termination tests (T11 to T13, T14a) also repeated 30 times under `-race`, because they are synchronization tests. Nothing else is repeated.
5. **Final review** (`opus`, read-only, `AGENT_WALL_MIN=12`, Agent tool): the order of construction and that no guardian observes a partial hierarchy with either executor; the three termination sequences and the `Done` order at every boundary; waits that never end, duplicated `Terminated` or `stop-accepted`; the hidden root node; that `Spawn` keeps its behavior and `s.root` is untouched; the adapted existing tests keep their assertions; regressions of U1-U4. Each finding cites a RULEBOOK rule or a source line. The brief gives the file list and the RULEBOOK sections.
6. **Scope control after the fact** (section 8): `git diff --stat` and `shasum` against the baseline; any `.go` file outside the 15 surfaces is a defect.
7. **Documentation by the coordinator**, once, at the end: the execution record of U5a, `pekko-actor/README.md` and `doc.go` only where U5a changes a statement (three guardian IDs and the intermediate behavior; no statement about top-level actors changes), and the RESUME. No ADR text changes in U5a (RULEBOOK section 9). No parity is claimed and CURRENT stays stale.
8. **Native review** only if the Owner asks and consents per envelope, on a candidate that fits the provider's context budget; commits at most about 700 changed lines each.
9. **PR** into `develop` for the code, draft, CI `go` and `label`; the execution record with commands, models, observed tokens, the coordination rule and its result, and every manual change. #445 stays open: U5b and U5c remain. No merge without the Owner's word.

## 6. What U5a enables (no coverage claimed)
The guardian cells that U5b attaches the top-level actors to. U5a does not run, mirror or claim S13, S14, S16 or any upstream `ActorSystemSpec` test: those wait for U5b, U5c and `System.Shutdown`.

## 7. Allowed edit surfaces (15 files, all under `pekko-actor/actor/`)
Production (3): `guardian.go` (new), `system.go`, `spawn.go`.
New tests (2): `guardian_startup_internal_test.go`, `guardian_internal_test.go`.
Existing tests, adapted only for the extra IDs and activations (10): `hierarchy_internal_test.go`, `hierarchy_concurrent_internal_test.go`, `names_test.go`, `termination_internal_test.go`, `throughput_internal_test.go`, `supervision_goside_internal_test.go`, `supervision_restart_goside_internal_test.go`, `spawn_supervised_goside_internal_test.go`, `ask_endpoint_internal_test.go`, `ask_firstreply_internal_test.go`.
Everything else is denied by the rendered permissions, including `cell.go`, `stop.go`, `hierarchy.go`, `watch.go`, `supervision*.go`, `errors.go`, `judge/`, `docs/`, READMEs, ADRs and workflows. `cell.go` and `hierarchy.go` are deliberately excluded because judge anchors live there; the plan does not need them.

## 8. Permission control (no probe in this run)
No Haiku probe. The permissions stay restrictive (the allow list above; everything else denied; the non-file rules are identical to U4's). After the run the coordinator checks the modified files with `git diff --stat` and `shasum` against the baseline. **This is a post-hoc scope control, not a test of isolation**: it shows what changed, not that the permissions would have blocked a write outside the scope. The record says so.

## 9. Module-close backlog (not run in U5a)
Mutants for U5a, to be run on immutable snapshots at the module close; candidates: the guardian loses its children on restart (the `PreRestart` is not a no-op); the root guardian's strategy Restarts instead of Stopping; the chain does not stop `/system`; the root guardian does not stop `/user` when `/system` stops; a first activation is handed over before the links are wired; `/user`'s strategy ignores `options.userStrategy`; the IDs are not in the order root, `/user`, `/system`. Plus everything already listed in `#461`: the U3 and U4 mutants, the judge campaign (#320, #121, #122), the repeated complete `-race`, the integral review, the S13/S14/S16 mirror (#175), the Pekko surface inventory, the carried gaps of U2 and the informative findings of U3 and U4 (#459, #460). A mutant that does not compile, times out, or is a harness error is INVALID, not a detection.

## 10. Exact execution command (after the Owner's final approval and the merge of #462)
Run from the root of the worktree, on branch `feat/actor-supervision-u5a`. `D` is any directory **outside** the repository (for example `$CLAUDE_JOB_DIR/tmp`); the two hash files are written from the **approved** values of section 11, never from the files on disk, and the launcher refuses to run if the files differ.

```
D=<directory outside the repository>
printf '%s  RULEBOOK.md\n' 5991232cdd05e49f7ee6a2f2f7f1ae4016a9f878fbc76a6f3953e9aa7bc25209 > "$D/rb.sha"
printf '%s  settings.template.json\n' 6f06ebbc499de8a3ab5ff4e0247b2cba72876bbfd4ddd33abbebe4e9aa01496b > "$D/st.sha"
AGENT_WALL_MIN=30 bash migration/stage-1/supervision-u5/u5a/run-agent.sh sonnet migration/stage-1/supervision-u5/u5a/implementer-prompt.md "$D/implementer.json" "$D/rb.sha" "$D/st.sha"
```

The launcher checks the RULEBOOK (`../RULEBOOK.md`, one directory up) and the permission template against those hashes, renders the permissions for this worktree, bounds the wall clock, caps the spend at 6 USD per agent (`AGENT_BUDGET_USD`), refuses to overwrite an existing output and keeps a failed output as `<out>.failed.<N>`.

## 11. Hashes for approval
Filled by the coordinator when presenting the package; the files are not edited after. The hash of this RUNBOOK cannot be inside it: it is given with the package in the PR and in `RESUME.md`.

| File | sha256 |
|---|---|
| `../RULEBOOK.md` (approved design, `981a6ea`) | `5991232cdd05e49f7ee6a2f2f7f1ae4016a9f878fbc76a6f3953e9aa7bc25209` |
| `settings.template.json` | `6f06ebbc499de8a3ab5ff4e0247b2cba72876bbfd4ddd33abbebe4e9aa01496b` |
| `run-agent.sh` | `57b936d4738b43820862595354f3b9e2e876cea24d5200b1ef4e205b1cc57cdb` |
| `manifest.tsv` | `9553023697bcd0cccf701f4ff9f9cca076ee8922b526a09fdd28c4e94443f736` |
| `implementer-prompt.md` | `d8dd53517e02af404ba9bb62a8768a19c0d8b1895663b0016822e397a25f7410` |

## 12. What approving this package authorizes
Approving the hashes authorizes **the execution of U5a only**, described above, within the cap of 560,000 tokens and the 60 minutes. It authorizes no merge, no change in U5b or U5c, no aggregate budget, and no native review (that needs the Owner's consent per envelope).
