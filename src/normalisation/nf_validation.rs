// Normal Form (NF) Validation for H1
//
// This module implements structural validation of Schema AST against the
// NF-H1 specification (core_spec/NORMAL_FORM_H1.md).
//
// SCOPE (WAVE 1):
// - Validate node kinds against admissible set
// - Fail hard on forbidden constructs
// - Provide clear error messages
// - No normalisation (transformation is separate)
// - No semantics (lowering's responsibility)
// - No registry access
//
// INVARIANTS:
// - Validation is deterministic
// - Validation is total (no panic, always returns Result)
// - Validation is pure (no side effects, no AST modification)
// - Validation is recursive (checks entire tree)

use crate::frontend::schema_ast::{SchemaAstNode, SchemaValue};
use crate::frontend::token::Span;
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════

/// NF validation error
#[derive(Debug, Clone)]
pub struct NfValidationError {
    pub message: String,
    pub span: Span,
    pub node_kind: String,
    pub parent_kind: Option<String>,
}

impl fmt::Display for NfValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NF Validation Error at {}..{}: {}",
            self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for NfValidationError {}

/// Validation result - Ok means NF-compliant
pub type NfValidationResult = Result<(), NfValidationError>;

/// Validate that a Schema AST conforms to NF-H1
///
/// This is the main entry point for NF validation.
///
/// Checks:
/// - All node kinds are in the admissible set
/// - No forbidden constructs present
/// - Structural validity (recursive)
///
/// Does NOT check:
/// - Type correctness
/// - Name resolution
/// - Semantic validity
///
/// Returns:
/// - Ok(()) if AST is NF-compliant
/// - Err(NfValidationError) if any violation is found
///
/// DETERMINISTIC: Same AST always produces same result
/// TOTAL: Never panics, always returns Result
/// PURE: No side effects, does not modify AST
pub fn validate_nf(node: &SchemaAstNode) -> NfValidationResult {
    validate_node(node, None)
}

// ═══════════════════════════════════════════════════════════════════════════
// NF-H1 ADMISSIBLE SET (Normative)
// ═══════════════════════════════════════════════════════════════════════════

/// NF-H1 Admissible Node Kinds
///
/// Source: core_spec/NORMAL_FORM_H1.md § 3
///
/// These are the ONLY node kinds permitted in NF-H1.
/// Any other node kind is a hard error.
///
/// WAVE 1 NOTE: This set is aligned with Semantic-Surface-0 lowering targets.
/// The generic names in the spec (e.g., LambdaExpr) map to surface names (e.g., LamExpr).
/// WAVE 3 NOTE: H1-specific wrapper nodes added to allow testing forbidden constructs.
const NF_ADMISSIBLE_NODES: &[&str] = &[
    // Program structure
    "Program",
    "FunctionDecl",
    "FnDecl", // H1 function declaration
    "Item",   // H1 top-level item wrapper
    // Binding and scope
    "LetExpr",
    "LetStmt", // H1 let statement
    "Block",
    "BlockContent", // H1 block content wrapper
    "Stmt",         // H1 statement wrapper
    "ExprStmt",     // H1 expression statement
    // Control flow
    "IfExpr",
    // Computation
    "LamExpr",  // Lambda abstraction (spec: LambdaExpr)
    "AppExpr",  // Application (spec: CallExpr for non-curried)
    "CallExpr", // Direct call (alternative to AppExpr)
    // Values
    "VarRef",  // Variable reference (spec: Ident)
    "Ident",   // Identifier (alternative to VarRef)
    "IntLit",  // Integer literal (spec: Literal)
    "BoolLit", // Boolean literal (spec: Literal)
    "UnitLit", // Unit literal (spec: Unit)
    "Literal", // Generic literal (covers IntLit, BoolLit, etc.)
    "Unit",    // Unit value
    // H1 structural nodes (to be normalized away)
    "ParamList",
    "Param",
    "Primary",
    "UnaryExpr",
    "MulExpr",
    "AddExpr",
    "RelExpr",
    "EqualityExpr",
    "ArgList",
    "CallSuffix",
    // Structural wrappers (transparent, stripped at lowering boundary)
    "Expr",
    "AtomicExpr",
];

/// NF-H1 Forbidden Constructs (Explicit)
///
/// Source: core_spec/NORMAL_FORM_H1.md § 4
///
/// These node kinds MUST NOT appear in NF.
/// They indicate failed or incomplete normalisation.
const NF_FORBIDDEN_NODES: &[&str] = &[
    // Surface control flow
    "ForExpr",
    "LoopExpr",
    "WhileExpr",
    // Surface pattern matching
    "MatchExpr",
    "Pattern",
    "PatternArm",
    // Implicit constructs
    "ImplicitReturn",
    "ImplicitSequence",
];

// ═══════════════════════════════════════════════════════════════════════════
// VALIDATION IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════

/// Validate a single node and recurse into children
fn validate_node(node: &SchemaAstNode, parent_kind: Option<&str>) -> NfValidationResult {
    let kind = node.kind.as_str();

    // Check if node kind is explicitly forbidden
    if is_forbidden_node(kind) {
        return Err(NfValidationError {
            message: format!(
                "forbidden node kind '{}' (surface construct must be normalized away)",
                kind
            ),
            span: node.span.clone(),
            node_kind: kind.to_string(),
            parent_kind: parent_kind.map(|s| s.to_string()),
        });
    }

    // Check if node kind is in admissible set
    if !is_admissible_node(kind) {
        return Err(NfValidationError {
            message: format!(
                "unknown or non-NF node kind '{}' (not in admissible set)",
                kind
            ),
            span: node.span.clone(),
            node_kind: kind.to_string(),
            parent_kind: parent_kind.map(|s| s.to_string()),
        });
    }

    // Recursively validate all child nodes
    for (_field_name, field_value) in &node.fields {
        validate_field(field_value, Some(kind))?;
    }

    Ok(())
}

/// Validate a field value (may contain nodes)
fn validate_field(value: &SchemaValue, parent_kind: Option<&str>) -> NfValidationResult {
    match value {
        SchemaValue::Node(child_node) => {
            validate_node(child_node, parent_kind)?;
        }
        SchemaValue::Nodes(child_nodes) => {
            for child_node in child_nodes {
                validate_node(child_node, parent_kind)?;
            }
        }
        SchemaValue::Token(_) | SchemaValue::Tokens(_) => {
            // Tokens are always valid in NF
        }
    }
    Ok(())
}

/// Check if a node kind is in the admissible set
fn is_admissible_node(kind: &str) -> bool {
    NF_ADMISSIBLE_NODES.contains(&kind)
}

/// Check if a node kind is explicitly forbidden
fn is_forbidden_node(kind: &str) -> bool {
    NF_FORBIDDEN_NODES.contains(&kind)
}

// ═══════════════════════════════════════════════════════════════════════════
// INSPECTION / DIAGNOSTICS
// ═══════════════════════════════════════════════════════════════════════════

/// Get the NF-H1 admissible node set (for inspection/docs)
pub fn get_admissible_nodes() -> &'static [&'static str] {
    NF_ADMISSIBLE_NODES
}

/// Get the NF-H1 forbidden node set (for inspection/docs)
pub fn get_forbidden_nodes() -> &'static [&'static str] {
    NF_FORBIDDEN_NODES
}

/// Collect all node kinds in an AST (for debugging/inspection)
pub fn collect_node_kinds(node: &SchemaAstNode) -> Vec<String> {
    let mut kinds = Vec::new();
    collect_node_kinds_recursive(node, &mut kinds);
    kinds.sort();
    kinds.dedup();
    kinds
}

fn collect_node_kinds_recursive(node: &SchemaAstNode, kinds: &mut Vec<String>) {
    kinds.push(node.kind.clone());

    for (_field_name, field_value) in &node.fields {
        match field_value {
            SchemaValue::Node(child) => {
                collect_node_kinds_recursive(child, kinds);
            }
            SchemaValue::Nodes(children) => {
                for child in children {
                    collect_node_kinds_recursive(child, kinds);
                }
            }
            _ => {}
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Helper to make test nodes
    fn make_node(kind: &str, span: Span) -> SchemaAstNode {
        SchemaAstNode {
            kind: kind.to_string(),
            fields: HashMap::new(),
            annotations: Vec::new(),
            span,
        }
    }

    fn make_node_with_child(kind: &str, child_field: &str, child: SchemaAstNode) -> SchemaAstNode {
        let mut fields = HashMap::new();
        fields.insert(child_field.to_string(), SchemaValue::Node(Box::new(child)));
        SchemaAstNode {
            kind: kind.to_string(),
            fields,
            annotations: Vec::new(),
            span: Span::new(0, 10),
        }
    }

    #[test]
    fn test_admissible_nodes_pass() {
        // All admissible nodes should pass validation
        for &kind in NF_ADMISSIBLE_NODES {
            let node = make_node(kind, Span::new(0, 10));
            assert!(
                validate_nf(&node).is_ok(),
                "Admissible node '{}' should pass validation",
                kind
            );
        }
    }

    #[test]
    fn test_forbidden_nodes_fail() {
        // All forbidden nodes should fail validation
        for &kind in NF_FORBIDDEN_NODES {
            let node = make_node(kind, Span::new(0, 10));
            let result = validate_nf(&node);
            assert!(
                result.is_err(),
                "Forbidden node '{}' should fail validation",
                kind
            );
            if let Err(e) = result {
                assert!(
                    e.message.contains("forbidden"),
                    "Error for '{}' should mention 'forbidden'",
                    kind
                );
            }
        }
    }

    #[test]
    fn test_unknown_node_fails() {
        let node = make_node("UnknownNode", Span::new(0, 10));
        let result = validate_nf(&node);
        assert!(result.is_err(), "Unknown node should fail validation");
        if let Err(e) = result {
            assert!(e.message.contains("unknown") || e.message.contains("not in admissible set"));
        }
    }

    #[test]
    fn test_recursive_validation() {
        // Valid parent with valid child should pass
        let child = make_node("Ident", Span::new(5, 8));
        let parent = make_node_with_child("LetExpr", "value", child);
        assert!(validate_nf(&parent).is_ok());
    }

    #[test]
    fn test_recursive_validation_fails_on_forbidden_child() {
        // Valid parent with forbidden child should fail
        let forbidden_child = make_node("ForExpr", Span::new(5, 8));
        let parent = make_node_with_child("Block", "stmt", forbidden_child);
        let result = validate_nf(&parent);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.node_kind, "ForExpr");
            assert_eq!(e.parent_kind, Some("Block".to_string()));
        }
    }

    #[test]
    fn test_collect_node_kinds() {
        let child = make_node("Ident", Span::new(5, 8));
        let parent = make_node_with_child("LetExpr", "value", child);
        let kinds = collect_node_kinds(&parent);
        assert_eq!(kinds, vec!["Ident".to_string(), "LetExpr".to_string()]);
    }

    #[test]
    fn test_wrapper_nodes_allowed() {
        // Wrappers are in admissible set (transparent at lowering boundary)
        let expr = make_node("Expr", Span::new(0, 10));
        assert!(validate_nf(&expr).is_ok());

        let atomic = make_node("AtomicExpr", Span::new(0, 10));
        assert!(validate_nf(&atomic).is_ok());
    }
}
