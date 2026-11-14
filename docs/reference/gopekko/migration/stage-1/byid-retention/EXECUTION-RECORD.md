# EXECUTION RECORD: retirement of terminated actors from `System.byID` (ADR 0024, T2-T5; issue #541)

Status: delivered as a draft PR stacked on #542, **not merged**; #541 stays open; T6 (field release) is not started. This record states what the kit executed and what it measured. It claims no Pekko parity beyond the listed cases, and no memory release: no heap or GC measurement was made.

## 1. Provenance and authority
- Package commit `7dce9cfc8b77d39f8e66790c1a58d9772f7a1696` on `feat/actor-byid-retention`. The Owner approved the package on 2026-10-20 ("Apruebo los hashes, ejecutá la unidad", and the instruction naming commit `7dce9cf`, the 9 hashes of `RUNBOOK.md` section 9 and `RESUME.md`, one Sonnet run of at most 6 USD and 30 minutes, at most one Opus review of at most 3 USD and 12 minutes, crossing the 450,000-token evaluation point up to 560,000, and one draft PR over 400 lines).
- Before launching, the full SHA-256 of the 8 files in `RUNBOOK.md` section 9 and of `RUNBOOK.md` were recomputed from the working tree at `7dce9cf` (clean tree) and matched `RUNBOOK.md` section 9 and `RESUME.md` exactly. No delta.
- Kit `migration-kit` at `cf91c9d5068d9aaf95a36164169f08c3e636c909`; `upstream/pekko` at `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`v1.7.1`), initialized in the worktree for the run. Loading the kit's rules is not what is claimed here: the launcher, the hash-checked rulebook and permission templates, the one-review lock and the two real `claude -p` processes below are.

## 2. What the kit executed (measured)
Both agents were launched with `run-agent.sh` (real `claude -p` processes rooted in the worktree; permissions only from the rendered templates; USD cap and wall clock imposed by the script). Figures are read from each agent's JSON output, kept outside the repository.

| | Implementer | Review |
|---|---|---|
| Role and model | `implementer`, `claude-sonnet-5-5` | `review`, Opus |
| Started (UTC) | 2026-10-20 22:51:11 | 2026-10-20 23:12:25 |
| Wall time (`duration_ms`) | 547,895 ms (9.1 min), limit 30 | 131,895 ms (2.2 min), limit 12 |
| Cost (`total_cost_usd`) | 1.9201571 USD, cap 6 | 0.9079624 USD, cap 3 |
| Turns | 96 | 20 |
| Input tokens | 104 | 36 |
| Cache-creation tokens | 157,285 | 64,324 |
| Output tokens | 75,066 (23,590 thinking) | 11,980 |
| **Tokens by the RUNBOOK convention** (input + output + cache creation) | **232,455** | **76,340** |
| Cache-read tokens (reported apart, not in the convention) | 5,401,491 | 768,132 |
| Exit | 0, `is_error` false | 0, `is_error` false |

- **Measured total of the two agents: 308,795 tokens, 2.828 USD, 11.3 minutes of agent time** (limit 45).
- **Not measured:** the coordinator's own session (reading, gates, mutants, corrections, documentation). By the RUNBOOK convention it is an estimate: reserve 160,000 plus one correction round of 30,000, so about **498,795 tokens in all**, above the 450,000 evaluation point and below the 560,000 ceiling. That is an estimate, not a measurement. The Owner authorized crossing 450,000.
- Preparation delegations before the package was approved (read-only, outside the two agents above, declared in `RUNBOOK.md` section 2): a Pekko-side reader (116,054 subagent tokens as reported by the harness) and a consumer review (166,389). They are not counted in the 308,795.
- Limits the script enforced: USD cap, wall clock, read-only review, `review.lock`. Limits it does not enforce: the token ceilings (450,000 and 560,000). Those were controlled by the coordinator reading the JSON after each agent, and were announced to the Owner before launch.
- No automatic relaunch happened: one implementer run and one review run.

## 3. Flow, as run
1. Baseline at `7dce9cf`: `go test -race -count=1 ./pekko-actor/...` green; coverage 98.3%; `golangci-lint` 2.14.0 `0 issues` (the installed 2.13.1 cannot load packages with go1.27.2, so the pinned v2.14.0 was built outside the repository).
2. Implementer (one run). It wrote the characterization file first and reported it passed before any production edit, implemented T3-T5, and reported `BLOCKED(surface)` for `guardian_goside_internal_test.go` (its `TestTopLevelActorsCountUnderUser` expected registry growth after stops). Its wrapper run stopped there because of `set -e`.
3. Scope control: only the 15 modified and 3 new files of `RUNBOOK.md` section 6 changed; the other 84 `.go` files of `pekko-actor/actor` keep their hash; no `judge/`, docs, `go.mod` or `go.sum` change.
4. T2 on the base: `registry_characterization_internal_test.go` copied into a scratch worktree of `7dce9cf` and run with `-race`: 10 tests, all pass on the unchanged base.
5. Coordinator corrections before the review: the blocked test adapted by the coordinator (outside the RULEBOOK §5 list, hence a deviation, see 6); a cyclomatic-complexity violation (12 > 10) in a new test fixed by splitting it; a new test for `ChildFailure.Child` added after mutant M-j survived.
6. One Opus review: **0 BLOCKER, 5 DEFECT, 4 INFO** (section 5).
7. One correction batch for the DEFECTs that concern tests (section 5). No production code changed after the review.
8. Documentation by the coordinator.

## 4. Verification (run by the coordinator, after the correction batch)
- `gofmt -l`: clean. `go vet ./pekko-actor/...`: clean. `go mod tidy -diff` in `pekko-actor`: clean. `golangci-lint` v2.14.0 on `pekko-actor`: `0 issues` (includes gocyclo at 10).
- `go test -race -count=1` of `pekko-actor`, `pekko-persistence`, `pekko-persistence-postgres`, `pekko-serialization`, `pekko-observability`: all pass. `go test -race -count=10 ./pekko-actor/actor`: pass. The `Retention` tests passed 20 consecutive `-race` runs before the last two tests were added, and those two (with the hierarchy tests) passed 20 more.
- Coverage of `pekko-actor`: 98.4% (base 98.3%); every function changed by the unit is at 100% except `newGuardian` (75%: the line added there executes; the uncovered part is a branch that existed before, for a guardian born stopping).
- `scripts/check_deps.sh`: ok. `.github/scripts/check-headers.sh`: 6 errors, **the same 6 at the base**, all in `pekko-persistence`, none in `pekko-actor`; not caused by this unit and not fixed here.
- Not run, by instruction: the Go judge, any mutation campaign, any workflow, any evidence tag. The judge was read, not run: by that reading it does not read the registry, and the retirement comes after the `terminated` and `done` events of `finalize`.

## 5. Directed mutants (scratch copy outside the repository, not committed)
All applied one at a time to the final code and run against the whole `pekko-actor/actor` package. **11 of 11 detected.** `RULEBOOK.md` section 8 defines only M-a to M-h; **M-i, M-j and M-k were added by the coordinator** (M-j after it survived the first run, M-k with the correction batch), and M-f was run as a variant (below). The mutants were re-run on the exact HEAD `c637b58` for the final review of the PR; the outputs were saved outside the repository (`mutants-head.txt`, `mutant-k-head.txt`) and not committed.

| Mutant | Detected by (first names; count) |
|---|---|
| M-a no retirement | `TestRepeatedCreateAndStopReturnToBaseline`, `TestRetentionAskTargetUsesTheProvenanceMark`, `TestRetentionGrowthLeavesOnlyTheGuardians` (12) |
| M-b retire before the watchers are told | `TestDescendantsFinalizeBeforeTheirAncestors`, `TestDoneOfAParentImpliesDoneOfEveryDescendant`, `TestARandomTreeStopsInTheRightOrder` (11, existing hierarchy tests) |
| M-c retire guardians | `TestGuardiansImmediateClose`, `TestRetentionGuardiansStayRegisteredAfterShutdown` (3) |
| M-d `askTarget` back to the registry | `TestAskTargetValidation`, `TestAskTerminated_K1_CompletedTermination` (16) |
| M-e `Done` fallback without the owner check | `TestRetentionDoneOfAHandleOfAnotherSystemIsUnknown`, `TestNamesAndHandles` (4) |
| M-f `Done` unknown without the error | `TestRetentionDoneOfRetiredAndUnissuedHandles`, `TestNamesAndHandles` (5) |
| M-g retire on a non-member signal | `TestRetentionStaleDuplicateAndForeignSignalsRetireNothingElse` (1) |
| M-h provenance mark not set | the `TestAskCancel_*` family and `TestRetentionAskTargetUsesTheProvenanceMark` (8) |
| M-i `idHandle` without `done` | `TestRetentionCharacterizationDoneOfAnIdentityHandle`, `TestRetentionHandlesFromChildAndParentCarryTheirState` (8) |
| M-j `ChildFailure.Child` as a bare handle | `TestRetentionChildFailureHandleCarriesTheTerminalState` (1) after it **survived the first run** and a test was added |
| M-k `addWatcher` accepts a closed cell | `TestRetentionWatchLinksWhileTheSubjectIsRetired` (1; added with the correction batch) |

Deviation from `RULEBOOK.md` section 8: M-f as written ("`Done` fallback accepting a user-implemented handle") cannot be built, because a user type cannot implement the unexported `terminal()` method of the interface. The mutant that was run is the variant above (the final `ErrUnknownHandle` becomes `nil`). A mutant that drops the `terminalHandle` assertion would have no handle to travel through, so this record does not claim it as detected.

## 6. Review findings and what was done
| # | Class | Finding | Action |
|---|---|---|---|
| 1 | DEFECT | `guardian_goside_internal_test.go` edited outside the RULEBOOK list | **Accepted as a deviation.** The implementer correctly reported `BLOCKED(surface)`; the coordinator made the edit (replaces `ids+i+1` with `ids+1`, keeps the "never reused" check, adds the return to baseline). The RULEBOOK §7 list missed this test because the consumer review had classified it as unchanged |
| 2 | DEFECT | Exact ID counts in `hierarchy_concurrent_internal_test.go` weakened to `hcAtMost`, no lower bound | **Fixed:** `hcAccountedFor` checks that every accepted spawn is registered or finished, at the root-gate sites. The upper bound stays |
| 3 | DEFECT | `registered >= 1 && registered <= 2` made a timing window part of the assertion | **Fixed:** the child's completion signal is held with `completionPoint`; the test asserts exactly 2, delivers `complete`, and asserts exactly 1 |
| 4 | DEFECT | R9 not forced through the seams; no `linkPoint` test | **Fixed:** `TestRetentionWatchLinksWhileTheSubjectIsRetired` parks a `Watch` between the live resolve and the link, retires the subject, and checks one `Terminated` and no link (mutant M-k) |
| 5 | DEFECT | M-f cannot be killed as written | **Recorded** in section 5 |
| 6 | INFO | `ChildFailure.Child` of a stale child is a bare `idHandle`, no longer `==` to `identity()` | Recorded; that is M4 as specified |
| 7 | INFO | A ghost cell or user handle carrying a **live** ID still gets `(done, nil)` through the registry | Recorded; base behavior (C9, I9), not new authority. A typed-nil `*ref` still panics in `lookup`, as at the base |
| 8 | INFO | The guardian flag is stored after `commit` publishes the guardian | Recorded; safe because no stop can reach a guardian before `newGuardian` returns; the reasoning is in a comment, not enforced |

The review did not read `registry_retention_api_test.go`, ADR 0024, or lines 1-149 and 265-end of `registry_characterization_internal_test.go`, and could not run the tests. The correction batch and the coordinator's other edits (the guardian test, the complexity split, the `ChildFailure.Child` test, the documentation) were **not re-reviewed**.

## 7. Traceability with Pekko (`manifest.tsv`, `RULEBOOK.md` section 2)
Compared by reading the pinned tree, then by the Go tests: termination order and field clearing `FaultHandling.scala:247-276`, `ActorCell.scala:718-726`; dead-letter mailbox `Mailboxes.scala:54-72`; watching a dead actor `DeathWatchSpec.scala:145-152`; name reuse `ActorSelectionSpec.scala:124-148`, `DeathWatchSpec.scala:248-255`. These are *coincidencia observada por lectura; validación comparativa pendiente*: no Pekko test was run and no JVM was started. The registry and its retirement, the terminal state in the handles, and `Done` for an ID-only handle are Go designs with no Pekko source (RULEBOOK 10).

## 8. Differences from the accepted design (ADR 0024)
- None in D1-D5. Implementation details added by the coordinator, **not accepted as ADR decisions**: the owner check and the registering-System mark for `Done` and `askTarget`; a nil `done` is unknown. They follow the Owner's implementation instruction of 2026-10-20 (a foreign, invalid or ID-only handle must not gain authority by knowing an ID). At the time of the run and of the reviews they were not accepted as ADR text. **The Owner accepted them as ADR text afterwards, on 2026-10-20** (section 11); ADR 0024 records it.
- Existing tests adapted beyond the RULEBOOK list: `guardian_goside_internal_test.go` (above).
- Assertions that are weaker than before, by necessity: where the retirement is asynchronous to `Done` and the test cannot hold the signal, exact counts of registered IDs became upper bounds in **four** sites of `hierarchy_concurrent_internal_test.go` (`raceRootGate`, `raceChildSpawns`, `hcSpawnsInFlightAtTheRootGate`, and `assertChildStoppedOnceWithItsParent`, which has the bound only). Two of them add `hcAccountedFor` ("registered or finished") and one adds a `lookup` per held child. That check cannot see an actor retired early after it already finished; the born-stopping test of section 10 covers that case deterministically.

## 9. Pending and limits
- T6 (field release) not started; heap not measured; `check-headers` has 6 errors that exist at the base; the Go judge, Linux evidence and the module-close gates of #461 are not part of this unit.
- No new contract decision appeared. Nothing was merged, closed or tagged.
- The owner-check and provenance-mark details of `Done` and `askTarget` were not accepted as ADR text during the run; they were accepted afterwards (section 11).

## 10. Independent review of the final candidate, and the corrective commit
- **Review of `c637b58`** (requested by the Owner, 2026-10-20). Two independent read-only reviewers with assigned roles, an architect and a tech lead, run as general-purpose agents because no agent type with those names exists in this environment. Neither received the conclusions of the earlier review. Both: **APPROVE WITH CONDITIONS, no blocker.** Harness-reported subagent tokens: 156,513 (architect) and 185,146 (tech lead). That is the harness's own figure, not broken down, not comparable with the convention of section 2 and not counted in the 308,795.
- **Findings and what the corrective commit did.**

| Source | Finding | Action |
|---|---|---|
| TL F1, test | `TestRetentionRealExecutorKeepsDoneAndAskConsistent` skipped a failed `Spawn` and counted nothing, so it could pass with no spawn at all (mutant: `reserve` fails for those names; survived) | **Fixed:** a failed `Spawn` is a failure and the test asserts 8 x 25 successful spawns. The mutant is now detected by assertion |
| TL F2 and architect F4, test | The bounds of `hierarchy_concurrent_internal_test.go` cannot see an actor retired early after it already finished (born-stopping spawn) | **Fixed in part:** `TestRetentionBornStoppingChildStaysRegisteredUntilTheOwnerCountsIt` pins it deterministically (`commitPoint` closes the gate, `completionPoint` holds the signal). The bounds stay as they are and section 8 states what they cannot see. Mutants B4 (retire at commit when the gate is closed) and B1 (retire at finalize) are detected by assertion |
| Architect F1, doc | README and the `Done` comment said `ChildFailure.Child` keeps its result; not for a child already retired | **Fixed** (README, comment of `System.Done`; comment only, no code change) |
| Architect F2, doc | stale "never pruned" and "nothing was executed" texts | **Fixed:** ADR 0024 (reading convention, judge line, ordering table, "Not verified" heading), ADR 0014 line 53, comment of `ask_terminated_internal_test.go` |
| TL F3, doc | the record presented the owner and provenance details as accepted by the Owner | **Fixed** in section 8 and in ADR 0024; the Owner's confirmation came later (section 11) |
| TL F4, doc | "11 of 11" without saying that RULEBOOK section 8 defines eight | **Fixed** in section 5 |
| Mine, doc | "every changed function at 100%" was false (`newGuardian` 75%) | **Fixed** in section 4 |
| TL "three sites" | the weakened sites are four | **Fixed** in section 8 |
| Architect F3 | `Done` of a typed-nil `*ref` panics in `lookup` | **Not changed:** it is the base behavior, outside this unit |
| TL F6, F7 | the API test of the parent covers `/user` only and two of its tests do not call `Shutdown`; a `FailedMessage` kept after a stale child failure holds a bare handle | **Not changed;** recorded, scenario of F7 not constructed |
| Architect F5 | validation limits (no heap, no judge, one platform) | Unchanged, section 9 |

- **Evidence of the corrective commit** (local, the tree before commit; production code differs from `c637b58` only in the comment of `System.Done`): `gofmt`, `go vet`, `go mod tidy -diff` clean; `golangci-lint` v2.14.0 `0 issues`; `go test -race -count=1` of `pekko-actor`, `pekko-persistence`, `pekko-persistence-postgres`, `pekko-serialization` and `pekko-observability`: 13 packages ok; `go test -race -count=10 ./pekko-actor/actor`: ok; the two new tests passed 30 consecutive `-race` runs each; coverage 98.4%; `scripts/check_deps.sh` ok. The 11 mutants of section 5 were re-run on this tree and all are detected; outputs saved outside the repository (`mutants-after-fix.txt`, `mutant-k-after-fix.txt`, `mutants-fix.txt` for V1, B4 and B1).
- **Independent review of the corrective commit `e8a3e87`** (a fresh reviewer agent, requested by the Owner; harness-reported subagent tokens 125,418, same caveat as above): **APPROVE WITH CONDITIONS, no blocker.** It re-ran its own mutants against the two new tests (five against the real-executor test and five against the born-stopping test, all detected by assertion, none by hang), ran both tests with `-race -count=100`, and checked that the only non-test `.go` change is the comment of `System.Done`. Its one defect (D-1: the "Feature" section of ADR 0024 described the base in the present tense) was fixed in `e443dcb`, a documentation-only commit of two lines checked only by its diff and by GitHub CI. Its INFO items (D-2: the mutant outputs and scripts live outside the repository; D-3: the `R9` label) are unchanged. Section 9 still applies.

## 11. Acceptance of the details and authorization of the integration
- **Details of the owner check and the provenance mark.** On 2026-10-20, after the reviews above, the Owner accepted them as text of ADR 0024 ("Acepto como texto del ADR 0024 los detalles implementados de chequeo de dueño y marca de procedencia de Done y askTarget"), in an instruction in the work thread, not a GitHub comment. ADR 0024 records it. It covers the details as implemented and nothing else.
- **Integration.** In the same instruction the Owner authorized integrating #542 and #543 by a prepared sequence, from `develop` `cd558941dc2b4789a7103c94192af734220b22e0`, #542 `7f52bb5fc3d40a04b8f3465c32629dd6ce558e21` and #543 `e443dcb243953f2c8dc326a4b77728fd2396c032`: squash of #542 protected by SHA, #543 retargeted to `develop`, `rebase --onto`, `push --force-with-lease` against the verified remote head (never `--force`), local checks and a green CI on the new head before merging #543 by squash protected by SHA, and a check of the resulting tree on `develop`. Not authorized: judge or campaigns, tags, closing issues, deleting branches, T6.
- The SHAs that the integration produces do not exist when this file is written; they are reported in PR #543 and in issue #541.
