"""The mutant table must describe the code as it is now: a pattern that no longer applies exactly once is an ERROR, never a valid mutant.
Three of the four patterns that went stale with the mailbox refactors (#302, #304) were rewritten against the current code; the fourth
targets code that no longer exists and its behaviour is already covered by a live mutant (retiring it is the Owner's decision)."""
import os, sys, unittest

import mutant_runner as mr

sys.path.insert(0, os.path.join(mr.ROOT, "tools", "parity"))
import check_divergence as cd

REWRITTEN = ("mailbox-close-forgets-queued-messages", "throughput-ignored", "failure-does-not-suspend-mailbox")
STALE_PENDING_OWNER_DECISION = {
    # the old patch popped the newest message out of `mb.user` in cell.rs; that queue now lives in message_queue.rs and the same behaviour
    # (newest message first) is the live mutant `mailbox-queue-is-lifo`
    "mailbox-is-lifo": "mailbox-queue-is-lifo",
}


class RewrittenPatterns(unittest.TestCase):
    def test_each_rewritten_pattern_applies_exactly_once_to_the_current_source_and_changes_it(self):
        for name in REWRITTEN:
            relative_file, old, new, _why = cd.MUTANTS[name]
            with open(os.path.join(mr.ROOT, mr.CRATE, relative_file), encoding="utf-8") as source_file:
                source = source_file.read()
            self.assertEqual(source.count(old), 1, f"{name}: the pattern must occur exactly once in {relative_file}")
            self.assertNotEqual(source.replace(old, new), source, name)

    def test_the_only_stale_pattern_left_is_the_one_a_live_mutant_already_covers(self):
        stale = mr.find_stale_patterns(cd.MUTANTS, mr.ROOT) + mr.find_stale_patterns(cd.TOOL_MUTANTS, mr.ROOT, "tools/parity/")
        self.assertEqual(sorted(stale), sorted(STALE_PENDING_OWNER_DECISION))
        for covering_mutant in STALE_PENDING_OWNER_DECISION.values():
            self.assertIn(covering_mutant, cd.MUTANTS)
            self.assertNotIn(covering_mutant, stale)

    def test_a_stale_pattern_is_still_an_error_for_the_runner(self):
        self.assertEqual(mr.exit_code(undetected=[], errors=["mailbox-is-lifo"], pending=[], control="CLEAN"), 2)


if __name__ == "__main__":
    unittest.main()
