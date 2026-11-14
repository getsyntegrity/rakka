# RULEBOOK: Remove the registrations of a subscriber that terminated (unit eventstream-termination, #1056 EVT-17)

Status: **approved at gate G2 by the Owner on 2026-11-07** for `docs/scopes/1056-eventstream-termination.md` (D-1 to D-7, with conditions on D-2 and D-3; D-8 rejected). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 0.

## 1. Unit

- Capability: when a subscribed actor of the System stops, the stream removes every registration it owns, **asynchronously**; when a subscriber unsubscribes from its last channel the stream stops watching it. It exports nothing new (no frozen API beyond `Subscribe`, `Unsubscribe`, `UnsubscribeAll` of earlier units); the watcher, the hook and the internal tests stay unexported.
- Pekko is the oracle (ADR 0033 D14): Pekko v1.7.1 (`9a54d12`), `docs/parity/oracles/eventstream-spec-2026-11-06.md` (S7, EV7; EV12 pending). Decisions: ADR 0038 D6. Evidence classes of the manifest: `READ` (Pekko code, read), `ORACLE` (scenario executed on Pekko). The `PORT-GAP` and `moved` rows point here. `manifest.tsv` follows the kit runner contract (`source`, `target` first).
- Layout: `pekko-actor/actor` only; no `spi/`, no edit of `registry.go`, `watch.go` or `stop.go`. `actor` imports no other goPekko module; `scripts/deps.allow` is untouched.
- Writable: exactly the slice table of the scope for the PR in hand (internal test files allowed). Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `pekko-actor/internal/`, another module, nor the scope file.

## 2. Slices

0 kit package; 0b oracle addendum EV12 (T7, another Dev); 1a the termination watcher (T1 to T4, T4b; split if above 400); 1b unwatch, concurrency, stopping System (T5 to T7); 2 README, record, cost-log row (T8, T9). Order 0, 0b, 1a, 1b, 2. **Slice 1b waits for EV12**; T7 stays pending until then (ADR 0033 D13).

## 3. Owner's conditions (G2) and deviations

- **D-2 condition.** The stream's internal `watcherLink` (ID minted per stream, registered with `addWatcher`, notified by `finalize`) must preserve every observable of ADR 0038 D6: removal is asynchronous, no window is promised or asserted (no count of dead letters inside it), and an already stopped subscriber is registered, `Subscribe` returns `true`, and it is removed asynchronously after `Subscribe` returned (both paths: refused `addWatcher`, and a retired record; a `Ref` of another System is resolved with `owner().lookup`). **If D6 cannot be followed in Go, STOP and report to the TL; no ADR amendment is written** without the Owner's OK. D-8 (remove inside `Subscribe`) was rejected: it contradicts D6.
- **D14.3 section, D-2 (minor).** Pekko's unsubscriber is an actor under `/system`; goPekko has no `/system` actor (#1055) and the stream's link replaces it. The only visible difference is the absence of that actor in the tree. When #1055 lands, do not migrate the link to an actor unless the migration changes something observable.
- **D14.3 section, D-3.** A foreign `Ref` (not an `ownedHandle`: no cell, no owner System) cannot be watched; `Subscribe` accepts it and the stream never retires it automatically. Observable as a retained subscriber and a dead letter per publication after it stopped; its registrations stay until `Unsubscribe`/`UnsubscribeAll`. Written in the doc comment, in `EXECUTION-RECORD.md` and in one line of the `pekko-actor` README (slice 2); T8 is a review item.
- D-4: `Subscribe` on a stopping System follows the observation of EV12, confirmed by the Owner before slice 1b; the default stays as merged (not refused, `TODO(EVT-02-stopping)`). D-5: the removal order among subscribers is unspecified. D-6: no count inside the window. Unknown is an answer (kit rule 8): use the most conservative form and leave `// TODO(EVT-17-<topic>): <reason>`.
- Not here: `/system` actor (#1055), `Unsubscribe` (#905), `Publish` (#870), the debug logs of the unsubscriber (ADR 0038 Deviation 2), the cross-system watch machinery.

## 4. Mandatory Stops

Report to the TL and stop; never skip, delete or weaken a test:

1. A change outside the allowed files is needed (including `stop.go`, `watch.go`, `registry.go`, `docs/adr/`, `scripts/deps.allow`, another module).
2. A test fails on the current code where the scope expects it to pass.
3. A slice projects over 400 authored lines (split it first; 1a splits as the scope says).
4. ADR 0038 D6 cannot be followed in Go (section 3), or EV12 contradicts the default of D-4.
5. Slice 1b would start before EV12 is merged.

## 5. Mutation rule

Each behavior test names a mutation in a "Mutations" table of its PR (behavior, mutation, commit, failing test), taken from the scope criteria table. Mutations are run by hand in a disposable worktree, never committed, and the named test must fail by assertion. T6 is probabilistic (`-count=100`; the report says how often it fails). No test asserts a count of dead letters inside the window. A test that survives is fixed. Slices 0, 0b and 2 are docs only: no mutation applies.

## 6. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>` line per file derived from one Pekko file. `eventstream_watch.go` adapts an actor's behavior and `eventstream.go` mixes sources: both carry none and the manifest records provenance, as in the previous packages. Checked by `.github/scripts/check-headers.sh`.
- go-specs for assertions; channels and gates (the fence is `<-System.Done(ref)`; an unexported seam holds the asynchronous removal on a gate), no `time.Sleep`, no polling, no `-race`; `-count=100` on the concurrent test (T6).
- Lock order: stream lock, then registry lock (`s.mu`), then the cell lock; the callback takes the stream lock with no cell lock held.
- Coverage at least 85% per module (never exclude production code); no `//nolint`; gocyclo at most 10.
- Every sentence of a PR that says something was observed names the oracle line or the assertion; every Pekko citation is re-opened by the Dev before the PR (path, line, content) and listed in the Evidence comment as `path:lines -> first line of the range`.
