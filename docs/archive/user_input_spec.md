Perfect — below is a **drop-in revision** of the spec that **adds concrete, detailed examples for every user input type**, while keeping the **authority boundaries unchanged**.

This is written so that:

* a language author can **copy/paste and adapt**
* an implementer can **mechanically validate**
* nothing turns into a “tutorial DSL”
* nothing weakens the semantic contract

You can replace the previous file entirely with this version.

---

# Language Lab — Language Author Input Specification (with Examples)

**Status:** Normative
**Scope:** All language author–supplied inputs
**Audience:** Language authors, Language Lab implementers, bridge authors

---

## 1. Purpose

This document specifies **all user-supplied inputs required by Language Lab** to define a programming language, and provides **concrete examples** of each.

Language Lab exists to turn **explicitly defined language artifacts** into **validated Axis Core IR**.

Execution, optimisation, and target adaptation are explicitly out of scope.

---

## 2. Foundational Constraints (Unchanged)

1. All semantics must be explicit
2. Lowering is the sole semantic authority
3. Language Lab always emits Axis Core IR
4. Core IR is Cap’n Proto encoded
5. No custom IRs exist inside Language Lab
6. All configuration uses existing formats (YAML / TOML)
7. Bridges adapt Core IR, not Language Lab

---

## 3. Lexical Specification

### 3.1 Purpose

Defines how source text becomes tokens.

### 3.2 Format

**YAML**

### 3.3 Example

```yaml
lexer:
  charset: ascii
  case_sensitive: true

  tokens:
    - name: IDENT
      pattern: "[A-Za-z_][A-Za-z0-9_]*"

    - name: INT
      pattern: "[0-9]+"

    - name: BOOL
      pattern: "true|false"

  keywords:
    - text: "if"
      token: IF

    - text: "let"
      token: LET

  operators:
    - text: "&&"
      token: AND_AND

    - text: "=="
      token: EQ_EQ

  punctuation:
    - text: "("
      token: LPAREN
    - text: ")"
      token: RPAREN
    - text: "{"
      token: LBRACE
    - text: "}"
      token: RBRACE

  whitespace:
    pattern: "[ \\t\\n\\r]+"
    skip: true

  comments:
    - pattern: "//.*"
      skip: true
```

### 3.4 Authority

* Structural only
* No semantics
* Deterministic

---

## 4. Syntax Specification

### 4.1 Purpose

Defines how tokens form structure.

### 4.2 Format

**YAML**

### 4.3 Example

```yaml
parser:
  start: Program

  grammar:
    Program:
      - Function*

    Function:
      - "fn" IDENT "(" ")" Block

    Block:
      - "{" Statement* "}"

    Statement:
      - LetStmt
      - ExprStmt

    LetStmt:
      - "let" IDENT "=" Expr ";"

    ExprStmt:
      - Expr ";"

    Expr:
      - IfExpr
      - BinaryExpr
      - CallExpr
      - Primary

    IfExpr:
      - "if" Expr Block "else" Block

    BinaryExpr:
      - Expr "&&" Expr
      - Expr "==" Expr

    CallExpr:
      - IDENT "(" ArgList? ")"

    ArgList:
      - Expr ("," Expr)*

    Primary:
      - INT
      - BOOL
      - IDENT
      - "(" Expr ")"
```

### 4.4 Authority

* Structural only
* Parser must not encode meaning

---

## 5. AST Schema

### 5.1 Purpose

Defines the **exact contract** between parsing and lowering.

### 5.2 Format

**YAML**

### 5.3 Example

```yaml
ast:
  nodes:
    Program:
      fields:
        functions: [Function]

    Function:
      fields:
        name: Ident
        body: Block

    Block:
      fields:
        statements: [Statement]

    Statement:
      union:
        - LetStmt
        - Expr

    LetStmt:
      fields:
        name: Ident
        value: Expr

    Expr:
      union:
        - IntLit
        - BoolLit
        - IdentRef
        - IfExpr
        - BinaryExpr
        - CallExpr

    IntLit:
      fields:
        value: int

    BoolLit:
      fields:
        value: bool

    IdentRef:
      fields:
        name: Ident

    IfExpr:
      fields:
        cond: Expr
        then_block: Block
        else_block: Block

    BinaryExpr:
      fields:
        op: string
        lhs: Expr
        rhs: Expr

    CallExpr:
      fields:
        name: Ident
        args: [Expr]
```

### 5.4 Authority

* Structural
* No behaviour
* Closed and deterministic

---

## 6. Semantic Specification

### 6.1 Purpose

Declares semantic constraints independently of syntax.

### 6.2 Format

**TOML**

### 6.3 Example

```toml
[semantics]
immutability = true
explicit_effects_only = true
allow_recursion = false

[semantics.forbidden]
implicit_mutation = true
implicit_control_flow = true
```

### 6.4 Authority

* Declarative constraints only
* Does not emit IR

---

## 7. Lowering Rules (Semantic Authority)

### 7.1 Purpose

Assigns meaning by mapping AST → Core IR.

### 7.2 Format

**YAML**

### 7.3 Example: Literal Lowering

```yaml
lowering:
  rules:
    - match:
        node: IntLit
        value: v
      emit:
        op: core.int_lit
        args: [v]

    - match:
        node: BoolLit
        value: v
      emit:
        op: core.bool_lit
        args: [v]
```

---

### 7.4 Example: `if` Expression

```yaml
    - match:
        node: IfExpr
        cond: c
        then_block: t
        else_block: e
      emit:
        op: core.if
        args:
          - lower: c
          - lower_block: t
          - lower_block: e
```

---

### 7.5 Example: Operator as Foreign Function (`&&`)

```yaml
    - match:
        node: BinaryExpr
        op: "&&"
        lhs: a
        rhs: b
      emit:
        call: "__and__"
        args:
          - lower: a
          - lower: b
      effects:
        - control_flow
```

This makes `&&` **just a registered function**, not special syntax.

---

## 8. Escape Hatch DSL (Lowering Only)

### 8.1 Purpose

Handle complex structural cases without executable code.

### 8.2 Example: Guarded Lowering

```yaml
    - match:
        node: BinaryExpr
        op: "=="
        lhs: a
        rhs: b
      requires:
        - same_type(a, b)
      emit:
        call: "__eq__"
        args:
          - lower: a
          - lower: b
      on_fail:
        error: "type_mismatch_in_equality"
```

### 8.3 Constraints

* Declarative only
* No loops
* No conditionals
* No direct IR emission

---

## 9. Core IR Registry

### 9.1 Purpose

Defines legal semantic operations.

### 9.2 Format

**TOML**

### 9.3 Example

```toml
[registry]

[[registry.fn]]
name = "__and__"
arity = 2
effects = ["control_flow"]
determinism = "short_circuit"

[[registry.fn]]
name = "__eq__"
arity = 2
effects = []
determinism = "deterministic"

[[registry.fn]]
name = "print"
arity = 1
effects = ["io"]
determinism = "nondeterministic"
```

---

## 10. Foreign Functions

### 10.1 Example

```toml
[[registry.fn]]
name = "eval"
arity = 1
effects = ["unknown"]
determinism = "unknown"
opaque = true
```

This allows highly dynamic behaviour **without lying**.

---

## 11. Annotations

### 11.1 Purpose

Carry non-semantic metadata.

### 11.2 Example

```yaml
annotations:
  - target: Function
    name: hot_path
    value: true

  - target: CallExpr
    name: legacy_behavior
    value: "v1_compat"
```

Annotations never affect validity.

---

## 12. Validation Policy

### 12.1 Format

**TOML**

### 12.2 Example

```toml
[validation]
strict = true

[validation.rules]
require_all_calls_registered = true
reject_unknown_effects = true
enforce_canonical_form = true
```

Validation failure = **language definition error**.

---

## 13. Acceptance Criterion (v0)

Language Lab v0 is accepted when:

* All inputs above are supplied
* Language Lab emits Cap’n Proto Core IR
* Output matches existing PoC semantics
* Existing PoC bridge consumes output unchanged

---

## 14. Final Statement

**Language Lab defines languages by explicit artifacts, assigns meaning exactly once during lowering, and emits Axis Core IR as the single semantic handoff.**

Everything else is projection.

