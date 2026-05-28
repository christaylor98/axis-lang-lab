# Core IR Contract — Language Lab

Status: Normative
Applies to: Axis Language Lab
Core IR Version: 0.2

---

## Purpose

This document defines the **binding contract** between **Axis Language Lab** and **Axis Core IR**.

It specifies:

* what Core IR **is**
* what Core IR **is not**
* what a **producer** (Language Lab) MUST guarantee
* what consumers MAY assume
* where the **semantic responsibility boundary** lies

This document is **authoritative** for all Language Lab work.

---

## Authority and Scope

Core IR is the **canonical serialized semantic authority** for Axis programs.

For Language Lab:

* Core IR is the **semantic endpoint**
* All meaning MUST be made explicit **before emission**
* After emission, semantics are frozen

Any failure to meet this contract is a **Language Lab defect**, not a consumer defect.

---

## Core IR Version

Language Lab targets **Core IR version 0.2**.

All emitted bundles MUST:

* declare version 0.2
* conform exactly to the Core IR 0.2 node set and invariants

Language Lab MUST NOT emit Core IR 0.1 bundles.

---

## Node Set (Closed)

The Core IR 0.2 node set is **fixed and closed**.

The only permitted node kinds are:

* CIntLit
* CBoolLit
* CUnitLit
* CLam
* CLet
* CIf
* CVar
* CApp

No other node kinds are permitted.

Language Lab MUST fully desugar all surface constructs into this node set **before emission**.

---

## Bundle Structure

Every emitted Core IR bundle MUST include:

* version
* core_term

The core_term is the **sole semantic root**.

Optional fields such as annotations, spans, string tables, or entrypoint metadata carry **no semantic authority**.

---

## Producer Obligations (Normative)

Language Lab, as a Core IR producer, MUST satisfy all of the following:

1. **Closed Node Set**

   * Only node kinds listed in this contract may be emitted

2. **Total Lowering**

   * All admitted surface constructs MUST lower completely
   * Partial or deferred lowering is forbidden

3. **Semantic Completeness**

   * The emitted core_term MUST fully determine program meaning
   * Removing all annotations MUST NOT change semantics

4. **Determinism**

   * Identical surface input MUST produce semantically identical Core IR
   * Ordering and structure MUST be canonical

5. **No Semantic Encoding in Metadata**

   * node_id values MUST NOT encode semantics
   * annotations MUST NOT encode semantics
   * spans are diagnostic only

6. **Single Root**

   * Exactly one core_term MUST be emitted
   * Multiple roots or implicit entrypoints are forbidden

Failure to meet any obligation is a **contract violation**.

---

## Node Identity (node_id)

node_id is optional and non-authoritative.

If present:

* node_id values MUST be unique within the bundle
* node_id values MUST be monotonically increasing integers
* node_id carries no semantic meaning

node_id MUST NOT be used for:

* classification
* policy encoding
* semantic interpretation

---

## Annotations

Annotations are **explicitly non-authoritative**.

Rules:

* Core IR semantics MUST be complete without annotations
* Annotations MUST be removable without semantic loss
* Nodes MUST NOT embed annotation payloads

Annotations exist solely for tooling, diagnostics, or analysis.

---

## Semantic Boundary (Critical)

Core IR is the **semantic boundary**.

Language Lab responsibilities end at Core IR emission.

After emission:

* no semantic repair is permitted
* no semantic inference is permitted
* no semantic enrichment is permitted

Any failure discovered after emission indicates:

* a Language Lab lowering defect, or
* a consumer defect

There is no shared responsibility.

---

## Explicit Non-Goals

Core IR is not:

* an execution format
* an optimization IR
* a validation framework
* an extensible schema within version 0.2

Language Lab MUST NOT rely on consumers to fix, infer, or reinterpret semantics.

---

## Enforcement

This contract is enforced by:

* Language Lab validation
* deterministic tests
* canonical output comparison

Violations MUST result in immediate failure.

---

## Summary

Language Lab MUST emit:

* complete
* deterministic
* semantically closed
* annotation-independent
* versioned Core IR 0.2 bundles

Core IR is the **semantic authority**.
Language Lab is the **semantic assigner**.
Consumers are **semantic readers only**.

---

End of document.