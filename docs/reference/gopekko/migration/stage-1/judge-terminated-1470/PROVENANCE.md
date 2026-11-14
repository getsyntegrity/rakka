# Go judge campaign with the Terminated mutations (#1470) (provenance)

One full campaign of `judge/mutate_go.py` after two Go mutations for the basic delivery of `Terminated` were added: GMtwice (Pekko M5, S07 concurrent `I2.a`) and GMwatchdead (M11, S08 `G6.b`). They break the contracts of
`DeathWatchSpec.scala:135` and `:145` (`upstream/pekko` v1.7.1). Nothing here changes the scenarios, the clauses, the comparator, the classification of any outcome or the exclusions. No timeout, crash, harness error or abort was reinterpreted.

## Not designed, and why

| Pekko analog | Go result | Decision |
|---|---|---|
| M58 (no notification on stop) | S07 ends as a harness error: "controlled run is quiescent before its completion witness". Never a detection. | not designed |
| M10 (ignore `Unwatch`) | survives in S09: the Go `Terminated` is enqueued at the tail, behind the `settle` message the scenario sends | not designed |
| M8 (`Terminated` ahead of an earlier message) | detected in S07 concurrent `I1.a` in 3 of 30 runs; the order rule is not asserted by a Pekko spec | not designed |
| M9 (notify before the final `PostStop`) | detected in S18 concurrent `I1.b` in 30 of 30; not asserted by a Pekko spec | not designed |

## Identity

| Item | Value |
|---|---|
| Campaign commit (`go_commit`) | `7ef8d4e3785ee0aab6a572e507d9ec8c7153ac17`, branch `test/go-mutations-terminated` |
| Root tree / `pekko-actor/` / `judge/` | `bac188f36c9f98212519c9bbc0e5c1f08df142c8` / `2b56faa72260eabee29c9da39f7cbf672f0fb5aa` / `816733d4fbbfb9feebc951113e4a6e0c02bad2d2` |
| Workspace | clean (0 dirty files in `judge/` and `pekko-actor/`); work directory and report outside the repository |
| Toolchain / platform | go1.27.2 darwin/arm64, Apple M4 Max, 16 CPUs, macOS 26.3.1, Python 3.14.5, `-race` |
| Command | `python3 -I judge/mutate_go.py $OUT/mut --runs 30 --report $OUT/mutation-report-go.tsv` |
| Time | 52 min 26 s |

## Result

21 of 21 mutations `met`, `unmet: none`, exit 0. 525 rows: 75 `failed`, 448 `survived`, 2 `excluded` (exactly `GM2 S22 concurrent` and `GMstop S22 concurrent`, as in `restart-gaps-173/EXCLUSIONS.md`);
0 `harness-error`, 0 `timeout`, 0 `invalid-campaign`. 20 of 20 designed mutations detected, `GMbenign` survived. GMtwice failed 16 of 30 runs in S07 concurrent (the report row; an earlier trial showed 19 of 30, which this text had by mistake) (S07 controlled survives: it has one run and no concurrency);
it also fired `I2.a` in S22 concurrent, a cell with no design. The report is installed byte for byte in `judge/go-mutation-report.tsv`. `scripts/check-judge-matrix.py` after installing it: exit 0; **CURRENT on darwin/arm64** detected 20/20,
benign 1/1. This report replaces the linux/amd64 one of `ba287d8` (#1464), which stays retained by its tag; there is no Linux report for the new inputs. CURRENT holds only while `pekko-actor/` stays tree `2b56faa7…` and the other inputs are unchanged.

| File | sha256 |
|---|---|
| report (`judge/go-mutation-report.tsv`) | `cbf8c81864cb1702f86c9202c6456b9027cd08e5e96f451af90c5cbdc6235be4` |
| runner console log | `81c668e36f8b32813b65cff09009ca5f487bf3e7d7879231e62849c44c879ad7` |
| `identity.txt` | `eac0e9bb6e3e4906e60c03d581eb902181c25ee5ad1d6eedaf01a9027a321c7f` |
| package `go-campaign-7ef8d4e.tar.gz` (22,076 files, 12,507,223 bytes) | `f050f67342258cfafeba85073ed85dd99121516f4d4e770f828c3b32a82015a9` |

The package, its `.sha256` and a per-file `go-campaign-7ef8d4e.SHA256SUMS` are in `/Users/pablogore/workspace/pablogore/goPekko-evidence/`, outside the repository and not committed.

## What it shows, and what it does not

- With this tree the 15 Go scenarios pass on the unmodified code, and each designed mutation is detected by its designed clause on darwin/arm64.
- One platform, one campaign of 30 concurrent runs. Not calibrated. Not full `pekko-actor` parity.

## Surviving a squash

`CURRENT` needs `go_commit` to be reachable. A squash drops it, so the tag `evidence/go-campaign-7ef8d4e` (lightweight, at the full SHA above, published 2026-11-07) retains it.
