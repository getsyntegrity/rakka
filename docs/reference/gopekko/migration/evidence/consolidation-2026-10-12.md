# Consolidation of shared-file entries, 2026-10-12

Base: `origin/develop` `051b41fe75736e1d38da9f22c50e2e7706ca0c0a`. Docs only. Figures are recorded as found in the cited sources; nothing here was measured independently.

## Consolidated

| Entry file | Content | Went into |
|---|---|---|
| `migration/stage-1/ask-endpoint-port/entries/cost-log.entry.tsv` (#161) | cost row `act-015-ask-endpoint-port` | `migration/cost-log.tsv` (tokens corrected, see below; `wall_clock_min` set to `unknown` because the 3 came from summing per-agent seconds (160 s) and was never measured as elapsed time (native review suggestion R3-001 of the second candidate; the derived 160 s stays in `EXECUTION-RECORD.md` section 3); model cell reduced to the single token `sonnet` like the earlier kit rows, the self-reported `claude-sonnet-5-5` stays in `EXECUTION-RECORD.md` section 3; rows ordered by timestamp, per the native review suggestion R3-002) |
| `migration/evidence/ana-003/coordinator-entries.md` (#350) | cost row `ana-003-depmap-persistence` (tokens `unknown`, 0 subagents) | `migration/cost-log.tsv`, copied unchanged |
| `.../entries/milestone-1.entry.md` (#161) | I-2 scenario table, with the statement that these are in-package tests, not judge scenarios | `migration/stage-1/milestone-1.md`, new subsection "Ask increment I-2" under "Scenarios" |
| `.../entries/actor-core-parity.entry.md` (#161) | parity note, no parity claimed | `docs/actor-core-parity.md`, new section 11 |
| `coordinator-entries.md` (#350) and `ten-005/shared-entries.md` (#393) | index/milestone lines and the #393 closure checklist | `migration/stage-1/milestone-1.md`, new section "Index of consolidated analysis and design items" (before "Out of scope") |

Also edited: `VALIDATION-RECORD.md` section 2 (new row for the post-merge review of PR #427, lineage `review-2e18d09c0d43992a`; it did not gate the merge) and `EXECUTION-RECORD.md` sections 3 (correction, evidence status).

## Arithmetic discrepancy (#161 cost row)

- Per-agent tokens in `EXECUTION-RECORD.md` section 2: 67,711 + 86,407 + 76,147 = **230,265**. The record (section 3) and the entry say **230,246**, a difference of 19.
- No other source for 230,246 exists in the repository, so it cannot be justified; the 230,265 sum can be recomputed from figures the record itself lists. `migration/cost-log.tsv` therefore records 230,265. Both numbers are self-reported by the orchestrating session.
- `EXECUTION-RECORD.md` section 3 keeps the original sentence and carries a one-line correction; the `entries/` file is left untouched as the original.
- Wall clock: 29 + 77 + 54 s = 160 s, consistent with "about 3 minutes" (the cost row records `unknown`, since the sum is not a measured elapsed time). The sum is not a measured elapsed time. Agent count 3 matches the three listed (implementer, two reviewers; no fixer, no third reviewer).

## Not consolidated

- **#393 cost-log row:** none. The kit did not run for TEN-005; a row would invent a step, tokens and wall clock (`ten-005/shared-entries.md`).
- **ADR index line for ADR 0016:** `docs/adr/` has no index file (only the template and ADR files), and none was created. The #393 line went into the milestone-1 index section instead, with the ADR path.
- **Cross-agent notes** of `coordinator-entries.md` (red base CI run, ADR 0017 not used, #161 untouched): left in their source file; they are notes, not shared-file entries.
- No cost-log or milestone checker exists in `scripts/` (only `check-judge-matrix.py`, `check-adr-signatures.sh`, coverage, deps and module scripts).

## Kit-run evidence status

Recorded in `EXECUTION-RECORD.md` section 3: authorization (section 4), execution recorded by the session (commit `59bd95b`, manifest, cost entry), and the absence of any raw kit-agent log in the repository or in `goPekko-evidence`. The kit is not credited with the runtime; no Pekko parity is claimed.
