# Block 1: GOV-48 / DTH-02 reconciliation, #1208 against develop, inventory queue

Date: 2026-10-26. Scope: only what the Owner asked for in the "first deliverable" of the block-by-block review of #1208. This is not a new general audit and does not restart the inventory.

Baselines:

| Item | Value |
|---|---|
| goPekko `origin/develop` | `13fb9953879462adba4218114d6615dc3d3156b9` (contains #1207 at `cad059c` and #1011 at `13fb995`) |
| Apache Pekko | v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`upstream/pekko`, read-only) |
| #1209 (draft) | branch `docs/gov-48-dth-02-reconciliation`, head `7b971c0` |
| #1208 (draft) | branch `docs/actor-typed-design-closure` |

Open PRs that matter here: #1208 and #1209 (documentation, draft), #1198 (audit), #1014 (ADR 0032, not integrated), #1013 (SYS-03 code, draft). Code present only in an open PR is not counted as implemented anywhere in this file.

## 1. Verification of #1209 against the GOV-48 decision

Method: an independent verifier, in a separate context, read `docs/parity/gov-48-dth-02-reconciliation-2026-10-26.md` against the Owner's words, `cad059c`/`13fb995` code and Pekko v1.7.1, and tried to refute each row. Its report is not committed; its findings were applied in `7b971c0`.

| Finding | Result |
|---|---|
| Owner decision quoted verbatim, points 1 to 3 and the alternative-evidence plan | Correct. "Esto no declara paridad ejecutada con Pekko" is preserved |
| Criterion 3 row said a System terminating under a foreign watcher was covered by `watch_cross_system_test.go:80` | **Wrong.** That test terminates the owning System before the watch is made. No committed test terminates a System while a foreign watcher is linked to a live subject. Corrected; T6 is a real gap |
| "14 test functions" | **Wrong.** 16 functions ran (14 `-run` patterns). Names listed in section 8 of #1209 |
| Window analysis (late withdrawal erasing a later registration) | Re-derived independently; no window found. Three removal paths were missing from the table and were added; the assumption that one watcher's `Context` calls are serialized by its activation (`watch.go:103-106`) is now stated |
| `Unwatch` timing | Pekko removes the watcher at the subject asynchronously (`DeathWatch.scala:70`, `remWatcher` `:199-215`); Go synchronously (`watch.go:262`). Already recorded as G6 in the merged `cross-system-watch-design.md:64`. No new Owner decision is indicated |
| Oracle cells of T1, T3, T4 | Reworded to "applicability pending determination" (ADR 0033 D13); none says "no counterpart" or "pending approval" |

Limits: the verifier ran the tests at `13fb995` (17 functions, `ok 1.834s`); the author ran 16 at `cad059c`-identical `pekko-actor`. Neither run is the full suite or the local gates. Reading is not execution, and no Pekko program was run.

Issue text: #1195 and #827 carry appended update blocks only; labels and original text untouched. The "14 test functions" sentence in the #827 block was corrected to 16 and the G6 note and the T6 gap were added.

## 2. DTH-02 (#827): residual scope and readiness

Source: #1209 sections 3, 6, 7, 9; code at `13fb995`.

| Criterion of #827 | Already in develop | Tests missing | Production change |
|---|---|---|---|
| 1, watcher stops | Yes, `acceptStop` plus `releaseForeign` plus post-link re-check (`stop.go:50-70`, `watch.go:204-206, 438-463`) | T2 | None indicated by reading |
| 1, subject stops | Yes, generation discard (`watch.go:274, 352`) | T1 (cross-System) | None indicated |
| 2 | Yes, in code (`watch.go:242-264, 477-499`) | T5 (cross-System, already queued) | None indicated |
| 3 | Partly | T6 (named, finite interleavings, plus owner System already terminated), T7 | None indicated; end-to-end shutdown stays NOT PROVEN by the lock analysis |
| Window "late withdrawal vs later registration" | Not found in the code, by reading | T3, T4 | None indicated |

Readiness: **ready to be planned as a tests-only story, conditional on one Owner confirmation** (wording of criteria 1 and 2, see section 6 of this file). The label `status:blocked` is unchanged; changing it is the Owner's decision. A failing test would add a production change at that point. The size of T1 to T7 against the 400-line heuristic is not estimated.

Oracle status per ADR 0033 D13: T2, T3, T4, T5, T6 are "applicability pending determination"; T5 has a candidate oracle (`DeathWatchSpec.scala:268-291`, read, not executed); the H5b cross-System probe is not compiled or executed, so it is not an oracle today. Go-deterministic `-race` tests are the Owner-approved alternative-evidence plan; no result is approved and no parity is declared.

## 3. Statements of #1208 corrected

| Statement | Correction |
|---|---|
| "No slice can start" (README line 3) | Narrowed to "none of the nine slice candidates evaluated in section 7" |
| #1008 IDN-14 "implemented in open PR #1011" | #1011 is merged (`13fb995`); issue #1008 remains open and `ready` |
| #827 "waits for GOV-48" | GOV-48 was decided on 2026-10-26; residual is tests plus one wording confirmation |
| "Ten ACTINV rows with no evidence path" | Wrong for nine (evidence in `docs/parity/actor-local-fidelity-audit-2026-10-22.md` and `actor-local-closure-audit-2026-10-23.md`); `ACTINV-091` (serializer registry) is in neither |
| "29 included without story" | 20 have open stories (extension), 3 only closed stories, 6 none (preStart/postStop, unbounded FIFO mailbox, serializers, 3 internal rows). The ten ids fall inside 18 of the 29 rows, so the counts overlap and do not add |

## 4. #1208 against develop `13fb995`

| State | Items |
|---|---|
| Integrated in develop | Cross-System watch (DTH-01, #1207, `cad059c`); `ActorPath` value (IDN-14, #1011, `13fb995`); GOV-15 option B amendments of ADR 0010, 0015, 0017, 0024; GOV-51 |
| Code only in an open PR | SYS-03 (#1013); ADR 0032 text (#1014). Neither is counted as implemented |
| Pending | GOV-48 residual tests (DTH-02); wording confirmation; the inventory queue below; the 56 PARTIAL stories |

## 5. Prioritized queue for the inventory gaps

Counts verified with a rewritten parser: 323 of the 338 rows parsed (14 INV-A rows are in tables without a scope column); 86 unclassified, 29 included-without-story, ten ACTINV rows. Rows were read one by one; keyword overlap alone was not accepted as overlap.

Structural finding: GOV-19 (#1044) registers only exclusions named in ADR 0033 D8/D9/D12, plan 2.3 and proposals GOV-04 to GOV-14. JDG-01 (#1026), JDG-04 (#1034) and CLS-02 (#1045) are the existing vehicles for per-row evidence, oracle and closure. None of them classifies the 86 rows. JDG-01 R2 asks for an owning story key per row, impossible for retained rows without a story. The closed exclusion-reason list has no value for internal (non-public) types.

| Priority | Block | Rows | Action | New stories? |
|---|---|---|---|---|
| Q1 | C5T: Typed public API likely covered by existing stories (ExtensibleBehavior to #809, ActorSystem accessors to #1176/#1177, #994, #958/#982, ...) | 10 | Link each row to its story and confirm; unblocks Typed stories | No |
| Q2 | C1: internal/implementation types (SystemMessage, FunctionRef, scheduler internals, queues, ...) | 20 | One Owner decision on how internal types are inventoried clears the block | No |
| Q3 | C2: Java/JVM-only surface (AbstractFSM, japi, ThreadPoolConfig, AffinityPool, JavaLogger, ...) | 25 | One proposal for java-api/jvm-only families; #1125/#1126 cover only DynamicAccess/AbstractActor | One proposal at most, after Owner approval |
| Q4 | C3: cluster/remote markers (AddressTerminatedTopic, MiscMessageSerializer, ServiceKeySerializer, DeliverySerializable) | 4 | Classify as `cluster-or-remote` or retain | No |
| Q5 | C4: covered by an existing proposal or decision story (DNS #1072/#974/#953, #1190, #1086, #1084, #1079, #792/#869, #1114) | 10 | Link only | No |
| Q6 | C6 generic utility types (Done, NotUsed, PekkoException family, PekkoVersion, Timeout, Version) and C5C Classic partially covered | 6 + 11 | Read and place | No |

Duplicate-story check: none of these blocks needs a new story at this point; overlaps with existing stories are listed above. The 29 included-without-story rows overlap the 86 only through shared ACTINV ids and are not additive.

## 6. Next audit block and strictly necessary Owner decisions

**Next block (Block 2):** the PARTIAL stories that gate code, with the Q1 rows folded in: #809 (P0, ready, 6 consumers), #1096 (P0, 5), #956 (P0, 4), #794 (P1, 4), #994, #989, #767 (6 consumers). Reason: they unblock the most downstream stories and several Typed rows depend on them.

Owner decisions strictly needed:

1. **DTH-02 wording** (needed for Block 1 readiness): confirm restating criterion 1 as two statements (generation for stale-notification discard, watcher ID for release at stop) and criterion 2 in watcher-local terms. The original text stays in #827.
2. **Internal types** (needed for Q2): how non-public Pekko types are inventoried, since the closed reason list has no value for them.

Not needed for Block 2: classification vehicle for the 86 rows, JDG-01's owning story key for rows without story (needed before Q3/Q6 are closed, not before Block 2).

## 7. Not verified

- No Pekko program was run; specs were read, not executed.
- The full suite and local gates were not run.
- The 86/29/ten counts come from a parser over markdown tables; 14 INV-A rows lack a scope column and were placed by reading.
- Category assignment (C1 to C6) is the Architect's reading and is not an Owner decision.
