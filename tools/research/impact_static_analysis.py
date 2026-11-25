#!/usr/bin/env python3.12
"""Slice 3 research: impact-based mutant selection, models A / B / C, over the 13 open PRs and the current 333-mutant inventory. Read-only: no mutant is executed,
no gate is touched. Output: impact.json (everything per PR and model) and impact.md (the tables)."""
import ast
import collections
import json
import os
import re
import subprocess
import sys
import time

START = time.time()
TREE = "/Users/pablogore/workspace/pablogore/rakka/.claude/worktrees/slice3-mailbox-requirements"
HERE = os.path.dirname(os.path.abspath(__file__))
CRATE = "crates/rakka-actor"
sys.path.insert(0, f"{TREE}/tools/parity")
sys.argv = ["x"]
import check_divergence as cd

PRS = (362, 363, 364, 365, 366, 369, 367, 370, 371, 368, 372, 373, 374)
SECONDS_PER_MUTANT = 1347.1 / 12      # ESTIMATE basis: the measured cold baseline of the 12-mutant benchmark (a single sample; not a per-kind measurement)


def git(*args):
    return subprocess.run(["git", "-C", TREE, *args], capture_output=True, text=True).stdout


# ---------------------------------------------------------------- inventory (the tip of the stack: 326 + 7 = 333)
INVENTORY = {}
for name, (rel, *_rest) in cd.MUTANTS.items():
    unit = cd.UNIT_MUTANTS.get(name)
    INVENTORY[name] = {"kind": "unit" if unit else "replay", "file": f"{CRATE}/{rel}", "unit": unit}
for name, (rel, *_rest) in cd.TOOL_MUTANTS.items():
    INVENTORY[name] = {"kind": "tool", "file": f"tools/parity/{rel}", "unit": None}
assert len(INVENTORY) == 333, len(INVENTORY)


def unit_test_file(unit):
    module, _test = unit
    if module.startswith("test:"):
        return f"{CRATE}/tests/{module[5:]}.rs"
    base = f"{CRATE}/src/" + module.replace("::", "/")
    return base + ".rs" if os.path.exists(f"{TREE}/{base}.rs") else base + "/mod.rs"


# ---------------------------------------------------------------- dependency graphs over crates/rakka-actor/src
SRC = sorted(os.path.relpath(os.path.join(d, f), TREE) for d, _s, fs in os.walk(f"{TREE}/{CRATE}/src") for f in fs if f.endswith(".rs"))
TEXT = {f: open(f"{TREE}/{f}", encoding="utf-8").read() for f in SRC}


def module_of(path):
    parts = path[len(f"{CRATE}/src/"):-3].split("/")
    if parts[-1] in ("mod", "lib"):
        parts = parts[:-1]
    return parts


MODULE_FILE = {tuple(module_of(f)): f for f in SRC}


def resolve(parts):
    for cut in range(len(parts), -1, -1):
        if tuple(parts[:cut]) in MODULE_FILE:
            return MODULE_FILE[tuple(parts[:cut])]
    return None


def expand(prefix, text):
    """`a::{b, c::{d, e}}` -> [prefix+a::b, prefix+a::c::d, ...] (glob and `self` keep their prefix)."""
    text = text.strip()
    if "{" not in text:
        return [prefix + [p for p in text.split("::") if p and p not in ("*", "self")]]
    head, body = text.split("{", 1)
    body = body[:body.rindex("}")]
    out, depth, current = [], 0, ""
    for ch in body + ",":
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
        if ch == "," and depth == 0:
            out += expand(prefix + [p for p in head.split("::") if p], current) if current.strip() else []
            current = ""
        else:
            current += ch
    return out


MODULE_EDGES = collections.defaultdict(set)
FLAGS = collections.defaultdict(set)
for f, text in TEXT.items():
    own = module_of(f)
    for statement in re.findall(r"\buse\s+([^;]+);", text):
        statement = " ".join(statement.split())
        if statement.startswith("crate::"):
            roots = [[]]
            targets = [t for p in roots for t in expand(p, statement[len("crate::"):])]
        elif statement.startswith("super::"):
            targets = []
            for base in (own[:-1] if own else [], own):                 # inline `mod tests { use super::* }` makes `super` the file itself: keep both readings
                targets += expand(base, statement[len("super::"):])
        elif statement.startswith("self::"):
            targets = expand(own, statement[len("self::"):])
        else:
            targets = []
        for target in targets:
            resolved = resolve(target)
            if resolved and resolved != f:
                MODULE_EDGES[f].add(resolved)
    for path in re.findall(r"\bcrate::((?:\w+::)*\w+)", text):
        resolved = resolve(path.split("::"))
        if resolved and resolved != f:
            MODULE_EDGES[f].add(resolved)
    for child in re.findall(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;", text, re.M):
        resolved = resolve(own + [child])
        if resolved and resolved != f:
            MODULE_EDGES[f].add(resolved)
    for flag, pattern in (("macro", r"macro_rules!"), ("cfg", r"#\[cfg\((?!test)|#!\[cfg"), ("include", r"include_str!|include_bytes!|include!"), ("reexport", r"\bpub\s+use\b"),
                          ("glob", r"::\*\s*[;,}]"), ("rename", r"\bas\s+\w+\s*[;,}]"), ("dyn", r"\bdyn\s+[A-Z]"), ("trait_impl", r"^\s*impl(?:<[^>]*>)?\s+[\w:]+(?:<[^>]*>)?\s+for\b")):
        if re.search(pattern, text, re.M):
            FLAGS[f].add(flag)

DEFINED = collections.defaultdict(set)         # CamelCase type-like item -> files defining it
for f, text in TEXT.items():
    for item in re.findall(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|trait|type|union)\s+([A-Z]\w+)", text, re.M):
        DEFINED[item].add(f)
IDENT_EDGES = collections.defaultdict(set)
AMBIGUOUS_IDENTIFIERS = 0
for f, text in TEXT.items():
    tokens = set(re.findall(r"\b[A-Z]\w+\b", text))
    for item in tokens & DEFINED.keys():
        for definer in DEFINED[item]:
            if definer != f:
                IDENT_EDGES[f].add(definer)
        AMBIGUOUS_IDENTIFIERS += len(DEFINED[item]) > 1

MACRO_DEFINED = collections.defaultdict(set)
for f, text in TEXT.items():
    for macro in re.findall(r"macro_rules!\s+(\w+)", text):
        MACRO_DEFINED[macro].add(f)
MACRO_EDGES = collections.defaultdict(set)
for f, text in TEXT.items():
    for macro, definers in MACRO_DEFINED.items():
        if re.search(rf"\b{macro}!", text):
            MACRO_EDGES[f] |= {d for d in definers if d != f}
INCLUDERS = collections.defaultdict(set)        # data file (repo-relative) -> source files that include it
for f, text in TEXT.items():
    for relative in re.findall(r'include_(?:str|bytes)!\(\s*"([^"]+)"', text):
        INCLUDERS[os.path.normpath(os.path.join(os.path.dirname(f), relative))].add(f)
FORWARD = collections.defaultdict(set)
for graph in (MODULE_EDGES, IDENT_EDGES, MACRO_EDGES):
    for f, ds in graph.items():
        FORWARD[f] |= ds
MODULE_ONLY = {f: set(ds) for f, ds in MODULE_EDGES.items()}


def reverse(graph):
    out = collections.defaultdict(set)
    for f, ds in graph.items():
        for d in ds:
            out[d].add(f)
    return out


def closure(start, graph):
    seen, todo = set(start), list(start)
    while todo:
        for nxt in graph.get(todo.pop(), ()):
            if nxt not in seen:
                seen.add(nxt)
                todo.append(nxt)
    return seen


# ---------------------------------------------------------------- the per-PR facts
def changed_files(pr):
    return json.load(open(f"{HERE}/pr-{pr}.json"))


def table_facts(source):
    """-> (tables {name: literal}, skeleton dump with the tables blanked) or None when the file cannot be understood."""
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return None
    tables = {}
    for node in tree.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name) and node.targets[0].id in ("MUTANTS", "TOOL_MUTANTS", "UNIT_MUTANTS", "EQUIVALENT", "SELFTEST"):
            try:
                tables[node.targets[0].id] = ast.literal_eval(node.value)
            except ValueError:
                return None            # a table that is not a literal: not understood, so not provably table-only
            node.value = ast.Constant(None)
    return tables, ast.dump(tree)


def check_divergence_delta(base_ref, head_ref):
    merge_base = git("merge-base", f"origin/{base_ref}", f"origin/{head_ref}").strip()
    old = git("show", f"{merge_base}:tools/parity/check_divergence.py")
    new = git("show", f"origin/{head_ref}:tools/parity/check_divergence.py")
    a, b = table_facts(old), table_facts(new)
    if a is None or b is None:
        return {"table_only": False, "new_or_changed": [], "why": "the tables are not literal or the file does not parse"}
    old_names = {n: v for t in a[0].values() if isinstance(t, dict) for n, v in t.items()}
    new_names = {n: v for t in b[0].values() if isinstance(t, dict) for n, v in t.items()}
    changed = sorted(n for n, v in new_names.items() if old_names.get(n) != v)
    equivalent_changed = a[0].get("EQUIVALENT") != b[0].get("EQUIVALENT")
    return {"table_only": a[1] == b[1], "new_or_changed": changed, "equivalent_set_changed": equivalent_changed,
            "why": "the code outside the mutant tables is identical" if a[1] == b[1] else "code outside the mutant tables changed"}


def classify(path, delta, tool_importers):
    """-> (class, detail). The classes and what each costs: NONE (no effect on any verdict), TABLE (only the new/changed mutants), SRC (graph closure), TESTFILE (the
    unit mutants that run that file), HARNESS_BINARY (every replay mutant), CORPUS (every replay mutant, strict), FULL (all 333)."""
    base = os.path.basename(path)
    if path.endswith(".md") or path.startswith("docs/") or path == "README.md" or path.startswith("rust/divergence-reports/"):
        return "NONE", "documentation or an output of earlier mutation runs"
    if path == "tools/package/generate.py" or path.startswith("tools/package/"):
        return "NONE", "documentation package tooling"
    if path.startswith(f"{CRATE}/src/") and path.endswith(".rs"):
        return "SRC", "crate source"
    if path.startswith(f"{CRATE}/tests/"):
        return "TESTFILE", "integration test file of the crate"
    if path.startswith(f"{CRATE}/examples/"):
        return "HARNESS_BINARY", "the parity binary every replay mutant is judged through"
    if path.startswith(("rust/app-cases/", "cases/")):
        return "CORPUS", "recorded corpus or case index"
    if path.startswith("oracle/"):
        return "FULL", "the oracle that produced the expected outputs"
    if base in ("Cargo.toml", "Cargo.lock") or path.endswith("/Cargo.toml"):
        return "FULL", "dependencies, features or configuration"
    if path == "tools/parity/check_divergence.py":
        if delta and delta["table_only"]:
            return "TABLE", "only the mutant tables changed (proved by AST equality of everything else)"
        return "FULL", "the mutation harness logic changed"
    if path.startswith("tools/") and base.startswith("test_"):
        return "NONE", "tests of the tooling (covered by the FAST gate, not read by any mutant verdict)"
    if path.startswith("tools/") and path.endswith(".py"):
        stem = os.path.splitext(base)[0]
        if not tool_importers.get(stem):
            return "NONE", "a new tool module that no existing file imports (inert until a wiring change)"
        return "FULL", "mutation infrastructure"
    return "FULL", "unknown kind of file: uncertainty widens the scope"


_IMPORTERS = {}


def importers_at(head_ref):
    if head_ref not in _IMPORTERS:
        _IMPORTERS[head_ref] = _importers_at(head_ref)
    return _IMPORTERS[head_ref]


def _importers_at(head_ref):
    """tool module stem -> files of tools/ that import it at the head of the PR (a module nothing imports is inert)."""
    listing = git("ls-tree", "-r", "--name-only", f"origin/{head_ref}", "tools/").split()
    out = collections.defaultdict(set)
    for path in listing:
        if path.endswith(".py"):
            for stem in re.findall(r"^\s*(?:from|import)\s+(\w+)", git("show", f"origin/{head_ref}:{path}"), re.M):
                out[stem].add(path)
    return out


def select(model, files, delta, importers):
    """-> (selected mutant names, notes). Models: A direct, B_forward / B_reverse / B (transitive), C conservative with full fallback, C_monotone (C + the corpus-addition lemma)."""
    paths = [f["path"] for f in files]
    classes = {p: classify(p, delta, importers) for p in paths}
    src = [p for p in paths if classes[p][0] == "SRC"]
    selected, notes, unknown = set(), [], []
    new = set(delta["new_or_changed"]) if delta else set()
    selected |= {n for n in new if n in INVENTORY}                         # a new or edited mutant has never been run: every model runs it
    if model == "A":
        selected |= {n for n, m in INVENTORY.items() if m["file"] in paths}
        unrepresented = [p for p in paths if classes[p][0] not in ("SRC", "NONE") and p not in {m["file"] for m in INVENTORY.values()}]
        notes.append(f"{len(unrepresented)} changed files are outside any mutated file and are ignored by this model")
        return selected, notes, unrepresented
    graph_forward, graph_reverse = FORWARD, reverse(FORWARD)
    if model == "B_module":
        module_reverse = reverse(MODULE_ONLY)
        reach = set(src) | closure(src, MODULE_ONLY) | closure(src, module_reverse)
        selected |= {n for n, m in INVENTORY.items() if m["file"] in reach}
        unrepresented = [p for p in paths if classes[p][0] not in ("SRC", "NONE")]
        notes.append(f"{len(unrepresented)} changed files are not crate source: this model cannot express them")
        return selected, notes, unrepresented
    if model in ("B", "B_forward", "B_reverse"):
        reach = set(src)
        if model in ("B", "B_forward"):
            reach |= closure(src, graph_forward)
        if model in ("B", "B_reverse"):
            reach |= closure(src, graph_reverse)
        selected |= {n for n, m in INVENTORY.items() if m["file"] in reach}
        unrepresented = [p for p in paths if classes[p][0] not in ("SRC", "NONE")]
        notes.append(f"{len(unrepresented)} changed files are not crate source (corpus, harness, oracle, tests, tools): this model cannot express them")
        return selected, notes, unrepresented
    # C_coarse, C and C_monotone
    everything = set(INVENTORY)
    replay = {n for n, m in INVENTORY.items() if m["kind"] in ("replay", "unit")}
    replay_only = {n for n, m in INVENTORY.items() if m["kind"] == "replay"}
    src_extra = set()
    for path in paths:
        cls, why = classes[path]
        if cls == "FULL":
            return everything, [f"FULL fallback: {path} ({why})"], [path]
        if cls == "HARNESS_BINARY":
            selected |= replay_only                                          # the unit mutants run `cargo test --lib/--test`: no example is built for them
            notes.append(f"{path}: every replay mutant (the example is part of the judged binary)")
        elif cls == "CORPUS":
            included_by = INCLUDERS.get(path, set())
            if included_by:                                         # a golden read through include_str!: its includers are changed code as far as the graph is concerned
                src_extra.update(included_by)
                notes.append(f"{path}: included by {sorted(included_by)}")
            if model in ("C", "C_coarse"):
                selected |= replay_only
                notes.append(f"{path}: every replay mutant (corpus changed; strict)")
            else:
                notes.append(f"{path}: not rerun for existing mutants under the monotonicity lemma (needs shadow-mode proof)")
        elif cls == "TESTFILE":
            selected |= {n for n, m in INVENTORY.items() if m["unit"] and unit_test_file(m["unit"]) == path}
            if path.endswith("common/mod.rs"):
                selected |= {n for n, m in INVENTORY.items() if m["unit"] and m["unit"][0].startswith("test:")}
        elif cls == "SRC":
            flags = FLAGS[path] & ({"macro", "cfg", "include"} if model == "C_coarse" else {"cfg"})      # refined C: macros and includes become edges, only cfg stays a fallback
            if flags:
                return everything, [f"FULL fallback: {path} uses {sorted(flags)}"], [path]
    src = list(src) + sorted(src_extra)
    src_reach = closure(src, graph_forward) | closure(src, graph_reverse) if src else set()
    selected |= {n for n, m in INVENTORY.items() if m["file"] in src_reach}
    for path in src:
        flags = sorted(FLAGS[path] & {"reexport", "glob", "rename", "dyn", "trait_impl"})
        if flags:
            unknown.append(f"{path}: {','.join(flags)}")
    selected |= {n for n, m in INVENTORY.items() if m["unit"] and unit_test_file(m["unit"]) in paths}
    return selected, notes, unknown


MODELS = ("A", "B_module", "B_forward", "B_reverse", "B", "C_coarse", "C", "C_monotone")
report = {"inventory": {"total": 333, "replay": sum(m["kind"] == "replay" for m in INVENTORY.values()), "unit": sum(m["kind"] == "unit" for m in INVENTORY.values()),
                         "tool": sum(m["kind"] == "tool" for m in INVENTORY.values()), "mutated_source_files": len({m["file"] for m in INVENTORY.values() if m["kind"] != "tool"})},
          "graph": {"files": len(SRC), "module_edges": sum(map(len, MODULE_EDGES.values())), "identifier_edges": sum(map(len, IDENT_EDGES.values())),
                    "identifiers_defined_in_several_files": AMBIGUOUS_IDENTIFIERS, "files_with_flags": {k: sorted(v) for k, v in FLAGS.items() if v}},
          "seconds_per_mutant_estimate_basis": round(SECONDS_PER_MUTANT, 1), "prs": {}}
for pr in PRS:
    data = changed_files(pr)
    delta = check_divergence_delta(data["baseRefName"], data["headRefName"]) if any(f["path"] == "tools/parity/check_divergence.py" for f in data["files"]) else None
    importers = importers_at(data["headRefName"])
    row = {"title": data["title"], "base": data["baseRefName"], "head": data["headRefOid"][:7], "changed_files": [f["path"] for f in data["files"]],
           "classes": {f["path"]: classify(f["path"], delta, importers)[0] for f in data["files"]}, "check_divergence_delta": delta, "models": {}}
    for model in MODELS:
        selected, notes, unknown = select(model, data["files"], delta, importers)
        excluded = sorted(set(INVENTORY) - selected)
        row["models"][model] = {"selected": len(selected), "excluded": len(excluded), "reduction_percent": round(100 * len(excluded) / 333, 1),
                                "estimated_serial_seconds": round(len(selected) * SECONDS_PER_MUTANT), "unknown_or_ambiguous": unknown, "notes": notes,
                                "selected_names": sorted(selected)}
    report["prs"][str(pr)] = row
report["analysis_seconds"] = round(time.time() - START, 1)
json.dump(report, open(f"{HERE}/impact.json", "w"), indent=1)

lines = ["| PR | files | NONE-class | A | B use-graph | B fwd | B rev | B | C coarse | C | C* (monotone) |", "|---|---|---|---|---|---|---|---|---|---|---|"]
for pr, row in report["prs"].items():
    cells = [f"{row['models'][m]['selected']} (-{row['models'][m]['reduction_percent']}%)" for m in MODELS]
    lines.append(f"| #{pr} | {len(row['changed_files'])} | {sum(c == 'NONE' for c in row['classes'].values())} | " + " | ".join(cells) + " |")
open(f"{HERE}/impact.md", "w").write("\n".join(lines) + "\n")
print("\n".join(lines))
print(json.dumps({k: report[k] for k in ("inventory", "analysis_seconds")}), json.dumps({k: v for k, v in report["graph"].items() if k != "files_with_flags"}))
