#!/usr/bin/env python3
"""Applies assertion-by-assertion comparison results (TSV: row_id status covered missing cases open_q note) to the
inventory and rebuilds cases/coverage.tsv (case -> rows). Usage: apply_comparison.py <compare-dir> [--apply]"""
import csv, glob, json, os, re, sys, collections
csv.field_size_limit(10**9)
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
os.chdir(ROOT)
sys.path.insert(0, os.path.join(ROOT, "tools", "package"))
from closures import closed_without_implementation
src = sys.argv[1]; APPLY = "--apply" in sys.argv
cmp_rows = {}
for f in sorted(glob.glob(os.path.join(src, "*.tsv"))):
    for i, l in enumerate(open(f, encoding="utf-8").read().split("\n")):
        if i == 0 or not l: continue
        x = l.split("\t"); assert len(x) == 7, (f, i, len(x))
        cmp_rows[x[0]] = dict(zip(["status", "covered", "missing", "cases", "open_q", "note"], x[1:]))
path = "docs/inventory/actor-classic.tsv"
lines = open(path, encoding="utf-8").read().split("\n")
header = lines[0].split("\t"); ix = {h: k for k, h in enumerate(header)}
changed = collections.Counter()
for k, ln in enumerate(lines[1:], 1):
    cols = ln.split("\t")
    if len(cols) != len(header) or cols[0] not in cmp_rows: continue
    r = cmp_rows[cols[0]]
    row = dict(zip(header, cols))
    if row["validation"] == "documented" or closed_without_implementation(row):
        print(f"kept (closed by Owner decision without own implementation): {row['id']}")
        continue
    old = (cols[ix["impl"]], cols[ix["validation"]])
    cols[ix["impl"]] = r["status"]
    cols[ix["validation"]] = "pending" if r["status"] == "pending" else ("oracle+spec" if cols[ix["validation"]] == "oracle+spec" else "oracle")
    note = f"comparado con casos 2.0.0: " + (f"cubre {r['covered']}" if r["covered"] != "-" else "sin aserciones cubiertas")
    if r["missing"] != "-": note += f"; falta {r['missing']}"
    if r["open_q"] != "-": note += f"; Q abierta {r['open_q']}"
    if r["cases"] != "-": note += f"; casos {r['cases']}"
    note = note.replace("\t", " ")
    base = re.sub(r"(^| \| )comparado con casos 2\.0\.0:.*$", "", cols[ix["notes"]]).rstrip()
    cols[ix["notes"]] = (base + " | " if base.strip() not in ("", "-") else "") + note
    changed[(old[0], r["status"])] += 1
    lines[k] = "\t".join(cols)
print("impl changes (old -> new):", dict(changed))
if APPLY:
    open(path, "w", encoding="utf-8").write("\n".join(lines))
    areas = sorted({r.split("-")[1] for r in cmp_rows})
    open("docs/inventory/compared-areas.txt", "w", encoding="utf-8").write("# Areas of the Classic inventory whose rows were compared assertion by assertion with the recorded cases\n" + "\n".join(areas) + "\n")
# coverage: case -> rows
cases = []
for f in sorted(glob.glob("cases/*.jsonl")):
    for l in open(f, encoding="utf-8"):
        cases.append((os.path.basename(f), json.loads(l)["name"]))
cited = collections.defaultdict(set)
for rid, r in cmp_rows.items():
    if r["cases"] == "-": continue
    for c in r["cases"].split(","):
        cited[c.strip()].add(rid)
group = lambda f, n: (f, re.split(r"[-_]\d|\d", n)[0].rstrip("-_"))
gmap = collections.defaultdict(set)
for f, n in cases:
    gmap[group(f, n)] |= cited.get(n, set())
out = ["case\tfamily\trows\tbasis"]
unmapped = 0
for f, n in cases:
    direct = cited.get(n)
    rows = direct or gmap[group(f, n)]
    basis = "cited" if direct else ("same-group" if rows else "none")
    unmapped += not rows
    out.append(f"{n}\t{f[:-6]}\t{','.join(sorted(rows)) or '-'}\t{basis}")
print(f"cases: {len(cases)}, without rows: {unmapped}")
if APPLY: open("cases/coverage.tsv", "w", encoding="utf-8").write("\n".join(out) + "\n")
