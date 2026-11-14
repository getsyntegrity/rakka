# Scope: Parse relative actor path text (#1089, IDN-17)

Status: **Draft for gate G2; not approved; no implementation.** Rewritten on 2026-11-04 against the ADR 0031 amendment of 2026-11-04 (PR #1324, merged into develop as `537002a`) and Apache Pekko v1.7.1 `9a54d1294c3ac7821bbed3c940769c3d5027d1ff` (`upstream/pekko`). The earlier text of this scope (empty segments rejected, `#uid` blocked, a Go-only "not relative" signal) is replaced; the Owner's principle is that every functional decision is Pekko's and only Go form differs.
**Kit run required (ADR 0017/0024)**: this is an adaptation of Pekko's `RelativeActorPath` (`java.net.URI` based) to Go; the Dev runs the migration kit with its own manifest, RULEBOOK and RUNBOOK. The Dev is a separate session; the TL does not implement; QA is independent.

## Authority (Owner decision, point 2 of the amendment, quoted in #1324)

"Path relativo: como RelativeActorPath de Pekko (segmentos vacíos conservados, texto vacío, `baz#1234` como elemento)." Points 5 and 6 also apply: nothing that differs from Pekko is accepted, and evidence that is not a differential scenario is recorded under ADR 0033 D13 without declaring parity.

## What this delivers

A function that takes relative path text and returns what `RelativeActorPath.unapply` returns (`A/Address.scala:170-180`): the elements, or Pekko's `None`.

```go
// shape of the contract, not the implementation; name and whether it is exported are open (Go form)
func parseRelativeActorPath(text string) (elements []string, ok bool) // ok == false is exactly Pekko's None
```

- `ok == false` when Pekko returns `None`: the text is absolute (`uri.isAbsolute`) or `java.net.URI` rejects it. Pekko does not tell the two apart, so this function does not either (no extra sentinel).
- Otherwise the elements are Pekko's `split(rawPath, rawFragment)` (`A/Address.scala:149-160`): empty segments kept, a trailing `""` kept, a leading `""` for a leading `/`, the text `""` gives `[""]`, a query is dropped, a fragment is appended to the last element (`foo/bar#1234` -> `["foo","bar#1234"]`).
- **No element validation.** Pekko's relative parser validates nothing; `/(String)` checks only `/` and `#` (`ChildActorPath` constructor, `A/ActorPath.scala:328-334`) and the `/(Iterable)` overload skips empty elements (`:203-204`); full name validation (`validatePathElement`, `A/ActorPath.scala:97-115`) happens at actor creation (`A/dungeon/Children.scala:274`), so what is done with the elements is navigation (#852) and selection (#893). The earlier `checkPathElement` rule is removed because it would reject what Pekko returns.
- No actor is created or resolved; the function takes only the text (observable proof: `idSeq` unchanged, `id.go:41-44`, with no System constructed).

## What this does not deliver

- Absolute text: #1322. Exact strings: #1323, #1092. Navigation: #852. Selection, wildcards, `Identify`: #893. Interpreting `..` (the extractor only splits; `getChild` interprets it, `ActorRef.scala:284-296`).

## Dependencies

| Dependency | State | Effect |
|---|---|---|
| ADR 0031 amendment 2026-11-04 (#1324) | merged, `537002a` | the authority for this scope |
| #1322 (absolute parser) | open | independent: no code shared; sequencing only |
| #893 (selection) | open, blocked | consumer; not a prerequisite for the rest of #893 |
| #1186/#1187 | open | to be re-proposed against the amendment; no longer decide any row here |

## Criteria -> scenario -> probe row

Path prefixes: `A/` = `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/`; `T/` = `upstream/pekko/actor-tests/src/test/scala/org/apache/pekko/actor/`. Pekko uses `getRawPath` and `getRawFragment`: percent-escapes are kept, not decoded (`%41` stays `%41`), and `java.net.URI` accepts non-ASCII "other" characters while rejecting spaces.

Level: **unit**, `pekko-actor/actor`, go-specs matchers, no sleeps. "Probe row" is the line of `migration/stage-1/identity-oracle/pekko-repro/observed.txt` (PR #1292, probe revision 2, and PR #1326 `5499f74`, revision 3, rows 197-210; Pekko v1.7.1 executed). A row with no probe line is **pending approval (ADR 0033 D13)**: its Pekko value is from source or the JDK, not an executed Pekko run. The probe line number is the row number.

| # | Input | Pekko outcome | Scenario (`TestParseRelative_...`) | Probe row / class |
|---|---|---|---|---|
| 1 | `foo/bar` | `["foo","bar"]` | `SeparatedNames` | **row 153**, Executed |
| 2 | `""` (empty text) | `[""]` (one empty element) | `EmptyText` | **row 154**, Executed |
| 3 | `foo//bar` | `["foo","","bar"]` | `EmptySegmentKept` | **row 155**, Executed |
| 4 | `foo/bar/` | `["foo","bar",""]` | `TrailingSlashKept` | **row 161**, Executed |
| 5 | `/foo` | `["","foo"]` | `LeadingSlashKept` | **row 162**, Executed |
| 6 | `../sibling`, `..`, `a/./b` | split only: `["..","sibling"]`, `[".."]`, `["a",".","b"]` | `DotElementsAreElements` | **rows 156, 163, 164**, Executed |
| 7 | `foo?x=1` | `["foo"]` (query dropped) | `QueryDropped` | **row 157**, Executed |
| 8 | `foo/bar#1234`; spec case `foo/bar/baz#1234` | `["foo","bar#1234"]`; spec `["foo","bar","baz#1234"]` | `FragmentIsPartOfTheLastElement` | **row 160** Executed (`foo/bar#1234`); `T/RelativeActorPathSpec.scala:38-40` SPEC for `baz#1234` |
| 9 | `a:b/c` | `None` (scheme `a`, absolute) | `SchemeIsNotRelative` | **row 158**, Executed |
| 10 | `foo/%4` | `None` (`URISyntaxException`) | `MalformedEscapeIsNone` | **row 159**, Executed |
| 11 | `foo`; a URL-encoded name (`pekko%3A%2F%2FClusterSystem%40127.0.0.1%3A7355`) | `["foo"]`; `["pekko%3A%2F%2FClusterSystem%40127.0.0.1%3A7355"]` (one element, escapes kept) | `SingleName`, `UrlEncodedName` | **rows 197, 198**, Executed; also SPEC `T/RelativeActorPathSpec.scala:28-37` |
| 12 | `pekko://sys/user/a` | `None` (`uri.isAbsolute`, `:174`) | `AbsoluteIsNone` | **row 199**, Executed |
| 13 | `foo bar`, `foo/a b`, `%`, `foo/%ZZ` | `None` for each (JDK `URISyntaxException`, JDK 11.0.32) | `URIRejectedIsNone` | **rows 200, 201, 202, 203**, Executed |
| 14 | `%2F`, `%41` | `["%2F"]`, `["%41"]` (escapes kept raw, not decoded) | `EscapesKeptRaw` | **rows 204, 205**, Executed |
| 15 | `#`, `#x`, `foo#` | `["#"]`, `["#x"]`, `["foo#"]` (one element each, `#` kept) | `FragmentOnlyAndEmptyFragment` | **rows 206, 207, 208**, Executed |
| 16 | `?` | `[""]` (the query is dropped and one empty element remains) | `LoneQueryIsEmptyElement` | **row 209**, Executed |
| 17 | `ä` | `["ä"]` (non-ASCII accepted) | `NonASCIIAccepted` | **row 210**, Executed |
| 18 | no lookup | `idSeq` unchanged over rows 1-17, no System constructed | `NoLookupNoActor` | Go-only; **pending approval (D13)** |
| 19 | contract and README, no parity claim | | `pekko-actor/README.md`, manifest | review |

The probe was extended as the Owner decided on 2026-11-04 (point 1b). Every row above has an executed line except row 18 (Go-only), row 19 (review, no behavior) and one input: the spec case `foo/bar/baz#1234` of row 8 has no probe line (row 160 executes `foo/bar#1234`), so it stays **pending approval (D13)**.

## Go form (not behavior)

Return shape `(elements, ok)`, the unexported default (ADR 0011; nothing outside `pekko-actor/actor` consumes it until #893), and the test names are Go form. A `java.net.URI`-compatible scan is implementation; it must reproduce the rows above, not a Go-looking subset. A text on which the Go scan and `java.net.URI` could disagree is an open divergence with a test, never an accepted one.

## Allowed files

| Path | PR |
|---|---|
| `docs/scopes/1089-parse-relative-actor-path.md` (this file) | #1286, before G2 |
| `migration/stage-1/idn-1089-parse-relative-actor-path/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | 1 |
| `pekko-actor/actor/parse_relative_actor_path.go` (new; Apache-2.0 header and exactly one `// Ported from upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/Address.scala` line; no other comment may contain the text `Ported from upstream/pekko`, `check-headers.sh`) and its `_test.go` and an internal test for `idSeq` | 2 |
| `pekko-actor/README.md` | 2 |
| `migration/stage-1/idn-1089-parse-relative-actor-path/EXECUTION-RECORD.md`, `migration/cost-log.tsv` (one row) | 3 |

Nothing under `docs/adr/`, `scripts/`, `.github/`, `go.mod`, `go.sum`; no change to `checkPathElement`, `validPathElement`, `errors.go`.

## Evidence and gates

Kit manifest rows: `T/RelativeActorPathSpec.scala:28-40`, `A/Address.scala:149-180`. Rows with a probe line are Executed (Pekko); the rows named above as having none stay "pending approval (D13)", which blocks a parity claim, not delivery. Gates (`AGENTS.md`, `pekko-actor`): `go test -count=1 ./...`, `go vet`, `gofmt -l`, golangci-lint (`gocyclo` <= 10), coverage >= 85%, diff coverage >= 85%, `scripts/check_deps.sh`, `check-headers.sh`, `make ci-fast`. `-race` only on explicit request.

## Delivery, size, tasks

Draft PRs, body `Refs #1089`, Conventional Commits, no merge. Forecast (estimate): PR 1 about 150 lines, PR 2 about 250, PR 3 about 70; each under 400. Tasks: T1 kit package; T2 parser and scenarios (RED then GREEN); T3 README; T4 execution record with gates bound to the final SHA.

## Decisions for G2

1. The amendment (#1324) is on develop as `537002a` (done).
2. Go form: `(elements, ok)` with `ok == false` as Pekko's `None`, unexported by default.
3. Extend the Pekko probe for rows 11-13 and the extra inputs: done (rows 197-210, PR #1326, `5499f74`).
4. #1089 labels (after #1324 merged); the issue text (AC-R1-2, R1, R2) still reflects the older reading and is edited only on the Owner's word.

## Risk

`Risk: none`: a new function with no change to existing behavior.
