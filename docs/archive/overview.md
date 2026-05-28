# Axis Language Lab — Overview

**Status:** User Documentation
**Applies to:** Post-Wave 5 (Execution)

---

## What Axis Language Lab Is

Axis Language Lab is a **compiler construction laboratory** for the Axis programming language.

It is:

* A deterministic compiler from Axis surface syntax to Core IR
* A testbed for semantic design decisions
* A reference implementation for Core IR production
* A runtime execution environment for Core IR

It is **not**:

* A production compiler
* An optimizing compiler
* A multi-target compiler
* A complete language implementation

Axis Language Lab exists to establish **semantic authority** and **canonical lowering behavior** for the Axis language.

---

## What Problem It Solves

Programming languages require a **single source of truth** for what programs mean.

Axis Language Lab provides this by:

* Making semantics **explicit** during lowering
* Producing a **canonical intermediate representation** (Core IR)
* Eliminating ambiguity about program meaning
* Defining execution behavior as a **projection** of Core IR

The lab ensures that:

* Semantics are never implicit
* Meaning is assigned exactly once (during lowering)
* All downstream consumers operate on the same canonical representation

---

## What It Does Not Solve

Axis Language Lab deliberately excludes:

* Production-grade performance
* Optimization
* Multiple execution backends
* Tooling integration (LSP, debuggers, etc.)
* Multi-file projects with complex dependency management
* Dynamic language features
* Concurrency or async semantics
* Security hardening

These are **deferred** to later gates or production implementations.

---

## Where Semantics Live

**Semantics are defined exclusively during lowering.**

The lowering phase is the **sole semantic authority** for Axis programs.

Before lowering:

* Surface syntax has **structure only**
* No semantic meaning is assigned
* AST nodes are purely structural

After lowering:

* Semantics are **frozen** in Core IR
* Meaning is explicit and complete
* No further semantic decisions occur

Core IR is the **canonical semantic artifact**.

---

## Where Execution Begins

Execution begins when a **validated Core IR bundle** is provided to the runtime interpreter.

The execution boundary is:

* **Input:** A Core IR bundle (version 0.2)
* **Process:** Deterministic interpretation of Core IR structure
* **Output:** Observable behavior via registry-bound function calls

The runtime is a **consumer** of Core IR, not a co-author.

It does not:

* Modify Core IR
* Infer semantics
* Optimize programs
* Introduce new behavior

Execution is a **projection** of Core IR, not a source of meaning.

---

## Architectural Invariants

The following are **non-negotiable invariants** across all waves:

1. **Lowering is the sole semantic authority**
2. **Core IR is configuration-free**
3. **Core IR is name-free (uses numeric handles only)**
4. **Execution does not alter semantics**
5. **Registry functions define all observable behavior**
6. **Validation occurs before execution**
7. **Semantics are deterministic and explicit**

Any violation of these invariants requires opening a new wave with explicit justification.

---

## Development Model

Axis Language Lab uses a **wave-based development model**.

Each wave:

* Introduces **one** well-defined capability
* Produces a **freeze note** when complete
* Locks semantics permanently

Frozen waves are **immutable**. Changes require new waves.

This ensures:

* Semantic stability
* Testability
* Incremental correctness

---

## Current State

As of Wave 5:

* Surface parsing is complete for a minimal expression language
* Lowering to Core IR is deterministic and total
* Core IR validation enforces structural correctness
* Registry-bound function calls are supported
* Configuration-driven profile selection works
* **Execution of Core IR is functional and tested**

---

## Key Documents

* [Pipeline](pipeline.md) — End-to-end flow from surface to execution
* [Core IR](core_ir.md) — Core IR mental model
* [Execution](execution.md) — Runtime execution semantics
* [Registry](registry.md) — Registry-bound function model
* [Frozen](frozen.md) — What is stable and what is evolving

---

## Audience

This documentation is written for:

* Compiler engineers
* Language designers
* Systems programmers
* Reviewers evaluating semantic correctness

It assumes:

* Familiarity with compiler construction
* Understanding of intermediate representations
* Skepticism toward implicit behavior

---

## Trust Model

Axis Language Lab is designed for **zero-trust semantic authority**.

No component may:

* Invent semantics
* Defer meaning to consumers
* Introduce implicit behavior
* Rely on unstated assumptions

If a semantic decision is not explicit in lowering or Core IR, **it does not exist**.

---

**End of Overview**
