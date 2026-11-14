# RULEBOOK: Classified event subscriptions on the EventStream (unit eventstream-registration, #836 EVT-02)

Status: **approved at gate G2 by the Owner on 2026-11-06** for `docs/scopes/836-eventstream-registration.md` (D-1 to D-7). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 0.

## 1. Unit

- Capability: `Subscribe[E, M any](s *System, subscriber Ref[M]) (changed bool, err error)` registers a subscriber for a channel (a Go type) in an index with the structure of Pekko's `SubclassifiedIndex`. It publishes, unsubscribes and watches nothing (#870, #905, #1056).
- Pekko is the oracle (ADR 0033 D14): Pekko v1.7.1 (`9a54d12`), `docs/parity/oracles/eventstream-spec-2026-11-06.md`. Decisions: ADR 0038 D1 to D9 and its Amendments, ADR 0036 EVT-1. Evidence classes of the manifest: `READ` (Pekko code, read), `ORACLE` (scenario executed on Pekko). Moved and `PORT-GAP` rows point here. `manifest.tsv` follows the kit runner contract (`source`, `target` first).
- Frozen API (change it only through the TL): `Subscribe`, `ErrNilSubscriber`, `ErrChannelNotAccepted` in package `actor`. The stream, the index, the relation and `subscriberEntry` stay unexported or under `internal/`. Identity of a subscriber is `Handle.ID()` (D-4).
- Layout: implementation in `pekko-actor/internal/classification` and `pekko-actor/actor`; no `spi/` (no real port here). `actor` imports no other goPekko module; `scripts/deps.allow` is untouched.
- Writable: exactly the slice table and "Allowed files" of the scope for the PR in hand. Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `pekko-actor/internal/deadletter/`, another module, nor the scope file.

## 2. Criteria and slices

| Slice | Criteria | Allowed files |
|---|---|---|
| 0 | kit package | `migration/stage-1/eventstream-registration/{manifest.tsv,RULEBOOK.md,RUNBOOK.md}` |
| 0b | X7b observation | `docs/parity/harness/...`, the oracle `.md` and its `.logs/` |
| 1a / 1b | X1, X2, X4a, X12 / X3, X4b | `internal/classification/index.go`, `index_test.go` |
| 2 | X5 | `internal/classification/channel.go`, `channel_test.go` |
| 3a / 3b | X8a, X9, X10 / X6, X7a, X7b, X8b, X13 | `actor/eventstream.go`, `eventstream_errors_test.go`, `eventstream_test.go`, `actor/system.go` (one field) |
| 4 | X11 | `pekko-actor/README.md`, `EXECUTION-RECORD.md`, `migration/cost-log.tsv` |

Paths under `internal/` and `actor/` are in `pekko-actor/`. Internal test files (`<file>_internal_test.go`) are allowed in any slice with the same budget. X7b stays pending until slice 0b observes EV4f (ADR 0033 D13).

## 3. Decisions and known gaps

- D-2: `E` must be assignable to `M`, checked at run time; violation is `ErrChannelNotAccepted`. The ADR 0038 amendment recording it is merged before slice 3a.
- D-3: equal types are sub-classes; otherwise `x` is a sub-channel of `y` when `x != y`, `y` is an interface, `x.Implements(y)`, and not (`x` an interface and `y.Implements(x)`). `any` is the root; `type Empty interface{}` is unrelated to it.
- D-5: a stopping System is not refused; leave `// TODO(EVT-02-stopping): <reason>` (kit rule 8) for #1056, whose oracle must cover it.
- Not here: publish and cache (#870), unsubscribe through the API (#905), termination removal (#1056), `ActorEventBus` and the reusable bus (#891), the adapter for another message type (#959). This is Go unit evidence, not V3.

## 4. Mandatory Stops

Report to the TL and stop; never skip, delete or weaken a test:

1. A change outside the allowed files is needed (including `docs/adr/`, `scripts/deps.allow`, another module).
2. A test fails on the current code where the scope expects it to pass.
3. A slice projects over 400 authored lines (split it first; the index slice is already split in 1a and 1b).
4. A contract above (the frozen API, ADR 0038) must change.

## 5. Mutation rule

Each behavior test names a mutation in a "Mutations" table of its PR (behavior, mutation, commit, failing test), taken from the scope criteria table. The mutation is applied to production code in a disposable worktree, never committed, and the named test must fail by assertion; a test that survives is fixed. Slices 0, 0b and 4 are docs only: no mutation applies.

## 6. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>` line per file derived from one Pekko file (`index.go`, `channel.go`: `util/SubclassifiedIndex.scala`); files that mix several Pekko sources or specs (`actor/eventstream.go`: `EventBus.scala` and `EventStream.scala`) carry none, and the manifest records provenance. Checked by `.github/scripts/check-headers.sh`.
- go-specs for assertions; channels and gates, no `time.Sleep`, no polling, no `-race`; `-count=100` on the concurrent test (X13).
- Coverage at least 85% per module (never exclude production code); no `//nolint`; gocyclo at most 10.
- Every sentence of a PR that says something was observed names the oracle line or the assertion; every Pekko citation is re-opened by the Dev.
