# Implementer prompt (one implementer, tests and mutants only)

Status: PREPARED, NOT APPROVED. Launched only after the Owner approves `RUNBOOK.md` and the package hashes. Tool rules: rg/fd/bat/eza only (never grep/find/cat/ls/sed). Artifacts in English; Conventional Commits are the coordinator's job (you do not commit). No AI attribution anywhere. Run each Bash command as a plain separate call.

## Read first, completely
1. `migration/stage-1/supervision-u4-followup/RULEBOOK.md` (read-only for you): every section.
2. `migration/stage-1/supervision-u4/EXECUTION-RECORD.md` sections 6 and 8 (the acceptance criteria this unit completes) and `docs/adr/0012-stop-before-first-activation.md`, the paragraph "U4 (ADR 0017 S10, #444)".
3. Production code you test, read-only: `pekko-actor/actor/spawn.go`, `cell.go` (`startActor`, `callPreStart`, `handOver`, `drain`), `stop.go` (`terminate`, `finalize`), `supervision_failure.go` (`failUpExcept`), `supervision_restart.go` (`reportInitFailure`).
4. The existing tests and helpers you reuse, do not duplicate: `hierarchy_internal_test.go` (the `System.commit` test around :553-567), `spawn_supervised_internal_test.go`, `spawn_supervised_goside_internal_test.go`, `termination_internal_test.go`, and the `ctlExec`, `recSink`, `journal`, `sup1`/`sup2` rigs; `lockPoint`, `userStrategy` and `sink` options in `system.go`.
5. The Pekko sources the RULEBOOK cites, in `upstream/pekko` (read-only), to check the expected results.

## Your task
Unit scope is RULEBOOK sections 3 and 4:
- **T460a and T460b** (no production change): write the tests in `pekko-actor/actor/spawn_factory_stop_internal_test.go`.
- **T459**: write `pekko-actor/actor/spawn_stopping_internal_test.go` **only if the coordinator tells you in the launch message that decision D1 is approved and the seam exists** (name and signature given there). Otherwise do not write it and do not touch production.
- **Mutants:** add the cells M459 (only if T459 exists), M460a, M460b-twice, M460b-none to `UNIT_MUTATIONS` of `judge/mutate_go.py`, with the named killer test and a run count, and add selftest cases to `judge/selftest_mutate_go.py` (anchor occurs once; the killer test exists; classification of INVALID). Do not edit existing selftest cases or any other cell.
- RED before GREEN (C7). For every test, show it fails against its directed mutant.
- If a test derived from the contract fails on the unmutated production code, STOP that item and report the evidence (C2). Do not change production. If you need something the RULEBOOK does not allow, or you see a difference from Pekko not declared in section 2, STOP and report (C5).

## Allowed edit surfaces
pekko-actor/actor/spawn_stopping_internal_test.go
pekko-actor/actor/spawn_factory_stop_internal_test.go
judge/mutate_go.py
judge/selftest_mutate_go.py

## Verification (report each as `<command>: <observed result>`; INVALID kept apart from detections)
- `go test -C pekko-actor -race -count=1 ./...`
- `go test -C pekko-actor -race -count=20 -run <your new tests> ./actor`
- `python3 -I judge/selftest_mutate_go.py` and `python3 -I judge/selftest.py`
- each new mutant through `python3 -I judge/mutate_go.py /Users/pablogore/.claude/jobs/b989ecff/tmp/u4f-run <ID> [...]` (a workdir outside the repository; NEVER pass `--help`, it creates a stray directory; read the docstring with bat), without `--report`; report detected/survived/INVALID with counts, and the failure message that detected it
- `gofmt -l pekko-actor` and `go -C pekko-actor vet ./...`
- `git diff --stat`: only the allowed surfaces
- `shasum -a 256 -c` against the baseline file the coordinator names in the launch message: no mismatch.
Do not run the full campaign, do not tag, do not push, do not commit. Do not repeat a failing run hoping for a different result.

## Return
Per item: done / partial / blocked, files, observed command results, mutant results, any evidence of a candidate defect or an undeclared difference. Report your own token usage if visible. Close with `## Key Learnings` (1-5 numbered factual sentences, at least 4 words each).
