# Scope: the Typed `Unhandled` result and its interpretation (#925, TYP-03, Slice A) — G2 decisions recorded (see section 11); no Dev started

Status (update, dated 2026-10-28, see section 11.1: U-2 (a), U-4 and U-7 were confirmed by the Owner in a later message that also reached the TL partly corrupted; nothing below is rewritten): **G2 decisions U-1 to U-7 recorded on 2026-10-28 at the head `98a254d3e6009f44f4cd3c27a848d917d99630db` of the proposal (section 11 and the Log), as read by the TL from two Owner messages that reached the TL partly corrupted; points marked "reading" there are to be confirmed or corrected by the Owner. No Dev has started and none is authorized by this text.** Written by the TL on 2026-10-28 against `develop` `1a48053a5b53fb4489b855f9a26d2670ca4e3a21`. Pekko baseline: v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.
The Owner authorized preparing this scope; the G2 decisions of section 11 were added afterwards; the scope changes no issue or label. #925 stays open and `status:blocked`. #954 stays open. Nothing here is implemented.
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: it ports Pekko behavior (`Behavior.scala`, `Behaviors.scala`, `ActorAdapter.scala`), so the Dev runs the kit with its own manifest, RULEBOOK and RUNBOOK.
**Evidence labels.** READ = source read, not executed. No Pekko spec or Typed oracle ran (no Typed harness, not V3; ADR 0033 D13: READ rows are alternative evidence pending Owner approval). The Go side was read at the base SHA, not run. No parity is claimed.

## 1. Dependencies, reconciled (what is declared, what is real)

The ACT2 reconciliation of #925 (2026-10-24) **replaced** #870 with #897: "the #870 dependency is wrong: the unhandled event is #897". So the declared prerequisites of #925 are #847 and #897, and #870 is **not** a direct prerequisite of #925. #870 appears only inside #897's own chain. The table separates the two deliverables of section 2.

| Item | State | Needed by Slice A (marker, interpretation, local record) | Needed by Slice B (publication of `UnhandledMessage`) |
|---|---|---|---|
| #847 (TYP-02) | closed, integrated | **yes**, delivered | yes, delivered |
| #897 (BHV-03) Classic `UnhandledMessage` publication | open, `status:blocked` | **no** (see below) | **yes**: it produces the event |
| #870 (EVT-03) publish through subscribed mailboxes | open, blocked | no | transitive, through #897 (declared there) |
| #871 (EVT-05) suppressed dead letters | open, blocked | no | transitive, through #897 (declared there; "reconcile with #870" is still open in #897) |
| #814 (MSG-02) sender in `Receive` | open, blocked | no (X6: no sender field) | through #897 (ADR 0032 D4) |
| #1036 (GOV-05) decision on the Classic `unhandled` hook (ADR 0010 inventory row) | open, `status:ready` | no (Typed-only; the local record is a different thing) | through #897 |
| ADR 0034 P4, P7, B9-B11 | accepted (B-1, P7 "as written") | **consumed** | consumed |
| ADR 0034 X4, X6 and decision B-4 (is the missing publication a *deferred capability*?) | **proposed** | classification not needed to build Slice A | needed to word the final claim |
| EVT-1 (where the event stream lives) | open | no | yes |

**Why Slice A does not need #897.** ADR 0034 P7 (accepted with B-1) fixes the outcome and says that, until the event-stream decision, "the implementation story records the unhandled message through a module-local hook and states that it is not yet published; it is never sent as a `DeadLetter`". `pekko-actor` has no unhandled hook (searched: no `Unhandled` symbol in `pekko-actor/actor`). The scope of #847 already left `Unhandled` out for exactly this reason ("its meaning is 'record the message through the module-local hook' (ADR 0034 P7, #925); shipping the marker alone would ship half of that row", `docs/scopes/847-typed-behavior-install.md:30-31`). So the marker and the local record ship **together**, and the publication follows #897.

**A finding in a sibling issue (not edited here).** #1181 (`empty`, `ignore`) declares "Consumed: Behavior core and unhandled marker (#847)" and a single dependency on #847. #847 did **not** deliver the marker (its scope excluded it, above), and `empty` is defined as "answers every message and signal with the unhandled result" (Pekko `Behavior.scala:305-306`). The real producer of the marker is **#925 Slice A**. So #1181 **keeps** its dependency on #847 (the Behavior core and the interpreter it runs on) and **adds** #925 Slice A as the producer of the `Unhandled` marker it consumes; its text attributes the marker to #847, which is the inaccuracy. This scope records it and does not edit #1181; the Owner decides whether to correct the issue.

## 2. Two deliverables, separated

| | Slice A (this scope) | Slice B (not specified here) |
|---|---|---|
| What | the `Unhandled` result, its interpretation in a turn, the death pact for an explicit decline of `Terminated`, and the module-local record (P7) | publishing `UnhandledMessage` to an event stream |
| Observable | the behavior is kept, nothing is a failure, nothing is success; the local record fires; no `DeadLetter` | a subscriber receives `UnhandledMessage` (Pekko `ActorContextSpec.scala:100` through `createUnhandledMessageProbe`) |
| Blocked by | nothing real (#847 is delivered) | #897 and its chain; EVT-1; B-4 |

**Effect on the issue's criteria.** Reconciled AC1 of #925: "unhandled emits UnhandledMessage and is not successful handling". Slice A delivers "is not successful handling" and the **local** record the accepted P7 prescribes; it does **not** deliver the emission of `UnhandledMessage`. So #925 is **not completed by Slice A** and stays open, the same arrangement as #954 Slice A. AC2 (`empty`/`ignore`) moved to #1181 and AC3 (`stopped`) to #1182 by the reconciliation.

## 3. Pekko behavior, read (all READ; nothing executed)

Paths under `upstream/pekko/actor-typed/src/main/scala/org/apache/pekko/actor/typed/` (typed) and `.../actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/` (specs).

| Rule | Source |
|---|---|
| `Behaviors.unhandled` is a value `BehaviorImpl.unhandled`, a return value only | `scaladsl/Behaviors.scala:65`; `internal/BehaviorImpl.scala:60,73` |
| `unhandled` (and `same`) cannot be an initial behavior: `validateAsInitial` throws | `Behavior.scala:240-243` |
| Interpreting `unhandled` or `same` as the *current* behavior is an error | `Behavior.scala:289-295` (ADR 0034 B11, READ) |
| A message result `Unhandled` makes the adapter call `unhandled(msg)`; the **current behavior is not replaced** | `internal/adapter/ActorAdapter.scala:166-167` (`next`); `Behavior.scala:187` (`canonicalize`: `UnhandledBehavior => current`) |
| For a non-signal message, `unhandled` ends in `super.unhandled`, which publishes `UnhandledMessage(msg, sender, self)` | `ActorAdapter.scala:220,228-229`; classic `Actor.scala:656-660` |
| For a signal other than `Terminated`, `unhandled` is a no-op ("that's ok") | `ActorAdapter.scala:226-227` |
| An unhandled `Terminated` (or `ChildFailed`) throws `DeathPactException` inside `interpretSignal` | `Behavior.scala:270-277` |
| `empty` answers `unhandled`, `ignore` answers `same` (both are #1181's) | `Behavior.scala:297-306` |
| Spec: a behavior that returned `unhandled` keeps working afterwards (`Miss`, then `Renew`), and an unhandled probe receives the message | `ActorContextSpec.scala:100-130` (the publication part needs the event stream) |
| Spec: `Behavior.interpretMessage` returns the unhandled result and the handler's side effect happened | `BehaviorSpec.scala:256-262` (uses the BehaviorTestKit and the internal predicate `isUnhandled`; no direct Go counterpart) |

## 4. Contract proposed for Slice A

In `pekko-actor-typed/typed/`. Names are indicative; the shapes are the contract.

1. **One export:** `func Unhandled[T any]() Behavior[T]`, a sealed value like `Same` (`behavior.go:68`). It carries the justification record required by EXP-1 in the PR (symbol, consumer, why the public API is not enough, what it does not expose, its test).
2. **Message turn.** A `Receive` handler that returns `Unhandled`: the turn **succeeds**, the current behavior (and its signal handler) is **kept**, nothing is a failure (no restart, no `StopSelf`, no cause), the message is **recorded through the module-local hook** (P7) and **never sent as a `DeadLetter`**. It is the same observable state as `Same` plus the record.
3. **Signal turn.** A signal handler (`ReceiveSignal`) that returns `Unhandled` for `Terminated` makes the interpreter apply the death pact of the accepted P6 (the existing path: generate the failure, no Typed supervision exists, `StopSelf`, the Classic decider not consulted) — this **completes the `TODO(TYP-04-edge)` of Slice B**. For any other signal (`PostStop`, and the declared-only types) `Unhandled` is dropped silently, as `Same` is (`ActorAdapter.scala:226-227`). Nothing is recorded for a signal.
4. **Initial and `Setup` results.** `Unhandled` as an initial behavior, or as the result of a `Setup` factory, fails the start like `Same` does (Pekko `validateAsInitial`, `Behavior.scala:240-243`).
5. **Out of Slice A:** the publication (Slice B), `Empty` and `Ignore` (#1181), `Stopped`/`StoppedWith` (#1182), the sender of the record (X6), `MessageAdaptionFailure`'s default-receive failure (B15, #975).
6. **Hook (decision U-2).** Not exported. See U-2: observing it needs an injection point, i.e. a seam.
7. **What changes in the code (so the Dev does not discover it late).** (i) `turn()` returns only a `bool` today: it needs to tell *succeeded and handled* from *succeeded and unhandled*, so `Receive` can record the message and `OnTerminated` can take the death pact. (ii) `start()` has no case for a new sealed variant and falls to its `default`, which reports `ErrNilBehavior`: it needs an `Unhandled` case that fails with `ErrInvalidInitialBehavior` (U6), or an `Unhandled` initial behavior would be reported as nil. (iii) A message handler that returns a `Setup` which resolves to `Unhandled` goes through `start()` inside `turn()` and needs the same case (Pekko validates the started result). (iv) The signal path in `OnTerminated` must distinguish the handler's `Unhandled` from `Same`.

**Unchanged by Slice A, to stay unchanged:** `Same` is still "handled" for `Terminated`; a handler that answers `Same` to every signal still swallows `Terminated` (Slice B's D-B4 trap), now with a documented way out: return `Unhandled`. The README and the `ReceiveSignal` doc comment change accordingly.

## 5. Scenarios (Go unit, real runtime, channels and gates, no sleeps, no `-race`)

Internal tests in `package typed`. **The record (U-2) cannot be observed without an injection point in production code**: an interpreter field or an unexported constructor that tests set. That is a **seam**, and Slice B's decision D-B3 rejected a test-only production hook (its option C, also rejected for #956). The difference, and its limit, are stated in decision U-2; this scope does not call U1 and U3 seamless.

| # | Scenario | Pekko source | Evidence |
|---|---|---|---|
| U1 | A message handler returns `Unhandled`: the local record fires once with the message; the next message is handled by the **same** behavior; no restart, no stop, no failure | `ActorContextSpec.scala:100-130` (the "keeps working" part) | READ, publication part not reproducible |
| U2 | Same, then a handler that replaces the behavior afterwards: the replacement applies (the unhandled step did not freeze the behavior) | `ActorContextSpec.scala:100-130` (`Renew`) | READ |
| U3 | `Unhandled` is not recorded as a `DeadLetter` and is not a failure: the decider count is 0, no dead letter observed | ADR 0034 P7 | READ |
| U4 | A signal handler returns `Unhandled` for `Terminated`: the actor stops through the default Stop, `PostStop` runs once, the decider (default and a custom one answering Restart) is not consulted | `Behavior.scala:270-277`; Slice B T7a/T7b | READ |
| U5 | A signal handler returns `Unhandled` for `PostStop`: no effect, `Done` closes, no panic | `ActorAdapter.scala:226-227` | READ |
| U6 | `Unhandled` as initial behavior, and as the result of a `Setup`: the start fails with the initial-behavior error | `Behavior.scala:240-243` | READ |
| U7 | Order: `Unhandled` on message N, then message N+1: the record precedes the next turn; and no overlap with a signal on the same cell | ADR 0034 B19 | READ |
| U8 | Regression: Slice A/B tests (PostStop once, T1-T14), #847 and #956 tests still pass; `Same` and `Receive` behave as before | | existing tests |
| U9 | Public surface: the only new export is `Unhandled` (and whatever U-5 decides); nothing from `package actor` | | review / API diff |

A scenario observes its result with channels and gates, never a sleep. Equivalence with a Pekko spec is not claimed: the specs above publish through an event stream that Go does not have.

## 6. Allowed files (proposal text, read through section 11; a Dev starts only when the Owner says so)

| Path | Edit | PR |
|---|---|---|
| `docs/scopes/925-typed-unhandled-marker.md`, `migration/stage-1/typed-925-unhandled/{manifest.tsv,RULEBOOK.md,RUNBOOK.md}` | scope (this document) and kit package | 1, 2 |
| `pekko-actor-typed/typed/behavior.go` (the marker), `interpreter.go` (turn, signal path, hook), `signal.go` (doc of `ReceiveSignal`) | edit, keep their `Ported from` lines | 3 |
| `pekko-actor-typed/typed/unhandled_internal_test.go` (U1-U7, U9), tests for U8 are the existing ones | new | 3 |
| `pekko-actor-typed/README.md`, `migration/stage-1/typed-925-unhandled/EXECUTION-RECORD.md`, `migration/cost-log.tsv` (one row, `subagents` counted from the logs) | update; limitations; gates bound to a SHA | 4 |

Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum`. Sequencing: Slice A edits `behavior.go` and `interpreter.go` with #1181 and #1182 and with #976; they run after it or after each other, never in parallel.

## 7. Gates, budget, delivery

Gates (commands in `AGENTS.md`, "Go"): unit tests of `pekko-actor-typed` (network isolation is Linux-only; the container variant where available, otherwise declared as not run), `go vet`, `gofmt -l`, `golangci-lint` v2.14.0 with `gocyclo` at most 10, `go mod tidy -diff`, module and diff coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `go-sdk-update.sh --check`. Run once per final tree, each result bound to its SHA. Remote CI is blocked by billing; local results are not presented as approved CI.

**Budget.** At most 400 authored lines per PR (additions plus deletions). Forecast: PR1 scope about 230; PR2 kit about 90; PR3 code and tests about 330 (production about 80, tests about 250); PR4 README, record and cost-log about 130. Forecasts, not measurements. If PR3 is projected over 400 the TL moves tests to PR4 (the arrangement used for #954 Slice B). Draft PRs, `Refs #925` (never `Closes`), no merge by the Dev.

## 8. Decisions for the Owner (the G2 request as proposed; the decisions taken are in section 11)

- **U-1, split.** Accept the two deliverables of section 2 and that Slice A does not complete AC1 (the emission of `UnhandledMessage`). Alternative: wait for #897 and ship marker and publication together (blocked by #897, #870, #871, #814, #1036 and EVT-1). **Recommend the split.**
- **U-2, the local record (P7), and the seam it needs.** The honest premise: any way for a *test* to see the record needs a field or a variable that tests can set, i.e. a production injection point. Options: (a) an **unexported** hook with an unexported constructor for tests (no export; a seam in production code that has a legitimate default, the no-op record of P7, and is removed when the event stream exists); (b) an exported option so an application can observe it (new public API that the event stream would later supersede, hence a break; needs an ADR 0011 justification); (c) log it (no logger accessor, #1179); (d) **no hook test**: Slice A is tested only for what is observable without a seam (behavior kept, no failure, no restart, no dead letter) and the record is declared an evidence gap with no coverage claimed. **Tension to decide:** D-B3 (Slice B) rejected a *test-only* production hook. (a) differs only in that accepted ADR 0034 P7 itself prescribes a module-local hook as the production mechanism, so the hook is a feature and not test scaffolding; the Owner may still read it as the same seam. **Recommend (a)** if the Owner accepts that reading; otherwise **(d)**. Cost of (a): in production nothing observes an unhandled message until Slice B, as P7 says; cost of (d): the record, the point of P7, has no test.
- **U-3, the export.** `Unhandled[T]() Behavior[T]`, justified under EXP-1 in the PR. No other export.
- **U-4, signal rule.** `Unhandled` returned for `Terminated` takes the death pact of P6 (as proposed); other signals are dropped. This reuses the accepted P6 and does not need a new ADR text.
- **U-5, the initial-behavior error.** `ErrInvalidInitialBehavior` is an existing export whose message says "Same is not a valid initial behavior". (a) Reuse the sentinel and generalize the message text (a change to an exported value's text: Risk item 3); (b) reuse it unchanged and let the wrapping name `Unhandled`; (c) add a new sentinel (a new export). **Recommend (b)**: `errors.Is` keeps working, no text change, no new export. Cost of (b): the sentinel's own text still says "Same is not a valid initial behavior", so the wrapped message reads awkwardly ("... got Unhandled").
- **U-6, the finding on #1181.** Whether to correct #1181's declared dependency (`marker from #847`). Precision (Owner): #1181 keeps #847 as its Behavior core and consumes the `Unhandled` marker from #925 Slice A. This scope does not edit #1181.
- **U-7, B-4.** Slice A's final wording ("not published, deferred capability") depends on the Owner accepting or amending decision B-4 / X4 / X6 (proposed). Until then Slice A says only "not published yet".

Nothing in this scope approves B-4, EVT-1, a hook, an export or #897.

## 9. Not delivered, not claimed

- The publication of `UnhandledMessage` (Slice B, #897 and its chain); `Empty` and `Ignore` (#1181); `Stopped` and `StoppedWith` (#1182); the sender (X6); the Typed-supervision capture of the death pact (#1379; this said #976 when written); `ChildFailed` (#1250, closed as not planned: now #1346); the public Typed watch (#1171).
- #925, #954, #1181 stay open and `status:blocked`. No parity claim. READ rows stay pending Owner approval (ADR 0033 D13). This scope does not change the tests of `pekko-actor-typed` to the go-specs convention (deferred by the Owner to a separate change).

## 10. Definition of Done (Slice A, once a Dev is authorized to start)

U1 to U9 pass at the final SHA; no function above complexity 10; gates green and bound to the SHA; the record says which rows are READ and what is not published; README and the `ReceiveSignal` comment match the code and state that nothing is published; a new QA session, independent of TL and Dev, issues a verdict on the exact head of each PR; no merge without the Owner; #925 closes only when its remaining criteria are met and the Owner closes it.

## 11. Owner decisions recorded (G2, 2026-10-28) and how this document reads them

The two Owner messages that carry these decisions reached the TL with words missing. The table records what is legible, the TL's reading of the rest, and what has to be confirmed. The scope text above (sections 1 to 10) keeps the alternatives as they were proposed.

| Decision | Recorded decision | Basis |
|---|---|---|
| U-1 | Split accepted. Slice B is the publication of `UnhandledMessage` (after #897). Slice A does not satisfy AC1 and does not allow closing #925. | legible fragment: "U-1: ... Slice B ... publicación ... no AC1 ni permite #925"; **reading:** "does not allow closing #925" expands "ni permite #925" |
| U-2 | The local record is a per-instance hook, with **no new public API**; it is the mechanism of ADR 0034 P7; its default is a no-op that stores nothing, publishes nothing and does nothing with the message in production. **Reading (to confirm):** the Owner revokes D-B3's rejection of a production injection point for this case, i.e. option (a) is accepted and U1/U3 may observe the record through that hook. If the Owner meant the opposite, option (d) applies and U1/U3 lose the record assertion. | "U-2: instancia, nueva API pública. mecanismo P7; revoca D-B3. predeterminado no-op almacena, publica ni hace mensaje producción" (words missing around "nueva API pública" and "revoca") |
| U-3 | `Unhandled[T]() Behavior[T]` is the only new export, justified under EXP-1 in the PR. | legible |
| U-4 | `Unhandled` returned for `Terminated` takes the death pact of P6 without consulting the Classic decider; other signals are dropped. **This does not decide** `MessageAdaptionFailure` or `ChildFailed`. | "U-4: acepto ... decider Unhandled. Esto ... decide MessageAdaptionFailure, ChildFailed" (the negation before "decide" is read, not legible) |
| U-5 | Reuse `ErrInvalidInitialBehavior`, keeping `errors.Is` working (option b). | legible |
| U-6 | Precision recorded in section 1 and in U-6: #1181 keeps #847 as Behavior core and consumes the `Unhandled` marker from #925 Slice A. #1181 itself is not edited. | "U-6: #1181 #925 #847" and the later "Precisar U-6: #1181 conserva #847 núcleo #925 Unhandled" |
| U-7 | Decision B-4, the publication of `UnhandledMessage` and EVT-1 stay **pending**; Slice A says only "not published yet", never "deferred capability" or "intentional difference". **Reading (to confirm).** | "U-7: B-4, UnhandledMessage EVT-1" (verb missing) |

**Unchanged by these decisions:** #925, #954 and #1181 stay open; the READ rows stay pending Owner approval (ADR 0033 D13); no parity claim; no migration to go-specs and no worktree cleanup now; PRs of at most 400 computable lines, merged only with `--match-head-commit` on the reviewed head and without `--admin`; the Dev does not start until the Owner says so for each PR (kit first, PR2).

### 11.1 Confirmation of 2026-10-28 (added later; the table above and the Log entry L2 are left as written)

A second Owner message, also received partly corrupted (fragments in the Log, L3), answers the three points that section 11 marked "to confirm". The table records what each fragment supports.

| Point | Confirmed | Fragment | Limit of the confirmation |
|---|---|---|---|
| U-2 | **Option (a):** the local record is a per-instance injection point, with no new public API; its default is a no-op that stores nothing and publishes nothing; it is the mechanism of ADR 0034 P7; D-B3 (Slice B) does not stand in the way for this case. | "U-2(a): inyección API no-op predeterminado almacena ni publica. P7; D-B3 vigencia" | Still a production injection point (a seam): unexported, no-op default, removed when the event stream exists. The phrase "D-B3 vigencia" is read as "D-B3 does not apply to this case", consistently with the earlier "revoca D-B3". |
| U-4 | `Unhandled` returned for `Terminated` takes the death pact without consulting the Classic decider; this decides neither `ChildFailed` nor `MessageAdaptionFailure`. | "U-4: decider Classic. decide ChildFailed ni MessageAdaptionFailure" | The negation "does not decide" is read; the pairing with "ni" is legible. |
| U-7 | B-4, X4, X6 and EVT-1 stay pending, and the publication waits for #897 and its chain; Slice A says only "not published yet". | "U-7: B-4/X4/X6 EVT-1 #897" | No verb is legible; "stay pending" is the reading that keeps every earlier statement true. |

**Also in that message (as read):** the kit package of PR 2 is updated with the confirmations (the start block is replaced by a dated note), U3 and U7 are aligned as READ Go-only assertions and U9 as a READ contract check of the public API, `make parity-guard` is added to the gates (`AGENTS.md`), and the evidence level is READ throughout; a Dev for PR 3 and then PR 4 follows; PR #1260 (the kit) is merged only on a head with an independent QA verdict. #925, #954 and #1181 stay open; no parity claim; READ rows stay pending Owner approval (ADR 0033 D13); go-specs and the worktree cleanup stay deferred.

Two earlier sentences are superseded for PR 3 and PR 4 by this confirmation, as read: the Status sentence "No Dev has started and none is authorized by this text" (it stays as the historical statement of the first version; PR 2, the kit, was started on the Owner's "adelante") and the sentence at the end of section 11 ("the Dev does not start until the Owner says so for each PR (kit first, PR2)").

## Log

L1 (2026-10-28, Owner, verbatim, Spanish):

> Después, preparar el scope de #925, todavía sin implementación. Reconciliar primero sus dependencias: su sección ACT2 ya reemplaza #870 por #897; no tratar ambas como prerrequisitos vigentes. Separar el marcador y su interpretación de la publicación de UnhandledMessage, y presentar qué puede entregarse respetando el contrato aceptado.
>
> Mantener #954 abierta, sin afirmar paridad ni aprobar implícitamente las filas READ. Diferir go-specs y la limpieza de worktrees.

L2 (2026-10-28, Owner, received partly corrupted; the fragments that reached the TL, verbatim):

> "G2 #1258 98a254d3e6009f44f4cd3c27a848d917d99630db U-1: Slice B. publicación Slice no AC1 ni permite #925. U-2: instancia, nueva API pública. mecanismo P7; revoca D-B3. predeterminado no-op almacena, publica ni hace mensaje producción. U-3: Unhandled[T]() Behavior[T] EXP-1. U-4: acepto decider Unhandled. Esto decide MessageAdaptionFailure, ChildFailed U-5: reutilizar ErrInvalidInitialBehavior conservando errors.Is U-6: #1181 #925 #847 U-7: B-4, UnhandledMessage EVT-1 SHA QA PR 400 #1258 PR #925 #954 aprueban READ go-specs limpieza worktrees siguen"
>
> "1. Actualizar #1258 G2 U-1 U-7. Precisar U-6: #1181 conserva #847 núcleo #925 Unhandled."

The TL's reading is in section 11. Nothing was invented beyond it; the two points marked "reading" (U-2 and U-7) and the missing negations of U-4 are the ones to confirm.

L3 (2026-10-28, Owner, received partly corrupted; the fragments that reached the TL, verbatim):

> "#1260 1. Registrar decisiones: U-2(a): inyección API no-op predeterminado almacena ni publica. P7; D-B3 vigencia U-4: decider Classic. decide ChildFailed ni MessageAdaptionFailure. U-7: B-4/X4/X6 EVT-1 #897. 2. Eliminar bloqueos PR. fechada. 3. parity-guard RUNBOOK AGENTS.md. 4. Alinear las U3/U7/U9 distinguiendo READ, API. nivel evidencia READ. 5. Publicar nuevo HEAD obtener QA delta, comprobando coherencia conjunto. Dev PR3 luego PR4, U-2/U-4/U-7. No #1260: SHA QA Mantener #925, #954 #1181 paridad. go-specs worktrees"

The TL's reading is in section 11.1. The words missing around "D-B3 vigencia", the verb of U-7 and the exact sense of "No #1260: SHA QA" are readings; the last one is read as "do not merge #1260 until an independent QA verdict exists on its final SHA".
