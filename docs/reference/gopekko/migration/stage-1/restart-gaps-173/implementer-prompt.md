# Implementer prompt (two units, run one at a time)

Status: PREPARED, NOT APPROVED. Use only after the Owner approves `BUDGET.md`. The coordinator passes exactly one unit section below plus the common part. Tool rules: rg/fd/bat/eza only (never grep/find/cat/ls/sed). Artifacts in English; Conventional Commits; no AI attribution.

## Common (read first, all of it)
1. `migration/stage-1/restart-gaps-173/RULEBOOK.md` (read-only): section 0 and your unit's section.
2. `migration/stage-1/restart-gaps-173/S22-GO-PROPOSAL.md` (S22 unit) and `EXECUTION-RECORD.md` Update 2 (B unit).
3. `CLAUDE.md` project rules (module README contract, no `//nolint`, gocyclo at most 10, coverage at least 85 %).
4. Never edit the RULEBOOK, the manifest, the records, `odd/`, or anything outside your surfaces. Do not push, merge, tag, or run the full campaign. Do not repeat a run hoping for a different result.
5. Advisory only: about 400 authored changed lines per task is a planning heuristic, not a cap.
6. Return, per item: done / partial / blocked, files, `<command>: <observed result>`, and every INVALID kept apart. Close with `## Key Learnings` (1-5 numbered factual sentences, at least 4 words each).
7. Stop and report (do not improvise) on: a Pekko difference not approved (rule C5), the B3 feasibility stop, a verification you cannot run.

## Unit B: deterministic GMoverlap
Do: RULEBOOK section B (B1-B7). Change the `GMoverlap` cell text and what the runner needs, add the selftest cases, run the verification of B5 in the background and read the result.

### Allowed edit surfaces
judge/mutate_go.py
judge/selftest_mutate_go.py

### Verification
python3 -I judge/selftest_mutate_go.py
python3 -I judge/selftest.py
the B5 run of GMoverlap on a copy outside the repository (workdir under /Users/pablogore/.claude/jobs/b989ecff/tmp), 20 sequential `go test -race -count=1 -run ^TestJudgeScenarios$` runs; report exit codes and race counts per run, kept apart from every earlier run.
sha256 comparison of the RULEBOOK section 4 files for unit B against the baseline the coordinator gives you.

## Unit S22: S22 in the Go gate (only after unit B is accepted)
Do: RULEBOOK section S (S1-S10). Test first for S2 (default-restart path with and without sink).

### Allowed edit surfaces
pekko-actor/actor/supervision_restart.go
pekko-actor/actor/judge_internal_test.go
pekko-actor/actor/supervision_defaultrestart_internal_test.go
judge/go-scope.json
judge/mutate_go.py
judge/selftest_go_scope.py
judge/selftest_mutate_go.py
docs/judge.md
migration/stage-1/milestone-1.md
pekko-actor/README.md
README.md
docs/adr/0017-supervision.md

### Verification
go test -C pekko-actor -race -count=1 ./...
go test -C pekko-actor -race -count=20 -run the new tests
python3 -I judge/selftest_mutate_go.py
python3 -I judge/selftest_go_scope.py
python3 -I judge/selftest.py
the harness S22 run in Go on the unmutated tree (`judge/run-go.sh` or the documented equivalent) and the GMdefaultps cell run alone on a copy outside the repository; report the fired rules (must contain G5.f)
gofmt -l pekko-actor; go -C pekko-actor vet ./...
the pinned golangci-lint 2.14.0 at /Users/pablogore/.claude/jobs/b989ecff/tmp/bin/golangci-lint with GOTOOLCHAIN=go1.27.2 from inside pekko-actor with --config ../.golangci.yml
sha256 comparison of the RULEBOOK section 4 files for unit S22 against the baseline.
