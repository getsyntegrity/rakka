# Differential run: sender() in preRestart after a death pact, scenario T6 (#814 Part 2)

- Date: 2026-10-29. Owner authorization: extend the JVM probe to the one scenario recorded as NOT EXECUTED (`preRestart` after a death pact). `Probe-T.scala` and `RUN-T.md` are untouched.
- Pekko: v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, exported with `git -C upstream/pekko archive v1.7.1 | tar -x -C <mktemp -d>` outside the repo (upstream/pekko untouched).
- Harness: `judge/pekko-harness/judge.sbt` and `Judge.scala` copied as `judge/run-pekko.sh` does, plus `Probe-T6.scala` copied as `judge/src/main/scala/ProbeT6.scala`.
- Environment: macOS (Darwin 25.3.0), OpenJDK 11.0.32 (Temurin), `JAVA_8_HOME=$HOME/.sdkman/candidates/java/8.0.504+1-zulu`, sbt 1.12.14.
- Command (in the copy): `sbt -batch "judge/runMain ProbeT6"`. Exit code: 0, first run.
- Output: `observed-T6.txt` (the OBS lines), sha256 `60eb3686f18d8d60210524a42f925ffda54b252cb8081c86b349f6b6a19f39aa`; probe sha256 `8a68fdb25e53f5594b0e3022bee5179bb0ee26b3b8b9c16f31494b259f09b288`. The scratch copy was deleted.

## Scenario

A supervisor with `OneForOneStrategy() { case _ => Restart }` runs one child. The child watches `victim-T6` in `preStart` and its `receive` handles only another message, so the `Terminated` falls to `unhandled`, which throws `DeathPactException` (`Actor.scala:658`). The strategy is explicit because Pekko's default decider would Stop on `DeathPactException`. The child's `preRestart` and `postRestart` print `sender()`.

## Result against the READ map

| Where | Observed | READ expectation | Match |
|---|---|---|---|
| `preRestart` sender | `victim-T6` (the terminated actor) | `currentMessage` is the delivered `Terminated` (`ActorCell.scala:553`, not reset on failure), read in the restart path (`FaultHandling.scala:101`) | yes |
| `preRestart` cause | `DeathPactException`, `dead=victim-T6` | `Actor.scala:658` | yes |
| `preRestart` message | `Some(Terminated)` | `FaultHandling.scala:101` | yes |
| `postRestart` sender | `deadLetters` (new instance, fields cleared) | `clearActorFields` (`ActorCell.scala:718-719`) | yes |

No difference between source reading and execution.

## Limits

- One run, one scenario, Classic Pekko on the JVM. It shows what Pekko does; it does not show what goPekko does.
- The Go side was compared by reading only: `sender_terminated_test.go` T6 ("PreRestart reads the observed actor", `tsPact`) asserts the same three facts for the Go death pact; `supervision_hooks.go:43` documents `FailedMessage` as the `Terminated`. The Go test was neither run nor changed here. No Go test appears to be missing for the observed facts (sender, message, cause; `PostRestart` absent is covered by T5). Whether a later Go test should also pin the cause (`ErrDeathPact`, the Go analogue of the observed `DeathPactException`) is left to the Owner and is not decided here.
- No parity and no V3 are claimed.
