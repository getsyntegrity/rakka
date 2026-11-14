# EXECUTION PACKAGE: persisted data contract and zero-custody scope (#391)

Purpose: state, before the validation runs, what this unit is, which kit process applies to it and what will and will
not be executed. The outcome is recorded afterwards in `VALIDATION-RECORD.md`; this file is not edited to match it.

## Unit

| Field | Value |
|---|---|
| Issue | #391 (ZCP-001); related #211 (PER-001), #350 (ANA-003, closed), #393 (TEN-005, closed), #461 |
| Deliverable | `docs/adr/0018-persisted-data-contract-and-zero-custody-scope.md`, status `proposed` |
| Nature | Analysis of upstream Pekko plus a proposed contract. **No code is ported and none is written.** |
| Base | `origin/develop` `c4d44b701415df3226be98b448b861339270e6d6` |
| Pekko | `upstream/pekko` v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (read-only) |
| Kit | `migration-kit` `cf91c9d5068d9aaf95a36164169f08c3e636c909` (read-only) |
| GoAkt | not consulted; no ADR claims it solved this better |
| Operator model | `claude-sonnet-5-5`, one agent, no subagents |

## Traceability chain (kit rule: source and test, then contract, then unit, then execution, then evidence)

| Link | Artifact |
|---|---|
| Source and tests | `source-anchors.tsv`: each Pekko `file:line` the ADR cites, with the substring expected on that line |
| Contract | the ADR: matrix of persisted data (Pekko to Go), boundary options, and the questions Q1 to Q8 |
| Unit | this package; there is no `manifest.tsv` because nothing is translated |
| Execution | `check-citations.py` and the documentation gates listed below |
| Evidence | `VALIDATION-RECORD.md` |

## Kit process: what applies and what does not

| Kit step | Applies? | Reason |
|---|---|---|
| Feasibility, judge setup (00, 00b) | No | done at stage 1; unchanged |
| Dependency map (01) | Reused, not re-run | ANA-003 already produced `migration/evidence/ana-003/depmap-imports` and `depmap-implicit` for `persistence`; no source changed since |
| Gap inventory (02) | No | this unit decides a data contract, not a translation inventory |
| Stress test, translation, build, parity (03 to 06) | No | no rule is amended, no file is translated, no Go code exists for this contract, no Pekko oracle can judge an own-extension contract |
| Manifest (`make_manifest.py`), queue runner, implementer and reviewer roles | No | no port; no fan-out |

Consequence: **the kit is not executed for this unit.** No kit command is run, no manifest is created, no cost row is
added to `migration/cost-log.tsv` (it records kit steps; none ran). Where a later unit ports the persisted-record
behavior described here (for example the journal SPI of #211), that unit runs the kit under its own manifest and cites
the matrix of this ADR as its input.

## What will be executed

1. `check-citations.py`: every Pekko citation in the ADR resolves to an existing line that contains the anchored text.
2. Existing documentation gates that read the touched paths: dependency direction, ADR signatures, judge matrix, file
   headers, ASCII, repository path references.
3. `gentle-ai review assess` over the committed diff, to read the risk tier (not to start a review).

## What will not be executed

`go test`, coverage, lint, mutation campaigns, the parity judge, `sbt` or any JVM run, a second review round. The diff
contains no `.go`, `go.mod`, `judge/` or script change under `scripts/`.
