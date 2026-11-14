# EXECUTION RECORD: Expose `deadLetters` and carry the sender in `DeadLetter` (#896, MSG-08)

Scope `docs/scopes/896-deadletters-ref-and-sender.md` (PR #1503, approved at gate G2 by the Owner on 2026-11-10 with D-1 to D-10 as written). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`), scope written against `develop` `46ce37ac`. Every PR says `Refs #896`, none `Closes`: the Owner closes #896 by hand after slice 2, with a comment that AC2 moved to #1087. This slice (2) is docs only: **no mutation applies**. Every fact below is taken from the PR bodies and QA comments (`gh pr view <n> --comments`); where a PR did not say it, this record does not either.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`). Tokens, wall clock and the number of subagent runs were not measured (`unknown` in the cost-log row).
2. Outputs: `DeadLetter.Sender` (`deadletter.go`), the carrying sites (`ref.go`, `stop.go`), `(*System).DeadLetters()` and the sink (`deadletters_ref.go`), the substitution in `System.publish`, the unexported `senderTeller` path, the D-5 `Terminated` sender (`watch.go`), tests (`deadletter_sender_test.go`, `deadletters_ref_test.go`, `deadletters_ref_internal_test.go`, P1 and S8 edited, 17 literals migrated), the kit package, the README section "Dead letters: the sender and `deadLetters`", this record and one cost-log row.
3. Deviations from the scope: the ID and path proposal of D-7 moved from slice 0 to slice 1b (RULEBOOK section 3, approved by the Owner on 2026-11-10); no slice 1c was needed (1b measured 383 lines); the scope's attribution of the port sites to `port.go:81, 118, 168` read "without PortHandler, postLane, publishPortDead" and the code has them as `postLane` (81), `deliverPort` without a handler (118) and `publishPortDead` (168); the README uses the code's order.

## 2. PRs

| Slice | PR | Head SHA | State at this record | Lines (authored) | Content |
|---|---|---|---|---|---|
| scope | #1503 | `64a8722c` | merged | n/a | scope, draft for G2 |
| ADR 0038 amendment (c) | #1504 | `dfd4e6d5` | merged | n/a | D7 names #896 for the sender (editorial; scope D-10) |
| 0 | #1509 | `fcdc9ec0` | merged, `bcc8ddbf` | n/a | kit package: manifest, RULEBOOK, RUNBOOK |
| 1a | #1513 | `d277d90c` | merged, `520fc489` | 159 (142 + 17) | `DeadLetter.Sender`; `tell` and `stop.go` pass the sender; P1 asserts it; `PORT-GAP(dl-sender-896)` removed from code |
| judge anchors | #1515 | `b013614d` | open | 8 in `judge/` + 52 in `PROVENANCE.md` | repairs the GM6a, GM6b and GM62 anchors that 1a broke and records the Go campaign on `22b7e71` (not part of the scope; needed by the checker) |
| 1b | #1517 | `b85ba16b` | open, base `fix/1513-go-anchors-sender` (#1515) | 383 (354 + 29) | `System.DeadLetters()`, the substitution in `publish`, the `TellFrom` path to the sink, D-5, literal migration |
| 2 | this PR | see `git log` | open, base `feat/896-slice1b-sink` (#1517) | see its description | README, this record, cost-log row |

Estimates in the scope: 1a ~210 (measured 159), 1b ~380 (measured 383), 2 ~80. All are under the 400 budget.

## 3. RED evidence: mutations (by hand, disposable worktree, never committed)

| Criterion | Mutation | Failing test | PR |
|---|---|---|---|
| S1 | `tell` publishes a nil sender | P1 `TestActorSystemSpec_LogDeadLetters`, `TestDeadLetterCarriesTheSender` and the S8 case of `sender_test.go` | #1513 (M1) |
| S2 | `stop.go` drops `e.sender` | the pending-at-stop case of `TestDeadLetterCarriesTheSender` | #1513 (M2) |
| S3 | `publish` does not substitute | `TestDeadLetterWithoutSenderIsDeadLetters` (and `TestDeadLetterCarriesTheSender`) | #1517 |
| S4 | `tellFrom` wraps a `DeadLetter` again | `TestDeadLettersRefPublishes` | #1517 |
| S5 | the sink is registered in `byID` (a stub record) | `TestDeadLettersIsNotAnApplicationActor` | #1517 |
| S7 | `publish` reuses the previous letter's sender | `TestDeadLetterSenderUnderConcurrency` (`-count=100`) | #1517 |
| D-5 | the `Terminated` letter is published without its subject as sender | `TestDeadLetterWithoutSenderIsDeadLetters` | #1517 |

A first attempt at the S5 mutation (registering a nil record) survived, because `lookup` returns nil for it; it was replaced by a real stub record and then failed by assertion (#1517 body). Tests were written after the implementation in 1a; the red state there is shown by mutation only (#1513 body). S6 and S8 are regressions (no mutation). QA reproduced the 1a mutations (both `ref.go` sites to nil, each site alone, `stop.go`) and re-ran 5 mutations of 1b, each red by assertion.

## 4. Gates, as reported in the PRs (macOS, no network isolation)

- **1a (#1513):** build and vet clean, `go test` ok in all 11 modules, `check_deps.sh` ok, `gofmt -l` empty, `pekko-actor` coverage 98.3%. Not run by the Dev: pinned golangci-lint, diff-cover. QA: `go build`, `go vet`, `scripts/check_deps.sh` exit 0, `go test` of `pekko-actor` ok (exit code read from the `ok` lines); golangci-lint and the coverage gate not run.
- **1b (#1517):** `scripts/each_module.sh go build ./...` 0; `go vet ./...` 0; `scripts/each_module.sh go test -count=1 ./...` all 11 modules ok (typed and persistence compile); `gofmt -l pekko-actor` empty; `scripts/check_deps.sh` 0; golangci-lint in `pekko-actor` 0 issues (`gocyclo -over 10` empty); coverage `pekko-actor` 98.4%; S7 `-count=100` ok. QA: `TestDeadLetterSenderUnderConcurrency` at `-count=20` ok; golangci-lint 0 issues.
- **`.github/scripts/check-headers.sh`** exits 1 with 7 errors (`pekko-actor/actor/names.go` and six `pekko-persistence` files), identical on `origin/develop` `c5e62a3d` (QA of #1513); none in this unit's diff.
- `grep dl-sender-896` over `*.go` is empty (DoD). The string remains in the documents listed in section 7.
- **Slice 2 (this PR):** docs only. `python3 scripts/check-judge-matrix.py`: exit 0. It prints `goPekko CURRENT on linux/amd64 ... detected 0/21 ... an input of the campaign changed since it ran` (and `goPekko HISTORICAL ... detected 21/21 designed`). The manifest check of the RUNBOOK prints nothing.

## 5. Not run

Network isolation (`unshare -rn` or `docker --network none`; macOS, the Owner runs the container variant), `-race` (not asked), `diff-cover` (module and new code far above 85% by the coverage report), a JVM scenario for the new behavior, and the Go judge campaign. The Evidence comment of the scope's RUNBOOK (Pekko lines as `path:lines -> first line`) was not posted on #1517 (its body says so). `selftest_go_report.py` timed out at the QA's 100 s cap on #1517 (the Dev reports exit 0); `selftest_mutate_go.py` exit 0. QA did not re-open `Mailbox.scala:434, 567`, `ActorRef.scala:689, 712`, `ref_string.go` or ADR 0031.

## 6. Judge

- Slice 1a broke the anchors of GM6a, GM6b and GM62 (`ref.go`, `stop.go`), so `check-judge-matrix.py` failed on `develop`; #1515 repairs them and records a full Go campaign on `22b7e71` (22/22 met, 21/21 designed detected, CURRENT on that commit).
- Slice 1b changes the Go core after that campaign: at `b85ba16b` the checker exits 0 and prints goPekko CURRENT 0/21. The anchor check `mutate_go.anchor_problems("pekko-actor/actor")` returns `[]`, so no anchor broke (#1517 body and QA).
- **New judge rule 4** (as quoted by the QA of #1517; the text of the rule is not in the repository files read for this record): a broken anchor, which the checker reports as exit 1, is a REJECT, and **a new Go campaign runs after the merge**. #1517 is exit 0 and was approved under the Owner's rule of 2026-11-10; the campaign is not run in this unit.
- **The Go campaign must be rerun after #1515, #1517 and this PR merge** (and, per the scope, after #871) to restore goPekko CURRENT. Until then the Go CURRENT is stale.
- Owner filter B (PR #1518, open, 2026-11-10): CURRENT binds the core files of `pekko-actor/` and excludes `*.md`. Until #1518 merges, the README edit of this PR also invalidates CURRENT; after it, a README-only change does not. #1518 itself edits `judge/mutate_go.py`, a comparator input, so it too needs a campaign on the merged head (its body).

## 7. The `dl-sender-896` marker in documents

Slice 2 may only touch the README, this record and the cost-log (RULEBOOK section 1, scope "Allowed files"), so none of these is edited; each is history and stays as written:

| Document | Handling |
|---|---|
| `docs/adr/0038-event-stream-subscription-semantics.md:148` | amendment (c), accepted; an ADR, not edited |
| `docs/scopes/896-deadletters-ref-and-sender.md` | the scope that removes the marker; its text is the criterion S1 |
| `docs/scopes/1475-public-ports.md:25` (AC-2) | describes the state before #896 ("Today the assertion is `PORT-GAP(dl-sender-896)`"); now stale as a status, true as history. An Owner decision whether to annotate it; not done here |
| `migration/stage-1/public-ports-1475/EXECUTION.md`, `RULEBOOK.md` | another unit's package (record of 2026-11-07); kept as history |
| `migration/stage-1/deadletters-ref-and-sender/{manifest.tsv,RUNBOOK.md}` | this package; the manifest row and the RUNBOOK name the marker as the target of 1a and stay as written. The manifest was **not** finalized: the RULEBOOK does not say to, and its rows are accurate (`READ`/`SPEC`/`ORACLE`, slices) |

## 8. Owner decisions (2026-11-10)

- G2: D-1 to D-10 approved as written. D-2 (an absent sender is `DeadLetters()` only in the dead-letter view; `Context.Sender` stays with ADR 0032 D3; revisiting it is another decision). D-4: the ignore reference is not here and AC2 is removed from the issue body (it moved to #1087).
- The ID and path proposal of D-7 moved to slice 1b (RULEBOOK section 3). Proposal in #1517: ID `deadletters-<n>` from its own counter, path `pekko://<system>/deadLetters`; decided by the Owner on 2026-11-10: the path is Pekko's, `pekko://<system>/deadLetters`; the `deadletters-` prefix of the ID is internal to Go and approved.
- The Owner's rule under which QA of #1517 approved with an exit-0 checker and a CURRENT 0/21; filter B (#1518).
- The ADR 0038 D7 editorial amendment (D-10) was made in #1504.

## 9. Not delivered (kept out), with owners

`Watch`/`Identify` on the sink and `ActorSelectionMessage` unwrapping (`PORT-GAP(dl-watch-896)`, D-6; today `Context.Watch(DeadLetters())` panics with `watchError{ErrInvalidHandle}`, Pekko answers with a `Terminated`); AC3 of the issue is therefore partial. The ignore reference: #1087. `SuppressedDeadLetter` and divergence 5a: #871. `Dropped`: #906. `UnhandledMessage`: #897. INFO/WARNING logs: #1479. `Forward`: #854. Migration of the callback users and removal of the hub: #1426. `Context.Sender` substituting `deadLetters`: not decided. The ADR 0011 / EXP-1 justification of the two exports is in the scope "Open API choices" and in the README section.

## 10. Evidence class and parity

READ/SPEC rows (Pekko code and specs read, not run): the manifest. ORACLE: oracle EV8 [006], [008] observes `sender=Actor[pekko://evs/deadLetters]` on Pekko 1.7.1. The Go tests are Go unit evidence; no parity beyond those lines is claimed, and the judge CURRENT is stale (section 6).
