# Incremental budget: B (GMoverlap) + S22 in Go + final review

Status: **PROPOSED, NOT APPROVED.** Nothing below is authorized until the Owner approves it in words. This file does not extend the old ceiling and does not authorize any agent.

## 1. Accounting already spent (not reset)

| Round | Source | Tokens |
|---|---|---|
| 1 | explorer 159,786 + writer 238,433 | 398,219 |
| 2 | lint/gocyclo run 80,203 + GMoverlap investigation 47,471 + kit-usage writer 136,003 | 263,677 |
| Total reported | subagents only | **661,896** |

The cap that earlier units used is 560,000 (evaluation stop 450,000). Both rounds count, so the unit is **101,896 over the cap (18.2 %)** before any new work. Parent-session coordination (this conversation, mutation runs, campaigns) is not measured and is not in the total; the real figure is higher. Round 1 and 2 are not described as a kit execution (RG1-RG3 stand).

## 2. Incremental plan (all new work, one ceiling)

| # | Work | Role | Model | Cap (tokens) | Other limits |
|---|---|---|---|---|---|
| 1 | Unit B: deterministic GMoverlap (`judge/mutate_go.py` and its selftest) | implementer, 1 agent | sonnet (mid tier) | 120,000 | wall 45 min active |
| 2 | Unit S22: runtime event, harness S22, GMdefaultps, docs | implementer, 1 agent, only after unit 1 is accepted | sonnet (mid tier) | 200,000 | wall 60 min active |
| 3 | One review of the final candidate | reviewer, 1 agent, read-only, separate from the writers; or the native review if the Owner chooses it | opus | 110,000 | USD 3, wall 12 min, `review.lock` |
| 4 | One correction round of confirmed BLOCKER/DEFECT findings | coordinator (0 agents) unless the Owner asks otherwise | n/a | 30,000 reserve | at most 1 round |
| 5 | Coordination, gates, mutants, the full campaign, publication package | parent session, 0 agents | n/a | 60,000 reserve (estimate, not measurable) | |
| | **Incremental ceiling** | | | **520,000** | evaluation stop at **416,000** (80 %) |

Worst case cumulative if every cap is reached: 661,896 + 520,000 = **1,181,896**. The ceiling is not a target. Writers run one at a time; no parallel writers.

## 3. Gates and stop criteria

1. Unit 2 starts only after unit 1 passes its acceptance and the coordinator's own gates.
2. The review is launched only if `incremental observed + remaining reserve + 110,000 <= 520,000` is confirmed with the numbers written down. If not, no review is launched and the record says so.
3. A correction round starts only if `observed + remaining reserve + 30,000 <= 520,000`.
4. At 416,000 incremental the coordinator stops and asks before any further agent or round.
5. A run killed by its wall or token cap counts its whole cap as spent and produces no accepted work.
6. Unit 1 has a stop (RULEBOOK B3, corrected by Addendum 1): if a data race remains with the evaluated design, the design is recorded as failed, with its evidence, and the unit stops. No claim that no design can exist. No serialization, no weaker comparator, no retry loop. Unit S22 does not start while B is blocked.
7. Any new difference from Pekko found by any unit stops that unit and returns to the Owner.
8. No repetition of a campaign or of a mutant looking for a clean run. A failure is investigated first.
9. The old ceiling is not raised by this file; approval of this budget is an explicit, separate Owner step.

## 4. Decision requested
Approve, change or reject: the roles and models in section 2, the 520,000 ceiling with the 416,000 stop, and the choice of reviewer (opus read-only agent, or the native review with its per-candidate consent, which the Owner answers).

## 5. Update by Addendum 2
Measured so far: unit B 131,865 (cap 120,000, +11,865, DV-B) and unit S22 248,725 (cap 200,000, +48,725, DV-S22); incremental total 380,590; cumulative subagents 1,042,486; coordination not measured. The ceiling is raised exceptionally from 520,000 to **600,000** for: one Opus review (up to 110,000), one correction round (reserve 30,000), coordination and remaining validations (estimated reserve 60,000): 380,590 + 110,000 + 30,000 + 60,000 = 580,590. The stop at 416,000 may be crossed for those activities. No new implementers, no functional extensions. The sections above keep their original text; this section supersedes their numbers.
