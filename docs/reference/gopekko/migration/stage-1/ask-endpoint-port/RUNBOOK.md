# RUNBOOK: Ask reply endpoint unit

Status: **executed once, 2026-10-12T20:44Z to 20:47Z** (results in `EXECUTION-RECORD.md`); the Owner approvals A, B, C, D1 are recorded in its section 4. Modeled on `../watch-test-port/RUNBOOK.md`. Mechanics come from the kit: `migration-kit/CLAUDE.md` (standing rules), `prompts/04-translation-kickoff.md`, `templates/settings.README.md`, `scripts/queue_runner.mjs`. The kit has no executable orchestrator: the loop is a Claude session that launches subagents with the model set explicitly.

## 1. Actors and permissions

| Actor | May | May not |
|---|---|---|
| Owner (human) | approve `RULEBOOK.md`, decide the model plan, **install and remove** `.claude/settings.json` | |
| Orchestrating session | read the queue, launch agents with `model` set on every call, collect findings, write the cost entry, run the gate and commit **after** the Owner removes the settings file | install `.claude/settings.json` (prompt 04: "never install it yourself"), edit the RULEBOOK, write the target by hand |
| Loop agents: implementer, two reviewers, third reviewer, fixer | per `settings.askport.json`: `Read` of this worktree; `Edit` of **only** `pekko-actor/actor/ask_endpoint_promiseref_internal_test.go`; read-only git, `rg`, `wc`, `gofmt`, `go vet`, `go build`, `eza` | `go test`, `sbt`, any writing git command (`rebase reset checkout stash push commit merge switch restore clean`), editing the RULEBOOK or any other file |

## 2. Authorizations that are missing (all Owner acts; none is recorded)

| # | Act | Applies to | Exact form |
|---|---|---|---|
| 1 | Approve `RULEBOOK.md` | the file at its commit in this branch | a written approval naming the commit, as the Watch rulebook records ("approved by the owner on <date> at <commit>") |
| 2 | Decide and confirm the model plan | section 7 of the RULEBOOK | written; "inheriting the session default is a process violation" (kit `CLAUDE.md`) |
| 3 | Install the permissions | `<worktree>/.claude/settings.json` | `cp migration/stage-1/ask-endpoint-port/settings.askport.json <worktree>/.claude/settings.json` (never committed); remove it after the loop |
| 4 | Decide the status of the manual authorship | `ask_endpoint.go`, `errors.go`, `ask_endpoint_internal_test.go` (candidate tree `f1a190723d323d7d885381d0457e4cc1e84a58d5` of `pekko-actor`) | either an explicit, bounded exception recorded as DEV-n in RULEBOOK section 8, or no exception (then the PR states the code is manual and outside the kit) |

Without 1, 2 and 3 the loop must not start (kit rule 3 and prompt 04). Without 4 nothing is assumed: the provenance stays "manual, outside the kit".

## 3. Procedure (the session, with the settings file installed)

1. From the worktree root: `node migration-kit/scripts/queue_runner.mjs --manifest migration/stage-1/ask-endpoint-port/manifest.tsv status` -> expect `0/1 done`. (Observed on this branch before execution: `0/1 done (0.0%)`.)
2. Verify `.claude/settings.json` exists and carries the denies; if it does not, STOP and wait for the Owner (prompt 04).
3. Implementer (`model` explicit): reads `RULEBOOK.md`, `SCOPE.md`, the spec, `ask_endpoint.go`, `ask_endpoint_internal_test.go`, `clock_internal_test.go`, `helpers_test.go`; writes the target; `gofmt` and `go vet`; status trailer.
4. Two adversarial reviewers (separate contexts, read-only, `model` explicit), a third only on disagreement (default: not confirmed), then the fixer on confirmed findings only.
5. Progress invariant (prompt 04): the target must exist and grow between polls; three minutes without growth is a stall to heal and report.
6. `node migration-kit/scripts/queue_runner.mjs --manifest ... status` expects `1/1 done`; `... verify` exits 1 only if a done target has zero bytes (observed semantics of `queue_runner.mjs`).
7. Write one tab-separated row per step to `entries/cost-log.entry.tsv` (header `step	timestamp	wall_clock_min	tokens	subagents	model`, `unknown` where not measured; the coordinator merges it into `migration/cost-log.tsv`).
8. Fill `EXECUTION-RECORD.md` section 2 with what each role actually did.

## 4. Gate (after the Owner removes the settings file)

In `pekko-actor/`: `gofmt -l .`, `go vet ./...`, `go mod tidy -diff`, `go test -race -count=1 -covermode=atomic ./...`, `golangci-lint run --config ../.golangci.yml ./...` at the version in `.golangci-version`. At the root: `scripts/check_coverage.sh`, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `python3 -I scripts/check-judge-matrix.py`. If the target changes the `pekko-actor` tree, renew the Go campaign (`judge/mutate_go.py --runs 30`). Record the observed results and the SHA they ran on. A failing ported test is classified before it is touched.

## 5. Provenance rule

The model id in the cost log is self-reported, not external verification. Results of the session's manual work are not kit results and are never presented as such (`EXECUTION-RECORD.md`, `AMENDMENTS.md` A-1).
