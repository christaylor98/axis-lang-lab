# Wave 6 — Allowed Execution Events (Authoritative)

## 1. Execution Lifecycle

These events bracket a single execution invocation.

1. **`ExecStart`**

   * Emitted once at execution entry

2. **`ExecEnd { status }`**

   * Emitted once at execution exit
   * `status ∈ { Success, RuntimeError }`

---

## 2. Expression Evaluation Boundaries

These events describe *when* Core IR nodes are evaluated, not *why*.

3. **`EvalEnter { node_kind }`**

   * Emitted immediately before evaluating a Core IR node

4. **`EvalExit { node_kind, outcome }`**

   * Emitted immediately after evaluation completes
   * `outcome ∈ { Value, Error }`

`node_kind` is an enum of Core IR node classes only:

* `Lit`
* `If`
* `Block` / `Let`
* `Call`
* `Unsupported`

No symbolic names. No IDs.

---

## 3. Control Flow (Conditionals Only)

These events expose *which path executed*, not why.

5. **`IfEnter`**

   * Emitted when beginning evaluation of an `If`

6. **`IfCondValue { value_kind }`**

   * Emitted after condition evaluation
   * `value_kind ∈ { BoolTrue, BoolFalse }`

7. **`IfBranch { branch }`**

   * Emitted when selecting a branch
   * `branch ∈ { Then, Else }`

No path prediction.
No nesting annotations.
No jump targets.

---

## 4. Registry Call Events (Load-Bearing)

These are the **most important** Wave 6 events.

8. **`CallEnter { target_id, argc }`**

   * Emitted immediately before invoking a registry function
   * `target_id` is numeric only
   * `argc` is the evaluated argument count

9. **`CallArgValue { index, value_kind }`**

   * Emitted after each argument evaluation
   * Arguments are reported **left-to-right**
   * `value_kind` is a summary only (e.g. Int, Bool, Unit)

10. **`CallExit { outcome }`**

    * Emitted after registry function returns or fails
    * `outcome ∈ { Value, Error }`

No function names.
No registry metadata.
No timing data.

---

## 5. Runtime Errors

These events surface failures without interpretation.

11. **`RuntimeError { kind }`**

    * Emitted exactly once per failing execution
    * `kind ∈ {`

      * `RegistryFailure`
      * `MissingBinding`
      * `MalformedCoreIR`
      * `TraceSinkFailure`
        `}`

No stack traces.
No recovery attempts.
No policy decisions.

---

## Explicitly Forbidden Events (Wave 6)

Wave 6 must **not** observe or emit events for:

* value contents (beyond type/kind)
* variable names
* registry names
* source locations
* timestamps or durations
* memory allocation
* instruction counts
* optimisation decisions
* scheduling or concurrency
* replay checkpoints
* debugger breakpoints
* policy enforcement

These belong to future waves **only if justified**.

---

## Cardinality Rules (Invariant)

* `ExecStart` → exactly once
* `ExecEnd` → exactly once
* `EvalEnter` / `EvalExit` → properly nested
* `CallEnter` → always paired with `CallExit`
* `RuntimeError` → at most once per execution

Violation of these rules is a runtime bug.

---

## Final Statement

This list is **complete and closed** for Wave 6.

Any additional execution observation:

* requires a new wave
* requires an explicit authority discussion
* must not be smuggled in “for convenience”

Wave 6 observes **structure, order, and effects** — nothing more.

