// NF Lowering — Universal Mechanical Lowering for Normal Form AST
//
// PURPOSE:
// This module implements FIXED, EMBEDDED lowering for Normal Form (NF) AST.
// It consumes ONLY NF-compliant Schema AST and emits Core IR 0.2.
//
// ARCHITECTURAL INVARIANTS:
// - Lowering is FIXED and EMBEDDED (not user-configurable)
// - Lowering is VERSIONED with the compiler
// - Lowering is the SOLE SEMANTIC AUTHORITY
// - No surface constructs may reach lowering
// - Lowering is DETERMINISTIC and TOTAL over NF
// - Lowering follows NORMAL_FORM_SPEC_0.1.yaml EXACTLY
//
// NF ADMISSIBLE NODES (from core_spec/NORMAL_FORM_SPEC_0.1.yaml):
// - IntLit, BoolLit, UnitLit
// - VarRef
// - LetExpr
// - LamExpr, AppExpr
// - IfExpr
// - CallExpr (registry function calls)
// - Expr (transparent wrapper)
// - Program (transparent wrapper)
//
// FORBIDDEN:
// - Any node kind not listed in NF spec
// - H1-specific nodes (FunctionDecl, Block, etc.)
// - Surface-specific constructs
//
// CORE IR 0.2 NODE SET (from core_ir_spec/axis-core-ir-0.2.md):
// - CIntLit, CBoolLit, CUnitLit
// - CLam, CLet, CIf
// - CVar, CApp, CCall

use crate::frontend::schema_ast::{SchemaAstNode, SchemaValue};
use crate::frontend::token::Span;
use crate::ir::core_ir::{CoreBundle, CoreTerm, IdentOrName};
use crate::registry::Registry;
use std::collections::HashMap;
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// CONTEXT AND ERRORS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct LoweringContext {
    pub registry: Registry,
    pub scope: Scope,
}

#[derive(Debug)]
pub struct Scope {
    bindings: Vec<HashMap<String, usize>>, // Stack of scopes
}

impl Scope {
    pub fn new() -> Self {
        Self {
            bindings: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.bindings.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        if self.bindings.len() > 1 {
            self.bindings.pop();
        }
    }

    pub fn bind(&mut self, name: String) {
        if let Some(scope) = self.bindings.last_mut() {
            let id = scope.len();
            scope.insert(name, id);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<usize> {
        for scope in self.bindings.iter().rev() {
            if let Some(&id) = scope.get(name) {
                return Some(id);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct LoweringError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for LoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NF Lowering error at {}..{}: {}",
            self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for LoweringError {}

// ═══════════════════════════════════════════════════════════════════════════
// MAIN LOWERING ENTRY POINTS
// ═══════════════════════════════════════════════════════════════════════════

/// Lower NF Schema AST to Core IR
///
/// PRECONDITIONS:
/// - node is NF-compliant (validated before lowering)
/// - no surface constructs present
///
/// POSTCONDITIONS:
/// - emits valid Core IR 0.2
/// - deterministic (same input → same output)
/// - total (all admitted nodes have rules)
pub fn lower_to_core_ir(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    // Dispatch based on node kind
    // NO WILDCARDS - total match over NF admissible set
    match node.kind.as_str() {
        // Literals (NF spec section: LITERALS)
        "IntLit" => lower_int_lit(node),
        "BoolLit" => lower_bool_lit(node),
        "UnitLit" => lower_unit_lit(node),

        // Variables (NF spec section: VARIABLES AND BINDING)
        "VarRef" => lower_var_ref(node),
        "LetExpr" => lower_let_expr(node, ctx),

        // Functions and Application (NF spec section: FUNCTIONS AND APPLICATION)
        "LamExpr" => lower_lam_expr(node, ctx),
        "AppExpr" => lower_app_expr(node, ctx),

        // Control Flow (NF spec section: CONTROL FLOW)
        "IfExpr" => lower_if_expr(node, ctx),

        // Registry Function Calls (NF spec section: REGISTRY FUNCTION CALLS)
        "CallExpr" => lower_call_expr(node, ctx),

        // Wrappers (NF spec section: STRUCTURAL WRAPPERS - transparent)
        "Expr" => {
            // Transparent: unwrap and recurse
            extract_field_node(node, "inner")
                .and_then(|child| lower_to_core_ir(child, ctx))
        }
        "Program" => {
            // Transparent: unwrap and recurse
            extract_field_node(node, "body")
                .and_then(|child| lower_to_core_ir(child, ctx))
        }

        // BOUNDARY VIOLATION: Non-NF node reaching lowering
        unknown => {
            Err(LoweringError {
                message: format!(
                    "BOUNDARY VIOLATION: node kind '{}' is not in NF admissible set (see NORMAL_FORM_SPEC_0.1.yaml)",
                    unknown
                ),
                span: node.span.clone(),
            })
        }
    }
}

/// Lower to Core IR bundle (convenience wrapper that takes NfAst)
pub fn lower_nf_to_bundle(
    node: &SchemaAstNode,
    registry: Registry,
) -> Result<CoreBundle, LoweringError> {
    let mut ctx = LoweringContext {
        registry,
        scope: Scope::new(),
    };

    let core_term = lower_to_core_ir(node, &mut ctx)?;
    Ok(crate::ir::core_ir::bundle_v0_2(core_term))
}

// ═══════════════════════════════════════════════════════════════════════════
// NODE LOWERING FUNCTIONS (NF SPEC COMPLIANT)
// ═══════════════════════════════════════════════════════════════════════════

/// Lower IntLit node
///
/// NF Spec:
///   IntLit:
///     required_fields: [value]
///     lowering_target: CIntLit
///
/// Schema fields:
/// - value: token (integer value)
fn lower_int_lit(node: &SchemaAstNode) -> Result<CoreTerm, LoweringError> {
    let value_token = extract_field_token(node, "value")?;
    let value = value_token.parse::<i64>().map_err(|e| LoweringError {
        message: format!("invalid integer literal '{}': {}", value_token, e),
        span: node.span.clone(),
    })?;
    Ok(crate::ir::core_ir::int_lit(value))
}

/// Lower BoolLit node
///
/// NF Spec:
///   BoolLit:
///     required_fields: [value]
///     lowering_target: CBoolLit
///
/// Schema fields:
/// - value: token ('true' or 'false')
fn lower_bool_lit(node: &SchemaAstNode) -> Result<CoreTerm, LoweringError> {
    let value_token = extract_field_token(node, "value")?;
    let value = match value_token.as_str() {
        "true" => true,
        "false" => false,
        other => {
            return Err(LoweringError {
                message: format!(
                    "invalid boolean literal '{}' (expected 'true' or 'false')",
                    other
                ),
                span: node.span.clone(),
            })
        }
    };
    Ok(crate::ir::core_ir::bool_lit(value))
}

/// Lower UnitLit node
///
/// NF Spec:
///   UnitLit:
///     required_fields: []
///     lowering_target: CUnitLit
fn lower_unit_lit(_node: &SchemaAstNode) -> Result<CoreTerm, LoweringError> {
    Ok(crate::ir::core_ir::unit_lit())
}

/// Lower VarRef node
///
/// NF Spec:
///   VarRef:
///     required_fields: [name]
///     lowering_target: CVar
///
/// Schema fields:
/// - name: token (identifier)
fn lower_var_ref(node: &SchemaAstNode) -> Result<CoreTerm, LoweringError> {
    let name = extract_field_token(node, "name")?;
    
    // ASSERT: variable name must not be the literal field label
    assert_ne!(name, "name", "BUG: VarRef name is literal 'name' - field extraction is wrong. Node: {:?}", node);
    assert!(!name.is_empty(), "BUG: VarRef name is empty");
    
    Ok(crate::ir::core_ir::cvar(IdentOrName::new(name)))
}

/// Lower LetExpr node
///
/// NF Spec:
///   LetExpr:
///     required_fields: [name, value, body]
///     lowering_target: CLet
///
/// Schema fields:
/// - name: token (identifier)
/// - value: NF expression node
/// - body: NF expression node
fn lower_let_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let name = extract_field_token(node, "name")?;
    
    // ASSERT: binder must not be the literal string "name"
    assert_ne!(name, "name", "BUG: LetExpr binder is literal 'name' - field extraction is wrong. Node: {:?}", node);
    assert!(!name.is_empty(), "BUG: LetExpr binder is empty");
    
    let value_node = extract_field_node(node, "value")?;
    let body_node = extract_field_node(node, "body")?;

    // Lower value
    let value_term = lower_to_core_ir(value_node, ctx)?;

    // Lower body in extended scope
    ctx.scope.push();
    ctx.scope.bind(name.clone());
    let body_term = lower_to_core_ir(body_node, ctx)?;
    ctx.scope.pop();

    Ok(crate::ir::core_ir::clet(
        IdentOrName::new(name),
        value_term,
        body_term,
    ))
}

/// Lower LamExpr node
///
/// NF Spec:
///   LamExpr:
///     required_fields: [param, body]
///     lowering_target: CLam
///
/// Schema fields:
/// - param: token (identifier)
/// - body: NF expression node
fn lower_lam_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let param = extract_field_token(node, "param")?;
    
    // ASSERT: parameter must not be the literal field label
    assert_ne!(param, "param", "BUG: LamExpr parameter is literal 'param' - field extraction is wrong. Node: {:?}", node);
    assert!(!param.is_empty(), "BUG: LamExpr parameter is empty");
    
    let body_node = extract_field_node(node, "body")?;

    ctx.scope.push();
    ctx.scope.bind(param.clone());
    let body_term = lower_to_core_ir(body_node, ctx)?;
    ctx.scope.pop();

    Ok(crate::ir::core_ir::lam(IdentOrName::new(param), body_term))
}

/// Lower AppExpr node
///
/// NF Spec:
///   AppExpr:
///     required_fields: [fn, arg]
///     lowering_target: CApp
///
/// Schema fields:
/// - fn: NF expression node (function)
/// - arg: NF expression node (argument)
fn lower_app_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let fn_node = extract_field_node(node, "fn")?;
    let arg_node = extract_field_node(node, "arg")?;

    let fn_term = lower_to_core_ir(fn_node, ctx)?;
    let arg_term = lower_to_core_ir(arg_node, ctx)?;

    Ok(crate::ir::core_ir::capp(fn_term, arg_term))
}

/// Lower IfExpr node
///
/// NF Spec:
///   IfExpr:
///     required_fields: [condition, then_branch, else_branch]
///     lowering_target: CIf
///
/// Schema fields:
/// - condition: NF expression node
/// - then_branch: NF expression node
/// - else_branch: NF expression node (REQUIRED)
fn lower_if_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let cond_node = extract_field_node(node, "condition")?;
    let then_node = extract_field_node(node, "then_branch")?;
    let else_node = extract_field_node(node, "else_branch")?;

    let cond_term = lower_to_core_ir(cond_node, ctx)?;
    let then_term = lower_to_core_ir(then_node, ctx)?;
    let else_term = lower_to_core_ir(else_node, ctx)?;

    Ok(crate::ir::core_ir::cif(cond_term, then_term, else_term))
}

/// Lower CallExpr node
///
/// NF Spec:
///   CallExpr:
///     required_fields: [name, args]
///     lowering_target: CCall
///
/// Schema fields:
/// - name: token (function name)
/// - args: list of NF expression nodes
fn lower_call_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let name = extract_field_token(node, "name")?;
    let arg_nodes = extract_field_nodes(node, "args")?;

    // Lookup registry function
    let entry = ctx
        .registry
        .find_by_name(&name)
        .ok_or_else(|| LoweringError {
            message: format!("registry function '{}' not found", name),
            span: node.span.clone(),
        })?;

    // Validate arity
    if arg_nodes.len() != entry.arity {
        return Err(LoweringError {
            message: format!(
                "arity mismatch for '{}': expected {} args, got {}",
                name,
                entry.arity,
                arg_nodes.len()
            ),
            span: node.span.clone(),
        });
    }

    // Core IR 0.3: use canonical function name instead of numeric ID
    let canonical_name = entry.name.clone();

    // Lower arguments
    let mut args = Vec::new();
    for arg_node in arg_nodes {
        args.push(lower_to_core_ir(arg_node, ctx)?);
    }

    Ok(crate::ir::core_ir::ccall(canonical_name, args))
}

// ═══════════════════════════════════════════════════════════════════════════
// FIELD EXTRACTION UTILITIES
// ═══════════════════════════════════════════════════════════════════════════

fn extract_field_node<'a>(
    node: &'a SchemaAstNode,
    field_name: &str,
) -> Result<&'a SchemaAstNode, LoweringError> {
    node.fields
        .get(field_name)
        .and_then(|v| match v {
            SchemaValue::Node(n) => Some(n.as_ref()),
            _ => None,
        })
        .ok_or_else(|| LoweringError {
            message: format!("missing field '{}' in node '{}'", field_name, node.kind),
            span: node.span.clone(),
        })
}

fn extract_field_nodes<'a>(
    node: &'a SchemaAstNode,
    field_name: &str,
) -> Result<Vec<&'a SchemaAstNode>, LoweringError> {
    node.fields
        .get(field_name)
        .and_then(|v| match v {
            SchemaValue::Nodes(ns) => Some(ns.iter().collect()),
            _ => None,
        })
        .ok_or_else(|| LoweringError {
            message: format!("missing field '{}' in node '{}'", field_name, node.kind),
            span: node.span.clone(),
        })
}

fn extract_field_token(node: &SchemaAstNode, field_name: &str) -> Result<String, LoweringError> {
    node.fields
        .get(field_name)
        .and_then(|v| match v {
            SchemaValue::Token(t) => Some(t.lexeme.clone()),
            _ => None,
        })
        .ok_or_else(|| LoweringError {
            message: format!(
                "missing token field '{}' in node '{}'",
                field_name, node.kind
            ),
            span: node.span.clone(),
        })
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::token::{Token, TokenKind};
    use crate::registry::RegistryEntry;

    fn make_test_registry() -> Registry {
        Registry {
            entries: vec![RegistryEntry {
                id: 1,
                name: "test_fn".to_string(),
                arity: 1,
                deterministic: true,
                profiles: vec!["test".to_string()],
            }],
        }
    }

    fn make_token(lexeme: &str) -> Token {
        Token {
            kind: TokenKind::Ident,
            lexeme: lexeme.to_string(),
            span: Span::new(0, lexeme.len()),
        }
    }

    #[test]
    fn test_lower_unit_lit() {
        let node = SchemaAstNode {
            kind: "UnitLit".to_string(),
            fields: HashMap::new(),
            annotations: Default::default(),
            span: Span::new(0, 2),
        };
        let result = lower_unit_lit(&node);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), CoreTerm::CUnitLit { .. }));
    }

    #[test]
    fn test_lower_int_lit() {
        let mut fields = HashMap::new();
        fields.insert("value".to_string(), SchemaValue::Token(make_token("42")));

        let node = SchemaAstNode {
            kind: "IntLit".to_string(),
            fields,
            annotations: Default::default(),
            span: Span::new(0, 2),
        };

        let result = lower_int_lit(&node);
        assert!(result.is_ok());
    }

    #[test]
    fn test_boundary_violation() {
        let node = SchemaAstNode {
            kind: "InvalidNode".to_string(),
            fields: HashMap::new(),
            annotations: Default::default(),
            span: Span::new(0, 10),
        };

        let mut ctx = LoweringContext {
            registry: make_test_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("BOUNDARY VIOLATION"));
    }
}
