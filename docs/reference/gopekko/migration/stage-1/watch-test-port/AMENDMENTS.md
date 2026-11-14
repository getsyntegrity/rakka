# Amendments to the rulebook

The rulebook is read-only for the loop (kit standing rule 1). These are applied, or rejected, by the owner between batches.

**Status, 2026-10-10: DEV-5, DEV-6 and DEV-7 are approved by the owner as a record of deviations and limits of evidence.** They do not equal coverage and do not accept parity. Applied to `RULEBOOK.md` section 7. Still pending, not approved by this: the assertions of case 216 that DEV-6 lists as missing; the mutants and the calibration of the Watch guarantees; Linux.

| Id | Amendment (approved) | Source |
|---|---|---|
| DEV-5 | Add to the deviation log of `RULEBOOK.md`: case 268 ("discard Terminated when unwatched between sysmsg and processing") keeps only the clause "Unwatch inside a handler discards a Terminated already queued". Pekko enqueues `Terminated` when the watcher processes the notification, goPekko at the moment of the notification (ADR 0010, "Difference from Pekko in where Terminated is placed"), so the Scala schedule cannot be reproduced. The marker `PORT-GAP(dw-unwatch-by-message)` in `watch_test.go` is a decided difference, not an unknown | reviewer 1 of the loop, finding F1 (MAJOR), verified against `watch.go` and ADR 0010 |
| DEV-6 | Record that case 216 does not assert the cause of `brother`'s stop (`ErrDeathPact` has no public check) nor that the observer stays alive | reviewer 1, findings 2 and 3 |
| DEV-7 | Record that `GMstop` was re-anchored (text only, same defect) because the reschedule condition of `drain` gained the lane; its `definition_sha256` therefore differs from the campaign of `e9c6bac`. Also record that the S18 gate of the Go judge harness now releases once, because GMdup (duplicates the last chain message) made the second `close` panic and left S18 in timeout; assertions and criterion unchanged | `judge/mutate_go.py`; `pekko-actor/actor/judge_internal_test.go`; campaigns `8b64ad5` and `cc786e4` |

## Findings of the native reviews that are still open (non-blocking, not fixed)

Each review approved its slice. These findings were reported as advisory and are recorded here so that they are neither lost nor read as closed. None of them is coverage of a Watch guarantee, and none justifies a new mutation campaign by itself.

### Slice 2 (`8820ede`, review `review-7f5e4568585fb07b`): the port, to be fixed only through the migrator

| Id | Where | What | Route |
|---|---|---|---|
| R3-unchecked-watch-errors (WARNING) | `watch_test.go:44-52` | the results of `Watch`, `Unwatch` and `WatchWith` are discarded, so an `ErrInvalidHandle` would show only as a 30 s timeout | migrator loop; the owner installs the settings |
| R3-system-never-shut-down | `watch_test.go:81` | `dwSystem` never stops its `System` | migrator loop |
| R3-268-slow-failure-signal | `watch_test.go:289-305` | a regression of case 268 is reported by a 30 s timeout, not by its cause | migrator loop |
| R3-stale-amendment-status | `watch_test.go:254` and the STATUS trailer | the comment says DEV-5 is "queued for the owner"; it is approved | **comment to be updated by the migrator**; the ADR 0010 line is corrected in this slice |
| R3-cost-log-nonnumeric | `migration/cost-log.tsv:5` | free text in numeric columns | corrected in this slice |

### Slice 6: the port findings of slice 2, corrected through the migrator

A kit loop (one fixer, two adversarial reviewers, all `model: "sonnet"`, no third ruler needed) corrected, in `watch_test.go` only:

- `R3-unchecked-watch-errors`: **the premise was wrong**. `Watch`, `Unwatch` and `WatchWith` return nothing; a rejected call panics and the actor stops. The real weakness was a 30 s timeout with no cause. The waits on a registration now also observe the `Done` of the actor that must stay alive and fail at once, naming the cause.
- `R3-268-slow-failure-signal`: case 268 uses the same wait; a regression is now reported in about one second with its cause.
- `R3-system-never-shut-down`: `dwSpawn` registers a `t.Cleanup` that stops each spawned actor (there is no `System` shutdown API).
- `R3-stale-amendment-status`: the comment and the STATUS trailer now say DEV-5 is approved by the owner as a record of a deviation and of a limit of evidence, not coverage and not parity.

The assertions of the seven tests were not changed. This does not close the other debts of this file: the assertions that DEV-6 lists as missing in case 216, the mutants and the calibration of the Watch guarantees, and Linux.

Minor findings of the loop, not fixed:

| Id | Where | What |
|---|---|---|
| L6-1 | `watch_test.go:339` | the local `watching` channel in the `WatchWithCustomMessage` test shadows the package-level function `watching` of `watch_api_test.go` (legal; it was already so; touches R1.2) |
| L6-2 | `watch_test.go:84-90` | the comment says `Stop` "never blocks"; it is true with the default executor and the test one, which was not proven for an arbitrary executor |
| L6-3 | `watch_test.go` STATUS trailer | `ported=6 skipped=4 reduced=1 gaps=1` adds up (6 + 4 + 1 = 11 Scala cases) only because case 268 counts both as ported and as a gap; the file does not say so |

### Slice 3 (`8a9bfd1`, review `review-b1d5e0f453df2934`): judge, checker and selftests

| Id | Where | What |
|---|---|---|
| R2-stale-none-message (WARNING) | `scripts/check-judge-matrix.py:432` | the "none" message lists the causes twice and the first list is the old, narrower one |
| R3-weakened-invalid-assertion (WARNING) | `judge/selftest_go_report.py:171-173` | the check on the recorded report echoes the invalid count of the data instead of requiring zero; the zero-invalid guarantee rests on the HISTORICAL assertion of `selftest_backfill.py` |
| R3-current-selftest-couples-to-head (WARNING) | `judge/selftest_backfill.py:305-306` | the selftest requires CURRENT 11/11 on the real repository: any later change of an input of the campaign turns it red until the campaign is run again. This is the intended gate (HISTORICAL alone must not close it) and also a cost the owner should know |
| R4-001 (SUGGESTION) | `scripts/check-judge-matrix.py:432` | when CURRENT is none the checker does not say which input failed (tree, digest or toolchain) |
| R3-go-version-read-silent (SUGGESTION) | `scripts/check-judge-matrix.py:347` | a missing or already prefixed `.go-version` silently drops every row out of CURRENT |
| R2-comparator-name-widened (SUGGESTION) | `judge/mutate_go.py:111` | `COMPARATOR_PATHS` and the column `comparator_sha256` now cover the campaign script and the toolchain file: the name is narrower than the content. Renaming changes a column and needs the campaign again |
| R2-doc-superseded-count (SUGGESTION) | `docs/actor-core-parity.md:371` | the superseded run is described as "12 of 12 met" and the current one as "11 of 11": the same shape counted two ways |

### Slice 4 (`eaaadb8`, review `review-ba373a6372eb6b40`): documentation and cost log

| Id | Where | What |
|---|---|---|
| R3-001 (SUGGESTION) | `migration/cost-log.tsv:5` | the model column of the step-3 row is the selector `sonnet` while the other rows hold the full identifier `claude-sonnet-5-5`, so a tool that groups by model splits that row off; the note that the identity is self-reported lives only in `RULEBOOK.md`. `wall_clock_min` and `tokens` hold `unknown`, like the earlier rows, and `unknown` is not documented as a sentinel |

### Slice 5: corrected

The duplicated "none" message of CURRENT (`R2-stale-none-message`) and the weakened assertion of the recorded report (`R3-weakened-invalid-assertion`) are corrected in slice 5; the rest of the slice 3 list above stays open.

Still pending as before, and independent of these findings: the mutants and the calibration of the Watch guarantees, Linux, and the assertions of case 216 that DEV-6 lists as missing.
