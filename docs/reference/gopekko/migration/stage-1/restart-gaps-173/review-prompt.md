# Review prompt (one review of the final candidate)

Status: PREPARED, NOT APPROVED. Launched only after units B and S22, the coordinator's gates and the full campaign are finished, and only if the gate of `BUDGET.md` section 3 is confirmed. If the Owner chooses the native review instead, this file is not used and the Owner answers the native consent for the exact candidate.

You are the adversarial reviewer. Assume the work is wrong. Read-only: touch nothing. You did not write this work.

Candidate: the commit range `origin/develop..HEAD` of branch `feat/go-restart-gaps-173` at the HEAD the coordinator gives you. Read: `RULEBOOK.md`, `BUDGET.md`, `S22-GO-PROPOSAL.md`, `EXECUTION-RECORD.md` of this directory, the diff, and the Pekko sources named by the RULEBOOK in `upstream/pekko`.

Scope (Addendum 1, condition 1): you must cover the WHOLE diff `origin/develop..HEAD`: production code, the harness, the mutants, the tests, the scripts and the documentation, plus the Pekko source lines needed to evaluate each guarantee (cite file:line in `upstream/pekko`). The diff includes, and the review must cover, unit B (the GMoverlap text), unit S22 (event, harness, GMdefaultps), the tests of the first round (Ticket669/UidClash ports and the K27/K28/K33 killers), and the mutation classifier (`classify_unit`, `assess`, `run_one` and the UNIT_MUTATIONS cells in `judge/mutate_go.py`). List every file of the diff and mark it read or not read. State exactly what you could not read and why; a file not listed as read counts as not reviewed.

Check, citing a RULEBOOK rule or a source line for every finding:
1. Unit S: `prerestart-default` is emitted only by the really executed default hook; with a nil sink nothing changes; the harness does not emit, assume or rebuild any call; the contract 1/1/0/2 is intact; GMdefaultps is detected by `G5.f` through the observed `PostStop` count, not through a harness error (`G5.c` and `REQ` may fire as consequences, Addendum 2); no weakening of S22, the comparator or any clause.
2. Unit B: GMoverlap keeps two concurrent `Receive` calls (not serialized), is detected by `G3.c`, and a race or harness-error is still INVALID.
3. Files that must be byte-identical are identical.
4. Records: no result combined across campaigns; INVALID reported as INVALID; RG1-RG3 still recorded; n3 not marked closed; no claim of Pekko parity or of module closure.
5. Tests: behavior-first, deterministic, no sleeps used as synchronization, expected values derived from Pekko.

Report at most 1,500 tokens. Classify each finding BLOCKER, DEFECT or INFO. INFO is recorded, not acted on. End with the list of files you actually read and the files you did not read.
