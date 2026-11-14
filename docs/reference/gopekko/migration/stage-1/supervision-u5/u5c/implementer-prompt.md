# Task: implement delivery U5c (public guardian strategy and top-level name validation) in pekko-actor/actor

You are the single implementer of this delivery: production code and tests, one session. You run non-interactively: nobody can answer questions during the run. This is **U5c only**. U5a and U5b are merged in `develop` (`eeac94d`) and are your base. `System.Shutdown`, `whenTerminated`, termination hooks, `AllForOne` and custom guardian `Props` are NOT part of it.

## Read first, in this order (do not re-plan or re-map)
1. migration/stage-1/supervision-u5/RULEBOOK.md (approved, read-only): sections 2 (D3, D7), 3.5 (the exact public API, copy its signatures and docs), 5 (rows 5, 6), 6 (T5p, T8, T15, P3, and the P7 note) and 9 (row U5c).
2. migration/stage-1/supervision-u5/u5c/RUNBOOK.md: sections 1, 2 (the probe result) and 6 (surfaces).
3. migration/stage-1/supervision-u5/u5b/EXECUTION-RECORD.md: the INFO items that enter this delivery.
4. migration/stage-1/supervision-u2/RULEBOOK.md section 4: conventions (go-specs tests only, headers on ported files, `errors.Is/As`, no `//nolint`, gocyclo at most 10 per function including literals, coverage at least 85%).
5. Pekko, read-only: `upstream/pekko/actor-tests/src/test/scala/org/apache/pekko/actor/ActorSystemSpec.scala` around line 336 (P3), `dungeon/Children.scala` 64-65 and 270-275 (`checkName`).
6. The code: system.go (`Config`, `newSystem`, `options.userStrategy`), guardian.go (`startGuardians`), spawn.go (`commitCell`, the name check), hierarchy.go (`spawnTarget`, `topLevel`), supervision.go (`Strategy`, `OneForOne`, `DefaultStrategy`), names.go (`validChildName`).

## Allowed edit surfaces (the only files you may write), all under pekko-actor/actor/
Production (8):
- guardian.go (`GuardianStrategy`, `GuardianDecider`, `GuardianOneForOne`, `StoppingGuardianStrategy`, and the adaptation to `Strategy[guardianMsg]`)
- system.go (`Config.GuardianStrategy`, the wiring in `newSystem`; `options.userStrategy` stays and wins when set)
- spawn.go (retire the `topLevel` condition; keep `t.parent != nil`, because the root guardian has name `""` and no parent)
- hierarchy.go (remove the `topLevel` field and its use)
- errors.go (comments only: `ErrInvalidName` now applies to top-level names; the root-owner text)
- supervision.go, supervision_hooks.go, watch.go (comments only: the stale "a top-level actor has no parent / stops directly (interim until the guardians, U5)")

New tests (2, create them):
- guardian_api_test.go (`package actor_test`: the external Example of RULEBOOK 3.5)
- guardian_config_internal_test.go (T5p, the T15 behaviors, P3)

Existing tests (3):
- names_test.go: replace `TestTopLevelNamesAreNotValidated` by T8 (the five names `""`, `"$x"`, `"a b"`, `"a/b"`, `"%zz"` give `ErrInvalidName` at top level, valid names are accepted, nothing is reserved after a refusal). The rest of the file stays. The probe shows this is the only existing test that fails.
- guardian_goside_internal_test.go: T7 only rules out `Ref[int]`; also assert the `/user` handle from `Parent()` is not a `Ref[guardianMsg]` (D5/D11: no Tell or Ask).
- supervision_goside_internal_test.go: V14's escalating case asserts only the first event; assert that `receive-failed` occurs exactly once, as its comment says.

Everything else is denied, including cell.go (judge anchors), stop.go, names.go, judge/, docs/, READMEs, ADRs, the RULEBOOK. **If you need a file outside the surfaces, stop and report `BLOCKED(surface): <file and why>`.**

## What to deliver (RULEBOOK 3.5, D3, D7)
- The exact public API of 3.5, unchanged in signature: `Config.GuardianStrategy GuardianStrategy`; `GuardianStrategy` (unexported fields), `GuardianDecider func(child Handle, cause error) (Directive, bool)`, `GuardianOneForOne(Budget, GuardianDecider)`, `StoppingGuardianStrategy()`. **No exported signature mentions `guardianMsg`**. Zero value = `DefaultStrategy`; a nil decider is undefined for every cause (Escalate); `Escalate` makes `/user` fail and the root guardian stops it (the chain ends the system); a panic in the decider is a failure of `/user` (the system ends). The decider receives no `Context` (document that limitation as 3.5 does).
- `options.userStrategy` (the U5a/U5b test seam) keeps working and takes precedence over `Config.GuardianStrategy` when set.
- Top-level names are validated like child names (D7): `Spawn(sys, "", …)`, `"$x"`, `"a b"`, `"a/b"`, `"%zz"` return `ErrInvalidName` before any reservation. The root guardian (name `""`, no parent) is exempt as before.
- No other exported identifier changes.

## Tests (RULEBOOK 6; deterministic: channels and controlled executors, no sleeps as coordination)
- T5p: the U5b scenario T5 through `Config{GuardianStrategy: GuardianOneForOne(…, escalate)}`, plus a nil decider and a decider that panics: the chain runs, every top-level actor ends, `Spawn` returns `ErrParentStopping`, one `stop-accepted` per guardian.
- T15: the zero value behaves as `DefaultStrategy` (a top-level `Receive` error restarts); `GuardianOneForOne` with a budget; `StoppingGuardianStrategy`; the external Example in `guardian_api_test.go` builds a system with each and has an `// Output:` block; if any signature needed a private type that file would not compile.
- P3 (ActorSystemSpec:336, ADAPTED): with `StoppingGuardianStrategy()` a top-level actor that fails in `Receive` is stopped and its watcher gets exactly one `Terminated`.
- Mark in a comment that P7 (`ActorSystemSpec:328,359`, `whenTerminated`) stays `PORT-GAP(system-shutdown)`.

## Judge anchors
Of your files none carries a `judge/mutate_go.py` anchor except hierarchy.go (` || len(n.out)+n.reserved != 0 {`, keep that exact text, once). If an anchor must move: `BLOCKED(anchor): <file and anchor>`.

## Hard constraints
- You CANNOT run `go test` (denied). Your loop is `gofmt -l pekko-actor/actor`, `go vet ./pekko-actor/...` (compiles the tests) and `go build ./pekko-actor/...`.
- No git write commands, no commits.
- Do not weaken the invariants of U1-U5b. If one concrete decision is unresolved and not decidable from the RULEBOOK or Pekko, report `NEEDS-DECISION: <the single question, the options, your recommendation>`.
- Time bound: 30 minutes wall. About 400 authored changed lines per task is a heuristic, not a cap.
- Written artifacts (code, comments, test names) are in English, neutral register.

## Final report (your last message)
1. Files changed (path and one line each). 2. gofmt -l, go vet, go build results. 3. Per test T5p, T8, T15, P3, the Example, and the two INFO assertions: the test name(s). 4. Confirmation that the exported surface equals RULEBOOK 3.5, no file outside the surfaces changed, and the hierarchy.go anchor is intact. 5. Open doubts and anything unverifiable without running tests. 6. If blocked: the BLOCKED or NEEDS-DECISION line first.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
