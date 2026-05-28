# Core IR 0.3 — Canonical Interchange Format

**Status:** **Canonical for Core IR version 0.3. Implementations MUST conform.**

---

## Overview

Core IR 0.3 is the **canonical, serialized semantic authority** for Axis program semantics and the interchange format consumed by bridges and tooling. It is intentionally **minimal**, **versioned**, and designed for **reliable consumption by downstream consumers**.

Core IR 0.3 extends Core IR 0.1 with **explicit node identity** and **side-band annotations**, without altering the semantic node set.

---

## Authority and Intent

* **Core IR is universal and authoritative** for consumers that accept Core IR bundles.
* **Core IR is inert data**: it carries **no execution semantics**, **no optimization semantics**, and **no operational behaviour**.
* **Core IR is bridge-consumable**: bridges ingest Core IR bundles as **immutable input** and use them for target-specific lowering, code generation, or analysis.

---

## Node Set (0.3)

The Core IR 0.3 node set is **fixed and closed**.

The **only** node kinds in Core IR 0.3 are:

* `CIntLit`
* `CBoolLit`
* `CUnitLit`
* `CLam`
* `CLet`
* `CIf`
* `CVar`
* `CApp`
* `CCall`

All higher-level Axis constructs **MUST** be fully desugared into this node set **before** a Core IR 0.3 bundle is emitted.

No other node kinds exist in Core IR 0.3; consumers **MUST NOT** expect or rely on additional node kinds.

---

## Node Identity (`node_id`)

Core IR 0.3 introduces **optional node identity**.

* `node_id` is a **bundle-local structural identifier**
* `node_id` values are:

  * monotonically increasing integers
  * unique within a single Core IR bundle
  * assigned exclusively by the Core IR producer

### Semantics and Guarantees

* `node_id` carries **no semantic authority**
* `node_id` provides **no guarantees** of stability across:

  * bundles
  * compiler versions
  * surfaces
  * transformations

### Explicit Exclusions

The following are **explicitly excluded by design**:

* external or user-defined `node_id` injection
* reserved ID ranges
* interpretation of numeric values
* encoding of semantics, classification, or policy in `node_id`

Any relaxation of these exclusions requires a **new major Core IR version**.

---

## Annotations

Core IR 0.3 introduces **side-band annotations**.

### Role of Annotations

Annotations are the **only sanctioned mechanism** for attaching metadata, hints, classifications, or tool-specific information to Core IR.

Annotations are **non-authoritative** and **MUST NOT** affect program semantics.

---

### Structural Separation

* Annotations are stored in a **dedicated annotation section**
* Core IR nodes **do not embed annotation payloads**
* Nodes reference annotations only via **annotation identifiers**

---

### Annotation Identity

* Annotation identifiers are:

  * opaque
  * meaningless
  * unique within the bundle
* Annotation identifiers **MUST NOT** encode:

  * namespace
  * semantics
  * classification

Namespaces, if used, are part of the **annotation payload**, not the identifier.

---

### Purity Invariant

A Core IR bundle MUST remain:

* valid
* authoritative
* semantically complete

if **all annotations are removed**.

---

## Bundle and Encoding Rules

* Top-level bundles **MUST** include:

  * a `version` field (string) identifying the Core IR version
  * a `core_term` (or equivalent root reference) identifying the program root

* Common auxiliary fields (for tooling) include:

  * `entrypoint_name`
  * `entrypoint_id`
  * `string_table`
  * `annotations`

Consumers **MUST** treat the core term as authoritative for semantics.

---

## Tagged Representation

* Every node object **MUST** include a tag naming the node kind
* The tag is the **canonical node discriminator** for Core IR 0.3

Field names in the bundle are **stable and authoritative** for the format.

Consumers **MUST** ignore unknown fields encountered in a bundle or node.
Unknown fields carry **no semantic authority**.

---

## Versioning Rules

* This document defines **Core IR version `"0.3"`**
* Each Core IR bundle **MUST** include a top-level `version` field
* Consumers **MUST** reject bundles with an **unknown major version**

Compatibility for minor or patch-level changes is **out of scope** for this document and must be handled by explicit version negotiation in later revisions.

---

## Spans and Diagnostics

* `span` (source-location metadata) is **optional** on nodes
* Spans are **diagnostic only** and **MUST NOT** affect program semantics
* Consumers may use spans to produce diagnostics, but spans carry **no semantic authority** and **MUST NOT** be relied on for correctness

---

## Calls (`CApp` and `CCall`)

Core IR includes two call forms:

* `CApp`: application of a Core IR term to an argument (lambda-calculus application).
* `CCall`: a call to a function referenced by the Axis registry contract.

### Normative: No “foreign” concept

Core IR 0.3 defines **no distinction** between “foreign” and “non-foreign” functions.

* A function is a function is a function.
* If a call target is valid under the registry rules used by the producer, it is a valid call target.
* Core IR does not encode “built-in”, “platform”, “shim”, “FFI”, or “foreign” categories.

Any such categorization is outside Core IR and must not be introduced as Core IR semantics.

---

## `CCall` (Registry-Targeted Call)

`CCall` represents a call to a registry-valid function target.

Core IR does not embed semantic meaning of the target; it carries an opaque identifier and arguments.

### `CCall` fields (conceptual)

* `target`: an opaque target identifier (producer-resolved).
* `args`: argument terms.

### Normative rules

1. Producers MUST ensure `target` refers to a registry-valid function under the producer’s registry resolution rules.
2. Consumers (including bridges) MUST treat `target` as an opaque token and MUST NOT:
   * load a registry
   * resolve names
   * reinterpret meaning
   * apply policy
3. Consumers MUST emit a direct call to the symbol corresponding to `target` under the consumer’s ABI rules.
4. If the symbol is not provided by linked artifacts, the build MUST fail loudly (e.g., link-time failure). No stubs, fallbacks, or silent behaviour.

---

## Bridge and Linking Model (Normative)

A bridge is a projection of Core IR into native artifacts.

### Library-first emission

Bridges SHOULD support compiling a Core IR bundle into a native library artifact (static or shared).

* The emitted library exports the symbols corresponding to functions defined by that bundle (per the bridge ABI).
* The emitted library may reference symbols not defined within the bundle.

### Linking is composition

Bridges MUST support composing multiple artifacts by linking:

* a library produced from Core IR bundle A
* a library produced from Core IR bundle B
* a shim library (platform services)
* any other libraries required

From Core IR’s perspective, all such providers are equivalent: they are simply sources of symbols.

### ABI token rule

The bridge ABI MUST treat call targets as stable link symbols.
The bridge MUST NOT encode extra meaning in the ABI beyond symbol identity and call shape.

---

## Bridge Constraints and Consumer Responsibilities

* Bridges and other consumers **MUST NOT** validate, execute, or reinterpret Core IR semantics beyond what the node kinds express
* Bridges **MUST** treat Core IR bundles as **immutable input**
* Consumers **MUST**:

  * enforce the **closed node set**
  * enforce the **versioning rules**
  * ignore annotations unless explicitly designed to consume them

---

## Section B — Explicit Non-Goals for Core IR 0.3

* **Not an execution format**
  Core IR 0.3 is not intended to be executed or to define runtime semantics.

* **Not an optimizer IR**
  Core IR 0.3 does not specify optimization passes or transformation semantics.

* **Not extensible within 0.3**
  Adding node kinds requires a **new major version**.

* **Not a validation contract**
  Core IR 0.3 is not a schema-driven validator for higher-level well-formedness beyond the node, encoding, and versioning rules above.

---

## Section C — Notes on Future Evolution (Non-Binding)

* Future versions may extend the node set or metadata model; such changes will require a **major-version increment** and explicit compatibility guidance.
* A formal, machine-readable schema may be provided for later versions to aid robust parsing and validation.
* Version negotiation and compatibility policies may be specified in later revisions if automated compatibility is required.

---

## Section D — Theoretical Foundations (Non-Normative)

Axis Core IR is based on the lambda-calculus tradition used in both programming language theory and real-world compilers. The core consists of lambda abstraction, application, variable binding (`let`), and conditional branching (`if`), which together form a **minimal, computationally complete semantic foundation**.

This structure is consistent with **A-normal form** and the internal “core languages” used in ML- and Haskell-family compilers, where all higher-level constructs are desugared prior to core IR emission.

---

## References and Rationale

The design of Axis Core IR follows well-established foundations in programming language theory and compiler practice. The intent is not to introduce a novel calculus, but to **freeze a minimal, stable semantic core** that has repeatedly proven sufficient for expressing general computation while remaining tractable for analysis, transformation, and compilation.

### Lambda Calculus Foundations

The core of Axis Core IR is grounded in the lambda calculus, which demonstrates that computation can be expressed using only variable reference, lambda abstraction, and application. Lambda calculus is computationally complete and forms the theoretical basis for most functional languages and many compiler intermediate representations.

> Church, A. *The Calculi of Lambda-Conversion*. Princeton University Press, 1941.

This foundation justifies the use of **application (`CApp`) as the sole computational primitive** in Core IR.

---

### Practical Core Languages and Desugaring

Modern language design commonly distinguishes between a rich surface language and a smaller core calculus into which surface constructs are desugared. This approach is described extensively in:

> Pierce, B. *Types and Programming Languages*. MIT Press.

Pierce presents languages in terms of:

* a surface syntax for programmer convenience, and
* a small core language consisting of lambda abstraction, application, let-bindings, and conditionals.

This directly motivates the inclusion of:

* `CLet` for explicit naming and sharing
* `CIf` as the sole primitive control construct

---

### A-Normal Form and Compiler Practice

Axis Core IR closely resembles **A-Normal Form (ANF)**, a widely used intermediate representation in compilers, where:

* all intermediate computations are explicitly named via `let`
* all control flow is made explicit
* all computation occurs through function application

This formulation is introduced and justified in:

> Flanagan et al. *The Essence of Compiling with Continuations*, 1993.

ANF is favored because it simplifies:

* program analysis
* transformation and optimization
* code generation

Axis Core IR adopts this structure as a **semantic normal form**, rather than merely an internal compiler convenience.

---

### Core Languages in Real Compilers

Many production compilers lower programs into a small internal “core language” with similar structure. A well-known example is the Core language used by the Glasgow Haskell Compiler (GHC), described in:

> Peyton Jones, S. *Implementing Functional Languages*.

These core languages typically consist of:

* lambda abstraction
* application
* let-bindings
* simple branching constructs

Axis Core IR aligns with this tradition while deliberately freezing the core as a **stable, serialized semantic interchange format**.

---

### Design Implication

Taken together, these works support the following design position:

* Lambda abstraction, application, let-binding, and conditional branching form a **minimal, computationally complete semantic core**
* All higher-level constructs (loops, pattern matching, operators, methods, effects) are **derivable** and are therefore excluded from the core
* A small, explicit core improves semantic clarity, analyzability, and long-term stability

Axis Core IR formalizes this approach by treating the core calculus as the **canonical semantic authority**, with execution, optimization, and tooling defined as projections over it.

Perfect — that’s the right way to do it 👍
Below is a **clean “change block”** you can append to the spec (or put at the top) to introduce **Core IR 0.3**, without rewriting the whole document.

This is written in **normative spec language**, matches the tone of the existing file, and makes the shift *explicit and intentional*.

---

## 🔁 Core IR 0.3 — Change Block (Normative)

### Status

Core IR **0.3** is a **backward-incompatible evolution** of Core IR 0.2.

Core IR 0.3 preserves the **semantic core and node set**, while updating the **call and linking model** to reflect Axis’s library-first, uniform-function semantics.

Consumers MUST treat Core IR 0.3 as a **distinct major version**.

---

## Summary of Changes from 0.2 → 0.3

Core IR 0.3 introduces the following changes:

1. **Removes the “foreign function” concept entirely**
2. **Defines all functions as registry-valid and equivalent**
3. **Reframes `CCall` as a uniform function call**
4. **Adopts a library-first bridge and linking model**
5. **Clarifies that Core IR has no execution- or origin-level distinctions**

No new node kinds are introduced.

---

## Normative Change: Function Uniformity

In Core IR 0.3:

> **There is no semantic or structural distinction between “foreign”, “built-in”, “platform”, “shim”, or “user-defined” functions.**

All functions are treated uniformly.

If a function identifier is valid under the registry rules used by the Core IR producer, it is a valid call target.

Core IR MUST NOT encode:

* function origin
* implementation location
* execution substrate
* platform classification

---

## Normative Change: `CCall` Semantics (0.3)

In Core IR 0.3, `CCall` represents:

> **A call to a registry-valid function identifier.**

### Revised interpretation

* `CCall` does **not** mean “foreign call”
* `CCall` does **not** imply special handling
* `CCall` does **not** imply runtime or platform semantics

It is simply a function call whose target is not expressed as a Core IR term.

### Updated guarantees

Producers MUST ensure:

* the `target` identifier is registry-valid

Consumers (including bridges) MUST:

* treat the target as an opaque symbol identity
* emit a direct call under the consumer’s ABI
* perform no registry lookup
* apply no semantic interpretation

---

## Normative Change: Library-First Bridge Model

Core IR 0.3 **normatively defines** the bridge output model.

### Required bridge behaviour

A Core IR 0.3 bridge:

1. MUST be capable of compiling a Core IR bundle into a **native library artifact**
2. MUST treat that library as a **first-class linkable unit**
3. MUST support linking multiple such libraries together

This includes (non-exhaustively):

* libraries produced from other Core IR bundles
* platform shim libraries
* standard libraries
* user-provided libraries

From the Core IR perspective, **all providers are equivalent**.

---

## Normative Change: Linking as Composition

Linking in Core IR 0.3 is defined as **pure composition**.

* A build succeeds if all required symbols resolve
* A build fails if any required symbol is missing
* There are no fallback rules
* There are no stub semantics

Core IR itself defines **no execution order, loading model, or runtime policy**.

---

## Explicit Non-Changes

The following aspects are **unchanged** from Core IR 0.2:

* Node set and semantic calculus
* Lambda-calculus foundations
* A-normal-form structure
* Node identity (`node_id`)
* Annotation purity and non-authority
* Versioning discipline

Core IR 0.3 is a **semantic clarification and projection rule change**, not a redesign of the core language.

---

## Rationale (Non-Normative)

Core IR 0.3 reflects Axis’s evolution from a compiler-internal representation toward a **semantic interchange and composition substrate**.

By removing origin distinctions and treating all functions uniformly, Core IR enables:

* module-level compilation
* library caching and reuse
* deterministic composition
* simple and predictable bridge implementations

This aligns Core IR with long-standing object- and library-based build models, while preserving a minimal, formally grounded semantic core.

---

## Migration Note (Informative)

* Core IR 0.2 producers that relied on “foreign function” interpretation MUST update emitters to conform to the uniform function model.
* Bridges MUST update `CCall` handling to remove any special-casing of foreign or platform calls.
* Core IR 0.2 bundles MUST NOT be interpreted as Core IR 0.3 without explicit version negotiation.