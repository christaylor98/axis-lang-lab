# Language Lab

## Definition

**Language Lab** is a controlled pipeline for designing, validating, and evolving programming languages by **explicitly defining how source text becomes Axis Core IR**.

It requires that **lexical rules, syntax, semantics, and lowering behavior are all declared artifacts**, not hidden inside compiler code.
--

If you want next, we can:

* Convert this into a **diagram-only page**
* Define **schemas** for each spec (lexer / lowering / semantics)
* Or write the **“Why Language Lab exists”** motivation page

This page is now internally complete and defensible.
The output of Language Lab is **validated Axis Core IR**.
Execution is explicitly out of scope.

---

## Core principle

> **Meaning must be explicit, inspectable, and stable.**
> No stage of interpretation may invent semantics.

---

## End-to-end pipeline

```
Source Text
  ↓
Lexical Specification
  ↓
Token Stream
  ↓
Syntax Specification
  ↓--

If you want next, we can:

* Convert this into a **diagram-only page**
* Define **schemas** for each spec (lexer / lowering / semantics)
* Or write the **“Why Language Lab exists”** motivation page

This page is now internally complete and defensible.
AST (structure only)
  ↓
Lowering Mapping (meaning assignment)
  ↓
Axis Core IR
  ↓
Semantic Validation
```

---

## Required components

### 1. Lexical Specification

Defines how characters become tokens.

Includes:

* Character set policy (ASCII / Unicode rules)
* Token definitions
* Identifier rules
* Keyword handling
* Whitespace and comment rules
* Literal forms (string, number, etc.)
* Lexical error behavior

Output:

* Token stream or structured lexical errors

Rule:

> Lexing is semantic. It must be explicit and reproducible.

---

### 2. Syntax Specification

Defines how tokens form structured constructs.

Includes:

* Grammar rules
* Operator precedence and associativity
* Grouping constructs
* AST node shapes and guarantees

Input:

* Token stream
  Output:
* AST representing **structure only**

Rule:

> The parser must not encode meaning.

---

### 3. Semantic Specification

Defines what constructs are **allowed to mean**.

Includes:

* Permitted control forms
* Effect discipline (immutability, purity, etc.)
* Type/value rules (or explicit lack thereof)
* Forbidden constructs
* Required invariants

Rule:

> If semantics are not declared, they do not exist.

---

### 4. Lowering Mapping (mandatory)

Defines how AST nodes become Axis Core IR.

This is the **meaning boundary**.

For each AST construct, the mapping specifies:

* Core IR operations emitted
* Registry bindings used
* Effect annotations
* Composition rules
* Validation conditions
* Explicit failure cases

Properties:

* Declarative
* Total (no fall-through)
* Deterministic
* Auditable

Rule:

> No construct may reach Core IR without an explicit lowering rule.

---

### 5. Core IR Registry

Defines the universe of valid semantic operations.

Includes:

* Core IR ops
* Function identities
* Effect signatures
* Calling contracts
* Stability guarantees

Rule:

> Lowering may only reference registered semantics.

---

### 6. Semantic Validator

Validates emitted Core IR against Axis invariants.

Checks:

* Immutability enforcement
* Explicit effects only
* Canonical form
* Registry correctness
* Absence of hidden control flow

Failure here indicates:

* A **language design error**, not just a bad program

---

### 7. Core IR Emitter

Produces the final artifact.

Emits:

* Canonical Axis Core IR
* Optional provenance metadata (token → AST → IR trace)
* Optional diagnostics

This artifact is:

* Language-agnostic
* Execution-agnostic
* Stable over time

---

## Required inputs (summary)

1. Lexical Specification
2. Syntax Specification
3. Semantic Specification
4. Lowering Mapping
5. Core IR Registry
6. Surface Program Source
7. Validation Policy

All are mandatory.
None may be implicit.

---

## Outputs produced

* Validated Axis Core IR
* Structured validation reports
* Optional provenance and debug metadata

---

## What Language Lab explicitly does NOT do

* ❌ Execute programs
* ❌ Optimize performance
* ❌ Infer intent
* ❌ Preserve legacy language quirks
* ❌ Hide semantics in tooling

Execution is handled later by bridges.

---

## One-sentence summary

> Language Lab is a system for turning **explicitly defined languages** into **stable Axis Core IR** by enforcing lexical, syntactic, semantic, and lowering rules as first-class artifacts.
