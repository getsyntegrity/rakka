# Scope: Navigate actor paths by parent and child (#852, IDN-04)

Status: **G2 approved by the Owner at 76810743150fea3a18be5e72bc39f2a5e627b5f6; the commits after that head (head ranges: `76810743150fea3a18be5e72bc39f2a5e627b5f6`..`fe93c8ca41d80d2ade9cb000e0a1bcbb6175072b`..`48270153b89368cec85a56385407556ad2250411`..this delta) are documentation only: no scenario, criterion text, risk, decision, symbol, zero-value rule, allowed file or command changed; the exhaustive list of changed hunks is in the approval record ("Changes after the approved head"), and the PR-split and kit-package text is a proposal, not an Owner decision.** No implementation. Written on 2026-10-29 against `origin/develop`
`7840e8893305b0817b76d7b7f6e7d209afe0b8d7`. Pekko oracle: `upstream/pekko` v1.7.1, `9a54d1294c3ac7821bbed3c940769c3d5027d1ff`
(paths relative to `upstream/pekko/actor/src/main/scala/org/apache/pekko/actor/`, abbreviated `M/`). The Pekko lines `M/ActorPath.scala`
72-75, 188, 203-204, 209, 286, 290-293 and 328-335 and the Go lines cited were re-opened at those SHAs on 2026-10-29. Pekko statements are
**Source** unless a row is marked **Executed** (see "Differential oracle"); no JVM run of Go and no parity is claimed. The issue is not edited and no label changes before G2. Every name, split,
path and domain below is a **proposal for the Owner**, except the six decisions recorded in the Owner approval record. The names `Parent` and `Descendant` are named in the Owner's decision 1; the Go signatures stay as in this scope.
ADR 0031's acceptance "does not accept ... any API name" (`docs/adr/0031-local-address-and-actor-path-values.md:9-10`).

**Kit run required (ADR 0017/0024, `AGENTS.md`).** `Parent` and `Descendant` are a **port/adaptation** of the `ActorPath` members
`parent` (`M/ActorPath.scala:188`, root `:286`) and `/(Iterable)` / `descendant` (`:203-204`, `:209`). `AGENTS.md` says tl-plan marks such a
scope "kit run required"; ADR 0017 "Delivery units" requires a manifest, RULEBOOK and RUNBOOK per unit. Precedent #1008 shipped no package
and is **not** an exception: it is not used as a reason to skip the kit here.

Revision (delta 6 after G2): attribution of the QA verdicts made precise, scratch references removed, head ranges explicit; no contract change. Delta 5: the approval record lists every change after the approved head, uses an explicit head range, marks the PR split and kit text as proposed, and cites the K and C QA comments.
Revision (documentation delta 4 after G2): executed rows re-read against probe revision 2; no contract change. Criterion 5 and R6 now ask the
future README to say "Go not run on a JVM; Pekko rows executed in a disposable copy (see the oracle section); no parity claim" instead of "Source only, no JVM run"; this is a README-wording requirement on the code PR, not a contract change.
Earlier deltas after G2 (documentation only): status wording, evidence upgrade, #812 cross-reference, `run-pekko.sh`, `~/.ivy2`, PR 2 split (S10), Judge.scala. Revision 3 (2026-10-29, round-2 QA on `af434c2`): coverage commands, #812 citations (head `c8d5a6f`), PR 1a/1b split, S11 domain, `errors.go` comment, S10 label and oracle wording corrected.

Revision vs `fb05860` (2026-10-29), after independent QA: S11 (stopped-actor path) dropped, scenarios renumbered S1-S11; `errors.go` and
spec citations corrected; hand-off to #812 cites rows by input text and states the sequencing; golangci-lint install and `$ROOT` forms copied
from `AGENTS.md`; `ActorSelectionSpec.scala:271` limited to the call; differential-oracle section added.

## Owner approval record (G2)

Gate G2 for #852 was approved by the Owner on 2026-10-29 at head `76810743150fea3a18be5e72bc39f2a5e627b5f6`. Decisions, verbatim:

1. "Entregar Parent y Descendant, con sus justificaciones individuales ADR 0011."
2. "Root queda fuera de esta unidad; no queda excluido del alcance general."
3. "Kit requerido según AGENTS.md."
4. "Parent de la ruta cero devuelve la ruta cero."
5. "Descendant sobre ruta cero devuelve ruta cero y ErrInvalidPath, incluso sin segmentos o con todos vacíos. Sobre una ruta válida, cero segmentos o todos vacíos devuelven el receptor sin error."
6. "Trasladar la comparación con parsing a #812, conservando trazabilidad y limitándola al dominio aceptado por ambas operaciones."

Scope limits, verbatim: "Esta aprobación cubre #852. No acepta las addenda #1187/#1186, no concede G2 a las otras unidades y no autoriza merges ni cierre de issues."

| Owner decision | Scope text it approves | Differs? |
|---|---|---|
| 1 | "Decisions for G2" 2 and "Export justification" (both symbols, individual rows) | no |
| 2 | "Decisions for G2" 3; "What this does not deliver" (`Root()`) | no |
| 3 | header "Kit run required"; "Decisions for G2" 4 (kit run itself) | no |
| 4 | "What this delivers", zero-value rule (`Parent(zero)`); "Decisions for G2" 5 | no |
| 5 | zero-value rule (`Descendant`); S5-S7; "Decisions for G2" 5 | no |
| 6 | "Hand-off to #812"; "Decisions for G2" 1 | no |

Not covered by an Owner decision, so still proposals: the kit package name and the PR split 1a/1b/2/3 ("Decisions for G2" 4, rest).

Changes after the approved head. Range: from the G2-approved head `76810743150fea3a18be5e72bc39f2a5e627b5f6` to `fe93c8ca41d80d2ade9cb000e0a1bcbb6175072b`, then to `48270153b89368cec85a56385407556ad2250411`, then to this delta (the commit that adds this sentence); the commits in that range include `72e8110`. Source of the list: `git diff 76810743150fea3a18be5e72bc39f2a5e627b5f6 HEAD -- docs/scopes/852-actor-path-navigation.md`
read hunk by hunk. The list is exhaustive; none of it is an Owner decision, and the contract (scenarios, criteria, risks, decisions, symbols, zero-value rule, allowed files, commands) is unchanged:
1. Status line: rewritten from "Draft for gate G2; not approved" to the approval wording (rewritten again in later deltas).
2. Header paragraph: Pekko statements are Source unless a row is marked **Executed**; "no JVM run" became "no JVM run of Go"; "every name, split, path and domain is a proposal" now excepts the six decisions, and names `Parent` and `Descendant` as named in decision 1.
3. Revision and delta lines at the top (including those of every later delta).
4. This Owner approval record (new section, with the decision table and the list you are reading).
5. Evidence cells of criteria 1, 2a and 2a (negative): READ replaced by Executed Pekko rows.
6. Criterion 5 and risk R6: the README wording required of the code PR changed from "Source only, no JVM run" to "Go not run on a JVM; Pekko rows executed in a disposable copy (see the oracle section); no parity claim" (a wording requirement, not a scenario or symbol change).
7. "Hand-off to #812" sequencing sentence: rewritten as a cross-reference to the amended #812 scope.
8. "Differential oracle" section: heading and body rewritten as executed evidence; its last sentence now says what the kit execution record is proposed to reference.
9. "Export justification" heading and State cells: now record Owner decision 1.
10. "Decisions for G2": one pointer line to this record.
11. PR-split proposal text: PR 1a size is an estimate; S10 is assigned to the later of the two PR 2 members; PR 3 is described as referencing the recorded Pekko run.
12. Attribution wording: the K and C comments are described as subagent QA verdicts relayed by the orchestrating session; the Owner's working-session allowance is reported as having no repository or GitHub trace.
13. Scratch references removed (none were left in the file at the delta-6 base; the record's scratch path is removed in PR #1296).
14. Head ranges: self-referential wording replaced by explicit head ranges.
Criterion 2b is unchanged as the Owner decided (decision 6: moved to #812, preserving traceability). The final shape of that comparison is an Owner decision pending in #812's G2; it is not deduced here.
The PR split 1a/1b/2/3 and the kit package name are **proposed**, not covered by the six decisions. The Owner's allowance to split into PRs of at most 400 authored lines keeping criteria and traceability (a contract-changing split needs a new G2) was reported by the Owner in the working session on 2026-10-29; no repository or GitHub trace. The split actually used is K #1294 (kit package), C #1295 (code, stacked on K) and R #1296 (record, stacked on C); this records how the proposal was executed and is not an Owner decision. QA verdicts of the implementation PRs are subagent QA verdicts relayed by the orchestrating session and published as PR comments (not human reviews, not Owner decisions, not merge approvals): K https://github.com/getsyntegrity/gopekko/pull/1294#issuecomment-6502138689 and C https://github.com/getsyntegrity/gopekko/pull/1295#issuecomment-6502139469. The #1284 verdicts (`fe93c8ca` APPROVE_WITH_MINORS; `48270153` APPROVE_WITH_MINORS) and the #1292 verdicts (`d44c80cd` APPROVE_WITH_MINORS; `d4ac1c47` APPROVE) were reported to the orchestrator; the #1292 ones are published at https://github.com/getsyntegrity/gopekko/pull/1292#issuecomment-6502420169; the #1284 ones will be published as a comment on #1284 after this delta, bound to the head of this delta (no verdict is claimed for that head).

**Contract differences to report to the Owner:** none; no approved decision differs from the scope text.

## What this delivers

Two navigation members on the existing `ActorPath` value (`pekko-actor/actor/actor_path.go`), pure value operations that never touch a
`System`, the registry or any actor:

```go
// shape of the contract (proposed names), not the implementation
func (p ActorPath) Parent() ActorPath                                 // root is its own parent; zero stays zero
func (p ActorPath) Descendant(names ...string) (ActorPath, error)     // skips "" names; any other invalid name fails the whole call
```

- `Parent`: `RootActorPath.parent = this` (`M/ActorPath.scala:286`); for a child, the path without its last element.
- `Descendant`: `/(child: Iterable[String])` folds the names and skips empty ones (`M/ActorPath.scala:203-204`; Java form `descendant`
  `:209`). Each non-empty name passes the same validator as `Child` (`checkPathElement`, ADR 0031 P-5), so `Child` rejects an empty name
  while `Descendant` skips it (ADR 0031 P-4/P-5, `docs/adr/0031-local-address-and-actor-path-values.md:92-93`).
- **Zero-value rule (proposal, Go-only; Pekko has no zero path).** The zero `ActorPath` is "no path": `Parent(zero)` returns zero.
  `Descendant` on the zero path returns **the zero value and `ErrInvalidPath`, whatever the names** (none, only empty, or valid), as
  `Child` does (`actor_path.go:50-52`). On a **valid (non-zero)** path, no names or only empty names return the receiver with no error.
  The zero check comes first, so "skip empties" never turns an invalid receiver into a success.

## What this does not deliver

- **Parsing** of text (#812) and relative text (#1089). Nothing here reads a string; no validator is duplicated or added.
- `Root()` is **not part of this unit; its delivery is not excluded from the general scope of ADR 0031 P-4** (which lists it as a member
  to mirror). The issue's criteria do not name it and no in-repo consumer needs it now. Owner decision 3.
- Selection, wildcards, `Identify`, mailboxes, `ActorSelection` (#893/#1022 front), references and incarnation (#853), strings with
  `#uid` (#894/#1186).
- No change to `Spawn`, `Ref`, equality (`==` stays the equality, ADR 0031 P-6) or any existing exported symbol.

## Dependencies, as they really are (verified with `gh` and `git show origin/develop:` on 2026-10-29)

| Dependency | State | Effect on #852 |
|---|---|---|
| #1008 (IDN-14) | closed; PR #1011 squash `13fb9953879462adba4218114d6615dc3d3156b9` | Delivered: `ActorPath`, `NewRootPath`, `Child`, `Address`, `Name`, `Elements`, `IsRoot`, `String` (`actor_path.go:34, 49, 79, 82, 85, 97, 109`). No `Parent`, `Root` or `Descendant` exist |
| #766 (IDN-02) ADR 0031 | accepted 2026-10-24 | P-4/P-5/P-7 are consumable |
| #812 (IDN-03) | open, blocked on the grammar decision #1187 | **Not a prerequisite for the implementation.** It receives the "equals parsing" comparison (criterion 2b, "Hand-off to #812") |
| #1187 / #1186 | open proposals, no Owner decision | Decides whether the parser rejects or skips empty segments; this scope compares no empty segment against the parser |
| #1020 (GOV-02) / #1021 (GOV-03) | open, `status:ready` | The per-export justification is recorded **here** (#1020 is only an index; its form is #1021) |
| Sharing `actor_path.go` | with #812 (parser), #1089 | A shared file is not a functional dependency. The parser should live in its own file (proposal for #812's scope) |

## Contracts consumed

| Contract | Source | State |
|---|---|---|
| `ActorPath` is an immutable comparable value; `==` is equality; no uid | ADR 0031 P-4, P-6, P-7; `actor_path.go:20-30` | accepted and implemented |
| `Child` validates through `checkPathElement` (empty, `/`, `#`, character set) | `actor_path.go:63-76`, `names.go:33-57` | implemented; reused unchanged |
| Zero `ActorPath` is "no path": prints `""`, not a root, no children | `actor_path.go:26`, `Child` at `:50-52` | implemented; navigation keeps it closed (S3, S7) |
| `ErrInvalidPath` (`errors.go:30`, comment from `:28`) | the only error class | reused, no new error |

## Criteria -> scenario -> Pekko source

Scenarios go in `pekko-actor/actor/actor_path_test.go` (external package, go-specs matchers as the file already does).

| # | Criterion (issue, restated) | Scenario | Pekko source | Evidence |
|---|---|---|---|---|
| 1 | "Parent navigation at root follows the documented root rule." | S1 `root.Parent() == root`; S2 for depth 1..3, `p.Parent() == q` where `p` was built with `q.Child(x)`; S3 `ActorPath{}.Parent() == ActorPath{}` (Go-only) | `M/ActorPath.scala:286` (root), `:188` (`def parent`) | Executed (Pekko oracle): `root.parent == root` -> `true`; `(root/user/a).parent` -> `pekko://sys/user`, `.parent.parent` -> `pekko://sys/`. No spec found asserts the root rule: `SupervisorMiscSpec.scala:178` and `AskSpec.scala:178`, `:198` (`sender().path.parent`) all use child paths (specs under `actor-tests/src/test/scala/org/apache/pekko/`) |
| 2a | "Appending multiple validated segments ..." with the reconciled wording "the multi-segment form skips empty names while Child rejects an empty name" | S4 `root.Descendant("user","","a") == root.Child("user").Child("a")`; S5 on a valid path, only empty names return the receiver, no error; S6 on a valid path, no names return the receiver, no error; S7 on the zero path, no names / only empty names / valid names all return the zero value and `ErrInvalidPath` | `M/ActorPath.scala:203-204`, `:209` | Executed (Pekko oracle): `root.descendant(user,'',a)` -> `pekko://sys/user/a`; `descendant(Seq(''))` -> `pekko://sys/` with elements n=1 [""]; empty list -> same instance (`same=true`). `ActorSelectionSpec.scala:271` calls `descendant(Seq("a","b").asJava)` (call only; the selection itself is excluded) with no empty name, so it does not assert skipping |
| 2a (negative) | A bad name is rejected, nothing is half-built | S8 for `"a/b"`, `"a#1"`, `"a b"`, `"%4"`: `ErrInvalidPath`, returned value is the zero path, receiver unchanged; S9 a bad name after valid ones fails the whole call; S11 for a table of **non-empty** names, `Descendant(n)` and `Child(n)` agree on accept/reject (same validator; the empty name is outside the table because `Descendant("")` skips while `Child("")` rejects) | Go difference: Pekko splits `name#uid` (`:290-293`) and checks only `/` and `#` (`:328-335`); ADR 0031 differences (b), (d) | Go design (P-5/P-7); accepted difference with the Executed Pekko row `fromString('pekko://sys/a#5')` -> `uid=5`, elements n=1 ["a"] (see "Differential oracle") |
| 2b | "... produces the same value as parsing the complete path" | **Not testable here** (needs the parser). Moved to #812 with a shared domain; see "Hand-off to #812" | `M/ActorPath.scala:72-75` (`fromString` builds with `/(Iterable)`) | blocked by #812 / #1187 |
| 3 | "Navigation does not depend on whether an actor currently exists." | Satisfied **by construction**: neither member takes a `System`, so no public-interface scenario can fail it. S10 (identical `Parent`/`Descendant` results while a `System` runs and after it stops) is a **non-falsifiable boundary note**, not evidence for criterion 3; it would become falsifiable only if a path were backed by a registry (a future change, #853). Binding a path to a real actor is **not** here: no API yields an actor's path (`pekko-actor/README.md` ~245-248) and #853 delivers it | `M/ActorPath.scala:286, 203-204` are pure | Go design |
| 4 | Positive and negative cases, named boundary, candidate SHA and gates | S1-S11; ordering/race boundary is "none: values are immutable"; SHA and gates in the PR comments and the execution record | | review |
| 5 | Contract and traceability updated; no sibling or parity claim | `pekko-actor/README.md` path section (lines 229-249 at `7840e88`): move `Parent`/`Descendant` from "pending (#852)" to implemented, keep #812/#853 pending, restate "Go not run on a JVM; Pekko rows executed in a disposable copy (see the oracle section); no parity claim" | | review |

Tests assert every observable effect through the public interface: returned value, error class and unchanged receiver. No scenario
sleeps or starts a goroutine. A failing scenario that is not a Go defect is run on Pekko before it is classified (`AGENTS.md`).

## Hand-off to #812 (criterion 2b; proposal; the #812 file is not edited here)

Original issue criterion, quoted: **"Appending multiple validated segments produces the same value as parsing the complete path."**
Traceability stays with #852 (criterion 2b above); the assertion is delivered by #812's scope. Proposed **shared input domain**: the
comparison `parse(String(p.Descendant(names...))) == p.Descendant(names...)` holds only for a **non-zero** receiver `p` and names that are
**valid elements** (non-empty, no `/`, no `#`, valid characters or `%XX`). Empty segments are **not** compared while #1187 is open.

| # | Receiver `p` | `names` | In the shared domain? | Expected |
|---|---|---|---|---|
| D1 | root `pekko://sys/` | `("user")` | yes | parse(String) `==` `/user` |
| D2 | root | `("user","a","b")` | yes | `==` `/user/a/b` |
| D3 | `/user` | `("a")` | yes (non-root receiver) | `==` `/user/a` |
| D4 | root | `("%41")`, `("$a")`, `("*")` | yes | element kept verbatim, nothing decoded (P-8); #812 rows by content: `.../user/%41` (P6), `.../system/$a` (P10, accepted), `.../user/boom/*` (P11, accepted) |
| D5 | root | `()` | yes | `p` itself: only the plain round trip of #812 AC1 |
| D6 | root | `("a b")`, `("a#1")`, `("a/b")`, `("%4")` | no, `Descendant` fails (S8) | n/a; parser side by content: `.../user/%4` (P7, rejected), `.../user/a b` (P9, rejected), `.../user/a#123` (P12, pending #1187/#1186, not a plain reject); `a/b` is two elements in text, no #812 row |
| D7 | root | `("user","","a")`, `("")` | **no, pending #1187** | whether the parser skips (Pekko, `M/ActorPath.scala:203-204`) or rejects the empty segment |
| D8 | the zero path | any | no | `Descendant` fails (S7); `String()` is `""`, which does not round-trip (#812 P21) |

Text proposed for the #812 scope (to add to its scenarios; this PR changes no file of #812): "RoundTripDescendant: for the receivers
`pekko://sys/` and `pekko://sys/user` and the names of rows D1-D5 of the #852 scope, `parse(String(p.Descendant(names...)))` equals
`p.Descendant(names...)` and the error is nil. Rows with an empty segment are added only after #1187 decides skip versus reject. Source of
the criterion: issue #852, criterion 2." Until #812 lands, #852 does not claim the comparison.

**Sequencing, not a functional prerequisite of #852:** #852 PR 2 (code) must be integrated before #812's RoundTripDescendant scenario can compile. The #812 scope (PR #1285) says "no navigation `Parent`/`Root`/`Descendant` (#852)" and, in its file-ownership table, "#852 has no edge to #812 or #1089"; both are amended in the #812 scope (`docs/scopes/812-parse-actor-path.md`: RoundTripDescendant scenario, boundary text, order; read at `32e587ab11ef47cf86dbf973985657dd92f16ee6`). Cited by text and section title, not by line numbers or head.

## Differential oracle (ADR 0033 D13) -- executed locally; proposed location and kit record pending

No spec asserts the root rule or the skip rule, so the pinned Pekko (`9a54d1294c3ac7821bbed3c940769c3d5027d1ff`) **was run** locally and
offline in a disposable `git archive` copy, never in `upstream/pekko`, which stayed untouched: `sbt -batch -Dsbt.offline=true actor/compile`
(about 22 s, no download), then a probe (`Probe.scala`). Versions: Temurin 11.0.32, sbt 1.12.14. Provenance: the `.class` files in
`upstream/pekko/actor/target/scala-2.13/classes` are byte-identical to a fresh compile in 1892 of 1893 files (only `Version$.class` differs).
Raw rows and `Probe.scala`: revision 2 of the probe (unambiguous element printing), PR #1292 (proposed durable location `migration/stage-1/identity-oracle/pekko-repro/`; not
part of this PR). `judge/pekko-harness/Judge.scala` has no scenario for path members and was not used.

**Executed rows used by this scope** (input -> output; uid values are random, so only relationships are claimed):
- Parent: `root.parent == root` -> `true`; `root.parent` -> `pekko://sys/`; `(root/user/a).parent` -> `pekko://sys/user`; its `.parent` -> `pekko://sys/`.
- Skip empties: `root.descendant(user,'',a)` and `root./(Seq(user,'',a))` -> `pekko://sys/user/a` with elements n=2 ["user","a"];
  `root.descendant(Seq(''))` -> `pekko://sys/` with elements n=1 [""]; `root.descendant(new java.util.ArrayList[String]())` -> `pekko://sys/`, `same=true`;
  `root./("")` -> `pekko://sys/`, elements n=1 [""]. Revision 1 printed element lists ambiguously (`List()` for both `Nil` and `List("")`); these rows are re-read from revision 2.
- Root elements: `root.elements` -> n=1 [""]; `ActorPath.fromString('pekko://sys')` -> elements n=1 [""], `isRoot=true`. This is exactly Go's `ActorPath.Elements()` for the root (`[""]`, ADR 0031, `actor_path.go`): oracle and Go agree.
- Accepted difference (Go design, ADR 0031 P-4/P-5): `ActorPath.fromString('pekko://sys/a#5')` -> `uid=5`, elements n=1 ["a"], i.e. Pekko splits
  `name#uid` (Source `M/ActorPath.scala:290-293`), whereas Go `Child`/`Descendant` reject `#` (S8). Same run: `...a#x` -> `NumberFormatException`; `...a#5#6` -> `MalformedURLException`.

**Not executed, still Go design or Source only:** zero-value rows (S3, S7, and the zero cases of S5/S6), `Parent` of the zero path (Pekko has no
zero path), Go-only error-class rows (S8, S9, S11), S10, the #812 comparison, and Pekko behavior beyond the recorded rows. No JVM run of Go; no
parity claim. The kit execution record (`EXECUTION-RECORD.md`, PR 3) is **proposed** to reference the recorded run (`Probe.scala` and rows at the proposed
location) and to add the Go-versus-Pekko differential for `Parent` and `Descendant`; this is a proposal, not an Owner decision.

## Export justification (ADR 0011 owner decision 2, ADR 0033 D3) -- approved by the Owner at G2 (decision 1)

Each symbol has its own justification; neither depends on the other.

| Symbol | Concrete benefit | Consumer | Alternative | State |
|---|---|---|---|---|
| `Parent` | A diagnostic or selection step climbs one level without re-parsing `String()` | #893 (relative `..`, blocked by #1022); no consumer exists in code today | keep unexported until #893 needs it | approved by the Owner at G2 (2026-10-29, decision 1) |
| `Descendant` | The issue's criterion 2 needs a multi-segment append that skips empties and validates, in one call that fails whole | #812 tests (could use `Child` chain), #893 | ship `Parent` only; callers chain `Child` | approved by the Owner at G2 (2026-10-29, decision 1) |

Proposed row for the #1020 index (capability names only): `ActorPath` navigation | consumers #852, #893 | justification: this section.

## Kit package and migration evidence (proposal; name and rows are for the Owner)

Proposed package `migration/stage-1/actor-852-path-navigation/` (name by the pattern of `typed-956-default-stop`; **a proposal**),
with `manifest.tsv` (`source target disposition pekko_row scenario evidence note`), `RULEBOOK.md`, `RUNBOOK.md` and
`EXECUTION-RECORD.md` (commands, kit `cf91c9d`, models, inputs, outputs, manual changes, gates bound to a SHA), plus one row in
`migration/cost-log.tsv`. Proposed dispositions (ADR 0017 "Delivery units": a row moves later or to `PORT-GAP` only after reading the
body, never silently dropped; `PORT-GAP` is never used for a pending decision):

| Pekko row | Proposed disposition | Scenario / reason |
|---|---|---|
| `M/ActorPath.scala:286` `RootActorPath.parent = this` | ported | S1 |
| `M/ActorPath.scala:188` `def parent` (child case) | ported | S2 |
| `M/ActorPath.scala:203-204` `/(Iterable)` skip-empty fold | adapted: returns `(ActorPath, error)`, validates each name, zero receiver fails | S4-S9, S11 |
| `M/ActorPath.scala:209` Java `descendant(Iterable)` | adapted into the variadic `Descendant` (no Java collection in Go) | S4 |
| `M/ActorPath.scala:72-75` `fromString` uses `/(Iterable)` | moved to #812's scope | "Hand-off to #812" |
| `ActorSelectionSpec.scala:271` `descendant(Seq("a","b"))` | adapted: the call only (`descendant(Seq(...))`); the selection itself is excluded | S4 (without empties), S10 |
| Pekko tests of `ActorPath` equality/`parent` in `ActorPathSpec.scala` | the RULEBOOK enumerates them after reading the bodies; each row ported, adapted, moved or `PORT-GAP` with the reason | no row is invented here |

## Allowed files (proposal; anything else needs a stop and a report to the TL)

| Path | Edit | PR |
|---|---|---|
| `docs/scopes/852-actor-path-navigation.md` | this file (PR #1284 contains only this file) | 1a |
| `migration/stage-1/actor-852-path-navigation/manifest.tsv`, `RULEBOOK.md`, `RUNBOOK.md` | new kit package (name a proposal) | 1b |
| `pekko-actor/actor/actor_path.go` | `Parent`, `Descendant`; keep its single `Ported from` line | 2 |
| `pekko-actor/actor/errors.go` | comment-only edit at `errors.go:28-30`: `Descendant` adds a producer of `ErrInvalidPath`, while the comment says only `NewRootPath` and `Child` return it | 2 |
| `pekko-actor/actor/actor_path_test.go` | S1-S11 | 2 |
| `pekko-actor/README.md` | the path paragraph only | 2 |
| `migration/stage-1/actor-852-path-navigation/EXECUTION-RECORD.md`, `migration/cost-log.tsv` (one row) | execution record, gate evidence bound to a SHA | 3 |

Nothing under `docs/adr/`, `scripts/`, `.github/`, `CLAUDE.md`, `AGENTS.md`, `names.go`, `address.go` or #812's scope file changes.
**Proposed PR split** (stacked, each at or under 400 authored lines, count stated in the body): PR 1a = this scope (PR #1284, estimate);
PR 1b = kit docs (manifest, RULEBOOK, RUNBOOK; for comparison the typed-956 package has 13, 32 and 20 lines, so about 70 to 120 lines, to be
measured); PR 2 = code, tests, README (estimate 40 + 140 + 15 lines plus the comment edit; split tests from fix only if measured above 400,
on a behavior boundary: S1-S3 with `Parent`, S4-S9 and S11 with `Descendant`, and S10, which covers both members, with the later of the two); PR 3 = execution record (referencing the recorded Pekko run) + cost-log row (about 100). The Dev is a
separate session; the TL does not implement; QA is independent.

## Risks and coverage (Task Size rule, items 1-6)

Item 1 (stored data): no. Item 2 (security): no. Item 3 (contracts others consume): adding exported methods is additive, no existing
symbol, flag or format changes. Item 4 (concurrency): no, values are immutable. Item 5 (delivery): kit docs only, no CI, dependency or
`deps.allow` change. Item 6 (no test would catch a regression): see the last column.

| # | Risk | Coverage |
|---|---|---|
| R1 | Export surface grows by two methods without a consumer today (ADR 0011) | uncovered by tests: governance; individual rows above, Owner decision 2 |
| R2 | Zero-value rule: Go-only behavior, easy to contradict (empties skipped on a zero receiver) | S3, S5, S6, S7 |
| R3 | Shared code: `Descendant` must reuse `Child`/`checkPathElement`, not copy the validator | S11 (agreement table, non-empty names) plus the existing `Child` tests unchanged; changing `Child` is a stop |
| R4 | Unrequested behavior change in existing exports | existing `actor_path_test.go` and `names` tests pass unchanged; new methods only, so no differential run against a base copy is needed |
| R5 | A bad name leaves a half-built or shared value | S8, S9 (zero value returned, receiver unchanged) |
| R6 | README overclaims parity or lists `Root`/parsing as done | uncovered by tests: PR 2 review against the module README contract ("Go not run on a JVM; Pekko rows executed in a disposable copy (see the oracle section); no parity claim") |
| R7 | #1187 decides that the parser rejects empty segments while `Descendant` skips them | uncovered: pending #1187; the shared domain excludes empties (D7) |
| R8 | Kit docs diverge from the code (rows missing) | uncovered by tests: QA checks the manifest against S1-S11 |

Risk: none of items 1-6 applies (additive exports, `Child`, `==` and the representation untouched); the residual risks R1, R6-R8 are
uncovered by tests and listed above.

## ADR impact and contradictions (ADR 0033 D5)

1. **Citation in the issue.** The reconciliation cites `ActorPath.scala:205` for the skip rule. Line 205 is blank; the rule is
   `:203-204` (`:209` for `descendant`). This scope uses the verified lines; the issue is not edited.
2. **Criterion 2 versus P-5.** The comparison with the parsed form is only true for text the parser accepts and only once #812 exists.
   Proposed wording: "Descendant produces the same value as chained `Child` calls, skipping empty names; equality with the parsed form
   is asserted by #812 over the shared domain."
3. **Dependency typing in the issue.** Its body lists #812 untyped; its reconciliation table says `depends_on` "only for AC2's comparison".
   Under the proposed wording #812 is a consumer, not a prerequisite.
4. **ADR 0031 P-4 versus the delivered subset.** P-4 lists `Parent`, `Root`, `Descendant`; #1008 delivered a subset. This unit delivers
   `Parent` and `Descendant`; `Root` is not part of it (Owner decision 3).
5. **Kit run.** Resolved in this revision: required (header). #1008 is not a precedent for skipping it.

## Verification (local; remote CI is not run)

Affected module: `pekko-actor` only (`scripts/deps.allow` and imports unchanged). From the repository root unless noted:
- `cd pekko-actor && go test -count=1 ./...` (unit tests; network isolation per `AGENTS.md`: the Linux `unshare -rn` form is not
  available on this macOS machine, so use the container variant or state the gap in the gate comment).
- `gofmt -l pekko-actor`; `.github/scripts/check-headers.sh` (the ported file keeps exactly one `Ported from` line);
  `scripts/check_deps.sh`.
- Complexity: install `GOBIN=$PWD/bin go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@$(<.golangci-version)`, check
  `bin/golangci-lint version` (it must report the `.golangci-version` tag built with the `.go-version` Go), then
  `cd pekko-actor && ../bin/golangci-lint run --config ../.golangci.yml ./...` (gocyclo <= 10, no `//nolint`).
- Coverage (from the repository root; `export ROOT="$PWD" COV="$(mktemp -d)"`), the `AGENTS.md` "Go" form: `scripts/each_module.sh bash -c
  'set -euo pipefail; go test -count=1 -covermode=atomic -coverprofile="$COV/$("$ROOT/scripts/check_coverage.sh" --profile-name "$MODULE_PATH")" ./...'`
  (the profile name encodes the Go module **path**, e.g. `github.com/getsyntegrity/gopekko/pekko-actor`, not the directory name), then
  `scripts/check_coverage.sh "$COV" 85`. The gate iterates **every** workspace module (`scripts/list_modules.sh`) and fails any without a
  profile, so a profile for `pekko-actor` alone makes the gate fail on the others; either produce all profiles through `each_module.sh`, or read only
  the `pekko-actor` row of the gate output and say so in the gate comment. Cobertura per module: `p="$COV/$(scripts/check_coverage.sh
  --profile-name github.com/getsyntegrity/gopekko/pekko-actor)"; go run github.com/boumenot/gocover-cobertura@v1.5.0 <"$p" >"${p%.out}.xml"`, then
  `pipx run --spec diff-cover==11.0.0 diff-cover "$COV"/*.xml --compare-branch=origin/develop --fail-under=85` on that report.
- `make ci-fast` (what `AGENTS.md` says CI runs: gofmt, build and selected fast unit tests); remote CI itself is not run.
- `make parity-guard` is **not** applicable (it checks the persistence adapters' manifest); the README is checked by review against the
  module README contract (no script exists).
- Running tests and lint over every module is not required (no other module imports a new symbol); but the coverage gate above does
  read every module's profile, which is why the coverage step goes through `each_module.sh`.
- `-race` is not run without the Owner's explicit authorization; no remote CI.

## Decisions for G2 (all recommendations are proposals)

See the Owner approval record for the decisions taken (the list below is the historical proposal list).

1. Criterion 2b: **move** the "equals parsing" assertion to #812 with the shared domain above (recommended; unblocks #852) or **keep** it
   and block #852 on #812.
2. Approve `Parent` and `Descendant` with their individual justifications, or ship `Parent` only; confirm the names.
3. `Root()`: not part of this unit (recommended), or add it.
4. Kit package name `migration/stage-1/actor-852-path-navigation/` and the PR split 1a/1b/2/3 (the kit run itself is required, not a decision).
5. Zero-value rule: `Parent(zero)` is zero; `Descendant` on the zero path returns the zero value and `ErrInvalidPath` even with no or only
   empty names; on a valid path, no or only empty names return the receiver with no error. Confirm.
