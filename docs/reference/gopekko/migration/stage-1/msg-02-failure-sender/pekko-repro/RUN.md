# Differential run: sender() in the decider, preRestart/postStop and Terminated/WatchWith (#814)

- Date: 2026-10-28. Owner authorization: focused JVM runs, differential only (G1/G2/G4/G6 and the Terminated/WatchWith equivalent).
- Pekko: v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, exported with `git -C upstream/pekko archive v1.7.1` into a `mktemp -d` outside the repo (upstream/pekko untouched).
- Harness files copied as `judge/run-pekko.sh` does (`judge/pekko-harness/judge.sbt`, `Judge.scala`), plus `Probe.scala` from this directory.
- Environment: macOS (Darwin 25.3.0), OpenJDK 11.0.32 (Temurin), sbt from sdkman, Scala = Pekko's build version.
- Command (in the copy): `sbt -batch "judge/runMain Probe"`. Exit code: 0 (second run; the first failed to compile on fatal unused-pattern-variable warnings, fixed in the probe).
- Note: sbt prints two `bnd: Invalid value for Bundle-Version, HEAD+...` errors (the archive has no git metadata); they did not stop the run.
- Output: `observed.txt` (the OBS lines of the run).

## Result against the READ map

| Scenario | Where | Observed | READ expectation | Match |
|---|---|---|---|---|
| G4 (parent Escalate, grandparent Restart) | decider | child-c | the failing child (m10, m12) | yes |
| G4 | preRestart sender / message | child-c / Some(Failed(child-c, boom, uid)) | child; Failed message (m13, m7) | yes |
| G6 (grandparent Stop) | decider / postStop | child-c / child-c | child (m10, m6) | yes |
| T1 | Terminated | victim-T1 (the watched actor) | m14 | yes |
| T2 | WatchWith message | victim-T2 | m15 | yes |
| T3 | postStop after death pact | victim-T3 | m5, m7 | yes |

In G2 terms: the child failed on a message told by actor `x-sender-*`; that name never appears in any line. Last line of observed.txt (G4 postStop, deadLetters) is the new instance stopped at system teardown: it is a reading taken when the system shuts down and does NOT prove the sender in the turn after the restart (PostRestart); that is covered only by source reading. No difference between source reading and execution.

The incarnation identity criterion (stop and respawn under the same name) is NOT covered by this probe.
