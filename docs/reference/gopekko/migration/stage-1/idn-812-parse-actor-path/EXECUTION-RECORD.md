# EXECUTION RECORD: parse local actor paths (#812, IDN-03)

Contract: scope `docs/scopes/812-parse-actor-path.md` (PR #1285, head `f26fccf30533e6d1ffad134ac700f985a2d49ca6`) and grammar proposal (PR #1290, head `369597b8dd226da16a0e8faa047dc2690fe12aac`), both squash-merged into `develop` after the Owner's integration authorization (section 9). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, base `develop` `abc94d55e635704db3ec30c6735227a4c9676048` (unchanged at the last fetch). Stack of draft PRs, all `Refs #812`, never `Closes`, in order **K #1306 (`3700deb754e167d4ff9fcdc2e9b9a38b0a04de80`) -> C #1307 (`5db52c8276e68a6f06c1dde27317be379dccf843`) -> C2 #1308 (`904621a2a364703b2f33cb83835b1e64d5f8ae1f`) -> R (this commit)**; **C must not merge without C2**. **Gate evidence was run on C `9254e985d94ad48e7683f3e4cc744f1421c8a6fb` and C2 `aee957f26ef775575ef91a1b2ed39b3da6636a53`; the later heads of K, C and C2 differ from those only by documentation (`git diff <old> <new> -- '*.go'` is empty for C and C2) and `go test -count=1 ./...` in pekko-actor passed on the new C2 head; any later change to Go files invalidates the evidence. R adds only documentation. The Owner authorized the integration afterwards (section 9); the #812 issue is not edited and the #1020 index is not created. QA of these heads is in section 8.**

## 1. Authorization (the Owner message as the coordinator relayed it)

The Owner's words, as relayed to the coordinator, are only **"sí a todas, concedo el G2 de #812"** (2026-10-29). The coordinator's reading of that reply, as the Dev was given it, is: (1) **Q10 = yes**: the #1292 oracle is accepted as the Pekko evidence for the rows it covers, and **Go-only evidence (ADR 0033 D13) is approved as the method** for the uncovered and Go-only rows (P3, P4, P5, P9, P12, P13, P21, P22, P23, P24, X1-X9, the typed errors and classes, the round-trip claim), **met only by running and reviewing the tests**; the probe was **not extended** and no new Pekko probe was run; (2) `#` inside the system part is `ErrInvalidAddress`, in the path `ErrInvalidPath`, confirmed; (3) G2 on #1285 `f26fccf` and #1290 `369597b`. Authorized: kit, implementation, independent QA. **Not decided, not authorized at G2:** G10 option (ii) (`NewLocalAddress` enforcing the shapes), relative text (Q9, Q11-Q18), the owner of the exact-string parse, the #812 issue edit, applying the addendum to ADR 0031, merge, issue closure, the #1020 index, remote CI, `-race` (the merge was authorized later: section 9; everything else in this list stays undecided or unauthorized).

## 2. Run

One Dev subagent (Claude Sonnet 5.5) ran the kit by hand under the RULEBOOK and RUNBOOK of K: tests first with RED kept (outside the repository), minimal code, gates. The kit queue runner was not used. Tokens and wall clock were not measured (`unknown`); one subagent is counted (lower bound). Commits: K `4f62eb3` and `3700deb` (kit package, then the C2 row and the wording correction); C `9254e98` (`parse_actor_path.go`, `parse_actor_path_test.go`, `errors.go` doc comments, README), then the normal merge of K `5db52c8`; C2 `df40303` and `aee957f` (`parse_actor_path_position_test.go`, then a staticcheck fix) and the normal merge of C `904621a`. Authored lines (additions + deletions, merges excluded; K 75 after the K1/K2 correction, from `git diff --numstat origin/develop`): K 75; C 338 (334 + 4) against K; C2 132 against C; R is this file plus one cost-log row. **The tests were split because one PR would have been 470 lines (338 in C against K and 132 in C2 against C, from `git diff --numstat` against the bases: 334 + 4 and 132 + 0)**: the position-rule, criterion 2b and AC3 tests moved to C2 (the Allowed files table permits a second test file); the contract is unchanged. `address.go`, `actor_path.go` and `names.go` are untouched (`git diff --stat`: only the Allowed files changed).

## 3. RED, mutants

- **RED** (a stub `parseActorPath` returning the zero `ActorPath` and a nil error): every row test failed (RoundTripTable, VerbatimPercent, EmptySegmentsAndSlash, MalformedEscape, IllegalElementChars, DollarAndStarElements, HashIsError, QueryIsError, MalformedAddressTable, OtherProtocol, HostAndPortExcluded, SystemCharset including the P24 consequence, DotDotElement, ErrorClassPosition, RoundTripDescendant); the two AC3 checks passed on the stub because they pin a negative (nothing is drawn). Then GREEN on the first implementation (37 passing subtests).
- **Ten own mutants** of `parse_actor_path.go`, each in a scratch copy, all **killed**: position rule swapped (4 failing subtests), `#` accepted (2), trailing slash accepted (3), protocol folded to lower case (2), system charset widened (1), elements containing `%` skipped (6), empty element skipped (3), `..` navigated (4), missing root slash accepted (3), host/port message removed (1). The mutant run used both test files; QA runs the independent mutation round.

## 4. Gates (macOS arm64, Darwin 25.3.0, go1.27.2, golangci-lint v2.14.0)

| Gate | C `9254e98` | C2 `aee957f` |
|---|---|---|
| `go test -count=1 ./...` (pekko-actor), `gofmt -l`, `go vet`, `go mod tidy -diff` | 0 | 0 |
| `go test -count=3 ./actor/` (flakiness; not `-race`) | 0 | 0 |
| `scripts/check_deps.sh`, `go-sdk-update.sh --check` | 0 | 0 |
| golangci-lint (gocyclo <= 10, no `//nolint`) | 0 | 0 (the first C2 run found staticcheck SA4000 in the AC3 test; fixed in `aee957f`) |
| coverage recipe + `check_coverage.sh 85` (pekko-actor) | 0, 98.60% | 0, 98.60% |
| diff-cover 11.0.0 >= 85% against `origin/develop` | 0, 93% (66 lines, 4 missing: lines 35-36 and 39-40, the unreachable error returns after `NewLocalAddress` and `NewRootPath`) | same |
| diff-cover against the base branch | 0, 93% (run for C against `docs/812-kit-package`); for C2 not run separately (tests only, no new executable line) | not run |
| `make ci-fast`, `git diff --check` | 0 | 0 |
| `check-headers.sh` byte-compared with the develop output | exit 1, 7 errors, diff empty (see note) | exit 1, 7 errors, diff empty |
| `unshare -rn` | 127 on macOS (no network isolation, accepted under G-6) | same |

Note: the first header run for C reported a spurious difference caused by a stale `git add -N` entry of a file I had moved to the next PR (`head: ... No such file`); I cleared the index entry and re-ran: identical to develop. The new files carry no `Ported from` marker (the doc comment says "adapted from"), so `check-headers.sh` does not check them and adds no error. `make parity-guard` and the other stack-wide gates are run on the candidate (see the PR bodies; the candidate head cannot be written here).

## 5. Go-only rows and their tests (approved as method; met only by running and reviewing the tests)

| Row | Test (`pekko-actor/actor`) | Result |
|---|---|---|
| P3, P4, P5 (missing slash, trailing slash, empty segment: `ErrInvalidPath`; Pekko accepts, rows 39/44, 45/50, 51/56) | `TestParse_EmptySegmentsAndSlash` | pass |
| P9 (space, non-ASCII, control character in an element) | `TestParse_IllegalElementChars` | pass |
| P12, P13 (`#` in the path; Pekko rows 69, 74, 123, 135, 141) | `TestParse_HashIsError` | pass |
| P21 (the zero path) | `TestParse_RoundTripTable` (P21 subtest) | pass |
| P22 (`my_sys` accepted; Pekko rows 81-86 reject) and P24 (shapes) | `TestParse_SystemCharset` | pass |
| P23 (`?query`; Pekko rows 75-80) | `TestParse_QueryIsError` | pass |
| X1 (protocol kept as written) | `TestParse_OtherProtocol` | pass |
| X2 (protocol shape) | `TestParse_MalformedAddressTable` | pass |
| X3, X7, X8, X9 (position rule, corner cases, generated strings vs an independent grammar) | `TestParse_ErrorClassPosition` (C2) | pass |
| X4 (host and port forms name the unsupported form) | `TestParse_HostAndPortExcluded` | pass |
| X5, X6 (`.` and `..` legal, no navigation) | `TestParse_DotDotElement` | pass |
| typed errors and classes, the round-trip claim | all rows; `TestParse_RoundTripTable` (table and generated property) | pass |
| criterion 2b of #852 | `TestParse_RoundTripDescendant` (C2) | pass |
| AC3 (no ID drawn, no actor, no registry change, equal value before and after stop) | `TestParse_NoLookupNoActor` (C2) | pass |

Rows decided by an accepted ADR or existing code are re-verified by the same tests (P1, P2, P6-P8, P10, P11, P14-P20). Executed Pekko rows are cited by number in the tests and the manifest, re-checked against `observed.txt` at develop `abc94d5`.

## 6. The pinned P24 consequence

`TestParse_SystemCharset` pins the documented behavior of the Owner's Q4 choice (the shapes are enforced by the parser only; G10 option (ii) is **not decided**): `NewLocalAddress("pekko", "a/b")` is constructible, the text of its child `x` is `pekko://a/b/x`, and the parser reads it as system `a`, path `b/x`, which is a different path; the shape `a b` is rejected instead of misparsed. The test is named and commented so a future G10 decision changes it knowingly. The README states the limit.

## 7. Deviations and limits

- The tests are split across two PRs (section 2): **C must not merge without C2**; with C alone three mutants survive (a system charset admitting `.`, whitespace trimming, a parser calling `nextID()`), as the QA of C reported. The parser's two defensive error returns after `NewLocalAddress` and `NewRootPath` are unreachable and uncovered (93% diff coverage); they are kept to avoid ignoring errors.
- **Breach of a standing rule, disclosed:** while verifying SHAs I ran one read-only `git cat-file -e` with the working directory **inside `upstream/pekko`** (the rule is never to run anything there). Nothing was modified: QA verified from outside that `git -C <repo>/upstream/pekko status --short` is empty and that its HEAD is the pin `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. I re-verified the pin afterwards with `git -C`, and every SHA of this record with `git cat-file -e <sha>^{commit}` through `git -C` (the Pekko pin and the kit commit through their submodule paths).
- One macOS machine, no `-race`, no remote CI, no network isolation, no JVM run of Go; no new Pekko probe; no parity is claimed. The QA is in section 8 (independent subagent reviews; no human review is claimed). The #812 issue is not edited; the #1020 index is not created.

## 8. Independent QA, each bound to its SHA

Each verdict is a relay of an independent read-only QA subagent report, posted through the Owner's account: **not a human review, not an Owner decision, not a merge approval**. A verdict covers only the SHA it names. AWM = APPROVE_WITH_MINORS.

| PR | Head | Verdict | Where |
|---|---|---|---|
| K #1306 | `4f62eb30d6da5d2f3d860bd12c7992f2f8d8ca8b` | AWM | https://github.com/getsyntegrity/gopekko/pull/1306#issuecomment-6508799848 |
| R #1309 | `9984c19c5547fd8acd1375a100e2d2a929c31dc9` | AWM | https://github.com/getsyntegrity/gopekko/pull/1309#issuecomment-6508800265 |
| C #1307 | `9254e985d94ad48e7683f3e4cc744f1421c8a6fb` | AWM | https://github.com/getsyntegrity/gopekko/pull/1307#issuecomment-6508915383 |
| C2 #1308 | `aee957f26ef775575ef91a1b2ed39b3da6636a53` | AWM | https://github.com/getsyntegrity/gopekko/pull/1308#issuecomment-6508915875 |
| candidate (local) | `831d74a99c9d66017383d85040db4fc7e2b4b527` | APPROVE (superseded by the new candidate) | relayed by the coordinator |
| K #1306 | `3700deb754e167d4ff9fcdc2e9b9a38b0a04de80` | AWM (the PR body minor was fixed afterwards) | https://github.com/getsyntegrity/gopekko/pull/1306#issuecomment-6509089060 |
| R #1309 and candidate2 | `302a97657553288a40848f6e9be8d62d1669d1c5`; candidate2 `9e50f880589b04b4872092729d1a7dbd7a3f9aee` (local) | AWM; APPROVE | https://github.com/getsyntegrity/gopekko/pull/1309#issuecomment-6509089421 |
| C #1307, C2 #1308 | `5db52c8276e68a6f06c1dde27317be379dccf843`, `904621a2a364703b2f33cb83835b1e64d5f8ae1f` | no verdict of their own: the merges of K into C and of C into C2 changed only two documents and no Go file (`git diff <reviewed> <new> -- '*.go'` is empty); the verdicts above on `9254e98` and `aee957f` cover the Go | not applicable |
| R #1309 (follow-up commit) | `649615521c4fb4f9a7128df301f647b32ba899ef` | **no verdict**: one sentence of section 2 (the order of the commit list) was corrected after the review of `302a976` | not applicable |

Open minors, not fixed: the X9 generator alphabet omits `o` and `1` (a code test file; not changed, and it can be added in a follow-up if the Owner wants); four unreachable defensive lines (93% diff-cover); C needs C2 (above; both are now on `develop`).

## 9. Integration (after the Owner's authorization)

Owner, 2026-10-29 (verbatim): "Autorizo integrar #812 en el orden propuesto". Method: a squash merge of each PR into `develop`, each bound to its pushed head by `--match-head-commit`, with a normal merge of `develop` into each successor before its squash (no rebase, no force-push, no `--admin`); C and C2 were merged one right after the other because C must not stand without C2. Squash commits on `develop`: grammar proposal #1290 `6fb4030541de46a82bb573d5cc9cbe8cf84a696a` (reviewed head `369597b`, merged unchanged); scope #1285 `be76d28e8dc14f48194855f9c118951c26489cad` (merged head `f5f6bf0d1bdc5caa8f079edc3100aece01db24c9` = G2 head `f26fccf` plus develop; the delta is the proposal file); K #1306 `d1acc04ea7963277bf62b982e825b809f16db9dc` (merged head `83939f4f7c6a9fbd4d8f8ad1a96f6f1d11be5470` = reviewed `3700deb` plus develop; the delta is the two documents); C #1307 `60150ae9734c3136a614b26fd8b432dbaa264b36` (merged head `972f5e0eec91da20570a7b77a6416ef101445b12`; the Go diff against `5db52c8` is empty); C2 #1308 `b5a86a3d6587cf89a3b89a31c8afb8ce6f5f13c7` (merged head `0e49b3dc2602c8e1d937177f48a040d7fe119516`; the Go diff against `904621a` is empty; `go test -count=1 ./...` in `pekko-actor` exits 0 with C and C2 together); R #1309: its own squash commit cannot be written here.
- No merge of `develop` into a successor produced a conflict; none changed a Go file against the reviewed heads.
- The QA covers the reviewed heads in section 8, not the merged heads above. No remote CI and no `-race` were run. The #812 issue is not edited and stays open; the addendum of #1187 is not applied to `docs/adr/0031`; G10 option (ii), relative text and the exact-string parse owner stay undecided; the #1020 index is not created.
