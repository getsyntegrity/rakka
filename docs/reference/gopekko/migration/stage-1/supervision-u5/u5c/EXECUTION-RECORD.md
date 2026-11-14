# U5c execution record (#445)

Scope: U5c only (public guardian strategy and validated top-level names). This is not a claim that `System.Shutdown`, `whenTerminated` or Pekko parity exist. Package: PR #466 (head `f321d52`, squash `0ad1401`), six hashes verified before the merge and again on the code branch. Code: branch `feat/actor-supervision-u5c`, commit `509eb87`, draft PR not merged.

## Delivered

- Public API, exactly RULEBOOK 3.5: `Config.GuardianStrategy`, `GuardianStrategy`, `GuardianDecider`, `GuardianOneForOne`, `StoppingGuardianStrategy`. No exported signature mentions `guardianMsg`. The zero value is `DefaultStrategy`; a nil decider escalates; `ok == false` escalates; a panicking decider ends the system; `options.userStrategy` (test seam) wins when set.
- Top-level names are validated like child names (D7): `""`, `"$x"`, `"a b"`, `"a/b"`, `"%zz"` return `ErrInvalidName` before any reservation. The root guardian (name `""`, no parent) stays exempt. The `topLevel` marker of U5b is retired. **Breaking change** for callers that used such names.
- Tests: T8 (`TestTopLevelNamesAreValidated`), T5p (`TestGuardianStrategyEscalationEndsTheSystem`: escalating, nil and panicking deciders), T15 (zero value, budget, stopping, precedence), P3 (`TestActorSystemSpec_StoppingStrategyDeliversOneTerminated`), external `ExampleGuardianOneForOne` and `ExampleStoppingGuardianStrategy`; INFO assertions T7 (`Ref[guardianMsg]`) and V14 (one `receive-failed`).
- Comment-only fixes in `errors.go`, `supervision.go`, `supervision_hooks.go`, `watch.go`.
- Documentation by the coordinator: README, `doc.go`, ADR 0014 amendment (b) (D4 replaced) and the history of amendment (a), ADR 0017 (S1 delivered, D-4 closed), `docs/actor-core-parity.md`, the RULEBOOK 3.3.1 amendment for the U5a-accepted order.

## Gates (local, by the coordinator)

gofmt (two files needed formatting; formatted, no semantic change), vet, `go test -count=1 ./...`, `go test -race -count=1 ./...`, the new and guardian tests 30x under `-race`, `golangci-lint` v2.14.0 (pinned) 0 issues, tidy, coverage 97.84% (1,202 statements), deps, headers, ADR signatures, modules selftest, SDK check, judge selftest ok, `hierarchy.go` anchor once. Scope: 13 files changed, exactly the 13 surfaces. The implementer could not run `gofmt`, `vet` or `build` (its Bash was denied partway); every check above was run by the coordinator.

## Token accounting

| Item | Tokens (measured: input + output + cache creation) |
|---|---|
| Implementer (`sonnet`, 47 turns, USD 0.68) | 99,350 |
| Focused Opus review | 104,415 |
| Total measured | 203,765 |

The review exceeded its hard limit of 100,000 by 4,415. Coordination, by the declared convention and NOT measured: reserve 90,000, no correction round (the review found no blocker or defect). **Estimated total: 293,765**, below the package range (320,000 to 420,000), the evaluation stop (450,000) and the cap (560,000). If the documentation pass were counted as an extra round the estimate would be 323,765.

## Review findings (one focused review; no BLOCKER, no DEFECT)

Recorded, not acted on (the Owner asked for no successive cycles):
- INFO-1: the `GuardianDecider` doc adds a paragraph to the text of RULEBOOK 3.5 (no `Context`; `Escalate` and a panic end the system). It agrees with 3.5's prose.
- INFO-2: no public-API test covers a decider returning `(Restart, false)` or `(Stop, false)`; the behavior rests on the generic path.
- INFO-3: T8 counts `byID` through `registered()`, not name-table reservations; "before any reservation" is shown by the code order in `spawn.go`.
- INFO-4: T5p's nil-decider case cannot tell "escalate" from "panic" (both end the system with one `receive-failed`); asserting that the cause is not a `*PanicError` would pin it. The panicking case skips the `receive-failed` count.
- INFO-5: `ExampleGuardianOneForOne` never sends a message, so its output only proves compilation and the D7 check; `ExampleStoppingGuardianStrategy` would catch a regression only as a hang.
- INFO-6: V14 no longer checks that `receive-failed` is the first event, only that it occurs once.
- Carried to #461: the comments of `cell.go:26,292` (judge anchors), the scheduling dependence of `TestSpawnRacingTheCloseIsStoppedOrRefused`, the mutants, the judge campaign, the repeated `-race`, the Pekko surface inventory, S13/S14/S16 (#175).

## Follow-up on INFO-2 to INFO-6 (tests only, no production change)

Each finding was checked against the code and the tests of `develop` at `a5dd1b6`. The author checked each strengthened assertion locally against a hand-applied mutant (applied, run, reverted). Those runs are not a campaign, are not kept in the repository and cannot be reproduced from it; the mutants named below are descriptions, not evidence files.

- INFO-2: closed. T5p (`TestGuardianStrategyEscalationEndsTheSystem`) now has the cases `(Restart, false)` and `(Stop, false)`. A wrapper that drops `ok` (`d, _ := f(child, cause); return d, true`) fails both.
- INFO-3: already covered, no change. `waRefuseTopLevel` (T8) asks for the same invalid name twice and requires `ErrInvalidName` both times. A reservation made before the validation and never released makes the second call `ErrNameTaken`; reordering `commitCell` that way fails T8 at that assertion.
- INFO-4: closed. The panicking decider of T5p panics with its own value, and every case now asserts the detail of `/user`'s single `receive-failed`: the cause of the top-level actor when `/user` escalates (an `Escalate`, an undefined cause, a nil decider) and the decider's panic value when it panics. A strategy without the nil guard calls a nil function and fails the nil case (`nil pointer dereference`); a count of one would have passed.
- INFO-5: closed. `ExampleGuardianOneForOne` now fails a top-level actor and prints that the decider received that actor and the cause. `ExampleStoppingGuardianStrategy` waits at most five seconds and prints `still running`, so a regression is an output mismatch instead of a hang. A `StoppingGuardianStrategy` that returns `Resume`, and a strategy that never consults the decider, each fail their example.
- INFO-6: narrowed, not closed as stated. On the `Restart` path (V14) the order of `receive-failed` is fixed by causality: the later events of the actor come from its own serialized activations, so a first-event assertion there detected nothing, not even `receive-failed` emitted 50 ms late. The order can break on the `Stop` path, where the parent's goroutine emits `stop-accepted`. `TestStoppingGuardianStrategy` now requires `[receive-failed, stop-accepted]`, and the delayed-emit mutant swaps them. V14 is unchanged and still asserts counts only.

INFO-6 is therefore covered on the `Stop` path only. The `Restart` path (V14) keeps count checks and no order check; a reordering or a late emission of `receive-failed` there was not detected by any assertion tried (200 runs of the reorder, one run of the 50 ms delay).

### Evidence of this follow-up (local; remote CI not executed: billing block)

- Candidate validated: `1d692febb5d0cd23d4b871afb340e04afc74102a` (base `develop` `a5dd1b6f1dc2f8918aaeb81a3b62e9df1a494222`). Diff of that candidate: 3 files, +71/-13, no production file. go1.27.2, darwin/arm64, `golangci-lint` v2.14.0.
- Tests executed on that tree: `gofmt`, `go vet`, `go mod tidy -diff`, `golangci-lint run ./...` (0 issues), `go test -race -count=1 ./...` of `pekko-actor`, `scripts/check_deps.sh`, `make ci-fast` (exit 0, served from the Go test cache).
- Coverage of `pekko-actor`: 98.42 % (1457 statements). `scripts/check_coverage.sh` was run with the profile of `pekko-actor` only, so it reported `FAIL (no profile ...)` for the other workspace modules. That is a missing profile, not a failure of those modules, and the global coverage gate was not run green.
- Independent review: one read-only review by a general-purpose agent over the diff of `a05cd45`, no blocker. The later delta (`1d692fe`: one re-wrapped comment line and the wording of this record, plus this documentation-only commit) has no independent review. It was inspected by its author; the code lines of the delta are comment text only.
- Not done: remote CI, a native RDD review, a mutation campaign.
- #461 stays open.

## Not done

`System.Shutdown`, `whenTerminated`, the termination hooks, `Kill` (#67, #192, #193); the module close (#461); parity. The native review was not requested (Owner's instruction). #445 is not closed by this record.
