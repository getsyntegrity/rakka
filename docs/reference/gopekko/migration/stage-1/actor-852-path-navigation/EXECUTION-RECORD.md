# EXECUTION RECORD: ActorPath navigation (#852, IDN-04)

Scope `docs/scopes/852-actor-path-navigation.md` (PR #1284; G2 approved by the Owner at `76810743150fea3a18be5e72bc39f2a5e627b5f6`; scope head when this record was written `fe93c8ca41d80d2ade9cb000e0a1bcbb6175072b`). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. Three stacked draft PRs, all `Refs #852`, never `Closes`: K #1294 (kit package), C #1295 (code, stacked on K), R (this record, stacked on C). C and R are not independent of K. **This record is bound to C's final head `d0c422a107cbc810604f4c3481e94db1f4f8bed8`; every later commit to C invalidates the gate evidence of section 4. PR R adds only documentation.**

## 1. Authorization and split

Owner G2 decisions (verbatim in the scope): (1) deliver `Parent` and `Descendant` with individual ADR 0011 justifications; (2) `Root` is outside this unit, not excluded from the general scope; (3) kit required per `AGENTS.md`; (4) `Parent` of the zero path returns the zero path; (5) `Descendant` on the zero path returns the zero path and `ErrInvalidPath` even with no or only empty names, and on a valid path no or only empty names return the receiver without error; (6) move the comparison with parsing to #812, keeping traceability and limiting it to the domain both operations accept. Scope limit: the approval covers #852 only, accepts neither #1187 nor #1186, grants no G2 to other units, authorizes no merge and no issue closure.

Later Owner instruction in the working session, 2026-10-29 (not a GitHub comment; recorded in the RUNBOOK of K): the unit may be split to keep each PR at or under 400 authored changed lines, keeping criteria and traceability; a contract-changing split needs a new G2. The split used (K, C, R) changed no criterion, scenario or symbol, so no new G2 was needed. The split into three PRs and the name of the kit package are proposals of the working session: they are not among the Owner's six G2 decisions and are not recorded as approved by them. No further split was required: K 71 authored lines, C 231 against K, R this PR (section 6).

## 2. Run

1. What happened: a Dev subagent implemented the scope under the RULEBOOK and RUNBOOK of the package (`RUNBOOK.md` "Procedure per PR"): tests first, RED kept, minimal implementation, gates. **The kit's queue runner (`migration-kit/scripts/queue_runner.mjs`) was not used**: the package holds only `manifest.tsv`, `RULEBOOK.md` and `RUNBOOK.md` (no queue on disk) and the RUNBOOK names no runner. No kit loop was run; this is a kit-governed manual run, not an automated kit loop. Independent QA subagents (not the Dev, not the TL) reviewed each PR.
2. Models: Claude Sonnet 5.5 (`claude-sonnet-5-5`) for the Dev, the writer of this record and the QA subagents. Tokens and wall clock were not measured (`unknown` in the cost-log row; the task notifications give per-run token usage, but resumed runs make a sum unreliable). Subagents that ran the unit steps K, C and R, counted from agent transcripts: 5 (cost-log row): Dev (`a51b1909465b5d631`), Dev continuation (`a1f57b4a6ce51a533`), writer of this record (`ad40b2820d341784c`), QA of K (`a9d7dd32e8d5447e3`), QA of C (`a573508969f3ebf1e`).
3. Inputs: the scope; the 7 manifest rows of `manifest.tsv`; Pekko `ActorPath.scala` lines 72-75, 188, 203-204, 209, 286, 290-293, 328-335; the Pekko oracle of PR #1292 (`observed.txt`, `Probe.scala`, `RUN.md`; head `d4ac1c479f478254bddd1f8dc44ea5d35f35eea1`, QA APPROVE), whose rows 1-10 and 69 the tests cite. Pekko was executed there in a disposable copy; Go was never run against Pekko.
4. Outputs: K: `manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md`. C: `pekko-actor/actor/actor_path.go` (`Parent`, `Descendant`), `actor_path_test.go` (S1-S11), `errors.go` comment, `pekko-actor/README.md` path paragraph. R: this file and one cost-log row.
5. Manual changes: none besides the commits below (source: `git log origin/develop..origin/feat/852-actor-path-navigation`).

| SHA | PR | Subject |
|---|---|---|
| `fea60fd` | K | docs(adr): add kit package for ActorPath navigation (#852) |
| `f9a2d45` | K | docs(adr): address QA findings on the #852 kit package |
| `a6a7aff` | C | feat(actor): add ActorPath.Parent and ActorPath.Descendant (#852) |
| `7ffe90c` | C | chore: merge docs/852-kit-package into feat/852-actor-path-navigation |
| `d0c422a` | C | test(actor): assert the rejection reason and cite the Pekko name#uid row in the Descendant tests |

## 3. RED and mutation evidence

RED (observed locally before the implementation; the output file is not in the repository, so it is not reproducible from the pushed branches): the new tests failed to compile with `p.Descendant undefined (type actor.ActorPath has no field or method Descendant)` and `root.Parent undefined (type actor.ActorPath has no field or method Parent)` (first error at `actor_path_test.go:194`; the compiler stopped at `too many errors`).

S8 mutants (Dev, after QA a6a7aff): a mutant returning a bare `ErrInvalidPath` and a mutant whose message lacked the `'#'` reason were killed by the strengthened assertions of `d0c422a` (`err.Error()` contains `'/'`, `'#'`, `outside`, `malformed`). Independent QA mutants a to m (13 in total): 12 killed, 1 equivalent survivor: `ActorPath{address: p.address}` in place of `elements: ""` as the zero return (an equal value, since the zero path has no elements). The 13 mutants are summarized (short description, not the full list) in the published QA comment on PR C (section 7); the full list is not stored in the repository and is not reproduced here.

## 4. Gates bound to C's final head `d0c422a`

Environment: macOS arm64 (Darwin 25.3.0), Go 1.27.2, golangci-lint v2.14.0, one machine. Run by the Dev and, separately, by the independent QA at `d0c422a` (QA re-ran all gates at the final head). Command forms are those of `AGENTS.md` "Go" and `docs/quality-gates.md`.

| Gate | Exit | Result |
|---|---|---|
| `cd pekko-actor && go test -count=1 ./...` | 0 | ok |
| `gofmt -l pekko-actor` | 0 | no output |
| `go vet ./...` (pekko-actor) | 0 | clean |
| `go mod tidy -diff` (pekko-actor) | 0 | no diff |
| `scripts/check_deps.sh` | 0 | ok |
| `.github/scripts/go-sdk-update.sh --check` | 0 | no drift |
| `golangci-lint run --config ../.golangci.yml ./...` (gocyclo <= 10) | 0 | 0 issues |
| coverage recipe through `each_module.sh` + `scripts/check_coverage.sh "$COV" 85` | 0 | pekko-actor 98.63% (1677 statements) |
| `diff-cover==11.0.0 ... --compare-branch=origin/develop --fail-under=85` | 0 | 100% (20/20 lines) |
| `make ci-fast` | 0 | ok |
| `.github/scripts/check-headers.sh` | **1** | 7 errors: `pekko-actor/actor/names.go` and six `pekko-persistence` snapshot files. Output byte-identical to a detached worktree of `origin/develop` `7840e88`: **pre-existing, not attributable to #852**. Not hidden; none of the 7 files is in this unit's diff. Evidence: the QA run on `d0c422a` and on that detached worktree gave byte-identical output (published C and R comments); the Dev ran the same gate list earlier. Not fixed because those files are outside the unit; fixing it needs its own unit or decision |
| `unshare -rn ...` | 127 (Dev run, captured); 1 (QA report only) | Dev: output captured by the Dev in a local gate log that is not stored in the repository: `bash: line 1: unshare: command not found` and `unshare exit 127`. QA: reported in its own words that `unshare` was unavailable (command not found) and gave exit code 1 for its own invocation; that value comes from the QA report only and is not captured in any file. `unshare` is Linux-only: on this macOS machine **no network isolation was possible for the unit tests**. `AGENTS.md` chose `unshare -rn ...`; its container variant (`docker run --rm --network none ...`) was **not run** |

Raw gate outputs are not stored in the repository and are not reproducible from it; the commands and exit codes are recorded here.

Not run: remote CI (GitHub Actions billing blocked; no remote result is attributed to any head); `-race` (not authorized by the Owner); `make parity-guard` (not applicable, persistence adapters only).

## 5. Differential and Pekko evidence

Differential for the shared code (`Child` shares `checkPathElement` with `Descendant`): `Child` on at least 15 names against a copy of the base commit in a `mktemp -d` directory (not committed), base and new both exit 0. Dev: 32 output lines, identical; QA: 78 lines, identical.

Pekko rows used by the tests (PR #1292, executed): rows 1-10 (`Parent`, skip-empty `descendant`, root elements) and row 69 (`fromString("pekko://sys/a#5")` splits `name#uid`: uid 5, element `a`). Accepted differences, not defects: Pekko splits `name#uid` while Go `Child` and `Descendant` reject `#` (S8, ADR 0031 P-7); zero-value rows (S3, S7 and zero cases of S5/S6) and error-class rows (S8, S9, S11) are Go design with no Pekko execution; S10 is a boundary note, non-falsifiable by construction. Criterion 2b (equality with the parsed form) moved to #812 and is **not asserted** here. **No parity is claimed**; Go was never run against Pekko and never on a JVM.

## 6. Size (authored lines, additions plus deletions)

K 71. C 231 against K. R: at `5b42018` R = 97 authored lines (96 record + 1 cost-log row); at `8615906` 103 (102 + 1); with this delta 106 (105 record + 1 cost-log row; `git diff --numstat origin/feat/852-actor-path-navigation..HEAD`). All three are under the 400-line budget.

## 7. Independent QA verdicts, each bound to its SHA

Author and role: each verdict is by an independent QA subagent (not the Dev, not the TL), read-only, Claude Sonnet 5.5. The published comments were posted by the orchestrating Claude Code session through the Owner's GitHub account on the QA's behalf, from the QA reports: they are relays of subagent reports, not human reviews, not Owner decisions and not merge approvals. Limit: a verdict covers only the SHA it names and what that QA reproduced; it is not a merge approval.

| PR | Head | Verdict | Where |
|---|---|---|---|
| K #1294 | `fea60fd` | APPROVE_WITH_MINORS | published: https://github.com/getsyntegrity/gopekko/pull/1294#issuecomment-6502138689 |
| K #1294 | `f9a2d45` | APPROVE | same comment |
| C #1295 | `a6a7aff` | APPROVE_WITH_MINORS | published: https://github.com/getsyntegrity/gopekko/pull/1295#issuecomment-6502139469 |
| C #1295 | `d0c422a` | APPROVE (QA re-ran all gates of section 4) | same comment |
| scope #1284 | `fe93c8ca` | APPROVE_WITH_MINORS | reported to the orchestrator; to be published as a comment on #1284 |
| scope #1284 | `48270153` | APPROVE_WITH_MINORS | reported to the orchestrator; to be published as a comment on #1284, bound to the head of its next delta (no verdict claimed for that head) |
| oracle #1292 | `d44c80c` | APPROVE_WITH_MINORS | published: https://github.com/getsyntegrity/gopekko/pull/1292#issuecomment-6502420169 |
| oracle #1292 | `d4ac1c4` | APPROVE | same comment |
| R #1296 | `5b42018` | APPROVE, two optional minors | published: https://github.com/getsyntegrity/gopekko/pull/1296#issuecomment-6502261650 |
| R #1296 | `8615906` | APPROVE (delta `5b42018..8615906`), two optional minors and a note; the minors are fixed by the next delta | same comment |
| R #1296 | this head | not claimed here | to be published as a comment on PR #1296, bound to that head |

Scope minors, still open: the disclosure list of post-G2 hunks is not exhaustive; the PR 3 sentence reads as a commitment; "this commit" is ambiguous. This record asserts no merge.

## 8. Traceability (criteria of the scope, tests of `pekko-actor/actor/actor_path_test.go` at `d0c422a`)

| Criterion | Scenario and test | Lines |
|---|---|---|
| 1 root and child parent | S1, S2, S3 in `TestParentOfTheRootIsTheRootAndOfAChildIsTheOneAbove` | 201-229 (S1 207-211, S2 212-220, receiver unchanged 221-224, S3 225-227) |
| 2a skip empties | S4, S5, S6 in `TestDescendantAppendsNamesAndSkipsEmptyOnes` | 231-265 (S4 236-245, S5 246-252, S6 253-258) |
| 2a zero receiver | S7 in `TestDescendantOnTheZeroPathFailsWhateverTheNames` | 267-279 |
| 2a negative | S8, S9 in `TestDescendantRejectsABadNameAndBuildsNothing`; S11 in `TestDescendantAndChildAgreeOnEveryNonEmptyName` | 281-309 (S8 291-299, S9 300-307); 311-334 |
| 2b parsing equality | not asserted: moved to #812 by Owner decision 6, not asserted; the final shape of the comparison is an Owner decision pending in #812's G2; blocked by #812 / #1187 | none |
| 3 no dependency on a running actor | satisfied by construction (no `System` parameter); S10 `TestNavigationDoesNotDependOnARunningSystem` is a non-falsifiable boundary note | 336-357 |
| 4 positive and negative cases, SHA and gates | S1-S11 above; SHA and gates in sections 2 and 4 | |
| 5 README and traceability, no parity claim | `pekko-actor/README.md` path paragraph in C; section 5 here | not a test |

## 9. Open items and limits

- The scope minors of section 7.
- Criterion 2b is pending #812 and the grammar decision #1187; empty-segment comparison against the parser is pending #1187.
- `check-headers.sh` exits 1 on 7 pre-existing files (section 4); a green repository-wide header gate is not claimed.
- Evidence comes from one macOS machine, without network isolation, without `-race`, without remote CI, with no run of Go on a JVM; the README of `pekko-actor` says so.
- This record changes no code, contract, issue or label. Merging, closing #852 and removing labels are Owner decisions.
