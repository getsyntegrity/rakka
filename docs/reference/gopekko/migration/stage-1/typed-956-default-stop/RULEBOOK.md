# RULEBOOK: Typed default Stop (unit typed-956-default-stop, #956 TYP-09)

Status: **approved at gate G2 by the Owner on 2026-10-28** for `docs/scopes/956-typed-default-stop.md` at head `9c716c3` (PR #1233, approval comment 6470881779; D-1 to D-6 accepted as recorded in the scope). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 1.

## 1. Unit

- Capability: a failed turn of a Typed actor (returned error, panic, nil behavior) stops the actor from inside the interpreter (mechanism M-B of ADR 0035): `StopSelf`, `Receive` returns nil, the parent's decider is not consulted, a watcher gets a plain `Terminated`, and `PostStop` reaches the behavior that was current when the turn failed. No exported symbol is added. Source: Pekko `ActorAdapter.scala:125-143,168-172`, `SupervisionSpec.scala:387,492,502`, `ClassicSupervisingTypedSpec.scala:74,84,105,130,142,152`, `TypedSupervisingClassicSpec.scala:61-71` at v1.7.1 (`9a54d12`), read, not run.
- Governing decisions: ADR 0033 D2, D5, D13; ADR 0034 P3, P4, P8, B17; ADR 0035 contract 1, S-1, S-6; Owner decisions D-1 to D-6 of the scope.
- Manifest: `manifest.tsv` follows the kit runner contract (`migration-kit/scripts/queue_runner.mjs`): `source` and `target` are the first two columns and the only ones it reads. The extra columns are the disposition record the runner ignores. Moved and `PORT-GAP` rows point at this RULEBOOK, whose recorded disposition is their output.
- Writable: exactly the "Allowed files" of the scope for the PR in hand. Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum`, the root `README.md`, nor the scope file or PR #1233.

## 2. Upstream search

Spec rows S1 to S3, S6 and S8 come from `SupervisionSpec.scala`, `ClassicSupervisingTypedSpec.scala` and `TypedSupervisingClassicSpec.scala`. No Typed spec was found for S4, S5 and S7 (reading of `ActorAdapter.scala` only). This is "no spec found by these searches", not a proof of absence.

## 3. Decisions

- **READ rows are alternative evidence pending Owner approval (ADR 0033 D13):** S4, S5, S7. Not differential Pekko parity, not V3, not an executed Pekko scenario.
- **D-2:** mechanism M-B only. A Typed parent and `ChildFailed` (M-A) belong to a separate story with no issue number.
- **D-3:** failure logging (contract 13) is pending; no TODO approximation, no `pekko-actor` export.
- **D-4:** `#847` S9/S10 keep "the turn fails, the actor stops"; "the previous behavior survives under Resume" is an obligation of #1183.
- **Failed start unchanged:** a failing `Setup` is still an `InitializationError` decided by the runtime; `// TODO(TYP-09-start)` marks the difference with Pekko.
- **Out of scope:** `Resume` (#1183), `supervise`/`OnFailure` (#1184), Typed parent, `ChildFailed`, restart, limits, backoff, stash, Slice B and B-2 of #954.
- This is Go unit evidence derived from Pekko specs, **not V3**. Delivering #956 does not close #954.

## 4. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>` line per file derived from one Pekko file (`interpreter.go` keeps its single line); `default_stop_test.go` mixes several Pekko specs and carries none. Checked by `.github/scripts/check-headers.sh`.
- Scenarios observe results through channels and gates; no `time.Sleep`, no polling, no `-race`.
- No new exported symbol; no `//nolint`; cyclomatic complexity at most 10.
- Unknown behavior: `// TODO(TYP-09-edge): <reason>` and keep going (kit rule 8).
- Comments and names claim no more than is shown: no Typed parity.
