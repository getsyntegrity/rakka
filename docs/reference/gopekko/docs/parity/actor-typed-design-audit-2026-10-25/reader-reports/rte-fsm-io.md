# Audit report: group rte-fsm-io (epics #758 RTE, #759 FSM, #762 IO)

Read-only. Baselines: Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (paths relative to `upstream/pekko`); goPekko snapshot `.../tmp/develop` (= origin/develop 9abfb40). Issue text from `tmp/bundles/epic758|759|762.md` and `issues_all.json`. ADR 0033 D8 read in full (develop/docs/adr/0033-...md:42). Paths `A/` = `actor/src/main/scala/org/apache/pekko/`, `AT/` = `actor-tests/src/test/scala/org/apache/pekko/`.

Depth rule used: DEEP = whole story body + reconciliation block read, Pekko source read for each AC, and the cited spec read in the relevant test(s). Where I read source but not the spec bodies, the story is SHALLOW even if I found errors in it (stated in the note column). No story is NOT REVIEWED at the level of "bodies read"; unread Pekko material is listed in section 7.

## 0. State of develop (verified, not trusted)

- `develop/pekko-actor` has NO router/routee/FSM/ByteString/Tcp/Udp/Dns code: case-insensitive grep for `router|routee|fsm|bytestring|tcp|udp|dns` over `pekko-actor/` returns zero source hits. `ls pekko-actor/actor` = actor, address, ask, cell, clock, deadletter, executor, guardian, hierarchy, mailbox, names, port, ref, registry, spawn, stop, supervision*, system*, watch. No scheduler, no sender, no PoisonPill, no stash, no ActorSelection, no Props.
- `DeadLetter` is `{Msg any; To ID}` (pekko-actor/actor/deadletter.go:22-25): no sender field (confirms RTE-20 #1111 "DeadLetter has no sender field yet").
- Runtime is generic: `Ref[T any]` (ref.go:18), `Actor[T any]` (actor.go:19); ADR 0010 D1 = A3 "ordinary generics in Ref[T], Context[T]" (0010:232). ADR 0010 `Status: proposed` (partly accepted: local watch increment, supervision). "Excluded for now" (routers, timers, stash, PoisonPill...) is the Consequences bullet at 0010:225, in a PROPOSED ADR; the binding gate is ADR 0033 D4 (accepted): each item classified individually. No accepted ADR exists for routing, FSM, io, ByteString. ADR 0008 (proposed) :117 says "Routers (later stage): Transparent: each routee gets the original envelope".
- Judge harness does not drive routers/FSM (delivery plan 5.2, B-02): a V3 claim for a RTE/FSM row cannot be made until B-02 exists. The Pekko specs exist (RoutingSpec, FSM*Spec), so ADR 0033 D13 asks for a differential scenario against Pekko where an applicable oracle exists; D13 does not make these rows "pending approval", whose alternative-evidence approval covers adaptations with no applicable oracle. How to label these rows in the meantime is an open question for the Owner.
- Implementation state for every story below: absent. Evidence present: none (no Go tests; Pekko specs exist, see matrices).

## 1. Coverage inventory (53 issues: 20 RTE + 14 FSM + 19 IO; plus the 3 epic bodies read)

Taxonomy: RTE/FSM = **4 Not designed** (no ADR/contract; D4 classification pending in GOV-12 #1042 / GOV-13 #1064 / GOV-14 #1043, all "Owner decision pending"; the decision slices RTE-01/FSM-01 are themselves ADR-writing stories). IO = D8 default rule is **1 Accepted** (principle only); every per-capability exclusion is **2 Proposed** (EXCLUDE_PROPOSAL in recon; records GOV-20..23,39 are "pending"). Where noted, ByteString rows are 2 Proposed lean `replaced-by-go-stdlib` (plan only; not in bodies).

| # | key | prio / readiness | depth | state | note |
|---|---|---|---|---|---|
| 796 | RTE-01 | P3 / blocked | DEEP | 4 | Router.scala:28-185 read; no upstream spec asserts NoRoutee path (searched `NoRoutee` across *Spec.scala/*Test.java: none) |
| 839 | RTE-02 | P3 / blocked | DEEP | 4 | Router.scala:149-183, RoutedActorCell.scala:61-114 |
| 944 | RTE-03 | P3 / blocked | SHALLOW | 4 | RouterConfig.scala:146-179 read; ConfiguredLocalRoutingSpec group tests not read |
| 912 | RTE-04 | P3 / blocked | SHALLOW | 4 | RoundRobin.scala:39-47 read; RoundRobinSpec :61-90 not read |
| 877 | RTE-05 | P3 / blocked | SHALLOW | 4 | Random.scala:38-41 read; RandomSpec not read |
| 913 | RTE-06 | P3 / blocked | SHALLOW | 4 | Router.scala:128-139 + Broadcast.scala:38; BroadcastSpec not read |
| 878 | RTE-07 | P3 / blocked | SHALLOW | 4 | RouterConfig.scala:196-262, RoutedActorCell.scala:116-132; RouteeCreationSpec only first test |
| 914 | RTE-08 | P3 / blocked | DEEP | 4 | RoutedActorCell.scala:159-192; RoutingSpec.scala:68-111 read |
| 879 | RTE-09 | P3 / blocked | DEEP | 4 | SmallestMailbox.scala:46-112; SmallestMailboxSpec.scala:26-60 (single test) |
| 915 | RTE-10 | P3 / blocked | SHALLOW | 4 | Balancing.scala + BalancingDispatcher.scala:45 read; BalancingSpec titles only |
| 880 | RTE-11 | P3 / blocked | SHALLOW | 4 | ConsistentHash.scala:28-152 read; MurmurHash.scala and spec not read |
| 945 | RTE-12 | P3 / blocked | DEEP | 4 | ScatterGatherFirstCompleted.scala:44-78; spec :106-115 |
| 916 | RTE-13 | P3 / blocked | DEEP | 4 | TailChopping.scala:76-115; TailChoppingSpec :88-135 |
| 946 | RTE-14 | P3 / blocked | SHALLOW | 4 | Resizer.scala:139-321 read; ResizerSpec not read |
| 969 | RTE-15 | P3 / blocked | SHALLOW | 4 | only OptimalSizeExploringResizer.scala:30,129,138 anchors; MetricsBasedResizerSpec (28 tests) not read |
| 1107 | RTE-16 | P3 / blocked | DEEP | 4 | ConsistentHashing.scala:150-257, 443-457; spec :78-100 |
| 1108 | RTE-17 | P3 / blocked | DEEP | 4 | RouterConfig.scala:82-91, 395-455; RoutedActorCell.scala:159-218; RoundRobinSpec:111-128; RoutingSpec:90-111 |
| 1109 | RTE-18 | P3 / blocked | SHALLOW | 4 | wrapper locations verified by grep only |
| 1110 | RTE-19 | P3 / blocked | DEEP | 4 | RouterConfig.scala:96,116-134,283-376; RoutingSpec:220-262 (ConfiguredLocalRoutingSpec titles only) |
| 1111 | RTE-20 | P3 / blocked | DEEP | 4 | Router.scala:128-144 |
| 797 | FSM-01 | P3 / blocked | SHALLOW | 4 | DSL anchors in FSM.scala checked; AbstractFSM.scala (Java wrapper) not read |
| 840 | FSM-02 | P3 / blocked | DEEP | 4 | FSM.scala:719-721, 813-909; FSMTransitionSpec:28-66,78-87 |
| 881 | FSM-03 | P3 / blocked | DEEP | 4 | FSM.scala:869-909 |
| 947 | FSM-04 | P3 / blocked | SHALLOW | 4 | source read (:813-909); FSMTimingSpec bodies not read except 43-72 |
| 970 | FSM-05 | P3 / blocked | SHALLOW | 4 | FSM.scala:125-155,818-829; TimerSpec/FSMTimingSpec not read |
| 917 | FSM-06 | P3 / blocked | DEEP | 4 | FSM.scala:881-894; FSMTransitionSpec:78-156 titles + 78-87 |
| 948 | FSM-07 | P3 / blocked | DEEP | 4 | FSM.scala:830-843; FSMTransitionSpec:89-114 |
| 949 | FSM-08 | P3 / blocked | SHALLOW | 4 | LoggingFSM FSM.scala:958-1019 read; FSMActorSpec:262-334 not read |
| 930 | FSM-09 | P3 / blocked | DEEP | 4 | FSM.scala:869-941; FSMActorSpec:188-262 titles + 223-262 |
| 1058 | FSM-10 | P3 / blocked | DEEP | 4 | FSM.scala:707-708,787-792; FSMActorSpec:74-80,148-154; FSMTimingSpec:140-152 |
| 1059 | FSM-11 | P3 / blocked | SHALLOW | 4 | AbstractFSMWithStashActorTest.java (Java) not opened |
| 1060 | FSM-12 | P3 / blocked | DEEP | 4 | FSM.scala:835-843; ListenerSpec exists (generic trait), not read |
| 1061 | FSM-13 | P3 / blocked | DEEP | 4 | FSM.scala:719-721,919-941; Actor.scala:620-631; ADR 0017:280-290 |
| 1062 | FSM-14 | P3 / blocked | SHALLOW | 4 | FSM.scala:650,896-907; grep confirms only FSM.scala:650 and PersistentFSMBase.scala + docs/fsm.md:216 mention setStateTimeout (no spec) |
| 808 | IO-01 | none / blocked | DEEP | 2 | IO.scala:20-37, ByteString.scala:31-36,77,94,158,1164-1170; ADR 0033 D8; plan 2.3 |
| 846 | IO-02 | none / blocked | DEEP | 2 | ByteString.apply copies (:36 -> CompactByteString.apply:1169-1170 `bytes.clone`); fromArrayUnsafe:94 does not |
| 886 | IO-03 | none / blocked | SHALLOW | 2 | symbols only |
| 920 | IO-04 | none / blocked | SHALLOW | 2 | ByteIterator.scala:428, newBuilder:158 only |
| 887 | IO-05 | none / blocked | SHALLOW | 2 | IO.scala only |
| 921 | IO-06 | none / blocked | SHALLOW | 2 | Tcp.scala symbols (161,195,449,532) verified; TcpListenerSpec not read |
| 922 | IO-07 | none / blocked | SHALLOW | 2 | Tcp.scala:133,469,475 verified |
| 923 | IO-12 | none / blocked | SHALLOW | 2 | Udp.scala:114,125,138,183,190 verified |
| 924 | IO-15 | none / blocked | SHALLOW | 2 | Dns.scala:76-85, DnsProtocol.scala:63,86 verified |
| 952 | IO-08 | none / blocked | SHALLOW | 2 | Tcp.scala:187,461 verified |
| 953 | IO-16 | none / blocked | SHALLOW | 2 | CachePolicy.scala:22,28 verified; SimpleDnsCache not read |
| 966 | IO-19 | none / blocked | SHALLOW | 2 | Udp.scala:98,164 verified |
| 972 | IO-09 | none / blocked | SHALLOW | 2 | Tcp.scala:351,373,388,401 verified |
| 973 | IO-13 | none / blocked | SHALLOW | 2 | UdpConnected.scala:89,108,120,145 verified |
| 974 | IO-17 | none / blocked | SHALLOW | 2 | async resolver files listed, not read |
| 984 | IO-10 | none / blocked | SHALLOW | 2 | Tcp.scala:432,442 verified |
| 985 | IO-14 | none / blocked | SHALLOW | 2 | Inet.scala not read |
| 995 | IO-18 | none / blocked | SHALLOW | 2 | Tcp.scala:187,425,475 verified |
| 996 | IO-11 | none / blocked | SHALLOW | 2 | Tcp.scala:215,230,246,573,578,586 verified |

Counts (stories): DEEP 20 (RTE 10, FSM 8, IO 2), SHALLOW 33, NOT REVIEWED 0. Epic bodies #758/#759/#762 read. Stable keys vs issue numbers: all 53 match the index and the plan (verified by `act2_index.tsv` and the epic mindmaps).

## 2. Capability matrices

### 2.1 RTE (Pekko routing package)

| Capability | Pekko contract | Go contract | state | impl | evidence | gap |
|---|---|---|---|---|---|---|
| RoutingLogic/Routee/NoRoutee/SeveralRoutees | Router.scala:33,51,58,66,76,83 | RTE-01 #796 (ADR to write) | 4 | absent | spec: none for NoRoutee path (searched all *Spec/*Test); RoutingSpec covers pools only | see F2, F3, F9 |
| Router.route: Broadcast unwrap, NoRoutee -> deadLetters only if sender is InternalActorRef, RouterEnvelope unwrap | Router.scala:128-144 | RTE-01/06/20 | 4 | absent | no spec (RTE-20 admits) | recon text is correct; RTE-20 R2/AC-R2-1 keep silent drop, flagged Owner choice (RTE-20 Design) |
| Router value add/remove | Router.scala:149-183 (append, filterNot all equals) | RTE-02 | 4 | absent | RoundRobinSpec:111-128 (via mgmt msgs) | the story's Go "duplicate/absent result" representation is a goPekko design question, not an accepted adaptation (Pekko behavior: code `Router.scala:149-183`, spec cited `RoundRobinSpec:111-128`) |
| Cell-level routee mutation: watch on add, unwatch on remove, delayed PoisonPill 100 ms for owned child | RoutedActorCell.scala:65-114 | RTE-02/08/17 | 4 | absent | RoutingSpec:68 | watch/unwatch on add/remove is in no AC (F11) |
| Routing at sender side; only management msgs (AutoReceived, Terminated, RouterManagementMessage) enter the router mailbox | RoutedActorCell.scala:147-151; RouterConfig.scala:82-85 | none | 4 | absent | RoutingSpec:68-88 implicit | F2 |
| RoundRobin | RoundRobin.scala:39-47 | RTE-04 | 4 | absent | RoundRobinSpec:61 | counter never reset (recon right) |
| Random | Random.scala:38-41 | RTE-05 | 4 | absent | RandomSpec:57 | injected randomness is a goPekko design question, not an accepted adaptation; `RandomSpec:57` exists and whether it is an applicable oracle for an injected source is open (ADR 0033 D13) |
| Broadcast (Broadcast envelope, BroadcastPool/Group) | Router.scala:130; Broadcast.scala:38,73,137 | RTE-06, RTE-18 | 4 | absent | BroadcastSpec:35,65 | ok |
| SmallestMailbox | SmallestMailbox.scala:48-112 (scores: suspended MaxValue-1, unknown size MaxValue-3, processing +1, terminated -> random) | RTE-09 | 4 | absent | SmallestMailboxSpec:28 (one test) | needs cell inspection port (F8) |
| Balancing (BalancingRoutingLogic = head routee; shared mailbox via dispatcher) | Balancing.scala:43-47,83-163; BalancingDispatcher.scala:45 (@deprecated) | RTE-10 | 4 | absent | BalancingSpec:138-173 | needs shared-queue mailbox; GOV-10/11 |
| ConsistentHash ring value | ConsistentHash.scala:30-152 (vnodes<1 rejected :34; MurmurHash :149-151) | RTE-11 | 4 | absent | ConsistentHashingRouterSpec | node key = ConsistentRoutee.toString = full-address path string (ConsistentHashing.scala:443-457): F13 |
| ConsistentHashing routing logic (mapping > ConsistentHashable > warn+NoRoutee; ring rebuilt when routees change; key types Array[Byte]/String/AnyRef via Serialization) | ConsistentHashing.scala:158-257 | RTE-16 | 4 | absent | ConsistentHashingRouterSpec:81,98 (lines verified) | key-type decision is Owner (RTE-16) |
| ScatterGatherFirstCompleted | ScatterGatherFirstCompleted.scala:44-78 | RTE-12 | 4 | absent | spec:108-114 empty case only | F7 |
| TailChopping | TailChopping.scala:76-115 | RTE-13 | 4 | absent | spec:90,105,123 | AskTimeoutException (recon right) |
| Pool (fixed) / Group / FromConfig / NoRouter | RouterConfig.scala:146-262,283-376 | RTE-03/07/19 | 4 | absent | RoutingSpec:112-262; RouteeCreationSpec:29 | F4 |
| Terminated routee removal; stop router when empty unless resizer | RoutedActorCell.scala:178-187; RouterConfig.scala:91,251 | RTE-08 (RESCOPE) | 4 | absent | RoutingSpec:68,90 (verified) | F5 |
| Management messages GetRoutees/AddRoutee/RemoveRoutee/AdjustPoolSize | RouterConfig.scala:395-455; RoutedActorCell.scala:170-216 | RTE-17 | 4 | absent | RoundRobinSpec:111-128; RoutingSpec:90 | F2, F3, F10 |
| DefaultResizer / ResizablePoolCell | Resizer.scala:139-321 (validation :154-164; resize triggered via Resize msg :286-293; first resize in preSuperStart :279-283) | RTE-14 | 4 | absent | ResizerSpec (not read) | F8, F12 |
| OptimalSizeExploringResizer | OptimalSizeExploringResizer.scala:30,129,138 | RTE-15 | 4 | absent | MetricsBasedResizerSpec (not read) | not examined |
| Pool/Group wrappers per logic | RoundRobin.scala:83,148; Random.scala:75,139; Broadcast.scala:73,137; ConsistentHashing.scala:296,385; TailChopping.scala:163,261; SmallestMailbox.scala:196; Balancing.scala:83 | RTE-18 | 4 | absent | per-logic specs | F12 |
| Listeners trait (Listen/Deafen/WithListeners) | routing/Listeners.scala | FSM-12 only | 4 | absent | ListenerSpec | F9 |

### 2.2 FSM

| Capability | Pekko contract | Go contract | state | impl | evidence | gap |
|---|---|---|---|---|---|---|
| startWith/when(+timeout)/initialize/unknown state | FSM.scala:475-488,719-721,774-782,881-883 | FSM-01/02 | 4 | absent | FSMActorSpec:170-186 (Next state does not exist, per story), FSMTransitionSpec:28-66,80-87 | no spec for missing startWith (story admits) |
| Event dispatch, per-message cancel/regeneration of state timeout | FSM.scala:813-866 | FSM-02/04 | 4 | absent | FSMTimingSpec:43 | F6 |
| goto/stay/using/replying/forMax/stop, reply ordering | FSM.scala:282,306,315,500-528,869-909 | FSM-03 | 4 | absent | FSMActorSpec | recon right (replies before callback and commit) |
| onTransition (stay silent, goto same notifies, nextStateData in callback) | FSM.scala:511-514,739-744,888-894 | FSM-06 | 4 | absent | FSMTransitionSpec:80,119,127,144,156 | ok |
| Subscribe/Unsubscribe/Listen/Deafen, CurrentState/Transition | FSM.scala:830-843,891 (TODO watch, listeners never auto-removed) | FSM-07, FSM-12 | 4 | absent | FSMTransitionSpec:89,103 "not fail when listener goes away" | ok; Go cleanup = divergence |
| State timeout (scheduleOnce TimeoutMarker(generation)) | FSM.scala:814-817,896-907 | FSM-04, FSM-14 | 4 | absent | FSMTimingSpec:33,43,53,62 | F6 |
| Named timers (single/fixedDelay/fixedRate; AutoReceivedMessage payload sent raw) | FSM.scala:125-155,553-644,818-829 | FSM-05 | 4 | absent | FSMTimingSpec:75-129; TimerSpec:124,308 (cited, not read) | ok |
| terminate/onTermination (guard once, cancels timers, no try/catch), postStop = terminate(Shutdown), default preRestart calls postStop | FSM.scala:910-941; Actor.scala:620-631 | FSM-09, FSM-13 | 4 | absent | FSMActorSpec:188,208,223 | F8(order), F14 |
| whenUnhandled (replaces; orElse default) | FSM.scala:707-708,787-792 | FSM-10 | 4 | absent | FSMActorSpec:74-80,148-154; FSMTimingSpec:140 | F15 |
| LoggingFSM (logDepth 0, ring, debug under config flag) | FSM.scala:958-1019 | FSM-08 | 4 | absent | FSMActorSpec:262,314 | ok |
| FSM + Stash | AbstractFSM.scala:575 (Java) | FSM-11 | 4 | absent | AbstractFSMWithStashActorTest.java (compile smoke only) | ok |
| Java/other: AbstractFSM, PersistentFSM, TestFSMRef | n/a | FSM-01 retag java-api; PersistentFSM out of scope; TestFSMRef (testkit) in no FSM story | - | - | TestFSMRefSpec exists | testkit group |

### 2.3 IO

| Capability | Pekko | Go contract | state | impl | gap |
|---|---|---|---|---|---|
| Socket/protocol parts: Tcp, Udp, UdpConnected, Inet options | io/Tcp.scala (933 lines), Udp.scala, UdpConnected.scala, Inet.scala, *Connection/*Listener/*Manager, SelectionHandler | D8 accepted principle; EXCLUDE `network-io` proposals GOV-20 #1069, GOV-21 #1070, GOV-22 #1071, GOV-39 #1088 (all pending) | 1 principle / 2 per capability | absent | F11 coverage, F16 reason list |
| DNS: Dns/DnsProtocol/InetAddressDnsResolver (OS resolver), SimpleDnsCache/CachePolicy (pure in-memory TTL structures), AsyncDnsResolver/DnsClient/TcpDnsClient/DnsMessage/ResolvConfParser/DomainName | io/Dns.scala, DnsProvider.scala, InetAddressDns*.scala, SimpleDns*.scala, dns/* | GOV-23 #1072 (three capabilities decided separately, pending) | 2 | absent | CachePolicy.Ttl / SimpleDnsCache are local pure logic; D8 "local pieces evaluated individually" requires an explicit record that a pure cache is excluded only because its sole consumer is excluded (nobody consumes it locally) |
| IO extension accessor | io/IO.scala:24-37 is a 3-member generic accessor (`Extension.manager`, `IO(key)(system)`); managers are created by each protocol extension | IO-05 EXCLUDE | 2 | absent | F18 wording |
| ByteString / ByteIterator / ByteStringBuilder | util scala-2.13 ByteString.scala:31 (copying apply :36/1169-1170, fromArrayUnsafe :94, newBuilder :158), ByteIterator.scala:428 | IO-01..04 (lean `replaced-by-go-stdlib`, plan 2.3, not in bodies); dossier U-01 already says "replaced by []byte ... X" | 2 (dossier asserts X; plan says PENDING) | absent | F10, F17 |
| Buffer plumbing: DirectByteBufferPool, ByteBufferCleaner.java, UnsynchronizedByteArrayInputStream.java | io/ | no story, no record found (grep of all 444 issue bodies: none) | none | absent | F11 |

## 3. Findings

**F1 (high) Typed routing stories depend on Classic RTE contracts the Pekko typed routers do not use; plus priority inversion.**
TSD-04 #991 and TSD-05 #961 (both `prio:P2`, blocked) list RTE-01 #796, RTE-04 #912, RTE-05 #877, RTE-03 #944/RTE-07 #878 as depends_on (their own recon tables). In Pekko the typed routers have an independent `RoutingLogic[T]` (actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/routing/RoutingLogic.scala:30-44) with their own `RoundRobinLogic` (routees sorted by path, `nextIdx` adjusted on `routeesUpdated`, :52-84, unlike the Classic AtomicLong counter never reset, RoundRobin.scala:40-47), `RandomLogic` (:87-98) and `ConsistentHashingLogic` (requires vnodes>0, `mapping: T=>String`, :101-122); only `pekko.routing.ConsistentHash` (:22) is shared. The typed pool stops when the last child stops and uses a plain spawn (PoolRouterImpl.scala:108-118). Pekko observed (code): the typed and Classic round-robin differ (cites above). Whether a goPekko Typed router reuses the Classic port is a goPekko design question; pending Owner decision. Pekko evidence is code only; typed routing specs not opened. D9 (ADR 0033:43) includes Typed routers, GOV-12 is P1 ready, but the RTE chain is P3 and blocked on GOV-12; JDG-10 #1095 (P2) also consumes P3 RTE-04/07.
Pekko observed (code): the typed pool/group routers use their own RoutingLogic[T] (RoutingLogic.scala:30-122) and share only ConsistentHash with Classic. Open question for the Owner: which Classic routing ports the Typed routers consume in goPekko. Missing evidence: Owner decision on GOV-12. No recommendation.
Evidence: files above; #991 and #961 reconciliation tables; GOV-12 #1042 R2.

**F2 (high) Routing happens on the sender thread, not in the router actor; no RTE story states it.**
`RoutedActorCell.sendMessage` calls `router.route(...)` for every non-management message (RoutedActorCell.scala:147-151); only AutoReceivedMessage, Terminated and RouterManagementMessage reach the router mailbox (RouterConfig.scala:82-85). Consequences not in any AC: routed messages are never enqueued on the router (no mailbox accounting, no ordering vs management messages, "in-flight messages may still use the old Router" RoutedActorCell.scala:68-70,81-83 follows from this); Pekko observed (code and scaladoc): non-management messages are routed on the sender's call path (RoutedActorCell.scala:147-151); AddRoutee/RemoveRoutee/AdjustPoolSize "may be handled after other messages" (RouterConfig.scala:428,434,447). No test found that pins the ordering. ADR 0008:117 (proposed) states routers are "Transparent: each routee gets the original envelope" and is not in any RTE ADR impact list; ask-based routers (RTE-12/13) reply without envelope metadata (ADR 0008 propagation table).
Correction: add to RTE-01 ADR scope and AC; add ADR 0008 and ADR 0030 (mailbox) to the RTE impact lists.

**F3 (high) Go `Ref[T]`/`Actor[T]` generics vs Pekko's untyped router/FSM protocols: undecided in every story.**
Classic protocol messages are untyped and delivered to actors of any type: `Broadcast(msg)`, `RouterEnvelope`/`ConsistentHashableEnvelope`, `GetRoutees`/`Routees`/`AddRoutee`/`RemoveRoutee`/`AdjustPoolSize` (RouterConfig.scala:395-455), FSM `StateTimeout`, `CurrentState`/`Transition`/`SubscribeTransitionCallBack`/`Listen`/`Deafen` (FSM.scala:830-843) which are sent to arbitrary subscriber refs. ADR 0010 D1 chose A3 (`Ref[T]`, `Context[T]`, develop/docs/adr/0010-actor-api.md:232; pekko-actor ref.go:18). None of RTE-01, RTE-06, RTE-16, RTE-17, FSM-01, FSM-07, FSM-12 says how a `Ref[T]` accepts a wrapper/management/notification message (T = any, a sum type, a separate control channel, adapter refs). This decides the public API shape of the whole family.
Correction: add as an explicit question to the RTE-01 and FSM-01 ADRs (mandatory sections) and to GOV-12/GOV-14 alternatives; Owner decision (section 4, O-3).

**F4 (medium) RTE-19 R2 / AC-R2-1 contradicts Pekko for pools.** Base `RouterConfig.withFallback` returns `this` (RouterConfig.scala:96) and NoRouter returns the other (:375), but not every router config merges. Pekko observed (code): pools that mix in PoolOverrideUnsetConfig take the fallback's supervisorStrategy when this one is the default and its resizer when this has none (RouterConfig.scala:116-134); BalancingPool merges only supervisorStrategy (Balancing.scala:150-163); ConsistentHashingPool/Group also take the fallback's hashMapping (ConsistentHashing.scala:357-361, 427-431); FromConfig (a Pool, RouterConfig.scala:307-354) and other Groups inherit the base. "The receiver wins" is therefore not general. Tests: ConfiguredLocalRoutingSpec group tests not read. AC must state each router's observed merge rule. Also `FromConfig` is a Pool with `nrOfInstances = 0` (:346) and `verifyConfig` throws `ConfigurationException` containing the path (:324-325; RoutingSpec:239-244 asserts `include("routerNotDefined")`): add the path-in-message requirement. RoutingSpec:220-262 ("no router", "router FromConfig") is the better oracle than the cited ConfiguredLocalRoutingSpec range alone.

**F5 (medium) RTE-08 #914: body ACs still contradict Pekko; epic #758 checklist still carries the old title; wrong spec; missing watch.** Body AC1/AC3 require replacement of stopped/failed routees (Pekko removes with stopChild=false, stops router when empty unless resizer: RoutedActorCell.scala:177-187; RouterConfig.scala:91,251; confirmed by RoutingSpec "evict terminated routees" :68-88 and "not terminate when resizer is used" :90-111). Recon cites RouteeCreationSpec as the spec; the verifying specs are RoutingSpec:68 and :90. Epic #758 checklist title is "Replace failed or stopped pool routees" while the issue title was changed. Removal-on-Terminated only works because `addRoutees` watches `ActorRefRoutee` and `removeRoutees` unwatches (RoutedActorCell.scala:72,87,93-100): not in any AC (affects RTE-02/07/17; ADR 0014 watch, ADR 0032 D5 only for Terminated sender, which RTE-17 correctly says is not needed).
Correction: rewrite #914 ACs per recon; fix spec cite; update epic title line; add "routees are watched on add, unwatched on remove" to RTE-02 or RTE-07.

**F6 (medium) FSM state timeout is re-armed on every processed message, not only on transitions.** `receive` cancels `timeoutFuture` and increments `generation` for any user message and for fired named timers (FSM.scala:818-829,844-850); `stay()` produces a State with `timeout = None` (FSM.scala:511-514), so `makeTransition` re-schedules the state's default timeout (:901-907) after every message, i.e. inactivity semantics ("all transitions into this state, including staying, receive this timeout"). (Pekko evidence: code FSM.scala:818-850, 901-907 and docs fsm.md:211-213; not proven by tests: FSMTimingSpec:43-51 does not assert re-arm on stay(); other FSMTimingSpec bodies not read). FSM-04 AC2 ("a transition or explicit timeout override invalidates the old state timer") and its recon sentence ("scheduleOnce at each transition") understate it; FSM-14 R1/R3 likewise. Correction: AC: any incoming non-internal message or fired named timer invalidates the current state timeout generation; after processing, the state's default (or the result's forMax) timeout is armed again, including for stay().

**F7 (medium) RTE-12 #945: body AC and recon imprecise vs Pekko.** Body AC1 "One temporary reply endpoint coordinates..." vs source: each routee gets its own `ask` (own temp ref) and one `Promise` is completed by the first reply (ScatterGatherFirstCompleted.scala:66-72). Non-empty timeout failure is `AskTimeoutException` (from `ref.ask` with `Timeout(within)`), `TimeoutException("Timeout due to no routees")` is only the empty case (:62; spec :108-114 asserts class TimeoutException only for the empty pool). Recon says "no reply within the configured time yields a timeout failure": specify the type. `Routee`s other than ActorRef/Selection are silently skipped (:72). Correction: reword AC1; add exact failure types.

**F8 (medium) No owner story for the mailbox/cell inspection port that RTE-09, RTE-14, RTE-15 need.** SmallestMailbox reads suspended, processing (`mailbox.isScheduled && currentMessage != null`), `hasMessages`, `numberOfMessages` (SmallestMailbox.scala:79-112); DefaultResizer pressure reads `isScheduled`, `hasMessages`, `currentMessage`, `numberOfMessages` (Resizer.scala:206-222). All three stories point to MBX-01 #789, but #789's body has no inspection operation (grep of its body for isScheduled/numberOfMessages/hasMessages/suspended/inspect/Router/resiz: 0 hits; its ACs are about queue discipline selection and system-message ordering). Recon of RTE-09 already says "MBX-01 may not suffice ... no story yet". Open question: which goPekko story owns the mailbox/cell inspection that SmallestMailbox and DefaultResizer read in Pekko (isScheduled, hasMessages, numberOfMessages, currentMessage, suspended). Missing evidence: MBX-01 body vs these reads (GitHub, not Pekko). No recommendation. Related: ADR 0030 (bounded mailbox, accepted) has `depthLocked`/`occupiedLocked` internals only (mailbox.go:20-33).

**F9 (medium) `routing/Listeners.scala` (Listen/Deafen/WithListeners) has no classification.** Only FSM-12 #1060 cites it (grep of issues_all: #1060, #942, #910 mention "Listeners"; #942/#910 not examined). ListenerSpec (AT actor/routing/ListenerSpec.scala) covers the generic trait; FSM-12 AC evidence says "no upstream FSM spec for Listen". Correction: add an inventory/exclusion record row for the generic `Listeners` trait (port or exclude with reason) so FSM-12 can consume a defined contract.

**F10 (medium) IO-01 #808 couples two independent decisions and blocks the P1 serialization decision.** IO-01 depends_on GOV-20/21/22/23/39 (five P3 pending proposals) although the ByteString/byte-value half needs none of them. GOV-43 #1190 (`prio:P1`, ready) lists "ByteString (id 21) disposition: #808 (IO-01)" and GOV-33 #1082 / SER-01 #804 / SER-09 #811 depend on the byte-value outcome. IO stories carry no priority label by design (plan 7.4) so the P1 consumer waits on an unprioritized, blocked story. Observed (issue graph): IO-01 depends_on GOV-20/21/22/23/39 and GOV-43 depends on its byte-value outcome. Open question: whether the byte-value decision is independent of the network-io decisions. No recommendation.

**F11 (medium) ADR 0033 D8 exclusion records do not cover every Pekko `io` file.** D8 requires every exclusion with source and reason. Files with neither story nor recorded exclusion (searched all 444 issue bodies by name): DirectByteBufferPool.scala (107 lines), ByteBufferCleaner.java, UnsynchronizedByteArrayInputStream.java, SelectionHandler.scala (387), WithUdpSend.scala, SimpleDnsManager.scala, DnsSettings.scala, dns/internal/ResolvConfParser.scala, DomainName.scala, DnsMessage.scala (DnsSettings is cited by 924/953/974 as a body source only; InetAddressDnsResolver only by GOV-23). Also `ByteString` members tied to the JVM (asByteBuffer(s), asInputStream, Java collection views) are not separated from the portable ones in IO-02..04. Open question: the exclusion record (source and reason) for each io file and for SimpleDnsCache/CachePolicy; the reason is an Owner decision under ADR 0033 D8/D13. No recommendation.

**F12 (medium) Reason-list attribution: "closed list of ADR 0033 D13" does not exist in ADR 0033.** 19 issues (#1070-#1088) say exclusions use "one closed exclusion reason from the list of ADR 0033 D13 (jvm-only, java-api, replaced-by-go-stdlib, replaced-by-goPekko-module, cluster-or-remote, network-io, owner-excluded)". ADR 0033 D13 (0033:47) says the exclusion list is closed but does not enumerate the seven codes; they are enumerated in the dossier (actor-typed-completion-design.md:342, section 7.1), which ADR 0033:64 (row D-10) names as "closed exclusion list (dossier 7.1)". Open question for the Owner: does D13 adopt the dossier list verbatim. No recommendation. Correction: none; see open question above.

**F13 (low) RTE-11/RTE-16 ring identity and hash.** The ring node key is `ConsistentRoutee.toString` = the routee's FULL address path string (ConsistentHashing.scala:443-457), hashed with `MurmurHash` (ConsistentHash.scala:149-151). Go requires Address/path values (ADR 0031 accepted) and an explicit decision on Pekko-compatible hashing; without it a differential V3 for key-to-routee mapping is impossible, only Go-side stability (recon already asks the question; add node-identity to the same decision). RTE-11 says the ring is a pure value with no Router dependency; true, but the node type needs ADR 0031 in its dependencies.

**F14 (low) FSM-13 R3 wording.** "initialize ... MUST send the initial CurrentState-style notification" is inaccurate. `initialize()` = `makeTransition(currentState)` (FSM.scala:719-720); the startWith State has `notifies = true` (:263), so it runs `onTransition` handlers and gossips `Transition(self, s, s)` (:888-891) to subscribers registered at that moment (spec: FSMTransitionSpec:80-87 "expectMsg(0 -> 0) // caused by initialize()"). `CurrentState` is only the reply to Subscribe/Listen (:834,839). Correction: R3/AC-R3-1 should say Transition(fsm, s, s) plus user onTransition callbacks.

**F15 (low) FSM-10 missing composition rule.** `whenUnhandled(f)` sets `handleEvent = f.orElse(handleEventDefault)` (FSM.scala:707-708): events not matched by a custom handler still produce the default warning. Add as R4. FSM-09 AC3 (callback failure) is correctly flagged by recon (terminate has no try/catch :937-940).

**F16 (low) Timer-origin events and `replying`.** State timeouts and named-timer fires are processed with `processMsg(StateTimeout, "state timeout")` / `processMsg(msg, t)` (FSM.scala:816,828) and replies go to `sender()` (:874,886); who the sender is for timer-origin events is not stated in FSM-03/04/05 (Pekko, from code reading only, not executed and no spec: state-timeout sender resolves to the implicit self in FSM.scala:898; named-timer sender resolves to the default noSender in FSM.scala:139-148 / Scheduler.scala:431. Pending verification by execution: sender() seen by replying() in a timer-origin event.). Correction: verify and record; needed by FSM-03 AC2 "original sender".

**F17 (low) ByteString semantic for Go.** IO-02 AC1 (no observation of later caller mutation) matches Pekko copying `apply` (ByteString.scala:36,1169-1170) while `fromArrayUnsafe` (:94) shares. Go has no immutable byte slice, so `replaced-by-go-stdlib` for IO-02/03/04 means "convention", and AC1-3 become documentation rules, not enforceable properties. This is the real content of the IO-01(b) Owner decision; alternatives in section 4 O-5.

**F18 (low) IO-05 / IO-18 details.** IO.scala is a 3-line accessor (IO.scala:24-37): the "network-io" reason applies to the managers it would hold, not to the accessor itself; record that if managers are excluded the accessor has no purpose. `Register.useResumeWriting` defaults to **true** (Tcp.scala:187), so IO-18's "requires Register(useResumeWriting)" is a default-on mode.

**F19 (low) RTE-18 inaccuracies and gaps.** "nine wrapper pairs": Pekko has six Pool+Group pairs (RoundRobin, Random, Broadcast, ScatterGather, TailChopping, ConsistentHashing), plus SmallestMailboxPool and BalancingPool (no group): eight families, 14 types (grep-verified locations above). Its dependency table omits RTE-03 #944 and the logic stories though the Contracts section lists them as consumed. Pool/Group knobs `usePoolDispatcher`, `routerDispatcher`/`withDispatcher`, `withSupervisorStrategy`, `withResizer`, `routingLogicController` (RouterConfig.scala:67,76,117-141,208-219) are in no story.

**F20 (low) RTE-17 vs RTE-14 overlap and dependency asymmetry.** Both use the cell mutation primitives (`addRoutees/removeRoutees(stopChild)`, delayed `PoisonPill` 100 ms via `stopIfChild`, RoutedActorCell.scala:84-114). RTE-14 depends on PoisonPill #770, GOV-07 #1038 and TIM-02 #821; RTE-17 does not, although RemoveRoutee(stopChild=true) and negative AdjustPoolSize need the same stop (RTE-17 lists "PoisonPill-like or direct stop" as open). Also RTE-14 recon cites AdjustPoolSize (:206-214) for the resizer shrink: the resizer path does not use AdjustPoolSize, it calls `removeRoutees` directly (Resizer.scala:303-307); AdjustPoolSize is RTE-17. Correction: make RTE-14 depend on the shared mutation primitive (RTE-02/17 split) and align stop dependencies.

**F21 (low) Stale AC text kept in bodies.** Bodies were "preserved unchanged" while recon lists AC changes required; an implementer reading the body gets contradictory ACs (RTE-08, RTE-06, RTE-09, RTE-12, RTE-13, FSM-02 AC1/AC3, FSM-03 AC3, FSM-07 AC3, FSM-09 AC3, IO-01). Acceptable only while blocked; require the AC rewrite as a gate before `status:ready` (recon already says ready only when contract accepted).

**F22 (low) Mis-citation of "ADR 0010 D4".** 16 occurrences in the RTE bundle cite "ADR 0010 D4 (routers excluded for now)". In ADR 0010 the decision table's D4 is non-blocking `AskFrom` inside actors (0010:235); the router exclusion is the "Excluded for now" bullet in Consequences (0010:225); the binding rule is ADR 0033 D4. Correction: cite 0010:225 and ADR 0033 D4.

**F23 (low) Dossier vs plan vs bodies on ByteString/routers.** Dossier U-01 (actor-typed-completion-design.md:~170) already states ByteString "replaced by `[]byte` ... X"; delivery plan 2.3 says PENDING on #808 with a lean; IO-01 says Owner decision O-6 pending. ADR 0033 D8 text says nothing about ByteString (only sockets/protocols). Dossier R-01 scopes Classic routers "R (subset)" while GOV-12 is an open Owner question and D9 only includes Typed routers. State these as 2 Proposed everywhere; do not treat dossier rows as decisions.

## 4. Owner decisions needed (alternatives and consequences; no recommendation)

- **O-1 Classic routers (GOV-12 #1042) and advanced routers (GOV-13 #1064).** Alternatives: (a) include local logics with pool/group wrappers; (b) only what Typed pool/group needs; (c) exclude Classic, Typed only. Consequences: (a) ~14 wrapper types and F2/F3 API work; (b) smallest shared port but F1 says Typed has its own logic so (b) may reduce to the ring value only; (c) RTE-01..20 become exclusion records; V3 oracle for Typed unaffected. Recommendation: none (Owner decision pending; Pekko evidence for each capability listed in section 2).
- **O-2 FSM facility (GOV-14 #1043).** Alternatives: include DSL (builder/options struct in Go) / exclude and document `state in struct` (ADR 0010). Consequences: include = 14 stories, timers + stash + listeners dependencies; exclude = Typed `Behaviors` cover state machines. Recommendation: none (Owner decision pending; Pekko evidence for each capability listed in section 2).
- **O-3 Untyped protocol messages on `Ref[T]` (F3).** Alternatives: routers/FSM only over `T = any`; sealed sum types per router; separate control port. Consequence: decides every wrapper/management message. Recommendation: none (Owner decision pending; Pekko evidence for each capability listed in section 2).
- **O-4 Exclusion reason vocabulary (F12).** Alternatives: accept the dossier list as ADR text; amend ADR 0033 D13. Recommendation: none (Owner decision pending; Pekko evidence for each capability listed in section 2).
- **O-5 Byte values (IO-01b).** Alternatives: (a) `[]byte` plus documented ownership/copy rule (no type); (b) small opaque immutable `bytes` type in a leaf package; (c) exclude entirely, keep `[]byte` in serializers. Consequences: (a) no code, AC1-3 of IO-02/03/04 become docs; (b) new export needs ADR 0011 justification and a dependency-direction decision (actor imports nothing); (c) GOV-33 id 21 serializer falls with it. Recommendation: none (Owner decision pending; Pekko evidence for each capability listed in section 2).
- **O-6 Network IO (GOV-20..23, 39).** Alternatives per D8: exclude `network-io`; adapter over `net`; exclude now and reopen. DNS: OS resolver (`net.Resolver`) is the only candidate for a "concrete local capability" (recon IO-15 caveat); decide per capability (resolve, cache, async queries). Recommendation: none (Owner decision pending; Pekko evidence for each capability listed in section 2).
- **O-7 Behavior choices already flagged in stories:** RTE-20 R2 (silent drop without sender vs always publish), RTE-16 key kinds, RTE-17 delayed stop, FSM-12/07 terminated-listener cleanup (divergence), FSM-13 keep onTermination on restart, FSM-08 debug logging scope. Recommendation: none (Owner decision pending; Pekko evidence for each capability listed in section 2).

## 5. ADRs needing creation, amendment or acceptance (none changed here)

- Architect proposal (not an Owner decision): New ADR (RTE-01 deliverable): routing boundary, including F2 (sender-side routing), F3 (typed refs), sender preservation, empty-route outcome; depends on ADR 0032 (PR #1014, not in develop) for sender; ADR 0010 `Status: proposed` is the gate record, ADR 0033 D4 the rule.
- Architect proposal (not an Owner decision): New ADR (FSM-01 deliverable): FSM DSL and F3; plus whether `Listeners` is generic (F9).
- Architect proposal (not an Owner decision): New/updated ADR for byte values and for `network-io` exclusions (D8) and the A-06 register; ADR 0021 O7 (open, per #1190) interacts via primitive serializer id 21.
- Architect proposal (not an Owner decision): ADR 0008 (proposed): routers "transparent" row needs reconsideration for ask-based routers; add to impact lists.
- Architect proposal (not an Owner decision): ADR 0017 (accepted) interplay: router supervision default (Pool.defaultSupervisorStrategy = OneForOne escalate, RouterConfig.scala:182-184; RoutingSpec:185 "default to all-for-one-always-escalate" test title mismatches the code: spec title says all-for-one, code says OneForOneStrategy; test body (RoutingSpec.scala:185-205) shows escalation to the router's supervisor which restarts the router and its three routees; the title says all-for-one, the code is OneForOne+Escalate) belongs in RTE-07's supervision decision.
- Architect proposal (not an Owner decision): ADR 0031 (accepted): ring identity (F13). ADR 0014: routee children/watch.
- Architect proposal (not an Owner decision): ADR 0033 text: if O-4 accepted, amend D13 interpretation.

## 6. Blocked vs potentially startable

- Everything RTE and FSM is `status:blocked`, P3, gated by GOV-12 #1042 / GOV-14 #1043 (P1, ready: these two proposal stories are the only startable items; they need no producer). RTE-10/12/13/14/15 additionally by GOV-13 #1064 (itself blocked on GOV-12), GOV-10 #1040, TIM-01/02/03, ASK-02/03/05, MBX-01 (+F8 port), MSG-06/GOV-07.
- RTE-11 #880 (ring value, pure) has no technical prerequisite beyond GOV-12 and the hash/node-identity decision (F13): the first RTE item that could start after O-1. RTE-01 #796 requires MSG-01/02 (sender), GOV-09/IDN-06 (selection routees) per recon, but selection routees can be deferred (split AC "exact-reference" first).
- FSM chain: FSM-01 -> FSM-02 -> FSM-03 -> (06, 07, 08, 09, 04); FSM-04 needs TIM-01/02; FSM-05 needs TIM-02/03/04, FSM-09, and PoisonPill gate; FSM-11 needs BHV-06; FSM-13 needs SUP-10 #781 (P2 ready, deliverable independently of FSM).
- IO: all `status:blocked` with no priority; none startable as implementation. Startable as documents: GOV-20/21/22/23/39 (P3 ready) and, after the F10 split, the byte-value decision.
- Cross-epic consumers: TSD-04/05 (P2), JDG-10 (P2), GOV-43 (P1), GOV-33, SER-09 wait on RTE/IO outcomes (F1, F10).

## 7. Pekko references with no spec; items not read

- No spec found (searched `NoRoutee` in all *Spec.scala/*Test.java; `setStateTimeout` across upstream; Listen/Deafen on FSM): NoRoutee dead-letter path (RTE-20), `setStateTimeout` behavior (FSM-14), FSM Listen/Deafen (FSM-12; ListenerSpec covers the generic trait), missing-startWith IllegalStateException and "Next state does not exist" after initialize (FSM-02 recon admits), restart-specific FSM behavior (FSM-13; FSMActorSpec:188 is stop, not restart), AbstractFSMWithStash only a Java compile smoke test.
- Not read (limits of this audit): RoundRobinSpec :61-110,130-175, RandomSpec, BroadcastSpec, BalancingSpec bodies, ResizerSpec, MetricsBasedResizerSpec, ConfiguredLocalRoutingSpec bodies, RouteeCreationSpec beyond :26-32, TimerSpec, FSMTimingSpec bodies except :33-72,136-152, FSMActorSpec except :74-80,148-154,223-262, AbstractFSM.scala, MurmurHash.scala, OptimalSizeExploringResizer body, SimpleDnsCache, InetAddressDnsResolver, async DNS sources, Inet.scala, all Tcp/Udp/Dns/ByteString specs (TcpConnectionSpec, TcpListenerSpec, UdpIntegrationSpec, ByteStringSpec), Udp/Tcp semantics beyond symbol/line verification, dossier/plan sections other than those cited, GOV-14 #1043 body, MBX/TIM/SUP/ASK/MSG/SYS stories referenced as dependencies (other groups), `routingLogicController` consumers (cluster), `RoutingSpec.scala:185` body.
- I did not execute Pekko or Go; all Pekko behavior statements are source/spec reading. Line numbers verified by opening the files at tag v1.7.1.
- Recon anchors checked and found correct: Router.scala 33,51,58,66,76,83,128,135-136; Random.scala 38-41; RoundRobin.scala 39-45; SmallestMailbox.scala 48,64-93; ConsistentHash.scala 30,34,49,68; ConsistentHashing.scala 54,67,158,205-257; ScatterGather :62; TailChopping 86,91,104-116; RoutedActorCell 71-91,103-110,159-187,207-214; FSM.scala 282,475,487,500,511,650,655,682,698,707,719,830-843,881-909,928-941,958-1011; Tcp/Udp/UdpConnected/Dns symbol lines listed above. Imprecise anchors: RTE-14 "RouterConfig.scala:226-231" is a doc comment (code: Resizer.scala:279-283); RTE-19 recon "RouterConfig.scala:283-325" fine; IO-15 "cached :76" (line 76 is `object Dns`; `cached` is at Dns.scala:51,66,121,139).
