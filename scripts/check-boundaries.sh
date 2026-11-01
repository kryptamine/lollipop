#!/usr/bin/env bash
# Workspace rules from ADR 0001 that neither clippy nor the compiler can state.
set -euo pipefail
cd "$(dirname "$0")/.."

# Captured first, so a failing cargo stops the script instead of reading as no match.
core_deps=$(cargo tree -p lollipop-core -e normal,build --prefix none)
if grep -E '^tauri' <<<"$core_deps"; then
  echo "lollipop-core must not depend on Tauri" >&2
  exit 1
fi

if grep -L '^workspace = true' crates/*/Cargo.toml | grep .; then
  echo "every crate must inherit the workspace lints" >&2
  exit 1
fi

if grep -rl 'unsafe_code' crates --include='*.rs' | grep -v '^crates/app/src/platform'; then
  echo "unsafe_code may only be allowed in crates/app/src/platform" >&2
  exit 1
fi

# A value read at compile time ends up in the binary, which is how a token would get there.
if grep -rnE '\b(option_)?env!\(' crates --include='*.rs' | grep -v 'env!("CARGO_'; then
  echo "only Cargo's own variables may be read at compile time" >&2
  exit 1
fi
