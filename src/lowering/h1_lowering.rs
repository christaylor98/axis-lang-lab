// H1 Lowering — Minimal, Mechanical
//
// WAVE 5: H1 Lowering (Minimal, Mechanical)
//
// This module implements FIXED, EMBEDDED lowering for H1 Normal Form (NF-H1).
// It consumes ONLY NF-H1 compliant Schema AST and emits Core IR 0.2.
//
// ARCHITECTURAL INVARIANTS:
// - Lowering is FIXED and EMBEDDED (not user-configurable)
// - Lowering is VERSIONED with the compiler
// - Lowering is the SOLE SEMANTIC AUTHORITY
// - No surface constructs may reach lowering
// - Lowering is DETERMINISTIC and TOTAL over NF-H1
//
// NF-H1 ADMISSIBLE NODES (from core_spec/NORMAL_FORM_H1.md):
// - Program, FunctionDecl
// - LetExpr, Block
// - IfExpr
// - LambdaExpr, CallExpr
// - Ident, Literal, Unit
//
// NF-H1 FORBIDDEN NODES:
// - ForExpr, LoopExpr, WhileExpr
// - MatchExpr, Pattern nodes
// - Any surface-specific constructs
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
            "H1 Lowering error at {}..{}: {}",
            self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for LoweringError {}

// ═══════════════════════════════════════════════════════════════════════════
// MAIN LOWERING ENTRY POINTS
// ═══════════════════════════════════════════════════════════════════════════

/// Lower NF-H1 Schema AST to Core IR
///
/// PRECONDITIONS:
/// - node is NF-H1 compliant (validated before lowering)
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
    // NO WILDCARDS - total match over NF-H1 admissible set
    match node.kind.as_str() {
        // Program structure
        "Program" => lower_program(node, ctx),
        "FunctionDecl" => lower_function_decl(node, ctx),

        // Binding and scope
        "LetExpr" => lower_let_expr(node, ctx),
        "Block" => lower_block(node, ctx),

        // Control flow
        "IfExpr" => lower_if_expr(node, ctx),

        // Computation
        "LambdaExpr" => lower_lambda_expr(node, ctx),
        "CallExpr" => lower_call_expr(node, ctx),

        // Values
        "Ident" => lower_ident(node, ctx),
        "Literal" => lower_literal(node),
        "Unit" => lower_unit(node),

        // Wrappers (transparent - strip and recurse)
        "Expr" | "AtomicExpr" => {
            extract_field_node(node, "variant")
                .and_then(|child| lower_to_core_ir(child, ctx))
        }

        // BOUNDARY VIOLATION: Surface nodes reaching lowering
        "ForExpr" | "LoopExpr" | "WhileExpr" => {
            Err(LoweringError {
                message: format!(
                    "BOUNDARY VIOLATION: surface node '{}' reached lowering (should have been normalized)",
                    node.kind
                ),
                span: node.span.clone(),
            })
        }

        "MatchExpr" | "Pattern" | "IdentPat" | "CtorPat" => {
            Err(LoweringError {
                message: format!(
                    "BOUNDARY VIOLATION: surface pattern node '{}' reached lowering (should have been normalized)",
                    node.kind
                ),
                span: node.span.clone(),
            })
        }

        // UNKNOWN NODE: Not in NF-H1 admissible set
        unknown => {
            Err(LoweringError {
                message: format!(
                    "unknown node kind '{}' - not in NF-H1 admissible set",
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

/// Lower to Core IR bundle (convenience wrapper)
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
// NODE LOWERING FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Lower Program node
///
/// Program contains a list of top-level items (functions).
/// For minimal H1, we expect exactly one function and lower it.
///
/// Schema fields:
/// - items: list of Item nodes
fn lower_program(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let items = extract_field_nodes(node, "items")?;

    if items.is_empty() {
        return Err(LoweringError {
            message: "empty program (no functions)".to_string(),
            span: node.span.clone(),
        });
    }

    // For minimal H1, lower first item only
    // (multi-function programs require more complex semantics)
    let first_item = &items[0];

    // Item is a wrapper - extract variant
    let function = extract_field_node(first_item, "variant")?;
    lower_to_core_ir(function, ctx)
}

/// Lower FunctionDecl node
///
/// Function declarations in H1 are:
///   fn name(params...) { body }
///
/// Lowering to Core IR:
///   CLam(params, body)
///
/// Schema fields:
/// - name: token (identifier)
/// - params: ParamList node
/// - body: Block node
fn lower_function_decl(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    // Extract function name (for debugging/inspection only)
    let _name = extract_field_token(node, "name")?;

    // Extract parameters
    let param_list = extract_field_node(node, "params")?;
    let params = extract_params(param_list)?;

    // Extract body
    let body_node = extract_field_node(node, "body")?;

    // Lower body in new scope
    ctx.scope.push();
    for param in &params {
        ctx.scope.bind(param.clone());
    }
    let body_term = lower_to_core_ir(body_node, ctx)?;
    ctx.scope.pop();

    // Build nested lambdas for multiple parameters
    // fn f(a, b, c) { body } → λa. λb. λc. body
    let result = params.iter().rev().fold(body_term, |acc, param| {
        crate::ir::core_ir::lam(IdentOrName::new(param.clone()), acc)
    });

    Ok(result)
}

/// Extract parameter names from ParamList
///
/// ParamList schema:
/// - params: list of Param nodes
///
/// Param schema:
/// - name: token (identifier)
fn extract_params(param_list: &SchemaAstNode) -> Result<Vec<String>, LoweringError> {
    let param_nodes = extract_field_nodes(param_list, "params")?;

    let mut params = Vec::new();
    for param_node in param_nodes {
        // Each param might be wrapped in ParamRest
        let actual_param = if param_node.kind == "ParamRest" {
            extract_field_node(param_node, "param")?
        } else {
            param_node
        };

        let name = extract_field_token(actual_param, "name")?;
        params.push(name);
    }

    Ok(params)
}

/// Lower LetExpr node
///
/// Let expressions bind a value to a name:
///   let x = value in body
///
/// Lowering to Core IR:
///   CLet(x, value, body)
///
/// Schema fields:
/// - name: token (identifier)
/// - value: expression node
/// - body: expression node
fn lower_let_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let name = extract_field_token(node, "name")?;
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
        IdentOrName::new(name.clone()),
        value_term,
        body_term,
    ))
}

/// Lower Block node
///
/// Blocks contain a sequence of statements/expressions.
/// For minimal H1, treat as transparent wrapper to content.
///
/// Schema fields:
/// - content: BlockContent node
fn lower_block(node: &SchemaAstNode, ctx: &mut LoweringContext) -> Result<CoreTerm, LoweringError> {
    let content = extract_field_node(node, "content")?;
    lower_block_content(content, ctx)
}

/// Lower BlockContent node
///
/// BlockContent contains a list of statements.
/// Statements are sequenced using nested let bindings.
///
/// Schema fields:
/// - stmts: list of Stmt nodes
fn lower_block_content(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let stmt_nodes = extract_field_nodes(node, "stmts")?;

    if stmt_nodes.is_empty() {
        return Ok(crate::ir::core_ir::unit_lit());
    }

    // Lower statements in sequence
    lower_stmt_sequence(&stmt_nodes, ctx)
}

/// Lower a sequence of statements
///
/// Statements are lowered as:
/// - LetStmt: CLet binding
/// - ExprStmt: evaluated for effect, result discarded
/// - Last statement: provides the block result
///
/// Sequencing uses nested let bindings:
///   { stmt1; stmt2; expr } → let _ = stmt1 in let _ = stmt2 in expr
fn lower_stmt_sequence(
    stmts: &[&SchemaAstNode],
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    if stmts.is_empty() {
        return Ok(crate::ir::core_ir::unit_lit());
    }

    if stmts.len() == 1 {
        return lower_stmt(stmts[0], ctx);
    }

    // Lower first statement
    let first = stmts[0];
    let rest = &stmts[1..];

    // Check if first is a LetStmt (creates binding) or other (effect only)
    let first_stmt = extract_field_node(first, "variant")?;

    if first_stmt.kind == "LetStmt" {
        // LetStmt: lower as CLet with continuation
        let name = extract_field_token(first_stmt, "name")?;
        let value_node = extract_field_node(first_stmt, "value")?;

        let value_term = lower_to_core_ir(value_node, ctx)?;

        ctx.scope.push();
        ctx.scope.bind(name.clone());
        let body_term = lower_stmt_sequence(rest, ctx)?;
        ctx.scope.pop();

        Ok(crate::ir::core_ir::clet(
            IdentOrName::new(name.clone()),
            value_term,
            body_term,
        ))
    } else {
        // Other statement: lower as let _ = stmt in rest
        let stmt_term = lower_stmt(first, ctx)?;
        let rest_term = lower_stmt_sequence(rest, ctx)?;

        Ok(crate::ir::core_ir::clet(
            IdentOrName::dummy(),
            stmt_term,
            rest_term,
        ))
    }
}

/// Lower Stmt node (wrapper)
///
/// Schema fields:
/// - variant: actual statement node
fn lower_stmt(node: &SchemaAstNode, ctx: &mut LoweringContext) -> Result<CoreTerm, LoweringError> {
    let variant = extract_field_node(node, "variant")?;

    match variant.kind.as_str() {
        "LetStmt" => {
            // LetStmt without continuation → just evaluate value
            let value_node = extract_field_node(variant, "value")?;
            lower_to_core_ir(value_node, ctx)
        }
        "ExprStmt" => {
            let expr = extract_field_node(variant, "expr")?;
            lower_to_core_ir(expr, ctx)
        }
        "IntentDirectiveStmt" => {
            // Intent directives are metadata only - emit unit
            Ok(crate::ir::core_ir::unit_lit())
        }
        _ => lower_to_core_ir(variant, ctx),
    }
}

/// Lower IfExpr node
///
/// Conditional expression:
///   if cond { then } else { else }
///
/// Lowering to Core IR:
///   CIf(cond, then, else)
///
/// Schema fields:
/// - condition: expression node
/// - then_block: Block node
/// - else_block: Block node
fn lower_if_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let cond_node = extract_field_node(node, "condition")?;
    let then_node = extract_field_node(node, "then_block")?;
    let else_node = extract_field_node(node, "else_block")?;

    let cond_term = lower_to_core_ir(cond_node, ctx)?;
    let then_term = lower_to_core_ir(then_node, ctx)?;
    let else_term = lower_to_core_ir(else_node, ctx)?;

    Ok(crate::ir::core_ir::cif(cond_term, then_term, else_term))
}

/// Lower LambdaExpr node
///
/// Lambda abstraction:
///   λx. body
///
/// Lowering to Core IR:
///   CLam(x, body)
///
/// Schema fields:
/// - param: token (identifier)
/// - body: expression node
fn lower_lambda_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let param = extract_field_token(node, "param")?;
    let body_node = extract_field_node(node, "body")?;

    ctx.scope.push();
    ctx.scope.bind(param.clone());
    let body_term = lower_to_core_ir(body_node, ctx)?;
    ctx.scope.pop();

    Ok(crate::ir::core_ir::lam(
        IdentOrName::new(param.clone()),
        body_term,
    ))
}

/// Lower CallExpr node
///
/// Function call expression:
///   target(args...)
///
/// Lowering depends on target:
/// - Registry function → CCall
/// - Local function → CApp chain
///
/// Schema fields:
/// - target: expression node
/// - suffixes: list of CallSuffix nodes
fn lower_call_expr(
    node: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let target_node = extract_field_node(node, "target")?;
    let suffixes = extract_field_nodes(node, "suffixes")?;

    // Lower target
    let mut result = lower_to_core_ir(target_node, ctx)?;

    // Apply each call suffix
    for suffix in suffixes {
        result = lower_call_suffix(result, suffix, ctx)?;
    }

    Ok(result)
}

/// Lower CallSuffix (function application)
///
/// Schema fields:
/// - args: ArgList node
fn lower_call_suffix(
    target: CoreTerm,
    suffix: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<CoreTerm, LoweringError> {
    let arg_list = extract_field_node(suffix, "args")?;
    let args = extract_args(arg_list, ctx)?;

    // Check if target is a registry call
    if let CoreTerm::CVar { ref name, .. } = target {
        // Try registry lookup
        if let Some(entry) = ctx.registry.find_by_name(&name.0) {
            // Registry function - emit CCall
            // Core IR 0.3: CCall takes canonical function name (not numeric ID)
            return Ok(crate::ir::core_ir::ccall(entry.name.clone(), args));
        }
    }

    // Local function - emit CApp chain
    // f(a, b, c) → ((f a) b) c
    let result = args
        .into_iter()
        .fold(target, |acc, arg| crate::ir::core_ir::capp(acc, arg));

    Ok(result)
}

/// Extract arguments from ArgList
///
/// Schema fields:
/// - args: list of ArgListRest nodes (each containing an expression)
fn extract_args(
    arg_list: &SchemaAstNode,
    ctx: &mut LoweringContext,
) -> Result<Vec<CoreTerm>, LoweringError> {
    let arg_nodes = extract_field_nodes(arg_list, "args")?;

    let mut args = Vec::new();
    for arg_node in arg_nodes {
        // Each arg might be wrapped in ArgListRest
        let actual_arg = if arg_node.kind == "ArgListRest" {
            extract_field_node(arg_node, "arg")?
        } else {
            arg_node
        };

        let arg_term = lower_to_core_ir(actual_arg, ctx)?;
        args.push(arg_term);
    }

    Ok(args)
}

/// Lower Ident node
///
/// Variable reference.
///
/// Lowering to Core IR:
///   CVar(name)
///
/// Schema fields:
/// - name: token (identifier)
fn lower_ident(node: &SchemaAstNode, ctx: &mut LoweringContext) -> Result<CoreTerm, LoweringError> {
    let name = extract_field_token(node, "name")?;

    // Check if in scope (for diagnostics)
    if ctx.scope.lookup(&name).is_none() {
        // Not in local scope - might be registry function (allowed)
        // Will be resolved during CCall emission
    }

    Ok(crate::ir::core_ir::cvar(IdentOrName::new(name.clone())))
}

/// Lower Literal node
///
/// Literal values (int, bool, unit).
///
/// Lowering to Core IR:
///   CIntLit, CBoolLit, or CUnitLit
///
/// Schema fields:
/// - value: token (literal value)
fn lower_literal(node: &SchemaAstNode) -> Result<CoreTerm, LoweringError> {
    let value_node = extract_field_node(node, "value")?;

    // Check literal type by token kind (if available)
    if let Some(token) = value_node.fields.get("token").and_then(|v| match v {
        SchemaValue::Token(t) => Some(t),
        _ => None,
    }) {
        // Match based on lexeme for boolean literals, or parse as int
        match token.lexeme.as_str() {
            "true" => Ok(crate::ir::core_ir::bool_lit(true)),
            "false" => Ok(crate::ir::core_ir::bool_lit(false)),
            _ => {
                // Try parsing as integer
                if let Ok(value) = token.lexeme.parse::<i64>() {
                    Ok(crate::ir::core_ir::int_lit(value))
                } else {
                    Err(LoweringError {
                        message: format!("unsupported literal value: {}", token.lexeme),
                        span: token.span.clone(),
                    })
                }
            }
        }
    } else {
        Err(LoweringError {
            message: "literal missing token value".to_string(),
            span: node.span.clone(),
        })
    }
}

/// Lower Unit node
///
/// Unit value.
///
/// Lowering to Core IR:
///   CUnitLit
fn lower_unit(_node: &SchemaAstNode) -> Result<CoreTerm, LoweringError> {
    Ok(crate::ir::core_ir::unit_lit())
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
                name: "print".to_string(),
                arity: 1,
                deterministic: true,
                profiles: vec!["test".to_string()],
            }],
        }
    }

    fn make_token(kind: &str, lexeme: &str) -> Token {
        let token_kind = match kind {
            "IDENT" => TokenKind::Ident,
            _ => TokenKind::Ident,
        };
        Token {
            kind: token_kind,
            lexeme: lexeme.to_string(),
            span: Span::new(0, lexeme.len()),
        }
    }

    fn make_ident_node(name: &str) -> SchemaAstNode {
        let mut fields = HashMap::new();
        fields.insert(
            "name".to_string(),
            SchemaValue::Token(make_token("IDENT", name)),
        );

        SchemaAstNode {
            kind: "Ident".to_string(),
            fields,
            annotations: Default::default(),
            span: Span::new(0, name.len()),
        }
    }

    fn make_unit_node() -> SchemaAstNode {
        SchemaAstNode {
            kind: "Unit".to_string(),
            fields: HashMap::new(),
            annotations: Default::default(),
            span: Span::new(0, 2),
        }
    }

    #[test]
    fn test_lower_unit() {
        let node = make_unit_node();
        let result = lower_unit(&node);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), CoreTerm::CUnitLit { .. }));
    }

    #[test]
    fn test_lower_ident() {
        let node = make_ident_node("x");
        let mut ctx = LoweringContext {
            registry: make_test_registry(),
            scope: Scope::new(),
        };

        let result = lower_ident(&node, &mut ctx);
        assert!(result.is_ok());

        if let CoreTerm::CVar { name, .. } = result.unwrap() {
            assert_eq!(name.0.as_ref(), "x");
        } else {
            panic!("Expected CVar");
        }
    }

    #[test]
    fn test_surface_node_rejection() {
        let node = SchemaAstNode {
            kind: "ForExpr".to_string(),
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
