# Audit 3: bounded SQLite runner evidence integrated; full parity remains open

Static audit of `develop` at **`e75742e7e4e3650a7a2de7325f431d853b680664`**, 2026-10-23. The bounded atomic route on both engines remains supported by recorded evidence. The non-atomic SQLite route and its cancellation/no-retry detectors are now integrated. Neither complete U6 validation, full Pekko parity nor survival of server/power failure is demonstrated.

The previous audit is [Audit 3 at 33b2127](auditoria-3-33b2127.md), integrated by #730. Its sections 1-14 describe that earlier SHA; its section 15 records subsequent closures. This document is the current assessment, not a rewrite of that historical record. Authorities: ADR 0026, ADR 0027, ADR 0028, ADR 0029 and ADR 0030; the read-side and parity proposals remain limited by their recorded acceptances.

## 1. Scope and evidence rules

- Audited SHA was read from GitHub's `refs/heads/develop`. Source files and recursive Git trees were fetched at immutable SHAs through the connected GitHub API. No local git checkout or submodule checkout was used.
- This is a delta-led static audit: the complete file list since `33b2127` was checked, the two added test files and relevant production paths were read, and unchanged axes were reconciled with the previous audit and current records. It is not a new exhaustive review of every production file or every upstream capability.
- No engine, test suite, judge, mutation campaign, container or behavioral checker was run. Historical command results below belong to their stated executors. This document received an author self-check, not a new independent review.
- **AD** = accepted design; **II** = integrated implementation; **RE** = recorded execution; **CD-engines** = executed comparison across engines; **CD-Pekko** = executed comparison with upstream. Reading matching code is alignment by reading, never CD-Pekko.
- **REAL** = engine behavior; **SIM** = fixture error, proxy or wrapper around a real engine; **MOCK** = no engine. L1 is real-engine logic; L2 is a client SIGKILL; L3 is a separate engine-process crash; L4 is machine/power failure. SQLite is embedded and has no independent server L3 here. L1/L2 are not power-loss evidence.
- Independence is attributed from a review's explicit executor/isolation description, never inferred from a GitHub account. A preserved summary is not a preserved raw log or full review report. Missing evidence is not reassigned to the author.

## 2. Changes since the audited baseline

The API comparison `33b2127ad62befb91b71b7f4bb4ee6698ef4d332...e75742e7e4e3650a7a2de7325f431d853b680664` contains **five commits**, 18 changed paths, and exactly two changed Go paths, both new integration-test files. No production Go file, module definition, script or workflow changed.

| PR | Integrated HEAD | Squash on develop | Effect |
|---|---|---|---|
| #731 | `71a94e870a771925d9caa1d61376a1238857f08b` | `bce4862d0bf8dd4b183f810a7797703c5352b4dc` | Documentary reconciliation, including M6 chronology |
| #732 | `312ed54e7082c1766520f5d7d25bcd96f20c02d3` | `48403b2309f0e5714481334a110a726dca551c78` | Five non-atomic SQLite runner tests, including a killed child |
| #730 | `e6f214ba5c367aaeaa0003fea7ad428e5c69f3b1` | `7bfb7405d93033bb03266d11512c95de6cdf5dc6` | Previous Audit 3, with later closure section |
| #733 | `a61c041ed17b52d5bd11c4b5fdc969dceb490441` | `03a091a41318100eccb7640f811f05e9c8808046` | Cancellation detector and guard for any Save after failed Save |
| #734 | `e5a67147776419a5cc73fe2ccabb51c791bfcb58` | `e75742e7e4e3650a7a2de7325f431d853b680664` | Consolidated documentation; m1 review-scope correction; incorporates #735 |

All five are merged, verified from current PR metadata. #734 merged at 2026-10-23T19:52:18Z (16:52:18 -03:00). Do not confuse that `mergedAt` with its squash commit's 19:52:17Z timestamp.

Recursive tree comparison independently performed in this audit:

| Comparison to audited SHA | Changed Go blobs | Consequence |
|---|---|---|
| `33b2127` | Only `gopekko-testint/run_at_least_once_sqlite_integration_test.go` and `gopekko-testint/run_at_least_once_detectors_sqlite_integration_test.go`, both added; none removed | Previous production behavior unchanged |
| `76d4c7e5b9189cc8c5b8b298bd98a35b8b96970f` | Only the added detector file | #732's whole-suite evidence predates the detector tests |
| `231f9b7e09748c7e2742307abd929086227bfe86` | None added, changed or removed | Exact Go blob identity with #733's tested commit; not proof of a whole-suite run at the audited SHA |

## 3. Execution evidence and its provenance

| Tested SHA | Executor and scope | Recorded result | Source and limit |
|---|---|---|---|
| `a603b87372` | Author, both engines, whole integration module | 143 top-level tests, 28 atomic, no FAIL/SKIP; 277.315 s | `migration/stage-2/persistence/units/411-u6-atomic-mode.md`, closure record. Historical evidence, no rerun here |
| `642cd5c41a09` and `14853c77038787f713d4750eb268af88deb18fc9` | Reported independent whole-suite runs | Reported 143 / 860 / 28 | Previous audit section 3.1 rows 3 and 5: executor/log/count provenance remains uncorroborated; not counted as newly recorded runs |
| `76d4c7e5b9189cc8c5b8b298bd98a35b8b96970f` | Author, whole suite, both engines, `-race` | 148 top-level tests, 869 PASS entries including subtests, 0 FAIL/SKIP, 277 s | #732 body. Historical local run, go1.27.2 darwin/arm64; no raw log in repo |
| Same `76d4c7e` | Reported independent whole-suite reviewer | Reported 148 / 869, 277 s | Previous audit section 15.1 and session account only. #732 has no issue comments and its body still says no independent review. Do not call this newly corroborated or author evidence |
| `e97e3af7f6194e8bf916ca4dd1f16611925a0a46` | Reported reviewer, older detector version | Reported 150 / 871, 276 s | Session account, not a current whole-suite result and not established by #733's final body/comments. Its guard was later broadened |
| `231f9b7e09748c7e2742307abd929086227bfe86` | Author, SQLite subset, `-race -count=10` | Seven top-level tests x10, 70 top-level PASS, 0 FAIL/SKIP, 4.5 s | [#733 comment 6373024145](https://github.com/getsyntegrity/gopekko/pull/733#issuecomment-6373024145); raw log not stored |
| Same `231f9b7` | Independent reviewer described as a fresh read-only agent, SQLite, scratch copy, no Docker | `-race -count=10`, 110 PASS lines counting subtests, 0 FAIL/SKIP, 4.2 s; targeted mutations fail by assertion | Same comment, explicitly labelled reviewer evidence. Full report/log not stored; no full-suite run |
| Same `231f9b7` | Another recorded review summary of the final delta | READY_WITH_MINORS; local gates and m1-m7 mutation checks | [#733 comment 6372981797](https://github.com/getsyntegrity/gopekko/pull/733#issuecomment-6372981797). Separate summary, not summed with the other review's counts |
| `e75742e` | None recorded for the whole suite | No whole-suite result found | Exact Go identity with `231f9b7` transfers that bounded evidence only |

The seven tests include the original five from #732. 70 and 110 are different counting levels, not conflicting pass rates. Neither 148/869 nor the reported 150/871 is a whole-suite run on the final broadened guard at `231f9b7` or on `e75742e`.

Review scope of documentation: #734 body records a consolidated review at `7ae8a64`, with its last corrections author-checked. Its earlier [comment 6372970725](https://github.com/getsyntegrity/gopekko/pull/734#issuecomment-6372970725) covers `ef9c5c0`, not the final HEAD. #730's corrected review chronology remains in the historical audit; no claim of independent review of the entire final document is inferred.

## 4. Status by axis

### 4.1 Actors

AD/II unchanged: ADR 0030's optional bounded mailbox overflows to dead letters without backpressure; the default remains unbounded. `Terminated` is placed at the mailbox tail, including unbounded watchers. Shutdown does not drain; it is not CoordinatedShutdown. Sources: ADR 0030, `pekko-actor/actor/mailbox.go`, `pekko-actor/internal/mailbox/queue.go`, previous audit section 4.1. No new actor execution exists in this delta.

- T6 retention remains open: #563, linked to ADR 0024 D5. The byID correction is already integrated and must not be reopened as if T6 replaced it.
- Actor closure remains #461. Fidelity issues #674-#685, #688, #689 and #691-#694 are still open in the enumerated issue set. No new issues were created.
- The Go gate contains 15 scenarios; this is not a count of executed Pekko scenarios or proof of complete parity.
- Current `pekko-actor` tree is `464a0f268d97f83956f1e4f6194652a91b493579`. The baseline tree was `8e13e29b382fa27b41427de820e49eb8b42f45ef`; recorded mutation rows inspected in `judge/go-mutation-report.tsv` use `f1697b67a11c06d524dac28d80e2aa68ddf63ce3`.
- `scripts/check-judge-matrix.py` compares the entire actor subtree, including its README. #734 changed that README, so freshness is still not established for HEAD even though actor Go blobs are unchanged. No checker or campaign was run here; this is a static hash mismatch, not a new failed campaign.

### 4.2 Persistence, SQLite and PostgreSQL

Sources: `gopekko-testint/README.md` status and per-unit matrices, previous audit section 4.2, current atomic fixtures. Production is unchanged since `33b2127`; existing execution claims remain at their recorded SHAs.

| Capability | Integrated / recorded evidence | Remaining limit |
|---|---|---|
| Migrations 0001-0003, journal, snapshots, checkpoints | Both adapters integrated; real-engine L1 recorded, including journal conformance | Passing migrations/store tests is not power-loss survival |
| Cross-process recovery | Orderly-close recovery on both engines, plus separate L2 client-kill cases | Keep orderly restart distinct from SIGKILL and engine crash |
| SQLite durability policy | Real pragma checks and shared `sqlitetx.Run`; rollback deadline is 5 s in implementation | Settings are not fsync/power-loss proof; deadline behavior on a real engine remains unshown |
| PostgreSQL client death | L2, real child SIGKILL and recovery | Server container never hard-stopped/restarted for L3 evidence |
| PostgreSQL COMMIT outcomes | REAL deferred failures/40001, SIM socket proxy outcomes | Dropped ACK is not server crash; no general command deduplication |
| SQLite COMMIT failure/cancellation | SIM driver wrappers against real files | Real SQLite COMMIT rejection/interruption not demonstrated |
| SQLite journal Delete | Functional floor/highest retention already exercised on real SQLite | Only the policy-refusal residual below remains |

**G5 residual, precisely:** `sqlitejournal.Journal.Delete` uses `sqlitetx.Run`, preserving the highest number. Functional Delete cases exist in `gopekko-testint/journal_sqlite_cases_integration_test.go`; absence of the literal test name `Journal.Delete` was never evidence of no coverage. `durWriters()` in `durability_policy_sqlite_integration_test.go` contains Migrate, journal Write, snapshot Save and checkpoint Save, but not Delete. Still missing is a real-engine case where Delete refuses a non-durable pragma configuration with `ErrNotDurable` and leaves existing events/highest unchanged. This is an extra detector for Delete's policy path, not an unimplemented Delete operation.

### 4.3 Read side: U1-U6, non-atomic Run, U7/U8

AD remains the accepted subsets in ADR 0026/0027, U5 amendments and U6 M1-M6 plus three extras at `072386623b0139e4d5671eca96e85dc7e5c5252c`, with the accepted 40003 amendment. No decision is accepted by this audit.

| Area | II / RE now | Limit |
|---|---|---|
| U1-U5 | Queries, runner and both checkpoint adapters integrated; real-store L1 recorded | No production Source; listing is not a projection Source |
| U6 atomic route | Test Source over real journal, RunAtomic, durable atomic effect/checkpoint, resume, RK1/RK2 on both engines | One persistence-ID, application-enforced single runner/writer and transaction preconditions; no global exactly-once claim |
| Non-atomic SQLite Run (#732) | Processing/checkpoint order; exclusive resume after checkpoint; failed Save redelivery; counter vs idempotent effect; handler Fail/RetryAndFail; killed child | Runner restart L1, killed child L2; test Source is a finite list, not the journal-backed atomic Source |
| Detectors (#733) | Cancel after handler/before Save: ErrInterrupted + context.Canceled, no Save of 3, checkpoint at 2, effect present, redelivery; failed Save: any subsequent Save triggers assertion | L1; SQLite only; injected Save failure never reaches the decorated durable store |
| U7 / U8 | Not started, ADR 0026 O1/O2/O6 and ADR 0027 vector/rebuild choices remain Proposed | Multi-ID Source/vector and rebuild are separate from crediting the already-tested single-ID routes |

Source paths: `pekko-projection/projection/pass.go`, `projection/atomic.go`, `projection/run.go` (all under `pekko-projection/`); `gopekko-testint/atomic_engines_integration_test.go`, `atomic_kill_integration_test.go`, and both `run_at_least_once*_integration_test.go` files.

The #733 guard closes a channel and cancels before Run returns; the test asserts on that channel independently of `alStore.saves`, which intentionally excludes rejected late calls. It detects retry of 3 and continuing to Save(4), rather than waiting for the old timeout. It assumes Sequence offsets and one synchronous runner. A retry in another goroutine after Run returned, cancel with Save in flight, other injected failing sequences and Save error after actual commit remain unshown. These are limits of the fixture, not newly proven production defects.

PostgreSQL non-atomic Run with pgcheckpoint remains without equivalent real-engine evidence. Do not rerun or reimplement the SQLite route as the next missing capability.

### 4.4 SQLSTATE, R1-R6 and chronology

The implementation read at HEAD (`pgcheckpoint/atomic.go`, `sqlitecheckpoint/atomic.go`) still classifies 40003/P0001 as uncertain, preserves cause and does not retry COMMIT failures. 40001 and 40P01/class-23 and the remaining class-40 prefix use the current rejection classification; 40000/40002 are unchanged. No broader whitelist acceptance is inferred.

| Refinement | Current effect / decision status |
|---|---|
| R1 | pgx classification grounding by reading; not a new execution or blocking design decision |
| R2 | Wording distinguishes before COMMIT from uncertain in-flight outcome; not a collective gate |
| R3 | Invalid configuration validation already implemented; acceptance not inferred |
| R4 | Narrowed handle/default and transaction-ending handler limits already documented/tested; acceptance not inferred |
| R5 | Separate author/reviewer evidence is practiced; no acceptance inferred |
| R6 | Context ended before COMMIT mapped to PhaseCheckpoint in both adapters; implemented/tested but still Proposed; an explicit accept/decline decision remains, without blocking unrelated work |

M6 is historical chronology, without retroactive authorization. #731 reconciled its three stale pending-decision phrases. Neither M6 ratification nor optional whitelist widening is a new P0. 40003 remains classification-only with a mock; this audit knows no established server mechanism for producing it at COMMIT. A proposed experiment must not be presented as a known mechanism.

### 4.5 Serialization, dependencies and guard

- Registry/manifest round trips through journals/snapshots on both engines have L1 evidence. #734 corrected the stale no-real-engine statement in `pekko-serialization/README.md`. Manifest version remains a serializer convention; event-version fields/upcaster chains remain missing (#407, #408, #388; snapshot evolution #409).
- `CLAUDE.md` and `scripts/deps.allow` agree at module level. The restriction that adapter imports of projection exist only for CheckpointStore/AtomicStore is prose: `check_deps.sh` checks owning modules, not purpose of use.
- **Additional control limitation, established by reading:** `check_deps.sh` invokes `go list -test -deps ./...` without `-tags integration`; `gopekko-testint/doc.go` is the only file that builds without that tag. Thus a normal run does not inspect the integration-tagged harness imports. The separate importguard invocation covers SQLite's sqlitemock rule, not that missing build configuration. Ambient GOFLAGS could change this, but the script does not require them. No forbidden import or failing check was demonstrated.
- Real-engine work remains confined to gopekko-testint in this delta: the only new Go files are tagged integration there. The previous whole-repository confinement finding is inherited; this audit did not rescan every untouched test outside the module.
- `docs/parity/capabilities.json` remains `inventory_status: bootstrap` with `capabilities: []`; the guard cannot certify functional coverage. Slices B-E and CI integration remain pending; no guard campaign or closure certification occurred.

## 5. Pekko alignment and comparative evidence

Pins rechecked through GitHub: core `v1.7.1` resolves to `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, matching the submodule; Projection `v1.1.0` resolves to `be34eadcb1771e45abf7207bddaeb22633b7d4f8`, an external source reference, not a submodule. Neither pin changed.

At that Projection commit, the following files were read again:

| Upstream path (apache/pekko-projection) | Go alignment / divergence by reading |
|---|---|
| `jdbc/src/main/scala/org/apache/pekko/projection/jdbc/internal/JdbcProjectionImpl.scala`, adaptedHandlerForExactlyOnce | Upstream saves offset before delegate in one session; Go AtomicStore saves checkpoint before handler |
| `jdbc/src/main/scala/org/apache/pekko/projection/jdbc/internal/JdbcSessionUtil.scala`, withSession | Commit after callback; rollback on exception, original cause preserved; same basic transaction boundary by reading |
| `core/src/main/scala/org/apache/pekko/projection/internal/HandlerRecoveryImpl.scala`, applyRecovery | Recovery wraps futureCallback; Go deliberately retries PhaseHandler only and never a COMMIT failure. Upstream behavior on lost ACK has not been experimentally reproduced |

**CD-engines remains partial:** existing shared journal/snapshot/checkpoint/atomic scenarios ran on SQLite and PostgreSQL according to their records. New non-atomic cases are SQLite-only, so they add no new cross-engine comparison. REAL, SIM and MOCK cases are not interchangeable.

**CD-Pekko for persistence/read side remains absent.** Historical actor-side executions are separately recorded in `docs/actor-core-parity.md` (controlled S01/S10/S11/S19; concurrent S01/S02/S10/S11/S19/S20/S21). No fresh actor comparison or whole-surface parity is established by those runs or this audit. They must not be combined with the 15-scenario Go gate into a fictitious 15-scenario Pekko campaign.

## 6. Documentation and remote CI

#731/#734 resolved the old broad documentary P0: root/module status, M6 history, CI description, ADR 0028/0029 historical wording, m1 and #733 integration. Remaining items are narrow:

| Location | Current issue | Treatment |
|---|---|---|
| Previous audit section 15.2 | Still says addressed in draft #734, not integrated | Historical pre-merge sentence; current audit/PR metadata supersede it. Do not reopen completed technical work |
| `docs/readside/traceability.md` opening Status / Quick path | Global statements exclude accepted U6 atomic scope while the U6 row includes it | Documentary contradiction; reconcile acceptance summary against ADR 0027 without accepting further decisions |
| Same file, old checked SHAs and `(to add)` column | Explicitly scoped historical checks and proposal column | Not automatically stale; no new check at the old pin is invented |
| `docs/adr/0027-read-side-projection-checkpoint-and-rebuild.md`, implementation planning rows | Says no crash test exists and both checkpoints have unit-only validation, while later records show partial L1/L2 | Scope as historical planning or link current evidence; do not claim the entire matrix is complete |
| `docs/readside/u6-atomic-mode-design.md`, pending row | Says nothing from listed refinements is implemented, while section 16.2/code show existing refinements and R6 | Clarify status/implementation separately; no collective R1-R6 acceptance gate |
| #733 body | Says no independent review although later comments record two summaries | Comments provide the later evidence; stale body does not erase them |

Remote CI observations: no workflow run was returned for audited squash `e75742e` at lookup. Latest CI run inspected was [40014802400](https://github.com/getsyntegrity/gopekko/actions/runs/40014802400), on `e5a6714`: job `go` reports failure, zero steps, start 19:52:16Z/end 19:52:18Z. The billing explanation is recorded by the sessions/PRs; the connector rejected the annotations endpoint, so that reason was not freshly verified here. GitHub's job conclusion is failure; code quality was not evaluated. Neither a passed CI nor a demonstrated functional test failure follows from it.

## 7. Priorities, dependencies and next work

No new P0 production defect is established by this static delta audit. This is a scoped finding, not a general correctness guarantee. Do not keep resolved documentation or M6 chronology as a global P0.

| Priority | Work | Existing tracker / dependency |
|---|---|---|
| P1 | Define the production Source scope, then deliver journal-to-projection use outside test fixtures | #412/#415; any multi-ID/vector choice needs the Proposed ADR 0026 decisions; avoid duplicating completed single-ID tests |
| P1 | Event/snapshot evolution and upcasters | #407 -> #408/#409, epic #388; design acceptance before implementation |
| P1 | Make dependency validation cover integration-tagged imports; build a real capability inventory for the parity guard | No dedicated issue identified by open-title search; guard slices B-E depend on the proposal's inventory/decision gates |
| P2 | Add real-engine SQLite Delete policy-refusal detector (G5 residual) | No dedicated open title found; shared accepted SQLite policy; no new Source or U7/U8 decision needed |
| P2 | Non-atomic Run on PostgreSQL | #414/#415 are related, not exact stand-alone coverage trackers; check ADR 0029 D3 authorization before choosing/running an engine scenario |
| P2 | 40P01, RK3, PG L3 and real-engine rollback deadline | Unshown; experiments must state feasibility and applicable D3 scope first |
| P2 | SQLite real COMMIT failure/interruption; 40003 on a server | No established mechanism demonstrated here; classification-only evidence is a valid explicit limit |
| P2 | Actor fidelity, T6 and campaign freshness; restore remote CI execution | #461/#563 and actor issues above; new actor campaign is a separate authorized execution, billing is an account prerequisite |
| P2 | Decide R6; implement U7/U8 when their designs are accepted | R6 is independent; U7/U8 related to #416 and ADR 0026/0027 choices |
| P2 | Reconcile the narrow documentary contradictions in section 6 | Text-only; no engine rerun required |

**Next small technical delivery recommended:** the G5 residual. Extend the real SQLite policy matrix to call Delete on populated data under a rejected pragma set, assert ErrNotDurable and unchanged events/highest via independent SQL, then show a targeted policy-bypass mutant is detected. This is a proposed next task, not executed or authorized by this audit. It fits the current policy without reopening Source/R6/U7/U8. Production Source design remains the higher-value capability work after its scope is chosen.

Power loss (L4), broader server-failure guarantees, upstream recovery after lost ACK, several IDs/rebuild and arbitrary kill timing remain unproved. No proposed experiment in this document establishes a feasible fault mechanism or a guarantee in advance.

## 8. Audit verification and limits

- Open issues were fully paginated through GitHub REST: pages of 100, 100, 100, 67, then 0 entries; 367 open issues and no open PRs at that lookup. Titles were filtered for the named capabilities and known tracker bodies read. No closed-issue body search was repeated. "No dedicated title found" is limited to that search and is not proof that no issue exists.
- PR metadata and issue comments of #730-#734 were reread. Raw reviewer reports/logs were not recovered. Old uncorroborated full-suite reports remain uncorroborated.
- Git tree comparisons were performed by path/blob identity, not by three-dot diff file counts alone. This avoids confusing a diverged squash history with unequal final code.
- Source and citation checks for this document: repository paths were checked against the immutable tree; Markdown tables checked for consistent columns; whitespace/conflict markers and the 400-line PR budget checked locally. These are document checks, not behavior tests.
- `check_deps.sh`, ADR signature compilation and the judge matrix checker were not executed in this partial source snapshot. This audit changes only one Markdown file and reports no new pass for those commands. Relevant script behavior was read where discussed.
- A final develop-ref lookup is recorded in the PR body. Audit findings remain attached to the SHA in the first paragraph even if another session advances develop. No merge, issue closure, branch deletion, Proposed acceptance or production capability implementation is part of this delivery.
