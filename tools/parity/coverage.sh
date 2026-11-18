#!/usr/bin/env bash
# Line-coverage gate: >= 95 % per crate, no exclusions (llvm-cov's default ignores tests/ and examples/ files only).
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
cargo llvm-cov -p rakka-actor --lib --tests --fail-under-lines 95
# rakka-actor-typed has no implementation yet (0 instrumented lines); once it has, the same gate applies:
#   cargo llvm-cov -p rakka-actor-typed --lib --tests --fail-under-lines 95
