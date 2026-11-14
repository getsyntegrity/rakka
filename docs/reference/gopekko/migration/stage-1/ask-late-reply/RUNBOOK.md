# RUNBOOK: Ask late reply (#163)

Status: **prepared, not executed.** Gate: the loop does not start until the Owner approves the RULEBOOK hash and section 2 (kit rule 3). Base: `develop` `7be6da7`. Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`.

## 1. Reuse of the approved #162 mechanics (no change)

| Artifact (already in `develop`, directory `migration/stage-1/ask-first-reply-v2/`) | sha256 | Use |
|---|---|---|
| `run-agent.sh` | `39611d651c66a54337b1ba6eb14c1e906089e101e0eb29e7487ebbaa3f48eead` | launched from that directory; its root is the worktree root |
| `settings.template.json` | `8ffce17a829f86a26dfb86b3c7b5895d80d160ec051152a74e8ce344901f665e` | permissions rendered per launch: write only `ask.go` and `ask_firstreply_internal_test.go`; `go test` denied; exact `gofmt`, `go vet`, `go build` forms |

No new permission and no new launcher: the unit extends the second approved file instead of creating a third, so the settings, the launcher and their probes are unchanged. If a separate test file is preferred, that is a one-line settings change that needs its own approval. Known follow-ups carried over, not in this unit: `run-agent.sh` does not escape the root when rendering (review R3-003), and a call to `ask` without an upper bound in an old test (R3-001).

The queue (`queue_runner.mjs`) is not usable here: the target already exists, so it would report done before the work. Progress is the presence of scenarios L1-L6 in the file (Grep by test name) and the gate.

## 2. Model plan, limits, token definition (PROPOSED)

| Role | Model | Count |
|---|---|---|
| Architect/TL: brief design review of the RULEBOOK scenarios before writing | `opus` | 1 |
| Implementer (scenarios L1-L6) and fixer (confirmed findings only) | `sonnet` | 1 + 0-1 |
| Final diff review as architect/TL and principal reviewer (contract, upstream source, diff only) | `opus` | 1 |
| Second reviewer, read-only | `sonnet` | 1 |
| Third reviewer, concrete disagreement only | `sonnet` | 0-1 |

Limits: <= 6 agents, <= 60 min wall, US$ 6 per agent as a cap, subscription only, stop if a step would need extra payment. **Token limit 600k is counted as input + output + cache creation per agent JSON; cache reads are reported separately and not counted.** This resolves the ambiguity found in #162 (202k by this definition, about 1.03 M including cache reads); the Owner approves this definition here. Models and costs are self-reported by the CLI JSON. Reviewer read-only access is controlled by the prompt and by `shasum` of the target before and after each reviewer; the settings do not enforce it.

## 3. Procedure after approval

1. Record `shasum -a 256` of `ask.go`, `ask_endpoint.go`, `errors.go`, the test file.
2. Architect/TL (Opus) reviews the RULEBOOK scenarios against ADR D4/D8 and the upstream lines (read-only, brief).
3. Implementer (Sonnet) extends the test file; `gofmt`, `go vet`, `go build` in the loop; `go test` is run by the orchestrator.
4. Orchestrator gate: `go test -race` including `-count` stress on the new scenarios, plus mutations of `ask_endpoint.go` that the scenarios must catch, restoring byte-identical after each: (a) a late reply overwrites the stored reply, (b) a late reply is not published, (c) a late reply is published twice.
5. Final review (Opus) and second review (Sonnet) in parallel; disagreements settled by a cited source line or a measurement before spending a third reviewer; fixer on confirmed findings only.
6. Gates: `gofmt`, `go vet`, `go mod tidy -diff`, `go test -race` in the six modules, `golangci-lint` v2.14.0 (pinned), `check_coverage.sh` over all six profiles and its selftest, `check_deps.sh`, `check-headers.sh`, `check-adr-signatures.sh`, `check-judge-matrix.py`, `modules_selftest.sh`, `go-sdk-update.sh --check`. The judge campaign is renewed only if its inputs change, proven by hash equality.
7. `EXECUTION-RECORD.md` (single record: authorship, models, costs by the definition above, reviews, gates, limits, the unreviewed deltas if any), cost entry, PR against `develop`.
