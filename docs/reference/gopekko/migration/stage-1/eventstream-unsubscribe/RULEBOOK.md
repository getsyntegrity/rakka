# RULEBOOK: Unsubscribe one classification or all registrations of an actor subscriber (unit eventstream-unsubscribe, #905 EVT-04)

Status: **approved at gate G2 by the Owner on 2026-11-07** for `docs/scopes/905-eventstream-unsubscribe.md` (D-1 to D-7). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 0.

## 1. Unit

- Capability: `Unsubscribe` removes the registrations of a subscriber at channel `E` and at every sub-channel of `E`, whichever channel supplied them (ADR 0038 D3, O-1); `UnsubscribeAll` removes every registration. Both keep the publish cache in step (ADR 0038 D5). The unwatch half and the removal of a stopped subscriber are #1056.
- Pekko is the oracle (ADR 0033 D14): Pekko v1.7.1 (`9a54d12`), `docs/parity/oracles/eventstream-spec-2026-11-06.md`. Decisions: ADR 0038 D3, D4, D5, Amendments (a) and (b). Evidence classes of the manifest: `READ` (Pekko code, read), `ORACLE` (scenario executed on Pekko). The `PORT-GAP` and `moved` rows point here. `manifest.tsv` follows the kit runner contract (`source`, `target` first).
- Frozen API (change it only through the TL), package `actor`: `Unsubscribe[E any](s *System, subscriber Handle) (changed bool, err error)` and `UnsubscribeAll(s *System, subscriber Handle) error`. `changed` is Pekko's `diff.nonEmpty`: it can be true for a subscriber never registered (U1b), so it is not "the subscriber was removed". Both return `ErrNilSubscriber` for a nil or typed-nil handle. The stream, the cache and the closure table stay unexported.
- Layout: implementation in `pekko-actor/actor`; no `spi/`. `actor` imports no other goPekko module; `scripts/deps.allow` is untouched.
- Writable: exactly the slice table of the scope for the PR in hand. Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `pekko-actor/internal/`, another module, nor the scope file.

## 2. Slices

0 kit package; 0b oracle addendum EV13 (U1b, U3b); 1a `Unsubscribe`, `UnsubscribeAll`, cache, closure cleanup (U1, U1b, U2, U4, U5, U8, U9); 1b the rule and the timing, tests only (U3, U6, U7); 2 README, record, cost-log row (U10). Order 0, 0b, 1a, 1b, 2. **EV13 is observed before ANY production code (Owner's condition at G2)**; U1b and U3b stay "inferred" until then.

## 3. Decisions and known gaps

- D-2: a handle, not a `Ref[M]`, because a removal needs only the identity; no assignability check. D-3: the rule of O-1 is exposed as is (v3 cuts the delivery via `I` too). D-4: U9, the closure cleanup through `hasSubscriptions(id)` under the stream lock, is Go housekeeping with no Pekko counterpart; the index nodes stay. D-5: no barrier in the public API.
- U3b: the scope's reading is that `Unsubscribe[K]` does not stop the delivery of `C` to a subscriber registered on `I` only. If EV13 contradicts any inferred row (U1b or U3b), the Dev stops and reports before slice 1a.
- Not here: unwatch and removal of a stopped subscriber (#1056), `Publish` (#870), Typed command (#959), the debug log of `unsubscribe` (ADR 0038 Deviation 2). Unknown is an answer (kit rule 8): where neither an ADR nor the oracle decides, use the most conservative form and leave `// TODO(EVT-04-<topic>): <reason>`.

## 4. Mandatory Stops

Report to the TL and stop; never skip, delete or weaken a test:

1. A change outside the allowed files is needed (including `docs/adr/`, `scripts/deps.allow`, another module).
2. A test fails on the current code where the scope expects it to pass.
3. A slice projects over 400 authored lines (split it first).
4. A contract above (the frozen API, ADR 0038) must change.
5. EV13 contradicts U1b or U3b, or any production code would precede EV13.

## 5. Mutation rule

Each behavior test names a mutation in a "Mutations" table of its PR (behavior, mutation, commit, failing test), taken from the scope criteria table. Mutations are run by hand in a disposable worktree, never committed, and the named test must fail by assertion; no ordinary test may depend on a crash. U7 is probabilistic (`-count=100`; the report says how often it fails). A test that survives is fixed. Slices 0, 0b and 2 are docs only: no mutation applies.

## 6. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>` line per file derived from one Pekko file; `actor/eventstream.go` mixes `EventBus.scala` and `EventStream.scala` and carries none (the manifest records provenance), as in RULEBOOK section 6 of `eventstream-registration` and `eventstream-publish`. Checked by `.github/scripts/check-headers.sh`.
- go-specs for assertions; channels and gates, no `time.Sleep`, no polling, no `-race`; `-count=100` on the concurrent test (U7).
- Coverage at least 85% per module (never exclude production code); no `//nolint`; gocyclo at most 10.
- Every sentence of a PR that says something was observed names the oracle line or the assertion; every Pekko citation is re-opened by the Dev before the PR (path, line, content) and listed in the Evidence comment as `path:lines -> first line of the range`.
