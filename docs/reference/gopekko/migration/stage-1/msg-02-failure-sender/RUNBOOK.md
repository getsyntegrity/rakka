# RUNBOOK: Sender of the parent's decider (#814, unit msg-02-failure-sender)

Status: **prepared; blocked on ADR 0032 D5a (Owner).** No model or cost plan is claimed. Pekko pin: `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.

## 1. Writable files

`pekko-actor/actor/supervision_failure.go`, `pekko-actor/actor/sender_failure_test.go`, `pekko-actor/README.md`; then `migration/stage-1/msg-02-failure-sender/EXECUTION-RECORD.md` and one `migration/cost-log.tsv` row. Nothing else.

## 2. Procedure

1. Observe RED: G1 (decider reads the child) fails on the base because the sender is absent.
2. Replace the `nil` store in `handleFailed` by the child identity; run the module tests.
3. Mutation checks listed in scope section 4; differential check of the existing supervision tests against a base copy in a `mktemp -d` directory.
4. Gates of `AGENTS.md` "Go" (scope section 10): gofmt, go vet, go test, coverage 85, diff coverage 85, golangci-lint v2.14.0, gocyclo 10, `scripts/check_deps.sh`, `make parity-guard`, `make ci-fast`.

## 3. Not recorded here

Agent counts, token and cost figures are not measured. The execution record holds the evidence bound to a SHA.
