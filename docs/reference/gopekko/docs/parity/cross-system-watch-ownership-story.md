# Story: design of subject ownership, cleanup and lock order for cross-System watch (GOV-51, issue #1200)

**Status: APPROVED AS DESIGN by the Owner on 2026-10-25 (work thread); not implemented.** Published as `GOV-51`, [#1200](https://github.com/getsyntegrity/gopekko/issues/1200), under epic [#1016](https://github.com/getsyntegrity/gopekko/issues/1016), `status:blocked`. Priority `prio:P2` is maintained by the Owner (decision of 2026-10-25, given in the work thread). This story is the **single source** of the scope and acceptance criteria; the GOV-51 issue body carries the same numbered criteria. Independent design story (Owner decision 7). Reserves no ADR number. Evidence rows (A#, G#, Q#, P#) are in the [design document](cross-system-watch-design.md); the lock inventory, call-site tables, verdicts P3, P4, P7, P8 and executed evidence are in the [lock analysis](cross-system-watch-lock-analysis.md); the carrier B demonstration (items 1-5, the 12-row interleaving table, executor assumptions, inline-executor compatibility, what could not be read) is in the [carrier B demonstration](cross-system-watch-carrier-b-demonstration.md); the amendment proposals are [ADR 0010](cross-system-watch-adr-0010-amendment-proposal.md), [ADR 0015](cross-system-watch-adr-0015-amendment-proposal.md), [ADR 0024](cross-system-watch-adr-0024-amendment-proposal.md) and the [ADR 0017 and README proposal](cross-system-watch-adr-0017-readme-amendment-proposal.md) (ADR 0017 and `pekko-actor/README.md`).

## Context and Owner decisions to honor

GOV-15 option (b) ([#1065](https://github.com/getsyntegrity/gopekko/issues/1065)): `Watch`, `WatchWith` and `Unwatch` accept a native handle of another local System. Read, not executed: `resolve` rejects a foreign handle today (`pekko-actor/actor/watch.go:117-123`); `leaveWatchees` withdraws by ID in the watcher's own registry (`watch.go:407-413`). **Provenance:** option (b) is recorded in #1065; the decisions below were given by the Owner on 2026-10-25 in the work thread, **not in a GitHub comment** (same rule as ADR 0024, lines 147-149).

Decided: 1 valid handle (Ask endpoint excluded; the owner check is technical, not an API). 2 an absent actor yields `Terminated`/the `WatchWith` message for Watch and WatchWith; **Q1 (a): `Unwatch` is silent**. 3 cross-System removal progresses without waiting for the subject to terminate, under explicit executor assumptions; retention is stated by holder and duration, never as a global "no retention"; **Q3: eventual release** following Pekko, same-System behavior unchanged. 4 amend the ADR 0015 rule now, message and comments later. 5 ID uniqueness is an invariant in force. 6 the Go-only evidence plan is approved as a plan only. 7 this story is separate. **Q2:** the Owner accepts incorporating differential evidence against Pekko after verifying the tooling; the probe `H5bCrossWatch.scala` is **written, not compiled, not executed**; the oracle is **NOT available and NOT executed**. **Q4:** inline executors are **not excluded for now** (direction accepted; the design is **not proven deadlock-free**); the two proposed tests are included. **Carrier:** B (closure on the subject System's executor) is the Owner's preferred alternative, **accepted as design on 2026-10-25** with the Q5 defer. **Q5 (direction accepted, 2026-10-25, work thread):** the foreign hand-off in `acceptStop` is registered with `defer` right after the `c.mu` section, to cover a panic and a Goexit under an inline `A.exec`; a design direction, not implemented; its tests are defined, not written, not executed.

State of the evidence (three classes, never merged). **READ:** the lock inventory (19 `System.mu` sites, 34 `cell.mu` sites, `Hub.mu`), the lock-order graph, all 16 `acceptStop` and all `Execute` call sites, and the five items of the carrier B demonstration were read; no counterexample to lock safety was found by reading, which is not positive evidence of absence. The demonstration is class READ under executor assumptions E1-E10 (items 1-4; item 5 for all twelve interleaving rows, derived by reading for the designed placement under a concurrent and an inline `A.exec`), not executed and not implemented, and found **one defect of placement**: in the designed code the foreign hand-offs are issued last in `acceptStop`, after `emit` and after `Execute(c.drain)` (today's code has `leaveWatchees` at `stop.go:56` and `Execute(c.drain)` at `stop.go:59`). Consequence: the release is handed off, not completed, when `acceptStop` returns, and can arrive after `Done(W)` closed; a test must not assert that `S.watchers` has no entry for W at the moment `Done(W)` closes, only after the subject System's executor ran the closure or after the subject finalized. Inline-executor compatibility of the designed placement is READ for (a) `acceptStop` with the drain run inline, (b) `acceptStop` on the subject's finalize stack, (c) both executors inline, and (d) an executor-internal lock (that is assumption E3), and **NOT PROVEN** for (e), the two planned tests, whose files do not exist. One residual gap (under an inline `A.exec`, a `runtime.Goexit` in a user hook run by the inline `Execute(c.drain)` would skip a plain last-statement foreign hand-off exactly as a panic does; the link would stay until the subject finalizes; absent with the production executor) is **closed in the design by the deferred hand-off** (Q5 direction accepted; READ for locks and order; not implemented; its five tests are not written). **EXECUTED:** two commands run on 2026-10-25 at commit `3b722fdac9197a0fea9365a253acdddf354db3ff` with go1.27.2 darwin/arm64, in `pekko-actor`, both with the race detector: `go test -race -count=1 -timeout 14m ./...` (the per-package results: packages `actor`, `internal/deadletter` and `internal/mailbox` ok) and `go test -race -count=1 -v ./actor` (the counts: 525 top-level tests, 1780 including subtests, 0 failed, 0 skipped). They exercise existing same-System behavior only; the cross-System hand-off is not implemented, so nothing of it was executed; a passing run does not prove lock order; no stress run and no reproduction of any interleaving. A throwaway toy program outside the repository (go1.27.2) was also run; its class is language semantics of a toy program, **not** evidence about goPekko code, and it proves nothing about lock order (design 6.5). **NOT PROVEN:** the items listed in design section 4 and 8.

## Scope (design; no code, no tests)

| # | The design must contain |
|---|---|
| S1 | Per-operation contract (Watch, WatchWith, Unwatch) for live, terminated, terminating-during-registration and absent subjects, with Pekko file:line (design A3-A13); a retired live handle versus a never-registered cell; the three distinct operations (obtain the owning System, ask it whether the actor is active, treat a retired actor as dead) and which of them adds no retention |
| S2 | The owner of a foreign subject kept per registration (`watchReg`, `watch.go:28-34`), `takeOutgoingLocked` returning `(id, owner)` pairs (`watch.go:378-386`), the normative "owner resolvable" sentence, no new exported API, no permanent registry |
| S3 | Eventual release for a foreign subject and unchanged synchronous release for the same System: the two carriers (A: item in the subject's activation; B: closure on the subject System's executor, the Owner's preferred), their comparison, the five-item demonstration of B, the placement requirement (foreign hand-offs last in `acceptStop`), the executor assumptions E1-E10, the retention table by holder and duration (H1-H5), the 12-row interleaving table (derived by reading for the designed placement), the Goexit residual and its closure by the deferred hand-off (Q5 direction accepted), the incompatible-notification table and the guarantee-difference table |
| S4 | Lock order per path P1-P8 and for the new hand-off, each verdict in the classes READ / EXECUTED / NOT PROVEN, with what the reading does not cover; verdicts copied, never upgraded; the lock inventory, call-site tables and seam table in the lock analysis |
| S5 | Reentrancy and inline executors (Q4): findings, the compatibility of the designed placement (a)-(d) READ and (e) NOT PROVEN, the stated limit, the two Q4 tests (reentrancy from the subject's finalization; controlled eventual removal on a controlled executor) and the five Q5 tests of the deferred hand-off, the minimal scope if an exclusion were ever chosen; shutdown acceptance for both Systems; test seams, including the missing subject-side seam |
| S6 | Evidence plan: Go-only scenarios, the Q3 test on a controlled executor, the Q4 inline-executor test, the ID uniqueness test, the proposed Pekko-side scenarios a-e, and the written-but-not-executed probe `migration/stage-1/watch-cross-system-probe/H5bCrossWatch.scala` (running it is not decided) |
| S7 | The ADR impact list (ADR 0033 D5): every ADR or document that states the rejection of a handle of another System, quoted in full with its line, with its impact and proposal status (ADR 0017 and the module README included, with an explicit proposal file, approved as design on 2026-10-25; README not applied), the tests that change, and the deferred error message and comments |

## Acceptance criteria (identical in the GOV-51 issue body)

1. **Contract (S1).** The design states each operation separately for each subject state, with a Pekko file:line and a spec or "no spec found" with its search scope; `Unwatch` of an absent handle is silent (Q1 (a)); it distinguishes obtaining the owning System, asking it whether the actor is active, and treating a retired actor as dead, and states which adds no retention and what the third stores.
2. **Owner (S2).** It states the data a registration keeps for a foreign subject, the `(id, owner)` pairs taken in the same critical section that sets `stopRequested`, the normative sentence with its verified handle producers, and why no API or registry is added.
3. **Eventual release (S3).** It shows that the link is released with no permanent retention (subject alive, subject terminating first, executor never closed, registry retired after finalize, watcher's System shut down, duplicates, the two stop-versus-register windows), that no notification incompatible with the watcher's state results, and the same-System versus cross-System guarantee difference. It states that the removal does not wait for the subject to terminate and progresses under the numbered executor assumptions E1-E10, and it states retention by holder with its duration (H1-H5) without any global "no retention" claim. It presents carriers A and B with the comparison of the carrier B demonstration (sections 4.3 and 4.6), records B as the Owner's preferred alternative, accepted as design on 2026-10-25, records the five-item demonstration as class READ under E1-E10 (no counterexample found by reading; not executed, not implemented; item 5 derived by reading for all twelve interleaving rows of the designed placement, under a concurrent and an inline `A.exec`), and records the placement requirement (in the designed code the foreign hand-offs are issued last in `acceptStop`, after `emit` and after `Execute(c.drain)`, distinguishing today's lines `stop.go:56` and `stop.go:59`). It states the lost ordering: the release is handed off, not completed, when `acceptStop` returns and can arrive after `Done(W)` closed, so a test asserts the removal only after the subject System's executor ran the closure or after the subject finalized. It states the residual of the placement (the foreign hand-off is skipped if `emit` or `Execute(c.drain)` panics, or ends the goroutine with `runtime.Goexit` under an inline `A.exec`; the link then stays until the subject finalizes) and records that this residual is closed in the design by the Owner's accepted Q5 direction: the foreign hand-off is registered with `defer` right after the `c.mu` section of `acceptStop` (the early-return path registers nothing), it must remain the last thing that runs in `acceptStop`, and it is not implemented.
4. **Locks (S4).** For each path P1-P8 and the new hand-off it lists the locks in order, including interfaces, closures and seams, with the verdict split into READ, EXECUTED and NOT PROVEN; the absence of a found cycle or of a test is never presented as positive evidence; no NOT PROVEN item is presented as proven; the executed evidence is stated exactly (the two `go test -race` commands on the existing same-System suite, with commit, toolchain, the source of the per-package results and the source of the counts) and is not presented as proof of lock order. The lock inventory, call-site tables and verdicts are in the lock analysis; the demonstration of carrier B and the assumptions are in the carrier B demonstration.
5. **Reentrancy, seams, shutdown (S5).** It presents the Q4 findings and records "direction accepted: inline executors not excluded for now; not proven deadlock-free", specifies the two Q4 tests (`TestCrossSystemInlineExecutorSubjectFinalizeRunsWatcherCallbackAndReenters` and the controlled eventual removal with a controlled executor), the five Q5 tests of the deferred hand-off (`TestAcceptStopIssuesTheForeignHandOffWhenAHookEndsTheStopperGoroutine`, `TestAcceptStopIssuesTheForeignHandOffWhenTheEventSinkPanics`, `TestAcceptStopIssuesTheForeignHandOffAfterTheWatchersOwnHandOver`, `TestAcceptStopTwiceIssuesOneForeignHandOff` and a production-executor test that polls with a bound until the link is absent; names are proposals, none asserts the absence of the link at `Done(W)`) and the subject-side seam, and states what a foreign watcher and a foreign subject see when either System shuts down. It records the compatibility of the designed placement with inline executors: (a)-(d) READ, (e) the two Q4 tests and the five Q5 tests NOT PROVEN because their files do not exist and they were never executed; and the premise that `acceptStop` called from the watcher's own activation has `need == false`, so the inline whole-termination case needs an external or parent caller.
6. **Evidence (S6).** It separates proposed Go-only scenarios, proposed Pekko-side scenarios, executed tests (only the existing same-System run), observed results (none for cross-System), Owner approval (plan only) and oracle availability (NOT available, NOT executed); it records that the probe `H5bCrossWatch.scala` was written and is not compiled and not executed, that `H5Probe` (one run on 2026-10-14) is the only recorded two-System output, and the impediments of the judge; it claims no parity.
7. **Impact (S7).** It contains the ADR impact list (ADR 0033 D5) below: each ADR or document that states the rejection of a handle of another System, quoted in full with its line, its impact and its proposal status; ADR 0017 and `pekko-actor/README.md` are listed as impacted and have an explicit proposal (`cross-system-watch-adr-0017-readme-amendment-proposal.md`), approved as design on 2026-10-25, applied for ADR 0017 and not for the README; the Owner decision (ADR 0017 amended in place; README only in the DTH-01 delivery) is recorded; it also lists the tests that change and the deferred message and comments.
8. **Unverified.** It lists what remains unverified, including every NOT PROVEN lock item and what the lock pass could not read.
9. **ADR texts.** The amendment proposals (ADR 0010, 0015, 0024, and ADR 0017 with the README) carry complete original and proposed text; the Owner approved the exact text as design on 2026-10-25 and it is applied to `docs/adr`; the Ask, Done and Stop contracts and authority stay out of the change.
10. **Approval.** The Owner approves the design in writing, including the carrier choice, before `DTH-01` or `DTH-02` starts. (Given on 2026-10-25 in the work thread; see "Acceptance record".)

## ADR impact list (ADR 0033 D5)

The Owner's decision named ADR 0010, 0015 and 0024 for amendment text. The search below found more statements of the same rejection.

| # | Document and lines | Impact | Proposal status |
|---|---|---|---|
| I1 | `docs/adr/0010-actor-api.md:170` | IMPACTED: contradicted and replaced | Complete original and proposed text in the ADR 0010 proposal; approved as design 2026-10-25 and applied |
| I2 | `docs/adr/0010-actor-api.md:162` | Extended, not contradicted (bounded acceptance of 2026-10-10 stays as history) | No change proposed |
| I3 | `docs/adr/0015-ask-and-reply.md:158-161` | IMPACTED: one scoped sentence is inserted after "... a second name for it." | Complete original and resulting paragraph in the ADR 0015 proposal; approved as design 2026-10-25 and applied |
| I4 | `docs/adr/0024-actor-registry-retention-after-termination.md:139` | IMPACTED: clarified so that it is about Done only | Complete original and proposed text in the ADR 0024 proposal; approved as design 2026-10-25 and applied |
| I5 | `docs/adr/0017-supervision.md:69` | IMPACTED by the cross-System watch change: states the rejection of refs of another System as a divergence pending correction | Explicit proposal in the ADR 0017 and README proposal (marker sentence, I5); approved as design 2026-10-25 and applied |
| I6 | `docs/adr/0017-supervision.md:493` | IMPACTED: case 3 (handle of another System) would no longer be an ErrInvalidHandle case for the three watch calls | Explicit proposal in the ADR 0017 and README proposal (marker sentence, I6); approved as design 2026-10-25 and applied |
| I7 | `docs/adr/0017-supervision.md:501-509` | IMPACTED: H5b says cases 2 and 3 keep the approved Go behavior (the watcher stops with ErrInvalidHandle); case 3 would change | Explicit proposal in the ADR 0017 and README proposal (marker sentence, I7); approved as design 2026-10-25 and applied |
| I8 | `docs/adr/0017-supervision.md:601-602` | IMPACTED: A9c lists "a handle of another System" as pending, with the watcher stopping in the interim | Explicit proposal in the ADR 0017 and README proposal (marker sentence, I8); approved as design 2026-10-25 and applied |
| I9 | `docs/adr/0017-supervision.md:620` | IMPACTED: D-1 states the rejection as accepted Go behavior and the Pekko behavior as acceptance with Terminated delivered | Explicit proposal in the ADR 0017 and README proposal (marker sentence, I9); approved as design 2026-10-25 and applied |
| I10 | `pekko-actor/README.md:65-67` | IMPACTED (module documentation, not an ADR): says a foreign handle fails the watcher with ErrInvalidHandle; the module README must be updated with the implementation (project README contract) | Explicit proposal in the ADR 0017 and README proposal (I10); applied only in the implementation story (DTH-01); text approved as design 2026-10-25; NOT applied |

Statements quoted in full (exact text of the cited lines):

**I1** `docs/adr/0010-actor-api.md:170`

```text
- **Handles.** A handle of this System whose actor has terminated is a normal subject. A valid reference of this System that has no actor behind it is treated as an actor that is already dead: `Terminated` is delivered (Pekko answers a watch of a dead reference with `existenceConfirmed = false`; goPekko's `Terminated` carries no such flag). A nil handle, a handle of another System, a zero ID or a `Handle` that is not an actor reference is rejected with `ErrInvalidHandle` and is never confused with a dead actor.
```

**I2** `docs/adr/0010-actor-api.md:162`

```text
Approved by the owner on 2026-10-10 for `Watch`, `WatchWith`, `Unwatch` and `Terminated` between actors of one System, as implemented in `pekko-actor/actor/watch.go`. This is a bounded acceptance: it does not accept the rest of this ADR (style A3 as a whole, supervision, ask, restart, hierarchy), and it is not a claim of parity with Pekko.
```

**I3** `docs/adr/0015-ask-and-reply.md:158-161`

```text
**One sentinel per condition [Owner, R2].** The condition "nil, foreign or non-native handle" has one sentinel,
`ErrInvalidHandle`, and Ask does not publish a second name for it. Local Watch (first drafted on the branch
`feat/actor-local-watch`, since merged as #41) declares `ErrInvalidHandle` in `pekko-actor/actor/errors.go`, with a
message specific to Watch ("invalid handle for watch: ..."). Rules as first agreed:
```

**I4** `docs/adr/0024-actor-registry-retention-after-termination.md:139`

```text
   registered by this System. A handle of another System, or any user-implemented handle, gains no authority by knowing an ID (implementation detail of D2, added with T4).
```

**I5** `docs/adr/0017-supervision.md:69`

```text
| H5b | **Not approved**: `ErrInvalidHandle` is **not** turned into an ordinary supervised failure as a substitute for compatibility between ActorSystems. The rejection of refs of another System is a **divergence pending correction**, with an issue and tests to be derived; it is separate from `nil` and from Go handles without an equivalent; it is **not parity** | S12, #443 |
```

**I6** `docs/adr/0017-supervision.md:493`

```text
| `ErrInvalidHandle` — case 3: a handle of **another System** | **none**: a reference of another ActorSystem of the same JVM is *not* rejected; `watch` and `watchWith` work and `Terminated` arrives **[Run]**; no upstream test observes it | **not classified** | **Divergence pending correction (#443)**. **Interim unchanged** (as case 2). Not parity, and **not** an ordinary supervised failure (owner, H5b) |
```

**I7** `docs/adr/0017-supervision.md:501-509`

```text
**H5b: the three `ErrInvalidHandle` cases are kept apart.** The owner did **not** approve turning `ErrInvalidHandle` into an ordinary
supervised failure as a substitute for compatibility between ActorSystems. Therefore: case 1 (`nil`) is supervised because Pekko has the
equivalent and treats it as an `Exception`; cases 2 and 3 keep **today's approved Go behavior** (the watcher stops with `ErrInvalidHandle`
and the decider is not consulted), which is an *interim exception to supervision recorded as a divergence*, not a classification. Case 3
is a **divergence pending correction**: Pekko accepts the reference (**[Run]**, `H5Probe`), no upstream test exists for it (the specs that
create more than one `ActorSystem` do not watch across systems), so its tests will be new: a Pekko-side judge scenario and its Go mirror
(#443). The restriction is **not marked as parity** anywhere. The approved Watch texts that list the three cases together
(`0010:170`, `pekko-actor/README.md:41`) are edited by the unit that makes case 1 supervised, and only for case 1; they keep stating cases 2
and 3 as they are, now labelled as a divergence.
```

**I8** `docs/adr/0017-supervision.md:601-602`

```text
**A9c — pending, not classified by this ADR:** `ErrInvalidHandle` for a Go handle without a demonstrable equivalent and for a handle of
another System (interim unchanged: the watcher stops and the decider is not consulted; divergence #443); a `Goexit` in `PostStop`
```

**I9** `docs/adr/0017-supervision.md:620`

```text
| D-1 | `Watch`, `WatchWith` and `Unwatch` reject a handle of another System with `ErrInvalidHandle` | accepted; `Terminated` is delivered **[Run]** | rejected; the watcher stops | #443 | unchanged; the decider is not consulted; documented as a divergence |
```

**I10** `pekko-actor/README.md:65-67`

```text
  `OnTerminated` stops the watcher (`ErrDeathPact`); registering again with another message, or mixing `Watch`
  and `WatchWith`, fails it with `ErrWatchConflict` (the parent's decider chooses; the default strategy restarts it, as it restarts an `IllegalStateException` in Pekko) (a repeat is a no-op only for a message comparable with `==`);
  a nil, foreign or non-actor handle fails it with `ErrInvalidHandle`. Each registration has its own generation,
```

Search made for other statements: `grep -rn "foreign\|ErrInvalidHandle\|another System\|another ActorSystem\|handle of another\|foreign System" docs/adr` and `docs/design.md`. Hits in ADR 0008, 0014, 0016 and 0019 concern tenants, signals or other meanings of "foreign" and are unrelated; the hit in ADR 0015 line 253 and ADR 0024 line 232 concern Ask and are unchanged (Ask, Done and Stop contracts stay out of this change). No other ADR states that Watch rejects a handle of another System. The ADR 0033 D5 rule applied here (line 39): "Before implementing each slice, the Architect lists the ADRs that affect the slice's contract and presents their contradictions for resolution."

**Decided 2026-10-25.** ADR 0017 (items I5-I9) is amended in place with the marker sentence of the [ADR 0017 and README proposal](cross-system-watch-adr-0017-readme-amendment-proposal.md) (applied); `pekko-actor/README.md` changes only in the DTH-01 delivery (not applied).

## Acceptance record (2026-10-25)

The Owner approved the design in an explicit message in the working thread (not a GitHub comment). The words, literally: "Apruebo como diseño: - Las enmiendas de ADR 0010, 0015 y 0024 presentadas en el consolidado de #1204, con las correcciones de #1206. - La enmienda explícita de ADR 0017, conservando el texto histórico y las referencias GOV-15 #1065 y DTH-01 #785. - Actualizar el README únicamente al entregar DTH-01. - El portador B con el defer definido en Q5, conservando los supuestos de ejecución y las pruebas como criterios obligatorios de implementación. - El diseño de GOV-51. Esta aprobación no certifica código implementado ni pruebas futuras."

Exact SHAs of the consolidated content that was approved (base: `develop` `9abfb40f6669f30ba5a05699408154a55953f253`; superseded #1199 `23f43cbafe7ecb8986188f8324843df55f698e2a`):

| PR | SHA |
|---|---|
| #1201 | `bb9079498f4df961f012754a1065c647ab07e7fe` |
| #1202 | `ce8c9a0ba1e9af8296d84ccca5228ed6c8025893` |
| #1203 | `ec6191176d5b3a3228564d3ee7649b0c2878f4a0` |
| #1204 | `eb03372aa3b02d034d067ba977c9f610bf4a492a` |
| #1205 | `c777836ea07b921cab34b9fc2ca1dd3a696a7958` |
| #1206 | `f590389a09491f075f3c3e0f1ec643a942be66c5` |

The application of the amendments to `docs/adr` is the later commit(s) on the #1206 branch (the coordinator records its SHA in the PR and the issues).

Requirements to accept the design (met by the approval) versus mandatory criteria of the implementation (pending):

| Requirements to accept the design (met) | Mandatory criteria of the implementation (pending) |
|---|---|
| Criteria 1-9: documentary, met by the content at the SHAs above (class READ where stated). | The seven proposed tests: two Q4 tests and five Q5 tests (T-Q5a..e). |
| Criterion 10: approval, given on 2026-10-25. | The ID-uniqueness test. |
| | The executor assumptions E1-E10. |
| | The defer constraint: the foreign hand-off is the last thing that runs in `acceptStop`. |
| | The `pekko-actor/README.md` change at the delivery of DTH-01. |
| | The unwatch addition to AC1 of DTH-01. |
| | Compiling and running the Pekko probe, with timeouts distinguished from absence of events, before it is used as an oracle. |

None of the right column blocks the acceptance of the design; it blocks only the completion of the implementation. The approval certifies no implemented code and no future test.

## Dependencies (nothing is marked ready)

`GOV-51` is the producer and `DTH-01` ([#785](https://github.com/getsyntegrity/gopekko/issues/785)) and `DTH-02` ([#827](https://github.com/getsyntegrity/gopekko/issues/827)) are the consumers (`depends_on`); both stay blocked until their own readiness is checked. GOV-15 ([#1065](https://github.com/getsyntegrity/gopekko/issues/1065)): its decision deliverable (option (b)) is delivered and recorded there; the issue stays open and is not a blocker by itself. The design approval is no longer a blocker. Remaining blockers for DTH-01 and DTH-02 are their own readiness: the documents merged by the Owner and their criteria checked; DTH-02 also needs DTH-01 and GOV-48 ([#1195](https://github.com/getsyntegrity/gopekko/issues/1195)) decided. Q5 is no longer an open Owner question (direction accepted: the `defer`); its implementation and its five tests belong to DTH-01/DTH-02. The change of `pekko-actor/README.md` lines 65-67 (text proposed in the [ADR 0017 and README proposal](cross-system-watch-adr-0017-readme-amendment-proposal.md), I10) is part of the DTH-01 delivery and is described in DTH-01 ([#785](https://github.com/getsyntegrity/gopekko/issues/785)); it is not done in this documentation stack. Independent of GOV-42 ([#1188](https://github.com/getsyntegrity/gopekko/issues/1188)): they meet only in `complete` (`hierarchy.go:233`). Related: GOV-48 ([#1195](https://github.com/getsyntegrity/gopekko/issues/1195)).
