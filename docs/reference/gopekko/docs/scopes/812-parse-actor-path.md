# Scope: Parse local actor paths (#812, IDN-03)

Revisions 1 to 4 (2026-10-29): QA rounds on the earlier draft; their history is in git. **Revision 5 (2026-10-29): final scope after the Owner decisions of 2026-10-29; written against `origin/develop` `abc94d55e635704db3ec30c6735227a4c9676048` (earlier text was against `7840e88`).** Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. Documentation only; `Risk: none` for this PR.

Status: **Owner decisions of 2026-10-29 recorded below; gate G2 is pending the Owner's grant on the final SHAs; no implementation, kit, merge, issue edit or #1020 index is authorized.** #812 stays `status:blocked`; this scope edits no issue and no label. Pekko was executed by the probe of PR #1292 (integrated, `migration/stage-1/identity-oracle/pekko-repro/`, 196 rows); rows are cited by number and re-checked against `observed.txt` on develop `abc94d5`.

## Owner decision of 2026-10-29: the message "sí a todas"

The Owner's reply was the two words **"sí a todas"**, to a decision package the coordinator presented for #812. The package table below is the **coordinator's text, not the Owner's words**; the Owner accepted its recommendations by that reply. The coordinator's question about refreshing #1285 and #1290 was part of the package the two words answer; that refresh is the coordinator's statement, not words of the Owner.

| Q | Recommendation of the package (coordinator's text), accepted | Consequence in this scope |
|---|---|---|
| Q1 | strict grammar | the accepted set is exactly the grammar below |
| Q2 | error for empty segments, trailing slash and a missing slash after the system; `..` is a legal element (G11), no navigation | rows P3-P5 and the `..` row |
| Q3 | accept any syntactically valid protocol; a mismatch is a resolution outcome | row P18; the protocol is kept exactly as written |
| Q4 | the `ActorSystem` name charset `[a-zA-Z0-9][a-zA-Z0-9-_]*` is enforced **by the parser only**; `NewLocalAddress` is unchanged (G10 option (i)) | rows P22, P24; the round trip is restricted |
| Q5 | `#` is an error in the path parser, `ErrInvalidPath` (ADR 0031 P-7) | rows P12, P13 |
| Q7 | AC1 and AC2 of #812 are rewritten as the ACT2 reconciliation proposes; the **issue text is not edited** by this work (an Owner action) | "Traceability of the acceptance criteria" |
| Q8 | the parser stays internal: unexported, no export justification, no #1020 row | "API" and "Export position" |
| protocol shape | URI-style scheme `[A-Za-z][A-Za-z0-9+.-]*` | grammar |
| Q6 (consequence of Q2) | empty segments are errors: **an accepted difference from Pekko**, which accepts `pekko://sys`, `//a` and a trailing slash (oracle rows 39/44, 45/50, 51/56) | rows P3-P5 |

**Not decided, kept pending (never inferred):** Q10 (accept the #1292 oracle as the Pekko evidence for the rows it covers, extend the probe, or D13 alternative evidence for the uncovered rows); Q9 and Q11-Q18 (relative text: stays with #1089, a separate decision); the owner of the **exact-reference-string parse** (#894 is encode-only and closed, and it is not needed by #812's first slice; whether #1092 under its option B owns it is unsettled); the issue edits; G2; the start of implementation; the kit run; merge. **The Owner has not approved Go-only evidence for #812** (ADR 0033 D13): the rows that depend on it are marked below.

## Traceability of the acceptance criteria of #812 (issue read-only, `gh issue view 812`; the issue is not edited; its title "Parse local actor paths" stays)

| # | Criterion (verbatim from the issue) | Where it is satisfied |
|---|---|---|
| AC1 | "Valid root, child and escaped-segment paths round-trip through canonical formatting." | rewritten by the ACT2 reconciliation and accepted (Q7) as **AC1'**: "a path whose elements contain `%XX` round-trips verbatim; nothing is escaped or decoded" (ADR 0031 P-8), restricted below to addresses the parser admits; rows P1, P2, P6; tests `TestParse_RoundTripTable`, `TestParse_VerbatimPercent` |
| AC2 | "Malformed addresses, illegal segments and invalid encodings fail without actor lookup." | rewritten (Q7) as **AC2'**: "malformed `%XX`, empty elements, `/` or `#` elements and invalid addresses fail with a classified error"; rows P3-P5, P7-P9, P12-P17, P19-P23 and the extra rows X1-X9 |
| AC3 | "Parsing does not create actors or resolve an actor incarnation." | unchanged; "AC3: how it is tested" below |
| AC4 | "Include focused executable positive and negative cases for this result, including the named ordering or race boundary; record the tested candidate SHA and applicable local gates." | all scenarios; the SHA and gates in the execution record; the ordering boundary is the evaluation order of the error classes (`TestParse_ErrorClassPosition`) |
| AC5 | "Update the public contract and traceability for this slice; do not claim completion of sibling stories or full actor parity." | `pekko-actor/README.md` (module contract), the kit manifest; no parity claim |

The rewrite of AC1 and AC2 is the Owner's decision (Q7); applying it to the issue body is left as an **Owner action**.

## What this delivers

One **unexported** function `parseActorPath(text string) (ActorPath, error)` that turns absolute local text `protocol://system/elements` into the `ActorPath` of #1008 and returns a classified error otherwise. Nobody calls it today; the first consumers are future work (#1092 for the exact-string resolution, #893, #1089). It builds nothing else: the address with `NewLocalAddress` (`address.go:42`), the root with `NewRootPath` (`actor_path.go:34`) and each element with `Child` (`:49`, which calls `checkPathElement`, `:100`, and `validPathElement`, `names.go:33`), so a parsed value cannot differ from a built one; **no second validator, no copy of the character set, no change to `checkPathElement`**. Parsing takes only the text: it creates no actor, draws no ID and resolves no incarnation.

**Round trip (restricted, Q4).** `ActorPath` is comparable and `==` is its equality (`actor_path.go:24`). For every `ActorPath p` built by `NewRootPath`, `Child` or `Descendant` over a non-zero `Address` **whose protocol satisfies `[A-Za-z][A-Za-z0-9+.-]*` and whose system satisfies `[a-zA-Z0-9][a-zA-Z0-9-_]*`**, `parseActorPath(p.String()) == p`. **Consequence of Q4 (the shapes are enforced by the parser only, G10 option (i)):** `NewLocalAddress` rejects only an empty protocol or system (`address.go:42-50`), so an `Address` whose system contains `/` (or `#`, `?`, `@`, `:` or a space) or whose protocol contains `:` or `/` is constructible. The text its paths print is then either **rejected** by the strict parser (most shapes) or, for a system containing `/`, **accepted and parsed to a different path**: for the system `a/b`, `pekko://a/b/x` parses as system `a` and path `b/x`, a silent misparse, not merely an unpromised round trip (shown by a scratch parser in the QA of this scope; the parser behaves as specified). **The round trip is not promised for such `Address` values.** `NewLocalAddress` is unchanged and only the parser enforces the shapes. The zero `ActorPath` prints `""` and does not round-trip (P21).

## What this does not deliver

Relative text such as `grand/child` (#1089, a separate decision); `Parent`/`Root`/`Descendant` (#852, code integrated; its criterion 2b is `TestParse_RoundTripDescendant` here); `ActorSelection`, wildcards (#893); host and port (`sys@host:1234`, ADR 0031 O1, ADR 0033 D8); a resolver, a stale rule or any parse of an **exact-reference string** (`path#token`): #894 is encode-only (integrated: `ExactRefString` only builds, `ref_string.go:22`) and the exact-string parse owner is unsettled (#1092 option B, not #812); no incarnation token. The ADR 0031 addendum of #1291 (accepted by the Owner) covers the **encoding token only**; the grammar addendum of #1187 (PR #1290) records the answers above and is not applied to ADR text by this scope. No change to `Spawn`, `validChildName` or `ErrInvalidName`.

## Dependencies (verified `gh` and develop `abc94d5`)

| Dependency | State | Effect on #812 |
|---|---|---|
| ADR 0031 (#766) | accepted 2026-10-24; O1 and O2 (`docs/adr/0031-local-address-and-actor-path-values.md:135-136`): local only, parsing deferred, "no grammar is defined here" | P-5 (`:93`), P-7 (`:95`), P-8 (`:96`) consumed; the grammar is the #1187 proposal, decided in the terms above |
| #1008, #1007 | delivered (`ActorPath`, `Address`) | nothing to wait for |
| #852 (IDN-04) | **code integrated** (issue still OPEN) | `Parent`, `Descendant` exist; `TestParse_RoundTripDescendant` can be written now |
| #853 (IDN-05), #894 (IDN-09) | **integrated, issues CLOSED**: `PathOf` (`ref.go:116`), `ExactRefString` (`ref_string.go:22`) | consumers of paths, not of the parser; no code edge |
| #1187 (GOV-40), #1186 (GOV-41) | OPEN; #1187 PR #1290 and the #1186 proposal (#1291, merged) record the decisions | #812's first slice needs only the grammar answers above |
| #1089 (relative text), #893, #1092 | OPEN, blocked | siblings and future consumers; no edge from #812 to them |
| #1020, #1021 | OPEN | **no #1020 row** (the parser is not exported); #1021 is not a dependency |
| PRs #1285, #1286, #1289, #1290, #1293 | OPEN drafts (documentation) | unchanged by this scope except #1285 (this file) and #1290 |

## Grammar and error classes (Q1-Q5, protocol shape)

```
path-text = protocol "://" system "/" [ element *( "/" element ) ]
protocol  = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )     ; kept exactly as written, never folded (Q3)
system    = ( ALPHA / DIGIT ) *( ALPHA / DIGIT / "-" / "_" )   ; the ActorSystem name set, enforced by the parser only (Q4)
element   = ADR 0031 P-5 element (checkPathElement): non-empty, no "/", no "#", Pekko's character set, "%XX" verbatim
```

`pekko://sys/` is the root. Elements are kept byte for byte (nothing escaped or decoded); `.` and `..` are legal elements (G11), no navigation. Anything else is rejected: `pekko://sys` (no slash), `//a`, a trailing slash after an element, empty text, `?query`, `#`, a bad `%`, a space, non-ASCII.

**Error-class rule (decided here; no new sentinel, no third class).** Evaluation order: (1) split at the first `://`; (2) check the protocol; (3) take the system part as everything after `://` up to the first `/` or the end; (4) check the path part. **A failure in the protocol or system part (steps 1-3: everything before the first `/` after `://`) is wrapped `ErrInvalidAddress`; the path part (step 4) fails with `ErrInvalidPath`, and the missing-root-slash case (`pekko://sys`, no `/` after a valid system part) is a path failure, `ErrInvalidPath`.** In particular: no `://` (including `""` and `/user/a`), an empty or malformed protocol, an empty system, a system with a character outside its shape (`@` and `:` included: the host and port forms, ADR 0031 O1, are `ErrInvalidAddress` with a reason naming the unsupported form), and `#`, `?`, `%` or a space **inside the system part** are `ErrInvalidAddress`; a missing root slash after a valid system part, an empty element, a trailing slash, an invalid element (P-5), a `#` or `?` in the path and a `%` not followed by two hex digits are `ErrInvalidPath`. Every error is `fmt.Errorf("%w: ...", sentinel)`, so `errors.Is` works; a rejected input returns the zero `ActorPath`. The code matches no accepted ADR text (`ErrInvalidAddress` and `ErrInvalidPath` already exist, `errors.go:26-30`); their doc comments gain the parser as a second source (`errors.go` row below). *Reading to confirm:* Q5's `#` is `ErrInvalidPath` where the `#` is in the path; a `#` in the system part falls under the position rule (address).

## Exact API (proposal for G2)

```go
// parseActorPath turns absolute local actor path text, "protocol://system/element/...", into its ActorPath. The text has
// exactly the form protocol "://" system "/" [ element { "/" element } ]: the protocol is a scheme ([A-Za-z][A-Za-z0-9+.-]*)
// kept as written, the system has the ActorSystem name set ([a-zA-Z0-9][a-zA-Z0-9-_]*), and each element is accepted by
// ActorPath.Child (nothing is escaped or decoded; "%XX" is kept verbatim). A failure in the protocol or system part (before the first "/" after "://") is an
// error wrapping ErrInvalidAddress (including the host and port forms); a failure in the path part wraps ErrInvalidPath (a
// missing root slash, an empty element, a trailing slash, "#", "?", a bad "%XX"); the ActorPath is then the zero value. It does no
// lookup, creates no actor and resolves no incarnation, and it does not know any System. A protocol that differs from
// LocalProtocol is accepted: a mismatch is a resolution outcome, not a parse error.
func parseActorPath(text string) (ActorPath, error)
```

File `pekko-actor/actor/parse_actor_path.go` (new). `git grep -i 'parseActorPath'` finds nothing on develop `abc94d5`: no collision. **No new exported name, no new sentinel.** To keep every function at cyclomatic complexity 10 or less, the parser is split: `splitScheme`, `checkProtocol`, `splitSystem`, `checkSystem`, `parseElements` (each returns a classified error), and `parseActorPath` only sequences them. Header: a Go design with logic adapted from Pekko's `Address.scala:221-231` and `ActorPath.scala:72-75`; the file carries the `Ported from` line only if the kit run decides so (`.github/scripts/check-headers.sh` requires exactly one such line in a ported file and counts any mention: a comment must say "adapted from", not the marker text).

## Rows (final outcome, class, test; `pekko://mysys` is spelled as in the issue)

Class: **Executed** = Pekko run by the probe of #1292, cited by `observed.txt` row (re-verified); **READ** = source reading; **Go-only** = the Go outcome has no Pekko row and is **pending D13 approval** (not approved: Q10 undecided). Where the Pekko side is Executed and the Go outcome differs, the difference is recorded as accepted by Q2 and Q6.

| Row | Input | Go outcome | Class and Pekko rows | Test (`TestParse_...`) |
|---|---|---|---|---|
| P1 | `pekko://mysys/` | the root of `pekko://mysys` (`==` `NewRootPath`) | Executed: rows 33, 38 | `RoundTripTable` |
| P2 | `pekko://mysys/user/a` | that path | Executed: rows 27, 32 | `RoundTripTable` |
| P3 | `pekko://mysys` (no slash) | `ErrInvalidPath` (missing root slash) | Pekko accepts it as root (rows 39, 44): accepted difference; Go-only | `EmptySegmentsAndSlash` |
| P4 | `pekko://mysys/user/` | `ErrInvalidPath` (trailing slash) | Pekko skips it (rows 51, 56); Go-only | `EmptySegmentsAndSlash` |
| P5 | `pekko://mysys/user//a` | `ErrInvalidPath` (empty element) | Pekko skips it (rows 45, 50); Go-only | `EmptySegmentsAndSlash` |
| P6 | `.../user/%41` | accepted, `Elements()` = `["user","%41"]` | Executed: rows 57, 62 | `VerbatimPercent` |
| P7 | `.../user/%4` | `ErrInvalidPath` | Executed: rows 63, 65, 68 (Pekko throws `MalformedURLException`; Go returns the typed error) | `MalformedEscape` |
| P8 | `.../user/%ZZ` | `ErrInvalidPath` | not a probe row (by analogy to row 63): READ | `MalformedEscape` |
| P9 | `.../user/a b`, `.../user/é` | `ErrInvalidPath` (Go is stricter, as `Child`) | Go-only | `IllegalElementChars` |
| P10 | `.../system/$a` | accepted | READ (`names.go:33` allows a leading `$`) | `DollarAndStarElements` |
| P11 | `.../user/boom/*` | accepted | SPEC (`ActorPathSpec.scala:113-114`) | `DollarAndStarElements` |
| P12 | `.../user/a#123` | `ErrInvalidPath` (Q5) | Pekko makes it a uid (rows 69, 74); Go-only | `HashIsError` |
| P13 | `.../user/a#`, `.../user/a#abc` | `ErrInvalidPath` (Q5) | Pekko throws `NumberFormatException` (rows 123, 135, 141); Go-only | `HashIsError` |
| P14 | `` (empty) | `ErrInvalidAddress` (no `://`) | Executed: rows 93-98 | `MalformedAddressTable` |
| P15 | `://hallo` | `ErrInvalidAddress` (empty protocol) | Executed: rows 99-104 (`://x`) | `MalformedAddressTable` |
| P16 | `pekko:///user` | `ErrInvalidAddress` (empty system) | READ (`Address.scala:192`) | `MalformedAddressTable` |
| P17 | `/user/a`, `user/a` | `ErrInvalidAddress` (no `://`; not parsed as relative) | Executed: rows 111-116 (`/user/a`); READ (`user/a`) | `MalformedAddressTable` |
| P18 | `akka://sys/some/ref` | accepted, protocol `akka` kept (Q3) | READ (`Address.scala:192`) | `OtherProtocol` |
| P19 | `pekko://my_sys@host:1234/some/ref` | `ErrInvalidAddress` (host and port: ADR 0031 O1) | Executed (Pekko side): rows 105-110; Go refusal is the O1 exclusion | `HostAndPortExcluded` |
| P20 | `s://dd@:12`, `s://dd@h:hd`, `a://l:1/b` | `ErrInvalidAddress` (`@` or `:` in the system part) | SPEC (`ActorPathSpec.scala:53-55`) | `HostAndPortExcluded` |
| P21 | the zero `ActorPath` printed as `""` | `ErrInvalidAddress` (as P14); does not round-trip | Go-only | `ZeroPathDoesNotRoundTrip` |
| P22 | `pekko://my_sys/user/a` | **accepted** (`_` is in the system set) | Pekko's own `fromString` rejects it (rows 81-86): a recorded divergence; Go-only | `SystemCharset` |
| P23 | `pekko://mysys/user/a?q` | `ErrInvalidPath` (Q2 strict) | Pekko drops the query (rows 75-80); Go-only | `QueryIsError` |
| P24 | protocol or system outside the shapes (`a/b`, `a b`, `a:b`) | `ErrInvalidAddress` for the parser input; an `Address` built by `NewLocalAddress` with such a system can print text that parses to a different path (system `a/b`: `pekko://a/b/x` parses as system `a`, path `b/x`) or fails: no round trip is promised | n/a; Go-only | `SystemCharset`, `RoundTripTable` |
| X1 | `PEKKO://sys/a` | accepted, protocol `PEKKO` kept (never folded) | Go-only | `OtherProtocol` |
| X2 | `a:b://s/a`, `1pekko://s/a` | `ErrInvalidAddress` (protocol shape) | Go-only | `MalformedAddressTable` |
| X3 | `pekko://s#y/a`, `pekko://s?y/a`, `pekko://s%y/a`, `pekko://s y/a` | `ErrInvalidAddress` (inside the system part) | Executed for `s y` (rows 87-92); Go-only for the rest | `ErrorClassPosition` |
| X4 | `pekko://sys@h/a`, `pekko://sys:1/a`, `pekko://s y@h/a` | `ErrInvalidAddress` (unsupported form) | Go-only | `HostAndPortExcluded` |
| X5 | `pekko://sys/user/..` | accepted, element `..`, no navigation (G11) | READ (`ActorRef.scala:284-296`) | `DotDotElement` |
| X6 | `pekko://sys/a/../b`, `pekko://sys/./a` | accepted, elements kept verbatim | Go-only | `DotDotElement` |
| X7 | `pekko://sys/a b/` and other mixed path failures | `ErrInvalidPath` | Go-only | `ErrorClassPosition` |
| X8 | text with a control character or non-ASCII in the system part | `ErrInvalidAddress` | Go-only | `ErrorClassPosition` |
| X9 | a table of generated short strings over `{p,e,k,o,:,/,#,?,%,4,1,z,@,space,.,$,_}` | accept/reject and class agree with a second, independent statement of the grammar | Go-only | `ErrorClassPosition` |

Rows that depend on Go-only evidence (**pending D13 approval, not approved**): P3, P4, P5, P9, P12, P13, P21, P22, P23, P24 and X1-X9. Rows decided by an accepted ADR or existing code and re-verified: P1, P2, P6-P8, P10, P11, P14-P20 (and AC2b, AC3).

**AC1 for #852 (criterion 2b).** `TestParse_RoundTripDescendant`: for receivers `pekko://sys/` and `pekko://sys/user` and names without empty segments, `parseActorPath(p.Descendant(names...).String())` equals `p.Descendant(names...)`; empty-segment inputs are excluded (Descendant skips them, the parser rejects them: Q2). `Descendant` is integrated, so the test is written now.

### AC3: how it is tested at develop `abc94d5`

`parseActorPath` takes only the text, so it can reach no System, registry or `Ref`. `idSeq` is now an `idCounter` (`id.go:44`, `:21`) and `nextID` goes through `idMint` (`id.go:48`, `:51`); `nextAskID` draws from the same counter. **(1) `TestParse_NoLookupNoActor`** (internal test, package `actor`, no `t.Parallel`): swap `idMint` for a minter that fails the test if it is called, restore it in `t.Cleanup`; read `idSeq.n.Load()` before and after running the whole table of valid and invalid inputs with **no System constructed**: the delta is zero. With a System from the `newTree` harness, `len(sys.byID)`, `Children()` of an owner and `ctlExec.Len()` are unchanged. Residual risk: a goroutine leaked from another test could advance `idSeq` during the window and make the counter check flaky; the `idMint` guard catches only actor IDs and is not affected by it. **(2) `TestParse_SameResultBeforeAndAfterStop`**: with a started System and an actor `a`, the result for `pekko://<system>/user/a` is `==` before the actor stops and after its termination is observed complete through the existing sleep-free helper `closedDone` (`termination_internal_test.go:48`) or `isClosed(cell.done)` (`cell_internal_test.go:89`); a path whose actor never existed parses to an `==` value. Neither test touches a mailbox or a clock.

## Differential oracle position (ADR 0033 D13)

The probe of #1292 ran `ActorPath.fromString`, `AddressFromURIString`, `ActorPathExtractor` and `RelativeActorPath.unapply` on the pinned Pekko; rows are cited above. **Executed rows:** P1 (33, 38), P2 (27, 32), P3 (39, 44), P4 (51, 56), P5 (45, 50), P6 (57, 62), P7 (63, 65, 68), P12 and P13 (69, 74, 117-141), P14 (93-98), P15 (99-104), P17 (111-116), P19 (105-110), P22 (81-86), P23 (75-80), X3 `s y` (87-92). **Not covered:** `%ZZ` (P8), `$a` as a `fromString` input (P10), `*` (P11), `pekko:///user` (P16), `akka://...` (P18), P20's inputs, `user/a`, X1-X9 except as noted. **Go-only rows** (the typed Go errors and their classes, P9, P21, the strict rejections of P3-P5, P12, P13, P23, P24, the round-trip claim, every Go-versus-Pekko difference) are Go design. **Q10 is not decided**: whether the oracle is accepted for the rows it covers, extended by a further probe, or whether D13 alternative evidence counts for the uncovered rows; the Owner has not approved Go-only evidence for #812. This is Go unit evidence, not V3; no parity is claimed.

## Export position (ADR 0011, 0033 D3; Q8)

The parser stays **unexported** (`parseActorPath`): no public commitment before a verified consumer, the grammar can still change, and nothing outside the package calls it. **Nothing is exported, so no per-export justification is written and no #1020 row is created.** The story's user-facing benefit (an exported inverse of `String()`) is not delivered by this slice; callers keep building paths with `NewRootPath` and `Child`. A later export needs its own decision.

## Kit (required: ADR 0017/0024, `AGENTS.md`)

An adaptation of Pekko's `ActorPathExtractor`, `AddressFromURIString` and `fromString` to Go, so the Dev runs the kit with `migration/stage-1/idn-812-parse-actor-path/{manifest.tsv,RULEBOOK.md,RUNBOOK.md}` and, in the record PR, `EXECUTION-RECORD.md` and one `migration/cost-log.tsv` row, all **created in the future implementation PRs, not now**. Kit commit `cf91c9d5068d9aaf95a36164169f08c3e636c909`. The manifest cites the Executed rows by number; the Go-only rows are listed explicitly and flagged **pending D13 approval** until the Owner decides Q10 and approves Go-only evidence. `PORT-GAP` records a known difference and never stands in for a pending decision (`docs/adr/0017-supervision.md:696-705`).

## Allowed files (future implementation; anything else is a stop; state at develop `abc94d5`)

| Path | Edit | State |
|---|---|---|
| `pekko-actor/actor/parse_actor_path.go` | new: `parseActorPath` and its helpers | absent |
| `pekko-actor/actor/parse_actor_path_test.go` | new: the `TestParse_...` scenarios (a second test file only if the first exceeds the PR budget) | absent |
| `pekko-actor/actor/errors.go` | doc comments of `ErrInvalidAddress` and `ErrInvalidPath` only (they say who returns them; the parser is a second source); no new sentinel | exists |
| `pekko-actor/README.md` | module contract: absolute parse implemented and unexported; limits (relative text, host and port); text row: "the round trip holds only for addresses whose protocol and system satisfy the parser's shapes; an Address built by NewLocalAddress with other characters is constructible and may print text that parses to a different path or fails" | exists |
| `migration/stage-1/idn-812-parse-actor-path/{manifest.tsv,RULEBOOK.md,RUNBOOK.md,EXECUTION-RECORD.md}`, `migration/cost-log.tsv` (one row) | kit package and record | absent; exists |
| `docs/scopes/812-parse-actor-path.md` | this file | exists |

Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `go.mod`, `go.sum` changes. SQLite parity (`CLAUDE.md`): not applicable, no persistence module is touched. Shared files with #852/#1089 (`errors.go`, `README.md`): merge in order; a shared surface creates no functional dependency.

## Validation plan (AGENTS.md "Go", once per final tree)

`cd pekko-actor && go test -count=1 ./...` (`unshare -rn` is unavailable on macOS: record the exit, accepted under G-6), `gofmt -l`, `go vet ./...`, `go mod tidy -diff`, `scripts/check_deps.sh`, `.github/scripts/check-headers.sh` byte-compared with the current develop output (the 7 pre-existing errors; **stop on any new one**), golangci-lint with gocyclo at most 10 (the parser is split as above; no `//nolint`), per-module coverage and `scripts/check_coverage.sh "$COV" 85`, diff-cover 11.0.0 at least 85%, `make ci-fast`, `.github/scripts/go-sdk-update.sh --check`, `git diff --check`. No `-race` unless the Owner asks. **PR budget (forecast, not measured):** kit PR about 250 authored lines; code PR about 110 (parser) + 260 (tests) + 20 (README, `errors.go`) = about 390; record PR about 90; each at most 400. If the code PR passes 400, move the tests to a second PR stacked after it, contract unchanged. The Dev starts only after the Owner grants G2 on the final SHAs and the kit is created.

## Points the Owner has NOT decided

Q10 (oracle for the covered rows, probe extension, D13 for the uncovered rows) and the approval of Go-only evidence; Q9 and Q11-Q18 (relative text); who owns the exact-reference-string parse; the error-class rule above is written here as the position rule (an Owner reading to confirm: `#` inside the system part is `ErrInvalidAddress`); **G10 option (ii)** (`NewLocalAddress` also enforces the shapes: it changes an exported constructor, ADR 0031 P-1 and ADR 0011 apply) would close the misparse above; it is **not decided**, the Owner chose parser-only (Q4); the issue edits (AC1/AC2 text, retitle is not proposed: the title stays); the ADR 0031 grammar addendum application; **G2**; the implementation start; the kit run; merge.

## Risk

`Risk: none` for this PR (documentation only). For the future code: no high-risk item, because the parser is **not exported** (item 3 would apply only if an export were added or if `checkPathElement`, the character set or the meaning of `ErrInvalidPath`/`ErrInvalidAddress` changed: then stop and report). It adds an unexported function reusing `checkPathElement`, `NewLocalAddress`, `NewRootPath` and `Child` unchanged; `Spawn` and `validChildName` are untouched; the existing tests of `ActorPath` and `Address` are the regression check for the shared validator.
