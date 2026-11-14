# U5a execution record (#445)

Scope: U5a only (guardian cells and their start-up). U5b and U5c are not authorized and not started. This is not a claim that U5 is complete or at parity with Pekko.

## Delivered

- `pekko-actor/actor/guardian.go`: root, /user and /system guardians as actors, strategies, Watch chain /user -> /system -> root, start-up in the RULEBOOK 3.4 order.
- `spawn.go`: `Spawn` split into `commitCell` plus the activation step, behavior unchanged.
- `system.go`: `guards`, `bootPoint` and `userStrategy` test seams, `newSystem` ends with `startGuardians`.
- Tests: `guardian_startup_internal_test.go` (T11, T12, T13, T14a), `guardian_internal_test.go` (T1, T4, 3.3.2), baselines adapted in four existing test files.
- Owner-approved extension outside the 15 surfaces: one `settleCtl(ex)` in `cell_internal_test.go` (TestActivation) and two in `lifecycle_internal_test.go`. No assertion changed.

## Deviation from the approved RULEBOOK (accepted by the Owner)

RULEBOOK 3.3.1, "stop /system" with an INLINE executor closes Done as `[user, system, root]`, not `[system, user, root]`, because the existing contract notifies watchers before closing Done. Root is still last, every Done closes once. T13 allows this order only for the inline executor. The RULEBOOK text is to be amended in a separate documentation change.

## Gates (local)

gofmt, vet, `go test -race -count=1 ./...`, start-up tests 30x with `-race`, golangci-lint 0 issues, coverage 98.14% (pekko-actor), deps, headers, ADR, SDK, judge anchors 22 intact, scope: 15 surfaces plus the 2 approved files.

## Token accounting

Measured (agents, from CLI JSON):

| Item | Tokens |
| --- | --- |
| Implementer | 301,272 |
| Focused Opus review | 108,972 |
| Total measured | 410,244 |

Estimated coordination reserve, NOT measured: 90,000 fixed + 30,000 for one extra correction round = 120,000.

Estimated accounting total: 530,244, under the 560,000 cap. The review exceeded the Owner's 100,000 limit by 8,972 (measured, before any coordination).

## Review findings (no blockers; none applied, no further round authorized by the budget)

- W1: T13 does not assert the `Terminated` deliveries of the 3.3.1 table (`checkClosed`). A duplicate delivery would pass because stop is idempotent.
- W2: boot-step numbering does not match its doc or RULEBOOK 3.4: `boot(1)` runs before construction, steps 2/3 and 4/5 are adjacent, so T13 covers 4 distinct boundaries, not 6.
- N3: `linkPoint`/`recheckPoint` are restored without `defer` (`guardian.go`).
- N4: the root guardian is committed under the hidden `top` node with name "", while 3.4 says it is under no owner.
- N5: `checkClosed` indexes an empty order and prints `e.orders` instead of `allowed`.
- N6: root's start step runs before /user and /system start; U5b should test a `Failed` from /user reaching root early.

## Extra round W1/W2 (Owner-authorized, done by the coordinator, no agents)

- Surfaces: `guardian.go` and `guardian_startup_internal_test.go` only.
- W1: optional internal hook `guardianSet.terminated`, nil in production, set from a `bootPoint` at step 2. T13 asserts that only {/system <- Terminated(/user), root <- Terminated(/system)} occur, each at most once, and exactly once each when "stop /user" is issued from step 4 on. A local throwaway mutation that delivers each Terminated twice made 36 scenarios fail; it was reverted.
- W2: `checkClosed` no longer indexes an empty order and prints the allowed set; the Watch wiring moved to `wireGuardians` with a deferred restore of `linkPoint`/`recheckPoint`; `startGuardians` documents the real boundaries of boot steps 1 to 7 (coverage unchanged: steps 2 to 7 still cover four distinct pre-activation boundaries and two activation boundaries). The stale `bootPoint` doc in `system.go` is outside the authorized surfaces and was not edited.
- Gates after the round: gofmt, vet, `go test -race ./...`, guardian tests 30x with `-race`, lint 0 issues, coverage 98.07%, anchors 22 intact.
- The native review approval (lineage review-7cd9e78295fbc614, acknowledged) covers HEAD 508d268 only. It does not cover the commit of this round, which has had no native review.
- Cost: no new agent tokens were measured; the coordinator's work is part of the estimated reserve. Authorized increment: up to 50,000 (maximum 580,244 accounted). Measured agent tokens stay 410,244.

## Follow-up to the second native review (Owner-authorized, coordinator only, no agents)

- Finding R3-exact-terminated-only-user-entry: T13 pinned exact `Terminated` counts only for "stop /user". Now `terminatedCounts` pins them for the three entries of RULEBOOK 3.3.1 across steps 2 to 7 and both executors: stop /user {system<-user 1, root<-system 1}; stop /system {0, 1}; stop the root guardian {0, 0}. The first two are the table's counts. For the root entry the table says "at most one" and every one of the 12 scenarios was observed to deliver none, so the zeros are pinned from that observation, not from the table.
- Verification of the detection: a lost delivery (root guardian not recorded) fails 24 messages, a duplicated one fails 36; both mutations were local, throwaway and reverted.
- Finding R3-bootpoint-doc-mismatch: the `bootPoint` comment in `system.go` now describes the real boundaries of steps 1 to 7. Comment only, no runtime change.
- The native approvals are historical evidence of their own HEADs only: review-7cd9e78295fbc614 for 508d268 and review-4f0e2619292a7012 for 72b0cd0. They do not cover the commit of this follow-up, which has had no native review.

## Owner decision on "stop the root guardian" (tests only)

- The contract for that explicit stop is 0 or 1 effective `Terminated` per watch pair, never a duplicate: a watcher that already accepted its stop may or may not process one, depending on the interleaving. The tolerance applies to that entry only. "stop /user" (both pairs exactly once) and "stop /system" (root<-system exactly once, system<-user exactly zero) stay exact, because there the delivery is what starts the closing chain.
- `checkNothingLeft` proves separately, for the three entries: all guardians completed exactly once (finalize once, so no second PostStop), asked to stop once each, no `poststop-panic`, the root guardian counts no outstanding child, no name remains under it, and the executor has nothing pending. The root guardian closes last (`checkClosed`).
- Mutations (local, reverted): a duplicated delivery fails 36 messages, a lost root<-system delivery fails 24.
- Native reviews remain historical evidence of their own HEADs (508d268, 72b0cd0). This follow-up has had none.

## Not done

U5b, U5c, the RULEBOOK amendment, module close, parity. #445 stays open.
