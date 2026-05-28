#!/usr/bin/env sh
set -e

# Get repository root (parent of scripts directory)
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

SRC=$1
OUT=${2:-build/ai1/app}

mkdir -p "$REPO_ROOT/build/ai1"

cargo run --bin axis --manifest-path "$REPO_ROOT/Cargo.toml" -- \
  --lexer "$REPO_ROOT/axis-surface-ai1-config/ai1-rpn-lexer.yaml" \
  --parser "$REPO_ROOT/axis-surface-ai1-config/ai1-rpn-parse.yaml" \
  --schema "$REPO_ROOT/axis-surface-ai1-config/ai1-rpn-ast.yaml" \
  --src "$SRC" \
  --reg "$REPO_ROOT/axis-surface-ai1-config/ai1-rpn-registry.axreg" \
  --out "$REPO_ROOT/build/ai1/program.coreir"

(cd "$REPO_ROOT/rust-bridge" && cargo build --release)

# Build with optional external library linking
# Usage examples:
#   ./scripts/build_ai1.sh input.ax
#   ./scripts/build_ai1.sh input.ax build/output --link-lib libaxis_std.a --link-search ./libs
"$REPO_ROOT/rust-bridge/target/release/axis-rust-bridge" build \
  "$REPO_ROOT/build/ai1/program.coreir" \
  --out "$OUT" \
  "$@"

echo "Executable emitted at $OUT"
