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


def imported_names(use_item):
    """The names a `use` brings into scope; None when it cannot be told (a glob, `self`)."""
    if "*" in use_item or re.search(r"\bself\b", use_item):
        return None
    grouped = re.search(r"\{(.*)\}", use_item, re.S)
    paths = grouped.group(1).split(",") if grouped else [use_item.rstrip(";")]
    return [re.split(r"::|\s+as\s+", path.strip())[-1] for path in paths if path.strip()]


def only_adds_scenarios(base, head):
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
        taken = imported_names(item) if kind == "use" else [] if kind == "impl" else [name]
        if taken is None or any(re.search(rf"\b{each}\b", old_code) for each in taken):
            return False
        new_functions |= {name} if kind == "fn" else set()
    if pr_selection.reaches_existing_code(["\n".join(added)]):
        return False
    labels = set()

    def drop_new_entry(entry):
        label, call = entry.group("label"), entry.group("call")
        if call not in new_functions or label in labels or label in base:
            return entry.group(0)
        labels.add(label)
        return ""

    return all(ENTRY.sub(drop_new_entry, counterpart) == item for item, counterpart in changed)


def proven(path, base, head):
    """-> the mutants the file newly defines (usually none) when `path` is proven independent of the existing scenarios; None otherwise."""
    if head is None:
        return None
    if path.startswith(RECORDINGS):
        return [] if base is None else None
    if path.startswith(CASES):
        if path.endswith("not-applicable.tsv"):      # it changes how an existing case is judged
            return None
        if base is not None and not lines_only_added(base, head):
            return None
        if path.startswith(CASES + "oracle-only/") and base is not None:      # the crate tests look lines up by their case id
            old_ids, new_ids = (collections.Counter(line.split()[0] for line in text.splitlines() if line.split()) for text in (base, head))
            if set(new_ids - old_ids) & set(old_ids):      # an added line repeats an existing id
                return None
        return []
    if path.startswith(ORACLE):      # a verdict reads the recordings, never the oracle
        return [] if path.endswith(".java") and (base is None or lines_only_added(base, head)) else None
    if path.startswith(EXAMPLES):
        return [] if path.endswith(".rs") and base is not None and only_adds_scenarios(base, head) else None
    if path == HARNESS and base is not None:
        delta = shadow_selection.table_delta(base, head)
        return delta.new_or_changed if delta.table_only and not delta.removed else None
    return None
