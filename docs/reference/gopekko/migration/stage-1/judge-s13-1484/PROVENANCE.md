# Go judge campaign after the S13 change (#1484) (provenance)

One full campaign of `judge/mutate_go.py` on `develop` at `ef0de70` (#1487: S13 replicates `DeathWatchSpec.scala:195-211` on both harnesses, GMnonotify cells in S13). Nothing here changes the judge, the
scenarios, the comparator, the classification of any outcome or the exclusions. No timeout, crash, harness error or abort was reinterpreted. It restores the checker on `develop`, which failed since #1487
because the recorded GMnonotify rows no longer matched its definition.

**The Pekko side of S13 is not validated yet.** #1487 changed a scenario shared with Pekko, and the Pekko harness was reviewed by reading only. If the Pekko campaign of S13 (baseline in both modes, and
M12) forces a change of the S13 spec or of either harness, this campaign is stale and runs again.

## Identity

| Item | Value |
|---|---|
| Campaign commit (`go_commit`) | `ef0de70238fee3dd2901d7b9b78bc2a5d4b7c369` (`develop`, squash of #1487), reachable without an evidence tag |
| Root tree / `pekko-actor/` / `judge/` | `0807bddb42795ab144aab46648e365cdf9118534` / `baf7315d6d22a61306f5f5fdc66e88c2fdf45040` / `774c9c56d55d3d2df35a48e4f0cd28d9db65866d` |
| Workspace | clean (0 dirty files in `judge/` and `pekko-actor/`); work directory and report outside the repository |
| Toolchain / platform | go1.27.2 (`.go-version`), built from the `go1.27.2` tag of github.com/golang/go (proxy.golang.org not reachable); linux/amd64, 2 CPUs, Python 3.13.18, `-race` |
| Modules | `golang.org/x/time v0.14.0` from a local file proxy built from its `v0.14.0` tag; `go.sum` verified it (unchanged) |
| Command | `python3 -I judge/mutate_go.py $OUT/mut --runs 30 --report $OUT/mutation-report-go.tsv` |
| Time | 2026-11-08 18:37:37 to 19:04:09 UTC (26 min 32 s) |
| Comparator digest (`comparator_sha256`) | `006cd61bba3eff34f6ad16122251720fea71bd2e94d6fb545e872944a2974b65` |

## Result

22 of 22 mutations `met`, `unmet: none`, exit 0. 550 rows: 87 `failed`, 461 `survived`, 2 `excluded` (exactly `GM2 S22 concurrent` and `GMstop S22 concurrent`, as in `restart-gaps-173/EXCLUSIONS.md`);
0 `harness-error`, 0 `timeout`, 0 `invalid-campaign`. 21 of 21 designed mutations detected, `GMbenign` survived. GMnonotify fails S13 by `G6.a` in both modes (the final Terminated after the restarts) and
S07 as before. One undesigned cell changed too: GMdup (every second message queued twice) now fails S13 by `REQ` in both modes, where it survived on `23389e3`, because the new S13 sends more
messages to `c` and a duplicate breaks the required sequence. It is a valid extra detection, not a designed cell, and nothing in the classification relies on it. The report is installed byte for byte in `judge/go-mutation-report.tsv`; it replaces the report of `23389e3` (`judge-joint-1474/PROVENANCE.md`), which stays as history.
`scripts/check-judge-matrix.py` with it: exit 0; goPekko **HISTORICAL** detected 21/21, benign 1/1; goPekko **CURRENT** on linux/amd64 detected 21/21, benign 1/1. Pekko CURRENT is unchanged: this campaign is Go only.

| File | sha256 |
|---|---|
| report (`judge/go-mutation-report.tsv`) | `5011055ccfdeec63b4b04f9951c61cee2753afd26c579ab5f5298a8b21990c00` |
| runner console log | `05d13a3edbf92b9ea421dcca7520c81c45450df33ecbae0beb85d8942070a195` |
| `identity.txt` | `63bf09d9e85a71b62a1f09aea359ea1b4a0914e9243237da8418ad3eae17fd15` |
| package `go-campaign-ef0de70.tar.gz` (23,195 files, 12,303,284 bytes) | `9d021dbc2a69b55dc2080b3b14b9455807a9014d6530091b2f8153ac8fa96c75` |
| `go-campaign-ef0de70.SHA256SUMS` (one line per file, paths relative to the root of the tar) | `77353a4273f2ef5404478e5caf65ba246f0737efd5b841f613df12d6bc15f3ea` |

The console log has the same digest as the campaign of `23389e3`: the runner prints one fixed line per mutation and both campaigns met every mutation. The package, its `.sha256` and
`go-campaign-ef0de70.SHA256SUMS` go to `/Users/pablogore/workspace/pablogore/goPekko-evidence/`, outside the repository and not committed.

## What it shows, and what it does not

- With this tree the 15 Go scenarios pass on the unmodified code, and each designed mutation is detected by its designed clause on linux/amd64.
- It does not show that the new S13 behaves the same on Pekko; that is the Pekko campaign of S13. One platform, one campaign of 30 concurrent runs; no V level by itself.

## Evidence tag

`go_commit` is on `develop`, so a squash does not drop it. By convention the tag `evidence/go-campaign-ef0de70` at the full SHA above is published from the Owner's machine (this session cannot push tags).

## Note (2026-11-10): the package of this campaign

The package of this campaign was not archived in `goPekko-evidence/`; it stayed only in the session that ran it. Its report is history, replaced by the campaign `e3456ab` (`judge-stage-close-896-889`).
