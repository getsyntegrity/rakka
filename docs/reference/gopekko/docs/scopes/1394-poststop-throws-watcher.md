# Scope: A watcher receives `Terminated` when the watched actor's `PostStop` throws (#1394)

Status: **draft, waiting for the Owner at gate G2.** Written on 2026-11-06 against `develop` `38768a8b`. #1394 has the labels `t:actor`, `mod:act`, `type:task` and no priority, as the Owner
asked. No issue is edited by this PR.
**Kit run: waiver proposed (decision D-1)**: this unit adds only a runtime-behavior test to `pekko-actor` and one line of a README; it ports and adapts no production code, because the reading of
`stop.go` below, and a probe the Dev confirms in the first run, say the behavior already exists. **Condition:** if a test fails on the current code and production must change, the Dev stops, reports to the TL, the waiver falls and the Owner is asked
before a production change in `pekko-actor` is planned. The Dev is a separate session, QA is independent of both, and the TL does not implement.

Contract sources: the issue #1394 (its acceptance: "a `pekko-actor` test in which `W` watches `T`, `T`'s `PostStop` panics (and a second case with `runtime.Goexit`), `T` is stopped, and `W`
receives exactly one `Terminated(T)`, after `T`'s `PostStop` ran; the test names the mutation of production code that makes it fail"), the oracle
[classic-poststop-throws-2026-11-06.md](../parity/oracles/classic-poststop-throws-2026-11-06.md) (PS1; `C1` to `C3` below are its section 1) and the indirect scenario W5c2 of
[typed-restart-child-wait-2026-11-06.md](../parity/oracles/typed-restart-child-wait-2026-11-06.md).

## What this delivers

A test in `pekko-actor` that pins: when the `PostStop` of a watched actor panics, or ends its goroutine with `runtime.Goexit`, a watcher still receives exactly one `Terminated` for it, after
that `PostStop` began, and keeps working. It also repoints the sentence of `pekko-actor-typed/README.md` that declares this case untested ("No test combines a watcher with a panicking
`PostStop`", line 227) at the test.

## Evidence: Pekko, and what goPekko does today

| Case | Pekko 1.7.1 | goPekko `develop` `38768a8b` (throwaway probe, **to be confirmed in the Dev's first run**) |
|---|---|---|
| `PostStop` throws / panics | one `Error` event, the watcher receives one `Terminated` and keeps working (oracle PS1 [002]-[005]; `F:254-257` catch, `F:266` `tellWatchersWeDied` in the `finally` chain) | expected equal (inferred from the reading of `stop.go:126,169-172` below, and from a TL probe whose command and output were not kept): the watcher receives `Terminated` exactly once, after `PostStop` began |
| `PostStop` ends with `runtime.Goexit` | no JVM counterpart | expected the same: one `Terminated`, after `PostStop` began (inferred, same reading) |
| `PostStop` returns normally (control) | the watcher receives one `Terminated` | expected the same (inferred) |

The probe is the existing `TestTerminatedFollowsEarlierMessagesAndPostStop` (`pekko-actor/actor/watch_api_test.go`) with the `PostStop` replaced by a panic and by `runtime.Goexit`; the TL ran it against
`develop` and it passed in all three cases, but no command, output or exit code was kept, so **this is not evidence in the sense of the claims-against-evidence rule: it is a hypothesis that the Dev's first run confirms or refutes** (Mandatory Stop if it refutes it: D-1 falls). **On that hypothesis the behavior already matches and no production change is planned.** The code that guarantees it: `callPostStop` recovers a panic and, for a `Goexit`,
hands the rest of the finalization to a fresh activation (`pekko-actor/actor/stop.go`, `callPostStop`), and `finalize` then calls `notifyWatchers` ("after PostStop and after closed", the comment
at the call) before `close(done)`.

## Not carried here

| Item | Why |
|---|---|
| The `Error` event that Pekko publishes with the cause of `postStop` (oracle [003]) | goPekko reports it as the `poststop-panic` event of the system (`TestPostStopPanicEventByPath` in `supervision_default_goside_internal_test.go`, `TestPostStopPanicAndGoexitAtEveryLevelStillComplete` in `termination_stress_internal_test.go` already pin the completion); not part of this unit |
| A Typed child (W5c2 of the child-wait oracle) | the Typed side is pinned by the tests of #997; this unit is the Classic runtime |
| A remote watcher | out of scope of `pekko-actor` local watch |

## Criteria, scenarios, sources and mutations

Test file in `pekko-actor/actor/` (package `actor_test`, the helpers of `watch_api_test.go`: `spawnW`, `watching`, `barrier`, `mustDone`, `seq`). go-specs, channels or gates, no sleeps, no `-race`.
A mutation is applied to production code in a disposable worktree and never committed (`qa-review`); the named test must fail by assertion.

| # | Criterion | Scenario | Source / oracle | Mutation | Slice |
|---|---|---|---|---|---|
| P1 | When the `PostStop` of the watched actor panics, the watcher receives exactly one `Terminated` for it, after `PostStop` began, and handles a later message | `TestWatchTerminatedAfterThrowingPostStop` row `panic` | oracle PS1; `F:254-257, 266` | M1: in `finalize`, skip `c.notifyWatchers(watchers)` (`pekko-actor/actor/stop.go:126`) when `callPostStop` reported a failure | 1 |
| P2 | The same when the `PostStop` ends with `runtime.Goexit` | the same test, row `goexit` | no JVM counterpart (decision D-2); the runtime's own `callPostStop` handoff | M2: remove the `c.sys.exec.Execute(c.finalize)` handoff of the Goexit branch: the finalization never completes and the watcher is never told | 1 |
| P3 | Control: a `PostStop` that returns normally gives the same observation (so the rows differ only by the failure) | the same test, row `normal` | oracle (the `finally` chain also runs) | none (control) | 1 |
| P4 | The README line 227 of `pekko-actor-typed` says what is now pinned and where | review | | none (documentation) | 1 |

The test asserts: the count of `Terminated` entries for the actor is exactly 1; the entry comes after `post-stop-begin` in the same ordered log; and the watcher answers a message sent after the
`Terminated` (so it is alive). It does not claim that the `Error` event is published (oracle [003] is not asserted: goPekko reports the failure through its own event, above). A failure of
M2 may appear as the 30-second helper timeout of `mustDone` ("the cell never finishes"): acceptable when the missing signal has no other observable, as accepted in [#1391](https://github.com/getsyntegrity/gopekko/pull/1391) and [#1401](https://github.com/getsyntegrity/gopekko/pull/1401). **M2 may fail through that timeout and not through a count assertion: this is recorded here so that the review of the implementation accepts it explicitly (decision D-5);** the Dev says
so in the PR.

## Slice (one PR, at most 400 authored lines)

| Slice | Delivers | Files | Criteria | Est. lines (raw → re-estimated) |
|---|---|---|---|---|
| **1. Test and README** | the test with three rows, and the README line | `pekko-actor/actor/watch_poststop_throws_test.go` (new), `pekko-actor-typed/README.md` (line 227 only) | P1 to P4 | tests ~70 → ~110; README ~2; ~115 |

The README line becomes (the Dev refines the wording and keeps it exact): "That a parent's children end before the watcher's `Terminated`, and that a child's throwing `PostStop`
does not stop the restart, are `pekko-actor` behavior, pinned there by `TestDescendantsFinalizeBeforeTheirAncestors` (...) and `TestPostStopPanicAndGoexitAtEveryLevelStillComplete` (...). A watcher
receives exactly one `Terminated` after a `PostStop` that panics or ends with `runtime.Goexit`: `TestWatchTerminatedAfterThrowingPostStop` (#1394)." Nothing else of the README changes.
Nothing under `pekko-actor/` except the new test file, nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md` or `AGENTS.md` changes; **no production file is committed** (a change there is
a Mandatory Stop and goes back to the TL and the Owner).

## Allowed files

Exactly the paths in the slice table, plus an optional `watch_poststop_throws_internal_test.go` under the internal-test rule of the [#976 scope](976-typed-restart.md) when the behavior lives in unexported state.
`docs/scopes/1394-poststop-throws-watcher.md` is edited only by the planning PR.

## Dependencies

None. It consumes the runtime of `pekko-actor` as it is on `develop`. It does not touch the dependency direction (`actor` imports no other goPekko module; the README of `pekko-actor-typed`
only names a test).

## Decisions for the Owner

- **D-1. Kit run: waived**, because the unit ports and adapts no production code (the behavior is expected to exist, to be confirmed by the Dev's first run, and the unit adds a test and a README line), as in #997. Condition: a failing test or a needed production change makes the Dev stop, reports to the TL, the waiver falls and the Owner is asked before any production change in `pekko-actor` is planned. Alternative: require the kit run, with a package of three files (~110 lines of documentation) before the test.
- **D-2. The `Goexit` row (P2) has no Pekko oracle**: `runtime.Goexit` has no JVM counterpart. ADR 0033 D13 says a row with no applicable oracle needs documented alternative evidence and the Owner's explicit approval, and stays "pending approval" until then. The alternative evidence proposed: the existing tests `TestPostStopPanicAndGoexitAtEveryLevelStillComplete` and `TestGoexitInPostStopIsNotAFailure`, the documented handoff of `callPostStop`, and the Pekko behavior for a throwing `postStop` (PS1), which the Goexit row mirrors. Proposed: keep the row, with this evidence and the "pending approval" mark in the PR. Alternative: drop the row and keep P1 and P3.
- **D-3. README wording** (P4): the line points at the test and keeps the sentences about the other two runtime behaviors. Alternative: only add "(see #1394)".
- **D-4. Labels of #1394:** it stays without priority and without `status:*` (Owner instruction); proposed: no change, and close it as completed after the PR merges, with an Update like #997's.

- **D-5. The 30-second timeout as a failure signal of M2.** `mustDone` fails the test after 30 s when the cell never finishes; if M2 (no `Execute(c.finalize)` handoff) makes the watcher's `Terminated` never arrive, the test fails through that timeout and not through a count assertion. Proposed: accept it, because the missing signal has no other observable (as in #1391 and #1401); the Dev says so in the PR and QA accepts it in the review of the implementation. Alternative: add a bounded wait on the watcher's channel that fails with its own message.

## Appetite

One small PR, about 115 lines. The risk is the Goexit row: the `Terminated` must be awaited deterministically (the fresh activation finishes the finalization), so the test uses the `mustDone` and `barrier` helpers, never a sleep.

## Definition of Done

- P1 to P3 have their rows and pass; P4 is reviewed; the PR names the mutations M1 and M2 in a "Mutations" table, reproduced by QA (unmutated green, mutated red).
- `scripts/each_module.sh go test -count=1 ./...`, `scripts/check_deps.sh` and the complexity lint green with the commands in `AGENTS.md`; `-count=100` of the new test; no production file changed (diff coverage: none to measure, say so).
- Every sentence that says something is observed names the log line or the assertion (rule of 2026-11-06); every Pekko citation is verified by the Dev; the evidence is a comment bound to the final head SHA with real exit codes (RUNBOOK-style: SHA, command, environment, exit code, full output).
