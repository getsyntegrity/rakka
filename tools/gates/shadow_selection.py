"""SHADOW selection of mutants: what a finer, evidence-based selection WOULD pick for a change, computed next to the authoritative one (gates.select_mutants) and never used for a verdict.
Every doubt widens the selection (fail closed): an unknown kind of file, logic outside the mutant tables, a removed definition, a modified or deleted corpus file, a non-literal table.
Derived-from: none (project tooling, Apache-2.0).

A change is a pair (status, path): `A` added, `M` modified, `M+` modified by added lines only (the caller counted no deletion), `D` deleted, `R` renamed."""
import ast
import collections
import dataclasses

HARNESS = "tools/parity/check_divergence.py"
TABLES = ("MUTANTS", "TOOL_MUTANTS", "UNIT_MUTANTS", "EQUIVALENT", "SELFTEST", "RETIRED_MUTANTS")
GENERATED_OUTPUTS = ("rust/app-cases/replay-report.md", "rust/divergence-reports/")
GLOBAL_PATHS = ("Cargo.toml", "Cargo.lock", "crates/rakka-actor/Cargo.toml", "tools/parity/replay.py", "tools/parity/replay_lib.py", "tools/parity/na_q_cfg_30.py")
EXAMPLE = "crates/rakka-actor/examples/"
SHARED_EXAMPLE_MODULES = ("main.rs", "scenario.rs", "json.rs", "path_case.rs", "text_escape.rs", "xorshift.rs")
Delta = collections.namedtuple("Delta", "table_only new_or_changed removed unchanged")


@dataclasses.dataclass
class Selection:
    everything: bool = False
    selected: list = dataclasses.field(default_factory=list)
    reasons: list = dataclasses.field(default_factory=list)
    family_modules: list = dataclasses.field(default_factory=list)


def _blanked(source):
    tree = ast.parse(source)
    tables = {}
    for node in tree.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name) and node.targets[0].id in TABLES:
            tables[node.targets[0].id] = ast.literal_eval(node.value)          # ValueError: a table that is not a literal is not understood
            node.value = ast.Constant(None)
    return tables, ast.dump(tree)


def _definitions(tables):
    """name -> everything that defines how the mutant is built and judged (its patch tuple and its unit-test entry)."""
    names = {}
    for table in ("MUTANTS", "TOOL_MUTANTS"):
        for name, definition in tables.get(table, {}).items():
            names[name] = [definition, tables.get("UNIT_MUTANTS", {}).get(name)]
    return names


def table_delta(old_source, new_source):
    """The change of the mutant tables of check_divergence.py, and whether it is the ONLY change (the AST of everything else is identical)."""
    try:
        old_tables, old_rest = _blanked(old_source)
        new_tables, new_rest = _blanked(new_source)
    except (SyntaxError, ValueError):
        return Delta(False, [], [], 0)
    old_names, new_names = _definitions(old_tables), _definitions(new_tables)
    changed = sorted(name for name, definition in new_names.items() if old_names.get(name) != definition)
    removed = sorted(set(old_names) - set(new_names))
    other_tables_changed = any(old_tables.get(t) != new_tables.get(t) for t in ("EQUIVALENT", "SELFTEST", "RETIRED_MUTANTS"))
    unchanged = sum(1 for name, definition in new_names.items() if old_names.get(name) == definition)
    return Delta(old_rest == new_rest and not other_tables_changed, changed, removed, unchanged)


def shadow_selection(changes, old_harness_source, new_harness_source, inventory):
    """`inventory`: mutant name -> repo path of the file it mutates. -> Selection."""
    result = Selection()
    by_file = collections.defaultdict(list)
    for name, path in inventory.items():
        by_file[path].append(name)

    def widen(path, kind):
        result.everything = True
        result.reasons.append((path, kind))

    def note(path, kind, names=()):
        result.reasons.append((path, kind))
        result.selected.extend(names)

    for status, path in changes:
        if path.startswith(GENERATED_OUTPUTS):
            note(path, "generated output")
        elif path.startswith("docs/") or path.endswith(".md"):
            note(path, "documentation")
        elif path == HARNESS:
            delta = table_delta(old_harness_source, new_harness_source)
            if delta.table_only and not delta.removed:
                note(path, "mutant definitions", delta.new_or_changed)
            else:
                widen(path, "harness logic (outside the mutant tables, or a removed definition)")
        elif path.startswith("tools/gates/") and path.rsplit("/", 1)[-1].startswith("test_"):
            note(path, "tests of the tooling")
        elif path in GLOBAL_PATHS or path.startswith("tools/gates/"):
            widen(path, "global harness or build")
        elif path.startswith(("rust/app-cases/", "cases/")):
            if status in ("A", "M+"):
                note(path, "corpus addition")
            else:
                widen(path, "corpus modification or deletion")
        elif path.startswith("oracle/"):
            note(path, "oracle source (a verdict reads the recordings, not the oracle)")
        elif path.startswith(EXAMPLE):
            module = path.rsplit("/", 1)[-1]
            if module not in SHARED_EXAMPLE_MODULES and path.startswith(EXAMPLE + "classic-parity/"):
                result.family_modules.append(module)
                widen(path, "family-specific harness (reported, not narrowed yet)")
            else:
                widen(path, "shared harness")
        elif path.startswith("crates/rakka-actor/tests/"):
            widen(path, "product tests (the unit mutants they kill are not mapped: fail closed)")
        elif path.startswith("crates/rakka-actor/src/"):
            note(path, "product", by_file.get(path, ()))
        elif path.startswith("tools/") and path.endswith(".py") and not path.rsplit("/", 1)[-1].startswith("test_"):
            note(path, "tooling (same rule as the authoritative selection)", by_file.get(path, ()))
        elif path.startswith("tools/") and path.rsplit("/", 1)[-1].startswith("test_"):
            note(path, "tests of the tooling")
        else:
            widen(path, "unknown kind of file (fail closed)")
    result.selected = [] if result.everything else sorted(set(result.selected))
    result.family_modules = sorted(set(result.family_modules))
    return result
