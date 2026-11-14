# Execution record: injectable System clock (#1129, TIM-11)

Scope: `docs/scopes/1129-injectable-clock.md` (a4284cda). Kit run: not required (scope section 7). Squad A.

## Decisions applied (Owner, scope section 0)

D1 = 1 (`Config.Clock`, `NewSystem` unchanged), D2 = 1 (exported `Clock` and `Timer`), D3 = 1 (no manual clock, no wall-clock constructor). **D4 (ADR 0033 D13 alternative evidence) was open when this record was written; it is now resolved**: the Owner accepted O1-O5 in the message of 2026-10-29 (O2 exclusively for P1/P2: no judge run, no S04/S15, no V3, no general parity), see section 9 of `docs/scopes/1129-clock-evidence-closure.md` and "Post-merge evidence" below.

## Evidence, by category

1. Executed in Go: `pekko-actor/actor/clock_api_test.go` (AC-R1-2, AC-R2-1, AC-R2-2); the existing Ask tests and `clock_internal_test.go` unchanged. RED was observed first (compile failure: `actor.Timer` and `Config.Clock` undefined).
2. Read from Pekko: nothing is ported. `testkit/.../ExplicitlyTriggeredScheduler.scala` (`timePasses`) was read as the analogue of a manually driven time; it is a scheduler, out of scope.
3. Differential scenario against Pekko: pending; no Pekko oracle exists for a Go seam.
4. Alternative evidence: the public scenarios above; D4 was resolved by the Owner on 2026-10-29 (see line 7 and section 9 of the scope).

## Surface

`Clock`, `Timer`, `Config.Clock`. Not exported: manual clock, wall-clock constructor, `Reset`, `Since`, schedulers. `ask.go` untouched; `clock` and `timer` stay as aliases.

## Post-merge evidence (Refs #1129)

Category 3 is no longer "pending": P1 (deadline fires only when the clock advances) and P2 (cancellation when the reply wins) were observed on Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`) with `ExplicitlyTriggeredScheduler`; see `pekko-repro/ClockRepro.scala`, `pekko-repro/OBSERVED.txt` and `docs/scopes/1129-clock-evidence-closure.md` (sections 4, 5, 9). P3 is a documented non-equivalence. Per the mensaje del Owner de 2026-10-29: O1 accepted (Go evidence `TestClockAPI_SystemReadsTheSuppliedClock`); O2 accepted exclusively for P1 and P2 compared by the section 4 tuples (not a judge-comparator run, not S04/S15 on the Go side, not V3 or general parity; "Stop exactly once" is additional Go evidence); O3 accepted (existing Ask tests); O4 confirmed (`TestRealClockFiresOnceAndStopPreventsIt` plus the System's use of `Stop`); O5 confirmed (EXP-1 records of #1267; #1020 is an index and does not block). #1129 is neither closed nor merged. Categories 1, 2 and 4 unchanged.
