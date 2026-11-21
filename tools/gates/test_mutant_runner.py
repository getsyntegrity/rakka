"""Tests of the budgeted, resumable mutant runner: ordering, evidence invalidation, budget, authorization, resume, cancellation, summary."""
import os, signal, subprocess, sys, tempfile, time, unittest

import mutant_runner as mr

HERE = os.path.dirname(os.path.abspath(__file__))


def write(root, rel, text):
    path = os.path.join(root, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as fh:
        fh.write(text)


def read_text(path):
    with open(path) as fh:
        return fh.read()


def mini_repo(root):
    write(root, "crates/rakka-actor/src/lib.rs", "pub mod actor;\npub mod dispatch;\npub mod util;\n")
    write(root, "crates/rakka-actor/src/actor/mod.rs", "pub mod cell;\n")
    write(root, "crates/rakka-actor/src/actor/cell.rs", "use crate::dispatch::queue::Queue;\n")
    write(root, "crates/rakka-actor/src/dispatch/mod.rs", "pub mod queue;\n")
    write(root, "crates/rakka-actor/src/dispatch/queue.rs", "// leaf\n")
    write(root, "crates/rakka-actor/src/util/mod.rs", "pub mod clock;\n")
    write(root, "crates/rakka-actor/src/util/clock.rs", "// unrelated\n")


class ImportGraph(unittest.TestCase):
    def test_use_and_mod_declarations_become_undirected_edges(self):
        with tempfile.TemporaryDirectory() as root:
            mini_repo(root)
            edges = mr.import_edges(root)
            cell, queue = "crates/rakka-actor/src/actor/cell.rs", "crates/rakka-actor/src/dispatch/queue.rs"
            self.assertIn(queue, edges[cell])
            self.assertIn(cell, edges[queue])
            self.assertNotIn("crates/rakka-actor/src/util/clock.rs", edges[cell])

    def test_a_type_name_that_differs_from_a_file_only_by_case_is_not_a_module(self):
        with tempfile.TemporaryDirectory() as root:
            mini_repo(root)
            write(root, "crates/rakka-actor/src/util/timeout.rs", "// module\n")
            write(root, "crates/rakka-actor/src/actor/user.rs", "use crate::util::Timeout;\n")
            edges = mr.import_edges(root)   # `Timeout` is a type: on a case-insensitive filesystem it must not resolve to timeout.rs
            self.assertNotIn("crates/rakka-actor/src/util/timeout.rs", edges["crates/rakka-actor/src/actor/user.rs"])


class StalePatterns(unittest.TestCase):
    def test_a_mutant_whose_pattern_no_longer_occurs_exactly_once_is_stale_and_never_valid(self):
        with tempfile.TemporaryDirectory() as root:
            write(root, "crates/rakka-actor/src/a.rs", "let x = 1;\\nlet y = 2;\\nlet y = 2;\\n")
            table = {"ok": ("src/a.rs", "let x = 1;", "let x = 2;", "w"), "gone": ("src/a.rs", "let z = 3;", "n", "w"),
                     "ambiguous": ("src/a.rs", "let y = 2;", "n", "w"), "missing-file": ("src/none.rs", "x", "n", "w")}
            self.assertEqual(mr.find_stale_patterns(table, root), ["gone", "ambiguous", "missing-file"])

    def test_stale_mutants_are_reported_as_errors_and_keep_the_run_from_passing(self):
        outcomes = {"gone": "STALE_PATTERN", "fine": "DETECTED"}
        errors = [n for n, o in outcomes.items() if o not in ("DETECTED", "UNDETECTED")]
        self.assertEqual(errors, ["gone"])
        self.assertEqual(mr.exit_code(undetected=[], errors=errors, pending=[], control="CLEAN"), 2)


class Ordering(unittest.TestCase):
    def test_changed_file_first_then_its_neighbours_then_the_rest(self):
        files = {"m-cell": "crates/rakka-actor/src/actor/cell.rs", "m-queue": "crates/rakka-actor/src/dispatch/queue.rs", "m-clock": "crates/rakka-actor/src/util/clock.rs"}
        edges = {files["m-cell"]: {files["m-queue"]}, files["m-queue"]: {files["m-cell"]}, files["m-clock"]: set()}
        order = mr.order_mutants(list(files), files, [files["m-cell"]], edges)
        self.assertEqual(order, [("m-cell", "changed"), ("m-queue", "neighbour"), ("m-clock", "other")])


class EvidenceInvalidation(unittest.TestCase):
    def setUp(self):
        self.base = dict(name="m", patch=("a.rs", "old", "new"), crate_state="c1", harness="h1", config={"fast_kill": True}, toolchain="rustc 1")

    def test_same_inputs_same_key(self):
        self.assertEqual(mr.mutant_key(**self.base), mr.mutant_key(**self.base))

    def test_every_relevant_input_invalidates(self):
        reference = mr.mutant_key(**self.base)
        for change in (dict(name="other"), dict(patch=("a.rs", "old", "newer")), dict(crate_state="c2"), dict(harness="h2"),
                       dict(config={"fast_kill": False}), dict(toolchain="rustc 2")):
            self.assertNotEqual(reference, mr.mutant_key(**{**self.base, **change}), change)

    def test_a_change_in_a_file_outside_the_mutated_files_neighbourhood_still_changes_the_mutant_key(self):
        with tempfile.TemporaryDirectory() as root:
            mini_repo(root)
            write(root, "Cargo.toml", "[workspace]\n")
            write(root, "Cargo.lock", "# lock\n")
            def key():
                return mr.mutant_key("m", ("src/actor/cell.rs", "o", "n"), mr.crate_fingerprint(root), "h", {"fast_kill": True}, "rustc 1")
            before = key()
            write(root, "crates/rakka-actor/src/util/clock.rs", "// edited, not imported by the mutated file\\n")
            self.assertNotEqual(before, key())

    def test_the_build_inputs_outside_the_crate_directory_change_the_crate_fingerprint(self):
        with tempfile.TemporaryDirectory() as root:
            mini_repo(root)
            write(root, "Cargo.toml", "[workspace]\\n")
            write(root, "Cargo.lock", "# lock 1\\n")
            before = mr.crate_fingerprint(root)
            write(root, "Cargo.lock", "# lock 2\\n")
            self.assertNotEqual(before, mr.crate_fingerprint(root))
            write(root, "Cargo.lock", "# lock 1\\n")
            self.assertEqual(before, mr.crate_fingerprint(root))
            write(root, "crates/rakka-actor/target/debug/noise", "build output")
            self.assertEqual(before, mr.crate_fingerprint(root))   # the build directory is not an input

    def test_the_control_key_reuses_the_same_crate_fingerprint(self):
        with tempfile.TemporaryDirectory() as root:
            mini_repo(root)
            write(root, "Cargo.toml", "[workspace]\\n")
            write(root, "Cargo.lock", "# lock 1\\n")
            before = mr.control_key(root, "h", "rustc 1")
            write(root, "Cargo.lock", "# lock 2\\n")
            self.assertNotEqual(before, mr.control_key(root, "h", "rustc 1"))

    def test_only_a_detected_result_is_fresh_errors_and_misses_are_retried(self):
        with tempfile.TemporaryDirectory() as root:
            store = mr.ResultStore(root)
            store.put("k1", {"outcome": "DETECTED"})
            store.put("k2", {"outcome": "BUILD_ERROR"})
            store.put("k3", {"outcome": "UNDETECTED"})
            self.assertTrue(store.is_fresh("k1"))
            self.assertFalse(store.is_fresh("k2"))
            self.assertFalse(store.is_fresh("k3"))
            self.assertFalse(store.is_fresh("missing"))
            self.assertEqual(store.get("k3")["outcome"], "UNDETECTED")


class BudgetAndAuthorization(unittest.TestCase):
    def test_full_run_without_authorization_is_refused(self):
        self.assertIn("authorize", mr.authorization_error(max_mutants=206, max_minutes=30, authorized=False))
        self.assertIsNone(mr.authorization_error(max_mutants=mr.DEFAULT_MAX_MUTANTS, max_minutes=mr.DEFAULT_MAX_MINUTES, authorized=False))
        self.assertIsNone(mr.authorization_error(max_mutants=206, max_minutes=100000, authorized=True))

    def test_raising_the_time_cap_needs_the_same_authorization_as_raising_the_count_cap(self):
        self.assertIn("authorize", mr.authorization_error(max_mutants=10, max_minutes=100000, authorized=False))
        self.assertIsNone(mr.authorization_error(max_mutants=10, max_minutes=mr.DEFAULT_MAX_MINUTES, authorized=False))

    def test_the_time_of_one_mutant_is_clamped_to_the_remaining_budget(self):
        self.assertEqual(mr.mutant_timeout(remaining_seconds=100), 100)
        self.assertEqual(mr.mutant_timeout(remaining_seconds=99999), mr.PER_MUTANT_TIMEOUT)
        self.assertEqual(mr.mutant_timeout(remaining_seconds=None), mr.PER_MUTANT_TIMEOUT)
        self.assertEqual(mr.mutant_timeout(remaining_seconds=0), 1)

    def run_plan(self, store, names, **kw):
        ran = []
        def execute(name):
            ran.append(name)
            return {"outcome": "DETECTED", "seconds": 1.0}
        clock = kw.pop("clock", lambda: 0.0)
        report = mr.run_plan([(n, "other") for n in names], {n: "key-" + n for n in names}, store, execute, clock=clock, **kw)
        return ran, report

    def test_count_budget_stops_and_reports_pending(self):
        with tempfile.TemporaryDirectory() as root:
            ran, report = self.run_plan(mr.ResultStore(root), ["a", "b", "c", "d"], max_mutants=2, max_seconds=None)
            self.assertEqual(ran, ["a", "b"])
            self.assertEqual((report["ran"], report["pending"], report["stopped_by"]), (2, ["c", "d"], "count budget"))

    def test_time_budget_stops_before_starting_the_next_mutant(self):
        with tempfile.TemporaryDirectory() as root:
            ticks = iter([0, 10, 50, 70, 70, 70])
            ran, report = self.run_plan(mr.ResultStore(root), ["a", "b", "c"], max_mutants=None, max_seconds=60, clock=lambda: next(ticks))
            self.assertEqual(ran, ["a", "b"])
            self.assertEqual(report["stopped_by"], "time budget")

    def test_a_resumed_run_skips_what_a_previous_run_already_proved(self):
        with tempfile.TemporaryDirectory() as root:
            store = mr.ResultStore(root)
            first, _ = self.run_plan(store, ["a", "b", "c"], max_mutants=2, max_seconds=None)
            second, report = self.run_plan(store, ["a", "b", "c"], max_mutants=2, max_seconds=None)
            self.assertEqual((first, second), (["a", "b"], ["c"]))
            self.assertEqual(report["pending"], [])
            self.assertEqual(report["fresh_before"], 2)

    def test_an_interrupted_run_keeps_the_results_already_written(self):
        with tempfile.TemporaryDirectory() as root:
            store = mr.ResultStore(root)
            def execute(name):
                if name == "b":
                    raise KeyboardInterrupt
                return {"outcome": "DETECTED", "seconds": 1.0}
            with self.assertRaises(KeyboardInterrupt):
                mr.run_plan([(n, "other") for n in "abc"], {n: "key-" + n for n in "abc"}, store, execute, clock=lambda: 0.0, max_mutants=None, max_seconds=None)
            self.assertTrue(store.is_fresh("key-a"))
            self.assertFalse(store.is_fresh("key-b"))


class ControlVerification(unittest.TestCase):
    def test_a_control_that_never_passed_is_run_even_when_no_mutant_ran(self):
        with tempfile.TemporaryDirectory() as root:
            store, calls = mr.ResultStore(root), []
            self.assertEqual(mr.ensure_control(store, "k", lambda: calls.append(1) or 0), "CLEAN")
            self.assertEqual(calls, [1])

    def test_a_control_that_already_passed_for_this_state_is_not_repeated(self):
        with tempfile.TemporaryDirectory() as root:
            store, calls = mr.ResultStore(root), []
            mr.ensure_control(store, "k", lambda: calls.append(1) or 0)
            self.assertEqual(mr.ensure_control(store, "k", lambda: calls.append(1) or 0), "CLEAN")
            self.assertEqual(calls, [1])

    def test_a_failing_control_is_reported_failed_and_retried_next_time(self):
        with tempfile.TemporaryDirectory() as root:
            store = mr.ResultStore(root)
            self.assertEqual(mr.ensure_control(store, "k", lambda: 1), "FAILED")
            self.assertEqual(mr.ensure_control(store, "k", lambda: 0), "CLEAN")

    def test_a_control_cut_by_the_time_budget_is_unverified_not_failed_and_is_not_stored(self):
        with tempfile.TemporaryDirectory() as root:
            store = mr.ResultStore(root)
            self.assertEqual(mr.ensure_control(store, "k", lambda: mr.TIMED_OUT, budget_limited=True), "UNVERIFIED")
            self.assertFalse(store.is_fresh("control-k"))
            self.assertEqual(mr.ensure_control(store, "k2", lambda: mr.TIMED_OUT, budget_limited=False), "ERROR")   # a hung control is an invalid run, not a regression

    def test_only_an_exit_of_1_means_the_control_found_a_divergence_every_other_abnormal_exit_is_an_error(self):
        for abnormal in (2, 143, 137, 77):
            with tempfile.TemporaryDirectory() as root:
                store = mr.ResultStore(root)
                self.assertEqual(mr.ensure_control(store, "k", lambda code=abnormal: code), "ERROR", abnormal)
                self.assertEqual(os.listdir(root), [])                  # an invalid run leaves no verdict behind
        with tempfile.TemporaryDirectory() as root:
            self.assertEqual(mr.ensure_control(mr.ResultStore(root), "k", lambda: 1), "FAILED")

    def test_a_control_error_is_exit_2_but_a_surviving_mutant_still_wins(self):
        self.assertEqual(mr.exit_code(undetected=[], errors=[], pending=[], control="ERROR"), 2)
        self.assertEqual(mr.exit_code(undetected=["x"], errors=[], pending=[], control="ERROR"), 1)

    def test_with_an_erroring_control_no_mutant_runs(self):
        with tempfile.TemporaryDirectory() as root:
            started = []
            control, report = mr.run_guarded(mr.ResultStore(root), "k", lambda: 2, lambda: started.append(1), budget_limited=False)
            self.assertEqual((control, report, started), ("ERROR", None, []))

    def test_an_unverified_control_makes_the_run_incomplete_not_a_failure(self):
        self.assertEqual(mr.exit_code(undetected=[], errors=[], pending=[], control="UNVERIFIED"), 3)
        self.assertEqual(mr.exit_code(undetected=["x"], errors=[], pending=[], control="UNVERIFIED"), 1)

    def test_an_interrupted_control_is_not_recorded_as_passed(self):
        with tempfile.TemporaryDirectory() as root:
            store = mr.ResultStore(root)
            def interrupted():
                raise KeyboardInterrupt
            with self.assertRaises(KeyboardInterrupt):
                mr.ensure_control(store, "k", interrupted)
            self.assertFalse(store.is_fresh("control-k"))


class MutantResultCollection(unittest.TestCase):
    def test_a_result_file_left_by_an_earlier_run_is_never_read_as_the_new_outcome(self):
        with tempfile.TemporaryDirectory() as root:
            result_file = os.path.join(root, "m.json")
            with open(result_file, "w") as fh:
                fh.write('{"mutants": {"m": ["DETECTED", null, null, "old run"]}, "control": null}')
            record = mr.execute_mutant("m", result_file, lambda: 137, lambda: 5.0)   # the child died without writing a new file
            self.assertEqual(record["outcome"], "HARNESS_ERROR")
            self.assertFalse(os.path.exists(result_file))

    def test_the_outcome_the_child_wrote_this_time_is_used(self):
        with tempfile.TemporaryDirectory() as root:
            result_file = os.path.join(root, "m.json")
            def child():
                with open(result_file, "w") as fh:
                    fh.write('{"mutants": {"m": ["DETECTED", null, null, "fast-kill: case x"]}, "control": null}')
                return 0
            record = mr.execute_mutant("m", result_file, child, lambda: 1.0)
            self.assertEqual((record["outcome"], record["line"]), ("DETECTED", "fast-kill: case x"))

    def test_a_timeout_is_reported_as_timeout(self):
        with tempfile.TemporaryDirectory() as root:
            self.assertEqual(mr.execute_mutant("m", os.path.join(root, "m.json"), lambda: mr.TIMED_OUT, lambda: 9.0)["outcome"], "TIMEOUT")


class BudgetKill(unittest.TestCase):
    def test_a_mutant_cut_by_the_time_budget_is_pending_and_never_stored(self):
        with tempfile.TemporaryDirectory() as root:
            store = mr.ResultStore(root)
            def execute(name):
                return {"outcome": "BUDGET_EXHAUSTED", "seconds": 1.0} if name == "b" else {"outcome": "DETECTED", "seconds": 1.0}
            report = mr.run_plan([(n, "other") for n in "abc"], {n: "key-" + n for n in "abc"}, store, execute, clock=lambda: 0.0, max_mutants=None, max_seconds=60)
            self.assertEqual((report["ran"], report["pending"], report["stopped_by"]), (1, ["b", "c"], "time budget"))
            self.assertIsNone(store.get("key-b"))
            self.assertNotIn("b", report["outcomes"])

    def test_the_clamped_timeout_firing_means_budget_exhausted_while_the_normal_one_is_a_timeout(self):
        with tempfile.TemporaryDirectory() as root:
            result_file = os.path.join(root, "m.json")
            self.assertEqual(mr.execute_mutant("m", result_file, lambda: mr.TIMED_OUT, lambda: 3.0, budget_limited=True)["outcome"], "BUDGET_EXHAUSTED")
            self.assertEqual(mr.execute_mutant("m", result_file, lambda: mr.TIMED_OUT, lambda: 3.0, budget_limited=False)["outcome"], "TIMEOUT")

    def test_a_timeout_counts_as_budget_only_when_the_time_given_was_less_than_a_normal_mutant_needs(self):
        self.assertTrue(mr.is_budget_limited(remaining_seconds=100))
        self.assertTrue(mr.is_budget_limited(remaining_seconds=mr.GENUINE_TIMEOUT_SECONDS - 1))
        self.assertFalse(mr.is_budget_limited(remaining_seconds=mr.GENUINE_TIMEOUT_SECONDS))
        self.assertFalse(mr.is_budget_limited(remaining_seconds=None))
        self.assertFalse(mr.is_budget_limited(remaining_seconds=mr.PER_MUTANT_TIMEOUT + 1))

    def test_a_hung_mutant_given_the_default_budget_is_a_timeout_error_not_pending_forever(self):
        default_budget = mr.DEFAULT_MAX_MINUTES * 60
        self.assertGreaterEqual(default_budget - 5, mr.GENUINE_TIMEOUT_SECONDS)   # a mutant started at the beginning of a default run
        with tempfile.TemporaryDirectory() as root:
            record = mr.execute_mutant("m", os.path.join(root, "m.json"), lambda: mr.TIMED_OUT, lambda: 1790.0,
                                       budget_limited=mr.is_budget_limited(default_budget - 5))
            self.assertEqual(record["outcome"], "TIMEOUT")


class ControlFirst(unittest.TestCase):
    def guarded(self, store, control_code, budget_limited=False):
        started = []
        control, report = mr.run_guarded(store, "k", lambda: control_code, lambda: started.append(1) or {"ran": 1}, budget_limited=budget_limited)
        return control, report, started

    def test_with_a_clean_control_the_mutants_run_after_it(self):
        with tempfile.TemporaryDirectory() as root:
            control, report, started = self.guarded(mr.ResultStore(root), 0)
            self.assertEqual((control, report, started), ("CLEAN", {"ran": 1}, [1]))

    def test_with_a_failing_control_no_mutant_runs_so_nothing_can_be_stored_as_detected(self):
        with tempfile.TemporaryDirectory() as root:
            control, report, started = self.guarded(mr.ResultStore(root), 1)
            self.assertEqual((control, report, started), ("FAILED", None, []))

    def test_with_a_control_the_budget_could_not_finish_no_mutant_runs_and_the_run_is_incomplete(self):
        with tempfile.TemporaryDirectory() as root:
            control, report, started = self.guarded(mr.ResultStore(root), mr.TIMED_OUT, budget_limited=True)
            self.assertEqual((control, report, started), ("UNVERIFIED", None, []))

    def test_a_detected_result_written_while_the_control_was_failing_cannot_exist(self):
        with tempfile.TemporaryDirectory() as root:
            store = mr.ResultStore(root)
            self.guarded(store, 1)
            self.assertFalse(store.is_fresh("control-k"))
            self.assertEqual([name for name in os.listdir(root) if not name.startswith("control-")], [])


class InputsChangingDuringTheRun(unittest.TestCase):
    def plan(self, store, key_now, names="ab"):
        ran = []
        def execute(name):
            ran.append(name)
            return {"outcome": "DETECTED", "seconds": 1.0}
        report = mr.run_plan([(n, "other") for n in names], {n: "key-" + n for n in names}, store, execute, clock=lambda: 0.0,
                             max_mutants=None, max_seconds=None, key_now=key_now)
        return ran, report

    def test_a_result_is_not_stored_when_the_inputs_changed_while_the_mutant_ran(self):
        with tempfile.TemporaryDirectory() as root:
            store, calls = mr.ResultStore(root), {"a": 0}
            def key_now(name):   # same before the run, different after it
                calls[name] = calls.get(name, 0) + 1
                return "key-" + name if calls[name] == 1 else "edited-" + name
            ran, report = self.plan(store, key_now, names="a")
            self.assertEqual(ran, ["a"])
            self.assertEqual(report["outcomes"]["a"], "INPUTS_CHANGED")
            self.assertIsNone(store.get("key-a"))
            self.assertIsNone(store.get("edited-a"))

    def test_a_mutant_whose_inputs_already_changed_before_it_starts_is_not_run(self):
        with tempfile.TemporaryDirectory() as root:
            ran, report = self.plan(mr.ResultStore(root), lambda name: "edited-" + name, names="a")
            self.assertEqual(ran, [])
            self.assertEqual(report["outcomes"]["a"], "INPUTS_CHANGED")

    def test_unchanged_inputs_store_the_result_as_before(self):
        with tempfile.TemporaryDirectory() as root:
            store = mr.ResultStore(root)
            ran, report = self.plan(store, lambda name: "key-" + name)
            self.assertEqual(ran, ["a", "b"])
            self.assertTrue(store.is_fresh("key-a") and store.is_fresh("key-b"))

    def test_inputs_changed_is_an_error_that_keeps_the_run_from_passing(self):
        errors = [n for n, o in {"a": "INPUTS_CHANGED"}.items() if o not in ("DETECTED", "UNDETECTED")]
        self.assertEqual(mr.exit_code(undetected=[], errors=errors, pending=[], control="CLEAN"), 2)


class CachedEvidenceIsRevalidated(unittest.TestCase):
    def plan(self, store, key_now):
        executed = []
        report = mr.run_plan([("a", "other")], {"a": "key-a"}, store, lambda name: executed.append(name) or {"outcome": "DETECTED", "seconds": 1.0},
                             clock=lambda: 0.0, max_mutants=None, max_seconds=None, key_now=key_now)
        return executed, report

    def test_a_cached_detected_is_not_accepted_when_the_current_key_is_different(self):
        with tempfile.TemporaryDirectory() as root:
            store = mr.ResultStore(root)
            store.put("key-a", {"outcome": "DETECTED"})
            executed, report = self.plan(store, lambda name: "key-for-edited-code")
            self.assertEqual(executed, [])                      # a resumed run that executes nothing...
            self.assertEqual(report["outcomes"]["a"], "INPUTS_CHANGED")   # ...still refuses the stale evidence
            self.assertEqual(report["fresh_before"], 0)

    def test_a_cached_detected_with_the_same_current_key_is_still_reused(self):
        with tempfile.TemporaryDirectory() as root:
            store = mr.ResultStore(root)
            store.put("key-a", {"outcome": "DETECTED"})
            executed, report = self.plan(store, lambda name: "key-a")
            self.assertEqual((executed, report["outcomes"]["a"], report["fresh_before"]), ([], "DETECTED", 1))

    def test_the_verified_control_must_still_match_the_state_the_verdict_is_issued_on(self):
        outcomes = {"a": "DETECTED"}
        self.assertEqual(mr.settle_control(outcomes, "control-key-1", "control-key-1"), {"a": "DETECTED"})
        settled = mr.settle_control(outcomes, "control-key-1", "control-key-2")
        self.assertEqual(settled["(control)"], "INPUTS_CHANGED")
        errors = [n for n, o in settled.items() if o not in ("DETECTED", "UNDETECTED")]
        self.assertEqual(mr.exit_code(undetected=[], errors=errors, pending=[], control="CLEAN"), 2)
        self.assertEqual(outcomes, {"a": "DETECTED"})          # the input mapping is not modified


class RunIsolation(unittest.TestCase):
    def test_every_run_gets_its_own_directory_even_with_the_same_clock_and_pid(self):
        with tempfile.TemporaryDirectory() as root:
            first = mr.new_run_directory(root, clock=lambda: 1000.0, pid=lambda: 7)
            second = mr.new_run_directory(root, clock=lambda: 1000.0, pid=lambda: 7)
            self.assertNotEqual(first, second)
            self.assertTrue(os.path.isdir(first) and os.path.isdir(second))
            self.assertEqual(os.path.dirname(first), os.path.dirname(second))

    def test_two_runs_of_the_same_mutant_do_not_delete_overwrite_or_read_each_others_result(self):
        with tempfile.TemporaryDirectory() as root:
            run_a, run_b = (mr.new_run_directory(root) for _ in range(2))
            file_a, file_b = mr.mutant_result_file(run_a, "m"), mr.mutant_result_file(run_b, "m")
            self.assertNotEqual(file_a, file_b)
            with open(file_a, "w") as fh:
                fh.write('{"mutants": {"m": ["DETECTED", null, null, "run A"]}, "control": null}')
            record_b = mr.execute_mutant("m", file_b, lambda: 137, lambda: 1.0)   # run B's child dies without writing
            self.assertEqual(record_b["outcome"], "HARNESS_ERROR")                 # B never reads A's file
            self.assertTrue(os.path.exists(file_a))                                # and does not delete it
            self.assertIn("run A", read_text(file_a))

    def test_resumable_checkpoints_stay_shared_between_runs_and_are_written_atomically(self):
        with tempfile.TemporaryDirectory() as root:
            first, second = mr.ResultStore(root), mr.ResultStore(root)
            first.put("k", {"outcome": "DETECTED"})
            self.assertTrue(second.is_fresh("k"))
            self.assertEqual([name for name in os.listdir(root) if name.endswith(".tmp")], [])


class BudgetSharedWithTheReplayLockWait(unittest.TestCase):
    def test_the_runner_and_the_parity_tools_agree_on_the_budget_exit_code_and_the_deadline_variable(self):
        sys.path.insert(0, os.path.join(mr.ROOT, "tools", "parity"))
        import replay_lib
        self.assertEqual(mr.BUDGET_EXHAUSTED_EXIT, replay_lib.EXIT_BUDGET_EXHAUSTED)
        self.assertEqual(mr.REPLAY_DEADLINE_ENV, replay_lib.REPLAY_DEADLINE_ENV)

    def test_children_get_the_absolute_deadline_of_the_run_budget(self):
        environment = mr.child_environment(run_started=1000.0, budget_seconds=1800, base={"PATH": "/bin"})
        self.assertEqual(float(environment[mr.REPLAY_DEADLINE_ENV]), 2800.0)
        self.assertEqual(environment["PATH"], "/bin")
        self.assertNotIn(mr.REPLAY_DEADLINE_ENV, mr.child_environment(run_started=1000.0, budget_seconds=None, base={"PATH": "/bin"}))

    def test_run_cancellable_hands_the_environment_to_the_child(self):
        with tempfile.TemporaryDirectory() as root:
            out = os.path.join(root, "seen")
            code = mr.run_cancellable(["sh", "-c", f'printf %s "$RAKKA_REPLAY_DEADLINE" > {out}'], 10, os.path.join(root, "log"),
                                      environment={**os.environ, mr.REPLAY_DEADLINE_ENV: "1234.5"})
            self.assertEqual(code, 0)
            self.assertEqual(read_text(out), "1234.5")

    def test_a_child_that_gave_up_waiting_for_the_replay_lock_leaves_the_mutant_pending_even_without_a_clamped_timeout(self):
        with tempfile.TemporaryDirectory() as root:
            record = mr.execute_mutant("m", os.path.join(root, "m.json"), lambda: mr.BUDGET_EXHAUSTED_EXIT, lambda: 3.0, budget_limited=False)
            self.assertEqual(record["outcome"], "BUDGET_EXHAUSTED")

    def test_a_control_that_gave_up_waiting_for_the_replay_lock_is_unverified_and_not_stored(self):
        with tempfile.TemporaryDirectory() as root:
            store = mr.ResultStore(root)
            self.assertEqual(mr.ensure_control(store, "k", lambda: mr.BUDGET_EXHAUSTED_EXIT, budget_limited=False), "UNVERIFIED")
            self.assertEqual(os.listdir(root), [])


class RecoveryAfterAKilledProcess(unittest.TestCase):
    DEAD, ALIVE = 2_000_000_000, os.getpid()

    def alive(self, pid):
        return pid == self.ALIVE

    def test_a_partial_checkpoint_of_a_dead_process_is_removed_and_everything_else_is_kept(self):
        with tempfile.TemporaryDirectory() as store, tempfile.TemporaryDirectory() as scratch:
            for name in (f"abc.{self.DEAD}.tmp", f"abc.{self.ALIVE}.tmp", "abc.json", ".registry.lock", "control-k.json", "notes.tmp"):
                write(store, name, "x")
            report = mr.recover_after_crash(store, scratch, pid_alive=self.alive)
            self.assertEqual(report["checkpoints_removed"], 1)
            self.assertEqual(sorted(os.listdir(store)), sorted([f"abc.{self.ALIVE}.tmp", "abc.json", ".registry.lock", "control-k.json", "notes.tmp"]))

    def test_scratch_directories_are_removed_only_when_their_owner_is_dead(self):
        with tempfile.TemporaryDirectory() as store, tempfile.TemporaryDirectory() as scratch:
            write(scratch, f"pekko-mutants-dead/{mr.SCRATCH_OWNER_MARKER}", str(self.DEAD))
            write(scratch, f"pekko-mutants-alive/{mr.SCRATCH_OWNER_MARKER}", str(self.ALIVE))
            write(scratch, "pekko-mutants-unmarked/readme", "a scratch of an older tool version: owner unknown, left alone")
            write(scratch, "pekko-mutants-garbled/" + mr.SCRATCH_OWNER_MARKER, "not a pid")
            write(scratch, "some-other-directory/file", "not ours")
            report = mr.recover_after_crash(store, scratch, pid_alive=self.alive)
            self.assertEqual(report["scratch_removed"], 1)
            self.assertEqual(sorted(os.listdir(scratch)), ["pekko-mutants-alive", "pekko-mutants-garbled", "pekko-mutants-unmarked", "some-other-directory"])

    def test_staging_files_of_a_dead_publisher_are_removed_from_the_binary_store(self):
        with tempfile.TemporaryDirectory() as store, tempfile.TemporaryDirectory() as scratch, tempfile.TemporaryDirectory() as binaries:
            write(binaries, f".staging-{self.DEAD}-ab12.tmp", "half a binary")
            write(binaries, f".staging-{self.ALIVE}-cd34.tmp", "being published right now")
            write(binaries, "0" * 64 + "/classic-parity", "a published binary")
            report = mr.recover_after_crash(store, scratch, pid_alive=self.alive, binaries_directory=binaries)
            self.assertEqual(report["staging_removed"], 1)
            self.assertEqual(sorted(os.listdir(binaries)), sorted([f".staging-{self.ALIVE}-cd34.tmp", "0" * 64]))

    def test_a_run_killed_with_sigkill_is_recovered_by_a_later_run_while_an_active_run_is_protected(self):
        sys.path.insert(0, os.path.join(mr.ROOT, "tools", "parity"))
        import check_divergence
        with tempfile.TemporaryDirectory() as root:
            runs, store, scratch = (os.path.join(root, name) for name in ("runs", "store", "scratch"))
            os.makedirs(store)
            os.makedirs(scratch)
            alive_run = mr.new_run_directory(runs)                                   # this (living) process owns a run...
            alive_scratch = check_divergence.make_scratch_directory(scratch)         # ...and a scratch directory
            driver = (f"import os, signal, sys; sys.path.insert(0, {HERE!r}); sys.path.insert(0, {os.path.join(mr.ROOT, 'tools', 'parity')!r});"
                      f"import mutant_runner as mr, check_divergence as cd;"
                      f"run = mr.new_run_directory({runs!r}); open(os.path.join({store!r}, 'key.' + str(os.getpid()) + '.tmp'), 'w').write('{{');"
                      f"cd.make_scratch_directory({scratch!r}); print(run, flush=True); os.kill(os.getpid(), signal.SIGKILL)")
            killed = subprocess.run([sys.executable, "-c", driver], capture_output=True, text=True)
            self.assertEqual(killed.returncode, -signal.SIGKILL)
            killed_run = killed.stdout.strip()
            self.assertTrue(os.path.isdir(killed_run))
            report = mr.recover_after_crash(store, scratch)
            self.assertEqual((report["checkpoints_removed"], report["scratch_removed"]), (1, 1))
            self.assertEqual(os.listdir(scratch), [os.path.basename(alive_scratch)])            # the living run's scratch is protected
            removed = mr.prune_run_directories(runs, keep=0)                                   # the OS released the killed run's marker...
            self.assertEqual(removed, [os.path.basename(killed_run)])                          # ...so it is just a finished run now
            self.assertTrue(os.path.isdir(alive_run))                                          # and the active one is never pruned

    def test_a_scratch_directory_made_by_check_divergence_carries_its_owner(self):
        sys.path.insert(0, os.path.join(mr.ROOT, "tools", "parity"))
        import check_divergence
        with tempfile.TemporaryDirectory() as parent:
            path = check_divergence.make_scratch_directory(parent)
            self.assertTrue(os.path.basename(path).startswith("pekko-mutants-"))
            self.assertEqual(read_text(os.path.join(path, check_divergence.SCRATCH_OWNER_MARKER)), str(os.getpid()))
            self.assertEqual(check_divergence.SCRATCH_OWNER_MARKER, mr.SCRATCH_OWNER_MARKER)


class RunDirectoryPruning(unittest.TestCase):
    def test_only_the_newest_run_directories_are_kept_and_nothing_else_is_touched(self):
        with tempfile.TemporaryDirectory() as root:
            runs = os.path.join(root, "runs")
            names = [f"20260101T0000{index:02d}-1-aaaa" for index in range(15)]
            for name in names:
                os.makedirs(os.path.join(runs, name))
            write(root, "keep-me/unrelated.txt", "not a run directory")
            removed = mr.prune_run_directories(runs, keep=5)
            self.assertEqual(sorted(name for name in os.listdir(runs) if name != mr.REGISTRY_LOCK_FILE), names[-5:])
            self.assertEqual(sorted(removed), names[:10])
            self.assertTrue(os.path.exists(os.path.join(root, "keep-me", "unrelated.txt")))

    def test_a_run_that_is_still_active_keeps_its_directory_even_when_more_than_ten_runs_start_after_it(self):
        import fcntl
        with tempfile.TemporaryDirectory() as root:
            runs = os.path.join(root, "runs")
            names = [f"20260101T0000{index:02d}-1-aaaa" for index in range(15)]
            for name in names:
                os.makedirs(os.path.join(runs, name))
            active_marker = open(os.path.join(runs, names[0], mr.ACTIVE_RUN_MARKER), "w")   # the oldest run is still running
            try:
                fcntl.flock(active_marker, fcntl.LOCK_EX)
                removed = mr.prune_run_directories(runs, keep=5)
                self.assertIn(names[0], os.listdir(runs))                       # the active run keeps its directory
                self.assertNotIn(names[0], removed)
                self.assertEqual(sorted(name for name in os.listdir(runs) if name != mr.REGISTRY_LOCK_FILE),
                                 [names[0]] + names[-5:])   # the limit applies to the finished runs only
            finally:
                active_marker.close()
            self.assertIn(names[0], mr.prune_run_directories(runs, keep=5))   # once it finished it is prunable like any other

    def test_a_concurrent_prune_cannot_delete_a_run_between_creating_its_directory_and_marking_it_active(self):
        import threading
        with tempfile.TemporaryDirectory() as root:
            pruned = []
            def prune_while_the_new_run_is_half_registered():
                # this is the instant the review reproduced: the directory exists, its `.active` marker does not yet
                pruner = threading.Thread(target=lambda: pruned.extend(mr.prune_run_directories(root, keep=0)))
                pruner.start()
                time.sleep(0.4)          # long enough for an unsynchronised prune to run to completion
                prune_while_the_new_run_is_half_registered.pruner = pruner
            run = mr.new_run_directory(root, after_directory_created=prune_while_the_new_run_is_half_registered)
            prune_while_the_new_run_is_half_registered.pruner.join(timeout=10)
            self.assertFalse(prune_while_the_new_run_is_half_registered.pruner.is_alive())
            self.assertTrue(os.path.isdir(run), "the prune deleted the run while it was being registered")
            self.assertTrue(os.path.exists(os.path.join(run, mr.ACTIVE_RUN_MARKER)))
            self.assertNotIn(os.path.basename(run), pruned)

    def test_pruning_still_removes_old_finished_runs_when_it_does_not_race_with_a_registration(self):
        with tempfile.TemporaryDirectory() as root:
            for index in range(12):
                os.makedirs(os.path.join(root, f"20260101T0000{index:02d}-1-aaaa"))
            self.assertEqual(len(mr.prune_run_directories(root, keep=10)), 2)
            self.assertEqual(len(os.listdir(root)), 10 + (1 if os.path.exists(os.path.join(root, mr.REGISTRY_LOCK_FILE)) else 0))

    def test_a_new_run_directory_is_marked_active_for_as_long_as_the_process_lives(self):
        import fcntl
        with tempfile.TemporaryDirectory() as root:
            run = mr.new_run_directory(root)
            probe = open(os.path.join(run, mr.ACTIVE_RUN_MARKER), "w")
            try:
                with self.assertRaises(BlockingIOError):
                    fcntl.flock(probe, fcntl.LOCK_EX | fcntl.LOCK_NB)
            finally:
                probe.close()

    def test_a_directory_that_is_not_a_run_is_neither_pruned_nor_counted(self):
        with tempfile.TemporaryDirectory() as root:
            runs = os.path.join(root, "runs")
            os.makedirs(os.path.join(runs, "foreign-directory"))
            for index in range(3):
                os.makedirs(os.path.join(runs, f"20260101T0000{index:02d}-1-aaaa"))
            mr.prune_run_directories(runs, keep=1)
            self.assertIn("foreign-directory", os.listdir(runs))

    def test_pruning_a_missing_or_small_directory_does_nothing(self):
        with tempfile.TemporaryDirectory() as root:
            self.assertEqual(mr.prune_run_directories(os.path.join(root, "absent"), keep=5), [])
            os.makedirs(os.path.join(root, "runs", "only-one"))
            self.assertEqual(mr.prune_run_directories(os.path.join(root, "runs"), keep=5), [])

    def test_the_changed_files_come_from_the_shared_parser_and_a_bad_base_is_a_usage_error(self):
        with self.assertRaises(mr.UsageError):
            mr.git_changed("origin/this-ref-does-not-exist-anywhere")


class OptionValues(unittest.TestCase):
    def test_a_value_that_is_another_flag_or_missing_is_a_usage_error(self):
        for argv in (["--max-mutants", "--list"], ["--max-mutants"], ["--base", "--list"], ["--base"]):
            cast = str if argv[0] == "--base" else int
            with self.assertRaises(mr.UsageError, msg=argv):
                mr.option(argv, argv[0], None, cast)
        self.assertEqual(mr.option(["--base", "origin/x"], "--base", None, str), "origin/x")


class HarnessFingerprint(unittest.TestCase):
    def build(self, root, runner_text):
        write(root, "tools/parity/check_divergence.py", "MUTANTS = {}\nTOOL_MUTANTS = {}\n")
        write(root, "cases/a.jsonl", "{}\n")
        write(root, "rust/app-cases/a.stdout", "x")
        write(root, "Cargo.toml", "[workspace]\n")
        write(root, "Cargo.lock", "")
        write(root, "crates/rakka-actor/Cargo.toml", "[package]\n")
        write(root, "tools/gates/mutant_runner.py", runner_text)

    def test_the_runner_s_own_source_is_part_of_the_fingerprint_so_no_manual_version_bump_is_needed(self):
        with tempfile.TemporaryDirectory() as root:
            self.build(root, "# stricter classification\n")
            before = mr.harness_fingerprint(root)
            write(root, "tools/gates/mutant_runner.py", "# stricter classification, edited\n")
            self.assertNotEqual(before, mr.harness_fingerprint(root))

    def test_the_replay_report_the_gates_rewrite_does_not_change_the_fingerprint(self):
        with tempfile.TemporaryDirectory() as root:
            self.build(root, "# runner\n")
            write(root, "rust/app-cases/replay-report.md", "first")
            before = mr.harness_fingerprint(root)
            write(root, "rust/app-cases/replay-report.md", "second")
            self.assertEqual(before, mr.harness_fingerprint(root))


class ControlKey(unittest.TestCase):
    def test_any_crate_file_the_toolchain_or_the_harness_changes_the_control_key(self):
        with tempfile.TemporaryDirectory() as root:
            mini_repo(root)
            write(root, "crates/rakka-actor/Cargo.toml", "[package]\\n")
            write(root, "crates/rakka-actor/src/deep/no_mutants.rs", "// no mutant targets this file\\n")
            reference = mr.control_key(root, "h", "rustc 1")
            self.assertEqual(reference, mr.control_key(root, "h", "rustc 1"))
            self.assertNotEqual(reference, mr.control_key(root, "h", "rustc 2"))
            self.assertNotEqual(reference, mr.control_key(root, "h2", "rustc 1"))
            write(root, "crates/rakka-actor/src/deep/no_mutants.rs", "// edited\\n")
            self.assertNotEqual(reference, mr.control_key(root, "h", "rustc 1"))


class Summary(unittest.TestCase):
    def test_summary_is_compact_even_with_hundreds_of_problems(self):
        text = mr.summary_text(total=206, detected=100, undetected=[f"u{i}" for i in range(80)], errors=[f"e{i}" for i in range(26)], pending=[], stopped_by=None, control="CLEAN")
        self.assertLessEqual(len(text.splitlines()), 12)
        self.assertLessEqual(len(text), 1500)
        self.assertIn("100/206", text)

    def test_exit_code_distinguishes_partial_from_failure(self):
        self.assertEqual(mr.exit_code(undetected=[], errors=[], pending=[], control="CLEAN"), 0)
        self.assertEqual(mr.exit_code(undetected=["x"], errors=[], pending=[], control="CLEAN"), 1)
        self.assertEqual(mr.exit_code(undetected=[], errors=["x"], pending=[], control="CLEAN"), 2)
        self.assertEqual(mr.exit_code(undetected=[], errors=[], pending=["x"], control="CLEAN"), 3)
        self.assertEqual(mr.exit_code(undetected=[], errors=[], pending=[], control="FAILED"), 1)


class Verdicts(unittest.TestCase):
    def test_exit_codes_map_to_the_proposed_q_x_20_verdicts(self):
        self.assertEqual([mr.verdict(c) for c in (0, 1, 2, 3, 143)], ["PASS", "FAIL", "ERROR", "INCOMPLETE", "CANCELLED"])

    def test_an_unknown_exit_code_is_an_error_never_a_pass(self):
        self.assertEqual(mr.verdict(77), "ERROR")

    def test_ctrl_c_is_a_cancellation_not_a_traceback(self):
        driver = (f"import sys; sys.path.insert(0, {HERE!r}); import mutant_runner as mr;"
                  "mr.main = lambda argv: (_ for _ in ()).throw(KeyboardInterrupt()); mr.run_as_script([])")
        result = subprocess.run([sys.executable, "-c", driver], capture_output=True, text=True)
        self.assertEqual(result.returncode, 143)
        self.assertIn("VERDICT CANCELLED", result.stdout)
        self.assertNotIn("Traceback", result.stderr)

    def test_an_unexpected_exception_is_an_error_exit_2_with_the_traceback_not_a_failed_mutation_run(self):
        driver = (f"import sys; sys.path.insert(0, {HERE!r}); import mutant_runner as mr;"
                  "mr.main = lambda argv: (_ for _ in ()).throw(OSError('disk full')); mr.run_as_script([])")
        result = subprocess.run([sys.executable, "-c", driver], capture_output=True, text=True)
        self.assertEqual(result.returncode, 2)
        self.assertIn("VERDICT ERROR", result.stdout)
        self.assertIn("disk full", result.stderr)        # the traceback is kept for diagnosis

    def test_a_cancelled_runner_prints_cancelled_and_exits_143(self):
        driver = (f"import sys; sys.path.insert(0, {HERE!r}); import mutant_runner as mr;"
                  "mr.main = lambda argv: sys.exit(143); mr.run_as_script([])")
        result = subprocess.run([sys.executable, "-c", driver], capture_output=True, text=True)
        self.assertEqual(result.returncode, 143)
        self.assertIn("VERDICT CANCELLED", result.stdout)


class Cancellation(unittest.TestCase):
    def test_terminating_the_runner_kills_the_child_and_its_grandchildren(self):
        with tempfile.TemporaryDirectory() as root:
            pid_file = os.path.join(root, "grandchild.pid")
            driver = (f"import sys; sys.path.insert(0, {HERE!r}); import mutant_runner as mr, signal;"
                      "signal.signal(signal.SIGTERM, lambda *_: sys.exit(143));"
                      f"mr.run_cancellable(['sh', '-c', 'sleep 300 & echo $! > {pid_file}; wait'], 600, {os.path.join(root, 'out.log')!r})")
            runner = subprocess.Popen([sys.executable, "-c", driver])
            for _ in range(100):
                if os.path.exists(pid_file) and read_text(pid_file).strip():
                    break
                time.sleep(0.05)
            grandchild = int(read_text(pid_file))
            runner.send_signal(signal.SIGTERM)
            runner.wait(timeout=20)
            for _ in range(100):
                try:
                    os.kill(grandchild, 0)
                except ProcessLookupError:
                    return
                time.sleep(0.05)
            os.kill(grandchild, signal.SIGKILL)
            self.fail("the grandchild survived the cancellation")

    def test_a_descendant_that_ignores_sigterm_is_killed_after_the_group_leader_exits(self):
        with tempfile.TemporaryDirectory() as root:
            pid_file = os.path.join(root, "stubborn.pid")
            script = f"(trap '' TERM; exec sleep 300) & echo $! > {pid_file}; sleep 300"
            code = mr.run_cancellable(["sh", "-c", script], 1, os.path.join(root, "out.log"))
            self.assertEqual(code, mr.TIMED_OUT)
            stubborn = int(read_text(pid_file))
            for _ in range(100):
                try:
                    os.kill(stubborn, 0)
                except ProcessLookupError:
                    return
                time.sleep(0.05)
            os.kill(stubborn, signal.SIGKILL)
            self.fail("a descendant that ignores SIGTERM survived the leader")

    def test_a_process_outside_the_group_is_never_signalled(self):
        with tempfile.TemporaryDirectory() as root:
            bystander = subprocess.Popen(["sleep", "300"])
            try:
                mr.run_cancellable(["sleep", "30"], 1, os.path.join(root, "out.log"))
                self.assertIsNone(bystander.poll())
            finally:
                bystander.kill()
                bystander.wait()

    def test_a_child_that_exceeds_its_timeout_is_killed_and_reported(self):
        with tempfile.TemporaryDirectory() as root:
            code = mr.run_cancellable(["sleep", "30"], 1, os.path.join(root, "out.log"))
            self.assertEqual(code, mr.TIMED_OUT)


if __name__ == "__main__":
    unittest.main()
