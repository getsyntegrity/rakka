# EXECUTION RECORD: sender of the parent's decider (#814 Part 1, unit msg-02-failure-sender)

Authority: scope `docs/scopes/814-failure-sender-decider.md`; RULEBOOK approved by the Owner, SHA-256
`2b26ae33ea421ef82a24331ff96937dc3818cb2b6d37c633322a12ceb77c1548`; ADR 0032 D5a (`d5a_accept`) and D5b (follow Pekko).
Base `0ecfc3cff5304c812ca57686b6b365b2cf4c62ad`. Pekko pin `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (read, plus the
JVM probe of scope section 8 for G1, G2, G4, G6). Environment: darwin/arm64, go 1.27.2, golangci-lint v2.14.0. Code
SHAs: tests and fix at `83a452f23db8a962dd83842773909845a49338d5`; README at `12211d52f86b62aa4d9a13635dfb767005b3b64a`;
reviewed head `7712e471b148d25f75467740dd9bf8f6029df9c0`. `git diff 12211d5 7712e47 -- pekko-actor` is empty, so the
gates ran on a tree equivalent to 83a452f plus the README change. Evidence source: the body of [PR #1275](https://github.com/getsyntegrity/gopekko/pull/1275) (code PR, 396
authored lines); this record is its docs companion (PR #1276).

## Change

`handleFailed` stores `setSender(child)` instead of `nil`; `TODO(MSG-02-failed)` is deleted. No export, no new type.

## Evidence (kept in four columns, never merged)

| Criterion | Pekko reading | Go test | JVM differential |
|---|---|---|---|
| G1, G2 | m10, m12 | `TestSenderOfTheDecider` G1, G2 | observed (`pekko-repro/`: `decider sender=child-c` in the G4 and G6 scenarios) |
| G3 | m10, m8 | G3 | none |
| G4, G5 | m13, m7, m6 | G4, G5 | G4 observed, G5 none |
| G6 | m6, m10 | G6 (both) | observed for the Stop case |
| G7, G8 | m10, m3 | G7, G8 | none |
| G9 | m10 | `TestStaleFailedSender` | none (Go-exclusive) |
| G10 | m10 | G10 | none |
| G11 | m10 | `TestSenderDuringFailures` F8 rewritten | none |
| G12 | unchanged code | `TestDeciderDirectivesUnchanged` (4 subtests: Resume, Restart, Stop, Escalate) | base vs candidate, below |
| ADR 0032 mechanism 1 (incarnation) | `ActorRef.scala:183-186` | G13 | not covered by the probe |

## Cross-check (read from git, nothing executed)

Verified against the repository, not by running anything: Pekko pin `9a54d12...` equals `git -C upstream/pekko rev-parse HEAD`;
the RULEBOOK SHA-256 equals `shasum -a 256` of `RULEBOOK.md` on `origin/squad-c/814-kit-and-jvm-evidence`; `pekko-repro/observed.txt`
on that branch holds the decider sender (G4, G6 scenarios), the PreRestart sender and `Failed` message (G4) and the PostStop
sender under Stop (G6), plus Part 2 lines (T1-T3) not claimed here; `TestDeciderDirectivesUnchanged` on
`squad-c/814-part1-decider-sender` has 4 subtests (Resume, Restart, Stop, Escalate).

## Observed RED and mutation checks

On the base, with the new tests and no fix: F8, G1, G2, G3, G4, G6, G7, G9, G10, G13 fail; F1-F7, F9, P1, P2, G5, G6
(idle), G8 and the four G12 subtests pass (they pin behavior that did not change). Reinstated bugs, each reverted:

| Mutation | Failing tests |
|---|---|
| `nil` store (the base) | F8 G1 G2 G3 G4 G6 G7 G9 G10 G13 |
| sender set after the early return | G9 |
| sender = the child's retained message sender | F8 G1 G2 G3 G4 G6 G7 G10 G13 |
| clear on return from `handleFailed` | G4 G6 |

## Differential

The new test files were run in a `mktemp -d` copy of the base source (`GOWORK=off`): the unchanged-behavior tests above
pass on both; only the criteria that ask for the failing child differ. The whole existing module suite passes on the
candidate (`scripts/each_module.sh go test -count=1 ./...`, exit 0).

## Gates (darwin/arm64, go 1.27.2, golangci-lint v2.14.0)

| Gate | Command | Exit | Result |
|---|---|---|---|
| format | `gofmt -l` | 0 | empty |
| vet | `go vet ./...` (pekko-actor) | 0 | clean |
| tests | `scripts/each_module.sh go test -count=1 ./...` | 0 | all modules |
| lint | golangci-lint (gocyclo 10, no nolint), pekko-actor | 0 | 0 issues |
| coverage | `scripts/check_coverage.sh` at 85 | 0 | pekko-actor 98.61% |
| diff coverage | vs origin/develop | n/a | 100% (1/1 changed production line) |
| deps | `scripts/check_deps.sh` | 0 | pass |
| parity | `make parity-guard` | 0 | pass |
| CI fast | `make ci-fast` | 0 | pass |
| headers | `.github/scripts/check-headers.sh` | 1 | 7 errors, all in `pekko-persistence*`; the same 7 on the base `0ecfc3c`; unrelated |

Size: 396 authored lines (code, tests, README) in PR #1275, counted with `git diff --numstat 0ecfc3c...HEAD`. A G9
internal test lives in `sender_failure_internal_test.go`, outside the scope's file list (test only), because an
unregistered child at a chosen moment has no public path.

## Not run / limits

- No `-race`; no GitHub Actions or remote CI.
- `unshare -rn` not run (Linux only; this machine is darwin).
- `check-headers.sh` fails with exit 1 and 7 errors in `pekko-persistence*`, identical on the base.
- JVM differential only for G1, G2, G4, G6 (scope section 8); G3, G5 and G7-G13 have Pekko reading plus Go evidence only.
- G7 is sequential, not with the parent suspended.

## Not covered

Part 2 (`Terminated`, `WatchWith`), `Forward`, `DeadLetter.Sender`, `deadLetters`, the typed API. The JVM probe does
not cover incarnation identity. G7 is run sequentially (the second failure after the first decision), not with the
parent suspended. No module parity is claimed.
