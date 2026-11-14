# Addendum 1: M67, an excess of PostStop (Owner option A)

Applies to the package of `3d476a1`/`423d0fe` (RULEBOOK, RUNBOOK, manifest, record). After the result of M66 (`timeout`, 30 of 30: S22 cannot turn an OMISSION of the second `PostStop` into a detection), the Owner chose option A: add **M67**, derived from the real Pekko path, to provoke a **third** `PostStop`.

## Authorization and conditions
- The extra call must run **before the termination notification**, so the end of the scenario cannot hide it. In Pekko v1.7.1 `finishTerminate` calls `a.aroundPostStop()` (`actor/dungeon/FaultHandling.scala:254`) and only afterwards `parent.sendSystemMessage(DeathWatchNotification(...))` (`:263`) and `tellWatchersWeDied()` (`:266`); the S22 completion predicate needs a `terminated`. An additional call placed in the same expression as the existing `aroundPostStop` therefore precedes every notification.
- S22, the comparator, the harness and their classifications stay intact. Detection must show the observed count and fire `G5.f`.
- **M66 and its 30 timeouts are kept** as they are: not replaced, withdrawn or reclassified as detected.
- One directed campaign of M67, local, no agents. If it does not detect `G5.f`, the cause is presented before any repetition.
- Documented apart: (1) `G5.f` detects an EXCESS of `PostStop`, if M67 shows it; (2) the omission tried by M66 remains undetected because of the completion predicate of S22. No claim of total mutation detection, nor of coverage of both senses of the exact count.
- The inherited selftests that fail or do not finish are not investigated here; their results are kept and a follow-up is opened.

## M67
**"A Stop that arrives while a restart waits for its children makes the old instance receive an extra PostStop (a third one), before the termination is notified."** Same flag as M66 (set in `faultRecreate` only while the restart waits for children, cleared when `finishRecreate` starts), and in `finishTerminate` the existing `a.aroundPostStop()` is followed, while the flag is set, by a second call in the same expression. Expected: S22 concurrent `failed`, `G5.f` fired with `c: 3 post-stop, expected 2` (a count above the expected one is a failure on a prefix and on a complete log). `G5.c` and `REQ` should not fire from it. Designed expectation: `[("S22", ["G5.f"])]`.

## Surfaces
As in RULEBOOK section 5, plus the follow-up issues on GitHub. `judge/mutations/M66.patch` and the M66 row stay byte-identical; only `M67.patch` and the M67 row are added.
