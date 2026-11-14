# RULEBOOK: a Pekko mutation for clause `G5.f` (M66), detected by S22

Read-only while the work runs. Source of the behavior: Apache Pekko v1.7.1 (`upstream/pekko`, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `develop` at `f9a8838e7c972460771c7b49401684512237c846`, branch `feat/judge-g5f-pekko-mutation`. One responsible (the coordinator); no explorer, no implementer, no reviewer agent in this unit.

## 1. Why
`scripts/check-judge-matrix.py` reports `pending G5.f: no mutation is designed to break it`. S22 asserts `G5.f` (exact hook counts: `pre-start` 1, `pre-restart` 1, `post-restart` 0, `post-stop` 2) on the Pekko side and on the Go side, but no Pekko mutation has ever been designed to break it, so the clause has no observed detection on Pekko. This unit adds that mutation and runs it. It changes no scenario, no comparator, no classification and no Go runtime.

## 2. The Pekko behavior (source, read at the pin)
With the default hooks, a Stop that arrives while a restart waits for its children gives the old instance **two** `postStop` calls:
- P1: the default `preRestart` unwatches and stops each child and then calls `postStop()` (`actor/Actor.scala:625-631`).
- `faultRecreate` (`actor/dungeon/FaultHandling.scala:92-114`) runs `aroundPreRestart`, `clearActorFields(recreate = true)` (which keeps the old instance) and `setChildrenTerminationReason(Recreation(cause))`; with children still alive `finishRecreate` waits.
- A `Terminate` in the wait runs `terminate()` (`FaultHandling.scala:180-212`): `setChildrenTerminationReason(Termination)` replaces the reason, so when the last child is removed `handleChildTerminated` (`:327-347`) calls `finishTerminate`, not `finishRecreate`.
- P2: `finishTerminate` (`:237-262`) calls `a.aroundPostStop()` on the same old instance, then notifies the watchers once. No `postRestart`, no new instance.
S22 (`judge/scenarios/S22.json`, `judge/pekko-harness/Judge.scala`) exercises exactly this and requires `post-stop` of `c` count 2 (REQ and `G5.f`).

## 3. The mutation M66 (what it breaks, on the real path)
**M66: "A Stop that arrives while a restart waits for its children skips the PostStop of the termination (the second one)."** Four exact edits, all in `actor/dungeon/FaultHandling.scala`, each matching exactly once:
1. a private flag `m66Waiting` declared before `faultRecreate`;
2. in `faultRecreate`, `if (!setChildrenTerminationReason(Recreation(cause))) finishRecreate(cause)` gets `else m66Waiting = true` (the restart is waiting for children);
3. `finishRecreate` clears the flag at its start (a restart that completes is unaffected);
4. in `finishTerminate`, `a.aroundPostStop()` is guarded by `!m66Waiting`.
Effect: a plain stop, a restart that completes and every other scenario behave as before; only a Stop that overtakes a waiting restart loses P2. It is the Pekko-side image of the pre-U7 Go guard (the termination `PostStop` consumed by the one of the default `PreRestart`), applied to the real termination path. The edits are anchored on source text, not on a result. Expected: S22 (concurrent) `failed`, the clause `G5.f` among the fired rules because `c` shows 1 `post-stop` instead of 2. `G5.c` and `REQ` may fire as consequences (the second `post-stop` is missing); the designed expectation is `G5.f` only, as for the Go mutant GMdefaultps.

## 4. Rules
- R1. The mutation is applied only to a disposable `git archive` export of the pinned commit, committed and tagged `v1.7.1` in a throwaway repository (`sbt-dynver` needs one). `upstream/pekko` and `reference/` are never touched.
- R2. Do not weaken or change S22, any other scenario, the comparator (`comparator.py`, `evaluate.py`), the harness (`Judge.scala`), `go_scope.py`, `go-scope.json`, `mutate_go.py`, or any classification. A `timeout`, `harness-error`, `compile-error` or `incomplete` outcome is not a detection. If M66 does not give `failed` with `G5.f`, that is reported as it is; the edits are not tuned to obtain a result, and a second design is a decision.
- R3. No change to the Go runtime (`pekko-actor/`), the Go campaign inputs, or the Go report. If those inputs stay identical the Go campaign is not repeated; the checker must still report CURRENT 18/18 afterwards.
- R4. Only M66 rows are added to `judge/mutation-report.tsv` (30 concurrent runs; the runner enforces it); every other row stays byte-identical. The campaign runs on committed `judge/` (no `+dirty`).
- R5. Registration points (and nothing else): `judge/mutate.py` (entry `M66`), `judge/mutations/M66.patch` (generated; the other patches must not change), `judge/mutation-report.tsv`, `docs/judge.md` (section 6 row; the S22 paragraph that says `G5.f` has no mutation), `migration/stage-1/milestone-1.md` (the matrix rows that list mutations), `docs/actor-core-parity.md` (the list of Pekko mutations), and the selftest enumeration that the new report row necessarily changes (`judge/selftest_calibrate.py`: `choose()` now also picks S22; recorded as a deviation, no check removed).
- R6. The records say what was run, the exact commit of `judge/` at run time, the toolchain (Java 11 for sbt, the JDK 8 of `JAVA_8_HOME`), the pinned Pekko commit and the patch sha256.

## 5. Surfaces
Allowed to change: `judge/mutate.py`, `judge/mutations/M66.patch`, `judge/mutation-report.tsv`, `judge/selftest_calibrate.py`, `docs/judge.md`, `docs/actor-core-parity.md`, `migration/stage-1/milestone-1.md`, `migration/stage-1/judge-g5f-pekko/`.
Byte-identical (compared with sha256 before and after): `judge/comparator.py`, `judge/evaluate.py`, `judge/go_scope.py`, `judge/go-scope.json`, `judge/scenarios/`, `judge/pekko-harness/`, `judge/mutate_go.py`, `judge/go-mutation-report.tsv`, `judge/mutations/M1..M65.patch`, `pekko-actor/`, `scripts/check-judge-matrix.py`, `.go-version`, the submodule pins.

## 6. Declared limits
Pekko-side only, one run of 30 concurrent runs on the author's machine (darwin/arm64, JDK 11 for sbt and JDK 8 for the harness): it is observed evidence for `G5.f` on Pekko, not a calibration, not Linux, and not a claim about any other restart-and-stop interleaving. M66 is a single design; it does not show that `G5.f` detects every way of breaking a second `postStop`.
