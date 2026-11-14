# RULEBOOK: `DeadLetterSuppression`, `SuppressedDeadLetter` and divergence 5a (unit suppressed-dead-letters, #871 EVT-05)

Status: **approved at gate G2 by the Owner on 2026-11-10** for `docs/scopes/871-suppressed-dead-letters.md` (D-1 to D-10 with the amendments of PR #1522). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 0.

## 1. Unit

- Capability: a message that implements `DeadLetterSuppression` has its dead letter published as `SuppressedDeadLetter{Msg, Sender, To}` (not `DeadLetter`), to the event stream only; `AllDeadLetters` is the channel of both. `PoisonPill` and `Terminated` carry the marker, `Kill` does not. Slice 2 closes divergence 5a (an undelivered `PoisonPill` publishes the event). Not in the unit: `ActorSelection`, `WrappedMessage` and the listener skip (#1479), `Dropped` (#906), `Watch`/`Identify` on the sink, #770.
- Pekko is the oracle (ADR 0017, ADR 0033 D14): v1.7.1 (`9a54d12`) and `docs/parity/oracles/suppressed-dead-letters-2026-11-10.md` (#1508); the oracle observations are the contract; `DL8b` is a race and is not used. Evidence classes of the manifest: `READ`, `SPEC` (a Pekko spec read), `ORACLE` (a scenario executed on Pekko). The `PORT-GAP` rows point here.
- Kit run (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`): port plus adaptation, kit pinned at `cf91c9d`. The Dev is a separate session, QA is independent of both, the TL does not implement.
- Layout: implementation in `pekko-actor/actor` (ADR 0011 owner decision 2); no `spi/`; `actor` imports no other goPekko module.
- Writable: exactly the slice table and "Allowed files" of the scope for the PR in hand. Nothing under `docs/adr/` (before slice 2), `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, another module, nor the scope file.

## 2. Rules for the Dev (G2 decisions)

- D-2: the marker is an exported interface with an exported method `DeadLetterSuppression()`. D-3: `SuppressedDeadLetter{Msg DeadLetterSuppression, Sender Handle, To ID}`, `To` always the `deadLetters` ID. D-4: the event goes to the stream only; the deprecated hub and `deadLettersPublished` count `DeadLetter` only. D-5: `AllDeadLetters` is sealed.
- D-6: classification in `System.publish`, one funnel; no call site and no judge anchor is edited.
- D-7: `Terminated` is suppressed from slice 1. The tests that pinned it as a `DeadLetter` are corrected to what Pekko does (recipient `deadLetters`, oracle DL3 and DL7); no assertion is weakened: subject, sender and count are kept. The PR says so.
- D-9: the V level of inventory row C-10b is assigned by the Owner; no PR of this unit sets it.
- EXP-1: the only new exports are `DeadLetterSuppression`, `SuppressedDeadLetter` and `AllDeadLetters` (scope "Exports"); a need for another is a Stop.
- The full-mailbox `Terminated` (sender = subject) is inferred from `DeathWatch.scala:111`, not observed; PRs and tests say "inferred".

## 3. Mandatory Stops

Report to the TL and stop; never skip, delete or weaken a test:

1. A change outside the allowed files is needed (another module, `docs/adr/` before slice 2).
2. A test fails on the current code where the scope expects it to pass; a scenario that fails is run on Pekko before it is classified (`gopekko-failure-triage`).
3. A slice projects over 400 authored lines (kit artifacts included): split it first, spill-over rule of the scope.
4. A contract above (D-2 to D-7, ADR 0032, ADR 0038 D7) must change, or another export is needed.
5. `pekko-actor-typed` or `pekko-persistence` stops compiling (`scripts/each_module.sh go build ./...`).
6. A `judge/mutate_go.py` anchor breaks: the same PR updates it with the same contract; `scripts/check-judge-matrix.py` exit 1 is a Stop.

## 4. Mutation rule

Each behavior test names a mutation in the "Mutations" table of its PR (mutation, test that fails): M1 `publish` ignores the marker; M2 recipient kept as the original; M3 `Kill` carries the marker; M4 the guard `!it.poison` restored (slice 2); M5 `takePortDeadLocked` without `poison` (slice 2); M6 `Terminated` without the marker; M7 the suppressed event also sent to the hub. Applied by hand in the working tree, never committed; the test must fail by assertion.

## 5. Conventions

- Header: Apache-2.0 plus one `// Ported from upstream/pekko/<path>.scala` line per file derived from one Pekko file (`.github/scripts/check-headers.sh`); `deadletter.go` keeps its single existing line.
- Tests: go-specs matchers and spies, channels and gates, no `time.Sleep`; unit only.
- Coverage at least 85% per module; no `//nolint`; gocyclo at most 10; README per the module README contract (slice 2).
- Every sentence that says something was observed names the oracle line or the assertion; every Pekko citation is re-opened by the Dev before the PR.
