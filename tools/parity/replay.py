#!/usr/bin/env python3
"""Replays every recorded Pekko 2.0.0 case against the Rust parity binary and reports
`N/N applicable + Q NA:Q-CFG-30 + K not applicable` (see replay_lib.py). Exit 0 only when all applicable cases match and
every NOT_APPLICABLE case is rejected explicitly.

  tools/parity/replay.py [binary]       replays that binary (default: target/release/examples/classic-parity)
  tools/parity/replay.py --build        compiles the default binary first (it cannot be combined with an explicit binary)
The replay always runs an immutable, content-addressed copy of the binary, made under the build lock (replay_lib.publish_binary).
Exit codes: 0 clean, 1 divergences, 2 refused usage / build failure / rejected tool output.
"""
import os, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import replay_lib as lib
import clean_replay_evidence as evidence


class UsageError(Exception):
    pass


def default_binary():
    return os.path.join(lib.ROOT, "target", "release", "examples", "classic-parity")


def prepare_binary(arguments):
    """-> the path of the immutable copy to replay. The build (with --build) and the copy happen under one build lock, so no other build can
    swap the file in between, and the copy is never taken from a half-written executable."""
    build = "--build" in arguments
    explicit = [argument for argument in arguments if argument != "--build"]
    if build and explicit:
        raise UsageError("--build compiles the default parity binary: it cannot be combined with an explicit binary")
    binary = explicit[0] if explicit else default_binary()
    with lib.build_lock():
        if build:
            subprocess.run(list(evidence.CLEAN_REPLAY_COMMAND), cwd=lib.ROOT, check=True)
        if not os.path.isfile(binary):
            raise UsageError(f"the binary does not exist: {binary}")
        return lib.publish_binary(binary)


def main(arguments):
    evidence.discard(evidence.evidence_path(lib.ROOT))      # a replay that fails or is interrupted must leave no evidence of an earlier one
    try:
        binary = prepare_binary(arguments)
    except UsageError as problem:
        print(f"refused: {problem}")
        return 2
    except subprocess.CalledProcessError as failure:
        print(f"refused: the build failed (exit {failure.returncode})")
        return 2
    with lib.holding(binary):         # the copy stays protected through the replay AND the classification (which runs the binary again)
        result = lib.run_replay(binary, 900)
        evaluation = lib.evaluate(binary, result)
        not_stored = evidence.leave_evidence(lib.ROOT, binary, getattr(result, "total", None), evaluation, os.environ)
    print("clean-replay evidence: " + ("stored for this gate run" if not_stored is None else f"not stored ({not_stored})"))
    if evaluation["ok"] is None:
        print("HARNESS ERROR:", evaluation["line"])
        return 2
    print(evaluation["line"])
    print(f"breakdown: pass {evaluation['passed']} / NA:Q-CFG-30 {len(evaluation['na_q_cfg_30'])} / other NA {evaluation['other_na']} / diverge {len(evaluation['applicable_failing'])}")
    for case in evaluation["na_q_cfg_30"]:
        print("NA:Q-CFG-30:", case)
    for case in evaluation["applicable_failing"]:
        print("DIVERGES:", case)
    for problem in evaluation["na_problems"]:
        print("NOT_APPLICABLE problem:", problem)
    return 0 if evaluation["ok"] else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
