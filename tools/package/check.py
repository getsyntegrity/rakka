#!/usr/bin/env python3
"""Coherence checks for the migration documentation package. Exit code 1 on any failure."""
import csv, glob, os, re, sys, collections
csv.field_size_limit(10**9)
ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
os.chdir(ROOT)
R = lambda p: list(csv.reader(open(p, encoding="utf-8", newline=""), delimiter="\t", quoting=csv.QUOTE_NONE))
fail = []
def check(cond, msg):
    if not cond: fail.append(msg)

rows = {}
for f in ("classic", "typed"):
    t = R(f"docs/inventory/actor-{f}.tsv")
    for x in t[1:]:
        check(len(x) == 14, f"{f}: row {x[0]} has {len(x)} fields")
        check(x[0] not in rows, f"duplicate id {x[0]}")
        rows[x[0]] = dict(zip(t[0], x))
children = collections.defaultdict(list)
for i in rows:
    if "." in i: children[i.rsplit(".", 1)[0]].append(i)
for i, r in rows.items():
    for tok in re.findall(r'A[CT]-[A-Z]+-\d+(?:\.\d+)?', r["depends_on"]):
        check(tok in rows or tok in children, f"{i}: unresolved dependency {tok}")
    check(r["analysis"] in ("extracted", "listed"), f"{i}: bad analysis")
    check(r["impl"] in ("pending", "partial", "done"), f"{i}: bad impl")
    if r["analysis"] == "listed": check(r["notes"].strip() not in ("", "-"), f"{i}: listed without reason in notes")
# decisions
reg = {x[0]: x for x in R("docs/decisions/register.tsv")[1:]}
ALIAS = {"Q-CFG-1", "Q-CFG-2"}
for p in glob.glob("docs/inventory/contracts/*.md") + ["docs/inventory/actor-classic.tsv", "docs/inventory/actor-typed.tsv"]:
    for q in set(re.findall(r'Q-(?:T-)?[A-Z]+-\d+', open(p, encoding="utf-8").read())):
        check(q in reg or q in ALIAS, f"{p}: undefined decision {q}")
check(all(x[2] in ("proposed", "blocking", "accepted", "accepted_with_changes", "rejected", "group") for x in reg.values()), "register: bad status")
# rows assigned to a spec; specs <= 5 tasks
mapped = {x[0]: x for x in R("docs/contract-map.tsv")[1:]}
check(set(mapped) == set(rows), "contract-map.tsv does not cover exactly the inventory rows")
for i, r in rows.items():
    if r["impl"] != "done": check(mapped.get(i, [""] * 4)[3] not in ("", "(implementada)"), f"{i}: not assigned to a spec")
# decisions a contract block declares must appear (normalized) in the generated traceability column
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))   # -I drops the script directory from sys.path
from contract_decisions import declared_decisions
for path in glob.glob("docs/inventory/contracts/*.md"):
    for row_id, declared in declared_decisions(open(path, encoding="utf-8").read()).items():
        if row_id not in mapped: continue
        in_map = set(re.findall(r"Q-(?:T-)?[A-Z]+-\d+", mapped[row_id][9]))
        for q in sorted(declared):
            check(q in reg, f"{row_id}: contract declares unknown decision {q}")
            check(q in in_map, f"{row_id}: contract declares {q} but contract-map.tsv decisions column omits it")
for p in glob.glob("docs/superpowers/plans/units/[0-9]*.md"):
    n = len(re.findall(r'^### Tarea \d+', open(p, encoding="utf-8").read(), re.M))
    check(n <= 5, f"{p}: {n} tasks")
# every row has a contract block
docs = "".join(open(p, encoding="utf-8").read() for p in glob.glob("docs/inventory/contracts/*.md"))
for i in rows:
    check(re.search(r'(?m)^#+ .*\b' + re.escape(i) + r'\b', docs) is not None, f"{i}: no contract block heading")
# citations path:line exist within file length
cache = {}
def nlines(p):
    if p not in cache:
        cache[p] = sum(1 for _ in open(p, "rb")) if os.path.isfile(p) else None
    return cache[p]
bad = 0
abbreviated = 0
for src in glob.glob("docs/inventory/contracts/*.md") + ["docs/inventory/actor-classic.tsv", "docs/inventory/actor-typed.tsv"]:
    for m in re.finditer(r'(?<![A-Za-z0-9_\-./])((?:actor|actor-tests|testkit|actor-typed|actor-typed-tests|actor-testkit-typed)/[A-Za-z0-9_\-./]+\.(?:scala|java|conf)):(\d+)(?:-(\d+))?(?!\d|-\d)(?!(?:,\d+(?:-\d+)?)* \(1\.7\.1\))', open(src, encoding="utf-8").read()):
        if "/..." in m.group(1) or "..." in m.group(1): abbreviated += 1; continue
        n = nlines("upstream/pekko/" + m.group(1))
        if n is None:   # short form relative to org/apache/pekko/ in one of the module trees
            for pre in ("actor/src/main/scala/org/apache/pekko/", "actor-tests/src/test/scala/org/apache/pekko/", "testkit/src/main/scala/org/apache/pekko/", "testkit/src/test/scala/org/apache/pekko/", "actor-typed/src/main/scala/org/apache/pekko/", "actor-typed-tests/src/test/scala/org/apache/pekko/", "actor-testkit-typed/src/main/scala/org/apache/pekko/"):
                cand = "upstream/pekko/" + pre + m.group(1)
                if nlines(cand) is not None: n = nlines(cand)
                if n is None and m.group(1).startswith("actor/") and nlines("upstream/pekko/" + pre + m.group(1)[6:]) is not None: n = nlines("upstream/pekko/" + pre + m.group(1)[6:])
                if n is not None: break
        if n is None or int(m.group(2)) > n or (m.group(3) and int(m.group(3)) > n): bad += 1; fail.append(f"{src}: citation {m.group(0)} out of range or missing") if bad <= 15 else None
# summaries and README numbers
cl = [r for i, r in rows.items() if i.startswith("AC-")]
ty = [r for i, r in rows.items() if i.startswith("AT-")]
for p, need in (("docs/inventory/actor-classic.md", f"**{len(cl)} filas**"), ("docs/inventory/actor-typed.md", f"**{len(ty)} filas**"),
                ("docs/implementation-order.md", f"{len(rows)} filas del inventario ({len(cl)} de Classic"), ("README.md", f"{len(rows)} filas ({len(cl)} Classic, {len(ty)} Typed)")):
    check(need in open(p, encoding="utf-8").read(), f"{p}: stale count (expected '{need}')")
ext = sum(r["analysis"] == "extracted" for r in rows.values())
check(f"{ext} `extracted` y {len(rows)-ext} `listed`" in open("README.md", encoding="utf-8").read(), "README.md: stale extracted/listed")
c = collections.Counter(x[2] for x in reg.values() if x[2] != "group")
counts_text = (f"{sum(c.values())} decisiones: {c['blocking']} bloqueantes, {c['proposed']} propuestas, {c['accepted']} aceptadas, "
               f"{c['accepted_with_changes']} aceptadas con cambios, {c['rejected']} rechazadas")
for doc in ("docs/implementation-order.md", "README.md", "docs/decisions.md"):
    check(counts_text in open(doc, encoding="utf-8").read(), f"{doc}: stale decision counts (expected '{counts_text}')")
# unit plan: explicit row -> unit assignment (docs/unit-rows.tsv)
if os.path.exists("docs/unit-rows.tsv"):
    map_ids = [x[0] for x in R("docs/contract-map.tsv")[1:]]
    assigned = collections.defaultdict(list)
    for x in R("docs/unit-rows.tsv")[1:]: assigned[x[0]].append(x[1])
    for rid in map_ids:
        check(len(assigned.get(rid, [])) == 1, f"unit-rows.tsv: {rid} is in {len(assigned.get(rid, []))} units (expected exactly 1)")
    for rid in assigned: check(rid in set(map_ids), f"unit-rows.tsv: unknown row {rid}")
    declared = dict(re.findall(r"(?m)^### (P\d+) .*\n\n- \*\*Resultado.*\n- \*\*Filas \((\d+);", open("docs/unit-plan.md", encoding="utf-8").read()))
    per_unit = collections.Counter(u for us in assigned.values() for u in us)
    check(set(declared) == set(per_unit), f"unit-plan.md units {sorted(set(declared) ^ set(per_unit))} differ from unit-rows.tsv")
    for u, n in declared.items(): check(per_unit.get(u) == int(n), f"unit-plan.md {u}: declares {n} rows, unit-rows.tsv has {per_unit.get(u)}")
# unit plan: generated parts (decisions per unit, execution order) and declared dependencies are consistent
if os.path.exists("docs/unit-plan.md"):
    import subprocess
    unit_plan = subprocess.run([sys.executable, "-I", os.path.join(ROOT, "tools/package/unit_plan.py"), "--check"], capture_output=True, text=True, env=dict(os.environ, RAKKA_PACKAGE_ROOT=ROOT))
    for line in unit_plan.stdout.splitlines():
        if line.startswith("FAIL"): fail.append("unit-plan: " + line[5:])
    check(unit_plan.returncode == 0 or any(l.startswith("FAIL") for l in unit_plan.stdout.splitlines()), "unit_plan.py --check crashed: " + unit_plan.stderr[-300:])
# provenance (decision Q-X-18, docs/legal/provenance-audit.md)
# 1. every Rust source of crates/ names where it comes from: either the Pekko 2.0.0 files it derives from, or the clean-room specification it implements.
# 2. nothing in crates/, tools/ or oracle/ mentions OpenJDK, the GPL or the Classpath exception (those words live only in docs/legal/).
# PENDING_CLEANROOM lists the files still waiting for their clean-room rewrite (Q-PATH-6); it must only shrink and ends empty.
PENDING_CLEANROOM = set()
for rs in sorted(glob.glob("crates/*/src/**/*.rs", recursive=True)):
    if os.path.basename(rs) in ("lib.rs", "mod.rs") or rs in PENDING_CLEANROOM: continue
    head = "".join(open(rs, encoding="utf-8").readlines()[:8])
    m = re.search(r"Derived from Apache Pekko 2\.0\.0 ([^\n]*?) \(Apache-2\.0\)\.", head)
    cleanroom = re.search(r"Clean-room implementation from (docs/legal/cleanroom/[\w.-]+-spec\.md) \(Apache-2\.0\)\.", head)
    check(m is not None or cleanroom is not None, f"{rs}: missing attribution header (`// Derived from Apache Pekko 2.0.0 <path> (Apache-2.0).` or `// Clean-room implementation from docs/legal/cleanroom/<file>-spec.md (Apache-2.0).`)")
    if cleanroom: check(os.path.isfile(cleanroom.group(1)), f"{rs}: the clean-room specification {cleanroom.group(1)} does not exist")
    if m:
        paths = re.findall(r"[\w-]+/src/(?:main|test)/[\w/.-]+\.(?:scala|java)", m.group(1))
        check(bool(paths), f"{rs}: attribution header names no Pekko source file")
        for p in paths: check(os.path.isfile("upstream/pekko/" + p), f"{rs}: attribution names {p}, which does not exist in upstream/pekko")
FORBIDDEN = re.compile(r"OpenJDK|\bGPL|Classpath[ -]exception")
for root in ("crates", "tools", "oracle"):
    for dp, dn, fn in os.walk(root):
        dn[:] = [d for d in dn if d not in ("target", "__pycache__", "node_modules", ".git")]
        for name in fn:
            path = os.path.join(dp, name)
            if path in PENDING_CLEANROOM or path in ("tools/package/check.py", "tools/package/generate.py") or not name.endswith((".rs", ".toml", ".py", ".sh", ".java", ".md", ".txt", ".golden", ".tsv", ".jsonl")): continue
            try: text = open(path, encoding="utf-8").read()
            except UnicodeDecodeError: continue
            hit = FORBIDDEN.search(text)
            check(hit is None, f"{path}: mentions {hit.group(0) if hit else ''!r}; GPL sources are not read or ported, only docs/legal/ may talk about them")
# docs/progress.md is generated: its totals must match the inventory
prog_path = "docs/progress.md"
check(os.path.exists(prog_path), "docs/progress.md missing (run tools/package/generate.py)")
if os.path.exists(prog_path):
    prog = open(prog_path, encoding="utf-8").read()
    excluded = [r for r in rows.values() if r["validation"] == "excluded" or "EXCLUIDA (autorizada" in r["notes"]]
    live = [r for r in rows.values() if r not in excluded]
    n_done = sum(1 for r in live if r["impl"] == "done"); n_part = sum(1 for r in live if r["impl"] == "partial")
    m = re.search(r"\| \*\*Total\*\* \| (\d+) \| (\d+) \| (\d+) \| (\d+) \| (\d+) \|", prog)
    check(m is not None and tuple(map(int, m.groups())) == (len(live), n_done, n_part, len(live) - n_done - n_part, len(excluded)), "docs/progress.md totals do not match the inventory (regenerate it)")
print(f"abbreviated_citations_skipped={abbreviated}")
print(f"rows={len(rows)} decisions={sum(c.values())} groups={len(reg) - sum(c.values())} citation_problems={bad} failures={len(fail)}")
for f in fail: print("FAIL", f)
sys.exit(1 if fail else 0)
