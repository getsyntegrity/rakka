# Oracle harness (Pekko Classic, sbt)

Shared sbt project (`pekko-actor` and `pekko-actor-typed` 1.7.1) that runs the scenarios of the Pekko oracles in `docs/parity/oracles/`. It is not a goPekko module and contains no Go code; no "Ported from" header applies. Owner decision of 2026-11-06 on #1417 (option C). The directory location is a proposal of the TL awaiting the Owner's confirmation.

| Oracle | Package | Scenarios |
|---|---|---|
| `docs/parity/oracles/eventstream-spec-2026-11-06.md` (next PR of the stack) | `eventstream` | `EV1` `EV1t` `EV1tv2` `EV1tv3` `EV2` `EV3` `EV4` `EV4f` `EV4fv2` `EV4fv3` `EV5a` `EV5b` `EV6` `EV7` `EV8` `EV9` `EV10` `EV11` `EV11v2` `EV11v3` `EV11v4` `EV13` `EV13v2` `EV13v3` `EV13v4` `EV12` `EV12v2` |
| `docs/parity/oracles/scheduler-lifecycle-2026-11-10.md` | `scheduler` | `SC1` `SC2fd` `SC2fr` `SC3fd` `SC3fr` `SC3o` `SC4fd-disp` `SC4fd-global` `SC4fd-plain` `SC4fr-disp` `SC4fr-global` `SC4fr-plain` `SC4sfd` `SC4sfr` `SC4ifd` `SC4ifr` `SC5once` `SC5fd` `SC5fr` `SC5fdlive` `SC5frlive` |
| `docs/parity/oracles/suppressed-dead-letters-2026-11-10.md` (story #871) | `deadletter` | `DL1` `DL1b` `DL2` `DL3` `DL4` `DL5` `DL6` `DL7` `DL8` `DL8b` `DL8c` |
| `docs/parity/oracles/unstash-deadletters-2026-11-10.md` (scopes of #1182 and #927) | `unstash` (needs `pekko-actor-typed`) | `US1` `US2` `US3` `US4` `US6` |
| `docs/parity/oracles/restart-stash-held-2026-11-11.md` (slice B of #1182, QA of PR #1545) | `restartstash` (needs `pekko-actor-typed`; reuses the `unstash` helpers) | `RS0` `RS1` `RS2` `RS3` |
| `docs/parity/oracles/deadletter-log-2026-11-11.md` (slice 0 of #1479) | `deadletterlog` (needs `pekko-actor-typed` only because it is on the classpath; sets `pekko.use-slf4j = off`) | `LG1` `LG2` `LG3` `LG4` `LG5` `LG6` `LG7` `LG8` `LG9` |

Older oracles were run from harnesses outside the repository and are not backfilled.

## Versions

- Pekko `pekko-actor` 1.7.1 (tag `v1.7.1`, the version pinned in `upstream/pekko`), Scala 2.13.18.
- sbt: the project asks for 1.10.7 (`project/build.properties`); the `sbt` launcher on the machine was 1.12.14 (sdkman) and downloads 1.10.7.
- JDK: the `eventstream` runs used Temurin 11.0.32+9 (`~/.sdkman/candidates/java/11.0.32-tem`). The `scheduler`, `deadletter` and `deadletterlog` runs used Java 1.8.0_504 (Zulu 8.96.0.205, `~/.sdkman/candidates/java/8.0.504+1-zulu`): sourcing sdkman replaces `JAVA_HOME` with JDK 11, so pass `sbt -batch -java-home "$HOME/.sdkman/candidates/java/8.0.504+1-zulu"` and check that the first line of the sbt output ends in `Java 1.8.0_504`; a run on another Java is not valid for that oracle.
- `fork := true`: each scenario can run in its own JVM.

## Commands

From this directory, with the JDK above first on `PATH`:

```sh
sbt compile
CP="$(sbt -batch 'export Runtime/fullClasspath' | tail -1)"   # the classpath is generated, never committed
java -cp "$CP" eventstream.Run EV1 > EV1.1.txt; echo $?        # one scenario, one JVM, exit code 0 on success
```

The same scenario through sbt: `sbt "runMain eventstream.Run EV1"`. Each scenario ends with `Runtime.halt` (exit code 0, or 1 after a line `HARNESS FAILURE`, or 3 from the 60 s watchdog).

## Reading a raw log

Every line is `[nnn +ms] text`; `nnn` is the order of the call to `log()`, `ms` is time since the JVM start. A line `x fence f` means that the actor `x` handled a `Fence` message sent directly from the publishing thread: all deliveries that thread made before it are already in the mailbox of `x` ahead of the fence, so the fence replaces sleeps as the proof that "nothing else arrived". Lines of `Collector` actors are `name recv event`.

## Layout

- `build.sbt`, `project/build.properties`: the build. `target/` and `project/target/` are ignored (`.gitignore`) and never committed.
- `src/main/scala/scheduler/Run.scala`: one method per scenario family of the scheduler oracle (`sc1`, `sc2`, `sc3p`, `sc3o`, `sc4`, `sc5`) and a small `org.apache.pekko.actor.Access` object that reaches `SchedulerException` and `ActorRef.isTerminated`, both `private[pekko]`. Its `[nnn +ms]` log format is the same.
- `src/main/scala/eventstream/Run.scala`: helpers (`Collector`, `Fences`, `Records`) and one method per scenario.
- `src/main/scala/unstash/Run.scala`: Typed actors spawned on a Classic system (`adapter._`), the same `Collector` and `Fences`, and `MailboxProbe` (package `org.apache.pekko.actor`) to poll `hasSystemMessages` and `numberOfMessages` of an actor's mailbox instead of sleeping. Run it as `java -cp "$CP" unstash.Run US1`.
- `src/main/scala/restartstash/Run.scala`: a supervised Typed actor whose restart waits for a child with a blocked `PostStop`, and a hook outside the supervisor (`Behaviors.intercept` around `supervise`) that logs and counts what the supervisor returned for each message or signal, so the content of the restart stash is established by a counter and not by a sleep. Reuses `Log`, `Collector`, `Fences`, `Watcher` and `Run.subscribe/fenceAll` of `unstash`. Run it as `java -cp "$CP" restartstash.Run RS1`.
- `src/main/scala/deadletterlog/Run.scala`: the oracle of the INFO log of the dead letters (`Collector` on `AllDeadLetters`, `InfoCollector` on `Logging.Info`, `Fences`; `DlProbe` in package `org.apache.pekko.actor` polls the mailbox of the `DeadLetterListener` and its termination). Run it as `java -cp "$CP" deadletterlog.Run LG1`.
- `src/main/scala/deadletter/Run.scala`: the oracle of suppressed dead letters (`Collector` on `DeadLetter`, `SuppressedDeadLetter` and `AllDeadLetters`, `Fences`, one method per scenario; `TerminatedHelper` and `StopHelper` in package `org.apache.pekko.actor` reach `private[pekko]` members). Run it as `java -cp "$CP" deadletter.Run DL1`.
