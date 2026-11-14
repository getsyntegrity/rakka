# RULEBOOK: Publish events through the subscribed actors' mailboxes (unit eventstream-publish, #870 EVT-03)

Status: **approved at gate G2 by the Owner on 2026-11-07** for `docs/scopes/870-eventstream-publish.md` (D-1 to D-10). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 0.

## 1. Unit

- Capability: `Publish(s *System, event any) error` delivers an event by `Tell` to each subscriber registered for a channel the event's dynamic type matches, once each (ADR 0038 D1 to D3); it adds the publish cache of Pekko and publishes the runtime's `DeadLetter` events to the stream. It unsubscribes and watches nothing (#905, #1056).
- Pekko is the oracle (ADR 0033 D14): Pekko v1.7.1 (`9a54d12`), `docs/parity/oracles/eventstream-spec-2026-11-06.md`. Decisions: ADR 0038 D1, D2, D3, D5, D7, D8 and Amendment (a), ADR 0036 EVT-1. Evidence classes of the manifest: `READ` (Pekko code, read), `ORACLE` (scenario executed on Pekko). The `PORT-GAP` row points here. `manifest.tsv` follows the kit runner contract (`source`, `target` first).
- Frozen API (change it only through the TL): `Publish(s *System, event any) error` and `ErrNilEvent` in package `actor`. The stream and the cache stay unexported. `SubscribeDeadLetters` keeps its signature and gains a `Deprecated:` paragraph (D-4).
- Layout: implementation in `pekko-actor/actor`; no `spi/`. `actor` imports no other goPekko module; `scripts/deps.allow` is untouched.
- Writable: exactly the slice table of the scope for the PR in hand. Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, another module, nor the scope file.

## 2. Slices

0 kit package; 0b oracle addendum EV11; 1a `Publish` and cache (P1 to P4); 1b delivery tests (P5 to P7); 2 `DeadLetter` events, cycle rule, deprecation (P8 to P10); 3 README, record, cost-log row (P11). Order 0, 0b, 1a, 1b, 2, 3. Details and files: the scope and `RUNBOOK.md`.

## 3. Decisions and known gaps

- D-3 / P9: a `DeadLetter` whose `Msg` is a `DeadLetter` reaches the hub callbacks and is not published to the stream again. Pekko is predicted to recurse (`Mailbox.scala:560-568` re-enters `ActorRef.scala:739-743`); EV11 observes it. If Pekko differs in a way that matters, the Dev stops and reports before slice 2. Marker `// TODO(EVT-03-deadletter-cycle): <reason>` (kit rule 8, "unknown is an answer").
- D-8: the mutation of P9 fails by a fatal stack overflow. This holds only because mutations are run by hand in a disposable worktree (Owner's condition at G2); no ordinary test may depend on a crash. The guard test passes or fails without crashing the binary.
- D-10: the delivery closure is keyed by `Handle.ID()`, first registered kept; a foreign `Ref` reusing an ID with another `M` is unsupported.
- Not ported: the `sys == null` branch of `EventStream.publish` (`EventStream.scala:51-54`). Not here: `Unsubscribe` (#905), removal of stopped subscribers (#1056), sender of `DeadLetter` (#871, #906), migration of callers (#1426), Typed commands (#959).

## 4. Mandatory Stops

Report to the TL and stop; never skip, delete or weaken a test:

1. A change outside the allowed files is needed (including `docs/adr/`, `scripts/deps.allow`, another module).
2. A test fails on the current code where the scope expects it to pass.
3. A slice projects over 400 authored lines (split it first).
4. A contract above (the frozen API, ADR 0038) must change.
5. EV11 shows Pekko behaving differently from the prediction in a way that matters to P9.

## 5. Mutation rule

Each behavior test names a mutation in a "Mutations" table of its PR (behavior, mutation, commit, failing test), taken from the scope criteria table. The mutation is applied to production code in a disposable worktree by hand, never committed, and the named test must fail by assertion (P9 by crash, D-8; P7 is probabilistic); a test that survives is fixed. Slices 0, 0b and 3 are docs only: no mutation applies.

## 6. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>` line per file derived from one Pekko file; a file that mixes several Pekko sources and has no header yet (`actor/eventstream.go`: `EventBus.scala` and `EventStream.scala`) carries none and the manifest records provenance, as in `eventstream-registration` RULEBOOK section 6; `actor/deadletter.go` **keeps its single existing line** (`// Ported from upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/ActorRef.scala`, `deadletter.go:10-12`), `check-headers.sh` requires exactly one, and the `Mailbox.scala` provenance of the slice 2 change lives in the manifest only. Checked by `.github/scripts/check-headers.sh`.
- go-specs for assertions; channels and gates, no `time.Sleep`, no polling, no `-race`; `-count=100` on the concurrent test (P7).
- Coverage at least 85% per module (never exclude production code); no `//nolint`; gocyclo at most 10.
- Every sentence of a PR that says something was observed names the oracle line or the assertion; every Pekko citation is re-opened by the Dev before the PR (path, line, content) and listed in the Evidence comment as `path:lines -> first line of the range`.
