# Audit report: group idn-msg-ask (epics #747 IDN, #748 MSG, #750 ASK)

Baselines: Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (paths relative to `upstream/pekko`); goPekko `origin/develop` `9abfb40` snapshot (`tmp/develop`, cited as `pekko-actor/...`); issue bodies fetched 2026-10-25 (bundles). ADR 0032 read only from PR #1014 (open, unmerged, head `71854b1`) via `gh api` GET; treated as NOT in develop. PR #1011 (IDN-14 implementation) is open, unmerged (`cf2ad31`).
Pekko JVM was never run; every Pekko statement below is source or spec reading. "Spec" = `*Spec.scala` read by me.

Depth markers: DEEP = story body + Pekko source + Pekko spec (or recorded "no spec") + the accepted/proposed ADR text + Go code. DEEP* = DEEP but one named source file was not opened. SHALLOW = partial, reason given.

## 1. Coverage inventory (44 issues)

State = decision-state taxonomy 1-7 (stated per story outcome). prio/ready from labels (blank = no label on the issue).

### IDN (#747, 22)
| # | Key | Issue state | prio | ready | Depth | Taxonomy |
|---|---|---|---|---|---|---|
| 765 | IDN-01 | CLOSED | - | - | SHALLOW (names.go:62-71 vs Children.scala:123-131 and Helpers.scala:118-123 verified; anonymous_name_internal_test.go names read; no Pekko spec searched; no judge scenario) | 6 |
| 766 | IDN-02 | CLOSED | - | - | DEEP (ADR 0031 read in full; ActorPath.scala 124-148, 196-212, 284-311, 476-494; ActorPathSpec.scala) | 1 (ADR 0031 accepted 2026-10-24); delivered as contract |
| 767 | IDN-10 | OPEN | P2 | blocked | SHALLOW (Props.scala:180-217 read; PropsCreationSpec test names only; Deployer.scala not opened) | 4 |
| 812 | IDN-03 | OPEN | P2 | blocked | DEEP* (ActorPathSpec:25-57, RelativeActorPath, ADR 0031 O2/P-8; `AddressFromURIString` Address.scala:185-232 body not opened) | 4 (grammar undecided, ADR 0031 O2 only excluded it) |
| 813 | IDN-11 | OPEN | P2 | blocked | DEEP (Props.scala:205-217, Deployer.scala:75,119, ActorContextImpl.scala:167, ActorLoggingSpec.scala:275-287) | 4 |
| 852 | IDN-04 | OPEN | P2 | blocked | DEEP (ActorPath.scala:196-212,286; ADR 0031 P-4/P-5) | 5 (contract accepted P-4; not implemented) |
| 853 | IDN-05 | OPEN | P2 | blocked | DEEP (ADR 0024:203, registry_characterization_internal_test.go:442, ref.go; ActorRef.scala equals not reopened) | 4 for the path accessor; ID identity part is 6 |
| 893 | IDN-06 | OPEN | P3 | blocked | DEEP (ActorSelection.scala:216-295, ADR 0010:225, GOV-09 body) | 4 |
| 894 | IDN-09 | OPEN | P3 | blocked | SHALLOW (ActorPath.scala:248 only by citation; ADR 0031 P-7/P-8 read) | 4 |
| 895 | IDN-12 | OPEN | P3 | blocked | DEEP (FaultHandling.scala:260-266,318-334, Children.scala:240-252, ADR 0014 deviation 8/D2, stop.go:105-120) | 3 (reverses accepted ADR 0014 D2 "Keep") |
| 933 | IDN-07 | OPEN | P3 | blocked | DEEP (ActorSelection.scala:216-295, ActorSelectionSpec:395-425) | 4 |
| 934 | IDN-08 | OPEN | P3 | blocked | DEEP (ActorCell.scala:575-579, ActorRef.scala:690-713, Actor.scala:76-95, ADR 0015 Q3) | 4 |
| 1007 | IDN-13 | CLOSED | - | - | DEEP (Address.scala:37-44,62-75,84-95,136-146 vs address.go; address_test.go names) | 6 |
| 1008 | IDN-14 | OPEN | - | ready (stale; PR #1011 in flight) | DEEP (ActorPath.scala:77-148,203,284-311; ActorPathSpec:58-76; ADR 0031 O4/O5; names.go) | 5 (implementation PR open) |
| 1009 | IDN-15 | CLOSED | - | - | DEEP (system.go:64-127, ActorRefProvider.scala:402, ActorSystem.scala:263,838-843) | 6 |
| 1023 | IDN-16 | OPEN | P2 | blocked | DEEP (ActorSelection.scala:50-61,216-295, ActorSelectionSpec:361-368, ActorRef.scala:690-713) | 4 |
| 1053 | IDN-18 | OPEN | P3 | blocked | DEEP (ActorPath.scala:307-311,481-494; ActorPathSpec has no compareTo) | 5 (ADR 0031 P-8 rule exists, but see F-11) |
| 1089 | IDN-17 | OPEN | P3 | blocked | DEEP (Address.scala:143-178 split(), RelativeActorPathSpec:24-41) | 4 |
| 1090 | IDN-19 | OPEN | P1 | ready | DEEP (hierarchy.go:405-422, ActorCell.scala:160-170, ActorRefSpec:465-476, ChildrenContainer not reopened for R2) | 6 for R1/R3 scope; R2 oracle applicability pending determination (D13) |
| 1091 | IDN-20 | OPEN | P3 | blocked | DEEP (ActorSelection.scala:262-289, ActorSelectionSpec:395-425) | 4 |
| 1092 | IDN-21 | OPEN | P3 | blocked | SHALLOW (ActorRefProvider.scala:145,151,622 by citation only; ADR 0031 P-7, ADR 0024 read) | 4 |
| 1093 | IDN-22 | OPEN | P2 | blocked | DEEP (ActorSelection.scala:79-99, ActorSelectionSpec:315-335, AskSupport.scala:92-135) | 4 |

### MSG (#748, 10)
| # | Key | State | prio | ready | Depth | Taxonomy |
|---|---|---|---|---|---|---|
| 768 | MSG-01 | OPEN | none | none | DEEP (ADR 0032 read in full from PR #1014; ActorCell.scala:140-148; ADR 0008, 0010, 0030 P-1) | 2/1 mixed: ADR 0032 D1,D2,D3,D4,D6,D7 accepted by Owner 2026-10-24 (message quoted in ADR), D5 open; ADR unmerged |
| 769 | MSG-04 | OPEN | P2 | blocked | DEEP (AbstractDispatcher.scala:40-52, ActorRef.scala:683-690, 733-740, ref.go:44-65, fidelity audit row 36) | 4 |
| 770 | MSG-06 | OPEN | P1 | blocked | DEEP (Actor.scala:40-62, ActorCell.scala:566-580, ActorRefSpec:422-440, DeathWatchSpec:140-160) | 4 |
| 814 | MSG-02 | OPEN | P1 | blocked | DEEP (ActorCell.scala:140-148, ADR 0032 rules 1-6 and compat plan) | 5 for ordinary delivery (D1,D2,D4 accepted); 4 for Terminated/WatchWith sender (D5 open) |
| 815 | MSG-05 | OPEN | P2 | blocked | DEEP (same sources as MSG-04) | 4 (gated by MSG-04 decision) |
| 816 | MSG-07 | OPEN | P1 | blocked | DEEP (ActorCell.scala:575, FaultHandling.scala:215-222, ActorRefSpec:442-463, RoutingSpec:158-168, DeathWatchSpec:196-210, ActorLifeCycleSpec:49-110) | 4 |
| 854 | MSG-03 | OPEN | P1 | blocked | DEEP (ActorRef.scala:158-164, ForwardActorSpec:28-60, ADR 0032, ADR 0010 D8, ADR 0015 D8) | 5 (D1,D4 accepted) |
| 896 | MSG-08 | OPEN | P1 | blocked | DEEP (ActorRefProvider.scala:66-73, ActorRef.scala:525-545,690-745) | 4 |
| 1031 | MSG-09 | OPEN | P1 | blocked | DEEP (FaultHandling.scala:215-222, supervision_decider.go:50-62, ActorSystemSpec:329-334, SupervisorHierarchySpec:770-790, DeathWatchSpec:196-210) | 4 (validation story) |
| 1099 | MSG-10 | OPEN | P2 | blocked | DEEP (Actor.scala:46,60, ActorRef.scala:690-745, DeadLetterSupressionSpec:38-62) | 4 |

### ASK (#750, 12)
| # | Key | State | prio | ready | Depth | Taxonomy |
|---|---|---|---|---|---|---|
| 773 | ASK-01 | OPEN | P1 | blocked | DEEP (ADR 0015 D1-D9 read; AskSupport.scala:92-135,345-390; ask.go; ask-from-receive-proposal.md) | 3 (ADR 0010 D4 vs ADR 0015 D1/D7) |
| 774 | ASK-07 | OPEN | P2 | blocked | DEEP (StatusReply.scala:37-105,173; StatusReplySpec:31-110; ADR 0015 D3/Q4) | 3 (ADR 0015 Q4 deferred vs plan C-28) |
| 775 | ASK-11 | OPEN | P2 | ready | DEEP (ask_terminated_internal_test.go:35-80, stop.go:105-120, hierarchy.go:225-262, AskSupport.scala:653-678,711-744) | startable test fix; no taxonomy contract; 6 once done (oracle applicability pending determination per D13) |
| 818 | ASK-02 | OPEN | P2 | blocked | DEEP (AskSupport.scala:345-390; proposal doc; ADR 0010 D4, ADR 0015 D1/D7) | 3 |
| 819 | ASK-09 | OPEN | P3 | blocked | DEEP (AskSupport.scala:312-319,372-376; AskSpec:41-59; judge I4; ADR 0015 Q2/D5) | 3 (reverses Owner decision Q2) |
| 820 | ASK-10 | OPEN | P3 | blocked | DEEP (AskSupport.scala:722-743; AskSpec:94-127; errors.go:55-68; ask.go) | 4 |
| 856 | ASK-03 | OPEN | P2 | blocked | DEEP (PipeToSupport.scala:25-60; PipeToSpec:1-90) | 4 |
| 857 | ASK-04 | OPEN | P2 | blocked | DEEP (PromiseRef.scala:80-150 partial, PromiseRefSpec:37-80, AskSupport.scala:590-680, ask_endpoint.go) | 4 (ADR 0015 D8 proposed) |
| 858 | ASK-06 | OPEN | none | blocked | DEEP (Actor.scala:320-326, AskSupport.scala:614-629, ADR 0015 Q3/Q4) | 3 (contradicts accepted-by-Owner ADR 0015 Q3/Q4) |
| 859 | ASK-08 | OPEN | P2 | blocked | DEEP (AskSupport.scala:100-113,345-352, StatusReplySpec:67-107) | 3 |
| 936 | ASK-05 | OPEN | P3 | blocked | DEEP (AskSupport.scala:92-135, AskSpec:128-190, ActorSelection) | 4 |
| 1024 | ASK-12 | OPEN | P3 | blocked | DEEP (PipeToSupport.scala:25-60, PipeToSpec:63-90) | 4 |

Counts: DEEP 40 (of which DEEP* 2: #812, #1090), SHALLOW 4 (#765, #767, #894, #1092), NOT REVIEWED 0. Total 44.

## 2. Capability matrix

Go state = `develop/pekko-actor` read by me.

| Capability | Pekko contract | Go accepted/proposed | State | Go implementation | Evidence | Gap |
|---|---|---|---|---|---|---|
| Anonymous child names | Children.scala:123-131 randomName; Helpers.scala:118-123 base64 (`$` + low 6 bits first) | IDN-01 closed; ADR 0031 "Go today" | 6 | names.go:62-71 `generateName`; spawn.go:98 `SpawnAnonymous` | anonymous_name_internal_test.go (sequence, concurrency, no-reuse); no Pekko spec found (searched ActorRefSpec, ActorPathSpec, ActorLifeCycleSpec by name only); no judge scenario | oracle applicability under D13 pending determination; no approval recorded |
| Local Address value | Address.scala:37-44,62-75,84-95,136-146 | ADR 0031 P-1..P-3, O1, O3c (accepted) | 6 | address.go:34-90 | address_test.go (10 tests); no Pekko spec; ActorPathSpec:40-47 only covers parsing | Go rejects empty protocol/system (address.go:42-51); Pekko does not (Address.scala:43) and ADR 0031 "Intentional differences" (a)-(g) omit it (F-12) |
| System name + local address | ActorSystem.scala:263,840-843; ActorRefProvider.scala:402 | ADR 0031 O3a-c | 6 | system.go:64-127 | tests per README | Name used unvalidated (system.go:70-74) vs Pekko IllegalArgumentException; not listed as intentional difference (F-12) |
| ActorPath value (root, child, validation, equality, text) | ActorPath.scala:77-148,284-311,328-335,449-494; ActorPathSpec:58-76 | ADR 0031 P-4..P-8, O4, O5 | 5 | absent in develop (PR #1011 open) | none yet | PR #1011 unmerged; label still ready |
| Path parse | Address.scala:170-232 URI-based; ActorPathSpec:25-57 | none (ADR 0031 O2 excluded; GOV-40 #1187 pending) | 4 | absent | - | grammar ADR missing |
| Relative path text | Address.scala:143-178; RelativeActorPathSpec:24-41 | none | 4 | absent | - | F-6 (empty segments not dropped upstream) |
| Path navigation, ordering | ActorPath.scala:203-209,286; compareTo 307,481 | ADR 0031 P-4, P-8 (compareTo in "Pekko contract", not in "Contract") | 5/4 | absent | no upstream compareTo spec | F-11 |
| Ref reports its path / incarnation identity | ActorRef.scala:175-190 equals(path+uid); ActorPath.scala uid | ADR 0024:203 identity=ID | 4 (path accessor) / 6 (ID identity) | ref.go:23-28 (ID only); registry_characterization_internal_test.go:442 | Go tests only | accessor surface undefined (F-7) |
| Children lookup/list | ActorCell.scala:160-170; Children.scala:44-55; ActorRefSpec:465-476 | ADR 0014 API block; impl-decisions O1 | 6 | hierarchy.go:405-422 | hierarchy_test.go:25, hierarchy_internal_test.go:161,203 | public-API tests + README text missing (as story says) |
| Spawn descriptor (Props/Deploy) | Props.scala:180-217; Deployer.scala:75,119 | ADR 0010:225 excludes dispatcher/mailbox/routers; GOV-10/11/12 | 4 | `SpawnOption` spawn.go:34 | - | descriptor shape undecided |
| Actor tags | Props.scala:124,205-217; Deployer.scala:68-75; ActorCell.scala:74; typed ActorLoggingSpec.scala:270-284 | none | 4 | absent | Classic: public `Props.deploy.tags` / `ActorContext.props` (code; no Classic spec in actor-tests/src); Typed MDC key `pekkoTags` (spec, actor-typed-tests) | F-8 |
| Name release timing | FaultHandling.scala:260-266,327-334; Children.scala:246 | ADR 0014 D2 "Keep" (accepted design), ADR 0024 I1 | 3 | stop.go:105-120 releases name (detach) before watchers | characterization tests only | story reverses an Owner decision |
| Selection (exact) / wildcard / no-match | ActorSelection.scala:50-61,216-295; ActorSelectionSpec:361-425 | ADR 0010:225 excludes; GOV-09 pending | 4 | absent | - | F-9 |
| Identify / ActorIdentity | ActorCell.scala:578; ActorRef.scala:698-700; Actor.scala:76-95 | ADR 0015 Q3 typed-only vs untyped Identify | 4 | absent | - | Go shape undefined |
| resolveOne | ActorSelection.scala:79-99; ActorSelectionSpec:315-335 | none | 4 | absent | - | F-10 (dependency) |
| Sender in envelope / sender() / forward | AbstractDispatcher.scala:44-50; ActorCell.scala:140-148; ActorRef.scala:158-164; ForwardActorSpec | ADR 0032 D1-D4,D6,D7 accepted (unmerged); D5 open; ADR 0010 D2 "no implicit sender" | 5 | absent (`Tell(ctx,msg)` only, ref.go:44) | none | F-1 (ADR text), F-2 (Handle can't reply) |
| Nil message | AbstractDispatcher.scala:44-50; ActorRef.scala:685,737; ActorRefProvider.scala:472; AskSupport.scala:619 | none; ref.go:42 "never fails" | 4 | ref.go:44 delivers it | no Pekko spec found (searched InvalidMessageException in actor-tests: only typed specs) | P1 in fidelity audit vs P2 in ACT2 (F-13) |
| PoisonPill | Actor.scala:40-50; ActorCell.scala:576; ActorRefSpec:422-440 | ADR 0010:225 excludes; GOV-07 | 4 | absent (`Stop` is immediate) | Pekko spec ActorRefSpec:422-440 shows messages ahead processed | API shape for Ref[T] undefined |
| Kill | Actor.scala:52-62; ActorCell.scala:575; FaultHandling.scala:215-222 | ADR 0010:225; ADR 0017:495 (gap #450); SUP-16/SUP-02 | 4 | absent; defaultDecider (supervision_decider.go:57-62) restarts any non-listed cause | none | F-4 (interim restart) |
| deadLetters / ignoreRef | ActorRefProvider.scala:66-73; ActorRef.scala:525-545,733-745 | ADR 0032 D3 "revisit with #896" | 4 | absent (`DeadLetter` has Msg/To only, deadletter.go:22) | - | F-3 circular D3 |
| Suppressed dead letters | ActorRef.scala:709-724; DeadLetterSupressionSpec:38-62 | EVT-05 | 4 | absent | - | depends on P0 EVT-05 |
| Ask (blocking) | AskSupport.scala:345-390,614-629,722-743 | ADR 0015 D1,D2,D5 ([Owner]), D3,D4,D7,D8,D9 [Proposal] | 6 | ask.go:104; ask_endpoint.go | ask_*_test.go (AK/K/C series), judge G8, I4, I5 | ADR 0015 header still "proposed" |
| Async Ask / AskFrom | AskSupport.scala:345-390 | ADR 0010 D4/D7 vs ADR 0015 D1/D7 | 3 | absent (README:573) | - | F-5 |
| pipeTo / pipeToSelection | PipeToSupport.scala:25-60; PipeToSpec:1-90 | none | 4 | absent | - | oracle exists (PipeToSpec) |
| PromiseRef | PromiseRef.scala:80-150; PromiseRefSpec:37-80 | ADR 0015 D8 [Proposal] | 4 | unexported askEndpoint only | - | spec exists, story says n/a |
| StatusReply / askWithStatus | StatusReply.scala; StatusReplySpec:31-107; AskSupport.scala:100-113,345-352 | ADR 0015 Q4 deferred [Owner] | 3 | absent | - | contradiction |
| Status.Failure as ask failure | Actor.scala:320-326; AskSupport.scala:620-624 | ADR 0015 Q3/Q4 | 3 | absent by design (ask.go:98-100 doc) | - | exclusion proposal |
| Ask timeout context | AskSupport.scala:722-743; AskSpec:94-127 | ADR 0015 D3 sentinel identity | 4 | errors.go:55-68 sentinels | - | F-14 |
| Ask terminated recipient | AskSupport.scala:312-319,372-376 (AskTimeoutException "had already been terminated") | ADR 0015 Q2/D5 Owner: ErrRecipientTerminated | 1 (ADR 0015 D5 Q2 accepted; differs from Pekko, which fails with AskTimeoutException, AskSupport.scala:312-319,372-375; no Owner decision under ADR 0033 D13 recorded in the audited files) | ask.go:111-130 | TestAskTerminated_K1..K8; judge I4 | story would reverse |
| GracefulStop | GracefulStopSupport.scala:59 | not assigned to any story in this group | - | absent | - | F-15 (see section 7) |

## 3. Findings

**F-1 [high] ADR 0010 D2 / 0017:691 / 0030 P-1 vs ADR 0032 unresolved while stories cite 0032 as binding.** ADR 0032 is on PR #1014 (open, unmerged, head 71854b1); #814, #854, #896, #1023, #1024, #1091, #1093, #1099 cite "ADR 0032 D3/D4/D6 (accepted)" as Accepted decisions. Within develop the only text is "no implicit sender" (ADR 0010:233, D2 "proposed"), ADR 0030:152 P-1 "DeadLetter carries no sender ... accepted adaptation, ADR 0010 D2", ADR 0017:691. ADR 0032:143 says 0010 "stays as written", so the contradiction is by design left open (ADR 0033 consequences also lists it). Correction: merge PR #1014 first (or mark every dependent story "waits for PR #1014"), and add an explicit amendment note to ADR 0010 D2, 0030 P-1, 0017:691 pointing to 0032 D1/D4/D6. #768 itself (MSG-01) is OPEN with no labels and no reconciliation block; it is the decision slice whose product is PR #1014 and it stays an open `depends_on` of #814 and #769.

**F-2 [high] MSG-02 (#814) has no acceptance criterion that lets a receiver reply to the sender.** ADR 0032 D2 types the sender as `Handle` (identity only, no `Tell`; hierarchy.go:405-409, README:174-180) and states "sending to it needs a typed ref, which MSG-02 defines" (ADR 0032 observable difference 5; compatibility plan item 3). #814 AC1-AC3 and the reconciliation add nothing for it. Pekko `sender() ! x` is the point of the story (ActorCell.scala:140-148; ForwardActorSpec:28-36 `sender() ! x`). Without the conversion MSG-02/MSG-03 only transport an opaque handle. Pending verification/decision: the story has no AC for replying to the sender; Pekko evidence ActorCell.scala:145, ForwardActorSpec.scala:28-36 (`sender() ! x`). No correction proposed.

**F-3 [medium] Circular reference on ADR 0032 D3.** #896 (MSG-08) takes D3 as binding ("DeadLetter sender follows D3/D4/D6"), but ADR 0032 D3 says "absent, explicit; revisit once #896 defines deadLetters". #1023 states the dead-letter sender substitution is "open until MSG-02 lands" but ADR 0032 assigns that revisit to #896. Pekko substitutes `deadLetters` when no sender (AbstractDispatcher.scala:44-50 `system.deadLetters`; ActorRef.scala:712,722). Correction: make the D3 revisit an explicit AC of #896 and fix #1023/#1091 to reference #896, not #814.

**F-4 [medium] Kill sequencing leaves an interim wrong default.** #816 (Kill) is independent of #778 (SUP-02, default decider mapping ActorKilledException to Stop; Pekko FaultHandling.scala:215-222). Go's defaultDecider (supervision_decider.go:57-62) maps every unlisted cause to Restart. If MSG-07 lands before SUP-02, a Kill under the default strategy restarts the child, the opposite of Pekko, until MSG-09 (#1031). Only a delivery order prevents it. Also: Pekko specs for Kill under a custom decider exist (ActorRefSpec:442-463, RoutingSpec:158-168, DeathWatchSpec:196-210, ActorLifeCycleSpec:49-110) but the reconciliation lists spec "n/a" for #816, and ActorSystemSpec:329-334 uses rootGuardianStrategy (ActorRefProvider.scala:518-522); SupervisorHierarchySpec:770-790 uses a manager with an empty AllForOne. DeathWatchSpec.scala:216-246 does use `SupervisorStrategy.defaultStrategy.decider` with `failed ! Kill` (:237) and its asserted sequence (:238-244) requires that `failed` terminated; Pekko evidence for default-decider Stop on Kill is therefore source (FaultHandling.scala:218-222) plus this spec (indirect, not asserted by name). Open question (Owner): whether #816 and #778 need a release order; the existing custom-decider specs listed above are candidate oracles for AC1/AC3 (what "applicable oracle" means is open, README section 4 item 6). No recommendation.

**F-5 [high] AskFrom contradiction is not just documentation; the Go context lacks any hook and three stories need it.** ADR 0010 D4 ("Non-blocking only" inside an actor; milestone 1 scope :224 includes `AskFrom`), D7 (refuse AskFrom in hooks, `ErrAskFromHook`, "Decided") versus ADR 0015 D1 (:116 "non-blocking AskFrom is out of scope") and D7 (:275 "Deferred"); README:573 "AskFrom not implemented"; docs/parity/ask-from-receive-proposal.md "proposal, no decision". ADR 0010 D4/D7 rows are marked "Decided" inside an ADR whose status is "proposed". #818 AC3 and #856 need a context-bound Ask. GOV-18 (#1068) is the gate; its alternatives (R2) cover keep-deferred vs schedule. Correction: nothing for ASK-02/03/04/08 can start before #1068 is decided; Pekko: ask is available inside an actor (AskSupport.scala:346-367; `ask(actorRef, message, sender)` :97). Open question: GOV-18 #1068 decision. Missing: Owner decision. No recommendation.

**F-6 [medium] #1089 R1 "empty segments dropped" contradicts Pekko.** `RelativeActorPath.unapply` returns `split(rawPath, rawFragment)` (Address.scala:149-162, 170-179); `split` keeps empty segments ("/user/a" yields "", "user", "a"; leading slash gives a leading ""). Empty names are dropped later, in `ActorSelection.apply` (ActorSelection.scala:216-228 `case x if x.nonEmpty`) and `ActorPath./(Iterable)` (ActorPath.scala:203-204). RelativeActorPathSpec:24-41 asserts only four non-empty cases; it does not assert dropping. Also the `foo/bar/baz#1234` case (spec :38) expects element `baz#1234` (fragment glued), which collides with ADR 0031 P-7 (`#` is an error in `Child`). Correction: R1 should state Pekko's `RelativeActorPath.unapply` returns `split(...)` with empty elements retained (Address.scala:149-160, 170-179; "/user/a" yields "", "user", "a"), and that empty names are dropped later by `ActorSelection.apply` (ActorSelection.scala:219-227) and `ActorPath./(Iterable)` (ActorPath.scala:203-204). RelativeActorPathSpec.scala:24-41 asserts four non-empty cases and `baz#1234` as one element. Open question (pending): how the `#uid` element is handled in Go; evidence missing: GOV-41. No recommendation.

**F-7 [medium] #853 (IDN-05) has no defined surface.** Reconciliation says outcome becomes "a Ref reports its ActorPath", but ADR 0032 compat plan rule 2 and #813 both say `Ref[T]` must not gain methods (breaks implementers), and `Handle` has only `ID()`. No surface (package function, `System.PathOf(h)`, `Context.Path()`) is named; anonymous ask endpoints (ask_endpoint.go) have no path. Also ADR 0031 P-7 assigns the "uid-bearing form" to #853 and #894, yet the rescoped #853 produces none, leaving #894/GOV-41 as sole owner. Title ("Report the local actor path of a reference") and user story ("retain an exact local actor identity across name reuse") of #853 disagree; the body was not rewritten. Correction: name the surface in the rewritten AC and decide the ADR 0011 justification (GOV-02) before any code.

**F-8 [medium] #813 (tags): Pekko Classic read surface and the Typed MDC oracle.** Pekko Classic exposes tags through public `Props.deploy.tags` (Deployer.scala:75, Props.scala:124) and `ActorContext.props` (ActorCell.scala:74); `Props.withActorTags` sets them. No Classic spec observes them (searched actor-tests/src). The only spec that observes tags is the Typed MDC test actor-typed-tests/.../scaladsl/ActorLoggingSpec.scala:270-284 (`ActorMdc.PekkoTagsKey` = "pekkoTags" -> "tag1,tag2"). Evidence: code (Classic read), spec (Typed MDC only). Go has no `Context.Log` (ADR 0033 D11 pending) and no Props. AC "tag inspection" has nothing to inspect through. Pending verification: which Go read surface, if any, the story requires. Missing evidence: the story's own AC text and an Owner decision under ADR 0033 D13; Pekko shows `Props.deploy.tags` (Classic) and the Typed MDC key only.

**F-9 [medium] GOV-09 (#1022) alternatives (a)/(b)/(c) do not cover questions that ~9 stories park there.** #1023 and #1093 defer to "#1022 Architect proposal" for typed vs untyped selection, timeout form, ADR 0011 export of resolve and not-found error; #893 returns "the current local reference" (a `Ref[T]` cannot be produced from a path without T, so only `Handle` is possible, as #1093 notes). #1022 R2 lists only (a) `System.Select(path)` + Identify, (b) exact lookup without wildcards or Identify, (c) exclude. Under (b), #933, #934, #1091, #1093 (all need wildcards/Identify) become dead, while #936/#1024/#1023 survive only if send-through-selection is in (b). Correction: add to #1022 R2 the questions typed/untyped, return `Handle` vs `Ref`, and a per-story survival table for each alternative.

**F-10 [medium] #1093 (resolveOne) depends on the wrong ASK story.** Pekko resolveOne = `this.ask(Identify(None))` on the selection (ActorSelection.scala:82), i.e. ask through a selection = ASK-05 (#936, AskSupport.scala:129-131) plus Identify (#934). #1093 lists ASK-12 (#1024, pipeToSelection, PipeToSupport.scala:37), which is a different mechanism; the story itself calls the dependency "only partly evidenced". Missing: #936. Correction: replace #1024 by #936 in #1093 and in the lane text; #936 AC2 "ambiguous" removal (already recommended) stays.

**F-11 [low] ActorPath ordering has rule text in ADR but not in its contract table.** ADR 0031 P-8 in the "Contract" table is the text rule; the compareTo rule is only item P8 in the Pekko-contract table (ADR 0031:56). #1053 says "ADR 0031 P-8 records ... right to left" and lists it as accepted binding. In the ADR the numbering clashes (Pekko-contract "P8" = compareTo; Contract "P-8" = text). Correction: in #1053 cite "ADR 0031 Pekko-contract row P8 (Source), not a Contract item". Open question: compareTo as a Contract item (no recommendation). Also note Pekko's root compare is `toString.compareTo` (ActorPath.scala:309) while child compare is name-by-name from the last element (ActorPath.scala:481-494); R1's wording "a root path orders before child paths of other roots by string" is a loose paraphrase of root-vs-child `1` (ActorPath.scala:310).

**F-12 [low] Two Go divergences are not in ADR 0031's "Intentional differences".** (1) `NewLocalAddress` rejects empty protocol/system (address.go:42-51; Pekko Address.scala:43 accepts), recorded only in README:228 and errors.go. (2) `Config.Name` is not validated (system.go:70-74) while Pekko throws IllegalArgumentException (ActorSystem.scala:840-843); this matters for the future parser and ActorPath text: a system name containing `/` or `@` breaks round-trips of IDN-03/IDN-09. Open question: system-name validation. Pekko: ActorSystem.scala:840-843 throws IllegalArgumentException for names not matching ^[a-zA-Z0-9][a-zA-Z0-9-_]*$; Address.scala:43 imposes no check. No recommendation.

**F-13 [medium] nil Tell priority inconsistent.** The fidelity audit lists nil Tell as P1 "observable Classic contract differs" (#452; docs/parity/actor-local-fidelity-audit-2026-10-22.md:36,210,219). ACT2 gives #769 P2 and #815 P2. Pekko throws `InvalidMessageException("Message is null")` from `!` of EmptyLocalActorRef/DeadLetterActorRef/PromiseActorRef (ActorRef.scala:685,737, AskSupport.scala:619) and Envelope.apply (AbstractDispatcher.scala:44-50), so in Pekko the exception is thrown to the *sender*, not the receiver. No Pekko spec exercises it (searched InvalidMessageException in actor-tests: no hit; only typed specs). Correction: note the priority inconsistency and that no Pekko spec exists for Classic `!` (oracle applicability pending determination, D13). The enumeration AC must also cover Go reality: nil only exists for pointer/interface/map/func/chan/slice T; a non-nil zero value is valid; the Go rejection channel is an open Owner decision since `Tell` has no return.

**F-14 [medium] #820 (ASK-10): oracle exists, listed "n/a".** AskSpec.scala:94-127 asserts the AskTimeoutException message contains target path, timeout ms, sender and message class (AskSupport.scala:722-735 builds "Ask timed out on [target] after [ms]. Message of type [..] was sent by [sender]"). Go sentinel `ErrAskTimeout` is returned as the value itself (errors.go:55-58, ask.go:95-99); wrapping breaks `==`. Pekko also includes the sender, which Go does not have (ADR 0032 D3 absent). Correction: cite AskSpec:94-127 as the V3 oracle and state in AC that the sender is omitted/absent; reconciliation's "accessor, not wrapping" stays correct.

**F-15 [low] Pekko specs present for stories whose reconciliation says "n/a".** #774 (StatusReplySpec:31-107, incl. `null` case :46 with no Go analogue, and the `getValue`-on-error throwing, StatusReply.scala:45-50), #859 (StatusReplySpec:82-107; the "invalid response shape" AC3 is `mapTo[StatusReply[Any]]` ClassCastException with no spec), #857 (PromiseRefSpec:37-80: first message completes, second goes to DeadLetter :54-64, negative timeout IllegalArgumentException :48, `PromiseRef.wrap`), #936 (AskSpec:128-190 incl. wildcard `*` and `..` replies that time out and dead-letter), #770 (ActorRefSpec:422-440 ordering: asks queued before PoisonPill are answered), #816 (see F-4), #1099 (spec uses a custom SuppressedMsg, not PoisonPill, DeadLetterSupressionSpec:38-62). Correction: add the line cites to the V3 evidence of each AC.

**F-16 [medium] #1099 and MSG-10 evidence mismatch.** DeadLetterSupressionSpec asserts the suppressed event goes to `system.deadLetters` as recipient (spec :58-60) and uses its own marker message; no spec sends PoisonPill to a dead actor and observes suppression (searched DeadLetterSupressionSpec, ActorRefSpec, DeathWatchSpec). Behavior is source-backed (ActorRef.scala:716-724). So AC-R1-1 is "ported intent" and AC-R2-1 has no spec found; both say alternative evidence. Whether an applicable oracle exists for either (the behavior is source-backed, `ActorRef.scala:716-724`) is pending determination under ADR 0033 D13.

**F-17 [medium] ADR 0015 D8 mis-cited.** #854 reconciliation cites "ADR 0015 D8 (Ask correlation on Forward)". ADR 0015 D8 is "Reply endpoint". The Forward rule is ADR 0010 D8 ("Ask correlation on Forward: once, for the request being processed", ADR 0010 decisions table row D8) and ADR 0008:116. Also ADR 0010 D8's whole Forward/ask correlation design rests on a runtime envelope (ADR 0008, "proposed") that ADR 0032 only "relies on". Correction: fix cites in #854 and add ADR 0008 (proposed, provenance) to #854/#814 ADR impact list; ADR 0008 line 3 says nothing is accepted.

**F-18 [medium] ASK-11 (#775) race is real by code order (read, not executed).** `close(c.done)` runs at stop.go:119 and the registry retirement runs after it (stop.go:120 -> hierarchy.go:233-242 `complete` -> `retireLocked`, under `s.mu`). The test takes `idsBefore` right after `doneOf` (ask_terminated_internal_test.go:40,77), so it can read the registry before the stopped target's entry is retired and compare against a smaller `idsAfter` later (line 81). No Pekko analogue (see story). Ready and startable now; the fix must wait on the retirement (e.g. `lookupID` returning nil), not on a sleep.

**F-19 [low] Plan vs issue disagreement on startability.** Delivery plan (develop, rev 4) lines 182/565 list #774 ASK-07 as STARTABLE; the issue is `blocked` by ADR 0015 Q4 (plan C-28 "R as (R, error)", design dossier C-28 same; ADR 0015 D3 says StatusReply deferred, unscheduled [Owner]). The plan's C-28 row proposes a shape the Owner explicitly deferred. Correction: amend plan rows 182/565 and C-28 to "waits #1068/Owner revisit of Q4".

**F-20 [medium] #1090 (IDN-19) status:ready conflicts with its own R2 and its test claims.** The issue acknowledges the tension (R2 evidence pending approval). Verified in code: Child/Children already exist (hierarchy.go:418,422) with tests (hierarchy_test.go:25; hierarchy_internal_test.go:161,203). R2 has no Pekko spec (source only: ChildrenContainer.scala:69-74,134-135,197-198 per the story; not reopened by me). Also Pekko `children` iterates in name order because of the TreeMap (stated by the story; `ChildrenContainer.scala:79,120,174` not reopened). Correction: split label: R1/R3 ready, R2 blocked until the D13 classification of its evidence is determined; or change to `status:blocked` until then.

**F-21 [low] IDN-14 (#1008) label.** Still `status:ready`, no prio label, no reconciliation block; the plan says "ready label is stale", PR #1011 open. Correction: set to `status:blocked`/in-flight with the PR as prerequisite or merge PR #1011.

**F-22 [low] #1092 IDN-21 and #894 split.** The encode half (#894) AC1 "carries address/path/incarnation" is blocked by GOV-41 (#1186) with the token form undecided; #1092 consumes it. Both are at the end of a 4-deep chain (GOV-41 -> #894 -> #1092, plus #812/#893/#1022) for a P3 consumer (TYP-22 #957, GOV-24). Acceptable but note the resolve half also depends on #893 which is gated by GOV-09 alternative (a); under (b)/(c) #1092 is dead.

## 4. Owner decisions needed (alternatives, consequences; no recommendation)

| ID | Decision | Alternatives | Consequence | Recommendation |
|---|---|---|---|---|
| OD-1 | Merge ADR 0032 (PR #1014) and resolve D5 (sender of Terminated/WatchWith) | none / watched actor as sender / keep sender-less | determines #814, #854, DTH-03 | No recommendation. |
| OD-2 | Reply capability from sender `Handle` (F-2) | typed conversion function; `Ref[any]`-style untyped path; no reply feature | MSG-02/03 usefulness; conflicts with ADR 0015 Q3 if untyped | No recommendation. Pekko: `sender()` returns an `ActorRef` usable with `!` (ActorCell.scala:145; ActorRef.scala:158-164). Open question: how a receiver replies to the sender `Handle` (ADR 0032 D2 is on unmerged PR #1014). Missing: Owner decision; no Pekko counterpart to a Handle-to-Ref conversion. |
| OD-3 | AskFrom timing (GOV-18) | keep deferred (amend 0010 D4/scope) / schedule non-blocking AskFrom | gates ASK-01,02,03,04,08, TYP-07 | No recommendation. |
| OD-4 | StatusReply revisit (ADR 0015 Q4) | keep deferred / adopt value type / adopt (R,error) | ASK-07,08, TYP-08 | No recommendation. Pekko has `askWithStatus` and `StatusReply` (AskSupport.scala:100-114,349-358; StatusReply.scala), so the ADR 0015 Q4 deferral differs from Pekko (identified only; ADR not changed). |
| OD-5 | Status.Failure (ASK-06) | exclude (replace by StatusReply) / rescope as 0015 amendment | ADR 0015 Q3/Q4 stay or change | No recommendation. Pekko completes the ask future with the Failure when a `Status.Failure` reply arrives (AskSupport.scala:620-623; Actor.scala:316-322). Accepted ADR 0015 Q3/Q4 differ from Pekko here (identified only; ADR not changed). |
| OD-6 | Recipient-terminated error identity (ASK-09) | keep ErrRecipientTerminated (recorded divergence) / change to timeout-like | judge I4 already normalizes both; public contract change | No recommendation. Pekko fails with AskTimeoutException (AskSupport.scala:312-319,372-375); ADR 0015 D5 Q2 (accepted) returns `ErrRecipientTerminated`; no Owner decision under ADR 0033 D13 recorded in the audited files. |
| OD-7 | Nil message policy (MSG-04) | keep Tell infallible / dead-letter nil / panic / typed API reject | high-risk item 3 | No recommendation. Pekko rejects null at send time by throwing to the caller (ActorRef.scala:685,737; AskSupport.scala:619; AbstractDispatcher.scala:45-50). Open question: the Go equivalent. Missing: Owner decision; no Pekko spec (typed specs only: ActorContextSpec, AdapterSpec, typed ActorSystemSpec). |
| OD-8 | PoisonPill, Kill classification (GOV-07/08) and API shape (control message for `Ref[T]`) | include/adapt/exclude; Port-based vs new method | MSG-06,07,09,10, JDG-08, TIM-17 | No recommendation. Pekko: PoisonPill (Actor.scala:50-58; ActorCell.scala:576, stops after earlier messages, ActorRefSpec.scala:420-441) and Kill (Actor.scala:65-75; ActorCell.scala:575, throws ActorKilledException) exist. Classification and Go shape are Owner decisions (ADR 0033 D4/D13). |
| OD-9 | ActorSelection/Identify (GOV-09) | (a) Select+Identify, (b) exact lookup only, (c) exclude | 9+ stories | No recommendation. Pekko supports wildcard `*`/`?`, `..`, Identify and fan-out (ActorSelection.scala:219-227; ActorSelectionSpec.scala:361-425). Restriction to exact paths or `Handle` is not shown by Pekko. |
| OD-10 | ADR 0014 D2 name release (GOV-42) | keep early release / move to parent-visible completion | high-risk items 3,4; touches cell.go/hierarchy.go | No recommendation. Pekko: FaultHandling.scala:260-266 detaches the dispatcher, then notifies the parent (DeathWatchNotification), then the watchers; the child name is released when the parent processes it (Children.scala:240-252 `removeChildAndGetStateChange`). Accepted ADR 0014 D2 releases earlier than watchers (story #895 says it reverses); ADR not changed. |
| OD-11 | Path grammar addendum (GOV-40) and incarnation token (GOV-41), system-name validation | per ADR 0031 O2; token = ID-based string / path#uid | IDN-03,09,17,21 | No recommendation. |
| OD-12 | D13 classification of the evidence of IDN-19 R2, IDN-17, IDN-18, IDN-21, ASK-11, MSG-09 R3, MSG-10 R2 | determine whether an applicable Pekko oracle exists and, if it does, require a differential scenario; alternative-evidence approval comes into question only for rows where the oracle is shown not to apply | readiness of #1090 and others | No recommendation. |
| OD-13 | Spawn descriptor shape after GOV-10/11/12 (IDN-10) | functional options (current) / Props value | ADR 0011 export | No recommendation. |

## 5. ADRs needing creation, amendment or acceptance (not changed by me)

- ADR 0032: merge (PR #1014) and later D3 revisit by #896, D5 by #814.
- ADR 0010 (proposed): amend D2 (sender), D4/D7 (AskFrom), "Excluded for now" per ADR 0033 D4 (PoisonPill, Kill, selection, dispatcher/mailbox, routers); accept or supersede in parts; D4 "Decided" rows live inside a proposed ADR.
- ADR 0015 (proposed header with Owner Q1-Q6): accept D3,D4,D7,D8,D9 (stale rationale in D8 about `byID`, GOV-18); amend for async result, `Status.Failure`, StatusReply, error context accessor.
- ADR 0008 (proposed): provenance/envelope; required by Forward correlation (ADR 0010 D8).
- ADR 0031 addenda: parser grammar (GOV-40), incarnation token (GOV-41), compareTo as a Contract item, empty-protocol/system-name differences (F-12).
- ADR 0014 D2 amendment only if Owner reverses (GOV-42).
- ADR 0011 individual justifications (GOV-02 is an index): ActorPath accessor, tags read surface, control messages, deadLetters, PromiseRef constructor, resolve/ not-found error.
- ADR 0017: Kill/ActorKilledException classification (SUP-16/SUP-02).
- New: reference/selection typing ADR if GOV-09 accepts (a).

## 6. Blocked vs startable

Startable now (no accepted-contract gap; verified):
- #775 ASK-11 (test-only, no prerequisite; confirm ordering race first, F-18).
- #1090 IDN-19 R1 and R3 only (R2 waits for the D13 classification of its evidence, F-20).
- #1008 IDN-14: implementation exists as PR #1011 (do not start a second one); merge it.
- GOV items that feed my stories are ready: #1068, #1038, #1039, #1022, #1087, #1186, #1187, #1188, #1020.

Blocked, with exact prerequisites:
- #768 MSG-01: merge PR #1014 (it is the deliverable); give it labels/reconciliation.
- #814 MSG-02: PR #1014 merged; D5 AC set; F-2 AC; GOV-02 (conversion export).
- #854 MSG-03: #814.
- #769 MSG-04 -> #815 MSG-05: Owner decision OD-7; #769 needs PR #1014 only for sender-in-error text.
- #770 MSG-06: GOV-07 (#1038) + GOV-02 (#1020); then #1099 also needs #816 and EVT-05 (#871).
- #816 MSG-07: GOV-08 (#1039), SUP-16 (#1112), GOV-02; release after SUP-02 (#778) (F-4).
- #1031 MSG-09: #816 and #778.
- #896 MSG-08: #814, EVT-01 (#793), GOV-02; ignoreRef half to GOV-38 (#1087).
- #1099 MSG-10: #770, #816, #871.
- #773 ASK-01, #818 ASK-02, #857 ASK-04, #819 ASK-09, #820 ASK-10, #774 ASK-07, #859 ASK-08, #858 ASK-06: #1068 (GOV-18) Owner decision; chain #773 -> #818 -> #856 -> #1024; #857 also #1129 (TIM-11), #1020; #859 also #774; #936 needs #818 and #893 and #1022.
- #856 ASK-03: #818 (and #773 for the mapping context/panic AC).
- #1024 ASK-12: #1023, #857, #814.
- IDN: #812 after PR #1011 and GOV-40; #852 after #1008 (+#812 only for AC2 comparison); #853 after #1008 and GOV-02; #893 after GOV-09, #852, #1089, #812, #853; #1089 after #812; #1023 after GOV-09, #893, #814; #1053 after #1008 and Owner export choice; #933/#1091 after #893 and GOV-09; #934 after #893, #814, GOV-09; #894 after #853, #812, GOV-41; #1092 after #894, #812, #893; #1093 after #1023, #934 and #936 (not #1024, F-10); #895 after #853 and GOV-42; #767 after GOV-02, GOV-10..12; #813 after #767 and GOV-02, and F-8 decision.
- Cycle check: none found in the reconciled edges of my 44 issues. Wrong-direction/missing: F-4 (MSG-07 to SUP-02 order), F-10, F-17.

## 7. Pekko items with no spec, limited search, items not read

No spec found (searched actor-tests / actor-typed-tests by name and by symbol):
- Anonymous name generation (randomName) and `Helpers.base64`: no spec.
- ActorPath `compareTo`: ActorPathSpec has none (EventBusSpec uses it indirectly, per story; not opened).
- Nil message `InvalidMessageException`: only typed specs reference it (ActorContextSpec, AdapterSpec, ActorSystemSpec, cluster-typed); none for Classic `!`.
- Default-decider Kill => Stop: source FaultHandling.scala:218-222; indirectly observed by DeathWatchSpec.scala:216-246 (defaultStrategy.decider, `failed ! Kill`). No spec asserts the Stop directly (searched actor-tests for `! Kill`: ActorRefSpec, ActorLifeCycleSpec, DeathWatchSpec, SupervisorTreeSpec, SupervisorMiscSpec, SupervisorHierarchySpec, ActorSystemSpec, LoggingReceiveSpec).
- `Children`/`Terminating` container semantics (R2 of #1090): source only per the story; I did not reopen ChildrenContainer.scala.
- PoisonPill-to-dead-actor suppression: spec uses a custom marker.
- resolveOne timeout branch: no spec.
- Classic AskSpec isTerminated recipient branch: no spec (typed AskSpec:71-82 only; I did not open the typed spec).

Not read / limited:
- Address.scala:185-232 (`AddressFromURIString`, `ActorPathExtractor`) body.
- Deployer.scala beyond lines 75,119; Props.scala beyond 180-217; PropsCreationSpec bodies; DeployerSpec.
- ActorRef.scala:175-190 (equals/hashCode) and ActorPath.scala uid lines 264-270, 369-378.
- GracefulStopSupport.scala:59 (`gracefulStop`) and any GracefulStop spec: none of the 44 issues in this group covers it, and I did not check which epic does (inventory index search for "graceful" returned no ACT2 story title). Possible uncovered capability in the area assigned (Pekko `pattern.gracefulStop`; typed `GracefulStopSpec` exists at actor-typed-tests/.../scaladsl/GracefulStopSpec.scala). Not verified further.
- ActorRefProvider.scala:145-151, 600-630 (resolveActorRef, temp registry) by citation only.
- ADR 0011, 0012, 0014 (full) not read in full: only D2/deviation 8 (0014) and the headers (0011, 0012); ADR 0012 is not exercised by my stories except as background for PoisonPill before first activation (not verified).
- `judge/pekko-harness/Judge.scala` not opened; judge rows read from docs/judge.md grep only.
- PR #1011 diff not read (state only).

## Key Learnings
1. ADR 0032 (PR #1014) is open and unmerged yet 8+ ACT2 stories cite its D3/D4/D6 as accepted, while ADR 0010 D2, ADR 0017:691 and ADR 0030 P-1 in develop still say there is no sender.
2. Pekko's RelativeActorPath.split keeps empty segments; dropping happens only in ActorSelection.apply and ActorPath./(Iterable), so story #1089 R1 misstates the oracle.
3. resolveOne is ask-through-selection plus Identify (ASK-05 plus IDN-08); #1093 wrongly lists ASK-12 (pipeToSelection) instead of #936.
4. Classic Kill specs mostly use custom deciders (default-decider Kill is observed only indirectly, DeathWatchSpec.scala:216-246), and Go's defaultDecider restarts unlisted causes, so delivering Kill before SUP-02 would make the default behavior opposite to Pekko.
5. The ASK-11 test race is real by code order: stop.go:119 closes done before stop.go:120/hierarchy.go:240 retires the registry entry.
