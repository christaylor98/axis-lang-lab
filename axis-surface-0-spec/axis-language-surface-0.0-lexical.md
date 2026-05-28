# axis-language-surface-0.0-lexical.md

## Axis Surface-0 — Lexical Specification

**Version 0.0 (AI-Only Surface)**
**Status:** Normative

---

## 1. Purpose

This document defines the **lexical structure** of **Axis Surface-0**.

Surface-0 is a **canonical, AI-native, binary-adjacent textual encoding**.
Its purpose is to provide the **lowest-entropy text form** that present-day LLMs
can reliably emit and that can be deterministically lowered into Axis Core.

If a behavior is not explicitly defined here, the lexer **MUST reject it**.

---

## 2. Scope and Non-Goals

### In Scope

* Character set
* Line structure
* Whitespace rules
* Tokens
* Error handling
* Determinism guarantees

### Out of Scope

* Parsing or grammar
* Stack semantics
* Registry resolution
* Types
* Semantics
* Execution
* Human readability concerns

---

## 3. Character Set

Surface-0 source text is defined over:

* **ASCII only** (`U+0000`–`U+007F`)

Any non-ASCII character **MUST cause a lexical error**.

---

## 4. Line Structure (Normative)

Surface-0 is **line-oriented**.

Rules:

* Each instruction occupies **exactly one line**
* Lines are terminated by `\n` or `\r\n`
* Empty lines are permitted
* Whitespace at line start/end is ignored
* There is **no multi-line construct**

Line order is preserved verbatim.

---

## 5. Whitespace

Whitespace characters:

* Space (`0x20`)
* Horizontal tab (`0x09`)
* Newline (`0x0A`)
* Carriage return (`0x0D`)

Rules:

* Whitespace separates tokens
* Whitespace has **no semantic meaning**
* Multiple whitespace characters collapse to a single separator
* Whitespace inside tokens is forbidden

---

## 6. Comments

Surface-0 **does not support comments**.

Any occurrence of:

* `//`
* `#`
* `/*`
* `;`

outside a literal **MUST cause a lexical error**.

Rationale: comments are human sugar and introduce non-semantic noise.

---

## 7. Tokens

Tokens are space-separated lexemes.

Each token has:

* kind
* raw lexeme
* byte span

No token lookahead or backtracking is permitted.

---

## 8. Keywords (Reserved)

The following tokens are **reserved keywords**:

```text
fn
end
arg
lit
call
```

Rules:

* Keywords are case-sensitive
* Keywords MUST NOT be used as identifiers

---

## 9. Identifiers

```
IDENT ::= [A-Za-z_][A-Za-z0-9_]* 
```

Rules:

* Case-sensitive
* Flat and atomic
* **No dots**
* **No hierarchy**
* **No qualification**

Any identifier containing `.` **MUST be rejected**.

---

## 10. Literals

### 10.1 Integer

```
INT ::= [0-9]+
```

* Base-10 only
* No sign
* No separators

---

### 10.2 Boolean

```text
true
false
```

---

### 10.3 String

```
STRING ::= " { ASCII char or escape } "
```

Supported escapes:

```
\"  \\  \n  \t
```

Unterminated or invalid strings are lexical errors.

---

## 11. Illegal Tokens

The following are **always illegal**:

* `(`
* `)`
* `{`
* `}`
* `.`
* `,`
* `:`
* `=>`
* `->`
* any operator symbol (`+`, `*`, etc.)

Surface-0 admits **no punctuation**.

---

## 12. Error Handling

Lexical errors:

* halt immediately
* report exact byte span
* provide category only (no recovery)

---

## 13. Determinism Guarantee

Given identical input, the lexer MUST:

* produce identical token streams
* produce identical spans
* fail deterministically

---

## 14. Guiding Principle

> **If it is not a token, it is not allowed.
> If it is allowed, it is a token.**

---
