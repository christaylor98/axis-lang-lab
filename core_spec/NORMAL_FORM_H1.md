# Normal Form H1 (NF-H1) — Structural Contract

**Version:** 1.0  
**Authority:** Normative  
**Status:** Canonical for H1 surface  

---

## 1. Purpose and Scope

This document defines **Normal Form H1 (NF-H1)**, the structural contract that Schema AST nodes MUST satisfy before entering lowering.

NF-H1 is:

* **Structural only** — defines shape, not semantics
* **Surface-agnostic** — not tied to H1 syntax
* **Mandatory** — all Schema AST must conform before lowering
* **Enforceable** — violations are hard compiler errors

NF-H1 defines:

* Admissible node kinds
* Forbidden constructs
* Structural preconditions for lowering

NF-H1 does NOT define:

* Normalisation rules (see `normalisation.yaml`)
* Lowering semantics (embedded in compiler)
* Execution behavior
* Optimizations

---

## 2. Position in Pipeline

```
Schema AST
    ↓
Normalisation (normalisation.yaml)
    ↓
NF-H1 AST
    ↓
NF Validation (this spec)
    ↓
Lowering (embedded, fixed)
    ↓
Core IR
```

**Critical invariant:** Lowering MUST receive only NF-H1 compliant AST.

---

## 3. NF-H1 Admissible Node Set

The following node kinds are PERMITTED in NF-H1:

### 3.1 Program Structure

| Node Kind      | Purpose                          |
|----------------|----------------------------------|
| `Program`      | Top-level program container      |
| `FunctionDecl` | Function/procedure declaration   |

### 3.2 Binding and Scope

| Node Kind | Purpose                    |
|-----------|----------------------------|
| `LetExpr` | Local binding (let x = e)  |
| `Block`   | Sequence of expressions    |

### 3.3 Control Flow

| Node Kind | Purpose                          |
|-----------|----------------------------------|
| `IfExpr`  | Conditional (if-then-else)       |

### 3.4 Computation

| Node Kind    | Purpose                    |
|--------------|----------------------------|
| `LambdaExpr` | Anonymous function         |
| `CallExpr`   | Function/operator call     |

### 3.5 Values

| Node Kind | Purpose                    |
|-----------|----------------------------|
| `Ident`   | Variable reference         |
| `Literal` | Literal value (int, bool)  |
| `Unit`    | Unit value                 |

### 3.6 Structural Wrappers (TRANSPARENT)

These nodes are permitted but MUST be normalized away before dispatch:

| Node Kind      | Purpose                          | Treatment           |
|----------------|----------------------------------|---------------------|
| `Expr`         | Expression wrapper               | Strip before lower  |
| `AtomicExpr`   | Atomic expression wrapper        | Strip before lower  |

**Note:** Wrapper normalization occurs at lowering entry boundary (Wave S2.1).

---

## 4. NF-H1 Forbidden Constructs

The following node kinds MUST NOT appear in NF-H1:

### 4.1 Surface Control Flow

| Forbidden Node | Reason                                      |
|----------------|---------------------------------------------|
| `ForExpr`      | Implicit iteration — must be explicit       |
| `LoopExpr`     | Infinite loop sugar — must use recursion    |
| `WhileExpr`    | Conditional iteration — must be explicit    |

### 4.2 Surface Pattern Matching

| Forbidden Node | Reason                                      |
|----------------|---------------------------------------------|
| `MatchExpr`    | Surface pattern match — must be desugared   |
| `Pattern`      | Pattern nodes — must be eliminated          |
| `PatternArm`   | Match arms — must be lowered to if/let      |

### 4.3 Implicit Constructs

| Forbidden Node    | Reason                                   |
|-------------------|------------------------------------------|
| `ImplicitReturn`  | Return must be explicit                  |
| `ImplicitSequence`| Sequence must use Block                  |

### 4.4 Surface-Specific Nodes

Any node kind introduced by a surface language and not listed in the admissible set is FORBIDDEN.

---

## 5. Validation Rules

NF validation MUST enforce:

### 5.1 Node Kind Check

For each node in the Schema AST:

1. Check `node.kind` against admissible set
2. If not admissible → HARD ERROR
3. Report:
   - Forbidden node kind
   - Node span (if available)
   - Parent context

### 5.2 Control Flow Explicitness

All control flow MUST be explicit:

* No implicit iteration
* No implicit sequencing beyond Block
* No implicit returns (future)

### 5.3 Structural Validity

* All nodes MUST have well-formed fields
* Field types MUST match schema expectations
* No dangling references

### 5.4 Wrapper Transparency

Wrappers (`Expr`, `AtomicExpr`) are PERMITTED in NF but MUST be transparent:

* Validation MAY skip wrappers
* Lowering MUST strip wrappers before dispatch
* Wrappers MUST NOT carry semantic information

---

## 6. Error Reporting

NF validation errors MUST include:

* **Error type:** "NF Validation Error"
* **Forbidden construct:** exact node kind
* **Location:** span (start..end)
* **Context:** parent node kind (if available)
* **Suggestion:** what should be used instead (if applicable)

Example error message:

```
NF Validation Error at 15..23: forbidden node kind 'ForExpr'
  Context: in Block at 10..50
  Suggestion: use explicit recursion with Lambda and Call
```

---

## 7. Non-Goals

NF-H1 validation does NOT:

* Perform type checking
* Resolve names or bindings
* Access registry
* Perform optimizations
* Modify the AST
* Assign semantics

**NF validation is purely structural enforcement.**

---

## 8. Relationship to Other Specifications

### 8.1 Contracts Document

NF-H1 implements the NF requirements from:

* `core_spec/AXIS_INPUT_SPECIFICATION_CONTRACTS-0.2.md` § 6.3-6.4

### 8.2 Architecture Document

NF-H1 validation appears in the pipeline as shown in:

* `docs/ARCHITECTURE.md` § 2

### 8.3 Normalisation

NF-H1 defines the TARGET of normalisation:

* Normalisation produces NF-H1 compliant AST
* Normalisation rules are NOT defined here
* See `normalisation.yaml` for transformation rules

### 8.4 Lowering

NF-H1 defines the INPUT to lowering:

* Lowering consumes ONLY NF-H1 AST
* Lowering assigns semantics
* Lowering is embedded and fixed

---

## 9. Evolution and Versioning

NF-H1 is versioned independently of:

* Surface languages
* Core IR
* Normalisation rules

Changes to NF-H1 require:

* Major version bump if admissible set changes
* Minor version bump if validation rules are refined
* No version change for documentation clarifications

Current version: **1.0**

---

## 10. Implementation Requirements

Implementations MUST:

* Validate ALL nodes recursively
* Fail HARD on any forbidden construct
* Provide clear error messages with spans
* Make validation observable (e.g., `--inspect nf`)
* Run validation AFTER normalisation, BEFORE lowering

Implementations MUST NOT:

* Skip validation for "trusted" inputs
* Attempt repair or auto-normalization
* Silently pass forbidden constructs
* Introduce semantic interpretation

---

## 11. Verification

NF-H1 compliance can be verified by:

1. **Golden tests:** Known-good NF ASTs pass validation
2. **Negative tests:** Known-bad ASTs fail with correct errors
3. **Pipeline tests:** Normalised output always passes validation
4. **Boundary tests:** Non-normalised ASTs fail validation

Test suites MUST cover:

* All admissible node kinds (positive)
* All forbidden node kinds (negative)
* Wrapper handling (transparency)
* Error message quality

---

## 12. Authoritative Status

This document is **NORMATIVE**.

Implementations MUST conform to this specification.

Any discrepancy between implementation behavior and this document MUST be resolved in favor of this document.

---

**END OF SPECIFICATION**
