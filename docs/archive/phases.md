Great moment to do this. What you want now is a **north-star execution document** that:

* makes Phase 1 feel *finished* (it is)
* defines Phase 2 narrowly enough to stay safe
* sketches later phases without over-committing
* gives you a place to evolve intent *without rewriting history*

Below is a **drop-in planning document** you can put under `docs/` (e.g. `docs/roadmap.md` or `docs/phases.md`).
It is written to match the discipline you’ve already shown: scope-locked, semantics-first, updateable.

---

# Axis Language Lab — Phase Roadmap

## Purpose of This Document

This document defines the **planned phases** of Axis Language Lab from Phase 2 onward.

It exists to:

* make scope explicit
* prevent accidental semantic creep
* clarify what is *intentionally out of scope* at each stage
* act as a living reference that can be updated as understanding improves

This document is **descriptive**, not prescriptive.
Completion of a phase is determined by **acceptance criteria**, not by feature count.

---

## Completed: Phase 1 — Minimal Semantic Spine

### Status

**Complete and frozen**

### What Phase 1 Established

* A full end-to-end pipeline:

  ```
  Text → Tokens → AST → Lowering → Core IR → Validation
  ```
* Exactly one semantic rule:

  ```
  fn f() {}  ⇒  CLam(_, CUnitLit)
  ```
* A canonical Core IR output
* A golden test locking semantics
* A validation pass enforcing invariants

### Explicit Non-Goals (and Why)

* No config
* No extensibility
* No runtime
* No registry
* No effects
* No general language

Phase 1 answers only one question:

> Can Language Lab assign meaning once, explicitly, and defend it?

That question is now answered **yes**.

---

## Phase 2 — Structural Expansion (Still Single Semantics)

### Intent

Phase 2 expands the **surface and AST shape** while preserving:

* a single semantic interpretation
* a single lowering rule per construct
* golden tests for every new semantic case

This phase is about **structure**, not expressiveness.

---

### Phase 2 Scope

#### New Surface Constructs (Tentative)

* Function body with a single explicit expression
* Explicit `return` of unit
* Optional empty parameter list vs explicit `_`

Examples:

```
fn f() { return (); }
fn f() { (); }
```

#### AST Changes

* Introduce minimal expression node(s)
* Preserve “structure only” rule
* No type inference
* No control flow

#### Lowering Changes

* New lowering rules for new AST shapes
* Each rule:

  * explicit
  * total
  * deterministic

#### Validation Changes

* Extend invariants to cover new Core IR shapes
* No weakening of Phase 1 invariants

#### Testing

* One golden test per new semantic rule
* Phase 1 golden remains untouched

---

### Explicit Non-Goals for Phase 2

* No variables
* No bindings
* No control flow
* No effects
* No registry
* No configuration

---

## Phase 3 — Binding and Identity

### Intent

Introduce **names, bindings, and scope** as first-class semantic concepts.

This is the phase where programs begin to *refer* to things.

---

### Phase 3 Scope

#### New Concepts

* Local bindings (e.g. `let`)
* Variable references
* Lexical scope

#### AST

* Binding nodes
* Variable reference nodes

#### Lowering

* Explicit binding semantics
* No implicit capture
* No mutation

#### Validation

* No free variables
* Scope correctness
* Deterministic binding resolution

#### Testing

* Golden tests for:

  * valid bindings
  * invalid scope
  * shadowing (if allowed)

---

### Explicit Non-Goals for Phase 3

* No mutation
* No effects
* No control flow
* No types beyond identity

---

## Phase 4 — Control Flow (Pure)

### Intent

Introduce **control flow** without effects or mutation.

This phase answers:

> Can we represent branching and sequencing semantically?

---

### Phase 4 Scope

* Conditional expressions (`if`)
* Sequencing
* Expression-oriented flow

Lowering must remain:

* explicit
* side-effect free
* deterministic

---

### Explicit Non-Goals for Phase 4

* No loops (unless purely structural)
* No mutation
* No effects
* No performance concerns

---

## Phase 5 — Effects (Constrained)

### Intent

Introduce **effects as explicit, constrained semantics**.

This is a major semantic boundary.

---

### Phase 5 Scope

* Explicit effect nodes
* Effect validation
* Effect visibility in Core IR

No implicit effects are allowed.

---

## Phase 6 — Registry and External Semantics

### Intent

Introduce **registry-bound semantics**:

* external functions
* foreign calls
* host interactions

This is where Language Lab begins to interact with the outside world.

---

## Phase 7 — Configuration and Policy

### Intent

Introduce **configuration as policy**, not semantics.

Config at this stage may control:

* enabled phases
* validation strictness
* tooling behavior

Config **must not** change Core IR meaning.

---

## Phase 8 — Projections and Tooling

### Intent

Build non-semantic projections:

* IR inspection
* pretty-printing
* analysis tools
* bridges

This phase does not affect language semantics.

---

## Completion Criteria (Language Lab “Done”)

Language Lab is considered **functionally complete** when:

* All semantic constructs are:

  * explicit
  * lowered
  * validated
  * covered by golden tests
* All variability is either:

  * structural (new AST)
  * semantic (new lowering rule)
  * or policy-level (config, tooling)
* No semantics are implicit or emergent

---

## Change Policy for This Document

* This document **will evolve**
* Past phases must not be retroactively redefined
* Changes should:

  * add clarity
  * narrow scope
  * reduce ambiguity

---

## Final Note

This roadmap intentionally prioritizes:

* semantic clarity over expressiveness
* explicitness over convenience
* invariants over flexibility

That is the core design philosophy of Axis Language Lab.
