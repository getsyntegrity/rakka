# ADR 0031 addendum (proposal): grammar of actor path text

- Status: **PROPOSAL**; the Owner decided the observable grammar answers listed in "Owner decisions of 2026-10-29" below (the message "sí a todas"); every other question stays `pending`. This file is **not applied** to ADR 0031: applying the addendum to `docs/adr/0031` is a separate step, and nothing here claims the addendum is accepted as ADR text. Nothing is implemented. Any "recommended", "orientation" or "strict" value outside the decisions section is an Architect/advisor proposal, never an Owner decision.
- Story: #1187 (GOV-40). Consumer: #812 (IDN-03). Sibling proposal on the meaning of `#`: [`0031-addendum-incarnation-token-1186.md`](0031-addendum-incarnation-token-1186.md) (#1186).
- Date: 2026-10-29. goPekko revision read: `7840e8893305b0817b76d7b7f6e7d209afe0b8d7`; facts re-checked against `develop` `abc94d55e635704db3ec30c6735227a4c9676048` where marked (revision 6).
- Revision note (QA round 1): oracle facts corrected (the judge harness and a built `target` exist; see "Differential oracle feasibility"); AC1'/T2 restricted to paths whose address the grammar can express, with the address charsets now a coupled Owner decision (G6, G10); error classes made total with a fixed evaluation order; JDK `java.net.URI` rows now Executed; evidence labels and the `address.go` citation corrected (`:42-50`); relative-text questions listed (11 to 18); `?` as a glob wildcard noted. Owner decision still `pending`.
- Revision note (final-round QA): question self-reference corrected (relative text as a separate grammar is question 11); evaluation order fixed inside the system-part step so every rejection is in exactly one row (mixed input added); probe labels upgraded to Executed (JDK 11.0.32) or the claim narrowed; `..` in absolute text added (G11); `String()` of out-of-grammar addresses relabelled Source; the stale "G1 to G9" range now reads G1 to G11. Owner decision still `pending`.
- Revision note: documentation delta after final QA; no contract change.
- Revision note (probe revision 2): evidence re-read against probe revision 2; no contract change.
- Revision note (revision 6, 2026-10-29): records the Owner decisions of that date, marks the open questions as not decided, and fixes stale facts against develop `abc94d5` (#853 and #894 integrated and closed, #852 code integrated, the oracle on develop); the outcome table, the oracle table and G1-G11 are not altered.
- Pekko: tag `v1.7.1`, commit `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`. Paths are relative to
  `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor` (tests: `upstream/pekko/actor-tests/src/test/scala/org/apache/pekko/actor`).
- **Evidence classes** (every outcome and claim below carries one; a row without a class is not evidence):

| Class | Meaning |
|---|---|
| **Source** | Read in the pinned Pekko or in this repository (**Go today**) at the revision above, with file and line |
| **Executed (Pekko oracle)** | Run on the pinned Pekko by the probe of PR #1292, revision 2 (`observed.txt`, 196 rows; durable location `migration/stage-1/identity-oracle/pekko-repro/`, integrated on develop by PR #1292 (squash `d7af9bc`)); cited as `Executed (Pekko oracle, probe revision 2, row n)`. Element lists are printed `n=<size> ["a","b"]`, so an empty list and `[""]` are distinct |
| **Executed** | Actually run by the author of this revision, in a scratch directory outside the repository; the tool and version are stated. A JDK or Go **library** run is not a run of Pekko code |
| **Hand trace** | Derived by reading Pekko code step by step. Not run. Never evidence of parity |
| **JDK-unverified** | Depends on `java.net.URI` behavior (JDK library, outside the pin) that was not run |
| **Go design proposal** | A choice made in this document. Not evidence of anything |

Executed in this revision: (1) Go's `net/url` (Go `go1.27.2`, `darwin/arm64`, `url.Parse`, `EscapedPath`) over the inputs of the table in "Observable
grammar"; (2) a throwaway `java.net.URI` probe (single-file `java P.java` under the installed OpenJDK `11.0.32` Temurin, in a `mktemp -d` scratch outside
every repository, deleted afterwards) over the inputs of the table that carry a JDK cell (plus the ones added in the final round: `%zz`, `pekko://sys@h/a`, `1pekko://`, `pekko://s y@h/a`, `pekko://sys/user/..`), printing scheme, host, raw authority, raw path, fragment and query or the exception. That probe
exercises only the **JDK URI class**, not Pekko; Pekko classes were run separately by the probe of PR #1292 (revision 2), cited by row. Whether JDK 11 and the JDK 8 that `judge/run-pekko.sh` defaults to behave identically for these inputs is unverified. See "Differential oracle feasibility".

## Owner decisions of 2026-10-29

The Owner's reply was the two words **"sí a todas"**, to a decision package the coordinator presented for #812. The package is the **coordinator's text, not the Owner's words**; the Owner accepted its recommendations by that reply. Decided: **Q1** strict grammar; **Q2** error for empty segments (G1), trailing slash (G2) and a missing slash after the system (G3), and `..` a legal element without navigation (G11); **Q3** accept any syntactically valid protocol, a mismatch is a resolution outcome (G5); **Q4** the system-name set `[a-zA-Z0-9][a-zA-Z0-9-_]*` enforced by the parser only, `NewLocalAddress` unchanged (G6, G10 option (i)); **Q5** `#` is an error in the path parser (`ErrInvalidPath`, ADR 0031 P-7; G7); **Q7** AC1 and AC2 of #812 are rewritten as AC1' and AC2' (the issue text is not edited by this work; an Owner action); **Q8** the parser stays internal (unexported, no export justification, no #1020 row); the protocol shape `[A-Za-z][A-Za-z0-9+.-]*`. **Q6** is a consequence of Q2: empty segments are errors, an accepted difference from Pekko (rows 39/44, 45/50, 51/56). The error classes follow a position rule written in the #812 scope (PR #1285): a failure before the first `/` after `://` is `ErrInvalidAddress` (host and port forms included), after it `ErrInvalidPath`; no new sentinel and no third class.

**Not decided (kept pending): Q9 and Q11 to Q18** (relative text, stays with #1089, a separate decision); **Q10** (the #1292 oracle as the Pekko evidence for the rows it covers, a probe extension, D13 alternative evidence for the uncovered rows) and the approval of Go-only evidence for #812; who owns the parse of exact-reference strings (#894 is encode-only and closed; whether #1092 under its option B owns it is unsettled, and #812's first slice does not need it); the issue edits; G2; the implementation. Numbering note: #1291 (token addendum, merged) numbers its questions separately throughout. Verified mapping against `docs/adr/proposals/0031-addendum-incarnation-token-1186.md` on develop: its Q8 ("Do you agree with #1187 on `#`: error in the path parser, exact-reference parser separate?") is this file's **Q5** (the `#` rule); its Q10 (who owns the exact-reference parser) has no counterpart here; its other numbers (Q1-Q7, Q9) ask different questions (token, stale outcome, tokenless text, foreign epoch and so on), not this file's Q1-Q4, Q6-Q9. This file keeps its own numbering.

## Quick path

1. The Owner picked the **observable grammar** (Q1 to Q5, Q7, Q8 above); what remains is Q10, relative text and the exact-string parse owner. The implementation (splitter, `net/url`, `String()` only) is secondary and is chosen against the properties in "Implementation options".
2. The Owner picks what `#` means (G7), jointly with #1186, and **where relative text is decided** ("Relative text: place of decision").
3. #812's first slice (the absolute parser) is unblocked by these decisions, pending its G2. #1089 stays blocked until relative text has its own decision.

The strict grammar of every sub-decision was accepted (Q1); the implementation (hand-written splitter or not) remains #812's, against T1 to T5.

## Question

Which absolute local text strings does a path parser (`parseActorPath`, unexported: decided, Q8) accept, which error class does each rejected string get and which
value results, given that ADR 0031 P-8 says nothing is escaped or decoded (`%41` stays `%41`) and that Go has no `java.net.URI`?

| Concern | Question | Owner of the answer |
|---|---|---|
| Syntax (observable) | Which text is accepted, with which error class and value | This proposal and #812 |
| Implementation | How the code gets there | #812, against the properties below. Not part of the decision |
| Identity | What makes two paths equal | ADR 0031 P-6 (incarnation never in equality). Not reopened |
| Resolution | Finding a live actor for a path | #893, #894, #1092. The parser performs **no lookup**, creates no actor and resolves no incarnation |

Out of scope (as in the story): writing the parser, the incarnation token form (#1186), `sys@host:port` forms (ADR 0031 O1), editing ADR 0031. Relative text
is treated only as a question of **who decides it** (section "Relative text: place of decision").

## Pekko evidence (Source)

| # | Claim | Location |
|---|---|---|
| E1 | `ActorPath.fromString(s)` matches `ActorPathExtractor(address, elems)` and returns `RootActorPath(address) / elems`; otherwise throws `MalformedURLException` | `ActorPath.scala:72-75` |
| E2 | `/(Iterable)` folds the elements and **skips empty ones**; empty segments never reach validation | `ActorPath.scala:203-204` |
| E3 | `RootActorPath./(String)` splits `name#uid` through `ActorCell.splitNameAndUid`; `Integer.valueOf` of a non-numeric fragment throws | `ActorPath.scala:290-293`, `ActorCell.scala:400-404` |
| E4 | `ChildActorPath` rejects only `/` and `#` in a name. It does **not** call `isValidPathElement`, so `fromString` accepts what P-5 rejects | `ActorPath.scala:331-335` |
| E5 | `isValidPathElement`: empty rejected; ASCII letters and digits, `ValidSymbols` or `%XX`; a leading `$` is rejected (via `:147`) | `ActorPath.scala:77`, `:124-148` |
| E6 | `ActorPathExtractor`: `new URI(addr)`; `getRawPath` null gives no match; else `AddressFromURIString` plus `split(path, rawFragment).drop(1)`. `URISyntaxException` gives no match. The query is never read | `Address.scala:221-231` |
| E7 | `PathUtils.split` cuts at **every** `/` (empty segments kept) and appends `#fragment` to the last element | `Address.scala:148-160` |
| E8 | `AddressFromURIString`: scheme required; no user info and no host gives no match; with no user info a port gives no match; with user info host and port are required | `Address.scala:190-202` |
| E9 | `RelativeActorPath.unapply`: `new URI`, `None` if `isAbsolute`, else `split(getRawPath, getRawFragment)`; empty segments kept; scaladoc examples `"grand/child"`, `"/user/hello/world"` | `Address.scala:163-180` |
| E10 | Root `elements` is `emptyActorPath = List("")` (Executed, probe revision 2, row 10: `n=1 [""]`) | `ActorPath.scala:150`, `:295` |
| E11 | Specs: parse own `toString`; malformed `""`, `"://hallo"`, `"s://dd@:12"`, `"s://dd@h:hd"`, `"a://l:1/b"` give `MalformedURLException`; `fromString("pekko://mysys/user/boom/*")` works | `ActorPathSpec.scala:25-28`, `:50-56`, `:114` |
| E12 | Relative spec: `"foo"`, `"foo/bar/baz"`, a URL-encoded name and `"foo/bar/baz#1234"` give `List("foo")`, `List("foo","bar","baz")`, `List(name)`, `List("foo","bar","baz#1234")`. No spec for `""`, a leading `/`, `..`, a query or a colon in the first segment | `RelativeActorPathSpec.scala:28-40` |
| E13 | A foreign address is **not a parse error**; it is a resolution outcome (`deadLetters`) | `ActorRefProvider.scala:622-627`, `:629-638` |
| E14 | A system name must match `^[a-zA-Z0-9][a-zA-Z0-9-_]*$` at `ActorSystem` creation (not a path-text rule) | `ActorSystem.scala:840` |
| E15 | `..` is handled by `getChild` of the reference (`getParent`), not by the extractors | `ActorRef.scala:284-296` |

Consequence (Source): Pekko's `fromString` differs from ADR 0031 P-5 and P-7. It skips empty segments (P-5 rejects them), validates no character set
(P-5 does) and turns `#5` into a uid (P-7 makes `#` an error). Copying Pekko's parser would contradict accepted contract items; copying only its URI
front end would not decide them either. So the grammar is a decision, not a port.

## Go today (Source)

- `ActorPath` has `NewRootPath`, `Child`, `Parent`, `Descendant`, `Address`, `IsRoot`, `Name`, `Elements`, `String`; **no parser and no `Root`** (revision 6, develop `abc94d5`: `Parent` and `Descendant` are integrated, #852 code; `pekko-actor/actor/actor_path.go:34`, `:49`, `:65`, `:81`, `:146`). `Child` rejects empty, `/`, `#` and invalid characters with `ErrInvalidPath`
  (`actor_path.go:100`, `names.go:33`). `NewLocalAddress` rejects only an **empty** protocol or system with `ErrInvalidAddress` (`address.go:42-50`; its comment at `:40-41` says "Nothing else is
  checked, as in Pekko ... the system-name rule belongs to the creation of a system"): Go imposes no character set on the protocol or system name today
  (Pekko does for the system at `ActorSystem` creation, E14).
- `String()` prints `pekko://sys/` for a root and `pekko://sys/a/b` otherwise; zero value prints `""` (`actor_path.go:146`).
- Consequence (Source: `names.go`/`address.go` read; no Go snippet was run): Go can construct values the grammar cannot express. `NewLocalAddress("pekko", "a/b")` prints `pekko://a/b/` and `NewLocalAddress("a:b", "s")`
  prints `a:b://s/`; a parser for the text of the first reads system `a` and path `/b`, so the value does not round-trip. The zero `ActorPath` prints `""`, which the
  table rejects. Round trip (AC1', T2) therefore cannot be universal unless the address charsets are also constrained (G6, G10).

## Contradiction in the body of #812 (quoted, not changed)

| #812 body (acceptance criteria) | ADR 0031 (accepted 2026-10-24) | Conflict |
|---|---|---|
| AC1: "Valid root, child and escaped-segment paths round-trip through canonical formatting." | P-8 (`:96`): "nothing is escaped or decoded (`%41` stays `%41`). No escaping exists because `/` and `#` cannot occur in an element." | There is no escaping step. A segment written with `%XX` round-trips only because it is kept **verbatim** |
| AC2: "Malformed addresses, illegal segments and invalid encodings fail without actor lookup." | P-5 (`:93`) rejects empty, `/`, `#`, characters outside the P1 set; `%` must be followed by two hex digits | "Invalid encodings" can only mean a malformed `%XX`; there is no decoding to fail |

Proposed corrected wording (**Go design proposal**; the issue is not edited, the Owner decides whether #812 is amended):

- AC1': "For every non-zero `ActorPath` value `p` whose protocol and system satisfy the address charsets of the grammar (G6, G10), parsing `p.String()` returns a value equal to `p`. Elements that contain `%XX` are kept verbatim, never decoded or re-encoded." The restriction is deliberate: with `address.go:42-50` unchanged, paths with a system such as `a/b` or a protocol such as `a:b` exist and are outside the grammar; the zero value prints `""`, which is rejected. If the Owner takes G10 option (ii), the restriction disappears for every value `NewLocalAddress` can produce.
- AC2': "Text with a malformed address, an unsupported address form (host or port), an invalid or empty segment (ADR 0031 P-5), a `%` not followed by two hex digits, or a `#` (per G7) fails with a classified error. No actor lookup, actor creation or incarnation resolution happens."

## Observable grammar (the decision)

This section is independent of how the parser is written. It defines, for any input string, the outcome. The grammar below is a **Go design proposal**; the
Owner may change any sub-decision, which changes the table rows that depend on it.

```
path-text = protocol "://" system "/" [ element *( "/" element ) ]
protocol  = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )   ; RFC 3986 scheme shape; kept exactly as written (G4, G5, G10)
system    = ( ALPHA / DIGIT ) *( ALPHA / DIGIT / "-" / "_" )   ; Pekko's ActorSystem name set (E14); non-empty (G6, G10)
element   = P-5 element: 1*( ALPHA / DIGIT / one of  - _ . * $ + : @ & = , ! ~ ' ;  / "%" HEXDIG HEXDIG )
```

Rules: no other character is accepted anywhere (so `#`, `?`, `%`, `@`, `:` or space in the system part, `#`, `?`, space, a second `/` in a row and a trailing `/` after an element are rejected, G1 to G3, G6 to G8);
elements are kept byte for byte (G9); the root is `protocol://system/` only; the result is the address plus the element list, built with
`NewLocalAddress` and `Child`. A rejected string yields one error class (below) and no value. The parser performs no lookup.

**Evaluation order** (Go design proposal; it makes the class of every rejection unique): (1) split at the first `://`; (2) check the protocol; (3) take the system part as
everything after `://` up to the first `/` or the end of the text; **inside this step, first** test for `@` or `:` (unsupported form, whatever other characters are present), **then** the charset (address error); (4) check the path: the root slash, then each element. The first failing step
decides the class, so a forbidden character in the system part is an address error even when the text also lacks a root slash, and a system part with both a forbidden character and `@` or `:` (for example `s y@h`) is an unsupported form.

| Error class (Go design proposal) | Raised for |
|---|---|
| `ErrInvalidAddress` | step 1: no `://` (including `""` and `/user/a`); step 2: empty protocol, or a protocol outside its charset (a leading digit, `a:b`); step 3: empty system part, or a system part that contains **neither** `@` nor `:` and has a character outside the system charset, for example `s#y`, `s?y`, `s%y`, `s y`, control characters |
| Unsupported-form class | step 3, tested before the charset: the system part contains `@` or `:`, alone or with other bad characters (user-info, host or port forms: `sys@h:1`, `sys@h`, `sys:1`, mixed `s y@h`; ADR 0031 O1). Its name, and whether it is a separate sentinel or wraps `ErrInvalidAddress`, were left to #812 under ADR 0011; the Owner's decisions of 2026-10-29 are applied by the position rule of the #812 scope: it is `ErrInvalidAddress`, no new sentinel, no third class |
| `ErrInvalidPath` | step 4: no root slash after the system part, an empty segment, a trailing slash, an invalid element (P-5; `..` is a legal element, G11), a `#` or `?` in the path, a `%` not followed by two hex digits |

Every rejection rule of the grammar is in exactly one row. If the Owner does not want the unsupported-form class named in this addendum, T1 and T5 are limited to accept/reject and to the sentinel family
(`ErrInvalidAddress` or `ErrInvalidPath`), and the split is left entirely to #812.

**Outcome table.** "Proposed" is the Go design proposal under the strict values. "Pekko `fromString`" shows the value and its class, with the evidence class of that
cell; a cell that cites "JDK URI Executed" means the JDK 11.0.32 `java.net.URI` probe above, the rest of the chain being Source or Hand trace. "`net/url`" is **Executed**
(Go go1.27.2, `url.Parse`; `Path` is the decoded path, `RawPath`/`EscapedPath` the raw one).

| Input | Proposed outcome | Pekko `fromString` (class) | Go `net/url` (Executed) |
|---|---|---|---|
| `pekko://sys/user/a` | OK `/user/a` | OK `/user/a` (Source E1, E6) | OK: scheme `pekko`, host `sys`, path `/user/a` |
| `pekko://sys/` | OK root | OK root (Executed (Pekko oracle, probe revision 2, rows 33, 38): `ActorPathExtractor.unapply` = `Some((pekko://sys, n=1 [""]))` (`PathUtils.split` of `/` gives `["",""]`, `.drop(1)` leaves `[""]`), `fromString` = `pekko://sys/`, elements n=1 [""], isRoot=true) | OK, path `/` |
| `pekko://sys` | `ErrInvalidPath` (G3) | OK root (Executed (Pekko oracle, probe revision 2, rows 39, 44): extractor = `Some((pekko://sys, n=0 []))` (raw path `""` splits to `[""]`, `.drop(1)` leaves `[]`), `fromString` = `pekko://sys/`, elements n=1 [""], isRoot=true; JDK URI Executed: raw path is the empty string, not null) | Accepted, path empty |
| `pekko://sys//a` | `ErrInvalidPath` (G1) | OK `/a`, empty skipped (Executed (Pekko oracle, probe revision 2, rows 45, 50): extractor n=2 ["","a"], `fromString` = `pekko://sys/a`, elements n=1 ["a"]) | Accepted, path `//a` |
| `pekko://sys/a/` | `ErrInvalidPath` (G2) | OK `/a` (Executed (Pekko oracle, probe revision 2, rows 51, 56): extractor n=2 ["a",""], `fromString` = `pekko://sys/a`, elements n=1 ["a"]) | Accepted, path `/a/` |
| `pekko://sys/a%41` | OK, element `a%41` verbatim (G9) | OK, element `a%41` (Executed (Pekko oracle, probe revision 2, rows 57, 62): elements n=1 ["a%41"]; Source E4: no char check) | Accepted; `Path` is `/aA` (decoded), `RawPath` is `/a%41` |
| `pekko://sys/a%4` | `ErrInvalidPath` (G9) | Error (Executed (Pekko oracle, probe revision 2, rows 63, 68): `fromString` throws `MalformedURLException` ("cannot parse as ActorPath: pekko://sys/a%4"), extractor `None`; cause JDK URI Executed: `Malformed escape pair`) | Error: `invalid URL escape "%4"` |
| `pekko://sys/%zz` | `ErrInvalidPath` (G9) | Error (Source E6 catch plus JDK URI Executed, JDK 11.0.32: `Malformed escape pair at index 12`; chain Hand trace) | Error: `invalid URL escape "%zz"` |
| `pekko://sys/a#5` | `ErrInvalidPath` (G7) | OK, uid 5 on `a` (Executed (Pekko oracle, probe revision 2, rows 69, 74): `fromString` elements n=1 ["a"], serialization `pekko://sys/a#5`, uid=5; extractor n=1 ["a#5"]; Source E3, E7) | Accepted; `Fragment` is `5`, `Path` is `/a` |
| `pekko://sys/a#` | `ErrInvalidPath` (G7) | Error (Executed (Pekko oracle, probe revision 2, rows 135, 140): `fromString` throws `java.lang.NumberFormatException` (`For input string: ""`), extractor still `Some` (n=1 ["a#"]); JDK URI Executed: fragment is the empty string, not null) | Accepted; `Fragment` empty, indistinguishable from no `#` |
| `pekko://sys/a#x` | `ErrInvalidPath` (G7) | Error (Executed (Pekko oracle, probe revision 2, rows 123, 128): `fromString` throws `java.lang.NumberFormatException` (`For input string: "x"`), not a `MalformedURLException` and not a typed error; extractor `Some` with n=1 ["a#x"]; accepted difference: Go returns a typed error) | Accepted; `Fragment` is `x`, `Path` is `/a` (not run) |
| `pekko://sys/a?x=1` | `ErrInvalidPath` (G8) | OK `/a`, query ignored (Source E6; JDK URI Executed: query `x=1`; Executed (Pekko oracle, probe revision 2, rows 75, 80: `pekko://sys/user/a?q` gives `pekko://sys/user/a`, elements n=2 ["user","a"])) | Accepted; `RawQuery` is `x=1` |
| `pekko://sys/a b` | `ErrInvalidPath` (P-5) | Error (Source E6 catch plus JDK URI Executed: `Illegal character in path`; chain Hand trace) | Accepted; `EscapedPath` is `/a%20b` |
| `""` | `ErrInvalidAddress` | Error (Executed (Pekko oracle, probe revision 2, rows 93-98): `fromString` throws `MalformedURLException` ("cannot parse as ActorPath:"), extractor `None`; Source `ActorPathSpec.scala:51`) | Accepted: empty URL, not absolute |
| `://x` | `ErrInvalidAddress` | Error (Executed (Pekko oracle, probe revision 2, rows 99-104): `fromString` throws `MalformedURLException`, extractor `None`; also by analogy to the spec case `://hallo` (`ActorPathSpec.scala:52`); JDK URI Executed: `Expected scheme name at index 0`) | Error: `missing protocol scheme` |
| `pekko:///a` | `ErrInvalidAddress` (G6) | Error (Source E8 line 192; JDK URI Executed: scheme `pekko`, host and authority null, so no match) | Accepted; host empty, path `/a` |
| `pekko://sys@h:1/a` | unsupported form (G6, O1) | OK remote address (Executed (Pekko oracle, probe revision 2, rows 105-110): `fromString` = `pekko://sys@h:1/a`, address `pekko://sys@h:1`, elements n=1 ["a"]; Source E8) | Accepted; user `sys`, host `h:1` |
| `pekko://sys:1/a` | unsupported form (G6) | Error (Source E8: port without user info) | Accepted; host `sys:1` |
| `/user/a` | `ErrInvalidAddress` (relative text is not this parser's input) | Error (Executed (Pekko oracle, probe revision 2, rows 111-116): `fromString` throws `MalformedURLException`, extractor `None`; Source E8: no scheme) | Accepted; not absolute, path `/user/a` |
| `PEKKO://sys/a` | OK, protocol `PEKKO` kept (G4) | OK, scheme as written (JDK URI Executed: `getScheme` returns `PEKKO`; the Pekko chain is Hand trace) | Accepted; scheme **lowercased** to `pekko` |
| `pekko://my_sys/a` | OK if G10 admits `_` (E14 does) | **Error** (Executed (Pekko oracle, probe revision 2, rows 81-86): `fromString` throws `MalformedURLException` ("cannot parse as ActorPath: pekko://my_sys/user/a"), `AddressFromURIString.apply/parse` throw it, `unapply` and the extractor `None`; cause JDK URI Executed: host `null`, authority `my_sys`, no user info (Source E8 line 192); `ActorSystem` itself accepts `my_sys`, E14) | Accepted; host `my_sys` (Executed) |
| `pekko://s y/a` | `ErrInvalidAddress` (step 3) | Error (Executed (Pekko oracle, probe revision 2, rows 87-92): `fromString` throws `MalformedURLException`, extractor `None`; JDK URI Executed: `Illegal character in authority`) | Error: `invalid character " " in host name` (Executed); with a tab: `invalid control character in URL` |
| `pekko://s#y/a` | `ErrInvalidAddress` (step 3) | Hand trace: JDK URI Executed gives host `s`, raw path empty, fragment `y/a`; likely accepted as root with the fragment dropped by `drop(1)`; not run | Accepted; host `s`, path empty, fragment `y/a` |
| `pekko://s?y/a` | `ErrInvalidAddress` (step 3) | Hand trace: JDK URI Executed gives host `s`, raw path empty, query `y/a`; likely accepted as root; not run | Accepted; host `s`, query `y/a` |
| `pekko://s%y/a` | `ErrInvalidAddress` (step 3) | Error (Source E6 catch plus JDK URI Executed: `Malformed escape pair`) | Error: `invalid URL escape "%y"` |
| `pekko://sys@h/a` | unsupported form (step 3) | Error (Source E8 line 197: user info without port; JDK URI Executed, JDK 11.0.32: user `sys`, host `h`, port `-1`; chain Hand trace) | Accepted; user `sys`, host `h` |
| `pekko://s y@h/a` (bad character and `@`) | unsupported form (step 3, `@` tested first) | Error (Source E6 catch plus JDK URI Executed: `Illegal character in authority at index 8`) | not run (mixed input added in the final round; the unsupported-form class is decided by the evaluation order, not by `net/url`) |
| `pekko://sys/user/..` | OK, element `..` (G11; P-5 admits `.`) | Hand trace: OK element `..` kept by the extractor (E15: `..` is handled by `getChild`); JDK URI Executed: raw path `/user/..` | not run |
| `a:b://s/a` | `ErrInvalidAddress` (step 2) | Error (Hand trace: JDK URI Executed parses scheme `a` with null authority, E8 no match) | Accepted; scheme `a`, **opaque** `b://s/a` |
| `1pekko://s/a` | `ErrInvalidAddress` (step 2) | Error (Source E6 catch plus JDK URI Executed, JDK 11.0.32: `Illegal character in scheme name at index 0`; chain Hand trace) | Error: `first path segment in URL cannot contain colon` |
| `pekko://a/b/a` (the text `NewLocalAddress("pekko","a/b")` prints for a child `a`) | system `a`, path `/b/a`: no round trip (see "Go today") | OK (Hand trace) | Accepted; host `a`, path `/b/a` |
| zero `ActorPath` (`String()` is `""`) | `ErrInvalidAddress`; outside AC1'/T2 | same as `""` | same as `""` |

Reading the table (Executed facts): `net/url` accepts empty segments, a trailing slash, a missing root slash, a query, a fragment, a host with a
port, `#` and `?` inside what the proposal calls the system part (they start the fragment or query), `a:b://s/a` as an opaque URL and the empty URL, and it
lowercases the scheme and decodes `Path`; it rejects a space or control character in the host and `1pekko://`. The JDK `java.net.URI` (Executed) rejects a space but accepts `_`
in an authority as a registry name (host null), which is what makes Pekko's own `fromString` reject `my_sys`. So **`url.Parse` alone does not define the accepted set**: every row where "Proposed"
is an error but `net/url` says "Accepted" needs an explicit post-check. A hand-written splitter does not define the accepted set either; it implements a
grammar that must first exist. That is why the grammar is the decision.

### Sub-decisions behind the table

| ID | Question | Pekko (class) | Strict value (proposal) | Lenient value |
|---|---|---|---|---|
| G1 | Empty segment | Skipped (Source E2; Executed, rows 45, 50) | Error (P-5 rejects empty) | Skip, as `Descendant` (P-4) |
| G2 | Trailing slash | Accepted, empty skipped (Source E2, E7; Executed, rows 51, 56) | Error | Accept and drop it |
| G3 | No slash after system | Root (Executed, rows 39, 44) | Error (not what `String()` prints) | Root |
| G4 | Case of the protocol | Kept as written (JDK URI Executed: `getScheme` returns `PEKKO`; Pekko chain Hand trace) | Kept exactly, never folded | Fold to lower case |
| G5 | Protocol other than `pekko` | Accepted; mismatch is a resolution outcome (Source E13) | Accept any syntactically valid protocol; mismatch left to resolution | Accept only `pekko` |
| G6 | System part | Empty gives no match; `sys@h:1` accepted as remote (Source E8); a name with `_` is accepted by `ActorSystem` (E14) but gives no match in `fromString` (Executed, rows 81-86; JDK URI Executed) | Charset of E14 (`[a-zA-Z0-9][a-zA-Z0-9-_]*`); `@` and `:` rejected as unsupported forms (O1); any other character is an address error | Any character except `/ @ : # ? %`, whitespace and control characters |
| G7 | `#` | Becomes a uid (Source E3, E7; Executed, rows 69, 74); a non-numeric `#x` throws `NumberFormatException` (rows 123, 135) | Error in the path parser (decided, Q5); the encoding token form of #1186 is built by `ExactRefString` (#894, integrated); the parse of exact strings is not owned by #894 (encode-only) and its owner is unsettled (P-7) | Accept `#` and hand the token to #1186's function |
| G8 | `?query` | Ignored (Source E6; Executed, rows 75-80) | Error | Ignore |
| G9 | `%XX` | Kept in the raw path (Source E6; Executed, rows 57, 62) | Verbatim; `%` without two hex digits is an error | none (decoding contradicts P-8) |
| G10 | Who enforces the address charsets (protocol G5/G6, system G6) | `Address(protocol, system)` checks nothing (`Address.scala:37-44`, as `address.go:40-41` says) | (i) the **parser only**; `NewLocalAddress` stays as is; AC1'/T2 restricted as above | (ii) `NewLocalAddress` also rejects what the grammar rejects (changes `address.go:42-50`, an exported constructor: ADR 0031 P-1 and ADR 0011 apply, and it departs from "as in Pekko"); then T2 is universal over non-zero values |
| G11 | `.` and `..` as elements of absolute text | `..` is not rejected by the extractors; `getChild` of a reference interprets it (E15; Hand trace) | Legal element (P-5 admits `.`, so `pekko://sys/user/..` parses to element `..`); no navigation in the parser. Tied to question 14 (relative text and `..`) | Reject `.` and `..` as elements |

**Consequence of G10 option (i), the one chosen by Q4 (the shapes are enforced by the parser only; `NewLocalAddress` unchanged):** an `Address` whose system contains `/` (or `#`, `?`, `@`, `:` or a space) is constructible, and the text of its paths is either rejected by the strict parser or, for a system containing `/`, accepted and parsed to a different path (for the system `a/b`, `pekko://a/b/x` reads as system `a`, path `b/x`); the round trip is not promised for such addresses (AC1', T2). Option (ii) (`NewLocalAddress` also enforces the shapes: an exported constructor, ADR 0031 P-1 and ADR 0011) would close it and is **not decided**.

## Implementation options (secondary; choose against the properties)

Any option is acceptable if and only if it passes **all** of these properties. The properties, not the option, are the contract of #812.

| Property | How it is tested |
|---|---|
| T1 Accepted set equals the grammar | The outcome table above as table-driven tests, plus a generated-input test against a second, independent statement of the grammar (for example a character-class scan written separately) over short strings of the alphabet `{p,e,k,o,:,/,#,?,%,4,1,z,@,space,.,$,_}`; both must agree on accept/reject and on the error class |
| T2 Round trip | `Parse(p.String())` equals `p` for every non-zero path whose address satisfies the grammar (AC1'); under G10 (ii), for every non-zero path `NewLocalAddress` and `Child` can build. Paths outside that set are a stated exclusion, not a pass |
| T3 Verbatim | `%XX` elements are byte-identical in and out; no decode, no re-encode, no case folding of the protocol |
| T4 No lookup | The parser touches no registry, `System` or `Ref` (compile-time: it takes only a string) |
| T5 Classified errors | Each rejected row returns its class and wraps the sentinel so `errors.Is` works |

| | (a) Hand-written splitter | (b) `net/url` based | (c) Accept only `String()` output |
|---|---|---|---|
| Idea | Split on `://` and `/`, build the address with `NewLocalAddress`, validate each element with P-5 | `url.Parse`, then map and post-check | A prefix check plus the splitter of (a) over the exact `String()` shape |
| What it needs to pass T1 | The grammar implemented once | A post-check for each Executed "Accepted" row above, and `EscapedPath` instead of `Path` to keep P-8; the scheme re-read from the input | Same as (a), restricted to the strict values |
| Risk | Drift from `String()`; covered by T2 | Accepted set depends on stdlib versions (Executed on go1.27.2 only); easy to read `Path` and break T3 | Too strict if the Owner later picks lenient values |

Common to all: the parser returns a value or a classified error and never touches the actor registry. With all-strict values, (a) and (c) accept the same set.
Architect orientation (not a decision): (a) with T1 to T5 as acceptance tests; (b) only with the post-checks written and tested.

## Relative text: place of decision

Today #1187 sends relative text to #1089 and #1089 (an implementation story, `status:blocked`) lists "R1 relative path text without a scheme MUST yield its
elements ... empty segments dropped" and "R2 absolute text MUST be reported as not relative", but **no document decides** the points below, and a story
body is not an addendum. Left alone, the implementer of #1089 would decide them in code.

Open points (none is answered by Pekko specs, E12): leading `/` (`/user/hello/world` is a scaladoc example, E9); empty text (Executed (Pekko oracle, probe revision 2, row 154): `RelativeActorPath.unapply('')` = `Some(n=1 [""])`, one empty segment, not an empty list); `..` (handled by `getChild`, not the extractor, E15); a colon in the first segment (Executed: Go `url.Parse("a:b/c")` reports scheme `a`, absolute, so
the text is "not relative"; JDK URI Executed: `a:b://s/a` parses with scheme `a`; Executed (Pekko oracle, probe revision 2, row 158): `RelativeActorPath.unapply('a:b/c')` = `None`); query (row 157: `foo?x=1` = `Some(n=1 ["foo"])`, dropped); fragment and `#uid` (E12 keeps `baz#1234` as an element; row 160: `foo/bar#1234` = `Some(n=2 ["foo","bar#1234"])`); empty segments
(Pekko keeps them, E9; rows 155, 161, 162: `foo//bar` n=3, `foo/bar/` n=3 ["foo","bar",""], `/foo` n=2 ["","foo"]).

Two conflicts that exist today and that this addendum does not resolve:

- **Empty segments.** #1089 R1 says empty segments are **dropped**; G1 (strict) makes them an **error** in absolute text. Both can hold only if relative and absolute text
  are different grammars. That is itself a decision (question 11 below), and Pekko keeps empty segments in the relative extractor (E9) while skipping them later in `/(Iterable)` (E2).
- **`#uid`.** #1089 AC-R1-2 accepts "the result equals the upstream spec case at line 38 [`RelativeActorPathSpec.scala:38-40`, element `baz#1234`], or the divergence is recorded as a
  PORT-GAP". G7 (strict) makes `#` an error in absolute text, and a PORT-GAP documents a decided
  difference; it cannot stand in for a pending decision. Until question 15 is answered, neither branch of AC-R1-2 can be claimed.

| Option | Consequence |
|---|---|
| (A) Widen this addendum (#1187) to cover relative text | One grammar and one set of error classes; the shared element rules (P-5, G7, G9) stay in one place. Costs: the proposal grows and the Owner decides a lower-priority topic (#1089 is `prio:P3`) in the same pass as the blocking one (#812) |
| (B) A separate decision issue and addendum (for example "GOV-xx relative path text grammar") | #812 is not delayed by P3 questions. Costs: one more issue to file and review; it must reuse the element, `#` and `%` rules decided here |

**Recommendation (Architect proposal, not a decision): (B)**, with this addendum fixing only the shared layer that relative text must inherit (P-5 per element,
G7 for `#`, G9 for `%`). Until relative text is decided, **#1089 (scope PR #1286) is blocked**, and so is any part of #893 that needs relative text. #812 is
not blocked by it. If the Owner prefers (A), questions 11 to 18 below become questions of this document; under (B) they move, unchanged, to the new issue. Questions 9 and 10 keep their numbers.

Relative-text questions (each with the consequence for #1089 and #893; the proposal states no value):

| Q | Question | Consequence if the answer is lenient or strict |
|---|---|---|
| 11 | Is relative text a separate grammar from absolute text? | If no, #1089 R1 (drop empties) contradicts G1 and must change; if yes, two error-class tables |
| 12 | Empty segments in relative text: dropped (as #1089 R1), kept (E9) or rejected (G1)? | Decides #1089 R1 and the wording of AC-R1-1 |
| 13 | Leading `/` (`/user/hello/world`, E9): relative, absolute-local, or invalid? | #1089 AC-R1-1 lists it; #893 needs it for anchored selection |
| 14 | Empty text and `..` (E15): empty list, error, or a path operation outside the parser? | `..` affects #893 navigation (`getChild`) |
| 15 | `#uid` and bare `#` in relative text (E12): kept as part of the element, rejected, or handed to #1186's token function? | Decides whether AC-R1-2 can pass or needs a decided PORT-GAP |
| 16 | A `:` in the first segment (`a:b/c`): relative element, or "absolute with scheme `a`"? | #1089 R2 (absolute text reported as not relative) |
| 17 | Query (`?`) in relative text: error or ignored (Pekko ignores it, E6)? | See `?` below |
| 18 | `?` and `*` as glob wildcards for #893 | Consequence, not a decision: Pekko treats `?` and `*` in a selection element as wildcards (`ActorSelection.scala:223`, `util/Helpers.scala:44-45`, Source), while P-5's symbol set contains `*` and not `?`, and G8 rejects `?` in path text. If #893 wants `?`, its pattern text cannot be this parser's input; `*` is already a legal element character |

## Differential oracle (executed)

**Absence of a spec did not mean absence of an oracle.** The pinned code was **run** on the input table: the probe of PR #1292 (revision 2; durable location `migration/stage-1/identity-oracle/pekko-repro/`, integrated on develop by PR #1292 (squash `d7af9bc`), `observed.txt`, 196 rows) printed the outcome of `ActorPath.fromString`, `AddressFromURIString` (`apply`, `parse`, `unapply`), `ActorPathExtractor.unapply` and `RelativeActorPath.unapply`, with element lists as `n=<size> ["a","b"]`. Revision 1 printed `List()` for both an empty list and `List("")` and is not used. The outcome table, the sub-decisions and the relative-text points cite its rows.

**Rows the probe did not cover** (their Pekko cells keep the label they had): `pekko://sys/%zz`, `pekko://sys/a b`, `pekko:///a`, `pekko://sys:1/a`, `PEKKO://sys/a`, `pekko://s#y/a`, `pekko://s?y/a`, `pekko://s%y/a`, `pekko://sys@h/a`, `pekko://s y@h/a`, `pekko://sys/user/..`, `a:b://s/a`, `1pekko://s/a`, `pekko://a/b/a`. They stay Source, Hand trace or JDK-executed as labelled; a further probe row would replace them. ADR 0033 D13 (alternative evidence plus Owner approval) applies to the Go-only rows (typed errors and their classes, the zero value, the round-trip claim, every Go-versus-Pekko difference), which have no Pekko row and stay Go design, pending Owner approval.

**Mechanism** (Source, unchanged): `judge/run-pekko.sh <pekko-copy> <outdir> <controlled|concurrent> [deadline_ms] [scenario ...]` runs `Judge.scala` in a `git archive` export of the pinned tag; the precedent for a focused one-off run is `migration/stage-1/msg-02-failure-sender/pekko-repro/` (`RUN.md`, `Probe.scala`, `observed.txt`). `Judge.scala` has no path-parsing scenario (0 matches for `fromString`, `RelativeActorPath`, `AddressFromURIString`, `ActorPathExtractor`); the executed probe is a `Probe`-style run, not a harness scenario. The proposal does not claim parity.

## Treatment of the points the story names

| Point | Proposal (orientation) |
|---|---|
| `#` | Error in the path parser (G7, decided, Q5). The encoding token is built by `ExactRefString` (#894, integrated); the parse of exact strings has no settled owner. Agreed with #1186, see its "Agreement with #1187" |
| Empty segments | Error (G1). Pekko skips them (Source E2); an intentional Go difference to be recorded next to ADR 0031's list (a) to (g) |
| Trailing slash | Error (G2), except the root text `pekko://sys/` |
| Uppercase scheme | Kept as written (G4) |
| Empty system | `ErrInvalidAddress` (G6) |
| `%XX` | Verbatim, never decoded; malformed `%` is an error (G9, P-8) |

## ADR impact list (ADR 0033 D5)

Status is read from the file at the revision above.

| ADR item | Status | Effect of this proposal |
|---|---|---|
| ADR 0031 O1 (`:135`) | accepted 2026-10-24 | `sys@host:port` stays rejected (G6) |
| ADR 0031 O2 (`:136`), "no grammar is defined here" | accepted | This addendum is the grammar O2 deferred; owner story #812 |
| ADR 0031 P-4 (`:92`), `Descendant` skips empty names | accepted, implemented (#852 code, `actor_path.go:81`) | With G1 strict, `Descendant` skips empties and the parser rejects them: they agree only on inputs without empties (affects #852) |
| ADR 0031 P-5 (`:93`), P-6 (`:94`) | accepted | P-5 reused per element; P-6 unchanged |
| ADR 0031 P-7 (`:95`), `#` is an error | accepted | Followed under G7 strict; G7 lenient would extend it |
| ADR 0031 P-8 (`:96`), nothing escaped or decoded | accepted | Violated by a `net/url` parser that reads `Path` (Executed row `a%41`); kept otherwise |
| ADR 0011 and ADR 0033 D3 (`:37`) | accepted | Whether the parser is **exported** needs a per-export justification. **Open**, belongs to #812. Its form is #1021; #1020 is only an index |
| ADR 0033 D8 (`:42`), D13 (`:47`) | accepted | Local forms only. D13 applies only after the oracle question above is answered |
| ADR 0024, 0014 D2 | accepted | Not involved |
| ADR 0032 D5 | split into D5a and D5b, both decided "follow Pekko" on 2026-10-28 (`Status` line 3 and "D5a and D5b" of `docs/adr/0032-local-sender-envelope.md` at the revision above) | Not involved |
| `address.go:42-50` `NewLocalAddress` | code, not an ADR item | Touched only if G10 option (ii) is chosen |

Citations in the body of #1187 to "ADR 0031:128" and "(:88)" are stale; the current lines are `:136` (O2) and `:96` (P-8).

## Consumers affected

| Consumer | Effect |
|---|---|
| #812 (parser) | Implements the grammar; AC1' and AC2'; acceptance tests T1 to T5 |
| #1089 (relative text) | **Blocked until relative text is decided** (previous section); reuses the shared layer |
| #852 (navigation) | code integrated; "appending segments equals parsing the full path" holds for inputs without empties (G1) |
| #853 (path of a reference) | integrated and closed; `String()` of that path must parse back (AC1', restricted to the admitted address shapes) |
| #894 / #1092 | #894 (encode) is integrated and builds `path#token` with `ExactRefString`; the parse of such a string needs a second function whose owner is unsettled (G7); the parse error is separate from a stale outcome (the parser never looks up) |
| #893 | Needs relative text and possibly `*`/`?` selection (Pekko accepts `*` in `fromString`, E11; `?` and `*` are glob wildcards in a selection, question 18); whether Go text allows wildcards is #893's decision |

## Recommendation and Owner decision

**Architect orientation (not a decision):** the strict grammar of G1 to G11 as the observable decision, tested by T1 to T5; implementation by a hand-written
splitter ((a)); relative text decided separately (B). Reason: the accepted set equals what `String()` prints, P-5, P-7 and P-8 hold by construction, errors
are classified, and the stdlib's measured laxness (Executed) is not inherited.

- Owner decision: **decided** for Q1 to Q5, Q7, Q8, the protocol shape and (as a consequence) Q6, by the message "sí a todas" (section "Owner decisions of 2026-10-29"); **`pending`** for Q9, Q10 and Q11 to Q18, the exact-string parse owner, the issue edits and G2.
- Decided by this document: nothing; the decisions are the Owner's. The addendum is not applied to ADR 0031 here.

### Owner questions

1. Observable grammar: strict (the table) or which lenient values? *(decided: strict, 2026-10-29)*
2. G1/G2/G3 (and G11 for `..`): reject empty segments, trailing slash and a missing root slash (strict), or skip them as Pekko does (lenient)? *(decided: reject; `..` a legal element)*
3. G5: accept any syntactically valid protocol and leave a mismatch to resolution, or only `pekko`? *(decided: accept any valid protocol)*
4. G6 and G10: system-name set of E14 (strict) or a wider set; protocol shape; and does `NewLocalAddress` enforce the same charsets (G10 option ii) or only the parser (option i)? Related rows: `my_sys`, `s y`, `a/b`, `a:b`. *(decided: the E14 set and the URI-style protocol shape, by the parser only, option (i))*
5. G7: is `#` an error in the path parser (uid form a separate function), or accepted and handed to #1186's token function? *(decided: error in the path parser)*
6. Is the intentional difference "empty segments are errors" (versus Pekko's `/(Iterable)`) accepted? *(accepted as a consequence of Q2)*
7. Do you want #812's AC1 and AC2 amended to AC1' and AC2' (yes or no)? *(decided: yes; the issue edit is an Owner action)*
8. Is the parser exported or internal? (ADR 0011, ADR 0033 D3) *(decided: internal)*
9. Relative text: (A) widen this addendum or (B) a separate decision issue? *(NOT decided)*
10. Accept the executed probe (PR #1292, revision 2) as the Pekko evidence for the rows it covers, and extend it over the uncovered rows of the outcome table (yes or no)? *(NOT decided)* If no for those rows, do Source, Hand trace and Go-only parse tests count as D13 alternative evidence ("pending approval")?
11. to 18. Relative-text questions (table in "Relative text: place of decision"); asked here under option (A), moved to a separate issue under (B). *(NOT decided)*
