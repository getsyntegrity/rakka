# EXECUTION RECORD: Typed `Unhandled` result and local record (#925, TYP-03, Slice A)

Scope: `docs/scopes/925-typed-unhandled-marker.md` (merged by PR #1258, squash `f682e5f4f8648cf4afa13bd9f949629836e0cff4`, reviewed head `2ed5d175fb0eda80ba79cd453075a6c7ed26bf5c`). Scope section 11.1 and the Owner log L3 are on the unmerged branch of PR #1261 (`docs/925-scope-confirm`); the kit package (`RULEBOOK.md`, `RUNBOOK.md`, `manifest.tsv` in this directory) is on the unmerged branch of PR #1260 (`docs/925-kit`). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1. Delivery: stacked draft PRs, `Refs #925`. Only #1258 is merged; #1260, #1261, #1262 and this PR are not.

## 1. What the kit run was

No kit loop ran: no kit agents and no campaign. The TL, Dev and QA agents (Claude Sonnet 5.5) followed the RULEBOOK by hand and wrote code and tests from the Pekko sources in `manifest.tsv`, read, not run. No JVM ran. The QA agent was independent of the TL and Dev and reviewed each PR head. Models and tokens are not reported by the session, so the cost row says `unknown` for wall clock and tokens.

**Cost row, `subagents` column.** The kit defines the column as the number of subagents a step ran (its example row has 35; `migration-kit/prompts/04-translation-kickoff.md`, line 62); earlier rows record 0 because no kit subagents ran, and the #954 Slice B row records 7 for its own session. This unit's row records **4**: the agents the orchestrating session started with the Agent tool for #925 Slice A, namely the Dev of PR 2 (kit, `abae4f856d68545ac`), the Dev of PR 3 (code, `a90c301481b9d2999`), the Dev of PR 4 (this PR, `ab903241f3ff2b491`; its transcript is not final when this record is written) and the independent QA agent `a6f1abb3f2f046e45`, which was started for an earlier unit (#954 Slice B) and resumed for this one; it is counted once here and is also counted in the Slice B row. Source: the agent transcripts of the session (`.../tasks/<agent id>.output`, outside the repository, not versioned; their existence was checked with `ls`, not read). The count excludes the TL/orchestrator, counts each agent once even when resumed, and is **not** a count of kit loop subagents (none ran). Wall clock and tokens stay `unknown`.

## 2. Inputs and outputs

| Input | Output |
|---|---|
| `Behavior.scala:184-192, 297`, `BehaviorImpl.scala`, `BehaviorSpec.scala`, `ActorContextSpec.scala` (see `manifest.tsv`) | `pekko-actor-typed/typed/{behavior,interpreter,signal}.go` (`Unhandled`, the unexported record hook, the death pact for a declined `Terminated`) |
| Scenarios U1 to U9 | `pekko-actor-typed/typed/unhandled_internal_test.go` (U8 is covered by existing tests, see section 5) |
| The module README contract | `pekko-actor-typed/README.md` "Unhandled (#925 Slice A)" and limitations; this record; one `migration/cost-log.tsv` row |

## 3. PRs, heads and verdicts

Three different things are kept apart: the **reviewed head** (the SHA an independent QA verdict is bound to), a **later head** (pushed after that review; it has its own QA verdict on its delta), and the **integrated commit** (the squash on `develop`). Squashes are pending except #1258, which is merged.

| PR | Branch | Content | Reviewed head (QA FAVORABLE) | Later heads, each with its own delta QA | Integrated commit |
|---|---|---|---|---|---|
| #1258 | `docs/925-unhandled-scope` | scope | `2ed5d175fb0eda80ba79cd453075a6c7ed26bf5c` | none | `f682e5f4f8648cf4afa13bd9f949629836e0cff4` (merged) |
| #1261 | `docs/925-scope-confirm` | scope confirmation (section 11.1, L3) | `d526c96da2c76b3b84e9a9663bf781aed903bc6a` | `9cc66adce8d7098da11522cd44cee158e95fd841` | pending |
| #1260 | `docs/925-kit` | PR 2, kit package | `818516899e40705b508a97997efcdbcb9cc682b5` | `9159abb6c7eedc0dcfa7d1769e940361acae8b34`, `fe67cc65c7dfdf91c346c60cc4f10875fdd625ec` | pending |
| #1262 | `feat/925-unhandled` | PR 3, code and tests | `9da55e347b8f23f21b59279bbfb91ca9199e16e6` | `4fb4aee953395ac2b37efd9fc5c56da44d45591e` (comment-only) | pending |
| this PR (PR 4) | `docs/925-record` | README, this record, cost-log, tests U2 and U5 | in its PR body (a record cannot cite its own head) | in the PR itself | pending |

## 4. Owner decisions

Two Owner messages reached the TL partly corrupted (the scope log entries L2 and L3), so those entries record what was legible and the confirmation of section 11.1 on #1261 re-reads them.

| Decision | Content |
|---|---|
| U-1 to U-7 (G2, 2026-10-28) | recorded in the scope as read by the TL from the two partly corrupted messages: U-1 split (Slice A is the marker and local record; the publication is Slice B); U-2 (a) a per-instance, unexported hook, no-op in production; U-3 `Unhandled[T]()` is the only export; U-4 `Unhandled` for `Terminated` takes the death pact, other signals dropped; U-5 reuse `ErrInvalidInitialBehavior` (option b); U-6 #1181 keeps #847 as Behavior core and consumes the marker from #925 Slice A; U-7 B-4/EVT-1 pending, wording "not published yet". U-2 (a), U-4 and U-7 were confirmed as read |
| "adelante" | started the kit PR (#1260) |
| PR 3 and PR 4 | authorized as read |

## 5. Scenarios and evidence labels

Labels: READ = derived from a read of Pekko, alternative evidence **pending Owner approval (ADR 0033 D13)**. The specs publish through an event stream that Go does not have, so **no Go test is claimed equivalent to a Pekko spec**.

| Scenario | PR | Test | Label |
|---|---|---|---|
| U1 a message answered `Unhandled` keeps the behavior and is recorded once | PR 3 #1262 | `TestUnhandled_U1_MessageKeepsBehaviorAndRecordsOnce` | READ |
| U2 a replacement after an `Unhandled` step applies | PR 4 (this PR) | `TestUnhandled_U2_ReplacementAfterUnhandledApplies` | READ; RED observed by mutation (section 6) |
| U3 not a dead letter, not a failure | PR 3 | `TestUnhandled_U3_NotDeadLetterNotFailure` | READ (Go-only assertion, no Pekko oracle) |
| U4 `Terminated` declined takes the death pact, decider not consulted | PR 3 | `TestUnhandled_U4_TerminatedTakesDeathPactWithoutDecider` | READ |
| U5 `Unhandled` from the `PostStop` handler has no effect | PR 4 | `TestUnhandled_U5_PostStopDropsUnhandled` | READ; regression guard, passes without the implementation, **no mutation evidence claimed** |
| U6 initial behavior or `Setup` result `Unhandled` fails the start; turn-time `Setup` to `Unhandled` fails the turn | PR 3 | `TestUnhandled_U6_InvalidInitialBehavior`, `TestUnhandled_U6_TurnTimeSetupToUnhandledFailsTurn` | READ |
| U7 the record precedes the next turn, no overlap | PR 3 | `TestUnhandled_U7_RecordPrecedesNextTurnWithoutOverlap` | READ (Go-only assertion, no Pekko oracle) |
| U8 existing behavior unchanged | existing tests of the module | the pre-existing suite | existing |
| U9 the only export added is `Unhandled` | PR 3 | `TestUnhandled_U9_PublicSurface` | contract check of the public API |

## 6. Evidence level and known gaps, honestly

Go unit evidence derived from a read of Pekko. **Not V3**, no Typed parity claimed, no Typed oracle.

- **U2 RED by mutation.** In a throwaway copy, `Receive` was changed to call `ctx.StopSelf()` in the `unhandled` case (Unhandled treated as a failed turn): U2 failed ("timed out waiting for the actor"). U5 passed under the same mutation and passes without the implementation; it guards a regression only.
- **Timing difference with Pekko.** A `Setup` returned by a handler that resolves to `Unhandled` (or `Same`) fails **this** turn in Go; Pekko fails the **next** turn (`Behavior.scala:184-192`). Pinned by U6's second test; registered gap.
- **The hook is a seam.** The unexported `onUnhandled` hook of `newActor` is a production injection point, accepted for this case (U-2 (a) as read) and to be removed when the event stream exists. In production it is nil: nothing is stored or published, so **no production caller can observe an unhandled message today**.
- **The hook runs outside `turn()`'s recover** (QA advisory, accepted: it is nil in production, so no production panic path exists).
- **Budget.** PR 3 landed at exactly 400 authored lines; tests U2 and U5 were moved to this PR for that reason.
- **QA mutation results (PR 3):** `OnTerminated` returning the error makes U4 fail; removing the record makes U1, U3 and U7 fail; `start()` without the `Unhandled` case makes U6 fail; adding `func Leak() {}` makes U9 fail.
- **Gates not run.** The network isolation variants (`unshare -rn`, container) were **not run** on this macOS host; tests ran without isolation, which is not equivalent. `-race` was not used. Remote CI was not run. `check-headers.sh` reports 7 errors in other modules that predate the stack.
- **Test style.** The tests use `t.Fatalf`, like the module; the go-specs migration is deferred by the Owner.
- **Finding, recorded and not edited.** #1181 declares the marker from #847, but #925 Slice A is the producer (scope section 1, U-6).

## 7. Deviations and gates

| Item | What | Why |
|---|---|---|
| Tests U2, U5 | land in PR 4, not PR 3 | PR 3 reached exactly 400 authored lines |
| Network isolation | not run | macOS host without `unshare` |
| Remote CI | not run | nothing is attributed to it |

Budget: `AGENTS.md` "PR budget and commits", at most 400 authored changed lines per PR; this PR's count and its gates, bound to its head SHA, are in its PR body.

## 8. Not delivered

- Publication of `UnhandledMessage` (Slice B, after #897 and an event stream; B-4, X4, X6 and EVT-1 stay pending). It is **not published yet**.
- `Empty` and `Ignore` (#1181, which consumes the marker this slice produces; see the finding above) and `Stopped` (#1182).
- The sender in the record (X6).
- `ChildFailed` (#1250) and `MessageAdaptionFailure` (#975).
- Typed supervision capture of the death pact (#1379; this said #976 when written).
- A public Typed watch (#1171).

#925, #954 and #1181 stay **open** and `status:blocked`. No parity claim. This PR changes no issue, label, ADR, scope, kit file or production code.
