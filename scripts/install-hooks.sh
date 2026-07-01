#!/usr/bin/env bash
# One-time setup: point git at the in-repo .githooks/ directory so local commits
# run fmt + clippy before being sent to origin (catches CI lint failures early).
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

git config core.hooksPath .githooks
chmod +x .githooks/pre-commit

echo "git hooks installed from .githooks/"
echo "  pre-commit: cargo fmt --check + cargo clippy -D warnings + cargo-sort"
echo
echo "To bypass in emergency: git commit --no-verify"
echo "To disable entirely:    git config --unset core.hooksPath"
