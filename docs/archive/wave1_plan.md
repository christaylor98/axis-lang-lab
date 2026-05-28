# Wave 1 Plan — Multi-Expression Semantics

## Status

PLANNED
Not started
No implementation implied by this document

---

## Purpose

Wave 1 introduces the **first intentional semantic expansion** of Axis Language Lab.

Up to Phase 2, all changes were structural and semantics-preserving.
Wave 1 is the point where **observable program meaning changes by design**.

This wave completes the **core expression semantics** of the language while keeping Core IR unchanged.

---

## High-Level Goal

Support **multiple expressions in a block** with **deterministic sequencing semantics**, such that:

* Expressions are evaluated in order
* The **last expression determines the block value**
* Earlier expressions may exist only for sequencing (future effects)
* Lowering assigns semantics explicitly
* Core IR output changes are intentional and test-guarded

---

## Non-Goals (Explicit)

Wave 1 does NOT include:

* Loops
* Conditionals
* Effects
* Mutation
* Variables
* Registry binding
* Configuration
* Optimisation

Those are later waves.

---

## Semantic Model (Authoritative)

### Block Semantics

A block with multiple expressions:

```
{
  e1;
  e2;
  e3;
}
```

Has the semantic meaning:

* Evaluate `e1`
* Evaluate `e2`
* Evaluate `e3`
* Result of block is value of `e3`

No implicit returns.
No short-circuiting.
No reordering.

---

## Structural Changes (AST)

### AST Extensions

* `Block` becomes:

  ```
  Block {
    exprs: Vec<Expr>
  }
  ```
* `Expr` remains minimal initially:

  * `UnitLit`
  * (future variants added later)

### Compatibility Rule

* A single-expression block:

  ```
  Block { exprs: [UnitLit] }
  ```

  MUST be semantically equivalent to Phase 2 behavior.

---

## Lowering Semantics (Core Authority)

Lowering defines meaning.

### Lowering Rules

1. A block with one expression lowers exactly as before.
2. A block with multiple expressions lowers to **explicit sequencing**.
3. Sequencing is encoded using existing Core IR constructs.
4. No new Core IR nodes are introduced in Wave 1.

### Required Properties

* Total (no fallthrough)
* Deterministic
* Explicit
* No frontend inference
* No validation inference

---

## Core IR Impact

### Core IR Version

* Core IR version remains unchanged

### Structural Impact

* Core IR byte output **will change** for multi-expression blocks
* This is intentional and required

### Compatibility Guarantee

* Phase 2 golden output remains valid and unchanged
* New golden outputs are added for Wave 1 cases

---

## Validation Rules

Validation must be updated to:

* Accept the new Core IR shapes produced by sequencing
* Continue rejecting:

  * unexpected node types
  * effects
  * registry references
  * node_id presence (unless later wave explicitly enables)

Validation must **not** assign semantics.

---

## Golden Tests (Mandatory)

Wave 1 introduces new golden tests.

### Required Tests

1. Single-expression block

   * Must match Phase 2 golden exactly
2. Two-expression block

   * Must produce stable, intentional Core IR bytes
3. Three-expression block

   * Must demonstrate left-to-right sequencing

### Test Properties

* Byte-for-byte comparison
* No normalization
* No regeneration of earlier goldens

---

## Execution Strategy (Bundled)

Wave 1 is executed as a **single bundled effort**, not micro-tasks.

### One Execution Window Includes

* AST update
* Parser update
* Lowering update
* Validation update
* Golden tests
* Documentation update

No partial merges.

---

## Acceptance Criteria (Wave Gate)

Wave 1 is complete when:

* Multi-expression blocks are supported end-to-end
* Phase 2 golden tests still pass unchanged
* New golden tests pass
* Validation enforces all invariants
* No semantic ambiguity remains
* No configuration is required to enable the feature

---

## Why Wave 1 Is Safe Now

Wave 1 is possible because:

* Core IR is stable
* Lowering is authoritative
* Validation is strict
* Golden tests are in place
* Phase 2 equivalence is proven

This wave is the **first controlled semantic expansion**, not an experiment.

---

## Next Waves (Context Only)

* **Wave 2:** Conditionals
* **Wave 3:** Effects and registry binding
* **Wave 4:** Configuration-driven language surfaces

These are explicitly out of scope for Wave 1.

---

## Change Control

Any deviation from this plan requires:

* Updating this document
* Explicit acknowledgment that semantics are changing

---

## End of Document
