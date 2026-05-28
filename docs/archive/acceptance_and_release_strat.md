# Axis v0.x — Release Acceptance Gates (Authoritative)

These gates sit **above** Waves 1–7.
They are about *readiness*, not architecture purity.

Waves explain *how the system is built*.
Gates explain *when it is ready to ship*.

---

## Gate 1 — PoC Parity (Semantic Equivalence)

### Goal

Prove Axis Language Lab is a **real successor**, not a re-implementation experiment.

### Acceptance Criteria

* Use the **same surface syntax** as the original PoC
* For all PoC examples:

  * Axis Lang Lab produces **semantically equivalent Core IR**
  * Structural differences are allowed
  * Observable behavior via registry calls must match

### Explicit Non-Goals

* No new syntax
* No performance parity requirement
* No execution improvements beyond Wave 5

### Why This Gate Exists

This is your **credibility gate**:

> “We did not lose meaning while rebuilding the system correctly.”

---

## Gate 2 — Multi-Surface Existence (Axis Identity)

### Goal

Demonstrate Axis is **not a single-language compiler**, but a *semantic system*.

### Required Surfaces

All three must exist and be tested:

1. **Axis Human Surface**

   * Readable, writable by humans
   * Stable enough for examples and docs

2. **Axis AI Semantic Surface**

   * AI-oriented representation
   * Explicit structure, minimal ambiguity
   * Designed for generation and transformation

3. **AI Direct IR Surface**

   * Core IR ingestion path
   * No parsing, no syntax
   * Pure semantic input

### Acceptance Criteria

* All three surfaces:

  * Lower into **the same Core IR**
  * Pass validation
  * Can be executed (Wave 5)

### Why This Gate Exists

This is the **Axis differentiation gate**.
It proves the “multiple surfaces, one meaning” claim.

---

## Gate 3 — Rust Surface Proximity (Ecosystem Bridge)

### Goal

Get *as close as practical* to a Rust surface **without lying**.

This is not “compile Rust”.
It is “can Rust developers see themselves here”.

### Acceptance Criteria

* A Rust-adjacent surface or mapping that:

  * Feels structurally familiar
  * Maps cleanly into Core IR
  * Does not introduce Rust-specific semantics into Core IR

### Explicit Non-Goals

* Full Rust compatibility
* Ownership/borrowing parity
* Procedural macros, lifetimes, etc.

### Why This Gate Exists

This is the **adoption gate**.
It reduces the psychological distance between Axis and real-world users.

---

## Gate 4 — Independent User Viability (Escape Velocity)

### Goal

A competent engineer can **work without you**.

### Acceptance Criteria

* Documentation explains:

  * What Axis is (and is not)
  * Core IR authority model
  * Registry model
  * Execution model
  * How to add a surface
  * How to add a registry
* Clear examples
* Clear repo structure
* Clear “start here” path

### This Is the Real Release Gate

Until this is true:

> You are demoing, not releasing.

---

## Multi-Release Strategy (Strongly Recommended)

Yes — **you should absolutely stage this**.

Here is the cleanest cut.

---

### Release A — **Axis v0.1: Semantic Core**

Includes:

* Gate 1 (PoC parity)
* Wave 1–5 frozen
* Single surface (PoC surface)
* Core IR + execution + registry
* Minimal docs

Audience:

* You
* Compiler people
* Early reviewers

Message:

> “This is the semantic core. It runs. It is disciplined.”

---

### Release B — **Axis v0.2: Multi-Surface System**

Includes:

* Gate 2
* All three surfaces
* Clear surface → Core IR story
* Stronger docs

Audience:

* Researchers
* Tool builders
* AI/system people

Message:

> “Axis is a semantic hub, not a language.”

---

### Release C — **Axis v0.3: Ecosystem Bridge**

Includes:

* Gate 3
* Rust-adjacent surface
* Better examples
* Clear adoption story

Audience:

* Practitioners
* Rust-curious engineers
* Systems folks

Message:

> “You can approach Axis from where you already are.”

---

### Release D — **Axis v0.4: Independent Use**

Includes:

* Gate 4
* Full docs
* Tutorials
* Contributor guidance

Audience:

* Everyone else

Message:

> “You don’t need the author in the room.”

---

## The Important Reframe

You didn’t lose acceptance criteria.

You **outgrew a single definition of “done”**.

Now you have:

* Waves → internal correctness
* Gates → external readiness
* Releases → narrative control

That’s exactly what a serious system needs.

