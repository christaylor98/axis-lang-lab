# Axis Core IR — User-Facing Explanation

**Status:** User Documentation
**Applies to:** Core IR version 0.2

---

## What Core IR Is

Core IR is the **canonical intermediate representation** for Axis programs.

It is:

* The output of lowering (compilation)
* The input to execution (runtime)
* The source of truth for program semantics
* A numeric, name-free, configuration-free representation

Core IR is **not**:

* An optimization IR
* An execution format (though it is executable)
* A debugging format
* A human-editable language

Core IR exists to **freeze semantics** in a form that downstream consumers (runtimes, validators, analyzers) can rely on.

---

## What Guarantees Core IR Provides

### 1. Semantic Completeness

A Core IR bundle **fully determines** what a program means.

* No external context is required
* No name resolution is needed
* No inference is permitted
* Removing metadata does not change meaning

If semantics are not explicit in Core IR, **they do not exist**.

### 2. Determinism

Given the same surface input and configuration, lowering produces **structurally identical** Core IR.

* Ordering is canonical
* Node identity assignment follows a fixed scheme
* No non-deterministic behavior exists in the compiler

### 3. Immutability

Once emitted, Core IR **never changes**.

* No optimization passes modify it
* No runtime feedback alters it
* No dynamic rewriting occurs

Core IR is a **frozen artifact**.

### 4. Configuration Independence

Core IR contains **no configuration**.

* No profile information
* No policy toggles
* No build settings

Configuration influences **which Core IR is produced**, not **what Core IR means**.

---

## What Core IR Intentionally Excludes

### 1. Names

Core IR contains **no symbolic names**.

* All function references are numeric registry IDs
* All variables are numeric De Bruijn indices or similar
* All scoping is explicit

Name resolution happens **during lowering only**.

### 2. Types (Currently)

Core IR 0.2 does **not** include type information.

* No type annotations
* No type checking at the Core IR level
* Type safety is enforced by lowering (currently implicit)

This may change in future Core IR versions.

### 3. Metadata as Semantics

Core IR metadata (annotations, spans, node IDs) is **non-authoritative**.

* Removing all metadata must not change semantics
* Tools may use metadata for diagnostics
* Runtime must not interpret metadata

### 4. Optimization Hints

Core IR contains **no optimization hints**.

* No inline pragmas
* No loop unrolling directives
* No vectorization markers

If optimization occurs, it happens **outside** Core IR.

---

## Why Core IR Is Numeric

Core IR uses numeric handles (registry IDs, variable indices) instead of names because:

1. **Name resolution is semantic**

   * Resolution happens during lowering
   * Lowering is the semantic authority
   * After lowering, names are no longer needed

2. **Numeric IDs are unambiguous**

   * No namespace collisions
   * No shadowing concerns
   * Deterministic dispatch

3. **Consumers don't need to resolve names**

   * Runtime doesn't do name lookup
   * Validators don't need symbol tables
   * Analysis tools work directly on structure

This makes Core IR **simpler to consume** and **harder to misinterpret**.

---

## Why Core IR Is Name-Free

Names are a **surface-level convenience**.

In Core IR:

* All function calls are resolved to numeric registry IDs
* All variable references are resolved to numeric bindings
* All scoping is explicit

This ensures:

* No ambiguity
* No implicit resolution
* No hidden dependencies

---

## Why Core IR Is Configuration-Free

Configuration influences **what gets compiled**, not **what the compiled artifact means**.

Examples:

* **Configuration:** "Use profile `safe`"
  **Effect on Core IR:** Some functions are rejected during lowering
  **No effect on Core IR:** The emitted Core IR structure itself

* **Configuration:** "Use registries A and B"
  **Effect on Core IR:** Function names resolve to IDs from A and B
  **No effect on Core IR:** The numeric IDs are fixed in the bundle

Core IR is **policy-free** because semantics must be **universal** across execution environments.

---

## Core IR Node Set (Closed)

Core IR 0.2 has **exactly eight node kinds**:

1. **CIntLit** — Integer literal
2. **CBoolLit** — Boolean literal
3. **CUnitLit** — Unit literal (empty value)
4. **CLam** — Lambda (function abstraction)
5. **CLet** — Let binding (introduces a variable)
6. **CIf** — Conditional expression
7. **CVar** — Variable reference
8. **CApp** — Function application (call)

No other nodes exist in Core IR 0.2.

All surface constructs (loops, pattern matching, modules, etc.) must be desugared into these eight nodes before emission.

This is a **hard constraint**.

---

## Node Identity (`node_id`)

Each Core IR node may have an optional `node_id`.

Properties:

* Assigned by the lowering compiler
* Unique within the bundle
* Monotonically increasing integers
* **No semantic meaning**

`node_id` is used for:

* Debugging
* Diagnostics
* Tool integration

`node_id` must **not** be used for:

* Semantic classification
* Policy encoding
* Execution decisions

---

## Annotations

Core IR bundles may include **side-band annotations**.

Annotations are:

* Stored separately from Core IR nodes
* Referenced by annotation identifiers
* **Non-authoritative**

Annotations may contain:

* Source spans (for error messages)
* Metadata (for tooling)
* Hints (for analysis)

Annotations must **not** encode semantics.

A Core IR bundle remains valid and complete if all annotations are removed.

---

## Bundle Structure

A Core IR bundle is a serialized data structure containing:

```
{
  "version": "0.2",
  "core_term": <root Core IR node>,
  "string_table": [...],   // optional
  "annotations": [...],    // optional
  "entrypoint_name": "...", // optional
  "entrypoint_id": ...     // optional
}
```

Required fields:

* `version` — Must be "0.2"
* `core_term` — The semantic root

All other fields are optional and non-authoritative.

---

## Mental Model

Think of Core IR as:

* **Assembly for semantics** — Low-level, explicit, canonical
* **Serialized proof of meaning** — If it's in Core IR, lowering decided it
* **Universal exchange format** — Any runtime that accepts Core IR 0.2 can execute it

Core IR is **not** meant to be:

* Human-written
* Human-readable (though it is inspectable)
* Subject to interpretation

Core IR is a **contract** between the compiler and all downstream consumers.

---

## Relationship to Lowering

Lowering **produces** Core IR.

Core IR **reflects** lowering decisions.

If lowering assigns semantics, Core IR makes them explicit.

If lowering rejects a program, no Core IR is emitted.

Lowering and Core IR together form the **semantic boundary** of Axis Language Lab.

---

## Relationship to Execution

Execution **consumes** Core IR.

The runtime interpreter:

* Reads Core IR structure
* Evaluates nodes in canonical order
* Dispatches registry calls by numeric ID
* Produces observable behavior

The runtime does **not**:

* Modify Core IR
* Reinterpret semantics
* Optimize or transform

Execution is a **projection** of Core IR, not a collaborator in defining meaning.

---

## Trust Model

Core IR establishes a **zero-trust semantic contract**.

Consumers of Core IR may assume:

* Semantics are explicit
* No hidden behavior exists
* No external resolution is required
* All necessary information is present

Consumers must **not** assume:

* Optimization
* Type safety (not yet enforced)
* Security properties
* Portability guarantees

Core IR guarantees **semantic clarity**, not operational properties.

---

## Stability and Versioning

Core IR 0.2 is **frozen**.

Changes to:

* Node set
* Node semantics
* Bundle structure
* Invariants

require a **new major version** (e.g., Core IR 0.3).

Additive changes (e.g., new optional metadata) may be introduced in **minor versions** (e.g., Core IR 0.2.1).

All Core IR bundles are versioned explicitly. Consumers must verify version compatibility before processing.

---

## Key Takeaways

* Core IR is **semantic**, not structural
* Core IR is **numeric**, not symbolic
* Core IR is **complete**, not partial
* Core IR is **frozen**, not mutable
* Core IR is the **canonical artifact** for program meaning

---

**End of Core IR Explanation**
