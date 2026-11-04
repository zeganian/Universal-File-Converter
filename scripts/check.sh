#!/usr/bin/env bash
#
# Runs every check that CI runs, in the same order.
#
# Usage: pnpm check

set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Rust formatting"
cargo fmt --all -- --check

echo "==> Rust lint"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> Rust tests"
cargo test --workspace

echo "==> TypeScript bindings"
./scripts/check-types-drift.sh

echo "==> Prettier"
pnpm format:check

echo "==> ESLint"
pnpm lint

echo "==> Type check"
pnpm typecheck

echo "==> Frontend tests"
pnpm test

echo "All checks passed."
