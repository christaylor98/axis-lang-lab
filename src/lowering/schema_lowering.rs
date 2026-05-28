// Schema AST → Core IR Lowering (Semantic Authority Boundary)
//
// This module implements the lowering layer that:
// - Consumes Schema AST (from schema builder)
// - Produces Core IR
// - Treats Schema AST as the sole semantic authority
// - Performs NO parsing, NO schema inference, NO execution
//
// FORBIDDEN:
// - Parsing tokens or lexemes
// - Inspecting Generic AST
// - Inventing semantics
// - Executing or interpreting
// - Compensating for missing schema data
// - Inferring field meanings
// - Adding defaults
//
// If semantics are missing or invalid: FAIL FAST

use crate::frontend::schema_ast::{SchemaAstNode, SchemaValue};
use crate::frontend::token::Span;
use crate::ir::core_ir::{CoreBundle, CoreTerm, IdentOrName};
use crate::registry::Registry;
use std::collections::HashMap;
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════

/// Context for lowering operations
pub struct LoweringContext {
    /// Registry for function lookups (read-only)
    pub registry: Registry,
    /// Scope for variable bindings
    pub scope: Scope,
}

/// Variable scope for tracking bindings
#[derive(Debug, Clone)]
pub struct Scope {
    /// Stack of scopes (inner to outer)
    bindings: Vec<HashMap<String, ScopeBinding>>,
}

/// A binding in the scope
#[derive(Debug, Clone)]
struct ScopeBinding {
    /// The identifier name
    _name: String,
    /// Span where bound
    _span: Span,
}

impl Scope {
    /// Create a new empty scope
    pub fn new() -> Self {
        Scope {
            bindings: vec![HashMap::new()],
        }
    }

    /// Push a new scope level
    pub fn push_scope(&mut self) {
        self.bindings.push(HashMap::new());
    }

    /// Pop the current scope level
    pub fn pop_scope(&mut self) {
        if self.bindings.len() > 1 {
            self.bindings.pop();
        }
    }

    /// Bind a variable in the current scope
    pub fn bind(&mut self, name: String, span: Span) {
        if let Some(current) = self.bindings.last_mut() {
            current.insert(
                name.clone(),
                ScopeBinding {
                    _name: name,
                    _span: span,
                },
            );
        }
    }

    /// Look up a variable (searches from inner to outer)
    pub fn lookup(&self, name: &str) -> bool {
        for scope in self.bindings.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }
}

/// Lowering error - all lowering failures
#[derive(Debug, Clone)]
pub struct LoweringError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for LoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lowering error at {}..{}: {}",
            self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for LoweringError {}

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════

/// Normalize a Schema AST node by eliminating wrapper nodes
///
/// WAVE S2.1: Pre-lowering normalization enforces the semantic boundary.
///
/// Wrapper nodes (Expr, AtomicExpr) are structural artifacts from parsing
/// and have no semantic content. They must be eliminated BEFORE lowering.
///
/// This function recursively unwraps:
/// - "Expr" nodes → inner node
/// - "AtomicExpr" nodes → inner node
///
/// All other nodes are preserved as-is but have their fields normalized.
///
/// Returns a normalized node where:
/// - No Expr or AtomicExpr nodes remain at any level
/// - All semantic nodes are preserved with their fields normalized
///
/// This is deterministic and total (no failures).
fn normalize_wrappers(node: &SchemaAstNode) -> SchemaAstNode {
    match node.kind.as_str() {
        "Expr" | "AtomicExpr" => {
            // Unwrap: extract inner node and continue normalization
            match node.fields.get("inner") {
                Some(SchemaValue::Node(inner)) => normalize_wrappers(inner),
                // If inner is missing or wrong type, preserve node as-is
                // (lowering will fail with proper error later)
                _ => node.clone(),
            }
        }
        _ => {
            // Semantic node: normalize all fields recursively
            let mut normalized_fields = HashMap::new();
            for (field_name, field_value) in &node.fields {
                let normalized_value = match field_value {
                    SchemaValue::Node(child) => {
                        SchemaValue::Node(Box::new(normalize_wrappers(child)))
                    }
                    SchemaValue::Nodes(children) => {
                        SchemaValue::Nodes(children.iter().map(|c| normalize_wrappers(c)).collect())
                    }
                    // Tokens don't contain nodes, pass through unchanged
                    SchemaValue::Token(t) => SchemaValue::Token(t.clone()),
                    SchemaValue::Tokens(ts) => SchemaValue::Tokens(ts.clone()),
                };
                normalized_fields.insert(field_name.clone(), normalized_value);
            }

            SchemaAstNode {
                kind: node.kind.clone(),
                fields: normalized_fields,
                annotations: node.annotations.clone(),
                span: node.span.clone(),
            }
        }
    }
}

/// Lower a Schema AST node to Core IR
///
/// This is the primary lowering entry point.
///
/// WAVE S2.1 BOUNDARY ENFORCEMENT:
/// - Wrapper nodes (Expr, AtomicExpr) are normalized away BEFORE dispatch
/// - Lowering dispatch operates ONLY on semantic nodes
/// - Receiving a wrapper node at dispatch is a HARD ERROR (boundary violation)
///
/// Lowering is:
/// - Node-driven: dispatched by SchemaAstNode.kind
/// - Field-driven: semantics come from named fields only
/// - Strict: missing fields or wrong types = error
/// - Deterministic: identical input → identical output
///
/// Returns:
/// - Ok(CoreTerm) on success
/// - Err(LoweringError) on any semantic or structural issue
///
/// WAVE S2: Semantic-Surface-0 lowering (mechanical, total, no wildcards)
pub fn lower_to_core_ir(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    // WAVE S2.1: Normalize wrappers before lowering
    let normalized = normalize_wrappers(node);

    // Dispatch based on node kind - TOTAL MATCH (no wildcards)
    // BOUNDARY INVARIANT: Wrapper nodes must not reach here
    match normalized.kind.as_str() {
        // Semantic-Surface-0 literal nodes
        "IntLit" => lower_int_lit(&normalized),
        "BoolLit" => lower_bool_lit(&normalized),
        "UnitLit" => lower_unit_lit(&normalized),

        // Semantic-Surface-0 variable reference
        "VarRef" => lower_var_ref(&normalized, ctx),

        // Semantic-Surface-0 binding and abstraction
        "LetExpr" => lower_let_expr(&normalized, ctx),
        "LamExpr" => lower_lam_expr(&normalized, ctx),

        // Semantic-Surface-0 control flow
        "IfExpr" => lower_if_expr(&normalized, ctx),

        // Semantic-Surface-0 application
        "AppExpr" => lower_app_expr(&normalized, ctx),

        // WAVE S2.1 BOUNDARY VIOLATION: Wrappers reaching dispatch
        "AtomicExpr" | "Expr" => Err(LoweringError {
            message: format!(
                "BOUNDARY VIOLATION: wrapper node '{}' reached lowering dispatch (should have been normalized)",
                normalized.kind
            ),
            span: normalized.span.clone(),
        }),

        // Legacy Surface-0 nodes (deprecated, kept for transition)
        "Lam" => lower_lam(&normalized, ctx),
        "If" => lower_if(&normalized, ctx),
        "Call" => lower_call(&normalized, ctx),
        "Ident" => lower_ident(&normalized, ctx),

        // EXPLICIT REJECTION - no wildcards
        unknown => Err(LoweringError {
            message: format!(
                "unknown schema node kind '{}' - not a valid Semantic-Surface-0 or Surface-0 node",
                unknown
            ),
            span: normalized.span.clone(),
        }),
    }
}

/// Lower a complete Schema AST to a Core IR bundle
///
/// This wraps the result in a CoreBundle for serialization.
pub fn lower_to_bundle(
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
// SEMANTIC-SURFACE-0 LOWERING FUNCTIONS (WAVE S2)
// ═══════════════════════════════════════════════════════════════════════════

/// Lower IntLit node (Semantic-Surface-0)
///
/// Required fields:
/// - "value": Token (INT_LIT)
///
/// Produces: CIntLit
fn lower_int_lit(node: &SchemaAstNode) -> Result<CoreTerm, LoweringError> {
    let value_token = get_field_token(node, "value")?;

    // Parse integer from token lexeme
    let value: i64 = value_token.lexeme.parse().map_err(|_| LoweringError {
        message: format!("invalid integer literal '{}'", value_token.lexeme),
        span: value_token.span.clone(),
    })?;

    Ok(CoreTerm::CIntLit {
        value,
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

/// Lower BoolLit node (Semantic-Surface-0)
///
/// Required fields:
/// - "value": Token (BOOL_LIT - "true" or "false")
///
/// Produces: CBoolLit
fn lower_bool_lit(node: &SchemaAstNode) -> Result<CoreTerm, LoweringError> {
    let value_token = get_field_token(node, "value")?;

    // Parse boolean from token lexeme
    let value = match value_token.lexeme.as_str() {
        "true" => true,
        "false" => false,
        other => {
            return Err(LoweringError {
                message: format!(
                    "invalid boolean literal '{}', expected 'true' or 'false'",
                    other
                ),
                span: value_token.span.clone(),
            })
        }
    };

    Ok(CoreTerm::CBoolLit {
        value,
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

/// Lower VarRef node (Semantic-Surface-0)
///
/// Required fields:
/// - "name": Token (IDENT)
///
/// Produces: CVar
fn lower_var_ref(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let name_token = get_field_token(node, "name")?;
    let name = &name_token.lexeme;

    // Check if bound in scope
    if !ctx.scope.lookup(name) {
        return Err(LoweringError {
            message: format!("unbound variable '{}'", name),
            span: name_token.span.clone(),
        });
    }

    Ok(CoreTerm::CVar {
        name: IdentOrName::new(name.clone()),
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

/// Lower LetExpr node (Semantic-Surface-0)
///
/// Required fields:
/// - "name": Token (IDENT)
/// - "value": Node (expression to bind)
/// - "body": Node (expression in extended scope)
///
/// Produces: CLet
fn lower_let_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    // Extract name field
    let name_token = get_field_token(node, "name")?;
    let name = name_token.lexeme.clone();

    // Extract value field
    let value_node = get_field_node(node, "value")?;

    // Lower value expression (in current scope)
    let value_term = lower_to_core_ir(value_node, ctx)?;

    // Push new scope for body
    ctx.scope.push_scope();
    ctx.scope.bind(name.clone(), name_token.span.clone());

    // Extract and lower body field
    let body_node = get_field_node(node, "body")?;
    let body_term = lower_to_core_ir(body_node, ctx)?;

    // Pop scope
    ctx.scope.pop_scope();

    Ok(CoreTerm::CLet {
        name: IdentOrName::new(name),
        value: Box::new(value_term),
        body: Box::new(body_term),
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

/// Lower LamExpr node (Semantic-Surface-0)
///
/// Required fields:
/// - "param": Token (IDENT)
/// - "body": Node (expression)
///
/// Produces: CLam
fn lower_lam_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    // Extract param field
    let param_token = get_field_token(node, "param")?;
    let param_name = param_token.lexeme.clone();

    // Extract body field
    let body_node = get_field_node(node, "body")?;

    // Push new scope for lambda body
    ctx.scope.push_scope();
    ctx.scope.bind(param_name.clone(), param_token.span.clone());

    // Lower body
    let body_term = lower_to_core_ir(body_node, ctx)?;

    // Pop scope
    ctx.scope.pop_scope();

    Ok(CoreTerm::CLam {
        param: IdentOrName::new(param_name),
        body: Box::new(body_term),
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

/// Lower IfExpr node (Semantic-Surface-0)
///
/// Required fields:
/// - "condition": Node (expression)
/// - "then_branch": Node (expression)
/// - "else_branch": Node (expression)
///
/// Produces: CIf
fn lower_if_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    // Extract condition field
    let cond_node = get_field_node(node, "condition")?;
    let cond_term = lower_to_core_ir(cond_node, ctx)?;

    // Extract then_branch field
    let then_node = get_field_node(node, "then_branch")?;
    let then_term = lower_to_core_ir(then_node, ctx)?;

    // Extract else_branch field
    let else_node = get_field_node(node, "else_branch")?;
    let else_term = lower_to_core_ir(else_node, ctx)?;

    Ok(CoreTerm::CIf {
        cond: Box::new(cond_term),
        then_branch: Box::new(then_term),
        else_branch: Box::new(else_term),
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

/// Lower AppExpr node (Semantic-Surface-0)
///
/// Required fields:
/// - "parts": Nodes (list of nodes - first is function, rest are arguments)
///
/// Produces: CApp (nested left-to-right for multiple arguments)
///
/// Example: AppExpr with parts=[f, x, y] lowers to CApp(CApp(f, x), y)
fn lower_app_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    // Extract parts field
    let parts = get_field_nodes(node, "parts")?;

    if parts.is_empty() {
        return Err(LoweringError {
            message: "AppExpr requires at least one part (the function)".to_string(),
            span: node.span.clone(),
        });
    }

    // Lower first part (the function base)
    let mut result = lower_to_core_ir(&parts[0], ctx)?;

    // Apply each subsequent part as an argument (left-to-right curried application)
    for arg_node in &parts[1..] {
        let arg_term = lower_to_core_ir(arg_node, ctx)?;
        result = CoreTerm::CApp {
            func: Box::new(result),
            arg: Box::new(arg_term),
            node_id: None,
            annotations: Vec::new(), // Applications inherit no annotations from AppExpr node
        };
    }

    Ok(result)
}

// ═══════════════════════════════════════════════════════════════════════════
// LEGACY SURFACE-0 LOWERING FUNCTIONS (DEPRECATED)
// ═══════════════════════════════════════════════════════════════════════════

/// Lower UnitLit node
///
/// Required fields: none
/// Produces: CUnitLit
fn lower_unit_lit(node: &SchemaAstNode) -> Result<CoreTerm, LoweringError> {
    // UnitLit has no fields, just emit with annotations
    Ok(CoreTerm::CUnitLit {
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

/// Lower Lam (lambda) node
///
/// Required fields:
/// - "param": Token (identifier)
/// - "body": Node (expression)
///
/// Produces: CLam
fn lower_lam(node: &SchemaAstNode, ctx: &mut LoweringContext) -> Result<CoreTerm, LoweringError> {
    // Extract param field
    let param_token = get_field_token(node, "param")?;
    let param_name = param_token.lexeme.clone();

    // Extract body field
    let body_node = get_field_node(node, "body")?;

    // Push new scope for lambda body
    ctx.scope.push_scope();
    ctx.scope.bind(param_name.clone(), param_token.span.clone());

    // Lower body
    let body_term = lower_to_core_ir(body_node, ctx)?;

    // Pop scope
    ctx.scope.pop_scope();

    Ok(CoreTerm::CLam {
        param: IdentOrName::new(param_name),
        body: Box::new(body_term),
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

/// Lower If (conditional) node
///
/// Required fields:
/// - "condition": Node (expression)
/// - "then_branch": Node (expression)
/// - "else_branch": Node (expression)
///
/// Produces: CIf
fn lower_if(node: &SchemaAstNode, ctx: &mut LoweringContext) -> Result<CoreTerm, LoweringError> {
    // Extract condition field
    let cond_node = get_field_node(node, "condition")?;
    let cond_term = lower_to_core_ir(cond_node, ctx)?;

    // Extract then_branch field
    let then_node = get_field_node(node, "then_branch")?;
    let then_term = lower_to_core_ir(then_node, ctx)?;

    // Extract else_branch field
    let else_node = get_field_node(node, "else_branch")?;
    let else_term = lower_to_core_ir(else_node, ctx)?;

    Ok(CoreTerm::CIf {
        cond: Box::new(cond_term),
        then_branch: Box::new(then_term),
        else_branch: Box::new(else_term),
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

/// Lower Call (function call) node
///
/// Required fields:
/// - "target": Token (identifier) or explicit target
/// - "args": Nodes (list of arguments)
///
/// Produces: CCall
fn lower_call(node: &SchemaAstNode, ctx: &mut LoweringContext) -> Result<CoreTerm, LoweringError> {
    // Extract target field (function name)
    let target_token = get_field_token(node, "target")?;
    let target_name = &target_token.lexeme;

    // Look up target in registry (clone necessary data to avoid borrow issues)
    let (canonical_target_name, expected_arity) = {
        let registry_entry = ctx
            .registry
            .entries
            .iter()
            .find(|e| &e.name == target_name)
            .ok_or_else(|| LoweringError {
                message: format!("unknown function '{}' in registry", target_name),
                span: target_token.span.clone(),
            })?;

        (registry_entry.name.clone(), registry_entry.arity)
    };

    // Extract args field
    let arg_nodes = get_field_nodes(node, "args")?;

    // Lower each argument
    let mut arg_terms = Vec::new();
    for arg_node in arg_nodes {
        let arg_term = lower_to_core_ir(arg_node, ctx)?;
        arg_terms.push(arg_term);
    }

    // Verify arity matches
    if arg_terms.len() != expected_arity {
        return Err(LoweringError {
            message: format!(
                "function '{}' expects {} arguments, got {}",
                target_name,
                expected_arity,
                arg_terms.len()
            ),
            span: node.span.clone(),
        });
    }

    Ok(CoreTerm::CCall {
        target_name: canonical_target_name,
        args: arg_terms,
        node_id: None,
        annotations: node.annotations.clone(),
    })
}

/// Lower Ident (identifier reference) node
///
/// Required fields:
/// - "name": Token (identifier)
///
/// Produces: Error (identifiers are not directly representable in Core IR 0.2)
fn lower_ident(node: &SchemaAstNode, ctx: &mut LoweringContext) -> Result<CoreTerm, LoweringError> {
    // Extract name field
    let name_token = get_field_token(node, "name")?;
    let name = &name_token.lexeme;

    // Check if bound in scope
    if !ctx.scope.lookup(name) {
        return Err(LoweringError {
            message: format!("unbound identifier '{}'", name),
            span: name_token.span.clone(),
        });
    }

    // Note: Core IR 0.2 does not have a direct "variable reference" construct
    // Variables are resolved through lambda binding and application
    // This is a limitation of the current Core IR design
    // For Wave 5, we reject free identifiers
    Err(LoweringError {
        message: format!("free identifier '{}' not supported in Core IR 0.2", name),
        span: name_token.span.clone(),
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// FIELD ACCESS HELPERS
// ═══════════════════════════════════════════════════════════════════════════

/// Get a required field as a single Node
fn get_field_node<'a>(
    node: &'a SchemaAstNode,
    field_name: &str,
) -> Result<&'a SchemaAstNode, LoweringError> {
    let value = node.fields.get(field_name).ok_or_else(|| LoweringError {
        message: format!("missing required field '{}' in {}", field_name, node.kind),
        span: node.span.clone(),
    })?;

    match value {
        SchemaValue::Node(boxed_node) => Ok(boxed_node.as_ref()),
        _ => Err(LoweringError {
            message: format!(
                "field '{}' in {} must be a Node, got {:?}",
                field_name,
                node.kind,
                variant_name(value)
            ),
            span: node.span.clone(),
        }),
    }
}

/// Get a required field as a list of Nodes
fn get_field_nodes<'a>(
    node: &'a SchemaAstNode,
    field_name: &str,
) -> Result<&'a [SchemaAstNode], LoweringError> {
    let value = node.fields.get(field_name).ok_or_else(|| LoweringError {
        message: format!("missing required field '{}' in {}", field_name, node.kind),
        span: node.span.clone(),
    })?;

    match value {
        SchemaValue::Nodes(nodes) => Ok(nodes.as_slice()),
        _ => Err(LoweringError {
            message: format!(
                "field '{}' in {} must be Nodes, got {:?}",
                field_name,
                node.kind,
                variant_name(value)
            ),
            span: node.span.clone(),
        }),
    }
}

/// Get a required field as a single Token
fn get_field_token<'a>(
    node: &'a SchemaAstNode,
    field_name: &str,
) -> Result<&'a crate::frontend::token::Token, LoweringError> {
    let value = node.fields.get(field_name).ok_or_else(|| LoweringError {
        message: format!("missing required field '{}' in {}", field_name, node.kind),
        span: node.span.clone(),
    })?;

    match value {
        SchemaValue::Token(token) => Ok(token),
        _ => Err(LoweringError {
            message: format!(
                "field '{}' in {} must be a Token, got {:?}",
                field_name,
                node.kind,
                variant_name(value)
            ),
            span: node.span.clone(),
        }),
    }
}

/// Get variant name for error messages
fn variant_name(value: &SchemaValue) -> &'static str {
    match value {
        SchemaValue::Node(_) => "Node",
        SchemaValue::Nodes(_) => "Nodes",
        SchemaValue::Token(_) => "Token",
        SchemaValue::Tokens(_) => "Tokens",
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::token::{Token, TokenKind};
    use std::collections::HashMap;

    // ═══════════════════════════════════════════════════════════════════════
    // TEST HELPERS
    // ═══════════════════════════════════════════════════════════════════════

    fn make_unit_lit() -> SchemaAstNode {
        SchemaAstNode {
            kind: "UnitLit".to_string(),
            fields: HashMap::new(),
            annotations: Vec::new(),
            span: Span::new(0, 2),
        }
    }

    fn make_int_lit(value: i64) -> SchemaAstNode {
        let mut fields = HashMap::new();
        fields.insert(
            "value".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::IntLit,
                lexeme: value.to_string(),
                span: Span::new(0, value.to_string().len()),
            }),
        );
        SchemaAstNode {
            kind: "IntLit".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, value.to_string().len()),
        }
    }

    fn make_bool_lit(value: bool) -> SchemaAstNode {
        let lexeme = if value { "true" } else { "false" };
        let mut fields = HashMap::new();
        fields.insert(
            "value".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::BoolLit,
                lexeme: lexeme.to_string(),
                span: Span::new(0, lexeme.len()),
            }),
        );
        SchemaAstNode {
            kind: "BoolLit".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, lexeme.len()),
        }
    }

    fn make_var_ref(name: &str) -> SchemaAstNode {
        let mut fields = HashMap::new();
        fields.insert(
            "name".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::Ident,
                lexeme: name.to_string(),
                span: Span::new(0, name.len()),
            }),
        );
        SchemaAstNode {
            kind: "VarRef".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, name.len()),
        }
    }

    fn make_registry() -> Registry {
        use crate::registry::RegistryEntry;
        Registry {
            entries: vec![
                RegistryEntry {
                    name: "print".to_string(),
                    arity: 1,
                    deterministic: false,
                    profiles: vec![],
                    id: 42,
                },
                RegistryEntry {
                    name: "add".to_string(),
                    arity: 2,
                    deterministic: true,
                    profiles: vec![],
                    id: 43,
                },
            ],
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // WAVE S2 TESTS — SEMANTIC-SURFACE-0 LOWERING
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_wave_s2_int_lit() {
        let node = make_int_lit(42);
        let result = lower_int_lit(&node);
        assert!(result.is_ok());
        match result.unwrap() {
            CoreTerm::CIntLit { value, .. } => assert_eq!(value, 42),
            _ => panic!("Expected CIntLit"),
        }
    }

    #[test]
    fn test_wave_s2_bool_lit_true() {
        let node = make_bool_lit(true);
        let result = lower_bool_lit(&node);
        assert!(result.is_ok());
        match result.unwrap() {
            CoreTerm::CBoolLit { value, .. } => assert!(value),
            _ => panic!("Expected CBoolLit"),
        }
    }

    #[test]
    fn test_wave_s2_bool_lit_false() {
        let node = make_bool_lit(false);
        let result = lower_bool_lit(&node);
        assert!(result.is_ok());
        match result.unwrap() {
            CoreTerm::CBoolLit { value, .. } => assert!(!value),
            _ => panic!("Expected CBoolLit"),
        }
    }

    #[test]
    fn test_wave_s2_unit_lit() {
        let node = make_unit_lit();
        let result = lower_unit_lit(&node);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), crate::ir::core_ir::unit_lit());
    }

    #[test]
    fn test_wave_s2_var_ref_bound() {
        let node = make_var_ref("x");
        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        // Bind x in scope
        ctx.scope.bind("x".to_string(), Span::new(0, 1));

        let result = lower_var_ref(&node, &mut ctx);
        assert!(result.is_ok());
        match result.unwrap() {
            CoreTerm::CVar { name, .. } => assert_eq!(name.0.as_ref(), "x"),
            _ => panic!("Expected CVar"),
        }
    }

    #[test]
    fn test_wave_s2_var_ref_unbound() {
        let node = make_var_ref("x");
        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_var_ref(&node, &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("unbound variable"));
    }

    #[test]
    fn test_wave_s2_let_expr() {
        // let x = 42 in x
        let mut fields = HashMap::new();
        fields.insert(
            "name".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::Ident,
                lexeme: "x".to_string(),
                span: Span::new(0, 1),
            }),
        );
        fields.insert(
            "value".to_string(),
            SchemaValue::Node(Box::new(make_int_lit(42))),
        );
        fields.insert(
            "body".to_string(),
            SchemaValue::Node(Box::new(make_var_ref("x"))),
        );

        let node = SchemaAstNode {
            kind: "LetExpr".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 20),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CLet {
                name, value, body, ..
            } => {
                assert_eq!(name.0.as_ref(), "x");
                match *value {
                    CoreTerm::CIntLit { value: v, .. } => assert_eq!(v, 42),
                    _ => panic!("Expected CIntLit for value"),
                }
                match *body {
                    CoreTerm::CVar { name: n, .. } => assert_eq!(n.0.as_ref(), "x"),
                    _ => panic!("Expected CVar for body"),
                }
            }
            _ => panic!("Expected CLet"),
        }
    }

    #[test]
    fn test_wave_s2_lam_expr() {
        // fn x => ()
        let mut fields = HashMap::new();
        fields.insert(
            "param".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::Ident,
                lexeme: "x".to_string(),
                span: Span::new(1, 2),
            }),
        );
        fields.insert(
            "body".to_string(),
            SchemaValue::Node(Box::new(make_unit_lit())),
        );

        let node = SchemaAstNode {
            kind: "LamExpr".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 10),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CLam { param, body, .. } => {
                assert_eq!(param.0.as_ref(), "x");
                assert_eq!(*body, crate::ir::core_ir::unit_lit());
            }
            _ => panic!("Expected CLam"),
        }
    }

    #[test]
    fn test_wave_s2_if_expr() {
        // if true then 1 else 2
        let mut fields = HashMap::new();
        fields.insert(
            "condition".to_string(),
            SchemaValue::Node(Box::new(make_bool_lit(true))),
        );
        fields.insert(
            "then_branch".to_string(),
            SchemaValue::Node(Box::new(make_int_lit(1))),
        );
        fields.insert(
            "else_branch".to_string(),
            SchemaValue::Node(Box::new(make_int_lit(2))),
        );

        let node = SchemaAstNode {
            kind: "IfExpr".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 20),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CIf {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                match *cond {
                    CoreTerm::CBoolLit { value: v, .. } => assert!(v),
                    _ => panic!("Expected CBoolLit"),
                }
                match *then_branch {
                    CoreTerm::CIntLit { value: v, .. } => assert_eq!(v, 1),
                    _ => panic!("Expected CIntLit"),
                }
                match *else_branch {
                    CoreTerm::CIntLit { value: v, .. } => assert_eq!(v, 2),
                    _ => panic!("Expected CIntLit"),
                }
            }
            _ => panic!("Expected CIf"),
        }
    }

    #[test]
    fn test_wave_s2_app_expr_single() {
        // f(x) where f and x are bound
        let mut fields = HashMap::new();
        fields.insert(
            "parts".to_string(),
            SchemaValue::Nodes(vec![make_var_ref("f"), make_var_ref("x")]),
        );

        let node = SchemaAstNode {
            kind: "AppExpr".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 5),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        // Bind f and x
        ctx.scope.bind("f".to_string(), Span::new(0, 1));
        ctx.scope.bind("x".to_string(), Span::new(2, 3));

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CApp { func, arg, .. } => {
                match *func {
                    CoreTerm::CVar { name: n, .. } => assert_eq!(n.0.as_ref(), "f"),
                    _ => panic!("Expected CVar for func"),
                }
                match *arg {
                    CoreTerm::CVar { name: n, .. } => assert_eq!(n.0.as_ref(), "x"),
                    _ => panic!("Expected CVar for arg"),
                }
            }
            _ => panic!("Expected CApp"),
        }
    }

    #[test]
    fn test_wave_s2_app_expr_curried() {
        // f(x)(y) - should produce CApp(CApp(f, x), y)
        let mut fields = HashMap::new();
        fields.insert(
            "parts".to_string(),
            SchemaValue::Nodes(vec![
                make_var_ref("f"),
                make_var_ref("x"),
                make_var_ref("y"),
            ]),
        );

        let node = SchemaAstNode {
            kind: "AppExpr".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 8),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        // Bind f, x, y
        ctx.scope.bind("f".to_string(), Span::new(0, 1));
        ctx.scope.bind("x".to_string(), Span::new(2, 3));
        ctx.scope.bind("y".to_string(), Span::new(5, 6));

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_ok());

        // Verify structure: CApp(CApp(f, x), y)
        match result.unwrap() {
            CoreTerm::CApp {
                func: outer_func,
                arg: y_arg,
                ..
            } => {
                // outer_func should be CApp(f, x)
                match *outer_func {
                    CoreTerm::CApp {
                        func: f_var,
                        arg: x_arg,
                        ..
                    } => {
                        match *f_var {
                            CoreTerm::CVar { name: n, .. } => assert_eq!(n.0.as_ref(), "f"),
                            _ => panic!("Expected CVar for f"),
                        }
                        match *x_arg {
                            CoreTerm::CVar { name: n, .. } => assert_eq!(n.0.as_ref(), "x"),
                            _ => panic!("Expected CVar for x"),
                        }
                    }
                    _ => panic!("Expected inner CApp"),
                }
                // y_arg should be y
                match *y_arg {
                    CoreTerm::CVar { name: n, .. } => assert_eq!(n.0.as_ref(), "y"),
                    _ => panic!("Expected CVar for y"),
                }
            }
            _ => panic!("Expected outer CApp"),
        }
    }

    #[test]
    fn test_wave_s2_wrapper_atomic_expr() {
        // AtomicExpr wrapping IntLit
        let mut fields = HashMap::new();
        fields.insert(
            "inner".to_string(),
            SchemaValue::Node(Box::new(make_int_lit(42))),
        );

        let node = SchemaAstNode {
            kind: "AtomicExpr".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 2),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CIntLit { value, .. } => assert_eq!(value, 42),
            _ => panic!("Expected CIntLit (wrapper should be transparent)"),
        }
    }

    #[test]
    fn test_wave_s2_wrapper_expr() {
        // Expr wrapping BoolLit
        let mut fields = HashMap::new();
        fields.insert(
            "inner".to_string(),
            SchemaValue::Node(Box::new(make_bool_lit(true))),
        );

        let node = SchemaAstNode {
            kind: "Expr".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 4),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CBoolLit { value, .. } => assert!(value),
            _ => panic!("Expected CBoolLit (wrapper should be transparent)"),
        }
    }

    #[test]
    fn test_wave_s2_totality_unknown_node() {
        // Verify unknown node kinds are rejected (no wildcards)
        let node = SchemaAstNode {
            kind: "UnknownNode".to_string(),
            fields: HashMap::new(),
            annotations: Vec::new(),
            span: Span::new(0, 10),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("unknown schema node kind"));
    }

    #[test]
    fn test_wave_s2_determinism() {
        // Lower the same node twice and verify identical output
        let node = make_int_lit(123);
        let mut ctx1 = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };
        let mut ctx2 = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result1 = lower_to_core_ir(&node, &mut ctx1).unwrap();
        let result2 = lower_to_core_ir(&node, &mut ctx2).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_wave_s2_bundle_production() {
        // Test complete lowering pipeline to bundle
        let node = make_int_lit(99);
        let registry = make_registry();

        let result = lower_to_bundle(&node, registry);
        assert!(result.is_ok());

        let bundle = result.unwrap();
        assert_eq!(bundle.version.as_ref(), "0.3");
        match bundle.core_term {
            CoreTerm::CIntLit { value, .. } => assert_eq!(value, 99),
            _ => panic!("Expected CIntLit"),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // LEGACY SURFACE-0 TESTS (EXISTING)
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_lower_lam() {
        // Create a lambda: \x -> ()
        let mut fields = HashMap::new();
        fields.insert(
            "param".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::Ident,
                lexeme: "x".to_string(),
                span: Span::new(1, 2),
            }),
        );
        fields.insert(
            "body".to_string(),
            SchemaValue::Node(Box::new(make_unit_lit())),
        );

        let node = SchemaAstNode {
            kind: "Lam".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 10),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CLam { param, body, .. } => {
                assert_eq!(param.0.as_ref(), "x");
                assert_eq!(*body, crate::ir::core_ir::unit_lit());
            }
            _ => panic!("Expected CLam"),
        }
    }

    #[test]
    fn test_lower_if() {
        // Create an if: if () then () else ()
        let mut fields = HashMap::new();
        fields.insert(
            "condition".to_string(),
            SchemaValue::Node(Box::new(make_unit_lit())),
        );
        fields.insert(
            "then_branch".to_string(),
            SchemaValue::Node(Box::new(make_unit_lit())),
        );
        fields.insert(
            "else_branch".to_string(),
            SchemaValue::Node(Box::new(make_unit_lit())),
        );

        let node = SchemaAstNode {
            kind: "If".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 20),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CIf { .. } => {}
            _ => panic!("Expected CIf"),
        }
    }

    #[test]
    fn test_lower_call_success() {
        // Create a call: print(())
        let mut fields = HashMap::new();
        fields.insert(
            "target".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::Ident,
                lexeme: "print".to_string(),
                span: Span::new(0, 5),
            }),
        );
        fields.insert(
            "args".to_string(),
            SchemaValue::Nodes(vec![make_unit_lit()]),
        );

        let node = SchemaAstNode {
            kind: "Call".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 10),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_ok());

        match result.unwrap() {
            CoreTerm::CCall {
                target_name, args, ..
            } => {
                assert_eq!(target_name, "print");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected CCall"),
        }
    }

    #[test]
    fn test_missing_field_error() {
        // Create a Lam without the required "param" field
        let mut fields = HashMap::new();
        fields.insert(
            "body".to_string(),
            SchemaValue::Node(Box::new(make_unit_lit())),
        );

        let node = SchemaAstNode {
            kind: "Lam".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 10),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("missing required field 'param'"));
    }

    #[test]
    fn test_wrong_field_type_error() {
        // Create a Lam with wrong type for "param" field
        let mut fields = HashMap::new();
        fields.insert(
            "param".to_string(),
            SchemaValue::Node(Box::new(make_unit_lit())), // Should be Token, not Node
        );
        fields.insert(
            "body".to_string(),
            SchemaValue::Node(Box::new(make_unit_lit())),
        );

        let node = SchemaAstNode {
            kind: "Lam".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 10),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("must be a Token"));
    }

    #[test]
    fn test_unknown_node_kind_error() {
        let node = SchemaAstNode {
            kind: "UnknownNode".to_string(),
            fields: HashMap::new(),
            annotations: Vec::new(),
            span: Span::new(0, 10),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("unknown schema node kind"));
    }

    #[test]
    fn test_registry_lookup_failure() {
        // Create a call to unknown function
        let mut fields = HashMap::new();
        fields.insert(
            "target".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::Ident,
                lexeme: "unknown_func".to_string(),
                span: Span::new(0, 12),
            }),
        );
        fields.insert("args".to_string(), SchemaValue::Nodes(vec![]));

        let node = SchemaAstNode {
            kind: "Call".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 20),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("unknown function"));
    }

    #[test]
    fn test_arity_mismatch_error() {
        // Create a call with wrong arity: print((), ())
        let mut fields = HashMap::new();
        fields.insert(
            "target".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::Ident,
                lexeme: "print".to_string(),
                span: Span::new(0, 5),
            }),
        );
        fields.insert(
            "args".to_string(),
            SchemaValue::Nodes(vec![make_unit_lit(), make_unit_lit()]), // print expects 1 arg
        );

        let node = SchemaAstNode {
            kind: "Call".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 20),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("expects 1 arguments"));
    }

    #[test]
    fn test_unbound_identifier_error() {
        // Create an identifier reference to unbound variable
        let mut fields = HashMap::new();
        fields.insert(
            "name".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::Ident,
                lexeme: "x".to_string(),
                span: Span::new(0, 1),
            }),
        );

        let node = SchemaAstNode {
            kind: "Ident".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 1),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("unbound identifier"));
    }

    #[test]
    fn test_determinism() {
        // Lower the same node twice and verify identical output
        let node = make_unit_lit();
        let mut ctx1 = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };
        let mut ctx2 = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result1 = lower_to_core_ir(&node, &mut ctx1).unwrap();
        let result2 = lower_to_core_ir(&node, &mut ctx2).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_span_preservation() {
        // Verify that spans are not widened or modified
        let node = SchemaAstNode {
            kind: "UnitLit".to_string(),
            fields: HashMap::new(),
            annotations: Vec::new(),
            span: Span::new(10, 12),
        };

        // Note: CoreTerm doesn't include span in current implementation
        // This test verifies that lowering doesn't fail due to span issues
        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        let result = lower_to_core_ir(&node, &mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scope_binding() {
        // Test that lambda properly binds parameter in scope
        let mut scope = Scope::new();
        assert!(!scope.lookup("x"));

        scope.bind("x".to_string(), Span::new(0, 1));
        assert!(scope.lookup("x"));
        assert!(!scope.lookup("y"));
    }

    #[test]
    fn test_scope_shadowing() {
        // Test that inner scope can shadow outer scope
        let mut scope = Scope::new();

        scope.bind("x".to_string(), Span::new(0, 1));
        assert!(scope.lookup("x"));

        scope.push_scope();
        scope.bind("x".to_string(), Span::new(10, 11));
        assert!(scope.lookup("x"));

        scope.pop_scope();
        assert!(scope.lookup("x"));
    }

    #[test]
    fn test_lower_to_bundle() {
        // Test the complete lowering pipeline
        let node = make_unit_lit();
        let registry = make_registry();

        let result = lower_to_bundle(&node, registry);
        assert!(result.is_ok());

        let bundle = result.unwrap();
        assert_eq!(bundle.version.as_ref(), "0.3");
        assert_eq!(bundle.core_term, crate::ir::core_ir::unit_lit());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // WAVE S2.1: WRAPPER NORMALIZATION BOUNDARY TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_wrapper_normalization_expr() {
        // Create an Expr wrapper around IntLit
        let inner = make_int_lit(42);
        let mut fields = HashMap::new();
        fields.insert("inner".to_string(), SchemaValue::Node(Box::new(inner)));

        let wrapper_node = SchemaAstNode {
            kind: "Expr".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 2),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        // Wrapper should be normalized away - lowering should succeed
        let result = lower_to_core_ir(&wrapper_node, &mut ctx);
        assert!(result.is_ok());

        // Result should be CIntLit(42)
        match result.unwrap() {
            CoreTerm::CIntLit { value, .. } => assert_eq!(value, 42),
            other => panic!("Expected CIntLit, got {:?}", other),
        }
    }

    #[test]
    fn test_wrapper_normalization_atomic_expr() {
        // Create an AtomicExpr wrapper around BoolLit
        let inner = make_bool_lit(true);
        let mut fields = HashMap::new();
        fields.insert("inner".to_string(), SchemaValue::Node(Box::new(inner)));

        let wrapper_node = SchemaAstNode {
            kind: "AtomicExpr".to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 4),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        // Wrapper should be normalized away
        let result = lower_to_core_ir(&wrapper_node, &mut ctx);
        assert!(result.is_ok());

        // Result should be CBoolLit(true)
        match result.unwrap() {
            CoreTerm::CBoolLit { value, .. } => assert_eq!(value, true),
            other => panic!("Expected CBoolLit, got {:?}", other),
        }
    }

    #[test]
    fn test_wrapper_normalization_nested() {
        // Create nested wrappers: Expr(AtomicExpr(IntLit(99)))
        let innermost = make_int_lit(99);

        let mut atomic_fields = HashMap::new();
        atomic_fields.insert("inner".to_string(), SchemaValue::Node(Box::new(innermost)));
        let atomic_wrapper = SchemaAstNode {
            kind: "AtomicExpr".to_string(),
            fields: atomic_fields,
            annotations: Vec::new(),
            span: Span::new(0, 2),
        };

        let mut expr_fields = HashMap::new();
        expr_fields.insert(
            "inner".to_string(),
            SchemaValue::Node(Box::new(atomic_wrapper)),
        );
        let expr_wrapper = SchemaAstNode {
            kind: "Expr".to_string(),
            fields: expr_fields,
            annotations: Vec::new(),
            span: Span::new(0, 2),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        // Both wrappers should be normalized away
        let result = lower_to_core_ir(&expr_wrapper, &mut ctx);
        assert!(result.is_ok());

        // Result should be CIntLit(99)
        match result.unwrap() {
            CoreTerm::CIntLit { value, .. } => assert_eq!(value, 99),
            other => panic!("Expected CIntLit(99), got {:?}", other),
        }
    }

    #[test]
    fn test_wrapper_normalization_in_fields() {
        // Create LetExpr with wrapper in value field
        // let x = Expr(IntLit(10)) in VarRef(x)

        let value_inner = make_int_lit(10);
        let mut value_wrapper_fields = HashMap::new();
        value_wrapper_fields.insert(
            "inner".to_string(),
            SchemaValue::Node(Box::new(value_inner)),
        );
        let value_wrapper = SchemaAstNode {
            kind: "Expr".to_string(),
            fields: value_wrapper_fields,
            annotations: Vec::new(),
            span: Span::new(0, 2),
        };

        let mut let_fields = HashMap::new();
        let_fields.insert(
            "name".to_string(),
            SchemaValue::Token(Token {
                kind: TokenKind::Ident,
                lexeme: "x".to_string(),
                span: Span::new(0, 1),
            }),
        );
        let_fields.insert(
            "value".to_string(),
            SchemaValue::Node(Box::new(value_wrapper)),
        );
        let_fields.insert(
            "body".to_string(),
            SchemaValue::Node(Box::new(make_var_ref("x"))),
        );

        let let_node = SchemaAstNode {
            kind: "LetExpr".to_string(),
            fields: let_fields,
            annotations: Vec::new(),
            span: Span::new(0, 20),
        };

        let mut ctx = LoweringContext {
            registry: make_registry(),
            scope: Scope::new(),
        };

        // Wrapper in value field should be normalized
        let result = lower_to_core_ir(&let_node, &mut ctx);
        assert!(result.is_ok());

        // Result should be CLet with CIntLit(10) as value
        match result.unwrap() {
            CoreTerm::CLet { value, .. } => match *value {
                CoreTerm::CIntLit { value: v, .. } => assert_eq!(v, 10),
                other => panic!("Expected CIntLit in value, got {:?}", other),
            },
            other => panic!("Expected CLet, got {:?}", other),
        }
    }

    #[test]
    fn test_normalize_wrappers_preserves_semantic_nodes() {
        // Verify that normalize_wrappers doesn't modify semantic nodes
        let semantic_node = make_int_lit(42);
        let normalized = normalize_wrappers(&semantic_node);

        assert_eq!(normalized.kind, "IntLit");
        assert_eq!(normalized.fields.len(), 1);
    }
}
