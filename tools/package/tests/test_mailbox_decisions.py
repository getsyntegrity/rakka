#!/usr/bin/env python3
"""ADR-0001 mailbox decisions as the Owner answered them on 2026-11-20: only Q-MBX-8, Q-MBX-9 and Q-MBX-10 are accepted.

Q-MBX-1, 2, 3, 5 and 6 stay proposed until the Owner answers them. Reads the generated register of the working tree.
"""
import csv, io, os, unittest

REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
ACCEPTED = ("Q-MBX-8", "Q-MBX-9", "Q-MBX-10")
STILL_PROPOSED = ("Q-MBX-1", "Q-MBX-2", "Q-MBX-3", "Q-MBX-5", "Q-MBX-6")


def register():
    text = open(os.path.join(REPO, "docs/decisions/register.tsv"), encoding="utf-8", newline="").read()
    return {r["id"]: r for r in csv.DictReader(io.StringIO(text, newline=""), delimiter="\t", quoting=csv.QUOTE_NONE)}


class MailboxDecisions(unittest.TestCase):
    def test_owner_answers_are_registered_with_date_and_author(self):
        reg = register()
        for qid in ACCEPTED:
            self.assertEqual("accepted", reg[qid]["status"], qid)
            self.assertEqual("2026-11-20", reg[qid]["decided_on"], qid)
            self.assertTrue(reg[qid]["decided_by"].startswith("Owner"), qid)
            self.assertTrue(reg[qid]["decision"], qid)

    def test_the_other_mailbox_decisions_are_not_approved(self):
        reg = register()
        for qid in STILL_PROPOSED:
            self.assertEqual("proposed", reg[qid]["status"], qid)
            self.assertFalse(reg[qid].get("decided_on"), qid)


if __name__ == "__main__":
    unittest.main()
