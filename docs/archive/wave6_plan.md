# Wave 6 Plan — Observability (Execution Tracing)

## Status

PLANNED
Not started

This document is authoritative for Wave 6.

---

## Purpose

Wave 6 introduces **observability** for Core IR execution.

Wave 6 adds a **trace projection** alongside the existing **runtime projection**:

* Core IR is still the sole semantic artifact
* The runtime still executes Core IR as-is (Wave 5 rules remain frozen)
* Wave 6 adds a structured, opt-in mechanism to **observe execution** without changing meaning

Observability is treated as a **separate projection** of execution, not a semantic extension.

---

## Non-Goals (Explicit)

Wave 6 does NOT include:

* changes to AST
* changes to lowering rules
* Core IR schema changes
* new language constructs
* optimisation
* concurrency / async execution
* debugger UI
* time-travel debugger
* replay or deterministic re-execution guarantees
* security sandboxing
* policy enforcement (allow/deny, quotas, etc.)
* runtime mutation of Core IR or registries

Wave 6 is **trace capture only**.

---

## Governing Constraints (Hard)

1. **Tracing must not affect semantics**
2. **Tracing must be opt-in**
3. **Tracing must not require Core IR changes**
4. **Tracing must not alter registry dispatch**
5. **Tracing must preserve structural ordering**
6. **Tracing must not introduce non-determinism**
7. **Tracing must not leak symbolic names into Core IR**
8. **Runtime remains the consumer; lowering remains the semantic authority**

---

## Concept: Trace as a Projection

Execution already exists (Wave 5).

Wave 6 adds:

> A structured stream of execution events emitted by the runtime during interpretation.

This stream is a **projection of what happened**, not a source of authority.

---

## Trace Architecture (Minimal)

Wave 6 introduces exactly:

1. **Trace Event Model**
2. **Trace Sink Interface**
3. **Runtime Instrumentation Hooks**
4. **Trace Tests**

No other runtime components are permitted.

---

## Trace Event Model

### Requirements

Trace events must be:

* structured (typed records, not ad-hoc strings)
* strictly ordered (the order reflects actual execution order)
* minimal but sufficient for later tooling (profiling, debugger, replay in future waves)

### Event Categories (Wave 6)

Wave 6 supports these event categories only:

#### A) Execution Lifecycle

* `ExecStart`
* `ExecEnd { status }`

#### B) Expression Evaluation Boundaries

* `EvalEnter { node_kind }`
* `EvalExit  { node_kind, outcome }`

`node_kind` is an enum describing Core IR node class (e.g. Lit, If, Block/Let, Call, Unsupported).

#### C) Control Flow

* `IfEnter`
* `IfCondValue { value_kind }`
* `IfBranch { then_or_else }`

No path prediction, no extra annotations.

#### D) Registry Calls (Load-Bearing)

* `CallEnter { target_id, argc }`
* `CallArgValue { index, value_kind }`
* `CallExit { outcome }`

Important:

* `target_id` is numeric only
* no function names
* no registry metadata included

#### E) Errors

* `RuntimeError { kind }`

### Data Discipline

Trace event payloads must be:

* numeric IDs only
* value summaries only (type/kind, not large blobs)
* no embedded names
* no config-derived metadata
* no “helpful” pretty-print that becomes an API

If values need to be inspected later, that is a future wave.

---

## Trace Sink Interface

Introduce a trace sink interface that:

* can be passed into the execution entry point (or via execution options)
* receives events synchronously in-order
* does not allocate excessively by default
* can be a no-op sink with near-zero overhead when disabled

Constraints:

* no global logging
* no implicit stdout/stderr tracing
* no background threads
* no buffering requirements (sink decides)

Minimum viable API:

* `on_event(event: TraceEvent)`

---

## Runtime Instrumentation Rules

Instrumentation must:

* emit events at **fixed interpreter points**
* be fully deterministic
* not change evaluation order
* not add extra evaluation
* not swallow errors

Tracing must not call registry functions.
Tracing must not introduce new observable effects.

---

## Configuration and Enablement

Tracing enablement is runtime-only:

* It is enabled by passing a sink (or options struct) to the execution call.
* When no sink is provided, tracing is disabled.

No langlab.toml changes in Wave 6.

---

## Error Handling

Trace emission must be “best effort” but not dangerous:

* If a sink fails (panic/error), behavior must be defined.
* Default Wave 6 rule: **sink failure aborts execution with explicit runtime error** (fail-closed).

(Alternative is fail-open/no-trace; pick one and document it. Wave 6 must commit to one.)

---

## Testing Strategy

### Required Tests

Tests must operate on Core IR directly.

1. **Lifecycle events emitted**

   * Start then End
2. **Deterministic ordering**

   * known Core IR produces exactly ordered event sequence
3. **Registry call trace**

   * CallEnter, args, CallExit appear correctly
4. **Conditional trace**

   * IfEnter, CondValue, Branch events
5. **Error trace**

   * RuntimeError emitted when registry fails / binding missing
6. **No tracing when disabled**

   * execution works without sink and emits nothing

### Golden Tests

Wave 6 MAY include event-sequence goldens only if:

* events are deterministic
* event payloads are stable and minimal
* comparisons are line-based and versioned

Binary goldens are discouraged.

---

## Acceptance Criteria

Wave 6 is complete when:

* Execution produces a structured trace when enabled
* Trace is strictly ordered and stable
* Trace contains numeric IDs only (no names)
* Tracing can be disabled with near-zero overhead
* Wave 5 execution semantics are unchanged
* No Core IR / lowering / registry spec changes occurred
* Runtime tests validate trace correctness

---

## Change Control

Any change that:

* alters evaluation order
* alters registry dispatch behavior
* adds new trace categories beyond this plan
* introduces policy enforcement or replay guarantees

requires a new wave.

---

## End of Document
