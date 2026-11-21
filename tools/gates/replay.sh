#!/usr/bin/env bash
# Replay gate: builds the release parity binary (under the build lock) and replays every recorded Pekko 2.0.0 case against an immutable copy of it.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
python3.12 tools/parity/replay.py --build
