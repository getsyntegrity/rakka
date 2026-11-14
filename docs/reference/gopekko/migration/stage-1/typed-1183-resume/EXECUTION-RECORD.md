# EXECUTION RECORD: Resume of a supervised Typed behavior (#1183, TYP-49)

Scope `docs/scopes/1183-typed-resume.md` (PR #1398, approved at gate G2 by the Owner on 2026-11-06). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`), scope written against `develop` `49a30191`. Delivered in `develop` on 2026-11-06 by the PRs of section 8. Every PR says `Refs #1183`, none `Closes`: #1183 stays open for the Owner's closure decision. This slice (4) is docs only: **no mutation applies**.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`). The cost-log row counts **19 subagent runs**: the orchestrator counted 16 subagent launches and 2 resumed runs (18 runs) through slice 3 from the session, plus the Dev of this slice. The QA reviews of slice 4 itself happen after the row is written and are not in the number. Tokens and wall clock were not measured (`unknown`).
2. Inputs: `Supervision.scala` (56-58, 139-156, 169-178), `SupervisorStrategy.scala` (35, 125), `Behavior.scala` (240-243), `SupervisionSpec.scala` (163, 834, 1107, 502, 201, 208), `ActorContextSpec.scala` (281), the executed oracle `docs/parity/oracles/typed-resume-2026-11-06.md`, ADR 0035 (contract 14, S-5), ADR 0033 D13.
3. Outputs: `Resume()` and `ResumeStrategy` (`supervise.go`), the resume branch of `fail` and the start rule `startDecisionFor` (`interpreter.go`), four test files (`resume_test.go`, `resume_children_test.go`, `resume_start_test.go`, `resume_nested_test.go`), the exported-surface list, the kit package (`manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md`), the oracle, the README section "Resume", this record and one cost-log row.
4. Deviations: S7 (`runtime.Goexit` never resumed) was dropped, because mutation M7 survives (`turn()` does not recover a `Goexit`, so `fail` never sees `ErrGoexit`); the Goexit row of `supervise_test.go` (#976 R4) covers it. The unit ran the kit with its own manifest, RULEBOOK and RUNBOOK (D-1).

## 2. RED evidence

The failing runs are the mutations of the production code (section 7, "Mutations by criterion"): each named test failed by assertion on the PR head, in a disposable worktree, never committed. The per-PR "Mutations" tables and the QA reproductions are in the PR threads. This slice has no test: no mutation applies.

## 3. Gates

Per code or test PR, on the pushed head: `gofmt -l`, `go vet`, `go test -count=1 -covermode=atomic` (module 99.3%), `-count=100` of the new tests, `golangci-lint` v2.14.0 (0 issues), `gocyclo -over 10`, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`. The last one reports 7 errors (`pekko-actor/actor/names.go`, six `pekko-persistence` files) that are identical on `origin/develop`; none is in `pekko-actor-typed`. The full output of each is in the evidence comment bound to the head (section 8). `unshare -rn` is not available on macOS; the unit tests use no network. This slice's evidence is in its own PR comment.

## 4. Size (authored lines, additions plus deletions, `gh pr view <n> --json additions,deletions`)

| PR | Content | Lines | Scope estimate (re-estimated) |
|---|---|---|---|
| #1397 | oracle | 111 | n/a |
| #1398 | scope | 158 | n/a |
| #1399 | slice 0, kit package | 111 | ~140 |
| #1400 | slice 1a | 194 (184 + 10) | ~175 |
| #1401 | slice 1b | 190 | ~125 |
| #1402 | slice 2 | 116 (108 + 8) | ~140 |
| #1403 | slice 3 | 121 | ~100 |
| this PR | slice 4 | see its description | ~120 |

All are under the 400 budget. Slices 1a and 1b measured above their re-estimates (194 against ~175, 190 against ~125).

## 5. Evidence class and pending approval (ADR 0033 D13)

- **ORACLE, executed on Pekko 1.7.1:** R1, R1b, R1c1, R1c2 (with its unsupervised control R1c2ctl), R3, R4a, R4c, R5a to R5e, R6a to R6c: 16 scenarios, 3 runs each. R2 is a check on the R1 run, not a separate scenario. R1c2 is inferred from the effect (the actor answers afterwards, the control stops with `ChildFailed(DeathPactException)`); the death pact is not logged.
- **READ/SPEC, read against the code or spec and not run:** `SupervisionSpec.scala` 163, 834, 1107, 502, 201 and 208, `ActorContextSpec.scala` 281; `Supervision.scala` 56-58, 139-156, 169-178; `Behavior.scala` 240-243. **Pending Owner approval as alternative evidence under D13** where no scenario was executed.
- **No Pekko counterpart:** the Go-only parts (the layer flag, the `ErrGoexit` exclusion of the matcher, the wrapped error returned by the start, the exported-surface list).
- **Not claimed:** parity of the module with Pekko Typed, V3, the message text of the start failure, the delivery of `ChildFailed` to a Typed parent.

## 6. Not delivered (kept out), with owners

Death pact under Resume (R1c2): #1379. Innermost-match tests R5c to R5e, the proof that the innermost layer wins and the title fix of `restart_childcases_test.go:199`: #1184. `UnstashException` under Resume: #927. The ERROR log line (ADR 0035 contract 13): #1179. `watchWith` children: #1171. A Typed child that fails and `ChildFailed` to a Typed parent: #1170, #1346. Backoff: #986. `Stop()` and `Stopped`: #1182. The gaps of Resume are in the README section "Resume".

## 7. QA and integration (taken from the PR threads, `gh pr view <n> --comments`)

- **#1397 (oracle): REJECT, then APPROVE.** CRITICAL: the oracle's R1c2 claimed more than the log showed (every `A.get` in the raw logs came before the child's `PostStop`, so none ran after the child ended). WARNING: "same events in the same order" was false for R1c2 and R3. Fixed by a rerun with a settle and an unsupervised control (R1c2ctl), and the text now names the orders that vary. Second review: APPROVE on `54b2753`.
- **#1398 (scope): APPROVE with three warnings and two suggestions**, then APPROVE on `035314e`. S7 was dropped because mutation M7 survives; S4 got M4; the start sentence says "wraps"; the prototype size says 37 lines (34 needed).
- **#1399 (slice 0): REJECT, then APPROVE.** F1: a mislabelled Pekko citation in the manifest: `SupervisionSpec.scala:208` is "support flatten supervise", and "support nesting supervise" is line 201. Fixed on `7eef7a8`, which cites both.
- **#1400 (slice 1a): REJECT, then APPROVE.** The evidence comment was missing (the body said "run locally"). Fixed with the full gate comment on a new head `ec9ceda`, which also drops the `Ported from` line the reviewer flagged as against RULEBOOK section 6. The reviewer reproduced M1 to M5 and added two mutations of its own. Correction (#1184 slice 2): the delta review of head `ec9ceda` reproduced M1, M2, M3 and M5 only, and did not re-run M4; M1 to M5 were reproduced by the first review (head `af2cf1e`).
- **#1401 (slice 1b): APPROVE** on `ec5c2f0`. WARNING: M6b fails only after the 30 s helper timeout, because the missing `Terminated` has no other observable and the test policy forbids a timer-based absence check.
- **#1402 (slice 2):** returned to the Dev **before review** by the orchestrator's pre-QA check, because the evidence comment had an empty `exit=` for the two mutations (the comment attributes it to a capture quirk; the test run itself printed FAIL). The QA then approved head `f5f0305`.
- **#1403 (slice 3): APPROVE** on `3b0ae38` with a WARNING: the rows R5a and R5b cannot show that the innermost layer wins. With `supervisorFor` picking the outermost matching layer, both rows still pass, because the classes are disjoint (as in the oracle). Moved to #1184 and stated in the README.
- **Mutations by criterion:** M1 (S1), M2 (S2), M3 (S3), M4 (S4) and M5 (a nil behavior in `OnFailure`) by `TestResume_KeepsStateAndDropsMessage`, `TestResume_UnmatchedStops`, `TestUnhandled_U9_PublicSurface` and `TestResume_OnFailureNilArguments` (#1400); M5 (a failed signal turn is asked to the layers, S5), M6 (the resume branch stops the children, S6) and M6b (it unwatches them; fails only after the 30 s helper timeout) by `TestResume_SignalFailure` and `TestResume_KeepsChildrenAndWatches` (#1401); M8 (a failed start is stopped, not reported to the parent) and M8b (the outer Restart retries the start) by `TestResume_FailedStart` (#1402); M9 (a restart drops the rebuilt inner layers) by `TestResume_NestedWithRestart` (#1403). M7 survives and S7 was dropped.
- **The throwaway prototype** (37 lines, of which a redundant `restart_limit.go` change was dropped, so 34 were needed) set the size estimate; it is not the implementation. Per the brief of this slice, its `startResumeFor` double-counted for the restart limit. Slice 2 walks the chain once in `startDecisionFor`; QA of #1402 found no double count.
- **Owner decisions of 2026-11-06:** D-1 kit run required; D-2 `Resume() ResumeStrategy`; D-3 #1183 to `status:ready`; D-4 the death pact under Resume (R1c2) stays in #1379, with a README known gap; D-5 R5c to R5e and the title fix stay in #1184; adjustment 1: `UnstashException` under Resume is owned by #927; adjustment 2: the cause reported for a failed start (an initialization failure wrapping `ErrInvalidInitialBehavior`).

## 8. PRs, squash merges and evidence

| PR | What | Merge commit |
|---|---|---|
| #1397 | oracle | `80d8a64d` |
| #1398 | scope | `fa5b4bfb` |
| #1399 | slice 0 | `175b17df` |
| #1400 | slice 1a | `b63c7da6` |
| #1401 | slice 1b | `82ffb933` |
| #1402 | slice 2 | `31d1af7f` |
| #1403 | slice 3 | `afa76baf` |
| this PR | slice 4: README, record, cost-log row | see `git log` |

The evidence comment of each code or test PR is bound to the head named in section 7 (`gh pr view <n> --comments`).

## 9. Criteria against the scope

| Criterion | Result |
|---|---|
| S1, S2, S3, S4 | delivered (#1400) |
| S5, S6 | delivered (#1401) |
| S7 | dropped (M7 survives); covered by the Goexit row of `supervise_test.go` |
| S8 | delivered (#1402) |
| S9 | delivered (#1403), with the limit that the rows do not pin the innermost rule (#1184) |
| S10 | this PR (README) |

## 10. Recurring failures (`AGENTS.md`, kit rule 6)

Incomplete evidence appeared twice (#1400 without a comment, #1402 with empty `exit=` values), and a mislabelled citation once (#1399). None reaches three occurrences, so no rule amendment is proposed yet.
