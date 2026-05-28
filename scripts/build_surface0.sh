#!/usr/bin/env sh
set -e

# Get repository root (parent of scripts directory)
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

SRC=$1
OUT=${2:-build/surface0/app}

mkdir -p "$REPO_ROOT/build/surface0"

cargo run --bin axis --manifest-path "$REPO_ROOT/Cargo.toml" -- \
  --lexer "$REPO_ROOT/axis-surface-0-config/semantic-surface-0-lexer.yaml" \
  --parser "$REPO_ROOT/axis-surface-0-config/semantic-surface-0-parse.yaml" \
  --schema "$REPO_ROOT/axis-surface-0-config/semantic-surface-0-ast.yaml" \
  --src "$SRC" \
  --reg "$REPO_ROOT/axis-surface-0-config/surface-0-registry.axreg" \
  --out "$REPO_ROOT/build/surface0/program.coreir"

(cd "$REPO_ROOT/rust-bridge" && cargo build --release)

"$REPO_ROOT/rust-bridge/target/release/axis-rust-bridge" build \
  "$REPO_ROOT/build/surface0/program.coreir" \
  --out "$OUT"

echo "Executable emitted at $OUT"
