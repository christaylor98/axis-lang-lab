# Axis Language Lab — Execution Model (Wave 5)

**Status:** User Documentation
**Wave:** 5 (FROZEN)
**Applies to:** Core IR 0.2 + Rust runtime bridge

---

## What Execution Means

**Execution** is the process of **projecting Core IR semantics into observable behavior**.

It is:

* The interpretation of Core IR structure
* The dispatch of registry-bound function calls
* The source of program output and effects

Execution is **not**:

* A source of semantics
* An optimization stage
* A transformation of Core IR
* A co-author of program meaning

Execution **projects** what lowering decided.

---

## Execution as Projection

Wave 5 establishes execution as a **projection model**.

This means:

* Core IR is consumed **as-is**
* Core IR is never modified, enriched, or annotated during execution
* The runtime is a **consumer**, not a collaborator
* Semantics are determined **before** execution, not during

The interpreter **walks** Core IR structure and **realizes** the semantics already frozen there.

---

## The Interpreter

The runtime interpreter is a **deterministic tree-walking interpreter**.

It operates by:

* Traversing Core IR nodes in canonical order
* Evaluating expressions bottom-up
* Preserving structural relationships
* Dispatching registry calls when encountered

### Interpreter Properties

The interpreter performs:

* **No optimization**
* **No caching**
* **No reordering**
* **No speculative execution**
* **No concurrency**
* **No async execution**

Any such behavior would require a new wave.

### Evaluation Order

The interpreter evaluates:

* Function arguments **left-to-right**
* Conditional branches **explicitly** (only the selected branch)
* Let bindings **sequentially**
* Applications **applicatively** (argument before function body)

Evaluation order is **deterministic and explicit**.

---

## Registry Runtime Adapter

Registry-bound function calls are the **only source of observable behavior** in Wave 5.

### Runtime Binding

At runtime:

* Numeric registry IDs (embedded in Core IR) are dispatched to runtime function implementations
* Function implementations are provided by the runtime environment
* Arity is enforced at dispatch time
* No name lookup occurs

The mapping from numeric IDs to functions is:

* **Fixed at startup**
* **Immutable during execution**
* **Deterministic**

### Registry Function Semantics

Registry functions may:

* Produce values
* Perform I/O
* Fail with errors
* Be non-deterministic (if marked as such)

The runtime does **not** inspect or interpret registry function implementations.

Registry functions are **opaque** to the interpreter.

---

## What Execution Does

### 1. Accepts Validated Core IR

Execution begins with a **validated Core IR bundle**.

* Core IR must pass validation before execution
* The runtime does not re-validate semantics
* Malformed Core IR should not reach the runtime

### 2. Traverses Core IR Deterministically

The interpreter walks the Core IR tree:

* `CIntLit`, `CBoolLit`, `CUnitLit` → Return literal values
* `CVar` → Look up bound value in environment
* `CLam` → Create a closure
* `CLet` → Evaluate binding, extend environment
* `CIf` → Evaluate condition, evaluate selected branch
* `CApp` → Evaluate function and argument, apply

### 3. Dispatches Registry Calls

When a registry-bound call (`CApp` with a registry function) is encountered:

* Extract numeric registry ID
* Look up function in runtime registry
* Evaluate arguments
* Dispatch to function implementation
* Return result (or propagate error)

### 4. Produces Observable Output

Observable behavior originates **only** from registry functions.

Examples:

* `print_int(42)` → Calls registry function, prints "42"
* `read_input()` → Calls registry function, returns user input
* `random()` → Calls non-deterministic registry function

The interpreter itself produces **no observable effects**.

---

## What Execution Does Not Do

### 1. Modify Core IR

Core IR is **immutable** during execution.

* No in-place updates
* No annotation injection
* No metadata modification

### 2. Infer or Reinterpret Semantics

The runtime **does not**:

* Infer intent
* Reinterpret nodes
* Introduce implicit behavior

If semantics are not explicit in Core IR, the runtime cannot proceed.

### 3. Optimize

The interpreter performs:

* No constant folding
* No inlining
* No dead code elimination
* No tail-call optimization

Execution is **faithful to Core IR structure**.

### 4. Introduce Concurrency

Execution is **sequential** by construction.

* No parallelism
* No async/await
* No task spawning

Any such behavior requires a new wave.

### 5. Recover from Errors

The runtime does not:

* Swallow errors
* Retry operations
* Apply fallback logic

Errors are surfaced immediately to the caller.

---

## Error Model

Runtime errors occur **only** in the following cases:

1. **Registry function failure**

   * The function itself signals an error
   * Example: `divide_by_zero()`

2. **Invalid registry binding**

   * Numeric registry ID is not bound to a function
   * Example: Core IR references ID `99`, but no function is registered

3. **Malformed Core IR**

   * Core IR violates structural invariants
   * Example: `CApp` with missing argument
   * This should **not happen** if validation ran

When an error occurs:

* Execution halts immediately
* Error is propagated to the caller
* No recovery is attempted

The runtime **does not hide errors**.

---

## Determinism Guarantees

The interpreter is **deterministic by construction**.

Given:

* The same Core IR bundle
* The same registry bindings
* The same deterministic registry functions

Execution produces **identical results** every time.

### Non-Determinism

Non-determinism is permitted **only** inside registry functions explicitly marked as non-deterministic.

Examples:

* `random()` — Produces unpredictable values
* `current_time()` — Returns current timestamp
* `read_input()` — Depends on external input

The **interpreter itself** introduces no non-determinism.

---

## Validation Boundary

Execution assumes Core IR is **already validated**.

The runtime does **not**:

* Revalidate Core IR structure
* Check arity constraints
* Verify scoping rules

Validation is a **compiler concern**, not a runtime concern.

If Core IR passes validation, the runtime trusts it.

---

## No Feedback Loop

Execution does **not** feed back into compilation.

* No runtime profiling
* No dynamic recompilation
* No optimization based on execution traces

This ensures:

* Deterministic compilation
* Reproducible builds
* No hidden dependencies

Execution is **one-way consumption** of Core IR.

---

## Testing Model

Wave 5 runtime tests:

* Operate on **Core IR directly** (not surface syntax)
* Do not invoke the parser
* Do not invoke lowering
* Verify execution correctness only

This isolates runtime behavior from frontend concerns.

---

## Concrete Example

Core IR (simplified):

```
CApp {
  func: CVar(1),  // registry ID 1 = print_int
  arg: CIntLit(42)
}
```

Execution steps:

1. Evaluate `CApp`
2. Evaluate `func` → Resolve `CVar(1)` to registry function ID `1`
3. Evaluate `arg` → `CIntLit(42)` evaluates to integer `42`
4. Dispatch: Call registry function `1` with argument `42`
5. Registry function executes → Prints "42" to stdout
6. Return unit value

---

## Runtime Environment

The runtime environment provides:

* Registry function bindings (numeric ID → implementation)
* I/O primitives (stdout, stderr, stdin)
* Error handling infrastructure

The runtime environment does **not** provide:

* Garbage collection (values are stack-allocated or trivially managed)
* Concurrency primitives
* Dynamic linking
* JIT compilation

---

## Architectural Consequences

With Wave 5 frozen:

* Core IR is now **load-bearing for execution**
* Any change to Core IR structure affects execution
* The runtime bridge is **no longer a proof-of-concept**
* Future waves must treat execution semantics as **fixed**

Changes to execution behavior are **semantic events** requiring a new wave.

---

## Performance Expectations

Wave 5 execution is **not optimized**.

Expected characteristics:

* Interpretation overhead
* No inlining
* No caching
* Stack-based evaluation

This is **intentional**.

Optimization is **out of scope** for Wave 5.

Future waves may introduce:

* JIT compilation
* AOT compilation
* Optimized interpreters

But these require **new semantic decisions** and cannot be introduced silently.

---

## Key Takeaways

* Execution is a **projection** of Core IR semantics
* The interpreter is **deterministic and explicit**
* Registry functions are the **only source of observable behavior**
* Execution does **not modify or reinterpret** Core IR
* Errors are **explicit and immediate**
* Validation happens **before** execution

---

**End of Execution Model Documentation**
