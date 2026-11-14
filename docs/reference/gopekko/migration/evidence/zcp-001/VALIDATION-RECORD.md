# VALIDATION RECORD: ADR 0018, persisted data contract and zero-custody scope (#391)

Purpose: one record of what ran for this documentation-only candidate. It claims nothing the commands below did not show.
The plan is in `EXECUTION-PACKAGE.md`, written before this record and not edited to match it.

**No Pekko oracle (sbt, TCK) was run and no parity claim is made.** The ADR reads Pekko v1.7.1 source and tests and
proposes an own-extension contract. **The migration kit was not executed**: no kit command, no manifest, no port, no cost
row (`migration/cost-log.tsv` is unchanged). **Remote CI did not run** when this record was written; this record does
not replace it.

Candidate: branch `docs/adr-0018-persisted-data-contract-391`, base `origin/develop`
`c4d44b701415df3226be98b448b861339270e6d6`. Run on darwin/arm64, 2026-10-17. One agent, no subagents, no review round.

## 1. Gates run (observed)

| Gate | Command | Result |
|---|---|---|
| Upstream citations | `python3 -I migration/evidence/zcp-001/check-citations.py . docs/adr/0018-persisted-data-contract-and-zero-custody-scope.md` | `anchors=88 citations=93 failures=0`: every anchor line contains its text and every cited line is anchored |
| Dependency direction | `scripts/check_deps.sh` | exit 0 |
| ADR signatures | `scripts/check-adr-signatures.sh` | exit 0 |
| Judge matrix | `python3 -I scripts/check-judge-matrix.py` | exit 0 (Go campaign detected 18/18, benign survived 1/1; unchanged by this diff) |
| Headers | `.github/scripts/check-headers.sh` | exit 0 (33 ported files, 134 Go files scanned) |
| Repository paths | every repo path in backticks in the ADR exists (scripted `test -e`) | no missing path |
| ASCII | `rg -nP '[^\x00-\x7F]'` over the ADR and `migration/evidence/zcp-001` | none |
| Pins | `git rev-parse HEAD` equals `origin/develop`; `git submodule status` | Pekko `9a54d12...`, kit `cf91c9d...`; `reference/goakt` not initialized and not used |
| Numbering | `docs/adr/` listing; `gh pr list --state open` | 0018 free; no open pull request |

The citation gate proves that a cited line exists and contains the anchored text. It does not prove that the ADR
reads the line correctly.

## 2. Gates that do not apply

`go test -race`, coverage (85%), `golangci-lint`, `gocyclo`, `go vet`, `tidy`, `govulncheck`, the mutation campaigns,
the parity judge and stress runs exercise Go code or the judge harness. The diff contains no `.go`, `go.mod`, `judge/`
or `scripts/` change. They were not run and no result is claimed. The invariants proposed in the ADR (sentinel absence
in logs and errors, byte ownership, the closed field list G1) have no scenario or test yet.

## 3. Risk assessment

- `gentle-ai review assess --base-ref origin/develop --committed-only` first returned `unassessable` (treated as high)
  because the worktree has an untracked `odd/` directory that is not part of this work and was not touched. It was
  re-run excluding untracked files with the inventory the tool printed.
- The result is **risk `medium`, `review_due: true` (`slice_budget_reached`)**, reason `executable_change` on
  `migration/evidence/zcp-001/check-citations.py` and `source-anchors.tsv`. The tool classifies every non-Markdown file
  as executable; the ADR, the package and this record are Markdown. Moving the script to text, removing its shebang and
  renaming the anchors file were tried and did not change the tier, so they were not kept.
- **No native review was started and no consent envelope was issued.** The request for this unit was to open no
  review round on its own initiative. The tier above is therefore unreviewed, not passed. Starting the review is the
  owner's choice: `gentle-ai review status --cwd . --contract gentle-ai.review-integration/v2 --agent claude-code --next-transition --base-ref origin/develop --committed-only`.

## 4. Review

None. No independent reviewer and no second pass were run, as requested. Statements of the ADR about Pekko were
checked only by the citation gate above, not by a second reader's judgment of what each line means.

## 5. Revision after the Owner's reading of the first draft

The first draft presented a serialized payload at the SPI (A1) as an adaptation required by Go, deferred event adapters
as a language consequence, called identifiers universally non-sensitive, stated byte ownership in a way that
contradicted its own aliasing test, and implied that type registration protects against sensitive data. The revision:
compares a typed boundary with an injected narrow contract against a serialized boundary and recommends the typed one
(Pekko's shape); keeps the adapter manifest separate from the serializer manifest and treats event adapters as pending
scope with the unmigrated effect stated, distinct from upcasters; separates the framework's technical format from
application-chosen content and leaves identifier exposure in logs to an Owner question; rewrites byte ownership as a
borrow-for-the-call rule with explicit completion points; and states that registration and serializers are
configuration mechanisms that do not prove absence of sensitive data. Five anchors were added (88 in total).
The same gates were re-run after the revision (citation gate `anchors=88 citations=93 failures=0`, `check_deps.sh` exit 0,
ASCII and path checks clean). No new review, no kit run and no JVM, `sbt` or TCK run took place.

## 6. Revision after the Owner's decisions on Q1 to Q8

The Owner adopted the recommended options (Q1 T, Q2 b, Q3 a, Q4 a, Q5 a, Q6 b, Q7 a, Q8 a). The ADR now records them,
states the covered and uncovered diagnostic formats, no longer treats the tenant id grammar as proof of harmlessness,
keeps event adapters, adapter manifest and serializer manifest separate from upcasters, rewrites H-Z3 from upstream
evidence (the only metadata producer in the tree is Replicated Event Sourcing in `persistence-typed`; the classic
`Eventsourced` has none), and resolves H-Z5 with a documentary sketch (interface, types, owning package, import graph,
injection point).

- Added `PTY` prefix to the citation script and three anchors (91 in total). Gate: `anchors=91 citations=98 failures=0`.
- H-Z5 evidence: a throwaway Go build outside the repository (a consumer-owned interface with builtin-only signatures,
  an implementation package, a composition root) compiled, and `go list -deps` showed no import edge between the
  interface package and the implementation package. It is not committed and does not test goPekko code. The sketch in
  the ADR is illustrative and was not compiled as written.
- Not verified: the sketch against the repository's dependency check, because `pekko-persistence` does not exist.
- The ADR names `pekko-persistence/spi` as a future module; the path check reports it missing on purpose.
- Re-run: `check_deps.sh` exit 0, `check-adr-signatures.sh` exit 0, `check-judge-matrix.py` exit 0, ASCII clean.
- No new review, no kit run, no JVM, `sbt` or TCK run, no campaign, no dependency change. The issue relations of #391 and
  #211 are unchanged and #391 is not closed.

## 7. Acceptance of the conceptual scope

The Owner accepted the conceptual contract of ADR 0018 and decisions Q1 to Q8 on 2026-10-17, with these precisions:
identifiers are omitted by default and exposed only by explicit opt-in, with no hashing as implicit protection (O1); O2 to
O5 are delegated to #211, #197, TEN-003 and PER-007 and do not block the contract; the Go sketch is illustrative; and H-Z5
shows conceptual viability, not an exclusive architecture or a permanent builtin-only signature. The ADR status is now
`accepted` for that scope only. Changes made: status block and scope of acceptance; masking removed in favor of omission;
the claim that generics force one event type per journal removed (`any` is a proposed mapping); H-Z5 reworded; a
"Decisions delegated" table replaces "Decisions still open".

- Gates re-run: citation gate `anchors=91 citations=98 failures=0`, `check_deps.sh` exit 0, `check-adr-signatures.sh`
  exit 0, `check-judge-matrix.py` exit 0, ASCII clean. The path check reports only the branch name and the future module
  `pekko-persistence/spi`, both expected.
- No agent, review, kit run, JVM, `sbt` or TCK run, or campaign. The relations of #211, #391 and other issues are unchanged
  and #391 is not closed. ADR 0016 remains `proposed`.
- Remote checks reported by the Owner on `6247331` were `go` and `label`; this session did not independently see
  which checks they were. The final SHA is in the pull request.
