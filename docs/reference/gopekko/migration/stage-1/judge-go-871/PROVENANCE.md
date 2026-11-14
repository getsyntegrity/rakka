# Go judge campaign after #871 slice 2 (provenance)

One full campaign of `judge/mutate_go.py` on `develop` at `675e1442eb6aa81c3b8aa39c1843e63962a3afab`, run on the Owner's Mac. `develop` already holds everything #871 changed in `pekko-actor/`: #1528 (slice 1,
`DeadLetterSuppression`, `SuppressedDeadLetter` and `AllDeadLetters` classified in `System.publish`) and #1532 (slice 2, divergence 5a closed in `port.go`). The commits after #1532 (#1536 and #1537, slice A2 of #927)
change only `pekko-actor-typed/` and documents, not the Go core of this judge. Nothing here changes the judge, the scenarios, the comparator, the classification of any outcome or the exclusions. No timeout, crash,
harness error or abort was reinterpreted. It replaces the Go report of `e3456ab` (`judge-stage-close-896-889`), which stays as history (see "Earlier campaigns").

## Identity

| Item | Value |
|---|---|
| Campaign commit (`go_commit`) | `675e1442eb6aa81c3b8aa39c1843e63962a3afab`, `develop` (#1537), a clean worktree of `origin/develop` (0 dirty files) |
| Root tree / `pekko-actor/` / `judge/` | `0c3f4017eb30fb50cafc2009566eb169709f1390` / `a2219b3fd80d23141f0399de51334688a675f559` / `639b148068acec91591663a1d6c231b912bd1a3f` |
| Toolchain / platform | go1.27.2 (`.go-version`), darwin/arm64, 16 CPUs, Darwin 25.3.0, Python 3.14.5, `-race` |
| Command | `python3 -I judge/mutate_go.py $OUT/mut --runs 30 --report $OUT/mutation-report-go.tsv` (`$OUT` outside the repository) |
| Time | 2026-11-11 14:41:08 to 15:02:02 UTC (20 min 54 s) |
| Comparator digest (`comparator_sha256`) | `564072ea1438ea6f2dee5aec776990c8eeb3fdaaffa38edba4ff00a4b6d8c55e` (the same in every row, and the same as in the campaign of `e3456ab`) |

The `judge/` tree differs from the one of `e3456ab` (`65056c68...`) only because #1526 (`calibrate.py`, the Pekko-side calibration) landed in between; the Go side of the judge (`mutate_go.py`, `evaluate.py`,
`go_scope.py`, the comparator) is the one of the campaign before.

Machine conditions, recorded as they were: the Mac was not idle. The load average was 4.25 (1 minute) at the start and 4.78 at the end, on 16 CPUs, and a `crashpad_handler` process of the Android emulator
(not started by this campaign, not stopped by it) was using about one CPU. No other Go, test or campaign job of this work ran during the campaign.

## Result

22 of 22 mutations `met`, `unmet: none`, exit 0. 550 rows: 86 `failed`, 462 `survived`, 2 `excluded` (exactly `GM2 S22 concurrent` and `GMstop S22 concurrent`, as in `restart-gaps-173/EXCLUSIONS.md`).
0 `timeout`, 0 `invalid-campaign`. The only runs with `harness_error` are the 60 runs of those two excluded cells (30 each), the recorded `gate-timeout`; no other cell has one. 21 of 21 designed
mutations detected by their designed clause; `GMbenign` survived in all 25 rows. The report is installed byte for byte in `judge/go-mutation-report.tsv`. `scripts/check-judge-matrix.py` with it: exit 0;
goPekko **HISTORICAL** on darwin/arm64 detected 21/21; goPekko **CURRENT** on darwin/arm64 detected 21/21, benign 1/1.

### Differences from the campaign of `e3456ab` (550 rows each, same keys)

- **Outcome counts:** 86 `failed` and 462 `survived` now, 87 and 461 before; `excluded` 2 and 2.
- **One row changed outcome:** `GMwatchdead` x `S07` x `concurrent`: before `failed` 1 run of 30 (fired `G6.a,I1.a,I1.b,I2.a,REQ`), now `survived` 30 of 30. That cell is not designed for `GMwatchdead`
  (`expected_rules` is empty in both reports), so the mutation stays `met` by its designed cell; the earlier failure was an incidental fire in one run.
- **Five rows keep the outcome `failed` with different counts of runs (pass/fail):** `GM1` x `S02` (9/21 before, 1/29 now), `GMdup` x `S07` (25/5, 23/7), `GMoverlap` x `S02` (12/18, 10/20), `GMtwice` x `S07` (10/20, 12/18)
  and `GMnonotify` x `S07` (2/28, 0/30). Concurrent timing varies between campaigns; none of these cells is a designed cell that changed its result.
- No row appeared or disappeared, and no `fired_rules` or `expected_hit` of a designed cell changed.

| File | sha256 |
|---|---|
| report (`judge/go-mutation-report.tsv`) | `ef184d73048c54a53113770d99b082913f874c8ce1f485cc582c14e999a83256` |
| runner console log (`console.log`, with the line `exit=0` appended by the command) | `4fcb4a6969b703c4f0b1fae7ca89ad85645084dc6259ac20beea5c568f9168f8` (the same digest as the campaign of `e3456ab`: the log holds only the status lines) |
| `identity.txt` | `71308e5d800bc5f0f4fac655b9585135ae7ff8b857ac230ad9e0b2db1a723afa` |
| package `go-campaign-675e144.tar.gz` (23,372 files, 13,665,845 bytes) | `d2e1ea8816b3a6cea6ff484789a821121b26ac46fecb4e19ab0c3faee3ce081f` |
| `go-campaign-675e144.SHA256SUMS` (one line per file, paths relative to the root of the tar) | `0e952da3c7564a0e9cf104230fd74f3c723a79201d94c5a7be378fb91fa97ce6` |

The package was built with `COPYFILE_DISABLE=1`, checked with `python3 -I judge/verify_package.py <dir> go-campaign-675e144` (exit 0, `valid`), and with its `.sha256` and `SHA256SUMS` it is in
`/Users/pablogore/workspace/pablogore/goPekko-evidence/`, outside the repository and not committed. Besides the files of the earlier package (`console.log`, `end.txt`, `identity.txt`, `mut/`,
`mutation-report-go.tsv`, `start.txt`) it holds `load_start.txt` and `load_end.txt` (the `uptime` at the start and at the end).

## What it shows, and what it does not

- With the Go core after #871 slices 1 and 2, the 15 Go scenarios pass on the unmodified code and each designed mutation is detected by its designed clause on darwin/arm64. The core file digest
  (`pekko-actor/` without `*.md`, #1510) of `675e1442` is the one of `develop`, so the campaign is CURRENT on it.
- It restores the Go side of the V2 and V3 rows of the inventory, which had no force between #1528 and this campaign. One platform (darwin/arm64), one campaign of 30 concurrent runs; no V level by itself, and
  it assigns none: the V level of row C-10b is assigned by the Owner.
- It says nothing about the Pekko side (campaign `147b6b3`), which is unchanged, and nothing about `pekko-actor-typed`, which this judge does not cover.

## Earlier campaigns

- `e3456ab` (`judge-stage-close-896-889`, darwin/arm64): replaced by this campaign; its package and its tag `evidence/go-campaign-e3456ab` stay as history.

## Evidence tag

`go_commit` is on `develop`, so a squash does not drop it. By convention the tag `evidence/go-campaign-675e144` at the full SHA above is published for the package.
