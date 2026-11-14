# EXECUTION RECORD: `SuppressedDeadLetter` and divergence 5a (#871, EVT-05)

Scope `docs/scopes/871-suppressed-dead-letters.md` (PR #1522, approved at gate G2 by the Owner on 2026-11-10). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`). Slice 0 is the kit package (PR #1527), slice 1 the types and the classification (PR #1528), slice 2 the code PR #1532 (`Refs #871`, never `Closes`); this PR is the record, the cost-log row and the manifest fix (apart from #1532 so that the 400-line budget holds). Written by the Dev of slice 2; the QA reviews are not in it.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`), one Dev session per slice; wall clock, tokens and subagents were not measured and stay `unknown` in the cost-log row.
2. Inputs: `ActorRef.scala:716-724, 736-743`, `Mailbox.scala:340-354, 400-412, 557-571`, the oracle `docs/parity/oracles/suppressed-dead-letters-2026-11-10.md` (DL1, DL1b, DL7, DL8, DL8c).
3. Outputs: the guard of `postLane` removed and `takePortDeadLocked` keeping the queued `PoisonPill` (`pekko-actor/actor/port.go`), the tests, the README section "Suppressed dead letters", the implementation note of ADR 0010, the evidence of inventory row C-10b (no V level), this record and one cost-log row.
4. Deviations from the scope: none of design. The `source` of the last `manifest.tsv` row is corrected (`ActorRef.scala` to `Mailbox.scala`), as the QA of PR #1527 asked; `TestPoisonPill_UndeliveredIsSuppressed` replaces the test that pinned the open divergence.

## 2. RED evidence (strict TDD this slice)

The new tests were written first and run against the code of slice 1: `TestOracleDL1PoisonPillToAStoppedActor` (DL1), `TestPoisonPillToAnActorWhoseStopWasAccepted` (S9), `TestOracleDL7PoisonPillToAFullBoundedMailbox` and `TestOracleDL8PoisonPillQueuedWhenTheStopIsAccepted` (DL8, DL8c) failed by assertion (`expected 0 to equal 1`: no `SuppressedDeadLetter`). DL1b and the nil-message test already passed, because slice 1 classifies in `System.publish`; they pin it. Then the two code lines changed and all went green.

## 3. Mutations (RULEBOOK section 4)

Applied by hand in the working tree, restored after each run (diff against a backup is empty), never committed. Each fails by assertion.

| Id | Criterion | Mutation | Failing tests |
|---|---|---|---|
| M4 | S6, S7, S9 | the guard `if !it.poison` restored in `postLane` | `TestOracleDL1PoisonPillToAStoppedActor`, `TestPoisonPillToAnActorWhoseStopWasAccepted`, `TestOracleDL7PoisonPillToAFullBoundedMailbox`, `TestPoisonPill_UndeliveredIsSuppressed` (3 cases) |
| M5 | S8 | `takePortDeadLocked` without `it.poison` | `TestOracleDL8PoisonPillQueuedWhenTheStopIsAccepted` (DL8 and DL8c), `TestPoisonPill_UndeliveredIsSuppressed` (queued case), `TestPoisonPill_DeadLetterOrderWithQueuedKillAndPoisonPill` |
| M7 | S10 | the suppressed event also sent to the hub | `TestSuppressedDeadLetterIsNotCountedByTheHub`, the slice 1 tests of the three probes, and the tests above |

M1, M2, M3, M6 and M8 are slice 1 mutations and were reproduced by its QA. S11 (a nil `Msg`) has no one-line mutation: the type assertion in `publish` refuses it, and `TestANilMessageIsNeverSuppressed` pins that.

## 4. Gates

On the head of PR #1532 (macOS arm64, go1.27.2): `gofmt`, `go vet`, `go test -race -count=1 ./...` of `pekko-actor` (exit 0), coverage 98.4% of the module (`-covermode=atomic`), diff coverage 2 of 2 changed statement lines (`port.go:81`, `port.go:159`; own count over the profile, `diff-cover` was not run), `golangci-lint` v2.14.0 with `../.golangci.yml` (0 issues, gocyclo at most 10), `scripts/check_deps.sh` (exit 0), `make ci-fast` (exit 0), `scripts/each_module.sh go test -count=1 ./...` (exit 0, 40 packages ok), `scripts/check-judge-matrix.py` (exit 0; Pekko baseline CURRENT 9/9; goPekko HISTORICAL 21/21; goPekko CURRENT 0/21, stale by the rule because a Go core file changed: it needs the campaign after the merge), `anchor_problems('pekko-actor/actor')` is empty (no `mutate_go.py` anchor changed). `check-headers.sh` reports 7 errors in files this PR does not touch (`names.go`, `pekko-persistence/**`). `unshare -rn` is not available on macOS: the network isolation was not run.

## 5. Size (authored lines)

| PR | Content | Estimate | Measured |
|---|---|---|---|
| #1527 | kit package | n/a | 81 |
| #1528 | slice 1 | ~345 | 399 |
| #1532 | slice 2: code, tests, README, ADR note, inventory | ~215 | 345 |
| this PR | record, cost-log row, manifest fix | n/a | 60 |

## 6. Evidence class and pending approval (ADR 0033 D13)

- **ORACLE, executed on Pekko 1.7.1, Java 1.8.0_504:** DL1, DL1b, DL7, DL8, DL8c of the oracle #1508, ported as Go tests (`suppressed_deadletter_poison_test.go`).
- **Go-only, no Pekko oracle, pending the Owner's approval:** S9 (a `PoisonPill` told after the stop was accepted), and the negative cases of S10 and S11. The full-mailbox `Terminated` (sender = subject) is inferred from `DeathWatch.scala:111`.
- **Not claimed:** a V level for C-10b (the Owner assigns it), V2 or V3, parity of the module, the order of the dead letters at stop (5c, #1321), `Tell(PoisonPill)` as `ref ! PoisonPill` (5b, #1320). The Go campaign after the merge of the code is pending; until then the Go CURRENT of the checker is stale.

## 7. Not delivered, with owners

`PORT-GAP(sel-871)`: the `ActorSelection` halves of the spec. `WrappedMessage` and the listener skip (DL6): #1479. `PORT-GAP(dl-watch-896)`: `Watch`/`Identify` on the sink. `Terminate()` as a suppressed system message: no counterpart. `Dropped`: #906. `DL8b` is a race and is not used. #770 is not edited.

## 8. Criteria against the scope

S1 to S5 delivered (#1528); S6 to S11 delivered in #1532 (the S10 positive cases are in #1528). `grep -rn "TODO(#871)\|OPEN DIVERGENCE 5a" pekko-actor` is empty.

## 9. Recurring failures (`AGENTS.md`, kit rule 6)

None. Note for tests that block an actor: a failed assertion before the gate opens leaves it blocked and the cleanup `Shutdown(bg)` waits forever; the new tests open the gate in a `t.Cleanup` (`openOnCleanup`).
