# RUNBOOK: Current sender (#814)

Status: **prepared; stage 2a is a Dev-run internal refactor, and no model or cost plan is claimed.** Base: scope SHA `39ff8e027bcfd733f006f614618adf9bb926c9f1`. Pekko pin: `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.

## 1. Writable files per stage

Those of the scope's section 7 for the stage. Stage 2a: `mailbox.go`, `cell.go`, `ref.go`, `stop.go`, `watch.go`, and existing tests only where compilation requires it.

## 2. Procedure

1. Observe RED before the change (the envelope test does not compile).
2. Implement the minimum and run the module tests.
3. Differential check of `Tell` against the base commit in a `mktemp -d` copy.
4. Gates of `AGENTS.md`: gofmt, go vet, go test, coverage, golangci-lint, gocyclo, go mod tidy, `scripts/check_deps.sh`.

## 3. Not recorded here

Agent counts, token and cost figures were not measured for this stage, so none are written. The execution record (stage 3) holds the evidence bound to a SHA.
