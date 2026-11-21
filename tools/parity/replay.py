#!/usr/bin/env python3
"""Replays every recorded Pekko 2.0.0 case against the Rust parity binary and reports
`N/N applicable + Q NA:Q-CFG-30 + K not applicable` (see replay_lib.py). Exit 0 only when all applicable cases match and
every NOT_APPLICABLE case is rejected explicitly.

  tools/parity/replay.py [--build] [binary]   (default: target/release/examples/classic-parity; --build compiles it first)
The replay always runs an immutable, content-addressed copy of the binary (replay_lib.publish_binary).
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import replay_lib as lib

import subprocess
arguments = [a for a in sys.argv[1:] if a != "--build"]
binary = arguments[0] if arguments else os.path.join(lib.ROOT, "target", "release", "examples", "classic-parity")
if "--build" in sys.argv[1:]:       # build and publish are one step under the build lock; the replay then uses the immutable copy
    with lib.build_lock():
        subprocess.run(["cargo", "build", "--release", "-p", "rakka-actor", "--example", "classic-parity"], cwd=lib.ROOT, check=True)
        binary = lib.publish_binary(binary)
else:
    binary = lib.publish_binary(binary)
res = lib.run_replay(binary, 900)
ev = lib.evaluate(binary, res)
if ev["ok"] is None:
    print("HARNESS ERROR:", ev["line"]); sys.exit(2)
print(ev["line"])
print(f"breakdown: pass {ev['passed']} / NA:Q-CFG-30 {len(ev['na_q_cfg_30'])} / other NA {ev['other_na']} / diverge {len(ev['applicable_failing'])}")
for c in ev["na_q_cfg_30"]: print("NA:Q-CFG-30:", c)
for c in ev["applicable_failing"]: print("DIVERGES:", c)
for p in ev["na_problems"]: print("NOT_APPLICABLE problem:", p)
sys.exit(0 if ev["ok"] else 1)
