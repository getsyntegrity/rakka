# Pekko campaign on Java 1.8 with the bounded harness (#1484) (provenance)

The Pekko side of S07, S08, S11, S13 and S21: their baselines on unmutated Pekko v1.7.1 and the mutations M58, M5, M11, M12, M64, M6 and M62, run with the harness, comparator and runners of `develop` at `147b6b36` (#1496). It
installs the first `judge/pekko-baseline-report.tsv` (9 rows) and replaces 15 rows of `judge/mutation-report.tsv` and adds 2 (M58 × S13, controlled and concurrent: the report goes from 68 to 70 rows) with rows that carry `jdk`, `sbt_version` and `platform`. It replaces the Pekko evidence of #1489, whose rows were JDK 11
(correction of 2026-11-09 in `migration/stage-1/judge-pekko-1484/PROVENANCE.md`); that package stays in `goPekko-evidence/` as history, with its `._*` problem. Nothing here changes the judge, a scenario, the comparator or the
classification of any outcome. No harness error, timeout or incomplete campaign occurred; the one designed cell that did not give `failed` is reported below as observed.

## Identity

| Item | Value |
|---|---|
| goPekko commit (harness, comparator, specs, patches, runners) | `147b6b36511543f6734885d98cb0ec08073a5ad4` (`develop`, squash of #1496), clean tree (every row's `harness_commit` and `comparator_commit`) |
| Root tree / `judge/` / `pekko-actor/` | `3c3a07515e809f6670da1a442e8db801c262cfb9` / `64323abd180281a04418f5a4f9b0e6fbfbdaf6e2` / `baf7315d6d22a61306f5f5fdc66e88c2fdf45040` |
| Pekko | `upstream/pekko` `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1) |
| Pekko copy | new `git archive` of v1.7.1 outside the repository, committed in a throwaway repository: commit `8a36f701d24f62cf6e4d08b4b6f0ded376dc7219`, tree `6b214fa85b0b2038e38d3fb201ee91d4ae123b7f`; `actor/` `2c90bf5c4fec7bfae7578c60c40bacd098ee160c` and `testkit/` `28d7d42859c066615200033648823ac584124670` equal `upstream/pekko`; the tracked `.sbtopts` is untouched |
| JVM / sbt | Java 1.8.0_504 (Azul Zulu), sbt 1.12.14. Observed: line 1 of every `sbt.log` (`baseline/sbt.log`, `mut/*/sbt.log`) reads `welcome to sbt 1.12.14 (Azul Systems, Inc. Java 1.8.0_504)`; the rows carry `jdk 1.8.0_504`; the runners refuse any other Java (`judge/jdk.py`) |
| Platform | darwin/arm64 (macOS 26.3.1), 16 CPUs, Python 3.14.5 |
| Runs | 30 per concurrent scenario, 1 controlled, deadline 10000 ms. The 300 of `docs/judge.md` are for calibration and were not run |
| Commands | `python3 -I judge/baseline.py run <copy> <workdir> S07 S08 S11 S13 S21`; then one `python3 -I judge/mutate.py run <copy> <workdir> <M>` per mutation in the order M58 M5 M11 M12 M64 M6 M62 (the driver `camp.py` stops if a baseline cell is not `passed` or if an M58 cell is not `failed`). Between the two, `judge/pekko-baseline-report.tsv` was moved out of the tree and back, because `mutate.py`'s preflight treats an untracked `judge/` file as a dirty tree (a defect of the preflight, not of the rows) |
| Date | 2026-11-09 |

## Baselines (unmutated Pekko; `passed` in every cell)

| Scenario | Mode | Runs | Pass | Outcome |
|---|---|---|---|---|
| S07 | controlled | 1 | 1 | passed |
| S07 | concurrent | 30 | 30 | passed |
| S08 | controlled | 1 | 1 | passed |
| S08 | concurrent | 30 | 30 | passed |
| S11 | controlled | 1 | 1 | passed |
| S11 | concurrent | 30 | 30 | passed |
| S13 | controlled | 1 | 1 | passed |
| S13 | concurrent | 30 | 30 | passed |
| S21 | concurrent | 30 | 30 | passed |

S21 declares only concurrent. 0 harness error and 0 timeout in every cell: the bounded wait of the harness (#1496) did not decide any unmutated run.

## Mutations (0 harness error and 0 timeout in every row)

| Mutation | Scenario | Mode | Runs | Failed runs | Pass | Outcome | Rules fired | Expected rule hit |
|---|---|---|---|---|---|---|---|---|
| M58 | S07 | controlled | 1 | 1 | 0 | failed | G6.a, I1.a, I1.b, I2.a, REQ | true |
| M58 | S13 | controlled | 1 | 1 | 0 | failed | G6.a | true |
| M58 | S07 | concurrent | 30 | **25** | **5** | failed | G6.a, I1.a, I1.b, I2.a, REQ | true |
| M58 | S13 | concurrent | 30 | 30 | 0 | failed | G6.a | true |
| M5 | S07 | controlled | 1 | 1 | 0 | failed | I2.a | true |
| M5 | S07 | concurrent | 30 | 30 | 0 | failed | I2.a | true |
| M11 | S08 | controlled | 1 | 1 | 0 | failed | G6.b | true |
| M11 | S08 | concurrent | 30 | 30 | 0 | failed | G6.b | true |
| M12 | S13 | controlled | 1 | 1 | 0 | failed | G6.d | true |
| M12 | S13 | concurrent | 30 | 30 | 0 | failed | G6.d | true |
| M64 | S21 | concurrent | 30 | 30 | 0 | failed | G3.c | true |
| M6 | S10 | controlled | 1 | 1 | 0 | failed | I3.a | true |
| M6 | S11 | controlled | 1 | 1 | 0 | failed | I3.a, REQ | true |
| **M6** | **S10** | **concurrent** | 30 | 0 | 30 | **survived** | none | **false** |
| M6 | S11 | concurrent | 30 | 30 | 0 | failed | REQ | **false** (`I3.a` was designed) |
| M62 | S11 | controlled | 1 | 1 | 0 | failed | I3.a, I3.b | true |
| M62 | S11 | concurrent | 30 | 30 | 0 | failed | I3.b | true |

**M58 (the cells that decide C-08a and C-08c).** Both scenarios give `failed` by `G6.a` in both modes, with no timeout: the old `M58 S07 concurrent` row of the 2026-10 campaign was a `timeout` (its note: "no end after 10000 ms"). The cause is *inferred* from the harness before #1496, whose concurrent wait ran to the whole
deadline (`Judge.scala` `while (!sc.done(rec) && System.nanoTime() < end)`); with the bounded wait of #1496 the run closes complete and the comparator fails the missing event. In S07 concurrent **5 of 30 runs passed** even with the mutant (25 failed). That is a result of the
run, not a fault: the campaign is `failed` (at least one failing run and every run judgeable), and the clause is detected. Why 5 pass is *inferred*, not observed: the Go side shows the same shape for the same mutation
(`GMnonotify` × S07 concurrent, `go-mutation-report.tsv`, 23 of 30 on the earlier campaign), consistent with a race in which `w`'s watch is processed after `c` has stopped and the late `Watch` takes the dead-letter path, which
does not go through `tellWatchersWeDied`; no log line was read to confirm it.

**M6 × S10 × concurrent is the designed cell that did not give `failed`** (issue #1490): `survived`, 30 of 30 pass, no rule fired; M6 × S11 × concurrent fails by `REQ`, not by `I3.a`. The same rows as in #1489, now on Java 1.8 with
the bounded harness. In S10 and S11 the rule `I3.a` is declared `"modes": ["controlled"]`, so it is not evaluated in concurrent (`judge/comparator.py:506`); M6 is detected by `I3.a` in controlled in both scenarios.

## Checker

`scripts/check-judge-matrix.py` with these reports: exit 0.
- Pekko CURRENT: rows 17/70; designed detections 15/58; mutations with a CURRENT detection 7/32 (the 17 rows above; the other 53 are HISTORICAL).
- Pekko baseline CURRENT: rows 9/9; scenarios with every declared mode CURRENT: S07, S08, S11, S13, S21.
- Pekko V3 evidence (CURRENT detection on Java 1.8 plus a CURRENT baseline of the scenario in every declared mode, same judge inputs): M11/S08, M12/S13, M5/S07, M58/S07, M58/S13, M62/S11, M64/S21 (both modes where the scenario declares two), M6/S11 (controlled only). Assigning V3 to a row is an Owner decision and stays outside this record.

## Package

Outside the repository, in `goPekko-evidence/`; the tag `evidence/pekko-campaign-147b6b3` points at the full SHA above. The tar was built with `COPYFILE_DISABLE=1`.

| File | sha256 |
|---|---|
| `judge/mutation-report.tsv` | `4cccbaeda1c39420077c5f04c51ce04d9121300dee5771464d698d0acc9ec208` |
| `judge/pekko-baseline-report.tsv` | `21c4979360870ef58bc8e38c8cca325aa36b08320768eefa13d97f9e5fa0f0a3` |
| `pekko-campaign-147b6b3.tar.gz` (885 files; the tar is 136,152 bytes: both reports, `identity.txt`, `baseline/` logs and `sbt.log`, `mut/<M>/` logs and `sbt.log`, driver logs, `camp.py`, `camp.out`) | `f63faaaae4e32529bf990ee844b1f0ba6c9fe5010f67dd5c9771860cb499f96c` |
| `pekko-campaign-147b6b3.SHA256SUMS` (one line per file, paths relative to the root of the tar) | `071875cfe78f0381aa8cbd6da4e13ae257cc780fd7428794e11cf0206b373e3e` |
| `pekko-campaign-147b6b3.tar.gz.sha256` | checks the tar |

`python3 -I -B judge/verify_package.py goPekko-evidence pekko-campaign-147b6b3` prints `valid` (tar digest, only regular files with relative paths, `SHA256SUMS` lists exactly the 885 files and every digest matches, `identity.txt` names a 40-hex commit that starts with `147b6b3`).

## What it shows, and what it does not

- On Pekko v1.7.1 and Java 1.8, with the harness of `147b6b36`, the five scenarios pass in every declared mode, and each of M58, M5, M11, M12, M64 and M62 is caught by its designed rule in every designed cell; M6 is caught only in controlled.
- One platform (darwin/arm64), one campaign of 30 concurrent runs per cell, no V level by itself. It does not calibrate the judge (the 300 runs). The bounded wait is a harness change whose false-failure risk on a loaded machine is
  the one recorded in the review of #1496; 0 baseline cell hit it here.
