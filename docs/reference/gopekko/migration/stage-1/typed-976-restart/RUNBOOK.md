# RUNBOOK: Typed restart within its limit (unit typed-976-restart)

Status: **prepared.** Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `develop` `620e57e` (the amended scope, #1354). Delivery is eleven sequential PRs (scope amendment of 2026-11-05, merged in #1354), each `Refs #976`, never `Closes`, no merge without the Owner. The scope is `docs/scopes/976-typed-restart.md`.

| PR | Content | Scenarios |
|---|---|---|
| 0 | this package | |
| 1a | `Supervise`, matchers, `Restart()`, restart of case (a), `PreRestart` | R1 to R4, R15 |
| 1b | equal-matcher replacement, nested-chain and restart-loop tests; merges together with 1a | R3b, R10 |
| 2a | `WithLimit` window | R5, R6 |
| 2b | initial start under a limit | R7 to R9 |
| 3a-1 | wait for children, hold messages | R17, R15 during the wait |
| 3a-2 | C1 | R11 |
| 3b | `WithStopChildren` | R18 |
| 3c-1 | Classic-parent `PreRestart` hook | R16 |
| 3c-2 | rethrow after the start | R12 to R14 |
| 4 | README, `EXECUTION-RECORD.md`, one `migration/cost-log.tsv` row, gate evidence bound to the final SHA | |

## Manifest completeness check

Every test of `SupervisionSpec.scala` must have a row in `manifest.tsv` (ported, adapted, moved or `PORT-GAP`). Run from the repository root; it must print `uncovered []`:

```
python3 - <<'PY'
import re
SS='upstream/pekko/actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/SupervisionSpec.scala'
tests=[i+1 for i,l in enumerate(open(SS).read().split('\n')) if re.match(r'\s*"[^"]*" in',l)]
cells=' '.join(l.split('\t')[3] for l in open('migration/stage-1/typed-976-restart/manifest.tsv').read().split('\n')[1:] if 'SupervisionSpec' in l)
nums={int(x) for x in re.findall(r'(?<![\d-])(\d{2,4})(?![\d-])',cells)}
print('uncovered',[n for n in tests if n not in nums])
PY
```

## Procedure per PR

1. TDD: write each scenario, watch it fail, implement the minimum, inside the allowed files. Slices 1a to 3c-2 all edit `typed/interpreter.go`: never in parallel.
2. Gates (commands in `AGENTS.md`, "Go"): unit tests of `pekko-actor-typed` with network isolation where the platform allows, `go vet`, `gofmt -l`, `golangci-lint` (gocyclo at most 10), `go mod tidy -diff`, module coverage at least 85%, diff coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `go-sdk-update.sh --check`. Run once per final tree.
3. Record each gate with the pushed SHA, command, environment, exit code and output. Report the real authored-line count of the PR against the estimate of the scope.
4. QA reviews each PR on its final SHA; every commit after the gates ran invalidates the evidence.

## Stops

A file outside the allowed list must change; `pekko-actor` needs an export; a scenario fails and is not a Go defect (run it on Pekko first); a slice projects over 400 authored lines. Report to the TL; never skip, delete or weaken a test.
