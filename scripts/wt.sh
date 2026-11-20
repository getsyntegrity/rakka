#!/usr/bin/env bash
# wt.sh — corre comandos siempre desde la raíz del worktree donde vive este script,
# sin depender del cwd de quien lo invoca.
#
# Uso:
#   scripts/wt.sh status          git status -sb + rama + HEAD
#   scripts/wt.sh init            codegraph init en el worktree
#   scripts/wt.sh gates           fmt, clippy, tests, deny, check.py (AGENTS.md)
#   scripts/wt.sh shell           shell interactivo en la raíz del worktree
#   scripts/wt.sh run <cmd...>    cualquier comando desde la raíz
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
cd "$ROOT"

cmd="${1:-status}"
[ "$#" -gt 0 ] && shift

case "$cmd" in
  status)
    echo "root:   $ROOT"
    echo "branch: $(git branch --show-current)"
    echo "head:   $(git log --oneline -1)"
    git status -sb
    ;;
  init)
    codegraph init
    ;;
  gates)
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo deny check licenses
    python3.12 tools/package/check.py
    ;;
  shell)
    echo "worktree: $ROOT"
    exec "${SHELL:-/bin/zsh}" -i
    ;;
  run)
    [ "$#" -gt 0 ] || { echo "uso: wt.sh run <cmd...>" >&2; exit 2; }
    "$@"
    ;;
  *)
    echo "subcomando desconocido: $cmd (status|init|gates|shell|run)" >&2
    exit 2
    ;;
esac
