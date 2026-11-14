# RULEBOOK: Typed StashBuffer and `WithStash` (unit typed-889-stash-buffer, #889 TYP-13)

Status: **approved at gate G2 by the Owner on 2026-11-10** for `docs/scopes/889-typed-stash-buffer.md` (D-1 to D-7 as written). Read-only inside any loop (kit `CLAUDE.md` rule 1); it does not change after PR 0.

## 1. Unit

- Capability: a bounded, actor-confined `StashBuffer[T]` (`IsEmpty`, `NonEmpty`, `Size`, `Capacity`, `IsFull`, `Stash`, `Head`, `ForEach`, `Exists`, `Clear`, function `Contains`) and its only constructor `WithStash(capacity, factory)`. `Stash` on a full buffer returns `ErrStashOverflow` and keeps what it holds; `Head` of an empty buffer returns `ErrStashEmpty`. No replay: `unstash` and `unstashAll` are #927; `Restart` stash is #999; `Stopped` is #1182.
- Governing decisions: ADR 0033 D13, D14.1 to D14.3; ADR 0035 and its amendment of 2026-11-10 (the six adaptations); ADR 0011 and EXP-1 (each export justified in the scope "Open API choices"); scope D-1 to D-7.
- Pekko v1.7.1 (`9a54d12`) is the oracle by reading; no Pekko run exists for this unit, so **not V3**. Evidence classes: `SPEC` (a Pekko spec row, read, not run), `READ` (Pekko code, read), `ALT` (no Pekko test: alternative evidence, ADR 0033 D13). `manifest.tsv` follows the kit runner contract (`source`, `target` first). `PORT-GAP` rows noted "accepted adaptation, ADR 0035 amendment 2026-11-10" are decided; `PORT-GAP(stash-unstash-927)` and `PORT-GAP(stash-spec-927)` point to #927.
- Kit run (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`): port plus adaptation, kit at `cf91c9d`. The Dev is a separate session, QA is independent of both, the TL does not implement.
- Layout: everything in `pekko-actor-typed/typed/`; `typed` keeps importing only `pekko-actor` (`scripts/deps.allow` untouched).

## 2. Slices and allowed files

Paths under `typed/` are in `pekko-actor-typed/`. Anything else is a Stop.

| Slice | Criteria | Allowed files |
|---|---|---|
| 0 | kit package | `migration/stage-1/typed-889-stash-buffer/{manifest.tsv,RULEBOOK.md,RUNBOOK.md}` |
| A | S1 to S7c | `typed/stash.go` (new), `typed/stash_test.go` (new, `package typed`), `typed/unhandled_internal_test.go` (only the `want` list of exported names in `TestUnhandled_U9_PublicSurface`; amended 2026-11-10 after Dev stop 3) |
| B | S8, S9, S10 | `typed/stash.go` (`WithStash`), `typed/stash_withstash_test.go` (new), `typed/example_test.go`, `typed/unhandled_internal_test.go` (the `want` list: `WithStash`), `pekko-actor-typed/README.md`, `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row |

`typed/behavior.go` only as an escape hatch (to place `WithStash` beside `Setup`); the plan is not to touch it. `docs/parity/actor-inventory.md` is not touched without an Owner decision. `stash.go` is edited by A, then B: never in parallel.

## 3. Rules for the Dev

- D-1: capacity is not validated (0 is full, negative is never full). D-2: a zero `T` is a message; no `ErrNilMessage`. S7a and S7b are labeled "no Pekko oracle test: alternative evidence".
- D-4: `StashBuffer[T]` is a struct pointer with unexported fields; **no exported constructor**; no locks (actor-confined, not safe for concurrent use); `ForEach` re-entrancy is documented as unsupported. Tests use `errors.Is`, never message text.
- D-5: the representation is the Dev's choice; it must clear a dropped slot for the garbage collector, and removal of the head should stay constant time for #927 (INFERRED from #927).
- D-7: `WithStash` is `Setup(func(ctx) { return factory(newStashBuffer(ctx, capacity)) })`; the captured `*Context[T]` stays unexported and unused outside S7c and S8. S7c pins a **`pekko-actor` invariant** (`pekko-actor/actor/spawn.go:156`, one `actor.Context` per cell), comparing `b.ctx.c` across turns; the usage rule of the captured context is #927's.
- Headers: Apache-2.0 plus exactly one `// Ported from upstream/pekko/actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala` line in `stash.go` (`.github/scripts/check-headers.sh`).
- Tests: go-specs matchers and `mock.Controller`/Spy (`AGENTS.md` "Test policy"); channels and gates, no `time.Sleep`, no `-race`, no real engine. Coverage at least 85% per module, no `//nolint`, gocyclo at most 10; the README follows the module README contract.

## 4. Mandatory Stops

Report to the TL and stop; never skip, delete or weaken a test:

1. A change in `pekko-actor` is needed (for example a context accessor the buffer cannot get from `typed.Context`).
2. A test fails on the current code where the scope expects it to pass; a failing scenario is run on Pekko before it is classified (`gopekko-failure-triage`).
3. A file outside the allowed list must change, or a slice projects over 400 authored lines (A about 340, B about 330; split first, README and record into a docs PR).
4. A decision above (D-1 to D-7, ADR 0035 amendment) must change, or a second export (a constructor) is needed.
5. `scripts/each_module.sh go build ./...` fails in another module.

## 5. Mutation rule

Each behavior test names a mutation in a "Mutations" table of its PR (behavior, mutation, commit, failing test), taken from the scope table: S1 `IsEmpty` always false; S2 `Head` removes; S3 `Head` returns zero and nil; S4 overflow drops the oldest or returns nil; S5 `ForEach` reversed, `Contains` always true; S6 `Clear` keeps the size; S7a `Stash` rejects capacity <= 0; S7b `Stash` rejects the zero value; S7c the buffer stores another cell's context; S8 `WithStash` passes nil or ignores `capacity`; S9 the matcher ignores `ErrStashOverflow`, or the restart keeps the old messages. Applied by hand in a disposable worktree, never committed; the named test must fail by assertion; a surviving test is fixed. S10 and slice 0 are docs.
