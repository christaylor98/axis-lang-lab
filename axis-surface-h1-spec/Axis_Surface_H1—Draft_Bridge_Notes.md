# 1. Reference Bridge (Conservative) — Specification v0.1

---

## 1.1 Purpose

The **Conservative Bridge** is the reference execution bridge for Axis.

It prioritizes:

* determinism
* debuggability
* semantic clarity

It is the **baseline correctness implementation** against which all other bridges are compared.

---

## 1.2 Design Principles

**C1.1**
Execution is strictly sequential unless explicitly required otherwise by semantics.

**C1.2**
All optimizations are disabled.

**C1.3**
All intent metadata is ignored except where semantics demand enforcement (`@intent.serial`).

---

## 1.3 Capability Profile

```toml
[bridge]
name = "conservative"

[capabilities]
parallel_loops = false
distributed_execution = false
channels = true
controllers = true
```

---

## 1.4 Execution Rules

### C4.1 Parallel intent

If Core IR metadata contains:

```
@intent.parallel = true
```

The Conservative Bridge MUST:

* execute sequentially
* preserve iteration order

This is always correct.

---

### C4.2 Serial intent

If Core IR metadata contains:

```
@intent.serial = true
```

The Conservative Bridge MUST:

* execute sequentially
* preserve strict ordering

---

### C4.3 Effects

* Effects are executed immediately when reached
* Effects are executed in program order
* No batching
* No reordering

---

## 1.5 Spawn and Join

* `spawn` executes synchronously
* `join` is a no-op returning the completed result

This preserves semantics without concurrency.

---

## 1.6 Channels

* Channels are implemented as in-process FIFO queues
* `send` enqueues
* `recv` blocks until value is available

---

## 1.7 Controllers

* Event handlers execute sequentially
* No parallel dispatch
* No batching
* No reordering

---

## 1.8 Failure Handling

* Any runtime error aborts execution
* No retries
* No recovery

---

## 1.9 Compliance Statement

The Conservative Bridge is:

* maximally boring
* maximally predictable
* maximally correct

It exists to define **ground truth execution**.

---

---

# 2. Parallel Bridge — Specification v0.1

---

## 2.1 Purpose

The **Parallel Bridge** exploits available parallelism on a single host.

It prioritizes:

* throughput
* CPU utilization
* opportunistic optimization

---

## 2.2 Design Principles

**P2.1**
Parallelism is applied only where explicitly permitted.

**P2.2**
Sequential fallback is always valid.

**P2.3**
Semantics are preserved exactly.

---

## 2.3 Capability Profile

```toml
[bridge]
name = "parallel"

[capabilities]
parallel_loops = true
distributed_execution = false
channels = true
controllers = true
```

---

## 2.4 Parallel Execution Rules

### P4.1 Parallel loops

If:

```
@intent.parallel = true
```

The bridge MAY:

* execute loop iterations concurrently
* use thread pools
* use work-stealing

The bridge MAY ALSO:

* execute sequentially

---

### P4.2 Serial loops

If:

```
@intent.serial = true
```

The bridge MUST:

* execute sequentially
* prohibit concurrent execution

---

## 2.5 Effects

* Effects MAY be executed concurrently if independent
* Effects MUST respect explicit sequencing
* Serial sections are never reordered

---

## 2.6 Spawn and Join

* `spawn` creates a concurrent execution context
* `join` waits for completion

Implementation strategy is unspecified.

---

## 2.7 Channels

* Channels MAY be backed by concurrent queues
* FIFO ordering per channel MUST be preserved

---

## 2.8 Controllers

* Handlers MAY be executed concurrently
* Serial constraints MUST be respected
* Event order MAY be relaxed unless constrained

---

## 2.9 Failure Handling

* Worker failure MAY cause retry
* If retry is impossible, execution MUST fail fast

---

## 2.10 Compliance Statement

The Parallel Bridge is:

* opportunistic
* performance-oriented
* semantically conservative

---

---

# 3. End-to-End Demo — H1 → Core IR → Bridge

This demo is **normative** and intended for documentation and validation.

---

## 3.1 H1 Program

```h1
fn main() {
  let prices = load_prices();

  @intent.parallel
  for p in prices {
    emit(compute(p));
  }
}

run main;
```

---

## 3.2 Schema AST (Conceptual)

```
Function main
 └─ Let prices = Call load_prices
 └─ ForLoop (iter = prices)
     ├─ Annotation: intent.parallel = true
     └─ Body:
         └─ Call emit
             └─ Call compute(p)
```

---

## 3.3 Core IR (Conceptual)

```
fn main():
  prices = call load_prices
  for_each prices:
    effect emit(call compute(p))
```

Metadata:

```
@intent.parallel = true
```

No semantics encoded in metadata.

---

## 3.4 Execution: Conservative Bridge

Behavior:

* Iterates prices sequentially
* Executes `emit(compute(p))` one at a time

Outcome:

* Correct
* Deterministic
* No parallelism

---

## 3.5 Execution: Parallel Bridge

Behavior:

* Iterations MAY execute concurrently
* Effects MAY be batched or parallelized

Outcome:

* Same observable semantics
* Higher throughput
* Ordering preserved where required

---

## 3.6 Key Property Demonstrated

* Same H1 program
* Same Core IR
* Same semantics
* Different execution strategies
* No code changes

---

## 3.7 Validation Criteria

This demo is correct if:

* Core IR is identical under all surfaces
* Output values are identical
* Effect ordering constraints are respected
* Removing parallel capability does not break correctness
