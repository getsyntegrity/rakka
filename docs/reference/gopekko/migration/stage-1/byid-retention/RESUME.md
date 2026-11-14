# RESUME: retirement of terminated actors from `System.byID` (ADR 0024, T2-T5; issue #541)

Status: **EXECUTED** after the Owner approved the package (hashes verified at commit `7dce9cf`); see `EXECUTION-RECORD.md` for the run, the measurements and the limits. The text below is the package as it was approved. Base `develop` `cd558941`, ADR 0024 text from PR #542 (`7f52bb5`); kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`.

## Read in this order
1. `RULEBOOK.md` (the Pekko evidence with line numbers, the goPekko facts, decisions D1-D5 and details M1-M6, invariants I1-I9, tests C1-C9 and R1-R10, mutants, declared differences).
2. `RUNBOOK.md` (flow, budget, procedure, surfaces, commands, hashes).
3. This file for the RUNBOOK's own hash.

## Hash of the RUNBOOK (cannot be inside it)
`RUNBOOK.md` sha256 `225bd8d01cb31b3cb1cfbdf042442881d2b443a6a58a967e113fd0c79409179b`. The other eight hashes are in RUNBOOK section 9.

## Next step
The Owner approves the hashes and says to run. Then follow RUNBOOK section 5 on branch `feat/actor-byid-retention`. Budget: stop at 450,000 tokens, ceiling 560,000, 45 minutes of agent time.

## Not authorized by this package
Merge, native review, a second review, a judge run or campaign, an evidence tag, closing #541, T6 (field release), any ID format change, `gopekko-testint`.
