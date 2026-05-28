# Axis Lexer Spec — Tokenization Contract

VERSION: 1.0  
AUTHORITY: This document defines lexer behavior for all Axis language surfaces.  
FROZEN: 2026-01-28

## SCOPE

This contract specifies how lexer specs are loaded, validated, and executed.  
Spec, code, and config MUST agree 100%.

## SPEC SCHEMA

All lexer specs MUST be valid YAML conforming to `LexerSpec` in `src/frontend/lexspec.rs`.

### Unknown Fields

FORBIDDEN.  
All lexer spec structs enforce `#[serde(deny_unknown_fields)]`.  
Any unknown field → hard error on load.

### Required Top-Level Structure

```yaml
lexer:
  # all config here
```

## TOKEN MATCHING ORDER (DETERMINISTIC)

Position advances left-to-right, byte-by-byte.  
At each position, try rules in this exact order:

1. **Whitespace** (if `whitespace.skip = true` → skip, no token)
2. **Comments** (if `skip = true` → skip, no token)
3. **Unit literal** (if defined, e.g., `()`)
4. **Punctuation** (longest-first, first match wins)
5. **String literal** (if delimiter matches)
6. **Int literal** (regex match at position 0)
7. **Bool literals** (if defined, boundary-checked)
8. **Identifiers** (regex match, keyword check applies)
9. **Unknown character → hard error**

## KEYWORD vs IDENTIFIER

- Keywords checked AFTER identifier regex match
- If lexeme ∈ keywords → `TokenKind::Keyword`
- Else → `TokenKind::Ident`
- Bool values listed as keywords → treated as keywords

## PUNCTUATION (LONGEST-MATCH)

Punctuation sorted longest-first internally.  
First match from current position wins.  
No backtracking.

Example:
- Config: `["=", "=>", "=="]`
- Input: `"==>=="`
- Match: `==`, `>=`, `==` (left-to-right, longest available)

## ERROR BEHAVIOR

### Unknown Character

```
Error: unexpected character 'X' at position N
```

With span.  
Hard error.  
Lexing terminates.

### Unterminated String

```
Error: unterminated string literal
```

Hard error.  
Lexing terminates.

## WHITESPACE and COMMENTS

- If `skip = true` → no token emitted
- If `skip = false` → token emitted with `TokenKind::Whitespace` or `TokenKind::Comment`
- Default: `skip = true`

## SPAN CONSTRUCTION

Every token MUST include:
- `start`: byte offset of first character
- `end`: byte offset AFTER last character
- EOF token: `Span { start: len, end: len }`

## CHARSET

Currently supported: `"ascii"`  
Non-ASCII characters → error (not yet supported)

## CASE SENSITIVITY

Default: `case_sensitive = true`  
Affects keyword matching and identifier pattern interpretation.

## LANGUAGE-SPECIFIC CONTRACTS

### Surface-0
- Keywords: `fn`, `end`, `arg`, `lit`, `call`
- Identifiers: `[A-Za-z_][A-Za-z0-9_.]*` (allows dots)
- Int literals: `[0-9]+`
- No punctuation
- No comments

### Semantic-Surface-0
- Keywords: `let`, `in`, `fn`, `if`, `then`, `else`
- Identifiers: `[a-zA-Z_][a-zA-Z0-9_]*`
- Int literals: `-?[0-9]+`
- Bool literals: `true`, `false` (as BoolLit tokens)
- Unit literal: `()` (single token, before punctuation check)
- Punctuation: `(`, `)`, `=`, `=>`
- Comments: `//` line comments

### AI-1 (RPN)
- Keywords: `app`, `lam`, `let`, `if`, `true`, `false`, `unit`
- Identifiers: `[a-zA-Z_][a-zA-Z0-9_]*`
- Int literals: `-?[0-9]+`
- Bool values as KEYWORDS (not BoolLit)
- No punctuation
- Comments: `//` line comments

### AI-2 (Explicit Prefix)
- Keywords: `let`, `lam`, `app`, `if`, `var`, `int`, `bool`, `unit`, `true`, `false`
- Identifiers: `[a-zA-Z_][a-zA-Z0-9_]*`
- Int literals: `-?[0-9]+`
- Punctuation: `(`, `)`, `,`, `=`
- Comments: `//` line comments

### Surface-H1
- Keywords: `let`, `fn`, `if`, `else`, `loop`, `while`, `for`, `in`, `match`, `spawn`, `join`, `channel`, `send`, `recv`, `controller`, `on`, `run`, `intent`
- Identifiers: `[A-Za-z_][A-Za-z0-9_]*`
- Forbidden identifiers: `async`, `await` (enforced by parser, not lexer)
- Int literals: `[0-9]+`
- String literals: `"..."` with escapes `\"`, `\\`, `\n`, `\t`, `\r`
- Punctuation: `(`, `)`, `{`, `}`, `[`, `]`, `,`, `;`, `:`, `.`, `@`, `<`, `>`, `=`, `=>`, `==`, `!=`, `<=`, `>=`, `+`, `-`, `*`, `/`, `!`, `_`
- Comments: `//` line, `/* */` block

## IMPLEMENTATION NOTES

### File Locations
- Spec schema: `src/frontend/lexspec.rs`
- Loader: `src/frontend/lexspec_load.rs`
- Engine: `src/frontend/lexer_engine.rs`
- Token model: `src/frontend/token.rs`

### Testing
- All languages: `tests/lexer_tests.rs`
- Per-language test modules in `tests/lexer/`

## INVARIANTS

1. Lexing is deterministic
2. Token order reflects source order
3. Spans are contiguous and non-overlapping
4. Every byte is accounted for (token or skipped)
5. Unknown characters → error (no silent recovery)
6. Config schema violations → error on load
7. No implicit defaults except documented in `lexspec.rs`
