# Probe: `whenTerminated` after the root guardian fails (Pekko 2.0.0, JVM 17)

Evidence class: **observed on the JVM, finite window** (the pending part); the direct `stop()` of the bubble walker stays **SOURCE_VERIFIED** (`ActorRefProvider.scala:428-490`, never probed).
A finite observation does not prove permanent non-termination: the window is 6 s.

## Question
The Rust tests `guardian_termination_tests::a_failed_root_guardian_leaves_when_terminated_pending` (PR #370/#371) claim that, when the root guardian fails, the bubble walker records the cause in its own
`causeOfTermination`, stops walking, discards the death notification and leaves the public `terminationPromise` pending. Until this probe that was read from the Scala only: on the JVM `ActorSystem.create`
fails first by the logger start-up timeout of the default `Slf4jLogger`.

## What does and does not work
1. `pekko.loggers = []` alone is **not enough**: the system still starts `org.apache.pekko.event.slf4j.Slf4jLogger` (`LoggersProbe`: the system's own `pekko.loggers` is `[Slf4jLogger]` even when `[]` is given).
   Both `pekko.loggers = []` **and** `pekko.use-slf4j = off` are needed (baseline then starts in 445 ms; `RootGuardianProbe` case A).
2. A bounded *default mailbox* (`BoundedMailbox`, capacity 1) does **not** reproduce the scenario: the logging actors themselves get that mailbox and time out, `create` throws after 5 s (`RootGuardianProbe` case B).
3. The exact scenario of the Rust test: a default `MailboxType` that **promises** `UnboundedMailbox.MessageQueue` (so that the selection-time check passes) but **creates a bounded queue only for the first actor
   that asks** (the root guardian). The first `create` call is `Actor[pekko://root-failure-probe/]`, the second `/user`, the third `/system` (printed by the probe). The re-check of `Dispatch.init` on the queue
   really created fails at the root. `ActorSystem.create` then **throws** (no handle is returned), but the mailbox type receives the `ActorSystem` in `create`, so the probe captures it and observes `whenTerminated`.

## Configuration (`RootFailureProbe`)
```
pekko.loggers = []
pekko.use-slf4j = off
pekko.stdout-loglevel = "OFF"
pekko.actor.default-mailbox.mailbox-type = "FirstBoundedMailbox"
```

## Results (JDK 17.0.21, Pekko 2.0.0 jars of `oracle/target/lib`)
| run | `ActorSystem.create` | `whenTerminated.isCompleted` at ~1 s / ~3 s / ~6 s after the failure |
|---|---|---|
| 1 | threw after 6072 ms (logger initialisation timeout) | false / false / false |
| 2 | threw after 5895 ms (logger initialisation timeout) | false / false / false |
| 3 | threw after 5905 ms (logger initialisation timeout) | false / false / false |
| 4 | threw after 858 ms (`cannot create children while terminating or terminated`) | false / false / false |
(An earlier run without the capture code threw after 742 ms with the same `IllegalStateException`; it had nothing to observe.)

**Reading:** in 4 of 4 runs the captured system's `whenTerminated` stayed pending for the whole observation window, whichever way `create` failed. This agrees with the Rust claim; it does not establish what happens
after 6 s, nor the behaviour of a direct `stop()` of the walker after the failure.

## Reproduce
```
javac -cp "oracle/target/lib/*" -d /tmp/probe-classes docs/evidence/mailbox-compat/probes/root-guardian-failure/*.java
java -Dslf4j.internal.verbosity=ERROR -cp "/tmp/probe-classes:oracle/target/lib/*" RootFailureProbe
```
Files: `FirstBoundedMailbox.java` (the mailbox type), `RootFailureProbe.java` (the scenario), `RootGuardianProbe.java` (cases A and B), `LoggersProbe.java` and `ConfigProbe.java` (why `pekko.loggers = []` is not enough).
