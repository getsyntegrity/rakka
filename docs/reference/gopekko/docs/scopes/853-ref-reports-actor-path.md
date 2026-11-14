# Scope: A Ref reports the ActorPath it was spawned at (#853, IDN-05)
Revision 4 (2026-10-29), against `f771a43`: (1) document base moved from `develop` `7840e88` to `db614b877a1c5ab7f00bf6364caa2c29ca1ac044` (#852
integrated; every code and Pekko line re-read); (2) Owner decisions D-2, D-4, D-5 (verbatim below); (3) D-2 made precise (`PathOf` for nil,
foreign, terminated and guardian handles); (4) stale dependency text fixed (#1020, #1021 not gates; D-4 "inconsistency" gone; kit run required).
Delta 2, against `c6cfd80` (head `943d77e`): citation fixes after QA; no contract change. Delta 3, against `943d77e`: revision note rewritten;
#894 is updated with a normal merge of develop (no rebase); storage paragraph split; no contract change. Earlier revisions 2 and 3 (QA findings:
evidence classes, oracle environment, order reasons) are folded in, their disclosures kept.

Delta 4 (2026-10-29), against `82c97fc`: record of the Owner G2 approval and conditions; conditions turned into stops; no change to the approved contract text except the three points that the Owner conditioned (G-2 Address wording, defensive failure treatment,
idHandle precondition). The only other changes follow from the Owner's conditions: G-3 (deferred handles documented as (zero, false)), D-6 (extra
Classic scenario and its allowed paths), G-6 (condition), P11/P12, and the inventory/stop rules.

Status: **G2 approved by the Owner on 2026-10-29 at head `82c97fc7690d85c1b9cb01c85f0372d57f663595`, with the conditions recorded below; the
commits after that head are this documentation delta (Delta 4, the record of the approval); no implementation yet.** Written against `develop`
`db614b877a1c5ab7f00bf6364caa2c29ca1ac044` and Apache Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. #853 stays `status:blocked`; this
scope edits no issue and no label. The branch is not rebased: references are read with `git show origin/develop:<path>` and cited "at develop db614b87".

**What G2 decided:** D-1, D-6, D-7 and G-1 to G-6 are approved with the conditions in the table below; D-2, D-4 and D-5 were decided earlier.
Three conditions are STOPs for the Dev ("Risk" (a) to (c)). Section "Scope for G2" is the approved scope.

Three words are kept apart here and in the sibling scopes #894 and #1092:

- **Logical path**: `ActorPath`, an address plus names. No incarnation; not unique over time (ADR 0031 P-6, P-7).
- **Exact reference**: a `Ref` and its identity, today the `ID` of the cell (ADR 0024, adaptation of Pekko's path plus uid).
- **Resolution**: finding a live actor from text or a path. Not in this story.

## Owner decisions for #853
Recorded verbatim (Spanish, Owner message of 2026-10-29):

> - D-2: función libre sobre Handle que devuelve (ActorPath, bool), sin agregar métodos a Ref[T]. El scope debe precisar nombre, firma y comportamiento para handles nulos o ajenos.
> - D-5: kit requerido.
> - D-4: justificación del export en el scope de #853 según EXP-1; #1020 actúa como índice y no como gate.

> Estas decisiones no aprueban implícitamente D-1, D-6 ni D-7. Presentá su estado y el scope final para G2. Sin implementación, merge ni cierre de issues.

| Item | Status | Condition or consequence |
|---|---|---|
| D-2 shape: free function over `Handle` returning `(ActorPath, bool)`, no method on `Ref[T]` | **Decided** (Owner, 2026-10-29) | Name, signature and behavior approved at G2 as G-1 to G-3 |
| D-5: kit run required | **Decided** (Owner, 2026-10-29) | Kit package, rows and gates under "Kit plan" |
| D-4: export justified in this scope per EXP-1; #1020 indexes, gates nothing | **Decided** (Owner, 2026-10-29) | "Export justification" below |
| D-1: rescoped outcome and criterion N1 | **Approved (G2, 2026-10-29)** | No further condition |
| D-6: P1-P4 as Go-only alternative evidence (ADR 0033 D13) | **Approved (G2, 2026-10-29)** | Go-only evidence for P1-P4 only. The extra Classic scenario also runs during the kit; if it contradicts the proposed contract the Dev stops the affected part and reports |
| D-7: order #853 then #894 | **Approved (G2, 2026-10-29)** | #894 is not implemented here |
| G-1: `PathOf(h Handle) (ActorPath, bool)` | **Approved (G2, 2026-10-29)** | None |
| G-2: native ref of another System returns its own path | **Approved (G2, 2026-10-29)** | Address equality must NOT be presented as proof of belonging to a concrete System instance unless that guarantee is verified (doc comment, README, scenario); the kit verifies what `NewSystem` enforces |
| G-3: guardians report a path; ID-only handles report false | **Approved (G2, 2026-10-29)** | Guardians included. `Terminated.Who()` and the stale failed child are DEFERRED and documented as `(zero, false)` in the doc comment, README and P10 |
| G-4: kit package `migration/stage-1/actor-853-ref-path/` | **Approved (G2, 2026-10-29)** | None |
| G-5: PR split K / C / R | **Approved (G2, 2026-10-29)** | May subdivide to meet the authorial budget if the contract does not change |
| G-6: macOS tests without network isolation; pre-existing `check-headers.sh` failure | **Approved (G2, 2026-10-29)** | Condition (Owner): compare against the CURRENT base at the final SHA and STOP on any new error. Measured at develop db614b87 (#852 delivery), not part of the condition: `check-headers.sh` gives 7 errors, output identical to the base |
| Immutable path initialised before the cell is published | **Approved, conditioned** | Subject to the inventory of Risk (a) |
| `idHandle` gets the path | **Conditioned** | Mandatory precondition, Risk (b): identity and equality unchanged, or STOP and consult the Owner |
| Zero path on a defensive failure | **Not approved** | Explicit treatment and proof of invariance, Risk (c) |
| Criteria rows N1-N4, P1-P12, N7 | **Approved** with D-1 and D-6 | P11 and P12 pending the Dev's proof |

## Owner G2 approval and conditions
Verbatim (Spanish, Owner message of 2026-10-29), granting G2 on the scope at head `82c97fc7690d85c1b9cb01c85f0372d57f663595`:

> Apruebo G2 de #853 sobre la propuesta presentada para #1287 en 82c97fc7690d85c1b9cb01c85f0372d57f663595, con estas condiciones:
> - D-1 aprobado.
> - D-6: apruebo evidencia solo de Go para P1-P4. Ejecutá también el escenario Classic adicional durante el kit. Si contradice el contrato propuesto, detené la parte afectada y reportalo.
> - D-7 aprobado: #853 antes de #894.
> - G-1: aprobado PathOf(h Handle) (ActorPath, bool).
> - G-2: aprobado devolver el path propio del handle de otro System. No presentes igualdad de Address como prueba de pertenencia a una instancia concreta de System sin verificar esa garantía.
> - G-3: aprobado, incluyendo guardianes. Acepto diferir el path de Terminated.Who() y del hijo fallido obsoleto; documentá explícitamente que esos handles devuelven cero, false.
> - G-4 y G-5 aprobados: kit y división K/C/R. Podés subdividir para cumplir el presupuesto autoral si el contrato no cambia.
> - G-6: acepto tests sin aislamiento de red en macOS y el fallo preexistente de headers, condicionado a comparar contra la base vigente y detenerte ante errores nuevos.
>
> Apruebo el path inmutable inicializado antes de publicar la celda, sujeto a verificar todas las vías de publicación y lectura. Antes de modificar idHandle, revisá sus comparaciones y usos como clave. Agregar el path no debe alterar identidad ni igualdad. No apruebo ocultar un fallo de construcción de path devolviendo cero para un actor válido. Demostrá la invariancia de los nombres aceptados y definí el tratamiento explícito del fallo defensivo; si exige cambiar el contrato, consultalo antes de implementarlo. Registrá esta aprobación y sus condiciones en #1287, obtené QA del delta y continuá con kit, implementación y QA independiente por SHA exacto. Sin otra autorización para los pasos aprobados. Sin merge, cierre de issues, CI remoto ni -race. No implementes #894 ni el índice de #1020.

## Post-G2 Owner decision (2026-10-29): Allowed files amended
Made by the Owner **after** G2, in the working session of 2026-10-29. It was **not** authorized earlier and is **not** part of the G2 approval above. Verbatim (Spanish): "Autorizo ampliar Allowed files para incluir las dos líneas de cell_release_internal_test.go, exclusivamente para reforzar el guard relacionado con esta unidad. Registrá la ampliación como decisión posterior a G2, sin afirmar autorización previa." Effect: `pekko-actor/actor/cell_release_internal_test.go` joins "Allowed files", restricted to those two lines. The contract text is unchanged. Before this decision the Dev had made that edit in PR C and reported it as a deviation.

## Scope for G2
### What it delivers
The ACT2 reconciliation of 2026-10-24 (`RESCOPE`) rewrote the outcome: **a `Ref` reports the `ActorPath` it was spawned at; identity across
name reuse is the existing `ID`.** AC1 to AC3 of the original issue describe behavior Go already has through the `ID`; they stay as
regression pins. New: `PathOf` (below) and criterion N1: two incarnations at the same path report equal paths and distinct `ID`s.

Where a cell would learn its path (read at develop db614b87, `pekko-actor/actor/`):

| Fact | Evidence |
|---|---|
| The cell stores `name`, `parent` (a `Handle`, an `ID` only) and the `owner` name table; no path, no `Address` | `cell.go:23-29` |
| The System address is fixed at creation | `system.go:108`, `system.go:139` (`System.Address`) |
| `commitCell` builds the cell from `spawnTarget` (`sys`, `owner`, `parent`) and the name | `spawn.go:129-163` (cell literal `:140-150`); `hierarchy.go:15-19` |
| A parent is known as a `Handle`: `Context` passes `x.c.identity()` (an `idHandle`), `System` passes the `/user` guardian handle | `hierarchy.go:23-29`; `cell.go:96` |
| `ID` is minted in `commit`, once per incarnation | `hierarchy.go:149` (`r.assignID(nextID())`) |
| The three guardians are cells; their handles are `idHandle` values, with no public accessor | `guardian.go:111-119`, `:174-178` |
| `ActorPath` has `NewRootPath`, `Child`, `Parent` (`:65`), `Descendant` (`:81`), `Address`, `Name`, `Elements`, `IsRoot`, `String`; `Root` is not present | `actor_path.go` (#1008; #852 integrated) |
| `releaseTerminated` drops the instance, factory, supervision and restart state; its comment lists `ctx, self, node, sys, owner, done, origin` as staying | `stop.go:132-146` |
| The README says no `Address` or `ActorPath` is attached to a `Ref`, and lists `Parent`/`Descendant` as implemented | `pekko-actor/README.md:244-255` |

**What is missing:** a path stored per cell, derived once at `commitCell` from the parent's path and `name`.

### PathOf contract (D-2 precision; every row below is a scenario; approved at G2 with the conditions recorded)

```go
// PathOf reports the logical path of the actor behind h. ok is false when h is nil, is not a handle this package issued,
// or carries no path; the ActorPath is then the zero value. Handles built from an ID alone (Terminated.Who(), the stale failed child of a
// supervisor) carry no path and report (zero, false): deferred, not an error. The path never carries an incarnation (ADR 0031 P-7). Its Address
// names a System by name; equal Addresses do not prove that two handles belong to the same System instance.
func PathOf(h Handle) (ActorPath, bool)
```

**Name (G-1).** `Handle`, `ID` and `Ref` are nouns; the existing exported functions over handles are verbs plus their object (`Spawn`,
`SpawnAnonymous`, `TellFrom`). `PathOf` reads "the path of this handle" and does not suggest a method of `Ref`. Bare `Path` would sit beside
the type `ActorPath` and read as a Pekko-style `ref.path` property. No `PathOf` exists in `pekko-actor` at develop db614b87 (searched).

**How it works.** An unexported interface (the pattern of `ownedHandle`, `watch.go:123`) is implemented by `*ref[T]` and `idHandle`; `PathOf`
asserts it. No method is added to `Ref[T]` or `Handle` (D-2). `ok` is true exactly when the stored path is not the zero `ActorPath`.

| # | Input | Result | Reason |
|---|---|---|---|
| P1 | nil `Handle` interface | zero, false | nothing to read |
| P2 | typed-nil `*ref[T]` inside a `Handle` | zero, false | `ref.go:29-34` already treats a nil receiver as "not resolvable" |
| P3 | `&ref[T]{}` with a nil cell | zero, false | same |
| P4 | a user type implementing `Ref`/`Handle`, or a hand-built `idHandle` | zero, false | not issued by this package; no Pekko analogue |
| P5 | native ref of the same System | its path, true | the case of the story |
| P6 | native ref created by another System of the process (**G-2**) | its own path, with that System's Address, true | the path is a property of the reference; its Address names a System by name, not a concrete instance (G-2 condition) |
| P7 | ref of a terminated actor, also after the registry retired it | the same path, true | the path is immutable metadata on the cell; `ref` keeps its cell (`ref.go:23`); `releaseTerminated` (`stop.go:138-146`) does not touch it and must not (its comment is updated to list the path) |
| P8 | guardian handles: `/user` (from `ctx.Parent()` of a top-level actor), `/system` and root (from the test-only `sys.guards`, not public: P8 is an internal-test scenario, only the `/user` part is reachable through the public API) (**G-3**) | `pekko://sys/user`, `pekko://sys/system`, `pekko://sys/`, true | Pekko gives them `rootPath`, `rootPath / "user"`, `rootPath / "system"` |
| P9 | anonymous child | path whose last element is the generated `$..` name, true | `Child` accepts a leading `$` (`names.go:30-32`) |
| P10 | a cell nobody committed (hand-built) and an `idHandle` built from an ID alone (`Terminated.Who()` at `watch.go:542`, a stale failed child at `supervision_failure.go:252`) (**G-3**, deferred) | zero, false | no cell is at hand; deferred by G2 (G-3) and stated in the doc comment, the README and this scenario; not an error |

**G-2 (approved).** P6 returns `(its own path, true)`. Condition: `p.Address()` is a name-based address, not proof that the handle belongs to a
concrete `System` instance. `Config.Name` is used as given and is not validated (`system.go:70-74`) and a search of `system.go` at develop db614b87 found no registry
of System names (absence is not shown by a line range; the kit must still verify), so two Systems may share an Address. The doc comment, the README and the scenario must not present Address equality as proof of
ownership unless the kit verifies a guarantee (whether `NewSystem` enforces anything per process). A `PathOf` with no `System` parameter cannot
tell "same" from "other"; the alternative false would need a `System` argument, against D-2, and is not chosen.

**Where the path is stored.** A `path ActorPath` field of `cell[T]`, immutable, set in the cell literal of `commitCell` (`spawn.go:140-150`),
which runs before `commit` publishes the cell (`hierarchy.go:142-162`). The cell becomes visible to other goroutines only through `commit`
under `System.mu`, so every reader sees the already-built cell literal after that publication. No lock is needed to read the path, and it is
never recomputed from the parent chain. The parent's path is read from `spawnTarget.parent` (an `idHandle` that now carries it); a nil parent (the root guardian only) uses
`NewRootPath(t.sys.address)`; every other cell uses `parentPath.Child(name)`. The Dev proves that the `Child` check cannot fail for a name `Spawn` accepts
(`validChildName` is `validPathElement` plus "no leading `$`", `names.go:26-33`; generated names use the alphabet of `names.go:65`);
a failure to build the path is never hidden by the zero path (Risk (c)). `cell.identity()`
(`cell.go:96`) copies the path into `idHandle`. Typed reaches the path from a plain `Ref` through the free function. No uid, no text encoding.
This REPLACES the earlier proposal "set at commit under the System lock" (the Owner's earlier brief). The Owner approved it at G2,
subject to the inventory of Risk (a).

### What it does not deliver

- Parsing path text (#812), `Root` and relative paths (#1089), resolving or encoding references (#893, #894, #1092), a uid or incarnation
  counter, any change of `ID` format (`id.go:9-18`), a path on `Handle` or `Ref` (D-2), any remote or cluster address (ADR 0031 O1).
- The path of a temporary ask endpoint (`ask_endpoint.go:57`, a `Handle` of this package with no path): `PathOf` reports false.

### Dependencies, as they really are

| Dependency | State verified today (gh, 2026-10-29) | Effect on #853 |
|---|---|---|
| #1008 (`ActorPath` value) | closed, PR #1011 `13fb995` | Consumed: `NewRootPath` and `Child` |
| #852 (`Parent`, `Descendant`) | issue open (not closed by this work); code in develop (squash `8f4977bc27d45e00d0e36ac81557180056967bc3`), scope squash `8873b10a6b3d45c54623ec977b6678095f5e74ee`, oracle evidence squash `d7af9bcc20ea1dbc5800c2d384589a017c48dde0` | Not needed by the implementation (`PathOf` uses `Child` only). Benefits tests: a scenario can assert `PathOf(child).Parent() == PathOf(parent)` |
| #812 (parser) | open, blocked | **Independent**: criteria and scenarios here parse nothing (issue #853 ACT2 table; this scope's criteria) |
| #1020 (GOV-02) | open, ready | **Not a gate.** Owner D-4 and accepted decision EXP-1 (`docs/parity/typed-owner-decisions-2026-10-27.md`): "Each `pekko-actor` export needed by another module is justified in the story that introduces it, with the symbol, the consumer, why the public API is not enough, what it does not expose, and its test. #1020 indexes those records and blocks no story; a story that adds no export needs no record." The ACT2 row "#1020 depends_on #853" in the issue body is stale (the issue is not edited) |
| #1021 (GOV-03, form) | open, ready | Not a gate: EXP-1 fixed the form; #1021 is still open |
| #1186, #1187 (ADR addenda) | open, ready | Not needed: #853 reports a path and an `ID`, no text form |
| #894, #893, #957 | open, blocked | Consumers; #894 after #853 (D-7: functional dependency plus the shared `ref.go` edit surface; the second PR is updated with a normal merge of develop (no rebase of a published branch) and must not rewrite the first PR's lines) |

### Contracts consumed

| Contract | Source | State |
|---|---|---|
| `ActorPath` value; equality ignores incarnation (P-6); no uid, `#` is an error in `Child` (P-7); `String()` verbatim (P-8) | ADR 0031 `:94-96` | accepted 2026-10-24 |
| A reused name is a different identity; retired IDs give `ErrUnknownHandle` | ADR 0024 table row, `:206` (the issue cites `:203`: stale, same content) | accepted |
| Restart keeps `Ref`, ID, name and `Done` | ADR 0017 `:348`; `supervision_restart_internal_test.go:294` | accepted, tested |
| Per-export justification | ADR 0011, ADR 0033 D3, EXP-1 | accepted |

### Criteria -> scenario -> Pekko source -> evidence class

Unit level, `pekko-actor/actor/`, go-specs matchers, `mock.Controller`/`Spy` for doubles, no sleeps; two incarnations are ordered by the
executor harness (`e.ex.RunAll()`) as in `registry_characterization_internal_test.go`. Classes: SPEC (a Pekko spec asserts it), Executed (PR
#1292, `observed.txt` row n at develop db614b87, probe revision 2, a Pekko v1.7.1 disposable copy), READ (source reading), Go design.

| # | Criterion | Scenario | Pekko source | Evidence |
|---|---|---|---|---|
| R1 | AC1 pin: a replacement at the same path gets a distinct identity | `TestRetentionCharacterizationNameReuse` (`registry_characterization_internal_test.go:442-460`, `fresh.ID() != old.ID()` at `:449`) unchanged | `ActorRef.scala:183-186` (equals = uid and path), `ActorCell.scala:392-397` (`newUid`); SPEC `ActorSelectionSpec.scala:124-148` (`a2 should not be a1`, `:142`) | SPEC + Executed (rows 183-185: two incarnations at `pekko://sys/user/a`, "uids differ" true); Go test exists |
| R2 | AC2 pin: restart keeps reference identity | `supervision_restart_internal_test.go:294` (W2) unchanged. Limit: W2 reads the same variable twice, so "same ID" is trivial there; the code PR adds an explicit `ID` equality assertion before and after the restart | `newUid` only at `dungeon/Children.scala:83` and `:320`; none in `dungeon/FaultHandling.scala` (searched) | READ (absence of a call) |
| R3 | AC3 pin: a stale exact reference cannot deliver to the replacement | same test: `old.Tell` dead-letters (`:450`, `:453`), `fresh` receives (`:454-455`) | `Children.scala:230-241` (`getSingleChild`: a uid that differs from `crs.uid` does not reach the child); SPEC `ActorSelectionSpec.scala:124-148` | SPEC for the path-based half only; READ for the uid guard; Executed rows 186-188 show resolution only |
| N1 | **New**: two incarnations at the same path report equal paths and distinct IDs; the old ref keeps reporting the same path after it terminated | stop `n`, spawn `n` again; `PathOf` of both equal, `ID`s differ | SPEC `ActorSelectionSpec.scala:124-148` (`a2.path should ===(a1.path)`, `:140`); `ActorPath.scala:163-168` (doc: equality ignores uid) and `:454-466` (`equals`) | SPEC + Executed (rows 19, 21: path equality ignores uid; rows 183-185); the Go form is `ID`, not uid |
| N2 | Top-level and nested actors report `pekko://sys/user/a` and `pekko://sys/user/a/b` | spawn under System and under `Context`; compare `String()` and `Elements()` | `ActorPath.scala:369` (`toString`) | path text Executed on a constructed path (row 13); rows 183-184 show the serialization form with uid of real actors; the path an actor reports, top-level and nested, READ |
| N3 | Restart keeps the path | same path before and after a Restart, same `ID` | no `newUid` in restart (as R2) | READ (absence of a call) |
| N4 | Anonymous child reports its generated `$..` name (P9) | `SpawnAnonymous`; last element equals the generated name | `Children.scala:128` (`randomName()`; uses `:59`, `:63`); `spawn.go:89-100` | READ |
| P1-P4 | nil, typed-nil, nil cell, user double (table above) | one scenario each | none: nil receivers and `Handle` doubles have no Pekko analogue | Go-only evidence, approved at G2 (D-6, ADR 0033 D13) |
| P5, P7 | same-System ref; terminated and retired ref (path stays) | spawn, stop, retire, `PathOf` | `ActorRef.scala:124` (`def path`) | READ |
| P6 | foreign System ref (G-2) | two Systems with distinct names; `PathOf` carries each Address | Pekko has no cross-System handle in one JVM view | Go design; G-2 |
| P8 | guardians `/user`, `/system`, root (G-3) | `ctx.Parent()`; test-only `sys.guards` | `ActorRefProvider.scala:402` (`RootActorPath`), `:538-545`, `:580`, `:595` | READ; rows 1-2, 16 execute only the root path value |
| P10 | uncommitted cell; ID-only `idHandle` (G-3, deferred) | hand-built cell; `Terminated.Who()`; stale failed child: each (zero, false) | none | Go design; G-3 |
| P11 | invariance: every name `Spawn` accepts and every generated name yields a path; the System address is never zero; a spawn's parent path is non-zero | table over names, `generateName` values, the default System name | none | Go design; pending the Dev's proof (Risk (c)) |
| P12 | the explicit treatment of the defensive failure (never the zero path for a valid actor) | forced failure through the chosen seam | none | Go design; pending the Dev's RULEBOOK decision (Risk (c)) |
| N7 | README states: a `Ref` reports a logical path; identity is the `ID`; path equality is not identity | doc check | | review |

Oracle environment (unchanged since revision 3): Temurin 11.0.32, `~/.sbt/boot` holds sbt 1.12.14; the Executed rows come from PR #1292 (revision 2 of the probe, `migration/stage-1/identity-oracle/pekko-repro/{Probe.scala,RUN.md,observed.txt}`).
No new differential is run in this documentation PR; the extra Classic scenario is approved at G2 (D-6): the kit runs it, and a result that
contradicts the contract stops the affected part and is reported. Go is never run on a JVM; "Executed" is Pekko-side evidence only. None of this is PORT-GAP.

### Allowed files

Exactly these paths (future implementation, after G2). Anything else is a stop and a report to the TL.

| Path | Edit |
|---|---|
| `docs/scopes/853-ref-reports-actor-path.md` | this file |
| `migration/stage-1/actor-853-ref-path/{manifest.tsv,RULEBOOK.md,RUNBOOK.md,EXECUTION-RECORD.md}`, `migration/cost-log.tsv` (one row) | kit package (D-5) |
| `pekko-actor/actor/ref.go` | `PathOf` and `(*ref[T]).path`; keeps its single `Ported from` line |
| `pekko-actor/actor/cell.go`, `spawn.go`, `hierarchy.go` | `path` field; compute in `commitCell`; `idHandle.path`; `identity()` copies it |
| `pekko-actor/actor/stop.go` | comment of `releaseTerminated` only (the path stays) |
| `migration/stage-1/actor-853-ref-path/pekko-repro/{Probe.scala,RUN.md,observed.txt}` | evidence of the extra Classic scenario (Owner D-6 condition; not a contract change) |
| `pekko-actor/actor/guardian.go`, `system.go` | only if the nil-parent root case needs it; otherwise untouched |
| `pekko-actor/actor/ref_path_internal_test.go` | new: N1-N4, P1-P12 (go-specs) |
| `pekko-actor/actor/cell_release_internal_test.go` | **post-G2 amendment, two lines only:** add `path` to the keep set and to the live-kept set of the guard "a field added to cell must be placed in one of the three sets" (see "Post-G2 Owner decision") |
| `pekko-actor/README.md` | module README contract: implemented vs planned; N7; update `:254-255` (no path on a `Ref`) |

### Export justification (EXP-1 shape; Owner D-4)

One export, individually justified here; #1020 only indexes it.

| EXP-1 field | Content |
|---|---|
| Symbol | `actor.PathOf(h Handle) (ActorPath, bool)` (name approved, G-1). The shape is the Owner's decision D-2 |
| Consumer | Honest account: at develop db614b87 no code in `pekko-actor-typed`, `pekko-serialization`, `pekko-persistence` or `pekko-projection` uses an `ActorPath` (searched: no hit) and none calls an accessor. The only identified consumers are the blocked #957, #894 and #893, which need the path of a `Ref`. Typed declares `def path: classic.ActorPath` (`actor-typed/.../typed/ActorRef.scala:63`, classic `ActorRef.scala:124`). The Owner decided the export shape anyway (D-2) |
| Benefit | Diagnostics and logs name the actor by path; #894 gets the path part of an exact reference; Typed reaches it from a plain `Ref` without a method on `Ref[T]` |
| Why the public API is not enough | `Ref` has `ID()` and `Tell`; `Handle` carries an `ID` only (`id.go:12`, `ref.go:18-21`). A consumer in another module cannot read the unexported cell, and copying names by hand breaks on `$..` names and on name reuse |
| What it does not expose | No uid, no incarnation, no cell, no registry, no parent chain, no text encoding; the path is metadata, never identity |
| Its test | N1-N4 and P1-P12 in `ref_path_internal_test.go`; README sentence N7 |
| Compatibility | Additive exported function; no method added to `Ref[T]` or `Handle`; no implementer breaks |

**Proposed row for the #1020 index** (a proposal; this story edits no issue; the index approves nothing and gates nothing): capability
`PathOf (Ref path report)`; introducing story #853; consumers #894, #893, #957; justification location: this section until the owning PR records it.

### Kit plan (D-5: kit run required, ADR 0017/0024, AGENTS.md)

Classification: **port/adaptation, "kit run required (ADR 0017/0024, AGENTS.md)"**. The Dev runs the kit with its own package, same plan as
#852: `migration/stage-1/actor-853-ref-path/{manifest.tsv,RULEBOOK.md,RUNBOOK.md,EXECUTION-RECORD.md}` and one `migration/cost-log.tsv` row
(G-4). Manifest columns as in #852: `source target disposition pekko_row scenario evidence note`. Proposed rows:

| Pekko row | Disposition | Scenario | Evidence |
|---|---|---|---|
| `ActorRef.scala:175-190` (equals, hashCode, toString by uid and path) | adapted (Go identity is `ID`) | R1, N1 | SPEC + Executed rows 19, 21, 183-185 |
| `ActorRef.scala:124` (`def path`) | adapted (free function `PathOf`) | N2, P5 | READ + Executed rows 183-184 |
| `Children.scala:83`, `:320` (`newUid` call sites), `ActorCell.scala:392-397` | adapted (no uid; the `ID`) | R2, N3 | READ |
| `Children.scala:128` (`randomName()`) | ported earlier (`SpawnAnonymous`); reused | N4, P9 | READ |
| `ActorSelectionSpec.scala:124-148` | adapted (spec for N1 and R1; selection excluded) | N1, R1 | SPEC |
| `ActorRefProvider.scala:402`, `:538-545`, `:580`, `:595` (guardian paths) | adapted | P8 | READ |
| `ActorRefSpec.scala:341-` (serialized `ActorRef`, `EmptyLocalActorRef`) | moved to #894 | none | - |
| `ActorPath.scala:163-168`, `:454-466` (equality ignores uid) | ported earlier (#1008); reused | N1 | SPEC + Executed |

**PR split (G-5, approved; forecast, not measured):** K = kit package files (about 120 lines); C = code and tests (about 60 lines of code, 250
of tests: `ref.go`, `cell.go`, `spawn.go`, `hierarchy.go`, `stop.go` comment, `ref_path_internal_test.go`); R = README, `EXECUTION-RECORD.md`
and the cost-log row (about 100 lines). Each stays at or under 400 authored lines. As reported by the Owner in the working session of
2026-10-29 (no repository trace), splitting a scope needs no new G2 when the contract does not change; a split that changes the contract
needs a new G2. The Owner allows subdividing to meet the authorial budget when the contract does not change.

**Extra Classic scenario (Owner D-6 condition; not a contract change).** The kit also runs one added Classic scenario covering rows N2, N3, N4,
P5, P7 and P8 (top-level and nested actor paths, restart keeps the path, anonymous child name, terminated actor path, guardians) on the pinned
Pekko in a disposable copy, offline, as in PR #1292 (`migration/stage-1/identity-oracle/pekko-repro/RUN.md`; `JAVA_8_HOME` is needed for both
sbt commands, steps 3 and 4 of that `RUN.md` at develop). If a result contradicts the proposed contract, the Dev stops the affected part and reports it.

### Verification

Commands from `AGENTS.md` "Go" for `pekko-actor`, once per final tree: `unshare -rn scripts/each_module.sh go test -count=1 ./...` (container
variant where namespaces are restricted), `gofmt -l`, `golangci-lint` (gocyclo at most 10), `go vet` and `go mod tidy -diff` as listed in
`docs/quality-gates.md`, module coverage at least 85% (`scripts/check_coverage.sh`), diff coverage at least 85%, `scripts/check_deps.sh`,
`.github/scripts/check-headers.sh` (no new Pekko-ported file expected). `-race` only on explicit request. `make parity-guard` checks the
persistence manifest and does not apply. **G-6 (approved with a condition):** the #852 delivery recorded two exceptions: unit tests run without
network isolation on macOS (`unshare` unavailable) and `check-headers.sh` exits 1 on seven pre-existing files. The Owner accepts both for #853
only if the Dev compares against the CURRENT base at the final SHA and STOPS on any new error (the 7 errors and the identical output are what was
measured at develop db614b87, not the condition). A scenario that fails and is not a Go defect is run on Pekko before it is classified.

### Risk

Risk: none for this documentation PR. For the code PR: an additive exported function is not item 3 by itself (D-2 adds no method to `Ref[T]`
or `Handle`, so no implementer breaks). The real risks are three STOP conditions set by the Owner at G2 (2026-10-29), plus the usual ones:

- **(a) Publication and read paths.** The immutable path set before the cell is published is approved SUBJECT TO an inventory, produced by the
  Dev in the RULEBOOK and the EXECUTION-RECORD, of every place that builds or copies a cell, `ref` or `idHandle` and of every reader: the
  `commitCell` literal (`spawn.go:140-150`); `commit` under `System.mu` (`hierarchy.go:142-162`); `cell.identity()` (`cell.go:96`);
  `watch.go:542`; `supervision_failure.go:252`; guardian creation (`guardian.go:122-132`, `:174-178`); restarts; tests that hand-build cells;
  readers: `PathOf` and the README. A path found missing, or written after publication, stops the work.
- **(b) `idHandle` precondition.** BEFORE modifying `idHandle` the Dev reviews every comparison and every use of `idHandle` or `Handle` as a
  `==`/`!=` operand, map key or set member, and proves that adding the path changes neither identity nor equality. This is mandatory, no longer
  "to be confirmed". If a comparable field changes equality between an `identity()`-built `idHandle` and an ID-only one, STOP and consult the Owner;
  the Dev proposes, and does not apply, an alternative that keeps identity unchanged.
- **(c) Defensive failure.** A failure to build the path must never be hidden by returning the zero path for a valid actor (the earlier "the
  defensive branch leaves the zero path" is no longer approved). The Dev proves the invariance: every name accepted by `Spawn` (`validChildName`)
  or generated by `generateName` is a valid path element; the System address is never the zero Address (an empty `Config.Name` becomes
  "default", `system.go:150-154`); the parent path of every spawn is non-zero. The Dev defines an EXPLICIT treatment in the RULEBOOK (candidates:
  an internal-invariant panic before publication, as `newGuardian` does for an unreachable error at `guardian.go:124-126`, or an explicit error),
  with evidence of which fits the existing `commitCell`/`Spawn` error handling. If the treatment would change the contract (exported behavior,
  a new error from `Spawn` for a valid name, the documented error set), STOP and consult the Owner. Scenarios P11 and P12 pin this, pending the proof.
- Usual risks: a new field in the shared `cell`/`commitCell` used by every spawn and restart path (covered by N3, P10 and the existing suite);
  termination must not clear the path in `releaseTerminated` (P7, N1); README claims, a path is metadata, not identity (N7).

### Open items and limits

Evidence comes from one macOS machine, without network isolation, without `-race`, without remote CI. Go was never run on a JVM and no
parity is claimed: Executed rows are Pekko-side only. Not verified: the Classic behavior of rows marked READ (the extra scenario is planned in the
kit); whether `NewSystem` guarantees anything per process about names (G-2); the exact line count of the code PR (a forecast). #853, #852, #1020
and #1021 stay open; nothing here closes or edits an issue. Not authorized: merge, closing issues, remote CI, `-race`, #894 and the #1020 index.
