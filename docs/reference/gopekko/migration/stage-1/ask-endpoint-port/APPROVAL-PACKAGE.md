# APPROVAL PACKAGE: Ask endpoint unit (#161, PR #424)

Status: **proposal. No approval is recorded here or anywhere in this repository.** Each decision below is the Owner's. The recommendation is the preparer's.
Prepared at HEAD `21b041f` (plus this file). The reviewed candidates and their evidence are in `EXECUTION-RECORD.md` and `AMENDMENTS.md`.

## 0. Acts that need the Owner (all together)

| # | Act | Recommended answer | Needed before |
|---|---|---|---|
| A | Approve the RULEBOOK at a named commit (section 1) | approve as is, or send amendments | the loop starts |
| B | Confirm the model plan and spend limits (section 2) | plan 2.1 | the loop starts |
| C | Install `settings.askport.json` (section 3), later remove it | install after A and B | the loop starts; removal before the gate |
| D | Decide the status of the manual code (section 4) | option D1: no exception, kit validates it | the loop starts |
| E | Ask the GitHub account owner to fix Actions billing | separate, any time | CI on the final SHA |

Nothing else is required of the Owner for this unit. The review coverage question (section 5) needs no act.

## 1. RULEBOOK

- Path: `migration/stage-1/ask-endpoint-port/RULEBOOK.md`, last changed in commit `02b8e29`.
- sha256: `2da88e48cde5e6dfadf7c86453381dd5e540159d53661b1ca9a0d7ac882be859`.
- Related, same directory: `manifest.tsv` (sha256 `20540d99…91b1c2`), `SCOPE.md`, `RUNBOOK.md`, `EXECUTION-RECORD.md`.
- Content: one unit, `PromiseRefSpec.scala` (Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`) to `pekko-actor/actor/ask_endpoint_promiseref_internal_test.go`. Cases: line 37 reduced, line 54 faithful, lines 48 and 67 omitted as `PORT-GAP`. Conventions R1.1 to R1.6, construct map, model plan (section 7), deviation log (empty).
- Known limit: authored by the orchestrating session (self-reported `claude-sonnet-5-5`), not by the largest model the kit recommends for rulebooks. Your review is its only check.
- **Recommendation:** approve, and record "approved at <commit>" in RULEBOOK section 8's header (that edit is yours or is made by the session only after you say so). If you want the largest-tier model to re-author it first, that is a separate, costed step; I do not recommend it for a one-file unit with four cases.

## 2. Model plan and spend limits

### 2.1 Recommended plan

The kit has no budget enforcement: limits below are observed and enforced by the orchestrating session, which stops and asks when one is hit. Token and wall-clock figures are proposals; no cost has been measured for this unit (`cost-log` rows are `unknown`).

| Phase / role | Count | Selector (explicit on every call) | Access | Limit per agent |
|---|---|---|---|---|
| Implement | 1 | `sonnet` | read worktree; edit only the target | 150k tokens, 15 min |
| Review | 2 (separate contexts) | `sonnet` | read-only | 100k tokens each, 10 min |
| Third review | 0 or 1, only on disagreement | `sonnet` | read-only; default "not confirmed" | 100k tokens, 10 min |
| Fix | 0 or 1, confirmed findings only | `sonnet` | edit only the target | 100k tokens, 10 min |
| **Total** | at most 6 agents | | | **600k tokens, 60 min wall clock** |

If the total or any agent limit is exceeded, or an agent stalls (no growth of the target for 3 minutes, prompt 04), the session stops and reports; it does not continue on inferred approval.
Self-reported model ids are recorded as such; they are not external verification.

### 2.2 Alternative

Reviewers on a larger tier (kit: "skeptic reviewers: largest or mid, depending on gap complexity"). Costs more; justified only if you consider the runtime review (section 4) high risk. **Not recommended** for this unit.

## 3. `settings.askport.json`

- Source: `migration/stage-1/ask-endpoint-port/settings.askport.json` (sha256 `d03d4381…b44b`), copied from the Watch unit's file with the paths changed.
- Destination (your act, never mine): `/Users/pablogore/workspace/pablogore/goPekko/.claude/worktrees/act-015-ask-endpoint/.claude/settings.json`. Command: `cp migration/stage-1/ask-endpoint-port/settings.askport.json .claude/settings.json` run from that worktree. It must not be committed.
- Deny (loop-wide): `git rebase|reset|checkout|stash|push|commit|merge|switch|restore|clean`, `sbt`, `go test`.
- Allow: `git status`, `git diff`, `git log`, `rg`, `wc`, `go build`, `go vet`, `gofmt`, `eza`; `Read` of the whole worktree; `Edit` of **only** `pekko-actor/actor/ask_endpoint_promiseref_internal_test.go`. `additionalDirectories`: the worktree.
- **Effect on the session:** while the file exists, every Claude Code session whose project directory is that worktree is bound by it, including the orchestrating one. It cannot run `go test`, cannot commit or push, and can edit only the target file. The gate (tests, lint, coverage, commit, push) therefore cannot run until you remove the file.
- Removal: `rm <worktree>/.claude/settings.json`, by you, after the loop and before the gate.
- **Recommendation:** install after A and B.

## 4. Manual authorship and how the kit validates it

### 4.1 What already exists (manual, outside the kit)

| Code | Where | Commit | Review |
|---|---|---|---|
| Endpoint, target validation | `pekko-actor/actor/ask_endpoint.go` (87 lines) | `4fc2748` | native review of `77a86d5..f90165a`, approved |
| `ErrInvalidHandle` text | `pekko-actor/actor/errors.go` | `4fc2748` | same |
| Internal tests (AK20, AK21, AK23, identity, isolation, Stop no-op) | `pekko-actor/actor/ask_endpoint_internal_test.go` | `4fc2748`, `774e0cb` | same, plus delta `c4d3905..02b8e29`, approved |

`pekko-actor` subtree at HEAD: `ab6ab673d85a3557e8dad9bdce13ce9b5d9a2391`.

### 4.2 Options

| Option | What the kit does | Provenance | Cost | Verdict |
|---|---|---|---|---|
| **D1 (recommended)** | Implementer ports `PromiseRefSpec` as a new test file against the existing runtime. The two reviewers and any third review the **port and the manual runtime read-only**, citing RULEBOOK rules or spec lines. Findings on the runtime are queued in `AMENDMENTS.md`, not fixed in the loop (the settings allow editing only the target). | stays "manual" for runtime and tests; the port is "kit" | smallest | validates the manual code against upstream cases and adversarial reading, without rewriting it |
| D2 | As D1, and widen `Edit` to the three manual files so the fixer may apply confirmed findings to them | mixed; every kit edit to a manual file is recorded in `EXECUTION-RECORD.md` | medium | use only if you want the kit to be able to change the runtime; needs a new settings file and an updated RULEBOOK section 0 |
| D3 | Kit re-implements `ask_endpoint.go` from RULEBOOK and SCOPE in a scratch path, then the two versions are diffed | manual and kit versions both kept | largest | rework; not recommended: the runtime is 87 lines and already reviewed |

The preparer does **not** assume any exception: under D1 no deviation is needed, because the manual code is not presented as kit output. If you choose to record one, it is DEV-n in RULEBOOK section 8 with the exact files.

### 4.3 What will and will not be claimed

Under D1 the PR may say: "`PromiseRefSpec` cases ported by the kit loop (model ids as configured), reviewed adversarially; the runtime and internal tests are manual, reviewed natively." It may **not** say the kit implemented the endpoint, and it still does not claim parity with Pekko (no sbt oracle run).

## 5. Review coverage of HEAD `21b041f`

| Range | Commits | Native review | Result |
|---|---|---|---|
| `77a86d5..f90165a` | `4fc2748`, `f90165a` | `review-8dab1b644bea1a1d` (target `sha256:de1005ac…`) | approved, acknowledged |
| `f90165a..fcdf56b` | `fcdf56b` (report) | none | generated; campaign evidence only |
| `fcdf56b..c4d3905` | `ff1b525`, `c4d3905` | `review-a08b1337b5bb22bc` (target `sha256:198a965f…`) | approved, acknowledged |
| `c4d3905..02b8e29` | `774e0cb` (test fix), `02b8e29` (docs, ADR 0015 D7/AK23) | `review-183e5dd85b4bb283` (target `sha256:2bff5625…`) | approved, acknowledged |
| `02b8e29..21b041f` | `21b041f` (report renewal) | none | generated; campaign evidence only |
| `77a86d5..21b041f` (whole) | all | `review-4c04d0694d3ae5b7` (target `sha256:665d09db…`) | **declined by the Owner for this candidate**; START is not requested again |

Changes after `02b8e29`: exactly one file, `judge/go-mutation-report.tsv` (252 rows replaced, generated by `judge/mutate_go.py --runs 30` at `02b8e29`: 53 failed, 199 survived, none unmet). The `pekko-actor` subtree is identical at `02b8e29` and `21b041f`. **No semantic delta is pending.** This approval package itself is a new docs file added after `21b041f`; it is covered only by whatever review you choose for it.
The generated report has no semantic approval and none is attributed to it.

## 6. Not part of this package

Actions billing (act E) is independent: no reruns until fixed; then re-verify HEAD and base, update the branch if `develop` received fixes, run CI on the resulting SHA. #424 stays draft and #161 open; #162 is not enabled as independent until the dependency integrates.
