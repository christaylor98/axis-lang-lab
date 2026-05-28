# `AXIS_AI_PRIMARY_SURFACE_PARSER_SPEC-0.1.md`

## Status

**Normative – AI-Primary Surface**

This document defines the parsing model for the Axis AI-Primary Explicit Prefix surface.

The parser defined here is **structural, deterministic, and fail-fast** by design.

---

## 1. Purpose

The AI-Primary parser exists to:

* Convert a deterministic token stream into a tree-shaped Concrete Syntax Tree (CST)
* Enforce explicit structure and arity
* Detect malformed programs as early as possible
* Preserve a one-to-one mapping with the semantic AST

The parser is **not a recovery mechanism** and **not an inference engine**.

---

## 2. Design Premise

This surface is optimized for **generation by LLMs**, not for human typing.

Empirical constraints that shape this parser:

* LLMs generate **trees better than stacks**
* LLMs validate **locally**, not globally
* Ambiguity must be rejected, not tolerated
* Arity must be explicit, not inferred

The parser therefore prioritizes **structural correctness over flexibility**.

---

## 3. Non-Goals

The AI-Primary parser does **not** attempt to:

* Infer missing structure
* Recover from syntax errors
* Guess intended semantics
* Apply precedence or associativity rules
* Accept shorthand or sugar
* Support layout-sensitive syntax
* Support multiple syntactic forms for the same construct

Any of the above would reduce determinism and harm AI generation quality.

---

## 4. Core Parsing Model

### 4.1 Expression-Only Grammar

The AI-Primary surface is an **expression language**.

Every valid program is a single expression.

There are no:

* statements
* declarations
* blocks
* implicit sequencing

Composition is expressed solely through expressions.

---

### 4.2 Prefix Form Requirement

All constructs use **prefix notation**.

The construct name appears **before** its operands.

This ensures:

* arity is known before children are parsed
* subtree boundaries are explicit
* validation can occur incrementally

Postfix and infix forms are explicitly forbidden in this surface.

---

### 4.3 Balanced Delimiters

Every construct introduces a balanced delimiter pair.

Example:

```
let( … )
lam( … )
app( … )
```

Balanced delimiters guarantee:

* structural completeness
* local closure validation
* unambiguous subtree termination

---

## 5. Explicit Arity Enforcement

Each construct has a **fixed, explicit arity**.

| Construct | Required Fields   |
| --------- | ----------------- |
| `let`     | name, value, body |
| `lam`     | param, body       |
| `app`     | fn, arg           |
| `if`      | cond, then, else  |
| `var`     | name              |
| `int`     | value             |
| `bool`    | value             |
| `unit`    | none              |

If the required fields are not present **exactly**, parsing fails immediately.

No defaults are permitted.

---

## 6. Named Fields (Critical)

Fields are **named**, not positional.

This is intentional.

### Rationale:

* Positional arguments require implicit ordering
* Implicit ordering increases hallucination risk
* Named fields provide redundancy and self-validation
* Named fields improve partial recovery during generation

The parser must reject:

* missing fields
* unknown field names
* duplicate fields

---

## 7. Grammar Determinism

The grammar must be:

* LL-style (or equivalent deterministic form)
* Free of ambiguity
* Free of precedence rules
* Free of backtracking heuristics

Given the same token stream, the parse result must be identical.

---

## 8. Failure Model

### 8.1 Fail-Fast Requirement

Parsing errors must be raised:

* at the earliest possible token
* at the smallest enclosing construct
* without attempting recovery

Late or cascading failures are considered bugs.

---

### 8.2 Examples of Hard Errors

* Missing required field
* Unexpected token
* Unterminated delimiter
* Duplicate field assignment
* Extra tokens after program end
* Invalid literal form

The parser must not “continue” after any of these.

---

## 9. Relationship to Schema AST

The CST produced by this parser is intentionally **isomorphic** to the Schema AST.

This guarantees:

* trivial CST → Schema projection
* no semantic decisions during parsing
* no grammar-driven semantics

Parsing defines **structure only**.

Semantics remain exclusively in lowering.

---

## 10. Relationship to Other Surfaces

| Surface    | Parser Characteristics      |
| ---------- | --------------------------- |
| Surface-0  | Instructional, compact      |
| Human (H1) | Ergonomic, expressive       |
| AI-Primary | Explicit, redundant, strict |

The AI-Primary parser is intentionally the **most restrictive**.

---

## 11. Explicit Rejections

The following are explicitly forbidden in this parser:

* Operator precedence
* Associativity rules
* Implicit grouping
* Juxtaposition
* Optional delimiters
* Context-dependent grammar rules
* Grammar-level macros

If a feature requires inference, it does not belong in this surface.

---

## 12. Determinism Guarantees

Given identical token streams:

* The parse tree is identical
* Field ordering does not affect semantics
* Errors are raised at identical positions

Any deviation indicates a parser bug.

---

## 13. Rationale Summary

This parser exists because:

* AI generation favors explicit trees
* Determinism beats flexibility
* Redundancy reduces hallucination
* Early errors reduce retries
* Parsing must never encode semantics

This is an engineering trade-off optimized for AI correctness.

---

## 14. Versioning

* This parser spec is versioned independently
* Grammar may evolve
* Semantics must not change
* Relaxations require a new surface version

---

**End of specification**
