# goPekko Stage 1 — Feasibility report

> **Historical (Owner scope change, 2026-10-17):** goPekko has no tenancy module. References to tenancy, `tenancy.Owner` or the tenant guard in this file describe the scope at the time and are not current requirements.


Date: 2026-10-08. Prompt: `migration-kit/prompts/00-feasibility.md`. Target language: Go. Kit path: `migration-kit`.
Scope: Pekko v1.7.1 (`upstream/pekko`, read-only), packages `actor`, `dispatch`, `event`, `pattern`, `routing`, `serialization` under `actor/src/main/scala/org/apache/pekko/`. `io/` and `util/` are out, except what the rest needs. Target modules: `pekko-actor`, `pekko-serialization`, `pekko-observability`, `pekko-tenancy`.
Read-only toward the codebase under survey. The only writes are under `migration/` (this report, raw measurements, the cost log). One build was run.

Conventions: a claim carries a path or a count. **(A#)** marks an assumption I could not verify; they are listed in §10.

> **Amended.** The parity criterion, part of the §3.3 seed list, the scope of the first implementation and the gate status changed after review. The original text below is kept as written; see [Amendments](#amendments) for what changed and why.

## 0. The six steps (restated from `migration-kit/README.md`)

1. **Create the map and the rules** — dependency map orders the work, gap inventory traces what Go demands, a rulebook (here: design doc + ADRs) decides each question once.
2. **Stress-test the rules** — bakeoff and pilot on nasty files; only rule changes survive.
3. **Translate everything** — implementer, two adversarial reviewers and a fixer per unit, fanned out over a disk queue.
4. **Compile** — one survey build grades everything; fixers work the error queue without compiler access.
5. **Run it** — hello world, then the smallest end-to-end command.
6. **Match behavior** — a judge that runs against old and new code through the public surface; old code is the executable spec.

For a redesign (README "If you're redesigning"): the rulebook becomes a design document, the bakeoff is invalid, the unit of work is a module or subsystem, behavior matching is unchanged.

## 1. What was surveyed

| Measure | Value | Source |
|---|---|---|
| In-scope Scala files | 111 | `measure.txt` |
| In-scope non-blank lines | 31,989 (of which 14,551 comment/scaladoc lines; 17,438 code lines) | `measure.txt` |
| Per package (files / non-blank lines) | actor 37 / 13,375 · dispatch 18 / 4,554 · event 11 / 3,211 · pattern 19 / 5,986 · routing 17 / 3,381 · serialization 9 / 1,482 | `measure.txt` |
| Out-of-scope files the scope needs | 20 in `util/`, 2 in the package root (`Done`, `PekkoException`), 1 in `japi/` (`JavaAPI`), **0 in `io/`** | `scopedeps.txt` |
| Out-of-scope files that depend on the scope | 28 in `io/`, 3 in `util/`, 1 root | `scopedeps.txt` |
| Whole source tree | 199 Scala files + 30 Java files under `actor/src/main` | depmap, `measure.txt` |
| Test corpus for the scope | 102 Scala files, ≈889 test cases, in `actor-tests/src/test/scala/org/apache/pekko/{actor,dispatch,event,pattern,routing,serialization}` | `test-classification.tsv` |

Skipped, and why: `io/` (28 files depend on actor, not the reverse; Go stdlib covers it), `util/` beyond the 20 needed files, `japi/`, the typed API (`actor-typed` is a separate module and was not in the scope you gave; the classic `org.apache.pekko.actor` API is the reference **(A1)**), the Java sources under `actor/src/main/java` (30 files, mostly annotations and abstract Java-facing classes), and every other Pekko module (remote, cluster, persistence, stream). I did not read the 111 files in full; counts come from greps and the dependency mapper, and I read in detail only the files named in §3.

## 2. The case for leaving (pain instances in this code)

Evidence from the in-scope source, not from the language's reputation:

1. **Cross-version source forks.** The same library is maintained as parallel sources for several Scala versions: 33 files under `actor/src/main/{scala-2, scala-2.12, scala-2.13, scala-2.13+, scala-3, scala-jdk-9}` (counts: 1 / 12 / 7 / 4 / 8 / 1, ≈7.2k lines). Example: `ByteString.scala` exists in both `scala-2.12/` and `scala-3/`. Go has one compiler and no such fork.
2. **The build refuses to run without a specific JDK.** `sbt actor/compile` failed in 1 s on a machine with JDK 11 and no JDK 8: "A JDK 8 installation was not found, but is required to build Apache Pekko" (first run of `build-baseline.log`). Toolchain tax paid before a single line compiles.
3. **Visibility enforced by convention.** 348 lines use `private[pekko]` and 108 use `@InternalApi` in the scope. Anything an application can import is effectively public API; Go's `internal/` makes the boundary a compiler fact.
4. **A second, Java-facing API.** 30 Java files plus `AbstractActor`, `AbstractFSM`, `AbstractProps` in `actor/` duplicate the Scala surface for Java users. A Go port has one surface.
5. **Hand-rolled concurrency around the JVM memory model.** 49 lines of `Unsafe`/field updaters, 23 `synchronized`, 36 `@volatile`; `dispatch/Mailbox.scala` is 1,046 lines of lock and bit-state management.
6. **Async style that is not Go's.** 212 lines of `Future[`/`Promise[` and 194 `implicit` lines; 163 `asInstanceOf`.

Honest reading: these are real costs of keeping Pekko's design, but they argue for *redesign in Go* more than for *leaving Pekko*. The reason to leave is outside this repo: the rest of the product is Go (`kit-logger`, `go-specs`, GoAkt as a reference) **(A2)**. If that premise is wrong, the case above is weaker than it looks.

## 3. Three calls

### 3.1 Structure-preserving or redesign? — **Redesign.**

Files that forced it:

- `dispatch/Mailbox.scala` (1,046 lines), `dispatch/AbstractDispatcher.scala` (542), `dispatch/Dispatcher.scala` (183), `dispatch/ForkJoinExecutorConfigurator.scala`, `dispatch/ThreadPoolBuilder.scala`, `dispatch/VirtualThreadSupport.scala`: the model is a pool of threads draining mailboxes with a lock and status bits. A one-to-one port would rebuild a thread-pool scheduler on top of goroutines; the idiomatic form is a goroutine and channel (or queue) per actor.
- `actor/ActorCell.scala` (734), `actor/FaultHandling.scala` (678) and `actor/dungeon/*.scala` (1,675 lines across 7 files, including `DeathWatch.scala` 265 and `FaultHandling.scala` 360): the cell mixes suspend counters, system messages and children containers because the mailbox is shared across threads. With one goroutine owning the actor, most of that state machine changes shape.
- `pattern/AskSupport.scala` and the 212 `Future`/`Promise` lines: `Future` maps to `context.Context` plus a reply channel, not to a generic future type.
- 348 `private[pekko]` lines: there is no Go equivalent; they become `internal/` and `spi/` placement decisions (`docs/design.md`).
- 54 reflection lines (`DynamicAccess`, `Class.forName`, `createInstanceFor`) behind `Props`: no reflection-based instantiation in idiomatic Go; they become factory functions.

### 3.2 What does verification cost?

- **Baseline (measured):** `sbt -batch actor/compile` → success in **19 s wall-clock** (sbt's reported 14 s), compiling 214 Scala and 30 Java sources. Command: `JAVA_8_HOME=<zulu-8> sbt -batch actor/compile` under JDK 11 (sbt 1.12.14). Raw: `build-baseline.log`, `build-baseline.time`.
  - This is **not a cold-machine figure**: the sbt launcher and the build definition were already warm from the failed first attempt. Dependencies of `actor` are few, so I expect the cold figure to be tens of seconds higher **(A3)**.
  - Typecheck is not a separate command in sbt; `compile` is the typecheck. I did not run `actor-tests/Test/compile` or any test (the prompt forbids test runs), so the test-side build time is **unknown**.
- **Scaling assumption for Go (A4):** `go build` and `go vet` scale roughly linearly with lines and parallelise per package. For the ≈20–30k Go lines I expect a module to build in seconds, not minutes. Not measured: today the four modules contain only `doc.go` files.
- **The referee that costs is the judge, not the compiler.** A parity run needs a JVM harness (sbt start-up per run) plus the Go harness, and timing-sensitive scenarios (restart windows, receive timeouts, schedulers: `SchedulerSpec` alone has 39 cases) need a controllable clock.

**Committed line: verification is cheap on the Go side (`go vet`/`go build` fit inside every unit's loop, so README Step 4 dissolves into Step 3) and expensive on the judge side; the files that force this are `dispatch/Mailbox.scala` and `actor/SchedulerSpec`-style timing tests, i.e. everything whose behavior depends on time and scheduling.**

### 3.3 Do the tests survive? — **Scenarios survive; the code does not. The judge does not exist yet.**

Every test in scope is ScalaTest running in-process on the JVM, so none of them runs against Go. "Survives" therefore means: the *scenario* exercises only public behavior and can be re-expressed in a language-neutral form. Classification is a heuristic by identifier (`classify_tests.sh`): a file is "internals" if it mentions `ActorCell`, `dispatch.sysmsg`, `LocalActorRef`, `@InternalApi`, `private[pekko]`, `ExtendedActorSystem`, `SystemMessage`, `Mailbox` and similar. I refined it by counting marker lines per file (`refine.sh`), because the first pass over-counted (see `SupervisorSpec` below).

| Bucket | Files | ≈Test cases | Meaning |
|---|---|---|---|
| Public surface | 63 | 357 | no internal identifiers; scenario is portable |
| Touches internals thinly (≤3 marker lines) | 23 | 310 | mostly public behavior; port the scenario, drop the internal assertion |
| Depends on internals (>3 marker lines) | 16 | 222 | dies with the language |
| **Portable as scenarios** | **86 (84%)** | **≈667 (75%)** | |
| **Dies** | **16 (16%)** | **≈222 (25%)** | |

Case counts are `in {` occurrences, so they are approximate **(A5)**.

Three examples that survive:
- `actor/ActorLifeCycleSpec.scala` (7 cases): start, restart and stop callback order; public.
- `actor/DeadLetterSupressionSpec.scala` (2 cases): dead-letter publication and suppression through the event stream; public.
- `event/EventStreamSpec.scala` (17 cases): subscribe, publish, unsubscribe, classifiers; public.

Three examples that die:
- `actor/SupervisorHierarchySpec.scala` (6 cases): 11 internal-marker lines, reads `context.asInstanceOf[ActorCell].mailbox.suspendCount` (line 170).
- `actor/dispatch/ActorModelSpec.scala` (11 cases): 31 marker lines, tests dispatcher internals.
- `actor/LocalActorRefProviderSpec.scala` (8 cases): 12 marker lines, tests the provider's internal ref types.

Two cases the heuristic got wrong, kept as evidence that triage is manual work: `actor/SupervisorSpec.scala` was flagged only because it defines its own class named `Mailbox` (line 115, a custom `MailboxType`), and `actor/DeathWatchSpec.scala` touches `dispatch.sysmsg` in one place (line 262) across 11 cases. Both are mostly public.

**Gate result.** The migration has no exit condition without a judge that runs against old and new code through the public surface. No such judge exists: there is no CLI, wire protocol or language-neutral harness at this scope; the public surface is a JVM API. **The verdict below therefore names "build the judge first" (`prompts/00b-judge-setup.md`) as a pre-Step-1 task.**

**Parity scenarios to seed `00b-judge-setup`** (each is a script of spawns and messages plus an expected trace of observable events; traces are compared after normalising timestamps and thread names; files go in `judge/scenarios/`):

| Family | Scenarios (seed) | Pekko source to read |
|---|---|---|
| Lifecycle | start order `preStart`; restart calls `preRestart` then `postRestart` and resets state; stop runs `postStop` and stops children first; failure inside `preStart` | `ActorLifeCycleSpec`, `ActorRefSpec`, `RestartStrategySpec` |
| Supervision | Resume / Restart / Stop / Escalate; one-for-one vs all-for-one; retry limit within a time window; failure of a child during a parent restart; nested hierarchy | `SupervisorSpec`, `SupervisorMiscSpec`, `SupervisorTreeSpec` (use `SupervisorHierarchySpec` for ideas only; it is internals) |
| Deathwatch | `Terminated` delivered exactly once; watching an already-dead actor; `unwatch` before and after termination; `Terminated` ordered after the messages already processed; watch across restart | `DeathWatchSpec` (11 cases; drop the line-262 internal assertion) |
| Message ordering | per sender–receiver FIFO (the only ordering Pekko guarantees, per its docs **(A6)**); stash/unstash order; `become`/`unbecome`; `forward` keeps the original sender; priority and control-aware mailboxes | `ActorWithStashSpec`, `ActorWithBoundedStashSpec`, `ForwardActorSpec`, `PriorityDispatcherSpec`, `ControlAwareDispatcherSpec` |
| Dead letters | message to a stopped actor reaches the event stream as `DeadLetter` with sender and recipient; suppression; suspension after a burst | `DeadLetterSupressionSpec`, `DeadLetterSuspensionSpec`, `EventStreamSpec` |
| Envelope metadata | tenant/trace metadata preserved across `tell`, `forward`, `ask`, router fan-out, stash/unstash, restart, and dead-letter publication; fail-closed when the tenant is missing | **no Pekko oracle** (see below) |
| Routing | round-robin, random, broadcast, smallest-mailbox, consistent-hash, resizers | `RoundRobinSpec`, `BroadcastSpec`, `ConsistentHashingRouterSpec`, `ResizerSpec` |
| Serialization | round-trip of primitives and registered types; manifest lookup; unknown manifest fails | `PrimitivesSerializationSpec` |

**The envelope-metadata family has no old-code oracle.** Tenancy and the shared metadata base do not exist in Pekko, so "old code is the spec" does not apply to them. They need a spec-based oracle (written requirements checked with `go-specs`), not a parity referee. The judge covers `pekko-actor` and `pekko-serialization`; `pekko-tenancy` and `pekko-observability` are verified by their own specs.

## 4. The six steps instantiated for this repo

Unit of work = **slice** inside a module (a whole module is too large for one reviewer context; `pekko-actor` alone is ≈30k source lines). Nine slices **(A7)**:

| # | Slice | Pekko source | Lines (non-blank) |
|---|---|---|---|
| S1 | actor core: refs, paths, system, props, extensions | `actor/` minus dungeon and fault handling | ≈9.5k |
| S2 | supervision and deathwatch | `actor/dungeon/*`, `actor/FaultHandling.scala` (also defines `SupervisorStrategy`) | ≈3.9k |
| S3 | dispatch and mailbox | `dispatch/` | 4,554 |
| S4 | event stream, dead letters, logging adapter | `event/` | 3,211 |
| S5 | patterns | `pattern/` | 5,986 |
| S6 | routing | `routing/` | 3,381 |
| S7 | serialization | `serialization/` | 1,482 |
| S8 | observability (new, no Pekko source beyond `event/Logging`) | design from ADRs | unknown |
| S9 | tenancy (new, no Pekko source) | design from ADRs | unknown |

S1+S2 lines come from the 13,375 in `actor/` and are approximate **(A8)**. The 20 `util/` files are folded into the slice that uses them.

| Step | What it means here | Kit prompt | Units × estimate |
|---|---|---|---|
| Pre-1: judge | scenario extraction from 86 files / ≈667 cases; Scala harness in `judge/pekko-harness`, Go harness in `judge/gopekko-harness`; validate against deliberately broken code | `00b-judge-setup.md` | 8 families × unknown per family; see §6 |
| 1. Map and rules | depmap done (`migration/stage-1/depmap*`); `docs/design.md` and ADRs instead of a rulebook; gap inventory of Scala-isms (212 Future/Promise, 194 implicit, 348 `private[pekko]`, 54 reflection, 163 `asInstanceOf`) | `01-dependency-map.md` (manifest step adapted: file→file mapping is invalid for a redesign, the manifest lists slices), `02-gap-inventory.md` | 1 map + 1 inventory + ≈8 ADRs |
| 2. Stress-test | bakeoff is invalid; substitute adversarial review of `docs/design.md`/ADRs, plus a disposable full run of S1–S3 | `03-stress-test.md` (modified) | 1 design review + 1 disposable run |
| 3. Translate | implementer, two reviewers, fixer per slice | `04-translation-kickoff.md` + `scripts/queue_runner.mjs` | 9 slices × 4 agents |
| 4. Compile | dissolves into Step 3: `go vet` and `go build` inside every loop; `migration/settings.go.json` already allows them and denies `go test` | `05-survey-build.md` only as a final sweep | 9 slices × seconds (A4) |
| 5. Run it | one actor spawned, one `Tell`, one `Ask`, clean stop | none | 1 |
| 6. Match behavior | run both harnesses over the scenario list, diff traces, triage each failure on the old code first; done-gate = scenario list signed off by you (no inherited suite) | `06-post-parity.md` | ≈8 families × scenarios |

## 5. Dependency map summary (`migration/stage-1/depmap*`, `depmap_scala.py`)

| View | Nodes | Edges | Cycles |
|---|---|---|---|
| File, imports only | 199 | 748 | 4 SCCs (sizes 2, 2, 5, 32) |
| File, with same-package references (`--implicit`) | 199 | 1,149 | 10 SCCs (largest 62 files) |
| Package | 21 | 91 | 2 SCCs: one of 14 packages, one of 3 (`io`) |

- The 14-package cycle contains `actor`, `actor.dungeon`, `dispatch`, `dispatch.affinity`, `dispatch.sysmsg`, `event`, `japi`, `pattern`, `routing`, `serialization`, `util` and the root. **The scope cannot be ported package by package in dependency order**, which is another reason the unit is the slice and not the package.
- Inside the scope there are 331 cross-package edges. Heaviest: routing→actor 63, pattern→actor 54, event→actor 47, actor→dispatch 34, dispatch→actor 26, actor→event 20, serialization→actor 17.
- Limits of the mapper (unverified against scalac, **A9**): string-interpolation bodies are blanked; members of a `package object` are not tracked individually.

## 6. GoAkt candidates — ADRs to write

GoAkt reference: `github.com/tochemey/goakt/v4`, local clone `../goakt` at `v4.6.1-1-g00da8115`. The files named below exist; whether GoAkt's approach is *better* is exactly what each ADR must prove (default: Pekko wins). ADR format: `docs/adr/0000-template.md`.

| ADR (proposed) | Feature | Pekko source | GoAkt candidate | Evidence the ADR must produce |
|---|---|---|---|---|
| 0001-mailbox-and-dispatch | mailbox, dispatcher, scheduling model on goroutines | `dispatch/Mailbox.scala`, `AbstractDispatcher.scala`, `Dispatcher.scala`, `Mailboxes.scala` | `actor/mailbox.go`, `actor/dispatcher.go`, `actor/ready_queue.go`, `internal/queue/`, plus 11 mailbox variants (`unbounded_*`, `bounded_*`, `*_priority_*`, `non_blocking_bounded_mailbox.go`) | throughput and latency benchmark on both designs; judge ordering scenarios; lines and moving parts; idiomatic Go? |
| 0002-serialization-protobuf | serializer registry, manifest, binding, default format | `serialization/Serialization.scala`, `Serializer.scala`, `SerializationExtension` | `remote/serializer.go`, `remote/proto_serializer.go` (also `cbor_serializer.go`, `json_serializer.go`), `protos/internal/*.proto` | judge round-trip scenarios; wire-compatibility stance with Pekko (decision for you); codegen cost; fits `pekko-serialization` depending only on `pekko-actor` |
| 0003-supervision | supervisor strategies and fault handling | `actor/FaultHandling.scala` (defines `SupervisorStrategy`), `actor/dungeon/FaultHandling.scala` | `supervisor/supervisor.go`, `actor/supervision.go`, `actor/supervision_signal.go` | judge supervision scenarios (Resume/Restart/Stop/Escalate, retry windows) |
| 0004-deathwatch | watch and `Terminated` delivery | `actor/dungeon/DeathWatch.scala` | `actor/death_watch.go`, `actor/remote_watch_registry.go` | judge deathwatch scenarios, especially ordering after in-flight messages |
| 0005-dead-letters-and-event-stream | dead letters, event bus | `event/EventStream.scala`, `event/DeadLetterListener.scala`, `DeadLetter` in `actor/ActorRef.scala` | `actor/dead_letter.go`, `eventstream/` | judge dead-letter scenarios; suppression semantics |
| 0006-routing | routers and resizers | `routing/*` | `actor/router.go`, `actor/router_option.go` | judge routing scenarios |

ADRs where **neither** project is the source and so no GoAkt comparison applies (Pekko default is moot): `0007-logging-adapter` (kit-logger wrapper; deferred, non-blocking), `0008-envelope-and-metadata` (shared base of tenancy and observability), `0009-tenancy-fail-closed`, `0010-actor-api-style` (classic vs typed semantics, A1).

## 7. Cost and duration

**Bands only. These are order-of-magnitude estimates derived from the repo, with per-unit figures that are my assumptions, not measurements (A10). The kit README's case-study numbers are not estimates for this repo.**

Drivers (visible multiplication):

| Component | Units | Per-unit estimate (A10) | Band |
|---|---|---|---|
| Translate: implementer + 2 reviewers + fixer + referee (go vet/build, judge run) per slice | 9 slices | 0.35–1.1M tokens/slice (implementer 150–400k, reviewers 2 × 100–250k, fixer 50–150k, referee 50–150k) | 3–10M |
| Disposable full run first (redesign substitute for the bakeoff) | ×2 on the translate line | | +3–10M |
| Design doc + ADR authorship and adversarial design review | ≈10 ADRs | 0.2–0.5M each | 2–5M |
| Judge: scenario extraction + two harnesses + validation against broken code | 8 families, 86 files / ≈667 cases | 0.4–1M per family | 3–8M |
| Kit prompts 01, 02, 03, 05, 06 overhead | 5 prompts | 0.2–0.6M each | 1–3M |
| **Total** | | | **≈10–40M tokens (order 10⁷)** |

Source size anchor: ≈32k non-blank in-scope lines, ≈10 tokens per line (A10) → ≈0.3M tokens to read the whole scope once; every reviewer rereads its slice, which is why the review topology dominates.

Per step, wall-clock and active attention (all A10; wall-clock assumes standard rate limits):

| Step | Wall-clock | Active attention |
|---|---|---|
| Pre-1 judge (00b) | 1–2 days | 3–5 h (scenario list sign-off) |
| 1. Map and rules (design doc, ADRs, inventory) | 1–2 days | 4–6 h (the ADR decisions are yours) |
| 2. Stress-test (design review, disposable run) | 0.5–1 day | 1–2 h |
| 3. Translate | 1–3 days | 2–3 h |
| 4. Compile | dissolved into 3 | none |
| 5. Run it | 1–2 h | 0.5 h |
| 6. Match behavior | 2–4 days | 4–8 h |
| **Total** | **≈1–2 weeks** | **≈15–25 h** |

## 8. Model plan (a decision for you at this gate)

Model IDs available per my environment: Fable 5.1 (`claude-fable-5-1`), Opus 5.5 (`claude-opus-5-5`), Sonnet 5.5 (`claude-sonnet-5-5`), Haiku 5.5 (`claude-haiku-5-5`). I do not know which tiers your plan allows **(A11)**. Every post-feasibility prompt (01–06) takes your choice as an explicit `[model]`.

| Phase | Recommended tier | Reason |
|---|---|---|
| Design doc, ADRs, judge scenario spec (00b), amendments | largest (Fable 5.1 or Opus 5.5) | one-time work, and an error in an ADR replicates into every slice; the scenario list is the exit condition |
| Adversarial review of the design (Step 2) | largest | a cheap reviewer misses the ordering and supervision subtleties that make the judge fail later |
| Skeptic reviewers inside Step 3, S2/S3 (supervision, deathwatch, dispatch) | largest or mid (Opus 5.5) | highest gap complexity: concurrency semantics |
| Skeptic reviewers, S1 and S4–S7 | mid (Sonnet 5.5) | lower gap complexity |
| Translation implementers | mid (Sonnet 5.5) | volume; two reviewers and `go vet`/`go build` stand behind them |
| Fixers | mid (Sonnet 5.5) | the compiler and the judge are the referee |
| Mechanical work (manifest, depmap reports, TSV/log upkeep) | small (Haiku 5.5) | deterministic scripts do the real work |

## 9. Verdict

**Migrate — but not yet: build the judge first.** The pre-Step-1 task is `prompts/00b-judge-setup.md`, using the scenario families in §3.3, and Step 1 does not start until the scenario list is signed off. Redesign is the right shape (§3.1), verification on the Go side is cheap (§3.2), and 75% of the existing test cases are portable as scenarios (§3.3). The cost is order 10⁷ tokens over one to two weeks (§7).

**The single fact that would change it (this repo cannot tell me):** whether a Pekko-side harness can produce *deterministic, language-neutral event traces* for lifecycle, supervision and deathwatch ordering. If the same scenario gives different traces across runs, the judge becomes statistical, "match behavior" has no clean exit condition, and the verdict moves to *later* or *don't*.

**How to check in under a day:** in `judge/pekko-harness`, write one scenario (an actor that fails once under a Restart strategy with a watcher), run it ≈100 times with `JAVA_8_HOME` set and sbt 1.12.14, and diff the normalised traces. If all 100 are byte-identical, the verdict holds. If not, find out whether the variation is scheduling (fixable with a controllable clock and a single-threaded dispatcher) or inherent to the scenario.

## 10. Assumptions (unverified) and what I read and ran

**Assumptions**
- A1: the reference is Pekko's classic actor API, not the typed API.
- A2: the product around goPekko is Go and that is the reason to port.
- A3: the cold sbt build is slower than the 19 s measured (launcher and build definition were warm).
- A4: Go build and vet time scales linearly with lines; not measured, no Go code exists yet.
- A5: test-case counts are `in {` matches, approximate.
- A6: Pekko guarantees ordering only per sender–receiver pair (from its documentation, not verified in this repo).
- A7: nine slices is my split; the design doc may change it.
- A8: the S1/S2 line split of `actor/` is estimated, not counted.
- A9: depmap limits (string interpolation, `package object` members).
- A10: all token, time and attention figures; per-unit costs are not calibrated against a real run.
- A11: which model tiers your plan allows.
- A12: legal: porting Apache-2.0 code needs the licence and NOTICE attribution preserved; I did not review the legal requirements.

**Read:** `migration-kit/prompts/00-feasibility.md`, `migration-kit/README.md`, `migration-kit/CLAUDE.md`, `migration-kit/scripts/depmap_python.py`; in Pekko: `build.sbt` (module definitions), `project/build.properties`, `dispatch/` file list and sizes, `actor/dungeon/*` sizes, the header of `dispatch/Mailbox.scala`, and the matched lines of `SupervisorSpec`, `DeathWatchSpec`, `SupervisorHierarchySpec`; in GoAkt: directory and file names only (`../goakt`), no source read.

**Ran:** `sbt -batch actor/compile` twice (first run failed on the missing JDK 8; second succeeded; both logged), the depmap over `actor/src/main/scala` (imports and `--implicit`), `classify_tests.sh`, `refine.sh`, `measure.sh`, `scopedeps.py` (all in `migration/stage-1/`), greps and line counts. No tests were run, no files under `upstream/pekko` or `migration-kit` were modified (`git status` is clean in both; sbt `target/` dirs are git-ignored).

**Artifacts in `migration/stage-1/`:** `feasibility.md`, `build-baseline.log`, `build-baseline.time`, `measure.txt`, `scopedeps.txt`, `test-classification.tsv`, `test-cases.txt`, `refine.tsv`, `refine.txt`, `depmap/`, `depmap-implicit/`, and the scripts that produced them.

## Amendments

### 2026-10-08: GoAkt reference path

- **Change:** wherever this report says `../goakt` (§6 "GoAkt candidates" and §10 "Read"), the reference is now `reference/goakt`, a git submodule of this repository.
- **Reason:** `../goakt` was a clone outside the repository, so a clean clone could not resolve the paths this report cites. The submodule pins the same commit the report was written against: `00da8115aa90488c648efcd3c804689b011ab9a4` (`v4.6.1-1-g00da8115`, the head of `main` on `Tochemey/goakt`).
- **Not changed:** no finding, count, estimate or verdict in this report. The GoAkt file names listed in §6 refer to that commit. The stage-1 scripts in `migration/stage-1/` did not reference `../goakt`, so none needed an edit.

### 2026-10-08: parity criterion, first scope and gate status

- **Why:** an owner review found that the parity criterion would fail correct Pekko runs, that the classic API was treated as approved, and that the first implementation scope was too wide to validate the architecture. Every guarantee was re-checked against `upstream/pekko` `v1.7.1`; the evidence is in [`docs/judge.md`](../../docs/judge.md).
- **Unchanged:** the measurements (§1, §5), the case in §2, the redesign call (§3.1), the test census (§3.3 table), the slices (§4) and the GoAkt candidates (§6).

| Section | Original conclusion | Status | Amended conclusion |
|---|---|---|---|
| §3.3 gate, §9 | Parity = normalised traces compared across runs; the deciding fact is whether 100 runs are byte-identical. | **Superseded** | Parity = observable events checked against guaranteed relations and invariants, per [`docs/judge.md`](../../docs/judge.md). Pekko guarantees only partial orders (judge §1), so identical traces are neither required nor sufficient. The deciding fact becomes: each family's guarantees can be checked from the public surface on Pekko, and the judge fails on every mutation in judge §6. |
| §3.3 seed, Deathwatch | "`Terminated` delivered exactly once"; "ordered after the messages already processed". | **Corrected** | Exactly once only for a single registration; re-registering may yield two (judge I2, `docs/actors.md:432-438`). The ordering is restated as judge I1: after every message the subject sent directly to the watcher, and after its `postStop`. Both are implementation behavior (tier I), not documented guarantees. |
| §3.3 seed, Dead letters | A message to a stopped actor reaches the event stream as `DeadLetter`. | **Qualified** | Pekko documents dead letters as best-effort (`docs/general/message-delivery-reliability.md:315-320`). Asserted as tier I (judge I3), strictly only in controlled mode. |
| §3.3 seed, Message ordering | Includes priority and control-aware mailboxes. | **Qualified** | The FIFO guarantee does not hold for non-FIFO mailboxes (`…:134-138`). They leave the first scope. |
| §3.3 seed, Supervision | Retry limit within a time window. | **Qualified** | Pekko measures the window with `System.nanoTime` (`actor/…/actor/FaultHandling.scala:66-80`), so a manual clock cannot drive it. Concurrent mode only. |
| §1, A1 | The classic API is the reference. | **Open** | The survey used the classic API; that does not approve it for goPekko's public API. Decided by ADR 0010 (proposed). |
| §6 | ADRs 0001-0006 first; 0008, 0009 and 0010 listed as "neither project is the source". | **Reordered** | ADR 0010 (API, file `0010-actor-api.md`) and ADR 0008 (envelope and metadata) come first: the judge's scenarios and the runtime's extension ports depend on them. ADR 0008 also states the fail-closed rules that §6 assigned to 0009; whether 0009 stays separate is the owner's call. |
| §4, Step 2 | Stress-test with a disposable full run of S1-S3. | **Replaced** | Milestone 1: one vertical path (spawn → tell/ask → failure → restart → watch → stop → dead letter) across S1-S4 plus ask from S5, judged on lifecycle, supervision, deathwatch, FIFO, dead letters, ask and the envelope. Other patterns and routing wait. |
| §4, Step 4 | `migration/settings.go.json` allows `go vet`/`go build` and denies `go test` in loops. | **Corrected** | The file exists but is not installed as `.claude/settings.json`, so no deny is active. It must be installed before the first implementation loop (milestone 1). |
| §7 | Token, time and attention bands. | **Unchanged, restated** | Still assumptions (A10), not measurements. They are recalibrated after milestone 1 with real per-step figures in `migration/cost-log.tsv`. |
| §10, A12 | Licensing not reviewed. | **Partly resolved** | `LICENSE` (Apache-2.0) and `NOTICE` exist and `CLAUDE.md` has the header rule. No legal review was done. |

**Gate status.** The verdict "migrate, but build the judge first" stands, and it is **not signed off**. Still open for the owner: the model plan (§8, A11), ADR 0010, ADR 0008, and accepting the judge contract. Nothing in Step 1 starts before that sign-off (`migration-kit/CLAUDE.md`, rule 3).
