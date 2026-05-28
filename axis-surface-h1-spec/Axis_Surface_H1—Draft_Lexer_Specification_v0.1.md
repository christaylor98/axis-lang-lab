# Axis Surface H1 — Lexer Specification v0.1 (Revised)

---

## 1. Input Model

* Input is a UTF-8 byte stream.
* The lexer MUST track:

  * byte offset
  * line
  * column
* Newlines are significant only for:

  * diagnostics
  * intent directive termination (see §7.3)

---

## 2. Whitespace and Newlines

### 2.1 Whitespace

```
WS := ' ' | '\t' | '\r'
```

* Whitespace separates tokens.
* Ignored otherwise.

### 2.2 Newlines

```
NL := '\n'
```

* Newlines are treated as whitespace for tokenization.
* Newlines terminate `@intent.*` directives.

---

## 3. Comments (Rust Style)

### 3.1 Line Comments

```
LINE_COMMENT := '//' <any chars except NL>*
```

* Consumed until newline.
* Produces no tokens.
* Newline still advances line counter.

---

### 3.2 Block Comments (Nested)

```
BLOCK_COMMENT := '/*' ( BLOCK_COMMENT | any-char )* '*/'
```

Rules:

* Block comments MAY nest.
* Nesting MUST be correctly balanced.
* Unterminated block comments are a **lexer error**.

This exactly matches Rust’s comment model.

---

## 4. Identifiers and Keywords

### 4.1 Identifier

```
IDENT := ( '_' | ASCII_ALPHA ) ( '_' | ASCII_ALNUM )*
```

* Case-sensitive.
* `_` alone is a valid token (used as wildcard in `match`).

---

### 4.2 Keywords (reserved)

The following lex as keyword tokens, not identifiers:

```
let fn if else
loop for in
match
spawn join
channel send recv
controller on run
```

---

### 4.3 Forbidden Keywords (Hard Lexer Error)

The following MUST produce a **lexer error**, not a parser error:

```
async
await
```

Rationale:

* Keeps “async is forbidden” explicit and early.
* Prevents accidental surface creep.

---

## 5. Literals

### 5.1 Integer Literals

```
INT := [0-9]+
```

* Decimal only.
* No `_` separators in v0.1.
* Unary `-` is tokenized separately.

---

### 5.2 String Literals — Normal

```
STRING := '"' ( ESC | ~["\\] | NL )* '"'
```

Where:

```
ESC := '\' ( '"' | '\' | 'n' | 't' | 'r' )
```

Rules:

* **Multiline strings ARE allowed**
* Newlines inside strings are preserved
* Unknown escapes are a lexer error

Example:

```h1
let s = "hello
world";
```

---

### 5.3 String Literals — Raw (Rust-Style, Recommended)

Raw strings avoid escaping and are ideal for configs, scripts, SQL, etc.

#### Syntax

```
RAW_STRING := 'r"' ( any-char )* '"'
```

Optionally extended later to:

```
r#" ... "#, r##" ... "##, etc.
```

Rules:

* Contents are taken verbatim
* No escape processing
* Multiline allowed
* Terminates only on the exact closing delimiter

Example:

```h1
let query = r"
  SELECT *
  FROM prices
  WHERE value > 100
";
```

This is lexer-simple and extremely user-friendly.

---

## 6. Operators and Punctuation

### 6.1 Single-Character Tokens

```
( ) { } [ ]
, ; :
. @
< >
```

---

### 6.2 Multi-Character Tokens (Maximal Munch)

The lexer MUST apply maximal munch.

```
=>   ==   !=
<=   >=
```

Optional (only if used in grammar):

```
->   &&   ||
```

---

## 7. Intent Directives (`@intent.*`)

### 7.1 Lexical Form

Intent directives are lexed as ordinary tokens:

```
@ IDENT("intent") . IDENT(<key>)
```

Example:

```h1
@intent.parallel
```

Tokens:

```
AT IDENT DOT IDENT
```

---

### 7.2 Allowed Keys (v0.1)

Lexer allows **any** identifier after `@intent.`

Parser validation is required.

Allowed keys (v0.1):

```
parallel
serial
```

Unknown keys are a **hard parser error** in v0.1.

---

### 7.3 Termination Rule

An intent directive MUST terminate at:

* newline `NL`, or
* semicolon `;`

Example (valid):

```h1
@intent.parallel
for x in items { ... }
```

Example (invalid):

```h1
@intent.parallel for x in items { ... }
```

This avoids ambiguity and keeps parsing simple.

---

## 8. Generics (`channel<T>`)

Lexing rules:

* `<` and `>` are always tokens
* No lexer-level disambiguation

Parser decides whether `< >` represent:

* generic parameters
* or comparison operators (if enabled)

(Recommendation: reserve `< >` for generics in v0.1.)

---

## 9. Error Conditions (Lexer MUST fail)

The lexer MUST emit a hard error on:

* Invalid UTF-8
* Unterminated string literal
* Unterminated raw string
* Unterminated block comment
* Unknown escape sequence
* Forbidden keyword token (`async`, `await`)
* Any unrecognized character

---

## 10. Minimal Token Set (v0.1)

Required token classes:

* Keywords (`KW_*`)
* `IDENT`
* `INT`
* `STRING`
* `RAW_STRING`
* `AT`
* punctuation tokens
* operator tokens
* EOF

Exact enum names are implementation-defined.

---

## 11. Lexer Invariants

* Lexer performs **no semantic analysis**
* Lexer performs **no intent validation**
* Lexer produces **lossless source spans**
* All annotations and strings survive lowering unchanged
