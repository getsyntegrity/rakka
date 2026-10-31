#!/usr/bin/env python3
"""Q-UTIL-1..5 are five independent decisions; Q-X-9 only groups them for presentation and is never answered as a block.

The Owner responses used here are test fixtures written into a temporary copy; no real decision is recorded.
"""
import csv, os, re, subprocess, unittest
from package_sandbox import PackageSandbox, OWNER_FILE, tree_digest

csv.field_size_limit(10**9)
MEMBERS = ["Q-UTIL-1", "Q-UTIL-2", "Q-UTIL-3", "Q-UTIL-4", "Q-UTIL-5"]
FIRST_RESPONSE = "Q-UTIL-1\taccepted\t2031-03-04\tfixture-owner-a\taceptar la recomendación de la fixture A\n"
SECOND_RESPONSE = "Q-UTIL-2\trejected\t2031-03-05\tfixture-owner-b\trechazar: alternativa (b) de la fixture B\n"


class IndependentDecisions(PackageSandbox):
    def register(self):
        lines = self.read("docs/decisions/register.tsv").splitlines()
        header = lines[0].split("\t")
        return {fields[0]: dict(zip(header, fields)) for fields in (line.split("\t") for line in lines[1:])}

    def status_counts(self):
        counts = {}
        for entry in self.register().values():
            counts[entry["status"]] = counts.get(entry["status"], 0) + 1
        return counts

    def test_five_decisions_exist_and_the_group_is_not_a_decision(self):
        self.regenerate()
        register = self.register()
        for member in MEMBERS:
            self.assertEqual("proposed", register[member]["status"], member)
            self.assertEqual("", register[member]["decided_on"], member)
        self.assertEqual("group", register["Q-X-9"]["status"])
        for member in MEMBERS:
            self.assertIn(member, register["Q-X-9"]["recommendation"] + register["Q-X-9"]["title"])
        self.assertIn(f"{len(register) - 1} decisiones", self.read("README.md"))

    def test_two_different_answers_persist_across_two_regenerations(self):
        self.regenerate()
        before = self.status_counts()
        self.append(OWNER_FILE, FIRST_RESPONSE + SECOND_RESPONSE)

        self.regenerate()
        after_first = tree_digest(self.root)
        self.regenerate()
        self.assertEqual(after_first, tree_digest(self.root))

        register = self.register()
        first, second = register["Q-UTIL-1"], register["Q-UTIL-2"]
        self.assertEqual(("accepted", "2031-03-04", "fixture-owner-a", "aceptar la recomendación de la fixture A"),
                         (first["status"], first["decided_on"], first["decided_by"], first["decision"]))
        self.assertEqual(("rejected", "2031-03-05", "fixture-owner-b", "rechazar: alternativa (b) de la fixture B"),
                         (second["status"], second["decided_on"], second["decided_by"], second["decision"]))
        for untouched in MEMBERS[2:]:
            self.assertEqual(("proposed", ""), (register[untouched]["status"], register[untouched]["decided_on"]), untouched)
        self.assertEqual(("group", ""), (register["Q-X-9"]["status"], register["Q-X-9"]["decided_on"]))

        counts = self.status_counts()
        self.assertEqual(before["proposed"] - 2, counts["proposed"])
        self.assertEqual((1, 1), (counts["accepted"], counts["rejected"]))
        expected = (f"{sum(n for status, n in counts.items() if status != 'group')} decisiones: {counts['blocking']} bloqueantes, "
                    f"{counts['proposed']} propuestas, 1 aceptadas, 0 aceptadas con cambios, 1 rechazadas")
        for document in ("README.md", "docs/decisions.md", "docs/implementation-order.md"):
            self.assertIn(expected, self.read(document), document)
        decisions_doc = self.read("docs/decisions.md")
        self.assertIn("fixture-owner-a, 2031-03-04", decisions_doc)
        self.assertIn("fixture-owner-b, 2031-03-05", decisions_doc)

    def test_a_rejection_that_names_the_chosen_alternative_closes_only_that_decision(self):
        self.regenerate()
        before = self.status_counts()
        answer = "rechazar: (a) no se adopta. ALTERNATIVA ELEGIDA Y VIGENTE: (b) mínima con (c)"
        self.append(OWNER_FILE, f"Q-UTIL-1\trejected\t2031-03-04\tfixture-owner-a\t{answer}\n")
        self.regenerate()
        entry = self.register()["Q-UTIL-1"]
        self.assertEqual(("rejected", answer), (entry["status"], entry["decision"]))
        self.assertIn("ALTERNATIVA ELEGIDA Y VIGENTE", self.read("docs/decisions.md"))
        counts = self.status_counts()
        self.assertEqual((before["proposed"] - 1, 1), (counts["proposed"], counts["rejected"]))
        for other in MEMBERS[1:]:
            self.assertEqual("proposed", self.register()[other]["status"], other)

    def test_answering_the_group_is_rejected(self):
        self.append(OWNER_FILE, "Q-X-9\taccepted\t2031-03-04\tfixture-owner-a\taceptar todas\n")
        with self.assertRaises(subprocess.CalledProcessError) as raised:
            self.regenerate()
        self.assertIn("Q-X-9", raised.exception.stderr)

    def test_a_response_cannot_collapse_two_members(self):
        self.append(OWNER_FILE, FIRST_RESPONSE + FIRST_RESPONSE)
        with self.assertRaises(subprocess.CalledProcessError) as raised:
            self.regenerate()
        self.assertIn("duplicate response for Q-UTIL-1", raised.exception.stderr)


if __name__ == "__main__":
    unittest.main()
