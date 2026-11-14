# RUNBOOK: Supervision U4, supervised `Spawn` creation failures (#444)

Status: PROPOSED for the Owner's final approval of the package. **Execution is NOT authorized by this file**: it starts only when the Owner approves the hashes of section 11 and says to run. Unit U4 of ADR 0017, kit `cf91c9d`, upstream `9a54d1294c` (`v1.7.1`). Base: `develop` at `7f7e292` (U3 merged). The design in `RULEBOOK.md` was approved by the Owner on 2026-10-15 (candidate `32b9d6a` of PR #457); this package is the execution plan for it. Delivery policy of 2026-10-15 applies (`docs/quality-gates.md`, "Per unit and at module close"). Procedures not mentioned here are those of `migration/stage-1/supervision-u3/RUNBOOK.md` as merged (immutable), with the lessons of `supervision-u3/EXECUTION-RECORD.md`.

## 1. What this plan is

The reduced flow of U3, reused for U4: one Sonnet implementer for production and tests in one session, one Opus review, the coordinator runs every test and gate. The U3 package files are under approved hashes and are **not edited**; U4 has its own copies of the launcher, permissions and manifest, so nothing of U3 needs an amendment to run U4.

Inconsistencies inherited from U3 and corrected here, with no change of budget: (a) the launcher header stated a token cap of 900,000 with an evaluation stop at 720,000; the U4 launcher says 560,000 and 450,000, as section 3 does, and that the figure counts agents and coordination; (b) the U3 RULEBOOK pointed to "RUNBOOK section 9" for the hashes; in U4 the hashes are in section 11 of this RUNBOOK, and the approved U4 RULEBOOK cites no section number; (c) `RESUME.md` of U3 kept saying "execution NOT authorized" after the run; the U4 `RESUME.md` states the status as it is and is updated when the status changes.

## 2. Scope decisions

RULEBOOK D1 to D8 (approved). The ACTINV identifiers (ACTINV-028, ACTINV-067) are not in the repository (RULEBOOK section 6): no ACTINV row is touched by this run; they are updated only once the inventory file is in the repository. U5 (guardians) and everything in RULEBOOK section 5 stay out.

## 3. Flow, limits, token definition (PROPOSED, the cap is not raised)

| Role | Model | Count |
|---|---|---|
| Implementer: production and tests in **one session**, through `run-agent.sh` | `sonnet` | 1 |
| Consultation on one concrete unresolved design decision | `opus` | 0-1 |
| Final review, as a read-only Agent-tool subagent (the launcher's permissions deny `git diff <range>`) | `opus` | 1 |
| Corrections of confirmed findings | the **coordinator**, inside the allowed surfaces (the implementer cannot be resumed: `--no-session-persistence`) | 0 agents |
| Verification of the corrections | tests and the author's own focused check, declared as such | 0 agents |

At most 3 agents. No Haiku probe (section 8). No second reviewer.

**Time: 60 minutes of active work in total.** `AGENT_WALL_MIN` for each agent: implementer 30, review 12; the rest is coordination, gates, corrections, documentation and the PR.

**Token budget: 560,000 is the cap, unchanged from U3; evaluation stop 450,000.** It counts agents (input + output + cache creation, from the CLI JSON or the agent result) plus coordination (an estimate, no telemetry). The basis is what U3 measured, not what U3 forecast:

| Item | U3 forecast | U3 observed | U4 expected |
|---|---|---|---|
| Implementer | 220,000 | 290,172 | 290,000 (fewer production lines than U3, but five existing test files are rewritten) |
| Opus review | 90,000 | 170,685 | 120,000 (bounded brief: file list, RULEBOOK sections, 12 minutes) |
| Consultation | 30,000 | none | 0 (30,000 if used) |
| **Agents** | 310,000-410,000 | 460,857 (plus a 130,229 verification pass that this plan does not include) | **about 410,000** |
| Coordination (gates, one targeted race, corrections, docs, PR) | about 90,000 | not measured | about 90,000 |
| **Total** | about 400,000-500,000 | above 450,000 | **about 500,000** |

The margin to the cap is small (about 60,000, 11%), smaller than U3's plan claimed. So: **the evaluation stop at 450,000 will most likely be reached after the review**, and then the coordinator stops and asks the Owner before any correction round and before any extra agent. A second agent pass does not fit this budget. **Forecast gate:** before each agent the coordinator writes `observed + forecast`; if it passes the cap it stops and asks. A run killed by the wall bound normally leaves no output, so its whole forecast is counted as spent. Nothing here is consumed until the Owner approves and says to run.

## 4. Lessons carried in (U3 execution record)
- The implementer cannot run `go test` (denied by configuration): the coordinator runs the suite and the targeted race, and the surfaces below already include every existing test the old contract makes false, so no W21-style extension is needed. If a failing test outside the list appears, the coordinator stops and asks the Owner for that one file.
- A reviewer that sees only part of a change reports missing symbols as blockers: review the whole work unit.
- Tests prove "not processed" with a controlled or inline executor or an explicit channel, never by reading a counter right after a `Tell`.
- The tool guard rejects compound commands and heredocs: scripts go to files outside the repository and run with `python3 -I`.
- A nested `protect` fires **both** enclosing Goexit handlers (U3 needed the `postStopGoexit` flag); Goexit tests use the real executor.
- A Go timeout, a build failure or a harness error is never a detection (relevant at module close).
- `check_coverage.sh` fails every module that has no profile in the directory: with only the `pekko-actor` profile the other rows show FAIL for lack of a profile, which is not a defect; the verdict of `pekko-actor` is the one that counts.
- The pinned `golangci-lint` (`.golangci-version`) is installed under a temporary `GOBIN`; the one on the machine may differ.
- **Judge anchors:** U4 edits `cell.go` and `stop.go`, where 11 of the 22 anchors of `judge/mutate_go.py` live (including the head of `callPreStart` and `PostStopper[T]); ok {`). The implementer prompt forbids moving them and tells it to put the factory step in a new function.

## 5. Procedure after the Owner approves

1. **Baseline:** branch `feat/actor-supervision-u4` clean at the head that holds this package; `git rev-parse HEAD:pekko-actor/actor` and `shasum -a 256` of every `.go` file of `pekko-actor/actor/`, written outside the repository; `git submodule status` shows the kit at `cf91c9d` and `upstream/pekko` at `9a54d1294c`; the approved hashes of section 11 written to files outside the repository and compared with the files.
2. **Implementer** (`sonnet`, one session, `AGENT_WALL_MIN=30`), with the exact command of section 10.
3. **Gates by the coordinator:** `gofmt -l`, `go vet`, `go mod tidy -diff`, pinned `golangci-lint` (`gocyclo` at most 10), coverage at least 85% of `pekko-actor` (`scripts/check_coverage.sh`), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/check-adr-signatures.sh`, `scripts/modules_selftest.sh`, `.github/scripts/go-sdk-update.sh --check`, the normal suite `go test -count=1 ./...`, and the anchor check (every `(file, anchor)` of `judge/mutate_go.py` occurs once in the file it names; 22 pairs were intact after U3). **If an anchor must change the work stops and reports; `judge/` is never edited.** Failures of the suite are fixed by the coordinator inside the surfaces, each recorded.
4. **One targeted `-race`** over the tests of Spawn, the start step, the factory failures, the inline-executor cases and the Goexit rows, `-count=1`, on the final code. The complete `-race` waits for the module close unless the Owner asks for it. A repeat (`-count=20`) only for a concrete failure, recorded.
5. **Final review** (`opus`, read-only, `AGENT_WALL_MIN=12`, Agent tool): focus on Goexit with an inline and with the real executor, the order inside `Spawn` and the start step, stop accepted before the first activation, parent stop or restart during initialization, the reservation and name lifecycle, hook accounting (no `PostStop` for a missing instance), and regressions of U1-U3 and of the anchors; each finding cites a RULEBOOK rule or a source line. The brief gives the file list and the RULEBOOK sections, not the whole repository.
6. **Scope control after the fact** (section 8): `git diff --stat` and `shasum` against the baseline; any `.go` file outside the 12 surfaces is a defect.
7. **Documentation by the coordinator** (manual, recorded): RULEBOOK rows 6-11: ADR 0011, 0012 and 0014, `pekko-actor/README.md`, `pekko-actor/actor/doc.go`, `docs/actor-core-parity.md`, `docs/actor-core-first-cut.md`, and the root `README.md` where it states otherwise. Gaps recorded as gaps; no parity claimed; CURRENT stays stale until the module closes and the text says so.
8. **Native review** only if the Owner's switch asks for it and he consents per envelope; commits of at most about 700 changed lines each; never the accumulated branch. Commits are made by the coordinator after `gofmt`, with Conventional Commit messages and no attribution lines.
9. **PR #457** (the unit's PR, draft against `develop`): the implementation commits go on this branch after the package; `EXECUTION-RECORD.md` with commands, models, observed tokens and every manual change. Issue #444 closes when its implementation and test criteria are met, linking the pending exhaustive validation to the module milestone. No merge without the Owner's word.

## 6. What U4 enables for the judge scenarios (no coverage claimed)
S14 (a watched child fails at its start and the watcher gets `Terminated`) gets its factory variant from U4 and its `PreStart` variant from U3. U4 does not run, mirror or claim S13, S14 or S16; that belongs to U8 (#175) and the module close.

## 7. Allowed edit surfaces (12 files, all under `pekko-actor/actor/`)
Production (5): `spawn.go` (the factory leaves `Spawn`), `errors.go` (sentinel docs), `cell.go` (the factory step before `callPreStart`), `stop.go` (the start step also runs from `terminate`, around `beginStartLocked`), `supervision_restart.go` (the Goexit of the factory of a restart).
Existing tests (5): `spawn_errors_test.go`, `hierarchy_internal_test.go`, `hierarchy_concurrent_internal_test.go` (the race of a factory held while the root gate closes), `termination_internal_test.go`, `example_test.go`.
New tests (2): `spawn_supervised_internal_test.go`, `spawn_supervised_goside_internal_test.go`.
Everything else is denied by the rendered permissions, including `hierarchy.go`, `system.go`, `watch.go`, `supervision_failure.go`, `supervision_hooks.go`, `doc.go`, every other test, `judge/`, `docs/`, READMEs, ADRs and workflows. This differs from the RULEBOOK's estimate of "about 16 files under `pekko-actor/` and `docs/`" in one way only: the documentation rows are done by the coordinator, as in U3, and not by the implementer.

## 8. Permission control (no probe in this run)
No Haiku probe. The permissions stay restrictive (the allow list above, everything else denied; the non-file rules are identical to U3's). After the run the coordinator checks the modified files with `git diff --stat` and `shasum` against the baseline. **This is a post-hoc scope control, not a test of isolation**: it shows what changed, not that the permissions would have blocked a write outside the scope. The record says so.

## 9. Module-close backlog (not run in U4)
Mutants for U4, to be written by the unit's record and run on immutable snapshots at the module close; candidates: `Spawn` calls the factory again; `Spawn` returns `ErrNilActor` or `ErrFactoryPanic`; the name is released at the failure and not at the termination; a failed first factory gets a `PostStop` or a `PreRestart`; a Restart after a factory failure loses a queued message; the Goexit of the factory of a restart stops the cell; a stop accepted before the first activation skips the factory. Also the U3 backlog: mutants M1-M15, the parity judge campaign and its tag, the complete module `go test -race`, stress runs, the integral review, the mirror of S13, S14 and S16 (U8, #175), the Pekko surface inventory, the carried gaps of U2 (#173, K27, K28, K33, the n3 exception) and the informative findings of the U3 native review. A mutant that does not compile, times out, or is a harness error is INVALID, not a detection.

## 10. Exact execution command (after the Owner's final approval)
Run from the root of the worktree, on branch `feat/actor-supervision-u4`. `D` is any directory **outside** the repository (for example `$CLAUDE_JOB_DIR/tmp`); the two hash files are written from the **approved** values of section 11, never from the files on disk, and the launcher refuses to run if the files differ.

```
D=<directory outside the repository>
printf '%s  RULEBOOK.md\n' 2e847c2ef21d4ba21bcd4c5efebc0c892950c2eecfebe56f6ce408b6fb6832ff > "$D/rb.sha"
printf '%s  settings.template.json\n' 08731a31446ba64822e24063fd1e60a893f77011be90ce5d21b5d86763dda9fe > "$D/st.sha"
AGENT_WALL_MIN=30 bash migration/stage-1/supervision-u4/run-agent.sh sonnet migration/stage-1/supervision-u4/implementer-prompt.md "$D/implementer.json" "$D/rb.sha" "$D/st.sha"
```

The launcher checks the RULEBOOK and the permission template against those hashes, renders the permissions for this worktree, bounds the wall clock, caps the spend at 6 USD per agent (`AGENT_BUDGET_USD`), refuses to overwrite an existing output and keeps a failed output as `<out>.failed.<N>`. The coordinator reads the tokens from `$D/implementer.json` after the run.

## 11. Hashes for approval
Filled by the coordinator when presenting the package; the files are not edited after. The hash of this RUNBOOK cannot be inside it: it is given with the package in the PR and in `RESUME.md`.

| File | sha256 |
|---|---|
| `RULEBOOK.md` (approved design, `32b9d6a`) | `2e847c2ef21d4ba21bcd4c5efebc0c892950c2eecfebe56f6ce408b6fb6832ff` |
| `settings.template.json` | `08731a31446ba64822e24063fd1e60a893f77011be90ce5d21b5d86763dda9fe` |
| `run-agent.sh` | `91d1fb8fe7438bb79ef4cb90de1cdd9d59f0ef8cd57e07048858cecdaee69b37` |
| `manifest.tsv` | `16c5d3642997ebbc4ff3f3db55555ff6a226398da4377ce884c91ba5720c5791` |
| `implementer-prompt.md` | `c67fc998b214e4da9be7cbc8ee726a4280a1527947e944e02353a09751ae40e0` |
