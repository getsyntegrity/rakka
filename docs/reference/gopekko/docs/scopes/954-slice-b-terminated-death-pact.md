# Scope: Typed `Terminated` delivery and death pact (#954, TYP-04, Slice B) — proposal for G2

Status: **G2 APPROVED by the Owner on 2026-10-28 at SHA `183171b9afdbf967bbbe73d79ecfa68a3b636a5f` (L3), with the limits recorded in L3.** Authorized: a Dev for **B1** (the kit package) and preparing **B0b** (the separate ADR amendment PR). **Not authorized:** B2 and B3 before the ADR amendment is accepted; accepting Appendix A (its text still needs review and explicit acceptance); re-slicing #954/#1171; any Pekko-parity claim; merge; closing issues. #954 stays open and `status:blocked`. The one wording precision of L3 item 5 was applied after the approved SHA and does not change a decision. The status of this document before G2 follows.

Previous status: **PROPOSAL, not approved.** Written by the TL on 2026-10-28 against `develop` `fd5a04ecbe66c2e035175252282142d8e9311c71` (Slice A is integrated:
PR #1236 `ab9b44c`, docs `a03e38d`, B17 test #1239). Pekko baseline: v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (submodule checked at that SHA).
The Owner authorized research and documentation only: this does **not** approve B-2, a watch source, a test seam, G2, Dev, merge or closing issues.
#954 stays open and `status:blocked`. **Revision 3** (after the Owner review of #1249 at `7fee406` and the Owner decisions L2): T7b uses `actor.Spawn`; D-B2, D-B3, D-B4, D-B5 and D-B6 recorded as taken; D-B1 compared again with a recommendation and an unperformed re-slice; the ADR 0034 P6 amendment is drafted (Appendix A, not accepted). One issue was created, [#1250](https://github.com/getsyntegrity/gopekko/issues/1250) (the M-A / `ChildFailed` story); no other issue or label is edited. Nothing here is implemented. Awaiting independent QA of the delta before the final G2 request.
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: Slice B ports Pekko behavior, so its Dev runs the kit (`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK.

**Evidence labels.** READ = source read, not executed. No Pekko test or Typed oracle was run for this document (ADR 0033: no Typed Pekko harness, so not V3; rows stay
alternative evidence pending Owner approval, ADR 0033 D13). The Go side was read, not run, at the base SHA.

## 1. What is accepted and what is only proposed

| Item | State | Source |
|---|---|---|
| P5 sealed `Signal` set, B16 | **accepted** (B-1, 2026-10-27), except the construction of `Terminated{Ref}` / `ChildFailed{Ref, Cause}` | `docs/adr/0034-typed-behavior-boundary.md:3-7, 62, 131-134` |
| P6 unhandled `Terminated`/`ChildFailed` returns an error wrapping `actor.ErrDeathPact` with the subject, raised before the runtime sees it (B14) | **accepted** | `0034:136-140` |
| B19 one actor, one mailbox | accepted (READ) | `0034:65` |
| **B-2**: how the interpreter builds `Ref` and `Cause` | **proposed, open** | `0034:134, 198`; `docs/parity/typed-behavior-decision-package-2026-10-27.md` (B-2 row) |
| ADR 0035 M-A / M-B, contracts 1-7, 9-15 | accepted; contract 8, D-f, cause shape of contract 14 proposed | `docs/adr/0035-typed-supervision.md:3-9, 36-41` |
| Slice A types: opaque `Terminated`, `ChildFailed`, `MessageAdaptionFailure` (no field, accessor or constructor) | merged on `develop` | `pekko-actor-typed/typed/signal.go:31-37` |
| `ChildFailed` goes to a separate M-A story, not #956, not B-2 | accepted (D-5) | `docs/scopes/954-typed-lifecycle-signals.md:104-108` |
| `PreRestart` -> #976, `MessageAdaptionFailure` -> #975 | accepted destinations | same scope, "What this does not deliver" |

Update (2026-11-04): in the ADR 0035 row above, contract 8 now follows Pekko (D-f withdrawn) and the cause shape of contract 14 follows Pekko (ADR 0035 amendment S-4, S-5).

Staleness found: ADR 0035:205 still labels ADR 0034 "proposed" although its header is "partly accepted". Not touched here (ADR edits need the Owner).

## 2. Behavior of Pekko v1.7.1 (READ) and what Go has

Paths are under `upstream/pekko/actor-typed/src/main/scala/org/apache/pekko/actor/typed/` (typed) and `actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/` (specs;
note: `actor-typed/src/test` does not exist).

| Fact | Pekko | Go today |
|---|---|---|
| `Terminated` carries `ref: ActorRef[Nothing]`, public `apply/unapply`, `equals` by ref | `MessageAndSignals.scala:63-65, 81-93` (spot-checked) | runtime `actor.Terminated{who ID}`, accessor `Who()`, no constructor, no existence/address flags (`pekko-actor/actor/watch.go:22-25`, ADR 0010 D9) |
| A classic `Terminated` becomes a typed signal; `ChildFailed` if a cause was recorded | `internal/adapter/ActorAdapter.scala:94-101` (spot-checked), `handleSignal` `:146-150` | the interpreter implements `Actor`, `PreStarter`, `PostStopper`, `PreRestarter`; **not** `TerminatedHandler` (`typed/interpreter.go`) |
| Result of the signal handler is applied like a message result (`canonicalize`, `Stopped`, `Unhandled`, `Failed`) | `ActorAdapter.scala:163-180` | `turn` (`interpreter.go`) installs `Receive`, restarts `Setup`; `Same` keeps; anything else or error/panic = failed turn |
| **Unhandled** `Terminated` (or `ChildFailed`, a subtype) raises `DeathPactException(ref)` inside `interpretSignal` so it can be supervised | `Behavior.scala:270-277` (spot-checked); fallback `ActorAdapter.scala:220-227` | runtime without `TerminatedHandler`: `watchError{ErrDeathPact}` to the decider (`watch.go:505-509`) |
| Signals and messages share one mailbox and cell | `ActorAdapter.scala:88-120` | `Terminated` is a system message taken **between turns**, put at the **tail** of the mailbox, takes a slot (a full mailbox -> dead letter) (`watch.go:266-375, 470-500`) |
| `watch` of an already-dead actor still delivers `Terminated` | `ActorContextSpec.scala:407`; classic `DeathWatch.scala:35-41` | delivered (`0010:170`) |
| A restart drops `watchWith` registrations | `SupervisionSpec.scala:618` | restart is #976, out of scope |
| Typed `watch`/`watchWith`/`unwatch` delegate to classic | `ActorContextAdapter.scala:106-117` | **no Typed `Watch`** in `typed/` (#1171 open) |

Specs that pin the behavior (READ, none executed): `WatchSpec.scala:88` (graceful termination), `:165` (a nested hierarchy where the middle actor receives a `ChildFailed`, a `Terminated` subtype, and `DeathPactException` is logged: the **`ChildFailed` path**, M-A story, not Slice B), `:206` (idempotent watch; the test only completes a promise, it does not assert a count),
`ActorContextSpec.scala:375, 407, 443` (watch before / after termination, unwatch) and `:483` (terminate upon not handling `Terminated`; its behavior declares `receiveSignal { case (_, Terminated(_)) => Behaviors.unhandled }`, an **explicit decline**, pending #1181, not the no-handler case), `BehaviorSpec.scala:226, 230, 234` and `:313, 317, 321` (`Terminated` reaches the
handler, also after a message, a message after it, and after `swap`), `SupervisionSpec.scala:1408` (a `DeathPactException` can be supervised), `OnSignalSpec.scala:36`, `DeferredSpec.scala:72, 90`.
`TerminatedSpec.scala` and `WatchSpec.scala:100` concern `ChildFailed` (M-A story, not here).

## 3. The three blockers, as proposals for the Owner

### D-B1 (B-2): how `Terminated` is built and what it holds

**Not approved (Owner, L2). This scope publishes no representation, accessor or export, and re-slices no issue.** What follows is the comparison the Owner asked for, a recommendation and the exact re-slice that would be needed
if B were chosen. `ID` = the runtime's `Terminated.Who()`; `Handle` = `interface{ ID() ID }` (`id.go:12`). All rows READ; nothing executed.

**Comparison against Pekko's `Terminated.ref: ActorRef[Nothing]`.**

| Operation of the Pekko value | Pekko source | With `ID` | With `Handle` | Class |
|---|---|---|---|---|
| Identity: `equals`/`hashCode` through `ref` (path + uid) | `MessageAndSignals.scala:81-93` | **kept**: comparable, process-unique, atomic counter (`id.go`) | **kept**, through `ID()` | conserved by both |
| Use as the subject of a later `Watch` / `Stop` | `ActorContext.scala:176` | lost | **kept**: `Context.Watch(h)`, `System.Stop(h)` take a `Handle` | *recoverable* for `ID` only if a runtime `ID`→`Handle` lookup is added (no exported lookup exists; searched; an unexported `System.lookupID` is internal to `pekko-actor`) or if the Handle is later stored; conserved by `Handle` |
| `tell` to the terminated actor (a dead letter) | `ActorRef.scala:47, 55` | lost | lost as such (`Handle` has no `Tell`); **recoverable** by the caller who knows `U`, with `h.(actor.Ref[U])`, because the stored handle is the native ref (READ; to be confirmed by the Dev in RED) | *inevitable* for `ID`; *recoverable* for `Handle` |
| `toString` `Terminated(<ref>)` | `MessageAndSignals.scala:86` | different text | different text | *recoverable* for both (a `String()` can be defined later) but never Pekko's text (it prints a path) |
| `compareTo` (total order on refs) | `ActorRef.scala:35` | lost | lost | *recoverable* by defining an order over `ID`, but not Pekko's path order |
| `path` | `ActorRef.scala:63` | lost | lost | **inevitable** for both: `ActorPath` (`actor_path.go`) is not linked to a ref or an `ID` in non-test code (searched). Recoverable only by a future `pekko-actor` change outside this story |
| Serialization (`SerializedActorRef`) | `ActorRef.scala:66, 84-110` | out of scope (ADR 0033) | out of scope | not a loss of this slice |

**Availability.** `ID` is available for **every** termination the runtime reports. A `Handle` exists only if an interpreter table recorded it when a Typed `Watch` ran; a termination registered any other way (for example the
internal tests of D-B3) would carry none. `Handle` gives identity nothing more than `ID`.

**Effect on #1171.**

| | `ID` (A) | `Handle` (B) |
|---|---|---|
| #1171's criteria R1-R4 | unchanged; #1171 consumes the delivery as its text already says | R1 (plain `watch` registers) must exist for the table to be filled; R4 (`unwatch`) must remove table entries; R2/R3 interact with the table |
| Relation with D-B2 | consistent: no public Typed watch in Slice B | **contradicts D-B2** (Owner accepted `OnTerminated` without a public Typed watch); B requires revoking D-B2 |
| Later upgrade | additive: a `Handle()` accessor can be added once #1171 stores handles; `ID()` stays, so nothing is removed or retyped | n/a |

**Exact re-slice that B would need (proposal only, not performed; no issue is edited).** (1) Move #1171's R1/AC-R1-1 (plain `watch`, idempotent) into #954 Slice B as a new criterion, together with a minimal public Typed
`Context.Watch` that records the handle in an interpreter table. (2) Narrow #1171 to R2-R4 and to table maintenance (`watchWith`, `unwatch` removing the entry, `ErrWatchConflict`). (3) Edit the bodies of #954 and #1171 and the
dependency table (`#954 → #1171` stays `depends_on`). (4) Revoke D-B2 and extend §7 with `context.go`. (5) Split a PR for the table. Cost: a larger Slice B, state per watch, two issues rewritten.

**Recommendation: A (`ID`)**, to be decided at G2. Reasons: it is the only option consistent with the accepted D-B2 and needs no cross-issue change; it conserves the identity contract; it forecloses nothing, because a `Handle()`
can be added later without removing `ID()`; and `Handle`'s extra operations (later subject, recoverable send) matter only once a public Typed watch exists, which is #1171's job. The cost to name plainly: the handler cannot
`Watch` or `Stop` the terminated actor from the signal alone (it would need the `Handle` it already holds, if any), and `path`, `toString` and ordering differ from Pekko (to be classified by the Owner, ADR 0033 D13).
`ChildFailed` is not decided here: in Pekko `ChildFailed <: Terminated`, equal by `ref`, and an unhandled one is also a death pact; sealed structs cannot reproduce that subtype relation, so #1250 must decide it.
The accepted option needs an ADR 0034 P5 amendment (own small PR, B0b).

### D-B2: real source of termination notifications, and #1171

**Owner decision (L2): option A accepted.** `OnTerminated` is implemented without a public Typed watch. Until #1171 delivers one, this is **not a capability a production caller can use**: `Terminated` reaches a Typed behavior only in internal tests (D-B3). The README says so.

The notification source is the existing runtime watch (`Context.Watch`, `watch.go:107`), delivered through `TerminatedHandler.OnTerminated` (`actor.go:40-43`). Slice B only needs the interpreter to
implement `OnTerminated`; it does not need to expose a watch API. #1171 is the user-facing Typed `Watch/WatchWith/Unwatch` and is **blocked by #954** (issue #1171, dependency table:
#954 -> #1171, `depends_on`; also blocked by #827), so Slice B cannot wait for it.

| Option | Implication |
|---|---|
| **A (recommended)** | Slice B delivers the signal and the death pact; **no Typed watch is added**. Tests register the watch through the wrapped runtime context from inside `package typed` (see D-B3). #1171 later adds the public API and consumes this delivery, as its issue already states. |
| B | Add a minimal Typed `Watch` in Slice B. Overlaps #1171's contract (idempotence, `ErrWatchConflict`, unwatch), risks two owners of `context.go`. |
| C | Wait for #1171. Impossible: it waits for #954. |

Consequence of A, stated honestly: until #1171 lands, **no production caller can make a Typed actor receive `Terminated`**; the delivery is proven by internal tests only. The README must say so.

### D-B3: deterministic test access

**Owner decision (L2): option A accepted** (internal tests in `package typed`, no new export, no seam).

| Option | Implication |
|---|---|
| **A (recommended)** | White-box tests in `package typed` (`*_internal_test.go`) reach the wrapped context (`Context[T].c`, an unexported field, `context.go`) and call the existing exported `actor.Context.Watch`. **No new export, no production seam, no `pekko-actor` change.** Precedent: `signal_b17_internal_test.go`; #956 S5 (`docs/scopes/956-typed-default-stop.md:75`). Limit: only reachable from inside the package, which is the point. |
| B | An `export_test.go` helper. Equivalent in power, adds a second mechanism; not needed. |
| C | A test-only production hook (as `guardian.go:118` in `pekko-actor`). Rejected for #956 (`956:191`); same reasoning here. |
| D | Export a watch helper from `typed`. Publishes API that #1171 should own. Rejected. |

Deterministic means channels and gates, no sleep, no `-race` (as Slice A). The watched actor is a plain `actor.Spawn` actor stopped with `System.Stop` and `Done`.

### D-B4: a handler that returns `Same` for `Terminated` (partial delivery, explicit)

**Owner decision (L2): option A accepted, as a partial delivery.** The explicit decline stays pending in #1181 and is not declared covered.

Pekko's `receiveSignal` takes a partial function; a signal it does not match is `Unhandled`, and an unhandled `Terminated` (or `ChildFailed`) is a death pact (`Behavior.scala:270-277`). The Slice A handler shape is **total**
(`func(ctx, s) (Behavior, error)`) and `Unhandled` does not exist yet, so a handler that answers `Same` to every signal (as the Slice A example does) would be treated as having handled `Terminated`.

Proposal: in Slice B **`Same` (or any replacement behavior) means handled**, and "unhandled" is only **"the active behavior has no signal handler"**. This is a **partial delivery** of the unhandled-signal contract:

- Delivered by Slice B: no handler -> death pact; handled -> result applied.
- **Not delivered, still pending:** a handler that explicitly *declines* a `Terminated` (Pekko `Behaviors.unhandled`, or a partial function that does not match) does **not** cause a death pact in Go. Follow-up: **#1181**
  (TYP-47, parent #925). Its R3 reads: "An actor whose behavior is `empty` MUST fail with the death pact when it receives an unhandled Terminated, and an actor whose behavior is `ignore` MUST NOT." That is narrower than a handler declining a signal in general: the general decline is the `Unhandled` marker of #925/#1181 and is pending too. #1181 is open and `status:blocked`;
  Slice B does not unblock it. Slice B must not be described as delivering the unhandled-signal semantics: the README and the execution record say "partial: explicit decline is pending #1181".
- A greppable `TODO(TYP-04-edge)` marks the interpreter branch #1181 completes. The trap (a total handler swallowing `Terminated`) is documented in the README and the `ReceiveSignal` doc comment.

Alternatives: B) block Slice B until #1181 delivers `Unhandled` (complete contract, but the delivery waits on a blocked chain); C) a nil result means "unhandled" (invents semantics; rejected). **Recommend A, stated as partial.**

### D-B5: the death pact, its supervision and the default Stop (contradiction and proposed resolution)

**Owner decision (L2): the proposed resolution below is chosen**, the literal reading of P6 is not. The ADR wording is Appendix A (prepared, not accepted).

Three things were conflated in the first draft of this scope and stay distinct here: **(1)** generating the death-pact failure while interpreting the signal; **(2)** letting Typed supervision capture it;
**(3)** applying the default Stop when nothing captures it.

**Contradiction in the accepted texts.**
- ADR 0034 P6 (accepted): `OnTerminated` returns an error wrapping `actor.ErrDeathPact`; "the interpreter raises it **before the runtime sees it**, so a Typed supervisor can catch it" (B14). Read literally, `OnTerminated`
  *returns* the error and the runtime hands it to the parent's decider (`watch.go:505-509, 533`: `terminatedFailed` -> `failUp`).
- ADR 0035 M-B (accepted): a Typed actor started by Classic or the system is wrapped in `supervise(stop)`; "the **child** stops itself; the Classic parent's strategy is **not consulted**" (`0035:40`). The Go
  interpreter implements that as `StopSelf` and `return nil` (`interpreter.go`, `Receive`).
- The first draft said the default decider "stops on `ErrDeathPact`" (`supervision_decider.go:59`) and treated that as the same outcome. It is not: "the default decider stops" describes **one** decider; "the decider is
  not consulted" describes the **mechanism**. They coincide only while the parent's decider is the default. A custom Classic decider that answers `Restart` separates them.

**Pekko evidence (READ, v1.7.1; nothing executed).** `interpretSignal` throws `DeathPactException` inside the interpretation "to allow supervision of deathpact exception" (`Behavior.scala:270-277`); Typed supervision can
capture it (`SupervisionSpec.scala:1408`, `onFailure[DeathPactException]`). When nothing captures it, a Typed actor started from Classic or the system carries `Behaviors.supervise(behavior).onFailure(SupervisorStrategy.stop)`
(`scaladsl/adapter/PropsAdapter.scala:34`, `scaladsl/adapter/package.scala:60, 74`, `internal/adapter/ActorSystemAdapter.scala:130`), so the wrapper stops the child and a Classic parent's directive is never asked
(`0035:40`; `ClassicSupervisingTypedSpec.scala`, "default to stop for supervision", cited by the ADR, not re-read). A Typed parent's adapter strategy answers Stop for a Typed child (`ActorAdapter.scala:240-258`, M-A, outside
this scope). Order in Pekko: generated in interpretation -> offered to Typed supervision -> otherwise default Stop, without the Classic parent.

**Proposed resolution (Owner decision; it refines how P6 is read, not its purpose).**
1. *Generate*: the interpreter builds the failure (an error wrapping `actor.ErrDeathPact` that names the subject) while interpreting the signal.
2. *Offer to Typed supervision*: no Typed supervision exists in Go (#956 delivers the default Stop only; #976/#1063 the restart profile). **Slice B implements nothing for this step** (Owner, L2: no early implementation of the capture pending #976); #976 inserts it ahead of step 3.
3. *Default Stop*: when not captured, apply M-B: `ctx.StopSelf()` and `return nil`; the parent's decider is **not consulted**, `PostStop` runs once, the watcher of this actor sees a plain `Terminated`. The cause is not
   recorded until failure logging exists (as M-B today).

**Alternative (literal P6):** `OnTerminated` returns the error and the runtime consults the parent's decider. Simpler, but with a custom decider answering `Restart` goPekko restarts where Pekko stops: an observable
divergence that a test using only the default decider cannot see. If the Owner prefers it, the divergence is recorded and T7b is replaced by its mirror. **Recommend the proposed resolution**, with a note on ADR 0034 P6 in
the same small ADR PR as B-2. A handler error or panic while handling `Terminated` follows the same M-B path as a message turn.

## 4. Slice B behavior to deliver (contract)

**No representation of `Terminated` and no new export is fixed by this scope** (D-B1 is open). The contract below is stated in terms of "the identity of the terminated actor", whatever the Owner accepts.

`interpreter[T]` gains `OnTerminated(ctx *actor.Context[T], t actor.Terminated) error`:

1. **Handler present** on the active behavior -> call it with the signal built from `t` (the behavior current after the previous turn, as for `PostStop`) and apply the result with the **same rules as a message turn**
   (`Same` keeps; `Receive` replaces behavior and its handler; `Setup` starts; error, panic or non-receive result = failed turn, M-B stop). A handled `Terminated` never produces a death pact.
2. **No handler** on the active behavior -> *generate* the death-pact failure (an error wrapping `actor.ErrDeathPact` naming the subject) while interpreting the signal, and, because no Typed supervision exists to capture it, *apply the default Stop*
   (M-B, D-B5): `StopSelf`, return nil, the parent's decider **not consulted**. **No hook, attach point or export is added for #976's future capture** (Owner, L2): #976 inserts its capture before this default.
3. `Stopped`, `Unhandled` and `Failed` results do not exist yet (#1182, #1181/#925). **Partial delivery (D-B4):** an explicit decline of `Terminated` by a handler is pending #1181; a `TODO(TYP-04-edge)` marks the branch.
4. Never overlaps a message or `PostStop`: delivery stays on the runtime's single cell (B19); the interpreter adds no goroutine or lock.

Not delivered: Typed `Watch/WatchWith/Unwatch` (#1171, unless the Owner re-slices it, D-B1 B), `ChildFailed` and its cause (#1113 + M-A story), `PreRestart` (#976), `MessageAdaptionFailure` (#975), the Typed-supervision
capture of the death pact (#976), explicit decline of `Terminated` (#1181), watch cleanup at restart, cross-System watch.

## 5. Scenarios (Go unit, real runtime, no double; observe with channels and gates)

| # | Scenario | Kind | Pekko source | Evidence |
|---|---|---|---|---|
| T1 | Watched actor stops; the active handler gets one `Terminated` whose identity (through the accessor accepted in D-B1) equals the watchee's | positive | `WatchSpec.scala:88`; `BehaviorSpec.scala:226` | SPEC (derived) |
| T2 | Same, after a message handled before the termination | order | `BehaviorSpec.scala:230` | SPEC |
| T3 | A message sent after the termination is still handled after `Terminated` | order | `BehaviorSpec.scala:234` | SPEC |
| T4 | After `swap` (handler returned a new receive behavior), `Terminated` goes to the **new** behavior's handler | positive | `BehaviorSpec.scala:313-321` | SPEC |
| T5 | Handler returns a replacement behavior for `Terminated`; the next message uses it | result applied | `ActorAdapter.scala:163-180` | READ |
| T6 | Handler returns `Same` for `Terminated`: treated as handled, no death pact, actor keeps running. **Partial by design (D-B4): an explicit decline is pending #1181 and is not tested here** | negative | `Behavior.scala:270-277` (only `Unhandled` throws) | READ |
| T7a | Behavior with no signal handler, parent with the **default** decider: the actor stops, `PostStop` runs once, the watcher of this actor gets a plain `Terminated`, and no failure with `ErrDeathPact` reaches the decider (the count of decider calls is 0) | death pact (default Stop) | `Behavior.scala:270-277`; `ActorAdapter.scala:220-227`; `PropsAdapter.scala:34`. `ActorContextSpec.scala:483` and `WatchSpec.scala:165` exercise the explicit-decline and `ChildFailed` variants (pending #1181 and the M-A story) and do not pin this row; neither asserts the decider count or `PostStop` | READ |
| T7b | Same, but the parent is a Classic guardian with a **custom decider that would answer `Restart` for every cause**: the actor still stops (no restart, instance and state not replaced), the decider is not consulted (count 0). The Typed child is spawned by that parent with `actor.Spawn(owner, name, factory)` (`spawn.go:85`), where `owner` is the parent's context and `factory` returns `typed.NewActor(b)`; the decider is an exported `actor.Supervisor`/`actor.OneForOne` (`supervision.go:48, 98, 114`); no new export. The Dev confirms the exact call in RED. If the Owner picks the literal reading of P6 this row is replaced by its mirror (decider consulted, `Restart` applied) and recorded as a divergence | death pact vs custom decider | `scaladsl/adapter/PropsAdapter.scala:34` (wrapper `supervise(stop)`); `0035:40` | READ |
| T8 | Death pact on a handler-less receive after `Setup` start and after `swap` | death pact | `Behavior.scala:270-277` | READ |
| T9 | Handler panics or errors on `Terminated`: actor stops (M-B), `PostStop` runs once on the active behavior, `Done` closes | failure (D-B5) | `ActorAdapter.scala:163-180` | READ |
| T10 | Watching an already-dead actor delivers `Terminated` to the active handler. **Go assertion**: the Pekko spec's last assertion is the TestProbe's own death-watch (`probe.expectTerminated`) and never checks the actor's `ReceivedSignal`, so it does not pin signal delivery | positive | `ActorContextSpec.scala:407` (READ, asserts less) | READ, pending Owner approval (ADR 0033 D13) |
| T11 | Watching twice yields one `Terminated` (the spec does not assert the count; the count is a Go assertion on the runtime's idempotence, `0010:151`) | idempotence | `WatchSpec.scala:206` | READ (spec asserts less) |
| T12 | **No overlap:** a message handler holds a gate, the watchee stops; `Terminated` runs only after the handler returns; an in-flight counter shared by both handlers never exceeds 1 | race/order | ADR 0034 B19 | READ |
| T13 | **Regression of the Go runtime order:** N messages queued before the notification are all handled before `Terminated`; fixed N, assert the invariant, not timing. This pins the goPekko runtime (the notification is queued at the tail of the mailbox and takes a slot, `watch.go:266-375`); **it is not presented as Pekko equivalence**: no Pekko spec or source cited here asserts mailbox position for a `Terminated` (the cited `DeathWatch.scala:99-119` shows how Classic enqueues it, read, not a spec of its ordering) | order (Go regression) | none found; `watch.go` (READ) | REGRESSION, no oracle |
| T14 | **Terminated vs stop:** the watcher is stopped while the notification is queued: no `Terminated` handler call after `PostStop`, `PostStop` exactly once, no panic | race | runtime drops a notification for a stopping watcher (`0010:171`) | READ |
| T15 | **Regression, Slice A:** S1-S10, #847 S1-S11 and #956 S1-S10 still pass; `PostStop` is still exactly once with a `Terminated` handler present | regression | | existing tests |
| T16 | Public surface: only what the Owner accepts in D-B1 is added; nothing from `package actor`; the `Terminated` representation is not published before that decision | contract | | review/API diff |

T13 is a Go-runtime regression and T14 rests on runtime ordering read in `watch.go`; if RED shows either is nondeterministic, the Dev stops and reports (no sleep, no weakening). Equivalence of a Go test to the Pekko spec is
**not** claimed: parity is not claimed from local tests.

## 6. Dependencies, as they really are

| Dependency | State | Effect |
|---|---|---|
| Slice A (#1236, `ab9b44c`) | integrated | consumed (`signal.go`, `interpreter.go`, `receiveB.onSignal`) |
| ADR 0034 B-2 | **proposed** | blocks Slice B until accepted (D-B1) |
| #1171 (TYP-37) | open, blocked **by #954** and #827 | not consumed; consumes Slice B |
| #925, #1181 (TYP-47), #1182 (TYP-48) | open, blocked | `Unhandled` and `Stopped` absent. D-B4 delivers **partially**; explicit decline of `Terminated` is pending #1181 |
| #1113 (SUP-17); M-A story [#1250](https://github.com/getsyntegrity/gopekko/issues/1250) (`TYP-52`) | open / created 2026-10-28 | `ChildFailed` only; not Slice B. #1250 `depends_on` #954 Slice B, #956, #1170, #1171, #1113; nothing upstream waits on it, so no cycle. Duplicate check before creation: no open or closed issue other than #1113, #954 and #1020 mentions `ChildFailed`. Note: #1113 and the #954 reconciliation text still say `ChildFailed` belongs to #954; not edited, they need a coordinated Owner correction |
| #976 (PreRestart + children), #1170 (Typed spawn) | open | unchanged by Slice B; the no-op `PreRestarter` and its temporary child treatment from D-4 remain until #976 |
| #956 | scope in `develop` | Slice B keeps its M-B rule for failed turns |
| #827 (DTH-02) | open | #1171's dependency, not Slice B's |

## 7. Allowed files (proposed; valid only after G2)

| Path | Edit | PR |
|---|---|---|
| `docs/scopes/954-slice-b-terminated-death-pact.md` | new (this document) | B0 |
| `docs/adr/0034-typed-behavior-boundary.md` | one amendment recording the accepted B-2 option, **only after the Owner accepts D-B1** | B0b |
| `migration/stage-1/typed-954-slice-b/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | new kit package; each Pekko spec row of §2 ported, adapted, moved or `PORT-GAP` | B1 |
| `pekko-actor-typed/typed/signal.go`, `interpreter.go` | edit (keep `Ported from` lines); the `Terminated` representation only as accepted in D-B1 | B2 |
| `pekko-actor-typed/typed/signal_term_bhv_test.go` (T1-T4, derived from `BehaviorSpec`; carries its `Ported from` line), `signal_term_internal_test.go` (T5-T14) | new | B2 / B3 |
| `pekko-actor-typed/typed/example_test.go`, `pekko-actor-typed/README.md` | no example is possible without a Typed watch; the README states what is implemented, that no production caller can watch before #1171, the D-B4 partial delivery and trap, and the accepted D-B1 divergence | B3 |
| `migration/stage-1/typed-954-slice-b/EXECUTION-RECORD.md`, `migration/cost-log.tsv` (one row) | commands, kit `cf91c9d`, models, outputs, gates bound to a SHA | B3 |

Nothing under `pekko-actor/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum`, the root `README.md`, `docs/scopes/954-typed-lifecycle-signals.md` or `docs/scopes/956-*` changes.
Sequencing: Slice B edits `signal.go` and `interpreter.go` with #925, #1182, #976 and #1178; they run after or between each other, never in parallel.

## 8. Evidence, gates and PR split (<= 400 authored lines each)

Gates (commands in `AGENTS.md`, "Go"): `pekko-actor-typed` unit tests with network isolation (`unshare` is Linux-only: on this host use the container variant), `go vet`, `gofmt -l`, `golangci-lint` (`gocyclo` <= 10, not raised),
`go mod tidy -diff`, module coverage and diff coverage >= 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `go-sdk-update.sh --check`. Once per final tree, each result bound to its SHA.
Remote CI is blocked by billing; local results are not presented as approved CI. Draft PRs `Refs #954`, never `Closes`.

| PR | Content | Estimate (authored lines) |
|---|---|---|
| B0 | this scope (docs only; the file is 320 lines at this revision, measured) | ~320 |
| B0b | ADR 0034 amendment (P5 representation + P6 wording of Appendix A, 0035:205 label), after G2 | ~60 |
| B1 | kit package (manifest, RULEBOOK, RUNBOOK) | ~250 |
| B2 | `OnTerminated` and the accepted `Terminated` representation (D-B1), tests T1-T11, T15, T16 | ~330 |
| B3 | T12-T14, README, execution record, cost-log | ~300 |

Estimates are forecasts, not measurements. Stop condition: if a PR is projected over 400, the TL moves tests to the next PR; also stop if a file outside the list must change, `pekko-actor`
needs an export, or a scenario fails and is not a Go defect (run it on Pekko first, `AGENTS.md`). Never skip, delete or weaken a test.

## 9. Definition of Done (Slice B)

- T1-T16 pass at the final SHA; no function above complexity 10; gates green and bound to the SHA; the record states which rows are READ and pending approval.
- README and example match the code; no claim that #954, #1171, `ChildFailed` or Typed parity is done.
- A new QA session, independent of TL and Dev, issues a verdict on each PR head. No merge without the Owner. #954 closes only when its Slice B criteria are met and the Owner closes it.

## 10. Single request for G2 (to be issued on this document's exact SHA after the independent QA verdict)

Taken by the Owner in L2 and reused as is: D-B2, D-B3, D-B4 (partial), D-B5 (behavior chosen), D-B6, and the creation of #1250. **Still to decide at G2:**

1. **D-B1**, the representation of `Terminated` (B-2): A (`ID`), B (`Handle`, needs the re-slice in D-B1 and would revoke D-B2), C or D. Recommendation: A, with the reasons and the additive path to a later `Handle()` in D-B1.
2. **Review and acceptance of the ADR 0034 P6 / B-2 amendment** wording of Appendix A, in its own small PR (B0b), including the 0035:205 label fix. Choosing the behavior (D-B5) is not accepting this text.
3. Confirmation that G2 here means: the Owner approves this scope at its exact SHA so a Dev may start **B1 (kit)** and, after the ADR PR is accepted, **B2/B3**. Silence approves nothing; Dev does not start without that approval.

## Appendix A. Draft of the ADR 0034 P6 amendment and its regression tests (proposal; `docs/adr/` is not edited here)

This is prepared text for the small ADR PR (B0b). Writing it does not accept it: the Owner has chosen the behavior (D-B5, L2) but has not reviewed this wording.

**Draft wording, replacing the second sentence of P6.** "If the active behavior has no signal handler when `Terminated` arrives (or `ChildFailed`, once the M-A story delivers it), the interpreter generates a failure that wraps
`actor.ErrDeathPact` and names the subject, while interpreting the signal (`Behavior.scala:270-277`). If a Typed supervision that captures that failure exists (ADR 0035, #976), it decides. Otherwise the interpreter applies the default
Stop of mechanism M-B: it stops the actor and returns no error to the runtime, so the parent's decider is not consulted, `PostStop` runs once and a watcher of this actor sees a plain `Terminated`. 'Before the runtime sees it' means
exactly this. Until #976 delivers it, no Typed supervision captures the failure. A handler that exists and handles the signal never produces a death pact. Other unhandled signals are dropped."

**Side effect to record in the same PR:** ADR 0035:205 still labels ADR 0034 "proposed" although its header is "partly accepted" (stale, found by this review).

**Regression tests the implementation PR must carry** (all Go unit, real runtime; they pin goPekko's behavior and claim no Pekko parity):

| # | Regression | Scenario |
|---|---|---|
| RG1 | the death pact never reaches the parent's decider | T7a, T7b: decider call count 0, default and custom (`Restart`) |
| RG2 | the handled `Terminated` never stops the actor | T5, T6 |
| RG3 | the failed-turn rule of #956 is unchanged when the signal handler fails | T9 and the existing #956 tests (T15) |
| RG4 | `PostStop` after a death pact runs once on the active behavior | T7a |
| RG5 | nothing is added to supervision: no hook, attach point or export for #976's capture | T16 (API diff) and review |

## Owner decisions already taken (L2, 2026-10-28) and how this revision records them

| Decision | Owner answer (L2) | Recorded as |
|---|---|---|
| D-B1 | not approved; compare `ID` and `Handle`; recommend; give the exact re-slice; **do not re-slice** | D-B1 below; no issue edited |
| D-B2 | accepted: `OnTerminated` without a public Typed watch; until #1171 it is **not a capability a production caller can use** | D-B2; README must say so (§7) |
| D-B3 | accepted: internal tests in `package typed`, no new export or seam | D-B3 |
| D-B4 | accepted: `Same` = handled for this partial delivery; explicit decline stays pending in #1181 and is **not declared covered** | D-B4 |
| D-B5 | chosen: generate the failure in interpretation; `StopSelf` when no Typed supervision captures it; decider not consulted; **no early #976 capture**; ADR 0034 P6 amendment prepared (Appendix A), not accepted | D-B5, §4, Appendix A |
| D-B6 | accepted, subject to the real budget of each PR | §8 |
| M-A story | created after a duplicate check: [#1250](https://github.com/getsyntegrity/gopekko/issues/1250) (`TYP-52`, `prio:P1`, `status:blocked`) | §6 |

## Log

L1 (2026-10-28, Owner, verbatim, Spanish):

> Preparar el siguiente paso de #954 (TYP-04), después de Slice A integrada. Objetivo: producir un scope revisable para desbloquear Slice B (`Terminated` y death pact), con decisiones concretas para el Owner. [...] Mantener #954 abierta y status:blocked mientras estos bloqueos sigan pendientes. Esta tarea autoriza investigación y preparación documental; no aprueba B-2 ni inicia implementación productiva, merge o cierre de issues. Entregar el scope, su SHA exacto, los bloqueos encontrados y una única solicitud de decisiones para G2. No inventar semántica ni reclamar paridad a partir de pruebas locales.

L2 (2026-10-28, Owner, verbatim, Spanish):

> Autorizo corregir T7b para usar actor.Spawn(owner, name, factory), conforme a la API real. Pedí a QA revisar únicamente ese delta y emitir el dictamen sobre el nuevo SHA.
>
> Decisiones del Owner:
>
> - D-B1: no apruebo todavía la representación ni las pérdidas enumeradas. Presentá una comparación de ID y Handle: operaciones conservadas, pérdidas inevitables, pérdidas recuperables y efecto sobre #1171. Incluí una recomendación y el alcance exacto de la redivisión propuesta. No redividir aún las issues.
> - D-B2: acepto OnTerminated sin agregar watch público de Typed. Debe quedar explícito que, hasta #1171, no constituye una capacidad utilizable por un llamador de producción.
> - D-B3: acepto tests internos en package typed, sin exports ni seams nuevos.
> - D-B4: acepto Same como manejado para esta entrega parcial. El rechazo explícito sigue pendiente en #1181 y no se declara cubierto.
> - D-B5: elijo generar el fallo en la interpretación y aplicar StopSelf si no existe una supervisión Typed que lo capture, sin consultar al decider Classic. No implementar anticipadamente la captura pendiente de #976. Preparar la enmienda de ADR 0034 P6 y sus pruebas de regresión; esta decisión no equivale a aceptar un texto de ADR todavía no revisado.
> - D-B6: acepto la división propuesta, sujeta al presupuesto real de cada PR.
> - Autorizo crear la story separada M-A/ChildFailed, comprobando antes que no exista un duplicado. Registrar alcance y dependencias sin introducir ciclos.
>
> Después de corregir T7b y resolver D-B1, presentá una solicitud final G2 sobre el SHA exacto, reutilizando estas decisiones.
>
> G2 sigue pendiente. No iniciar implementación, mergear ni cerrar issues. Validación local, sin esperar CI remoto verde.

L3 (2026-10-28, Owner, verbatim, Spanish):

> Apruebo G2 del scope #954 Slice B en PR #1249, SHA 183171b9afdbf967bbbe73d79ecfa68a3b636a5f, con este alcance:
>
> 1. D-B1: elijo A (ID), conservando D-B2. Acepto las diferencias documentadas respecto de Pekko para esta entrega. No autoriza redividir #954/#1171 ni afirmar paridad completa.
>
> 2. Mantengo las decisiones ya aprobadas D-B2 a D-B6 y la creación de #1250. No reabrirlas.
>
> 3. Autorizo asignar Dev para B1, el kit de migración, y preparar B0b, el PR separado de enmienda del ADR. El apéndice A todavía no queda aceptado: requiere revisión y aceptación explícita de su texto.
>
> 4. B2 y B3 solo pueden comenzar después de aceptada la enmienda del ADR. Mantener QA independiente.
>
> 5. Corregí "no existe lookup ID→Handle" por "no existe lookup exportado ID→Handle". Esta precisión queda autorizada y no cambia las decisiones G2. Registrá el SHA resultante y pedí revisar únicamente ese delta, sin reiniciar la aprobación completa.
>
> Registrá la aprobación y sus límites. Validación local, sin esperar CI remoto verde. Sin merge ni cierre de issues; #954 permanece abierta.

L4 (2026-10-28, Owner, verbatim, Spanish):

> Autorizo las tres: corregí T10 en #1249 y el cosmético de `:407`.

Applied after the G2 approval at `183171b`, as a reduction of an evidence claim: T10 is relabelled READ (it was "SPEC (derived, asserts less)"). No decision, scenario, file or contract changes. The kit manifest of #1252 already carried this label.

## Correction note (TL, 2026-10-28; not an Owner entry; nothing above is rewritten)

The Owner's wording of L2 and L3 above (for example "El rechazo explícito sigue pendiente en #1181") is quoted as written and is not changed. The attribution in it and in the scope text is corrected here: **the explicit decline of a signal (Pekko `Behaviors.unhandled`, the `Unhandled` result) belongs to #925, which keeps the unhandled marker. #1181 delivers `empty` and `ignore` over that marker and does not deliver the decline.** Where this scope says "pending #1181" for the decline (D-B4, the "Not delivered" lists, scenario T6, the dependency row, the decision table), read #925. What stays correct: "#925, #1181" as joint references to the unhandled/empty/ignore family; the quotation of #1181 R3 (it concerns the `empty` behavior failing on an unhandled `Terminated`, not a handler's decline). The correction is carried out in `docs/adr/0034-typed-behavior-boundary.md` (proposed section, pending the Owner), in the README and code comments (corrected in place) and in the execution record (section 10).

