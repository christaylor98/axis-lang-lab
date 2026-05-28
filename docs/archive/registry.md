# Axis Language Lab — Registry Model

**Status:** User Documentation
**Applies to:** Axis Registry 0.1

---

## What Registries Are

A **registry** is an **explicit, enumerable set of callable functions** available to an Axis program.

Registries exist to:

* Eliminate implicit built-ins
* Make all callable operations explicit
* Provide a stable contract between surface syntax and runtime behavior
* Enable profile-based admission control

**If a function is not in the active registry, it does not exist.**

---

## Why Registries Exist

Traditional languages have **implicit built-ins**:

* `print()` just works
* `len()` is assumed to exist
* Math functions are "always there"

This creates:

* Ambiguity about what functions are available
* Hidden dependencies on runtime libraries
* Difficulty reasoning about program capabilities

Axis Language Lab requires **explicit enumeration** of all callable functions.

This ensures:

* No guessing about what exists
* Clear boundaries for what programs can do
* Deterministic resolution during lowering

---

## Registry Format

Registries are stored in `.axreg` files.

Each registry is a **flat, line-oriented declarative file**.

Example:

```
fn print_int
  arity 1
  deterministic false
  profile safe
end

fn add
  arity 2
  deterministic true
  profile safe
  profile unsafe
end
```

### Registry Record Structure

Each function declaration includes:

* **Name:** Canonical identifier (e.g., `print_int`)
* **Arity:** Number of required arguments
* **Deterministic flag:** Whether the function is deterministic
* **Profile(s):** Which execution profiles admit this function

No other metadata is permitted in registry 0.1.

---

## Why Calls Are Registry-Bound

All function calls in Axis programs are **registry-bound**.

This means:

* Every `Call` in the AST must resolve to a registry entry
* Resolution happens **during lowering**
* Numeric registry IDs are embedded into Core IR
* Runtime dispatch uses numeric IDs only

### Example

Surface syntax:

```axis
print_int(42)
```

Lowering:

1. Look up `print_int` in the active registry
2. Find entry:

   ```
   fn print_int
     arity 1
     deterministic false
     profile safe
   end
   ```
3. Check arity: `1` argument expected, `1` provided ✓
4. Check profile: If active profile is `safe`, admit ✓
5. Assign numeric ID (e.g., `1`)
6. Emit Core IR: `CApp { func: CVar(1), arg: CIntLit(42) }`

At runtime:

* Dispatch numeric ID `1` to the `print_int` implementation

No name lookup occurs at runtime.

---

## How Numeric IDs Are Assigned

Numeric registry IDs are assigned **deterministically during lowering**.

Rules:

* Each unique function name in the active registry gets a unique numeric ID
* IDs are assigned in a canonical order (e.g., lexicographic)
* IDs are **bundle-local** (valid only within that Core IR bundle)
* The same function may have different IDs in different bundles

IDs are **not stable** across:

* Different registry sets
* Different compilation runs
* Different compilers

IDs are **stable** within a single Core IR bundle.

---

## Deterministic vs Non-Deterministic Functions

Registry functions are marked as either **deterministic** or **non-deterministic**.

### Deterministic Functions

Properties:

* Same inputs always produce same outputs
* No observable side effects
* Referentially transparent

Examples:

* `add(2, 3)` → Always returns `5`
* `multiply(4, 5)` → Always returns `20`

Deterministic functions:

* Can be optimized (future)
* Can be cached (future)
* Can be reordered (future)

### Non-Deterministic Functions

Properties:

* May produce different outputs on repeated calls
* May perform I/O
* May depend on external state

Examples:

* `print_int(42)` → Performs I/O (stdout)
* `random()` → Returns unpredictable values
* `current_time()` → Depends on system clock

Non-deterministic functions:

* Must be executed in order
* Cannot be cached
* Cannot be eliminated

The determinism flag is **not enforced** by the runtime in Wave 5.

It is **declarative metadata** for future optimizations or tooling.

---

## Execution Profiles

Profiles control **which functions are visible** during lowering.

### Profile Admission

Each registry function declares which profiles it is admitted under.

Example:

```
fn unsafe_pointer_cast
  arity 1
  deterministic true
  profile unsafe
end

fn safe_add
  arity 2
  deterministic true
  profile safe
  profile unsafe
end
```

If the active profile is `safe`:

* `safe_add` is visible
* `unsafe_pointer_cast` is **not visible**

If the active profile is `unsafe`:

* Both functions are visible

### Profile Semantics

Profiles define **visibility**, not semantics.

* Profiles do not change what a function does
* Profiles do not alter Core IR structure
* Profiles only affect **which functions can be called**

If a function is not admitted by the active profile, **lowering fails**.

---

## Why Registries Define Observable Behavior

In Axis Language Lab, **all observable behavior originates from registry functions**.

This includes:

* I/O (print, read, write)
* Non-determinism (random, time)
* External interactions (network, file system)

The Core IR interpreter itself produces **no observable effects**.

If a program has no registry-bound calls, it:

* Performs no I/O
* Produces no side effects
* Returns a value (or loops forever)

This makes program behavior **explicit and inspectable**.

---

## Registry Resolution During Lowering

Registry resolution is a **semantic operation** that happens during lowering.

### Resolution Process

1. Parse surface syntax → AST with `Call` nodes
2. For each `Call(name, args)`:

   a. Look up `name` in the active registry
   b. If not found → **lowering fails**
   c. If found, check arity
   d. If arity mismatch → **lowering fails**
   e. If profile mismatch → **lowering fails**
   f. Assign numeric ID
   g. Emit Core IR with numeric ID

3. Produce Core IR bundle with numeric IDs only

### No Runtime Resolution

The runtime does **not** perform registry lookups.

It only:

* Dispatches numeric IDs to registered functions
* Enforces arity at dispatch time
* Propagates errors

Name resolution is **complete before Core IR is emitted**.

---

## Multiple Registries

An Axis program may use **multiple registries** simultaneously.

Example configuration:

```
registries:
  - stdlib.axreg
  - math.axreg
  - io.axreg
```

### Registry Ordering

Registries are searched **in order**.

If a function name appears in multiple registries:

* The **first** match is used
* No ambiguity error is raised
* This is intentional (allows overriding)

### Numeric ID Assignment Across Registries

Numeric IDs are assigned **globally** across all active registries.

Example:

* `stdlib.axreg` contains `add`, `subtract`
* `io.axreg` contains `print_int`, `read_line`

ID assignment (deterministic order):

* `add` → ID `1`
* `print_int` → ID `2`
* `read_line` → ID `3`
* `subtract` → ID `4`

The order is canonical and deterministic.

---

## Registry Immutability

Registries are **immutable during execution**.

* No dynamic registry modification
* No function injection
* No runtime registration

Registries are loaded **once** at startup and remain fixed.

This ensures:

* Deterministic dispatch
* Predictable behavior
* No hidden dependencies

---

## Explicit Non-Goals

Registries do **not** include:

* Function implementations
* Type signatures (registry 0.1)
* Documentation strings
* Deprecation markers
* Versioning (beyond the registry spec version)
* Hierarchy or namespaces

These may be added in future registry versions but are **out of scope** for 0.1.

---

## Concrete Example

Registry file (`stdlib.axreg`):

```
fn print_int
  arity 1
  deterministic false
  profile safe
end

fn add
  arity 2
  deterministic true
  profile safe
end
```

Surface program:

```axis
fn main() {
  print_int(add(2, 3))
}
```

Lowering:

1. Resolve `add`:

   * Found in registry
   * Arity: 2 ✓
   * Profile: safe ✓
   * Assign ID `1`

2. Resolve `print_int`:

   * Found in registry
   * Arity: 1 ✓
   * Profile: safe ✓
   * Assign ID `2`

3. Emit Core IR:

   ```
   CApp {
     func: CVar(2),  // print_int
     arg: CApp {
       func: CVar(1),  // add
       arg: CTuple([CIntLit(2), CIntLit(3)])
     }
   }
   ```

At runtime:

1. Evaluate inner `CApp` (add): Dispatch ID `1` with `(2, 3)` → Returns `5`
2. Evaluate outer `CApp` (print_int): Dispatch ID `2` with `5` → Prints "5"

---

## Key Takeaways

* Registries **explicitly enumerate** all callable functions
* All calls are **registry-bound**
* Resolution happens **during lowering**
* Numeric IDs are **deterministic and bundle-local**
* Profiles control **visibility**, not semantics
* Registries define **all observable behavior**

---

**End of Registry Model Documentation**
