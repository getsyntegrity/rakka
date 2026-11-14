# Differential run: Kill under the default decider and under a restarting decider (#1031, MSG-09)

- Date: 2026-11-02. Owner brief: executable Kill evidence for scenarios A (default Stop) and B (custom Restart); run the Pekko JVM test when the environment allows.
- Pekko: v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, exported with `git archive` from the main checkout's `upstream/pekko` into a `mktemp -d` outside the repo (upstream untouched).
- Harness: `judge/pekko-harness/judge.sbt` copied to the copy's root, plus `KillProbe.scala` from this directory (the same method as `msg-02-failure-sender/pekko-repro`).
- Environment: macOS (Darwin 25.3.0), sbt 1.12.14 and OpenJDK 11.0.32 (Temurin) from sdkman; `JAVA_8_HOME` = sdkman Zulu 8.0.504 (Pekko's build requires it; `judge/run-pekko.sh` sets the same).
- Command (in the copy): `sbt -batch "judge/runMain KillProbe"`. Exit code 0. sbt prints two `bnd: Invalid value for Bundle-Version` errors (the archive has no git metadata); they do not stop the run.
- Output: `observed.txt` (the OBS lines).

## Result against the Go tests
| Scenario | Pekko observed | Go assertion | Match |
|---|---|---|---|
| A: `Kill` never reaches `receive` | `childReceives=0` | `rec.got` empty (`TestKillE2E_DefaultStrategyStopsTheChildAndNotifiesTheWatcherOnce`) | yes |
| A: instances | `made=1` | `made == 1` | yes |
| A: PostStop | `postStops=1` | `== 1` | yes |
| A: PreRestart / PostRestart | `0` / `0` | `rec.failed` empty / `postRestarts == 0` | yes |
| A: Terminated to the watcher | `terminated=1` | `count == 1` after a barrier | yes |
| B: decider cause | `ActorKilledException(Kill) isActorKilled=true` | `errors.Is(cause, ErrActorKilled)`, one cause | yes (class check vs `errors.Is`) |
| B: preRestart message | `Some(Kill)` | `FailedMessage.Value() == actor.Kill` | yes |
| B: PreRestart / PostRestart | `1` / `1` | once each | yes |
| B: child keeps working | `ping` answered by the new instance, `made=2` | `probe` received, `made == 2` | yes |
| B: Terminated to the watcher | `terminated=0` | `count == 0` | yes |

## Limits of this run
- `postStops=2` in scenario B is not compared: the Pekko default `preRestart` calls `postStop` on the old instance and the second call is the new instance stopped when the probe terminates the system. The Go test actor overrides `PreRestart` without calling PostStop, so its count differs by construction.
- The probe polls with `Thread.sleep(10)` until the Terminated arrives (a JVM probe, not a Go test), and reads the counters after `system.terminate()`.
- The scenarios were observed once, not in a calibrated campaign. The comparison is by the OBS tuples above, not by `judge/comparator.py`; the judge harness does not drive Kill (S-scenarios have no Kill step).
- No logging assertion (`LoggingReceiveSpec.scala:279-290`) was compared.
