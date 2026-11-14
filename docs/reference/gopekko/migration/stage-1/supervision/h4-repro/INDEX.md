# Evidence index (ADR 0017, supervision)

Everything here was produced on 2026-10-14 against the pinned Apache Pekko Classic runtime (`v1.7.1`, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`),
exported from `upstream/pekko` into a disposable directory; neither `upstream/pekko`, `judge/` nor `pekko-actor/` was modified. None of it was
produced by the migration kit, and none of it is a parity claim. Read `RESULTS.md` first.

| File | What it is | Kind |
|---|---|---|
| `RESULTS.md` | the results, with the program's own output lines reproduced verbatim | summary |
| `H4Repro.scala` | restart with held children followed by a Stop (H4), and the window probe (H6) | program run on Pekko |
| `H5Probe.scala` | `watch`/`watchWith`/`unwatch` with a null reference and with a reference of another ActorSystem (H5) | program run on Pekko |
| `A2A4Probe.scala` | a decider returning `null` (A2) and the 24-cell budget matrix on the real `ChildRestartStats` (A4) | program run on Pekko |
| `run.sh` | how each program is run (`run.sh <copy> <out> [H4Repro\|H5Probe\|A2A4Probe]`) | script |
| `h4-h6-sbt.out`, `h5-sbt.out`, `a2a4-sbt.out` | the **raw sbt output** of the three runs (build log included) | raw evidence |
| `winconv/main.go` | the Go window conversion checked against the 13 values observed in H6 | Go check |
| `a5-edge/oracle.py`, `oracle.out` | a line-by-line **transcription** of `requestRestartPermission` with an injected clock, and the exact-zero edge cases; **not a Pekko run** | derived from source |
| `a5-edge/matrix_check.py`, `matrix_check.out` | the transcription compared with the real class on the 24 cells of `a2a4-sbt.out` (24 compared, 0 mismatches) | cross-check |

**Correction of one sentence of `RESULTS.md`.** `RESULTS.md` says the raw sbt outputs "are kept outside the repository". They are now committed
here (`h4-h6-sbt.out`, `h5-sbt.out`, `a2a4-sbt.out`). `RESULTS.md` is not edited, so that the hash the owner reviewed stays valid.

**What is not shown by any of this.** The exact-zero reading of the clock (A5) cannot be driven on the real class (`System.nanoTime`), so those
expectations come from the transcription. The `Receive` case of a null reference was not run. The `winconv` and `a5-edge` checks are not
tests of goPekko and are not part of any CI gate.
