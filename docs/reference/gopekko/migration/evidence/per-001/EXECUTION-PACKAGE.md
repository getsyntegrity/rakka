# EXECUTION PACKAGE: Journal SPI and sequence contract (#211)

Purpose: state, before the validation runs, what this unit is, which kit steps apply and which do not, and what will and
will not be executed. The outcome is recorded afterwards in `VALIDATION-RECORD.md`; this file is not edited to match it.

## Unit

| Field | Value |
|---|---|
| Issue | #211 (PER-001). Inputs: ANA-003 (#350), ADR 0018 (#391), ADR 0016 (#393, `proposed`) |
| Deliverable | `docs/adr/0019-journal-spi-and-sequence-contract.md`, status `proposed` |
| Nature | Analysis of upstream Pekko plus a proposed contract. **No code is ported and none is written.** |
| Base | `origin/develop` `b032bba288af19333b372ef33cc1420a9166df07` |
| Pekko | `upstream/pekko` v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (read-only) |
| Kit | `migration-kit` `cf91c9d5068d9aaf95a36164169f08c3e636c909` (read-only) |
| GoAkt | not consulted |
| Operator model | `claude-sonnet-5-5`, one agent, no subagents |

## Chain (kit rule: source and test, then contract, then unit, then execution, then evidence)

| Link | Artifact |
|---|---|
| Source and tests | `SOURCE-ANCHORS.md`: each Pekko `file:line` the ADR cites, with the text expected on that line |
| Contract | the ADR: where "no gaps" applies, TCK traceability table, contract by class, ADR 0016 dependency table, questions Q1 to Q7 |
| Unit | this package. There is no `manifest.tsv` because nothing is translated |
| Execution | the citation script (in `VALIDATION-RECORD.md`) and the documentation gates listed below |
| Evidence | `VALIDATION-RECORD.md` |

## Kit steps before a future port of the journal

None of these ran for this unit and none is claimed. They are what the port unit must do first.

| Step | Status for the journal | Note |
|---|---|---|
| 00 feasibility | Not done for `persistence`. Stage 1 feasibility did not cover it | The model plan is the owner's decision at this gate; later prompts take the chosen tiers as explicit placeholders |
| 00b judge setup | **Not done. No persistence judge exists and none is created here** | The candidate oracle is the upstream TCK (`JournalSpec`), which needs a JVM and `sbt` run that has not been done. Whether it, or hand-ported cases, is the oracle is the judge unit's decision. Cases that are extensions or depend on ADR 0016 have no Pekko oracle |
| 01 dependency map | ANA-003 produced one (`migration/evidence/ana-003/`); its two skeptic reviewers did not run | Reuse only after those reviews. It shows one file cycle across `Eventsourced`, `JournalProtocol`, `Persistence`, `Persistent` and `MessageSerializer` |
| 02 gap inventory | Not done | Records what Go demands that Scala does not; follows the judge decision |
| 03 stress test of the rulebook | Not done | Needed because the cycle above may need a rule amendment before translation |
| 04 translation kickoff | Not done | Requires 00 to 03 and the owner's acceptance of the contract questions |
| 05 survey build, 06 post parity | Not done | After translation |

## What was executed for this unit

1. Reading of the pinned source and tests listed in the anchors (documentary analysis, not a kit step).
2. The citation script: every anchor row matches its line, and every `ALIAS:line` in the ADR has a row.
3. A negative check showing the script fails on a false citation.
4. Existing documentation gates that read the touched paths: dependency direction, ADR signatures, judge matrix, file
   headers, ASCII, repository path references.

## What was not executed

`go test`, coverage, lint, mutation campaigns, the parity judge, `sbt` or any JVM run, native review, subagents, and the
kit. The diff contains no `.go`, `go.mod`, `judge/` or `scripts/` change.
