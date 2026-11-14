# Differential run: sender() around Terminated deliveries, scenarios T3, T4, T5, T7, T9 (#814 Part 2)

- Date: 2026-10-29. Owner authorization: focused JVM runs for T3, T4, T5, T7 and the incarnation identity T9; T1, T2 and the death-pact `postStop` are reused from `observed-T.txt` (not rerun).
- Pekko: v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, exported with `git -C upstream/pekko archive v1.7.1 | tar -x -C <mktemp -d>` outside the repo (upstream/pekko untouched).
- Harness files copied as `judge/run-pekko.sh` does (`judge/pekko-harness/judge.sbt`, `Judge.scala`), plus `Probe-T.scala` (copied as `judge/src/main/scala/ProbeT.scala`).
- Environment: macOS (Darwin 25.3.0), OpenJDK 11.0.32 (Temurin), `JAVA_8_HOME=$HOME/.sdkman/candidates/java/8.0.504+1-zulu`, sbt 1.12.14, Scala = Pekko's build version.
- Command (in the copy): `sbt -batch "judge/runMain ProbeT"`. Exit code: 0 on the second run. The first run failed to compile (exit 1): `ActorPath.uid` is `private[pekko]`; the probe was changed to print `ActorRef.hashCode` (derived from the uid) and `==` (which compares path and uid, `ActorRef.scala:183-186`).
- Note: sbt prints two `bnd: Invalid value for Bundle-Version, HEAD+...` errors (the archive has no git metadata); they did not stop the run.
- Output: `observed-T-extended.txt` (the OBS lines), sha256 `fe977c0cd106ead3fe0473f62403f74284f6cbe812ccc9a1461670d064798d4e`; probe sha256 `afa31a33ea7781097be4460ebe2d29d65af8cc0448996e05e834963b80d4a3d3`. The scratch copy was deleted.

## Result against the READ map

| Scenario | Where | Observed | READ expectation | Match |
|---|---|---|---|---|
| T3 (retained sender x-sender-T3 from a resumed failure) | Terminated | victim-T3 | the observed actor replaces the retained sender (`ActorCell.scala:553`) | yes |
| T4 | Terminated / postStop after success | victim-T4 / deadLetters | success clears (`:559`) | yes |
| T5 | Terminated / preRestart / postRestart | victim-T5 / victim-T5 (message is the Terminated) / deadLetters | failure keeps until restart | yes |
| T7 | next message after a resumed Terminated failure | tell: deadLetters; tell from z: z-T7 | next delivery replaces | yes |
| T9 | Terminated after stop and respawn of `inc` | sender == old: true, sender == new: false, Terminated.actor == sender; hash differs old/new | per-incarnation identity | yes |

Not covered: T8 (a subject of another System) has no Pekko counterpart: Pekko never crosses actor systems in a watch. It is a Go mechanism check, not a differential claim. T6 `preRestart` after a death pact was not run; only the observed `postStop` (T3 of `observed-T.txt`) backs T6. No difference between source reading and execution.
