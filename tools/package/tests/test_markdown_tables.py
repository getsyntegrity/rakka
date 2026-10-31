#!/usr/bin/env python3
"""A literal `|` inside a cell of a generated Markdown table is escaped, so every row keeps the header's column count.

Runs in a temporary copy of the documentation package; the real repository is never modified.
"""
import csv, glob, os, re, unittest
from package_sandbox import PackageSandbox

csv.field_size_limit(10**9)
UNESCAPED_PIPE = re.compile(r"(?<!\\)\|")
TRACE_HEADER = "| Fila | Comportamiento |"


def trace_rows(text):
    """Lines of every traceability table: the header and the rows after it."""
    inside = False
    for line in text.splitlines():
        if line.startswith(TRACE_HEADER):
            inside = True
        elif inside and not line.startswith("|"):
            inside = False
        if inside:
            yield line


class MarkdownTables(PackageSandbox):
    def assert_every_trace_row_has_the_header_columns(self):
        checked = 0
        for path in sorted(glob.glob(os.path.join(self.root, "docs/superpowers/plans/units/[0-9]*.md"))):
            rows = list(trace_rows(open(path, encoding="utf-8").read()))
            columns = len(UNESCAPED_PIPE.findall(rows[0])) if rows else 0
            for row in rows:
                self.assertEqual(columns, len(UNESCAPED_PIPE.findall(row)), f"{os.path.basename(path)}: {row[:100]}")
                checked += 1
        self.assertGreater(checked, 0)

    def test_generated_rows_keep_their_columns(self):
        self.regenerate()
        self.assert_every_trace_row_has_the_header_columns()

    def test_a_pipe_in_a_cell_is_escaped(self):
        path = os.path.join(self.root, "docs/inventory/actor-classic.tsv")
        rows = list(csv.reader(open(path, encoding="utf-8", newline=""), delimiter="\t", quoting=csv.QUOTE_NONE))
        header = rows[0]
        target = next(r for r in rows[1:] if r[header.index("impl")] != "done")
        target[header.index("rust_equivalent")] = "`Timeout(a|b)` con |x| y"
        with open(path, "w", encoding="utf-8", newline="") as out:
            out.writelines("\t".join(r) + "\n" for r in rows)
        self.regenerate()
        self.assert_every_trace_row_has_the_header_columns()
        generated = "".join(open(p, encoding="utf-8").read() for p in glob.glob(os.path.join(self.root, "docs/superpowers/plans/units/[0-9]*.md")))
        self.assertIn(r"`Timeout(a\|b)` con \|x\| y", generated)


if __name__ == "__main__":
    unittest.main()
