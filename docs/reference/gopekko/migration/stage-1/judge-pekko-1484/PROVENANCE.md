# Pekko campaign for S13 and its neighbours (#1484) (provenance)

Pekko side only: the S13 baseline, the baselines of S07, S08, S11 and S21, and the mutations M12, M5, M11, M6, M62 and M64 on Apache Pekko v1.7.1, run with the harness and comparator of `develop` at
`ef0de70` (#1487). Nothing here changes the judge, a scenario, the comparator or the classification of any outcome. No harness error, timeout or incomplete campaign occurred, and none was reinterpreted.
It installs 13 rows in `judge/mutation-report.tsv` (the other 55 rows stay as they were). It does not touch the Go report.

## Identity

| Item | Value |
|---|---|
| goPekko commit (harness, comparator, specs, patches) | `ef0de70238fee3dd2901d7b9b78bc2a5d4b7c369`; `judge/` at `4c79c274` (#1488) differs only in `judge/go-mutation-report.tsv` |
| Root tree / `judge/` / `pekko-actor/` | `0807bddb42795ab144aab46648e365cdf9118534` / `774c9c56d55d3d2df35a48e4f0cd28d9db65866d` / `baf7315d6d22a61306f5f5fdc66e88c2fdf45040` |
| Pekko | `upstream/pekko` `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1) |
| Pekko copy | `git archive` of v1.7.1 outside the repository, committed in a throwaway repository: commit `5e68be93c6f27750fcbeffa6174091d87d8d5183`, tree `6b214fa85b0b2038e38d3fb201ee91d4ae123b7f`; its `actor/` tree `2c90bf5c4fec7bfae7578c60c40bacd098ee160c` equals `upstream/pekko` `HEAD:actor` |
| Platform | darwin/arm64 (macOS 26.3.1), 16 CPUs, Python 3.14.5 |
| JDK / sbt | Baselines: JDK 8 (zulu 8.0.504+1) first on `PATH`, sbt 1.12.14, `sbt about` ok first (stated by the operator in `identity.txt`; no baseline log records the JDK, unlike the mutation logs). Mutations: sbt 1.12.14 started by `judge/mutate.py` through `bash -lc`, which picked the sdkman default JDK (Eclipse Adoptium 11.0.32, printed at the top of each `mut/*/sbt.log`), with `JAVA_8_HOME` exported to the 8.0.504 zulu JDK. The two protocols differ; the report rows are from the second |
| Runs | 30 per concurrent scenario (`CONCURRENT_RUNS`), 1 controlled, deadline 10000 ms. The 300 of `docs/judge.md` are for calibration and were not run |
| Commands | `JUDGE_RUNS=30 judge/run-pekko.sh <copy> <out> controlled\|concurrent 10000 <scenarios>` for the baselines; `python3 -I judge/mutate.py run <copy> <workdir> <M>`, one process per mutation in the order M12 M5 M11 M6 M62 M64 |
| Date | 2026-11-08 |

## Baselines (`evaluate.py --expect-runs`, complete campaigns)

| Scenario | controlled (1 run) | concurrent (30 runs) |
|---|---|---|
| S13 | pass | 30 pass |
| S07 | pass | 30 pass |
| S08 | pass | 30 pass |
| S11 | pass | 30 pass |
| S21 | not declared | 30 pass |

0 fail, 0 harness-error, 0 timeout in every cell.

## Mutations

Rows as written by `mutate.py` (counts are runs: `fail` is the number of runs the comparator failed). 0 harness-error and 0 timeout in every row.

| Mutation | Scenario | Mode | Runs | Pass | Fail | Outcome | Rules fired | Expected rule hit |
|---|---|---|---|---|---|---|---|---|
| M12 | S13 | controlled | 1 | 0 | 1 | failed | `G6.d` | yes |
| M12 | S13 | concurrent | 30 | 0 | 30 | failed | `G6.d` | yes |
| M5 | S07 | controlled | 1 | 0 | 1 | failed | `I2.a` | yes |
| M5 | S07 | concurrent | 30 | 0 | 30 | failed | `I2.a` | yes |
| M11 | S08 | controlled | 1 | 0 | 1 | failed | `G6.b` | yes |
| M11 | S08 | concurrent | 30 | 0 | 30 | failed | `G6.b` | yes |
| M6 | S10 | controlled | 1 | 0 | 1 | failed | `I3.a` | yes |
| M6 | S11 | controlled | 1 | 0 | 1 | failed | `I3.a`, `REQ` | yes |
| **M6** | **S10** | **concurrent** | **30** | **30** | **0** | **survived** | none | **no** |
| M6 | S11 | concurrent | 30 | 0 | 30 | failed | `REQ` | **no** (`I3.a` was designed) |
| M62 | S11 | controlled | 1 | 0 | 1 | failed | `I3.a`, `I3.b` | yes |
| M62 | S11 | concurrent | 30 | 0 | 30 | failed | `I3.b` | yes |
| M64 | S21 | concurrent | 30 | 0 | 30 | failed | `G3.c` | yes |

## The designed cell that was not met: M6 × S10 × concurrent

`mutate.py` designs M6 (the dead-letter actor drops the publication) to break `I3.a` in S10 and in S11, without distinguishing the mode. In concurrent the cell did not give `failed`:
`survived`, 30 of 30 runs pass, no rule fired (row `M6 S10 concurrent` of `judge/mutation-report.tsv`). M6 × S11 × concurrent did fail, but by `REQ` (missing `dead-letter` of `M5`), not by `I3.a`,
so its `expected_hit` is `false`. The results of these rows (runs, pass, fail, outcome, rules fired) are the same as the ones that `judge/mutation-report.tsv` already held for M6; what changed is that they were produced again, at `ef0de70` (`harness_commit` and `comparator_commit`). Observed in the files of this tree, not reasoned:

- `judge/scenarios/S10.json` and `judge/scenarios/S11.json` declare the rule `I3.a` with `"modes": ["controlled"]`, and `judge/comparator.py:506` skips a rule whose `modes` do not include the run's mode. In concurrent
  neither scenario evaluates `I3.a`; only `I3.b` runs, and it cannot see a dead letter that is missing.
- The scenario-level `required` of S11 is `dead-letter M5`; S10's is `received M1` only. That is why S11 concurrent fails by `REQ` and S10 concurrent has nothing to fail by.
- `docs/judge.md`, row I3, states the same split: strict (published once) in controlled; in concurrent only "never both received and dead-lettered, never twice". Pekko documents dead letters as best effort
  (`upstream/pekko/docs/src/main/paradox/general/message-delivery-reliability.md:315-320`: "This delivery happens on a best-effort basis; it may fail even within the local JVM").

Not concluded here: whether the gap is in the mutation's design table or in the scenario (an issue is opened for that). M6 is detected by its designed rule `I3.a` in controlled, in S10 and S11.

## Package

Outside the repository, in `goPekko-evidence/`; the tag `evidence/pekko-campaign-ef0de70` points at the full SHA above.

| File | sha256 |
|---|---|
| `judge/mutation-report.tsv` (installed byte for byte from the campaign) | `393e71b2179736299043a7d8a1822cec67b58eeb2bf54e0355fab5ba9b86dc6e` |
| `pekko-campaign-ef0de70.tar.gz` (760 files, 138,615 bytes: report, `identity.txt`, baseline logs, per-mutation logs and `sbt.log`, `drive.py`, `drive.out`) | `5e5544d41203bf19ccc3f0ab07a2c0474b73df19217150ebec9cac6f4af0aa8f` |
| `pekko-campaign-ef0de70.SHA256SUMS` (one line per file, paths relative to the root of the tar) | `22d14aa02c2c53b88417ddcda73bf533bdbaa0158ceb40446b50eafe788f88e4` |
| `pekko-campaign-ef0de70.tar.gz.sha256` | checks the tar; verified with `shasum -a 256 -c` |

The tar was extracted and all 760 lines of the SHA256SUMS verified `OK`.

## Checker

`scripts/check-judge-matrix.py` with the report installed: exit 0. Pekko CURRENT: rows 13/68; designed detections 11/55; mutations with a CURRENT detection 6/32. The other rows are HISTORICAL (the comparator
or the harness changed since). goPekko CURRENT on linux/amd64 is unchanged: detected 21/21, benign survived 1/1.

## What it shows, and what it does not

- On Pekko v1.7.1 the 13 rows above were produced with today's harness and comparator; five mutations are caught by their designed rule in every designed cell, and M6 is caught by its designed rule only in controlled.
- One platform, one campaign of 30 concurrent runs, no V level by itself. It does not calibrate the judge (the 300 runs), and the baselines of S07, S08, S11 and S21 were run with JDK 8 while the mutations ran under the JDK 11 launcher.

## Correction of 2026-11-09: the JVM was JDK 11 for the baselines too

The text above, which stays as it was written, says that the baselines ran on JDK 8 ("JDK 8 (zulu 8.0.504+1) first on `PATH`") and that only the sbt launcher of the mutations was JDK 11. **That is wrong.** Both ran on
**JDK 11 (Eclipse Adoptium / Temurin 11.0.32)**, the Pekko code included:

- `judge/run-pekko.sh` and `judge/mutate.py` source `~/.sdkman/bin/sdkman-init.sh` before calling `sbt`, and that script overwrites `PATH` and `JAVA_HOME` with the sdkman default (Temurin 11) (*inferred* from the script's effect: after sourcing it, `java -version` printed 11.0.32 and `JAVA_HOME` was the sdkman `current` link, in a shell that had JDK 8 first; the log banners below are the only observation inside this package). A JDK 8 first on `PATH` before the call does not survive it.
- The `judge` sbt project does not fork `run` (`judge.sbt` sets no `fork` and sbt's default is `false`; `show judge/run/fork` printed `false` in an interactive sbt session on 2026-11-08, output not kept), so the harness and the Pekko classes it exercises run inside the JVM of sbt.
- `JAVA_8_HOME` only feeds Pekko's own build (`jdk8home` in `upstream/pekko/project/JdkOptions.scala:106`); it does not choose the JVM that runs the harness.
- Observed, not reasoned: the first line of every `sbt.log` of the package (`mut/*/sbt.log`) and of the baseline logs (`baseline/*.log`) reads `welcome to sbt 1.12.14 (Eclipse Adoptium Java 11.0.32)`.

What this changes in the record: the Identity row "JDK / sbt" and the sentence "the baselines of S07, S08, S11 and S21 were run with JDK 8" are void; the 13 rows of `judge/mutation-report.tsv` and the baselines of the package are **JDK 11
history**. They are not evidence on JDK 8, and nothing in this file should be read as that. Their results as recorded above are not re-checked or changed by this correction; it speaks only of the JVM.

The package `pekko-campaign-ef0de70` (sha256 `5e5544d41203bf19ccc3f0ab07a2c0474b73df19217150ebec9cac6f4af0aa8f`) has a second problem found later: the tar was built on macOS and holds `._*` AppleDouble entries that its
`SHA256SUMS` does not list (`judge/verify_package.py` reports 771 problems; the 760 listed files do verify). It stays in `goPekko-evidence/` as history. Its `identity.txt` (line 9) repeats the wrong JDK 8 statement and cannot be edited there; this section supersedes it.

The operator session reports a rerun of the baselines and of four of the mutations on JDK 8 on 2026-11-08, with `-java-home` in the `.sbtopts` of the disposable copy (banner `Java 1.8.0_504`) and the same outcomes (1 of 1 and 30 of 30 in every cell). Its logs are not in the repository or in the package, so this is **unverified and not evidence**; it was not installed.
The Pekko-side evidence on JDK 8 will come from one new campaign with runners that pass `-java-home "$JAVA_8_HOME"` themselves and refuse a row when the sbt banner is not 1.8, and its package replaces this one.
