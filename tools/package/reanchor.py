#!/usr/bin/env python3
"""Re-anchor path:line citations of the inventory from Pekko v1.7.1 to v2.0.0 (one-off migration tool).

Uses `git diff -M -U0 v1.7.1 v2.0.0` of upstream/pekko. Citations whose cited lines did not change are shifted;
citations whose cited lines changed, or whose file was renamed/deleted, are shifted when possible and reported so the
row gets a "revisar contra 2.0.0" note. Usage: reanchor.py [--apply]"""
import re, subprocess, glob, os, sys, csv, collections
csv.field_size_limit(10**9)
ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
os.chdir(ROOT)
APPLY = "--apply" in sys.argv
UP = "upstream/pekko"
OLD, NEW = "v1.7.1", "v2.0.0"
TOP = ["actor", "actor-tests", "actor-typed", "actor-typed-tests", "actor-testkit-typed", "testkit", "docs"]
diff = subprocess.run(["git", "-C", UP, "diff", "-M", "-U0", "--no-color", OLD, NEW, "--"] + TOP, capture_output=True, text=True, check=True).stdout
files = {}   # old path -> dict(new=path|None, hunks=[(a,b,c,d)])
cur = None
for line in diff.split("\n"):
    if line.startswith("diff --git "):
        m = re.match(r"diff --git a/(.*) b/(.*)$", line); cur = dict(old=m.group(1), new=m.group(2), hunks=[], kind="modified", added=False)
        files[m.group(1)] = cur
    elif cur is not None:
        if line.startswith("deleted file"): cur["kind"] = "deleted"; cur["new"] = None
        elif line.startswith("new file"): cur["kind"] = "added"
        elif line.startswith("rename to"): cur["kind"] = "renamed"
        else:
            m = re.match(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@", line)
            if m: cur["hunks"].append((int(m.group(1)), int(m.group(2) or 1), int(m.group(3)), int(m.group(4) or 1)))
        if cur["kind"] == "added": pass
added = [f["new"] for f in files.values() if f["kind"] == "added"]

def newline(hunks, l):
    """Map old line l -> (new line, inside_change)."""
    off = 0
    for a, b, c, d in hunks:
        if b == 0:                       # pure insertion after old line a
            if a < l: off += d
            continue
        if a + b - 1 < l: off += d - b
        elif a <= l: return c + min(l - a, max(d - 1, 0)), True
    return l + off, False

PREFIXES = ["", "actor/src/main/scala/org/apache/pekko/", "actor-tests/src/test/scala/org/apache/pekko/", "testkit/src/main/scala/org/apache/pekko/", "testkit/src/test/scala/org/apache/pekko/", "actor-typed/src/main/scala/org/apache/pekko/", "actor-typed-tests/src/test/scala/org/apache/pekko/", "actor-testkit-typed/src/main/scala/org/apache/pekko/"]
CIT = re.compile(r'(?<![A-Za-z0-9_\-./])((?:actor|actor-tests|testkit|actor-typed|actor-typed-tests|actor-testkit-typed)/[A-Za-z0-9_\-./]+\.(?:scala|java|conf)):(\d+(?:-\d+)?(?:,\d+(?:-\d+)?)*)')
def old_exists(p):
    r = subprocess.run(["git", "-C", UP, "cat-file", "-e", f"{OLD}:{p}"], capture_output=True)
    return r.returncode == 0
resolved = {}
def resolve(p):
    """short or full path as written -> (prefix, old full path) or None"""
    if p in resolved: return resolved[p]
    out = None
    for pre in PREFIXES:
        for cand in (pre + p, pre + p[6:] if p.startswith("actor/") else None):
            if cand and ("/..." not in cand and "..." not in cand) and old_exists(cand): out = (cand[:len(cand) - len(p)] if cand.endswith(p) else None, cand); break
        if out: break
    resolved[p] = out
    return out

changes = collections.defaultdict(list)   # row id -> [descriptions]
stats = collections.Counter()
def rewrite(text, row_of):
    def sub(m):
        p, spec = m.group(1), m.group(2)
        r = resolve(p)
        if r is None: stats["unresolved"] += 1; return m.group(0)
        _, full = r
        f = files.get(full)
        if f is None: stats["unchanged_file"] += 1; return m.group(0)
        if f["kind"] == "deleted":
            stats["deleted"] += 1; dest = [a for a in added if os.path.basename(a) == os.path.basename(full)]; changes[row_of(m.start())].append(f"{os.path.basename(full)} eliminado en 2.0.0 (cita {spec} sin reanclar, marcada (1.7.1))" + (f"; posible destino {', '.join(dest)}" if dest else "")); return m.group(0) + " (1.7.1)"
        newfull = f["new"]
        newp = p
        if newfull != full:
            suffix_old = full[len(full) - len(p):] if full.endswith(p) else None
            stats["renamed"] += 1
            newp = newfull[len(newfull) - len(p):] if suffix_old and newfull.endswith(os.path.basename(p)) and len(newfull) >= len(p) and full.endswith(p) and newfull.rsplit("/",1)[0] == full.rsplit("/",1)[0] else newfull
        parts, desc, touched = [], [], False
        for part in spec.split(","):
            s, _, e = part.partition("-"); s = int(s); e = int(e) if e else s
            ns, i1 = newline(f["hunks"], s); ne, i2 = newline(f["hunks"], e)
            inside = i1 or i2 or any((b == 0 and s <= a < e) or (b > 0 and a <= e and a + b - 1 >= s) for a, b, c, d in f["hunks"])
            parts.append(f"{ns}-{ne}" if e != s or ns != ne and part.count("-") else f"{ns}")
            if part.count("-") and ns == ne: parts[-1] = f"{ns}"
            if inside: touched = True; desc.append(f"{part}->{ns}-{ne}" if ns != ne else f"{part}->{ns}")
            if ns != s or ne != e: stats["shifted"] += 1
        if touched:
            stats["content_changed"] += 1
            changes[row_of(m.start())].append(f"{os.path.basename(newfull)} (líneas {', '.join(desc)}) cambió entre 1.7.1 y 2.0.0")
        if newfull != full:
            changes[row_of(m.start())].append(f"{os.path.basename(full)} renombrado a {newfull}")
        return f"{newp}:{','.join(parts)}"
    return CIT.sub(sub, text)

HEAD = re.compile(r'(?m)^#+ (A[CT]-[A-Z]+-\d+(?:\.\d+)?)\b')
# contracts (markdown): attribute citations to the row of the closest preceding defining heading
for path in sorted(glob.glob("docs/inventory/contracts/*.md")):
    text = open(path, encoding="utf-8").read()
    marks = [(m.start(), m.group(1)) for m in HEAD.finditer(text)]
    def row_of(pos, marks=marks):
        r = None
        for s, i in marks:
            if s <= pos: r = i
            else: break
        return r or "?"
    new = rewrite(text, row_of)
    if APPLY and new != text: open(path, "w", encoding="utf-8").write(new)
# inventory TSVs: citations belong to the row on their own line
for name in ("classic", "typed"):
    path = f"docs/inventory/actor-{name}.tsv"
    lines = open(path, encoding="utf-8").read().split("\n")
    out = []
    for ln in lines:
        rid = ln.split("\t", 1)[0]
        out.append(rewrite(ln, lambda pos, rid=rid: rid) if rid.startswith("A") else ln)
    # write notes later (below), keep text for now
    text2 = "\n".join(out)
    if APPLY: open(path, "w", encoding="utf-8").write(text2)

# notes: append "revisar contra 2.0.0" to rows
if APPLY:
    for name in ("classic", "typed"):
        path = f"docs/inventory/actor-{name}.tsv"
        lines = open(path, encoding="utf-8").read().split("\n")
        for k, ln in enumerate(lines):
            rid = ln.split("\t", 1)[0]
            if rid in changes:
                ds = sorted(set(changes[rid]))
                note = "revisar contra 2.0.0: " + "; ".join(ds)
                cols = ln.split("\t")
                cols[-1] = (cols[-1] + " | " if cols[-1].strip() not in ("", "-") else "") + note
                lines[k] = "\t".join(cols)
        open(path, "w", encoding="utf-8").write("\n".join(lines))

# 2.0.0 files without any citation (possible omissions)
cited = set()
for path in glob.glob("docs/inventory/**/*", recursive=True) + glob.glob("docs/inventory/contracts/*"):
    if os.path.isfile(path):
        for m in CIT.finditer(open(path, encoding="utf-8").read()):
            r = resolve(m.group(1))
            if r:
                f = files.get(r[1]); cited.add(f["new"] if f and f["new"] else r[1])
            cited.add(m.group(1))
def is_cited(p): return p in cited or any(p.endswith("/" + c) or p == c for c in cited)
uncited = [p for p in added if (p.startswith("actor/src/main") or p.startswith("actor-typed/src/main") or p.startswith("actor-testkit-typed/src/main") or p.startswith("testkit/src/main")) and p.endswith((".scala", ".java")) and not is_cited(p)]
print(dict(stats)); print("rows with changes:", len(changes)); print("added files:", len(added), "uncited main:", len(uncited))
if "--list-added" in sys.argv or APPLY:
    with open("docs/inventory/pekko-2.0.0-reanchor.md", "w", encoding="utf-8") as o:
        o.write("# Reanclaje del inventario a Pekko 2.0.0\n\nGenerado por `tools/package/reanchor.py` (git diff -M -U0 v1.7.1 v2.0.0 de `upstream/pekko`). Las citas `path:linea` se movieron al sitio equivalente de 2.0.0; las filas cuyo contenido citado cambio llevan en `notes` \"revisar contra 2.0.0: ...\" sin cambiar su estado.\n\n")
        o.write(f"- Citas desplazadas sin cambio de contenido: {stats['shifted']}\n- Citas cuyo contenido cambio: {stats['content_changed']} (filas con nota: {len(changes)})\n- Citas a archivos eliminados: {stats['deleted']}\n- Citas a archivos renombrados: {stats['renamed']}\n- Citas que no se pudieron resolver (abreviadas): {stats['unresolved']}\n\n")
        o.write("## Archivos nuevos de 2.0.0 (produccion) sin cita en el inventario: posibles omisiones\n\n")
        for p in sorted(uncited): o.write(f"- `{p}`\n")
        o.write("\n## Filas con nota \"revisar contra 2.0.0\"\n\n")
        for r in sorted(changes): o.write(f"- {r}\n")
