# Oracle: the EventStream (Pekko Classic v1.7.1)

- Date: 2026-11-06. Evidence only; this document recommends nothing and changes no ADR.
- Pekko `pekko-actor` 1.7.1 (tag `v1.7.1`, `upstream/pekko` at `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), Scala 2.13.18, sbt 1.10.7, JDK Temurin 11.0.32+9, default fork-join dispatcher, Classic actors only.
- Purpose: the evidence for ADR 0038 (subscription semantics of the goPekko event stream, issue #793) and for the acceptance criteria of #836, #870, #905 and #1056, and the condition (d) of ADR 0036 Amendment 2026-11-06 (e) (what `MessageBuffer.drop` publishes); EV11 adds the evidence for D-3 and criterion P9 of the scope of #870 (a `DeadLetter` that cannot reach a full `DeadLetter` subscriber); EV13 adds the evidence for criteria U1b and U3b of the scope of #905 (`unsubscribe` of a never-registered subscriber from a channel that is not a node, and from an unrelated channel); EV12 adds the evidence for D-4 of the scope of #1056 (`subscribe`, `unsubscribe` and `publish` on the `EventStream` of a System that is terminating and of one that is terminated).
- Method: read the code (section 1), run each scenario in its own JVM, three runs (section 2). The harness is `docs/parity/harness/` (README: commands, versions) and the logs are in [`eventstream-spec-2026-11-06.logs/`](eventstream-spec-2026-11-06.logs/README.md), trimmed to the cited lines with their original `[nnn]`. Citations `[nnn]` are lines of run 1; runs 2 and 3 are compared in each row.
- Rule "claims against evidence": a statement in the Observed column names the log line; anything not seen in a log is in an Inferred column or row.

## 1. What the source says (code reading, Pekko 1.7.1)

`E` = `actor/src/main/scala/org/apache/pekko/event/EventStream.scala`, `B` = `.../event/EventBus.scala`, `U` = `.../event/EventStreamUnsubscriber.scala`, `R` = `.../actor/ActorRef.scala`, `T` = `actor-tests/src/test/scala/org/apache/pekko/event/EventStreamSpec.scala`. Directories as in `upstream/pekko`.

| # | Claim | Location |
|---|---|---|
| S1 | Subscribers are `ActorRef`, channels are `Class[_]`, events are `Any`; the event's channel is `event.getClass`; `A` is a subchannel of `B` when `B.isAssignableFrom(A)`, so subscribing to a class, a superclass or an interface receives subtypes | `E:34-49`, `E:25-29` |
| S2 | Delivery is `subscriber ! event` (an enqueue into the mailbox, from the publishing thread). Only when the stream has no system (`sys == null`) and the subscriber is terminated is it unsubscribed instead | `E:51-54` |
| S3 | `subscribe` and both `unsubscribe` throw `IllegalArgumentException("subscriber is null")` for a null subscriber, before anything else; `subscribe` first registers the subscriber with the unsubscriber and then calls the bus | `E:56-62`, `E:64-66`, `E:74-76`; `T:102-113` |
| S4 | `subscribe` returns `diff.nonEmpty` (false when the pair was already registered, "or otherwise"); `unsubscribe(s, c)` returns `diff.nonEmpty`; `unsubscribe(s)` returns `Unit`. State changes run under `subscriptions.synchronized` | `B:159-175`, `B:38-62` |
| S5 | `publish` looks up `cache(c)`, a `Set[Subscriber]`, and calls `publish(event, _)` once per element. A subscriber registered for several matching classes is in the set once. The index keeps `values` as a set and adds a value only `if (!(values contains value))` | `B:177-190`, `util/SubclassifiedIndex.scala:47-53` |
| S6 | `publish` reads a published `cache` snapshot (a `@volatile` immutable map) without taking the lock when the class is cached; `unsubscribe` replaces the cache under the lock. So a publication that read the cache before the replacement can still address the subscriber | `B:146-147`, `B:165-175`, `B:177-190` |
| S7 | Termination: a system actor `eventStreamUnsubscriber-N` watches every subscriber (`Register` -> `context.watch`) and on `Terminated` calls `eventStream.unsubscribe(actor)`. The removal is asynchronous; the class comment says that reordering of subscribe/unsubscribe is accepted | `U:35-65`, `U:28-33`, `E:86-88`, `E:131-141`, `E:150-161`; `T:298-325`, `T:327-356` |
| S8 | A bounded mailbox with `pushTimeOut.length >= 0` that fails `queue.offer` sends `DeadLetter(handle.message, handle.sender, receiver)` to `deadLetters`; `Envelope` replaces a missing sender by `system.deadLetters` | `dispatch/Mailbox.scala:557-571`, `dispatch/AbstractDispatcher.scala:43-52` |
| S9 | `DeadLetter(message, sender, recipient)` requires non-null sender and recipient. `DeadLetterActorRef.!`: a `DeadLetter` is published as is; any other message is wrapped as `DeadLetter(message, <sender, or deadLetters if none>, this)` and published. `Dropped(message, reason, sender, recipient)` is an `AllDeadLetters`, "published on the eventStream wrapped in `DeadLetter`" | `R:592-595`, `R:733-743`, `R:616-622` |
| S10 | `MessageBufferMap.drop(id, reason, deadLetters)` sends, for each entry, `deadLetters ! Dropped(msg, reason, ref, ActorRef.noSender)`, then removes the id and returns the entry count. It reads `bufferMap.get(id)` of a `java.util.HashMap` and calls `.nonEmpty` on it | `util/MessageBuffer.scala:265-272`, `:184-187` |
| S11 | Overflow of the bounded mailbox of a subscriber of `DeadLetter` re-enters the stream (code reading, the prediction EV11 tests): the overflow path builds `DeadLetter(handle.message, handle.sender, receiver)` whose `message` is the `DeadLetter` that was being delivered, and tells `deadLetters`; `DeadLetterActorRef.!` publishes a `DeadLetter` as is (`specialHandle(d.message, d.sender)` is `false` for a `DeadLetter` message, `R:693-720`, `R:745-751`); `publish` does `subscriber ! event` to every subscriber again, the full one included. Nothing in the path breaks the cycle | `dispatch/Mailbox.scala:560-568`, `R:739-743`, `E:51-54` |

Reading of S5: the answer to #870 "once per matching subscriber" is in the code; EV4 runs it.

## 2. Scenarios and results

Common method: every "nothing arrived" is shown by a fence (a direct message from the publishing thread, queued behind its earlier deliveries), never by a sleep. Event classes: `Base`, `Sub extends Base with Iface`, `Unrelated`; EV1t adds `trait I`, `trait J extends I`, `class C extends J`; EV4f reuses `Base`. Streams are `new EventStream(sys, false)` (as `T:155`), except EV7 to EV9, which use `sys.eventStream` because they need the dead-letter route.

### EV1 Return values (`T:157`, `T:166`, `T:193-196`, `T:311-312`)

| Step | Observed (run 1) | Runs 2, 3 |
|---|---|---|
| `subscribe(a, Base)` twice | `true` [001], `false` [002] | same |
| one publish of `Base(p1)` after the two subscriptions | one `a recv Base(p1)` [003]; no second one | same |
| `unsubscribe(b, Base)`, `b` never subscribed; `unsubscribe(a, Unrelated)`, never subscribed to it | `false` [006], `false` [007] | same |
| `unsubscribe(a, Base)` twice | `true` [008], `false` [009] | same |
| `unsubscribe(a)` after `a` subscribed to `Base`, `Iface`, `Unrelated` | returns `()` [012]; no `recv` of `Sub(p3)` or `Unrelated(p4)` in the log | same |
| `a` subscribed only to `Base`; `unsubscribe(a, Sub)` (a subclass) | `true` [015]; no `recv` of `Sub(p5)` in the log, so the subclass channel stopped delivering; then `unsubscribe(a, Base)` still returns `true` [018] | same |

Inferred: a second subscription is a no-op of the index (S5), not a counter. Informs #836 (repeat subscription), #905 (one classifier, all classifiers), #793.

### EV1t EV1 with traits: `unsubscribe` of a sub-interface (new scenarios `EV1t`, `EV1tv2`, `EV1tv3`; `SI` = `util/SubclassifiedIndex.scala`)

`trait I`, `trait J extends I`, `class C extends J`; `a` on a fresh `new EventStream(sys, false)`. Each variant publishes `C(c1)` (control), calls one `unsubscribe`, publishes `C(c2)`, calls a second `unsubscribe`, publishes `C(c3)`; every publish is followed by a fence, and the `-- C(...)` markers show where each publish happened.

| Variant | Subscriptions of `a` | First `unsubscribe` | `C(c2)` afterwards | Second `unsubscribe`; `C(c3)` |
|---|---|---|---|---|
| `EV1t` (the Owner's case) | only `I` ([001], `true`) | `unsubscribe(a, J)` returns `true` [005] | no `a recv C(c2)` between the markers [007] and [008]; the control `a recv C(c1)` is [002] | `unsubscribe(a, I)` returns `true` [008]; no `recv` of `C(c3)` |
| `EV1tv2` | only `J` ([001], `true`) | `unsubscribe(a, I)` (the superinterface) returns `true` [005] | no `recv` of `C(c2)` | `unsubscribe(a, J)` returns `false` [008]; no `recv` of `C(c3)` |
| `EV1tv3` | `I` [001] and `J` [002], both `true` | `unsubscribe(a, J)` returns `true` [006] | no `recv` of `C(c2)` (between [008] and [009]), although `I` is still registered | `unsubscribe(a, I)` returns `true` [009]; no `recv` of `C(c3)` |

Runs 2 and 3: the same lines with the same numbers in all three variants. Citations are to the files `EV1t.1.txt`, `EV1tv2.1.txt`, `EV1tv3.1.txt`.

Inferred, from reading `SI` (not seen in a log): `removeValue(key, value)` (`SI:157-163`) walks to the most specific known super-key; if `key` itself is not a node (`J` in `EV1t`), `innerRemoveValue` creates it under that node with the parent's value set (`SI:174-176`), `integrate` re-parents the already known sub-keys such as `C` below it (`SI:222-228`), and `n.removeValue(value)` (`SI:61-67`) removes the value from the new node and from all its sub-keys. The result is a non-empty diff, hence `true` (`B:165-171`), and `C` has lost `a` in the cache although `a` was registered on `I`. In `EV1tv3` the node `J` exists and `removeValue` on it reaches `C` the same way; the index stores one set of values per key and not which classifier supplied them, so the registration on `I` does not keep `C`. The reading matches what `EV1` showed with classes.

#### Result for O-1 of ADR 0038

The quirk of EV1 [015]-[018] appears with traits exactly as with classes. In `EV1t`, `a` is registered only on `I`; `unsubscribe(a, J)` returns `true` [005] and `C(c2)` is no longer delivered, although `C` implements `J` and `I`, and `a` was never registered on `J`; the later `unsubscribe(a, I)` returns `true` too [008]. So the equivalent for interfaces is already observed and the O-1 option "reproduce it with the interface equivalent" has this evidence. Two control variants refine the picture: unsubscribing the superinterface `I` from a subscriber of `J` also returns `true` and stops delivery [`EV1tv2` 005], and unsubscribing `J` from a subscriber registered on both `I` and `J` stops delivery of `C` entirely, not only through `J` [`EV1tv3` 006]: the registration on `I` does not survive. The quirk is therefore not limited to classes.

### EV2 Null arguments (`T:102-113`)

| Call | Observed | Runs 2, 3 |
|---|---|---|
| `subscribe(null, Base)`, `unsubscribe(null, Base)`, `unsubscribe(null: ActorRef)` | `IllegalArgumentException: subscriber is null` [001], [002], [003] | same |
| on fresh streams: `unsubscribe(a, null)`, `subscribe(a, null)` | `false` [006]; `true` [007] | same |
| `publish(Base)` on the stream of the previous row | `NullPointerException` [008] | same |

Beyond the spec: a null channel is accepted by `subscribe` and then breaks `publish` of that stream; an earlier exploratory run also threw from `unsubscribe` after it. Informs #793 (what the contract rejects).

### EV3 Matching (`T:150-296`)

Subscribers: `exactBase` (`Base`), `exactSub` (`Sub`), `iface` (`Iface`), `unrelated` (`Unrelated`), `anyRef` (`AnyRef`), `charSeq` (`CharSequence`).

| Publish | Observed | Runs 2, 3 |
|---|---|---|
| `Sub(s1)` (subclass of `Base`, implements `Iface`) | received by `anyRef` [002], `iface` [003], `exactBase` [004], `exactSub` [005]; not by `unrelated`, `charSeq` | same four; the order of lines differs |
| `Base(b1)` | `exactBase` [014], `anyRef` [015]; not `exactSub`, `iface` | same, order may differ |
| the `String` `hello` | `anyRef` [024], `charSeq` [026] (a JDK interface) | same |
| `Nobody`, no subscription of that stream matches | no `recv` between [033] and the fences [034]-[040]; no `DeadLetter` reached the `DeadLetter` subscriber | same |
| `publish(null)` | `NullPointerException` [042] (from `null.getClass`, `E:49`; Inferred) | same |

Informs #836 (exact, superclass, interface), #793 (typing and filtering). The order in which the subscribers of one publication receive it varies between runs ([002]-[005] in run 1; run 2 and 3 list `exactSub` before `exactBase` or `iface`): no order among subscribers is promised (S5, a `Set`).

### EV4 Duplicate class matches (nearest spec tests `T:259-279`, subscribe/unsubscribe of classes and traits; no spec test covers duplicate matches)

`x` is subscribed to `Base`, `Iface` and `Sub`; `y` to `Sub` then `Base`; `z` to `AnyRef` then `Base`. One publish of `Sub(d1)`.

| Observed (run 1) | Runs 2, 3 |
|---|---|
| subscribe returns: `x` `true`, `true`, `true` [001]-[003]; `y` `true`, `true` [004], [005]; `z` `true` [006], then `false` [007] (Inferred: its `Base` registration was already covered through `AnyRef`) | same |
| each of `x`, `y`, `z` has exactly one `recv Sub(d1)`: [008], [009], [010] | same, order differs |
| `x` unsubscribes from `Base` only (`true` [015]); `Sub(d2)` still reaches `y` [016], `x` [017], `z` [018], once each | same |

Answer for #870 AC1 "one publication addresses each matching subscriber once under duplicate class matches": Pekko delivers one copy per subscriber ([008]-[010]). Also informs #836 (a registration covered by another is reported `false`, [007]) and #905 AC1 (unsubscribing one classifier keeps the others, [015]-[017]).

### EV4f Covered registration on a fresh stream (new scenarios `EV4f`, `EV4fv2`, `EV4fv3`; `SI` = `util/SubclassifiedIndex.scala`)

EV4 [007] returned `false` for `z subscribe Base` after `z subscribe AnyRef`, but `Base` was already a node there (`x` had registered on it first). EV4f isolates the question on a fresh `new EventStream(sys, false)` with no other subscriber on `Base`. One publish of `Base(d1)` follows the subscriptions, then a fence on `z`; the `-- one publish` marker follows the fence.

| Variant | Setup | `subscribe` returns | Deliveries of `Base(d1)` to `z` |
|---|---|---|---|
| `EV4f` | fresh stream; `z` on `AnyRef`, then `z` on `Base` | `true` [001], `true` [002] | one: `z recv Base(d1)` [003], nothing else before the marker [005] |
| `EV4fv2` | `x` on `Base` first, then `z` on `AnyRef`, then `z` on `Base` | `x` `true` [001]; `z` `true` [002], `false` [003] | one: `z recv Base(d1)` [004] (and one for `x` [005]) |
| `EV4fv3` | fresh stream; `z` on `Base`, then `z` on `AnyRef` | `true` [001], `true` [002] | one: `z recv Base(d1)` [003], nothing else before the marker [005] |

Runs 2 and 3: identical lines with identical numbers in all three variants (`cmp`). Citations are to `EV4f.1.txt`, `EV4fv2.1.txt`, `EV4fv3.1.txt`.

Inferred, from reading `SI` (not seen in a log): in `innerAddValue` (`SI:137-150`) a key is added below every node whose key it subclasses; only when no node qualifies (`!found`, `SI:145`) is a new `Nonroot` created and the result `integrate(n) ++ n.innerAddValue(key, value) :+ (key -> v)` (`SI:146-148`) gets the entry `(key -> v)`, which makes the diff non-empty, hence `true` (`B:159-163`, `subscribe` returns `diff.nonEmpty`). In `EV4f` the `AnyRef` node holds `{z}` and has no sub-keys, so for `Base` the recursion finds no sub-node of `AnyRef` that `Base` subclasses (`!found` inside `AnyRef`): the new node `Base` is created with the parent's values `{z}` and reported as a change although `z` already received `Base` through `AnyRef`. In `EV4fv2` `Base` is a node, `integrate` makes it a child of the new `AnyRef` node (`SI:222-228`) and adds `z` to it (`SI:47-52`); the later `subscribe(z, Base)` reaches the node, finds `z` already in its values and returns an empty diff, hence `false`. In `EV4fv3`, `AnyRef` is not a subclass of the node `Base` (`!found` at the root), so the new `AnyRef` node is created and `(AnyRef -> v)` is reported: `true`. One delivery in all variants: the cache of `Base` holds the set `{z}`, and `publish` iterates a set (S5).

#### Result for X7b of the scope of #836

On a fresh stream, Pekko 1.7.1 returned `true` for `subscribe(z, Base)` after `subscribe(z, AnyRef)` [`EV4f` 002], and `z` received exactly one copy of `Base(d1)` [`EV4f` 003]; the `false` of EV4 [007] appears only when `Base` is already a node because another subscriber registered on it first (`EV4fv2` 003). The Boolean of a covered registration therefore depends on the shape of the index and not only on the pair `(z, Base)`: it is `true` when no node for `Base` exists (also `EV4fv3` 002, where `AnyRef` is added after `Base`) and `false` when the node exists and `z` is already in its values. Delivery is one copy per subscriber in all three variants.

### EV5 Unsubscribe linearization (nearest spec test `T:86-100`, manage subscriptions; no spec test covers in-flight delivery under a blocked subscriber; #905 AC3, #793 AC2)

`s` subscribed to `Base`; `e1` is delivered and `s` blocks inside `receive`; the publisher publishes `e2`, `e3`, `e4`, then unsubscribes, then publishes `e5`, then the gate opens. EV5a unsubscribes with `unsubscribe(s, Base)`, EV5b with `unsubscribe(s)`.

| Variant | Observed (run 1) | Runs 2, 3 |
|---|---|---|
| EV5a | publisher returned from `e2`-`e4` while `s` is blocked [004]; `unsubscribe` `true` [005]; `e5` published after it returned [006]; `s` then receives `e2` [008], `e3` [009], `e4` [010] and nothing after them (no `recv Base(e5)`) | same |
| EV5b | the same lines; `unsubscribe(s)` returns `()` [005]; `e2`-`e4` [008]-[010] | same |

Result: the deliveries accepted into the mailbox before the unsubscribe complete (`e2`, `e3`, `e4`; `e1` was already in `receive`); a publication started after `unsubscribe` returned is not delivered (`e5`). Inferred: this is the direct consequence of S2 (delivery is the enqueue; the unsubscribe changes who is addressed, never what is already in a mailbox).

### EV6 A blocked subscriber does not block the publisher (#870 AC3)

`s` blocked inside `receive`; a publisher thread publishes 1000 events. Observed: the publisher thread finished with the subscriber still blocked (`finished=true`) [003]; after the gate opened `s` received all 1001 events in publish order [006]. Runs 2, 3: same. The wait is a 10 s timeout, used only to fail; the result does not depend on timing. Inferred: delivery runs in the publisher's thread only as far as the mailbox enqueue (S2).

### EV7 Termination (`T:298-356`; #1056, #905 AC2)

| Step | Observed (run 1) | Runs 2, 3 |
|---|---|---|
| `subscribe(victim, Base)` twice, `victim` live | `true` [001], `false` [002] (`T:311-312`) | same |
| `victim` stopped and its `Terminated` seen by a watcher; then 40 publishes | `dead letters for victim in total = 0` [045]; live subscriber `obs` received `40 of 40` [047] | same |
| `racer` subscribed, stopped with `PoisonPill`, 300 publishes at once | 1 dead letter, for `r1` [049], [051]; `in total = 1` [351] | same (1 in each) |
| `dead` already terminated when subscribed | `subscribe` returns `true` [353]; first publish gives a dead letter for `w1` [354], [356]; `in total = 1` [396]; `subscribe(dead, Base)` again returns `true` [397] | run 2: `in total = 0`, run 3: `1` |

Result: a terminated subscriber stays registered until the unsubscriber processes its `Terminated`; in that window a publish reaches `deadLetters` as `DeadLetter(event, deadLetters, subscriber)` [049], [354]. After the removal there is no further dead letter (`victim`: 0 of 40 [045]; `racer`: 1 of 300 [351]). The size of the window is timing: 0 or 1 dead letters in the kept runs (an earlier exploratory run saw 2 for `racer`; EV7 is the only scenario whose counts vary). Inferred (no log shows a live subscriber inside a window; [047] is the run without one): other subscribers are unaffected because `publish` iterates the set and each `!` is independent (S5). Informs #1056 AC-R1-1 (removal), AC-R2-1 (a terminated actor that subscribes is registered, then removed: `subscribe` returns `true`, so "no registration remains" is true only after the removal), AC-R3-1.

### EV8 Mailbox admission (bounded, capacity 1, push timeout 0; #870 AC2)

`s` blocked inside `receive` on `e1`; `e2` published (fills the mailbox); `e3` and `e4` published; subscribers of `DeadLetter` (`dl`) and `Dropped` (`dr`).

| Observed (run 1) | Runs 2, 3 |
|---|---|
| `e3` and `e4` each reach `dl` as `DeadLetter(message=Base(e3\|e4), sender=Actor[pekko://evs/deadLetters], recipient=<s>)` [006], [008] (Inferred, from S8: the `DeadLetter` is produced inside the `publish` call; no log line shows the thread) | same |
| `dr` received nothing (no `dr recv` line) | same |
| `s` finally received only `e1`, `e2` [014]; each `publish` call returned and the next step ran ([003]-[008] follow in order) | same |

Result: bounded overflow of an event subscriber is a `DeadLetter` with sender `deadLetters` (S8), not a `Dropped`, and the publisher is not told. A stopped but still registered subscriber is the EV7 case (`DeadLetter` too).

### EV9 `MessageBuffer.drop` (ADR 0036 Amendment 2026-11-06 (e))

A `MessageBufferMap` with two entries under `id1`: `m-with-sender` (sender `snd`) and `m-no-sender` (no sender). `drop("id1", "reason-x", system.deadLetters)`; subscribers of `Dropped` (`dr`), `DeadLetter` (`dl`) and `AllDeadLetters` (`all`).

| Observed (run 1) | Runs 2, 3 |
|---|---|
| `drop` returns `2` [002] | same |
| `dl` receives `DeadLetter(message=Dropped(m-with-sender, reason-x, snd, null), sender=deadLetters, recipient=deadLetters)` [003] and `DeadLetter(message=Dropped(m-no-sender, reason-x, null, null), sender=deadLetters, recipient=deadLetters)` [004] | same, `dl` and `all` interleave differently |
| `all` receives the same two `DeadLetter` [006], [007] | same |
| `dr` receives nothing (no `dr recv` line) | same |
| `drop("nope", ...)` on an id with no entries | `NullPointerException` [011] | same |

The expected-by-reading answer is confirmed: `Dropped(msg, reason, entrySender, noSender)`, with `noSender` printed as `null` in both the sender (second entry) and recipient fields, wrapped in `DeadLetter(Dropped, deadLetters, deadLetters)`, and no event of class `Dropped` on the stream. Surprise: `drop` of an unknown id throws (S10: `HashMap.get` returns `null`).

### EV10 Unsubscribe while a publisher runs (`B:165-190`)

A publisher thread publishes ids `0..19999`; when it passed 1000 the other thread calls `unsubscribe(s)`; after it returns the harness reads the id being published. Observed: the delivered ids are `0..m-1` without gaps and `m-1 <= ` the id seen after the return in every run: `delivered=3031 maxId=3030` [004]; runs 2 and 3: 1947 and 3218, same two flags `true`. So no publication that started after `unsubscribe` returned was delivered, and nothing delivered was missing before it. The count differs per run (the race is real); this is the only evidence about S6, and it shows no violation, not that none can occur. Inferred: the window of S6 (a publish that read the cache before the replacement) is shorter than what 20000 publishes resolve here.

### EV11 A `DeadLetter` that cannot reach a full `DeadLetter` subscriber (new scenarios `EV11`, `EV11v2`, `EV11v3`, `EV11v4`; D-3 and P9 of the scope of #870)

Setup, on `sys.eventStream`: `s` has the bounded mailbox of EV8 (capacity 1, push timeout 0), is subscribed to `DeadLetter` and blocks inside `receive` on its first message; `dl2` has an unbounded mailbox and is subscribed to `DeadLetter`; `victim` is stopped and its `Terminated` is seen. The harness sends `m1`..`mN` from the main thread to `victim` (each is a dead letter, EV7), logs what each send did, fences `dl2`, opens the gate, and summarizes what `s` and `dl2` hold (`summary` lines: number received, number of `DeadLetter` wrappers around the innermost message minus one = `nested`). Code-reading prediction (S11): the stream is re-entered with `DeadLetter(DeadLetter(...))` and Pekko recurses.

| Variant | Setup | `send` results | `s` received | `dl2` received | Exit codes (runs 1, 2, 3) |
|---|---|---|---|---|---|
| `EV11` | N = 3; `s` on `DeadLetter`, subscribed before `dl2` | `m1`, `m2` returned [004], [009]; `m3` threw `java.lang.StackOverflowError` [010] | `m1` [005], then `m2` [015]; `received=2`, no nested [017] | `m1` [006]; `received=2` (`m1`, `m2`), no nested, `m3` never arrived [013] | 3, 0, 0 |
| `EV11v2` | N = 1 (control: no overflow) | `m1` returned [004] | `m1` [005]; `received=1` [014] | `m1` [006]; `received=1` [011] | 0, 0, 0 |
| `EV11v3` | N = 3; `s` on `AllDeadLetters` | as `EV11` [004], [009], [010] | as `EV11` [005], [015], [017] | as `EV11` [013] | 0, 0, 0 |
| `EV11v4` | N = 3; `dl2` subscribed before `s` | as `EV11` [004], [009], [010] | `received=2`, no nested [017] | `received=1004`, of which 1001 nested, `maxNested=1001`, innermost `m1`, `m2`, `m3` [013] | 0, 0, 3 |

Observed beyond the table. Runs 2 and 3: `EV11`, `EV11v2`, `EV11v3` give the same lines with the same numbers (only the order of [005] and [006] may swap, both are logged by actor threads); `EV11v4` run 2 has `received=980`, `nested>=1:977`, `maxNested=977` [013] (the depth is not fixed). `EV11v4` run 3 shows `HARNESS FAILURE ... fence fdl2 timed out` [012]: `dl2` did not handle the fence within 10 s after the `StackOverflowError` [010]. `EV11` run 1 and `EV11v4` run 3 ended with exit code 3, and the `WATCHDOG` line closes both trimmed files (`EV11.1.txt`, `EV11v4.3.txt`); the scenario lines of `EV11` run 1 were complete before it. An earlier exploratory run of `EV11v3` (not kept) ended the same way. `s` never received a nested `DeadLetter` in any run (`nested>=1:0`).

Inferred (not seen in a published log):
- From the harness stderr, which is kept outside the repository and is not published (seen by the author, not checkable here): in the two exit 3 runs `Await.result(sys.whenTerminated, 10.seconds)` threw `TimeoutException`, so the 60 s watchdog ended the JVM; the stderr of the other ten runs was empty.
- The nesting is the overflow path of S11 repeated: `s` is full when the `DeadLetter(m3)` arrives; `enqueue` builds `DeadLetter(DeadLetter(m3...), ...)` and tells `deadLetters`, which publishes it as is, `subscriber ! event` reaches `s` again, it is still full, and so on. The only thing that ends the cycle is the `StackOverflowError`, which reaches the caller of the first `!` [010]. The depth 1001 and 977 [013] in `EV11v4` is what the stack allowed, so it is a JVM limit and not a Pekko one.
- Why `dl2` sees the nesting only in `EV11v4`: the publisher iterates the subscribers one by one on the same thread (S5); with a small set the order is probably the order of subscription (not observed; EV3 says no order is promised for larger sets). In `EV11` `s` is first, the recursion starts before `dl2` gets `m3` and the error unwinds everything, so `dl2` lost `m3` [013]; in `EV11v4` `dl2` is first and receives `m3` and every nested wrapper before each re-entry reaches `s`.
- `EV11v3`: `DeadLetter` is an `AllDeadLetters` (`R:592`), so a subscriber of `AllDeadLetters` gets the same path; same result as `EV11`.
- The hang at termination: a `StackOverflowError` thrown through the dispatcher or mailbox code may leave a mailbox without the scheduled run it needs (`dl2` did not run its fence in `EV11v4` run 3), or block `terminate`. The cause was not investigated.

#### Result for D-3 of the scope of #870

Pekko recursed and nested: a `DeadLetter` that cannot be delivered to a full `DeadLetter` subscriber is republished wrapped in another `DeadLetter`, to the same full subscriber, again and again; the other subscriber sees up to 1001 nested wrappers [`EV11v4` 013, run 1], and the cycle ends only with a `StackOverflowError` thrown to whoever sent the message that started it [`EV11` 010]. The code-reading prediction (S11) is confirmed in every part. Two refinements: which subscriber sees the nesting depends on the order of subscription (in `EV11` `dl2` saw none and lost `m3`, in `EV11v4` it saw 1001), and, Inferred from [012] of `EV11v4` run 3 and the exit 3 runs, the failing run may also leave the stream unusable. The full subscriber itself never received a nested `DeadLetter` and ended with `m1` and `m2` only.

Consequence for the scope of #870 (statement of the TL): Pekko republishes `DeadLetter(DeadLetter(...))` until a `StackOverflowError` reaches the sender (EV11 [010], EV11v4 [013]), so the scope's conservative rule (no republication of a failed-delivery `DeadLetter`) is a deliberate deviation from Pekko's observable behavior (Go cannot recover a stack overflow) that slice 2 documents under D14.3.

### EV12 `subscribe` and `unsubscribe` on the stream of a System that is terminating and of one that is terminated (new scenarios `EV12`, `EV12v2`; D-4 of the scope of #1056; `E` = `event/EventStream.scala`, `EU` = `event/EventStreamUnsubscriber.scala`, `AS` = `actor/ActorSystem.scala`)

The stream is `sys.eventStream` (it has the system's unsubscriber, unlike the `new EventStream(sys, false)` of EV1 to EV6). All actors of the scenario are system actors (`systemActorOf`, children of the system guardian, chosen so that they are not children of the user guardian). `terminating` is made reproducible in two ways, and the harness logs the moment the hold is entered:

- `EV12`: a `CoordinatedShutdown` task of phase `before-service-unbind` (timeout raised to 30 s in the harness config) waits on a latch; `sys.terminate()` is called from the main thread and returns at once. Inferred, not logged: the user guardian is still alive (it is stopped at the end of the shutdown, `AS:1112-1117`). Line [006] says the hold is entered.
- `EV12v2`: `terminate()` runs the shutdown to the end, and a user child `blocker` blocks inside `postStop` on a latch. Inferred, not logged: the user guardian is therefore stopping. Observed: `whenTerminated` is not complete [007]. Line [006] says the child is inside `postStop`.

In both, `state terminating` logs `whenTerminated.isCompleted=false` [007]. The latch is then opened, `whenTerminated` completes [073], and the same calls run in `state terminated` [074]. `rec` is a `MinimalActorRef` subclass of the harness whose `!` records the message synchronously (a dead actor cannot show a delivery); `late` is an actor.

| Call | Terminating (`EV12` and `EV12v2`, run 1) | Terminated (run 1) |
|---|---|---|
| unsubscriber (`/system/eventStreamUnsubscriber-1`) | exists: `Identify` through the selection `/system/eventStreamUnsubscriber-*` answers `Some(...)` [008], [009]; `terminated=false` [007] | `terminated=true` [074] |
| `subscribe(late, Base)`, `subscribe(rec, Base)` | `true` [010], `true` [011]; no exception | `true` [075], `true` [076]; no exception |
| `publish(Base(p1))` | returns `()` [015]; `rec` got it [014], [018]; `late` got it [016] | returns `()` [078]; `rec` got it [077], [079]; `late` is a dead actor, not observable |
| `unsubscribe(late, Base)`, `unsubscribe(rec, Base)`, `unsubscribe(late)` | `true` [019], `true` [020], `()` [021] | `true` [080], `true` [081], `()` [082] |
| `publish(Base(p2))` after the unsubscribes | returns `()` [022]; `rec` got 0 [026] | returns `()` [083]; `rec` got 0 [084] |
| `victimA` (subscribed before `terminate()`) and `victimB` (subscribed while terminating) stopped; 20 publishes | `subscribe(victimB)` `true` [027]; both `Terminated` seen by the watcher [028], [029]; dead letters for `victimA`=0, `victimB`=0 over 20 publishes [071] | not run (no actor is alive) |

Observed beyond the table. Runs 2 and 3 of both variants: the same lines (as a set) with the same results; the order of lines logged by different actor threads varies (`rec recv`/`late recv`, the two `watcher saw`), so numbers shift by one or two in places; the line `watcher saw Terminated(eventStreamUnsubscriber-1)` (run 1: [072]) is absent in run 2 of both variants; why (Inferred: the watcher is itself a system actor and may stop before reporting) was not studied. Exit code 0 in all 6 runs.

Inferred (not seen in a log):
- `subscribe` returns the Boolean of the index and sends `Register` to the unsubscriber ref; it does not look at the state of the system (`E:56-62`, `E:130-141`). In the terminated state that send goes to a stopped actor; the logs show only that nothing was thrown. The same for `unsubscribe` and `UnregisterIfNoMoreSubscribedChannels` (`E:64-72`, `E:74-80`, `E:150-161`).
- The unsubscriber is created with `systemActorOf` (`EU:86-95`) and the terminating states ran, as far as the logs show, before `whenTerminated` completed [007] and the unsubscriber answered [008] in both; the order in which it stops relative to other system actors was not studied.
- 0 dead letters for the two stopped subscribers [071] is consistent with the unsubscriber having processed their `Terminated` (the EV7 mechanism, `EU:35-63`), but no line shows the unsubscribe; as in EV7 a window of 0 or 1 dead letter is possible.
- `terminate()` starts `CoordinatedShutdown` and `finalTerminate` stops the user guardian at its end (`AS:1089-1104`, `AS:1112-1117`); that is why the hold of `EV12` leaves the user guardian alive and the hold of `EV12v2` happens while it stops. No log shows the guardian state; [006] shows only that the hold was entered.

#### Result for D-4 of the scope of #1056

Pekko 1.7.1 does not refuse: on a System that is terminating, in both ways of being so, `subscribe`, `unsubscribe` and `publish` do not refuse: `subscribe` returns `true` and throws nothing [010], [011], the subscriber receives the next publication [014], [016], and a subscriber that terminates in that state, whether registered before or during it, causes no dead letter over 20 publications [071] (that they were removed is Inferred: no line shows the unsubscribe). On a System that is terminated, `subscribe` and `unsubscribe` still return the same Booleans and throw nothing [075], [076], [080]-[082], a ref still receives a publication while subscribed [077] and none after the unsubscribe [084]; the system's unsubscriber is terminated [074]. What was not observed is the removal of a subscriber in the terminated state (no actor is alive to terminate).

### EV13 `unsubscribe` of a never-registered subscriber, from a channel that is not a node and from an unrelated channel (new scenarios `EV13`, `EV13v2`, `EV13v3`, `EV13v4`; U1b and U3b of the scope of #905; `SI` = `util/SubclassifiedIndex.scala`, `B` = `event/EventBus.scala`)

Streams are `new EventStream(sys, false)`; traits as in EV1t (`trait I`, `trait J extends I`, `class C extends J`), plus `trait K` unrelated to `I` and `class C2 extends I with K`. Each variant except `EV13v4` (which has no control publish) publishes a control event, calls one `unsubscribe`, publishes a second event; every publish is followed by a fence.

| Variant | Setup | `unsubscribe` returned | Deliveries |
|---|---|---|---|
| `EV13` (U1b) | `x` on `C`; `J` is not a node; `unsubscribe(b, J)`, `b` never registered | `true` [006] | `x recv C(c1)` [002] before; `x recv C(c2)` [007] after |
| `EV13v3` (U1b control) | as `EV13` but `y` on `J` first, so `J` is a node | `false` [008] | `C(c1)` to `x` [003] and `y` [004]; `C(c2)` to `y` [009] and `x` [010] |
| `EV13v2` (U3b) | `a` on `I` only; control publish of `C2(c1)`; `unsubscribe(a, K)`, `K` unrelated to `I` | `true` [005] | `a recv C2(c1)` [002] before; `a recv C2(c2)` [006] after |
| `EV13v4` (U3b without the control publish) | as `EV13v2` but no publish of any `C2` before the `unsubscribe` | `false` [002] | `a recv C2(c2)` [003] after |

Runs 2 and 3: identical lines with identical numbers in all four variants (`cmp`); exit code 0 in all 12 runs. Citations are to `EV13.1.txt`, `EV13v2.1.txt`, `EV13v3.1.txt`, `EV13v4.1.txt`. Nothing in the logs shows the node structure; the explanation is Inferred.

Inferred, from reading `SI` and `B` (not seen in a log): `unsubscribe(b, J)` runs `removeValue` (`SI:157-163`, `B:165-171`). When no node is a super-key of `J`, `innerRemoveValue` (`SI:166-178`) creates a node `J` with the parent's values and `integrate` (`SI:222-228`) adopts the known sub-keys below it; the changes it returns are the adopted nodes with their values, so they are non-empty even though `b` was never in any set, and `unsubscribe` returns `true` (`B:165-171`). In `EV13` the adopted node is `C` with `{x}`; `n.removeValue(b)` (`SI:61-67`) removes nothing, and the cache entry of `C` is rebuilt from `findValues`, which still holds `x`, so `x` keeps receiving. In `EV13v3` `J` is a node, `innerRemoveValue` reaches it, `b` is in no set, nothing changes: `false`. For `EV13v2`: `K` is not a super-key of any node, so a node `K` is created, and `findSubKeysExcept` (`SI:202-214`) finds sub-keys of `K` anywhere in the tree; the control publish of `C2(c1)` had added `C2` to the index as a key (`publish` calls `addKey`, `B:177-187`, `SI:111`), so `C2` is found and adopted, the changes are non-empty and the result is `true`; `a` is registered on `I` and the cascade removes from `K`'s own node, so `C2` still reaches `a`. In `EV13v4` no `C2` node exists, `findSubKeysExcept` finds nothing, the changes are empty: `false`.

#### Result for U1b and U3b of the scope of #905

U1b: Pekko returned `true` for `unsubscribe(b, J)` with `b` never registered and `J` not a node [`EV13` 006], and `x` kept receiving `C` events [`EV13` 007]; with `J` already a node it returned `false` [`EV13v3` 008]. The prediction held on both counts. U3b: the delivery prediction held (`a` still received `C2` after `unsubscribe(a, K)` [`EV13v2` 006], [`EV13v4` 003]), but the Boolean prediction `false` held only when the event class had never been published [`EV13v4` 002]; after a publish of `C2(c1)` Pekko returned `true` [`EV13v2` 005] (Inferred reason: the publish made `C2` a node that `K` adopts). The Boolean for an unrelated channel therefore depends on which classes the stream has already seen, not on the pair `(a, K)`.

## 3. Summary

| Scenario | One line |
|---|---|
| EV1 | `subscribe` of a pair already registered returns `false` and delivers once; unsubscribe of an unknown pair returns `false`; `unsubscribe(s)` returns `()`; `unsubscribe(a, Sub)` with only `Base` registered returns `true` and stops `Sub` delivery |
| EV1t | with traits `I`, `J extends I`, `C implements J`: `unsubscribe(a, J)` with only `I` registered returns `true` and stops `C` delivery [005]; the quirk of EV1 appears with interfaces; unsubscribing `J` from a subscriber of `I` and `J` also stops `C` via `I` |
| EV2 | a null subscriber is `IllegalArgumentException("subscriber is null")` for all three calls; a null channel is accepted by `subscribe` and breaks the stream (`NullPointerException` on `publish`) |
| EV3 | class, superclass, interface and `AnyRef` match; a `Sub` is delivered to `Base`, `Sub`, `Iface` and `AnyRef` subscribers; an unmatched event is dropped silently (no dead letter); `publish(null)` throws `NullPointerException`; no order among subscribers |
| EV4 | surprise-free: one copy per subscriber under duplicate class matches; a covered registration reports `false` |
| EV4f | on a fresh stream `z` on `AnyRef` then `Base` returns `true` [002] (not `false`), one copy delivered [003]; with `Base` already a node (`x` first) it returns `false` [003]; `Base` then `AnyRef` returns `true`, one copy |
| EV5 | accepted deliveries complete after `unsubscribe` returns (`e2`-`e4`); a later publish is not delivered (`e5`); same for `unsubscribe(s)` |
| EV6 | the publisher returns while the subscriber is blocked; all events arrive in order afterwards |
| EV7 | termination removal is asynchronous: in the window a publish gives `DeadLetter(event, deadLetters, subscriber)` (0 or 1 in these runs); an already-terminated subscriber is registered (`true`) then removed |
| EV8 | a full bounded mailbox gives `DeadLetter(event, deadLetters, subscriber)`, no `Dropped`, no error to the publisher |
| EV9 | confirmed: `DeadLetter(Dropped(msg, reason, entrySender, null), deadLetters, deadLetters)`, no `Dropped` event; surprise: `drop` of an unknown id throws `NullPointerException` |
| EV10 | no violation of "nothing delivered after `unsubscribe` returns" and no gap before it in 3 runs (counts vary) |
| EV11 | a `DeadLetter` that cannot reach a full `DeadLetter` subscriber is republished as `DeadLetter(DeadLetter(...))` to the same subscriber until a `StackOverflowError` reaches the sender [010]; a subscriber ahead of the full one sees up to 1001 nested wrappers (`EV11v4` [013]), one behind it sees none and loses the event (`EV11` [013]); with no overflow (`EV11v2`) nothing nests |
| EV12 | on a System that is terminating (a held shutdown phase, or a user child blocked in `postStop`) and on one that is terminated: `subscribe` `true` and `unsubscribe` `true`/`()` with no exception [010], [019], [075], [080]; a publication reaches the subscriber while terminating [014], [016] and a non-actor ref after termination [077]; the unsubscriber exists while terminating [008] and is terminated after [074]; no dead letters over 20 publishes after two subscribers were stopped while terminating [071] (Observed); that they were removed by the unsubscriber is Inferred |
| EV13 | `unsubscribe` of a never-registered subscriber returns `true` when the channel is not a node but has a known sub-channel (`J`, with `C` registered) [006] and `false` when it is a node [008]; from an unrelated trait `K` it returns `true` after a publish made the event class a node (`EV13v2` [005]) and `false` before (`EV13v4` [002]); delivery to the registered subscribers is unchanged in all four |

## 4. Differences with the current goPekko (code reading only; no test was run)

Read: `pekko-actor/actor/deadletter.go`, `pekko-actor/actor/system.go` (`eventSink`, `emit`), `pekko-actor/internal/deadletter/hub.go`.

| Subject | Pekko (this oracle) | goPekko today |
|---|---|---|
| Event stream | `EventStream` with class channels (S1) | none. The only fan-out is `System.SubscribeDeadLetters(fn func(DeadLetter)) (cancel func())` (`deadletter.go:35-42`) over `deadletter.Hub` (`hub.go:31-106`); `eventSink` (`system.go:17-25`, `emit` at `:198-203`) is unexported, string-kinded and for tests |
| Subscriber | an `ActorRef`; delivery is an enqueue (S2, EV6) | a `func(DeadLetter)` run synchronously on the publishing goroutine, in registration order, possibly concurrently, which must not block or panic (`deadletter.go:35-39`, `hub.go:45-71`): a blocked callback blocks the publisher, the opposite of EV6 |
| Event types | `DeadLetter(message, sender, recipient)`, `Dropped`, any `AnyRef` | `DeadLetter{Msg any; To ID}` only (`deadletter.go:22-25`): no sender; no `Dropped`; no other event type |
| Matching, duplicates | class hierarchy, one copy per subscriber (EV3, EV4) | none: every callback receives every `DeadLetter` once |
| Return values | `Boolean` of S4 (EV1) | `Subscribe` returns an idempotent `cancel` (`hub.go:92-105`); no "already subscribed" result |
| Unsubscribe linearization | accepted deliveries complete, later ones are not made (EV5, EV10) | `Publish` calls a snapshot of the callbacks outside the lock (`hub.go:50-70`): a callback may still run after `cancel` returned if the publication took its snapshot earlier. `Barrier` (`hub.go:117-126`) waits for such publications; Pekko has no counterpart because its delivery is an enqueue |
| Termination | `eventStreamUnsubscriber` watches subscribers (S7, EV7) | not applicable to callbacks; no watcher |
| Bounded overflow | `DeadLetter` to `deadLetters` (EV8) | a `DeadLetter{Msg, To}` publication (`ref.go:78-82`) without the sender |
| `MessageBuffer.drop` | `DeadLetter(Dropped(...))`, no `Dropped` event (EV9) | no `MessageBuffer` and no `Dropped` event type |

## 5. Limits of this evidence

- One JVM per run, the default dispatcher with `parallelism-max = 4`, Classic actors; no Typed `EventStream.Subscribe`/`Publish` adapters (see the EVT-1 oracle for `Dropped` on Typed).
- Delivery order between subscribers of one publication is not fixed and was not studied beyond the three runs (EV3, EV4, EV9).
- EV7: the window between termination and removal depends on scheduling; only the bound "0 or 1 dead letter" was seen. No scenario forces the window open.
- EV10 shows no violation in three runs of a race; it does not prove its impossibility (S6).
- The `debug` flag of `EventStream` and `startUnsubscriber()` were not exercised separately; the system's stream was used for EV7 to EV9. Remote and cluster subscribers, `ActorEventBus` use outside `EventStream`, and a `sys == null` stream (`E:52`) were not run.
- EV1t: one trait hierarchy (a diamond or several unrelated interfaces on `C` was not run); only `C` as event class; the `U` explanation is code reading, not a log; the `unsubscribe(a, I)` calls after the first one are the harness's own continuation and say nothing beyond the Boolean.
- EV4f: only `AnyRef` and `Base` were used as the covering and covered classifiers (not an interface or a deeper chain); one event class (`Base`); the `SI` explanation is code reading, not a log; `x` in `EV4fv2` is not fenced (its delivery was seen in all three runs, the answer only needs `z`).
- EV11: one sender (the main thread, not an actor: a `StackOverflowError` inside an actor's `receive` was not run, and Pekko's handling of fatal errors there is not claimed); capacity 1 with push timeout 0 only (EV8's configuration); one `Base` event class; the order of the subscribers is the order of subscription only as far as `EV11` and `EV11v4` show; the depth of nesting depends on the thread stack (default JDK 11 settings) and varies per run; 2 of 12 runs ended with exit code 3 (termination or fence timeout after the error), cause not studied; the Typed `EventStream` adapters, a bounded `Dropped` subscriber (`EV11v3` used `AllDeadLetters`, which also matches `DeadLetter`) and a subscriber of `Dropped` alone (it cannot be reached by this path) were not run.
- EV12: one stream (the system's), one event class, one `terminate()` with a held shutdown phase or a blocked `postStop`; the terminating states were read at a point where the hold was entered, not through the whole shutdown; the terminated state has no live actor, so delivery there was seen only through a harness ref that is not an actor and removal on termination was not run; the absence of dead letters [071] is not proof of an unsubscribe (window of EV7); `Identify` waited at most 2 s for an answer; the shutdown phases after the held one (service unbind, requests, stop of the user guardian) were not held one by one; remote, cluster, a system with `coordinated-shutdown.run-by-actor-system-terminate = off` and an `ActorSystem.close()` path were not run.
- EV13: one hierarchy (`I`, `J`, `C`, and `K`, `C2`), one registered subscriber per node, a subscriber that is not registered anywhere; only the Boolean and the deliveries of one event after the call were read; the node structure is code reading. A subscriber `b` registered elsewhere, other classifiers of the same event class, and the effect of earlier publishes of other classes were not run. `EV13v2` shows the Boolean depends on the published classes; which other publishes change it was not explored.
- The null-channel behavior (EV2) depends on the index state: an earlier exploratory run (not kept) saw `unsubscribe(a, null)` throw after `subscribe(a, null)`; the kept runs use fresh streams.
