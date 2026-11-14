# Source anchors for ADR 0019 (#211)

Each row names a short alias used by the ADR, a file under `upstream/pekko` (Apache Pekko v1.7.1,
`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), a line, and a text that line must contain.
The validation record shows the script that checks the rows and every `ALIAS:line` citation of the ADR.

| Alias | File | Line | Text on that line |
|---|---|---|---|
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 88 | `case WriteMessages(` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 102 | `breaker.withCircuitBreaker(asyncWriteMessages(prep))` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 110 | `asyncWriteMessages returned invalid number of results` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 117 | `sendWriteResponse(WriteMessagesSuccessful` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 134 | `WriteMessageRejected(p, e, actorInstanceId)` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 148 | `WriteMessagesFailed(e, atomicWriteCount)` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 153 | `WriteMessageFailure(p, e, actorInstanceId)` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 162 | `case r @ ReplayMessages(` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 174 | `val readHighestSequenceNrFrom = math.max(0L, fromSequenceNr - 1)` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 183 | `val toSeqNr = math.min(toSequenceNr, highSeqNr)` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 184 | `if (toSeqNr <= 0L \|\| fromSequenceNr > toSeqNr)` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 191 | `if (!p.deleted)` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 199 | `RecoverySuccess(highSeqNr)` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 202 | `ReplayMessagesFailure(e)` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 209 | `case d @ DeleteMessagesTo(` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 216 | `DeleteMessagesFailure(e, toSequenceNr)` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 230 | `The batch is only for performance reasons` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 241 | `must be written to the data store atomically` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 244 | `UnsupportedOperationException` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 251 | `uncertainty about if the messages were stored or not` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 263 | `it must not be included in a later replay` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 267 | `Data store connection problems must not be signaled as rejections` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 272 | `Calls to this method are serialized by the enclosing journal actor` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 277 | `A PersistentActor will not send a new WriteMessages request before` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 284 | `requests for the highest sequence number may be made concurrently` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 293 | `def asyncWriteMessages(messages` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 303 | `def asyncDeleteMessagesTo(persistenceId: String, toSequenceNr: Long)` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 301 | `journal must maintain the highest sequence number and never decrease it` |
| ARC | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncRecovery.scala | 30 | `The future must be completed with a failure if any of the persistent messages` |
| ARC | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncRecovery.scala | 37 | `is the lowest of what was returned by` |
| ARC | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncRecovery.scala | 42 | `This call is NOT protected with a circuit-breaker` |
| ARC | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncRecovery.scala | 66 | `Journal must maintain the highest sequence number and never decrease it` |
| ARC | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncRecovery.scala | 57 | `def asyncReplayMessages(persistenceId: String` |
| ARC | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncRecovery.scala | 80 | `def asyncReadHighestSequenceNr(persistenceId: String` |
| PER | persistence/src/main/scala/org/apache/pekko/persistence/Persistent.scala | 50 | `require(payload.nonEmpty, "payload of AtomicWrite must not be empty!")` |
| PER | persistence/src/main/scala/org/apache/pekko/persistence/Persistent.scala | 61 | `AtomicWrite must contain messages for the same persistenceId` |
| PER | persistence/src/main/scala/org/apache/pekko/persistence/Persistent.scala | 68 | `this assumes they're gapless` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 353 | `private def nextSequenceNr()` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 410 | `sequenceNr = nextSequenceNr(),` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 434 | `sequenceNr = nextSequenceNr(),` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 442 | `eventBatch ::= atomicWrite` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 367 | `journal ! WriteMessages(journalBatch, self, instanceId)` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 533 | `if (toSequenceNr == Long.MaxValue \|\| toSequenceNr <= lastSequenceNr)` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 782 | `val highestSeqNr = Math.max(highestJournalSeqNr, lastSequenceNr)` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 878 | `case WriteMessageRejected(p, cause, id)` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 882 | `updateLastSequenceNr(p)` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 886 | `case WriteMessageFailure(p, cause, id)` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 905 | `case WriteMessagesSuccessful =>` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 909 | `case WriteMessagesFailed(_, writeCount)` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 790 | `case ReplayMessagesFailure(cause)` |
| RFL | persistence/src/main/scala/org/apache/pekko/persistence/journal/ReplayFilter.scala | 92 | `if (r.persistent.sequenceNr < seqNo)` |
| RFL | persistence/src/main/scala/org/apache/pekko/persistence/journal/ReplayFilter.scala | 105 | `it is alright with == seqNo` |
| REF | persistence/src/main/resources/reference.conf | 141 | `mode = repair-by-discard-old` |
| INM | persistence/src/main/scala/org/apache/pekko/persistence/journal/inmem/InmemJournal.scala | 85 | `for (w <- messages; p <- w.payload)` |
| INM | persistence/src/main/scala/org/apache/pekko/persistence/journal/inmem/InmemJournal.scala | 94 | `Future.successful(Nil) // all good` |
| INM | persistence/src/main/scala/org/apache/pekko/persistence/journal/inmem/InmemJournal.scala | 171 | `highestSequenceNumbers.updated(p.persistenceId` |
| INM | persistence/src/main/scala/org/apache/pekko/persistence/journal/inmem/InmemJournal.scala | 109 | `if (highest != 0L && max != 0L)` |
| LDB | persistence/src/main/scala/org/apache/pekko/persistence/journal/leveldb/LeveldbStore.scala | 91 | `withBatch(batch =>` |
| LDB | persistence/src/main/scala/org/apache/pekko/persistence/journal/leveldb/LeveldbStore.scala | 170 | `leveldb.write(batch, leveldbWriteOptions)` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 88 | `def supportsAtomicPersistAllOfSeveralEvents` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 110 | `AtomicWrite(List(persistentRepr(i), persistentRepr(i + 1)))` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 134 | `"replay all messages" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 141 | `"replay messages using a lower sequence number bound" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 148 | `"replay messages using an upper sequence number bound" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 155 | `"replay messages using a count limit" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 162 | `"replay messages using a lower and upper sequence number bound" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 169 | `"replay messages using a lower and upper sequence number bound and a count limit" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 176 | `"replay a single if lower sequence number bound equals upper sequence number bound" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 183 | `"replay a single message if count limit equals 1" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 190 | `"not replay messages if count limit equals 0" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 194 | `"not replay messages if lower  sequence number bound is greater than upper sequence number bound" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 198 | `"not replay messages if the persistent actor has not yet written messages" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 202 | `"not replay permanently deleted messages (range deletion)" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 220 | `"not reset highestSequenceNr after message deletion" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 237 | `"not reset highestSequenceNr after journal cleanup" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 255 | `"reject non-serializable events" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 261 | `val event = if (i == 7) notSerializableEvent` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 282 | `case WriteMessageRejected(PersistentImpl(payload, 7L` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 286 | `case WriteMessageSuccess(PersistentImpl(payload, 8L` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 293 | `"serialize events" in` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 328 | `"return metadata" in` |
| EAF | persistence/src/test/scala/org/apache/pekko/persistence/EventSourcedActorFailureSpec.scala | 233 | `"call onPersistRejected and continue if persist rejected" in` |
| EAF | persistence/src/test/scala/org/apache/pekko/persistence/EventSourcedActorFailureSpec.scala | 213 | `"call onPersistFailure and stop when persist fails" in` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 770 | `updateLastSequenceNr(p)` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 359 | `if (!writeInProgress && journalBatch.nonEmpty)` |
| WJB | persistence/src/main/scala/org/apache/pekko/persistence/journal/WriteJournalBase.scala | 45 | `val adapter = eventAdapters.get(payload.getClass)` |
| ESO | persistence/src/main/scala/org/apache/pekko/persistence/Eventsourced.scala | 354 | `sequenceNr += 1L` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 234 | `Journals should aim to persist events in-order` |
| AWJ | persistence/src/main/scala/org/apache/pekko/persistence/journal/AsyncWriteJournal.scala | 288 | `the PersistentActor may reuse` |
| INM | persistence/src/main/scala/org/apache/pekko/persistence/journal/inmem/InmemJournal.scala | 167 | `ms :+ pr` |
| INM | persistence/src/main/scala/org/apache/pekko/persistence/journal/inmem/InmemJournal.scala | 117 | `math.min(toSequenceNr, highestSequenceNr(persistenceId))` |
| LDB | persistence/src/main/scala/org/apache/pekko/persistence/journal/leveldb/LeveldbStore.scala | 87 | `var persistenceIds = Set.empty[String]` |
| LDB | persistence/src/main/scala/org/apache/pekko/persistence/journal/leveldb/LeveldbStore.scala | 94 | `Try {` |
| LDB | persistence/src/main/scala/org/apache/pekko/persistence/journal/leveldb/LeveldbStore.scala | 95 | `a.payload.foreach { p =>` |
| LDB | persistence/src/main/scala/org/apache/pekko/persistence/journal/leveldb/LeveldbStore.scala | 110 | `persistenceIds += a.persistenceId` |
| LDB | persistence/src/main/scala/org/apache/pekko/persistence/journal/leveldb/LeveldbStore.scala | 138 | `math.min(toSequenceNr, readHighestSequenceNr(nid))` |
| LDB | persistence/src/main/scala/org/apache/pekko/persistence/journal/leveldb/LeveldbStore.scala | 183 | `counterToBytes(persistent.sequenceNr)` |
| LDB | persistence/src/main/scala/org/apache/pekko/persistence/journal/leveldb/LeveldbStore.scala | 184 | `Key(nid, persistent.sequenceNr, 0)` |
| TKS | persistence-testkit/src/main/scala/org/apache/pekko/persistence/testkit/internal/TestKitStorage.scala | 98 | `updateOrSetNew(key, v => v ++ elems)` |
| TKS | persistence-testkit/src/main/scala/org/apache/pekko/persistence/testkit/internal/TestKitStorage.scala | 113 | `max(getLastSeqNumber(upd), oldSn)` |
| TKS | persistence-testkit/src/main/scala/org/apache/pekko/persistence/testkit/internal/TestKitStorage.scala | 163 | `value.dropWhile(reprToSeqNum(_) <= toSeqNumberInclusive)` |
| TKE | persistence-testkit/src/main/scala/org/apache/pekko/persistence/testkit/EventStorage.scala | 59 | `elems.groupBy(_.persistenceId)` |
| TKP | persistence-testkit/src/main/scala/org/apache/pekko/persistence/testkit/PersistenceTestKitPlugin.scala | 48 | `messages.map(aw` |
| TKP | persistence-testkit/src/main/scala/org/apache/pekko/persistence/testkit/PersistenceTestKitPlugin.scala | 55 | `storage.tryAdd(data)` |
| ORS | persistence/src/test/scala/org/apache/pekko/persistence/journal/AsyncWriteJournalResponseOrderSpec.scala | 98 | `sequenceNr = 0L,` |
| TCK | persistence-tck/src/main/scala/org/apache/pekko/persistence/journal/JournalSpec.scala | 122 | `journal ! WriteMessages(msgs, probe.ref, actorInstanceId)` |
| PRS | persistence/src/main/scala/org/apache/pekko/persistence/Persistence.scala | 82 | `def persistenceId: String` |
| PRS | persistence/src/main/scala/org/apache/pekko/persistence/Persistence.scala | 86 | `When empty, looks in `pekko.persistence.journal.plugin`` |
| PRS | persistence/src/main/scala/org/apache/pekko/persistence/Persistence.scala | 90 | `def journalPluginId: String = ""` |
| PRS | persistence/src/main/scala/org/apache/pekko/persistence/Persistence.scala | 94 | `When empty, looks in `pekko.persistence.snapshot-store.plugin`` |
| PRS | persistence/src/main/scala/org/apache/pekko/persistence/Persistence.scala | 98 | `def snapshotPluginId: String = ""` |
