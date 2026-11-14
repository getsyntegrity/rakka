# Go judge campaign after DeadLetter gained its sender (#896, slice 1a) (provenance)

One full campaign of `judge/mutate_go.py` on `22b7e71`, the anchor repair of GM6a, GM6b and GM62 on top of `develop` `520fc48` (#1513, slice 1a of #896: `DeadLetter.Sender`). #1513 changed the dead-letter
publish sites of `ref.go` and `stop.go`, so those three anchors no longer occurred and `scripts/check-judge-matrix.py` failed on `develop` (the anchor check of #1464). `22b7e71` changes only those
anchors (and the selftest fixture that doubles one of them): each mutation breaks the same contract as before on the current code. Nothing else changes: not the scenarios, the comparator, the
classification of any outcome or the exclusions. No timeout, crash, harness error or abort was reinterpreted.

## Identity

| Item | Value |
|---|---|
| Campaign commit (`go_commit`) | `22b7e71d00a36b09a437ed95210349a47345b5d0`, branch `fix/1513-go-anchors-sender` (parent `520fc48` on `develop`) |
| Root tree / `pekko-actor/` / `judge/` | `90a90ccf58415a468a83102a0b472454fa3d6437` / `646134c2f9b66928d778b9818d362ff5fedcea07` / `10aec15f9d5c9dd529174d71a12b1c2221e8572f` |
| Workspace | clean (0 dirty files in `judge/` and `pekko-actor/`); work directory and report outside the repository |
| Toolchain / platform | go1.27.2 (`.go-version`), built from the `go1.27.2` tag of github.com/golang/go (proxy.golang.org not reachable); linux/amd64, 2 CPUs, Python 3.13.18, `-race` |
| Modules | `golang.org/x/time v0.14.0` from a local file proxy built from its `v0.14.0` tag; `go.sum` verified it (unchanged) |
| Command | `python3 -I judge/mutate_go.py $OUT/mut --runs 30 --report $OUT/mutation-report-go.tsv` |
| Time | 2026-11-10 17:39:59 to 18:04:40 UTC (24 min 41 s) |
| Comparator digest (`comparator_sha256`) | `8a4474813096f96e2c5d5c2681a720b7c6fe246a4793813a5833ef2b3233a4aa` |

## Result

22 of 22 mutations `met`, `unmet: none`, exit 0. 550 rows: 87 `failed`, 461 `survived`, 2 `excluded` (exactly `GM2 S22 concurrent` and `GMstop S22 concurrent`, as in `restart-gaps-173/EXCLUSIONS.md`);
0 `harness-error`, 0 `timeout`, 0 `invalid-campaign`. 21 of 21 designed mutations detected, `GMbenign` survived. Same counts as the campaign of `ef0de70` (`judge-s13-1484/PROVENANCE.md`), which stays as
history. The report is installed byte for byte in `judge/go-mutation-report.tsv`. `scripts/check-judge-matrix.py` with it: exit 0; goPekko **HISTORICAL** detected 21/21, benign 1/1; goPekko **CURRENT** on
linux/amd64 detected 21/21, benign 1/1 (0/21 on `develop` since #1502 changed `pekko-actor/README.md`, then failing since #1513).

| File | sha256 |
|---|---|
| report (`judge/go-mutation-report.tsv`) | `428874a567a38e62d076423298693ff7a895bc6c0cbdc1b86be2c1aa2e11d1b5` |
| runner console log | `05d13a3edbf92b9ea421dcca7520c81c45450df33ecbae0beb85d8942070a195` |
| `identity.txt` | `38acab3782a9c7f8ed216b75219ab6eb4fa30b1cb8e7a283a7d95555c6ff725e` |
| package `go-campaign-22b7e71.tar.gz` (23,217 files, 12,334,598 bytes) | `0208389a273f054df71a947ce6cb80cdb88af95e32bc9157a59a8a846dda3edf` |
| `go-campaign-22b7e71.SHA256SUMS` (one line per file, paths relative to the root of the tar) | `efef6a4c342cd18c6176039f549da01b07e1dde4ee0087218e0112e413b37a38` |

The console log has the same digest as the earlier campaigns that met every mutation (the runner prints one fixed line per mutation). The package, its `.sha256` and `go-campaign-22b7e71.SHA256SUMS` go to
`/Users/pablogore/workspace/pablogore/goPekko-evidence/`, outside the repository and not committed.

## What it shows, and what it does not

- With the Go core of slice 1a of #896, the 15 Go scenarios pass on the unmodified code, and each designed mutation is detected by its designed clause on linux/amd64.
- It restores the Go side of the V2 and V3 rows of the inventory until the next change of the Go core (slice 1b of #896). One platform, one campaign of 30 concurrent runs; no V level by itself.

## Evidence tag

A squash drops `22b7e71` from `develop`, so the tag `evidence/go-campaign-22b7e71` at the full SHA above retains it; it is published from the Owner's machine (this session cannot push tags), or the PR is
merged with a merge commit.

## Note (2026-11-10): the package and the tag of this campaign

The commit `22b7e71` is not in `develop` (the squash of #1515 dropped it), so by the Owner's decision its tag is not to be published. The package is in `goPekko-evidence/` (copied there while #1515 was reviewed) and the tag `evidence/go-campaign-22b7e71` had already been published at the Owner's earlier request; it is kept until the Owner says to delete it. This report is history, replaced by the campaign `e3456ab` (`judge-stage-close-896-889`).
