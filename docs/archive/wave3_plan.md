# Wave 3 Plan — Registry-Bound Function Calls

## Status

PLANNED
Not started

This document is authoritative for Wave 3.

---

## Purpose

Wave 3 introduces **registry-bound function calls**.

This is the first wave where:

* programs can cause observable behavior
* Core IR interacts with externally defined semantics
* name resolution becomes a semantic gate

There is **no separate effect system** in Wave 3.

All observable behavior is expressed by calling **non-deterministic or profile-restricted registry functions**, exactly as defined in `axis-registry-0.1`.

---

## Non-Goals (Explicit)

Wave 3 does NOT include:

* an effects DSL
* effect inference
* mutation
* variables
* loops
* control flow beyond what already exists
* async or concurrency
* runtime configuration
* dynamic registry modification

---

## Governing Constraints (Authoritative)

1. **All callable operations are registry functions**
2. **All calls must resolve during lowering**
3. **No unresolved names enter Core IR**
4. **Determinism and profiles are registry concerns**
5. **Lowering is the only resolution point**

---

## AST Changes (Minimal)

### New Expression Form

Add to `Expr`:

```
Call {
  name: Ident,
  args: Vec<Expr>
}
```

Constraints:

* `name` must be a simple identifier
* no method calls
* no dynamic dispatch
* no higher-order calls

---

## Frontend Surface

### Minimal Syntax

```
foo()
foo(a, b)
```

Constraints:

* call syntax only
* no implicit calls
* no operators lowered to calls in Wave 3

---

## Registry Model (Unchanged)

Registry is defined exactly by **axis-registry-0.1**.

Each entry provides:

* function name
* arity
* determinism flag
* allowed execution profiles
* semantic contract (opaque to Language Lab)

Language Lab does **not** interpret function meaning.

---

## Lowering Rules (Semantic Authority)

Lowering of `Expr::Call` proceeds as follows:

1. Resolve `name` against the active registry
2. Fail if:

   * function is missing
   * arity mismatch
   * profile disallows usage
3. Lower arguments left-to-right
4. Emit a Core IR call node bound to the registry entry

No registry lookup is permitted outside lowering.

---

## Core IR Impact

### Required Core IR Addition

Introduce a single call form, for example:

```
CCall {
  target: RegistryId,
  args: Vec<CoreTerm>
}
```

Constraints:

* target must be a resolved registry identifier
* Core IR contains no symbolic names
* no implicit effects or ordering beyond structure

A Core IR version bump is **allowed** if required.

---

## Validation Rules

Validation must enforce:

* all call targets are resolved
* no symbolic identifiers exist in Core IR
* call arity matches registry contract
* registry metadata is present where required

Validation must reject:

* unresolved calls
* malformed call nodes
* calls in earlier-wave contexts that disallow them

---

## Golden Tests (Mandatory)

Wave 3 must add goldens for:

1. Call to deterministic registry function
2. Call to non-deterministic registry function
3. Call with arguments
4. Conditional containing calls (structure preserved)

Goldens must:

* encode resolved registry IDs
* be byte-stable
* not modify earlier goldens

---

## Execution Strategy

Wave 3 is executed as **one bundled wave**.

Included:

* AST extension
* parser update
* lowering update
* registry integration
* Core IR extension
* validation update
* golden tests

No partial merges.

---

## Acceptance Criteria

Wave 3 is complete when:

* calls resolve exclusively through the registry
* unresolved calls are rejected during lowering
* Core IR contains no symbolic names
* validation enforces registry contracts
* all prior waves remain unchanged

---

## Change Control

Any deviation from this document requires:

* explicit update to this file
* acknowledgement of semantic expansion

---

## End of Document
