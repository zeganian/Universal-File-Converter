#!/usr/bin/env bash
#
# Fails when the committed TypeScript bindings differ from the ones the
# Rust domain types produce.
#
# The generated bindings are committed on purpose, so the repository has
# to contain exactly what the Rust types describe.
#
# Usage: pnpm check:types-drift

set -euo pipefail

cd "$(dirname "$0")/.."

./scripts/generate-types.sh >/dev/null

status=0

# Regenerated files that differ from the committed copy.
if ! git diff --quiet -- src/types/generated; then
  echo "The generated TypeScript bindings are out of date." >&2
  git diff --stat -- src/types/generated >&2
  status=1
fi

# Generated files that were never committed in the first place.
untracked="$(git ls-files --others --exclude-standard -- src/types/generated)"
if [[ -n "$untracked" ]]; then
  echo "Generated TypeScript bindings are not committed:" >&2
  echo "$untracked" >&2
  status=1
fi

if [[ "$status" -ne 0 ]]; then
  echo "Run 'pnpm generate:types' and commit the result." >&2
  exit "$status"
fi

echo "The generated TypeScript bindings are up to date."
