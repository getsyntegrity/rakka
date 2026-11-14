# EXECUTION RECORD: The innermost matching Typed supervisor decides (#1184, TYP-50)

Scope `docs/scopes/1184-innermost-supervision.md` (PR #1406, approved at gate G2 by the Owner on 2026-11-06). Kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`, Pekko v1.7.1 (`9a54d12`), scope written against `develop` `e114ea95`. Delivered in `develop` on 2026-11-06 by the PRs of section 8. PRs #1409, #1410 and #1411 say `Refs #1184`; #1405, #1406 and #1408 mention #1184 without `Refs`; #1407, #1412 and #1413 do not use `Closes`. #1184 was closed as completed on 2026-11-06, after the merge of this slice, under the Owner's delegation (closure Update in the issue). This slice (2) is docs only: **no mutation applies**.

## 1. Run

1. Model: Claude Sonnet 5.5 (`claude-sonnet-5-5`). The cost-log row counts **19 subagent runs**, the orchestrator's count of the subagent launches of this unit through this slice: 18 before it (the reviews and delta reviews of #1405 to #1412, including the two reviews of the rule PR #1407 and the review of the oracle addendum #1412, and the Devs of slices 0, 1a and 1b) plus the Dev of this slice (the first count, 18, left out the review of #1412 and was corrected). The QA review of slice 2 itself happens after the row is written and is not in the number. Tokens and wall clock were not measured (`unknown`).
2. Inputs: `InterceptorImpl.scala` (82-85, 90-98, 101-103, 106-122), `Supervision.scala` (75-83, 294-306, 335-337), `Behavior.scala` (76-86, 218-232), `SupervisionSpec.scala` (201-211, 849, 1270, 1637, 1644), the executed oracles `docs/parity/oracles/typed-nested-supervision-2026-11-06.md` (N1 to N11) and `typed-resume-2026-11-06.md` (R5a to R5e), ADR 0035 and its Amendment (2026-11-06), ADR 0033 D13.
3. Outputs: `dedupeLayers` and its three call sites (`typed/interpreter.go`), the tests `nested_replacement_test.go` and `nested_innermost_test.go`, two adapted #976 tests, the kit package (`manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md`), the README section "Nesting", this record and one cost-log row.
4. Deviations: the first throwaway prototype run of the TL did not apply because of a wrong `git apply` path and was repeated (reported by the orchestrator; no PR thread records it, so it is not independently checked here). The scope's Mandatory Stops were not reached. The unit ran the kit with its own manifest, RULEBOOK and RUNBOOK (D-1).

## 2. RED evidence

The failing runs are the mutations of the production code (section 7, "Mutations"): each named test failed by assertion on the PR head, in a disposable worktree, never committed. Before the change, with the throwaway prototype of the production change and the two #976 tests unmodified, exactly those two tests failed (`TestRestartStopChildren_Nested` case 3 and `TestRestartLimit` "a rethrown failure reaches the enclosing supervisor"); the QA of #1406 reproduced that. This slice has no test: no mutation applies.

## 3. Gates

Per code or test PR, on the pushed head: `gofmt -l`, `go vet`, `go test -count=1 -covermode=atomic` (module 99.3%), `-count=100` of the new tests, `golangci-lint` v2.14.0 (0 issues), `gocyclo -over 10`, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`. The last one reports 7 errors (`pekko-actor/actor/names.go`, six `pekko-persistence` files) that are identical on `origin/develop`; none is in `pekko-actor-typed`. Diff coverage of #1410: 100% of 8 lines. `unshare -rn` is not available on macOS; the unit tests use no network. The full output of each is in the evidence comment bound to each head (section 8). This slice's evidence is in its own PR comment.

## 4. Size (authored lines, additions plus deletions, `gh pr view <n> --json additions,deletions`)

| PR | Content | Lines | Scope estimate (re-estimated) |
|---|---|---|---|
| #1405 | oracle | 97 | n/a |
| #1406 | scope | 145 | n/a |
| #1408 | ADR 0035 amendment | 25 (23 + 2) | n/a |
| #1409 | slice 0, kit package | 153 | ~111 |
| #1410 | slice 1a | 264 (256 + 8) | ~265 |
| #1411 | slice 1b | 165 (164 + 1) | ~170 |
| #1407 | rule PR (claims against evidence) | 5 (4 + 1) | n/a |
| #1412 | oracle addendum N11 | 6 (4 + 2) | n/a |
| this PR | slice 2 | see its description | ~110 |

All are under the 400 budget. Slice 0 measured above its estimate (153 against ~111); 1a and 1b matched.

## 5. Evidence class and pending approval (ADR 0033 D13)

- **ORACLE, executed on Pekko 1.7.1:** N1 to N11 (the oracle holds 16 scenarios in all; N11 was run later, on 2026-11-06, and merged by #1412) and R5c, R5d, R5e of the Resume oracle, 3 runs each, identical events in the three runs. In every row the deduplication of supervisors is **inferred** from effects: no log line shows the stack (oracle section 4).
- **READ/SPEC, read against the code or spec and not run:** `InterceptorImpl.scala` 82-85, 90-98, 101-103, 106-122; `Supervision.scala` 75-83, 294-306, 335-337; `Behavior.scala` 76-86, 218-232; `SupervisionSpec.scala` 201-211, 849, 1270, 1637, 1644. **Pending Owner approval as alternative evidence under D13** where no scenario was executed. That an enclosing supervisor of a narrower class than the inner one, and more than two layers, behave as read while the start is failing: read from `Supervision.scala` 294-306 and 335-337, no scenario ran them.
- **Go-only parts, no Pekko counterpart:** `dedupeLayers` is a pass over a list of layers (outermost first) through `joinLayers`, not a walk of an interceptor stack; the second row of `TestNested_ReturnedSuperviseReplacesEqual` was inferred from N3 and N1 when written (the QA of #1410 recommended an oracle scenario) and is now the shape of N11.
- **Not claimed:** parity of the module with Pekko Typed, V3, a flattened `.onFailure().onFailure()` form (D-3), `Stop()` variants, `ChildFailed` to a watching parent.

## 6. Not delivered (kept out), with owners

Explicit `Stop()` variants of the rows: #1182. `ChildFailed(cause)` asserted for a watching parent in the stop rows: #1346. Adapter-function failures through nested supervisors (`SupervisionSpec.scala:1637`, `:1644`): #975. Nesting with backoff: #986. A flattened form: Owner D-3 (no API). The rethrow of an exhausted `withLimit` through a match-all layer to a Classic parent: delivered by #976 (`TestRethrowParentKinds`). The gaps are in the README section "Nesting".

## 7. QA and integration (taken from the PR threads, `gh pr view <n> --comments`)

- **#1405 (oracle): APPROVE, then REJECT on a delta, then APPROVE twice.** The first review approved `cf96e90` with four WARNINGs (a vacuous path-hash exception; inferences written as observations; a mislabelled spec range; C1 overstated). The delta `8bf0525` was rejected: CRITICAL, three sentences stated a mechanism as observed with no log line behind it (N6d "the outer Restart took the rethrow", N3 and N4 in the summary, N7d and N2 "rethrown"), against the Owner's rule of 2026-11-06. Fixed by splitting the tables into an Observed column that cites `[nnn]` log lines and an Inferred column (APPROVE on `6e79240`). That review found C5 (the deduplication branch) stated backwards: the enclosing interceptor drops itself when an equal one exists in the started stack, not the reverse; fixed, APPROVE on `ba2491e`.
- **#1407 (the rule PR, claims against evidence in dev-scope and qa-review): APPROVE twice.** First review on `9f19946` (two WARNINGs on the definition of "parity criterion" and on pre-existing claims), delta review on `7196d7d` (APPROVE; the dev-scope definition agrees with the qa-review one). The rule was born in this unit: a sentence that says something is observed names the log line or the assertion, and an unbacked parity claim is CRITICAL.
- **#1406 (scope): APPROVE with four warnings, then APPROVE on a delta with two warnings, then APPROVE.** Warnings: only the start site had a named test and mutation; `SS:201-211` and `SS:849` do not back the overlap rows; DoD said "no test changes" though titles and rows change; the docs estimate did not say the 1.55 factor was not applied; the N6c "row" could not be added to the loop of `TestRestartStopChildren_Nested` without changing its assertion code (it became its own test); "measured sizes 144, 111, 120" counted an estimate (120) as measured.
- **#1408 (ADR 0035 amendment, D-5): APPROVE, then a delta APPROVE.** WARNINGs: "four runs" for four `Setup` executions; "is caught by the enclosing one, whatever their classes" could be read as regardless of the thrown class. Fixed: the decision text now says "if the class of the enclosing supervisor matches the exception", and the Evidence paragraph says that the narrower-class case and more than two layers are read from the code, no scenario ran them.
- **#1409 (slice 0): APPROVE** on `bb90db8`. WARNING: the manifest note of `Supervision.scala:294-306` said "a failing start is rethrown outward", but `handleExceptionOnStart` rethrows only when the restart is unlimited or `restartCount + 1 >= maxRestarts`, and otherwise retries through `prepareRestart`. Corrected by this slice (TL instruction, outside the slice table). SUGGESTION: the RUNBOOK could mention `git submodule update --init --reference`; added by this slice.
- **#1410 (slice 1a): APPROVE** on `daac8a7`. The D-2 conditions were verified: every assert/expect line of the two adapted tests is byte-identical, only the matcher of one layer changes. WARNING: row 1 of `TestNested_ReturnedSuperviseReplacesEqual` (N3, a returned `Resume`) passes under M2 and M2b because the innermost layer decides anyway; row 2 observes the replacement and was inferred, not an oracle run. That led to the oracle addendum #1412 (N11).
- **#1411 (slice 1b): APPROVE** on `8006abc`. WARNINGs: the PR mutation table named no mutation for R5c (a killing mutation existed, and the QA reproduced it); N7d does not assert the oracle's third failure (`setup#4`). The R5c killing mutation was added to the PR table.
- **#1412 (oracle addendum N11): APPROVE** on `44f4f91`. WARNING: the Purpose line still listed N3, N4, N9 without N11.
- **Mutations** (all never committed, each reproduced by the QA of its PR): M1a (`PreStart` does not dedupe) by `TestNested_EqualMatcherReplacesAfterStart` (N1, N7) and `TestNested_EqualMatcherInnerRethrowStops` (N6c); M1b (the turn site does not dedupe) by `TestNested_ReturnedStackDedupes`; M1c (the restart site does not dedupe) by `TestNested_RebuiltStackDedupes`; M2 (`joinLayers` appends without replacing) by `TestNested_ReturnedSuperviseReplacesEqual` row 2 and the T1 tests; M2b (the turn site appends without replacing; isolates T2) by row 2 only; M3 (a returned layer replaces every enclosing layer) by N4, R5d, N2, N7d; M4 (`supervisorFor` picks the outermost matching layer) by N4, R5d, R5e, N2; M5 (an exhausted limit stops the actor instead of searching outward) by N2 and N7d; M6 (dedupe while collecting the layers in `startInto`) by `TestRestartLimit_InitialStart` (`expected 1 to equal 4`). The R5c killing mutation (the QA of #1411 called it M6 in that thread, not the M6 of #1410): `joinLayers` keeps the enclosing layer on an equal pair and drops the inner one; it fails R5c and N1, N7, N6c. M1a and M4 do not fail R5c.
- **The defect the oracle found:** goPekko did not deduplicate equal-matcher layers after a successful start. `joinLayers` removed from the layers in force those that an added layer equals, but never an equal layer among the layers just started, and the initial start did not dedupe at all; with an inner `Restart.WithLimit(1)` and an outer layer of the same matcher Pekko stops at the limit (N1, N7, N6c) and goPekko let the outer layer take the rethrow. The fix has three sites, each necessary alone and seen by exactly one row: without the `PreStart` site N1 keeps running, without the turn site N9 keeps running, without the restart site N10 reaches only `setup#3`. Dedupe while collecting the layers (M6) breaks the failing-start case (N8).
- **The throwaway prototype** (15 lines of production, 12 added and 3 modified, a disposable worktree, never committed) set the size estimate and the scope's table; it is not the implementation. QA of #1406 reproduced it: N1 and N9 stop, N10 reaches `setup#4`, N8 keeps 4 runs, and exactly two existing tests fail. The first run did not apply (a wrong `git apply` path, see section 1) and was repeated.
- **The two #976 tests adapted under D-2** with their assertions unchanged: `TestRestartStopChildren_Nested` case 3 (N6d: the inner layer now has `ErrorIs`, the outer `AnyError()`) and `TestRestartLimit` "a rethrown failure reaches the enclosing supervisor" (N7d). The equal-matcher forms that do what Pekko does are `TestNested_EqualMatcherInnerRethrowStops` (N6c) and `TestNested_EqualMatcherReplacesAfterStart` (N7).
- **Owner decisions of 2026-11-06:** D-1 kit run required (production code); D-2 the two #976 tests adapted, with the conditions that N6d/N7d are kept with their assertions unchanged, the equal-matcher form is added and each change is explained with its oracle scenario; D-3 no flattened form (`OnFailure` returns a `Behavior`); D-4 no explicit `Stop` yet (#1182); D-5 the ADR 0035 amendment in its own PR, merged before slice 1a; D-6 #1184 to `status:ready`.

## 8. PRs, squash merges and evidence

| PR | What | Merge commit |
|---|---|---|
| #1405 | oracle | `f54d79d1` |
| #1406 | scope | `b647bd61` |
| #1407 | rule: claims against evidence (born in this unit) | `e4c5dc22` |
| #1408 | ADR 0035 amendment (precondition of slice 1a) | `4b152df6` |
| #1409 | slice 0 | `e2d35a78` |
| #1410 | slice 1a | `d16c5842` |
| #1411 | slice 1b | `381e8a8e` |
| #1412 | oracle addendum N11 | `e35f2be4` |
| this PR | slice 2: README, record, cost-log row, carried fixes | see `git log` |

The evidence comment of each code or test PR is bound to the head named in section 7 (`gh pr view <n> --comments`).

## 9. Criteria against the scope

| Criterion | Result |
|---|---|
| T1a, T1b, T1c, T2, T3, T7 | delivered (#1410) |
| T4, T5, T6, T8 | delivered (#1411) |
| T9 | this PR (README, record, the three carried wording fixes) |

## 10. Recurring failures (`AGENTS.md`, kit rule 6)

The class **"a claim outran its evidence"** occurred again in this unit. Instances in the threads: #1405 (mechanisms stated as observed, CRITICAL; C5 stated backwards; C1 overstated); #1406 ("measured sizes" that included an estimate; spec lines cited for rows they do not back); #1408 ("four runs" for four executions; "whatever their classes"); #1409 (the manifest note "rethrown outward"); #1404 (carried into this slice: "three executed oracles", "Resume stops the actor on a death pact" with no test behind it). Related, an omission and not an overclaim: #1411 named no mutation for R5c.

The class already has its rule (#1407, merged `e4c5dc22` at 18:16Z; the REJECT of #1405 cites the same Owner rule of 2026-11-06, which #1407 puts into the skills). It **recurred after the rule was merged**: #1405 (18:27Z), #1406 (18:29Z), #1408 (18:32Z), #1409 (18:42Z) and #1411 (19:15Z) all merged after it, and #1408, #1409 and #1411 were reviewed against it and drew WARNINGs of the same class, none CRITICAL. The merge times do not show when each review ran relative to the merge of the rule, so the count of reviews that applied the merged text is not known. The QA caught each instance before merge, so the rule works as a gate but does not yet prevent the writing of the claim. No new amendment is proposed here; the Owner may decide whether it calls for one.
