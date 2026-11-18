#!/usr/bin/env python3
"""The Behavior::Deferred contract does not propose a FnOnce factory (it contradicts reuse and restart) and the choice stays unaccepted."""
import os, re, unittest
from package_sandbox import REPO


def read(relative):
    return open(os.path.join(REPO, relative), encoding="utf-8").read()


class DeferredContract(unittest.TestCase):
    def test_no_document_proposes_a_fnonce_deferred_factory(self):
        for relative in ("docs/inventory/contracts/typed-core.md", "docs/inventory/actor-typed.tsv"):
            self.assertIsNone(re.search(r"Deferred\(Box<dyn FnOnce", read(relative)), relative)

    def test_observable_requirements_are_documented_and_the_representation_decision_is_the_owners(self):
        block = read("docs/inventory/contracts/typed-core.md")
        start = block.index("## AT-BEH-007")
        section = block[start:block.index("\n## ", start + 5)]
        for requirement in ("(R1)", "(R2)", "(R3)", "Q-T-BEH-1"):
            self.assertIn(requirement, section)
        register = {line.split("\t")[0]: line.split("\t")[2] for line in read("docs/decisions/register.tsv").splitlines()[1:]}
        self.assertEqual(register["Q-T-BEH-1"], "accepted")   # answered by the Owner on 2026-11-18 (lote 01)


if __name__ == "__main__":
    unittest.main()
