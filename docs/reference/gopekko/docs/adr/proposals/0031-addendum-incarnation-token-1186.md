# ADR 0031 addendum (proposal): the incarnation token in an exact reference string

- Status: **ACCEPTED by the Owner on 2026-10-29 in the terms presented** ("Acepto la adenda de ADR 0031 en los términos presentados", message quoted in the next section), for the token form for **encoding** in #894 only. The file stays in `docs/adr/proposals/` and `docs/adr/0031` is **not** edited by this PR: applying the addendum to the ADR text is a separate step, not done here. The resolution questions (Q3, Q4, Q5, Q9; Q8 with #1187) are **not decided** and the Owner approved no additional clause by reference. Any "recommended" or "orientation" value outside the Owner section is an Architect proposal. Nothing is implemented.
- Story: #1186 (GOV-41). Consumers: #894 (IDN-09), #1092 (IDN-21), #853 (IDN-05). Sibling proposal on path text: [`0031-addendum-actor-path-text-grammar-1187.md`](0031-addendum-actor-path-text-grammar-1187.md) (#1187).
- Date: 2026-10-29. goPekko revision read: `7840e8893305b0817b76d7b7f6e7d209afe0b8d7`; facts re-checked against `develop` `899d93e0f13f2e7673652b2a060329d9ee8f6b51` where marked (revision 5).
- Revision note (all rounds, in order): (1) ADR 0032 D5 status aligned to the ADR (D5a/D5b decided 2026-10-28); injection seam and shared-minter rules (`idSeq` also feeds `nextAskID`) specified for the process epoch; `crypto/rand` behavior Executed (`go doc`); `ActorRef.scala` and `address.go` citations corrected; P-7 line `:95` made consistent. (2) The oracle was run (196 rows) and is cited from PR #1292, revision 2 of the probe, whose element lists print as `n=<size> [...]` (none of the rows quoted here prints an element list; the stale-resolution rows below were re-checked against that file and are unchanged). (3) Owner question 10 asks exactly one decision, A (#894 owns encode and parse of exact-reference strings, #1092 consumes) or B (#1092 owns the parse), and states the single order and the three parse errors (below); alternatives (a) to (d) unchanged; "deterministic for equal spawn order" corrected. (4) Last QA findings: number 0 at the wrap is refused (panic), never skipped; "no `#`" listed as MALFORMED only provisionally pending question 5; the cross-reference to the oracle section now names "Differential oracle (executed)" in #1187's proposal. Owner decision still `pending`. (5) Revision 5 (2026-10-29): records the Owner design decisions of that date, marks the resolution questions as deferred to #1092, applies the permanence rule to the exhaustion row, and fixes facts against develop `899d93e` (PathOf exists, the commit citation, the quick path); the resolution proposals below are otherwise untouched.
- Pekko: tag `v1.7.1`, commit `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. Paths are relative to
  `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor` (so `dungeon/Children.scala` is `.../actor/dungeon/Children.scala`).
- **Evidence classes** (every claim carries one):

| Class | Meaning |
|---|---|
| **Source** | Read in the pinned Pekko, or in this repository (**Go today**) at the revision above, with file and line |
| **Executed** | Run by the author in a scratch directory outside the repository, or a read-only query. In this document: the arithmetic of the collision bounds (`python3 -I`, `math.expm1`), and `go doc crypto/rand.Read` on Go `go1.27.2` `darwin/arm64` |
| **Executed (oracle)** | Pekko v1.7.1 run locally on 2026-10-29; see "Differential oracle feasibility" |
| **Hand trace** | Derived by reading code. Not run |
| **JDK-unverified** | Behavior of the Java library, not run |
| **Go design proposal** | A choice made here. Not evidence |

Pekko behavior marked **Executed (oracle)** below was run on the pinned v1.7.1 after this document's first revisions (offline `sbt actor/compile` in a disposable `git archive` copy, then a probe; 196 rows; durable artifacts: draft PR #1292, `migration/stage-1/identity-oracle/pekko-repro/observed.txt` at commit `d4ac1c479f478254bddd1f8dc44ea5d35f35eea1`, revision 2). No Go code was run and no Go parity is claimed.

## Owner message of 2026-10-29 (G2 grant and addendum acceptance), as received, and the earlier decisions

Complete message of 2026-10-29, quoted verbatim (Spanish): "Concedo G2 de #894 sobre: #1288: d65884dbbb5aa5c17b9efa197c43558b0385a6c7 ; #1291: 5ac4ae89ecc47a1acbb989fec0d632ac4be04255. Apruebo el contrato presentado: solo codificación, API ExactRefString(h Handle) (string, bool), token época-número, contador compartido con agotamiento permanente y los casos codificables/no codificables enumerados. Esto incluye raíz y guardianes no codificables y referencia original terminada con codificación conservada. Acepto la adenda de ADR 0031 en los términos presentados. Autorizo ejecutar el kit, implementar y obtener QA independiente. Los casos solo-Go quedan aceptados como método de evidencia; su cumplimiento requiere ejecutar y revisar las pruebas. Sobre (m): rechazo dejar una reserva colgada. El agotamiento debe liberar la reserva pendiente antes de propagar el panic, sin publicar una celda parcial ni alterar las demás reservas. El contador permanece agotado. Amplío Allowed files a spawn.go y hierarchy.go exclusivamente si son necesarios para esa corrección. Elegí el cambio mínimo. Probá que, después de recuperar el panic: - la reserva se libera; - el dueño puede completar su cierre; - no se reutilizan IDs ni se publica un actor parcial. Registrá esta modificación posterior a los SHA citados y obtené QA del delta documental antes de implementar. No necesitás otra aprobación si se mantiene exactamente este contrato. Las lecturas (a)-(m) quedan confirmadas solo en cuanto coincidan con las decisiones expresas de este mensaje y las anteriores; no apruebo cláusulas adicionales por referencia. Mantené las excepciones de validación ya aceptadas. Sin CI remoto ni -race. Sin merge, cierre o edición de issues ni creación del índice #1020. Entregá PRs draft, SHA exactos, QA y resultados locales." It covers: the contract of #894 (encode only), the token form (d) for encoding, the shared counter with permanent exhaustion, the root and guardians not encodable, the addendum accepted in the terms presented. It does **not** decide Q3, Q4, Q5, Q9 (#1092), Q8 (#1187) or the parse owner. Earlier the same day: the Owner's message reached the coordinator **fragmentary**; the fragment is kept as a trace only (not a verbatim quote); the classification below applies the later, complete G2 message to it. Fragment as received (trace only): "Apruebo decisiones diseño #894: Solo codificación, resolución. J1/J2: path#<época>-<número>, proceso, compartida Systems. Unicidad probabilística, remota. J3: guardianes no codificables. J4: entradas #1092. J7: el ID. J8: comportamiento propuesto, resultado ("", false) no Terminated.Who() Encoder como Handle, ni Handle. Proponé scope final. Kit requerido solo-Go Pekko, 1. agotamiento números, panic. concurrencia aisladas. 2. eliminar AC2/AC3: No 3. No fijar #1092. 4. ID Pekko. PRs #1186 #894, QA delta, SHA G2. Incluí API". Reconstructed decisions (the exact design is in the #894 scope, PR #1288):

- **Encode only** in #894: no parse, no resolution.
- **J1/J2, token form (d) `path#<epoch>-<number>`:** random process epoch shared by all Systems of the process; uniqueness across processes only probabilistic, no absolute uniqueness and no remote resolution promised. **Accepted by the Owner on 2026-10-29** ("token época-número"; "Acepto la adenda de ADR 0031 en los términos presentados"), for encoding only; applying the addendum to the ADR text is a separate step.
- **Express in the G2 message** (the letters are those of the #894 scope's points (a)-(m)): (a) **J3** root and guardians not encodable; (b) a terminated original `Ref` keeps its encoding, and the "casos codificables/no codificables enumerados" of the J8 table apply (`Terminated.Who()`, an ID-only handle, is covered only as one of those enumerated non-encodable cases); (c) the encoder is a free function `ExactRefString(h Handle) (string, bool)`, no method on `Ref` or `Handle`; (e) **Condition 1**: the counter is shared by both minters and its exhaustion is permanent (never wrap or reuse a number, even after a recovered panic; isolated edge tests with concurrency; see the Exhaustion row); (h) the G2 grant and what it authorizes (listed below); (m) decided; (i) kit required, Go-only evidence accepted as the method.
- **Matching the earlier, fragmentary message only** (confirmed to that extent; no clause approved by reference): (d) AC2/AC3 of #894 not deleted silently; (f) the future resolver outcome for terminated or stale references not fixed (#1092); (g) the ID numeric format as a documented contract with the accepted differences from Pekko; (j) **J4**, tokenless input is #1092's and the encoder never emits one; (k) **J7**, the public identity stays the ID; (l) no absolute uniqueness and no remote resolution promised.
- **Not expressly decided by the Owner's messages:** the 16-hex epoch width and the guarded-counter code shape; they are specified in the #894 scope as the design presented.
- Authorized by the G2 message: kit, implementation, independent QA. Not authorized: merge, issue edits, the #1020 index, remote CI, `-race`.

Questions below, by number: **Q2** token: (d) accepted for encoding (G2 message). **Q7** identity: the ID stays the public identity (J7). **Q6** root or guardian: root and guardians not encodable (G2 message). **Q1** (strings may leave the process, uniqueness probabilistic): matches the earlier fragment only, point (l). **Q3** (stale outcome S1/S2), **Q4** (invariant N1), **Q5** (tokenless text), **Q9** (foreign epoch vs stale): resolution questions, **deferred to #1092 and not decided**. **Q8** (`#` in the path parser) stays with #1187. **Q10** (parse owner) is out of #894 ("encode only"); who owns the parse is not settled here. Text below that assigns the parse to #894 (option A) is superseded for #894 by "encode only" and kept only as proposal text for the future owner of the parse.

## Quick path

1. Encode design: closed by the Owner decisions above (token (d), probabilistic across processes, root and guardians not encodable). The #894 scope is the final design; G2 for #894 was granted by the Owner on 2026-10-29 (message above).
2. Still open for the Owner: the ADR addendum acceptance, the `#` rule shared with #1187 (Q8), and the resolution questions deferred to #1092 (Q3, Q4, Q5, Q9).
3. #1092 is not unblocked by this: it still needs #893 and a settled parse owner; the oracle is on develop (PR #1292, integrated).

Alternative (d), "process epoch plus incarnation number", was accepted by the Owner for encoding on 2026-10-29 (message above), and is specified in "Token specification" (the epoch width and the counter code shape are not expressly decided, see above). If the Owner says
strings never leave the process, (a) is enough and cheaper.

## Question

What token does an exact local reference string carry so that a reference to an old incarnation can **not** resolve to a new actor that reused the name
(#894: "Name reuse cannot make an old encoded reference resolve to the new actor"; #1092 AC-R2-1: "the new actor is not returned and the stale outcome
occurs"; #853: "A stale exact reference cannot deliver to the replacement actor")? The word "never" in those criteria is only as strong as the scope in
which the token is unique; the scopes are separated below.

Baseline (Source, Go today): no string form exists. `ActorPath` has no accessor of an incarnation, `String()` never shows one (ADR 0031 P-7), and `ActorPath` has a path accessor through the free function `PathOf(h Handle) (ActorPath, bool)` (`pekko-actor/actor/ref.go:116`, #853 integrated; no method on `Ref`; `ref.go:18-21`), and no encoder exists.

| Concern | Question | Where decided |
|---|---|---|
| Syntax | What the token looks like and where it sits in the text | This proposal, jointly with #1187 |
| Identity | What a `Ref` is, and what makes two incarnations different | ADR 0024 (identity is the ID), #853 |
| Resolution | Finding the live actor, or reporting live, stale or foreign | #894, #1092, #893 |

## Pekko evidence (Source)

| # | Claim | Location |
|---|---|---|
| E1 | `toSerializationFormat` is declared abstract on `ActorPath` | `ActorPath.scala:248` |
| E2 | `ActorPath` carries `private[pekko] def uid: Int`; equality of paths ignores it | `ActorPath.scala:264`, `:163-168` |
| E3 | `RootActorPath.toSerializationFormat = toString`: a root carries **no** uid | `ActorPath.scala:297-299` |
| E4 | `ChildActorPath.toSerializationFormat` builds the path and appends the uid fragment | `ActorPath.scala:374-378` |
| E5 | `appendUidFragment` appends `#` and the `Int` unless the uid is `undefinedUid` | `ActorPath.scala:449-452` |
| E6 | `undefinedUid = 0`; `newUid()` takes `ThreadLocalRandom.current.nextInt()` and retries on 0, so a uid is a random non-zero **signed** 32-bit integer: 2^32 - 1 values | `ActorCell.scala:390-398` |
| E7 | `splitNameAndUid` cuts at the first `#` and parses the rest with `Integer.valueOf`; a non-numeric fragment throws (empty fragment: Executed, oracle row `ActorPath.fromString('pekko://sys/a#')` throws `NumberFormatException`) | `ActorCell.scala:400-404` |
| E8 | `ActorRef.equals` is `path.uid == other.path.uid && path == other.path`; `toString` prints `#uid` unless the uid is undefined; `hashCode` uses the uid when defined | `ActorRef.scala:183-186` (equals), `:188-190` (toString), `:175-178` (hashCode) |
| E9 | `getSingleChild` with `#uid`: the live child is returned **only if** `uid == undefinedUid \|\| uid == crs.uid`; otherwise `getFunctionRefOrNobody(childName, uid)`, which gives `Nobody`. A mismatching uid never reaches the actor that reused the name; **uid 0 matches any incarnation** | `dungeon/Children.scala:230-244`, `:72-75` |
| E10 | `resolveActorRef(String)`: an unparsable path or a foreign address gives `deadLetters`; an unmatched path gives `new EmptyLocalActorRef(...)`, not an error | `ActorRefProvider.scala:622-627`, `:629-638`, `:643-657` |

Gap: no spec in `actor-tests` asserts a stale uid on resolve (`LocalActorRefProviderSpec.scala` has no uid case). The stale behavior of E9 and E10 is Source reading **and Executed (oracle)**: see "Differential oracle feasibility" below. ADR 0033 D13 applies only to what the pinned code cannot answer.

In Pekko a missing path and a stale uid produce the **same** result (`Nobody`, then `EmptyLocalActorRef`).

## Go today (Source)

| Fact | Location |
|---|---|
| `type ID string`, documented as identifying one actor "for the life of the process" | `pekko-actor/actor/id.go:8-9` |
| One process-wide counter `idSeq` (`atomic.Uint64`); `nextID()` returns `"actor-" + decimal(counter)` | `id.go:14-18` |
| The ID is allocated at commit, under the System lock, in the same critical section that publishes the record | `pekko-actor/actor/hierarchy.go:149-169` |
| ADR 0024 I2: "An ID is never issued twice (`idSeq` is monotonic)". Holds **within one process**: the counter restarts at 1 in every process | `docs/adr/0024-actor-registry-retention-after-termination.md:158` |
| ADR 0024: "in Go identity is the ID, not path plus uid (adaptation)" | `0024-...md:206` |
| A retired entry is removed at completion (alternative B, accepted); an ID-only handle then gets `ErrUnknownHandle`, which also means "never issued by this System" | `0024-...md:126`; `errors.go:44-47` |
| ADR 0024 alternative D (per-System ID sequence) was offered and would change the ID format | `0024-...md:128` |
| System name: `NewLocalAddress` rejects only an empty name; default name `"default"` (ADR 0031 O3b), so two Systems or two runs can share it | `address.go:42-50` |
| `nextAskID()` draws from the **same** `idSeq`: `"ask-" + decimal(idSeq.Add(1))`, so ask identities and actor identities interleave in one counter | `pekko-actor/actor/ask_endpoint.go:38-40` |

## Guarantee scopes

Three scopes. "Exact" means no input can produce a false match. "Probable" means a false match has a stated, non-zero probability. "Not guaranteed" means
a false match can be produced deterministically.

| Scope | What it means in goPekko |
|---|---|
| S-1 one running System | Name reuse inside one `System`, the case of #894/#1092/#853 |
| S-2 several Systems, same process | `idSeq` is process-wide (`id.go:15-18`), so IDs differ across Systems; the system name may be equal |
| S-3 restart or another process | The counter restarts at 1 (`actor-1`, `actor-2`, ...); two processes that spawn in the same order mint the same IDs |

| Alternative | S-1 | S-2 | S-3 |
|---|---|---|---|
| (a) `path#ID` | **Exact**: IDs are not reissued in a process (ADR 0024 I2, Source) | **Exact**: process-wide counter | **Not guaranteed**: `…/user/a#actor-7` from process A matches a live `/user/a` with ID `actor-7` in process B; the same number is minted only when the two processes made the same sequence of ID draws, and `nextAskID` draws from the same `idSeq`, so earlier asks shift the numbers |
| (b1) new random 32-bit uid (Pekko shape, E6) | **Probable**: see bounds below; retired incarnations are not remembered | **Probable** | **Probable**, same per-attempt bound |
| (b2) new random 64-bit uid | **Probable**, bound about 2^32 times smaller | **Probable** | **Probable** |
| (c) no token | **Not guaranteed**: a reused name resolves to the new actor. Fails the requirement | same | same |
| (d) process epoch plus incarnation number (recommended) | **Exact**: the number is the ID's counter value | **Exact** | **Probable**: needs the 64-bit epochs to be equal **and** the numbers to be equal |

### Collision arithmetic (Executed: `python3 -I`, `p = 1 - exp(-k(k-1)/(2M))`)

A uniform random value over `M` equally likely non-zero values never "eliminates" collisions; it makes them improbable. Pekko's uid has `M = 2^32 - 1 =
4,294,967,295` values (E6). For a stale reference held against one current incarnation, the chance of a false match is `1/M = 2.33e-10` per reference. The
chance that **some two** of `k` incarnations at one path share a value (the birthday bound, relevant if every pair must stay distinguishable):

| k incarnations of the same path | M = 2^32 - 1 (b1) | M = 2^64 - 1 (b2) |
|---|---|---|
| 2 | 2.33e-10 | 5.4e-20 |
| 100 | 1.15e-6 | 2.7e-16 |
| 1,000 | 1.16e-4 | 2.7e-14 |
| 10,000 | 1.16e-2 (about 1.2%) | 2.7e-12 |
| 65,536 | 3.9e-1 (about 39%) | 1.2e-10 |
| 1,000,000 | 1.0 | 2.7e-8 |

These figures are a model, not a measurement: they assume a uniform source and independence. They say nothing about a particular implementation's
generator.

**What the System can detect on a collision.** At mint time (the commit, `hierarchy.go:149-169`) the System knows only the incarnations it still holds:
the live ones and the three guardians (`byID`). It can redraw when the new value equals a **registered** one. It cannot detect equality with a **retired**
incarnation (ADR 0024 alternative B removes the entry at completion; remembering retired values would add unbounded memory or a retention policy that
ADR 0024 rejected), nor with any incarnation of another process. So for (b1) and (b2) a collision with a retired incarnation, which is exactly the stale
case, is undetectable by design and stays at the bounds above. For (a) and (d) no collision exists in a process, so no detection or retry is needed.

## Alternatives

| | (a) `path#<ID>` | (b) `path#<uid>`, new random uid (b1 32-bit, b2 64-bit) | (c) none | (d) `path#<epoch>-<number>` |
|---|---|---|---|---|
| Consequence for ID format | ID text becomes a **string contract**; adopting alternative D of ADR 0024 later would require changing the token scope (for (a), also invalidating every string already issued) | None | None | The ID's **numeric part** becomes a string contract; the `actor-` prefix does not. Adopting alternative D later would require changing the epoch scope |
| ADR 0024 retention | None (resolution by path plus comparison) | None | None | None |
| Equality (P-6) | None | None | None | None |
| New state | none | a uid per cell (#853) and a generator | none | one process epoch (a constant per process); no new field per cell |
| Cost | Lowest | Medium | Zero, unusable | Low |
| Consequence for #894 and #1092 | Smallest; documents the process-lifetime limit | Needs the field first | AC "old reference does not resolve to the new actor" cannot be met | Same as (a) plus the epoch rule |

**Why `ID` alone cannot serve when strings leave the process.** The ID is stable and unique only inside one process (`id.go:8-9`, ADR 0024 I2). If a string
must survive a restart or cross to another process, it needs a component that differs between processes; `ID` has none (the S-3 column). If strings never
leave the process, `ID` is enough and no second value is justified: that is alternative (a) and Owner question 1 decides it.

(c) is the do-nothing option and is listed only to be rejected on the stated requirement.

## Token specification (accepted by the Owner on 2026-10-29 for encoding, in the terms presented)

Alternative (d). It is specified so that each line can be a test.

| Item | Specification (Go design proposal) |
|---|---|
| Text form | `<path-text>#<token>`, path text per #1187 (which contains no `#`), exactly one `#`, the token ends the string (a text with no `#` is provisional: malformed or path selection, pending question 5) |
| Token syntax | `token = epoch "-" number`. `epoch` = exactly 16 lowercase hexadecimal digits `[0-9a-f]`, not all zeros. `number` = decimal digits, no sign, no leading zero, value 1 to 2^64 - 1 (at most 20 digits). Maximum token length is 37 characters |
| Canonical form | One string per token (fixed-width lowercase epoch; number without leading zeros), so string equality is token equality. Uppercase hex, leading zeros, `+`, spaces and `%` are malformed |
| Zero | Epoch zero and number zero are never minted and are malformed; zero is never a wildcard (Pekko: `#0` matches any incarnation, E9) |
| Relation to `#` of #1187 | The path parser rejects `#` (#1187, G7). A separate exact-reference function splits at the single `#`, parses the prefix with the #1187 grammar and the token with this syntax. The token has no `%` |
| Root | A token on a root (`pekko://sys/#...`) is malformed (Pekko roots carry none, E3). Whether a guardian has an exact string is Owner question 6 |
| Generation | `number` is the numeric value behind the `ID` allocated by `nextID()` at cell commit, in the same critical section that publishes the record (`hierarchy.go:149-169`). `epoch` is 8 bytes from the operating system's cryptographic source, read once per process at first use and then constant. The token is **computed** from these two, so no new per-cell field and no second counter exist |
| Randomness failure | Executed (`go doc crypto/rand.Read`, Go `go1.27.2`): `Read` "never returns an error, and always fills b entirely"; it "crashes the program irrecoverably if an error is returned", and the default reader's OS APIs "are documented to never return an error on all but legacy Linux systems". So fail-closed is provided by the standard library by crashing, not by an error the caller can handle; the contract is "never a predictable epoch" and the zero epoch is redrawn (probability `2^-64`) |
| Process epoch: state and test seam | The epoch is a **second** process-wide mutable value next to `idSeq` (`id.go:14` calls `idSeq` "the one process-wide mutable global"; this would no longer be true, and the comment and ADR 0024 wording would need updating). Specified seam (Go design proposal): the token is built by a function that takes the epoch as an argument; the production code passes the once-initialized process epoch; tests build tokens with another epoch value through that function, so the foreign-epoch case needs no global mutation and no second process. The epoch itself is read through one unexported variable initialized once (`sync.Once` or equivalent), never reassigned in production |
| Uniqueness scope | Exact in S-1 and S-2 (process-wide counter, ADR 0024 I2). Probable in S-3: a false match needs equal epochs (about `2^-64 = 5.4e-20` per pair of processes, birthday bound about `P^2 / 2^65` for `P` processes, Executed row above) **and** an equal number at the same path |
| Collision handling | Within a process none is possible (the counter does not reissue). Across processes it is undetectable (no shared state) and stated as a limit in the contract. No redraw or retry exists |
| Exhaustion | The counter is a `uint64`; 2^64 mints at 10^9 per second is about 584 years (Executed). **Permanent exhaustion** (Owner condition 1, express in the G2 message): `nextID` today would wrap to reuse; both minters, `nextID` and `nextAskID` (`ask_endpoint.go:40`), must draw from one guarded counter whose `next()` uses a compare-and-swap loop and never increments past `math.MaxUint64`: at that value it panics with an unexported sentinel and stays exhausted forever, so a recovered panic never lets a later call wrap or reuse a number; number 0 is never issued (the first value is 1). A plain add-then-check is not enough: after the wrap the counter would continue 1, 2, 3 and reuse numbers, and with concurrency several goroutines could pass the check. A wrap seen by one minter stops both. Validation: isolated edge tests on their own counter instances (near-max draws; exactly k draws then panic; a repeated panic after recovery with no repeated number; N goroutines racing across the boundary with each remaining number issued once and no duplicate), never on the process-wide counter. The hot-path cost of a CAS loop instead of one `Add` is a recorded risk with a differential and benchmark in the validation (#894 scope) Decision (m), decided by the Owner after the cited SHAs: an exhaustion panic inside `commit` (`hierarchy.go:156`, under `System.mu`) **releases its own pending name reservation before the panic propagates** (a `defer` in `commitCell` calling the existing `rollback`), publishes no partial cell and leaves other reservations untouched; the counter stays exhausted; a hanging reservation is rejected. Analysis, intended code and edge tests C7 and C8: #894 scope, "Post-G2-SHA modification" |
| Not a secret | The number is predictable (spawn order). The token is an incarnation name, not a capability, and grants no authority |

Order of evaluation (Go design proposal; one order, stated in the decision package #1293 and, as a proposal, in the #1092 scope; for #894 (superseded for #894: encode only; see the Owner design decisions section)). **Parse** (owner #894, option A of question 10 (superseded for #894: encode only; see the Owner design decisions section); pure syntax, System-independent, no lookup, calls #812's absolute path parser): step 1 empty text: typed error EMPTY; step 2 `#` count and token split; step 3 absolute-path prefix through #812 (a prefix that is a root fails too): MALFORMED; step 4 token syntax: CORRUPTED TOKEN. **Resolution** (owner #1092, consumes #894 and #893): step 5 the address differs from the System's: **foreign**; step 6 the epoch differs from the process epoch (minted by another process or run): **foreign** (question 9 decides foreign or stale); step 7 look the **path** up, never the ID (so ADR 0024 retention is irrelevant, a retired actor needs no entry to be called stale); step 8 the live actor's number equals the token's number: live; otherwise, or when the path is missing: **stale**. Parse cannot know this System's address, so a corrupted token with a foreign address is reported malformed first: order malformed, then foreign, then stale.

| Error category | Raised when | Notes |
|---|---|---|
| Malformed (parse, #894) | three typed parse errors, all in this one category: **empty** text; **malformed** (more than one `#`; no `#` is provisional, pending question 5, a root, a wrapped #1187 path error); **corrupted token** (empty token `path#`, any violation of the token syntax) | classified errors; names left to #894 under ADR 0011 |
| Foreign (resolution, #1092) | address (protocol or system name) differs from the System's, or the epoch differs from the process epoch; decided only after parse succeeded | typed error (#1092); Pekko gives `deadLetters` (E10). Owner question 9: should an epoch mismatch be merged into stale? |
| Stale | well-formed, same System and epoch, path missing or live actor has another number | outcome S1 or S2 below. A retired and a never-issued number are not told apart, as with `ErrUnknownHandle` (ADR 0024 B) |

The stale outcome must not be `ErrUnknownHandle`: that sentinel is for ID-only handles (`errors.go:44-47`) and keeps its meaning.

### Stale outcomes (Go design proposal)

| Option | What `Resolve` returns | Consequence |
|---|---|---|
| S1 typed error | no reference and a "stale" sentinel (name left to #894) | Idiomatic Go; caller cannot send by mistake; differs from Pekko (E10); missing and stale may be told apart |
| S2 dead-letter-style reference | a reference whose `Tell` goes to dead letters, as Pekko's `EmptyLocalActorRef` (E10) and as ADR 0024 does for a terminated actor (`0024-...md:205`) | Closest to Pekko; a caller may not notice staleness; needs the dead-letter sink |

### PROPOSED INVARIANT N1 (not accepted; Owner question 4)

An obsolete exact reference is **never** redirected by path to the actor that reused the name. If the token does not match the live incarnation, the result
is the stale outcome, never the actor at that path. Selecting by path alone is a separate, explicit operation (#893: "neither spawns an actor nor converts an
exact reference into a path alias"). Resolution compares token and path; it does not trust either alone. Derived from E9; Pekko shows the same behavior in Source.

## Scenarios

| # | Scenario | (a) ID | (b1) 32-bit uid | (c) none | (d) epoch+number |
|---|---|---|---|---|---|
| 1 | Live actor, same incarnation, same process | resolves | resolves | resolves | resolves |
| 2 | Actor stopped, name free | stale | stale | not found | stale |
| 3 | Stopped, name reused in the same System | stale (exact) | stale, except with probability 1/M = 2.3e-10 per reference | **new actor** (violates the requirement) | stale (exact) |
| 4 | Name reused in another System of the same process (same system name) | stale (exact) | probable | new actor | stale (exact) |
| 5 | String from another process or earlier run, same path live | **may resolve** (when the number coincides; it depends on the whole prior sequence of ID draws, asks included) | probable | resolves the wrong actor | foreign (epoch differs), false match about 5.4e-20 |
| 6 | Corrupted token (`#`, `#0`, `#x`, `#0000...-1`) | malformed | malformed | n/a | malformed |
| 7 | Foreign system name | foreign, before lookup | same | same | same |
| 8 | Retired handle looked up by ID in the registry | `ErrUnknownHandle` | not applicable | not applicable | not applicable (path lookup) |

## Differential oracle feasibility

The mechanism discussion is in the section "Differential oracle (executed)" of the #1187 proposal. This probe did not use `judge/run-pekko.sh` (which takes a caller-supplied `<pekko-copy>` and only copies `Judge.scala` and `judge.sbt` into it; `Judge.scala` has no stale-uid scenario). The precedent `migration/stage-1/msg-02-failure-sender/pekko-repro/` states that "the incarnation identity criterion (stop and respawn under the same name) is NOT covered by this probe".

**Executed (oracle), Pekko v1.7.1 pin `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`, 2026-10-29.** Offline `sbt -batch -Dsbt.offline=true actor/compile` in a disposable `git archive` copy (about 22 s, exit 0, no download; `upstream/pekko` untouched), then a probe printing 196 rows. Of the 1893 `.class` files in `upstream/pekko/actor/target/scala-2.13/classes`, 1892 are byte-identical to a fresh compile and only `Version$.class` differs, so the pre-built classes are verified as the pin. Durable artifacts: draft PR #1292, `migration/stage-1/identity-oracle/pekko-repro/observed.txt` at commit `d4ac1c479f478254bddd1f8dc44ea5d35f35eea1` (revision 2: element lists print as `n=<size> ["a","b"]`; uid values are random per run, only their relationships are evidence). Scenario: start an `ActorSystem`, spawn child `a`, record its serialized form, stop it, spawn `a` again, then resolve strings through `resolveActorRef(String)`. Observed rows (`stale.*`):

| Input | Observed (Executed) |
|---|---|
| Old string with its `#uid` | `EmptyLocalActorRef`, terminated, equal to the old ref, old path; not the new actor (`stale.uids differ` = true) |
| String without uid | the new actor |
| `#0` | the new actor (wildcard, E9) |
| `#x` | `NumberFormatException` escapes `resolveActorRef` (also `ActorPath.fromString('pekko://sys/a#x')` and `...a#` throw) |
| Current uid | the new actor |
| Nonexistent path | `EmptyLocalActorRef` |
| Other system, remote address, relative string, `""` | `DeadLetterActorRef` |

So E9 and E10, and E7's "whether it escapes `resolveActorRef`" (it does) and the empty fragment (`NumberFormatException` for `""`), are now Executed rather than Hand trace or JDK-unverified.

The run turned the E9 and E10 readings into observations. It does **not** speak about the random collision probability (a statistical property
of `ThreadLocalRandom`, not of the code path) and has no Go counterpart for (d), whose epoch and numbers are a Go design: for (d) the evidence stays
Go-only (ADR 0033 D13, "pending approval") whatever the harness shows.

## ADR impact list (ADR 0033 D5)

Status read from the file at the revision above.

| ADR item | Status | Effect of this proposal |
|---|---|---|
| ADR 0031 P-6 (`:94`), incarnation never in equality | accepted 2026-10-24 | Kept by all alternatives |
| ADR 0031 P-7 (`:95` in the file at the revision above; the `:87` cited in the issue bodies is stale), no uid in the path text, separate function owned by #853/#894 | accepted | This proposal is that function's token form; no contradiction |
| ADR 0031 P-8 (`:96`), nothing escaped or decoded | accepted | Kept; the token is plain ASCII with no `%` |
| ADR 0031 O1 (`:135`), local only | accepted | Kept; no host or port form |
| ADR 0024 (D1-D5 accepted 2026-10-20), identity is the ID (`:206`) | accepted | (a) and (d) keep the ID as identity; (d) adds only a process epoch. (b) adds a second per-cell identifier. #1186's body cites `ADR 0024:203`, the table header; the sentence is at `:206` |
| ADR 0024 I2 (`:158`), an ID is never reissued | accepted | True per process only; (d) is built on exactly that scope and adds the epoch for S-3 |
| ADR 0024 alternative D (`:128`), per-System sequence | offered, not adopted | (a) and (d) tie the numeric part to the process counter; adopting D later would require changing the token scope (for (d), the epoch scope) |
| ADR 0014 D2 | accepted | Not affected by the token form |
| ADR 0011 and ADR 0033 D3 | accepted | Any new export (token accessor, parse function, parse errors, resolve function, stale sentinel) needs its own per-export justification, prepared in the scope of the story that exports it (#894 for encoder, parser and parse errors; #1092 for resolver and stale outcome), with #1020 as the index. **Open** |
| ADR 0033 D8 (`:42`), D13 (`:47`) | accepted | Local only. Go-only tests for the stale outcome are "pending approval" until the Owner approves them |
| ADR 0032 D5 | split into D5a and D5b, both decided "follow Pekko" by the Owner on 2026-10-28 (`Status` line 3 and "D5a and D5b" of `docs/adr/0032-local-sender-envelope.md` at the revision above; read, not changed) | Not involved |

Contradictions: none between accepted items and these alternatives. One tension: "in Go identity is the ID, not path plus uid" (`:206`) versus (b). With (d)
the ID stays the identity and the token is only its textual name plus a process epoch; the Owner decides whether that wording is acceptable (question 7).

## Agreement with #1187

Both proposals must say the same about `#`:

- The **path parser** of #1187 rejects any `#` with `ErrInvalidPath` (ADR 0031 P-7). It never returns a token.
- The **exact-reference parser** is a separate function, **proposed** to be owned by #894 (consumed by #1092) with option A of Owner question 10 (superseded for #894: encode only; see the Owner design decisions section); who owns the parse is not settled here. It does no lookup and does not know this System's address (foreign is a resolution outcome). It splits at the single `#`, parses the prefix with the #1187
  grammar and the token with the syntax above.
- If the Owner instead chooses in #1187 (G7) to accept `#` in the path parser and hand the token on, the token syntax above still applies. If the Owner picks
  (c) here, `#` stays an error everywhere and no exact-reference parser exists.
- Pekko's relative spec keeps `baz#1234` as an element (`RelativeActorPathSpec.scala:39`); `#` in relative text is decided where #1187 places relative text.
- The same `%XX` and verbatim rule (P-8) applies to the path prefix.

## Consumers affected

| Consumer | (a) | (b) | (c) | (d) |
|---|---|---|---|---|
| #894 encode and parse (proposed owner of the parse); #1092 resolves | Encodes the ID; process-lifetime limit documented | Needs the uid from #853 | AC "old reference does not resolve to the new actor" cannot pass | Encodes epoch and number; S-3 limit documented |
| #1092 | AC-R2-1 passes in-process; AC-R3-1 passes | Same, plus a test that fixes the uid | AC-R2-1 impossible | AC-R2-1 exact in-process; foreign-epoch case is an added test |
| #853 incarnation identity | No new state | Owns the uid field and generator | Unchanged | No new field; the incarnation value is the ID's number |
| #893 | Path-only selection stays separate (N1) | Same | Same | Same |
| #812 / #1089 | `#` rejected in the path parser; relative `#` per #1187 | Same | Same | Same |

## Recommendation and Owner decision

Token (d) is accepted for encoding (G2 message). **Architect orientation (not a decision), for the resolution part only:** resolution by path lookup plus token comparison, stale outcome S1 or S2 per the Owner, invariant N1 in
both cases. Reasons: the string can be copied out of the process, and the failure of (a) there is a **silent delivery to the wrong actor**; (b) is only
probable even inside one System and cannot detect a collision with a retired incarnation; (d) is exact wherever the counter reaches and probable only across
processes, with no new per-cell state. If the Owner says strings never leave the process, (a) is sufficient, with its limit written into the contract. If the
Owner wants to keep ADR 0024 alternative D open, (b2) is the alternative to (d).

- Owner decision: the addendum is **accepted** in the terms presented (above); **pending** for the resolution questions; the encode-side design decisions of 2026-10-29 are recorded in the section near the top.
- Decided by this document: nothing. The Owner accepted the addendum and the token form for encoding on 2026-10-29 in the terms presented (message quoted above); the syntax details, error categories and stale outcome beyond that are proposals.

### Owner questions

1. May an exact reference string be used outside the process that produced it (yes or no)? If no, (a) is enough. *(matches the earlier fragment only, point (l))*
2. Token: pick (a), (b1), (b2), (c) or (d). *(answered for encoding: (d), accepted by the Owner on 2026-10-29, G2 message)*
3. Stale outcome: S1 (typed error) or S2 (dead-letter-style reference)? *(deferred to #1092, not decided)*
4. Is the proposed invariant N1 (an obsolete exact reference is never redirected to the actor that reused the name) accepted? *(deferred to #1092, not decided)*
5. Text without a token given to the exact resolver: error, or an explicit path selection? *(deferred to #1092, not decided; J4)*
6. Does a root or guardian reference have an exact string? *(answered by J3: root and guardians not encodable, express in the G2 message)*
7. Is the ADR 0024 wording "identity is the ID, not path plus uid" kept with (b) or (d), the token being only the textual name of the incarnation? *(answered by J7: the public identity stays the ID)*
8. Do you agree with #1187 on `#`: error in the path parser, exact-reference parser separate? *(stays with #1187)*
9. Is a foreign epoch reported as foreign (proposed) or merged into stale? *(deferred to #1092, not decided)*
10. Who owns the exact-reference parser (split at `#`, token parse) and its typed parse errors? *(out of #894, which is encode only; not settled here)* **A (recommended):** #894 owns encode and parse of exact-reference strings (calling #812's absolute path parser) and **three** typed parse errors (empty, malformed, corrupted token; the single "Malformed" category above); #1092 consumes that contract and owns only resolution (foreign address or epoch, live exact string to the same actor, stale exact string to the stale outcome; path-only selection stays #893), in the order of evaluation above (steps 1-4 parse, steps 5-8 resolution). **B:** #1092 owns the parse and the malformed-text error type, and #894 stays encode-only. Owner decision `pending`; the scopes of #894 and #1092 are written as the proposal for A, and each has its own per-export justification with #1020 as the index.
