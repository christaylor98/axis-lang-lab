Got it. Below is the **formal, v0.1 Bridge Interface Specification**.
This is the **mechanical contract** between Axis Core IR and any bridge.

No narrative. No sales. No future promises.

---

# Axis Bridge Interface — Specification v0.1

---

## 1. Purpose

A **Bridge** is a runtime adapter that consumes **Axis Core IR** and executes it in a concrete environment.

Bridges are:

* replaceable
* externally selected
* execution-focused

Bridges are **not**:

* semantic authorities
* language surfaces
* optimisers of meaning

---

## 2. Inputs (Normative)

A bridge MUST accept the following inputs.

### 2.1 Core IR

The canonical, invariant Core IR produced by lowering.

Properties:

* semantics complete
* syntax-free
* deterministic
* surface-agnostic

The bridge MUST treat Core IR as read-only.

---

### 2.2 Effect Graph

An explicit representation of:

* effect-producing nodes
* effect-consuming nodes
* sequencing constraints
* dependency edges

The bridge MUST execute effects according to this graph.

---

### 2.3 Metadata (Opaque)

Optional metadata attached to Core IR nodes.

Examples:

* `@intent.parallel`
* `@intent.serial`

Rules:

* `@intent.parallel` has **no semantic force** and may be ignored
* `@intent.serial` is mandatory and MUST be enforced
* Metadata may influence execution strategy

Lowering MUST ignore metadata entirely.

---

## 3. Outputs (Normative)

A bridge MUST produce:

* Program execution
* Observable effects (IO, messaging, etc.)
* Exit status or failure signal

A bridge MUST NOT:

* emit modified Core IR
* rewrite semantics
* infer missing intent

---

## 4. Mandatory Bridge Obligations

These obligations apply to **all** bridges.

---

### B1. Semantic Preservation

The bridge MUST execute Core IR such that:

* all semantic effects occur
* effect ordering implied by the graph is preserved
* serial sections are not reordered

If this cannot be satisfied, the bridge MUST fail fast.

---

### B2. Correct Fallback

If an optimisation is unsupported:

* the bridge MUST degrade to a correct execution
* the bridge MUST NOT reject the program due to missing optimisation capability

Example:

* Parallel intent + no parallel capability ⇒ sequential execution

---

### B3. Effect Execution

The bridge MUST:

* interpret effect nodes
* execute them exactly once unless semantics specify otherwise
* respect explicit sequencing

The compiler and lowering stage MUST NOT execute effects.

---

### B4. Isolation of Concerns

The bridge MUST NOT:

* change program meaning
* introduce new semantics
* infer execution intent not present in metadata
* depend on surface-specific structure

---

## 5. Optional Bridge Capabilities

Bridges MAY support additional capabilities.

Capabilities are **declared**, never assumed.

---

### 5.1 Capability Declaration

Each bridge MUST publish a capability profile.

Example:

```toml
[bridge]
name = "asg-horizontal"

[capabilities]
parallel_loops = true
distributed_execution = true
max_nodes = 60
channels = true
controllers = true
```

Capabilities affect optimisation only, never correctness.

---

### 5.2 Capability Rules

* Absence of a capability MUST NOT cause failure
* Presence of a capability permits optimisation
* Capabilities are advisory, not contractual

---

## 6. Parallelism Semantics

### 6.1 Parallel Intent

If Core IR metadata contains:

```
@intent.parallel = true
```

The bridge MAY:

* execute iterations concurrently
* distribute work
* vectorise computation

The bridge MAY ALSO:

* execute sequentially

Both are correct.

---

### 6.2 Serial Intent

If Core IR metadata contains:

```
@intent.serial = true
```

The bridge MUST:

* preserve execution order
* prohibit reordering
* prohibit concurrent execution within scope

---

## 7. Concurrency and Distribution

### 7.1 Spawn and Join

For spawn semantics:

* the bridge MUST create an independent execution context
* the returned handle MUST represent completion
* `join` MUST block or wait until completion

Implementation strategy is unspecified.

---

### 7.2 Channels

For channel semantics:

* the bridge MUST preserve send/receive ordering per channel
* blocking, buffering, and transport are implementation-defined
* delivery MUST be reliable unless explicitly documented otherwise

---

### 7.3 Distributed Execution

If supported:

* the bridge MAY distribute work across hosts
* failures MAY be retried
* partial execution MUST NOT violate semantics

If not supported:

* execution MUST degrade to local execution

---

## 8. Controllers and Event Handling

For controller semantics, the bridge MUST:

* bind declared event sources
* invoke handlers on event arrival
* execute returned effects
* manage lifecycle

The bridge MAY:

* batch events
* parallelise handlers (if allowed)
* apply backpressure

The bridge MUST NOT:

* reorder serial handlers
* drop events unless explicitly documented

---

## 9. Execution Boundary

The bridge MUST respect a single execution boundary.

Before boundary:

* program describes semantics only

After boundary:

* effects are executed

No implicit execution is permitted.

---

## 10. Failure Modes

### 10.1 Semantic Failure

If Core IR is invalid:

* the bridge MUST fail immediately

---

### 10.2 Runtime Failure

If an effect fails:

* behavior is bridge-defined
* retries, backoff, or abort may occur
* semantics MUST NOT be violated

---

## 11. Bridge Selection

Bridge selection is external to the language.

Examples:

```bash
axis run --bridge conservative
axis run --bridge aggressive
axis run --bridge asg-horizontal
```

Code and Core IR remain unchanged.

---

## 12. Compliance Summary

A compliant bridge:

* preserves semantics
* degrades safely
* executes explicit effects
* respects intent metadata
* isolates execution concerns

> **The bridge decides *how*.
> Core IR defines *what*.**

---

If you want next, the logical follow-ups are:

* **Reference Bridge (Conservative) spec**
* **Parallel Bridge spec**
* **ASG / Horizontal Bridge spec**
* **End-to-end demo mapping H1 → Core IR → Bridge**

Just say which.
