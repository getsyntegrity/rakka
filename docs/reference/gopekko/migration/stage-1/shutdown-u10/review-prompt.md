# Task: one focused, read-only review of delivery U10 (ordered minimal termination of the system; #192, #193)

You are the only reviewer of this delivery. You are read-only: no Edit, no Write. You can run `git status`, `git diff` (also with a path, for example `git diff ced37b9 -- pekko-actor/actor/spawn.go`), `git diff --stat`, `git show <rev>:<path>`, `go vet`, `eza`, `wc`, and read any file with the Read tool. You cannot run tests. You run non-interactively and a **spend cap is enforced by the launcher** (USD and wall clock): when it is hit your session ends with whatever you have. Work in this order and keep your findings as you go:

1. `git diff ced37b9 --stat` (the base of this delivery; the code may be committed or not), then the diff of the production files: system_shutdown.go, errors.go, spawn.go. Judge these before anything else.
2. migration/stage-1/shutdown-u10/RULEBOOK.md sections 1 to 8, to check the diff against D1-D9, I1-I9 and SH1-SH11.
3. Only then the tests: system_shutdown_internal_test.go and system_shutdown_api_test.go.
4. The guardian wiring only to settle a doubt: pekko-actor/actor/guardian.go (`wireGuardians`, `startGuardians`, the `guardian` actor) and stop.go (`acceptStop`, `finalize`).

## What to look for (assume the work is wrong)
- **Order and finalization:** `Terminate` stops `/user` and nothing else; `WhenTerminated` is the root guardian's `done` and cannot close before the descendants finished; the same channel on every call. I1 is relations, not a total order: R3 is claimed only when the start is at `/user`.
- **Idempotence and races:** repeated and concurrent `Terminate` and `Shutdown`; the gate; a `Spawn` racing the first `Terminate` either commits and is stopped or fails, never a leaked running actor.
- **`Shutdown`:** it calls `Terminate` first; nil takes precedence over a cancelled or expired `ctx`, including when both are ready together; on `ctx` it returns **`ctx.Err()` itself, unwrapped** (no package error exists); it never claims termination; the termination continues; from a handler or hook it can only end by `ctx` and no automatic detection exists.
- **Errors:** `ErrSystemStopping` has `Error()` exactly `actor: system is stopping` and is-a `ErrParentStopping`; only a top-level `Spawn` returns it; a `Spawn` from a `Context` still returns `ErrParentStopping`; no existing `errors.Is` check is weakened; no `ErrShutdownDeadline` exists.
- **No helper goroutine** (no `go` statement in the three functions; no test counts goroutines, so judge it by reading), the work goes through the System's executor as `System.Stop` does; no new `System` field; no change to guardians, `stop.go`, `cell.go`, `hierarchy.go`, `ref.go`, `system.go`, the executor or any judge-anchored file. The doc comments must not say `Terminate` "does not run user code" or "never blocks": under an inline executor (tests only) it runs the cascade in the caller (RULEBOOK D10).
- **Scope guard:** nothing about phases, hooks, `registerOnTermination`, cancellation of handlers, Ask cancellation, journal writes, durability or draining is implemented or promised, including in doc comments.
- **Exported surface is exactly** `(*System).Terminate`, `(*System).WhenTerminated`, `(*System).Shutdown` and `ErrSystemStopping`.
- **Tests assert what they claim** and would fail under the eight mutants of RULEBOOK 8 (Terminate stopping the wrong guardian, WhenTerminated returning another channel, Shutdown ignoring ctx or wrapping its error, not calling Terminate, Spawn keeping ErrParentStopping, ErrSystemStopping not wrapping ErrParentStopping, a goroutine or channel per call, Shutdown claiming termination on ctx). A vacuous test is a DEFECT. SH4 tests the listed executor orders (FIFO and LIFO prefixes, deferred, inline, real) and must not claim every interleaving; SH8 uses a counting executor. Existing test files are not edited (check `git diff --stat`).
- Nothing outside the surfaces changed; `go.mod`, `go.sum` and `judge/` untouched.

## Output
Findings only, classes BLOCKER / DEFECT / INFO, each citing a RULEBOOK rule or a source `path:line`. INFO is recorded, not acted on. **The whole final report is at most 1,500 tokens.** If you could not read something, say exactly what, so it is recorded as unreviewed. No praise, no restatement of the diff, no second pass: this is the only review of this delivery.

Close your final response with a `## Key Learnings` block of 1-5 numbered items, each a standalone factual sentence (at least 20 characters, at least 4 words); omit it if there is no reusable learning.
