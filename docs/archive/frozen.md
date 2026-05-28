# Axis Language Lab — What Is Frozen

**Status:** User Documentation
**Last Updated:** Post-Wave 5

---

## Purpose

This document lists **what is stable** and **what is evolving** in Axis Language Lab.

Frozen components are **immutable** and may be relied upon.

Non-frozen components are **experimental** and subject to change.

---

## Frozen Waves

The following waves are **complete and frozen**:

### Wave 1: Multi-Expression Semantics

**Status:** FROZEN
**Introduced:** Deterministic sequencing of multiple expressions in blocks

Guarantees:

* Block expressions evaluate in order
* Last expression determines block value
* Lowering rules are fixed
* Golden outputs are canonical

### Wave 2: Conditional Expressions

**Status:** FROZEN
**Introduced:** `if` expressions with explicit branches

Guarantees:

* Conditional semantics are explicit
* Only the selected branch is evaluated
* Core IR conditional nodes are stable
* Lowering rules are deterministic

### Wave 3: Registry-Bound Function Calls

**Status:** FROZEN
**Introduced:** Explicit registry-based function calls

Guarantees:

* All calls resolve to registry entries during lowering
* Numeric registry IDs are deterministic
* No runtime name resolution
* Profile-based admission is enforced

### Wave 4: Configuration and Profile Selection

**Status:** FROZEN
**Introduced:** Configuration-driven registry and profile selection

Guarantees:

* Configuration is policy, not semantics
* Core IR is configuration-free
* Profile admission is lowering-time only
* Registry ordering is deterministic

### Wave 5: Runtime Projection and Execution

**Status:** FROZEN
**Introduced:** Execution of Core IR via runtime interpreter

Guarantees:

* Execution is a projection of Core IR semantics
* Interpreter is deterministic
* Registry dispatch is numeric-only
* No optimization or transformation occurs

---

## Frozen Phases

### Phase 7: Configuration as Policy

**Status:** FROZEN
**Established:** Configuration is non-semantic

Guarantees:

* Configuration may restrict admission
* Configuration may not alter semantics
* Configuration is evaluated pre-lowering
* Core IR contains no configuration

---

## Frozen Specifications

### Core IR 0.2

**Status:** FROZEN

The Core IR 0.2 node set is **closed**:

* CIntLit
* CBoolLit
* CUnitLit
* CLam
* CLet
* CIf
* CVar
* CApp

No additional nodes will be added to Core IR 0.2.

Changes require a new major version (e.g., Core IR 0.3).

### Axis Registry 0.1

**Status:** FROZEN

Registry format is fixed:

* Flat, line-oriented structure
* Required fields: name, arity, deterministic, profile
* No nesting or hierarchy
* Comments are line-only

Changes require a new registry spec version.

---

## Frozen Contracts

### Lowering Contract

**Status:** FROZEN

Lowering invariants:

* Lowering is the **sole semantic authority**
* Lowering must be total, deterministic, explicit
* No implicit semantics
* No deferred resolution

### Core IR Contract

**Status:** FROZEN

Core IR guarantees:

* Semantic completeness (no external context required)
* Configuration-free (no policy embedded)
* Name-free (numeric handles only)
* Immutable (never modified after emission)

### Execution Contract (Wave 5)

**Status:** FROZEN

Execution invariants:

* Execution is a projection, not a source of semantics
* Interpreter is deterministic
* Registry dispatch is numeric-only
* No optimization or transformation

---

## What Users Can Rely On

Users can rely on:

1. **Deterministic compilation**

   * Same input + configuration → same Core IR

2. **Semantic stability**

   * Lowering rules are fixed for frozen waves
   * Core IR meaning does not change

3. **Execution faithfulness**

   * Runtime executes Core IR as-is
   * No hidden transformations

4. **Registry resolution**

   * All calls resolve during lowering
   * Numeric IDs are deterministic within a bundle

5. **Configuration isolation**

   * Configuration affects visibility, not semantics
   * Core IR is policy-free

---

## What Is Still Experimental

The following are **not frozen** and may change:

### Wave 6+

**Status:** NOT FROZEN

Future waves may introduce:

* Loops
* Pattern matching
* Modules
* Type checking
* Error handling constructs
* Async/concurrency primitives

These are **not part of the frozen semantics**.

### Tooling

**Status:** NOT FROZEN

Tooling (LSP, debuggers, formatters) is **out of scope** for current freeze.

### Optimization

**Status:** NOT FROZEN

No optimization is performed in Wave 5.

Future optimization passes would require:

* New waves
* Explicit semantic decisions
* Verification against frozen semantics

### Type System

**Status:** PARTIALLY FROZEN

Core IR 0.2 does **not include type information**.

Type checking may be introduced in:

* A new Core IR version, or
* A separate validation layer

This is **not decided**.

### Multi-File Projects

**Status:** NOT FROZEN

Waves 1–5 operate on **single-file programs** (or programmatically constructed ASTs).

Multi-file project support is **deferred**.

---

## Stability Guarantees

### What "Frozen" Means

A frozen wave guarantees:

* **Semantic stability:** Lowering rules will not change
* **Output stability:** Golden test outputs are canonical
* **Contract stability:** Documented invariants are binding

Frozen does **not** guarantee:

* Performance
* Optimization
* Tooling support
* User ergonomics

### How Frozen Components May Evolve

Frozen components may evolve only through:

1. **Bug fixes** (if semantics were implemented incorrectly)
2. **Clarifications** (if documentation was ambiguous)
3. **New major versions** (e.g., Core IR 0.3, Registry 0.2)

Any change that alters observable behavior requires:

* A new wave or phase
* Explicit justification
* Updated freeze notes

---

## Testing Stability

Frozen waves have **canonical golden outputs**.

These outputs:

* Must not change
* Are regression-tested
* Define correct behavior

Any test change indicates:

* A bug in the implementation, or
* A regression (unintended change)

---

## Version Control

### Core IR Versioning

Core IR uses **semantic versioning**:

* Major version (e.g., 0.2 → 0.3): Breaking changes
* Minor version (e.g., 0.2 → 0.2.1): Additive changes
* Patch version: Bug fixes only

All Core IR bundles are versioned explicitly.

### Registry Versioning

Registry spec uses **version numbers** (e.g., 0.1 → 0.2).

Changes to registry format require a new version.

### Wave Versioning

Waves are **numbered sequentially** (Wave 1, Wave 2, ...).

Waves do not have sub-versions.

---

## Deprecation Policy

Frozen components are **not deprecated**.

If a component becomes obsolete:

* It is **superseded** by a new version
* The old version remains valid
* Migration is explicit

Example:

* Core IR 0.1 is superseded by Core IR 0.2
* Core IR 0.1 remains valid (but Language Lab no longer emits it)
* Consumers may support both versions

---

## Change Control

Any proposal that:

* Alters frozen wave semantics
* Changes Core IR node set
* Modifies lowering rules
* Affects execution behavior

**must**:

1. Open a new wave or phase
2. Provide a written plan
3. Justify the semantic change
4. Produce a new freeze note when complete

No silent changes are permitted.

---

## Current State Summary

As of Wave 5:

| Component                | Status      | Stability       |
|--------------------------|-------------|-----------------|
| Waves 1–5                | Frozen      | Stable          |
| Phase 7                  | Frozen      | Stable          |
| Core IR 0.2              | Frozen      | Stable          |
| Registry 0.1             | Frozen      | Stable          |
| Lowering Contract        | Frozen      | Stable          |
| Execution (Wave 5)       | Frozen      | Stable          |
| Wave 6+                  | Not Frozen  | Experimental    |
| Optimization             | Not Frozen  | Non-existent    |
| Type System (in Core IR) | Not Frozen  | Non-existent    |
| Multi-file projects      | Not Frozen  | Non-existent    |
| Tooling (LSP, etc.)      | Not Frozen  | Out of scope    |

---

## What This Means for Users

Users can:

* **Rely on:** Frozen wave semantics, Core IR structure, execution behavior
* **Trust:** Lowering is deterministic, Core IR is canonical
* **Expect:** No silent breaking changes to frozen components

Users should **not**:

* Assume optimization will occur
* Expect type checking (not yet implemented)
* Rely on undocumented behavior

---

## Future Work

Potential future waves (not frozen, not committed):

* Wave 6: Loops and iteration
* Wave 7: Pattern matching
* Wave 8: Modules and namespaces
* Wave 9: Type checking and inference
* Wave 10: Error handling constructs

These are **not promises**—they are **possible directions**.

Any such work will:

* Have its own plan
* Produce its own freeze note
* Extend (not replace) frozen semantics

---

## How to Know What Is Frozen

Check for:

* A freeze note (e.g., `wave5_freeze_notes.md`)
* Documented invariants (e.g., `lowering_contract.md`)
* Canonical test outputs (e.g., `tests/golden/`)

If a component has a freeze note, **it is frozen**.

If a component has no freeze note, **it is not frozen**.

---

## Key Takeaways

* **Waves 1–5 are frozen** (semantics are stable)
* **Core IR 0.2 is frozen** (node set is closed)
* **Registry 0.1 is frozen** (format is fixed)
* **Execution is frozen** (projection model is stable)
* **Future waves are not frozen** (may introduce new semantics)

---

**End of Frozen Components Documentation**
