#!/usr/bin/env bash
set -e

# Build example external library
echo "Building example external library..."

cd examples

rustc --crate-type=staticlib \
  --extern axis_rust_bridge=../rust-bridge/target/debug/libaxis_rust_bridge.rlib \
  libaxis_example.rs \
  -o libaxis_example.a \
  -L dependency=../rust-bridge/target/debug/deps

echo "Library built: examples/libaxis_example.a"
echo ""
echo "To use this library when building an Axis program:"
echo "  ./scripts/build_ai1.sh program.ax output \\"
echo "    --link-lib libaxis_example.a \\"
echo "    --link-search ./examples"
