"""Q-X-22 selection rules for the PR tier: they only ever remove files from the authoritative selection (gates.select_mutants) after proving
something about them, and any doubt keeps the conservative selection. MILESTONE never uses them.

R2: a source file whose only change is added `pub mod x;` lines selects nothing, when every file of each new module is new and holds nothing
that can reach existing code: only std macros, derives and plain attributes, and every impl is for a type declared in the same scope.

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
# Lists of what is understood: anything else (an unknown macro or attribute, a foreign derive) keeps the conservative selection.
STD_MACROS = {"assert", "assert_eq", "assert_ne", "debug_assert", "debug_assert_eq", "debug_assert_ne", "format", "format_args", "vec", "write",
              "writeln", "print", "println", "eprint", "eprintln", "panic", "unreachable", "todo", "unimplemented", "matches", "concat", "env"}
PLAIN_ATTRIBUTES = {"cfg", "test", "derive", "doc", "allow", "warn", "deny", "expect", "must_use", "inline", "non_exhaustive", "default",
                    "should_panic", "ignore", "repr", "cold", "track_caller"}
STD_DERIVES = {"Debug", "Clone", "Copy", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash", "Default"}
PATH_NAMES = {"std", "core", "alloc", "test", "bool", "char", "str", "f32", "f64", "u8", "u16", "u32", "u64", "u128", "usize",
              "i8", "i16", "i32", "i64", "i128", "isize"}
OUT_OF_REACH_OF_THIS_READER = re.compile(r"\bmacro_rules\b|\bextern\b|\bconst\s+_\b")
IMPL = re.compile(r"(?:\A|(?<=[{};\]]))\s*(?:unsafe\s+)?impl\b")      # an impl item, not `impl Trait` in a type position
MACRO = re.compile(r"((?:r#)?[A-Za-z_]\w*)!\s*[\(\[\{]")               # rustfmt (a PR gate) leaves no space before the `!`
ATTRIBUTE = re.compile(r"#\s*!?\s*\[\s*([^\(\]=]*?)\s*(?=[\(\]=])")
DERIVE = re.compile(r"#\s*!?\s*\[\s*derive\s*\(([^)]*)\)")
LOCAL_TYPE = re.compile(r"\b(?:struct|enum|union)\s+([A-Za-z_]\w*)")


def code_only(source):
    """The source without comments and with every literal emptied. It does not follow nested comments: callers refuse them."""
    return LITERALS_AND_COMMENTS.sub(lambda m: " " if m.group(0).startswith("/") else '""', source)


def block_of(code, position):
    """The blocks open at `position`, by where each one opens: two positions with the same answer share a scope."""
    blocks = []
    for index, character in enumerate(code[:position]):
        if character == "{":
            blocks.append(index)
        elif character == "}" and blocks:
            blocks.pop()
    return tuple(blocks)


def self_type(code, after_impl):
    """The name of the type an `impl` header is for; None when it is not a single plain name or is one of the impl's own parameters."""
    header = code[after_impl:].replace("->", "  ")
    depth, top_level, parameters, index = 0, [], "", 0
    if header.lstrip().startswith("<"):
        for index, character in enumerate(header):
            depth += {"<": 1, ">": -1}.get(character, 0)
            if character == ">" and depth == 0:
                break
        parameters, header = header[:index], header[index + 1:]
    for character in header:
        depth += {"<": 1, ">": -1}.get(character, 0)
        if character == "{" and depth == 0:
            break
        top_level.append(character)
    target = re.split(r"\bwhere\b", "".join(top_level))[0]
    target = re.split(r"\s+for\s+", target)[-1].strip()
    plain = re.fullmatch(r"([A-Za-z_]\w*)\s*(<.*>)?", target, re.S)
    if not plain or "#" in parameters or re.search(rf"[<,]\s*(?:const\s+)?{plain.group(1)}\b", parameters):
        return None
    return plain.group(1)


def reaches_existing_code(sources):
    """Whether the files of a new module may change how code outside it behaves."""
    for source in sources:
        code = code_only(source)
        # nested comments, or a `/*` the reader could not close, would let code hide as a literal: not followed
        if "/*" in code or re.search(r"/\*(?:(?!\*/).)*/\*", source, re.S) or OUT_OF_REACH_OF_THIS_READER.search(code):
            return True
        if any(found.group(1) not in STD_MACROS or code[:found.start()].rstrip().endswith("::") for found in MACRO.finditer(code)):
            return True
        if any(found.group(1) not in PLAIN_ATTRIBUTES for found in ATTRIBUTE.finditer(code)):
            return True
        if any(name.strip() and name.strip() not in STD_DERIVES for found in DERIVE.finditer(code) for name in found.group(1).split(",")):
            return True
        # an impl is local only when its type is declared in the same scope of the same file and never imported (a `cfg` twin could be either)
        imported = set(re.findall(r"\w+", " ".join(re.findall(r"\buse\s[^;]*;", code))))
        declared = {(block_of(code, found.start()), found.group(1)) for found in LOCAL_TYPE.finditer(code) if found.group(1) not in imported}
        if any((block_of(code, found.end()), self_type(code, found.end())) not in declared for found in IMPL.finditer(code)):
            return True
    return False


def added_modules(base, head):
    """The modules declared by a change that only adds `pub mod x;` lines; None for any other change."""
    old, new = base.splitlines(), head.splitlines()
    modules = []
    for tag, _i1, _i2, start, end in difflib.SequenceMatcher(a=old, b=new, autojunk=False).get_opcodes():
        if tag == "equal":
            continue
        # what stands above must be a finished item: an attribute (also behind comments) would now apply to the new line
        above = code_only("\n".join(new[:start])).rstrip()
        if tag != "insert" or "/*" in above or (above and not above.endswith((";", "}"))):
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
        if module in PATH_NAMES or re.search(rf"\b{module}\b", code_only(base)):      # it could change how an existing path resolves
            return False
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
