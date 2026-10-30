#!/usr/bin/env python3
"""Each inventory row has exactly one contract block (the heading that defines it) and the generated map and specs point to it.

The expected location is computed here from the contract headings, independently of the generator. Runs in a temporary copy.
"""
import csv, glob, os, re, subprocess, unittest
from package_sandbox import PackageSandbox

csv.field_size_limit(10**9)
CONTRACTS = "docs/inventory/contracts"
DEFINING_HEADING = re.compile(r"(?m)^#+ (A[CT]-[A-Z]+-\d+(?:\.\d+)?)\b")


def read_tsv(path):
    return list(csv.reader(open(path, encoding="utf-8", newline=""), delimiter="\t", quoting=csv.QUOTE_NONE))


class ContractLocation(PackageSandbox):
    def inventory_row_ids(self):
        ids = []
        for module in ("classic", "typed"):
            ids += [row[0] for row in read_tsv(os.path.join(self.root, f"docs/inventory/actor-{module}.tsv"))[1:]]
        return ids

    def defining_documents(self):
        documents = {}
        for path in sorted(glob.glob(os.path.join(self.root, CONTRACTS, "*.md"))):
            for match in DEFINING_HEADING.finditer(open(path, encoding="utf-8").read()):
                documents.setdefault(match.group(1), []).append(f"{CONTRACTS}/{os.path.basename(path)}")
        return documents

    def test_map_points_to_the_document_whose_heading_defines_each_row(self):
        self.regenerate()
        documents = self.defining_documents()
        contract_map = {row[0]: row[10] for row in read_tsv(os.path.join(self.root, "docs/contract-map.tsv"))[1:]}
        row_ids = self.inventory_row_ids()
        self.assertEqual(sorted(row_ids), sorted(contract_map))
        wrong = [(row_id, contract_map[row_id], documents[row_id]) for row_id in row_ids
                 if documents.get(row_id) != [contract_map[row_id]]]
        self.assertEqual([], wrong, f"{len(wrong)} rows whose map entry is not their single defining block")

    def test_task_references_list_every_document_of_their_rows(self):
        self.regenerate()
        documents = self.defining_documents()
        checked = 0
        for path in sorted(glob.glob(os.path.join(self.root, "docs/superpowers/plans/units/[0-9]*.md"))):
            for line in open(path, encoding="utf-8").read().splitlines():
                match = re.match(r"Filas: (.*?)\. Contratos y aserciones: (.*?) \(bloque", line)
                if not match:
                    continue
                rows = [row_id.strip() for row_id in match.group(1).split(",")]
                expected = sorted({documents[row_id][0] for row_id in rows})
                self.assertEqual(expected, sorted(re.findall(r"`([^`]+)`", match.group(2))), f"{os.path.basename(path)}: {rows}")
                checked += 1
        self.assertGreater(checked, 0)

    def test_row_without_a_contract_block_is_rejected(self):
        row_id = self.inventory_row_ids()[0]
        path = self.defining_documents()[row_id][0]
        text = self.read(path)
        self.write(path, re.sub(rf"(?m)^(#+) {re.escape(row_id)}\b", r"\1 (sin contrato)", text, count=1))
        with self.assertRaises(subprocess.CalledProcessError) as raised:
            self.regenerate()
        self.assertIn(row_id, raised.exception.stderr)

    def test_row_with_two_contract_blocks_is_rejected(self):
        row_id = self.inventory_row_ids()[0]
        other_document = next(p for p in glob.glob(os.path.join(self.root, CONTRACTS, "*.md")) if f"{CONTRACTS}/{os.path.basename(p)}" != self.defining_documents()[row_id][0])
        self.append(os.path.relpath(other_document, self.root), f"\n## {row_id} — bloque duplicado\n")
        with self.assertRaises(subprocess.CalledProcessError) as raised:
            self.regenerate()
        self.assertIn(row_id, raised.exception.stderr)


if __name__ == "__main__":
    unittest.main()
