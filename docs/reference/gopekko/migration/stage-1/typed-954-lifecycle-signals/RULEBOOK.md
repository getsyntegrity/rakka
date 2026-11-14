# RULEBOOK: Typed signals infrastructure and PostStop delivery (unit typed-954-lifecycle-signals, #954 TYP-04 Slice A)

Status: **approved at gate G2 by the Owner on 2026-10-28** for `docs/scopes/954-typed-lifecycle-signals.md` (head `c5e7f19`, PR #1234; D-1, D-4, D-5, D-6 accepted, D-2, D-3, D-7 and D-8 as recommended by the scope). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 1.

## 1. Unit

- Capability: `Signal`, `PreRestart`, `PostStop`, `Terminated`, `ChildFailed`, `MessageAdaptionFailure` (types), `ReceiveSignal`, `ErrSignalBaseNotReceive` in `pekko-actor-typed/typed/`; delivery of `PostStop` only. Source: Pekko `MessageAndSignals.scala`, `Behavior.scala:198` and `ActorAdapter.scala:309-331` at v1.7.1 (`9a54d12`), read, not run.
- Governing decisions: ADR 0033 D5, D13; ADR 0034 P2, P3, P4, P6, P8, B16, B17, B19; Owner decisions of the scope.
- Manifest: `manifest.tsv` follows the kit runner contract (`migration-kit/scripts/queue_runner.mjs`): `source` and `target` are the first two columns and the only ones it reads. The extra columns are the disposition record the runner ignores. Moved and `PORT-GAP` rows point at this RULEBOOK, whose recorded disposition is their output.
- Writable: exactly the "Allowed files" of the scope for the PR in hand. Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum`, the root `README.md`, nor PR #1233.

## 2. Upstream search

Spec rows S1 and S2 come from `BehaviorSpec.scala` (Lifecycle and Become lifecycle). No Typed spec was found for S3 to S10 (grep over `actor-typed-tests` for `PostStop`, `receiveSignal`, `interpretSignal`; plus reading). This is "no spec found by these searches", not a proof of absence.

## 3. Decisions

- **READ rows are alternative evidence pending Owner approval (ADR 0033 D13):** S3 to S10. Not differential Pekko parity, not V3, not an executed Pekko scenario.
- **D-4:** the interpreter gets a no-op `actor.PreRestarter`, temporary. It delivers no `PreRestart`; it also skips the runtime's unwatch and stop of children at a restart, restored by #976 and required before #1170. A Stop that abandons a restart is an evidence gap with no coverage claimed.
- **D-2:** `Terminated`, `ChildFailed`, `MessageAdaptionFailure` are opaque (no exported field, accessor or constructor); B-2 is not approved.
- **D-7:** a `PostStop` handler error is raised as a panic with `// TODO(TYP-04-edge)`.
- **Out of scope:** delivery of `PreRestart`, `Terminated`, `ChildFailed`, `MessageAdaptionFailure`; `Stopped`, `Unhandled`, `Empty`, `Ignore`; supervision; Typed watch and spawn.
- This is Go unit evidence derived from Pekko specs, **not V3**. Delivering A does not close #954.

## 4. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>` line per file derived from one Pekko file (`signal.go`: `MessageAndSignals.scala`; `signal_bhv_test.go`: `BehaviorSpec.scala`); `signal_runtime_test.go` has none. Checked by `.github/scripts/check-headers.sh`.
- Scenarios observe results through channels and gates; no `time.Sleep`, no polling, no `-race`. S9 uses a fixed N and an invariant.
- Only the symbols of "Contract produced" are exported; another export is a stop. No `//nolint`; cyclomatic complexity at most 10.
- Unknown behavior: `// TODO(TYP-04-edge): <reason>` and keep going (kit rule 8).
- Comments and names claim no more than is shown: no Typed parity.
