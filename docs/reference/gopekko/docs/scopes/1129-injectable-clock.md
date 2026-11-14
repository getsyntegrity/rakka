# Scope: injectable System clock (#1129, TIM-11)

Status: **G2 approved by the Owner (partial, see section 0); ready for Dev.**
Written by the TL on 2026-10-28 against `origin/develop` `80695c90972cf3c27b3b3ea108a4ad6548174780`. Pekko baseline: v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`.
**Evidence labels.** READ = source read at the base SHA, not executed. No code was run for this scope. No Pekko parity is claimed.
**Kit run:** not required, with the reason in section 7 (Owner may overrule). **Squad:** A (time). It changes no issue and no label.

## 0. Owner authorization (real user message, 2026-10-28; relayed by the coordinator; prevails over any incompatible text below)

G2 is approved only for: `Config.Clock` as the injection point with the `NewSystem` signature unchanged (D1 = 1); exported interfaces `Clock` and `Timer`, minimal surface per the internal clock and real consumers (D2 = 1); the wall clock stays the default; **no manual clock and no extra wall-clock constructor** (D3 = 1); Ask guarantees kept; an individual EXP-1 / ADR 0011 justification per export.
**D4 is not pre-approved.** The Owner authorizes writing and running the public scenarios of section 5 but does **not** accept their results as alternative evidence under ADR 0033 D13. The final report and PR must separate four things: (1) evidence executed in Go; (2) behavior read from Pekko; (3) the applicable differential scenario, still pending; (4) alternative evidence that truly needs Owner acceptance. Sections 7 and 9 are read accordingly: D1 to D3 are decided as above, D4 is open and stays "pending approval".
**Pekko pin check.** `git -C upstream/pekko rev-parse HEAD` in the TL worktree returned the goPekko repository HEAD, not the pin: the submodule is not checked out (`git submodule status` shows `-9a54d1294c3ac7821bbed3c940769c3d5027d1ff`). The pin could not be verified here, so the only Pekko citation in this scope (`ExplicitlyTriggeredScheduler.scala:44,67`) is taken from the issue text and is **not** a verified reading. The Dev runs `git submodule update --init upstream/pekko`, checks that `git -C upstream/pekko rev-parse HEAD` equals the pin, and only then reads or cites Pekko.

## 1. Objective

A System created through the public API with a caller-supplied clock reads time and arms the Ask deadline on that clock, and the deadline fires only when the clock reports it due. The default (no clock supplied) stays the wall clock and the observable behavior of Ask with it does not change. Nothing else: no scheduler, no timers, no ManualTime.

## 2. Base state, verified (READ)

| Fact | Where |
|---|---|
| `clock`, `timer`, `realClock` are unexported | `pekko-actor/actor/clock.go:8-24` |
| The only way to set the clock is the unexported `options.clock` (test seam) | `system.go:29`; `newSystem` defaults nil to `realClock{}` at `system.go:146-153` |
| `Config` has `Name`, `Throughput`, `GuardianStrategy`, `Logger` only; `NewSystem(cfg Config)` takes no options | `system.go:69-90,128` |
| **Real consumers of the clock inside `pekko-actor`:** Ask reads `sys.clock` for the precheck, the remaining time and `AfterFunc` (`ask.go:58,149-176`); **`StartTime` and `Uptime` also read it** (`system.go:139,144`, SYS-03 #1013). The issue text says "only Ask uses it"; that is stale for `Now`. | READ |
| Ask calls `t.Stop()` once after the Ask settles and ignores its result | `ask.go:70-83` |
| **Real consumers outside `pekko-actor`: none.** No `.go` file of `pekko-actor-typed`, `pekko-projection`, `pekko-persistence*` or `pekko-observability` references a clock or `AfterFunc` (grep, non-test). | READ |
| The fake clock for internal tests exists only in `clock_internal_test.go`; many internal Ask tests use `options{clock: fc}` | READ |

### Dependencies: real versus historical

| Item | State on 2026-10-28 | Needed by this scope? |
|---|---|---|
| EXP-1 (Owner decision, accepted 2026-10-27, `docs/parity/typed-owner-decisions-2026-10-27.md:32`) | accepted | **yes, the rule that applies.** It says an export is justified in the story that introduces it (symbol, consumer, why the public API is not enough, what it does not expose, its test), that #1020 blocks no story and that it "approves no specific export". |
| #1021 (GOV-03, form of the justification) | **open**, `status:ready`, no merged deliverable (PR #1210 merged ADR 0034/0035, not GOV-03) | **No.** It is "undelivered" as the label says, but EXP-1 already fixed the form it was asked to propose. The `depends_on` row and `status:blocked` in #1129 are stale on this point. This scope does not close #1021 or relabel anything. |
| #1020 (GOV-02, index of records) | open | No. Blocks nothing (EXP-1). The record from section 6 is to be indexed there later. |
| ADR 0011 owner decision 2 | accepted | **yes**: each new export is individually justified. |
| ADR 0033 D3, D5, D13 | accepted | yes (D5: impact list below; D13: no oracle for a Go seam, see section 7) |
| ADR 0015 D9 (time source) | **proposed** ADR, label `[Proposal]` | consulted only: the internal clock exists and Ask uses it. It does not authorize the export. |
| #776 TIM-01, #821 TIM-02, #822/#823, #860, M-04 ManualTime, judge controlled mode | open, later | **No.** They are *future consumers*, listed in the story as motivation. They are not dependencies and justify no export beyond what section 5 needs. |
| Sibling squads: #814 (B), #1098 (C) | in flight | No logical dependency; file overlap in section 8. |

## 3. Minimal surface (proposed contract; signatures only, nothing is written in the repo)

```go
// package actor (pekko-actor/actor/clock.go)
type Clock interface {
    Now() time.Time
    AfterFunc(d time.Duration, f func()) Timer
}
type Timer interface {
    Stop() bool // reports whether it prevented the call
}

// package actor (system.go)
type Config struct {
    // ...existing fields...
    Clock Clock // nil means the wall clock
}
```

Each name, with the EXP-1 record the Dev copies into the PR (the symbol, the consumer, why the public API is not enough, what it does not expose, its test):

| Export | Consumer (real) | Why the public API is not enough | Does not expose | Test |
|---|---|---|---|---|
| `Config.Clock` | the test author who builds a System from outside the module (the story's user); the System itself reads it for Ask, `StartTime`, `Uptime` | `options.clock` is unexported and `NewSystem` accepts no options | the executor, the sink and every other test seam stay in `options` | AC-R1-2, AC-R2-1 |
| `Clock` | same | a caller-supplied value needs a named type; `Now` and `AfterFunc` are exactly what the System calls | no scheduler API, no `Since`, no `Sleep`, no ticker | AC-R2-1 |
| `Timer` | same | `AfterFunc` returns a value whose type an outside package must name to implement `Clock`; a method returning an unexported type cannot be implemented outside the package | only `Stop() bool` | AC-R3-1 |

Deliberately **not** exported: a wall-clock constructor (the zero value is the wall clock; no consumer needs the value), a manual clock (a double in the test file; ManualTime is M-04), `Reset`, `Since`, any scheduler. Each of them would need its own EXP-1 record and has no consumer today. Stated honestly: **no module in the repository consumes these names yet**; the consumer is the test author named in the story, and the later timer stories are motivation, not consumers. EXP-1 allows justifying that, and approves no export, so the Owner decides (section 9, D2).

Internal names: to keep `ask.go`, the internal tests and the many `options{clock: fc}` call sites untouched, `clock.go` keeps `type clock = Clock` and `type timer = Timer` (unexported aliases of the exported interfaces; they expose nothing). `options.clock` stays as the internal test seam; `newSystem` takes `cfg.Clock` first and falls back to `o.clock`, then to `realClock{}`. The alternative (delete `options.clock`, migrate every internal test to `Config.Clock`) rewrites many test files for no behavior and is rejected for the budget and collision reasons in section 8.

## 4. Observable outcome and limits

- A System with `Config.Clock` set: `Ask` computes its remaining time with that clock's `Now`, arms with its `AfterFunc`, and returns `ErrAskTimeout` only when that clock fires the callback. `StartTime`/`Uptime` follow it.
- Zero `Config.Clock`: identical to today.
- Ask calls `Timer.Stop` exactly once per armed timer after the Ask settles and ignores its result (current behavior, kept).
- Limits: the restart window uses `options.nanotime`, not this clock (judge.md section 5); a manual clock cannot drive it. No timer outside Ask is armed anywhere. A `Clock` whose `AfterFunc` returns a nil `Timer` is a contract violation and is not defended (current code derefs it; not changed).

## 5. Criteria → scenarios

The issue's ACs are kept; the evidence column states what really distinguishes correct from incorrect. Tests use go-specs and spies, no sleeps, no polling (channels and gates). All public tests live in a new `package actor_test` file with a caller-written manual clock double.

| Criterion | Scenario (public API unless noted) | Distinguishes | Level |
|---|---|---|---|
| AC-R1-1 | Existing `ask_api_test.go` scenarios with a short real deadline and no reply still return `ErrAskTimeout` with no `Config.Clock`. No new test, they must stay green. | default stays wall clock | unit |
| AC-R1-2 | System with a manual clock whose `Now` is far from `time.Now()`: a context deadline expressed on that clock is not yet due, a deadline equal to its `Now` returns `ErrAskTimeout` without calling `build`. Also `System.StartTime()` equals the double's initial `Now`. | a System that ignores the seam sees the wall clock and fails both | unit |
| AC-R2-1 | Ask with a 1s deadline on the manual clock, spy actor that never replies: advance 999ms, `ErrAskTimeout` not yet returned (observed through a gate, not a sleep); advance 1ms, it returns. | firing early or never | unit |
| AC-R2-2 (added) | Reply wins: after the reply, the double records exactly one `Stop` call; advancing past the deadline afterwards changes nothing (no second outcome, the late reply is the dead letter). | the System stops its timer exactly once | unit |
| AC-R3-1 | Reframed. The `Stop() bool` contract belongs to each `Clock` implementation; a public test of a double the test itself wrote proves nothing. So: (a) the **real clock** half stays in the existing internal test `clock_internal_test.go` (`TestRealClockFiresOnceAndStopPreventsIt`: `Stop` is true before the call and false after, the callback does not run), kept green; (b) through the public API, the System's use of `Stop`: AC-R2-2 above. | n/a (the real clock is not exported) | unit (internal + public) |
| AC-R4-1 | The three records of section 3 in the PR body and `scripts/check_deps.sh` green. | | doc + lint |

If the Owner prefers that AC-R3-1 be tested publicly against the real clock, the only way is to export a wall-clock value, which this scope does not propose (section 9, D3).

## 6. Allowed files

```
pekko-actor/actor/clock.go
pekko-actor/actor/system.go
pekko-actor/actor/clock_api_test.go
pekko-actor/README.md
docs/adr/0011-pekko-actor-layout.md   (only if the Owner asks for the record there; default: the PR body)
```

`ask.go`, `clock_internal_test.go` and the internal Ask tests are **not** touched (aliases, section 3). Nothing else may change.

## 7. ADR impact (ADR 0033 D5, before implementing)

| ADR | Effect |
|---|---|
| 0011 owner decision 2, EXP-1 | three new exports, one record each (section 3); no accepted contract changes, so **no new ADR** (EXP-1) |
| 0033 D3 | consumed; D3's Typed module needs no export here |
| 0033 D13 | no Pekko oracle applies to a Go seam (the Pekko analogue, `ExplicitlyTriggeredScheduler.scala:44,67`, is a scheduler, out of scope). Alternative evidence: the public tests of section 5. Recorded as **pending Owner approval**. |
| 0015 D9 | proposed; no contradiction (the internal clock exists and Ask uses it); does not authorize the export |
| 0010 | proposed; lists an injectable clock as a requirement; does not authorize the export by itself |
| 0017 / 0024 T6 kit run | **Not required**: this scope ports no Pekko behavior and adapts no Pekko source file; it exports an existing Go interface. If the Owner reads `ExplicitlyTriggeredScheduler` as an adapted source, say so in D1 and the Dev runs the kit. |
| Dependency direction | `actor` imports nothing from goPekko; unchanged (`scripts/check_deps.sh`) |

## 8. Collisions and independence

| Scope | Files | Overlap with this scope |
|---|---|---|
| #1129 (A) | `clock.go`, `system.go`, `clock_api_test.go`, `pekko-actor/README.md` | |
| #814 (B) | `cell.go`, `context.go`, `ref.go`, README | **README only** |
| #1098 (C) | `internal/mailbox/*` | none |

- `system.go`: a Config field plus three lines in `newSystem`; the same file is not in B's or C's list. If B or C also edit `Config` or `newSystem` the Dev rebases; no function is moved.
- `README.md`: the one shared file. Mitigation: edit in two anchored places only: a new short section next to the `StartTime`/`Uptime` text (~line 260) and the sentence "clock.go holds the unexported `clock` seam" in the file map (~line 705) plus the two stale mentions (~lines 269, 443). **Sequencing:** this scope's README commit is the last one; it rebases on whichever of B/C merged first and does not reflow any paragraph.
- No dependency on a shared central point beyond that; scope is not reduced.

## 9. Owner decisions (D1 to D3 decided in section 0; D4 open)

The alternatives below stay as the record of what was offered. Decided: D1 = 1, D2 = 1, D3 = 1. Open: D4.

**D1. Seam form** (the story's open choice 1; ADR 0011 decision 2). Single choice:
1. `Config.Clock` field, `NewSystem(cfg)` unchanged. *Recommended*: additive, no signature change, matches how `Logger` and `GuardianStrategy` are supplied.
2. A functional option, `NewSystem(cfg Config, opts ...Option)`. Changes the signature of an existing export and opens a new family of options with no other user.
3. Decline; the seam stays unexported.

**D2. Names and `Timer`** (open choices 1 and 2). Single choice:
1. Exported `Clock` and `Timer` interfaces as in section 3. *Recommended*: the interface method must return a named type to be implementable outside the package.
2. `Clock` only, with `AfterFunc` returning `interface{ Stop() bool }`. Same surface, no `Timer` name (avoids a collision with later scheduler names, the non-blocking risk of the story), but the interface type is anonymous in a public signature.
3. Different names (state which).
4. Decline.

**D3. Manual clock and the real-clock public test** (open choice 3). Single choice:
1. Export nothing else; the manual clock is a test double, ManualTime waits for M-04; the real-clock `Stop` test stays internal. *Recommended*.
2. Also export a manual clock from `pekko-actor` (needs its own EXP-1 record and a consumer; none exists before M-04).
3. Also export a wall-clock constructor so AC-R3-1 can be public (a new export without a consumer).

**D4. ADR 0033 D13.** Approve the public scenarios of section 5 as alternative evidence (no Pekko oracle for a Go seam), or require something else.

Also for the Owner, outside this scope: the `status:blocked` label and the `GOV-03 depends_on` row of #1129 look stale because EXP-1 settled the form; the TL changes neither.

## 10. Appetite and size

~35 lines of production (`clock.go` doc and exports, `system.go` field and fallback), ~110 of tests, ~30 of README, ~25 of execution record and notes: about 200 authored changed lines, within the 400 budget (`AGENTS.md`). The plan row C-01 estimates 120-180. No split. If the record or README grows past 400 in total, the README goes to its own PR after this one.

## 11. Definition of Done

- Every criterion of section 5 has its scenario, passing, and the existing Ask tests are green unchanged.
- The three EXP-1 records are in the PR body; ADR impact of section 7 reflected; D1-D4 recorded as accepted before the first commit.
- Unit tests: `scripts/each_module.sh go test -count=1 ./...`; `scripts/check_deps.sh`; coverage 85% per module and diff coverage 85%; gocyclo at most 10 (commands in `AGENTS.md`).
- `pekko-actor/README.md` documents the seam, its default, its limits (restart window, no timers) and what is not implemented.
- No `ask.go` change and no behavior change with the default clock; `go.mod` of `pekko-actor` unchanged.
