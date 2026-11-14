# Go judge campaign at the close of the stage (#896, #889) (provenance)

One full campaign of `judge/mutate_go.py` on `develop` at `e3456abd36541a1865323d128ca8f1b304ea5daa` (squash of #1519, the last PR of the stage), run on the Owner's Mac. `develop` already holds
everything the stage changed in `pekko-actor/`: #1513 (slice 1a of #896, the sender of `DeadLetter`), #1517 (slice 1b, `System.DeadLetters()` and the substitution), #1519 (slice 2, README and a test) and
the anchor repair of #1515. Nothing here changes the judge, the scenarios, the comparator, the classification of any outcome or the exclusions. No timeout, crash, harness error or abort was reinterpreted.
It replaces the Go report of `22b7e71` (Linux), which stays as history (see "Earlier campaigns").

## Identity

| Item | Value |
|---|---|
| Campaign commit (`go_commit`) | `e3456abd36541a1865323d128ca8f1b304ea5daa`, `develop` (#1519), a clean worktree of `origin/develop` (0 dirty files) |
| Root tree / `pekko-actor/` / `judge/` | `ce56360e554120b2b740feacea887139ebcbd09c` / `95c10953c6ce258849c423e69d26de786d6fb2ed` / `65056c6835cfcee45b6eeaf1b72b2dca9c4a5e3c` |
| Toolchain / platform | go1.27.2 (`.go-version`), darwin/arm64, 16 CPUs, Darwin 25.3.0, Python 3.14.5, `-race` |
| Command | `python3 -I judge/mutate_go.py $OUT/mut --runs 30 --report $OUT/mutation-report-go.tsv` (`$OUT` outside the repository) |
| Time | 2026-11-10 22:51:00 to 23:12:20 UTC (21 min 20 s) |
| Comparator digest (`comparator_sha256`) | `564072ea1438ea6f2dee5aec776990c8eeb3fdaaffa38edba4ff00a4b6d8c55e` (the same in every row) |

## Result

22 of 22 mutations `met`, `unmet: none`, exit 0. 550 rows: 87 `failed`, 461 `survived`, 2 `excluded` (exactly `GM2 S22 concurrent` and `GMstop S22 concurrent`, as in `restart-gaps-173/EXCLUSIONS.md`).
0 `timeout`, 0 `invalid-campaign`. The only runs with `harness_error` are the 60 runs of those two excluded cells (30 each), the recorded `gate-timeout`; no other cell has one. 21 of 21 designed
mutations detected by their designed clause; `GMbenign` survived in all 25 rows. Same counts as the campaigns of `ef0de70` and `22b7e71`. The report is installed byte for byte in
`judge/go-mutation-report.tsv`. `scripts/check-judge-matrix.py` with it: exit 0; goPekko **HISTORICAL** on darwin/arm64 detected 21/21; goPekko **CURRENT** on darwin/arm64 detected 21/21, benign 1/1.

| File | sha256 |
|---|---|
| report (`judge/go-mutation-report.tsv`) | `f4d674789b1086242e2d35460d43038933bad407e712a19a50acfa16efef4c0a` |
| runner console log (`console.log`, with the line `exit=0` appended by the command) | `4fcb4a6969b703c4f0b1fae7ca89ad85645084dc6259ac20beea5c568f9168f8` |
| `identity.txt` | `2bcc0fdfe6df8107fe833b5a8bbfc4df9f9374f60c4b13412a5b7334baceb957` |
| package `go-campaign-e3456ab.tar.gz` (23,282 files, 13,536,835 bytes) | `adff305a116c363d92821ed7c7b5699ece00d98dd718485ccdaef670d915364e` |
| `go-campaign-e3456ab.SHA256SUMS` (one line per file, paths relative to the root of the tar) | `02982ea1868e4a6e8a440595580a8da5bc362415a2737b748e89e1b4e1d1f70a` |

The console log differs from the earlier campaigns' digest only by the appended `exit=0` line. The package was built with `COPYFILE_DISABLE=1`, checked with `python3 -I judge/verify_package.py <dir> go-campaign-e3456ab`
(exit 0, `valid`), and with its `.sha256` and `SHA256SUMS` it is in `/Users/pablogore/workspace/pablogore/goPekko-evidence/`, outside the repository and not committed.

## What it shows, and what it does not

- With the Go core of the whole stage (slices 1a, 1b and 2 of #896; slices A and B of #889 live in `pekko-actor-typed`, not in the Go core of this judge), the 15 Go scenarios pass on the unmodified code and each
  designed mutation is detected by its designed clause on darwin/arm64. The core file digest (`pekko-actor/` without `*.md`, #1510) of `e3456abd` is the one of `develop`, so the campaign is CURRENT on it.
- It restores the Go side of the V2 and V3 rows of the inventory until the next change of the Go core. One platform (darwin/arm64), one campaign of 30 concurrent runs; no V level by itself.
- It says nothing about the Pekko side (campaign `147b6b3`), which is unchanged.

## Earlier campaigns

- `ef0de70` (`judge-s13-1484`): its package was not archived in `goPekko-evidence/`; it stayed only in the session that ran it. Its report is history, replaced by this campaign.
- `22b7e71` (`judge-sender-896`, Linux): its commit is not in `develop` (#1515 was squashed). Its package is in `goPekko-evidence/` (copied there on 2026-11-10 while #1515 was reviewed) and its tag
  `evidence/go-campaign-22b7e71` was published on 2026-11-10 at the Owner's request, before the decision that it should not be. The tag is kept until the Owner says to delete it. Its report is history, replaced by this campaign.

## Evidence tag

`go_commit` is on `develop`, so a squash does not drop it. By convention the tag `evidence/go-campaign-e3456ab` at the full SHA above is published for the package.
