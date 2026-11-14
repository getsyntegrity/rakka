# RULEBOOK: Typed Terminated delivery and death pact (unit typed-954-slice-b, #954 TYP-04 Slice B)

Status: **G2 approved by the Owner on 2026-10-28** for `docs/scopes/954-slice-b-terminated-death-pact.md` (SHA `183171b9afdbf967bbbe73d79ecfa68a3b636a5f`, final text `4cc2c37`, PR #1249; Owner decisions L2 and L3). D-B1 option A (ID) is accepted, with the documented differences from Pekko; **this package does not publish the `Terminated` representation**. Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after B1.

## 1. Unit

- Capability: `OnTerminated` on the Typed interpreter (`pekko-actor-typed/typed/`): the active behavior's signal handler gets `Terminated`; a behavior with no handler gets the default Stop (death pact). Source: Pekko `Behavior.scala:270-277`, `ActorAdapter.scala:163-180,220-227` and `PropsAdapter.scala:34` at v1.7.1 (`9a54d12`), read, not run.
- Governing decisions: ADR 0033 D5, D13; ADR 0034 P5, P6, B19 (the P5/P6 amendment is B0b and is not yet accepted); scope D-B1 to D-B6.
- Manifest: `manifest.tsv` follows the kit runner contract (`migration-kit/scripts/queue_runner.mjs`): `source` and `target` are the first two columns and the only ones it reads. The extra columns are the disposition record. Moved and `PORT-GAP` rows point at this RULEBOOK.
- Writable per PR: exactly the "Allowed files" of the scope (section 7). Nothing under `pekko-actor/`, `docs/adr/` (except the separate B0b), `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum`, the root `README.md`.

## 2. Dispositions

- Go targets: `signal_term_bhv_test.go` (T1-T4, derived from `BehaviorSpec`, carries its `Ported from` line) and `signal_term_internal_test.go` (T5-T14, no `Ported from` line).
- `ActorContextSpec.scala:483` is **PORT-GAP**: an explicit decline of `Terminated` waits for #1181. `WatchSpec.scala:165` is **moved** to #1250 (`ChildFailed`). `SupervisionSpec.scala:1408` is **moved** to #976. Slice B ports none of them.
- T13 has no Pekko oracle (Go-runtime regression). T7b, T12 and T14 are Go-only READ rows. T15 and T16 are regression and API-diff checks, not spec rows.

## 3. Evidence labels

- Nothing was executed against Pekko. No V3. No Pekko parity claim. SPEC and READ rows are alternative evidence **pending Owner approval (ADR 0033 D13)**.
- Equivalence of a Go test to a Pekko spec is not claimed; each test states what it asserts and nothing more.

## 4. Rules for B2 and B3 (the later Dev)

- **Start condition:** B2 and B3 begin only after the Owner accepts the ADR 0034 amendment (B0b). Until then, stop.
- TDD: write each scenario, watch it fail (RED), implement the minimum, inside the allowed files. T7b confirms the exact `actor.Spawn` and `actor.OneForOne` call in RED.
- No `time.Sleep`, no polling, no `-race`; observe through channels and gates. T13 uses a fixed N and asserts the invariant. If T13 or T14 is nondeterministic in RED, stop and report; never weaken it.
- No new export except what the Owner accepted in D-B1; nothing from `package actor`. Internal tests are in `package typed`. No `//nolint`; cyclomatic complexity at most 10.
- **Decider not consulted:** a handler-less `Terminated` applies the default Stop (`StopSelf`, return nil). The parent's decider is not consulted (count 0), even when a custom decider would answer `Restart`. No hook, attach point or export for #976's future capture.
- Reuse the M-B rule for a failed turn (error, panic or non-receive result stops the actor). The interpreter adds no goroutine or lock (B19).
- Unknown behavior or a declined handler result: `// TODO(TYP-04-edge): <reason>` and keep going (kit rule 8). The D-B4 partial delivery is marked at its branch.
- Headers: Apache-2.0 plus exactly one `// Ported from upstream/pekko/<path>` line, only on a file derived from one Pekko file (`signal_term_bhv_test.go`: `BehaviorSpec.scala`); checked by `.github/scripts/check-headers.sh`.
- PR budget: at most 400 authored changed lines per PR (additions plus deletions, generated files excluded).

## 5. Stop conditions

A file outside the allowed list must change; `pekko-actor` needs an export; a scenario fails and is not a Go defect (run it on Pekko first); the line budget would be exceeded; the Owner has not accepted the ADR amendment. Report to the TL; never skip, delete or weaken a test.
