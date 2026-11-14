# RUNBOOK: Supervision U1, the Failed channel, suspension and OneForOne Stop/Resume/Escalate (#168, #446)

Status: **prepared, not executed.** The loop does not start until the Owner approves the hashes in section 8 (kit rule 3). Base: `develop` at the commit recorded in `EXECUTION-RECORD.md` when the loop starts. Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, ADR 0017 (ACCEPTED 2026-10-14).

## 1. What changes versus the previous unit (#166, ask-terminated; needs explicit approval)

| Aspect | #166 | U1 |
|---|---|---|
| Behavior | an Ask on a terminated target fails with a sentinel | a child that fails under a parent that **declares** a strategy is suspended (recursively) and reported to the parent, which decides Stop, Resume or Escalate; a parent that declares none keeps today's behavior (the child stops directly) |
| Public surface | one exported identifier | new exports: `Directive` (`Resume`, `Stop`, `Escalate`), `Decider[T]`, `Strategy[T]`, `OneForOne`, `Budget`, `Supervisor[T]`, `PanicError`, `ErrInvalidDirective`, the decider helpers (names PROPOSED, RULEBOOK D8) |
| Writable files | 4 | **12**: six existing production files (`cell.go`, `stop.go`, `mailbox.go`, `hierarchy.go`, `actor.go`, `errors.go`), three new production files (`supervision.go`, `supervision_failure.go`, `supervision_decider.go`), three new test files. **Every existing test stays read-only**: the interim keeps all existing event sequences |
| Upstream tests | one file, adapted | two files (`SupervisorHierarchySpec` x2 tests, `SupervisorMiscSpec` x1 test) plus a Go-side file with no Pekko source |
| Permissions | settings of #166 | new template with the 12 files; launcher identical in structure, with a guard that refuses to run without the RULEBOOK and settings-template hash files, a wall-clock bound and a retry-safe output |
| Judge | campaign renewed | see section 6: the next functional change in `pekko-actor` is when a renewal is due; U1 is that change |

## 2. Scope decisions to confirm

RULEBOOK section 1 lists D1-D12: the error name `ErrInvalidDirective`; only `Resume`/`Stop`/`Escalate` delivered and value 2 invalid; causes (returned errors as is, `*PanicError`, no `ChildFailure`/`InitializationError`); `Budget` exported and unused in U1; the interim for a parent without strategy and for init failures; the parent that cannot take the `Failed`; `makeDecider` limited to the pair form and the adapter (trapExit form moves to U2) with PROPOSED names; the counted, recursive suspension model; no new event kinds; the unknown state of `ContextMisuse` and how it is decided; the test classification. Portable, adapted and moved assertions with upstream lines are in RULEBOOK section 5.

## 3. Model plan, limits, token definition (PROPOSED; same shape as the approved #166 plan)

| Role | Model | Count |
|---|---|---|
| Architect/TL: design review of the RULEBOOK and the behavior specification before any code | `opus` | 1 |
| Implementer (runtime, tests) and fixer (confirmed findings only) | `sonnet` | 1 + 0-1 |
| Architect/TL final review and principal reviewer (with an exact FIXSPEC when it finds something) | `opus` | 1 |
| Second reviewer, read-only | `sonnet` | 1 |
| Review of the fix delta | `opus` | 1 |

Limits (PROPOSED): <= 6 loop agents plus one `haiku` agent for the permission probe, <= 60 min wall, US$ 6 per agent as a cap, subscription only, stop if a step would need extra payment. Tokens (limit 600k) are input + output + cache creation per agent JSON, **the Haiku probe included**; cache reads are reported apart. Models and costs are self-reported by the CLI JSON. Reviewer read-only access is controlled by the prompt and by `shasum` of the writable files before and after each reviewer; the settings do not enforce it. U1 touches more files than #166 (12 against 4, 487k tokens were used by a 6-file unit in #165), so the limit may bind: if a step would pass it, stop and report; the Owner decides whether to raise it. The implementer prompt may be split in two (runtime, then tests) only with the Owner's word. The 600k token cap is not enforced mechanically by `run-agent.sh`: the orchestrator checks it from the JSON after each agent (the wall-clock bound is mechanical through `AGENT_WALL_MIN`, default 60, and the USD cap is passed to the CLI).

## 4. Lessons carried in (from #164, #165, #166)

- The design review (Opus) happens before any code and its guidance goes to the implementer prompt.
- The implementer prompt states the header gate (a `// Ported from` line must end in `.scala`), the shell rule, the staticcheck SA1032 rule, and requires calls that should return immediately to run through a bounded helper too (T6 of #164 hung the binary).
- Mutations are written against the real code after the implementation; each surviving mutant is reported with the test that closes it. For this unit the list includes: Resume also resuming siblings; Resume not resuming grandchildren; a counter that goes below zero; suspension not recursive; the failing child skipped in the wrong case on Escalate; Stop not applied; `ok == false` not escalating; a zero or value-2 directive accepted; a decider panic not failing the parent; a stale `Failed` calling the decider; a `Failed` decided before the parent is resumed; the `Failed` item placed in the runtime lane; the interim changed (a parent without strategy suspending); a double suspension of the failing child on the decider-panic or invalid-directive path (V9/V10 must kill it: after the grandparent's Resume the child's counter is 0 and it processes a message).
- Measured results are kept as measured: detected over total, with the undetected ones named.
- Evidence reuse is claimed only after comparing the inputs of the **whole** tree the gate reads (including the README), not only `pekko-actor/actor`.
- Honest accounting: orchestrator edits are recorded as manual; comments-only fixes need no fixer; a PORT-GAP is recorded, never silently dropped.

## 5. Procedure after approval

1. Before the probe, record `shasum -a 256` of ALL writable existing files (the six existing production files) and of every existing test file, and confirm `git status --porcelain` is clean for them. Then probe the new settings with a `haiku` agent through `run-agent.sh` without modifying any existing file: the probe reads the rendered permission rules (read-only), attempts writes only to the NEW files listed as writable (`supervision.go`, `supervision_failure.go`, `supervision_decider.go` and the three new tests), removing them afterwards, and attempts writes to `watch.go`, `spawn.go`, `clock.go`, an existing test file and `judge/probe_stub.py` (expected to be refused, so nothing is written), plus `gofmt -w` on a non-target and `go test` (both expected denied). It never writes the six existing writable production files; their allow rules are checked only by reading the rendered rules. The same step reports (read-only) whether `ContextMisuse` exists in `pekko-actor` non-test code (RULEBOOK D11); if it does not, the Owner decides before the loop.
2. After the probe, compare against the baseline of step 1: the recorded `shasum` values must be unchanged and `git diff --stat` must be empty for the existing files. Restore mechanism: `git checkout -- <file>` for any existing file that differs (restore-by-git), then repeat the check; the step 1 baseline, not a post-probe measurement, is the starting state. New files must be absent after the probe.
3. Opus design review (read-only); a BLOCKER goes to the Owner.
4. Sonnet implements section 2 of the RULEBOOK and scenarios V1-V17; `gofmt`, `go vet`, `go build` in the loop; the orchestrator runs `go test`, including `-race` and a repeated `-race` run of the new tests.
5. Orchestrator gate and **isolated mutations** (copies of `pekko-actor`, `GOWORK=off`; the delivery tree is only read).
6. Opus final review and Sonnet second review in parallel, with the checks the Owner cares about: Resume touching only the failing child's subtree, counter balance, the Escalate skip rule, the position of the `Failed` item outside the runtime lane, the interim paths unchanged; disagreements settled by a cited line or a measurement; fixer on confirmed findings only; Opus reviews the fix delta.
7. Native review by sections, each requested **once**: if the diff exceeds the engine's lens budget (735 lines were accepted in one range in #165; 1,461 were refused in #164), review by work-unit commits (runtime types and decider; failure channel and suspension; tests; fix), each with its exact base.
8. The orchestrator updates the `pekko-actor` README (supervision section, limitations, interim), `docs/actor-core-parity.md` and the root README rows that mention #168/#446 (manual, recorded) after the loop.
9. Gates: `gofmt`, `go vet`, `go mod tidy -diff`, `go test -race` in the modules, `golangci-lint` (version in `.golangci-version`), `gocyclo` <= 10, coverage >= 85% per module, `check_deps.sh`, `check-headers.sh`, `check-adr-signatures.sh`, `check-judge-matrix.py`, `modules_selftest.sh`, `go-sdk-update.sh --check`. Local-only checks follow `docs/ci-economy.md`.
10. One `EXECUTION-RECORD.md` (commands, kit version, models, inputs, outputs, every manual change), one cost entry, and a PR against `develop`. No merge without the Owner's word.

## 6. Judge campaign and evidence tags (needs the Owner's decision)

The judge gate stays as accepted. ADR 0017 states that the Go campaign is renewed with the **next functional change in `pekko-actor`**, over its final code, and that its commit is preserved with an authorized `evidence/*` tag. U1 is the next functional change, so a renewal is due when U1's code is final. **This package does not renew it and does not publish any tag.** The Owner decides: authorize the renewal (`judge/mutate_go.py`) and one more `evidence/*` tag for the commit cited by the renewed report, or defer the renewal and accept the stale evidence. Until the Owner decides, nothing in `judge/` is touched. Also: the judge has **no U1 scenarios**; S13/S14 run on both sides only once U3 lands, S16 after U8 (ADR 0017), so U1 claims no Pekko parity beyond the three ported tests.

## 7. Known follow-ups (not in this unit)

Everything in RULEBOOK section 3 (U2-U9, #450); the `Failed` to dead letters question (D7); the `ContextMisuse` expiry if the Owner defers it (D11); the trapExit `makeDecider` (U2); carried over from earlier units: `run-agent.sh` does not escape the worktree root.

## 8. Hashes for approval

The orchestrator fills these placeholders when presenting the package; the files are not edited after.

| File | sha256 |
|---|---|
| `RULEBOOK.md` | `<sha256 of RULEBOOK.md>` (also passed to `run-agent.sh` as a one-line hash file) |
| `RUNBOOK.md` | in the approval request only (a hash of this file cannot be written inside it): `<sha256 of RUNBOOK.md>` |
| `manifest.tsv` | `<sha256 of manifest.tsv>` |
| `settings.template.json` | `<sha256 of settings.template.json>` (also passed to `run-agent.sh` as a one-line hash file) |
| `run-agent.sh` | `<sha256 of run-agent.sh>` (usage: `run-agent.sh <model> <prompt-file> <out-json> <rulebook-sha256-file> <settings-sha256-file>`; env `AGENT_BUDGET_USD`, `AGENT_WALL_MIN`) |
| `EXECUTION-RECORD.md` skeleton | `<sha256 of EXECUTION-RECORD.md>` |
