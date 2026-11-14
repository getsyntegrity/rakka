# Go judge campaign after #1182 slice B and #821 slice 1 (provenance)

One full campaign of `judge/mutate_go.py` on `develop` at `dbdf15b601a1b1b3d27da4ac678e116a801305b5`, run on the Owner's Mac. `develop` holds everything the two code PRs after the previous campaign changed in
`pekko-actor/`: #1545 (`Context.System()` and the inputs a restart held published as dead letters when the replay meets `Stopped`, `actor.go`, `system.go` and their tests) and #1544 (`System.Scheduler()`,
`Cancellable`, `ScheduleOnce`, `scheduler.go`, `scheduler_once.go`, `system.go` and their tests). Nothing here changes the judge, the scenarios, the comparator, the classification of any outcome or the
exclusions. No timeout, crash, harness error or abort was reinterpreted. It replaces the Go report of `675e144` (`judge-go-871`), which stays as history (see "Earlier campaigns").

## Identity

| Item | Value |
|---|---|
| Campaign commit (`go_commit`) | `dbdf15b601a1b1b3d27da4ac678e116a801305b5`, `develop` (#1547), a clean worktree of `origin/develop` (0 dirty files) |
| Root tree / `pekko-actor/` / `judge/` | `a4f839206c5c2711d6b92eb6c9193d0bb39642e7` / `1df1a20fd6883f51d9a78e552dadd66da62554de` / `6d8946d4cbe0cfe7e1a469700779ef570882cb75` |
| Toolchain / platform | go1.27.2 (`.go-version`), darwin/arm64, 16 CPUs, Darwin 25.3.0, Python 3.14.5, `-race` |
| Command | `python3 -I judge/mutate_go.py $OUT/mut --runs 30 --report $OUT/mutation-report-go.tsv` (`$OUT` outside the repository) |
| Time | 2026-11-11 19:55:27 to 20:17:01 UTC (21 min 34 s) |
| Comparator digest (`comparator_sha256`) | `564072ea1438ea6f2dee5aec776990c8eeb3fdaaffa38edba4ff00a4b6d8c55e` (the same in every row, and the same as in the campaigns of `675e144` and `e3456ab`) |

Since the campaign of `675e144` the `pekko-actor/` tree changed by 6 `.go` files (404 added lines; the two PRs above) plus the README of the module (`pekko-actor/README.md`, +32/-6; documentation, not part of the Go core digest), and `judge/` changed only by #1541 (`mutate.py`,
`mutation-report.tsv`, `selftest_runner.py`: the expectation of M6 in the concurrent cells of the Pekko report). The Go side of the judge (`mutate_go.py`, `evaluate.py`, `go_scope.py`, the comparator) is the one of the
campaign before.

Machine conditions, recorded as they were: the Mac was not idle. The load average was 3.82 (1 minute) at the start and 3.48 at the end, on 16 CPUs. A `crashpad_handler` process of the Android emulator (pid 65736,
not started by this campaign, not stopped by it) used about one CPU throughout (`procs_start.txt` and `procs_end.txt` list the six busiest processes by CPU, as `ps aux` printed them, at the start and just after the end). At the start, besides the emulator process (100%), the
list shows `WindowServer` (48%) and four lighter processes (19%, 9%, 7% and 6%). At the end it shows the emulator process (99%), a root `CoreServices` process (pid 353, 93%) and four more `CoreServices`/metadata
processes (pids 57483, 57486, 57475 and 57168, 22% to 27% each; they started at 17:16 and 17:18 local time, that is, in the last minutes of the campaign or just after it: `procs_end.txt` was taken right after the
campaign ended, together with the packaging, not at the instant of its last run). No other Go, test or campaign job of this work ran during the campaign.

## Result

22 of 22 mutations `met`, `unmet: none`, exit 0. 550 rows: 86 `failed`, 462 `survived`, 2 `excluded` (exactly `GM2 S22 concurrent` and `GMstop S22 concurrent`, as in `restart-gaps-173/EXCLUSIONS.md`).
0 `timeout`, 0 `invalid-campaign`. The only runs with `harness_error` are the 60 runs of those two excluded cells (30 each), the recorded `gate-timeout`. 21 of 21 designed mutations detected by their designed
clause; `GMbenign` survived in all 25 rows. The report is installed byte for byte in `judge/go-mutation-report.tsv`. `scripts/check-judge-matrix.py` with it: exit 0; goPekko **HISTORICAL** on darwin/arm64
detected 21/21; goPekko **CURRENT** on darwin/arm64 detected 21/21, benign 1/1.

### Differences from the campaign of `675e144` (550 rows each, same keys)

- **No row changed its outcome.** 86 `failed`, 462 `survived` and 2 `excluded` in both. No row appeared or disappeared; `expected_rules`, `expected_hit`, `fired_rules` and `benign` are the same in all 550 rows.
- **Five rows keep their outcome with different counts of runs (pass/fail)**, all in the `concurrent` mode: `GM1` x `S02` (1/29 before, 10/20 now), `GMdup` x `S07` (23/7, 21/9), `GMoverlap` x `S02` (10/20, 11/19),
  `GMtwice` x `S07` (12/18, 11/19) and `GMtwice` x `S22` (10/20, 6/24). Concurrent timing varies between campaigns; none of these cells changed its outcome, and no mutation went undetected.

| File | sha256 |
|---|---|
| report (`judge/go-mutation-report.tsv`) | `2ac7d137174c05ffa0ef843267fd5abedf4a823d46a6dce02cff50da724c3864` |
| runner console log (`console.log`, with the line `exit=0` appended by the command) | `4fcb4a6969b703c4f0b1fae7ca89ad85645084dc6259ac20beea5c568f9168f8` (the same digest as the two earlier campaigns: the log holds only the status lines) |
| `identity.txt` | `724f644f8b16716798485c299dfe0a7d7820bb0928201eea8ceebe3dbe7d2697` |
| package `go-campaign-dbdf15b.tar.gz` (23,462 files, 13,798,873 bytes) | `8bd06c47ee0ad6f26c2232592d1f5e7ab68614ec49ac3bf4e616ca7622a301f5` |
| `go-campaign-dbdf15b.SHA256SUMS` (one line per file, paths relative to the root of the tar) | `e24e1aac930e964d3721a5ed67a3bb00986529c600e09b78d7f442188ce92364` |

The package was built with `COPYFILE_DISABLE=1`, checked with `python3 -I judge/verify_package.py <dir> go-campaign-dbdf15b` (exit 0, `valid`), and with its `.sha256` and `SHA256SUMS` it is in
`/Users/pablogore/workspace/pablogore/goPekko-evidence/`, outside the repository and not committed. Besides the files of the earlier package (`console.log`, `end.txt`, `identity.txt`, `load_start.txt`,
`load_end.txt`, `mut/`, `mutation-report-go.tsv`, `start.txt`) it holds `procs_start.txt` and `procs_end.txt` (the six busiest processes by CPU at the start and just after the end).

## What it shows, and what it does not

- With the Go core after #1545 and #1544, the 15 Go scenarios pass on the unmodified code and each designed mutation is detected by its designed clause on darwin/arm64. The core file digest (`pekko-actor/`
  without `*.md`, #1510) of `dbdf15b6` is the one of `develop`, so the campaign is CURRENT on it.
- It restores the Go side of the V2 and V3 rows of the inventory, which had no force between #1545 and this campaign. One platform (darwin/arm64), one campaign of 30 concurrent runs; no V level by itself, and it
  assigns none: the V level of row C-10b is assigned by the Owner.
- It says nothing about the Pekko side (campaign `147b6b3`), which is unchanged, and nothing about `pekko-actor-typed`, which this judge does not cover. The scheduler is not a scenario of this judge: the new
  code of #1544 is exercised by the Go scenarios only through the unchanged paths, so this campaign shows it did not disturb them, not that it matches Pekko.

## Earlier campaigns

- `675e144` (`judge-go-871`, darwin/arm64): replaced by this campaign; its package and its tag `evidence/go-campaign-675e144` stay as history.

## Evidence tag

`go_commit` is on `develop`, so a squash does not drop it. By convention the tag `evidence/go-campaign-dbdf15b` at the full SHA above is published for the package.
