# RUNBOOK: Bound the Typed restart stash (unit typed-999-restart-stash)

Status: **prepared, not signed off.** Sources pinned: Pekko `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (v1.7.1), kit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. Base: `develop` `2b799d9d`. The scope is `docs/scopes/999-restart-stash-bound.md`. Each PR is `Refs #999` (and `Refs #906` for U1), never `Closes`, and merges only with the Owner's later authorization.

| PR | Content | Scenarios | Depends on |
|---|---|---|---|
| K0 | this package (docs) | | G2 |
| O1a... | oracle for plain `Restart` capacity, split by measured size | feeds S2 to S5, S7 | G2 |
| U1 | `actor.Dropped` (#1562) | EV9, D1 | G2 |
| U2 | bound, default 1000, `WithStashCapacity`, publish `Dropped` | S1 to S5 | U1, O1 integrated, K0 signed off |
| U3 | `Dropped(PostStop)`, replay and failure | S6 to S8 | U2 |

Scenarios: S1 default capacity (W3d); S2 `WithStashCapacity(n)` overflow (D1, O1); S3 capacity 0 (O1); S4 negative (O1); S5 full stash with awaited and non-awaited `Terminated` (O1); S6 stop during the wait, full and not full (W3e, W3a); S7 failure during replay keeps the stash (O1, `SupervisionSpec.scala:1195, 1232`); S8 stop during replay, leftovers are `DeadLetter` (W3c, RS1, RS2).

## Manifest reference check (six enumerated references only)

This check is **not** a discovery of the relevant tests of `SupervisionSpec.scala`: it only verifies that the six enumerated line numbers 882, 912, 1195, 1199, 1232 and 1237 appear in a `SupervisionSpec` row of `manifest.tsv`. `uncovered []` means those six have a row, nothing more. Run from the repository root:

```
python3 - <<'PY'
import re
rows=open('migration/stage-1/typed-999-restart-stash/manifest.tsv').read().split('\n')[1:]
cells=' '.join(l.split('\t')[3] for l in rows if 'SupervisionSpec' in l and len(l.split('\t'))>3)
nums={int(x) for x in re.findall(r'(?<![\d-])(\d{3,4})(?![\d-])',cells)}
print('uncovered',[n for n in (882,912,1195,1199,1232,1237) if n not in nums])
PY
```

## Scenario completeness review (separate, by reading `SupervisionSpec.scala` at the pin)

Done by reading the spec for every test that stashes, unstashes or drops during a restart, **including those already implemented** by #976 and #997 (their rows are in `migration/stage-1/typed-976-restart/manifest.tsv`). It is a human review, not a script, and it lists what was found by searching the spec for `stash`/`Stash`; other tests may exist.

| Spec lines | Test | Plain `Restart`? | Disposition in this unit |
|---|---|---|---|
| 575, 618 | stop children when restarting; restart after stopping `watchWith` children (575 reads: GetState and `Terminated` stashed, delivered to the new behavior; 618 was found by its `stash` comment at 649) | yes | **already implemented** by #976 (`restart_children_test.go`); regression only, not a U2/U3 deliverable |
| 691 | stop children when restarting second time during unstash | yes | adapted to S7 (failure during replay); the #976 manifest marks it moved; found by the `stash` search, its body was **not studied** for this review and it was not run; O1 `CF1` observes a replay failure, not this test |
| 882, 912, 991, 1007, 1022 | backoff: `Dropped` and `withStashCapacity(0)` | no (`restartWithBackoff`) | moved to #986 |
| 579, 695, 1355 | backoff variants (stop children, second time during unstash, duplicate in a setup) | no | moved to #986 |
| 1195, 1232 | retention during an unstash exception and a start exception, `restart.withStashCapacity(4)` | yes | adapted to S7; read, not run; a failure during the start is **not** observed by O1 and stays inferred |
| 1199, 1237 | the backoff variants of the above | no | moved to #986 |

The manifest rows added with this review are 575, 618, 691, 579, 695, 991 and 1355. The six-reference check above does not cover them; this table does.

## What the kit runner status does and does not show

`migration-kit/scripts/queue_runner.mjs` reads only the `source` and `target` columns and treats a target that **exists on disk** as done (`existsSync(target)`), and `verify` only checks that done targets are non-empty. Many targets here already exist because they are existing files (for example `pekko-actor-typed/typed/interpreter.go`): that status does **not** show that the change this unit adds to them is implemented. The scenario and evidence columns need the semantic review above; TDD, the gates and QA below are not replaced by the runner's status.

## Procedure per PR

1. TDD: write each scenario, watch it fail for the right reason, implement the minimum, inside the allowed files. U2 and U3 both edit `typed/interpreter.go`: never in parallel.
2. Gates (commands in `AGENTS.md`, "Go"): unit tests of the module with network isolation where the platform allows, `go vet`, `gofmt -l`, `golangci-lint` (gocyclo at most 10), `go mod tidy -diff`, module coverage at least 85%, diff coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, `go-sdk-update.sh --check`, README contract. Run once per final tree.
3. Record each gate with the pushed SHA, command, environment, exit code and output. Report the real authored-line count against the estimate.
4. QA reviews each PR on its final SHA, from a clean read-only context; every commit after the gates ran invalidates the evidence. Merge needs the Owner's later authorization.

## Stops

A file outside the allowed list must change; `pekko-actor` needs another export; a scenario fails and is not a Go defect (run it on Pekko first); an oracle contradicts the scope; a PR projects over 400 authored lines (split or ask, no exception is granted); the RULEBOOK is not signed off. Report to the TL; never skip, delete or weaken a test.
