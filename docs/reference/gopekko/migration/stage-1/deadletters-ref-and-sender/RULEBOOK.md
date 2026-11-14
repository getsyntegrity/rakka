# RULEBOOK: Expose `deadLetters` and carry the sender in `DeadLetter` (unit deadletters-ref-and-sender, #896 MSG-08)

Status: **approved at gate G2 by the Owner on 2026-11-10** for `docs/scopes/896-deadletters-ref-and-sender.md` (D-1 to D-10 as written). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 0.

## 1. Unit

- Capability: `DeadLetter` gains `Sender Handle`; `(*System).DeadLetters() Ref[any]` returns the local dead-letter sink (Pekko `provider.deadLetters`, `ActorRefProvider.scala:69`). A sender absent at any publication site is `System.DeadLetters()` **in the dead-letter view only** (`ActorRef.scala:585-589`); `Context.Sender` and the envelope stay "absent" (ADR 0032 D3, D-2); changing that is another decision. ADR 0038 D7 (which story gives `DeadLetter` its sender) is amended by the editorial PR #1504 (scope D-10); no Go depends on it. The unit does not touch `Watch`/`Identify` on the sink, `SuppressedDeadLetter` (#871), `Dropped` (#906), logs (#1479), the ignore reference (#1087), `Forward` (#854), or callers (#1426).
- Pekko is the oracle (ADR 0017, ADR 0033 D14): v1.7.1 (`9a54d12`), `docs/parity/oracles/eventstream-spec-2026-11-06.md` (EV8 [006], [008]). Evidence classes of the manifest: `READ` (Pekko code, read), `SPEC` (a Pekko spec row, read, not run), `ORACLE` (scenario executed on Pekko). The `PORT-GAP` rows point here. `manifest.tsv` follows the kit runner contract (`source`, `target` first).
- Kit run (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`): port plus adaptation; kit pinned at `cf91c9d`. The Dev is a separate session, QA is independent of both, the TL does not implement.
- Layout: implementation in `pekko-actor/actor` (ADR 0011 owner decision 2); no `spi/`. `actor` imports no other goPekko module; `scripts/deps.allow` is untouched.
- Writable: exactly the slice table and "Allowed files" of the scope for the PR in hand. Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, another module, nor the scope file.

## 2. Slices

0 kit package; 1a the sender is carried (S1, S2); 1b `DeadLetters()`, substitution, literal migration (S3 to S8; a 1c for the literals if 1b passes 400 lines); 2 README, record, cost-log row (S9). Order 0, 1a, 1b, 2. Files per slice: the scope and `RUNBOOK.md`.

## 3. Rules for the Dev (G2 decisions)

- D-2: `Sender` is never nil in a published `DeadLetter` **from slice 1b** (slice 1a carries the explicit sender and leaves it nil when absent). The substitution happens once, in `System.publish` (`deadletter.go:39`); the eight sites pass the sender they hold or nil (sites: scope "Sites found").
- D-3 / EXP-1: the only new exports are `DeadLetter.Sender` and `(*System).DeadLetters()`, each justified in the scope "Open API choices" (consumers, ADR 0011 benefit, alternative) and in the README by slice 2. The sink type and the sender-aware `TellFrom` path (an unexported interface checked beside `ref.go:55`) stay unexported. No other export; a need for one is a Stop.
- D-5: `watch.go:308` carries the watched subject as `Sender` only if the 814 delivery holds it there; otherwise `deadLetters` and `// TODO(MSG-08-terminated-sender): <reason>`. The Dev reads `watch.go:296-312` and reports.
- D-6: `Watch`/`Identify` on the sink are not ported (`PORT-GAP(dl-watch-896)`); the Dev records in the S5 test what `Context.Watch(DeadLetters())` does today (UNVERIFIED until then). `ActorSelectionMessage` unwrapping is not ported.
- D-7: the sink's ID is minted outside the registry without colliding with `idCounter` (`id.go:20-26`); its path is `/deadLetters` (`ActorRefProvider.scala:414-417`) if ADR 0031 allows it, else a PORT-GAP with a marker `// TODO(MSG-08-deadletters-path): <reason>` (kit rule 8, "unknown is an answer"). The scope (D-7, approved as written) has the Dev propose both in slice 0; this RULEBOOK **moves that proposal to the slice 1b PR**, the one that creates the sink, because it needs the reading of `id.go` and of ADR 0031 that slice 0, a documents-only slice, does not do. The Owner approved the move on 2026-11-10.
- The cycle rule of #870 is unchanged (S6): a `DeadLetter` whose recipient is the sink is not re-published. A `DeadLetter` told to the sink is published as is (`ActorRef.scala:739`).
- Existing literals migrate with the same assertions on `Msg` and `To` (S8); `sender_test.go:238` (S8 of ADR 0032) pins the dropped sender and changes in 1a.

## 4. Mandatory Stops

Report to the TL and stop; never skip, delete or weaken a test:

1. A change outside the allowed files is needed (including `docs/adr/`, `scripts/deps.allow`, another module).
2. A test fails on the current code where the scope expects it to pass; a scenario that fails is run on Pekko before it is classified (`gopekko-failure-triage`).
3. A slice projects over 400 authored lines (split it first).
4. A contract above (D-2, D-3, ADR 0032) must change, or a third export is needed.
5. `pekko-actor-typed` or `pekko-persistence` stops compiling (`scripts/each_module.sh go build ./...`).

## 5. Mutation rule

Each behavior test names a mutation in a "Mutations" table of its PR (behavior, mutation, commit, failing test), taken from the scope criteria table (S1 `tell` publishes `Sender: nil`; S2 `stop.go:123` drops `e.sender`; S3 `publish` does not substitute; S4 a `DeadLetter` is wrapped again; S5 the sink is registered; S7 `publish` reuses the previous sender). The mutation is applied by hand in a disposable worktree, never committed, and the named test must fail by assertion; S7 runs `-count=100`; a test that survives is fixed. S6 and S8 are regressions (no mutation); slices 0 and 2 are docs only.

## 6. Conventions

- Header: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>.scala` line per file derived from one Pekko file (`.github/scripts/check-headers.sh`); `deadletter.go` **keeps its single existing line** (`ActorRef.scala`, `deadletter.go:10-12`) and the `AbstractDispatcher.scala` provenance lives in the manifest only; `deadletters_ref.go` (new) carries `ActorRef.scala`; test files carry the line of their one Pekko spec or none when they mix several (manifest records provenance).
- Tests: go-specs matchers and `mock.Controller`/Spy for doubles (CLAUDE.md, AGENTS.md "Test policy"); channels and gates, no `time.Sleep`, no polling, no `-race`.
- Coverage at least 85% per module (never exclude production code); no `//nolint`; gocyclo at most 10; README follows the module README contract.
- Every sentence of a PR that says something was observed names the oracle line or the assertion; every Pekko citation is re-opened by the Dev before the PR and listed in the Evidence comment as `path:lines -> first line of the range`.
