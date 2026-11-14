# Addendum 2: budget exceptions, deviations, GMdefaultps clause, historical evidence (fifth brief)

Applies on top of the package of `361d871` and `ADDENDUM-1.md`. Changes the documents it names and nothing else. Campaign 3 (tree `6b89490e0cae091a9df1889cc4c93e2686507363`, commit `3170ef84940ec71f1ec1a3ece8293223ad9581d4`, darwin/arm64, go1.27.2) runs with its inputs unmodified; this addendum edits only documents under `migration/stage-1/restart-gaps-173/`, which are not inputs of the campaign (`provenance()` in `judge/mutate_go.py` looks at `judge` and `pekko-actor` only).

## 1. Deviations: both implementer runs exceeded their caps
| ID | Unit | Cap | Measured | Excess |
|---|---|---|---|---|
| DV-B | B (GMoverlap) | 120,000 | 131,865 | +11,865 |
| DV-S22 | S22 | 200,000 | 248,725 | +48,725 |

Both runs are recorded with their real consumption. Neither is to be described as done within its cap. No launcher enforced the caps, so these are deviations by omission of a mechanism, recorded as deviations.

## 2. Exceptional ceiling
The Owner raises the incremental ceiling from 520,000 to **600,000**, exclusively for: one independent Opus review (up to 110,000, USD 3, 12 minutes); one correction round (reserve 30,000); coordination and remaining validations (estimated reserve 60,000, an estimate and not a measured consumption). Check: 380,590 + 110,000 + 30,000 + 60,000 = 580,590. The stop at 416,000 may be crossed for those activities only. **Not authorized:** new implementers, functional extensions, a second review, a native review in addition to Opus. If the real consumption of an activity is higher than its forecast, the real figure is recorded.

## 3. Review
Launched after the campaign result is evaluated and the candidate is stable, once. It must cover the whole diff `origin/develop..HEAD`, including unit B, unit S22, the first-round tests and the mutation classifier. If the campaign contains an INVALID row: the cause and impact are presented first, no campaign is launched automatically, and the review is not spent on a candidate that is still going to change. A correction that changes campaign inputs (`pekko-actor/`, `judge/`, specs, comparator, toolchain) makes the campaign evidence stop being CURRENT until the corresponding validation is repeated.

## 4. GMdefaultps clause (corrects "exclusively G5.f")
The required detection is `G5.f`. `G5.c` and `REQ` may fire as consequences of the missing second `PostStop` (measured in unit S22: `G5.f` "c: 1 post-stop, expected 2", plus `G5.c` and `REQ`). Accepted provided `G5.f` fails because of the observed `PostStop` count and not because of a harness error. Applied in `RULEBOOK.md` S6, `S22-GO-PROPOSAL.md` section 5 and `review-prompt.md`.

## 5. Historical evidence
`judge/go-mutation-report.tsv` and the tag `evidence/go-campaign-50fe7c2` correspond to the commit, tree and **definition of GMoverlap that existed then**; they are kept as they are. The 24 FAIL rows that `scripts/check-judge-matrix.py` reports after unit B ("definition_sha256 does not match the current definition") say that the old rows are not valid for the new definition. They are not a refutation of that historical run. A new valid report can renew CURRENT; what the checker shows for HISTORICAL is verified separately after any change of the file, and replacing the TSV is not by itself claimed to settle both questions.

## 6. Hashes
`PACKAGE-HASHES.tsv` is regenerated for the files changed by this addendum (`RULEBOOK.md`, `S22-GO-PROPOSAL.md`, `review-prompt.md`, `BUDGET.md`) and adds `ADDENDUM-2.md`.
