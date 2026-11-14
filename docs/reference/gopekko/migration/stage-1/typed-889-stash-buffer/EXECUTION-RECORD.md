# EXECUTION RECORD: Bounded Typed StashBuffer and `WithStash` (#889, TYP-13)

Scope `docs/scopes/889-typed-stash-buffer.md` (PR #1505, approved at gate G2 by the Owner on 2026-11-10). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`). Every PR says `Refs #889`, none `Closes`. Written by the Dev of slice B; the QA reviews of slice B happen after this record and are not in it. Merge commits are not known when this is written (the PRs were open): see `git log`.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`). The Dev of each slice was a separate session. Wall clock, tokens and the subagent count of slices 0 and A were not measured by this session and stay `unknown` in the cost-log row.
2. Inputs: `Behaviors.scala:45-49`, `StashBufferImpl.scala:42-43, 49-53, 59, 63-96, 127-150`, `StashBuffer.scala:26-34, 37-38, 45-124, 172`, `StashBufferSpec.scala`, `EventSourcedBehaviorStashSpec.scala:644-688` and `StashManagement.scala:55-70` (nearest evidence, default Stop only), ADR 0035 and its amendment of 2026-11-10, ADR 0033 D13.
3. Outputs: `StashBuffer[T]`, the ten methods, `Contains`, `ErrStashOverflow`, `ErrStashEmpty` (`typed/stash.go`, slice A) and `WithStash` (slice B); `stash_test.go`, `stash_withstash_test.go`, `ExampleWithStash`, the exported-surface list, the README section "Stash", the kit package (`manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md`), this record and one cost-log row.
4. Deviations: none from the scope. S9 reached through `Supervise(WithStash(...))` with a handler that returns the `Stash` error, as D-3 requires.

## 2. RED evidence

Slice B tests were written before `WithStash`: the first run failed to compile (`undefined: WithStash`), then, once it existed, the mutations below failed by assertion on the slice B head, in a disposable edit of `stash.go` restored after each run (never committed).

## 3. Mutations of slice B (RULEBOOK section 5)

| Criterion | Mutation | Failing test | Failure |
|---|---|---|---|
| S8 | `WithStash` passes a nil context | `TestWithStash_BufferIsBoundToTheBehaviorContext` | `expected <nil> not to be nil` |
| S8 | `WithStash` ignores `capacity` (uses 1) | `TestWithStash_BuildsBufferOverSetup` | `expected 1 to equal 7` |
| S9 | `Stash` returns an error that does not wrap `ErrStashOverflow` (the matcher cannot select it) | `TestWithStash_OverflowIsCaughtBySupervision` (Resume and Restart rows) | the actor takes the default Stop; fails after the 30 s helper timeout |
| S9 | the restart keeps the old buffer (`WithStash` caches it) | `TestWithStash_OverflowIsCaughtBySupervision`, Restart row | `got "buf:1:a", want "buf:0:"` |

The mutations of slices 0 and A (S1 to S7c) are in the tables of PR #1516.

## 4. Gates

Per PR, on the pushed head: `gofmt -l`, `go vet`, `go test -count=1 -covermode=atomic`, `scripts/each_module.sh go test -count=1 ./...`, `golangci-lint` with `../.golangci.yml`, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh` (7 errors that are identical on `origin/develop`, none in `pekko-actor-typed`). `unshare -rn` is not available on macOS. The output is in the evidence comment of the PR.

## 5. Size (authored lines)

| PR | Content | Estimate |
|---|---|---|
| #1514 | slice 0, kit package | ~110 |
| #1516 | slice A | ~340 (measured 327) |
| this PR | slice B | ~330 |

The measured size of slice B is in its PR description.

## 6. Evidence class and pending approval (ADR 0033 D13)

- **SPEC/READ, read against Pekko v1.7.1 and not run:** every buffer row (S1 to S6, S7, S8). Not V3, no Pekko run exists for this unit.
- **No Pekko oracle test: alternative evidence:** S7a (capacity 0 and negative), S7b (zero value), S9 (overflow under `OnFailure`; **approved by the Owner on 2026-11-10**). S7a and S7b stay pending the per-row approval.
- **Not claimed:** parity of the module with Pekko Typed, V3, the text of any error, behavior of the captured context outside the actor's own turns (#927).

## 7. Not delivered, with owners

`unstash`, `unstashAll`, `UnstashException`, `StashSpec` and the unstash cases of `StashBufferSpec` (PORT-GAP): #927, which also owns the usage rule of the context the buffer holds. The bounded stash of a restart: #999. `Stopped`: #1182. A Pekko run of `withStash` under `onFailure` is not proposed.

## 8. Criteria against the scope

| Criterion | Result |
|---|---|
| S1 to S7c | delivered (#1516) |
| S8, S9, S10 | delivered (this PR) |

## 9. Recurring failures (`AGENTS.md`, kit rule 6)

None in this slice.
