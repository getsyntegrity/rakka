"""Q-X-22 selection rules for the PR tier: they only ever remove files from the authoritative selection (gates.select_mutants) after proving
something about them, and any doubt keeps the conservative selection. MILESTONE never uses them.

R2: a source file whose only change is added `pub mod x;` lines selects nothing, when every file of each new module is new and holds nothing
that can reach existing code (an impl for a type the module does not declare, a macro at item level, an export or link attribute...).

Derived-from: none (project tooling, Apache-2.0).
"""
import difflib, json, os, re, time

import gates

SOURCE_PREFIX = gates.CRATE_PREFIX + "src/"
LITERALS_AND_COMMENTS = re.compile(r'''
    //[^\n]*
  | /\*.*?\*/
  | b?c?r(?P<hashes>\#*)".*?"(?P=hashes)
  | b?c?"(?:\\.|[^"\\])*"
  | b?'(?:\\(?:u\{[^}]*\}|x..|.)|[^'\\\n])'
''', re.S | re.X)
REACHES_OUTSIDE = re.compile(r"\bmacro_rules\b|\bextern\b|\binclude!|#\s*!?\[\s*(?:macro_export|macro_use|no_mangle|export_name|link_section|"
                             r"global_allocator|panic_handler|alloc_error_handler|path|unsafe)\b")
STARTS_ITEM = r"(?:\A|(?<=[{};\]]))\s*"
IMPL = re.compile(STARTS_ITEM + r"(?:unsafe\s+)?impl\b")
ITEM_MACRO = re.compile(STARTS_ITEM + r"[A-Za-z_][\w:]*\s*!\s*[\(\[\{]")
LOCAL_TYPE = re.compile(r"\b(?:struct|enum|union)\s+([A-Za-z_]\w*)")


def code_only(source):
    """The source without comments, with every literal emptied (nested block comments are cut at the first end: the rest then reads as code, which only widens)."""
    return LITERALS_AND_COMMENTS.sub(lambda m: " " if m.group(0).startswith("/") else '""', source)


def at_item_level(code, position):
    """True when every block open at `position` is a `mod` block: what stands there is an item of a module."""
    blocks, header_start = [], 0
    for index, character in enumerate(code[:position]):
        if character == "{":
            blocks.append(bool(re.search(r"\bmod\s+\w+\s*$", code[header_start:index])))
        elif character == "}" and blocks:
            blocks.pop()
        if character in "{};":
            header_start = index + 1
    return all(blocks)


def self_type(code, after_impl):
    """The name of the type an `impl` header is for, or None when it is not a single plain name."""
    header = code[after_impl:].replace("->", "  ")
    depth, top_level, index = 0, [], 0
    if header.lstrip().startswith("<"):      # skip the generic parameters of the impl itself
        for index, character in enumerate(header):
            depth += {"<": 1, ">": -1}.get(character, 0)
            if character == ">" and depth == 0:
                break
        header = header[index + 1:]
    for character in header:
        depth += {"<": 1, ">": -1}.get(character, 0)
        if character == "{" and depth == 0:
            break
        top_level.append(character)
    target = re.split(r"\bwhere\b", "".join(top_level))[0]
    target = re.split(r"\s+for\s+", target)[-1].strip()
    plain = re.fullmatch(r"([A-Za-z_]\w*)\s*(<.*>)?", target, re.S)
    return plain.group(1) if plain else None


def reaches_existing_code(sources):
    """Whether the files of a new module may change how code outside it behaves."""
    codes = [code_only(source) for source in sources]
    local_types = {name for code in codes for name in LOCAL_TYPE.findall(code)}
    for code in codes:
        if REACHES_OUTSIDE.search(code):
            return True
        if any(self_type(code, found.end()) not in local_types for found in IMPL.finditer(code)):
            return True
        if any(at_item_level(code, found.end() - 1) for found in ITEM_MACRO.finditer(code)):
            return True
    return False


def added_modules(base, head):
    """The modules declared by a change that only adds `pub mod x;` lines; None for any other change."""
    old, new = base.splitlines(), head.splitlines()
    modules = []
    for tag, _i1, _i2, start, end in difflib.SequenceMatcher(a=old, b=new, autojunk=False).get_opcodes():
        if tag == "equal":
            continue
        previous = next((line for line in reversed(new[:start]) if line.strip()), "")
        if tag != "insert" or previous.lstrip().startswith("#["):      # an attribute above would now apply to the new line
            return None
        for line in new[start:end]:
            declared = re.fullmatch(r"pub mod ([a-z_]\w*);", line)
            if line.strip() and not declared:
                return None
            modules += [declared.group(1)] if declared else []
    return modules or None


def adds_only_independent_modules(path, read_base, read_head, list_head):
    if not (path.startswith(SOURCE_PREFIX) and path.endswith(".rs")):
        return False
    base, head = read_base(path), read_head(path)
    modules = added_modules(base, head) if base is not None and head is not None else None
    if modules is None:
        return False
    stem = os.path.basename(path)[:-len(".rs")]
    folder = os.path.dirname(path) if stem in ("mod", "lib", "main") else path[:-len(".rs")]
    for module in modules:
        files = list_head(f"{folder}/{module}")
        if not files or any(read_base(file) is not None for file in files) or reaches_existing_code([read_head(file) for file in files]):
            return False
    return True


def narrow(changed, mutants, tool_mutants, unit_mutants, read_base, read_head, list_head):
    """-> {selected, rules, not_selected_by_rule}. `read_base`/`read_head`: path -> text or None; `list_head`: module path without
    extension -> its files."""
    conservative = gates.select_mutants(changed, mutants, tool_mutants, unit_mutants)
    excused = [path for path in changed if adds_only_independent_modules(path, read_base, read_head, list_head)]
    selected = gates.select_mutants([path for path in changed if path not in excused], mutants, tool_mutants, unit_mutants)
    return {"selected": selected, "rules": ["Q-X-22 R2: " + path for path in excused],
            "not_selected_by_rule": [name for name in conservative if name not in selected]}


def select_for_pr(base, mutants, tool_mutants, unit_mutants, repo=gates.ROOT):
    """`narrow` over the checkout against `base`; the report of what each rule removed is kept under .gates/selection/ (never pruned)."""
    merge_base = gates.git_raw("merge-base", base, "HEAD", repo=repo).strip()

    def read_base(path):
        try:
            return gates.git_raw("show", f"{merge_base}:{path}", repo=repo)
        except (gates.UsageError, UnicodeDecodeError):
            return None

    def read_head(path):
        try:
            with open(os.path.join(repo, path), encoding="utf-8") as source_file:
                return source_file.read()
        except (OSError, UnicodeDecodeError):
            return None

    def list_head(module):
        found = [module + ".rs"] if os.path.isfile(os.path.join(repo, module + ".rs")) else []
        for folder, _folders, names in os.walk(os.path.join(repo, module)):
            found += [os.path.relpath(os.path.join(folder, name), repo) for name in names]
        return sorted(found)

    changed = gates.changed_files(base, repo=repo)
    report = narrow(changed, mutants, tool_mutants, unit_mutants, read_base, read_head, list_head)
    report.update(base=merge_base, changed=changed)
    report["report_path"] = os.path.join(".gates", "selection", f"{time.strftime('%Y%m%dT%H%M%S')}-{os.getpid()}.json")
    path = os.path.join(repo, report["report_path"])
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as out:
        json.dump(report, out, indent=1)
    return report
