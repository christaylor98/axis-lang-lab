# axis-language-surface-0.0.md

## Axis Surface-0 — Canonical AI Surface

**Version 0.0**
**Status:** Normative

---

## 0. Purpose

Axis Surface-0 is the **canonical, AI-native surface** for Axis.

It is:

* not human-oriented
* not expressive
* not ergonomic
* not semantic

Surface-0 exists solely to provide a **textual encoding of a computation graph**
that present-day LLMs can emit **without ambiguity**.

---

## 1. Design Principles (Normative)

Surface-0 obeys the following invariants:

1. One meaning only
2. No inference
3. No precedence
4. No hierarchy
5. No optional syntax
6. No human sugar
7. Deterministic lowering
8. Canonical serialization

If two Surface-0 texts differ, they **MUST NOT** represent the same program.

---

## 2. Program Structure

A Surface-0 file is a **bag of functions**.

* File boundaries have **no semantic meaning**
* Function order has **no semantic meaning**
* Concatenation of files is semantics-preserving

---

## 3. Function Definition

```
fn <IDENT>
<instruction>*
end
```

Rules:

* `fn` begins a function
* `end` terminates it
* No nesting
* No parameters declared here
* No return syntax

Each function is validated **independently**.

---

## 4. Execution Model (Normative)

Surface-0 describes a **stack-based graph construction program**.

* Instructions manipulate a virtual stack
* Stack effects are defined **only by registry operators**
* Surface-0 itself defines no semantics

At `end`:

* stack height MUST equal **exactly 1**
* otherwise: compile-time error

---

## 5. Instructions (Exhaustive)

Surface-0 admits exactly three instructions.

### 5.1 Argument Reference

```
arg <INT>
```

Pushes the N-th argument (zero-based) onto the stack.

Argument arity is defined by the registry.

---

### 5.2 Literal

```
lit <TYPE> <VALUE>
```

Examples:

```
lit int 42
lit bool true
lit str "hello"
```

Literal types and legality are profile-defined.

---

### 5.3 Operator Call

```
call <IDENT>
```

* `<IDENT>` MUST resolve to a registry operator
* Operator defines:

  * stack pop count
  * stack push count
  * type rules
  * effects

Surface-0 does not encode arity.

---

## 6. Name Resolution

Surface-0 defines:

* no imports
* no aliases
* no qualification
* no overloading

All `call` identifiers are resolved by **exact match** against the registry.

Failure to resolve is a compile-time error.

---

## 7. Validation Rules (Mandatory)

A Surface-0 function is valid iff:

1. No instruction causes stack underflow
2. All `call` targets resolve
3. Stack height is known at every instruction
4. Stack height at `end` is exactly 1
5. All literals are profile-admitted

No recovery is permitted.

---

## 8. Canonical Formatting

Surface-0 has **exactly one valid print form**:

* One instruction per line
* Single spaces between tokens
* No trailing whitespace
* No comments
* No blank tokens

Canonicalization MUST occur before hashing or comparison.

---

## 9. Relationship to Binary and Core IR

Surface-0 is:

* the **AI-authoring surface**
* the **canonical textual form**

Binary graph representations and Core IR:

* are **derived artifacts**
* MUST be deterministically derived from Surface-0
* MUST NOT introduce semantics

---

## 10. Explicit Non-Goals (Locked)

Surface-0 does not include:

* variables
* names
* blocks
* conditionals
* loops
* pattern matching
* projection
* records
* enums
* mutation
* control flow syntax
* comments
* human readability

All such constructs exist **above** or **below** this surface.

---

## 11. Summary

Surface-0 is:

* not a programming language
* not an IR
* not a human interface

It is a **canonical, AI-native graph assembly encoding**.

> **Surface-0 is text that behaves like binary.**