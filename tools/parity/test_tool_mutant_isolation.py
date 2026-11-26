"""A mutant of tools/parity runs the tests of the tools on a COPY of them. The tests find their repository from their own location (the data under cases/ and rust/, the sibling tools/gates), so the
copy must sit in a mirror of the repository: otherwise the unmutated copy already fails and every tool mutant is either a vacuous DETECTED (tests broken for a reason that has nothing to do with the mutation) or,
when the failure is an import error, a HARNESS_ERROR."""
import os
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import check_divergence

NESTED_RUN = "RAKKA_TOOL_MUTANT_ISOLATION_TEST"       # the copy under test contains this very test: it must not start itself again
UNCHANGED_LINE = "same_double = oracle.double() is not None and oracle.double() == rakka.double()  # (a)"


class ToolMutantsRunTheToolTestsInAMirrorOfTheRepository(unittest.TestCase):
    @unittest.skipIf(os.environ.get(NESTED_RUN), "the copy under test; the outer run already checks it")
    def test_a_tool_mutant_that_changes_nothing_observable_is_undetected_because_the_unmutated_tests_pass_in_the_copy(self):
        with tempfile.TemporaryDirectory(prefix="pekko-tool-mutants-") as scratch, mock.patch.dict(os.environ, {NESTED_RUN: "1"}):
            outcome, _passed, _total, detail = check_divergence.run_tool_mutant(
                scratch, "tool-mutant-that-changes-nothing", "na_q_cfg_30.py", UNCHANGED_LINE, UNCHANGED_LINE + "  # unchanged", "no observable change")
        self.assertEqual(check_divergence.UNDETECTED, outcome, f"the unmutated tests of the tools must pass in the copy; the run ended: {detail}")


if __name__ == "__main__":
    unittest.main()
