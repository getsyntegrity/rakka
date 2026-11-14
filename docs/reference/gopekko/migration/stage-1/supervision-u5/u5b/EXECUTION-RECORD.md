# U5b execution record (#445)

Scope: U5b only (top-level actors under `/user`). U5c is not authorized and not started. This is not a claim that U5 is complete or at parity with Pekko. Package: PR #464 (head `1eb0d8c`, squash `e4de3e3`), six hashes verified before and after the merge. Code: branch `feat/actor-supervision-u5b`, PR #465 (draft, not merged).

## Delivered

- `Spawn(System)` creates children under `/user` (`System.spawnTarget()` = `/user` node and handle); top-level failures reach `/user`'s decider (default: Restart without limit; Stop for `*InitializationError`, `ErrDeathPact`, `ErrGoexit`); `Context.Parent()` of a top-level actor returns the `/user` handle; stopping `/user` stops its actors first and closes the admission of `Spawn` (`ErrParentStopping`); `System.root` is retired.
- Production: `guardian.go` (`userNode`), `hierarchy.go`, `spawn.go` (`topLevel` marker), `system.go`, `supervision_failure.go` (comments). No exported identifier changed.
- Tests: new `guardian_goside_internal_test.go` (T2, T3, T5, T6, T7, T9, T10, T13b, T14b, T16, P1, P2, P4, P5, P6); 19 existing test files adapted keeping purpose and assertions (the `stopAllUser()` seam is used only where a test is about the stop path or typed causes).

## Differences from the approved RULEBOOK (declared in the package, accepted by the Owner)

1. `topLevel` marker on `spawnTarget`: top-level names stay unvalidated until U5c (D7), because `spawn.go` validates when `parent != nil`.
2. 19 existing test files adapted (the RULEBOOK section 9 named 7 plus the interim tests); measured by a throwaway probe.
3. `guardian.go` is a production surface (it holds `/user`'s node).
4. The U5a-accepted order of `Done` for "stop /system" with the inline executor (`[user, system, root]`) stays as U5a pinned it; the RULEBOOK 3.3.1 text is not amended here.

## Gates (local, by the coordinator)

gofmt, vet, `go test -count=1 ./...` (first run green), `go test -race -count=1 ./...`, sync tests (T13b, T14b, T16, T5, T6, start-up) 30x under `-race`, `golangci-lint` v2.14.0 (pinned) 0 issues, tidy, coverage 97.90% (`pekko-actor`), deps, headers, ADR signatures, modules selftest, SDK check, judge anchor ` || len(n.out)+n.reserved != 0 {` once in `hierarchy.go`, judge selftest ok. Scope: 25 files changed, exactly the 25 surfaces. `gocyclo` is enforced through the pinned lint. Commit `d697aac` is 828 insertions and 160 deletions, above the planning heuristic of about 700 per commit.

## Token accounting

Measured (agents, input + output + cache creation, from the CLI JSON or the agent result):

| Item | Tokens |
|---|---|
| Implementer (`sonnet`, 158 turns, USD 2.74) | 298,504 |
| Focused Opus review | 127,642 |
| Total measured | 426,146 |

The review exceeded its 100,000 limit by 27,642 (measured). Coordination, by the declared rule and NOT measured: reserve 90,000 plus one authorized extra round 30,000 = 120,000. **Estimated total: 546,146**, under the 560,000 cap. The evaluation stop (450,000) was reached after the review (516,146) and the coordinator stopped and asked; the extra round was then authorized by the Owner (coordinator only, no new agents, no new review).

## Review findings (one focused review; no blockers; no behavior defects)

- DEFECT-1 (fixed in the extra round): doc comments contradicted D4/D5 in `hierarchy.go` (`Parent()`) and `spawn.go` (top-level "stopped directly"). Also fixed in the extra round: `actor/doc.go`, `pekko-actor/README.md` (nine passages), ADR 0014 amendment (a).
- INFO, recorded and not acted on (linked to U5c / documentation): stale comments in `cell.go:26,292`, `watch.go:361`, `supervision.go:69`, `supervision_hooks.go:40` (files outside this delivery's surfaces; fix with U5c or the module close); T7 only rules out `Ref[int]` (also assert not `Ref[guardianMsg]`); V14's escalating case asserts only the first event (a `receive-failed` count of 1 would match its comment); `TestSpawnRacingTheCloseIsStoppedOrRefused` depends on scheduling for how often it races; U5a's own follow-up: RULEBOOK 3.3.1 text for the accepted order.
- The native review was not requested for this delivery (Owner's instruction). Reviewer verdict: U5b meets D4, D5, D9, D11, 3.2, 3.3.1, 3.6 and T2-T16.

## Not done

U5c (`Config.GuardianStrategy`, name validation D7, T5p, T8, T15, P3), `System.Shutdown`, `whenTerminated`, module close, mutants, judge campaign, parity. #445 stays open.
