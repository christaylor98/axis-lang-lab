# Axis Language Lab — Wave 5 Freeze Note

**Wave:** 5
**Title:** Runtime Projection and Execution
**Status:** FROZEN
**Applies to:** Core IR 0.2 + integrated Rust bridge

---

## Purpose of Wave 5

Wave 5 introduces **runtime execution** of Axis Core IR.

This is the first wave in which:

* Core IR can be executed end-to-end
* Registry-bound calls may produce observable behavior
* Semantic authority is intentionally handed from the compiler to a runtime

Wave 5 does **not** change what programs mean.

It defines **how Core IR is projected into execution**, not how semantics are derived.

---

## What Wave 5 Establishes

Wave 5 establishes the following as **authoritative and stable**:

1. **Core IR is executable**
2. **Execution is a projection of Core IR**
3. **Lowering remains the sole semantic authority**
4. **Registry functions define all observable behavior**
5. **Execution authority begins at the runtime boundary**

---

## Execution Model (Frozen)

### Projection Model

* Execution consumes Core IR **as-is**
* Core IR is not modified, enriched, or annotated
* No execution metadata is embedded into Core IR
* The runtime is a **consumer**, not a co-author

Execution walks Core IR structure directly.

---

### Interpreter Semantics

The runtime interpreter:

* Traverses Core IR deterministically
* Preserves structural ordering
* Evaluates arguments left-to-right
* Executes conditionals explicitly
* Invokes registry-bound calls when encountered

The interpreter performs:

* No optimisation
* No caching
* No reordering
* No speculative execution
* No concurrency
* No async execution

Any such behavior requires a new wave.

---

### Registry Runtime Binding

Registry dispatch is:

* **Numeric-only**
* Fixed at startup
* Immutable during execution

Properties:

* Calls are dispatched by numeric registry ID
* Arity is enforced at runtime
* No name lookup occurs
* No fallback resolution exists
* No dynamic registry mutation is permitted

All observable behavior originates from registry functions.

---

## Error Model (Frozen)

Wave 5 permits runtime errors **only** in the following cases:

1. Registry function failure
2. Invalid or missing runtime registry bindings
3. Malformed Core IR (should not occur if validation ran)

Runtime errors are:

* Explicit
* Immediately surfaced to the caller
* Never swallowed
* Never reinterpreted

The runtime does not attempt recovery or policy decisions.

---

## Determinism Guarantees

* Deterministic registry functions must produce stable results
* Non-determinism is permitted **only** inside registry functions explicitly defined as such
* The runtime introduces no new sources of non-determinism

The interpreter itself is deterministic by construction.

---

## Validation Boundary

* Core IR **must** pass validation before execution
* Runtime execution does not revalidate semantics
* No feedback loop exists from execution to compilation
* Core IR is not mutated during or after execution

Validation remains a compiler concern, not a runtime concern.

---

## Testing Scope (Frozen)

Wave 5 runtime tests:

* Operate on Core IR directly
* Do not use frontend parsing
* Do not invoke lowering
* Do not regenerate compilation goldens

Tests verify execution correctness only.

---

## Explicit Non-Goals (Confirmed)

Wave 5 explicitly does **not** include:

* New language constructs
* AST changes
* Lowering rule changes
* Core IR schema changes
* Optimisation
* JIT compilation
* Concurrency or async semantics
* Scheduling policies
* Security hardening
* Debugger support
* Instrumentation hooks
* Configuration expansion

Any such work requires a new wave.

---

## Architectural Consequences (Important)

With Wave 5 frozen:

* Core IR is now a **load-bearing execution artifact**
* The bridge is no longer a proof-of-concept
* Any change to execution behavior is a **semantic event**
* Any reinterpretation of Core IR during execution is forbidden

Future waves must treat Wave 5 execution semantics as fixed.

---

## Completion Statement

Wave 5 is complete and frozen.

Core IR execution exists, is explicit, deterministic, registry-driven, and semantically constrained.

No further changes may be made under the scope of Wave 5.

---

**End of Wave 5 Freeze Note**
