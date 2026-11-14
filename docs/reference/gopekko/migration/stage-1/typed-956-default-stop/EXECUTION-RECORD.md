# EXECUTION RECORD: Typed default Stop (#956, TYP-09)

Scope `docs/scopes/956-typed-default-stop.md` (PR #1233; G2 `9c716c3`, deltas D-7 and D-8 re-confirmed at `ea63958`). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`), base `develop` `8ebd25a`. Four draft PRs, all `Refs #956`: PR 1 #1240 (kit package, T1), PR 2a #1241 (T2, T4), PR 2b #1242 (T3, T5, stacked on 2a), PR 3 (this record, T6). G2 said three PRs; D-8 made it four because PR 2 projected 528 authored lines.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`), one AO Dev session, no subagents. Tokens and wall clock were not measured (`unknown` in the cost-log row).
2. Inputs: `SupervisionSpec.scala:387,492,502`, `ClassicSupervisingTypedSpec.scala:74-160`, `TypedSupervisingClassicSpec.scala:61-71`, `ActorAdapter.scala:125-141,168-172`, read, not run. Dispositions in `manifest.tsv` (12 rows).
3. Outputs: `interpreter.go` (`Receive` stops the actor on a failed turn, `turn` recovers panics), `default_stop_test.go` (S1-S8), `typed_test.go` (S9/S10 rewritten), `signal_runtime_test.go` (S6/S10 on an AllForOne trigger), `pekko-actor-typed/README.md` ("Default Stop").
4. Manual changes and deviations: (a) tests use plain `testing`, not go-specs (`go.mod` is outside the allowed files; Owner option 3, module-wide migration is a later PR); (b) `default_stop_test.go` is `package typed` because S5 needs the wrapped runtime context; (c) the old `restartHarness` of `signal_runtime_test.go` was replaced by an AllForOne parent (D-7); (d) one staticcheck ST1023 finding in S8 fixed before PR 2b was pushed.

## 2. RED evidence (observed locally, before the interpreter change)

Run on the unchanged interpreter of `develop` `8ebd25a`. A hang is the failure mode: the Typed actor was restarted, so it never terminated; the 30 s timer of the helper turns the hang into a failure. Line numbers are those of the local draft. Not reproducible from the pushed branches.

| Scenario | Observed before the change | After |
|---|---|---|
| S1 error | FAIL, `timed out waiting for the actor` (30 s) | pass |
| S2 panic | FAIL, same | pass |
| S3 nil | no own result: the 90 s package timeout expired inside this subtest | pass |
| S4 | FAIL, 30 s timeout | pass |
| S5 | FAIL, 30 s timeout | pass |
| S6 | FAIL, 30 s timeout (the Typed sibling was restarted) | pass |
| S7 | **PASS** before and after (negative guard, not a red-to-green) | pass |
| S8 | FAIL, 30 s timeout | pass |

After the change three existing tests hung: `TestS9S10_FailedTurnInstallsNothing` (rewritten, D-4), `TestSig_S6_...` and `TestSig_S10_...` (rewritten, D-7). The spike that found the AllForOne trigger was the TL's. No mutation check was run.

## 3. Gates (local; remote CI is blocked by GitHub billing and attributes nothing)

Full output is in the gate comments of each PR: #1241 `1fc46bf`, #1242 `641efaf`. macOS arm64, go1.27.2, golangci-lint v2.14.0, no `unshare` (no network isolation). Both: `go test` ok, `go vet`, `gofmt -l`, golangci-lint (gocyclo <= 10), `go mod tidy -diff`, `scripts/check_deps.sh`, `go-sdk-update.sh --check`: exit 0; module coverage 100.0%. Diff coverage: PR 2a `interpreter.go` 100%; PR 2b no covered lines (test-only). `check-headers.sh`: exit 1 on 7 pre-existing files outside the diff, none under `pekko-actor-typed`.

## 4. Size (authored lines, additions plus deletions)

PR 1: 65. PR 2a: 383. PR 2b: 163 (versus 2a). PR 3: this record plus one cost-log row. The combined PR 2 was 528.

## 5. Evidence class and pending approval

Go unit evidence derived from specs, **not V3**, no Pekko run. SPEC rows: S1-S3, S6, S8. **READ rows, pending Owner approval (ADR 0033 D13): S4, S5, S7.** No parity is claimed.

## 6. Not delivered (kept out)

`Resume` and the "previous behavior survives" half of the old #847 S10: #1183 (D-4). `supervise`/`OnFailure`: #1184. Typed parent, `ChildFailed` (M-A): a separate story, now #1250 (it had no issue number when this record was first written). Failure logging (contract 13): pending (D-3), proposed #1179. Start failure reported to the parent: gap, `TODO(TYP-09-start)`. Restart, limits, backoff, stash, Slice B and B-2 of #954.

## 7. QA and integration

Independent QA (not the TL, not the Dev) issued APPROVE on the tested candidates of #1240 to #1243; section 9 lists the verdicts. #1233 (scope) has no QA verdict: its approvals are the Owner's G2 comments of 13:54Z and 14:21Z. A QA verdict is not a merge approval. The merges were squash merges made by the orchestrator under the Owner's authorization with `--match-head-commit`; the source is the QA post-merge comment on #1242 (16:44Z), not independently confirmed per PR. #956 keeps `status:blocked` and is open; #954 stays open. The pre-squash SHAs above are the tested candidates; they are not part of the history of `develop`, which holds only the squash commits. Section 8 maps one to the other.

## 8. Candidate versus integrated commit

Verified on 2026-10-28 with `gh pr view` (head and squash commit of each PR) and, for each squash commit, `git diff --numstat <head> <squash> -- <files the squash changed>`: no file differs (0-line diff) between the last pushed head and the squash commit. The original SHAs in sections 1 to 7 are kept as written. A tested candidate is not an ancestor of `develop`; the squash commit is the integrated one.

| PR | Tested candidate (last pushed head) | Other SHAs cited above | Integrated squash commit |
|---|---|---|---|
| #1233 scope | `ea63958` | `9c716c3` (G2) | `f5a511a` |
| #1240 PR 1, kit package | `0c9b599` | | `35df8f3` |
| #1241 PR 2a | `1fc46bf` | | `4248f48` |
| #1242 PR 2b | `05d789b` | `641efaf` (head the gates of section 3 name; not an ancestor of `05d789b`, so a rebase separates them; the 0-line check above was made on `05d789b`, not on `641efaf`) | `cb04968` |
| #1243 PR 3, cost-log | `f07b01e` | | `0918589` |

The Pekko line ranges cited in section 1 item 2 were corrected in `manifest.tsv` and `RULEBOOK.md` by PR #1244; this record keeps them as run. #956 and #954 stay open. No parity and no remote CI result is claimed.

## 9. Verification by head: executed, inherited, not executed

Added after integration. It does not replace the SHAs of sections 1 to 8. Sources: the gate and QA comments of #1240 to #1243 (UTC, 2026-10-28). "Executed" means a run on that exact head by the named actor; "inherited" means the result of an earlier head is reused because the checked content is identical.

| Head | Executed | Inherited by equivalence | Not executed |
|---|---|---|---|
| `0c9b599` (#1240, kit package) | QA APPROVE 14:00Z: scope diff (3 files, 65 lines), manifest column count | none | no code to run |
| `1fc46bf` (#1241, PR 2a) | Dev gates (section 3); QA APPROVE 14:31Z in a fresh worktree; QA addendum 14:34Z (one WARNING, `ErrNilBehavior` comment) | none | see below |
| `641efaf` (#1242, first head) | Dev gates (section 3); QA APPROVE 14:31Z | none | verdict obsolete after the rebase |
| `05d789b` (#1242, tested candidate) | Dev re-run of the gates 14:45Z; QA re-review APPROVE 14:55Z, only the checks the changed base affects | lint, `check_deps.sh`, headers and `go-sdk-update.sh --check` of the `641efaf` QA run; `README.md` blob equal on both heads (`7b20bf72`) | none beyond the common list |
| `f07b01e` (#1243, PR 3) | QA APPROVE 14:32Z: diff and file count, cost-log column count, read of the record (docs-only head, no code gates) | gate results and the QA mutation run (S1-S6, S8, rewritten S9/S10 fail; S7 passes) come from QA runs on `1fc46bf` and `641efaf`, not on this head | none beyond the common list |
| `cb04968` (`develop`, squash of #1242) | QA post-merge 16:44Z, own runs in a fresh worktree on `cb04968`: `go test -count=1 -cover ./...` in `pekko-actor-typed` (ok, 100.0%), `go test -count=3 ./typed/` (ok), `go vet`, `gofmt -l`, `go mod tidy -diff` (all 0); S8 present; `README.md` and `default_stop_test.go` blobs equal to `05d789b` | lint (gocyclo), `check_deps.sh`, headers and `go-sdk-update.sh --check`: not repeated on this head, results of the approved head | none beyond the common list |

Squash commits `4248f48`, `35df8f3` and `0918589` are tied to their candidates by the 0-line diff of section 8, not by a new run.

**Never executed, for any head:** a Pekko run of any scenario (specs are read, not run; not V3); `-race`; network isolation (`unshare`, none on macOS); remote CI (blocked by GitHub billing, attributes nothing); a mutation check by the Dev (QA ran one). `check-headers.sh` exits 1 on 7 files outside this diff, none under `pekko-actor-typed`; the Dev's `05d789b` gate comment pipes it through `tail`, which masks the exit code.

## 10. Criteria against the approved M-B scope

Scope: `docs/scopes/956-typed-default-stop.md`, mechanism M-B only (decision D-2), deltas D-7 and D-8. Evidence class for every row: Go unit evidence derived from Pekko specs, read and not run.

**Satisfied** (tests pass at the candidates of section 8; QA verdicts in section 9):
- Criterion 1: a failed turn (S1 error, S2 panic, S3 nil behavior) stops the actor, the parent decider is not consulted, the watcher sees one `Terminated`, no restart. Rows `SupervisionSpec.scala:387, 492, 502` and `ClassicSupervisingTypedSpec.scala:74, 84, 105, 130, 142, 152` (as corrected by #1244).
- Criterion 1 (`PostStop`): S8, mandatory by D-1 (a), once and before `Terminated`.
- Criterion 3: S6, the Classic `DefaultStrategy` is unchanged (`TypedSupervisingClassicSpec.scala:61-71`).
- Start failures unchanged (existing #847 S5/S6).
- D-7: S6 and S10 of Slice A of #954 kept every original assertion on the AllForOne trigger. D-4: the "turn fails, actor stops" half of #847 S9/S10.
- Criterion 4, recorded part: candidate SHAs and gates (sections 3, 8, 9). The ordering boundary it names is S4, a READ row, and is pending below. Criterion 5: `pekko-actor-typed/README.md` ("Default Stop").

**Delivered but pending Owner approval (ADR 0033 D13):** READ rows S4 (queued mail), S5 (children) and S7 (`runtime.Goexit`). The Pekko citations QA flagged on them were corrected by #1244. Parity is not claimed.

**Pending (not satisfied):**
- Failure logging at ERROR (ADR 0035 contract 13): `PORT-GAP`, decision D-3, proposed destination #1179, not scheduled.
- Start failure reported to the parent: gap, `TODO(TYP-09-start)`.
- `ErrNilBehavior` doc comment (`pekko-actor-typed/typed/behavior.go:21`) still says "a start or a turn"; since #1241 a nil turn stops the actor and records no cause. #1246 added the NullPointerException source and left that wording. `behavior.go` was outside the allowed files, so it is a separate change.
- A green remote CI result: none exists.

**Excluded from #956 (not required, not claimed):** explicit `Resume` and the previous-behavior obligation of D-4 (#1183); `supervise`, `OnFailure` and most specific strategy (#1184); a Typed parent and `ChildFailed` (M-A), tracked by #1250; restart, limits, backoff, stash (#976, #997, #986, #999); Slice B and B-2 of #954; `Stopped`/`StoppedWith` (#1182); `Unhandled` (#925, #1181); the Typed guardian (#994); Classic-Typed adapters (ADR 0033 D12); decision S-2.

## 11. Proposed closure conclusion (a proposal, not a decision)

Two statements are kept apart.

**M-B behavior delivered.** The Typed default Stop for a failed turn (error, panic, nil behavior), with mandatory `PostStop` and an unchanged Classic default, is integrated as Go unit evidence derived from Pekko specs. Nothing excluded (section 10) is required for it.

**Definition of Done not yet satisfied.** The scope's "Evidence and validation" asks for unit tests with network isolation and green gates. The record shows: (a) no network isolation on any head (macOS has no `unshare`; the container variant of `AGENTS.md` was not run); (b) `.github/scripts/check-headers.sh` exits 1 on 7 files outside this module, so that gate is not green repository-wide although none is under `pekko-actor-typed`; (c) the READ rows and failure logging of section 10. A green remote CI result is not required: its dispensation is recorded at G2 (billing block). Closing #956 on the M-B scope is therefore a proposal that depends on the Owner either having the missing validation run or accepting (a) and (b) as exceptions.

No Typed parity, V3 or CI claim. This record changes no contract, issue, label or code.

Owner decisions still open:
1. Run the missing validation (network-isolated unit tests via the container variant, and the header check scoped to `pekko-actor-typed`), or accept (a) and (b) above as recorded exceptions.
2. Close #956 on the M-B scope above, or keep it open until the READ rows or failure logging are settled. Closing does not remove `status:blocked`; removing the label is a separate Owner decision.
3. Edit the body of #956 to the restated criterion 1 and drop its criterion 2 (scope, "ADR impact", item 1), or leave the issue as written.
4. ~~Create the M-A / `ChildFailed` story, so that its traceability has an issue number.~~ Resolved after this record was written: the story is #1250 (no new decision is needed).
5. Authorize the one-comment fix of `behavior.go:21` as its own change.
6. Approve or reject the READ rows S4, S5 and S7 under ADR 0033 D13.
