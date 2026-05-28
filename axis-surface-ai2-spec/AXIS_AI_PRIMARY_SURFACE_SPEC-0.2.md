# `AXIS_AI_PRIMARY_SURFACE_SPEC-0.2.md`

## Status

**Normative – Experimental (AI-Primary)**
This document defines the AI-primary language surface for Axis.

This surface is designed **explicitly for generation by LLMs** (ChatGPT, Claude, Sonnet, etc.), not for human authorship.

---

## 1. Purpose

The AI-Primary surface exists to maximize:

* Structural correctness of generated programs
* Local verifiability during generation
* Deterministic parsing
* Semantic equivalence with existing Axis expression semantics

The surface is optimized for **probabilistic sequence generators**, not human readability and not parser convenience.

---

## 2. Non-Goals

The AI-Primary surface explicitly does **not** attempt to:

* Be ergonomic for humans
* Minimize token count at all costs
* Encode semantics implicitly
* Provide syntactic sugar
* Introduce new semantic constructs
* Replace other Axis surfaces

This surface is a **projection**, not a semantic authority.

---

## 3. Design Constraints (Hard)

The following constraints are mandatory and were chosen based on observed LLM behavior:

1. **Tree-shaped structure**
2. **Prefix form**
3. **Explicit arity**
4. **Named semantic roles**
5. **Balanced delimiters**
6. **No operator precedence**
7. **No implicit binding or scope**
8. **Fail-fast parsing**

If any of these constraints are violated, the surface is no longer AI-optimal.

---

## 4. Why Postfix (RPN) Was Rejected

Postfix (RPN) encoding was evaluated and rejected as the AI-primary surface because:

* It requires implicit stack tracking
* Structural validity is only known at end-of-input
* Arity errors are global, not local
* LLMs do not reliably maintain abstract stacks
* Repair after partial failure is difficult

RPN remains valid as:

* A compression format
* A parser-friendly encoding
* A non-primary projection

But it is **not optimal for LLM generation**.

---

## 5. Why Explicit Prefix Was Chosen

Explicit prefix form with named fields was chosen because it:

* Announces arity before children
* Allows local validation at every token
* Makes missing or extra structure obvious
* Reduces hallucinated structure
* Enables partial recovery during generation
* Maps cleanly to expression ASTs

This choice minimizes retries and maximizes correctness.

---

## 6. Core Syntax Model

The AI-Primary surface is an **expression language**.

Each construct is represented as a prefix form with explicit, named fields.

### General Shape

```
construct(
  field1 = <expr>,
  field2 = <expr>,
  ...
)
```

There is no positional meaning.
Field names are semantically significant.

---

## 7. Canonical Constructs

The following constructs are defined. No others are permitted.

### `let`

```
let(
  name = <identifier>,
  value = <expr>,
  body = <expr>
)
```

### `lam`

```
lam(
  param = <identifier>,
  body = <expr>
)
```

### `app`

```
app(
  fn = <expr>,
  arg = <expr>
)
```

### `if`

```
if(
  cond = <expr>,
  then = <expr>,
  else = <expr>
)
```

### `var`

```
var(name = <identifier>)
```

### `int`

```
int(value = <integer>)
```

### `bool`

```
bool(value = true | false)
```

### `unit`

```
unit()
```

---

## 8. Grammar Properties

* No operator precedence
* No implicit associativity
* No inferred arity
* No sugar
* No shorthand forms
* No implicit variables

All structure must be explicit.

---

## 9. AST Mapping

This surface maps **1-to-1** onto the existing Axis Schema AST used by the Human (H1) surface.

* No new AST node types are introduced
* No lowering rules are changed
* No Core IR changes are required

The AI-Primary surface and H1 surface are **semantic siblings**.

---

## 10. Determinism Guarantees

Given identical source text:

* Parsing is deterministic
* AST projection is deterministic
* Lowering is deterministic
* Emitted Core IR is deterministic

Any divergence indicates a bug.

---

## 11. Error Model

Errors are detected:

* At the earliest possible token
* At the smallest possible subtree

Examples of hard errors:

* Missing required field
* Unknown field name
* Duplicate field
* Incorrect literal type
* Unterminated construct

No recovery is attempted at the grammar level.

---

## 12. Relationship to Other Surfaces

| Surface    | Role                              |
| ---------- | --------------------------------- |
| Surface-0  | Canonical instruction encoding    |
| Human (H1) | Human-friendly expression syntax  |
| AI-Primary | AI-friendly expression syntax     |
| RPN (AI-1) | Optional compression / projection |

No surface is semantically authoritative except via lowering.

---

## 13. Rationale Summary

This surface was chosen because:

* LLMs generate explicit trees more reliably than stacks
* Redundancy improves correctness
* Named structure reduces hallucination
* Local validation reduces retries
* Parser simplicity is less important than generator correctness

This is an **engineering decision**, not a stylistic one.

---

## 14. Versioning

* This spec is versioned independently
* Syntax may evolve
* Semantics must not

Any change that alters semantics requires a new surface.

---

## 15. Open Questions (Explicit)

* Whether optional whitespace or line breaks should be permitted
* Whether field ordering should be enforced or flexible
* Whether a machine-compact variant should be standardized

These are deferred.

---

**End of specification**