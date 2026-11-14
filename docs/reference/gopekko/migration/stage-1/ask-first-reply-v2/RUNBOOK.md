# RUNBOOK: Ask first reply v2 (#162)

Status: **prepared, not executed.** Gate: the loop does not start until the Owner approves section 5 (kit rule 3). Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, base `origin/develop` `acbce17`.

## 1. Package

| File | Purpose |
|---|---|
| `manifest.tsv` | queue: 2 source -> target rows |
| `RULEBOOK.md` | read-only during the loop |
| `settings.template.json` | agent permissions with a `__ROOT__` placeholder; not a Claude settings file by itself |
| `run-agent.sh` | renders the template with this worktree's root into a temp file and launches one real `claude -p` process rooted here; the temp file is deleted on exit |

## 2. Change versus previous units (needs explicit approval)

| Aspect | `ask-endpoint-port` (#160/#161) and closed #429 | This unit |
|---|---|---|
| Runtime (`ask.go`) | written manually, outside the kit | **kit implementer writes it** (target 1) |
| Test | kit ported one test (`PromiseRefSpec` or `AskSpec:34`) | kit writes the full scenario file (target 2) |
| Edit surface | one file | exactly two files (`Write` and `Edit` on each) |
| Permission delivery | Owner copies `.claude/settings.json` into the worktree | `--settings` + `--setting-sources ""` per launched process; no file is installed under `.claude/`; the committed template is rendered to a temp file per launch |
| `go test` in loop | denied | **still denied**; orchestrator runs it at the gate and feeds failures back as confirmed fixer input |
| Manual authorship | runtime and fixes | only the orchestrator's integration: gate runs, commits, docs, PR. Any manual code fix is recorded as a deviation |

## 3. Model plan and limits (APPROVED by the Owner at `4b7d5a2`; roles updated 2026-10-14, see section 7)

| Role | Model | Count |
|---|---|---|
| Implementer (runtime, ported tests) | `sonnet` | 1 |
| Architect/TL: brief initial design review, then final diff review (contract, upstream source, relevant diff only; no mechanical work) | `opus` | 1 |
| Principal reviewer (this is the Architect/TL pass above; not a separate agent) | `opus` | 0 |
| Second reviewer (separate context, read-only) | `sonnet` | 1 |
| Third reviewer (concrete disagreement only, default not-confirmed) | `sonnet` | 0-1 |
| Fixer (confirmed findings only) | `sonnet` | 0-1 |

Limits (approved): <= 6 agents, <= 600k tokens, <= 60 min wall, `AGENT_BUDGET_USD` 6 per agent as a cap, not an authorization of extra charges: the available subscription is used and the loop stops if a step would require extra payment. Reviewer read-only access is controlled by the prompt and by hash comparison; the settings do not enforce it. Models and costs are self-reported by the CLI JSON (`total_cost_usd`, `usage`), not externally verified. Reviewers share the same permissions; their read-only duty is a prompt rule. Targets are new, untracked files, so `git diff` cannot see them: the orchestrator records `shasum -a 256` of both targets before each reviewer and compares after it. Any change is a process violation to log. Only the per-agent USD budget is enforced by the launcher; the token, agent-count and wall-clock limits are tracked by the orchestrator in `entries/cost-log.entry.tsv`.

## 4. Verified permission behavior (probes, 2026-10-14, model `haiku`)

Real `claude -p` processes launched by `run-agent.sh` from this worktree (rendered template, `--setting-sources ""`). Probe 3 ($0.0050), raw tool results:

| Attempt | Result |
|---|---|
| `git diff --output=<file>`, `go build -o <path>`, `gofmt -w` on a non-target file, `fd --exec`, `Write` to a non-target file | DENIED |
| `Write` to `ask.go` | allowed (file created, then removed) |
| `git diff --stat`, `go vet ./pekko-actor/...`, `gofmt -l` | not in the denial list of the run (inferred allowed; the agent's own report was garbled) |

Denials are confirmed by the run's `permission_denials` list and by the absence of the `/tmp` marker files and of `errors_probe.go`.

Earlier probes found two defects in the first template, fixed before this version: `Write` rules rendered with a single leading `/` (all target writes were denied) and `--output:*` deny patterns that did not match `--output=<file>` (it created a file in `/tmp`, removed). Probe stubs (`ask.go`, `/tmp` markers) were removed. Allowed but unprobed: `rg`, `fd`, `bat` and `go build -o` variants are not in the allow list; reading goes through the built-in `Read`, `Grep` and `Glob` tools. The session that prepares this package is not restricted by this file; only launched agents are.

## 5. Owner approvals required (concrete artifacts)

1. `RULEBOOK.md` at the commit that contains it (name the SHA).
2. Model plan and limits (section 3).
3. `settings.json` (section 4 evidence) as the agents' permissions.
4. Authorship change (section 2), recorded afterwards as DEV-1.

## 6. Procedure after approval

1. `node migration-kit/scripts/queue_runner.mjs --manifest migration/stage-1/ask-first-reply-v2/manifest.tsv status` -> expect `0/2 done`.
2. Implementer via `run-agent.sh sonnet <prompt> <out.json>` (prompt: RULEBOOK, `ask_endpoint.go`, helpers, ADR D4/D8; writes the two targets; `gofmt`, `go vet`, `go build`).
3. Two reviewers, then third only on disagreement, then fixer on confirmed findings. Each finding cites a rule or source line.
4. Gate (orchestrator): `gofmt -l .`, `go vet ./...`, `go mod tidy -diff`, `go test -race -count=1 -covermode=atomic ./...`, `golangci-lint` (pinned version) in `pekko-actor/`; at root `scripts/check_coverage.sh`, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `python3 -I scripts/check-judge-matrix.py`, `scripts/check-adr-signatures.sh`. Judge/mutation inputs: renew only if `go-scope.json` inputs change, proven by hash equality; otherwise record "not renewed".
5. Cost row per step in `entries/cost-log.entry.tsv`; honest record in `EXECUTION-RECORD.md`.
6. Delta review: candidate once, then only fix deltas.

## 7. Change log of approved artifacts

| Date | Change | Authorized by | Previous hash | Notes |
|---|---|---|---|---|
| 2026-10-14 | Package approved at `4b7d5a2` | Owner | n/a | RULEBOOK `f0e8f327541594245bb7f6f835a27a1d45e3375c60031c943d0d1990f2f178d3`, RUNBOOK `a1e6b8575d907492f6c5eafa71a3c5edd6da9162422219419820ffc79630c0c5`, settings.template.json `6a560e1050961e318a415c5957672a2e864a68e79d5bb581f0572ee17748f0f1`, manifest.tsv and run-agent.sh unchanged by the entries below |
| 2026-10-14 | `settings.template.json`: `gofmt -l:*` and `gofmt -d:*` replaced by the exact forms `gofmt -l pekko-actor/actor`, `gofmt -d <target>` (x2). Reason: a prefix rule let the flags be combined with `-w` on any file; checking afterwards detects the write but does not prevent it (review finding R1-001) | Owner, bounded correction; no re-approval of the rest | `6a560e1050961e318a415c5957672a2e864a68e79d5bb581f0572ee17748f0f1` | new hash in the commit message |
| 2026-10-14 | Section 3 roles: Sonnet implements, ports tests and fixes; Opus acts as Architect/TL and principal reviewer; second reviewer Sonnet. Total limits unchanged | Owner | RUNBOOK `a1e6b857...` | An implementer launched on `opus` by mistake was stopped after about two minutes; its partial `ask.go` (1.8 KB) was archived outside the repository and not reused. Its cost is not in the JSON (the process was killed) and is recorded as unknown |
