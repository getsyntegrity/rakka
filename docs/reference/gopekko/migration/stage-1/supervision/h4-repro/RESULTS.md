# Reproductions against the pinned Apache Pekko Classic runtime (H4, H5, H6)

Purpose: the original runtime defines the port (owner answer H4, 2026-10-14). These are controlled reproductions, not
judge scenarios and not tests of goPekko. Nothing in `judge/` or `pekko-actor/` was modified to produce them.

## Environment and inputs

- Source: Apache Pekko `v1.7.1`, commit `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, taken from the submodule `upstream/pekko`
  (read-only, `git submodule status` shows no local change).
- Disposable export: `git -C upstream/pekko archive --format=tar v1.7.1`, unpacked into a directory outside the repository,
  committed and tagged `v1.7.1` in a throwaway git repository (`sbt-dynver` needs one). `actor/` tree id of the export
  `2c90bf5c4fec7bfae7578c60c40bacd098ee160c` equals the submodule's; `testkit/` tree id `28d7d42859c066615200033648823ac584124670`
  equals the submodule's.
- Build: sbt `1.12.14` on `Eclipse Adoptium Java 11.0.32`, Pekko compiled by its own build (`actor/target/scala-2.13`), with the
  judge's unchanged sbt project `judge/pekko-harness/judge.sbt`. The reproduction sources are added to that project only in the
  disposable export.
- Commands (from the repository root): `migration/stage-1/supervision/h4-repro/run.sh <copy> <out> H4Repro` (H4 and H6) and
  `... H5Probe` (H5). 2026-10-14. Sources: `H4Repro.scala`, `H5Probe.scala` (this directory). The raw sbt outputs are kept outside the
  repository; the program's own lines are reproduced below verbatim.
- Models and migrator: none. These programs were written by the orchestrator; the migration kit was not used for them.

## H4: restart whose children have a retained termination, followed by a Stop

Setup: parent `P` (strategy `OneForOneStrategy() { case _ => Restart }`) supervises `C`; `C` has two children `g1`, `g2` whose
`postStop` blocks on a latch; a watcher `W` watches `C`. The driver sends `boom` to `C` (it throws), waits until `C.preRestart`
has returned and both `g1`, `g2` have started their `postStop`, asks `P` to `context.stop(C)`, waits one second so the `Terminate`
system message is processed while the children are still held, releases the latch and waits for `Terminated`. Three modes, three runs
each; every run of a mode gave the same counts and the same order of the hooks of `C`; only the begin order of the two held children
(`g1`, `g2`) varied (manual mode, run 2).

`default-hooks` (the default `Actor.preRestart`: unwatch and stop the children, then call `postStop()`), run 1:

```
001 C#1.constructor
002 C#1.preStart
003 G(g1).preStart
004 G(g2).preStart
005 C#1.preRestart(message=Some(boom))
006 G(g1).postStop-begin
007 C#1.postStop
008 C#1.preRestart-returned
009 G(g2).postStop-begin
010 DRIVER: restart waiting for retained children; requesting Stop of C
011 DRIVER: releasing the retained children
012 G(g1).postStop-end
013 G(g2).postStop-end
014 C#1.postStop
015 W.Terminated(c)
SUMMARY run1 mode=default-hooks: terminatedDelivered=true constructors=1 C.preStart=1 preRestart=1 postRestart=0 C.postStop=2 C#1.postStop=2 C#2.postStop=0 W.Terminated=1
```

(runs 2 and 3: identical summary.) `manual-children-stop` (`preRestart` stops the children itself and does not call `postStop`), run 1:

```
001 C#1.constructor
002 C#1.preStart
003 G(g1).preStart
004 G(g2).preStart
005 C#1.preRestart(message=Some(boom))
006 C#1.preRestart-returned
007 G(g1).postStop-begin
008 G(g2).postStop-begin
009 DRIVER: restart waiting for retained children; requesting Stop of C
010 DRIVER: releasing the retained children
011 G(g1).postStop-end
012 G(g2).postStop-end
013 C#1.postStop
014 W.Terminated(c)
SUMMARY run1 mode=manual-children-stop: terminatedDelivered=true constructors=1 C.preStart=1 preRestart=1 postRestart=0 C.postStop=1 C#1.postStop=1 C#2.postStop=0 W.Terminated=1
```

(runs 2 and 3: the same counts; `g1`/`g2` start order varied in run 2, nothing else.) `plain-stop` (control: no restart), run 1:

```
001 C#1.constructor
002 C#1.preStart
003 G(g1).preStart
004 G(g2).preStart
005 DRIVER: requesting Stop of C (no restart)
006 G(g1).postStop-begin
007 G(g2).postStop-begin
008 DRIVER: releasing the retained children
009 G(g1).postStop-end
010 G(g2).postStop-end
011 C#1.postStop
012 W.Terminated(c)
SUMMARY run1 mode=plain-stop: terminatedDelivered=true constructors=1 C.preStart=1 preRestart=0 postRestart=0 C.postStop=1 C#1.postStop=1 C#2.postStop=0 W.Terminated=1
```

Observed, for the pinned runtime:

1. With the default `preRestart`, a Stop that reaches the actor while the restart waits for its children **abandons the restart**:
   no new instance is built (`constructors=1`), no `postRestart`, no second `preStart`.
2. In that case `postStop` of the **old instance is called twice**: once by the default `preRestart` (item 007, before the children
   finished) and once by the termination (item 014, after the children finished). The watcher gets exactly one `Terminated`, after
   the second call.
3. When `preRestart` does not call `postStop` (`manual-children-stop`), the old instance's `postStop` runs once, after the children.
   In the plain stop it also runs once, after the children (the termination rule of ADR 0014).
4. During the restart the default `preRestart` calls `postStop` while the children are still stopping (`007` precedes both children's
   `postStop-end`; the latch holds them): the cleanup of the old instance during a restart does not wait for the children. The order in
   which the two children begin their `postStop` is not guaranteed.

Contradiction with the judge: judge G5 says "`post-stop` runs once per instance" (`docs/judge.md:69`). The observed behavior is two
calls for the default hooks when a Stop interrupts a restart. The Pekko documentation that G5 cites (`actors.md:941-957`) describes the
termination order (children first, then `postStop`) and does not say "once per instance", so it is not contradicted. This is recorded in
ADR 0017 (S11) with a proposed correction of the contract/scenario; the Go runtime is not changed.

## H6: what the real `OneForOneStrategy` stores for (maxNrOfRetries, withinTimeRange)

Read by reflection from the field `retriesWindow` of a real `OneForOneStrategy` (Pekko 1.7.1):

```
n=-1 Inf                           -> (None,None)
n=0  Inf                           -> (Some(0),None)
n=3  Inf                           -> (Some(3),None)
n=-1 10s                           -> (None,Some(10000))
n=3  10s                           -> (Some(3),Some(10000))
n=3  0                             -> (Some(3),Some(0))
n=3  -1ms                          -> (Some(3),None)
n=3  1ns                           -> (Some(3),Some(0))
n=3  999999ns                      -> (Some(3),Some(0))
n=3  1000000ns (1ms)               -> (Some(3),Some(1))
n=3  1999999ns                     -> (Some(3),Some(1))
n=3  2147483647ms (Int.Max)        -> (Some(3),Some(2147483647))
n=3  2147483648ms (2^31)           -> (Some(3),Some(-2147483648))
n=3  4294967295ms (2^32-1)         -> (Some(3),Some(-1))
n=3  4294967296ms (2^32)           -> (Some(3),Some(0))
n=3  4294967301ms (2^32+5)         -> (Some(3),Some(5))
n=3  Long.Max ns                   -> (Some(3),Some(2077252342))
TimeUnit.MILLISECONDS.toNanos(-2147483648) = -2147483648000000
```

The conversion proposed for Go, `int32(within.Milliseconds())` (a signed 64-bit division truncating toward zero, then an explicit
truncation to the low 32 bits), reproduces every row: `winconv/main.go` runs it on 13 inputs (the window rows above plus `10s`) and
compares with the stored values observed here; on 2026-10-14 `go run` reported 13 matches, 0 mismatches, and
`windowNanos(-2147483648) = -2147483648000000`, equal to the value observed in Pekko.

## H5: `watch`, `watchWith` and `unwatch` with a null reference and with a reference of another ActorSystem

Program `H5Probe.scala`, run on the same export. The call is made in `preStart` of a child of a parent whose decider records the
failure and stops the child.

```
--- null (watch) ---
  parent saw: org.apache.pekko.actor.ActorInitializationException: pekko://A/user/p-null-watch/watcher: exception during creation, root cause message: [ActorRef is not internal: null] (is Exception=true) default decider would be: Stop
--- null (watchWith) ---
  parent saw: org.apache.pekko.actor.ActorInitializationException: ... root cause message: [ActorRef is not internal: null] (is Exception=true) default decider would be: Stop
--- null (unwatch) ---
  parent saw: org.apache.pekko.actor.ActorInitializationException: ... root cause message: [ActorRef is not internal: null] (is Exception=true) default decider would be: Stop
--- other-system (watch) ---
  parent saw: no failure reported to the parent
  Terminated delivered for pekko://B/user/x-watch existenceConfirmed=true
  termination observed after stopping the subject: true
--- other-system (watchWith) ---
  parent saw: no failure reported to the parent
  custom WatchWith message delivered
  termination observed after stopping the subject: true
--- other-system (unwatch) ---
  parent saw: no failure reported to the parent
  termination observed after stopping the subject: false
```

Observed in the run: with a null reference the call made in `preStart` reaches the parent as an `ActorInitializationException` whose root
cause message is `ActorRef is not internal: null`, for `watch`, `watchWith` and `unwatch`; the wrapper is an `Exception` and the default
decider would Stop it. A reference of **another ActorSystem of the same JVM is not rejected**: `watch` and `watchWith` work and the
termination is delivered.

From the source, not from the run: the root cause is `IllegalArgumentException` (`dungeon/DeathWatch.scala:47-48,63-64,77-78`, the probe
printed only the wrapper class), and a null reference used from `receive` would raise it as a plain `Exception`, for which the default decider
gives Restart (`FaultHandling.scala:222`); that case was not run.

## A5: the reading of exactly zero (owner decision 2026-10-14)

`a5-edge/oracle.py` is a line-by-line transcription of `ChildRestartStats.requestRestartPermission` (`FaultHandling.scala:49-89`) with the
clock injected, plus the rejected "explicit flag" variant for comparison; `a5-edge/oracle.out` is its output. **It is a transcription of the pinned
source, not a run of Pekko**: the real class reads `System.nanoTime` and cannot be driven to a chosen reading, so the exact-zero expectations are
derived from the source text. With `retries = 2`, `window = 1000 ms`, readings `0, 100 ms, 1050 ms` upstream gives `allow, allow, deny` (the flag
variant `allow, allow, allow`); with `retries = 1`, readings `-2000 ms, 0, 1050 ms` upstream gives `allow, allow, deny` (flag variant: third
allowed). Controls (no zero reading; reading exactly at the window edge; one nanosecond past it) agree in both.

## Issues

Divergences and missing capabilities found here are registered in #443 (Watch across Systems), #444 (Spawn), #445 (guardians), #446 (Resume,
Escalate), #447 (AllForOne), #448 (default strategy, `ErrGoexit`), #449 (stop during restart, scope of G5) and #450 (explicit gaps); the
existing #168-#175 cover the restart units and the Go gate of S13/S14/S16.

## A2 and A4 (third round, 2026-10-14)

`A2A4Probe.scala`, run on the same export (`run.sh <copy> <out> A2A4Probe`; raw output `a2a4-sbt.out`).

**A2, an invalid directive.** `Directive` is a sealed trait with exactly four values (`FaultHandling.scala:118-161`), so user code cannot create an
invalid one; the only out-of-domain value on the JVM is `null`. Observed: a decider returning `null` makes the **parent** fail with
`scala.MatchError: null`; the parent's `preRestart` received that reason and `Some(org.apache.pekko.dispatch.sysmsg.Failed)` as the message.
No upstream equivalent of a Go invalid directive exists; the Go validation is Go-specific (ADR 0017 A2).

**A4, the matrix.** The real `ChildRestartStats.requestRestartPermission`, five calls in a row per cell, for `maxNrOfRetries` in `{-1, 0, 1, 3}` and
`withinTimeRange` in `{Inf, -1 ms, 0, 10 s, 2^31 ms, 2^32 ms}`: 24 cells, reproduced exactly by the transcription (`a5-edge/matrix_check.out`: 24
compared, 0 mismatches). Key facts: no count with no window is the only unlimited cell; no count with a finite window allows one restart per window;
a count of 0 always denies; a zero or overflowed window makes the count ineffective.
