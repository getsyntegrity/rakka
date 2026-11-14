# Extra Classic scenario: the path an ActorRef reports (unit actor-853-ref-path, Owner condition D-6)

- Date: 2026-10-29. Evidence class: **Executed** (38 rows produced by running Pekko code; nothing is read from source here).
- Purpose: Owner G2 condition on #853 (D-6): "Ejecutá también el escenario Classic adicional durante el kit. Si contradice el contrato propuesto, detené la parte afectada y reportalo." The scenario covers rows N1, N2, N3, N4, P5, P7, P8 and P9 of `docs/scopes/853-ref-reports-actor-path.md` (scope PR #1287, reviewed head `360951389190f36ea85fcad25f84e0970e9b6586`, not merged).
- Pekko: v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. The submodule HEAD equals the pin: `git -C upstream/pekko rev-parse HEAD` printed `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` before and after the run.
- `upstream/pekko` was never modified: `git -C upstream/pekko status --short` was empty before and after, and nothing was run inside it. `Probe.scala` is an artifact of the disposable copy; it is stored here only as evidence and is not built in this repository.
- Output: `observed.txt` is the 38 `ROW|...` lines of the run with the `ROW|` prefix removed (`<label>|<value>`) and trailing whitespace removed (none was present). Element sequences are printed as `n=<size> ["a","b"]` because Scala `toString` shows `List("")` and `Nil` identically.
- Go was never run on a JVM, and no parity is claimed: the rows describe Pekko only.

## Environment

- macOS (Darwin 25.3.0), JDK OpenJDK 11.0.32 active (`java -version`), sbt launcher with the version declared by `project/build.properties` (1.12.14 in `~/.sbt/boot`), Scala 2.13 (the `actor` module).
- `JAVA_8_HOME=~/.sdkman/candidates/java/8.0.504+1-zulu` was set for BOTH sbt commands (steps 3 and 4), as in `migration/stage-1/identity-oracle/pekko-repro/RUN.md` (PR #1292).

## Commands, in order

1. Export the pinned tree into a disposable directory outside the repository (cwd: the goPekko repository root):

   ```
   <copy>=$(mktemp -d)
   git -C upstream/pekko archive --format=tar 9a54d1294c3ac7821bbed3c940769c3d5027d1ff | tar -x -C <copy>
   ```

2. Add the probe to the copy (the file declares `package org.apache.pekko.actor.probe853`):

   ```
   mkdir -p <copy>/actor/src/main/scala/probe
   cp migration/stage-1/actor-853-ref-path/pekko-repro/Probe.scala <copy>/actor/src/main/scala/probe/Probe.scala
   ```

3. Compile the actor module (cwd: `<copy>`):

   ```
   JAVA_8_HOME=~/.sdkman/candidates/java/8.0.504+1-zulu sbt -batch -Dsbt.offline=true "actor/compile"
   ```

   Exit 0. First run (fresh copy): 33 s wall, `[success] Total time: 18 s`; rerun after trimming the probe (incremental): 9 s wall, `[success] Total time: 4 s`.

4. Run the probe (cwd: `<copy>`):

   ```
   JAVA_8_HOME=~/.sdkman/candidates/java/8.0.504+1-zulu sbt -batch -Dsbt.offline=true "actor/runMain org.apache.pekko.actor.probe853.Probe"
   ```

   Exit 0. 10 s wall on the first run, 9 s (`[success] Total time: 3 s`) on the rerun whose output is `observed.txt` (the first run printed 47 rows; the 9 dropped rows only repeated names and text forms already shown: identical values elsewhere in the table).
## Scenario
A real classic `ActorSystem("sys")`: a top-level actor `a`; a parent `par` with a named child `b` and an anonymous child; an anonymous top-level actor; a `Boss` whose child `f` throws `IllegalStateException` on a message, so the default supervisor restarts it (the `postRestart` hook counts down a latch, so there is no sleep); an actor `s` stopped with `gracefulStop` (so its termination is awaited, not slept on) and then created again under the same name; the three guardians read from `system.provider`.

## Results: row -> proposed contract -> result -> contradicts?
| Row | Proposed contract (scope) | Result (`observed.txt`) | Contradicts? |
|---|---|---|---|
| N2, P5: top-level `a` | path `pekko://sys/user/a`, elements `user`, `a` | `pekko://sys/user/a`, `n=2 ["user","a"]`, parent `pekko://sys/user` | no |
| N2: nested `par/b` | `pekko://sys/user/par/b` (the contract's nested example is `a/b`); parent path is the parent's path | `pekko://sys/user/par/b`, `n=3 ["user","par","b"]`; `b.path.parent == par.path` true | no |
| N4, P9: anonymous child | last element is a generated name starting with `$`, parent is the owner's path | nested: name starts with `$` true, 3 elements, parent equal to `par.path`; top-level: name starts with `$` true, 2 elements, parent `pekko://sys/user` | no |
| N3: restart keeps the path | same path before and after a restart; same reference identity | `pekko://sys/user/boss/f` before and after, equal true, the `ActorRef` is the same instance (`eq`), uid unchanged (`postRestart` reached true) | no |
| P7: terminated actor | the same path after termination | `isTerminated` false before, true after `gracefulStop`; `s.path` still `pekko://sys/user/s`, equal to the path before, uid unchanged | no |
| N1: two incarnations at one path | equal paths, distinct identities | `s1.path == s2.path` true, uid differ true, `s1 == s2` false (ref equality uses uid and path) | no |
| P8: guardians | root `pekko://sys/`, `/user` `pekko://sys/user`, `/system` `pekko://sys/system` | `rootGuardian` `pekko://sys/` (`n=1 [""]`, a `RootActorPath`), `guardian` `pekko://sys/user` (`n=1 ["user"]`), `systemGuardian` `pekko://sys/system` (`n=1 ["system"]`); `guardian.parent == rootGuardian.path` true; `rootPath == rootGuardian.path` true | no |

No result contradicts the proposed contract. Two points where the Go rendering is not the Pekko rendering, neither a contradiction: (1) Pekko's `ActorRef` equality includes the uid, Go's reference identity is the `ID` and `PathOf` is path only (the scope's adaptation, rows R1 and N1); (2) the Go `ActorPath.Elements()` of the root is `[""]`, the same as the Pekko root `n=1 [""]` row above (`actor_path.go`, #1008).

## Limits
- Only the `actor` module was compiled; no other Pekko module was built or run. One run only: no repetition, so no flakiness assessment (the uid values are random and are never printed, only their relationships).
- The scenario does not cover a stopped actor's path after the registry retired it (Pekko has no registry), foreign-System refs, nil or typed-nil handles, `Terminated.Who()` or the stale failed child: they have no Pekko analogue (P1-P4, P6, P10 to P12 are Go-only).
- The sbt output contains the `Invalid Pekko version: HEAD+<timestamp>` OSGi warnings because the exported archive has no git metadata; both sbt commands exited 0.
- This is not a kit execution record and not a gate artifact. No Go parity is claimed.
