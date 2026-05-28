# Axis Surface H1 — Draft Specification v0.1

---

## 1. Scope and Purpose

**Axis Surface H1** is a readable syntax surface over Axis semantics.

H1 is:

* a projection
* not a semantic authority
* not privileged over other surfaces

All H1 programs MUST desugar into **Normal Form (NF)** via normalisation, and then lower into **Core IR** using the fixed, embedded lowering implementation.

---

## 2. Normative Invariants

**H1.1**
Core IR is the sole semantic authority.

**H1.2**
Normalisation transforms H1 constructs into Normal Form (NF).

**H1.3**
Lowering is fixed, embedded, and shared across all surfaces.

**H1.4**
If an H1 construct cannot be normalized to NF, it is invalid.

**H1.5**
Execution strategy is never expressed in H1 syntax.

---

## 3. Lexical and Structural Properties

### 3.1 Immutability

All bindings are immutable.

```h1
let x = expr;
```

Rebinding introduces a new value.
There is no mutation syntax in v0.1.

---

### 3.2 Blocks

```h1
{
  stmt;
  expr
}
```

Blocks:

* are expression-valued
* evaluate to their final expression
* impose sequencing for side effects

---

## 4. Expressions and Control Flow

### 4.1 Conditionals

```h1
if cond {
  a
} else {
  b
}
```

* Condition must evaluate to boolean
* No implicit truthiness
* Always expression-valued

---

### 4.2 Loops

#### Infinite loop

```h1
loop {
  body
}
```

Represents intentional unbounded repetition.

Normalisation transforms this to explicit recursion or canonical loop construct.

Lowering (fixed, embedded) produces:

* explicit recursion, or
* a canonical loop primitive

Execution strategy is unspecified.

---

#### Iteration

```h1
for x in iterable {
  body
}
```

Represents iteration over a collection or source.

No ordering, parallelism, or execution guarantees are implied.

---

## 5. Pattern Matching

```h1
match value {
  A(x) => expr,
  B(y) => expr,
  _    => expr,
}
```

Properties:

* deterministic
* no inference
* no implicit exhaustiveness checks

Lowered to:

* explicit tests
* explicit bindings
* explicit branches

---

## 6. Functions

```h1
fn name(args) {
  body
}
```

Functions:

* are pure by default
* may return effect values
* do not execute effects themselves

---

## 7. Side Effects

### 7.1 Principle

Side effects are **explicit values** that flow through the program.

The compiler describes effects.
The bridge executes effects.

---

### 7.2 Effectful Operations

Examples:

```h1
print(x);
let v = read_file("file");
send(tx, v);
let r = recv(rx);
```

Semantics:

* effect creation is explicit
* ordering is structural
* no effect executes during compilation

---

### 7.3 Sequencing

```h1
print(a);
print(b);
```

Represents an explicit sequence of effects.

Reordering is not permitted unless explicitly allowed by annotation.

---

## 8. Concurrency and Multiprocessing (Intent Only)

### 8.1 Spawning

```h1
let h = spawn {
  expr
};

let r = join h;
```

Semantics:

* describes concurrent work
* returns an opaque handle
* synchronization is explicit

No execution model is implied.

---

### 8.2 Channels

```h1
let ch = channel<T>();
let tx = get_sender(ch);
let rx = get_receiver(ch);

send(tx, v);
let x = recv(rx);
```

Channels express communication intent only.

Transport, buffering, and blocking behavior are bridge concerns.

---

## 9. Controllers

### 9.1 Definition

```h1
controller Name {
  on Event(x) => handler(x),
}
```

Controllers describe:

* event bindings
* reaction structure

They do not describe:

* scheduling
* threading
* delivery guarantees

---

### 9.2 Execution

```h1
run Name;
```

The bridge:

* binds event sources
* executes returned effects
* manages lifecycle

---

## 10. Annotations (v0.1 — Closed Set)

Annotations express **intent only**.

They:

* do not affect normalisation
* do not affect lowering (lowering is fixed and embedded)
* do not introduce semantics
* are preserved as metadata into Core IR

### 10.1 Allowed Annotations

#### `@intent.parallel`

```h1
@intent.parallel
for x in items {
  body
}
```

Meaning:

> Iterations are independent.
> Parallel execution is permitted but not required.
> Sequential execution is always valid.

This is advisory metadata only. Bridges may execute sequentially or concurrently. Both are correct.

---

#### `@intent.serial`

```h1
@intent.serial
for x in items {
  body
}
```

Meaning:

> Execution order must be preserved.
> Parallel execution is not permitted.

---

### 10.2 Annotation Rules

* Annotations attach to:

  * loops
  * blocks
* `@intent.parallel` is advisory only and may be ignored
* `@intent.serial` is mandatory for all compliant bridges
* Lowering MUST ignore annotations

---

## 11. Explicitly Forbidden Constructs

The following are **hard errors** in H1:

* `async`
* `await`
* futures
* executors
* threads
* locks
* schedulers
* time-based semantics

These are bridge concerns only.

---

## 12. Execution Boundary

There is exactly one execution boundary:

```h1
run main;
```

Before `run`:

* all code describes semantics only

After `run`:

* effects are executed by the bridge

---

## 13. Resurfacing Guarantee

Any H1 program:

* lowers to Core IR
* may be resurfaced into another surface
* preserves semantics exactly

Formatting, naming, and layout are not preserved.

---

## 14. v0.1 Summary

Axis Surface H1 v0.1 provides:

* immutable-by-default bindings
* explicit side effects
* intent-only concurrency
* optional parallel permission
* no execution trivia
* full semantic reversibility

> **H1 describes meaning.
> The bridge decides how it runs.**

