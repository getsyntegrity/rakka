# Audit report: group evt-pat-ser (epics #756 EVT, #757 PAT, #761 SER)

Read-only. Baselines: Pekko v1.7.1 `9a54d129` (verified `git rev-parse`); goPekko `origin/develop` `9abfb40` (snapshot `tmp/develop/`, plus `git show origin/develop:` for `pekko-serialization`, which is absent from the snapshot). ADR 0032 read from open PR #1014 (state OPEN, not merged) only to check what the stories cite.
Method note: my extraction of each bundle kept User story, Boundary, ACs, Dependencies, Upstream links and the whole ACT2 reconciliation block. Older stories' "Out of scope / Architect handoff" boilerplate and the newer stories' "Context / Scope / Requirements / Design" sections were NOT read. DEEP below means: ACs + reconciliation read, and the Pekko source AND spec behind each AC opened (or absence of a spec recorded), plus the governing ADR text. Everything else is SHALLOW (anchors/lines checked, behavior or spec not fully read).

## 1. Coverage inventory

Totals: 52 stories. DEEP 21, SHALLOW 31, NOT REVIEWED 0.
State = decision-state taxonomy (4 = Not designed, 5 = Designed not implemented, 3 = Contradictory, 1 = Accepted). No story is 6 or 7: `develop/pekko-actor` has no event bus, no pattern package, no breaker/backoff, and `pekko-serialization` has no built-in serializers. Readiness is the label; only #1052 is `status:ready`.

### EVT (#756), 18 stories
| # | Key | Prio | Ready | Depth | State | Note |
|---|---|---|---|---|---|---|
| 793 | EVT-01 | P0 | blocked | DEEP | 4 | decision slice; ADR does not exist |
| 836 | EVT-02 | P0 | blocked | DEEP | 4 | title says "typed" (collides with pekko-actor-typed) |
| 870 | EVT-03 | P0 | blocked | DEEP | 4 | |
| 905 | EVT-04 | P0 | blocked | DEEP | 4 | recon SPLIT -> #1056 |
| 871 | EVT-05 | P0 | blocked | DEEP | 4 | cites ADR 0032 as accepted (F-2) |
| 906 | EVT-06 | P0 | blocked | DEEP | 4 | recon SPLIT -> #1057 (F-2, F-14) |
| 907 | EVT-07 | P3 | blocked | DEEP | 4 | F-12 |
| 940 | EVT-08 | P3 | blocked | SHALLOW | 4 | `DeadLetterSuspensionSpec` located, not read |
| 794 | EVT-11 | P1 | blocked | SHALLOW | 3 (body scope vs recon) / D11 part 1 | F-9 |
| 848 | EVT-13 | P1 | blocked | SHALLOW | 4 | `MarkerLoggingSpec`/`LoggerSpec` not read |
| 849 | EVT-14 | P1 | blocked | SHALLOW | 4 | |
| 837 | EVT-09 | P3 | blocked | SHALLOW | 4 | Lookup bus |
| 872 | EVT-10 | P3 | blocked | SHALLOW | 4 | Scanning bus |
| 891 | EVT-12 | P3 | blocked | SHALLOW | 4 | Subclassification bus |
| 1052 | EVT-15 | P1 | **ready** | DEEP | 5 (D11 accepted; D13 evidence pending) | F-10 |
| 1054 | EVT-16 | P3 | blocked | SHALLOW | 4 | wrong GOV edge (F-13) |
| 1056 | EVT-17 | P1 | blocked | DEEP | 4 | F-7 |
| 1057 | EVT-18 | P1 | blocked | DEEP | 4 | F-11 |

### PAT (#757), 23 stories
| # | Key | Prio | Ready | Depth | State |
|---|---|---|---|---|---|
| 795 | PAT-09 | P2 | blocked | SHALLOW | 4 |
| 838 | PAT-04 | P2 | blocked | SHALLOW | 4 |
| 873 | PAT-01 | P2 | blocked | DEEP | 4 (F-1) |
| 874 | PAT-02 | P2 | blocked | DEEP | 4 |
| 875 | PAT-05 | P2 | blocked | SHALLOW | 4 |
| 876 | PAT-10 | P2 | blocked | SHALLOW | 4 |
| 908 | PAT-03 | P2 | blocked | SHALLOW | 4 |
| 909 | PAT-06 | P2 | blocked | DEEP | 4 |
| 910 | PAT-11 | P2 | blocked | SHALLOW | 4 |
| 911 | PAT-13 | P2 | blocked | SHALLOW | 4 |
| 941 | PAT-07 | P2 | blocked | DEEP | 4 |
| 942 | PAT-12 | P2 | blocked | SHALLOW | 4 |
| 943 | PAT-14 | P3 | blocked | SHALLOW | 4 |
| 965 | PAT-16 | P3 | blocked | SHALLOW | 4 |
| 968 | PAT-08 | P3 | blocked | DEEP | 4 |
| 981 | PAT-15 | P3 | blocked | SHALLOW | 4 |
| 1100 | PAT-17 | P2 | blocked | DEEP | 4 |
| 1101 | PAT-18 | P2 | blocked | SHALLOW | 4 |
| 1102 | PAT-19 | P2 | blocked | DEEP | 4 (F-6) |
| 1103 | PAT-20 | P2 | blocked | DEEP | 4 (F-5) |
| 1104 | PAT-21 | P3 | blocked | DEEP | 4 |
| 1105 | PAT-22 | P3 | blocked | DEEP | 4 |
| 1106 | PAT-23 | P3 | blocked | SHALLOW | 4 |

### SER (#761), 11 stories
| # | Key | Prio | Ready | Depth | State |
|---|---|---|---|---|---|
| 804 | SER-01 | P2 | blocked | DEEP | 4 (ADR 0021 O7 open) (F-4) |
| 805 | SER-03 | P3 | blocked | SHALLOW | 4 (decision slice) |
| 806 | SER-04 | P3 | blocked | SHALLOW | 4 (recon calls it contradictory; see F-15) |
| 807 | SER-05 | P3 | blocked | SHALLOW | 4 (expected exclusion) |
| 811 | SER-09 | P2 | blocked | SHALLOW | 4 (F-4) |
| 845 | SER-07 | P3 | blocked | SHALLOW | 4; crit. 3 = 1 (Owner decision option A, 2026-10-25, GOV-46 #1193 body) |
| 850 | SER-10 | P3 | blocked | SHALLOW | 4 |
| 851 | SER-11 | P3 | blocked | SHALLOW | 4; crit. 3 = 1 (option A, GOV-45 #1192); evidence classification under D13 pending determination |
| 884 | SER-06 | P3 | blocked | SHALLOW | 4 |
| 885 | SER-08 | P3 | blocked | SHALLOW | 4; crit. 1-3 = 1 (option (a), GOV-47 #1194) |
| 951 | SER-02 | P2 | blocked | SHALLOW | 4 |

Owner-decision lines in #845/#885/#851 verified in the GOV bodies (#1193/#1194/#1192 contain "Owner decision recorded on 2026-10-25: option A / A / (a)"); the three GOV issues are still OPEN and no ADR carries them. I treat these as unambiguous recorded Owner decisions (taxonomy 1) but not ADR-accepted.

## 2. Capability matrix

Go state column: verified against `develop/pekko-actor` (only `actor/`, `internal/deadletter`, `internal/mailbox`; no `pattern`, no event bus) and `origin/develop:pekko-serialization`.

| Capability | Pekko contract | Go contract (ADR/issue) | State | Go implementation | Evidence | Gap |
|---|---|---|---|---|---|---|
| Event stream subscribe/duplicate/null | `EventStream.scala:56` subscribe returns Boolean; spec `EventStreamSpec.scala:102,108` (null rejected), `:311-312` (true then false on duplicate) | #793 decision, #836 | 4 | absent. Only `SubscribeDeadLetters(fn)` (`actor/deadletter.go:36`), synchronous callback hub `internal/deadletter/hub.go` | Pekko spec exists; no differential path (judge has no subscribe kind) | return value of subscribe (spec-asserted) is in no AC; Go has no class hierarchy so subtype matching is Go-only (F-11) |
| Subchannel classification / sub-class match | `EventBus.scala:136,177-190,206-209`; `EventStreamSpec.scala:150-279` | #793, #891 | 4 | absent | Pekko oracle is JVM subclassing; not reproducible 1:1 | D13: applicability of that oracle to the type-relationship ACs pending determination |
| Publish via mailbox | `EventStream.scala:51-53` (`subscriber ! event`) | #870 | 4 | absent | no Pekko spec for blocked subscriber or bounded overflow (recon says so; I searched `EventStreamSpec` headings) | oracle applicability under ADR 0033 D13 pending determination |
| Unsubscribe one/all/on termination | `EventStream.scala:64,74`; `EventStreamUnsubscriber.scala:59`; `EventStreamSpec.scala:298-355` | #905, #1056 | 4 | absent (Watch is `Context`-only: `watch.go:98`) | spec asserts async removal via debug log + `awaitAssert` | F-7 |
| DeadLetter/Suppressed/Dropped/Unhandled family | `ActorRef.scala:579-630`, `Actor.scala:297`; `DeadLetterSupressionSpec.scala:36-100` | #871, #906, #1057, #897 | 4 | `DeadLetter{Msg,To}` only (`deadletter.go:20`); no sender, no reason, no Suppressed/Dropped/Unhandled | spec exists (AllDeadLetters, SuppressedDeadLetter recipient = deadLetters) | F-2, F-11, F-14 |
| Dead-letter logging (maxCount, suspend, shutdown flag) | `DeadLetterListener.scala:36-44,81-92,94-117,141-154`; `ActorSystem.scala:441-447,1068-1069,1113-1114`; spec `DeadLetterSuspensionSpec.scala`, `ConfigSpec.scala:81-87` | #907, #940 | 4 | absent | no spec for maxCount counting (searched `log-dead-letters` in actor-tests: only ConfigSpec) | F-12 |
| Actor-scoped logging (ActorLogging) | `Actor.scala:340-345` | #794, ADR 0033 D11 | D11 = 1; export = 4 | `Config.Logger` used only for supervision (`supervision_logging.go`, `system.go:81-89`); no `Context.Log` | no Pekko spec opened for ActorLogging | F-9 |
| MDC clear per turn | `Actor.scala:375-380` (set, `finally clearMDC`) | #848 | 4 | absent | none | goPekko design: ADR 0033 D11 (accepted) uses a per-actor `With` logger; no Pekko oracle was found for the per-turn MDC clear, so whether an applicable oracle exists is pending determination (Pekko code: `Actor.scala:375-380`); alternative evidence and Owner approval under ADR 0033 D13 come into question only if the oracle is shown not to apply |
| Log markers | `Logging.scala:1687,832-851`; `ActorLogMarker.scala:26`; `MarkerLoggingSpec` | #849 | 4 | absent | spec not read | kit-logger mapping undefined |
| kit-logger test fake | D11 (accepted): capture `With`, honor `SetLevel`, guard `Entries` | #1052 | 5 | absent | verified shipped fake: `kitlogtest/mocklogger.go:98` `With` returns `m`, `:102` `SetLevel` no-op, `:31` `Entries` exported slice (v0.2.0, the version in `pekko-actor/go.mod`) | F-10 |
| Lookup / Scanning / Actor(Managed) buses | `EventBus.scala:70,93,219,277`; `EventBusSpec.scala:180,200,302-353` | #837, #872, #1054 | 4 | absent | ActorEventBusSpec exists | classification has no GOV story (F-13) |
| Logging surface (LoggingAdapter, LogSource, LogEvent hierarchy, loggers, filter) | `Logging.scala:44,317,438,1192,1560,2017,2049` | only GOV-29 #1078 (LoggingReceive), GOV-30 #1079 (filter) | 4 / no record | absent | | F-8 |
| gracefulStop | `GracefulStopSupport.scala:48-49,59-70`; `PatternSpec.scala:36-60` | #873 | 4 | absent (`Stop(Handle)` exists: `stop.go:20`) | Pekko spec exists | F-1 |
| after / timeout | `FutureTimeoutSupport.scala:33,50-56,97`; `PatternSpec.scala:59-95` | #874, #908 | 4 | absent | spec exists (in PatternSpec) | F-14 |
| retry (count, fixed, function, backoff, shouldRetry) | `RetrySupport.scala:37,48,74,80,115,172,188,211,287,334,404`; `RetrySpec.scala:62,97,120` | #838, #875, #1101 | 4 | absent | spec exists | `shouldRetry` overloads (since 1.1.0) in no AC (F-8) |
| CircuitBreaker core | `CircuitBreaker.scala:151-164,220,700-716,780,857-908,964,1011,1060,1105-1118,1137` | #795, #876, #910, #911, #1100, #1104 | 4 | absent | `CircuitBreakerSpec` exists, all TimingTest with real scheduler (recon); judge cannot drive it (JDG-09 #1094: breaker clock is `System.nanoTime`, concurrent-only) | F-3, F-8 |
| Breaker listeners | `CircuitBreaker.scala:539-682,723-769` | #942, #965 | 4 | absent | | Pekko notifies only if not ignored-exception (`:902`) and in 3 distinct code paths (`:869,872,885,892`); ACs silent |
| Breaker registry | `CircuitBreaker.scala:67,115`; `CircuitBreakersRegistry.scala:63,79-97` | #943 | 4 | absent | HOCON | ok per recon |
| Backoff on failure / stop | `BackoffOptions.scala:75,190,273-343,361,415`; `internal/BackoffOnRestartSupervisor.scala:55-110`; `internal/BackoffOnStopSupervisor.scala:57-120`; `HandleBackoff.scala:25-70` | #909, #941, #968, #981, #1102-#1106 | 4 | absent | `BackoffSupervisorSpec.scala:188,214,246,292,341,394,410`; `BackoffOnRestartSupervisorSpec.scala:205` | F-5, F-6 |
| Primitive serializers | `PrimitiveSerializers.scala:24-66,166-200`; `reference.conf:841-847`; `PrimitivesSerializationSpec.scala:40-62` | #804, #811 (ADR 0021 O7 open) | 4 | absent. Registry rejects ids <=40 (`serialization/builder.go:104-111`, `serialized.go:31`) | spec only round-trips and buffer==array (LITTLE_ENDIAN comment) | F-4 |
| Buffer serializer | `Serializer.scala:176-223,436-464` | #805, #850 | 4 | absent | `SerializeSpec.scala:258-275` (not read) | decision slice |
| Nested depth | `NestedDeserialization.scala:47-61`; `Serialization.scala:163,237`; `reference.conf:785` | #806, GOV-44 | 4 | absent; ADR 0021 O1 says no limit | oracle only in `remote` (`NestedPayloadDepthSpec`) | F-15 |
| Decompression limit | `Decompression.scala:57-80`; `DecompressionSpec` | #807, #851 | 4 | absent | `DecompressionSpec` exists | expected exclusion |
| Async serializer | `AsyncSerializer.scala:25-71` | #845, #885 | 4 / criteria 1 | absent | `AsyncSerializeSpec` exists | ok |
| serialize-messages check | `dungeon/Dispatch.scala:37,174,180`; `reference.conf:128-140` | #884 | 4 | absent | none in actor-tests | needs hook port |
| ActorRef serialization | `Serialization.scala:51,72`; `ActorRef.scala:463-492` | #951 | 4 | absent; `pekko-serialization/go.mod` requires only go-specs | none | P2 depends on P3 stories (F-16) |

## 3. Findings

**F-1 (high) PAT-01 #873: recon states the wrong timeout result.** Recon AC "timeout sends Unwatch and returns false (GracefulStopSupport.scala:68-69)". Pekko: `:48-49` scaladoc "completed with failure AskTimeoutException"; `:66-68` `case _ => Unwatch; false` is returned for any success value that is not Terminated(target.path), the failure mapper `t => {Unwatch; t}` re-throws; `PatternSpec.scala:36-60` "complete Future with AskTimeoutException when actor not terminated within timeout". `false` is never returned on timeout. Also the cited `GracefulStopSpec` does not exist in actor-tests (only `actor-typed-tests/.../scaladsl/GracefulStopSpec.scala`); the spec is `PatternSpec`. An "already-terminated target completes" case (`PatternSpec.scala:44-48`) is in no AC. Correction: the AC must match Pekko: timeout fails (GracefulStopSupport.scala:48-49,68) and sends Unwatch (:68); the Go error type is an open question (ADR 0015 sentinels), no recommendation. PatternSpec:36-60 is the Pekko spec for the timeout failure; the already-terminated case (`PatternSpec.scala:44-48`) is in no AC.

**F-2 (high) Stories cite ADR 0032 as "(accepted)" but it is not in develop.** #871, #906, #1057 (via #814) and #1103 list "ADR 0032 D3/D4/D6 (accepted)". ADR 0032 is only `docs/adr/0032-local-sender-envelope.md` on open PR #1014; its status line is "partly accepted" from a session message, "not a GitHub comment". Per the brief it is not part of develop. Substantive conflict: ADR 0032 D3 = absent sender is `nil`, "to be reconsidered once #896 defines deadLetters". Pekko's `DeadLetter` and `SuppressedDeadLetter` forbid a null sender and substitute `deadLetters` (`ActorRef.scala:593-594,612-613,689,723`); only `Dropped` may carry `noSender` as sender (`ActorRef.scala:618-628`); `MessageBuffer.scala:268` passes the stored ref as sender and `noSender` as recipient. `docs/judge.md:42` records the same ("Pekko replaces a missing sender with deadLetters"). The stories never say how the family's sender is represented. Correction: until #1014 merges, label the dependency "proposed (PR #1014)"; add to #871/#906 an AC naming sender representation per event type, and make the DeadLetter-family sender part depend on #896 (MSG-08) as well as #814.

**F-3 (high) The `depends_on #1020 (GOV-02)` edge cannot be satisfied for most of this group.** #1020's scope (and AC-R1-1) is exactly the plan A-03 / dossier 11.2 candidates: clock, mailbox head insert, `/system` spawn, `Context.Log`, event stream, dead-letter sink, child failure cause. ADR 0033 D3 covers "unexported `pekko-actor` API that `pekko-actor-typed` needs". PAT-09 (a "new public pattern package"), EventBus variants #837/#872/#891, Dropped/SuppressedDeadLetter types, the retry/backoff/breaker helpers and the serializer ports are not Typed-needed exports and have no #1020 row. Meanwhile ADR 0011 owner decision 2 (`0011-pekko-actor-layout.md:262-278` region, "`actor/` stays one cohesive package") means package/module placement of CircuitBreaker, Backoff, retry, EventBus is itself undecided; no story or ADR decides it. Plan P-02 even says `pipeTo/gracefulStop/after/retry` are "R as Go idioms" (`port.go:12` documents the goroutine idiom), which conflicts with PAT stories adding exported helpers. Correction: Owner decision (see section 4, O-1) plus either extend #1020 with rows for these exports or replace the edge by a dependency on the placement decision.

**F-4 (high) SER-01/SER-09 rest on Pekko-fixed ids that the Go registry rejects, and on an unstated wire-format choice.** Ids 1 (JavaSerializer), 4 (ByteArraySerializer), 18, 19, 20, 21 (primitive-bytestring), 35 (`reference.conf:838-847`; id 21 is not named in F-4's story evidence) are inside 0-40, which `pekko-serialization/serialization/builder.go:104-111` (`checkID`, `ErrReservedSerializerID`) refuses for every registered serializer; `serialized.go:31` `FirstUserSerializerID = 41`; `doc.go` says the package is "not a wire format, not a source of built-in serializers". Neither story defines the registration path for reserved ids, whether bytes must equal Pekko's (Pekko Long array format is little-endian, always 8 bytes: `PrimitiveSerializers.scala:35-47`; the spec only checks round-trip and array==buffer, no literal bytes: `PrimitivesSerializationSpec.scala:40-62`), nor ids for Go scalars Pekko has no serializer for (float64, uint*, int8/16). Recon says SER-01/SER-09 are "independent" on id allocation, but both consume the same bypass. The delivery plan (`actor-typed-delivery-plan.md` ~line 182) still lists #804/#806/#811 as STARTABLE; recon now blocks them (ADR 0021 O7/O1): plan and recon disagree. Correction: GOV-43 #1190 must also decide the reserved-id bypass, wire-compat yes/no and the extra-scalar id policy; amend SER-01 AC1 and SER-09; update plan row.

**F-5 (medium) PAT-20 #1103 evidence claim is wrong.** AC-R2-2 evidence: "no direct upstream spec for ForwardTo found by grep". `BackoffSupervisorSpec.scala:214-244` ("use provided actor while stopped and withHandlerWhileStopped is specified") exercises `ForwardTo` and the reply reaches the original sender. So a Pekko oracle exists and D13 requires a differential scenario, not Go-only evidence. Correction: cite `:214-244`; same check for AC-R1-1 (`:246`) and AC-R2-1 (`:188`) which are correct.

**F-6 (medium) PAT-19 #1102 mixes two mechanisms and its time-range AC does not discriminate.** Time range exists only in `BackoffOnRestartSupervisor.scala:69-75` (one `scheduleOnce` is scheduled at the first restart, `BackoffOnRestartSupervisor.scala:69-75`; when it fires, the count is reset only if no further restart was counted since, `HandleBackoff.scala:61-64`). `BackoffOnStopSupervisor.scala:78-93` ignores `withinTimeRange` for counting. `BackoffOpts` has no time-range option; it is reachable only via `withSupervisorStrategy(OneForOneStrategy(...))` (`BackoffOptions.scala:288` doc: replaces `maxNrOfRetries`). The boundary says both variants are bounded by the range. AC-R2-1 (failures at 0 and 31s, range 30s) is also reached by the default `AutoReset(minBackoff)` (`BackoffOptions.scala:361,415`; `HandleBackoff.scala:44-51`), so it cannot show the window works unless auto-reset is off or longer. Correction: restrict the time range to the on-failure variant, state the reset semantics per HandleBackoff.scala:61-64, make AC-R2-1 configure manual reset or an auto-reset longer than the window, record the interplay with #1105.

**F-7 (medium) EVT-17 #1056 AC-R2-1 contradicts its own cited spec.** "stopped actor subscribes -> receives nothing and no registration remains". `EventStreamSpec.scala:327-355`: `subscribe(terminated, A)` returns `true` (registration accepted), removal is asynchronous through `EventStreamUnsubscriber` (observed via debug log "unsubscribing ... from all channels") and the re-subscribe is under `awaitAssert`. Whether an event published between subscribe and removal reaches a dead letter is pending verification; question: does publish to a terminated subscriber produce a DeadLetter; evidence missing: no spec in EventStreamSpec publishes in that window; source EventStream.scala:51-53 only. Also `:311-312` pins subscribe's Boolean result (the "no-duplicate result" of #836 AC1) which no AC states. Correction: restate as eventual removal with the observable return value and the dead-letter window.

**F-8 (medium) Capabilities with no story and no exclusion/decision record found** (searched the 444-row `act2_index.tsv` titles and the three bundles; I did not read all 444 bodies):
- Events/logging: `LoggingAdapter`/`LogSource`/`LogEvent` hierarchy and the logger-on-bus mechanism (`Logging.scala:44,317,438,1192,2017,2049`), `LoggerMailbox`, `jul`/stdout/default loggers, `log-dead-letters` setting grammar `off|on|N` (`ActorSystem.scala:441-445`), `EventStream.startUnsubscriber/initUnsubscriber` (`EventStream.scala:86,94`), `ActorClassificationUnsubscriber` (removal for `ManagedActorClassification`, excluded by #1054's boundary and covered by no story). Plan row E-02 says "R via kit-logger (D11)", but D8/D13 require a recorded source+reason per exclusion.
- Patterns: breaker `succeed()/fail()` (`CircuitBreaker.scala:488-499`; only appears inside PAT-10's audit text) and `isClosed/isOpen/isHalfOpen` (`:507-527`), `CircuitBreakerTelemetry` (InternalStableApi), `retry` with `shouldRetry` (`RetrySupport.scala:80,172,334`, since 1.1.0; PAT-04's "nonretryable failure" implies it, recon omits it), `BackoffSupervisor.calculateDelay`.
- Serialization: serializer bindings/`SerializationSetup`, `NullSerializer` beyond id 0, `SerializerWithStringManifest` (ported: `spi/serializer.go:10`). D8 says the extension stays replaced; no record lists what is excluded.
Correction: add GOV classification stories (or rows in an existing exclusion inventory) for each; E-06's Owner classification of EventBus variants "has no GOV story" (recon of #837/#872/#907 says so).

**F-9 (medium) EVT-11 #794: body and recon still disagree, and no AC has a Pekko oracle.** Body boundary keeps "level/filter integration"; recon says remove it (GOV-30 #1079) and fix sources (ActorLogging `Actor.scala:340`, not LoggingReceive). Body text is "preserved unchanged", so the contradiction stays in the issue. AC1 "explicit across restart" and AC3 "logging failure cannot change the directive or run a handler twice" have no Pekko counterpart (recon admits AC3). Correction: rewrite the boundary line, the D13 classification of AC1/AC3 is pending determination (no Pekko counterpart found; recon admits AC3).

**F-10 (medium) EVT-15 #1052 is `status:ready` with an undefined placement.** The fake is "in pekko-actor" but its location/visibility is not stated; consumers include #794, #848, #849, #907 and the future `pekko-actor-typed`. An exported package needs the ADR 0011 justification (no #1020 row exists), a test-private helper cannot be reused cross-module. AC-R1-1 is stated by the story as Go-only evidence; whether an applicable Pekko oracle exists is pending determination. D11's description of the shipped fake is accurate (v0.2.0 `With` returns the receiver, `SetLevel` no-op; `Entries` is an exported field guarded by an internal mutex that readers do not take). Correction: decide placement first or narrow the story to a module-private fake; keep ready only then.

**F-11 (medium) Differential evidence for the event stream is overstated.** #1057 AC-R2-1: "oracle exists: dead letters are in the judge". `docs/judge.md:38-53` records only a `dead-letter` event (message, recipient) from a harness-owned subscriber; no SuppressedDeadLetter, Dropped, AllDeadLetters, subscribe, duplicate-subscription or sender field. JDG-02 #1027 (extension for Classic additions) body mentions neither the event stream nor suppressed letters; JDG-09 #1094 covers backoff/breaker only; `capabilities.json` has 0 capabilities (status "bootstrap") and `ACTINV-066/067` cited by #793 do not exist in the repo. Pekko's own answer for duplicate delivery is single copy (subscriber set union, `EventBus.scala:206-209`; `DeadLetterSupressionSpec.scala:49-82`). So V3 for EVT-03/04/05/06/18 has no judge path today. Where a Pekko spec exercises the behavior (for example `DeadLetterSupressionSpec.scala:49-82`), ADR 0033 D13 asks for a differential scenario against Pekko; the alternative-evidence approval of D13 covers only behavior with no applicable oracle. Open question: whether JDG-02 (#1027) adds event-stream kinds to the judge, and how each EVT story is labelled until then (Owner decision; no recommendation).

**F-12 (medium) EVT-07/08 ACs and recon are imprecise.** `DeadLetterListener` subscribes only to `DeadLetter`, `Dropped`, `UnhandledMessage` (`:41-43`), never `SuppressedDeadLetter`, so a suppressed event cannot reach it; `:154 isWrappedSuppressed` tests a `WrappedMessage` whose inner message is `DeadLetterSuppression`. Recon's "suppressed ignored (:154)" is the wrong reading. The listener publishes an `Info` event carrying `ActorLogMarker.deadLetter` (`:141-147`), so #907 has an unstated dependency on #849 (markers). Pekko: DeadLetterListener is a system actor subscribed to DeadLetter, Dropped, UnhandledMessage (DeadLetterListener.scala:40-44) that publishes an Info event with `ActorLogMarker.deadLetter` (:139-147). Open question: the Go counterpart; missing: Owner/ADR decision. No recommendation (dependency on #1055 SYS-18 is also unmentioned). Correction: reword AC2, note the unstated #849 edge.

**F-13 (medium) Wrong or inconsistent edges.** (a) #1054 EVT-16 `depends_on #1035 (GOV-04 become/unbecome)` has nothing to do with EventBus (recon of siblings: "no ADR 0010 exclusion item covers EventBus variants"); it also lacks the #1020 edge its sibling bus stories carry. (b) #1056 depends on #905 (explicit unsubscribe) but needs the registrations of #836/#870 and a watch mechanism (Watch is `Context`-only: `watch.go:98`); #1055 is "independent". (c) #873 hard-depends on #770 (PoisonPill, behind GOV-07 #1038) although recon says a custom stop message needs none: split the default-message AC. (d) #875 (retry backoff, which depends on #838) is the producer of `calculateDelay` for #909/#941; the shared piece does not need the retry loop (`RetrySupport.scala:404`), so the chain #821->#838->#875->#909 is longer than the mechanism requires. (e) #906 (Dropped) has no in-tree producer in classic Go: Pekko producers are `MessageBuffer.drop`, typed stash/Topic/GroupRouter (`Supervision.scala:123`, `TopicImpl.scala:96`, `GroupRouterImpl.scala:99,153`); the producer->#906 edges are not recorded on any story. (f) Priority inversions: P0 #871/#906 depend on P1 #814; P2 #951 depends on P3 #894 and #1083.

**F-14 (low) Spec names in recon blocks that do not exist.** `FutureTimeoutSupportSpec`, `AfterSpec` (#874, #908), `GracefulStopSpec` (#873), `DeadLetterSpec` (#907) are absent from actor-tests (searched with `find`). Real locations: `PatternSpec.scala:36-60` (gracefulStop), `:59-76` (after), `:78-95` (timeout); `DeadLetterSuspensionSpec.scala`, `ConfigSpec.scala:81-87`. Also #906 AC "Dropped sender" depends on `MessageBuffer.scala:268` passing the stored sender as `sender` (verified).

**F-15 (low) SER-04 label.** Recon says ADR 0021 O1 is "contradiction with this story's premise". ADR 0021 (accepted 2026-10-19, O1 deferred) says nesting is pending and "does not claim parity", i.e. undecided, not forbidden: state 4, not 3. The real blocker is depth propagation: `spi.Serializer.FromBinary(data, manifest)` has no context (O6) and Pekko uses a thread-local (`NestedDeserialization.scala:47-61`).

**F-16 (low) Hidden local-only boundaries.** #951 SER-02: `pekko-serialization/go.mod` does not require `pekko-actor` yet (only go-specs v0.4.0); `scripts/deps.allow` allows it (verified), but #951 also needs `serializedActorPath` transport information scope (GOV-34 #1083, P3). #884 SER-06 needs an actor-side hook port because ADR 0033 D3 forbids `actor` importing serialization; recon says so, but the hook is not in #1020's candidate list.

## 4. Owner decisions needed

| ID | Decision | Alternatives and consequences | Recommendation |
|---|---|---|---|
| O-1 | Placement of patterns, breaker, backoff, buses, DeadLetter-family types | (a) inside `actor` package (ADR 0011 d.2 cohesive, simplest imports, grows the package); (b) new `pattern`/`event` packages in `pekko-actor` (matches Pekko layout, needs d.2 exception); (c) separate module (needs `go.work`/`deps.allow` lines). Also decides whether #1020 must index them. | No recommendation. |
| O-2 | Sender representation in dead-letter-family events | `nil` per ADR 0032 D3 (differs from Pekko DeadLetter/Suppressed) vs `deadLetters` ref (needs #896) vs per-type | No recommendation. Pekko: DeadLetter and SuppressedDeadLetter require a non-null sender (ActorRef.scala:593-594, 612-613) and substitute `deadLetters` when sent without one (ActorRef.scala:689,723; AbstractDispatcher.scala:51); Dropped may carry `noSender` (ActorRef.scala:618-628). ADR 0032 D3 (unmerged) differs; no Owner decision under ADR 0033 D13 recorded. |
| O-3 | Go meaning of event classification (EVT-01) | exact `reflect.Type` only; plus interface satisfaction; explicit class tokens | No recommendation. Pekko: classification is by class/subclass (EventBus.scala:136,177-190; EventStreamSpec.scala:150-279) and is JVM subclassing; the Go meaning is an open Owner decision. |
| O-4 | Built-in serializers (GOV-43): reserved-id bypass, Pekko-byte compatibility, extra scalars | Pekko wire bytes (little-endian, ids fixed) vs Go-native bytes with Pekko ids kept; extra scalars take ids >=41 or none | not formed; needs the Owner's wire-compat intent |
| O-5 | Inclusion/exclusion records for F-8 items and EventBus variants | include, exclude with reason, or defer | No recommendation. |
| O-6 | D13 classification of rows the stories treat as Go-only | for each row, determine whether an applicable Pekko oracle exists; alternative evidence and approval come into question only where the oracle is shown not to apply | batch list: #1052 AC-R1-1, #873 AC3, #874 AC3, #875 AC3, #981 AC2, #794 AC1/AC3, #848, #806, #851 AC3, #845 AC3, #885 |
| O-7 | gracefulStop / retry error types (AskTimeout analogue) | share ASK error taxonomy vs new errors | No recommendation. |

## 5. ADRs needing creation, amendment or acceptance (not changed)

- New: event stream / subscription semantics (#793 is the vehicle); pattern package contract (placement, `context.Context` instead of futures per ADR 0033 consequences, error taxonomy, breaker/backoff/retry semantics). Neither exists.
- ADR 0021: amendments only after GOV-43/44/45/46/47 decisions (O1, O6, O7 are still deferred in its Status; the GOV-45/46/47 outcomes are recorded only in issues).
- ADR 0010: "Excluded for now" list does not cover EventBus variants, dead-letter logging, LoggingAdapter family (recon of #837/#872/#907 notes the same).
- ADR 0008 is `proposed`; its propagation table includes dead letters (judge C2 row). "Sender and recipient metadata remain present" in #871 is ambiguous between sender/recipient and ADR 0008 metadata.
- ADR 0032: merge of PR #1014 pending; D5 open.
- ADR 0033 D8/D11/D13 are accepted and sufficient for the accepted parts; no amendment needed.

## 6. Blocked vs potentially startable

- Startable now: #1052 (label ready; resolve F-10 placement first). #1020 and #1190 (GOV, outside this group) are the real gates.
- Potentially startable after one prerequisite: #793 (ADR proposal; only the formal #1020 row), #805 and #807 (decision slices, independent), #795 minimal slice (closed->open counting, fail-fast, open exception, no timers) after O-1 and a #1020 row.
- Blocked, exact prerequisites: all EVT beyond #793 chain from #793->#836->#870->#905/#871->#906->#1057/#1056/#907->#940 (the last also needs #1129 clock); #794/#848/#849 need #1052, #1020 and GOV-29/30 decisions; PAT scheduler-bound stories need #821 (P0, blocked) or #1129; #838/#874/#908 also need #773; #909/#941 need #875, #854, #777; SER-01/09 need #1190; SER-04 needs #1189; SER-07 needs #773 and ADR 0021 O6; SER-08 needs #845; SER-10 needs #805; SER-11 needs #807; SER-02 needs #894, #853, #1083; SER-06 needs #843, #1021.
- No dependency cycle found. Missing or wrong edges: F-13.

## 7. Pekko references with no spec or limited search

- No spec found for: EventStream blocked-subscriber/bounded-overflow behavior (recon and my heading search); HalfOpen configurable probe limit (none exists); breaker max-cap, jitter, constructor validation (no spec: recon's grep, I confirmed `requires` at `CircuitBreaker.scala:163-164` only); DeadLetterListener `maxCount` counting (searched `log-dead-letters` in actor-tests: ConfigSpec only); `ActorLogging`/`DiagnosticActorLogging` MDC (not searched in `LoggerSpec`; unread); buffer overflow in `ByteBufferSerializer` (recon: none).
- Read source only, spec not opened: `Closed.callFails` (`:964-988`) and `HalfOpen` bodies (`:1011-1055`) beyond anchors; `EventBusSpec` bodies (headings and lines 124-137 anchors only); `MarkerLoggingSpec`, `LoggerSpec`, `LoggingReceiveSpec`, `DeadLetterSuspensionSpec`, `AsyncSerializeSpec`, `DecompressionSpec`, `SerializeSpec`, `CircuitBreakerMTSpec/StressSpec`, `BackoffSupervisorSpec` auto-reset cases, `RetrySpec` bodies; `Logging.scala`, `LoggingFilter` internals; `typed` counterparts (`ActorLogging` typed, `Topic`, `EventStream` typed) belong to other groups.
- Not read: story sections outside AC/boundary/recon for the 52 issues (see method note); `docs/design.md`; ADR 0014/0017/0030 bodies beyond status lines.
- Claims taken from the issues and spot-verified only for line anchors: second-pass audit statements in #870, #876, #911, #845, #850, #851, #885.
