# Wave 4: Schema-Driven AST Projection

## Overview

Wave 4 implements a **schema-driven AST projection layer** that transforms the Generic AST (from Wave 3) into a typed, semantic Schema AST using **explicit transformations only**.

## Key Principle

> **Every transformation must be declared in ast_schema.yaml**

No inference. No defaults. No silent semantics.

---

## Architecture

```
Generic AST (Wave 3)  +  AST Schema (YAML)  →  Schema AST (Wave 4)
      ↓                        ↓                       ↓
  Structural            Transformation Rules      Semantic
  Projection            (Explicit Only)           Structure
```

---

## Data Structures

### SchemaAstNode

The output of schema projection - a typed, semantic AST node:

```rust
pub struct SchemaAstNode {
    pub kind: String,                        // Semantic node kind
    pub fields: HashMap<String, SchemaValue>, // Named fields
    pub span: Span,                          // Derived or explicit span
}
```

### SchemaValue

Values stored in schema AST fields:

```rust
pub enum SchemaValue {
    Node(Box<SchemaAstNode>),  // Single child node
    Nodes(Vec<SchemaAstNode>), // Multiple child nodes
    Token(Token),              // Single token
    Tokens(Vec<Token>),        // Multiple tokens
}
```

### AstSchema

The schema definition loaded from YAML:

```rust
pub struct AstSchema {
    pub nodes: HashMap<String, SchemaNodeDef>,
    pub unhandled_behavior: UnhandledNodeBehavior,
}

pub struct SchemaNodeDef {
    pub match_kind: String,                      // Generic AST kind to match
    pub fields: HashMap<String, FieldExtraction>, // Field extraction rules
    pub span_rule: Option<SpanRule>,             // Optional span override
}
```

---

## Schema Definition Language

### YAML Format

```yaml
nodes:
  NodeName:
    match: GenericAstKind
    fields:
      field_name:
        from: <extraction_rule>
```

### Extraction Rules

- **`child(n)`** - Extract child at index n (0-based)
- **`children(start..end)`** - Extract children in range [start, end)
- **`token(TOKEN_KIND)`** - Extract first token of specific kind
- **`tokens(TOKEN_KIND)`** - Extract all tokens of specific kind

### Token Kinds

- `IDENT` - Identifier
- `INT_LIT` - Integer literal
- `BOOL_LIT` - Boolean literal
- `STRING_LIT` - String literal
- `UNIT_LIT` - Unit literal
- `KW_<name>` - Keyword (e.g., `KW_if`, `KW_else`)
- `PUNCT_<char>` - Punctuation (e.g., `PUNCT_+`, `PUNCT_(`)
- `EOF` - End of file

### Example Schema

```yaml
nodes:
  IntLiteral:
    match: Atom
    fields:
      value:
        from: token(INT_LIT)
        
  IfExpr:
    match: If
    fields:
      condition:
        from: child(1)
      then_branch:
        from: child(2)
      else_branch:
        from: child(4)
        
  TupleExpr:
    match: Tuple
    fields:
      items:
        from: children(1..4)
```

---

## API

### Core Function

```rust
pub fn project_schema_ast(
    ast: &ASTNode,
    schema: &AstSchema
) -> Result<SchemaAstNode, SchemaAstError>
```

### Schema Loading

```rust
pub fn load_schema_from_file<P: AsRef<Path>>(path: P) 
    -> Result<AstSchema, SchemaLoadError>

pub fn load_schema_from_string(yaml: &str) 
    -> Result<AstSchema, SchemaLoadError>
```

---

## Usage Example

```rust
use axis_lang_lab_working::frontend::{
    ast_builder::build_generic_ast,
    schema_ast::project_schema_ast,
    schema_load::load_schema_from_file,
};

// 1. Build Generic AST (Wave 3)
let generic_ast = build_generic_ast(&parse_tree)?;

// 2. Load schema
let schema = load_schema_from_file("ast_schema.yaml")?;

// 3. Project to Schema AST
let schema_ast = project_schema_ast(&generic_ast, &schema)?;

// 4. Access semantic fields
use schema_ast::SchemaValue;

match &schema_ast.fields["condition"] {
    SchemaValue::Node(node) => {
        // Use typed condition field
    }
    _ => panic!("unexpected field type"),
}
```

See `examples/wave4_schema_example.rs` for a complete working example.

---

## Error Handling

All projection failures produce `SchemaAstError` with:
- Detailed error message
- Span information (source location)

### Common Errors

```
child(4) out of bounds: node 'If' has only 3 children
token(IntLit) not found in node 'Atom'
unhandled AST node 'While'
children(2..5) invalid: node 'Test' has 3 children
```

Errors are **fatal and explicit** - no silent failures.

---

## Projection Rules

1. ✅ **Schema Lookup**: Generic AST node must have matching schema entry
2. ✅ **Explicit Extraction**: All fields extracted via declared rules only
3. ✅ **Recursive Projection**: Child nodes projected using schema recursively
4. ✅ **Span Derivation**: Span computed from extracted fields or explicit
5. ✅ **Error on Missing**: Missing schema/children/tokens → fatal error
6. ✅ **Deterministic**: Same input always produces same output

---

## Testing

Wave 4 includes comprehensive tests covering:

- ✅ Happy-path projection (token, child, children extraction)
- ✅ Missing child indices (out of bounds errors)
- ✅ Missing tokens (token not found errors)
- ✅ Unhandled nodes (reject vs passthrough modes)
- ✅ Invalid children ranges
- ✅ Span correctness (derived from fields)
- ✅ Schema loading from YAML
- ✅ Determinism verification
- ✅ Edge cases (empty fields)

Run tests:
```bash
cargo test --test wave4_schema_ast
```

All 12 tests pass ✅

---

## What Wave 4 Does

✅ Loads and validates schema from YAML  
✅ Matches Generic AST nodes to schema definitions  
✅ Extracts fields via explicit rules  
✅ Projects child nodes recursively  
✅ Derives spans from extracted fields  
✅ Reports errors explicitly and fatally  
✅ Ensures deterministic transformation  

---

## What Wave 4 Does NOT Do

❌ Infer field meaning  
❌ Auto-flatten lists  
❌ Skip tokens silently  
❌ Rename without schema instruction  
❌ Add semantic defaults  
❌ Modify Generic AST  
❌ Allow implicit transformations  

---

## Design Philosophy

Wave 4 is **intentionally strict**:

- **Explicit over implicit**: Every transformation declared in schema
- **Fail fast**: Missing schema/fields → immediate error
- **Auditable**: All semantic decisions visible in YAML
- **Reviewable**: Schema changes don't require code changes
- **Traceable**: Clear mapping from Generic AST → Schema AST

This makes semantic transformations:
1. Reviewable (read the schema)
2. Modifiable (edit YAML, not code)
3. Auditable (trace every decision)
4. Verifiable (test schema changes independently)

---

## Integration

### Module Location
- `src/frontend/schema_ast.rs` - Core projection engine
- `src/frontend/schema_load.rs` - Schema loading and validation

### Dependencies
- Wave 3: Generic AST (`ast_builder`)
- Wave 2: Parser runtime (transitively)
- Wave 6: Token definitions
- External: `serde_yaml` for YAML parsing

### Exposed API
```rust
pub use axis_lang_lab_working::frontend::{
    schema_ast::{
        project_schema_ast,
        SchemaAstNode,
        SchemaValue,
        AstSchema,
    },
    schema_load::{
        load_schema_from_file,
        load_schema_from_string,
    },
};
```

---

## Files

| File | Purpose | Lines |
|------|---------|-------|
| `src/frontend/schema_ast.rs` | Schema AST types & projection engine | ~420 |
| `src/frontend/schema_load.rs` | Schema YAML loading & validation | ~330 |
| `tests/wave4_schema_ast.rs` | Comprehensive integration tests | ~670 |
| `examples/wave4_schema_example.rs` | Usage demonstration | ~180 |
| `WAVE4_SCHEMA_AST_COMPLETE.md` | Completion documentation | ~550 |

**Total: ~2150 lines**

---

## Next Steps (Wave 5)

Wave 5 will:
- Consume Schema AST
- Lower to Core IR
- Apply type checking
- Perform semantic analysis

Wave 4's explicit schema makes Wave 5's semantic analysis **reviewable and modifiable**.

---

## Completion Criteria ✅

✅ `project_schema_ast()` exists  
✅ All transformations are schema-driven  
✅ No silent semantics exist  
✅ Errors are precise and fatal  
✅ Tests cover all failure modes  
✅ Deterministic output proven  
✅ Schema loading implemented  
✅ Span rules correctly applied  
✅ Unhandled node behavior configurable  

**Wave 4: Schema-Driven AST Projection — COMPLETE** ✅
