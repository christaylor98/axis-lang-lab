# Lowering Contract — Language Lab

Status: Normative
Applies to: Axis Language Lab
Core IR Version: 0.2

---

## Purpose

This document defines the **lowering contract** for Axis Language Lab.

Lowering is the **only stage** at which semantic meaning is assigned to surface syntax.

This contract specifies:

* what lowering is
* what lowering MUST do
* what lowering MUST NOT do
* what constitutes a lowering failure

This document is **authoritative** for all lowering logic.

---

## Authority Boundary

Lowering is the **semantic boundary** between:

* surface structure
* canonical Core IR

Before lowering:

* constructs have **structure only**
* no semantic meaning is assigned

After lowering:

* semantics are **fully explicit**
* meaning is frozen in Core IR

Lowering MUST be:

* explicit
* total
* deterministic

---

## Inputs to Lowering

Lowering operates on:

* a validated AST produced by the parser
* an active lowering profile (if applicable)
* the fixed Core IR node set

Lowering MUST NOT depend on:

* runtime behavior
* execution results
* optimization assumptions
* consumer behavior

---

## Outputs of Lowering

Lowering MUST produce:

* exactly one Core IR core_term
* a Core IR bundle conforming to core_ir_contract.md

Lowering MUST NOT produce:

* partial Core IR
* deferred semantics
* placeholders
* unresolved references

---

## AST Node Set (Phase 1)

For Phase 1, the AST node set admitted to lowering is **explicitly limited**.

Admitted AST nodes:

* FunctionDecl
* EmptyBlock

No other AST node kinds are admitted.

Any AST node not listed above MUST cause lowering to fail.

---

## Totality Requirement

Lowering MUST be **total** over the admitted AST node set.

For every admitted AST node:

* a lowering rule MUST exist
* the rule MUST either:

  * emit valid Core IR, or
  * fail explicitly

Implicit fallthrough is forbidden.

---

## Determinism Requirement

Lowering MUST be deterministic.

Given identical AST input:

* emitted Core IR MUST be structurally identical
* ordering MUST be canonical
* node identity assignment MUST follow a deterministic scheme

Non-deterministic behavior is forbidden.

---

## Lowering Rules (Phase 1)

### Rule L1 — Function Declaration

Input:

* FunctionDecl
* zero parameters
* empty body

Lowering:

* MUST emit a CLam node
* the lambda parameter MUST be a dummy binding
* the lambda body MUST be CUnitLit

No other interpretation is permitted.

---

### Rule L2 — Empty Block

Input:

* EmptyBlock

Lowering:

* MUST emit CUnitLit

---

## Prohibited Lowering Behavior

Lowering MUST NOT:

* invent semantics
* infer intent
* insert implicit effects
* introduce control flow not present in Core IR
* introduce new node kinds
* rely on annotations for semantics
* defer meaning to consumers

Any such behavior is a contract violation.

---

## Failure Semantics

Lowering MUST fail if:

* an AST node is not admitted
* no lowering rule exists for a node
* a rule cannot be applied totally
* Core IR invariants would be violated

Lowering failures are **Language Lab errors**.

They MUST NOT be recovered from.

---

## Relationship to Core IR Contract

Lowering MUST satisfy all producer obligations defined in:

```
docs/core_ir_contract.md
```

In case of conflict:

* core_ir_contract.md takes precedence

---

## Observability Requirements

Lowering MUST be observable.

If lowering executes loops or recursion:

* progress counters MUST be emitted
* counters MUST follow the global counter pattern
* silence during long-running lowering is forbidden

---

## Explicit Non-Goals

Lowering is not responsible for:

* optimization
* execution semantics
* registry resolution
* type checking beyond structural requirements
* profile negotiation beyond admission checks

---

## Enforcement

This contract is enforced by:

* compile-time checks
* deterministic tests
* golden Core IR comparisons

Violations MUST result in immediate failure.

---

## Summary

Lowering is the **only meaning-assignment stage**.

It MUST be:

* explicit
* total
* deterministic
* closed over the admitted AST set
* compliant with Core IR 0.2

No meaning exists before lowering.
No meaning changes after lowering.

---

End of document.