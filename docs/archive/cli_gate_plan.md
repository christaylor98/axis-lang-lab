
# Axis Language Lab — Gate 1 CLI & Process Flow (Revised)

## Scope Correction (Locked)

* **Language Lab CLI stops at Core IR**
* **Execution is delegated to a bridge**
* **This documentation references the Rust bridge only as an example**
* **End-to-end “compiler” workflows are explicitly future work**

This keeps semantic authority clean and future-proof.

---

## Mental Model for the User (Corrected)

> “Axis Language Lab produces **Core IR**.
> A **bridge** decides what execution means.”

Language Lab:

* parses
* lowers
* validates
* emits Core IR

Bridges:

* consume Core IR
* execute it
* analyze it
* emit artifacts

---

## Revised High-Level Process Flow

```
┌──────────┐
│  Source  │   (.ax)
└────┬─────┘
     │
     ▼
┌──────────┐
│  Parse   │
└────┬─────┘
     │
     ▼
┌──────────┐
│ Lowering │  ← semantic authority
└────┬─────┘
     │
     ▼
┌──────────┐
│ Core IR  │  (.axir / .capnp)
└────┬─────┘
     │
     ├── validate
     ├── inspect
     └── hand off to a bridge
```

**Language Lab ends here.**

---

## CLI Structure (Language Lab Only)

Binary name (still fine):

```
axis
```

But its responsibility is now **explicitly bounded**.

---

## Gate 1 CLI Commands (Corrected)

### 1. `axis parse`

**Purpose:**
Validate surface syntax only.

```bash
axis parse path/to/file.ax
```

* Lexer + parser
* No lowering
* No Core IR
* No bridge involvement

---

### 2. `axis lower`

**Purpose:**
Produce the canonical semantic artifact.

```bash
axis lower path/to/file.ax \
  --out build/main.axir
```

* Parses
* Lowers
* Resolves registry calls
* Emits Core IR
* Fails on semantic errors

**This is the most important Gate 1 command.**

---

### 3. `axis validate`

**Purpose:**
Assert Core IR structural correctness.

```bash
axis validate build/main.axir
```

* Loads Core IR
* Runs validation
* No execution
* Bridge-agnostic

This allows:

* AI-generated IR
* third-party tools
* future surfaces

---

### 4. `axis inspect`

**Purpose:**
Human-readable inspection of Core IR.

```bash
axis inspect build/main.axir
```

* Structural view only
* No execution semantics
* No registry behavior

This may internally reuse inspection code from bridges, but **conceptually belongs to Language Lab**.

---

### 5. `axis build` (Optional Convenience)

**Purpose:**
Human-friendly shortcut.

```bash
axis build path/to/file.ax
```

Equivalent to:

```bash
axis lower file.ax --out build/main.axir
axis validate build/main.axir
```

**Does NOT:**

* execute
* produce binaries
* invoke a bridge

---

## What Language Lab Explicitly Does NOT Do

The CLI **must not**:

* execute Core IR
* bind registry runtime implementations
* emit native executables
* claim to be a full compiler

Any such step is a **bridge responsibility**.

---

## How Execution Is Documented (User-Facing Truth)

In the **user docs**, execution is described like this:

> After producing Core IR with Axis Language Lab, you hand that Core IR to a **bridge**.
> Different bridges define different projections:
>
> * execution
> * analysis
> * native compilation
> * instrumentation

### Rust Bridge (Current Example)

For now, documentation may say:

> The reference bridge under development is the **Rust bridge**, which can:
>
> * load Core IR
> * inspect it
> * execute it via an interpreter (Wave 5)

But this is clearly labeled:

> *This is one bridge, not the definition of execution.*

---

## What Users Can Build *Today*

With this model, a user can already build:

* a language frontend
* a semantic compiler
* a Core IR producer
* a bridge-based execution pipeline (Rust)

What they **cannot yet** build (and we say so explicitly):

* a polished end-to-end compiler binary
* a portable runtime artifact
* a multi-file project system

That honesty is a feature.

---

## How This Sets Up the Future (Without Promising It)

Later (post-Gate 1), you can introduce:

* `axis bridge rust run …`
* `axis bridge wasm emit …`
* `axis bridge <name> …`

But **none of that leaks into Gate 1 docs or CLI contracts**.

---

## What To Do Next (Concrete)

The next **mechanical step** is to:

1. Freeze this CLI contract (names + scope)
2. Implement:

   * `parse`
   * `lower`
   * `validate`
   * `inspect`
3. Update user docs to:

   * explain Language Lab ends at Core IR
   * explain Rust bridge as *example only*

Perfect — here is a **single, macro-wave execution prompt** you can paste into a fresh thread and just *start building*.
This is deliberately **coarse-grained**, execution-biased, and refinement-friendly.

---

## Axis Language Lab — Gate 1 CLI Macro-Wave Plan (Single Prompt)

You are implementing the **Axis Language Lab CLI** to satisfy **Gate 1**.

This CLI is **Language Lab–scoped only**:

* it **ends at Core IR**
* it **does not execute**
* it **does not bind runtimes**
* it **does not emit native artifacts**

Execution is a **bridge concern**.
The Rust bridge is referenced **only as an example**, not as authority.

This plan defines **macro waves**.
Each wave produces something usable before refinement.

---

## Macro Wave A — CLI Skeleton & Contract Lock

**Goal:** Make the CLI *exist* and be structurally correct.

### Deliverables

* Single binary: `axis`
* Subcommand scaffold:

  * `parse`
  * `lower`
  * `validate`
  * `inspect`
  * `build` (alias only)
* Help text that explicitly states:

  * Language Lab ends at Core IR
  * Execution requires a bridge
* Stable command names (no flags bikeshedding yet)

### Rules

* No semantics added
* No new pipeline behavior
* CLI may call existing library APIs directly
* If a command is incomplete, it must fail loudly with “NOT IMPLEMENTED”

**Acceptance**

* `axis --help` tells the truth
* `axis <cmd> --help` exists for every command
* No command implies execution

---

## Macro Wave B — Parse & Lower Wiring (Gate 1 Backbone)

**Goal:** Drive the real semantic pipeline from the CLI.

### Deliverables

* `axis parse file.ax`

  * runs lexer + parser
  * reports syntax errors
* `axis lower file.ax --out file.axir`

  * parses
  * lowers
  * resolves registry calls
  * emits Core IR (Cap’n Proto)
* Proper exit codes:

  * parse errors ≠ lowering errors

### Rules

* Lowering remains the sole semantic authority
* No CLI-level interpretation
* No special cases for PoC examples

**Acceptance**

* PoC examples can be lowered via CLI
* Emitted Core IR matches library output used in tests

---

## Macro Wave C — Core IR Validation & Inspection

**Goal:** Make Core IR a first-class user artifact.

### Deliverables

* `axis validate file.axir`

  * loads Core IR
  * runs validation
  * prints structural errors
* `axis inspect file.axir`

  * prints a structural, human-readable view
  * no execution semantics
  * no registry behavior

### Rules

* Bridge-agnostic
* Inspection is structural only
* No new IR data introduced

**Acceptance**

* Users can validate IR from *any* source (including AI)
* Inspection output is stable and readable

---

## Macro Wave D — Convenience & Flow Polishing (Still Gate 1)

**Goal:** Make the CLI usable without lying.

### Deliverables

* `axis build file.ax`

  * equivalent to:

    * `axis lower`
    * `axis validate`
* Basic filesystem hygiene:

  * create output dirs if missing
  * refuse to overwrite without flag (or document behavior)
* Clear error messages pointing to:

  * parse vs lower vs validate

### Rules

* `build` is a shortcut, not a new phase
* Still no execution
* Still no bridge invocation

**Acceptance**

* A user can go from `.ax` → `.axir` with one command
* CLI feels intentional, not accidental

---

## Macro Wave E — Documentation Alignment (Lock It In)

**Goal:** Ensure users understand what they’re holding.

### Deliverables

* Update / add:

  * `docs/user_workflow.md`
* Explicit statements:

  * “Language Lab ends at Core IR”
  * “Execution requires a bridge”
  * “Rust bridge is an example only”
* Concrete examples using the new CLI

### Rules

* No forward promises
* No mention of future linking, multi-file projects, or native binaries
* No hiding awkward truths

**Acceptance**

* A new user can:

  * run the CLI
  * understand outputs
  * know exactly what is *not* supported yet

---

## Macro Wave F — Gate 1 Closure

**Goal:** Declare Gate 1 *technically satisfied*.

### Deliverables

* PoC examples run through:

  * `axis lower`
  * `axis validate`
* Core IR equivalence confirmed
* No changes to frozen waves
* CLI contract considered **stable for Gate 1**

### Rules

* Stop when Gate 1 is satisfied
* No refactors “while we’re here”
* Refinement comes later

---

## Non-Negotiable Constraints (Global)

* No execution in Language Lab
* No runtime binding
* No native artifacts
* No multi-file semantics
* No imports/modules
* No registry behavior beyond resolution

Anything violating this is **out of scope**.

---

## Final Instruction

Implement this **top-down**:

* skeleton → wire → validate → polish → stop

Do not perfect.
Do not over-design.
Do not anticipate Gate 2+.

When this plan completes, **Axis is usable as a language frontend producing canonical Core IR**, which is exactly what you need to move forward.

---

If you want, next we can:

* map each macro wave to **exact Rust modules**
* or collapse this into a **GitHub milestone / task list**
* or start directly with **Macro Wave A (CLI skeleton)**

Just say where you want to begin.


