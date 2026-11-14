# U4 resume point (informational, not part of the approval hashes)

State: **implemented, pending close.** The design (`RULEBOOK.md`, D1-D8) and the execution package were approved by the Owner and the run happened: implementer (Sonnet, 303,360 tokens), one Opus review (160,512 tokens, no blocker or major finding), corrections F1 and F2 by the coordinator, all gates green on the final code (suite, complete `-race`, lint 0 issues, coverage 98.34%, 22 judge anchors). The code is in `375c97b` and `3012f47`; the evidence is in `EXECUTION-RECORD.md`. PR #457 is a draft; no merge.

Native review of the code: **done and approved** on `1777d37..dee0a81` (lineage `review-0e22c2e9921f8c85`, risk medium, one consolidated lens), acknowledged; it authorizes no merge. It found two informative test gaps, not demonstrated defects, both recorded in `EXECUTION-RECORD.md` section 8 with acceptance criteria and both pending for the module close: F3 (the `stopping` branch of `Spawn`, `spawn.go:78-80`) and `R3-factory-goexit-under-accepted-stop` (`cell.go:215-235`: a failed factory after an accepted stop must reach no decider and emit one `stop-accepted`; a factory Goexit inside `terminate` must complete the termination exactly once). The commit that records this is documentation only and is not covered by that review.

Pending for the unit (record section 8): the U4 mutants (written, not run).

Pending for the module (not run in U4): the two test gaps above, mutants M1-M15 and the U4 mutants, the judge campaign and tag, the repeated complete `-race`, stress runs, the integral review, the S13/S14/S16 mirror (U8, #175), the Pekko surface inventory, the carried gaps of U2, and the ACTINV rows (the inventory is not in the repository). No parity is claimed and CURRENT stays stale.

Do not re-run the implementer from this package: its permissions and prompt describe the pre-run state.
