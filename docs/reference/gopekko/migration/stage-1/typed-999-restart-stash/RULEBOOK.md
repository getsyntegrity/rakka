# RULEBOOK: Bound the Typed restart stash and publish `Dropped` (unit typed-999-restart-stash, #999 TYP-26; #906 prerequisite)

Status: **proposed; awaiting the Owner's sign-off of this RULEBOOK and its hash before any U2 or U3 loop.** The scope `docs/scopes/999-restart-stash-bound.md` (integrated by #1560, `2b799d9dd9cbcaff28db50237d25ebb44ecd6d73`) has G2 by the Owner's express instruction of 2026-11-13, with D-7 "kit run for U2/U3 with K0, no waiver". That G2 does not approve this file or a hash that did not exist when it was given. Read-only inside any loop (kit `CLAUDE.md` rule 1) once signed.

## 1. Unit

- Capability: a Typed actor under `Supervise(b).OnFailure(match, Restart())` that is waiting for its children bounds what it retains (`WithStashCapacity(n)`, default 1000, an internal constant of `typed`, D-3). A message or non-awaited `Terminated` that arrives at a full stash is discarded and published once as `actor.Dropped{Msg, "Stash is full in [RestartSupervisor]", Sender: nil, Recipient: self}`; the awaited children's `Terminated` never occupies capacity; a stop during the wait with a full stash publishes one `Dropped(PostStop)`; the stash survives a failure during replay with its first capacity. Source: Pekko `Supervision.scala:120-123, 254-286, 365-376, 406-408`, `StashBufferImpl.scala:59-75`, `SupervisorStrategy.scala:145-177` at v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`).
- Governing decisions: ADR 0035 contract 5; ADR 0036 EVT-1.4 and its Amendments of 2026-11-05 and 2026-11-06 (e); ADR 0033 D13, D14; ADR 0038 D1; EXP-1; Owner decisions D-1 to D-10 of the scope (2026-11-13).
- Manifest: `manifest.tsv` follows the kit runner contract (`migration-kit/scripts/queue_runner.mjs`): `source` and `target` are the first two columns and the only ones it reads. Moved rows point at this RULEBOOK.
- Writable per PR: exactly the "Allowed" files of its unit in the scope. U2 and U3: `pekko-actor-typed/typed/{interpreter.go, supervise.go}`, new `restart_stash_*_test.go`, `pekko-actor-typed/README.md`, rows of this manifest. Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum`, the root `README.md`, the judge, nor the scope file.

## 2. Upstream search

Rows come from `Supervision.scala`, `StashBufferImpl.scala`, `SupervisorStrategy.scala`, `SupervisionSpec.scala` (the restart variants of the retention tests; the two backoff tests that assert `Dropped` are not evidence for plain `Restart`) and the executed oracles W3a, W3c, W3d, W3e, D1, RS0 to RS2, EV9. "No spec found" means found by these searches only.

## 3. Decisions

- **O1 first.** U2 starts only after O1 (capacity zero, positive and negative on plain `Restart`, non-awaited `Terminated` against a full stash, capacity across a failure) is integrated. There is **no oracle waiver** (D-4). If an oracle contradicts an expectation of the scope, the evidence is kept, the conflict is recorded and the Owner decides; no expectation is edited to get green.
- **U1 first.** `actor.Dropped` (#1562) is integrated before U2 publishes it (D-1, D-2, D-9).
- **Nested supervisors.** Pekko keeps one buffer per `RestartSupervisor`; Go has one `held` per interpreter. Conservative rule (kit rule 8): the capacity of the layer that creates the stash holds until it is released, with `// TODO(TYP-26-nested): <reason>`.
- **Out of scope:** backoff and `ScheduledRestart` (#986), `AllDeadLetters` membership (#1057), `MessageBuffer.drop` and the rest of #906, group-router stash, `Topic.Publish`, `ChildFailed` (#1346), logging of the drop (#1179, #1479), a `System` setting for the default.
- This is Go unit evidence derived from Pekko sources and oracles, **not V3** and no Typed parity. Delivering U2 and U3 does not close #999, #906, #986 or #1057.
- **Judge (D-8):** U1 changes `core_sha`; the Go campaign is renewed after U1 integrates, coordinated with other core changes. U2 and U3 do not touch `pekko-actor/`.
- **Budget:** every PR at most 400 authored lines, additions plus deletions, nothing excluded beyond `AGENTS.md`. No exception is granted (D-10). A unit that does not fit is split or a concrete request goes to the Owner first.

## 4. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>` line per file derived from one Pekko file; test files that mix several sources carry none. Checked by `.github/scripts/check-headers.sh`.
- Scenarios observe results through channels, gates and `Subscribe[Dropped, M]`; no `time.Sleep`, no polling, no `-race`. go-specs for assertions; doubles through the project's spies.
- No `//nolint`; cyclomatic complexity at most 10.
- Unknown behavior: `// TODO(TYP-26-edge): <reason>` and keep going (kit rule 8).
- Comments and names claim no more than is shown: no Typed parity.
