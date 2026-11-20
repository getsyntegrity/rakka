"""The control replay of check_divergence must run against a binary built from the current tree, never a leftover one."""
import unittest

import check_divergence as cd


class ControlPhase(unittest.TestCase):
    def setUp(self):
        self.saved = (cd.control_binary_for_fast_kill, cd.replay, cd.classify)

    def tearDown(self):
        cd.control_binary_for_fast_kill, cd.replay, cd.classify = self.saved

    def test_the_control_is_built_before_it_is_replayed(self):
        calls = []
        cd.control_binary_for_fast_kill = lambda: calls.append("build") or "/fresh/binary"
        cd.replay = lambda binary: calls.append(("replay", binary)) or object()
        cd.classify = lambda result, binary: (cd.UNDETECTED, 10, 10, "ok")
        ok, passed, total, line, _result = cd.run_control_replay()
        self.assertEqual(calls, ["build", ("replay", "/fresh/binary")])
        self.assertTrue(ok)

    def test_a_control_that_cannot_be_built_fails_and_is_never_replayed_against_a_stale_binary(self):
        calls = []
        cd.control_binary_for_fast_kill = lambda: None
        cd.replay = lambda binary: calls.append(binary)
        ok, _passed, _total, line, _result = cd.run_control_replay()
        self.assertFalse(ok)
        self.assertEqual(calls, [])
        self.assertIn("build", line)


if __name__ == "__main__":
    unittest.main()
