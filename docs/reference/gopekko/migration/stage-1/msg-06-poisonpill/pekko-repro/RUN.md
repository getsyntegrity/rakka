# Upstream observation of PoisonPill (#770, MSG-06)

- Date: 2026-11-04. Pekko v1.7.1 (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`), `git archive` of `upstream/pekko` into a `mktemp -d` outside the repo, `judge/pekko-harness/judge.sbt` at its root and `PoisonProbe.scala` in `judge/` (the method of `msg-09-kill-e2e/pekko-repro`). `JAVA_8_HOME` = sdkman Zulu 8, sbt 1.12.14, OpenJDK 11.0.32.
- Command: `sbt -batch "judge/runMain PoisonProbe"`, exit 0 (the two `bnd: Invalid value for Bundle-Version` errors do not stop it). Output: `observed.txt` (the OBS lines).
- Limits: observed once. The JVM probe waits with `Thread.sleep` (10 ms polls; 200 ms before releasing the gate in scenario 3). `postStops` and `terminated` in the `final` line add both systems.

| # | Pekko observed (`observed.txt`) | Go test | Match |
|---|---|---|---|
| 1 | 1, 2 run; PoisonPill skips `receive`; `DeadLetter 3`; one Terminated per system | `TestPoisonPill_RunsEarlierMessagesThenStopsWithoutFailing` | yes |
| 2 | PoisonPill to the stopped actor: `SuppressedDeadLetter PoisonPill`, no `DeadLetter` | `TestPoisonPill_UndeliveredPublishesNoDeadLetter` | **no: open divergence 5a (#871)**; Go publishes nothing |
| 3 | Stop accepted with `2, Kill, 3, PoisonPill, 4` queued: `DeadLetter 2, Kill, 3`, `SuppressedDeadLetter PoisonPill`, `DeadLetter 4` (mailbox order) | `TestPoisonPill_DeadLetterOrderWithQueuedKillAndPoisonPill` | **no: open divergence 5c**; Go publishes `2, 3, 4, Kill` and nothing for PoisonPill |
| 4 | `preRestarts=0`: PoisonPill is no failure | same test as 1 (`rec.failed` empty) | yes |

Not observed by the probe (no Pekko evidence, Go tests only): PoisonPill to a suspended actor, full bounded mailbox, PoisonPill racing Stop. Their rows stay "pending approval (ADR 0033 D13)".
