# Axis Language Lab — Parser Discovery & Delivery Specification

**Status:** Authoritative Design (Pre-Implementation)
**Phase:** Unit-Isolated Parser Development
**Scope:** Lex → Parse → AST (NO Lowering, NO Core IR)
**Audience:** Language Lab implementors and reviewers

---

## 0. Design Positioning

The parser is **structural plumbing**.

It:

* Consumes lexer tokens
* Enforces grammar correctness
* Produces a **structural AST**
* Owns **no semantics**
* Is **entirely user-configured**
* Is **deterministic by construction**

The parser does **not**:

* Perform evaluation
* Perform name resolution
* Perform type checking
* Perform lowering
* Perform registry interaction
* Perform error recovery

---

## 1. Lexer → Parser Interface Contract

### 1.1 Input Contract

**Parser input:**

* A complete token stream produced by the user-configured lexer
* Exactly one EOF token at the end

```rust
struct Token {
    kind: TokenKind,
    lexeme: String,
    span: Span,
}

enum TokenKind {
    Keyword(String),
    Ident,
    IntLit,
    BoolLit,
    StringLit,
    UnitLit,
    Punct(String),
    Eof,
}

struct Span {
    start: usize,
    end: usize,
}
```

### 1.2 Parser Assumptions (Non-Negotiable)

The parser **assumes**, but does not re-verify:

* Tokens are well-formed
* Spans are monotonic and non-overlapping
* `span` accurately maps to source text
* Lexer has already failed if tokenization was invalid

If lexer failed, parser **must not run**.

### 1.3 Token Access Rules

The parser:

* Reads `kind`, `lexeme`, and `span`
* Never modifies tokens
* Never inspects raw source text
* Never re-lexes

---

## 2. Parser Specification Authority

### 2.1 Source of Truth

The **only authority** for parsing behavior is:

```
parsing.yaml
```

The parser implementation is a **mechanical executor** of this spec.

Changing the spec **must change behavior**.

---

## 3. Grammar Model (Authoritative)

### 3.1 Grammar Form

* Grammar is **context-free**
* Grammar must be **mechanically unambiguous**
* Grammar must be **non-left-recursive**
* Grammar is **validated at spec-load time**

### 3.2 Grammar Entry Point

```yaml
parser:
  start: Program
```

The `start` nonterminal **must exist**.

---

## 4. Production Mini-Grammar (Locked)

Production strings are parsed using the following grammar.

### 4.1 Production Syntax (EBNF)

```
production   := alternation
alternation  := sequence ("|" sequence)*
sequence     := term*
term         := atom postfix?
postfix      := "*" | "+" | "?"
atom         := nonterminal
              | terminal
              | "(" alternation ")"

nonterminal  := IDENTIFIER          # e.g. Expr, Block
terminal     := token_terminal
              | keyword_terminal
              | punct_terminal

token_terminal   := "<" IDENT ">"          # <IDENT>, <INT_LIT>
keyword_terminal := "kw:" STRING           # kw:"if"
punct_terminal   := "punct:" STRING        # punct:"+"
```

### 4.2 Binding Rules

* Postfix (`* + ?`) binds tighter than sequence
* Sequence binds tighter than alternation
* Whitespace is insignificant
* No comments inside production strings
* YAML string rules apply (no parser-level escaping)

### 4.3 Examples

Valid:

```
Expr -> Term ( punct:"+" Term )*
If   -> kw:"if" Expr Block kw:"else" Block
Atom -> <IDENT> | <INT_LIT> | "(" Expr ")"
```

Invalid (spec-load error):

* Unbalanced parentheses
* Postfix applied to nothing
* Undefined nonterminal
* Unknown token kind
* Inline regexes

---

## 5. Terminal Matching Rules (Corrected)

**Parser terminals never match raw strings implicitly.**

All terminals are explicit and typed.

### 5.1 Allowed Terminal Forms

| Spec Form    | Matches TokenKind          |
| ------------ | -------------------------- |
| `<IDENT>`    | `TokenKind::Ident`         |
| `<INT_LIT>`  | `TokenKind::IntLit`        |
| `<BOOL_LIT>` | `TokenKind::BoolLit`       |
| `kw:"if"`    | `TokenKind::Keyword("if")` |
| `punct:"+"`  | `TokenKind::Punct("+")`    |

### 5.2 Explicit Rule

* `"true"` **never implicitly means BoolLit**
* If lexer emits `BoolLit("true")`, grammar must use `<BOOL_LIT>`
* If lexer emits `Keyword("true")`, grammar must use `kw:"true"`

No inference. No dual meaning.

---

## 6. Grammar Validation (Spec-Load Time)

### 6.1 Structural Validation

Fail if:

* YAML is invalid
* `parser.start` missing
* Grammar map missing
* Referenced nonterminal undefined
* Production string malformed

### 6.2 Left Recursion Detection (Required)

The spec loader **must reject**:

1. Direct left recursion

   ```
   A -> A α
   ```

2. Indirect left recursion

   ```
   A -> B α
   B -> A β
   ```

3. Nullable-mediated recursion

   ```
   A -> B α
   B -> ε
   ```

#### Required Algorithm

1. Compute nullable nonterminals
2. Build left-edge graph, following nullable prefixes
3. Any cycle = **fatal spec error**

---

## 7. Ambiguity Policy (Final)

### 7.1 Policy

**Ambiguous grammars are forbidden.**

Ambiguity is a **spec error**, not a runtime behavior.

### 7.2 What Is Detected

At minimum:

* FIRST/FIRST conflicts
* FIRST/FOLLOW conflicts caused by nullable productions

This is **mechanical ambiguity detection**, not full CFG ambiguity solving.

### 7.3 Failure Mode

Spec load fails with a diagnostic identifying:

* The nonterminal
* The conflicting alternatives
* The token prefix causing ambiguity

---

## 8. Spec / Lexer Compatibility (Strengthened)

### 8.1 Mandatory Validation

At spec-load time:

* Every `token:` terminal must correspond to a lexer token kind
* Every `kw:` terminal must exist in lexer keywords
* Every `punct:` terminal must exist in lexer punctuation/operators

### 8.2 Failure Mode

Mismatch = **fatal spec error**

No warnings. No fallback.

---

## 9. Parser Execution Model

### 9.1 Strategy

* Recursive descent
* Backtracking between alternatives
* No left recursion
* Deterministic due to grammar correctness

### 9.2 Determinism Guarantee

Same token stream + same spec ⇒ same AST
Always.

---

## 10. AST Model (Wave 1 Default Locked)

### 10.1 Default Output

**Wave 1 output = Generic ASTNode only**

Typed AST generation is **explicitly deferred**.

```rust
struct ASTNode {
    kind: String,            // nonterminal name
    children: Vec<ASTChild>,
    span: Span,
}

enum ASTChild {
    Node(ASTNode),
    Terminal(Token),
}
```

### 10.2 AST Construction Rules

* One AST node per successful nonterminal parse
* Terminals become leaf nodes
* Repetition ⇒ vectors
* Optional ⇒ zero or one node
* Node span = union of child spans

### 10.3 Span Invariants (Corrected)

Removed invariant:

* ❌ “Every token must be covered by an AST node”

Retained invariants:

* Root AST span covers first → last non-EOF token
* All non-EOF tokens are consumed
* AST spans are monotonic and accurate

---

## 11. Error Handling

### 11.1 Parse Errors

* Fail fast
* No recovery
* Single error

```rust
struct ParseError {
    message: String,
    span: Span,
}
```

Examples:

```
expected kw:"if", found IDENT
no production matched for Expr
unexpected token after end of parse
```

---

## 12. Unit Testing Requirements

Parser tests **must be isolated**:

* No lowering
* No Core IR
* No registry
* No execution

### Required Test Classes

1. Valid parse tests
2. Parse failure tests
3. Determinism tests
4. Spec authority tests
5. Invariant tests
6. Spec rejection tests (left recursion, ambiguity, mismatch)

---

## 13. Rust Ejection Path (Mandatory)

### 13.1 Requirement

The parser **must be ejectable** to native Rust such that:

* Generated parser produces **identical AST**
* Equivalence tests are mandatory
* Generated code replaces runtime parser transparently

### 13.2 Authority Rule

`parsing.yaml` remains the source of truth.

Generated Rust is a **compiled projection**, not a new authority.

---

## 14. Implementation Phases (Non-Normative)

1. Spec loader + validation
2. Grammar analysis (nullable, recursion, ambiguity)
3. Runtime parser engine
4. AST construction
5. Unit tests
6. Rust codegen
7. Equivalence tests

---

## 15. Final Position

This parser is:

* Deterministic
* Config-authoritative
* Structurally pure
* Semantically inert
* Testable in isolation
* Bootstrappable

It is **not**:

* A convenience parser
* A forgiving parser
* A semantic engine

---

**End of Document**