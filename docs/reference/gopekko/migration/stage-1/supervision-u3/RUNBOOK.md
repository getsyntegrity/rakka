# RUNBOOK: Supervision U3, default strategy and failure classification (#448)

Status: PROPOSED, **execution NOT authorized** (the Owner does not authorize running it with the current weekly quota). Unit U3 of ADR 0017, kit `cf91c9d`. Base: `develop` at `0bbc65d` (U2 merged). Delivery policy of 2026-10-15 applies (`docs/quality-gates.md`, "Per unit and at module close"). Procedures not mentioned here are those of `migration/stage-1/supervision-u2/RUNBOOK.md` as merged (immutable), with the lessons of `supervision-u2/EXECUTION-RECORD.md` sections 6, 12 and 13.

## 1. What this plan is

The reduced flow the Owner accepted as procedure on 2026-10-15. It reuses this package as is: nothing is mapped or planned again when work resumes (section 10).

## 2. Scope decisions

RULEBOOK D1 to D8 (approving the RULEBOOK hash approves them). AllForOne (#447), logging (#451) and user-defined strategies (#450) **stay inside the migration objective of `pekko-actor`**; they are outside U3 only. "Usable v0.1" may be partial; "module migrated" needs an inventory of the Pekko surface with every capability implemented, adapted or pending (a separate read-only task, priced when requested). Pending items do not disappear by being outside the MVP.

## 3. Flow, limits, token definition (PROPOSED)

| Role | Model | Count |
|---|---|---|
| Implementer: production and tests in **one session** | `sonnet` | 1 |
| Consultation on one concrete unresolved design decision (not a repeat of the design) | `opus` | 0-1 |
| Final review, focused on Goexit and routing | `opus` | 1 |
| Corrections of confirmed findings | the same implementer, resumed | 0-1 |
| Verification of the corrected findings only | the same reviewer | 0-1 |

At most 4 agents. No Haiku probe in this run (section 8). No second reviewer. The orchestrator runs the gates, the targeted `-race`, the commits, the documentation and the PR.

**Time: 60 minutes of active work in total, everything included** (implementation, consultation, review, corrections, verification, gates, commits, PR). Sub-limits inside that total, exported by the orchestrator as `AGENT_WALL_MIN` for each agent: implementer 30, review 12, corrections 6, verification 4; the rest is orchestration and closure. The default of `run-agent.sh` (45) is an upper bound, not a plan. No campaign and no tag fit in this run.

**Token budget: 560,000 is a PROPOSAL, pending, NOT an authorization to consume.** Evaluation stop 450,000. Estimate (agents measured from the CLI JSON as input + output + cache creation; coordination is an estimate without telemetry):

| Item | Expected, no findings | With findings and a consultation |
|---|---|---|
| Implementer (about 1,300 lines, one session) | 220,000 | 220,000 |
| Opus consultation | 0 | 30,000 |
| Opus final review | 90,000 | 90,000 |
| Corrections | 0 | 40,000 |
| Verification of the corrected findings | 0 | 30,000 |
| **Agents** | **310,000** | **410,000** |
| Coordination (gates, targeted race, commits, PR, docs, consents) | about 90,000 | about 90,000 |
| **Total** | **about 400,000** | **about 500,000** |

Margin for corrections about 60,000 (15% of the agents). **Forecast gate:** before each agent the orchestrator writes `observed + forecast`; if it passes the approved cap it stops and asks. A run killed by the wall bound normally leaves no output, so its whole forecast is counted as spent. Nothing here is consumed until the Owner authorizes a cap and says to resume.

## 4. Lessons carried in
- A reviewer that sees only part of a change reports missing symbols as blockers: review whole work-unit commits, never artificial halves.
- Tests prove "not processed" with an inline or controlled executor, never by reading a counter right after a `Tell`.
- The tool guard rejects compound commands and heredocs: scripts go to files and run with `python3 -I`.
- A Go timeout, a build failure or a harness error is never a detection (relevant at module close).

## 5. Procedure after the Owner resumes

1. **Baseline:** `git rev-parse HEAD:pekko-actor/actor`, `shasum -a 256` of every `.go` file in `pekko-actor/actor/`, `git status --porcelain` clean; write the approved hashes (section 11) to files outside the repository; `git submodule status` shows the kit at `cf91c9d`.
2. **Implementer** (`sonnet`, one session, through `run-agent.sh`, `AGENT_WALL_MIN=30`): production and tests per the RULEBOOK; normal tests during implementation; `gofmt`, `go vet`, `go build` in the loop. If one concrete design decision stays unresolved it asks the orchestrator, who may consult Opus on that decision only.
3. **Gates by the orchestrator:** `gofmt -l`, `go vet`, `go mod tidy -diff`, pinned `golangci-lint` (`.golangci-version`, `gocyclo` at most 10), coverage at least 85% (`scripts/check_coverage.sh`), `scripts/check_deps.sh`, headers, ADR signatures, modules selftest, `go-sdk-update.sh --check`, the normal suite `go test -count=1 ./...`, and the anchor check (every `(file, anchor)` of `judge/mutate_go.py` occurs once in the files it names). **If an anchor must change the work stops and reports; `judge/` is never edited.**
4. **One targeted `-race`** over the Goexit and continuation tests at the end of the change, `-count=1`. A repeat (`-count=20`) only if a concrete failure justifies it, and it is recorded.
5. **Final review** (`opus`, read-only, `AGENT_WALL_MIN=12`): Goexit and routing first, each finding citing a RULEBOOK rule or a source line. Corrections only for confirmed findings; the same reviewer verifies only the corrected findings.
6. **Scope control after the fact** (section 8): `git diff --stat` and `shasum` against the baseline; any file outside the 13 allowed surfaces is a defect.
7. **Native review** only if the Owner's switch asks for it and he consents per envelope: commits of at most about 700 changed lines each, never the accumulated branch; proportional, not an integral review. Commits are made by the orchestrator after `gofmt`, with Conventional Commit messages and no attribution lines.
8. **Documentation by the orchestrator** (manual, recorded): `pekko-actor/README.md`, `doc.go`, `docs/actor-core-parity.md`, root `README.md`. Gaps recorded as gaps; no parity claimed; CURRENT stays stale until the module closes and the text says so. The rows that call S13/S14 "out of scope" stay until U8.
9. **PR against `develop`**, CI `go` and `label`; `EXECUTION-RECORD.md` with commands, models, observed tokens, every manual change. Functional issues close when their implementation and test criteria are met, linking the pending exhaustive validation to the module milestone; an issue whose own criterion is equivalence stays open. No merge without the Owner's word.

## 6. What U3 enables for the judge scenarios S13 and S14 (no coverage claimed)

- **S13** (`w` watches `c`; `c` throws; then `c` receives `R`; expected `R` received by the second instance, `G6.d`): needs a parent that declares nothing to **restart** the child. U3 supplies it; Go tests T1 and T2 have the same shape.
- **S14** (`w` spawns a child whose first `PreStart` throws and watches it; expected `Terminated` at `w`, no second `pre-start`, `G7.b`): needs a parent that declares nothing to **stop** it. U3 supplies it; Go test T4 has the same shape.
- Today `judge_internal_test.go` has no `judgeS13`/`judgeS14`, and `docs/actor-core-parity.md:361` and `pekko-actor/README.md:302,341` still call them out of scope. **U3 does not run, mirror or claim them**; that belongs to U8 (#175) and to the module close.

## 7. Allowed edit surfaces (13 files, all under `pekko-actor/actor/`)

Production: `supervision.go`, `supervision_failure.go`, `supervision_decider.go`, `supervision_restart.go`, `cell.go`, `watch.go`, `errors.go`.
Tests: `supervision_default_internal_test.go` (new), `supervision_default_goside_internal_test.go` (new), and minimal edits to `supervision_goside_internal_test.go` (`TestSup1Interim`), `lifecycle_internal_test.go` (the `errGoexit` detail), `watch_internal_test.go`, `watch_test.go` (death pact).
Everything else is denied by the rendered permissions, including `stop.go`, `hierarchy.go`, `spawn.go`, `system.go`, `judge/`, `docs/`, READMEs and workflows.

## 8. Permission control (no probe in this run)

The Haiku probe is **not** run. The permissions stay restrictive (the allow list above, everything else denied). After the run the orchestrator checks the modified files with `git diff --stat` and `shasum` against the baseline. **This is a post-hoc scope control, not a test of isolation.** It shows what changed; it does not prove that the permissions would have blocked a write outside the scope. The record says so and never states that isolation was tested.

## 9. Module-close backlog (not run in U3)

Mutants M1 to M15 of the RULEBOOK (section 5.3) on immutable snapshots; the parity judge campaign and its exact `evidence/*` tag; the complete module `go test -race`; stress runs; the integral review; the mirror of S13, S14 and S16 in the Go gate (U8, #175); the inventory of the Pekko surface (implemented, adapted, pending) for "module migrated". Carried gaps of U2: #173 (`UidClashTest:104`, `Ticket669Spec:37`, `SupervisorHierarchySpec:767`), mutants K27, K28, K33, and the open n3 exception.

## 10. Resume point

When the Owner resumes: do **not** re-map or re-plan. Read this RUNBOOK and the RULEBOOK, confirm the weekly quota and an authorized cap, write the hashes of section 11 outside the repository, and start at section 5 step 1 from branch `docs/u3-package`. A short copy of this point is in `RESUME.md` (not part of the hashes).

## 11. Hashes for approval

Filled by the orchestrator when presenting the package; the files are not edited after.

| File | sha256 |
|---|---|
| `RULEBOOK.md` | `c2f0a3f4498bd3f1b5fb2f67686b2b7216645313df8db56c123aaeb84613b0aa` |
| `settings.template.json` | `1670e0c099fb07cf67dff1d6fe81f3d6feaf8283a9692b049698424ad52349c2` |
| `run-agent.sh` | `f7cec4a87ef684f5a5fb1cf5f907d13472b7a4a2d4a17f7c70a0eaf955587b3a` |
| `manifest.tsv` | `d588a8c3de3e783a796ae84a904f0e1aba62c1553d1dd09c12f3bb612a3ebe7e` |
