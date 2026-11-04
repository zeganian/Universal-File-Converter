#!/usr/bin/env bash
#
# Regenerates the TypeScript bindings from the Rust domain types.
#
# The bindings are produced by the `ts-rs` export tests inside
# `crates/ufc-core`. `TS_RS_EXPORT_DIR` is configured in
# `.cargo/config.toml` and points at `src/types/generated`, so the
# generated files land next to the code that consumes them.
#
# Usage: pnpm generate:types

set -euo pipefail

cd "$(dirname "$0")/.."

# The export tests are named `export_bindings_*`; the filter keeps the
# regular unit tests from running.
cargo test -p ufc-core --lib export_bindings

echo "TypeScript bindings regenerated in src/types/generated."
