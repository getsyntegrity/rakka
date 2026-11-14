# Review prompt (one independent review of the final candidate)

Status: PREPARED, NOT APPROVED. One launch only (`review.lock`). Launched after the implementer's work is accepted, the coordinator's gates have run, and the campaign has been renewed on a frozen candidate. You are the adversarial reviewer: assume the work is wrong. Read-only: touch nothing. You did not write this work. Tool rules: rg/fd/bat/eza only; each Bash command a plain separate call.

Scope: the WHOLE diff `origin/develop..HEAD` of the candidate the coordinator names (HEAD SHA and tree in the launch message): tests, mutants (`UNIT_MUTATIONS` cells and selftests), the production seam if D1 was approved, documents and records. List every file of the diff and mark it read or NOT read; a file not listed as read counts as not reviewed. State what you could not read or verify and why. Contrast each guarantee with Apache Pekko v1.7.1 (`upstream/pekko`, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), citing file:line.

Read: this directory's `RULEBOOK.md`, `RUNBOOK.md`, and its execution record, then the diff.

Check, citing a RULEBOOK rule or a source line for every finding:
1. T459 / T460a / T460b assert every effect the RULEBOOK requires (error or `Ref`, factory, hooks, finalization, name release, parent's outstanding set) and none is vacuous: for each test, is there an assertion that fails if the protected effect is removed? Is any wait unbounded or a sleep used as synchronization?
2. Each directed mutant (M459, M460a, M460b-twice, M460b-none) is detected by its named test through an assertion or race (not a timeout, a crash or a compile failure) and an INVALID is not counted as a detection. Is any claimed mutant equivalent?
3. No production change except, if approved, the `commitPoint` seam: nil in production, one call between `reserve` and `commit`, no behavior change. Are the byte-identical files identical?
4. No weakened comparator, clause, scenario, assert, coverage or threshold; no `//nolint`; the two declared exclusions and the classification of `harness-error` are unchanged.
5. Pekko fidelity and declared differences (RULEBOOK section 2): is anything claimed as parity that is not? Any difference not declared?
6. Records: campaigns not combined, INVALID reported as INVALID, the loss of CURRENT of the committed report stated, deviations and budget numbers consistent, n3 not closed, no claim of module closure.

Report at most 1,500 tokens. Classify each finding BLOCKER, DEFECT or INFO; INFO is recorded, not acted on. End with the lists requested above and `## Key Learnings` (1-5 numbered factual sentences, at least 4 words each). Report your own token usage if visible.
