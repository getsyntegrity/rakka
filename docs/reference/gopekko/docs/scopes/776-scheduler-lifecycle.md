# Scope: Specify scheduler lifecycle and cancellation (#776, TIM-01)

Status: **approved at G2 by the Owner, 2026-11-10**, together with the acceptance of ADR 0039 (O-1 to O-10 as recommended, with the Owner's precisions recorded in the ADR). First written as a draft for G2 on 2026-11-09; Written by the TL on 2026-11-09 against `origin/develop` `46ce37ac`. Pekko baseline: v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. There is no pitch (`docs/pitch/` is empty); like #870, #836 and #1129 this scope is written from the issue, its ACT2 reconciliation and the oracle sources.
**Evidence labels.** READ = source opened in `upstream/pekko` at the pin, not executed. No code and no Pekko run is part of this scope.
**Kit run: NOT required (Owner may overrule).** The unit produces an ADR (Accepted by the Owner on 2026-11-10) and one README note and **no Go**; it ports no Pekko behavior and adapts no Pekko source file. The rule for ports and adaptations (ADR 0017 "Delivery units", ADR 0024 T6 (`docs/adr/0024-actor-registry-retention-after-termination.md:258-259`), `AGENTS.md` "Kit runs") applies to the stories that implement the contract (#821, #822, #823, #1131, #860), which the TL marks "kit run required" when it scopes them. Precedent: `docs/scopes/1129-injectable-clock.md:6`. If the Owner reads the ADR as a port of `LightArrayRevolverScheduler`, say so at G2 and the Dev runs the kit.

## Objective

A reviewable ADR 0039, status **Accepted** (Owner, 2026-11-10), that specifies `scheduleOnce`, fixed delay and fixed rate as three contracts, `Cancellable` semantics and races, scheduler close, the clock and executor seams and the rules that constrain the keyed timer, with every Pekko rule cited by `path:line`, and that isolates every Owner decision in "Decisions for the Owner". It claims no scheduler and no parity.

## Contracts consumed

- `pekko-actor/actor/clock.go` (`Clock`, `Timer`) and `Config.Clock` at `system.go:99-102` (#1129), as the clock seam; `executor.go` as the (unexported) executor seam; `system_shutdown.go:30-66` for the termination the close hangs on.
- ADR 0011 EXP-1 (`docs/parity/typed-owner-decisions-2026-10-27.md:36`), ADR 0033 D3, D4, D5, D13, D14.
- Pekko: `Scheduler.scala`, `LightArrayRevolverScheduler.scala`, `dungeon/TimerSchedulerImpl.scala`, `SchedulerSpec.scala`, `TimerSpec.scala` (paths in the ADR).

## Criteria to scenarios

Documents only, so each criterion is a check a reviewer runs on the files. "Check" is a command or a reading step; no test is written.

| Criterion (issue AC / reconciliation) | Check | Level |
|---|---|---|
| AC1. scheduleOnce, fixed delay and fixed rate defined separately | ADR section "How Pekko solves it" P1 has one column per operation; each cell carries a `Scheduler.scala` or `LightArrayRevolverScheduler.scala` line; `grep -c "S:\|L:"` is not zero in every row | doc |
| AC2. Cancellation races and close, from upstream cases | P2 cites `SchedulerSpec` :127-154, :316-346; P5 cites :520-548, :661-697 and `L:170-247`; open each cited range at the pin and read that it says what the ADR says | doc |
| AC3. Clock and executor seams; dispatcher or dedicated work | ADR O-4 (executor) and C1/O-7 (clock) state what exists today (`clock.go`, `executor.go`) and the options; Pekko evidence L:383-392, TI:103-107 | doc |
| AC4. Alternatives, selected proposal, observable differences, Owner decisions; Accepted only by the Owner | the file header says `Status: **Accepted**` with the Owner's date and the Owner table holds his literal text; a "Decisions for the Owner" section exists with O-1 to O-10, each with question, options, recommendation and evidence; the Deviations are marked | doc |
| AC5. Public contract and traceability updated, no sibling claimed | `pekko-actor/README.md` has the note below and nothing that says a scheduler exists; the ADR "Not covered" lists #1077, #1131, #861, Typed | doc |
| ACT2: non-positive interval rejected | ADR P1 table row and P3; cites SS:701-737, `L:224-228` | doc |
| ACT2: periodic task that throws aborts | ADR P4 and C6; cites S:75-76, SS:270-314 | doc |
| ACT2: close rejects new, pending never run | ADR P5 and C7; cites L:170-207, SS:525-545 and :684-697 | doc |
| ACT2: `SchedulerTask.cancelled` is #1077's | ADR P2 last bullet and "Not covered" | doc |
| ACT2: exports justified individually | ADR "Exports" section: no export approved, names deferred to the introducing stories (EXP-1) | doc |
| Anchors are true | the reviewer opens every `path:line` of the ADR at `upstream/pekko` (`git -C upstream/pekko rev-parse HEAD` must equal the pin) and reports any that does not say what the ADR claims; `.github/scripts/check-headers.sh` is not affected (no Go) | doc |

The ADR was accepted by the Owner on 2026-11-10; the sibling stories (#821, #822, #823, #860, #1131) stay open, and #821 waits for the oracle run of O-2/O-3 (`docs/parity/oracles/scheduler-lifecycle-2026-11-10.md`).

## Proposed README note (for `pekko-actor/README.md`)

One sentence appended to the "Injectable clock" bullet (`README.md:298-305`), which today ends "No Pekko parity is claimed (a Go seam has no oracle; ADR 0033 D13 evidence is pending Owner approval)." The bullet at :303 already says "no scheduler or timer other than Ask's is armed"; the note adds only the pointer:

> The contract of a future scheduler (one-shot, fixed delay, fixed rate, `Cancellable`, close) is specified in [ADR 0039](../docs/adr/0039-scheduler-lifecycle-and-cancellation.md), **Accepted** (Owner, 2026-11-10); no scheduler or `Cancellable` API exists yet in this module.

The README states the scheduler nowhere else: `grep -n -i "sched" pekko-actor/README.md` finds the clock bullet (:303) and unrelated words such as "scheduled" (:363, :385). The note is the only change to the file.

## Allowed files

```
docs/adr/0039-scheduler-lifecycle-and-cancellation.md
docs/scopes/776-scheduler-lifecycle.md
pekko-actor/README.md                          (the one sentence above)
```

`docs/actor-typed-delivery-plan.md` is **not** touched: row C-02 (:273) stays "U"; its wording "cancel true only if it prevented execution" is flagged in ADR O-2 and corrected only if the Owner accepts that decision. Nothing else may change.

## Collisions and independence

| Scope | Shared file | Handling |
|---|---|---|
| #896 (dead letters, `docs/896-dead-letters-scope`) | `pekko-actor/README.md` | Merge order matters. This scope edits one sentence at the end of one bullet (:303-305); #896 edits the dead-letter text elsewhere. Whichever merges second rebases; no paragraph is reflowed. If #896 is first, re-anchor on the clock bullet; if this one is first, #896 rebases over one added sentence. |
| #821, #822, #823, #1131, #860 | none today | They consume the ADR once accepted; they edit Go and the README after this one. |

## Appetite

One half-day of TL work plus the Owner's reading. Size: ADR 199 lines, this scope 71, README 3 lines: **273 authored changed lines, within the 400 budget** (`AGENTS.md`). No split. No generated, vendored or lock files.

## Definition of Done

- Every row of "Criteria to scenarios" is checked by someone other than the author; the anchor check is done against the pin, not a drifted checkout.
- The ADR status is Accepted (Owner, 2026-11-10); the Owner table records the decision; O-1 to O-8 each carry question, options, recommendation and evidence; O-9 (evidence rule) and O-10 (acceptance and split) are procedural and carry a recommendation only.
- No Go, no test, no change to `go.mod`, to `docs/actor-typed-delivery-plan.md` or to any label or issue. `scripts/check_deps.sh` and the Go gates are unaffected and are not run for a docs-only change.
- Anything the author could not verify is listed in the PR body (see the ADR's evidence line: all Pekko rules are READ; the periodic `cancel()` result and the one-shot zero delay want an oracle run, O-2, O-3, O-9).
