#!/usr/bin/env sh
set -e

echo "[TRACE] Script started"
echo "[TRACE] Raw arguments: $@"
echo "[TRACE] Number of arguments: $#"

# Get repository root (parent of scripts directory)
REPO_ROOT="/home/chris/dev/axis-lang-lab-working"
echo "Repository root: $REPO_ROOT"

# Capture the directory from which the script was called
CALLER_DIR="$PWD"
echo "[TRACE] Caller directory: $CALLER_DIR"

SRC=$1
REG=$2
OUT=${3:-build/ai2/app}
echo "[TRACE] Before shift - SRC=$SRC REG=$REG OUT=$OUT"

# Shift arguments safely
if [ $# -ge 3 ]; then
  echo "[TRACE] Shifting 3 arguments"
  shift 3
else
  echo "[TRACE] Shifting all $# arguments"
  shift $#
fi
LINK_ARGS="$@"
echo "[TRACE] After shift - LINK_ARGS=$LINK_ARGS"

# Convert relative paths to absolute paths based on caller directory
echo "[TRACE] Checking SRC path..."
if [ -n "$SRC" ] && [ "${SRC#/}" = "$SRC" ]; then
  echo "[TRACE] SRC is relative, converting to absolute"
  SRC="$CALLER_DIR/$SRC"
else
  echo "[TRACE] SRC is absolute or empty"
fi
echo "[TRACE] Final SRC: $SRC"

echo "[TRACE] Checking REG path..."
if [ -n "$REG" ] && [ "${REG#/}" = "$REG" ]; then
  echo "[TRACE] REG is relative, converting to absolute"
  REG="$CALLER_DIR/$REG"
else
  echo "[TRACE] REG is absolute or empty"
fi
echo "[TRACE] Final REG: $REG"

echo "[TRACE] Checking OUT path..."
# Handle output path - if relative and doesn't start with 'build/', make it absolute
# Otherwise, resolve it relative to REPO_ROOT
if [ -n "$OUT" ]; then
  case "$OUT" in
    /*) 
      echo "[TRACE] OUT is absolute"
      # Already absolute, keep as-is
      ;;
    # build/*)
    #   echo "[TRACE] OUT starts with build/, making relative to repo root"
    #   # Relative to repo root
    #   OUT="$REPO_ROOT/$OUT"
    #   ;;
    *)
      echo "[TRACE] OUT is relative to caller directory"
      # Relative to caller directory
      OUT="$CALLER_DIR/$OUT"
      ;;
  esac
fi
echo "[TRACE] Final OUT: $OUT"

echo "Arguments:"
echo "  Source: $SRC"
echo "  Register: $REG"
echo "  Output: $OUT"
echo "  Link args: $LINK_ARGS"

echo "[TRACE] Creating build directory..."
mkdir -p "$REPO_ROOT/build/ai2"
echo "[TRACE] Build directory created"

echo "[TRACE] Running axis compiler..."
echo "[TRACE] Command: $REPO_ROOT/target/release/axis --lexer ... --src $SRC --reg $REG"
"$REPO_ROOT/target/release/axis" \
  --lexer "$REPO_ROOT/axis-surface-ai2-config/ai2-explicit-lexer.yaml" \
  --parser "$REPO_ROOT/axis-surface-ai2-config/ai2-explicit-parse.yaml" \
  --schema "$REPO_ROOT/axis-surface-ai2-config/ai2-explicit-ast.yaml" \
  --src "$SRC" \
  --reg "$REG" \
  --out "$REPO_ROOT/build/ai2/program.coreir"
echo "[TRACE] Axis compiler completed"

echo "[TRACE] Building rust-bridge..."
(cd "$REPO_ROOT/rust-bridge" && cargo build --release)
echo "[TRACE] Rust-bridge build completed"

echo "[TRACE] Running axis-rust-bridge..."
echo "[TRACE] Command: $REPO_ROOT/rust-bridge/target/release/axis-rust-bridge build ... --out $OUT"
"$REPO_ROOT/rust-bridge/target/release/axis-rust-bridge" build \
  "$REPO_ROOT/build/ai2/program.coreir" \
  --out "$OUT" \
  $LINK_ARGS
echo "[TRACE] Axis-rust-bridge completed"

echo "Executable emitted at $OUT"
