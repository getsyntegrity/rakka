# Owner decisions for the actor and actor-typed design (audit #004, single current list, 2026-10-27)

> **Resolution note 2026-11-04 (Owner decisions recorded in ADR 0035, ADR 0036 and ADR 0033 D14/D15).** S-2, S-3, S-4, S-5 and EVT-1 are resolved and B-4 is partially resolved (X4 by ADR 0036; X6 open)
> by the amendments listed in the section "Resolutions (added 2026-11-04)" at the end. The recommendations in the rows below are kept as written; S-3 in
> particular was recommended as "Accept as a divergence" and the Owner decided to follow the fixed Pekko behavior instead.

**Status: documentation only.** This is the one current list of the Owner decisions that audit #004 leaves open. It replaces, for
decision tracking, section 5 of `typed-behavior-supervision-readiness-2026-10-26.md`, the decision tables of the two decision packages
and section 5 of `typed-q2-q6-classification-2026-10-27.md` (PR #1215); those documents keep the evidence. Section 0 records the five decisions the
Owner took on 2026-10-27; everything else here is still open.

Text under decision: ADR 0034 and ADR 0035 as in commit `99edce8` (PR #1217, which aligns them with the verified findings of
[typed-behavior-decision-package-2026-10-27.md](typed-behavior-decision-package-2026-10-27.md) and
[typed-supervision-decision-package-2026-10-27.md](typed-supervision-decision-package-2026-10-27.md)). The acceptance wordings below
supersede the wordings in those packages. Baselines: goPekko `develop` `77a9237`; Apache Pekko v1.7.1 `9a54d12`.

Already decided and not asked again: ADR 0033 (2026-10-24, D1 to D13 with the D13 interpretation); the ADR 0010, 0015, 0017 and 0024
amendments of 2026-10-25 (cross-System watch, implemented by #1207); the GOV-48 decision of 2026-10-26 (PR #1209, section 1).

Four distinctions apply to every row. **Integrating** a report or a proposed ADR publishes text; it decides nothing. **Accepting** a
contract makes it binding for the stories that consume it. **Implementing** delivers the capability. **Parity** is shown only by the
evidence ADR 0033 D13 requires. Each wording below accepts a contract or a classification only.

## 0. Decisions taken by the Owner on 2026-10-27

The Owner accepted five decisions in the working session on 2026-10-27 (recorded from the work thread; not a GitHub comment). The
words, literally: "Acepto B‑1, S‑1, S‑6, EXP‑1 y DTH‑02 con las redacciones corregidas del informe adjunto." The "corrected wordings" are the ones below. They cite the content integrated in `develop` `2029305e8ca0efb0866b2d6c8dba3bcd33f307a4`. The
earlier citations of `99edce8` (head of the PR #1217 branch) and `128d0b5` (head of the PR #1209 branch) stay valid historical references
to the reviewed commits; the same content was integrated by squash in `2029305` (#1217) and `7114abd` (#1209).

| Decision ID | Status | Accepted wording (exact) | Reserved: not accepted by this decision |
|---|---|---|---|
| B-1 | **Accepted**, with the correction that P5 (except its construction mechanism) and P7 (as a hook) are part of it: P3, P4 and P6 deliver `PostStop`, `Terminated` and `ChildFailed`, which P5 defines, and P4's `Unhandled` row refers to P7 | "I accept ADR 0034, as published in `docs/adr/0034-typed-behavior-boundary.md` at `develop` `2029305e8ca0efb0866b2d6c8dba3bcd33f307a4`, section 'Decision (proposed)': decisions P1, P2, P3, P4, P6 and P8; decision P5 except how `Terminated{Ref}` and `ChildFailed{Ref, Cause}` obtain their `Ref` and cause (decision B-2); decision P7 as written (a module-local hook until the event-stream decision); and differences X1, X3 and X5 of section 'Differences', as the Typed `Behavior[T]` contract. P2's interceptor constructors are a deferred capability. X2, X4 and X6 are decided separately. This accepts a contract only: no implementation, no parity claim, no merge." | B-2 (how `Terminated{Ref}` and `ChildFailed{Ref, Cause}` get their `Ref` and cause), B-3 (X2), B-4 (X4, X6), B-5, interceptors |
| S-1 | **Accepted** | "I accept ADR 0035 alternative (a), as published in `docs/adr/0035-typed-supervision.md` at `develop` `2029305e8ca0efb0866b2d6c8dba3bcd33f307a4`, sections 'Pekko evidence: the contracts' and 'Proposal (Architect)': contracts 1 to 7 and 9 to 15, mechanisms M-A and M-B, and differences D-c and D-e (language adaptations). Contract 3 is accepted for the M-A path and for matchers narrower than `AnyError()`; the M-B case with a match-all `Restart.WithLimit` is decision S-2. D-b and D-d are Pekko's behavior, not differences. Publication of `Dropped` is a deferred capability. Contract 8, D-a, D-f and the cause shape of contract 14 are decided separately. ADR 0017 is unchanged. This accepts a contract only: no implementation, no parity claim, no merge." | S-2 (M-B match-all `withLimit`), S-3 (D-a), S-4 (contract 8 and D-f), S-5 (cause shape of contract 14), publication of `Dropped` |
| S-6 | **Accepted** | "ADR 0033 D2 is read as stated in the paragraph 'Proposed precision of ADR 0033 D2' of section 'ADR impact list (ADR 0033 D5)' of `docs/adr/0035-typed-supervision.md` at `develop` `2029305e8ca0efb0866b2d6c8dba3bcd33f307a4`: under a Typed parent a Classic child is decided by Pekko's adapter decider (initialization, kill and death-pact errors stop; other non-fatal failures restart). This records how D2 applies and does not change its acceptance." | S-2 |
| EXP-1 | **Accepted** | "Each `pekko-actor` export needed by another module is justified in the story that introduces it, with the symbol, the consumer, why the public API is not enough, what it does not expose, and its test. #1020 indexes those records and blocks no story; a story that adds no export needs no record. An ADR is written only when the export changes an accepted contract. This approves no specific export." | every concrete export; the location of the event stream (EVT-1) |
| DTH-02 | **Accepted** | "I confirm criteria 1 to 4 of section 12 of `docs/parity/gov-48-dth-02-reconciliation-2026-10-26.md` at `develop` `2029305e8ca0efb0866b2d6c8dba3bcd33f307a4` as the acceptance criteria of #827, replacing its three behavioral criteria. The original text stays in the issue. This approves no test result, changes no label and declares no parity." | the test results of #827, its labels, any parity claim |

Still open after these decisions: B-2, B-3, B-4, B-5, S-2, S-3, S-4, S-5, EVT-1 and every row of sections 3 and 4. ADR 0034 and ADR 0035
are **partly accepted**: their status lines say which parts.

Update (2026-11-04): S-2, S-3, S-4, S-5 and EVT-1 were resolved on 2026-11-04 and B-4 was partially resolved (X4 by ADR 0036; X6 open, pending the `UnhandledMessage`/`Dropped` oracle); see "Resolutions (added 2026-11-04)" at the end of this document.

## 1. Decisions that block the first deliveries (all five accepted on 2026-10-27; section 0 has the wording that applies)

| Decision ID | Question | Existing authority | Recommendation | Difference from Pekko | Affected stories | Exact acceptance wording |
|---|---|---|---|---|---|---|
| B-1 | Is the core of ADR 0034 the Typed `Behavior[T]` contract? | ADR 0033 D1 | Accept | None beyond Go error values (X1) and the sealed shape (X3); interceptors deferred | #809, #847; then #925, #954, #994 | "I accept ADR 0034 decisions P1, P2, P3, P4, P6 and P8 and differences X1, X3 and X5, as in commit `99edce8`, as the Typed `Behavior[T]` contract. P2's interceptor constructors are a deferred capability. P5, P7, X2, X4 and X6 are decided separately. This accepts a contract only: no implementation, no parity claim, no merge." |
| S-1 | Is ADR 0035 alternative (a) the Typed supervision contract, as a new ADR with ADR 0017 unchanged? | ADR 0033 D2 (an ADR extending 0017 is required) | Accept | None for the accepted items; `Dropped` publication, manual time and Typed timers deferred | #956, #1183, #1184, #976, #997; then #986, #999 | "I accept ADR 0035 alternative (a), as in commit `99edce8`: contracts 1 to 7 and 9 to 15, mechanisms M-A and M-B, and differences D-c and D-e (language adaptations). D-b and D-d are Pekko's behavior, not differences. Publication of `Dropped` is a deferred capability. Contract 8, D-a, D-f, the cause shape of contract 14 and the M-B case of decision S-2 are decided separately. ADR 0017 is unchanged. This accepts a contract only: no implementation, no parity claim, no merge." |
| S-6 | Is ADR 0035's reading of ADR 0033 D2 accepted? | ADR 0033 D2 (accepted; not asked again) | Accept as a precision | None: it states Pekko's parent-kind rules | #956, #994 | "ADR 0033 D2 is read as stated in ADR 0035 (commit `99edce8`, section 'Proposed precision of ADR 0033 D2'): under a Typed parent a Classic child is decided by Pekko's adapter decider (initialization, kill and death-pact errors stop; other non-fatal failures restart). This records how D2 applies and does not change its acceptance." |
| EXP-1 | How is each `pekko-actor` export that another module needs justified? (readiness decision 6) | ADR 0033 D3; ADR 0011 owner decision 2; #1021 | Record in the introducing story | None (governance) | removes #1020 as a blocker of #847, #794, #767 | "Each `pekko-actor` export needed by another module is justified in the story that introduces it, with the symbol, the consumer, why the public API is not enough, what it does not expose, and its test. #1020 indexes those records and blocks nothing. An ADR is written only when the export changes an accepted contract." |
| DTH-02 | Are the criteria of #827 restated as proposed? | GOV-48 decision of 2026-10-26 (PR #1209) | Confirm | None claimed; oracle status per criterion stays as stated | #827 (tests only) | "I confirm criteria 1 to 4 of section 12 of `docs/parity/gov-48-dth-02-reconciliation-2026-10-26.md` (PR #1209 at `128d0b5`) as the acceptance criteria of #827. The original text stays in the issue. This approves no test result and declares no parity." |

## 2. Decisions that block later capabilities

| Decision ID | Question | Existing authority | Recommendation | Difference from Pekko | Affected stories | Exact acceptance wording |
|---|---|---|---|---|---|---|
| B-2 | How does the interpreter build `Terminated{Ref}` and `ChildFailed{Ref, Cause}` (P5)? | ADR 0033 D1 | The interpreter records the cause of every failure of a watched child and resolves the `Ref` from its watch table | None | #954, #956 | "P5 of ADR 0034 is accepted with mechanism (a) of decision B-2: the Typed interpreter records the failure cause of each watched child and resolves the `Ref` from its watch table. This is a contract precision, not an implementation." **Update 2026-10-28:** B-2 was accepted for `Terminated` only, by `ID` (the watch-table wording of this row is not adopted); the cause half stays open (#1250). See ADR 0034, "Amendment (accepted)". |
| B-3 | Does a Typed `Tell` accept nil (X2)? | #452 (open); ADR 0017 does not authorize accepting nil | One rule for Classic and Typed, decided in #452; Pekko rejects nil | Pekko throws `InvalidMessageException` | #452, #847 | "X2 of ADR 0034 is not accepted as a language adaptation. Nil messages follow the decision of #452 for both Classic and Typed; until then, accepting nil is an open divergence." |
| B-4 | Is P7 delivered as a local hook, and are X4 and X6 deferred? | ADR 0033 Consequences (event stream not pre-approved) | Yes | `UnhandledMessage` not published until the event stream exists | #925, #897, #870 | "P7 of ADR 0034 is accepted as a module-local hook until the event stream exists. X4 and X6 are deferred capabilities, not intentional differences. They close when the event stream exists; where it lives is decision EVT-1." |
| B-5 | Can an adapter ref be watched and asked? | none accepted | Deferred capability owned by #888; the Typed context maps a watch on an adapter ref to its owner meanwhile | Watch and ask fail in Go until delivered | #888, #975 | "Watching and asking a message-adapter ref are deferred capabilities owned by #888. Until delivered, the Typed context maps a watch on an adapter ref to its owner." |
| S-2 | M-B, match-all `Restart.WithLimit`, limit exhausted on a message or signal: follow Pekko? | none (Pekko behavior, CODE only) | Follow Pekko (alternative A); no Go impediment found | None under A | #976, #1184 (one criterion each) | "In the M-B case of decision S-2, goPekko follows Pekko: the rethrown cause reaches the parent, whose strategy decides; under the default strategy an `Exception` restarts the actor (the Typed `setup` runs again, the limit count starts again, the actor's children are kept and restarted), a `DeathPactException` stops it and any other non-fatal cause escalates; a start failure always stops it. Evidence is CODE only; a differential scenario is required before V3 (ADR 0033 D13)." |
| S-3 | Unlimited Restart with a `setup` that fails during restart (D-a) | ADR 0033 D13 | Accept as a divergence | Pekko ends in a fatal `StackOverflowError` (JVM exit by default) or restarts forever when setup created children; goPekko stops the actor | #976 | "D-a of ADR 0035 is accepted as a divergence with a concrete impediment: goPekko stops the actor where Pekko ends in a fatal `StackOverflowError` or restarts forever. Alternative evidence: a Go test of the stop. This is a divergence, not parity." |
| S-4 | Does an unowned `ResetRestartCount` reach the user behavior (contract 8, D-f)? | none; `SupervisionSpec.scala:1049-1051` asserts that it does | Follow Pekko | None under the recommendation | #986 | "Contract 8 follows Pekko: an unowned `ResetRestartCount` reaches the user behavior as an opaque signal and is ignored when unhandled. D-f is withdrawn." |
| S-5 | Which cause does the parent see when Resume meets a failing `setup` (contract 14)? | none | Follow Pekko's cause shape | None under the recommendation | #1183 | "Contract 14 follows Pekko's cause shape: the parent sees an initialization error that wraps the validation error." |
| EVT-1 | Where are `UnhandledMessage` and `Dropped` published? (readiness decision 5) | ADR 0011 owner decision 2; ADR 0033 Consequences | A stream in `pekko-actor`, introduced and justified by its Classic stories | None once delivered | #925, #999, #897, #870, #906 | "The event stream that publishes `UnhandledMessage` and `Dropped` is part of `pekko-actor`, justified individually in its introducing story; until it exists, Typed records these events through a module-local hook and does not publish them." |

## 3. Classification decisions that block no code

They decide how inventory rows are recorded (PR #1215, section 5, where the alternatives are). A story is blocked only by the
decisions and contracts it consumes.

| Decision ID | Question | Existing authority | Recommendation | Difference from Pekko | Affected stories | Exact acceptance wording |
|---|---|---|---|---|---|---|
| C-1 | Are the seven reason codes of `docs/actor-typed-completion-design.md:342` the vocabulary of ADR 0033 D13? | ADR 0033 D13 (the list is closed; the codes are only in the dossier) | Confirm | none (vocabulary) | #1044 and every GOV story citing the list | "The exclusion vocabulary of ADR 0033 D13 is the seven codes of `docs/actor-typed-completion-design.md:342`. Each use still needs a source and a reason per row." |
| OD-1 | May an internal type be recorded as "no Go surface; behavior owned by story X"? | ADR 0033 D13 | Record the status; add no code | none | #1044, #1026 | "An internal Pekko type is recorded as 'no Go surface; behavior owned by <story>'. It is not an exclusion code." |
| OD-2 | Which code covers external-dependency, Go-idiom and JVM-implementation replacements? | ADR 0033 D13 | `jvm-only` for JVM tooling, `owner-excluded` for `kit-logger`, one approved batch for the Go idioms | none | #1044 | "Rows replaced by JVM tooling use `jvm-only`; rows replaced by `kit-logger` use `owner-excluded`; the Go-idiom rows of PR #1215 section 5 OD-2 are approved as one batch under `replaced-by-go-stdlib`." |
| OD-3 | One rule for the Java facades? | ADR 0033 D7, D8, D13; #1126 | One rule plus a register table | none | #1126, #1044 | "Java facades (`javadsl`, `japi`, `Patterns`, `EventBusJavaAPI`, builders) are classified `java-api` by one rule, listed in the register with the story that owns their behavior." |
| OD-4 to OD-11 | Serialization of `Ref`/`ServiceKey`, distributed markers, DNS, Classic config keys, `AbstractBehavior`, the context guard, version and setup shape, `util` collections | as listed in PR #1215 section 5 | as listed there | per row | as listed there | Decided one by one with the wording "OD-n of PR #1215 section 5 is accepted as recommended there." No block approval. |

## 4. Technical choices the Architect can make now

Error and type names (`ErrNilBehavior`, `ErrInvalidInitialBehavior`, `ErrDeathPact`, signal names); matcher values (`AnyError()`,
`ErrorIs`, `ErrorAs`) and how the interpreter recognises its own default wrapper; option names and the backoff clock; the interim mapping
of a watch on an adapter ref to its owner (B-5). They stay inside the contracts above and need no approval.

## 5. What the first stories need

| Story | Needs |
|---|---|
| #827 DTH-02 (tests only, Classic) | DTH-02 |
| #1096 TYP-34 module skeleton | ADR 0033 D3 (accepted) and EXP-1 for any export it adds |
| #847 TYP-02 Behavior interpreter | B-1, EXP-1, #1096 |
| #956 TYP-09 Typed default Stop | S-1, S-6, #847 |
| #954, #925, #976, #1183, #1184, #997, #986, #999 | the rows of section 2 named against them, plus #847 or #956 |

## Resolutions (added 2026-11-04)

Historical record: the rows above are not edited. This section links each one to the decision that replaces it. Sources: [ADR 0033](../adr/0033-actor-typed-scope-and-architecture.md) D14/D15, the amendment of [ADR 0035](../adr/0035-typed-supervision.md) and [ADR 0036](../adr/0036-typed-event-stream-and-interceptors.md).

| Decision ID | Resolved by | Outcome |
|---|---|---|
| S-2 | ADR 0035, amendment, S-2 | Follow Pekko (as recommended). Evidence is code reading only, so it stays **To be verified** until a differential scenario exists. |
| S-3 (D-a) | ADR 0035, amendment, S-3 case (c) "D-a"; ADR 0033 D15 | **Not** accepted as a divergence. Follow the fixed Pekko behavior, for an actor with **no children at restart time** (the case the oracle verified; the variant with children, e.g. a `setup` that creates children or `withStopChildren(false)`, is **To be verified** and nothing is decided for it): the loop is detected and goes through the fatal-error path (`on`: log, CoordinatedShutdown, termination callbacks, `os.Exit(255)`; `off`: the ActorSystem terminates, no `os.Exit`). Oracle: `docs/parity/oracles/d-a-restart-setup-unbounded-2026-11-04.md`. |
| S-4 (D-f) | ADR 0035, amendment, S-4 | Follow Pekko (as recommended). D-f is withdrawn. |
| S-5 | ADR 0035, amendment, S-5 | Follow Pekko's cause shape (as recommended). |
| EVT-1 | ADR 0036, "EVT-1: publication to the EventStream" | `UnhandledMessage` and `Dropped` are published to the `EventStream` of `pekko-actor`. |
| B-4 | ADR 0036, "Consequences" (X4 only) | **Partially resolved: X4 by ADR 0036; X6 open (pending the `UnhandledMessage`/`Dropped` oracle).** The module-local hook stays until the stream exists and X4 closes when it is delivered. The sender field of `UnhandledMessage` (X6) has no decision yet (see ADR 0034, "Amendment (2026-11-04)"). |

### Owner wording (2026-11-04)

The Owner's words, kept in Spanish with an English translation in brackets. The first column links the decision that records each one.

| Decision | Owner wording (2026-11-04) | Recorded in |
|---|---|---|
| D14 | "toda decisión funcional debe ser tal cual apache pekko" [every functional decision must be exactly as Apache Pekko] | [ADR 0033, D14](../adr/0033-actor-typed-scope-and-architecture.md) |
| S-2, S-4, S-5 | "supervisión Typed como en Pekko" [Typed supervision as in Pekko] | [ADR 0035, amendment S-2, S-4, S-5](../adr/0035-typed-supervision.md) |
| S-3 | "siguiendo como lo hace apache pekko" [following how Apache Pekko does it] | [ADR 0035, amendment S-3](../adr/0035-typed-supervision.md) |
| EVT-1 | "`UnhandledMessage` y `Dropped` se publican en el EventStream de `pekko-actor`" [`UnhandledMessage` and `Dropped` are published to the EventStream of `pekko-actor`] | [ADR 0036, EVT-1](../adr/0036-typed-event-stream-and-interceptors.md) |
| Interceptors | "se exponen como en Pekko" [they are exposed as in Pekko] | [ADR 0036, Interceptors](../adr/0036-typed-event-stream-and-interceptors.md) |
| D15 and D-a | The Owner accepted ("dale" [go ahead]) the proposal to follow Pekko's behavior with the fix of apache/pekko#3620 and to register rule D15. What was accepted is the text of D15.1: "When Pekko behavior is acknowledged upstream as a bug (an apache/pekko issue labeled `bug` with a merged fix in `main` or in a maintained release branch), gopekko follows the **fixed** behavior, even if no published release contains the fix yet." | [ADR 0033, D15](../adr/0033-actor-typed-scope-and-architecture.md); [ADR 0035, amendment S-3 case (c)](../adr/0035-typed-supervision.md) |
| D14 clarifications | Ratified by the Owner on 2026-11-05 (as reported in the Owner instruction; no verbatim text recorded): D14.2, D14.3 and D14.4, each in full, are Architect clarifications. Only D14.1 follows the Owner wording above. | [ADR 0033, D14](../adr/0033-actor-typed-scope-and-architecture.md) |

## Resolutions (added 2026-11-05)

Historical record: nothing above is edited or deleted; the rows of the 2026-11-04 section that these entries refine (B-4 "partially resolved"; the precondition of D-a) stay as written. Owner wording on 2026-11-05: "ok", accepting the proposals recorded in the amendments below. The third column quotes, as accepted, the decision text of each amendment. Evidence: the oracles [`evt-1-unhandled-dropped-2026-11-05.md`](oracles/evt-1-unhandled-dropped-2026-11-05.md) and [`d-a-with-children-2026-11-05.md`](oracles/d-a-with-children-2026-11-05.md).

| Decision | Owner wording | Accepted text | Recorded in |
|---|---|---|---|
| X6 (and B-4) | "ok" | "The `UnhandledMessage` event of goPekko has the fields `message`, `sender` and `recipient`, as in Pekko ... Typed sends ...: the `deadLetters` ref of the system. It is not nil and not the temporary actor of an ask." B-4 is resolved: X4 by ADR 0036, X6 by this amendment. | ADR 0034, Amendment (2026-11-05) |
| EVT-1, per site | "ok" | "goPekko behaves as Pekko 1.7.1 at each site": restart stash overflow and group router with no routees publish a `Dropped` event (sender noSender; recipient the actor or the router); `Topic.Publish` with no subscribers publishes a `DeadLetter` wrapping a `Dropped`, with no `Dropped` event; group router stash full is To be verified. | ADR 0036, Amendment (2026-11-05) |
| D-a, precondition | "ok" | "D-a applies to an unbounded `restart` whose `setup` fails on every restart when the restart does not wait for children to stop: there are no children to stop, or `withStopChildren(false)`." | ADR 0035, Amendment (2026-11-05) |
| D-a, sub-case C1 | "ok" | "(c-async) ... the restart waits for children to stop ...: an unbounded asynchronous loop, no fatal error, the system stays alive." | ADR 0035, Amendment (2026-11-05) |

Links: [ADR 0034](../adr/0034-typed-behavior-boundary.md), [ADR 0035](../adr/0035-typed-supervision.md), [ADR 0036](../adr/0036-typed-event-stream-and-interceptors.md).

## Resolutions (added 2026-11-05, `Port`)

Historical record: nothing above is edited or deleted. Owner wording on 2026-11-05: "si", accepting the proposal of the audit [`port-and-persistence-receiveport-vs-pekko-2026-11-05.md`](audits/port-and-persistence-receiveport-vs-pekko-2026-11-05.md), and "si" to the second adjustment of 2026-11-05 (point 5 and the scope of D14).
The third column quotes, as accepted, the decision text of ADR 0037.

| Decision | Owner wording | Accepted text | Recorded in |
|---|---|---|---|
| `Port` | "si" | "`Port` is a Go adaptation. Its API belongs to gopekko; its Pekko counterpart is the send to the `ActorRef`. Admission, order and capacity are those of ADR 0030 D6. ... An instance that is not a `PortHandler`: a post to it is a failure of the actor and goes through its supervision, as a message of the wrong type does in Pekko Typed ... Persistence `ReceivePort` with a foreign type: it is a failure of the actor and goes through its supervision." | ADR 0037 |
| `Port`, point 5 | "si" | "when the Behavior does not implement `SnapshotResultReceiver`, `ReceivePort` logs `SnapshotCompleted` at DEBUG and `SnapshotFailed` at ERROR, with the content of the default handler of Pekko (message and cause)". The default handler is a fallback, not an addition: Pekko runs the handler of the user first. | ADR 0037 |
| `Port`, point 5 (revision of 2026-11-05) | Owner instruction of 2026-11-05 (not a "si"): "el log por defecto se aplica a CADA resultado de snapshot que el receptor no maneje ... Un receptor que no maneja un resultado no lo silencia." | "the default log applies to each snapshot result that the receiver does not handle, and a receiver that does not handle a result does not silence it": `SnapshotCompleted` at DEBUG, `SnapshotFailed` at ERROR, with the Pekko texts. This revises the accepted text of the `Port`, point 5 row above (which said "when the Behavior does not implement `SnapshotResultReceiver`"); that row is not edited. The current all-or-nothing interface must change. | ADR 0037, point 5; issue #1344 |
| Scope of D14 | "como regla general" | "D14 is a project-wide rule: it applies to every gopekko module, including persistence (ADR 0018), regardless of the scope of this ADR." | ADR 0033, Amendment (2026-11-05) |

Links: [ADR 0037](../adr/0037-port-as-a-go-adaptation.md), [ADR 0033](../adr/0033-actor-typed-scope-and-architecture.md).

## Resolutions (added 2026-11-05, c)

Historical record: nothing above is edited or deleted. Owner wording on 2026-11-05: "si dale", accepting the proposals recorded in the amendments below. The third column quotes, as accepted, the decision text of each amendment. Evidence: the oracles [`s-2-supervision-2026-11-05.md`](oracles/s-2-supervision-2026-11-05.md) and [`interceptors-2026-11-05.md`](oracles/interceptors-2026-11-05.md), and the audit [`untyped-delivery-paths-2026-11-05.md`](audits/untyped-delivery-paths-2026-11-05.md).

| Decision | Owner wording | Accepted text | Recorded in |
|---|---|---|---|
| S-2.6 | "si dale" | "Initial start fails. It is retried up to the limit, then the actor stops without consulting the parent's decider ... Start fails after the parent restarted the actor. The start of the new instance is retried and the failure reaches the parent's decider once more." It replaces "A start failure always stops it". This is the first version of the text; the next row revises it. | ADR 0035, Amendment (2026-11-05, c) |
| S-2.6, rule (revision) | "si — apruebo S-3 (b) y la regla revisada de S-2.6" (2026-11-05), on the Owner instruction "Si lo explicás con archivo:línea: escribí la regla en S-2.6" | "Where the rethrow lands depends on whether it happens while the supervisor's `aroundStart` is still running ... Rethrow inside the start ... the actor stops and the parent's decider is **not** consulted ... Rethrow after the start returned ... reaches the parent's decider." Evidence: section 6.2 of the oracle [`s-2-supervision-2026-11-05.md`](oracles/s-2-supervision-2026-11-05.md). | ADR 0035, Amendment (2026-11-05, c) |
| S-2.3c | "si dale" | "When the parent restarts the actor, the existing children are kept and restarted and the new `setup` creates its children again, so after the restart the actor has **two** children ... goPekko copies this (D14)." | ADR 0035, Amendment (2026-11-05, c) |
| S-2, verification | "si dale" | "S-2.1 to S-2.5 are confirmed ...; S-2.6 is rewritten above ... the \"To be verified\" of S-2 is closed." | ADR 0035, Amendment (2026-11-05, c) |
| Interceptors | "si dale" | "goPekko behaves as Pekko 1.7.1": `logMessages` logs messages and signals; deduplication by `isSame` also at start, the inner one stays; interceptors inside `supervise` are created again and see `PreRestart`, those outside do not; `monitor` sends before the handler runs; the outer interceptor runs first before and last after; `transformMessages` with no match is an `UnhandledMessage` with the original message; the outer `withMdc` wins. Still To be verified: MDC during signals, `BehaviorSignalInterceptor`, the `interceptMessageClass` bypass. | ADR 0036, Amendment (2026-11-05, c) |
| Untyped routes | "si dale" | "The message type is a generic parameter, and no existing route lets a value of the wrong type reach the handler of a `typed.Behavior` ... The rule 'a wrong type is a failure' stays in force." The PR that introduces a receptionist, routers, `Topic`, message adapters, `transformMessages` or Classic-Typed interop must audit its route again with the criterion of the audit `untyped-delivery-paths-2026-11-05.md` and link the result. `Port.Post(any)` and the persistence `ReceivePort` are decided by ADR 0037. | ADR 0036, Amendment (2026-11-05, c) |

2026-11-05, S-3 case (b) with a limit: **accepted** by the Owner. Owner wording: "si — apruebo S-3 (b) y la regla revisada de S-2.6". It approves the instruction "Si confirma Supervision.scala:302, S-3 (b) pasa a decir: con n ≤ 1 el arranque inicial no se reintenta; con n ≥ 2 se reintenta hasta n veces (n reinicios, n + 1 corridas de setup)", which the oracle (section 6.1 of `s-2-supervision-2026-11-05.md`) confirms, and the amendment of ADR 0035 states.

## Resolutions (added 2026-11-06)

Historical record: nothing above is edited or deleted. Owner wording on 2026-11-06: "si — apruebo el scope de #1184 (G2) ... D-5: enmienda a ADR 0035 en una PR aparte, con el patrón de siempre". The third column quotes, as accepted, the decision text of the amendment. Evidence: the oracle [`typed-nested-supervision-2026-11-06.md`](oracles/typed-nested-supervision-2026-11-06.md).

| Decision | Owner wording | Accepted text | Recorded in |
|---|---|---|---|
| Deduplication of equal supervisors | "si — apruebo el scope de #1184 (G2)", "D-5: enmienda a ADR 0035 en una PR aparte" | "A `supervise` whose class is exactly equal to an enclosing one's makes the enclosing one disappear **once a stack has started**: after the initial start, after a stack returned by a handler has started, and after a restart has rebuilt the stack (`InterceptorImpl.preStart` runs `aroundStart` and then `deduplicate`, `InterceptorImpl.scala:82-85`, `106-122`). **While the start is failing, the enclosing supervisor still receives what an inner one rethrows**: a Restart that rethrows at the start (an unlimited restart, or an exhausted limit, `S:294-306`) is caught by the enclosing supervisor if the class of the enclosing supervisor matches the exception, whether or not the two supervisors have the same class." It replaces "at start and after every turn". | ADR 0035, Amendment (2026-11-06) |

Links: [ADR 0035](../adr/0035-typed-supervision.md), [ADR 0036](../adr/0036-typed-event-stream-and-interceptors.md).
