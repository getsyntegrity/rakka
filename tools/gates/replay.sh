#!/usr/bin/env bash
# Replay gate: builds the release parity binary the recording is replayed against, then replays every recorded Pekko 2.0.0 case.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
cargo build --release -p rakka-actor --example classic-parity
python3.12 tools/parity/replay.py
