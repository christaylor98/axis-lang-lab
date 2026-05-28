Good call. This wants to be **permanent, boring, and authoritative** — not a discussion doc, not a roadmap, not a README.

Below is a **ready-to-drop document** you can put straight into the repo.
It is written to *freeze intent*, not invite debate.

I’ll assume something like `docs/core-ir-design-decisions.md`, but the filename is up to you.

---

# Core IR — Design Decisions and Constraints

**Status:** Permanent
**Audience:** Core IR producers, bridge authors, tooling authors
**Purpose:** To document non-negotiable design decisions that define the long-term stability and authority of Core IR.

This document records **intentional exclusions and constraints**.
These are not accidental limitations — they are **load-bearing architectural choices**.

---

## 1. Core IR Authority Model

Core IR is the **canonical semantic authority** for Axis programs once emitted.

* Core IR is **inert data**
* Core IR defines **no execution semantics**
* Core IR defines **no optimization semantics**
* Core IR defines **no runtime behavior**

Execution, optimization, analysis, and policy enforcement are **projections** performed by bridges and tools.

---

## 2. Closed Node Set

Core IR 0.1 defines a **closed and fixed node set**.

* No extension mechanisms exist within a major version
* Adding node kinds requires a **new major Core IR version**
* Higher-level constructs MUST be fully desugared prior to emission

This constraint exists to:

* preserve semantic stability
* simplify bridge implementations
* prevent semantic drift

---

## 3. Node Identity (`node_id`)

### 3.1 Purpose

`node_id` exists solely as a **bundle-local structural address** to support:

* annotation attachment
* tooling correlation
* diagnostics and analysis

It is **not** a semantic identifier.

---

### 3.2 Assignment Rules

* `node_id` values are assigned **exclusively by the Core IR producer**
* `node_id` values are **monotonically increasing integers**
* `node_id` values MUST be **unique within a single Core IR bundle**
* `node_id` values MAY be omitted entirely

---

### 3.3 Explicit Non-Guarantees

`node_id` provides **no guarantees** of:

* stability across bundles
* stability across compiler versions
* stability across surfaces
* stability across transformations
* semantic meaning
* ordering semantics

Recompilation, refactoring, or implementation changes are expected to change `node_id` values.

---

### 3.4 Explicit Exclusions

The following are **explicitly excluded by design**:

* external injection of `node_id` values
* user-defined or surface-defined `node_id` values
* reserved ID ranges
* interpretation of numeric ranges
* encoding of semantics, classification, or policy in `node_id`

Any relaxation of these exclusions requires a **new major Core IR version**.

---

## 4. Annotations

### 4.1 Role of Annotations

Annotations are the **only sanctioned mechanism** for attaching metadata, hints, classifications, or tool-specific information to Core IR.

Annotations are **side-band data** and are **non-authoritative**.

---

### 4.2 Structural Separation

* Annotations are stored in a **dedicated annotation section**
* Core IR nodes do **not** embed annotation payloads
* Nodes reference annotations only via **lightweight annotation IDs**

This separation ensures:

* minimal node footprint
* semantic purity
* tooling flexibility
* independent evolution of metadata

---

### 4.3 Annotation Identity

* Annotation IDs are **opaque and meaningless**
* Annotation IDs are **unique within the bundle**
* Annotation IDs carry **no namespace or semantic encoding**

---

### 4.4 Annotation Payload

All metadata, classification, and meaning MUST live **inside the annotation payload**, including but not limited to:

* namespaces
* tooling hints
* surface information
* policy metadata
* analysis results
* debug information

Namespaces, if used, are part of the **annotation payload**, not the annotation ID.

---

### 4.5 Attachment Rules

* Annotations attach to nodes **only via explicit references**
* No implicit attachment mechanisms exist
* No positional or structural inference is permitted

---

### 4.6 Purity Invariant

A Core IR bundle MUST remain:

* valid
* authoritative
* semantically complete

if **all annotations are removed**.

Annotations MUST NOT be required for correct semantic interpretation of Core IR.

---

## 5. Consumer Responsibilities

Consumers (bridges, tools, analyzers):

* MUST treat Core IR as immutable input
* MUST ignore unknown fields
* MUST NOT reinterpret node identity
* MUST NOT rely on annotation presence for correctness

Consumers MAY:

* read annotations
* interpret annotation payloads within their own domain
* discard annotations entirely

---

## 6. Design Rationale (Non-Normative)

These constraints exist to prevent:

* semantic leakage via metadata
* accidental contracts between tools
* tribal knowledge encoded in conventions
* silent erosion of Core IR authority

All meaning must be:

* explicit
* inspectable
* detachable
* owned by the layer that introduces it

---

## 7. Change Policy

Any change to the rules in this document requires:

* a new **major Core IR version**
* explicit documentation of compatibility impact

No exceptions.

---


### End of Document