# Wave 5 Plan — Runtime Projection and Execution

## Status

PLANNED
Not started

This document is authoritative for Wave 5.

---

## Purpose

Wave 5 introduces **runtime projection** of Core IR.

This is the first wave where:

* Core IR may be executed
* registry-bound calls may have observable behavior
* the compiler hands authority to a runtime

Wave 5 does **not** change compilation semantics.
It defines **how Core IR is projected into execution**, not what programs mean.

---

## Non-Goals (Explicit)

Wave 5 does NOT include:

* new language constructs
* changes to AST
* changes to lowering rules
* Core IR schema changes
* optimisation
* JIT compilation
* concurrency
* async semantics
* scheduling policies
* sandboxing or security hardening
* debugger support

Wave 5 is **minimal execution only**.

---

## Governing Constraints (Hard)

1. **Core IR is the sole input to execution**
2. **Lowering remains the semantic authority**
3. **Runtime does not reinterpret semantics**
4. **Registry functions define observable behavior**
5. **Execution is explicit and opt-in**

---

## Execution Model

### Projection Concept

Execution is a **projection** of Core IR into a runtime model.

* Core IR is not modified
* Core IR is not enriched
* Execution walks Core IR structure directly

The runtime is a **consumer**, not a co-author.

---

## Runtime Architecture (Minimal)

### Components

Wave 5 introduces:

1. **Core IR Interpreter**
2. **Registry Runtime Adapter**
3. **Execution Harness**

No other runtime components are permitted.

---

### Core IR Interpreter

Responsibilities:

* Traverse Core IR
* Evaluate expressions deterministically
* Preserve structural ordering
* Invoke registry-bound functions when encountered

Constraints:

* No optimisation
* No speculative execution
* No reordering
* No caching

---

### Registry Runtime Adapter

Responsibilities:

* Map numeric registry IDs to concrete host functions
* Enforce registry arity at runtime
* Dispatch calls with evaluated arguments

Constraints:

* No dynamic registry mutation
* No fallback resolution
* No symbol lookup by name
* Registry binding is fixed at startup

---

### Execution Harness

Responsibilities:

* Load Core IR bundle
* Load registry runtime implementations
* Invoke execution explicitly
* Capture outputs and errors

Execution must be:

* explicit (no auto-run)
* synchronous
* single-threaded

---

## Error Model

Wave 5 introduces runtime errors only for:

* registry function failure
* invalid runtime binding
* malformed Core IR (should not happen if validation ran)

Runtime errors must be:

* explicit
* surfaced to the caller
* not silently swallowed

---

## Determinism Rules

* Deterministic registry functions must produce stable results
* Non-determinism is allowed **only** via registry functions marked non-deterministic
* The runtime must not introduce new sources of non-determinism

---

## Validation Boundary

Before execution:

* Core IR must pass validation (Wave 3 + Wave 4 rules)
* Runtime does not revalidate semantics

After execution:

* No feedback loop into compilation
* No mutation of Core IR

---

## Testing Strategy

### Required Tests

1. Execute Core IR with deterministic calls
2. Execute Core IR with non-deterministic calls
3. Conditional execution with calls
4. Error propagation from registry functions

Tests must:

* operate on Core IR directly
* not use frontend parsing
* not regenerate goldens

---

## Golden Tests (Optional, Limited)

Wave 5 may include **runtime output goldens** only if:

* output is deterministic
* output is text or numeric
* comparison is stable

Binary output goldens are discouraged in Wave 5.

---

## Execution Strategy

Wave 5 is executed as **one bundled wave**.

Includes:

* runtime interpreter
* registry runtime adapter
* execution harness
* runtime tests

Excludes:

* performance work
* configuration extensions
* concurrency

---

## Acceptance Criteria

Wave 5 is complete when:

* Core IR can be executed end-to-end
* Registry-bound calls invoke host functions
* Deterministic behavior is preserved
* Non-determinism is contained to registry functions
* No compilation semantics changed
* Waves 1–4 remain unchanged

---

## Change Control

Any change that:

* alters Core IR
* alters lowering semantics
* introduces new execution authority

requires a new wave.

---

## End of Document
