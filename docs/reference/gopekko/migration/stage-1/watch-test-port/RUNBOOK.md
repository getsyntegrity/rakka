# Runbook: porting DeathWatchSpec through the migration kit

Status: **prepared, not run.** Reconstructed from the kit (`prompts/04-translation-kickoff.md`, `templates/settings.README.md`,
`scripts/queue_runner.mjs`, `CLAUDE.md`) and the pilot's `permissions.md` (commit `32237c5`). Where the kit and this project disagree, the deviation
is in `RULEBOOK.md` section 7. The earlier session's RUNBOOK is lost and is not an authority.

## 1. Who does what

| Actor | May | May not |
|---|---|---|
| Loop agents (implementer, two reviewers, third ruler, fixer), all on `claude-sonnet-5-5` | read the rulebook, the spec and the worktree; edit **only** `pekko-actor/actor/watch_test.go`; `gofmt`, `go vet`, `go build`, read-only git | `go test`, `sbt`, any writing git command, editing the rulebook, creating other files |
| Orchestrating session (this one) | read the queue (`queue_runner.mjs`), launch the agents with the model set explicitly, collect findings, append to `migration/cost-log.tsv` | install `.claude/settings.json`, edit the rulebook, write the target by hand |
| Owner | install and remove `.claude/settings.json`; confirm the model plan; approve the rulebook | |
| Gate (after the settings file is removed) | this session runs `gofmt`, `go vet`, `go test -race`, the judge and the matrix; it commits | |

## 2. Preconditions, all the owner's

1. **`RULEBOOK.md` approved** by the owner on 2026-10-10 at `89f6abb` (this scope only).
2. **Model plan, confirmed by the owner**: one model for all agents, selector `model: "sonnet"`, explicit on every agent call. A probe agent launched with that selector reported `model_id=claude-sonnet-5-5`: a self-report, not an
   external verification. The kit calls an untriggered default a deviation; this one is declared, not inherited.
3. **Install the permissions.** The session's primary directory is the main checkout, and Claude Code reads project settings from there, so the file goes there and **binds this whole session** while it exists
   (it denies `go test`, `git commit` and `git push`):

   ```
   cp /Users/pablogore/workspace/pablogore/goPekko-worktrees/local-watch/migration/stage-1/watch-test-port/settings.watchport.json \
      /Users/pablogore/workspace/pablogore/goPekko/.claude/settings.json
   ```

   `.claude/` already exists there and holds other things: add one file, replace nothing. Its content is `migration/settings.go.json` (deny list identical) plus `gofmt`, `eza`, and read/edit of the worktree and of the single target
   file. It must not be committed.
4. After the loop, **remove** `.claude/settings.json` (`rm /Users/pablogore/workspace/pablogore/goPekko/.claude/settings.json`), because the gate needs `go test` and the commit.

## 3. Procedure (the session, with the file installed)

1. `node migration-kit/scripts/queue_runner.mjs --manifest migration/stage-1/watch-test-port/manifest.tsv status` from the worktree root: `0/1 done`.
2. Verify `.claude/settings.json` exists and has the denies; halt if it does not (kit prompt 04: never install, never overwrite).
3. Implementer (`model: "sonnet"`, no isolation, absolute paths): read `RULEBOOK.md`, the spec, `watch.go`, `watch_api_test.go`, `helpers_test.go`; write `watch_test.go`; gofmt and vet; status trailer.
4. Two adversarial reviewers (separate contexts, read-only, `model: "sonnet"`): assume it is wrong; each finding cites a rule or a spec line.
5. Third ruler only on disagreement, default not-confirmed. Fixer (`model: "sonnet"`) applies confirmed findings only; what it cannot fix it flags.
6. Progress invariant: the target must exist and grow; three minutes without growth is a stall to heal and report.
7. `queue_runner.mjs ... status` and `verify` (`1/1 done`, non-empty). Append one tab-separated line to `migration/cost-log.tsv`: `step	timestamp	wall_clock_min	tokens	subagents	model`.

## 4. The gate (the owner has removed the file)

`gofmt -l`, `go vet ./...`, `go test -race -count=1 ./...` in `pekko-actor/`, repeated runs of the new tests, then the matrix and selftests. A failing ported test is classified before it is touched: if Pekko's
behavior is the reference and the Go runtime disagrees, it is a runtime finding, not a test to weaken. Corrections to the runtime made by hand are listed and justified in the PR.

## 5. What is not recovered

The earlier session's ADR 0013 (A1 to A12), RULEBOOK, RUNBOOK, manifest, `settings.deathwatch.json` and `order.txt`. Only what is in this directory is used.
