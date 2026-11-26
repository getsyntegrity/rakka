"""Q-X-22 R1: proves that a change to the corpus, the oracle, the parity harness or the mutant tables leaves the existing scenarios, their
expected values and the way they run untouched. Anything not proven keeps the conservative selection (all mutants).

Textual additivity is not the proof: the Rust harness is compared item by item. Known limit: a new case runs in the same process as the old
ones of its family, so state they share is not analysed.

Derived-from: none (project tooling, Apache-2.0).
"""
import collections, difflib, re

import pr_selection
import shadow_selection

HARNESS = "tools/parity/check_divergence.py"
RECORDINGS, CASES, ORACLE, EXAMPLES = "rust/app-cases/", "cases/", "oracle/", "crates/rakka-actor/examples/"
ITEM = re.compile(r"\A(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(fn|struct|enum|impl|use)\b\s*([A-Za-z_]\w*)?")
ENTRY = re.compile(r'[ \t]*\(\s*"(?P<label>(?:\\.|[^"\\])*)",\s*(?P<call>\w+)\(\),?\s*\),?[ \t]*\n?')
CRATE_SOURCE = "crates/rakka-actor/src/"
STD_TYPES = {"Arc", "Mutex", "RwLock", "Condvar", "AtomicBool", "AtomicUsize", "AtomicU64", "Ordering", "Duration", "Instant", "HashMap",
             "HashSet", "BTreeMap", "BTreeSet", "VecDeque"}


def covers(path):
    return path == HARNESS or path.startswith((RECORDINGS, CASES, ORACLE, EXAMPLES))


def lines_only_added(base, head):
    opcodes = difflib.SequenceMatcher(a=base.splitlines(), b=head.splitlines(), autojunk=False).get_opcodes()
    return all(tag in ("equal", "insert") for tag, *_positions in opcodes)


def items(source):
    """The top-level items of a Rust file, without comments."""
    text = pr_selection.LITERALS_AND_COMMENTS.sub(lambda m: "" if m.group(0).startswith("/") else m.group(0), source)
    mask = pr_selection.LITERALS_AND_COMMENTS.sub(lambda m: "_" * len(m.group(0)), text)
    found, depth, start = [], 0, 0
    for index, character in enumerate(mask):
        depth += (character in "{([") - (character in "})]")
        closes_block = character == "}" and depth == 0 and not mask[index + 1:].lstrip().startswith(";")
        if closes_block or (character == ";" and depth == 0):
            found.append(text[start:index + 1].strip())
            start = index + 1
    return found + ([text[start:].strip()] if text[start:].strip() else [])


def imported_types(use_item, read_head):
    """The names a `use` brings into scope when each is shown to be a plain type (a trait could change how existing method calls resolve):
    a struct or enum declared in the crate module it comes from, or a known std type. None otherwise (also globs, `self`, renames, nesting)."""
    shaped = re.fullmatch(r"(?:pub\s+)?use\s+(\w+(?:::\w+)*)::(?:(\w+)|\{([\w\s,]+)\});", use_item.strip())
    if not shaped:
        return None
    path, single, group = shaped.groups()
    names = [single] if single else [name.strip() for name in group.split(",") if name.strip()]
    root, *modules = path.split("::")
    if root == "std":
        return names if set(names) <= STD_TYPES else None
    if root != "rakka_actor" or not modules:
        return None
    stem = CRATE_SOURCE + "/".join(modules)
    source = read_head(stem + ".rs") or read_head(stem + "/mod.rs") or ""
    declared = set(re.findall(r"\bpub\s+(?:struct|enum)\s+(\w+)", pr_selection.code_only(source)))
    return names if set(names) <= declared else None


def in_registration_list(item, position):
    """True when `position` stands directly inside the first `[` (array or `vec!`) of the body of the function that registers the
    scenarios (`rust_lines`)."""
    mask = pr_selection.LITERALS_AND_COMMENTS.sub(lambda m: "_" * len(m.group(0)), item)
    listed = mask.find("[", mask.find("{") + 1)
    if not re.search(r"\bfn rust_lines\b", item.split("{")[0]) or listed < 0:
        return False
    opened = []
    for index, character in enumerate(mask[:position]):
        if character in "([{":
            opened.append(index)
        elif character in ")]}" and opened:
            opened.pop()
    return bool(opened) and opened[-1] == listed


def only_adds_scenarios(base, head, read_head):
    """Every existing item is untouched, except that a case list may gain `("new label", new_fn(),)` entries; new items take no name the old
    code mentions and cannot reach it."""
    old, new = items(base), items(head)
    old_code = pr_selection.code_only(base)
    added, changed = [], []
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(a=old, b=new, autojunk=False).get_opcodes():
        if tag == "equal":
            continue
        remaining = new[j1:j2]
        for item in old[i1:i2]:
            header = item.split("{")[0]
            counterpart = next((candidate for candidate in remaining if candidate.split("{")[0] == header and "{" in item), None)
            if counterpart is None:
                return False
            remaining.remove(counterpart)
            changed.append((item, counterpart))
        added += remaining
    new_functions = set()
    for item in added:
        found = ITEM.match(item)
        if not found:
            return False
        kind, name = found.groups()
        taken = imported_types(item, read_head) if kind == "use" else [] if kind == "impl" else [name]
        if taken is None or any(re.search(rf"\b{each}\b", old_code) for each in taken):
            return False
        new_functions |= {name} if kind == "fn" else set()
    if pr_selection.reaches_existing_code(["\n".join(added)]):
        return False
    labels = set()

    def without_new_entries(counterpart):
        def drop(entry):
            label, call = entry.group("label"), entry.group("call")
            if call not in new_functions or label in labels or label in base or not in_registration_list(counterpart, counterpart.index("(", entry.start())):
                return entry.group(0)
            labels.add(label)
            return ""
        return ENTRY.sub(drop, counterpart)

    return all(without_new_entries(counterpart) == item for item, counterpart in changed)


def belongs_to_a_new_case(path, read_base, read_head):
    """A file under the recordings is part of a new case only: some dot-prefix of its first component names a `.toml` this change adds and
    none names one the base already had (a fixture or sidecar would change how that recorded case runs or is judged)."""
    parts = path[len(RECORDINGS):].split("/")[0].split(".")
    stems = [RECORDINGS + ".".join(parts[:count]) + ".toml" for count in range(1, len(parts) + 1)]
    return not any(read_base(stem) is not None for stem in stems) and any(read_head(stem) is not None for stem in stems)


def proven(path, base, head, read_base, read_head):
    """-> the mutants the file newly defines (usually none) when `path` is proven independent of the existing scenarios; None otherwise."""
    if head is None:
        return None
    if path.startswith(RECORDINGS):
        return [] if base is None and belongs_to_a_new_case(path, read_base, read_head) else None
    if path.startswith(CASES):
        if path.endswith("not-applicable.tsv"):      # it changes how an existing case is judged
            return None
        if base is not None and not lines_only_added(base, head):
            return None
        if path.startswith(CASES + "oracle-only/") and base is not None:      # the crate tests look lines up by their case id
            old_ids, new_ids = (collections.Counter(line.split()[0] for line in text.splitlines() if line.split()) for text in (base, head))
            # some tests look a line up with `starts_with`: an added id may neither repeat nor be a prefix of an existing one, nor the reverse
            if any(new.startswith(old) or old.startswith(new) for new in new_ids - old_ids for old in old_ids):
                return None
        return []
    if path.startswith(ORACLE):      # a verdict reads the recordings, never the oracle
        return [] if path.endswith(".java") and (base is None or lines_only_added(base, head)) else None
    if path.startswith(EXAMPLES):
        return [] if path.endswith(".rs") and base is not None and only_adds_scenarios(base, head, read_head) else None
    if path == HARNESS and base is not None:
        delta = shadow_selection.table_delta(base, head)
        return delta.new_or_changed if delta.table_only and not delta.removed else None
    return None
