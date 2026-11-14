# Runbook: public ports of #1475 through the migration kit

Status: **prepared, not run.** Modeled on `migration/stage-1/watch-test-port/RUNBOOK.md`. Preconditions are the Owner's; no loop agent runs until they are met.

## 1. Who does what

| Actor | May | May not |
|---|---|---|
| Loop agents (implementer, two reviewers, third ruler, fixer), implementer and fixer `model: "sonnet"`, reviewers and third ruler `model: "opus"`, explicit | read the rulebook, the specs and the worktree; edit **only** the three target files; `gofmt`, `go vet`, `go build`, read-only git | `go test`, `sbt`, any writing git command, editing the rulebook, creating other files |
| Orchestrating session | read the queue, launch the agents, collect findings, append to `migration/cost-log.tsv` | install `.claude/settings.json`, edit the rulebook, write the targets by hand |
| Owner | approve the rulebook, confirm the model plan, install and remove `.claude/settings.json` | |
| Gate (settings removed) | the session runs `gofmt`, `go vet`, `go test -race`, the mutations, then commits | |

## 2. Preconditions, all the Owner's

1. **`RULEBOOK.md` approved** (this PR is the proposal).
2. **Model plan confirmed** (sonnet for implementer and fixer, opus for the two reviewers and the third ruler, explicit on every call; Owner, 2026-11-07).
3. **Install the permissions** (binds the whole session while it exists; deny list of `migration/settings.go.json` plus the allows of `settings.publicports.json`; never committed):
   `cp migration/stage-1/public-ports-1475/settings.publicports.json <main checkout>/.claude/settings.json`
4. After the loop, remove it: `rm <main checkout>/.claude/settings.json`.

## 3. Procedure (the session, with the file installed)

1. `node migration-kit/scripts/queue_runner.mjs --manifest migration/stage-1/public-ports-1475/manifest.tsv status`: `0/3 done`.
2. Verify `.claude/settings.json` exists with the denies; halt if not.
3. Implementer per target: read the rulebook, the spec lines, the public API and the existing helpers; write the target; gofmt and vet; status trailer.
4. Two adversarial reviewers per target (separate contexts, read-only): assume it is wrong; each finding cites a rule or a spec line.
5. Third ruler only on disagreement (default not-confirmed). Fixer applies confirmed findings only.
6. Progress invariant: the target must exist and grow; three minutes without growth is a stall.
7. `queue_runner.mjs ... status` and `verify` (`3/3 done`). Append one line to `migration/cost-log.tsv`.

## 4. The gate (settings removed)

`gofmt -l`, `go vet ./...`, `go test -race -count=1 ./...` in `pekko-actor/`, repeated runs, then per test the named mutation of the scope (P1: no dead letter or two; P3: GMoverlap-like double Receive; P4: restart notifies, GMrestartnotify, or double
final notify) reproduced on a copy: the test must fail by an assertion. A failing port is classified with `gopekko-failure-triage` before anything is touched. The execution record (`EXECUTION.md`, this directory) lists commands, results, mutations and PORT-GAPs.
