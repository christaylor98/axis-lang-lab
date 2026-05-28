# Wave C: Sealing Guide

This guide explains how to seal Axis Language Lab for production distribution.

---

## Quick Start

### 1. Generate Sealed Artifacts

```sh
cargo run --bin seal
```

This creates:
- `src/generated/embedded_lexer.rs`
- `src/generated/embedded_parser.rs`
- `src/generated/embedded_schema.rs`
- `src/generated/embedded_lowering.rs`
- `src/generated/embedded_manifest.rs`

### 2. Build Sealed Binary

```sh
cargo build --release --features sealed
```

### 3. Run Sealed Binary

```sh
./target/release/axis run samples/program.ax
```

No spec paths needed - everything is embedded!

---

## Commands

### Seal Command

Generate all embedded specs:

```sh
cargo run --bin seal
```

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Wave C: Sealing Axis Language Lab
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔧 Generating embedded lexer...
   ✓ src/generated/embedded_lexer.rs
🔧 Generating embedded parser...
   ✓ src/generated/embedded_parser.rs
🔧 Generating embedded schema...
   ✓ src/generated/embedded_schema.rs
🔧 Generating embedded lowering...
   ✓ src/generated/embedded_lowering.rs
🔧 Generating artifact manifest...
   ✓ src/generated/embedded_manifest.rs

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Sealing complete!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Manifest Summary:
  Build: 2026-01-24T12:00:00Z
  Commit: abc123def456
  Branch: main
  Compiler: rustc 1.75.0
  Specs: 4 embedded

To build sealed binary:
  cargo build --release --features sealed
```

### Inspect Manifest

Display embedded build information (sealed builds only):

```sh
# Build first
cargo build --features sealed

# Inspect
cargo run --features sealed -- inspect manifest
```

**Output:**
```
Artifact Manifest
═══════════════════════════════════════

Build Information:
  Timestamp: 2026-01-24T12:00:00Z
  Git Commit: abc123def456
  Git Branch: main
  Compiler: rustc 1.75.0

Embedded Specs (4):
  • lexer
    Path: lang-lab-poc-userfiles/lexer.yaml
    Hash: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    Size: 2048 bytes
  • parser
    Path: lang-lab-poc-userfiles/parsing.yaml
    Hash: ...
    Size: 4096 bytes
  • schema
    Path: lang-lab-poc-userfiles/ast_schema.yaml
    Hash: ...
    Size: 1024 bytes
  • lowering
    Path: lang-lab-poc-userfiles/lowering.yaml
    Hash: ...
    Size: 512 bytes

Metadata:
  wave: C
  mode: sealed
```

JSON output:

```sh
cargo run --features sealed -- inspect manifest --format json
```

### Inspect Spec

Show info about an embedded spec (sealed builds only):

```sh
cargo run --features sealed -- inspect spec lexer
cargo run --features sealed -- inspect spec parser
cargo run --features sealed -- inspect spec schema
cargo run --features sealed -- inspect spec lowering
```

---

## Development Workflow

### Unsealed Development

During development, use runtime-loaded specs for flexibility:

```sh
cargo run -- run \
  --lexer lang-lab-poc-userfiles/lexer.yaml \
  --parser lang-lab-poc-userfiles/parsing.yaml \
  --schema lang-lab-poc-userfiles/ast_schema.yaml \
  --file samples/program.ax
```

### Sealed Production

For production distribution:

1. **Finalize specs** - ensure all YAML specs are correct
2. **Seal** - run `cargo run --bin seal`
3. **Build** - run `cargo build --release --features sealed`
4. **Distribute** - ship `target/release/axis` binary

The sealed binary:
- Contains all specs embedded
- Requires no external configuration
- Is fully deterministic
- Can be verified via manifest inspection

---

## Testing

### Determinism Tests

Verify that builds are deterministic:

```sh
cargo test --test wave_c_determinism_tests determinism_tests
```

Tests prove:
- Same input → same Core IR (every time)
- No environment variable influence
- No timestamp pollution

### Sealed Mode Tests

Verify sealed mode equivalence:

```sh
# Seal first
cargo run --bin seal

# Test
cargo test --features sealed --test wave_c_determinism_tests sealed_tests
```

Tests prove:
- Sealed and unsealed modes produce identical Core IR
- Embedded specs match original YAML (via hashes)
- Manifest is accessible

### Trust Invariant Tests

Verify security properties:

```sh
cargo test --test wave_c_determinism_tests trust_invariant_tests
```

Tests prove:
- Environment cannot influence output
- No hidden configuration files consulted
- Traceability (Wave B) is preserved

---

## Build Modes

### Unsealed (Default)

```sh
cargo build
```

- Specs loaded from YAML at runtime
- Flexible - can swap specs without recompiling
- Development-friendly

### Sealed (Production)

```sh
# Seal first
cargo run --bin seal

# Build
cargo build --features sealed
```

- All specs embedded at compile time
- Zero external dependencies (except source files)
- Deterministic and reproducible
- Distribution-ready

---

## Guarantees

### Determinism ✅

- Same input + same specs → identical Core IR (bitwise)
- No randomness
- No timestamps in Core IR
- Environment-isolated

### Security ✅

- Sealed binaries cannot load external specs
- All specs are SHA-256 verified
- Build provenance is embedded
- No configuration backdoors

### Compatibility ✅

- Wave B traceability preserved
- Execution semantics unchanged
- API-compatible with unsealed mode

---

## Troubleshooting

### "Sealed mode enabled but embedded specs not found"

**Problem:** Building with `--features sealed` before running seal tool.

**Solution:**
```sh
cargo run --bin seal
cargo build --features sealed
```

### "Manifest inspection only available in sealed builds"

**Problem:** Trying to inspect manifest in unsealed build.

**Solution:**
```sh
cargo build --features sealed
cargo run --features sealed -- inspect manifest
```

### Determinism test failures

**Problem:** Pipeline is not producing identical output.

**Solution:**
1. Check for unintended randomness in code
2. Verify no timestamps in Core IR generation
3. Ensure no environment variable usage in pipeline

---

## Architecture

### Codegen Flow

```
YAML Specs (lang-lab-poc-userfiles/*.yaml)
    ↓
seal tool (src/bin/seal.rs)
    ↓
codegen modules (src/codegen/*.rs)
    ↓
Generated Rust (src/generated/embedded_*.rs)
    ↓
Compiled into binary (--features sealed)
    ↓
Sealed artifact
```

### Runtime Flow (Sealed Mode)

```
Source file (program.ax)
    ↓
Embedded lexer spec → Tokens
    ↓
Embedded parser spec → Parse tree
    ↓
Embedded schema spec → Schema AST
    ↓
Embedded lowering spec → Core IR
    ↓
Execution
```

No external files needed after source is read.

---

## Integration

### CI/CD Pipeline

Example GitHub Actions:

```yaml
name: Build Sealed Release

on:
  release:
    types: [created]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Seal artifact
        run: cargo run --bin seal
      
      - name: Build sealed binary
        run: cargo build --release --features sealed
      
      - name: Inspect manifest
        run: cargo run --release --features sealed -- inspect manifest
      
      - name: Run determinism tests
        run: cargo test --features sealed --test wave_c_determinism_tests
      
      - name: Upload release binary
        uses: actions/upload-artifact@v2
        with:
          name: axis-sealed
          path: target/release/axis
```

### Docker

Example Dockerfile:

```dockerfile
FROM rust:1.75 as builder

WORKDIR /build
COPY . .

RUN cargo run --bin seal
RUN cargo build --release --features sealed

FROM debian:bookworm-slim

COPY --from=builder /build/target/release/axis /usr/local/bin/

ENTRYPOINT ["axis"]
```

---

## Version Control

### What to Commit

✅ Commit to Git:
- All source code
- YAML specs
- Tests
- Documentation
- `Cargo.toml`, `build.rs`

❌ Do NOT commit:
- `src/generated/embedded_*.rs` (generated files)
- `target/` (build artifacts)

### Why Not Commit Generated Files?

Generated files should be created at build time to ensure:
- Specs and generated code stay in sync
- Manifest reflects actual build environment
- Hashes are accurate

Developers should run `cargo run --bin seal` locally or in CI.

---

## FAQ

**Q: Can I use sealed mode in development?**  
A: You can, but unsealed mode is more flexible. Sealed mode is optimized for production.

**Q: Does sealing change execution semantics?**  
A: No. Sealed and unsealed modes produce identical Core IR. Sealing is purely about embedding specs.

**Q: Can I inspect a sealed binary's provenance?**  
A: Yes! Run `axis inspect manifest` to see build timestamp, git commit, spec hashes, etc.

**Q: Are sealed builds reproducible?**  
A: Yes. Same specs + same source → identical Core IR. Manifest timestamps may differ, but Core IR is deterministic.

**Q: Does Wave C break Wave B traceability?**  
A: No. Traceability is preserved. Sealed mode adds build provenance on top of existing trace graphs.

**Q: Can sealed mode load external specs?**  
A: No. That's the point! Sealed mode rejects all external configuration for security and determinism.

---

## Summary

Wave C provides:
- ✅ Deterministic builds
- ✅ Zero runtime configuration
- ✅ Embedded specs with integrity hashing
- ✅ Build provenance tracking
- ✅ Distribution-ready artifacts

Use `cargo run --bin seal` → `cargo build --features sealed` → ship!
