# Go judge campaign on linux/amd64 at 23389e3 (#1474) (provenance)

One full campaign of `judge/mutate_go.py` over the 21 designed Go mutations plus `GMbenign`, run by another session on linux/amd64 at the develop commit `23389e3a` (squash of #1481). It is the joint campaign of #1474: it includes GMnonotify (M58, #1478) with `awaitOrQuiesce` as the final wait of S07, S14, S18 and S22. Nothing here changes a scenario, a clause, the comparator, the classification of any outcome or any exclusion. No timeout, crash, harness error or abort was reinterpreted.

A first run of this commit on darwin/arm64 was cancelled by the Owner before it finished. It produced no report, no package and no tag, and is not evidence of anything.

## Identity

| Item | Value |
|---|---|
| Campaign commit (`go_commit`) | `23389e3a0596abf70bbcacb67238b6c5a15c29fa`, on `develop` |
| Root tree / `pekko-actor/` / `judge/` | `10f496306ad86ff33980db8a6ca0fda81703623b` / `52692dcb22484e0301b3c247a8d1b7edd15ab0e0` / `d3d7c3b9e4899c6e13c80db21c29d49a9a0f4412` |
| Workspace | clean (`dirty_files 0`); work directory and report outside the repository |
| Toolchain / platform | go1.27.2 linux/amd64, Python 3.13.18, Linux 6.18.44-fc-v310, `-race` |
| Command | `python3 -I judge/mutate_go.py <workdir> --runs 30 --report <report>` by the Owner's order (`-race` is built into the runner); the exact command line was not recorded in the package: `time.log` shows `exit 0` and every row carries `runs` 30 or 1 as designed |
| Time | 2026-11-08T01:46:18Z to 02:11:20Z, 25 min 02 s |

The three trees above equal `git rev-parse 23389e3a:pekko-actor`, `:judge` and `23389e3a^{tree}` in this repository.

## Result

550 rows over 22 mutations: 83 `failed`, 465 `survived`, 2 `excluded`; 0 `harness-error`, 0 `timeout`, 0 `invalid-campaign`. The 2 `excluded` are exactly `GM2 S22 concurrent` and `GMstop S22 concurrent`, 30 of 30 runs `gate-timeout`, unchanged from `restart-gaps-173/EXCLUSIONS.md` and from the earlier campaigns: S22 gave no classification change.
`scripts/check-judge-matrix.py` after installing the report byte for byte in `judge/go-mutation-report.tsv`: **exit 0**; **goPekko CURRENT on linux/amd64: detected 21/21 designed, benign survived 1/1**.

| Mutation | Cell | Result |
|---|---|---|
| GMnonotify | S07 controlled / concurrent (`G6.a`) | failed 1/1, failed 23/30; no harness error in S07, S14, S18 or S22 |
| GMnonotify | S08 controlled / concurrent | survived (no design there; `GMwatchdead` covers `G6.b`) |
| GMtwice | S07 concurrent (`I2.a`) | failed 9/30 |
| GMwatchdead | S08 controlled / concurrent (`G6.b`) | failed 1/1, failed 30/30 |

GMtwice failing 9 of 30 in S07 concurrent (16 of 30 on darwin in #1471) shows how timing-dependent that detection is; it is designed because it fails in at least one run per campaign, and that holds here.

| File | sha256 |
|---|---|
| report (`judge/go-mutation-report.tsv`) | `e77b0f5beb53afc69f3bde96de6927b56cb1f39a61232d06587e0d1c355b391c` |
| runner console log | `05d13a3edbf92b9ea421dcca7520c81c45450df33ecbae0beb85d8942070a195` |
| `identity.txt` | `e9facff899d1cf5e24244ad9db19324bc9158f04e916adcddbcfdd7f5975e141` |
| package `go-campaign-23389e3.tar.gz` (12,208,582 bytes) | `86d3388a26cadaadb5531e78df4664f002098cccb89090b71cf0c147860b18fc` |
| `go-campaign-23389e3.SHA256SUMS` (23,195 files) | `ca4cf45cf663ba7209f8d98110237d1af4ce413dd25bf5866ac8f01c3bb47b64` |

The package, its `.sha256` and the per-file `.SHA256SUMS` are in `/Users/pablogore/workspace/pablogore/goPekko-evidence/`, outside the repository and not committed. Every file of the package was verified against the `.SHA256SUMS` before installing (0 mismatches).

## What it shows, and what it does not

- With this tree the 15 Go scenarios pass on the unmodified code in both modes, and each of the 21 designed mutations is detected by its designed clause on linux/amd64.
- One platform, one campaign of 30 concurrent runs. Not calibrated. Not full `pekko-actor` parity.
- CURRENT holds only while `pekko-actor/` stays tree `52692dcb…` and the comparator, `mutate_go.py`, specs and toolchain are unchanged. This report replaces the darwin one of `7ef8d4e` (#1471) as the CURRENT record; `ba287d8` (#1464) and `7ef8d4e` stay retained by their tags.

## Surviving a squash

`go_commit` `23389e3a` is already a commit of `develop`, so it stays reachable. The lightweight tag `evidence/go-campaign-23389e3` also pins it.
