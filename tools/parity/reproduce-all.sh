#!/usr/bin/env bash
# Reproduces slice 1 (ActorPath.fromString) end to end. Requires JDK 17, Maven, cargo, rust-anything.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
export JAVA_HOME="${JAVA_HOME:-/opt/homebrew/opt/openjdk@17}" PATH="${JAVA_HOME:-/opt/homebrew/opt/openjdk@17}/bin:$PATH"

echo "== 1. build the Pekko 2.0.0 oracle";          (cd oracle && mvn -q -B package)
echo "== 2. build the Rust port + unit tests";      cargo build --release -p rakka-actor --example classic-parity && cargo test --release --workspace
echo "== 3. generate and record cases with real Pekko"
python3 tools/parity/make_cases.py
python3 tools/parity/make_scenarios.py
python3 tools/parity/make_scenarios_2b.py
python3 tools/parity/make_scenarios_3a.py
python3 tools/parity/make_scenarios_3b1.py
python3 tools/parity/make_scenarios_3b2.py
ALL_CASES="$(mktemp)"; cat cases/path-from-string.jsonl cases/scenario-2a.jsonl cases/scenario-2b.jsonl cases/scenario-3a.jsonl cases/scenario-3b1.jsonl cases/scenario-3b2.jsonl > "$ALL_CASES"
rm -rf rust/app-cases
rust-anything app record . --inputs "$ALL_CASES" --no-traps --name classic-parity -- oracle/run.sh </dev/null
echo "== 4. replay the recording against the Rust binary"
python3.12 tools/parity/replay.py
echo "== 5. differential fuzz (JVM vs Rust), 3 x 100k inputs"
for seed in 1 2 3; do python3 tools/parity/fuzz_diff.py --count 100000 --seed "$seed"; done
echo "== 5b. differential fuzz of ActorSystem scenarios (JVM vs Rust), 3 x 1500"
for seed in 1 2 3; do python3 -u tools/parity/fuzz_scenarios.py --count 1500 --seed "$seed"; done
echo "== 5c. differential fuzz of tell/sender/forward/PoisonPill/dead letters (JVM vs Rust), 3 x 1000"
for seed in 1 2 3; do python3 -u tools/parity/fuzz_tell.py --count 1000 --seed "$seed"; done
echo "== 5d. differential fuzz of hierarchy / watch / unwatch / name reuse (JVM vs Rust), 3 x 1000"
for seed in 1 2 3; do python3 -u tools/parity/fuzz_hier.py --count 1000 --seed "$seed"; done
echo "== 6. deliberate divergences must be detected"
python3 tools/parity/check_divergence.py
