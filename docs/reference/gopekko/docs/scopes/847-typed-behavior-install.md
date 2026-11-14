# Scope: install and replace a Typed receive Behavior (#847, TYP-02)

Status: **approved by the Owner at gate G2 on 2026-10-27** as the text of commit `9310063` (this file differs from that commit only in this status line). Written by the TL on 2026-10-27 against `develop` `108880b0100c11ae838a772c23af952e5a54b827`
(the tree of #1225). Nothing here is implemented. **Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: this scope
ports Pekko behavior, so the Dev runs the migration kit with its own manifest, RULEBOOK and RUNBOOK; loading the kit's rules does not count.

## What this delivers

`pekko-actor-typed` has no code yet. This story adds the first code: a Go developer describes an actor as a `Behavior[T]` value, runs it
on the existing `pekko-actor` runtime, and every processed message returns the behavior that handles the next one. In Pekko this is
`Behaviors.setup` (`Behaviors.scala:39`) and `Behaviors.receive` (`:114`), interpreted by `Behavior.start` and `Behavior.canonicalize`
(`Behavior.scala:198`, `:184`). In Go (names are indicative; the shapes are the contract, section "Contract produced"):

```go
var counter func(n int) typed.Behavior[string]
counter = func(n int) typed.Behavior[string] {
    return typed.ReceiveMessage(func(msg string) (typed.Behavior[string], error) { return counter(n + 1), nil })
}
start := typed.Setup(func(*typed.Context[string]) (typed.Behavior[string], error) { return counter(0), nil })
ref, err := actor.Spawn(sys, "counter", typed.NewActor(start)) // an ordinary pekko-actor Ref[string]
```

The observable result is three guarantees: `Setup` runs once when the actor starts and not again; the behavior returned for message *n*
handles message *n+1* and later; and neither `Same` nor a replacement changes the actor's `Ref` or its mailbox.

## What this does not deliver

Each item has its own story, so none of it may enter this diff:

- `Unhandled`, `Empty`, `Ignore` (#925, #1181) and `Stopped` / `StoppedWith` (#1182). `Unhandled` is left out because its meaning is
  "record the message through the module-local hook" (ADR 0034 P7, #925); shipping the marker alone would ship half of that row.
- Signals: `PreRestart`, `PostStop`, `Terminated`, `ChildFailed`, `MessageAdaptionFailure`, `ReceiveSignal` and the death pact (#954). They need
  the construction of `Terminated{Ref}` and `ChildFailed{Ref, Cause}`, which is still the open Owner decision B-2.
- `receivePartial`, `receiveMessagePartial`, `receiveMessageWithSame` and a signal-only behavior (#1178). Plain `receive`, `receiveMessage` and
  `setup` belong here, as #1178's own out-of-scope list says.
- Typed supervision, including Typed's default Stop (#956, #976, #1183, #1184), `Props` (#1174), the Typed System (#994), context operations
  beyond `Self` (#1170, #1171, #1179, #926), stash, timers, the event stream, message adapters, interceptors and `Extend`.
- The nil message on `Tell` (X2, open decision B-3, #452). `Tell` here is the runtime's own.

## Dependencies, as they really are

| Dependency | State today | Effect on #847 |
|---|---|---|
| #1096 (TYP-34) | closed as completed, integrated as `108880b` | Delivered: the module and its two placeholder packages exist |
| #809 (TYP-01) | still open; ADR 0034 is published and **partly accepted** by Owner decision B-1 (2026-10-27) | #847 consumes only the accepted parts listed below. #809 is not declared delivered and ADR 0034 is not declared accepted in full |
| #1020 (GOV-02) | open, still named in #847's body | **Not a blocker.** EXP-1 (accepted 2026-10-27) says #1020 blocks no story, and this story adds no `pekko-actor` export (checked below) |
| #1021 (GOV-03) | open | Not a dependency of #847 |
| ADR 0035, S-1, S-6 | partly accepted | Not consumed: supervision belongs to #956 and later |

The `Owner decisions` list of 2026-10-27 (section 5) states what #847 needs: "B-1, EXP-1, #1096". All three are satisfied.

## Contracts consumed, with their acceptance state

| Contract | Source | State | Used for |
|---|---|---|---|
| Typed runs on the shared runtime; A3 stays | ADR 0033 D1 | accepted 2026-10-24 | one interpreter actor, no second runtime |
| Separate module, depends only on `pekko-actor` | ADR 0033 D3 | accepted; wired by #1096 | where the code lives |
| Evidence rule for V3 and rows without oracle | ADR 0033 D13 | accepted | the "Evidence" section |
| P1 one interpreter actor; the Typed ref is the native `actor.Ref[T]` | ADR 0034 | accepted (B-1) | `NewActor`, `Self()` |
| P2 sealed `Behavior[T]`; handlers `func(ctx, msg) (Behavior[T], error)`; special behaviors are zero-size values | ADR 0034 | accepted (B-1), names indicative; interceptors deferred | `Setup`, `Receive`, `ReceiveMessage`, `Same` only |
| P3 start: run every `Setup` in `PreStart`, then validate; `Same` is an invalid initial behavior | ADR 0034 | accepted (B-1) | criterion 1 and its start failures |
| P4 turn rows `Same`, `Setup`, "any other", `(nil, nil)`, error or panic | ADR 0034 | accepted (B-1) | criterion 2 |
| P8 a failed turn hands its cause to supervision | ADR 0034 | accepted (B-1) | the failure scenarios |
| X1 failures are returned errors, X3 sealed `Behavior[T]`, X5 no `failed(cause)` value | ADR 0034 | accepted (B-1) | the API shape |
| EXP-1 an export is justified in the story that adds it | Owner list | accepted 2026-10-27 | none needed here |

Not consumed, and still undecided where noted: P5 (construction open, B-2), P6 (needs P5), P7 (needs `Unhandled`), X2 (B-3 open), X4 and X6
(B-4 open), B-5, and every row of the `Unhandled` and `Stopped` result in P4.

The `pekko-actor` public API this story uses, all exported today, so **no `pekko-actor` file changes and EXP-1 requires no record**:
`actor.Actor[T]` and `actor.PreStarter[T]` (`actor.go:19`, `:27`), `actor.Context[T].Self()` (`actor.go:50`), `actor.Ref[T]` (`ref.go:18`),
`actor.Spawn` (`spawn.go:85`, used by callers and tests), `*actor.InitializationError` (`supervision_hooks.go:22`), `*actor.PanicError`
(`supervision.go:118`), and for tests only `actor.Config.GuardianStrategy` with `actor.StoppingGuardianStrategy` or `actor.GuardianOneForOne`
(`guardian.go:41`, `:65`).

## Contract produced

New public package `github.com/getsyntegrity/gopekko/pekko-actor-typed/typed`, one package, with the interpreter unexported in it (the sealed
`Behavior[T]` needs the interpreter in the same package). Only these symbols are exported; any other export needs a stop and a report.

```go
type Behavior[T any] interface{ /* sealed: unexported method */ }

type Context[T any] struct{ /* wraps *actor.Context[T]; valid only during the handler call */ }
func (c *Context[T]) Self() actor.Ref[T]

func Setup[T any](factory func(ctx *Context[T]) (Behavior[T], error)) Behavior[T]
func Receive[T any](onMessage func(ctx *Context[T], msg T) (Behavior[T], error)) Behavior[T]
func ReceiveMessage[T any](onMessage func(msg T) (Behavior[T], error)) Behavior[T]
func Same[T any]() Behavior[T]
func NewActor[T any](initial Behavior[T]) func() actor.Actor[T] // pass it to actor.Spawn; each call builds a fresh interpreter

var ErrNilBehavior, ErrInvalidInitialBehavior error
```

`NewActor` runs nothing. The interpreter implements `actor.Actor[T]` and `actor.PreStarter[T]` only; it does not implement `PostStopper`,
`TerminatedHandler` or `PreRestarter`, which belong to #954 and #956. `Receive` and `ReceiveMessage` return plain `Behavior[T]` now; #954 adds
`ReceiveSignal` without breaking callers.

## Criteria → scenario → Pekko source

The issue's wording is kept unchanged in the issue. Two sentences are ambiguous, so this table restates them; the restatement needs the Owner's
confirmation at G2 (as the Owner did for #827), see decision D-1. "Level" is unit-level: the runtime is real, no double, no integration test.
Evidence labels follow ADR 0034: SPEC is a Pekko spec whose body was read (none was run), READ is Pekko code only.

| # | Criterion (restated) | Scenario in `pekko-actor-typed/typed/` | Pekko source | Evidence |
|---|---|---|---|---|
| 1 | `Setup` runs once, when the actor starts and before its first message; building the behavior runs nothing; `Same` and a replacement never run it again. One start of the cell is one incarnation; what a runtime restart does is #976/#1183 | S1 `NewActor` and the factory call run nothing; spawning runs setup and a witness arrives before any `Tell` | `Behaviors.scala:39`; `Behavior.scala:198`; `ActorAdapter.scala:298-305` | SPEC `DeferredSpec.scala:51` |
| 1 | same, ordering boundary | S2 messages told while setup is held on a gate are handled only after setup returns, in order | `ActorAdapter.scala:298-305` | READ |
| 1 | same, nested | S3 `Setup` returning `Setup` starts both, each once | `Behavior.scala:198-212` | SPEC `DeferredSpec.scala:99` |
| 1 | same, never again | S4 after N messages, including replacements and `Same`, the setup counter is still 1 | `Behavior.scala:184-192`; `ActorAdapter.scala:165-178` | READ, with `DeferredSpec.scala:51` indirect |
| 1 (negative) | a start that returns `Same` or a setup that fails is a start failure, not a running actor | S5 setup returning `Same` fails with `ErrInvalidInitialBehavior`; S6 setup returning an error fails; both as `*actor.InitializationError`, no message processed, actor stopped | `Behavior.scala:235-243`; `ActorAdapter.scala:302` | SPEC `DeferredSpec.scala:147` (`setup(same)`) and `:63` (factory exception) |
| 2 | The behavior returned for message *n* handles message *n+1* and later, until another result replaces it; message *n* is unaffected | S7 B0 handles m1 and returns B1; m2 and m3 are handled by B1; `Same` keeps the current behavior | `Behavior.scala:184-192`; `ActorAdapter.scala:167-178` | SPEC `BehaviorSpec.scala:242` (two messages) and `:349-381` with `:285-289` (swap) |
| 2 | a returned `Setup` is started at once and the next message goes to its result | S8 handler returns `Setup(...)`; the next message reaches the started behavior | `Behaviors.scala:32-37`; `Behavior.scala:187-190` | READ |
| 2 (negative) | a failed turn installs nothing | S9 `(nil, nil)` fails with `ErrNilBehavior`; S10 a returned error and a panic fail with that cause, the returned behavior is ignored, and under a test `Resume` decider the previous behavior still handles the next message; the cause reaches the decider (P8) | `ActorAdapter.scala:125-141,167-178` (behavior assigned only after `interpretMessage` returns) | READ; Pekko's null result is an `NullPointerException`, Go's is `ErrNilBehavior` (ADR 0034 B12) |
| 3 | `Same` and a replacement keep the actor's `Ref` and mailbox | S11 `Self()` equals the `Ref` from `Spawn` before and after; messages queued before a replacement (held on a gate) are each handled once, in order, by the behavior current when each is taken | `ActorContextSpec.scala:535`; `BehaviorSpec.scala:301-303` (`GetSelf` after `Swap`); one classic actor per Typed actor (ADR 0034 B19) | SPEC for the `Ref` (indirect, through `BehaviorTestKit`); **READ for the mailbox** |
| 4 | Focused positive and negative cases, the ordering boundary named, the candidate SHA and the applicable gates recorded | S1 to S11 above; ordering boundaries are S2 and S11; the SHA and gates go in the PR comments and the execution record | | |
| 5 | Public contract and traceability updated; no sibling or parity claim | `pekko-actor-typed/README.md` and the root `README.md` state cell | | review |

A scenario must observe its result, not infer it from a sleep: gates are channels, and `-race` is not used.

Edge left open on purpose (`Unknown is an answer`): a `Setup` returned by a handler whose factory returns `Same`. Pekko leaves an unstarted deferred
behavior as the current one and fails the next turn (`Behavior.scala:187-190,294-295`; no spec found). The most conservative Go form is to fail the
turn that returned it with `ErrInvalidInitialBehavior`. Leave `// TODO(TYP-02-edge): timing differs from Pekko` at that line and record it in the
execution record as a gap; write no parity claim for it.

## Allowed files

Exactly these paths. Anything else needs a stop and a report to the TL.

| Path | Edit | PR |
|---|---|---|
| `docs/scopes/847-typed-behavior-install.md` | this file | 1 |
| `migration/stage-1/typed-847-behavior-install/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | new kit package: every Pekko spec row ported, adapted, moved or `PORT-GAP` | 1 |
| `pekko-actor-typed/typed/behavior.go`, `context.go`, `interpreter.go` | new | 2 |
| `pekko-actor-typed/typed/*_test.go` (a file derived from one Pekko spec carries that spec's single `Ported from` line; Go-only evidence goes in files with none) | new | 2 |
| `pekko-actor-typed/typed/example_test.go` | new executable example for the README | 2 |
| `pekko-actor-typed/go.mod`, `go.sum` | the `require` of `pekko-actor` (as `pekko-persistence/go.mod` does, with the existing `replace`) and of `go-specs` for tests; `go.sum` is new | 2 |
| `pekko-actor-typed/README.md` | implemented versus planned, usage, limitations, the interim failure policy, link to the example | 2 |
| `README.md` (root) | the "State" cell of the `pekko-actor-typed` row only (it says "Scaffold only") | 2 |
| `migration/stage-1/typed-847-behavior-install/EXECUTION-RECORD.md`, `migration/cost-log.tsv` (one row) | commands, kit version `cf91c9d`, models, inputs, outputs, manual changes; gate evidence bound to a SHA | 3 |

Nothing under `pekko-actor/`, `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md` or `AGENTS.md` changes. The `spi/` and `internal/` placeholders of
#1096 stay as they are. Files ported from Pekko carry the Apache-2.0 notice and exactly one `// Ported from upstream/pekko/<path>.scala` line:
`behavior.go` from `actor-typed/.../typed/Behavior.scala`, `interpreter.go` from `actor-typed/.../typed/internal/adapter/ActorAdapter.scala`;
`context.go` is new Go code and has no line. `.github/scripts/check-headers.sh` checks it.

There is one scope, so the independence table is trivial: no other scope shares these files. #925, #954, #1178 and #1182 will add files to
`pekko-actor-typed/typed/` after this one; they depend on #847, not the other way round.

## ADR impact and contradictions (ADR 0033 D5), before implementing

1. **ADR 0034 "Consequences" says #847 "can implement P1 to P4 and P6".** This scope leaves P6, and the `Unhandled` and `Stopped` rows of P4, to
   #954, #925 and #1182. The sentence is permissive, P6 needs P5 whose construction (B-2) is open, and none of #847's criteria names a signal. No ADR
   text changes in this story; a later docs pass can reword that sentence.
2. **Typed default Stop (ADR 0033 D2, accepted) versus the runtime's default Restart (ADR 0017, `supervision.go:103-110`).** Until #956, a Typed
   actor whose turn fails is decided by the runtime: `/user` restarts it, the factory runs again and the new interpreter starts from the initial
   behavior, with `Setup` run again. That is the Classic default, not a Typed contract. The module README states it as a limitation, and the tests
   install an explicit decider instead of depending on it. #956 depends on #847, so it cannot come first.
3. **ADR 0010 keeps `become` excluded.** No contradiction: replacement happens inside the interpreter (ADR 0034 P1); the runtime gains no `become`.
4. **ADR 0011 / EXP-1.** No `pekko-actor` export is added, so no per-export record is due. The `typed` package is this module's own surface.
5. **X2 / B-3 (nil on `Tell`) is open.** The README says nil acceptance is an open divergence (#452) and no test relies on a nil message.
6. **INV-T row for `Behavior[T]`** says "oracle: pending determination". This scope makes the determination per row (the Evidence column above).

No ADR is amended by this story.

## Evidence and validation

**Migration evidence (kit run).** The kit package derives the Go tests from the Pekko specs named above: `DeferredSpec.scala:51,63,99,147`,
`BehaviorSpec.scala:242,285-289,301-303,349-381` and `ActorContextSpec.scala:535`. The `canonicalize behaviors` test of `ActorContextSpec.scala:100`
includes an `unhandled` step, so its manifest row is moved to #925 rather than dropped. The execution record lists commands, kit version, models,
inputs, outputs and manual changes. Pekko has been read, not run: no JVM run is part of this scope.

**Level reached, honestly.** Go unit evidence derived from Pekko specs. It is **not V3**: V3 needs a differential scenario against Pekko, and no
Typed Pekko harness exists yet (ADR 0033 "Consequences"). The rows marked READ are rows where the TL found no Typed spec: the TL read
`DeferredSpec`, `BehaviorSpec` and `ActorContextSpec`, and `grep InvalidMessageException` finds only null-*message* tests
(`ActorContextSpec.scala:665`), none for a null returned behavior. The TL did **not** search the rest of `actor-typed-tests` for a handler that
returns a `Setup`, or for a test of mailbox identity, so "no spec found" is not yet established for those two rows: the RULEBOOK must run and
record that search first, and a spec it finds replaces the READ label and becomes the oracle. For rows that stay READ, ADR 0033 D13 makes
their Go tests alternative evidence that stays **pending approval** by the Owner. That does not block delivery of this story; it blocks any
V3 or parity claim.

**Local gates, proportional** (commands live in `AGENTS.md`, "Go"): unit tests with network isolation for `pekko-actor-typed`, `go vet`,
`gofmt -l`, `golangci-lint` with `gocyclo` at most 10, `go mod tidy -diff`, coverage of the module at least 85% (the module now has executable
code, so its row stops being "skipped"), diff coverage at least 85%, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh`, and
`go-sdk-update.sh --check`. Run once per final tree; a gate whose tree, command and environment are identical to a published run is cited, not
repeated. `pekko-actor` is untouched, so its tests are not re-run. No `-race`, no workbench, no campaign. Use go1.27.2 and the
`golangci-lint` rebuilt in `bin/` at `.golangci-version`, as for #1096.

**Remote CI.** The exception that let #1225 merge without remote CI covered #1225 only. This scope grants no merge and no such exception; if
remote CI still cannot run, the PR says so and the Owner decides.

## Delivery, size and tasks

One story, delivered as three small PRs because the 400 authored-line budget cannot hold the kit package, the code with its tests, and the
execution record together. This follows the precedent of U9 (#476 execution package, #477 code) and T6 (#738, #739, #743). Draft PRs, body
starting `Refs #847` (never `Closes`), Conventional Commits, normal pushes, no merge. #847 stays open.

| Task | Result | Check | PR |
|---|---|---|---|
| T1 | Kit package: manifest, RULEBOOK, RUNBOOK; this scope (244 lines) ships with it | every spec row above has a disposition; RULEBOOK is read-only afterwards | 1 (the kit files must stay under about 150 lines to fit 400) |
| T2 | `Behavior[T]`, `Context[T]`, `Setup`, `NewActor`, the start path and its errors | S1 to S6 pass | 2 |
| T3 | `Receive`, `ReceiveMessage`, `Same`, canonicalize, failed-turn handling, returned `Setup` | S7 to S10 pass | 2 |
| T4 | Identity and ordering evidence S11, the executable example, README and root README | S11 passes; README meets the module README contract | 2 |
| T5 | Execution record, cost-log row, gate evidence bound to the final SHA | each gate comment names the SHA, command, environment, exit code | 3 (about 100 lines) |

PR 2 is projected at 400 to 450 authored lines and is the risk. **Stop condition:** if the Dev projects PR 2 over 400, stop and report; the TL
then moves T4 (S11, the example and the README text that cites them) into PR 3. That cut falls on a behavior boundary: identity evidence does not
change how install and replacement work. A scope that needs more than these five tasks is wrong: stop and report, do not grow it.

Other stops: a file outside the allowed list must change; `pekko-actor` needs an export; a scenario fails and is not a Go defect (run it on Pekko
before classifying, per `AGENTS.md`); go1.27.2 cannot be obtained. Never skip, delete or weaken a test to make a failure go away.

## Decisions for G2

**Blocking decisions: none.** Every contract this story consumes is accepted or delivered. The items below need the Owner's eyes at G2 but do
not stop the Dev, because Pekko decides each one if the Owner is silent (kit rule 7).

- **D-1, wording of criteria 1 and 2** (needed by criteria 1 and 2). The issue says "setup executes once for the accepted incarnation" and "a
  Receive result becomes active for the next message only". The second can be read as "reverts after one message", which contradicts Pekko.
  Alternatives: (a) the restated criteria above, faithful to Pekko; (b) keep the literal sentences and let QA interpret; (c) make "incarnation"
  include a restart, which pulls ADR 0035 into this story. **Recommend (a).**
- **D-2, boundary: no `Unhandled`, no P6** (needed by the issue's reconciliation note, which names `unhandled`). Alternatives: (a) leave both out
  (recommended); (b) ship `Unhandled` as an inert marker that keeps the behavior and records nothing, which duplicates half of #925 and invites a
  parity claim; (c) ship P6 as ADR 0034's Consequences line suggests, which needs the open B-2.
- **D-3, public shape** (criterion 2 and 3 need some entry point and context). `typed` package, a `Context[T]` wrapper exposing only `Self()`, and
  `NewActor` as a factory for `actor.Spawn`. Alternatives rejected: a type alias of `actor.Context[T]`, because Go cannot add Typed-only methods
  to a type of another module, which #1170, #1171 and #1179 will need; a Typed `Spawn` wrapper, which is #1170, #994 and #1174. Inside P2 and
  the Owner list section 4, so it needs no approval; shown so it can be overridden.
- **D-4, interim failure policy** (criterion 2 negative cases). Until #956 a failing Typed actor follows the runtime's Restart default. Alternatives:
  (a) accept and document it (recommended); (b) make the interpreter stop on failure, which contradicts P8 and ADR 0035; (c) wait for #956,
  impossible because #956 needs #847.
- **D-5, three PRs** because of the budget and the kit precedent. Alternatives: one PR (breaks the 400 budget); drop the kit package (breaks the
  `AGENTS.md` rule that a port runs the kit).
- **D-6, kit artifacts in `migration/stage-1/typed-847-behavior-install/`**, since ADRs 0033, 0034 and 0035 name Stage 1 for the Typed profile.
- **Labels.** #847 keeps `status:blocked` until G2. After approval the TL proposes `status:ready`; this scope changes no label.

The orphan to watch: `Behaviors.receiveMessage` (`Behaviors.scala:133`) is in this scope and `receiveMessageWithSame` (`:153`) is in #1178.
Nothing is orphaned.

## Definition of Done

- S1 to S11 each pass at the final SHA, and T2 to T4 each meet their check.
- Gates of "Evidence and validation" are green with evidence bound to that SHA; no function above complexity 10.
- The kit execution record exists and states what was read, not run, and which rows are READ and pending approval.
- Module README and root README match the code; no sibling story and no parity is claimed.
- A new QA session, independent of the TL and the Dev, issues a verdict on the exact head of each PR.
- No merge without the Owner. #847 stays open until its criteria are met and the Owner closes it.
