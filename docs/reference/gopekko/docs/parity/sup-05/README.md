# SUP-05 (#779): traceability of the zero, finite and unlimited restart-budget values

Story: [#779](https://github.com/getsyntegrity/gopekko/issues/779) (`SUP-05`, inventory row `ACTINV-038`; the bodies of the issues cite `ACTINV-041`, an identifier that `INV-A-actor.md` does not contain). Baseline: goPekko `develop` `9a7076fa6e612d7f35454fa66a164142141c6eed`; its Go code is byte-identical to `108880b0100c11ae838a772c23af952e5a54b827`, the candidate of the sibling story [SUP-06](../sup-06/README.md) (#825). Pekko pin: v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`upstream/pekko`, read-only). Contract: [ADR 0017](../../adr/0017-supervision.md), section S4 and decision A4.

**This story verifies an implementation and a contract that already exist.** The `MaxRetries`/`Within` matrix was accepted by the Owner in ADR 0017 on 2026-10-14 and the budget code has been in `develop` since the supervision units U2 and U6. This change adds this document, a documentation-only amendment to ADR 0017 S4 (the alternatives for the zero `Budget{}`, which the ADR did not record), one clarification in `pekko-actor/README.md` and a one-line correction in the SUP-06 document. It changes no production code, no test, no A4 row, no signature and no accepted behavior, creates no new ADR and introduces no policy. It does not close or modify #779 and claims no parity beyond the evidence classes below.

## The decisions that shape this change

The Owner's decisions on #779 reached this session as a message of the Owner in the working session on 2026-10-28; they are **not** a GitHub comment. The words, as received (English):

> **D-1 APPROVED.** ADR 0017 S4/A4 is the accepted and reviewable contract for SUP-05. Do not create a new ADR or reopen the accepted behavior.
>
> **D-2 APPROVED.** Add a minimal documentation amendment to ADR 0017 S4 covering the alternatives for zero-value `Budget{}` and the existing chosen behavior. Preserve the implementation and accepted A4 matrix. Distinguish `DefaultStrategy` from the literal zero-value Budget. Do not introduce a new policy.
>
> **D-3 APPROVED.** Prepare one documentation-only draft PR containing: `docs/parity/sup-05/README.md` with AC1-AC5 evidence; the minimal ADR 0017 clarification; a concise clarification in `pekko-actor/README.md`; the corrected inventory-row reference in the existing SUP-06 documentation: `ACTINV-038`, not `ACTINV-041`.

History of the message. The first message approved D-1 to D-4 but arrived cut off inside D-3, so the wording of D-4 was not received and the first version of this pull request declared no waiver. On 2026-10-28 the Owner's completing message arrived (again relayed by the orchestrator session `gopekko-1`, not a GitHub comment). It leaves D-1 and D-2 unchanged. The rest of D-3 and D-4, as received (English):

> **D-3 APPROVED (remaining instructions).** The documentation-only PR must include: `docs/parity/sup-05/README.md` with AC1-AC5 traceability; the minimal clarification in ADR 0017 S4; the distinction between `DefaultStrategy` and literal `Budget{}` in `pekko-actor/README.md`; the correction from `ACTINV-041` to `ACTINV-038` in SUP-06 documentation; the existing candidate SHA, upstream Pekko source mappings, test results and applicable local gates. PR #1230 already exists. Reuse it; do not create another PR.
>
> **D-4 APPROVED: PR-specific validation exception.** For PR #1230 only, the Owner explicitly accepts:
>
> 1. Reusing valid SUP-05/SUP-06 evidence on identical Go code.
> 2. The reported focused tests and local `make ci-fast` results.
> 3. Not rerunning `golangci-lint` or `gocyclo` because this PR changes documentation only.
> 4. Proceeding without successful remote GitHub Actions checks because billing restrictions prevented the `go` and `label` jobs from starting.
>
> This is an exception exclusively for PR #1230. Document all checks that were not executed. Never describe unavailable CI as passing. Do not rerun tests or push empty commits to retrigger GitHub Actions.

**What the exception covers, exactly.** It applies to pull request #1230 and to nothing else: no other pull request, no other story, no later change to these files. It accepts four things (the numbered points above) and waives no other merge requirement. It does not turn any unexecuted check into an executed one, and the remote checks are not described as passing anywhere in this document or in the pull request.

## What the code does

`Budget{MaxRetries int32; Within time.Duration}` bounds the restarts of one child (`pekko-actor/actor/supervision.go:34-43`). `requestPermission` in `supervision_budget.go:42-59` answers allow or deny exactly as Pekko's `requestRestartPermission` (`FaultHandling.scala:58-64`): a count below 1 denies, a count with no window is a plain limit, a window turns the count into "at most N per window" and no count with no window is unlimited. `DefaultStrategy` is a different value: `OneForOne` with `Budget{MaxRetries: -1, Within: -1}` (`supervision_default_internal_test.go:298`), the spelling of Pekko's constructor defaults. The literal `Budget{}` denies every restart and is never a default; ADR 0017 S4 now records the alternatives that were not adopted.

## Acceptance criteria

| AC | Criterion (issue text, abridged) | Class | Evidence |
|---|---|---|---|
| AC1 | Explicit `MaxRetries`/`Within` matrix including zero, finite, unlimited and overflow | **PROVEN** | ADR 0017 S4 (`docs/adr/0017-supervision.md`, matrix of 4 counts by the window columns, 24 cells with the negative and `2^32 ms` windows) and row A4. Go: `supervision_budget.go:19-78`. Tests: `TestSup2BudgetMatrix` (24 cells, `supervision_budget_internal_test.go:195`), `TestSup2BudgetUnlimited` (`:276`), `TestSup2Conversions` (`:325`), `TestSup2Int32Wrap` (`:361`). Pekko: `FaultHandling.scala:58-88`, `:303-308`, `:550`. |
| AC2 | Separate `DefaultStrategy` defaults from a zero `Budget` literal | **PROVEN** | `DefaultStrategy` is `Budget{-1,-1}` and does not invent a limit: `TestDefaultStrategyHasNoInventedLimit` (`supervision_default_internal_test.go:282`, `:298`). `Budget{}` denies: the doc comment `supervision.go:34-37`, row A4, the `Budget{}` case of `sup1Decisions` (`supervision_goside_internal_test.go:251`) and `TestAllForOneBudgetZeroStopsTheChild` (`supervision_allforone_internal_test.go:336`). The separation is now stated in ADR 0017 S4 and in `pekko-actor/README.md`. |
| AC3 | Preserve previously accepted rules | **PROVEN** | `git diff 9063651 develop -- docs/adr/0017-supervision.md` (the squash that accepted the ADR) touches no line of A4, of S4 or of the `MaxRetries` rows; this change only adds text after them. Verified on 2026-10-28. |
| AC4 | A reviewable ADR with source-backed alternatives, the selected proposal, observable differences and unresolved Owner decisions; not Accepted without the decision | **PROVEN by this change** | The reviewable ADR is ADR 0017 S4/A4 (decision D-1). It already had the selected proposal, the source and the observable differences, and the Owner's acceptance of 2026-10-14. The missing alternatives for the zero `Budget{}` are recorded in the new S4 subsection (decision D-2), with the observable difference and the reason each is not adopted; the amendment is marked in the status of the ADR. Unresolved Owner decisions for this slice: none. |
| AC5 | Public contract and traceability of this slice, no claim about sibling stories or full parity | **PROVEN by this change** | Public contract: the doc comment of `Budget` (`supervision.go:34-43`), row A4, and the new paragraph of `pekko-actor/README.md`. Traceability: this document. The inventory row of `SupervisorStrategy` (`INV-A-actor.md:66`, `ACTINV-038`) lists #779 and #825 and stays an unchanged dated snapshot. No claim about #825 beyond its own document. |

## Evidence classes

- **Historical Pekko run, not re-run here.** The 24 cells were produced on 2026-10-14 by calling the real `ChildRestartStats.requestRestartPermission` (`migration/stage-1/supervision/h4-repro/a2a4-sbt.out:442-465`); the Go table repeats those verdicts one for one.
- **Source reading and transcription, not a Pekko run.** The exact window edge and the zero reading come from `FaultHandling.scala:66-88` and `migration/stage-1/supervision/h4-repro/a5-edge/oracle.out:1-21`. The alternatives table in ADR 0017 S4 is analysis of the accepted text and of Pekko's constructor defaults (`:468-470`, `:582-584`); it is not a record of what was debated when the ADR was accepted.
- **Current.** The Go tests, run by us, listed below. No Pekko scenario was executed for this change.

## Gates

Reused from SUP-06, not repeated: on `108880b` (same Go code), unit tests of `pekko-actor` and of every module offline, `gofmt`, `go vet`, `scripts/check_deps.sh`, coverage `pekko-actor` 98.58%. Details in [`../sup-06/README.md`](../sup-06/README.md).

New, on this branch (base `develop` `9a7076fa6e612d7f35454fa66a164142141c6eed`, no `.go` file changed), 2026-10-28, no `-race`, no workbench, offline with `unshare -rn`:

| Gate | Result |
|---|---|
| `cd pekko-actor && go test -count=1 -v -run 'TestSup2BudgetMatrix\|TestSup2BudgetUnlimited\|TestSup2Conversions\|TestSup2Int32Wrap\|TestSup2ZeroReading\|TestDefaultStrategyHasNoInventedLimit\|TestDefaultDecider\|TestSup1InvalidDirective\|TestAllForOneBudgetZeroStopsTheChild' ./actor` | `ok`, 9 test functions PASS |
| `make ci-fast` (local equivalent of the CI job `go`) | exit 0; some packages were served from the Go test cache, whose inputs are unchanged |

**Checks not executed, listed in full:**

- `golangci-lint` v2.14.0 and `gocyclo`: not run (the pinned binary is not on this machine and installing it needs the network). The Owner accepts this for this pull request only, because it changes documentation only (D-4, point 3). No Go line changed; that the result would be the same is an inference, not a run.
- `-race`, the workbench and any Pekko scenario: not run.
- Remote GitHub Actions jobs `go` and `label`: **never started.** GitHub refused both with zero steps and the annotation "The job was not started because recent account payments have failed or your spending limit needs to be increased". They gave no result, they did not pass, and nothing ran remotely on this head. The Owner accepts proceeding without successful remote checks for this pull request only (D-4, point 4).
- The local `make ci-fast` and the nine focused tests are the evidence accepted in their place (D-4, point 2), together with the SUP-05 and SUP-06 evidence reused on identical Go code (D-4, point 1).

## What stays open

#779 stays **open and unmodified** with its labels. Closing it as completed and removing its status label belong to the Owner, after this pull request is integrated under a separate, explicit merge authorization and the final tree and the evidence are reconciled. The window anchor under `AllForOne` (observation G3 of SUP-06) is not part of this story.
