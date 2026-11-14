# Scope: Retain messages in a bounded Typed StashBuffer (#889, TYP-13)

Status: **approved at G2 by the Owner, 2026-11-10**. Written on 2026-11-10 against `develop` `46ce37ac`. It carries the decisions the Owner approved on 2026-11-10 (D-1 to D-7, "Decisions" below). G2 decisions of the same day: the supervision test (S9) is approved alternative evidence (ADR 0033 D13); slice A adds a test that the internal `actor.Context` is the same object across all calls of the cell (`spawn.go:156`), the usage rule being left to #927; the adaptations are recorded in a dated amendment to ADR 0035 (this PR), the README and the kit manifest (ADR 0033 D14.3, precedent D-c and D-e). #889 is `prio:P1`; its dependency #847 is closed (checked with `gh issue view 847` on 2026-11-10), and #899, #855 and #1098 stopped being dependencies by the backlog-hygiene decision of 2026-11-09 (comment on #889).
**Kit run required (ADR 0017 "Delivery units", ADR 0024 T6, `AGENTS.md`)**: the unit ports `StashBufferImpl` and the buffer operations of `StashBuffer` (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala:38-96, 127-150`; `.../scaladsl/StashBuffer.scala:45-124`) and `Behaviors.withStash` (`.../scaladsl/Behaviors.scala:45-49`), and ports the cases of `StashBufferSpec.scala` that do not need `unstash`, so the Dev runs the migration kit (`cf91c9d`) with its own manifest, RULEBOOK and RUNBOOK under `migration/stage-1/typed-889-stash-buffer/` (the name is INFERRED from its siblings `typed-1183-resume`, `typed-976-restart`). The Dev is a separate session, QA is independent of both, and the TL does not implement.

Contract sources: [ADR 0033](../adr/0033-actor-typed-scope-and-architecture.md) (D4 stash is mandatory; D13 validation level; D14.1 to D14.3 Pekko is the behavior, a deviation needs its own ADR), [ADR 0035](../adr/0035-typed-supervision.md) (the `Supervise` and `OnFailure` rules this scope's overflow test uses), [ADR 0011](../adr/0011-pekko-actor-layout.md) (export justification; applied per export below, in the style of `behavior.go:88-98`), [typed-owner-decisions-2026-10-27.md](../parity/typed-owner-decisions-2026-10-27.md) (EXP-1: each export is justified in the story that introduces it, with symbol, consumer, why the public API is not enough, what it does not expose, and its test; #1020 indexes those records), the issue #889 with its ACT2 reconciliation (2026-10-24) and its Update of 2026-11-09. [ADR 0037](../adr/0037-port-as-a-go-adaptation.md) is about `Port` and does not bear on this scope.

## What this delivers

A bounded, actor-confined message buffer for Typed behaviors, and the one constructor that hands it to a behavior: `WithStash`. A behavior stashes a message of its own protocol type `T` while it is not ready for it, asks the buffer about its state (empty, size, capacity, full, head, iterate, contains, exists), and clears it. When the buffer is full, `Stash` fails and the buffer keeps what it already holds. Nothing here replays a message: `unstash` and `unstashAll` are #927. The buffer is private to the behavior: like Pekko's (`StashBufferImpl.scala:38-96`, a singly linked list owned by the buffer), it uses neither the Classic `Stash` nor the mailbox.

## Decisions (Owner, 2026-11-10)

The Owner approved these on 2026-11-10; they are stated as decided. Each says how it is applied.

- **D-1. Capacity is not validated.** "Owner, 2026-11-10". Exactly as Pekko: `isFull = size == capacity` (`StashBufferImpl.scala:69`), `capacity` is a plain constructor value (`:42-43`) and `withStash` checks nothing (`Behaviors.scala:45-49`). So capacity 0 is full from the start and the first `Stash` overflows; a negative capacity is never full (`size == capacity` is never true), so the buffer is unbounded. Both are consequences of the source, not promises. `StashBufferSpec` has no case for either (its capacity cases are `:65-78` with 3 and `:267-273` with 42), so the tests for them are labeled **"no Pekko oracle test: alternative evidence"** (ADR 0033 D13; the evidence is the source line, read, not run) and stay pending the Owner approval that D13 requires per row.
- **D-2. Go does not reject the zero value of `T`.** "Owner D13 approval, 2026-11-10". Pekko's `if (message == null) throw new NullPointerException` (`StashBufferImpl.scala:72`) is a JVM protection against a null reference, not a contract of the buffer: Go has no null, and a zero `T` (`""`, `0`, a nil pointer, a nil interface) is a legitimate message. The issue's criterion "A null message is rejected" is therefore **replaced by a documented difference** ("Go has no null"), see "Issue criteria: replacement text". No sentinel such as an `ErrNilMessage` is added. The README and the doc comment of `Stash` say it. A test pins that a zero value is stashed and read back (alternative evidence: no Pekko case exists for `null`, `StashBufferSpec.scala` has none).
- **D-3. `Stash(msg T) error` returns `ErrStashOverflow` when full, and the buffer stays intact.** "Owner, 2026-11-10". Pekko throws `StashOverflowException` (`StashBufferImpl.scala:73-76`) before touching the list; `StashBufferSpec.scala:65-78` stashes m1 to m3 with capacity 3, `intercept`s the exception on m4 (`:70-72`), intercepts it again on a second m4 (`:74-76`) and asserts `size == 3` (`:77`). Go: the error wraps `ErrStashOverflow` with the capacity (`fmt.Errorf("%w: capacity %d", ErrStashOverflow, capacity)`; text is not API, tests use `errors.Is`), is returned by the second call too, and `Size()` is still the capacity. The handler returns that error from `Receive`, as it would let the exception escape. **Required test**: `Supervise(b).OnFailure(ErrorIs(ErrStashOverflow), strategy)` catches the failure (spelling as in `supervise.go:41-43, 106-123` and the README "Supervision and restart": `Supervise`, `OnFailure`, `ErrorIs`, `Restart().WithLimit(...)` or `Resume()`), see "Evidence for the supervision test" for what Pekko shows and does not show.
- **D-4. The API, as proposed earlier and approved.** "Owner, 2026-11-10". `Head() (T, error)` returns `ErrStashEmpty` (message "head of empty buffer", `StashBufferImpl.scala:127-129`; Pekko throws `NoSuchElementException`, asserted by `StashBufferSpec.scala:59-61`). The eleven operations of `StashBuffer.scala:52-124` become: `IsEmpty`, `NonEmpty`, `Size`, `Capacity`, `IsFull`, `Stash`, `Head`, `ForEach`, `Contains`, `Exists`, `Clear`. `Contains` is a **package-level generic function** `Contains[T comparable](b *StashBuffer[T], m T) bool`, because a Go method cannot add a constraint; its equality is `==` of a comparable `T`, as Pekko's `exists(_ == message)` (`StashBufferImpl.scala:141-142`). `StashBuffer[T]` is a **struct pointer with unexported fields, not an interface**: Pekko's trait is `@DoNotInherit` ("Not for user extension", `StashBuffer.scala:43-45`). There is **no exported `NewStashBuffer`**: Pekko's `StashBuffer.apply` is `@InternalApi private[pekko]` (`StashBuffer.scala:26-34`); the only way to get a buffer is `WithStash`. The buffer is **actor-confined and not safe for concurrent use**: no locks, as Pekko's `_size`, `_first` and `_last` are plain `var`s (`StashBufferImpl.scala:49-53, 59`) and the scaladoc says "A non thread safe mutable message buffer" (`StashBuffer.scala:37-38`); the README and the doc comment say it. `ForEach` re-entrancy (calling `Stash` or `Clear` from the callback) is **documented as unsupported**: Pekko's linked list happens to visit a node appended during the walk (`StashBufferImpl.scala:131-137`), and Go does not promise that.
- **D-5. Not decided here, kept as the Dev's choice:** the internal representation of the buffer (slice, ring or linked list). It must clear a dropped slot so the garbage collector can free the message (as `Clear` drops the list, `StashBufferImpl.scala:91-96`), and #927 will need to remove the head in constant time (`dropHeadForUnstash`, `:103-112`); that need is INFERRED from #927's scope, not yet written.
- **D-6. Kit run: required** (a port). The alternative, a waiver, is not proposed.
- **D-7. The internal buffer captures the context in this slice, without exporting it.** "Owner's adjustment, 2026-11-10". Pekko builds the buffer with its context: `setup(ctx => { val stash = StashBuffer[T](ctx, capacity); factory(stash) })` (`Behaviors.scala:45-49`; `StashBufferImpl.apply(ctx, capacity)`, `StashBufferImpl.scala:42-43`), and unstash uses it (`unstashed(ctx, next)`, `interpretMessage(..., ctx, ...)`). If `WithStash` were built without the context, #927 would have to change its signature. So:

  ```go
  // WithStash is Behaviors.withStash (Behaviors.scala:45-49): built over Setup, it creates the buffer from the
  // behavior's Context and passes it to factory.
  func WithStash[T any](capacity int, factory func(b *StashBuffer[T]) (Behavior[T], error)) Behavior[T]
  ```

  `WithStash` is `Setup(func(ctx *Context[T]) (Behavior[T], error) { return factory(newStashBuffer(ctx, capacity)) })`. How the context is available (read at `behavior.go:76-79`, `interpreter.go:535-542`, `context.go:15-19`): `Setup` stores `factory func(*Context[T]) (Behavior[T], error)` and the interpreter calls `v.factory(c)` with the `*Context[T]` it built for that start (`interpreter.go:72`, `&Context[T]{ctx}`; or the one of a turn or restart, `:250, 289, 353`). `Context[T]` is a one-field struct, `struct{ c *actor.Context[T] }`, and `actor.Context[T]` is `struct{ c *cell[T] }` created once per spawn (`pekko-actor/actor/spawn.go:156`, `c.ctx = &Context[T]{c: c}`). So the unexported field of the buffer, `ctx *Context[T]`, holds the typed wrapper that `Setup` was called with; nothing in slice A or B reads it except S7c and the test below. **Caveat, INFERRED and to be settled before #927:** both `Context` types are documented "valid only during that call" (`context.go:15`, `actor.go:46`). The inner `*actor.Context[T]` is in fact one value for the life of the cell (`spawn.go:156`), and a restart reruns `Setup`, so the buffer of the new incarnation gets a new capture, as Pekko's does; but the documented lifetime says a stored `*Context[T]` must not be used outside the actor's own turns. #927 owns that rule; this scope only stores the pointer. A test pins that the factory receives a buffer **bound to the behavior's context** (`TestWithStash_BufferIsBoundToTheBehaviorContext`, internal, package `typed`, because the field is unexported): spawn `NewActor(WithStash(n, factory))`, and in `factory` record `b.ctx`; the test asserts `b.ctx != nil` and that `b.ctx.Self().ID()` equals the ID of the `Ref` that `actor.Spawn` returned (compared inside the factory, or the ID sent through a channel, so that the test goroutine never reads the context after the call) (`Self` is "the one Spawn returned", `context.go:17-19`). Mutation: `WithStash` passes `nil` to `newStashBuffer` (the test fails by assertion).

## Proposed Go API (for the Owner at G2)

```go
// pekko-actor-typed/typed/stash.go (shape of the contract, not the implementation)

var (
	ErrStashOverflow = errors.New("typed: stash buffer is full")   // StashOverflowException, StashBufferImpl.scala:73-76
	ErrStashEmpty    = errors.New("typed: head of empty buffer")   // NoSuchElementException, StashBufferImpl.scala:127-129
)

type StashBuffer[T any] struct { /* unexported: items, capacity, ctx *Context[T] */ }

func (b *StashBuffer[T]) IsEmpty() bool            // StashBuffer.scala:52
func (b *StashBuffer[T]) NonEmpty() bool           // :59
func (b *StashBuffer[T]) Size() int                // :66
func (b *StashBuffer[T]) Capacity() int            // :73
func (b *StashBuffer[T]) IsFull() bool             // :78   Size() == Capacity()
func (b *StashBuffer[T]) Stash(msg T) error        // :87   ErrStashOverflow (wrapped with the capacity) when full; buffer intact
func (b *StashBuffer[T]) Head() (T, error)         // :95   ErrStashEmpty when empty
func (b *StashBuffer[T]) ForEach(f func(T))        // :103  does not remove; re-entrancy unsupported
func (b *StashBuffer[T]) Exists(p func(T) bool) bool // :119
func (b *StashBuffer[T]) Clear()                   // :124
func Contains[T comparable](b *StashBuffer[T], m T) bool // :111  == of a comparable T

func WithStash[T any](capacity int, factory func(b *StashBuffer[T]) (Behavior[T], error)) Behavior[T] // Behaviors.scala:45-49
```

Differences from Pekko, each deliberate and listed in the README: `Stash` returns an error and not `this` (no chaining); `Head` returns an error and not an exception; no null check (D-2); the factory returns `(Behavior[T], error)` like `Setup`; `Contains` is a function; no Java DSL (`javadsl.StashBuffer`, `anyMatch`, `forEach(Procedure)` are the same operations for another language); `StashOverflowException` as a class hierarchy (`StashBuffer.scala:172`, `javadsl/StashBuffer.scala:160`) is a sentinel matched with `errors.Is`, as ADR 0035 D-e maps matchers.

## Open API choices (ADR 0011 and EXP-1, recorded here; the index #1020 links this section)

Style of `behavior.go:88-98`: symbol, consumer, why the public API is not enough, what it exposes and what not, test. `typed` exports to application code, not to another module, so EXP-1 binds by analogy with ADR 0033 D3 ("each exported symbol needs its individual ADR 0011 justification", ADR impact of the ACT2 reconciliation).

| Export | Symbol and consumer | Why nothing existing is enough | What it does not expose | Test |
|---|---|---|---|---|
| `StashBuffer[T]` and its eleven operations (ten methods, `Contains`) | Symbol: `StashBuffer` (`StashBuffer.scala:45-124`, read, not run against Pekko). Consumer: a behavior that must hold a message of its protocol until it can handle it (a loading phase, an initialization), and #927 which replays it. | `Setup`, `Receive` and `Same` keep no queue; a user slice or closure would repeat the capacity rule and the overflow error, and #927 needs one buffer type to attach `unstash` to. | no constructor (D-4), no field, no context, no way to build a buffer outside `WithStash`; the replay methods are #927. | S1 to S7, S9 |
| `WithStash` | Symbol: `Behaviors.withStash` (`Behaviors.scala:45-49`). Consumer: the same behaviors; the only constructor of a buffer. | `Setup` alone gives no buffer and no context-bound construction (D-7). | the context the buffer holds (unexported, used by #927). | S8 and the example |
| `ErrStashOverflow` | Symbol: `StashOverflowException` (`StashBuffer.scala:172`, `StashBufferImpl.scala:73-76`). Consumer: a handler that wants to react (drop, log, fail) and `Supervise(...).OnFailure(ErrorIs(ErrStashOverflow), ...)`. | a caller could not tell overflow from another failure; `ErrorIs` needs a comparable target. | the capacity or the message in a typed field (only in the error text). | S4, S8 |
| `ErrStashEmpty` | Symbol: `NoSuchElementException("head of empty buffer")` (`StashBufferImpl.scala:127-129`). Consumer: a caller of `Head` that branches with `errors.Is`. | without a sentinel, `Head`'s empty case cannot be told from a future failure. | nothing else. | S3 |

## Issue criteria: replacement text

The issue body was replaced on 2026-11-10 by the Owner's decision with this text (this scope edits no label; `status:ready` is the Owner's). The ACT2 reconciliation criteria were in the issue; the ACT2 reconciliation (2026-10-24) already says `UPDATE`, lists the omissions and requires "A null message is rejected" (replaced here by D-2). The text that replaces each criterion:

| # | Current criterion (issue body) | Replacement |
|---|---|---|
| AC-1 | Size/capacity/isFull/head/clear queries match the documented buffer state. | `IsEmpty`, `NonEmpty`, `Size`, `Capacity`, `IsFull`, `Head` and `Clear` report the state of the buffer after every operation, as `StashBufferSpec.scala:33-40, 42-63, 221-229, 267-273` assert for their Pekko counterparts. `Head` on an empty buffer returns `ErrStashEmpty` (Pekko: `NoSuchElementException`, `StashBufferImpl.scala:127-129`). |
| AC-2 | Overflow does not overwrite an existing retained value. | `Stash` on a full buffer returns `ErrStashOverflow` (Pekko: `StashOverflowException`, `StashBufferImpl.scala:73-76`), **retains** every message already held in its order and leaves `Size` at the capacity (`StashBufferSpec.scala:65-78`). The failure goes through supervision: `Supervise(...).OnFailure(ErrorIs(ErrStashOverflow), ...)` matches it. |
| AC-3 | The buffer is actor-confined and does not claim concurrent mutable access. | Unchanged. The buffer is not safe for concurrent use and has no locks (`StashBufferImpl.scala:49-53, 59`; `StashBuffer.scala:37-38`); the README and the doc comment say so. Re-entrant `Stash` or `Clear` from the `ForEach` callback is unsupported. |
| AC-4 | Focused executable positive and negative cases, including the named ordering or race boundary; record the tested SHA and local gates. | Positive and negative cases S1 to S9 of this scope, including insertion order (`StashBufferSpec.scala:80-92`); there is no race boundary (a confined buffer). The execution record binds the evidence to the tested SHA. |
| AC-5 | Update the public contract and traceability for this slice; do not claim sibling stories or full actor parity. | README of `pekko-actor-typed` states what exists and what is #927, #999, #1182; the manifest of the kit package is the traceability; no parity is claimed. |
| New | (omitted by the issue; ACT2 reconciliation) nonEmpty, foreach, contains, exists | `NonEmpty`, `ForEach` (in insertion order, without removing), `Contains` (`==` of a comparable `T`) and `Exists` (`StashBufferSpec.scala:80-104`). |
| New | Capacity | Capacity is **not validated** (D-1): 0 overflows at the first `Stash`, a negative capacity is never full. |
| Replaced | (ACT2) "A null message is rejected" | Go has no null: a zero `T` is stashed (D-2). A documented difference, pending the D13 approval already given for this point; no sentinel exists. |

## Criteria, scenarios, sources and mutations

Test files are in `pekko-actor-typed/typed/`; all tests use go-specs, channels or gates, no sleeps, no `-race`, no real engine. A mutation is applied to production code in a disposable worktree and never committed (`qa-review`); the named test must fail by assertion. `SBI` = `StashBufferImpl.scala`, `SB` = `StashBuffer.scala`, `SBS` = `StashBufferSpec.scala` (paths: `upstream/pekko/actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/`, `.../scaladsl/`, and `upstream/pekko/actor-typed-tests/src/test/scala/org/apache/pekko/actor/typed/scaladsl/`). The Pekko spec builds the buffer with `StubbedActorContext` that "will never be invoked" (`SBS:27-29`); Go has the analogue in slice A as a buffer built with a nil context (the field is unused until #927).

| # | Criterion | Scenario (test) | Source / oracle | Mutation | Slice |
|---|---|---|---|---|---|
| S1 | A new buffer is empty; after one `Stash` `IsEmpty` is false and `NonEmpty` true | `TestStashBuffer_Empty` | `SBS:33-40`; `SB:52, 59` | `IsEmpty` always false | A |
| S2 | `Size` follows `Stash` (0, 1, 2); `Head` returns the first message without removing it (`Size` stays 2). The Pekko case also drops with `unstash` (`SBS:52-62`): that part is a **PORT-GAP to #927**; the Go case uses `Clear` for the drop | `TestStashBuffer_AppendAndHead` | `SBS:42-63` (adapted); `SBI:67, 127-129` | `Head` removes the element | A |
| S3 | `Head` on an empty buffer returns `ErrStashEmpty` (`errors.Is`) and `Size` stays 0 | `TestStashBuffer_HeadOfEmpty` | `SBS:59-62`; `SBI:127-129` | `Head` returns the zero value and nil error | A |
| S4 | Capacity 3: m1 to m3 are retained; m4 returns `ErrStashOverflow` (wrapped with the capacity), a second m4 returns it again, `Size` is 3, `IsFull` is true, and the order m1, m2, m3 is intact | `TestStashBuffer_EnforcesCapacity` | `SBS:65-78`; `SBI:69, 73-76` | overflow drops the oldest or overwrites the last element; or `Stash` returns nil when full | A |
| S5 | `ForEach` visits in insertion order without removing; `Contains` and `Exists` answer true for a held message and false for an absent one | `TestStashBuffer_OrderContainsExists` | `SBS:80-92, 94-104`; `SBI:131-150` | `ForEach` visits in reverse; `Contains` always true | A |
| S6 | `Clear` empties the buffer (`Size` 0) and the buffer accepts a new message afterwards (`Size` 1) | `TestStashBuffer_Clear` | `SBS:221-229`; `SBI:91-96` | `Clear` leaves the size | A |
| S7 | `Capacity` returns the value given (42) | `TestStashBuffer_Capacity` | `SBS:267-273` | `Capacity` returns the size | A |
| S7a | **No Pekko oracle test: alternative evidence.** Capacity 0: the first `Stash` returns `ErrStashOverflow`, `IsFull` is true on an empty buffer. Negative capacity: `IsFull` is false and a number of `Stash` calls far above 0 all succeed | `TestStashBuffer_CapacityIsNotValidated` | `SBI:42-43, 69` (read, not run); D-1 | `Stash` or the constructor rejects capacity <= 0 | A |
| S7b | **No Pekko oracle test: alternative evidence.** A zero `T` is stashed and read back: `""` for `string`, nil for a pointer, nil for an interface; `Size` counts it | `TestStashBuffer_ZeroValueIsAMessage` | `SBI:72` (the null check Go does not port); D-2 | `Stash` rejects the zero value | A |
| S7c | A buffer built with a real context keeps the same internal `actor.Context` object across all the calls of the cell (the context is one value per cell, `spawn.go:156`; a restart reruns `Setup` and builds a new buffer). The usage rule for the captured context is #927's | `TestStashBuffer_ContextIsTheSameObjectAcrossCalls` | `spawn.go:156`; `Behaviors.scala:45-49` | S7c pins a `pekko-actor` invariant (`spawn.go:156`), not code of `stash.go`: it compares the inner pointer (`b.ctx.c`) with the one seen in a later turn; the mutation is the buffer storing the context of another cell (a different inner pointer) | A |
| S8 | `WithStash(n, factory)` runs `factory` once, when the actor starts, with an empty buffer of capacity `n`, and uses the behavior the factory returns; the buffer is bound to the behavior's context (D-7): `b.ctx` is non-nil and its `Self().ID()` equals the `ID()` of the `Ref` `Spawn` returned. A factory that returns an error fails the start (as `Setup`) | `TestWithStash_BuildsBufferOverSetup`, `TestWithStash_BufferIsBoundToTheBehaviorContext` (internal) | `Behaviors.scala:45-49`; `SBI:42-43`; `behavior.go:76-79` | `WithStash` passes `nil` as the context; `WithStash` ignores `capacity` | B |
| S9 | The overflow error from a handler is caught by supervision: `Supervise(WithStash(1, ...)).OnFailure(ErrorIs(ErrStashOverflow), Resume())` keeps the actor alive with the buffer unchanged (the failed message is dropped), and with `Restart()` it reruns `Setup`, giving a **new empty buffer** (the state is rebuilt, as Pekko rebuilds the behavior). An unmatched cause (`ErrorIs(someOtherErr)`) lets the actor take the default Stop. See "Evidence for the supervision test" | `TestWithStash_OverflowIsCaughtBySupervision` (rows: Resume, Restart, unmatched) | no Pekko test shows this (see below): **no Pekko oracle test: alternative evidence, approved by the Owner on 2026-11-10 (ADR 0033 D13)**; matching rules `supervise.go:41-43, 106-123`, ADR 0035 | the layer's matcher ignores `ErrStashOverflow` (the failure goes to the default Stop); the restart row: the rebuilt buffer keeps the old messages | B |
| S10 | README, example and kit manifest: `ExampleWithStash` runs and prints; the README section states what exists, the differences from Pekko, the confinement, the unsupported `ForEach` re-entrancy, the capacity not validated, the zero value; links the example and the tests; says what #927, #999 and #1182 own | review; `ExampleWithStash` (Output compared) | module README contract | none (documentation) | B |

Every sentence of the PRs that says something was observed names the oracle line or the assertion. All rows are "read against Pekko v1.7.1 specs and sources, not run": Pekko was not run for this unit (no oracle file), so **not V3**; rows S7a, S7b and S9 have no Pekko spec at all and stay "pending approval" (ADR 0033 D13) until the Owner approves the alternative evidence per row (D-1, D-2 give that approval for the behavior; the row approval stays the Owner's).

### Evidence for the supervision test (S9)

What exists in Pekko, opened on 2026-11-10 at the pin (`9a54d12`):

- `StashOverflowException` extends `RuntimeException` (`actor-typed/src/main/scala/org/apache/pekko/actor/typed/scaladsl/StashBuffer.scala:172`; the javadsl class extends it, `javadsl/StashBuffer.scala:160`). Under Typed's default supervision a non-fatal exception stops the actor; with `Supervise(...).onFailure[...]` it is decided by the matching layer (ADR 0035). `grep -rn StashOverflowException` over `actor-typed-tests` finds it **only** in `StashBufferSpec.scala:70, 74` (the `intercept` of the throw). **There is no test in `actor-typed-tests` that shows a `StashOverflowException` handled by `onFailure`.** `StashSpec.scala` does not mention it either.
- The nearest Pekko evidence is in another module and shows the **default** case, not `onFailure`: `persistence-typed-tests/src/test/scala/org/apache/pekko/persistence/typed/scaladsl/EventSourcedBehaviorStashSpec.scala:644-~688 (log :673, `expectTerminated` :679)` ("fail when stash has reached limit if configured to fail") expects an ERROR log of `StashOverflowException` (`:673`) and `probe.expectTerminated(c, 10.seconds)` after the stash is overfilled; the code that throws is `persistence-typed/src/main/scala/org/apache/pekko/persistence/typed/internal/StashManagement.scala:55-70` (`case StashOverflowStrategy.Fail => throw e`). That is the exception escaping the behavior and stopping the actor, in a persistence behavior, not a `Supervise(...).onFailure` test. `GuardianStartupBehavior.scala:50` throws it too (untested here).

So S9 is **"no Pekko oracle test: alternative evidence"**: the rule that a `RuntimeException` is matched by `onFailure[Class]` is established by `SupervisionSpec` for other exception classes (e.g. the resume row cited in the #1183 manifest, `SupervisionSpec.scala:163, 834`), and the Go rules `ErrorIs` on a returned error are the existing, tested ones of #976 and #1184. The test therefore pins that the new sentinel composes with the existing matcher; it does not claim a Pekko test of that composition. If the Owner wants Pekko evidence, an oracle (a `Behaviors.withStash(1)` overflowing under `onFailure[StashOverflowException](restart/resume)`, run on Pekko 1.7.1, as `typed-resume-2026-11-06.md` did) is a separate slice; it is not proposed (cost against the benefit of a composition of two verified rules).

## Slices (one PR each, at most 400 authored lines, sequential)

Estimates: the TL's raw numbers times 1.55 (the deviation measured in #976); the TL wrote no prototype and compared against existing files (`behavior.go` 130 lines, `signal.go` 73, `supervise.go` 161, `example_test.go` 119). **If a slice measures above 400 authored lines (additions plus deletions, without generated files, `go.sum` or `.claude/skills/`), the TL splits it** (`AGENTS.md`, "PR budget and commits"); the Dev stops and reports instead of opening the PR.

| Slice | Delivers | Files | Criteria | Est. lines (raw → re-estimated) |
|---|---|---|---|---|
| **0. Kit package** | manifest, RULEBOOK, RUNBOOK | `migration/stage-1/typed-889-stash-buffer/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | kit run | ~110 docs (#1183's kit files measure 111 lines) |
| **A. The buffer** | `StashBuffer[T]` with its unexported fields (items, capacity, `ctx`), the unexported `newStashBuffer(ctx, capacity)`, the ten methods, `Contains`, `ErrStashOverflow`, `ErrStashEmpty`; tests against a buffer built with a nil context (S7c: with the context of a spawned cell) | `pekko-actor-typed/typed/stash.go` (new), `pekko-actor-typed/typed/stash_test.go` (new, `package typed`, internal because the constructor is unexported) | S1 to S7c | production ~75 raw → ~115 (about `supervise.go`'s size less the strategies, plus doc comments); tests ~130 raw → ~200, plus ~25 for S7c; **~340** |
| **B. `WithStash` and its evidence** | `WithStash` (in `stash.go`); the spawn-based tests S8 and S9; `ExampleWithStash`; the README section; the kit manifest rows; the execution record; the cost-log row | `pekko-actor-typed/typed/stash.go`, `pekko-actor-typed/typed/stash_withstash_test.go` (new; S8 internal, S9 can be external), `pekko-actor-typed/typed/example_test.go`, `pekko-actor-typed/README.md`, `migration/stage-1/typed-889-stash-buffer/EXECUTION-RECORD.md`, `migration/cost-log.tsv` | S8, S9, S10 | production ~12 raw → ~18; tests ~90 raw → ~140; example ~25 → ~40; README ~45 docs → ~70; record ~85 docs (the #1183 record is 87); **~330** |

Order: 0, A, B. `stash.go` is edited by A and then by B (sequenced, not parallel). Nothing under `pekko-actor/`, `docs/adr/` (the ADR 0035 amendment is in the scope PR, not in a slice), `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md` or another module changes; a needed `pekko-actor` change (for example, a context accessor the buffer cannot get from `typed.Context`) is Mandatory Stop 1 and goes back to the TL. A test that fails on the current code where the implementation passes is also a stop. The estimate for B is near the budget if the README grows; the Dev measures after the tests and splits the README and record into a docs PR if the total passes 400.

| Pair | Shared file | Resolution |
|---|---|---|
| 0, A, B | none between 0 and the others | slice 0 owns new files only |
| A, B | `typed/stash.go` | sequenced (A first) |

Independence from the other open Typed scopes: this scope adds new files plus `example_test.go` and the module README, which the delivered scopes already edited and the open ones (#927 later) will edit again; #927 consumes `stash.go` and is sequenced after B. `behavior.go` and `interpreter.go` are **not** edited (`WithStash` is built over the existing `Setup`, which is the reason for D-7 not needing a new behavior variant); `behavior.go` appears in the allowed files only as an escape hatch below.

## Allowed files

Exactly these paths. Anything else needs a stop and a report to the TL.

| Path | Edit |
|---|---|
| `docs/scopes/889-typed-stash-buffer.md` | this file, edited by the planning PR only |
| `migration/stage-1/typed-889-stash-buffer/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | kit package, slice 0 |
| `migration/stage-1/typed-889-stash-buffer/EXECUTION-RECORD.md`, `migration/cost-log.tsv` (one row per kit run) | Dev, slice B, after the gates; unknown metrics stay `unknown` |
| `pekko-actor-typed/typed/stash.go` | new (A), `WithStash` (B); one `// Ported from upstream/pekko/actor-typed/src/main/scala/org/apache/pekko/actor/typed/internal/StashBufferImpl.scala` line, Apache-2.0 header (`check-headers.sh`) |
| `pekko-actor-typed/typed/stash_test.go`, `stash_withstash_test.go` | new |
| `pekko-actor-typed/typed/example_test.go` | `ExampleWithStash` |
| `pekko-actor-typed/README.md` | the stash section; the list of exports in "Public usage"; the status line |
| `pekko-actor-typed/typed/behavior.go` | **only if needed** (for example to place `WithStash` beside `Setup`); the plan is not to touch it |
| `docs/parity/actor-inventory.md` | **only if the traceability row requires it** (row `TY-05` of `docs/parity/actor-inventory.md:120`, and `ACTINV-111` at `docs/parity/actor-typed-design-closure-2026-10-26/inventory/INV-T-typed.md:100`). The file is shared with other stories, so editing it is a later decision of the Owner and not part of this scope unless the Owner says so |

`scripts/deps.allow` is not touched: `typed` keeps importing only `pekko-actor` (unchanged).

## Not carried here (each with its owner)

| Item | Owner |
|---|---|
| `unstash`, `unstashAll`, `unstash(behavior, n, wrap)`, `UnstashException`, ordering of replayed messages, clear-while-unstashing, "fail quick on invalid start behavior" (`SBS:42-63` unstash half, `:106-219`, `:231-265`; `StashSpec.scala` in full) | [#927](https://github.com/getsyntegrity/gopekko/issues/927) (TYP-14). The context captured by D-7 exists so that #927 does not change `WithStash`. |
| The bounded stash of the restart (a `Restart` with a stash for messages that arrive while it waits) | [#999](https://github.com/getsyntegrity/gopekko/issues/999) (TYP-26) |
| `Stopped` and `StoppedWith` | [#1182](https://github.com/getsyntegrity/gopekko/issues/1182) (TYP-48) |
| Classic stash (the `Stash` trait, `unstashAll` on the mailbox) and the mailbox head insert | [#855](https://github.com/getsyntegrity/gopekko/issues/855), [#898](https://github.com/getsyntegrity/gopekko/issues/898), [#899](https://github.com/getsyntegrity/gopekko/issues/899), [#1098](https://github.com/getsyntegrity/gopekko/issues/1098): not dependencies (Owner, 2026-11-09); the Typed buffer uses neither (`StashBufferImpl.scala:38-96`) |
| The Java DSL of the buffer (`anyMatch`, `forEach(Procedure)`) | not applicable to Go |
| An exported `NewStashBuffer`, or a stash for `persistence` behaviors | no consumer; not proposed (D-4) |

## Kit run: ported sources, tests and PORT-GAPs

The migration kit (`cf91c9d`) runs inside this scope under `migration/stage-1/typed-889-stash-buffer/`. The manifest columns are `source, target, disposition, pekko_row, scenario, evidence, note` (as in `typed-1183-resume/manifest.tsv`). Rows planned (one row per Pekko case; evidence `READ` for source, `SPEC` for a Pekko test read and not run):

| Pekko source | Go target | Disposition | Row | Scenario |
|---|---|---|---|---|
| `StashBufferImpl.scala:42-43, 49-53, 59, 63-69` | `typed/stash.go` | ported | constructor, `isEmpty`, `nonEmpty`, `size`, `isFull` | S1, S4, S7a |
| `StashBufferImpl.scala:71-89` | `typed/stash.go` | adapted | `stash`: error instead of exception; no null check (`:72`); returns nothing (`this`) | S4, S7b |
| `StashBufferImpl.scala:91-96` | `typed/stash.go` | ported | `clear` (the `stashCleared` hook is a no-op, `:264`, not ported) | S6 |
| `StashBufferImpl.scala:127-129` | `typed/stash.go` | adapted | `head`: `ErrStashEmpty` | S2, S3 |
| `StashBufferImpl.scala:131-150` | `typed/stash.go` | ported | `foreach`, `contains`, `exists` | S5 |
| `StashBuffer.scala:45-124` | `typed/stash.go` | adapted | the public operations; trait to struct (D-4) | S1 to S7 |
| `StashBuffer.scala:172`, `javadsl/StashBuffer.scala:160` | `typed/stash.go` | adapted | `StashOverflowException` to `ErrStashOverflow` | S4, S9 |
| `Behaviors.scala:45-49` | `typed/stash.go` | ported | `withStash` over `Setup`, with the context (D-7) | S8 |
| `StashBufferSpec.scala:33` "answer empty correctly" | `stash_test.go` | ported | | S1 |
| `StashBufferSpec.scala:42` "append and drop" | `stash_test.go` | adapted | the `unstash` drops (`:52, :57`) are **PORT-GAP to #927**; the Go case drops with `Clear`; the `NoSuchElementException` on an empty `head` (`:59-61`) becomes `ErrStashEmpty` | S2, S3 |
| `StashBufferSpec.scala:65` "enforce capacity" (`:65-77`) | `stash_test.go` | ported | the javadsl subclass check (`:73-76`) is not applicable; the second overflow is kept | S4 |
| `StashBufferSpec.scala:80` "process elements in the right order" | `stash_test.go` | adapted | the `unstash` between the two `foreach` (`:88`) is a **PORT-GAP to #927**; the Go case checks the order and then `Clear` | S5 |
| `StashBufferSpec.scala:94` "answer 'exists' and 'contains' correctly" | `stash_test.go` | ported | | S5 |
| `StashBufferSpec.scala:106-219` (unstash cases) | none | **PORT-GAP** | unstash, unstashAll, undefer, wrap, numberOfMessages | #927 |
| `StashBufferSpec.scala:221` "clear" | `stash_test.go` | ported | | S6 |
| `StashBufferSpec.scala:231, 261` (clear while unstashing; fail quick on invalid start behavior) | none | **PORT-GAP** | | #927 |
| `StashBufferSpec.scala:267-273` "answer thruthfully about its capacity" | `stash_test.go` | ported | | S7 |
| `StashSpec.scala` (all cases) | none | **PORT-GAP** | | #927 |
| `EventSourcedBehaviorStashSpec.scala:644-~688 (log :673, `expectTerminated` :679)`, `StashManagement.scala:55-70` | `stash_withstash_test.go` | cited as the nearest evidence of the default | not ported (persistence) | S9 |
| (no Pekko case) | `stash_test.go`, `stash_withstash_test.go` | **no Pekko oracle test: alternative evidence** | capacity 0 and negative, zero value, overflow under `onFailure` | S7a, S7b, S9 |

## Does the Go CURRENT of the judge change?

No. Verified on 2026-11-10: the Go campaign inputs are `pekko-actor/` (bound by its git tree, `judge/mutate_go.py:254-256`: `_git("status", "--porcelain", "--", "judge", "pekko-actor")` and `rev-parse commit:pekko-actor`), and the files in `COMPARATOR_PATHS` (`judge/mutate_go.py:213`: `judge/evaluate.py`, `judge/go_scope.py`, `judge/go-scope.json`, `judge/scenarios`, `judge/mutate_go.py`, `.go-version`); `docs/judge.md` says the committed report is CURRENT "only while the git tree of `pekko-actor/`" and those inputs "are those it was run with". `grep -rn actor-typed` over `judge/` and `scripts/check-judge-matrix.py` finds nothing (no `actor-typed` reference exists in them). This scope changes files only under `pekko-actor-typed/`, `migration/`, `docs/scopes/` and (optionally) `docs/parity/`, and no `pekko-actor/` file, so no tree hash the judge binds changes. If the Dev finds a need to edit `pekko-actor/` (Mandatory Stop 1), that claim stops being true and the CURRENT of the Go report turns stale; that is a reason to stop and ask.

## Dependencies

- Consumes: `Setup`, `Behavior[T]`, `Context[T]` (`behavior.go:76-79`, `context.go:15-19`), the `Supervise` / `OnFailure` / `ErrorIs` / `Resume` / `Restart` of #976, #1183 and #1184 (delivered), the interpreter's use of `Setup` (`interpreter.go:535-542`). #847 is closed.
- Is consumed by: #927 (adds `unstash`/`unstashAll` to the buffer), #999 (indirectly).

## Appetite

Three small PRs (one kit package, two code). The risk is small and sits in two places: the lifetime of the captured context (D-7, INFERRED caveat) and the README, which must separate what exists from #927.

## Definition of Done

- S1 to S9 (with S7a to S7c) have their tests (S7a, S7b, S9 labeled "no Pekko oracle test: alternative evidence") and S10 its text; each behavior PR names its mutation in a "Mutations" table, reproduced by QA (unmutated green, mutated red).
- `scripts/each_module.sh go test -count=1 ./...`, `scripts/check_deps.sh` and the complexity lint (gocyclo <= 10) with the commands of `AGENTS.md`; diff coverage >= 85% on new and modified lines; per-module coverage >= 85%; `typed` still imports only `pekko-actor`; no `-race`; `unshare` does not exist on macOS and the evidence says so when it applies.
- Every ported file carries Apache-2.0 and exactly one `// Ported from ...` line (`check-headers.sh`).
- The README states what is implemented, the differences (error instead of exception, no null check, function `Contains`, no constructor), the confinement and the unsupported `ForEach` re-entrancy, the capacity rule, and that #927, #999 and #1182 are not delivered; it claims no parity.
- The PRs say `Refs #889`, never `Closes`; the issue is closed by hand by the Owner. Evidence is bound to the final head SHA with real exit codes.
- A new QA session issues a verdict on the exact head of each PR.

## Open points at G2: resolved by the Owner, 2026-11-10

1. **ADR 0033 D14.3.** Decided: the adaptations (capacity not validated, no rejection of the zero value, `ErrStashOverflow`, `ErrStashEmpty`, a free `Contains`, `WithStash` with a captured context) are recorded in the dated amendment "2026-11-10" of `docs/adr/0035-typed-supervision.md` (precedent D-c and D-e), in this PR, plus the module README and the kit manifest (`PORT-GAP` rows with the note "accepted adaptation, ADR 0035 amendment 2026-11-10").
2. **The captured context lifetime.** Decided: accepted; slice A pins that the internal context is one object across the calls of the cell (S7c). The usage rule is #927's.
3. **Supervision test.** Decided: S9 is approved alternative evidence (D13).
4. **`status:ready`** stays the Owner's.
