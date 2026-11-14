# EXECUTION RECORD: encode an exact local reference string (#894, IDN-09)

Contract: scope `docs/scopes/894-encode-exact-ref-string.md` (PR #1288, final head `3ad7e7230ad14d5f00c5090443255ceab4cf4e18`) and token proposal (PR #1291, final head `33d16d863aabb8abf1323aeff45c8cb337eb266d`), both squash-merged into `develop` after the Owner's integration authorization (section 7). G2 was granted by the Owner on 2026-10-29 on `d65884dbbb5aa5c17b9efa197c43558b0385a6c7` and `5ac4ae89ecc47a1acbb989fec0d632ac4be04255` (complete message quoted in the scope); the post-G2-SHA modification (release the reservation before an exhaustion panic propagates) is in the scope. Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, base `develop` `899d93e0f13f2e7673652b2a060329d9ee8f6b51` (unchanged at the last fetch). Stack of draft PRs, all `Refs #894`, never `Closes`, in order **K #1301 (`f193f2b444b42dced9dc1e10e08074ca5aaf8432`) -> C1 #1302 (`28219ecbdaa7003e2401ccc8590f29ed23dc7a36`) -> C2 #1303 (`00d64480ceaeeee7f3e1a507a9395c7e0ee00a38`) -> R (this commit)**. **Gate evidence is bound to C1 `28219ec` and C2 `00d6448`; any later commit to them invalidates it. R adds only documentation. The Owner authorized the integration afterwards (section 7); issues are not edited or closed and the #1020 index is not created. QA of these heads is in section 8.**

## 1. Authorization

Authorized by the G2 message: kit, implementation, independent QA. At G2, not authorized: merge, issue edits or closure, the #1020 index, remote CI, `-race` (the integration was authorized later: section 7; the rest stays unauthorized). The Go-only cases are accepted as the evidence **method**; their fulfilment requires running and reviewing the tests (sections 4 and 6). The optional Classic serialize-after-stop scenario (E6) was **not run: proposal only** (not accepted by the Owner). Kit package K has no probe.

## 2. Run

One Dev subagent (Claude Sonnet 5.5) ran the kit by hand under the RULEBOOK and RUNBOOK of K: tests first with RED kept (outside the repository), minimal code, gates. The kit queue runner was not used. Tokens and wall clock were not measured (`unknown`); one subagent is counted (lower bound). Commits: K `f193f2b` (kit package); C1 `28219ec` (`id.go`, `ask_endpoint.go`, `spawn.go`, `id_counter_internal_test.go`); C2 `3e87607` (`ref_string.go`, `ref_string_test.go`, README) and `00d6448` (test for the non-actor ID branch). Authored lines (additions + deletions, merges excluded): K 75; C1 258 (253 + 5) against K; C2 258 (256 + 2) against C1; R is this file plus one cost-log row. All under 400, so no follow-up PR was needed.

## 3. RED, mutants, differential, benchmark

- **RED C1** (a wrapping `next()` stub): C1, C2, C3, C4, C5, C7, C8 failed; C6 passed. **Unfixed `commitCell` mutant** (counter fixed, release absent): C7 (both cases) and C8 failed (`expected 2 to equal 1`, `expected 1 to equal 0`): the Owner's required behavior is real, the tests do not pass without the `defer`. Then GREEN (12 subtests).
- **Own mutants on the counter**, each run in a scratch copy: wrap instead of panic (RED above); add-then-check (`Add`, panic on 0): killed by C2, C3, C4; two counters for the two minters: killed by C4b; release removed: killed by C7 and C8.
- **RED C2** (stub returning `("", false)`): E1-E4, E6, E8, E9 and the README check E11 failed; E5 and E7 passed because they pin negative answers.
- **Differential** (scratch copies of develop `899d93e` and of C1, public API driver: three named spawns, two anonymous, two asks, a spawn, a duplicate): the ID text and order (`actor-4` ... `actor-8`, `ask-9`, `ask-10`, `actor-11`) and the errors are identical, 11 lines, stderr empty on both, exit 0.
- **Benchmark** (16 cores, macOS, 2,000,000 iterations, 5 runs): serial `nextID` 15.5 -> about 22.5 ns/op, `nextAskID` 16.2 -> 21.9 ns/op; parallel `nextID` (all cores hammering the counter) about 65 -> about 360-540 ns/op. This is the declared cost of the CAS loop; in production `nextID` runs under `System.mu` per spawn.

## 4. Gates (macOS arm64, Darwin 25.3.0, go1.27.2, golangci-lint v2.14.0)

| Gate | C1 `28219ec` | C2 `00d6448` |
|---|---|---|
| `go test -count=1 ./...` (pekko-actor), `gofmt -l`, `go vet`, `go mod tidy -diff` | 0 | 0 |
| `scripts/check_deps.sh`, `go-sdk-update.sh --check` | 0 | 0 |
| golangci-lint (gocyclo <= 10, no `//nolint`) | 0 (0 issues) | 0 (0 issues) |
| coverage recipe + `check_coverage.sh 85` | 0, pekko-actor 98.65% | 0, pekko-actor 98.67% |
| diff-cover 11.0.0 >= 85% against `origin/develop` | 0, 100% (17 lines, 0 missing) | 0, 100% (34 lines, 0 missing); a first run was 94% (2 lines of `encodeExact` unreached), fixed by the second commit |
| `make ci-fast`, `git diff --check` | 0 | 0 |
| `check-headers.sh` byte-compared with the develop output | exit 1, 7 errors, diff empty | exit 1, 7 errors, diff empty |
| `unshare -rn` | 127 on macOS (no network isolation, accepted G-6) | same |

Coverage of the new code: every line of `id.go`'s new functions, of `commitCell`'s release, and of `ref_string.go` is covered (0 missing lines). The combined candidate gates (each_module test/gofmt/vet, `make parity-guard`, ci-fast, coverage, diff-cover) are in the body of PR #1303 and the candidate comment, because the candidate head cannot be written in this record. diff-cover against the base branch was not run separately: K adds no Go, so the lines equal those against develop (reasoning, not a measurement).

## 5. Go-only rows and their tests (accepted as method; met only by running and reviewing the tests)

| Go-only row | Test (`pekko-actor/actor`) | Result |
|---|---|---|
| token form, one `#`, Go side of E1, E3, E6 | `ref_string_test.go` `TestExactRefStringOfLiveRefs` E1, E3, E6 | pass |
| epoch (16 lowercase hex, never zero, constant), token length | `TestExactRefStringTokenBuilder` E9 | pass |
| `("", false)` results | `TestExactRefStringNotEncodable` E7 | pass |
| root and guardians refused | `TestExactRefStringOfLiveRefs` E2 | pass |
| foreign System handling | `TestExactRefStringNotEncodable` E8 | pass |
| determinism | `TestExactRefStringOfLiveRefs` E4 | pass |
| ID numeric contract | E9 (`idNumber` table), E11 `TestReadmeStatesTheExactRefStringContract` | pass |
| counter exhaustion, permanence, concurrency, shared minters | `id_counter_internal_test.go` `TestIDCounterEdges` C1-C6 and C4b | pass |
| release of the reservation, owner stop, no ID reuse, no partial actor, other reservations untouched | `TestExhaustionPanicInsideCommitReleasesItsReservation` C7, `TestExhaustionReleasesOnlyItsOwnReservation` C8 | pass |

Pekko analogues (READ and Executed rows of K, not re-run here): E1 (row 15), E3 (rows 183-185), E6 (`ActorRef.scala:188-190`, READ only).

## 6. Deviations and limits

- **Naming:** the process-wide counter is `idSeq` (its former name, now an `idCounter`), not `ids` as the scope text says, because `hierarchy_internal_test.go` already defines a function `ids` in the package (a compile error otherwise). No API, token, counter-design, release or Allowed-files item changes; flagged for QA.
- `ask_endpoint.go`: besides the `nextAskID` line, the now-unused `strconv` import is removed (a compile necessity). An extra test C4b (the two minters share the process counter) was added to kill the separate-counter mutant. `hierarchy.go` is not edited.
- The C7/C8 seam `idMint` and the tests are the ones of the scope; the only test that touches shared state is C7/C8 (swap and restore, no parallel run).
- One macOS machine, no `-race`, no remote CI, no network isolation, no JVM run of Go; no parity is claimed. The QA is in section 8 (independent subagent reviews; no human review is claimed). The Owner's decisions on issue edits, the #1020 index and the open QA minors are not made.

## 7. Integration (after the Owner's authorization)

Owner, 2026-10-29 (verbatim): "Autorizo integrar #894 en el orden propuesto". Method: a squash merge of each PR into `develop` in the order scope, proposal, K, C1, C2, R, each bound to its pushed head by `--match-head-commit`, with a normal merge of `develop` into each successor before its squash (no rebase, no force-push, no `--admin`). Squash commits on `develop`: scope #1288 `97950a847b5687cf91388d9ea2c25db21033ef56` (reviewed head `3ad7e72`, merged unchanged); proposal #1291 `00cb15c24c2317e14eb6d2d81aefeef699ee38a1` (merged head `a258e2a817351cb77de5979db01563ddf34cb7e5` = QA-approved `33d16d8` plus develop; the delta is the scope file); K #1301 `e9b033d5cc086ee36ea595158e92494a75843328` (merged head `82f6a27705ee22b8ca53aaaa3834f69c4a5efb6e` = QA-approved `f193f2b` plus develop; the delta is the scope and proposal files); C1 #1302 `fb1b1937063ef7c764448051b58f6f386d4d1f2a` (merged head `8527c78bdcc7a52957898d533cf4abe02f614b83`; the Go diff against the QA-approved `28219ec` is empty); C2 #1303 `2debfcd81c77c917e8574081d6912376cd3fef64` (merged head `fe910582965708f47565f2a738d22ccd619f9f83`; the Go diff against `00d6448` is empty; `go test -count=1 ./...` in `pekko-actor` exits 0 on it); R #1304: its own squash commit cannot be written here.
- No conflict occurred in any merge of `develop` into a successor. The merges of `develop` into K, C1, C2 and R changed no Go file against the reviewed heads.
- The QA covers the reviewed heads in section 8, not the merged heads above (they add only earlier squashes to the reviewed content). The remote checks are not required and no remote CI was requested. The ADR 0031 addendum was accepted by the Owner in the terms presented, but its application to `docs/adr/0031` is a separate step not done here; #894, #1186 and #812 stay open; the #1020 index is not created.

## 8. Independent QA (relays of read-only subagent reviews; not human reviews, not Owner decisions)

| PR | Reviewed head | Verdict | Open minors |
|---|---|---|---|
| scope #1288 | `3ad7e72` | APPROVE | none |
| proposal #1291 | `33d16d8` | APPROVE | cosmetic nit |
| K #1301 | `f193f2b` | APPROVE_WITH_MINORS | RULEBOOK line 17 names the counter `ids`; the code uses `idSeq` |
| C1 #1302 | `28219ec` | APPROVE_WITH_MINORS | the `committed` flag is equivalent on the success path |
| C2 #1303 | `00d6448` | APPROVE_WITH_MINORS | the README link to the scope resolves once the scope is on `develop` (done by this integration order); the epoch redraw branch is untestable |
| R #1304 | `772fa35` | APPROVE_WITH_MINORS | diff-cover against the base branch was reasoned, not measured |
| local candidate | `61c8ec9` | APPROVE | not applicable (not pushed) |

The mutation round (21 mutants on the counter and the encoder: the meaningful ones killed; the survivors are equivalent or untestable) and the benchmark (parallel `nextID` about 65 -> 360-470 ns under 16-way contention) are in the C1 and C2 relays. The comment URLs are on PRs #1288, #1291, #1301, #1302, #1303 and #1304.
