# RUNBOOK: Supervision U2, Restart, hooks, FailedMessage, InitializationError, conversions and the finite Budget (#169-#174)

Status: **prepared, not executed.** The loop does not start until the Owner approves the hashes in section 8 (kit rule 3). Base: `develop` at the commit recorded in `EXECUTION-RECORD.md` when the loop starts (it must contain U1, `6f5741c`). Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909` (read-only submodule), ADR 0017 (ACCEPTED 2026-10-14).

## 1. What changes versus the previous unit (U1, #168/#446; needs explicit approval)

| Aspect | U1 | U2 |
|---|---|---|
| Behavior | a child under a parent that **declares** a strategy is suspended and the parent decides Stop, Resume or Escalate | adds `Restart`: new instance from the retained factory, `PreRestart`/`PostRestart` with defaults, survivors restarted, mailbox and watches kept, `FailedMessage`, `InitializationError`, the `Resume` conversions, and the finite `Budget` with its explicit conversions. A `PreStart` failure under a declaring parent becomes supervised (supersedes U1 D6/V15) |
| Public surface | `Directive` (3 values), `Budget`, `OneForOne`, `Strategy`, `Supervisor`, `PanicError`, `ErrInvalidDirective`, `MakeDecider`, `DeciderFunc`, `CauseDirective` | adds `Restart`, `InitializationError`, `FailedMessage`, `ChildFailure`, `PreRestarter`, `PostRestarter`, `MakeRestartDecider` (name PROPOSED); no exported default-hook helper |
| Writable files | 12 | **15**: eight existing production files, three new production files, three new test files, and **two edit sites** in one existing U1 test file (D2, D6). Every other existing test stays read-only |
| Upstream tests | 3 | eleven Pekko files (RULEBOOK 5.1), 26 Go-side scenarios, 37 mutants |
| Judge | renewal due | the Go campaign is stale from the first `pekko-actor` change; it is **renewed once, at the end** (section 6) |
| Launcher | hardened `run-agent.sh` | the same file, renamed (`supervision-u2`), keeping a failed run's output and using `timeout -k 30s` |

## 2. Scope decisions to confirm

RULEBOOK section 1 lists D1-D22: names (D1); `Restart` delivered and `Directive(2)` no longer invalid (D2); `FailedMessage` content (D3); hooks and defaults, no helper to call them (D4); factory retention (D5); supervised first-start `PreStart` failure (D6); init-failed and fatal states (D7); children and order (D8); the `PostStop` guard and the S11 interim (D9); budget and clock seam (D10, D11); the trapExit decider name (D12); events (D13); interim paths (D14); the attribution layout of ported tests (D15); the four U1 precisions as **open** again (D16: transitional labelling, `errors.Is` only, `ContextMisuse` absent, Unknown stays Unknown); the test classification and mutants (D17); the soft-stop reading (D18); and four Go choices that follow resolved Pekko facts: recreate to a terminated child (D19), restart send order (D20), a custom `PreRestart` that calls `PostStop` (D21), the stale message after `Resume` (D22). Two further owner decisions live in this RUNBOOK: the number of implementer runs and the token cap (section 3), and the judge renewal and tag (section 6).

## 3. Model plan, limits, token definition (PROPOSED)

| Role | Model | Count |
|---|---|---|
| Permission probe (read-only rules, writes only to new files) | `haiku` | 1 |
| Architect/TL design review of the RULEBOOK before any code | `opus` | 1 |
| Implementer: **run A runtime** (production files), then **run B tests** (test files) | `sonnet` | 2 (or 1, Owner's choice) |
| Fixer (confirmed findings only) | `sonnet` | 0-1 |
| Final review and principal reviewer (exact FIXSPEC when it finds something) | `opus` | 1 |
| Second reviewer, read-only | `sonnet` | 1 |
| Review of the fix delta | `opus` | 0-1 |

**Owner decision: one or two implementer runs.** Recommended **two**: run A writes only the eleven production files and must leave `go build`, `go vet` and the existing U1 suite (orchestrator run) green; the orchestrator gates it (lint, anchors, race) before run B writes the three test files and the two U1 edit sites. Reason: U1's single run used 50 turns for about 1,526 authored lines; U2 is forecast at about 2,950, a single run risks the turn and wall bounds and hides a runtime defect until the tests exist. One run is allowed if the Owner prefers it; the forecast below then replaces A and B by a single 640k-token run.

**Limits (PROPOSED):** <= 9 loop agents; <= 120 min total wall (agent time was about 21 min in U1); each agent <= 60 min (`AGENT_WALL_MIN`, mechanical) and <= US$ 6 (`AGENT_BUDGET_USD`, passed to the CLI); subscription only, stop if a step would need extra payment. Tokens are input + output + cache creation per agent JSON, **the Haiku probe included**; cache reads are reported apart. Models and costs are self-reported by the CLI JSON. Reviewer read-only access is controlled by the prompt and by `shasum` of the writable files before and after each reviewer; the settings do not enforce it.

**Token cap: 1,400,000 (PROPOSED), planned evaluation stop at 1,120,000 (80%).** U1 used 624,746 against a proposed 600,000 (`supervision-u1/EXECUTION-RECORD.md` section 4) and the Owner had to accept the overage. How U2 is estimated from U1's measured per-role counts:

| Step | U1 measured | Basis for U2 | Forecast |
|---|---|---|---|
| Probe (haiku) | 46,569 | same shape | 50,000 |
| Design review (opus) | 101,512 | RULEBOOK is 46.7 KB against 24.8 KB (1.9x); the code read is similar | 170,000 |
| Implementer (sonnet) | 290,544 for about 1,526 authored lines (440 production, 1,086 test) = **190 tokens per line** | about 750 production lines (protocol 350, hooks 120, budget 150, edits 130) and about 2,200 test lines (restart 900, budget 650, Go-side 650): 2,950 lines x 190 = 560,000, plus 15% to read the context twice when split | run A 250,000; run B 400,000 |
| Final review (opus) | 99,914 | diff about 1.9x, reviewers read selectively | 150,000 |
| Second review (sonnet) | 86,207 | same | 120,000 |
| Fixer + fix-delta | not run in U1 | U1 needed 11 manual fixes; assume one fixer and one delta review | 120,000 + 60,000 |
| **Total** | **624,746** | | **1,320,000** |

The line counts are the author's forecast, not measurements; the 190 tokens per line is measured on U1 only. At the U1 rate (US$ 4.66 for 625k) 1.32M tokens is about **US$ 9.8**.

**The 1,120,000 stop is a planned evaluation point, not a failure.** The Owner approved the cap and a stop at 1,120,000 "para evaluar lo restante", counting both implementer runs and every other agent. The cumulative forecast crosses 1,120,000 after the two reviewers, before the fixer:

| After | Step forecast | Cumulative forecast | Against 1,120,000 |
|---|---|---|---|
| Probe | 50,000 | 50,000 | below |
| Design review | 170,000 | 220,000 | below |
| Run A | 250,000 | 470,000 | below |
| Run B | 400,000 | 870,000 | below |
| Both reviewers | 150,000 + 120,000 | 1,140,000 | **crossed: evaluation stop** |
| Fixer | 120,000 | 1,260,000 | above, within the cap |
| Fix-delta review | 60,000 | 1,320,000 | above, within the cap |

At the evaluation stop the orchestrator stops, writes observed against forecast in the record, and proceeds with the remaining mandatory steps (fixer, fix-delta review) only if `observed + their forecast <= 1,400,000`; otherwise it asks the Owner. Reading "evaluar lo restante" as permission to proceed autonomously when the projection stays within the cap is an **open Owner decision** (RULEBOOK D18).

**Forecast gate (mandatory):** before launching each agent the orchestrator writes `observed so far + forecast of this agent` in the record; if that sum passes the cap, it stops and asks the Owner; if it passes 1,120,000 for the first time, it is the evaluation stop above. Two agents are launched in parallel only if the sum of **both** forecasts passes the same test (U1 launched two reviewers when the total was already about 439k, which projected past the cap). If a step would pass the cap the Owner decides whether to raise it; a fixer or a fix-delta review is never skipped silently for cost: the skip is recorded as a Needs-your-decision item. The cap is not mechanical in `run-agent.sh`; the orchestrator reads it from the JSON after each agent. A failed run is not lost: `run-agent.sh` keeps its output as `<out>.failed.<N>` and its tokens and USD are read from there, so the totals do not undercount.

## 4. Lessons carried in (from U1 and earlier units)

- The design review (Opus) happens before any code and its guidance goes to the implementer prompts. The prompts state the header gate (exactly one `// Ported from ...scala` line per file, RULEBOOK D15), the shell rule, the `staticcheck` SA1032 rule, and that calls that should return immediately also run through a bounded helper.
- **Tests never prove "not processed" by reading a counter after a `Tell`** (U1 V9 passed 194 of 200 and V2 185 of 200 under a mutation that ignored suspension). RULEBOOK R4.3 requires the controlled executor (`ctlExec`) or an inline executor like `sup1InlineExec`, the assertion read after the target's own activation, channel waits only, a registration signal before racing a watcher, and a named killing mutant per scenario (R4.8). The implementer prompts repeat it verbatim.
- Mutations are written against the real code after the implementation (RULEBOOK 5.3 is the minimum list: K1-K37). Each mutant is run `-count=20` on a copy; **detected means failing in 20 of 20**. A mutant caught in fewer runs marks a non-deterministic assertion and the test is rejected. Exception, race-only mutants (K27): detected means W20 fails under `-race` in 20 of 20 runs with a deterministic interleaving through the `lockPoint` seam; if none can be written, K27 is reported as measured "detected k of 20 under -race" and listed as a survivor-with-reason, and the test is not rejected for it. Results are reported as measured: detected over total, survivors named with the test that closes them.
- Evidence reuse is claimed only after comparing the inputs of the **whole** tree the gate reads (including the README and any new test file), not only `pekko-actor/actor`.
- Honest accounting: orchestrator edits are recorded as manual; comments-only fixes need no fixer; a PORT-GAP is recorded, never silently dropped; `partial` or `blocked` reports are never rounded up.
- First argument of `judge/mutate_go.py` is a work directory: never call it with `--help` (U1 created a stray `--help/` directory).

## 5. Procedure after approval

1. **Baseline.** Record `git rev-parse HEAD:pekko-actor/actor` (the baseline tree), `shasum -a 256` of **every** `.go` file in `pekko-actor/actor/` and `git status --porcelain` (clean). Write the two approved hashes, one line each, to files **outside** the repository (the launcher refuses unless both hash files exist and match `RULEBOOK.md` and `settings.template.json`). Confirm `git submodule status` shows the kit at `cf91c9d`.
2. **Probe** (`haiku`, through `run-agent.sh`, no existing file written): read the rendered permission rules; attempt writes only to the **six new files** (`supervision_hooks.go`, `supervision_restart.go`, `supervision_budget.go`, `supervision_restart_internal_test.go`, `supervision_budget_internal_test.go`, `supervision_restart_goside_internal_test.go`, all under `pekko-actor/actor/`) and report what it wrote; it cannot remove them (the rendered permissions deny `rm` and `mv`, and Write/Edit cannot delete); attempt writes to `errors.go`, `mailbox.go`, `watch.go`, `clock.go`, `actor.go`, an existing test file other than the allowed one, and `judge/probe_stub.py` (expected refusals), plus `gofmt -w` on a non-target and `go test` (denied). The allow rules of the eight existing production files and of the existing U1 test file are checked by **reading** the rendered rules only. It also reports (read-only) that `ContextMisuse` does not exist in non-test code.
3. **Orchestrator cleanup, then compare with step 1.** The orchestrator removes exactly those six untracked files and records the removal as a manual change. Then the tree hash and every shasum must equal step 1, `git status --porcelain` must be empty, `git diff --stat` empty, and the six files absent. Restore mechanism: `git checkout -- <file>` for any existing file that differs, then repeat the check; the step 1 baseline, not a post-probe measurement, is the starting state.
4. **Design review** (`opus`, read-only). A BLOCKER goes to the Owner. Its guidance is passed verbatim to the implementer prompts.
5. **Implement** in the number of runs the Owner chose. Run A: production files; `gofmt`, `go vet`, `go build` in the loop. **Orchestrator gate after A**: the existing suite and the U1 suite `go test -race -count=1 ./actor/` except the two superseded sites (they fail until run B edits them; record which), the pinned lint (below), the anchor check (below), and `go test -race` on a smoke test the orchestrator writes in the scratchpad, not the repo. Run B: the three test files and the two edit sites; `gofmt`, `go vet` in the loop; the orchestrator runs `go test`.
6. **Gates after the implementation** (orchestrator, foreground, bounded output): `gofmt -l`, `go vet`, `go mod tidy -diff`, `go test -race -count=20 ./actor/`, coverage >= 85% per module (`scripts/check_coverage.sh`), `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `scripts/check-adr-signatures.sh`, `scripts/check-judge-matrix.py` (expected CURRENT 0/13 until section 6), `scripts/modules_selftest.sh`, `.github/scripts/go-sdk-update.sh --check`. **Lint:** the installed `golangci-lint` may be older than `.golangci-version` (v2.14.0; 2.13.1 was installed when this package was written). Install the pinned one under a temporary `GOBIN` (`GOBIN=$(mktemp -d) go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@v2.14.0`), check `--version` equals `.golangci-version`, and run it in the module. `gocyclo` (<= 10) is enforced through `golangci-lint` (`.golangci.yml:12-17`), not a separate binary. **Anchor check** (read-only; run after each loop step that changed `cell.go`, `stop.go` or `hierarchy.go`, and before section 6): a short script that loads `MUTATIONS` from `judge/mutate_go.py` and requires every `(file, anchor)` of RULEBOOK section 0 to occur exactly once in `pekko-actor/actor/<file>`. **If U2 needs to change an anchor the loop stops and reports; `judge/` is never edited.**
7. **Isolated mutations** on a copy of `pekko-actor` (`GOWORK=off`; the delivery tree is only read), K1-K37 plus additions, each `-count=20` with `-race -timeout 120s`; "detected over total" reported with survivors.
8. **Two reviewers in parallel** (forecast gate first): `opus` final and `sonnet` second, read-only, adversarial, each finding citing a RULEBOOK rule or a source line; disagreements settled by a cited line or a measurement; checks the Owner cares about: the restart order of R5-R8, counter balance (R14), the stash rule (R11), no `Terminated` on a restart, the `PostStop` guard, the data race of the actor swap (R16), the unchanged interim paths. Fixer on confirmed findings only; `opus` reviews the fix delta.
9. **Native review by work-unit commits.** The Owner consents per envelope. The engine refused a 1,461-line and a 2,000-line range in earlier units and accepted 735, so a range above about 735-900 changed lines may be refused and the whole branch is **never** one range. Commits, each with an exact base (the last boundary whose review was acknowledged; the first base is the branch point) and each followed by `gentle-ai review assess --cwd <repo> --agent claude-code --base-ref <boundary> --committed-only --json`: **C1** exported types, hooks, defaults, `MakeRestartDecider` (`supervision_hooks.go`, `supervision.go`, `supervision_decider.go`); **C2** restart protocol (`supervision_restart.go`, `cell.go`, `stop.go`, `hierarchy.go`, `spawn.go`, `supervision_failure.go`); **C3** budget (`supervision_budget.go`, `system.go`); **C4** budget tests; **C5-C6** restart tests, split by file region so each is <= about 700 lines; **C7** Go-side tests and the two U1 edit sites; **C8** fixes; **C9** documentation. If C1 or C2 does not build alone it is reviewed as a pair and the pair must stay <= about 900 lines, otherwise split by file. Commits are made by the orchestrator after `gofmt` (final normalization before review) and carry no attribution lines (Conventional Commits only).
10. **Documentation by the orchestrator** (manual, recorded): `pekko-actor/README.md` (restart section, limitations, interim, what is implemented versus planned), `pekko-actor/actor/doc.go`, `docs/actor-core-parity.md`, root `README.md` rows for #169-#174; the module README contract applies. It is committed **before** section 6, because the README is part of the `pekko-actor` tree the campaign records.
11. **Judge campaign, once** (section 6), then `EXECUTION-RECORD.md` (commands, kit version, models, forecasts and observed tokens, inputs, outputs, every manual change), one cost entry in `entries/cost-log.entry.tsv`, and a PR against `develop`. No merge without the Owner's word.

## 6. Judge campaign and evidence tags (needs the Owner's decision)

The Go campaign becomes stale (CURRENT 0/13 in `check-judge-matrix.py`) at the first `pekko-actor` change; a **new test file also changes the tree**. U1 repeated the campaign three times because tests and a parity row changed the tree after each run. For U2:

- **One renewal, on the final committed inputs**: after the last code, test and README commit, and after the anchor check, run `python3 -I judge/mutate_go.py <workdir> --runs 30 --report judge/go-mutation-report.tsv` on a copy, then `scripts/check-judge-matrix.py` (expected CURRENT 13/13). The report and `EXECUTION-RECORD.md` live outside the `pekko-actor` tree, so committing them does not stale it. Any later `pekko-actor` change (including a README line) requires another campaign and a new Owner authorization.
- If the campaign fails to apply an anchor (`anchor occurs N times`) or a mutation is not met, the loop stops and reports; the fix is a U2 code change that restores the anchor, never an edit of `judge/`.
- **The `evidence/*` tag of the exact commit the report cites needs explicit Owner authorization.** U1's was granted in advance. Owner choices: (a) authorize in advance one renewal and one tag `evidence/go-campaign-<7-char commit>` for the final commit, verified against its manifest; (b) decide at the end; (c) defer the renewal and accept stale evidence. Until the Owner decides, nothing in `judge/` is touched and no tag is published. The judge has **no U2 scenario**: S13/S14 run on both sides only once U3 lands, S16 after U8, S11 in U7, so U2 claims no Pekko parity beyond the ported tests.

## 7. Known follow-ups (not in this unit)

RULEBOOK section 3 (U3-U9, #450); the S11 stop during a restart as Pekko makes it (U7); logging of failures (U9); `AllForOne` runs of the tests adapted here (U6, `Ticket669Spec`); the exported way to call the default hooks (D4); the `originalCause` of `PostRestartException` (D1); the pre-existing `PostStop`-after-failed-`PreStart` divergence (R17.4); carried over from U1: the wall bound is per agent (the failed-run output and `timeout -k` are done in this unit's `run-agent.sh`), `run-agent.sh` does not escape the worktree root, and the open native findings listed in `supervision-u1/EXECUTION-RECORD.md` section 7.

## 8. Hashes for approval

The orchestrator fills these placeholders when presenting the package; the files are not edited after. `run-agent.sh` changed after the package review (failed-run output kept as `<out>.failed.<N>`, and `-k 30s` on the wall bound), so any hash taken before that change is stale.

| File | sha256 |
|---|---|
| `RULEBOOK.md` | `<sha256 of RULEBOOK.md>` (also passed to `run-agent.sh` as a one-line hash file) |
| `RUNBOOK.md` | in the approval request only (a hash of this file cannot be written inside it): `<sha256 of RUNBOOK.md>` |
| `manifest.tsv` | `<sha256 of manifest.tsv>` |
| `settings.template.json` | `<sha256 of settings.template.json>` (also passed to `run-agent.sh` as a one-line hash file) |
| `run-agent.sh` | `<sha256 of run-agent.sh>` (usage: `run-agent.sh <model> <prompt-file> <out-json> <rulebook-sha256-file> <settings-sha256-file>`; env `AGENT_BUDGET_USD`, `AGENT_WALL_MIN`) |
| `EXECUTION-RECORD.md` skeleton | `<sha256 of EXECUTION-RECORD.md>` |
