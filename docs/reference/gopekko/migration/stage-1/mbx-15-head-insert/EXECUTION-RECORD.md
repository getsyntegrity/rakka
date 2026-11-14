# EXECUTION RECORD: Head insert in the mailbox storage (#1098, MBX-15, partial delivery)

Refs #1098. This record closes nothing and marks no public criterion as met.

## 1. Owner approval (kit rule 3, sign-off T1), prior to any code

The Owner approved the kit package commit `f7f6171efb4d0eefce4b8c2c369b469b3a58edb0` (branch `docs/1098-mbx-15-scope`) in a real user message of
2026-10-28, before the Dev wrote code. RULEBOOK SHA-256 approved: `385e80ab0261e4a908b74634dd77e5502a474d75841eb0edfefd6b9fcde1fd7b`; the Dev
verified it with `shasum -a 256` on the file at that commit (match) before writing any code. The RULEBOOK is unchanged by this PR.

The approval covers only the internal primitive `Queue.PushFront` without cell wiring. It is **not** an approval of results, of Pekko parity or of the
criteria of issue #1098. The ordering contract stays "pending approval" (ADR 0033 D13): the evidence is READ, not V3.

## 2. What this delivers

- `Queue[T].PushFront(msg T)` in `pekko-actor/internal/mailbox/queue.go`: the message is the next one `Pop` returns; head inserts are LIFO; growth, `Len` and
  cleared slots follow `Push`. The queue does not reverse a sequence, does not know a capacity and publishes no dead letters. No export, nothing under
  `pekko-actor/actor/`, no README change.
- Scenarios Q1 to Q9 in `queue_test.go` (go-specs, one goroutine, no sleep, no clock): `TestQueuePushFrontOrder` (Q1-Q3), `TestQueuePushFrontLimits`
  (Q4-Q6, two cases for Q6), `TestQueuePushFrontPending` (Q7, Q8), `TestQueuePushFrontAgainstReference` (Q9). AC-7 (`Len`) is asserted inside Q1, Q4, Q6.
- The type comment now states the head-insert exception to FIFO.

## 3. RED and mutation per scenario (observed)

First RED: a stub `PushFront` that appended at the tail failed Q1, Q2, Q3, Q5, Q6 (both), Q7, Q8, Q9; Q4 passes under the tail stub (a lone message pops
alone either way), so Q4 was made red with a second mutation. Mutations applied on the implementation, each reverted afterwards:

| Mutation | Red scenarios |
|---|---|
| Tail insert instead of head insert (stub) | Q1, Q2, Q3, Q5, Q6 (both), Q7, Q8, Q9 |
| Index `(head-1) % len` without `+ len` (no wrap below zero) | Q1, Q2, Q3, Q4, Q5 (index out of range [-1]; also others reached on the first insert) |
| Grow only when the buffer is empty (no grow when full) | Q3, Q6, Q7, Q9 |
| Missing `size++` | Q1 to Q9 (Q4 and the `Len` assertions included) |
| `Pop` does not clear the consumed slot | Q8 |

The tail-insert mutation was re-run after the final restructuring of the test file (Q9 moved to a helper to keep complexity at most 10): Q1, Q2, Q3, Q5,
Q6, Q7, Q8, Q9 red. "Grow ignoring `head`" (scope text for Q6) was not run as a separate mutation; the grow path is the existing `grow()`.

## 4. Gates

Tested tree: code commit `17c865e8877ff543eca9a4d386c7d7acc5a3ccd7` (the commit that adds `queue.go` and `queue_test.go` changes). The commit that adds this
record and the cost-log row changes only documentation and a TSV row, so those gates are not re-run for it. Environment: macOS (Darwin 25.3.0, arm64),
Go `1.27.2` (`.go-version`), golangci-lint `v2.14.0` built with go1.27.2 in `bin/` (installed with `GOBIN`). Remote CI is not used; no remote result is attributed.

| Gate | Command | Exit |
|---|---|---|
| gofmt | `gofmt -l pekko-actor` | 0, no output |
| vet | `go -C pekko-actor vet ./...` | 0 |
| Unit tests | `go -C pekko-actor test -count=1 ./...` | 0 (actor, internal/deadletter, internal/mailbox ok) |
| Lint, gocyclo at most 10 | `cd pekko-actor && ../bin/golangci-lint run --config ../.golangci.yml ./...` | 0, 0 issues |
| Tidy | `go -C pekko-actor mod tidy -diff` | 0, no diff |
| Coverage, all modules | `scripts/each_module.sh` + `go test -covermode=atomic -coverprofile` per module, then `scripts/check_coverage.sh "$COV" 85` | 0 and 0; `pekko-actor` 98.59%, lowest module `pekko-persistence` 95.49% |
| Diff coverage | `pipx run --spec diff-cover==11.0.0 diff-cover "$COV"/*.xml --compare-branch=origin/develop --fail-under=85` | 0; `queue.go` 100% (6 of 6 lines) |
| Architecture lint | `scripts/check_deps.sh` | 0 |
| Headers | `bash .github/scripts/check-headers.sh` | **1, not green**: 7 errors in files this PR does not touch (`pekko-actor/actor/names.go`, 5 files under `pekko-persistence/`, `pekko-persistence/spi/snapshot.go`); none names `queue.go` or `queue_test.go`. Not fixed here (outside the allowed files); to be handled by the TL |

Not run: the network-isolated variant (`unshare -rn`) because the platform is macOS; the container variant was not used; `-race` (not requested).
The unit tests use no network. `origin/develop` was the local remote-tracking ref at the time of the run, not refreshed.

## 5. Read, not run

Pekko was read from `upstream/pekko` at `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1) by the scope author (`Mailbox.scala:576-598`, `Stash.scala:245-250,
271-277`) and consumed here through the scope; the Dev did not rerun a Pekko oracle and sbt was not run. No differential against Pekko exists. All Pekko-related
claims are alternative evidence pending Owner approval (ADR 0033 D13), not parity.

## 6. What remains for the consumers (no criterion is marked met)

| Item | State |
|---|---|
| AC-R1-1 / AC-R2-1 public receive order | Queue-level evidence only (Q1, Q2). The public ordering test belongs to the first consumer that wires the primitive (#898 or #855) |
| AC-R3-1 full bounded mailbox, dead-lettered once | Nothing delivered; deferred by the Owner (D-2) with the overflow rule to #855 / #898 (bounded stash capacity: #899) |
| AC-R4-1 existing tests unchanged | Existing queue and mailbox tests pass unchanged at queue level |
| AC-R5-1 export justification | No export added |
| Cell wiring: `laneItem.after`, `pushed`, `maxDepth`, stop guard | Not delivered; see the scope hand-off section |

No issue or label is edited by this PR; #1098 stays open.

## 7. Cost and size

Authored lines: `queue.go` +16/-2, `queue_test.go` +187, this record and one cost-log row; under the 400-line budget. Metrics not measured are `unknown` in the
cost-log row.
