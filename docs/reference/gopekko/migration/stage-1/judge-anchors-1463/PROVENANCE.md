# Go judge campaign after the anchor repair (#1463) (provenance)

One full campaign of `judge/mutate_go.py`, the first on Linux. It runs on the commit that repairs the stale anchors (`542c3df`) and restores the invariant that keeps GMoverlap free of data races
(`ba287d8`). Nothing here changes the scenarios, the comparator, the classification of any outcome or the exclusions: `542c3df` changes only the anchors (the text each mutation edits), so each
mutation breaks the same contract as before on the current code. No timeout, crash, harness error or abort was reinterpreted.

## Earlier attempts, kept apart

| Commit (not on `develop`) | What happened | Used here |
|---|---|---|
| `2c2ddcb` | Stopped by hand after 10 mutations: its GM6a edit was identical to its anchor, so GM6a changed nothing and survived. Fixed by amending (`542c3df`); `_edit` now refuses an edit that changes nothing. | No |
| `542c3df` | Complete, exit 1: 18 of 19 `met`, `unmet: GMoverlap`. GMoverlap was one `harness-error` row: `go test -race` reported a DATA RACE between the extra worker's `callReceive` (`cell.go:351`, the read of `failedMsg.present`) and `clearCurrent` (`cell.go:326`), which #1270 made the finalization call and which wrote `failedMsg` unconditionally. Its logs, judged by hand as an observation only, showed S21 concurrent failing 30 of 30 with `G3.c`; by the rules that is not a detection. | No |

`ba287d8` makes `clearCurrent` write `failedMsg` only when there is one, as `callReceive` already does for the same reason (its comment: a mutation that overlaps two Receives must not race on the field).
The behavior is unchanged (the field ends zero either way); both branches are covered by the unit tests. This campaign ran on it.

## Identity

| Item | Value |
|---|---|
| Campaign commit (`go_commit`) | `ba287d81d86d5f3dd99a0671089dae76f85e8b1d`, branch `fix/1463-go-mutation-anchors` |
| Root tree / `pekko-actor/` / `judge/` | `0b06cd0fc72230b5d916164eecba160a6e272913` / `2b56faa72260eabee29c9da39f7cbf672f0fb5aa` / `7edcd8b5e71f4c2c7a537a2d7b62fc52b8af6374` |
| Workspace | clean (0 dirty files in `judge/` and `pekko-actor/`); work directory and report outside the repository |
| Toolchain / platform | go1.27.2 (`.go-version`), built from the `go1.27.2` tag of github.com/golang/go because proxy.golang.org was not reachable; linux/amd64, 2 CPUs, Python 3.13.18, `-race` |
| Modules | `golang.org/x/time v0.14.0` served from a local file proxy built from the `v0.14.0` tag of github.com/golang/time; `go.sum` verified it (no `-mod=mod`, `go.sum` unchanged) |
| Command | `python3 -I judge/mutate_go.py $OUT/mut --runs 30 --report $OUT/mutation-report-go.tsv` |
| Time | 2026-11-07 13:53:08 to 14:17:06 UTC (23 min 58 s) |
| Comparator digest (`comparator_sha256`) | `fb52ee8620921ac41bafa3aaeeea373cedd4b3a0b976973e3f7123497f07d831` (differs from the darwin campaign `6742250` because `judge/mutate_go.py` is one of its inputs) |

## Result

19 of 19 mutations `met`, `unmet: none`, exit 0. 475 rows: 71 `failed`, 402 `survived`, 2 `excluded` (exactly `GM2 S22 concurrent` and `GMstop S22 concurrent`, as in
`restart-gaps-173/EXCLUSIONS.md`); 0 `harness-error`, 0 `timeout`, 0 `invalid-campaign`. 18 of 18 designed mutations detected, `GMbenign` survived. The same counts as the darwin campaign on
`6742250`. The report is installed byte for byte in `judge/go-mutation-report.tsv`. `scripts/check-judge-matrix.py` after installing it: exit 0; **HISTORICAL** detected 18/18, benign 1/1; **CURRENT**
detected 18/18, benign 1/1 (0/18 before). CURRENT holds only while `pekko-actor/` stays tree `2b56faa7…` and the other inputs are unchanged.

| File | sha256 |
|---|---|
| report (`judge/go-mutation-report.tsv`) | `d847ca4b5e1844c7d7d8618fc637e7f5b6900bd0d689dfe9cd146d854321b4e6` |
| runner console log | `79a97b332130dea1f7889f47e7aa89c11154a2fa9c5c654d2d9bcf5b257bf320` |
| `identity.txt` (commit, trees, toolchain, CPUs, dirty count) | `12cda9344db2232b5f5a81cb78c2b9cdd79e9edd81c7a90b94afa64ea5003bb2` |
| package `go-campaign-ba287d8.tar.gz` (19,973 log and work files, 10,432,772 bytes) | `fd3988f64b08ca30adde355a11bade94fb92e79b06ef88e7881db3b31c20c442` |

The package, its `.sha256` and a per-file `go-campaign-ba287d8.SHA256SUMS` are in `/Users/pablogore/workspace/pablogore/goPekko-evidence/`, outside the repository and not committed.

## What it shows, and what it does not

- With this tree the 15 Go scenarios pass on the unmodified code, and each designed mutation is detected by its designed clause on linux/amd64.
- It accredits only the existing matrix: one platform, one campaign of 30 concurrent runs. S19 and S21 keep their calibrated floors, which this campaign does not replace. Not full `pekko-actor` parity.

## Surviving a squash

`CURRENT` needs `go_commit` to be reachable. A squash drops it, so the tag `evidence/go-campaign-ba287d8` at the full SHA above must retain it, or the PR is merged with a merge commit. The Owner
authorized publishing it (2026-11-07) and the tag was published on 2026-11-07, a lightweight tag that never moves; `git ls-remote origin refs/tags/evidence/go-campaign-ba287d8`
returns `ba287d81d86d5f3dd99a0671089dae76f85e8b1d`. Do not change `pekko-actor/` or the other campaign inputs
between this commit and the merge.
