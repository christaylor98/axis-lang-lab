#!/usr/bin/env sh
set -e

# Get repository root (parent of scripts directory)
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

SRC=$1
OUT=${2:-build/h1/app}

mkdir -p "$REPO_ROOT/build/h1"

cargo run --bin axis --manifest-path "$REPO_ROOT/Cargo.toml" -- \
  --lexer "$REPO_ROOT/axis-surface-h1-config/surface-h1-lexer.yaml" \
  --parser "$REPO_ROOT/axis-surface-h1-config/surface-h1-parse.yaml" \
  --schema "$REPO_ROOT/axis-surface-h1-config/surface-h1-ast.yaml" \
  --src "$SRC" \
  --reg "$REPO_ROOT/axis-surface-h1-config/surface-h1-registry.axreg" \
  --out "$REPO_ROOT/build/h1/program.coreir"

(cd "$REPO_ROOT/rust-bridge" && cargo build --release)

# Build with optional external library linking
# Usage examples:
#   ./scripts/build_axh.sh input.axh
#   ./scripts/build_axh.sh input.axh build/output --link-lib libaxis_std.a --link-search ./libs
"$REPO_ROOT/rust-bridge/target/release/axis-rust-bridge" build \
  "$REPO_ROOT/build/h1/program.coreir" \
  --out "$OUT" \
  "$@"

echo "Executable emitted at $OUT"
