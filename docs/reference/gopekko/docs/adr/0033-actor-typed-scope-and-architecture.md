# ADR 0033: Scope and architecture to complete Pekko `actor` and `actor-typed`

- Status: accepted by the Owner on 2026-10-24, in the working session, with the interpretation of D13 recorded below. The acceptance resolves only the scope and architecture decisions of this ADR. It does not accept or modify any other ADR, declares no parity, and keeps ADR 0032 D5 open. It does not authorize changes on GitHub or opening stories.
- Date: 2026-10-24
- Stage: 1
- Supporting evidence: [docs/actor-typed-completion-design.md](../actor-typed-completion-design.md) (inventory, gaps, judge plan, decisions still open).

## Feature

Complete the local behavior of the Apache Pekko modules `actor` and `actor-typed` in goPekko, with the Go adaptations explicitly recorded. This ADR fixes only the architecture and the scope rules that the Architect needs to cut slices. It does not claim parity and does not state that either module is complete.

## How Pekko solves it

- Module: `actor` (`upstream/pekko/build.sbt:117`, no project dependencies, external `config`) and `actor-typed` (`build.sbt:481`, `dependsOn(actor, slf4j)`, external `slf4j-api`). `actor-testkit-typed` (`build.sbt:579`) and `actor-typed-tests` (`build.sbt:584`) are test infrastructure.
- Concept: Typed runs on the Classic runtime. A `Behavior[T]` is interpreted by an actor; signals replace lifecycle methods; supervision wraps the behavior (default Stop) instead of being owned by the parent (default Restart).
- Files: `actor-typed/src/main/scala/org/apache/pekko/actor/typed/` (`Behavior.scala`, `scaladsl/Behaviors.scala`, `SupervisorStrategy.scala`, `internal/Supervision.scala`, `scaladsl/StashBuffer.scala`, `scaladsl/TimerScheduler.scala`, `receptionist/`, `delivery/`, `pubsub/`).
- Pin: tag `v1.7.1`, commit `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. Production dependencies stay `actor` plus a logging bridge; `actor-typed` has no source import of cluster, remote, stream or persistence.

## How GoAkt solves it

Not consulted for this decision. No ADR shows GoAkt solving a relevant feature better, and `CLAUDE.md` makes Pekko the default source. GoAkt is not adopted.

## Evidence

- Current state: `pekko-actor` implements the Classic local core under API style A3 (`pekko-actor/actor/ref.go:18`, `actor.go:19`, `spawn.go:85`). Typed has no code. Stash, timers, scheduler, event stream beyond dead letters, extensions, `become`, `PoisonPill` and `Kill` are absent (design dossier sections 4 and 11.2).
- Judge: covers the Classic local core only (`docs/judge.md`, `judge/pekko-harness/Judge.scala`). Nothing exists for Typed.
- ADR 0010 separates API shape (a) from semantics (b). It recommends A3 and keeps A2 (`Behavior[T]`) as a costed alternative; it fixes Classic semantics only for supervision (D3). It does not forbid a Typed layer.
- Issue tracking: the ACT2 program (epic #746) already holds Typed stories (#763, TYP-01 #809 "Typed Behavior boundary on the shared runtime").
- Complexity and idiom: not measured. The layer's allocation cost per message (closures returning behaviors, ADR 0010:56) is an open risk.

## Decision

The Owner accepted this ADR on 2026-10-24 in the working session, so D1 to D13 are accepted decisions. D13 carries the Owner's interpretation of D-10 given at acceptance. ADR-local ids map to the dossier ids in parentheses.

1. **D1 (D-01) API.** A3 stays the runtime and its API. A public `Behavior[T]` layer is added on top for Typed. It is implemented through the same runtime, mailbox and delivery path; it does not introduce a second runtime.
2. **D2 (D-02) Profiles.** Classic and Typed coexist as two profiles on the shared runtime. Classic keeps the defaults of ADR 0017. Typed uses Stop by default on failure. A profile is selected when the actor is created and does not change the behavior of the other profile. The Typed supervision contract (per-behavior `supervise`, restart limits, backoff, stash during backoff, `DeathPactException`) is not decided here: it needs its own ADR extending ADR 0017, which defers Typed (ADR 0017:89-117, 635-637).
3. **D3 (D-03) Module.** Typed lives in a separate Go module `pekko-actor-typed` that depends on `pekko-actor`. Its shared dependencies are defined and published in `pekko-actor` first. For each internal API that must be exported, the Architect justifies the export under ADR 0011 (owner decision 2: no exported internals without a concrete benefit). This is a new module-boundary decision. ADR 0011 rejected splitting the `actor` package; a separate module that consumes `pekko-actor` does not by itself split `pekko-actor`, so it is not treated as a contradiction of ADR 0011. The tension is narrower and concrete: any `pekko-actor` API that the new module needs and that is unexported today (candidates in dossier 11.2) must be justified individually against that owner decision. The module needs a `go.work` entry, one `scripts/deps.allow` line, a `go.mod` with a `replace`, a README meeting the module README contract, and an update of the dependency-direction text in `CLAUDE.md`.
4. **D4 (D-06) Mandatory capabilities.** Stash and timers are mandatory for completing Typed. The other ADR 0010 "Excluded for now" items (`become`/`unbecome`, receive timeout, routers, actor selection, `PoisonPill`, `Kill`, dispatcher and mailbox configuration) are not made mandatory in block: each is reviewed against the inventory and classified individually.

   *Note 2026-11-04 (additive; the text of D4 above and the accepted decision are unchanged).* Among the ADR 0010 "Excluded for now" items that D4 sends to individual classification, these are already classified as **included** by Owner decisions of 2026-11-04: `Kill` and `PoisonPill` (ADR 0010 amendment 2026-11-04, `3fee182`, #1318) and `ActorSelection` and `Identify` (ADR 0010 amendment 2026-11-04 (b), `42cf138`, #1325). The remaining items (`become`/`unbecome`, receive timeout, routers, dispatcher and mailbox configuration) are still pending individual classification. This note declares no parity; evidence follows D13.
5. **D5 (D-11) ADR hygiene.** Pending ADRs are not accepted in block. Before implementing each slice, the Architect lists the ADRs that affect the slice's contract and presents their contradictions for resolution. The contradictions already found are listed in the design dossier (sections 2.1 and 11.4).
6. **D6 (D-12) ADR 0032 D5.** D5 (sender of `Terminated` and the `WatchWith` message) stays open until its alternatives and evidence are reviewed (resolved with #814). It blocks the Classic sender contract for `Terminated`/`WatchWith`. It is not treated as blocking Typed unless a slice demonstrates that dependency.
7. **D7 Meaning of "100%".** "100%" means coverage of the local capabilities included in the inventory matrix, with the distributed-systems exclusions and the JVM-only exclusions enumerated explicitly. It is not total parity with Apache Pekko, which includes cluster and remoting. Every module README and every closure report states this.
8. **D8 (D-04) `actor` scope.** The local public capabilities of `actor` are ported or adapted; the scope is not limited to the core. Every subpackage is reviewed in the inventory. Local `serialization` capabilities with an applicable Go contract are included (the Pekko `Serialization` extension stays replaced by `pekko-serialization`). `actor.io` is excluded where it depends on sockets, protocols or network transport, and no adapter over `net` is built unless the inventory identifies a concrete local capability that justifies it. Every exclusion is recorded with its source and reason.
9. **D9 (D-05) Typed services.** Receptionist, routers, pubsub/Topic, `SpawnProtocol` and delivery are included in their local modes. The distributed parts and the durable queue implementations that depend on persistence are excluded.
10. **D10 (D-06, D-07) Mandatory capabilities and testkit.** Stash and timers are mandatory (D4). The testkit subset is `TestProbe`, `BehaviorTestKit`/`TestInbox`, `ActorTestKit` and manual time. Tools tied to JUnit, logback and JVM-specific APIs are excluded.
11. **D11 (D-08) Logging.** Logging context uses `kit-logger` `With` per actor and does not depend on global fields. The test fake must capture `With` fields, honor `SetLevel` and guard `Entries` against concurrent access; the shipped `kitlogtest.MockLogger` does none of these.
12. **D12 (D-09) Adapters.** Classic↔Typed adapters are resolved one by one, one ADR each. `internal/adapter` is not ported as a block. Each included adapter needs a local contract, tests and a justification.
13. **D13 (D-10, D-13) Closure and tooling.** The exclusion list is closed and limited to the agreed local frontier and documented JVM-only APIs. V3 requires a differential scenario against Pekko whenever an applicable oracle exists. For adaptations that can only be validated in Go (no applicable Pekko oracle, for example Go-only adaptations and S-tier relations), this ADR requires documented alternative evidence and explicit Owner approval. Such a row is never marked V3 by default and its level is never lowered silently; while the approval is missing, the row is recorded as pending approval. The inventory carries, per such row, the alternative evidence and the approval reference. The actor inventory and the judge are created first and are actor-specific; shared parity tooling is extended afterwards only if it can represent both domains without distorting either.

## Not decided here

The per-capability classification of the inventory (each exclusion needs a source and a reason); the Typed supervision contract; the `Behavior[T]` boundary; the validation clause for rows without a Pekko oracle (D13); and the individual Classic↔Typed adapters (D12). These belong to the inventory and to follow-up ADRs.

## Options considered for D8 to D13

Each row lists the options that were considered (dossier ids). The accepted decision for each is in D8 to D13 above. The "Reviewer orientation" column is the earlier review recommendation, not an Owner answer. Detail and evidence: dossier sections 4, 5, 7 and 8.

| Id | Question | Alternatives | Consequence of each | Reviewer orientation (not an answer) |
|---|---|---|---|---|
| D-04 | Scope of `actor` per subpackage (`event`, `pattern`, `routing`, `io`, `serialization`, `japi`, `util`, top-level files, `reference.conf`) | (a) port or adapt per the dossier 4.3 classification, exclude with a recorded reason; (b) also build an actor adapter over `net` for `io`; (c) core only | (a) closes `pattern`, a `routing` subset and an `event` adaptation, and excludes JVM-only and Java-only parts with ADR-backed reasons; (b) adds about 7k upstream lines of `io` design for no Go-specific gain; (c) leaves Typed services and patterns uncovered | (a): include local capabilities of `actor` unless a documented exclusion applies |
| D-05 | Local Receptionist, routers, pubsub `Topic`, reliable delivery, `SpawnProtocol` | in (local only); defer; out | in: Typed services covered, delivery adds 3.5k upstream lines and a durable-queue SPI question; defer: closure waits; out: those rows become exclusions that need ADR reasons | in for the local parts of Receptionist, routers, pubsub and `SpawnProtocol`; exclude only the cluster, remoting and durable-queue parts |
| D-07 | Testkit scope | out; needed subset (`TestProbe`, `BehaviorTestKit`/`TestInbox`, `ActorTestKit`, manual time); full | out: each test re-invents probes; subset: matches what `actor-typed-tests` imports; full: ports logback and JUnit tooling with no Go meaning | subset sufficient to verify the included APIs |
| D-08 | Logging and MDC mapping onto `kit-logger` | global fields; per-actor `With`; purpose-built fake for tests | the `kit-logger` mock drops `With` arguments, ignores `SetLevel` and exposes `Entries` unlocked, so per-actor fields cannot be asserted with it; real `Sampling`/`RateLimit` can drop entries | `kit-logger` with a documented mapping, tested through a purpose-built fake |
| D-09 | Classic↔Typed adapters and coexistence | none in the first delivery; per-adapter ADR; full port of `internal/adapter` | none: simplest, one runtime; per-adapter: explicit cost per conversion; full: about 1.1k upstream lines for a layer Go does not have | decide each adapter explicitly, none by default |
| D-10 | Acceptable exclusions for "100%" and the minimum validation level | closed exclusion list (dossier 7.1); levels V1, V2, V3 (dossier 7.2) | a weak bar lets rows go green on unit tests alone; V2 needs a judge scenario and a detecting mutation per row | contract tests and a judge scenario per mandatory capability |
| D-13 | Extend the parity-guard tooling (`capabilities.json`) to actors, or keep a separate inventory | extend; separate document first | extending touches unapproved tool design (Slice B to E of `docs/parity/proposal.md` have not started) | separate document first |

## Consequences

- A new, Typed-oriented judge family is needed: a typed Pekko harness beside `Judge.scala`, new event kinds and clauses in `judge/comparator.py`, new matrix rows in `migration/stage-1/milestone-1.md`, and a Go typed surface. Design can start now; execution needs the layer.
- Classic is currently the only normative source in the repository (ADR 0010 D3, `migration/stage-1/feasibility.md:37,227`, fidelity audit line 9). With this ADR accepted, Typed is a normative source for the Typed profile only; Classic remains the normative source for the Classic profile.
- Shared foundations come first in `pekko-actor`: an injectable clock or scheduler seam, timers, stash with a replay ordering rule, an event stream with `Dropped` and `UnhandledMessage`, `Context.Log`, and a way to spawn system actors. The dossier (11.2) lists each candidate and why the public API does or does not already suffice. None is pre-approved as an export.
- Observable differences to record and test: default supervision per profile; `Receive` returns `error`; explicit sender and reply references; `Terminated` as a hook in Classic and as a signal in Typed; futures replaced by `context.Context` deadlines; typed Go options instead of HOCON; local-only `Address`/`ActorPath`.
- Existing contradictions are not resolved by this ADR: sender (ADR 0010 D2, 0017, 0030 P-1 versus 0032), `AskFrom` (ADR 0010 D4 versus 0015 D1/D7), ADR 0008 status, and ADR 0011 versus 0021 on `pekko-serialization` imports. The module boundary of D3 is a new decision with a narrow tension with ADR 0011 (described in D3), not a contradiction listed here. D5 of this ADR requires them to be presented per slice.
- Out of scope remains: cluster, remoting, persistence, streams, `actor-cluster-typed`, `persistence-typed`, and durable delivery queue implementations.
- Follow-up ADRs required before the corresponding slices: Typed supervision (extends 0017); Typed `Behavior[T]` boundary (TYP-01 #809); the public extension surface of `pekko-actor` (ADR 0011 justification); the per-item review of the ADR 0010 exclusions; the inventory format and its validation levels.
- This ADR changes no code, no gate and no other ADR. Number check on 2026-10-24 after `git fetch`: `origin/develop` at `a3aa35d` contains ADRs up to 0031; no remote-tracking ref contains `docs/adr/0033-*`; a PR search for "ADR 0033" returned none; ADR 0032 exists only on the branch of PR #1014. 0033 must be re-checked against the then-current base when this ADR is published, and renumbered if it has been taken.

## Amendment (accepted): D14, guiding principle of compatibility with Pekko

- Status: **Accepted** by the Owner on 2026-11-04. Additive: the text of D1 to D13 and their acceptance do not change.
- Source of the decision: Owner text of 2026-11-04 ("every functional decision must behave exactly as Apache Pekko").

### Context

D13 already requires a Pekko oracle for V3 validation and D7 bounds the scope of "100%". There is no general written rule for what to do
when a behavior is in doubt. Without it each ADR repeats the criterion or decides it differently.

### Decision

1. **D14.1 General rule.** Every functional decision behaves exactly as Apache Pekko.
2. **D14.2 Source of truth.** When a behavior is in doubt, Pekko is the source of truth: first its official documentation
   (<https://pekko.apache.org/docs/pekko/current/>); if the documentation is not enough, its source code in `upstream/pekko` (tag `v1.7.1`).
   If the two differ, the code prevails and the difference is noted in the ADR (as D-b of ADR 0035 does).
   *(D14.2 in full is an Architect clarification, ratified by Owner 2026-11-05.)*
3. **D14.3 Deviations.** Every deviation needs its own ADR. The ADR justifies why it departs from Pekko and cites the evidence.
   A deviation without an ADR is a defect, not a decision.
   *(D14.3 in full is an Architect clarification, ratified by Owner 2026-11-05.)*
4. **D14.4 Scope.** ADRs from 0035 onward apply this principle. Earlier ADRs are not rewritten; their differences from Pekko are
   handled under D14.3 when the topic is touched.
   *(D14.4 is an Architect clarification, ratified by Owner 2026-11-05. Only D14.1 follows the Owner's wording ("toda decisión funcional debe ser tal cual apache pekko"); D14.2 and D14.3 are Architect clarifications too, marked above. D15.3 relies on D14.3.)*

### Consequences

- A design question whose answer exists in Pekko is closed by citing the source. It does not go through an "own" alternative.
- D13 and D14 are compatible: D13 sets the validation level; D14 sets the behavior criterion.
- No change to code, gates or other ADRs. No parity claim.

### References

- <https://pekko.apache.org/docs/pekko/current/typed/fault-tolerance.html>
- <https://pekko.apache.org/docs/pekko/current/typed/actor-lifecycle.html>

## Amendment (accepted): D15, upstream-acknowledged bugs

- Status: **Accepted** by the Owner on 2026-11-04. Additive: the text of D1 to D14 and their acceptance do not change.
- Source of the decision: Owner decision of 2026-11-04, taken with the D-a oracle evidence (`docs/parity/oracles/d-a-restart-setup-unbounded-2026-11-04.md`).

### Context

D14 makes Pekko the source of truth. Pekko 1.7.1 also contains defects that the maintainers have acknowledged and fixed in later branches.
Copying such a defect is not "behaving as Pekko" in any useful sense; waiting for a published release would block the work. The case that
raised this is D-a (ADR 0035): with `jvm-exit-on-fatal-error = off` Pekko 1.7.1 hangs because of apache/pekko#3620, fixed after 1.7.1.

### Decision

1. **D15.1 Rule.** When Pekko behavior is acknowledged upstream as a bug (an apache/pekko issue labeled `bug` with a merged fix in `main` or in
   a maintained release branch), gopekko follows the **fixed** behavior, even if no published release contains the fix yet.
2. **D15.2 Evidence.** The ADR or decision that applies D15 links the issue and the fix PR(s), and states the Pekko commit or build against which
   the fixed behavior was verified (oracle).
3. **D15.3 Not a deviation.** Applying D15 is not a deviation under D14 and needs no deviation ADR.
4. **D15.4 Revisit.** If the upstream fix is later reverted or changed, the decision that applied D15 is revisited.

### Consequences

- A bug acknowledged and fixed upstream is not copied. The fixed behavior is verified by an oracle built from the fixing commit.
- A defect without an acknowledged upstream bug (issue labeled `bug` and merged fix) is still Pekko behavior under D14.
- No change to code, gates or other ADRs. No parity claim.

## Amendment (2026-11-05): scope of D14

- Status: **Accepted** by the Owner on 2026-11-05. Additive: the text of D1 to D15 and their acceptance do not change.
- Source of the decision: Owner wording of the original D14 request, "como regla general" (see the Owner table, Resolutions of 2026-11-05, `Port`).
- Reason: D14.4 limits D14 by ADR number ("ADRs from 0035 onward") and says nothing about modules, while this ADR is scoped to `actor` and
  `actor-typed` and lists persistence as out of its scope. The reading was ambiguous.

### Decision

D14 is a project-wide rule: it applies to every gopekko module, including persistence (ADR 0018), regardless of the scope of this ADR.
D14.4 (which ADRs apply it) is unchanged.

## Amendment (accepted): the V scale and what V2 and V3 require per row (D-10, D13, O-1)

- Status: **Accepted** by the Owner on 2026-11-07 (decision on [#1467](https://github.com/getsyntegrity/gopekko/issues/1467): the strict rule). Additive: the text of D1 to D15 and their acceptance do not change. It resolves O-1 of `docs/actor-typed-delivery-plan.md`. **Corrected on 2026-11-07 by the Owner** (points 2, 3 and 6): the scale is cumulative; the first text kept PAT and PS outside V2. **Clarified on 2026-11-08 by the Owner** (point 7): the contract of a row is what the body of the Pekko spec asserts.
- Source of the decision: Owner wording of 2026-11-07, "regla estricta" of #1467; every functional decision stays "tal cual Apache Pekko" (D14.1).

### Context

D-10 asks for a minimum validation level per row and D13 fixes only the oracle condition of V3. O-1 kept V0 to V3 as descriptive labels until the Owner approved a scale. A literal reading of "a judge scenario and a detecting mutation per row" lets a row that names several capabilities reach V2 on one of them (the evidence of #1466 for C-01..C-02, C-04..C-05, C-06, C-08, C-10 and D-01 shows it: each has scenarios and detecting mutations, none covers every contract of the row).

### Decision

1. **Contract of a row.** A contract is one behavior that Apache Pekko asserts in its specs, cited with `path:line` of `upstream/pekko` at `v1.7.1`. A row is covered when each of its contracts is.
2. **V2.** A row is V2 when it is V1 and all of these hold: (a) a scenario of the Go judge (`judge/go-scope.json`) that passes; (b) at least one designed Go mutation (`judge/mutate_go.py`) that the scenario detects by its designed clause, CURRENT in `judge/go-mutation-report.tsv` (the checker `scripts/check-judge-matrix.py` reports it); (c) every contract of the row is covered by (a) and (b). If a contract is not covered, the row is split into sub-rows, one per contract, and only the sub-rows that meet (a) to (c) are V2; the others stay `unassigned` with what they lack written in the row.
3. **V3.** A row is V3 when it is V2 (so V1 and V0 too) and a differential scenario passes on Apache Pekko `v1.7.1`, with Pekko-side evidence bound to a commit and CURRENT for the specs of the judge as they are.
4. **No applicable Pekko oracle.** The row stays `pending approval` (D13). It is never V3 by default and its level is never lowered silently. D13 applies only to rows without a Pekko oracle: a row whose plan evidence kind is "5.2: -" for another reason is not `pending approval`.
5. **Sub-rows.** A sub-row has a derived id (`C-08a`, `C-08b`...); the parent row stays in the inventory as a grouper and carries no level of its own.
6. **The scale is cumulative.** V2 requires V1, V1 requires V0 and V3 requires V2. V0: a Go test of the public API that asserts every observable effect of the row's contracts (the PAT evidence kind). V1: V0 plus a Pekko spec ported or adapted to Go, cited by name and `path:line` of `upstream/pekko`, with its PORT-GAPs listed (the PS evidence kind); a test that does not cite a Pekko spec is not PS. V2 and V3 as in 2 and 3. Until a row meets the rule its "V label" stays `unassigned`, and what it lacks (PAT, PS, scenario, mutation, contract) is written in the row.

7. **The body asserts, the title does not (Owner, 2026-11-08).** The contract of a row (point 1) is what the body of the Pekko spec asserts. The title of a test is not an assertion: a promise in a title ("only one") that the body does not check is not part of the contract and does not condition the level. A Go PAT or PS that asserts more than the Pekko body (for example "exactly one") is valid and conservative, and does not condition the level either.

### Consequences

- The plan's V0 and V1 stand as written. The plan's restated V2 ("V1 plus a Go-side scenario with a detecting mutation") stands and gains the contract coverage of point 2; its V3 (which also asks for `-race` and the module-close campaign) is superseded by point 3. The judge already runs with `-race`.
- No row is V2 or V3 by this amendment alone; levels are assigned only by an inventory change that cites the evidence.
- No change to code, gates or other ADRs. No parity claim.
