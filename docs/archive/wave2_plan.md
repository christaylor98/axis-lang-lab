# Wave 2 Plan — Conditional Expressions

## Status

PLANNED
Not started
No implementation implied by this document

---

## Purpose

Wave 2 introduces **branching semantics** via a minimal conditional expression.

This is the first wave where **control flow becomes data-dependent**, while still avoiding:

* loops
* effects
* mutation
* variables

Wave 2 remains expression-oriented and deterministic.

---

## High-Level Goal

Support an **if expression** with block bodies:

```
if cond {
  then_exprs...
} else {
  else_exprs...
}
```

With the semantic meaning:

* Evaluate `cond`
* If true: evaluate `then` block
* Else: evaluate `else` block
* Result of the `if` expression is the result of the selected block

---

## Non-Goals (Explicit)

Wave 2 does NOT include:

* Loops
* Short-circuit boolean operators
* Pattern matching
* Effects
* Variables
* Registry binding
* Configuration
* Optimisation

---

## Semantic Model (Authoritative)

### Conditional Semantics

For:

```
if C { T } else { E }
```

Semantics are:

1. Evaluate `C`
2. Exactly one branch is evaluated
3. Result is the value of the chosen branch
4. Non-chosen branch is not evaluated
5. No implicit effects

Conditionals are **expressions**, not statements.

---

## Structural Changes (AST)

### AST Extensions

* Add to `Expr`:

  ```
  If {
    cond: Expr,
    then_block: Block,
    else_block: Block
  }
  ```
* Reuse existing `Block { exprs: Vec<Expr> }`
* No new literal types introduced in this wave

---

## Frontend Surface

### Accepted Syntax (Minimal)

```
if <expr> { <block> } else { <block> }
```

Constraints:

* `else` is mandatory
* Blocks must be present on both branches
* No dangling `if`
* No `else if` sugar

Parser must remain strict.

---

## Lowering Semantics (Core Authority)

### Lowering Rules

* `If` lowers to explicit conditional Core IR form
* Branch blocks are lowered independently
* Branch values are unified explicitly
* No implicit fallthrough
* No duplication of branch evaluation

### Core IR Constraints

* Core IR **may need to grow** in this wave
* If so:

  * exactly one new conditional construct
  * no sugar nodes
  * semantics must be explicit

Lowering remains the **only semantic authority**.

---

## Core IR Impact

### Versioning

* Core IR version MAY bump (only if required)
* Any bump must be intentional and documented

### Structural Impact

* Conditional form becomes part of Core IR
* Wave 1 goldens remain unchanged
* New Wave 2 goldens added

---

## Validation Rules

Validation must enforce:

* Conditional nodes are well-formed
* Both branches produce values
* No effects
* No registry references
* No node_id presence
* No unreachable constructs

Validation does NOT:

* infer semantics
* normalize conditionals
* optimize branches

---

## Golden Tests (Mandatory)

### Required Golden Tests

1. Simple if with unit branches
2. Nested if expressions
3. If inside a multi-expression block

### Properties

* Byte-for-byte comparison
* Wave 1 goldens remain untouched
* Failure indicates semantic drift

---

## Execution Strategy (Bundled)

Wave 2 is executed as **one bundled wave**, not micro-tasks.

Includes in one execution window:

* AST extension
* Parser update
* Lowering update
* Core IR update (if required)
* Validation update
* Golden tests

No partial merges.

---

## Acceptance Criteria (Wave Gate)

Wave 2 is complete when:

* Conditionals work end-to-end
* Wave 1 behavior unchanged
* Golden tests prove intentional IR changes
* Validation enforces all invariants
* No configuration is required to enable the feature

---

## Why Wave 2 Is Safe Now

Wave 2 is safe because:

* Sequencing semantics are already explicit
* Lowering is authoritative
* Validation is conservative
* Golden tests exist
* Wave 1 discipline is proven

---

## Next Waves (Context Only)

* **Wave 3:** Effects and registry binding
* **Wave 4:** Configuration-driven language surfaces

These are out of scope for Wave 2.

---

## Change Control

Any deviation from this plan requires:

* Updating this document
* Explicit acknowledgment of semantic change

---

## End of Document