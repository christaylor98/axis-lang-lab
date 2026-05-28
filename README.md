# Axis Language Lab

**A deterministic, spec-driven compiler that lowers source code into a small, frozen, canonical intermediate representation — Axis Core IR.**

Axis Language Lab exists to make **program meaning explicit, inspectable, and stable** — independent of surface syntax, runtime, or execution strategy. Source goes in; a canonical Core IR bundle comes out. The same source and specifications always produce byte-identical IR.

Execution is treated as a *projection* of meaning, not its source.

---

## Why this exists

Most systems tangle three things together: the surface language, the meaning of a program, and how it runs. Axis separates them hard.

- A **spec-driven front end** (lexer, parser, schema, normalisation — all YAML-configured) turns surface syntax into a single canonical structural form.
- **Mechanical lowering** (fixed Rust, no configuration) assigns meaning, producing **Axis Core IR**: a closed set of nine node kinds and nothing else.
- **Bridges** project Core IR into executable or target-specific forms. Execution lives here, downstream of meaning.

The payoff of a *small, closed* IR is that meaning becomes **analysable**. Because Core IR has exactly nine node kinds and routes all domain values through opaque foreign calls, you can mechanically reason about the structure of a program — what depends on what, what reaches a decision, what reaches an effect — in ways that are intractable over a general-purpose language. (See [Analysing Core IR](#analysing-core-ir).)

---

## Conceptual layout

```
Surface Language(s)
        │
        ▼
   IR Compiler            (spec-driven: lexer → parser → schema → normalisation)
        │
        ▼
   Axis Core IR  0.3      (closed 9-node set; canonical, frozen contract)
        │
        ▼
     Bridge(s)            (Rust bridge: compile to native; execution as projection)
```

---

## Core IR 0.3

Core IR is a small lambda-calculus IR with a **closed, frozen** set of nine node kinds:

| Node | Purpose |
|------|---------|
| `cIntLit`, `cBoolLit`, `cUnitLit` | The only literals — enough to drive control flow |
| `cVar`, `cLam`, `cApp` | Variable reference, abstraction, application |
| `cLet` | Binding (and the mechanism for sequencing) |
| `cIf` | Conditional branching |
| `cCall` | External call to a foreign function, by bare canonical name |

Two design choices make the IR both tiny and universal:

- **Everything beyond control flow is a foreign value.** Strings, lists, documents, handles — none exist *in* the IR. They are obtained, transformed, and consumed through `cCall` as opaque values the IR routes but never inspects. This is why nine nodes suffice for any domain.
- **Annotations are side-band and non-authoritative.** A bundle must remain valid and execute identically if every annotation is removed. Metadata never affects semantics.

The canonical form is the Cap'n Proto schema in [`core_ir_spec/`](core_ir_spec/). Adding a node kind is a major-version change — the node set is frozen by contract.

---

## Quick start

### Prerequisites

- **Rust** 1.70+
- **Git**

### Build

```bash
git clone https://github.com/christaylor98/axis-lang-lab.git
cd axis-lang-lab
cargo build --release
```

### Emit your first Core IR bundle

```bash
./target/release/axis \
  --lexer     axis-surface-ai2-config/ai2-explicit-lexer.yaml \
  --parser    axis-surface-ai2-config/ai2-explicit-parse.yaml \
  --schema    axis-surface-ai2-config/ai2-explicit-ast.yaml \
  --normalize axis-surface-ai2-config/ai2-explicit-normalize.yaml \
  --src       examples/ai2/ai2-simple-int.ai2 \
  --reg       axis-surface-ai2-config/ai2-explicit-registry.axreg \
  --inspect   core-ir
```

This runs the full pipeline — source → tokens → parse tree → typed AST → Normal Form → Core IR — and prints the emitted IR.

---

## The pipeline

1. **Lexer** — tokenises source per `lexer.yaml`.
2. **Parser** — builds a generic parse tree per `parsing.yaml`.
3. **Schema AST** — projects the parse tree into a typed AST per `ast_schema.yaml`.
4. **Normalisation** — converts surface AST into Normal Form, removing sugar (`normalisation.yaml`). Mandatory and total; makes structure explicit without assigning meaning.
5. **Mechanical lowering** — fixed Rust (`src/lowering/`), no config. Consumes Normal Form only, assigns semantics by 1:1 mapping, resolves registry bindings, emits canonical Core IR. **This is the sole semantic authority.**

Lowering is deterministic and total. Before it, programs have structure but no meaning; after it, meaning is frozen in Core IR. **Language Lab stops at Core IR** — execution is a bridge's job.

---

## Observability

Every stage is inspectable with `--inspect <stage>`, emitting deterministic, machine-readable YAML:

`lexer` · `grammar` · `cst` · `schema` · `ast` · `nf` · `registry` · `core-ir` · `pipeline` · `all`

If a stage shows something unexpected, it is because the spec was written that way — not because the compiler inferred something silently. There is no guessing.

---

## Bridges

Core IR is consumed by **bridges**, which project it into executable form. The reference **Rust bridge** ([`rust-bridge/`](rust-bridge/)) compiles a Core IR bundle into a native library/executable: each `cCall` becomes an `extern "C"` symbol that the linker resolves against shim libraries. Library-first linking; consumers treat call targets as opaque ABI symbols and never load registries.

The same Core IR can be projected by multiple bridges with identical observable behaviour — execution strategy is decoupled from meaning.

---

## Analysing Core IR

A closed nine-node IR is not just executable — it is **statically analysable** in ways a general-purpose language is not. Because control flow is explicit and all domain operations are opaque named calls, you can mechanically trace how values flow through a program.

[`tools/audit/`](tools/audit/) contains a worked example: a **load-bearing dataflow checker** that, given a Core IR bundle, validates its structure and then traces whether the output of any chosen call reaches a branch condition or a consequential effect — distinguishing calls that *drive decisions* from calls that are *side effects*. This kind of decidable structural analysis is exactly what the closed IR enables and what an unrestricted language cannot offer soundly. See [`tools/audit/README.md`](tools/audit/README.md).

---

## Status

**Early, exploratory reference implementation.** Pre-1.0; the model is stable enough to build on but may evolve.

Works today:
- Spec-driven lexer, parser, schema projection
- YAML-driven normalisation (mandatory, total, NF-validated)
- Deterministic mechanical lowering to Core IR 0.3
- Sealed builds with embedded specifications
- Full pipeline observability
- Rust bridge: Core IR → native executable, end to end

Explicitly out of scope: runtime/VM in the lab itself, optimisation, LSP/IDE tooling, multi-file project management, sandboxing, async. Language Lab is a **front end** — it stops at Core IR.

---

## License

MIT. See [LICENSE](LICENSE).
