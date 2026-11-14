# Addendum 1: Owner authorization and conditions (fourth brief)

Applies to the package of `361d871`, verified before execution: the seven hashes of `PACKAGE-HASHES.tsv` matched at that commit. This addendum records the authorization and the conditions. It changes the documents named below and nothing else; any further change of contract, surface, model or budget returns to the Owner.

## Authorization
- Incremental budget up to **520,000 tokens**, evaluation stop at **416,000**, an explicit exception to finish unit B, unit S22 and the review of #480. The earlier consumption stays recorded: 661,896 reported tokens plus unmeasured coordination; the accounting is not reset. The budgeted cumulative figure 1,181,896 is a ceiling sum, not a measured total, and does not include coordination precisely.
- Roles and models as in `BUDGET.md` section 2: B one Sonnet implementer up to 120,000; S22 one Sonnet implementer up to 200,000, only after B is accepted; one independent Opus review up to 110,000, USD 3, 12 minutes; correction and coordination reserves as in `BUDGET.md`.
- No native review in addition to the Opus review.
- The report and the tag are prepared; the tag is not pushed. PR #480 stays draft; no merge; #173 and #461 stay open.

## Conditions
1. **Review scope.** The reviewer covers the whole diff `origin/develop..HEAD`: production, harness, mutants, tests, scripts and documentation, plus the Pekko lines needed to evaluate the guarantees, and declares what it could not read. Applied in `review-prompt.md`.
2. **B3 corrected.** A remaining race means the evaluated design failed: it is recorded with design, sites, run and log, and B stops with that evidence. No general impossibility is claimed. S22 does not start while B is blocked. Applied in `RULEBOOK.md` B3.
3. **Hashes.** `PACKAGE-HASHES.tsv` is verified before execution and regenerated for the files changed by this addendum (`RULEBOOK.md`, `review-prompt.md`, and this file added).
4. **Kit process.** The work runs through the package: baseline recorded first, one implementer at a time, edit surfaces and byte-identical files checked by the coordinator after each run, inputs, execution and results kept in `EXECUTION-RECORD.md`. Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` is the origin of the behavior. RG1-RG3 stand; the earlier work is not retroactively a kit execution.
5. **Budget is an operating limit, not a target.** When a measured consumption is higher than a forecast, the real figure is recorded; a cap is never written in place of a measurement.
6. **Evidence rules.** INVALID stays separate, campaigns are never combined, nothing is repeated looking for luck. Expected final campaign: 18 of 18 designed mutations plus the benign mutation surviving.

## Unit B design approved for the first run
The single design evaluated in unit B is the one of `RULEBOOK.md` B2 (the extra worker proceeds only after it sees, under `c.mu`, that `c.actor` is installed). If it fails, B3 applies.
