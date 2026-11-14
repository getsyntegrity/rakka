# VALIDATION RECORD: ADR 0019, Journal SPI and sequence contract (#211)

Purpose: one record of what ran for this documentation-only candidate. It claims nothing the commands below did not show.
The plan is in `EXECUTION-PACKAGE.md`, written before this record and not edited to match it.

Status note (2026-10-17): the Owner accepted ADR 0019 in its contractual scope after these checks; the acceptance changed no check result.

Scope change note (2026-10-17): the Owner then removed tenancy from the contract (goPekko has no tenancy module). ADR 0019 was revised accordingly and five anchor rows (`PRS`) were added for the persistence identity and plugin selection; the checks below were re-run on the revision and give `anchor_rows=111 citations_checked=176 failures=0`. The earlier results in this record describe the text before that revision.

**No Pekko oracle (sbt, TCK) was run and no parity claim is made.** The ADR reads Pekko v1.7.1 source and tests and
proposes a contract, part of it an extension. **The migration kit was not executed**: no kit command, no manifest, no
port, no cost row (`migration/cost-log.tsv` is unchanged). **No persistence judge exists.** No review of any kind and no
subagent was used. Remote CI was not observed when this record was written.

Candidate: branch `docs/adr-0019-journal-spi-211`, base `origin/develop` `b032bba288af19333b372ef33cc1420a9166df07`.
Run on darwin/arm64, 2026-10-17. The gates below were run again after the documentation revision of the same day
(case-by-case upstream reading, Owner decisions, Q2 to Q6 refinement); that revision also edits
`docs/analysis/ana-003-journal-recovery.md` (J1) and adds 20 anchor rows. The execution package was not regenerated.
The revision read the following additional upstream files, without running anything: `InmemJournal.scala`,
`LeveldbStore.scala`, `persistence-testkit` `EventStorage.scala`, `TestKitStorage.scala` and
`PersistenceTestKitPlugin.scala`, `AsyncWriteJournalResponseOrderSpec.scala`.

## 1. Gates run (observed)

| Gate | Command | Result |
|---|---|---|
| Upstream citations | the script of the appendix, run from outside the tree: `python3 -I check_per.py . docs/adr/0019-journal-spi-and-sequence-contract.md migration/evidence/per-001/SOURCE-ANCHORS.md` | `anchor_rows=106 citations_checked=172 failures=0` after the revisions of 2026-10-17 (it was 86 and 117 at the first commit): every anchor line contains its text and every `ALIAS:line` citation of the ADR has an anchor row |
| Script is not vacuous | the same script on a file containing `AWJ:999` and `TCK:5` | exit 1, two `CITE FAIL` lines |
| Dependency direction | `scripts/check_deps.sh` | exit 0 |
| ADR signatures | `scripts/check-adr-signatures.sh` | exit 0 |
| Judge matrix | `python3 -I scripts/check-judge-matrix.py` | exit 0 (unchanged by this diff) |
| Repository paths | every repo path in backticks in the ADR and the package exists (scripted `test -e`) | no missing path except this record, created after the check and present now |
| ASCII | a search for non-ASCII bytes over the ADR and `migration/evidence/per-001` | none |

What the citation script proves and what it does not: it proves that a cited line exists and contains the anchored text,
for every citation written as `ALIAS:line`. It does not prove that the ADR reads the line correctly. A review of the
previous ADR of this series found that its script skipped citations written as a bare line number; this ADR writes every
citation with its alias, so none is skipped. Statements marked `[Inference]` in the ADR are reasoning from the cited
code and no test asserts them.

## 2. Gates that do not apply

`go test -race`, coverage, `golangci-lint`, `gocyclo`, `go vet`, `tidy`, the mutation campaigns and the parity judge
exercise Go code or the judge harness. The diff contains no `.go`, `go.mod`, `judge/` or `scripts/` change. They were not
run and no result is claimed. The conformance cases proposed in the ADR have no test yet.

## 3. Review

No native review was started and none is claimed. The unit adds Markdown only. A review of this ADR is a separate
decision of the owner.

## 4. Known limits

- ANA-003 was used as an input and its journal claims were re-read against the source. One refinement resulted (the
  rejection gap, see the ADR). Other ANA-003 claims were not all re-read; the ADR cites only what it re-verified.
- Reference plugin behavior (in-memory, LevelDB) was read for the lines cited only. Plugins outside the pinned tree were
  not read.
- ADR 0016 is used as `proposed`; the ADR lists exactly which points depend on it.

## Appendix: citation script

Extract the block to a file outside the repository and run it with `python3 -I`. It reads `upstream/pekko` read-only
and writes nothing.

```python
import pathlib
import re
import sys

# usage: python3 -I check_per.py <repo-root> <adr.md> <SOURCE-ANCHORS.md>
# 1. Every anchor row (alias, file, line, text) must name an existing file under upstream/pekko whose line
#    contains the text, and each alias must map to one file.
# 2. Every ALIAS:line citation in the ADR must have an anchor row with that alias and line.
# Exit 1 on any failure.
root = pathlib.Path(sys.argv[1])
adr = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")
anchors = pathlib.Path(sys.argv[3]).read_text(encoding="utf-8").splitlines()
base = root / "upstream/pekko"

bad = 0
alias_file = {}
anchored = {}
cache = {}
row = re.compile(r"^\| ([A-Z]{3}) \| (\S+) \| (\d+) \| `(.*)` \|$")
for ln in anchors:
    m = row.match(ln)
    if not m:
        continue
    alias, rel, num, text = m.group(1), m.group(2), int(m.group(3)), m.group(4).replace("\\|", "|")
    if alias_file.setdefault(alias, rel) != rel:
        print(f"ALIAS CONFLICT {alias}")
        bad += 1
    if rel not in cache:
        fp = base / rel
        cache[rel] = fp.read_text(encoding="utf-8").split("\n") if fp.is_file() else None
    src = cache[rel]
    if src is None or num > len(src) or text not in src[num - 1]:
        print(f"ANCHOR FAIL {alias}:{num} expected {text!r}")
        bad += 1
    anchored.setdefault(alias, set()).add(num)

cites = re.findall(r"\b([A-Z]{3}):(\d+)\b", adr)
known = set(alias_file)
checked = 0
for alias, num in cites:
    if alias not in known:
        continue  # not an alias of this ADR (for example an unrelated token)
    checked += 1
    if int(num) not in anchored[alias]:
        print(f"CITE FAIL unanchored {alias}:{num}")
        bad += 1
print(f"anchor_rows={sum(len(v) for v in anchored.values())} citations_checked={checked} failures={bad}")
sys.exit(1 if bad else 0)
```
