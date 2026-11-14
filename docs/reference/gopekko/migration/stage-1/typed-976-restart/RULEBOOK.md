# RULEBOOK: Typed restart within its limit (unit typed-976-restart, #976 TYP-10)

Status: **approved at gate G2 by the Owner on 2026-11-05** for `docs/scopes/976-typed-restart.md` (PR #1351, merged into `develop` `6f60234`; D-1 to D-4 accepted as recorded in the scope). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 0.

## 1. Unit

- Capability: a Typed actor whose behavior is wrapped in `Supervise(b).OnFailure(match, Restart())` (optionally `.WithLimit(n, within)` and `.WithStopChildren(bool)`) restarts when a turn fails with a matching cause: `PreRestart` to the old behavior only, the initial behavior rebuilt (`Setup` reruns), the cell, ID, mailbox and watches kept. Past the limit, or when the initial start is not retried, the cause is rethrown with the consequences of ADR 0035 S-2.6. Source: Pekko `Supervision.scala:290-440`, `SupervisorStrategy.scala:44-45, 162-216, 266`, `ActorAdapter.scala:309-322` at v1.7.1 (`9a54d12`), read, and the oracles of `docs/parity/oracles/` (executed).
- Governing decisions: ADR 0033 D2, D5, D13, D14, D15; ADR 0034 P3, P4, P8; ADR 0035 contracts 2, 3, 6, S-2.6, S-3 cases (a), (b), (c-async); Owner decisions D-1 to D-4 of the scope.
- Manifest: `manifest.tsv` follows the kit runner contract (`migration-kit/scripts/queue_runner.mjs`): `source` and `target` are the first two columns and the only ones it reads. Moved and `PORT-GAP` rows point at this RULEBOOK, whose recorded disposition is their output.
- Writable: exactly the "Allowed files" and slice table of the scope for the PR in hand. Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum`, the root `README.md`, nor the scope file.

## 2. Upstream search

Spec rows come from `SupervisionSpec.scala` and `ActorContextSpec.scala`; the S-2, S-3 and D-a-with-children scenarios are executed oracles. "No spec found" below means found by these searches only, not a proof of absence.

## 3. Decisions

- **Two restarts (D-4).** The restart by a Classic parent keeps the children (S-2.3c); the Typed restart stops them by default and waits (`Supervision.scala:362-363, 385-388, 421-427`). The TODO at `interpreter.go:168-170` is corrected, not implemented.
- **Known gaps, stated in the README:** c-sync fatal path (#1345); restart stash unbounded until #999 (Pekko: `restart-stash-capacity`, overflow publishes `Dropped`, ADR 0036).
- **Overlap with #997:** `WithStopChildren` and the default wait are delivered here; the stop-during-wait boundary stays in #997.
- **Out of scope:** `Resume` (#1183), the tests of the nested rule (#1184; the equal-matcher replacement mechanism itself is slice 1b, to keep the supervisor chain bounded), backoff (#986), bounded stash (#999), timers, logging (#1179), M-A and `ChildFailed` to a Typed parent (#1170, #1171), explicit `Stop()` (D-1).
- **R10 has no Pekko counterpart.** It guards the Go-specific deviation from the recursive restart of `Supervision.scala:386` (the restart is a loop); the C1 oracle shows an endless asynchronous loop and does not support a bounded-stack scenario, so it is not cited for R10.
- This is Go unit evidence derived from Pekko specs and oracles, **not V3**. Delivering #976 does not close #986, #997, #999, #1183, #1184 or #1345.

## 4. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>` line per file derived from one Pekko file; test files that mix several specs carry none. Checked by `.github/scripts/check-headers.sh`.
- Scenarios observe results through channels and gates; no `time.Sleep`, no polling, no `-race`.
- No `//nolint`; cyclomatic complexity at most 10; go-specs for assertions.
- Unknown behavior: `// TODO(TYP-10-edge): <reason>` and keep going (kit rule 8).
- Comments and names claim no more than is shown: no Typed parity.
