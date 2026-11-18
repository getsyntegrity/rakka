#!/usr/bin/env python3
"""Replays every recorded Pekko 2.0.0 case against the Rust parity binary and reports
`N/N applicable + K not applicable` (see replay_lib.py). Exit 0 only when all applicable cases match and
every NOT_APPLICABLE case is rejected explicitly.

  tools/parity/replay.py [binary]      (default: target/release/examples/classic-parity)
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import replay_lib as lib

binary = sys.argv[1] if len(sys.argv) > 1 else os.path.join(lib.ROOT, "target", "release", "examples", "classic-parity")
res = lib.run_replay(binary, 900)
ev = lib.evaluate(binary, res)
if ev["ok"] is None:
    print("HARNESS ERROR:", ev["line"]); sys.exit(2)
print(ev["line"])
for c in ev["applicable_failing"]: print("DIVERGES:", c)
for p in ev["na_problems"]: print("NOT_APPLICABLE problem:", p)
sys.exit(0 if ev["ok"] else 1)
