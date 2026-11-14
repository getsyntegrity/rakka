# RULEBOOK: install and replace a Typed receive Behavior (unit typed-847-behavior-install, #847 TYP-02)

Status: **approved at gate G2 by the Owner on 2026-10-27** for `docs/scopes/847-typed-behavior-install.md` (commit `9310063`). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 1.

## 1. Unit

- Capability: `Setup`, `Receive`, `ReceiveMessage`, `Same`, `Context[T].Self()` and `NewActor` in `pekko-actor-typed/typed/`, running on the `pekko-actor` runtime. Source: Pekko `Behaviors.scala:39,114` and `Behavior.scala:184,198` at v1.7.1 (`9a54d12`), read, not run.
- Governing decisions: ADR 0033 D1, D3, D13; ADR 0034 P1 to P4, P8 (accepted by Owner decision B-1); Owner decisions D-1 to D-6 of the scope.
- Manifest: `manifest.tsv` follows the kit runner contract (`migration-kit/scripts/queue_runner.mjs`): `source` and `target` are the first two columns and the only ones it reads; a row is done when its target file exists. The extra columns (`disposition`, `pekko_row`, `scenario`, `evidence`, `note`) are the disposition record the runner ignores. Ported, adapted and READ rows point at the Go file that holds the result; moved and `PORT-GAP` rows point at this RULEBOOK, whose recorded disposition is their output.
- Writable: exactly the "Allowed files" of the scope for the PR in hand. Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`.

## 2. Upstream search for the READ rows (TL, 2026-10-27, grep plus reading, nothing run)

Method: grep over `actor-typed-tests/src/test` (scala, scala-3, java), each candidate read in context. This is "no spec found by these searches", not a proof of absence.

- **S8, a `Setup` returned by a handler.** One Typed spec: `StashSpec.scala:612-630` (`actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/StashSpec.scala`) returns `Behaviors.setup { _ => stash.unstashAll(Behaviors.same) }`; it needs `StashBuffer`, so its row moves to #889/#927. Coverage of the search: 35 files under `.../src/test/scala/org/apache/pekko/actor/typed` contain `Behaviors.setup`, and 84 files across all test trees (53 scala, 18 of them under `scala/docs`; 1 scala-3; 30 java). A structural filter covered all 84; `DeferredSpec`, `SupervisionSpec` and `StashSpec.scala:612-630` were read in context. The other uses seen are spawn arguments, nested setups or wrapped by `supervise`; two hits in `scala/docs` and `java` were false positives. This is grep plus reading, not a proof of absence. S8 has no portable spec and stays READ. The `Setup` returning `Same` edge also has no spec found.
- **S11, Ref and mailbox identity.** Ref identity: SPEC, indirect (`ActorContextSpec.scala:535-543`; `BehaviorSpec.scala:301-303` through `BehaviorTestKit`). Mailbox identity across a replacement: no spec found; mailbox specs cover selection, type or dropped messages (`MailboxSelectorSpec`, `ActorSystemSpec.scala:189-207`, `EventStreamSpec.scala:106`, `TimerSpec.scala:141,189`). Stays READ.

## 3. Decisions

- **READ rows are alternative evidence, accepted by the Owner (ADR 0033 D13):** S2, S4, S8, S9, S10 and the mailbox part of S11. History: at G2 on 2026-10-27 the Owner accepted that these rows are recorded as READ and pending; the evidence column of manifest.tsv still carries that pre-decision label ("READ, pending Owner approval") and is superseded by this entry. **Owner decision of 2026-10-28** (a message of the Owner relayed by the orchestrator, session gopekko-1): "The Owner accepts Go tests as alternative READ evidence for S2, S4, S8, S9, S10 and the mailbox portion of S11." This accepts the evidence under ADR 0033 D13. It is **not** differential Pekko parity, **not** V3 certification and **not** an executed Pekko scenario; the rows still block any V3 or parity claim. A spec found later replaces the READ label.
- **Out of scope:** `Unhandled`, `Empty`, `Ignore`, `Stopped`, signals, P6, `receivePartial`, supervision, `Props`, stash, timers, nil `Tell` (X2). `ActorContextSpec.scala:100` moves to #925, `StashSpec.scala:612-630` to #889/#927.
- **Gap kept explicit:** a `Setup` returned by a handler whose factory returns `Same`. Conservative form: fail the turn with `ErrInvalidInitialBehavior`, marked `// TODO(TYP-02-edge): timing differs from Pekko`. No parity claim.
- **Interim divergence:** until #956 a failing Typed actor follows the runtime's Restart default, not Typed's Stop (ADR 0033 D2 versus ADR 0017). Tests install an explicit decider; no Typed parity claim.
- This is Go unit evidence derived from Pekko specs, **not V3**.

## 4. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>.scala` line in files derived from one Pekko file (`behavior.go`: `Behavior.scala`; `interpreter.go`: `ActorAdapter.scala`); `context.go` has none. Checked by `.github/scripts/check-headers.sh`.
- Scenarios S1 to S11 observe results through channels as gates; no `time.Sleep`, no polling, no `-race`, no workbench.
- Only the symbols of "Contract produced" are exported; another export is a stop. No `//nolint`; cyclomatic complexity at most 10.
- Unknown behavior: `// PORT-GAP(<id>): <reason>` and keep going (kit rule 8).
- Comments and names claim no more than is shown: no Typed parity.
