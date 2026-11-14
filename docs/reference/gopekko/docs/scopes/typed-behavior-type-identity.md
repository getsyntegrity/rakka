# Scope: make `Behavior[T]` carry its protocol type (fix to #847, TYP-02)

Status: **G2 approved by the Owner** for this scope only ("limited to this fix and its tests"). Written by the TL on 2026-10-28 against
`develop` `1515c55`. **Context only: #847 is not reopened and no issue is created.** This is a defect found in code that #847 delivered
(`pekko-actor-typed/typed/behavior.go`, PR #1227 to #1229 stack). It is not a port of new Pekko behavior, so no kit run is required; the
Pekko contract it protects was read, not executed (see "Evidence").

## The problem, confirmed

`Behavior[T]` is declared as `interface{ behavior() }` (`pekko-actor-typed/typed/behavior.go:29`). The private method never mentions `T`, so
`Behavior[string]` and `Behavior[int]` have the same method set. In Go an interface value is assignable to any interface type whose methods
it has, so the compiler accepts a message-protocol mix with no conversion. The review finding was a reading of the code; the TL compiled it.

Reproduction (a throwaway module in `/tmp` with `replace` directives onto this worktree, production code untouched, offline):

```go
s := typed.Receive(func(_ *typed.Context[string], m string) (typed.Behavior[string], error) { return typed.Same[string](), nil })
var i typed.Behavior[int] = s          // (1) compiles
_, err := actor.Spawn(sys, "mix", typed.NewActor[int](s)) // (2) a real Receive[string] as NewActor[int]'s argument: compiles
```

Command: `GOFLAGS=-mod=mod GOPROXY=off GOWORK=off go build ./... && go vet ./... && go run .` with Go **1.27.2** (`.go-version`, run without
`GOTOOLCHAIN=local`): build and vet exit 0; `go build` also exits 0 with the local 1.27.1 toolchain (`GOTOOLCHAIN=local`). At run time the actor fails to start: the
guardian's decider receives `actor: initialization failed` and `errors.Is(cause, typed.ErrNilBehavior)` is true. The interpreter's
`start[int]` type-switches on `receiveB[int]`, does not recognise a `receiveB[string]`, and falls into its `default` branch. A protocol
mistake therefore surfaces as a nil-behavior error at start instead of a compile error. The finding is confirmed, not refuted.

## What Pekko requires, and what Go must therefore reject or keep

Pekko's `Behavior` is `abstract class Behavior[T](...)` with no variance annotation (`Behavior.scala:49`), so Scala treats
`Behavior[String]` and `Behavior[Int]` as unrelated types; the doc on `narrow` calls the relation "contravariant ... which cannot be
expressed directly" and offers the explicit `narrow[U <: T]` (`Behavior.scala:51-56`). The only unchecked cast is `unsafeCast`, which is
`private[pekko]`, internal API (`Behavior.scala:68`). Every public constructor and consumer binds one `T` end to end: `Behaviors.setup[T]`
(`Behaviors.scala:39`), `same[T]` (`:57`), `receive[T]` (`:114`), `receiveMessage[T]` (`:133`), `Behavior.start[T]` (`Behavior.scala:198`)
and the adapter's `spawn[T](behavior: Behavior[T], ...)` (`scaladsl/adapter/package.scala:71`). Even `same`, `stopped` and the other
singletons are produced per `T` through `unsafeCast` inside Pekko (`BehaviorImpl.scala:58-62`), never exposed as a cross-protocol value.

| Use | Pekko | Go after this scope |
|---|---|---|
| Same `T` everywhere (`Setup`, `Receive`, `ReceiveMessage`, `Same`, `NewActor`, handler results) | compiles | **compiles** (kept) |
| Aliases of one type (`type A = string`) | the same type | **compiles** (kept) |
| `Behavior[string]` used as `Behavior[int]`: assignment, `NewActor[int]` argument, `Setup`/`Receive`/`ReceiveMessage` result of the other `T` | does not compile (unrelated types) | **rejected by the compiler** |
| `Behavior[any]` or `Behavior[Stringer]` used as `Behavior[string]`, and `Behavior[string]` used as `Behavior[MyString]` | does not compile without `narrow` | **rejected** (Go has no implicit relation between instantiations) |
| Explicit safe narrowing `narrow[U <: T]` | public, `Behavior.scala:56` | **not available today** and **not added here** (see "Not delivered") |
| Unsafe cast between protocols | internal only (`unsafeCast`) | none exists; none is added |

The Go compiler cannot express Pekko's `narrow` (it needs a subtype relation between type arguments), so this is a missing capability,
not a new divergence introduced by the fix: nothing that worked before is removed, because before the fix the only "working" mixes were
the ones that fail at start. The Owner-level question of whether and how to offer `Narrow` is a follow-up, not decided here.

## Mechanism chosen

Make the private method mention `T`: `type Behavior[T any] interface{ behavior(T) }`, and give `setupB[T]`, `receiveB[T]` and `sameB[T]`
the method `behavior(T)`. Two instantiations then have different method signatures (`behavior(string)` versus `behavior(int)`), so neither
implements the other, while the interface stays sealed (an unexported method can only be satisfied inside the package), the constructors
keep their signatures and the interpreter's type switches (`sameB[T]`, `setupB[T]`, `receiveB[T]`) do not change. `sameB[T]` stays a
zero-size value. The method is never called, so run-time behavior is identical. A different spelling of the same idea, `behavior() T`, was
considered; `behavior(T)` was chosen because the argument form needs no zero value and reads as "accepts only protocol `T`".

Rejected: a struct wrapper `Behavior[T] struct{ impl ... }` would also bind `T`, but it changes the public type, the meaning of a nil
`Behavior[T]` (`ErrNilBehavior`, ADR 0034 B12) and every constructor, which exceeds "fix the marker". Also rejected: a runtime check in
`start`, which keeps the mistake out of the compiler, the opposite of the goal.

An existing property is untouched and stays out of scope: a type in another package that embeds a `typed.Behavior[T]` value inherits the
private method and so can still implement the interface. The interpreter's `default` branch keeps rejecting it as `ErrNilBehavior`.

## ADR impact (ADR 0033 D5), recorded before implementation

| ADR | State | Impact |
|---|---|---|
| 0034 P2, X3 | accepted (B-1, 2026-10-27) | **Consistent, no edit.** P2 says "`Behavior[T]` is a sealed interface (an unexported method)", names indicative; the method still is unexported and `T` still is the only parameter. X3 (sealed, extension through `Extend`) unchanged |
| 0034 B12 / P4 `(nil, nil)` row | accepted | Unchanged: nil results and a nil `initial` still fail with `ErrNilBehavior`; only the cross-protocol way of reaching it disappears at compile time |
| 0034 P3 | accepted | Unchanged: `Same` is still the invalid initial behavior |
| 0033 D1, D3, D13 | accepted | One runtime, module boundary and evidence rule untouched; no `pekko-actor` change, so EXP-1 needs no record |
| 0035 | partly accepted | Not consumed |

No accepted contract changes, so no Owner decision is requested beyond the G2 already given.

## Not delivered (each has its own place)

The interpreter, supervision, signals and the pending `Setup` then `Same` timing case (`interpreter.go`, the `TODO(TYP-02-edge)`), anything in
`pekko-actor`, #956, `Narrow`/`widen`/`transformMessages`, `Extend`, `Unhandled`, `Stopped`. A follow-up for an explicit `Narrow` is named
here so the chain is visible; it is not created.

## Criteria, scenarios, Pekko source

The negative scenarios build small programs with the real `go build` in a temporary module under `t.TempDir()` (a `replace` onto this
module, offline, `GOWORK=off`, `GOPROXY=off`): no framework and no tracked fixture files. The repository has no compile-failure mechanism
today (searched for `compilefail`, `go/types`, `exec.Command("go"` in tracked Go files). Each case is a **pair** that differs only in the
type argument: the `good` twin must build, the `bad` twin must fail. That guards against passing because of a bad import, a syntax error,
a missing module, offline resolution or an unused variable.

| # | Criterion | Scenario (`pekko-actor-typed/typed/behavior_protocol_test.go`) | Pekko source | Evidence |
|---|---|---|---|---|
| 1 | Same-protocol programs still compile: `Setup`, `Receive`, `ReceiveMessage`, `Same`, `NewActor`, alias type, interface `T`, named `T` | `good` twin of every row, plus a combined control built first (proves imports, deps and offline resolution work) | `Behaviors.scala:39,57,114,133`; `Behavior.scala:198` | READ |
| 2 | Cross-protocol use is rejected: assignment, `NewActor[int](Receive[string])`, `Setup`, `Receive` and `ReceiveMessage` results | one `bad` twin each, asserting exactly one error, at the marked line, naming both instantiations and the `behavior` method (`does not implement`) | `Behavior.scala:49` invariant class | READ |
| 3 | Related but distinct types are rejected: `any`/`Stringer` versus `string`; named versus underlying type | `bad` twins `any_to_string`, `named_to_underlying` | `Behavior.scala:51-56` | READ |
| 4 | The contract stays sealed | a package outside `typed` declaring its own `behavior(T)` does not satisfy `Behavior[T]` (passes before and after: a characterization guard, not part of the red/green proof) | ADR 0034 P2 | none needed |
| 5 | Run-time behavior is unchanged | the existing `typed_test.go`, `identity_test.go` and `example_test.go` pass unmodified | n/a | existing tests |

Red/green proof: the criterion 2 and 3 `bad` twins must **fail** on the unfixed `behavior.go` (the program compiles, the test reports it) and
**pass** after the fix. Criteria 1, 4 and 5 pass on both sides. A compiler diagnostic is evidence about Go typing only; it makes **no
claim of parity with Pekko**, whose side is a reading of `Behavior.scala`, not a Scala compile.

## Allowed files

| File | Change |
|---|---|
| `pekko-actor-typed/typed/behavior.go` | the interface and three method declarations, plus a comment on why the method mentions `T` |
| `pekko-actor-typed/typed/behavior_protocol_test.go` | new: the table above |
| `pekko-actor-typed/README.md` | one line in "Public usage" that `Behavior[T]` binds its protocol at compile time, and the `Narrow` gap in "Current limitations" |
| `docs/scopes/typed-behavior-type-identity.md` | this file |

Nothing else may change: not `interpreter.go`, `context.go`, other tests, `go.mod`/`go.sum`, any ADR, any other module. The authored-line
budget is 400 (`AGENTS.md`); this change is well under it.

## Tasks (at most five)

1. **TL, done:** reproduction, contract, ADR impact (this file). Check: the "problem" section.
2. **Dev:** write the test table; run it on the unfixed code and record that the criterion 2 and 3 cases fail for the right reason. Check: red output saying each `bad` program compiled unexpectedly.
3. **Dev:** apply the marker change; the same test goes green, and the before and after of the mixing example are captured. Check: criterion 2 and 3 green, existing tests green.
4. **Dev:** the two README lines. Check: README matches the code.
5. **QA, independent:** review at the exact HEAD from a clean context: `T` really distinguishes protocols, sealing intact, the negative test cannot pass because of an unrelated failure. Check: a verdict bound to the HEAD SHA.

## Definition of Done

- Gates from `AGENTS.md` for the module: `scripts/each_module.sh` unit tests for `pekko-actor-typed` offline (`unshare -rn`), `gofmt`,
  `go vet`, `gocyclo` ≤ 10 through `bin/golangci-lint`, `scripts/check_deps.sh`, `scripts/check_coverage.sh` at 85% for the module, diff
  coverage ≥ 85% (the production diff is declarations only), `.github/scripts/check-headers.sh`. No `-race`, no workbench, no campaigns.
- Every module still builds after the change (`pekko-actor-typed` is the only importer of `Behavior[T]`; build and vet of the other modules
  prove it).
- Commits follow Conventional Commits, carry no `Co-Authored-By`, are pushed normally and never force-pushed. One draft PR, not merged.
