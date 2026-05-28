# Lexer Codegen: YAML → Static Rust

## Overview

The Axis language lab provides two equivalent lexer implementations:

1. **Runtime spec-driven lexer** (`src/frontend/lexer_engine.rs`)
   - Reads YAML spec at runtime
   - Compiles regex patterns dynamically
   - Flexible for development/experimentation

2. **Generated static lexer** (`src/generated_lexer.rs`)
   - Generated from YAML spec via `axis codegen`
   - Compiled Rust code with static regexes
   - Zero runtime overhead

**Source of Truth**: YAML spec file (`lang-lab-poc-userfiles/lexer.yaml`)

**Equivalence Guarantee**: 26 comprehensive tests prove both implementations produce identical token streams.

## Usage

### Generate Static Lexer

```bash
cargo run --bin axis -- codegen \
  --spec lang-lab-poc-userfiles/lexer.yaml \
  --out src/generated_lexer.rs
```

### Using the Generated Lexer

```rust
use axis_lang_lab_working::generated_lexer;

let tokens = generated_lexer::lex("fn main() { }")?;
for token in tokens {
    println!("{:?}: {}", token.kind, token.lexeme);
}
```

### Verify Equivalence

```bash
cargo test --test equivalence_runtime_vs_generated
```

All 26 tests **must pass** to guarantee generated code matches runtime behavior.

## Architecture

### Code Generation Pipeline

```
YAML Spec (lang-lab-poc-userfiles/lexer.yaml)
    ↓
axis codegen --spec ... --out ...
    ↓
Generated Rust Code (src/generated_lexer.rs)
    ↓
Compile into binary (zero runtime cost)
```

### Generated Code Structure

```rust
// Lazy static regex compilation (once_cell)
static WHITESPACE_RE: Lazy<Regex> = Lazy::new(|| ...);
static COMMENT_RES: Lazy<Vec<Regex>> = Lazy::new(|| ...);

// Const arrays for keywords/punctuation
const KEYWORDS: &[&str] = &["enum", "fn", "if", ...];
const PUNCTUATION: &[&str] = &["=>", "->", ...];

// Main lexer function
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    // ... pattern matching logic identical to runtime ...
}
```

### Equivalence Testing Strategy

**Invariant**: `∀ input: runtime_lex(input) = generated_lex(input)`

Test categories:
- Empty source
- Keywords (fn, match, enum, proj)
- Identifiers
- Literals (int, bool, string, unit)
- Punctuation and operators
- Comments and whitespace
- Complex programs
- Error cases (both must reject invalid input identically)
- Stress tests (100+ tokens, nested structures)
- Determinism (multiple runs produce same output)

## Maintenance

### When to Regenerate

Regenerate `src/generated_lexer.rs` whenever:
- YAML spec changes (`lang-lab-poc-userfiles/lexer.yaml`)
- Adding new token kinds
- Modifying regex patterns
- Changing lexical rules

### Workflow

1. Edit YAML spec
2. Run codegen: `cargo run --bin axis -- codegen --spec ... --out ...`
3. **Critical**: Run equivalence tests: `cargo test --test equivalence_runtime_vs_generated`
4. If tests fail → codegen bug or spec interpretation mismatch
5. Commit both YAML spec and generated code together

## Performance

**Runtime Spec-Driven**:
- Regex compilation: ~1-5ms first call
- Subsequent calls: comparable to generated

**Generated Static**:
- Regex compilation: compile-time (once_cell lazy init)
- Zero YAML parsing overhead
- Suitable for production builds

## Design Decisions

### Why Both Implementations?

- **Runtime**: Rapid iteration during language design
- **Generated**: Production performance, ahead-of-time validation

### Why Equivalence Tests?

- Proves codegen correctness
- Prevents silent divergence
- Documents expected behavior
- Regression protection

### Source of Truth

YAML spec is authoritative. Generated code is a derived artifact.

**Never** manually edit `src/generated_lexer.rs` - always regenerate from spec.

## Future Extensions

Possible enhancements:
- Generate parser codegen (YAML → static Rust parser)
- Multi-target codegen (emit C/Python/JavaScript lexers)
- Optimize generated code (branchless token matching)
- Integration tests using property-based testing (quickcheck/proptest)
