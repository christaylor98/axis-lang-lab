# Axis Language Lab — End-to-End Pipeline

**Status:** User Documentation
**Applies to:** Post-Wave 5 (Execution)

---

## Pipeline Overview

The Axis Language Lab pipeline transforms **surface syntax** into **executable behavior** through a series of well-defined, authority-bounded phases.

```
Surface Input → Parsing → Lowering → Core IR → Validation → Execution
     |            |          |          |          |            |
  .axis file    AST    [SEMANTIC    Canonical   Structural   Observable
                       AUTHORITY]   Artifact    Checks       Behavior
```

Each phase has:

* Clear inputs and outputs
* Explicit authority boundaries
* No backflow of information
* Deterministic behavior

---

## Phase 1: Surface Input

**Artifact:** `.axis` source files (or inline test strings)

Surface syntax is parsed according to the Axis surface grammar.

At this stage:

* No semantic meaning is assigned
* Structure is validated syntactically
* No name resolution occurs
* No type checking occurs

Surface syntax is **inert structure**.

---

## Phase 2: Parsing

**Input:** Surface syntax (text)
**Output:** Abstract Syntax Tree (AST)
**Authority:** Structural correctness only

The parser produces an AST that represents the **structure** of the input program.

AST properties:

* Nodes correspond to surface constructs
* No semantic elaboration occurs
* Names are preserved as-is
* No desugaring happens yet

Parsing failures are **syntax errors**, not semantic errors.

The AST is passed to the lowering phase unchanged.

---

## Phase 3: Lowering (Semantic Authority)

**Input:** AST
**Output:** Core IR bundle
**Authority:** **SOLE SEMANTIC AUTHORITY**

Lowering is where **all program meaning is assigned**.

### What Lowering Does

* Resolves names (e.g., registry function lookups)
* Assigns semantics to surface constructs
* Desugars all surface forms into Core IR nodes
* Enforces profile admission rules
* Produces a complete, self-contained Core IR bundle

### Lowering Contract

Lowering must be:

* **Total:** Every admitted AST node has a lowering rule
* **Deterministic:** Same AST always produces same Core IR
* **Explicit:** No implicit semantics
* **Final:** Semantics are frozen after lowering

If lowering cannot assign semantics (e.g., unresolved name, profile violation), it **fails explicitly**.

### Registry Resolution

During lowering, all function calls are resolved against the active registry.

* Name lookup happens **only during lowering**
* Numeric registry IDs are embedded into Core IR
* No symbolic names enter Core IR

Registry resolution is **semantic**, not structural.

### Configuration Boundary

Configuration (e.g., active profile, registry selection) is evaluated **before lowering**.

Configuration influences:

* Which registry entries are visible
* Which profiles are active
* Admission rules

Configuration does **not** alter Core IR meaning or structure.

---

## Phase 4: Core IR

**Artifact:** Core IR bundle (version 0.2)
**Authority:** Canonical semantic representation

Core IR is the **output** of lowering.

Properties:

* **Numeric only:** All names resolved to IDs
* **Configuration-free:** No policy embedded
* **Self-contained:** Fully determines program meaning
* **Immutable:** Never modified after emission

Core IR is the **semantic endpoint** of compilation.

### Core IR Bundle Structure

A Core IR bundle contains:

* `version`: "0.2"
* `core_term`: The semantic root (a Core IR node)
* Optional metadata (annotations, string tables, spans)

Metadata is **non-authoritative** and may be removed without semantic loss.

---

## Phase 5: Validation

**Input:** Core IR bundle
**Output:** Pass/fail + diagnostic messages
**Authority:** Structural correctness checks

Validation enforces **structural invariants** of Core IR.

Validation checks:

* Well-formedness (e.g., valid node types)
* Arity constraints (e.g., function calls match expected arguments)
* Scoping rules (e.g., variable references are in scope)
* Type consistency (future)

Validation does **not**:

* Assign semantics
* Modify Core IR
* Infer meaning
* Optimize

Validation failures indicate:

* A lowering bug, or
* Malformed Core IR

Validation is **not semantic**—it is structural verification only.

---

## Phase 6: Execution

**Input:** Validated Core IR bundle
**Output:** Observable behavior (via registry functions)
**Authority:** Projection of Core IR semantics

Execution interprets Core IR deterministically.

### Execution Model

The runtime interpreter:

* Traverses Core IR structure directly
* Evaluates nodes in canonical order
* Executes registry-bound calls when encountered
* Produces observable output via registry functions

The interpreter performs:

* **No optimization**
* **No reordering**
* **No caching**
* **No concurrency**
* **No speculative execution**

Execution is **deterministic by construction** (modulo non-deterministic registry functions).

### Registry Runtime Binding

At runtime:

* Numeric registry IDs are dispatched to registered functions
* Function implementations are provided by the runtime environment
* All observable behavior originates from registry functions

The runtime does not:

* Modify Core IR
* Reinterpret semantics
* Introduce new behavior

Registry functions are the **only source of observable effects**.

### Error Handling

Runtime errors occur only when:

* A registry function fails
* A numeric registry ID is not bound
* Core IR is malformed (should not happen if validation passed)

Errors are surfaced immediately and never silenced.

---

## Authority Boundaries

### Semantic Authority

**Lowering is the sole semantic authority.**

* Before lowering: no meaning assigned
* During lowering: semantics determined
* After lowering: semantics frozen

No other phase may assign, infer, or alter semantics.

### Validation Authority

Validation enforces **structural correctness**, not semantics.

It verifies that Core IR conforms to its schema but does not interpret meaning.

### Execution Authority

Execution **projects** Core IR semantics into observable behavior.

It does not:

* Change what a program means
* Introduce new semantics
* Optimize or transform

Execution authority begins **at the runtime boundary** and ends with registry function dispatch.

---

## Data Flow Summary

| Phase      | Input              | Output            | Authority                  |
|------------|--------------------|-------------------|----------------------------|
| Parsing    | Surface text       | AST               | Structural                 |
| Lowering   | AST                | Core IR           | **Semantic (sole)**        |
| Validation | Core IR            | Pass/fail         | Structural verification    |
| Execution  | Validated Core IR  | Observable output | Projection (registry-bound)|

---

## No Backflow

Information flows **forward only**.

* Execution does not inform lowering
* Validation does not inform lowering
* Core IR is never regenerated based on execution results

This ensures:

* Deterministic compilation
* Reproducible semantics
* No hidden feedback loops

---

## Concrete Example

Input surface syntax:

```axis
fn main() {
  print_int(42)
}
```

### After Parsing

AST:

```
FunctionDecl {
  name: "main",
  params: [],
  body: Block {
    exprs: [
      Call {
        name: "print_int",
        args: [IntLit(42)]
      }
    ]
  }
}
```

### After Lowering

Core IR (simplified):

```
CLam {
  param: 0,  // dummy binding
  body: CApp {
    func: CVar(1),  // registry ID 1 = print_int
    arg: CIntLit(42)
  }
}
```

Name `"print_int"` resolved to numeric ID `1`.

### After Validation

Validation confirms:

* `CApp` has valid `func` and `arg`
* `CVar(1)` is a valid registry reference
* Structure is well-formed

### During Execution

Interpreter:

1. Evaluates `CLam` (creates closure)
2. Evaluates `CApp`
3. Dispatches to registry function ID `1` with argument `42`
4. Registry function executes (prints "42" to stdout)

---

## Key Takeaways

* **Lowering assigns all semantics**
* **Core IR is the canonical artifact**
* **Validation verifies structure**
* **Execution projects semantics into behavior**
* **Registry functions are the only observable effect source**

---

**End of Pipeline Documentation**
