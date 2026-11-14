# Scope: Encode an exact local reference string (#894, IDN-09)

Revisions 1 to 10 (2026-10-29): QA rounds on the earlier draft that also covered parsing; their history is in git. **Revision 11 (2026-10-29): final design after the Owner's design decisions of 2026-10-29** (below). Written against `develop` `899d93e0f13f2e7673652b2a060329d9ee8f6b51` and Apache Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. Documentation only; Risk: none for this PR.

Status: **G2 granted by the Owner on 2026-10-29 on #1288 `d65884dbbb5aa5c17b9efa197c43558b0385a6c7` and #1291 `5ac4ae89ecc47a1acbb989fec0d632ac4be04255`** (message below, as received). Kit execution, implementation and independent QA are authorized; merge, issue edits or closure, the #1020 index, remote CI and `-race` are **not**. The approved contract is exactly that of `d65884d` plus the modification below ("Post-G2-SHA modification"). **This PR is documentation only; there is no implementation yet, and QA of this documentary delta comes before any implementation.** #894 stays `status:blocked`; this scope edits no issue and no label. The #1186 proposal (PR #1291) records the token form and the addendum acceptance.

Kept apart: **logical path** (`ActorPath`: ADR 0031 P-6, P-7), **exact reference** (a `Ref`; identity is its `ID`: ADR 0024) and **resolution** (finding a live actor: #893, #1092, never here).

## Earlier Owner design decision of 2026-10-29, as relayed to the coordinator (fragmentary message)

The message reached the coordinator **fragmentary**; the list is the coordinator's reconstruction and is **pending the Owner's confirmation**; it is not a verbatim quote. Fragment as received (trace only): "Apruebo decisiones diseño #894: Solo codificación, resolución. J1/J2: path#<época>-<número>, proceso, compartida Systems. Unicidad probabilística, remota. J3: guardianes no codificables. J4: entradas #1092. J7: el ID. J8: comportamiento propuesto, resultado ("", false) no Terminated.Who() Encoder como Handle, ni Handle. Proponé scope final. Kit requerido solo-Go Pekko, 1. agotamiento números, panic. concurrencia aisladas. 2. eliminar AC2/AC3: No 3. No fijar #1092. 4. ID Pekko. PRs #1186 #894, QA delta, SHA G2. Incluí API".

Reconstruction: **encode only** (no parse, no resolution); **J1/J2** token `path#<epoch>-<number>`, a random process epoch shared by all Systems, cross-process uniqueness only probabilistic, no absolute uniqueness and no remote resolution promised; **J3** guardians not encodable; **J4** tokenless input is #1092's; **J7** the public identity stays the `ID`; **J8** proposed behavior with `("", false)` for non-encodable cases; the encoder is a free function over `Handle`; kit required and Go-only evidence for cases with no Pekko equivalent (enumerated below); conditions 1 to 4 (permanent counter exhaustion, AC2/AC3 traceability, no #1092 outcome fixed, ID numeric format and Pekko differences). Not authorized: G2 itself, implementation, merge, issue edits, the #1020 index.

**Points (a)-(m), after the G2 message.** The Owner confirmed them "solo en cuanto coincidan con las decisiones expresas de este mensaje y las anteriores" and approved no additional clause by reference. **Express in the G2 message:** (a) J3 includes the root and the guardians ("raíz y guardianes no codificables"); (b) a terminated original `Ref` keeps its encoding ("referencia original terminada con codificación conservada"), and `Terminated.Who()` as an ID-only handle is covered only as one of the "casos codificables/no codificables enumerados" of the J8 table; (c) the free function `ExactRefString(h Handle) (string, bool)`, no method on `Ref` or `Handle` (the API text of this scope); (e) permanent exhaustion ("contador compartido con agotamiento permanente", "El contador permanece agotado"); (h) now: G2, kit, implementation and QA authorized; merge, issue edits, the #1020 index, remote CI and `-race` not; (i) kit required and Go-only evidence accepted as the method; (m) decided (below). **Matching the earlier, fragmentary message but not repeated expressly in the G2 message, so confirmed only to that extent and not approved beyond it:** (d) AC2/AC3 not deleted silently, traceability kept (fragment "eliminar AC2/AC3: No"); (f) the #1092 resolver outcome not fixed ("No fijar #1092"); (g) the ID numeric format as a documented contract and the Pekko differences ("4. ID Pekko"); (j) J4, tokenless input is #1092's; (k) J7, the public identity stays the `ID`; (l) "no absolute uniqueness and no remote resolution promised" ("Unicidad probabilística, remota"). **Not expressly decided:** the name `ExactRefString` is approved as part of "API ExactRefString(h Handle) (string, bool)"; the 16-hex epoch width and the guarded-counter code shape are **not expressly decided** by the Owner's messages; they are specified here only as the design presented, and nothing here claims they are approved by reference. The Owner's phrase "No necesitás otra aprobación si se mantiene exactamente este contrato" is kept as written; a change to either would be a change to this design and is reported, not assumed approved.

## Owner message of 2026-10-29 (G2 grant), as received

Complete message, quoted verbatim (Spanish): "Concedo G2 de #894 sobre: #1288: d65884dbbb5aa5c17b9efa197c43558b0385a6c7 ; #1291: 5ac4ae89ecc47a1acbb989fec0d632ac4be04255. Apruebo el contrato presentado: solo codificación, API ExactRefString(h Handle) (string, bool), token época-número, contador compartido con agotamiento permanente y los casos codificables/no codificables enumerados. Esto incluye raíz y guardianes no codificables y referencia original terminada con codificación conservada. Acepto la adenda de ADR 0031 en los términos presentados. Autorizo ejecutar el kit, implementar y obtener QA independiente. Los casos solo-Go quedan aceptados como método de evidencia; su cumplimiento requiere ejecutar y revisar las pruebas. Sobre (m): rechazo dejar una reserva colgada. El agotamiento debe liberar la reserva pendiente antes de propagar el panic, sin publicar una celda parcial ni alterar las demás reservas. El contador permanece agotado. Amplío Allowed files a spawn.go y hierarchy.go exclusivamente si son necesarios para esa corrección. Elegí el cambio mínimo. Probá que, después de recuperar el panic: - la reserva se libera; - el dueño puede completar su cierre; - no se reutilizan IDs ni se publica un actor parcial. Registrá esta modificación posterior a los SHA citados y obtené QA del delta documental antes de implementar. No necesitás otra aprobación si se mantiene exactamente este contrato. Las lecturas (a)-(m) quedan confirmadas solo en cuanto coincidan con las decisiones expresas de este mensaje y las anteriores; no apruebo cláusulas adicionales por referencia. Mantené las excepciones de validación ya aceptadas. Sin CI remoto ni -race. Sin merge, cierre o edición de issues ni creación del índice #1020. Entregá PRs draft, SHA exactos, QA y resultados locales."

The earlier section is the fragmentary message of the same day; this message is complete and prevails where they differ.

## Post-G2-SHA modification (2026-10-29): the exhaustion panic releases its reservation

Decided by the Owner **after** the SHAs cited by the G2 grant (message above, "Sobre (m)"): a recovered exhaustion panic must release the pending reservation **before** propagating, publish no partial cell and leave the other reservations untouched; the counter stays exhausted. The alternative of leaving the reservation hanging is **rejected**. The Allowed files grow to `spawn.go` (below). Verified by reading develop `899d93e`:

- **Before and after line 156.** `commit` (`hierarchy.go:149-169`) takes `System.mu` and defers its unlock (`:151`); before `:156` it only **reads** (`e := n.names[name]` and its check). The panic arises in `nextID()`, the **argument** of `r.assignID(nextID())` (`:156`), so `assignID` is never called, and everything written after it (`markOrigin`, `e.rec = r`, `s.byID[...]`, `n.reserved--`, `n.out`, `inherit`) is never reached. Visible state at the panic: the reservation made by `reserve` (`hierarchy.go:105-121`: `n.names[name] = &entry{tok: tok}`, `n.reserved++`) and nothing else: no record, no `byID` entry, no `n.out` entry; lookups skip an entry whose `rec == nil` (`child` and `children` require `e.rec != nil`). The cell built by `commitCell` is referenced only by that function's locals.
- **Where to fix.** `commit` has one caller, `commitCell` (`spawn.go:160`); `commitCell` has two, `spawn` (`spawn.go:113`) and `newGuardian` (`guardian.go:123`). A fix in `commitCell` covers all of them. A fix inside `commit` cannot call `rollback` (it takes `System.mu`, which `commit` holds: a non-reentrant deadlock) and would have to duplicate its lines under the lock. The existing `rollback` (`hierarchy.go:125-135`: lock `:126`, `n.reserved--` `:130`, `readyLocked` `:131`, unlock `:133`, `schedule` `:134`; `readyLocked` is `:190-196`, its condition `:191`; `outstanding` is `:359-362`) frees **only** the entry with `rec == nil` and the same `tok` (`n.reserved--`), then calls `readyLocked(n)` and, after unlocking, `schedule(n, fire)` (`hierarchy.go:199-203`, doc comment `:198`), which hands the owner's finalization to the executor when the owner is terminating with nothing outstanding: this is "the owner can complete its close". `commit`'s own deferred unlock runs before the deferred release below, so `rollback` finds `System.mu` free: no double unlock, no deadlock. A plain `defer` runs while panicking and the panic then continues with the same value and stack: no `recover` and no re-panic are needed.
- **Chosen: option (ii), `spawn.go` only, about 7 lines, `hierarchy.go` untouched.** Intended shape (not applied):

```go
	committed := false
	defer func() { // commit panicked (an exhausted ID counter): free only this reservation, then the panic goes on
		if !committed {
			t.sys.rollback(t.owner, name, tok)
		}
	}()
	stopping, ok := t.sys.commit(t.owner, name, tok, c) // registered before the first Execute
	committed = true
```

The normal path and the `!ok` path (`spawn.go:161-163`, explicit `rollback`, `ErrNameTaken`) are unchanged: `committed` is true on both. No exported behavior and no error set changes (the panic is the existing sentinel of Condition 1). `hierarchy.go` would be edited only if the Dev finds the correction needs more than these lines; that is then a stop to report.

## Traceability of the acceptance criteria of #894 (issue read-only, `gh issue view 894`; the issue is not edited)

| # | Criterion (verbatim from the issue) | Where it goes |
|---|---|---|
| AC1 | "The encoding carries the address/path/incarnation information needed by the accepted contract." | **#894**: `ExactRefString`, rows E1, E3 |
| AC2 | "A valid local round trip resolves the same actor cell." | **moved to #1092** (resolution); not deleted. Not asserted here; the encode half is E1 and E3 |
| AC3 | "Name reuse cannot make an old encoded reference resolve to the new actor." | **moved to #1092**; not deleted. Here only: two incarnations at one path encode to different strings (E3) |
| AC4 | "Include focused executable positive and negative cases for this result, including the named ordering or race boundary; record the tested candidate SHA and applicable local gates." | **#894**: rows E1-E9 and C1-C8, validation plan; the race boundary is C5 |
| AC5 | "Update the public contract and traceability for this slice; do not claim completion of sibling stories or full actor parity." | **#894**: README row E11, this table; no parity claim |
| New AC (ACT2, 2026-10-24) | "the encoding carries the Go ID or incarnation token, as decided in the ADR 0031 addendum (ADR 0024:203)" | **#894**: token form (d), approved in the G2 message of 2026-10-29 (token epoch-number, addendum accepted in the terms presented); the addendum text (#1186) stays a proposal |

The issue's ACT2 text moves AC2/AC3 to #1092 (`IDN-21`) and lists #812 as `depends_on` ("parse the encoded string"). Under "encode only" parsing is not #894's: the parse of exact strings and the foreign and stale outcomes are pointers only (who owns the exact-string parse is not settled here, and #1092's scope is not edited by this PR; #812 is the absolute path parser, #1089 relative text, #893 live lookup). **Proposal only, for the Owner:** retitle the issue "Encode exact local reference strings", remove AC2/AC3 from its body with a link to #1092, drop `depends_on #812`.

### Retired rows (moved out; none deleted silently)

| Retired item (earlier drafts) | Moved out to |
|---|---|
| P1 encode-then-parse round trip | #1092, with the parse; the encode half is E1 |
| P2 empty text, P4 corrupted token, P5 malformed text, P6 distinguishable errors | the unit that owns the exact-string parse (#1092, not settled here) |
| P3 foreign-address check, P7 tokenless text (J4) | #1092 (resolution) |
| P8 parse performs no lookup; the three typed parse errors, the parts type, the parse function and their proposed exports | the parse owner; **not exported by #894** |
| D-3 (#812 as dependency of the parse), D-9 (parser owner), D-10 (split for parse rows) | resolved by "encode only"; the split is re-forecast below |
| E5 `Child("a#b")` note; E6 terminated encode; E7 zero or foreign `Handle` | kept below (E5, E6, E7 and J8) |

## Design

**Token (J1/J2).** String = `ActorPath.String()` + `"#"` + token, exactly one `#` (a path has none: ADR 0031 P-7, `ActorPath.Child` rejects `#`). Token = `epoch "-" number`. **epoch**: 16 lowercase hex digits from 8 bytes of `crypto/rand`, read once per process (`sync.OnceValue` or equivalent), not all zero (redrawn), shared by all Systems of the process: it is process state, not System state. **number**: the decimal value of the numeric part of the actor's ID (`actor-<n>`, `id.go:18`), 1 to 2^64-1, no sign, no leading zero. Token at most 16 + 1 + 20 = 37 characters. No new per-cell field and no second counter.

**Uniqueness.** Exact within a process, several Systems sharing a name included (the counter is process-wide, `id.go:14-18`), while the counter is not exhausted. Across processes only probabilistic: a false match needs equal 64-bit epochs and equal numbers, about 2^-64 per pair of processes (birthday about P^2 / 2^65 for P processes). No absolute uniqueness and no remote resolution is promised; a collision across processes is undetectable. Randomness: `crypto/rand.Read` "never returns an error, and always fills b entirely" and crashes the program irrecoverably if the OS fails (`go doc crypto/rand.Read`, go1.27.2, Executed).

**J3 root and guardians.** Not encodable. Detection: `p.IsRoot() || p.Parent().IsRoot()` (`actor_path.go:65-72`: the parent of the root is the root, the parent of `/user` is the root). Verified: guardians are built only at `guardian.go:174-177` with names `""`, `"user"`, `"system"` (depth 0 and 1); every other actor descends from `/user` through `System.spawnTarget` (`hierarchy.go:23-25`) or from a `Context`, so its depth is at least 2. Test: both predicates over the three guardians and over depth-2 and depth-3 paths.

**J4.** The encoder never emits a tokenless string; how a tokenless input is treated is #1092's. **J7.** The public identity stays `Handle.ID()`; the string only names the incarnation; ADR 0024 wording unchanged.

**J8, results of `ExactRefString(h)`:**

| Input | Result | Why |
|---|---|---|
| a live `Ref`, or the handle from `Context.Parent`/`Child`/`Children` | `(string, true)` | `PathOf` ok and an `actor-<n>` ID |
| nil, typed-nil `*ref`, ref without a cell | `("", false)` | `PathOf` false (`ref.go:116-123`) |
| user double of `Ref`/`Handle`, hand-built cell | `("", false)` | not issued by this package |
| ID-only handle: the stale failed child, `idHandle{id: ..}`; `Terminated.Who()` is an `ID`, out of scope | `("", false)` | no path (#853 G-3) |
| ask endpoint (`ask-<n>`) | `("", false)` | `askEndpoint` has no `actorPath` method (`ask_endpoint.go:43-49`; grep: no hit), so `PathOf` is false; the prefix is also checked |
| root, `/user`, `/system` | `("", false)` | J3 |
| **terminated original `Ref`** (point (b)) | the **same string as while alive** | `ref` keeps its cell (`ref.go:23`); `releaseTerminated` keeps `path` and `cid` (`stop.go:132-146`); no registry lookup, no liveness test |
| handle issued by **another System instance** | encoded with **its own** path and Address | like `PathOf` (#853 G-2). "Foreign" = issued by a different System instance; Address equality is **not** proof of belonging to one (`NewSystem` keeps no name registry). The encoder has no notion of "this System" |

**Condition 3.** The outcome of the future resolver for terminated or stale references (stale error S1, dead-letter-style S2, invariant N1 of #1291) is **not fixed here**; it belongs to #1092.

## Exact API (proposal; G2 pending)

A free function in package `actor`, file `pekko-actor/actor/ref_string.go`, no method added to `Ref[T]` or `Handle`, following `PathOf` (`ref.go:116`). `git grep -i 'ExactRefString\|exactRef\|RefString'` finds nothing on develop `899d93e`: no collision.

```go
// ExactRefString encodes the exact incarnation of the actor behind h as text: the path and a token,
// "pekko://sys/user/a#<epoch>-<number>". ok is false, and the string is "", when h is nil, is not a handle this
// package issued (a user type, a hand-built cell), carries no path (an ID alone, as the stale failed child of a
// decider), names a request endpoint, or is the root or a guardian (/user, /system). A terminated actor keeps its
// encoding: the string is the same as while it was alive, and nothing is looked up. A handle of another System is
// encoded with its own path and Address; an Address does not prove that two handles belong to the same System
// instance. The number is the numeric part of the ID ("actor-<n>"): that decimal format is part of this contract,
// the "actor-" prefix is not. The epoch is random per process and shared by its Systems: the string is unique
// within the process, and across processes only probabilistically (about 2^-64 per pair of processes); no remote
// resolution and no absolute uniqueness is promised. It does not parse, resolve or validate anything, and the
// public identity of a handle remains its ID.
func ExactRefString(h Handle) (string, bool)
```

Name: it says "exact reference" (the ADR 0024 and #1186 wording) and "string", as the sibling of `PathOf`. Alternatives considered: `EncodeRef` (suggests a parse pair and a `Ref` method), `RefString` (loses "exact", could read as the logical path), `ExactRef` (reads as a type). Unexported seams: `encodeExact(h Handle, epoch string) (string, bool)` (tests pass other epochs; the global is never mutated); `processEpoch` (`sync.OnceValue`); `refToken(epoch string, n uint64) string`; `idNumber(id ID) (uint64, bool)` (strips `actor-`, rejects sign, leading zero and zero); `type idCounter struct{ n atomic.Uint64 }` with `next() uint64`; `mint(prefix string, c *idCounter) ID`.

**Export justification (ADR 0011, 0033 D3):** one export, `ExactRefString`. Consumers: #1073 and #957 (a Typed string form), #1092 (test use). Why the public API is not enough: `Handle` carries an `ID` only, no path and no epoch. Not exposed: the epoch, the counter, a parse function, a parts type, errors. The #1020 index is **not** opened; its row is a proposal only (`Exact reference encoder`; location: this section).

## Condition 1: permanent counter exhaustion (design)

`nextID` (`id.go:18`) and `nextAskID` (`ask_endpoint.go:40`) both call `idSeq.Add(1)`; a `uint64` wraps after 2^64 draws (about 584 years at 10^9 per second) and would reuse numbers. Replace `idSeq` by one `idCounter` used by both minters: `next()` loops `cur := n.Load()`; if `cur == math.MaxUint64` it panics with an unexported sentinel (`errIDExhausted`) **without changing the value**, so every later call panics again; otherwise `CompareAndSwap(cur, cur+1)` and returns `cur+1`. The first value is 1, so 0 is never issued; the last number issued is `MaxUint64`. `nextID` is `mint("actor-", &ids)` and `nextAskID` is `mint("ask-", &ids)`, so a wrap seen by one stops both. **Why add-then-check is not enough:** after the wrap the counter continues 1, 2, 3 and reuses numbers, and with concurrency several goroutines can pass a check made before the add. **Panic inside `commit`: decided by the Owner after the cited SHAs.** The exhaustion panic raised in `commit` (`hierarchy.go:156`) releases its own name reservation before it propagates (see "Post-G2-SHA modification": a `defer` in `commitCell`, `spawn.go` only), publishes no partial cell and touches no other reservation; the counter stays exhausted. Edge tests C7 and C8. **Risk:** a CAS loop replaces one `Add` on every spawn and ask; contention under load is measured by the benchmark of the validation plan. The comment at `id.go:14` ("the one process-wide mutable global") must be updated to "the process-wide mutable globals": the epoch and `idMint` join the counter.

**The `idMint` seam.** `idMint` is a package-level function variable in `id.go` that replaces the **minter** (the function that draws a number and formats the ID), **not the counter**; `nextID` calls it. It exists only so that C7 and C8 can make `nextID` panic inside `commit` without exhausting the real process counter, so it is a second process-wide mutable value in production code next to the epoch. C7 and C8 swap it, restore it in `Cleanup`, must **not** run in parallel (no `t.Parallel`) and must run while no other System is spawning. A per-System seam (an option like `commitPoint` or `lockPoint` in `system.go`) was considered and **not chosen**: it would edit `system.go` and `hierarchy.go`, outside the Allowed files the Owner extended (`spawn.go`, and `hierarchy.go` only if the correction needs it). No Allowed file is added for the seam.

## ID numeric format as contract, and accepted differences from Pekko (condition 4)

The numeric part of `actor-<decimal>` becomes a string contract; the `actor-` prefix does not. Ask IDs (`ask-<n>`) draw from the same counter and are never encoded; numbers are not reproducible across runs (asks consume numbers: `ask_endpoint.go:38-40`). Tests that use ID text only build hand-made handles (`git grep -n '"actor-[0-9]' -- 'pekko-actor*/**/*_test.go'` at `899d93e`: 8 lines in 5 test files, re-counted: `ask_endpoint_internal_test.go` 1, `context_stop_internal_test.go` 1, `ref_path_internal_test.go` 2, `watch_api_test.go` 1, `watch_internal_test.go` 3; plus one `"ask-"` mention): none depends on counter values.

| Pekko (source or oracle) | goPekko design (accepted difference) |
|---|---|
| uid is a random non-zero 32-bit `Int` per incarnation (`ActorCell.scala:390-398`), not guaranteed unique | token `epoch-number`: exact in a process, probabilistic across processes |
| `ChildActorPath` appends `#uid` unless undefined (`ActorPath.scala:374-378`, `:449-452`); `RootActorPath` serializes as `toString`, no uid (`:299`) | root and guardians not encodable; Pekko's guardian serialization is not covered by any oracle row (rows 28-37 of `migration/stage-1/actor-853-ref-path/pekko-repro/observed.txt` show path text only) |
| `#0` is a wildcard matching any incarnation (`dungeon/Children.scala:230-244`) | zero is never issued and never a wildcard; no parse here |
| `ActorRef.toString` prints the uid regardless of termination (`ActorRef.scala:188-190`) | a terminated `Ref` keeps its encoding (J8) |
| a bad reference throws or resolves to `deadLetters` (`ActorRefProvider.scala:622-626`) | `("", false)`: no exception, no resolution |

## Pekko facts for the encoder

Source reading: `toSerializationFormat` is part of `ActorPath` (`ActorPath.scala:248`); the uid is internal (`:264`); equality of refs uses uid and path, of paths ignores uid (`ActorRef.scala:183-186`, `ActorPath.scala:163-168`). Oracle (Executed, Pekko v1.7.1 in a disposable copy; `migration/stage-1/identity-oracle/pekko-repro/observed.txt`, integrated by PR #1292 as `d7af9bc`, 196 rows, rows re-checked on develop `899d93e`): row 15 `ua.toSerializationFormat` = `pekko://sys/user/a`; row 17 `root.toSerializationFormat` = `pekko://sys/`; row 23 `ua.withUid(5).toSerializationFormat` = `pekko://sys/user/a#5`; rows 183-185 two incarnations of one name serialize differently (`stale.uids differ` true: a relationship, as uids are random).

## Criteria -> scenario -> evidence

Unit level, `pekko-actor/actor/`, go-specs matchers, doubles via `mock.Controller`/`Spy`, no sleeps. **Go-only** = no Pekko equivalent, alternative evidence (ADR 0033 D13): accepted by the G2 message as the evidence method; its fulfilment requires running and reviewing the tests.

| # | Scenario | Evidence class |
|---|---|---|
| E1 | encode a live `Ref`: path part equals the `PathOf` text, token `<epoch>-<n>`, one `#` | Pekko READ + Executed (row 15); the token is Go-only |
| E2 | root, `/user`, `/system` give `("", false)` | Go-only (Pekko serializes the root without a uid; guardians not executed) |
| E3 | two incarnations of one name: strings differ only in the number | Pekko Executed (rows 183-185, a relationship); the Go token is Go-only |
| E4 | the same `Ref` encoded twice gives one string | Go-only (Pekko: single calls only, rows 15 and 23) |
| E5 | `Child("a#b")` fails, so no `#` can reach the path part | READ; existing `Child` tests |
| E6 | a terminated `Ref` (also after retirement) gives the same string as alive; no lookup | Pekko READ (`ActorRef.scala:188-190`, not executed); the Go result is Go-only |
| E7 | the `("", false)` table of J8: nil, typed-nil, nil cell, user double, hand-built cell, ID-only handles, ask endpoint | Go-only |
| E8 | a handle of another System (also one with the same name) encodes with its own path and Address | Go-only |
| E9 | the token builder with other epochs: format, length at most 37, epoch zero never produced; `idNumber` rules | Go-only |
| E10 | the new non-test files import and call no resolver or parser (grep) | grep |
| E11 | README states the contract, the ID numeric format and the probabilistic scope | doc check |
| C1 | counter on its own instance: near-max sequence, last number `MaxUint64`, then panic | Go-only |
| C2 | exactly k draws succeed, every later draw panics | Go-only |
| C3 | after a recovered panic the next draw panics again and no number repeats | Go-only |
| C4 | `mint(prefix, c)` shares state: a wrap through `ask-` stops `actor-` and the reverse | Go-only |
| C5 | N goroutines race across the boundary: exactly the remaining numbers are issued, each once, the rest panic, no duplicate | Go-only |
| C6 | the first value is 1; C1-C6 use their own `idCounter` instances and never touch the process-wide counter (C7 and C8 replace the minter, not the counter: see the `idMint` seam) | Go-only |
| C7 | exhaustion panic from a spawn, **recovered** (`idMint` seam): the reservation is released (`outstanding(n) == 0`, no `n.names` entry, the name is not `ErrNameTaken` afterwards); the owner completes its stop (stop the parent: its `done` closes and `outstanding` of its node is 0, as the `closedDone` helper and `System.outstanding` show); no partial actor (`len(byID)`, `Children()`, `Child(name)` unchanged); the counter stays exhausted: the next spawn panics again and releases its own reservation; no number is reused | Go-only |
| C8 | two reservations in one owner (a second reserved through `commitPoint`): the first spawn's panic releases only its own (`outstanding` drops by one, the other entry and token are intact); the other then commits normally with a fresh minter and a number never issued before | Go-only |

Go-only cases, explicitly: the token form, the epoch, the `("", false)` results, the root and guardian refusal, foreign System handling, counter exhaustion and its permanence, the ID numeric contract (E2, E4, E7-E9, C1-C8 and the Go side of E1, E3, E6). Optional extra evidence, **a proposal for the Owner**: a Classic scenario in the kit run that serializes a ref after `stop` (E6), in a disposable copy as in PR #1292. **Kit run required** (per the relayed decisions, point (i)): package `migration/stage-1/ref-encode-894/{manifest.tsv,RULEBOOK.md,RUNBOOK.md,EXECUTION-RECORD.md}`, created in the future implementation PRs; this PR writes no kit file.

## Allowed files (future implementation; anything else is a stop)

| Path | Edit | State at develop `899d93e` |
|---|---|---|
| `pekko-actor/actor/ref_string.go` | new: `ExactRefString`, token builder, `processEpoch`, `idNumber` | absent |
| `pekko-actor/actor/ref_string_test.go` | new: E1-E9 | absent |
| `pekko-actor/actor/id.go` | `idCounter`, `mint`, `ids`, `nextID` (the unexported package-level function variable `idMint`: the **minter** that draws and formats `actor-<n>`, not the counter; the only test seam of C7 and C8, see below); update the `:14` comment | exists |
| `pekko-actor/actor/ask_endpoint.go` | the `nextAskID` line only (`:40`) | exists |
| `pekko-actor/actor/spawn.go` | only the about 7 lines of the release in `commitCell` (`:160`); **added by the Owner after the cited SHAs** | exists |
| `pekko-actor/actor/hierarchy.go` | **only if** the correction cannot be made in `spawn.go` (not expected; a stop to report otherwise); also authorized by the Owner after the cited SHAs | exists |
| `pekko-actor/actor/id_counter_internal_test.go` | new: C1-C8 (no id test file exists in `pekko-actor/actor`, `git ls-files`) | absent |
| `pekko-actor/README.md` | module contract, implemented vs planned | exists |
| `migration/stage-1/ref-encode-894/{manifest.tsv,RULEBOOK.md,RUNBOOK.md,EXECUTION-RECORD.md}`, `migration/cost-log.tsv` (one row) | kit package | absent; exists |
| `docs/scopes/894-encode-exact-ref-string.md` | this file | exists |

`hierarchy.go:156` calls `nextID()`; the signature is unchanged and `hierarchy.go` is not edited unless the row above applies. `git grep 'idSeq\|nextID\|nextAskID'` over `pekko-actor*` finds only `id.go:14-18`, `ask_endpoint.go:38-40,53` and `hierarchy.go:156`; no test references them. SQLite parity (CLAUDE.md): not applicable, no persistence module is touched. Nothing under `docs/adr/` changes here; the ADR 0031 addendum is #1186's.

## Validation plan (AGENTS.md commands, once per final tree)

`cd pekko-actor && go test -count=1 ./...` (`unshare -rn` is unavailable on macOS: record the exit, accepted under G-6), `gofmt -l`, `go vet ./...`, `go mod tidy -diff`, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh` against the current base (the 7 pre-existing errors; a new error stops), golangci-lint (gocyclo <= 10), per-module coverage and `scripts/check_coverage.sh "$COV" 85`, diff-cover 11.0.0 >= 85%, `make ci-fast`, `.github/scripts/go-sdk-update.sh --check`, `git diff --check`. Plus the isolated counter tests C1-C8; a **differential** of Spawn, SpawnAnonymous and Ask ID minting on a scratch copy of the base against the change (same ID sequence and text for the same inputs); a **micro-benchmark** of `nextID` and `nextAskID` before and after (the CAS loop cost, a stated risk). No `-race` unless the Owner asks. **PR budget:** forecast about 230 authored lines for the counter PR (`id.go`, `ask_endpoint.go`, the about 7 lines of `spawn.go`, C1-C8: C7 and C8 add about 70) and about 215 for the encoder (code, E1-E9, README), about 445 together before the kit package: **split into the counter PR first, then the encoder PR**, contract unchanged; each stays under 400. The kit package goes in its own PR as in #852 and #853.

## ADR impact (ADR 0033 D5)

ADR 0031 P-7/P-8 (the addendum is accepted by the Owner in the terms presented, #1291; applying it to the ADR text is a separate step, not done here): no serialization form exists today; the token is plain ASCII with no `%`, nothing escaped. P-6: the token lives in the string, not in `ActorPath` equality. ADR 0024: identity stays the `ID`; the numeric part becomes a string contract, and `I2` ("an ID is never issued twice") gains the permanence rule. ADR 0011/0033 D3: one export, above. This PR amends no ADR file.

## Decisions status

**Decided by the Owner (message of 2026-10-29, above):** G2 on #1288 `d65884d` and #1291 `5ac4ae8`; the contract of this scope (encode only, `ExactRefString(h Handle) (string, bool)`, token epoch-number, one shared counter with permanent exhaustion, the enumerated encodable and non-encodable cases, the root and guardians not encodable, a terminated original `Ref` keeps its encoding); the ADR 0031 addendum accepted in the terms presented (token form for encoding only); kit execution, implementation and independent QA authorized; Go-only cases accepted as the evidence method (their fulfilment requires running and reviewing the tests); the (m) alternative of a hanging reservation **rejected**, with the release before the panic. **Still Owner-pending or not authorized:** merge, issue edits or closure (title, AC text, labels, `depends_on #812`), the #1020 index, remote CI and `-race`; every resolution question of #1291 (Q3, Q4, Q5, Q9; Q8 with #1187) stays not decided. QA of this documentary delta precedes any implementation.

## Risk

Risk: none for this PR (documentation only). For the future code: item 3 (a new export, and the ID numeric format becomes a consumed contract) and item 4 (concurrency: a guarded counter on every spawn and ask); both are named high-risk items, so the code PRs get an independent verifier. None of this is implemented.
