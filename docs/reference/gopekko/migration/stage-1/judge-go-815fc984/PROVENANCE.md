# Go judge campaign on develop 815fc984 (provenance)

One full campaign of `judge/mutate_go.py` on `develop` at `815fc984cc0bac484107d2ea4b3a3f06f7d2fae8`, run on the Owner's Mac. `develop` is unchanged since the campaign was executed (no new commits on `pekko-actor/`), so the campaign is **CURRENT** on this HEAD. Nothing here changes the judge, the scenarios, the comparator, the classification of any outcome or the exclusions. No timeout, crash, harness error or abort was reinterpreted. It replaces the Go report of `dbdf15b` (`judge-go-1182-821`), which stays as history.

## Identity

| Field | Value |
|---|---|
| **go_commit** | `815fc984cc0bac484107d2ea4b3a3f06f7d2fae8` (origin/develop) |
| **tree** (pekko-actor) | `392a810c0f6e398d98ab28f11e3d2c6dbfc69dd2` |
| **comparator_sha256** | `564072ea1438ea6f2dee5aec776990c8eeb3fdaaffa38edba4ff00a4b6d8c55e` |
| **go_version** | `go1.27.2` (pinned in `.go-version`) |
| **platform** | `darwin/arm64` |
| **runs** | 30 concurrent, 1 controlled |
| **command** | `python3 -I judge/mutate_go.py <workdir> --report <workdir>/go-mutation-report.tsv` |
| **working directory** | git worktree detached at `815fc984` (submodules initialized) |
| **exit code** | 0 |

## Command & timing

The campaign was launched from a detached worktree at `815fc984` with recursive submodules. The exact command was:

```
python3 -I judge/mutate_go.py /Users/pablogore/workspace/pablogore/gopekko-campaign-work2 \
  --report /Users/pablogore/workspace/pablogore/gopekko-campaign-work2/go-mutation-report.tsv
```

Start: `2026-11-13T14:45:33Z` (UTC). Duration: approximately 20 minutes. Exit code: 0.

The runner console log (`judge/campaign-log.txt`) contains only the 25-line status summary produced by `mutate_go.py` at completion; it is not a substitute for the per-scenario logs stored in the evidence package.

## Results

22 of 22 mutations `met`, `unmet: none`, exit 0. 550 rows: `failed` + `survived` + `excluded`. 0 `timeout`, 0 `invalid-campaign`. The only runs with `harness_error` are the 60 runs of the two excluded cells (30 each), the recorded `gate-timeout`. 21 of 21 designed mutations detected by their designed clause; `GMbenign` survived in all its rows.

**Exclusions** (2 cells, not applicable — precondition of S22 not reached):
- `GM2/S22/concurrent`: gate-timeout in all 30 runs
- `GMstop/S22/concurrent`: gate-timeout in all 30 runs

These exclusions are documented in `restart-gaps-173/EXCLUSIONS.md` and were present in the previous campaign as well.

## Checker

`scripts/check-judge-matrix.py` (run from the campaign branch with its Git history): exit 0, no errors.

- **goPekko CURRENT on darwin/arm64**: detected 21/21 designed, benign survived 1/1
- The Go core files (`pekko-actor/` without `*.md`), comparator, campaign script, specs, and pinned toolchain are identical to HEAD → **CURRENT**

Full checker output is in `judge/check-judge-matrix-log.txt`.

## Evidence files in the repository

| File | SHA-256 |
|---|---|
| `judge/go-mutation-report.tsv` (550 data rows) | `7ea0898b3634c1f6724a17b5a161307cd04bd9934e2f1dfbc5695093eda00b29` |
| `judge/campaign-log.txt` (25-line runner summary) | `05d13a3edbf92b9ea421dcca7520c81c45450df33ecbae0beb85d8942070a195` |
| `judge/campaign-manifest.json` (provenance metadata) | `b8ec7cbb2843dfcc8903ce092118e6483577c1c75a27a8f7a1e7cf45c3457c6b` |
| `judge/check-judge-matrix-log.txt` (checker output) | `51d899bd7ad684ba90bd20c950654c8411c68583cfb0e8f1b308cb679e3bed38` |
| `migration/stage-1/judge-go-815fc984/PROVENANCE.md` (this file) | *(self-referential — verify with `sha256sum` after final commit)* |

## Evidence package (outside the repository)

The full evidence package with per-mutation logs, per-scenario logs, specs, and identity is kept outside the checkout at:

```
/Users/pablogore/workspace/pablogore/goPekko-evidence/
```

| File | SHA-256 |
|---|---|
| Package `go-campaign-815fc984.tar.gz` (23,455 files, 13 MB) | `389a2cdf8a5d72d255e12ef85eccae63d66970ced3e27fda7dda3c0b8e916b5c` |
| `go-campaign-815fc984.SHA256SUMS` (one line per file) | `c1478bf7c9c280b572bf322f7cf86c351e5c49812c1f6c2a8d3668263f46cf0f` |

The package was built with `COPYFILE_DISABLE=1`, verified with `python3 -I judge/verify_package.py <dir> go-campaign-815fc984` (exit 0, `valid`).

## Budget note (AGENTS.md § "PR budget and commits")

AGENTS.md states: *"at most 400 authored changed lines (additions plus deletions). Authored lines exclude generated code (files headed `// Code generated ... DO NOT EDIT.`, depmap output under `migration/stage-N/`), vendored files, lockfiles (`go.sum`) and `.claude/skills/`."*

The 4 evidence files in `judge/` account for 625 additions + 550 deletions = 1,175 authored changed lines. The exact budget count for all 5 files (including this PROVENANCE.md) is recorded in the PR description.

None of these files match the exclusion criteria in AGENTS.md:
- The TSV is machine-generated campaign output, but it does not carry the `// Code generated` header required by the rule.
- None are vendored files, lockfiles, or `.claude/skills/` files.

**Excepción puntual aprobada para PR #1557**: El Owner autoriza una excepción única y documentada, limitada exclusivamente a estos 5 archivos de campaña (`judge/go-mutation-report.tsv`, `judge/campaign-log.txt`, `judge/campaign-manifest.json`, `judge/check-judge-matrix-log.txt`, `migration/stage-1/judge-go-815fc984/PROVENANCE.md`) y al diff final de este PR. Esta excepción no modifica el límite general de 400 líneas en AGENTS.md, no autoriza futuras excepciones, cambios funcionales ni archivos adicionales, y no establece precedente para otros PR. Autorización atribuida expresamente a la instrucción del Owner que acompaña a esta corrección documental.

## Procedure deviation

The previous iteration of this campaign used `git push -f` (force push) to replace commit `c0bae9bc` with `74f38db4`, which included the complete set of 5 evidence files. The task instructions specified "push normal, sin force." The force push was used to correct an incomplete commit, not as a necessary step. This deviation is documented here; the history was not rewritten again to conceal it. Subsequent commits (`95c2f70c`, `fce50685`) were pushed normally to correct the provenance.

## What it shows, and what it does not

- With the Go core at `815fc984`, the 15 Go scenarios pass on the unmodified code and each designed mutation is detected by its designed clause on darwin/arm64. The core file digest of `815fc984` is the one of `develop`, so the campaign is **CURRENT** on it.
- One platform (darwin/arm64), one campaign of 30 concurrent runs. No V level is assigned or changed.
- It says nothing about `pekko-actor-typed` (this judge does not cover Typed parity) and nothing about the scheduler complete correctness — the scheduler code is exercised only through the unchanged paths of the Go scenarios.

## Earlier campaigns

- `dbdf15b` (`judge-go-1182-821`, darwin/arm64): replaced by this campaign; its package and PROVENANCE stay as history.
