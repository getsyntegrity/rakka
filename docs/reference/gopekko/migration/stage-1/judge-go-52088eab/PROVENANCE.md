# Go judge campaign on develop 52088eab (provenance)

One full campaign of `judge/mutate_go.py` (30 concurrent runs, 1 controlled, `-race`) on `develop` at `52088eabea976414e2b1df26d2f37625ea2510ee`, the integration commit of U1 (#1562, `actor.Dropped`), run on the Owner's Mac as the renewal required before U2 of #999. It replaces the Go report of `815fc984` (`judge-go-815fc984`), which stays as history. Nothing here changes the judge, scenarios, comparator, mutations, expectations, classifications or exclusions. Refs #999, Refs #906.

## Identity

| Field | Value |
|---|---|
| **go_commit** / **tree** (`pekko-actor`) | `52088eabea976414e2b1df26d2f37625ea2510ee` / `2f2dfcce991a40545a5d8ae9abac3b3a2b43e788` |
| **comparator_sha256** (runner definition) | `564072ea1438ea6f2dee5aec776990c8eeb3fdaaffa38edba4ff00a4b6d8c55e` (as at 815fc984); `judge/comparator.py` file sha256 `7a7a1d7b…fc024` |
| toolchain, platform | `go1.27.2` (`.go-version`), `darwin/arm64`, macOS 26.3.1, Apple M4 Max, 16 CPUs, Python 3.14.5 |
| runs, start/end (UTC), exit | 30 concurrent + 1 controlled; `2026-11-13T23:18:52Z` to `23:40:15Z`; exit 0 |
| tree used | clean worktree at `52088eab`, submodules initialized (pekko `9a54d12`, kit `cf91c9d`, goakt `00da811`) |

Command, from the worktree root: `python3 -I judge/mutate_go.py <workdir> --runs 30 --report <workdir>/go-mutation-report.tsv`. Input hashes (runner, evaluator, scope, scenarios, mutations, test file, `go.work`) are in `run-info/identity-pre.txt` of the package. `judge/` is byte-identical to 815fc984 except the evidence files.

## Results

22 of 22 mutations `met`, `unmet: none`. 550 rows (22 mutations x 25 cells): 87 `failed`, 461 `survived`, 2 `excluded`; 0 `timeout`, 0 `invalid-campaign`. 21 of 21 designed mutations (38 designed rows) `failed` with every designed clause fired; `GMbenign` survived 25 of 25 rows. 49 further `failed` rows are cells a mutation was not designed for (collateral); they count as no detection. Compared cell by cell with the 815fc984 report, all 550 outcomes are identical; only pass/fail counts of six concurrent cells differ (for example `GM1/S02` 8/22 became 6/24), and in one of them (`GMearly/S10/concurrent`, a collateral cell) `fired_rules` went from `G5.c` to `G5.c,G5.d`.

## The 60 `harness_error` runs: the two excluded cells

`GM2/S22/concurrent` and `GMstop/S22/concurrent` (30 runs each) are the only runs with a harness error, and they are the recorded exclusions of `migration/stage-1/restart-gaps-173/EXCLUSIONS.md` (Owner option C; exactly these two cells). They are **not** zero errors and are never counted as passed, survived or detected. Re-read from the preserved logs: every one of the 60 runs has one `gate-timeout`, shows `c` ran its first `PostStop`, and no child began `PostStop`, so the precondition of S22 (a restart waiting for held children) was not reached. Control `GMdefaultps/S22/concurrent`: precondition reached in 30 of 30 runs, no `gate-timeout`, `failed` by `G5.f` as designed. The runner excludes a cell only when all 30 runs show this; one different run would have made the mutation unmet. The exclusion says the scenario cannot run under those mutants, not that they are harmless: both are detected elsewhere by their designed clauses (GM2: S10 `G5.a`/`G5.d`; GMstop: S19 `G5.b`).

## Checks

- `scripts/check-judge-matrix.py`: exit 0; goPekko **CURRENT on darwin/arm64**, detected 21/21, benign survived 1/1. `judge/check-judge-matrix-log.txt` is unchanged (the checker prints no commit).
- `judge/verify_package.py <dir> go-campaign-52088eab`: exit 0, valid. `SHA256SUMS` checked on a fresh extraction: 23,531 of 23,531 OK.
- `judge/mutate_go.py --revalidate` on the extracted package: same comparator, every row reproduces from the preserved logs. The packaged report, `console.log` and `manifest.json` equal `judge/go-mutation-report.tsv`, `campaign-log.txt` and `campaign-manifest.json`.

## Currency

CURRENT while `pekko-actor/` without `*.md`, `judge/` inputs (comparator, runner, specs, mutations) and the pinned toolchain are as at `52088eab`. Since 815fc984, `pekko-actor/` changed by `actor/dropped.go`, `actor/dropped_test.go`, `actor/scheduler_close_test.go` and `README.md`, so the old campaign was not reused. A change only in documents or `*.md` does not end it.

## Machine load

Not exclusive: an Android emulator `crashpad_handler` (about 99% CPU) and Spotlight `mdworker` ran at launch. No rule of `docs/judge.md`, `AGENTS.md`, `judge/mutate_go.py` or the checker requires a dedicated machine for a Go campaign (CPU count is recorded only for calibration). No run timed out and no outcome changed against 815fc984, so no effect was found. Exclusivity is not claimed.

## Evidence files and package

| File | SHA-256 |
|---|---|
| `judge/go-mutation-report.tsv` (550 rows) | `37ccb48c2e08343421eb9941d37df16dc6933c8582e96e4f49f8645e57c44dcd` |
| `judge/campaign-log.txt` (runner stdout, 25 lines) | `05d13a3edbf92b9ea421dcca7520c81c45450df33ecbae0beb85d8942070a195` |
| `judge/campaign-manifest.json` | `2114bfe98331f86c6e87494d7ee1e197a5592957951c4e32323f89a5045da1c4` |
| `go-campaign-52088eab.tar.gz` (23,531 files, 13,965,339 bytes) | `212479fd75b4a5f98ed744d67f534c2bf09573c530cea197e742d036ca5f7ba8` |
| `go-campaign-52088eab.SHA256SUMS` | `c970987cede19e166273ee90d469e7163a3bb41a17e26a0f612ef06ad08dfbeb` |

The package (`console.log`, report, `identity.txt`, `specs/`, `mut/<mutation>/` logs and mutated copies, `run-info/`) is at `/Users/pablogore/workspace/pablogore/goPekko-evidence/` on the Owner's Mac only; no shared location is authorized yet.

## Budget (AGENTS.md "PR budget and commits")

The diff is 1,163 authored lines (610 additions, 553 deletions; the Owner informed 1,182) in three files, none generated, vendored or a lockfile. **Exception approved by the Owner (express instruction, 2026-11-13, not by QA or merge)**, limited to this diff and these files: `judge/campaign-manifest.json`, `judge/go-mutation-report.tsv`, this file. It does not change `AGENTS.md` and authorizes no future exception, functional change, evidence waiver or RULEBOOK change. A larger diff needs a new delta presented first.

## What it shows, and what it does not

On darwin/arm64 with `-race`, the 15 Go scenarios pass on unmodified code and each designed mutation is detected by its designed clause. It is one platform and one campaign of 30 concurrent runs: no Linux evidence, no V level, nothing about `pekko-actor-typed`, no parity claim for Actor, Typed, #999 or #906.
