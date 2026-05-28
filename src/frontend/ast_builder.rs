// Wave 3: ParseTree → Generic AST (Structural Projection)
//
// THIS IS PROJECTION, NOT TRANSFORMATION.
//
// This module re-encodes ParseTree structure into a Generic AST
// without ANY semantic interpretation.
//
// FORBIDDEN:
// - Dropping nodes
// - Flattening structure
// - Renaming nodes
// - Reordering children
// - Normalizing expressions
// - Inferring operators
// - Attaching meaning
//
// The Generic AST is a faithful structural record of parsing, nothing more.

use crate::frontend::parser_runtime::{ParseNode, ParseTree};
use crate::frontend::token::{Span, Token};
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════

/// Generic AST Node - structural projection of ParseTree
///
/// This is intentionally "ugly" - it mirrors ParseTree structure exactly.
/// No enums per grammar rule, no field extraction, no typed children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ASTNode {
    /// Grammar nonterminal name (copied from ParseTree.rule)
    pub kind: String,
    /// Children in parse order (no reordering, no dropping)
    pub children: Vec<ASTChild>,
    /// Span derived from children (never invented)
    pub span: Span,
}

/// Child of an AST node
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ASTChild {
    /// Non-terminal (recursively projected)
    Node(ASTNode),
    /// Terminal token (copied verbatim)
    Terminal(Token),
}

/// AST build error - indicates internal bug, not user error
#[derive(Debug, Clone)]
pub struct ASTBuildError {
    pub message: String,
}

impl fmt::Display for ASTBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AST build error: {}", self.message)
    }
}

impl std::error::Error for ASTBuildError {}

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════

/// Build Generic AST from ParseTree via structural projection
///
/// This function performs a mechanical re-encoding:
/// - ParseTree structure → ASTNode structure
/// - No interpretation
/// - No simplification
/// - No semantic decisions
///
/// Errors indicate internal bugs (violated ParseTree invariants),
/// NOT user errors.
pub fn build_generic_ast(tree: &ParseTree) -> Result<ASTNode, ASTBuildError> {
    build_node(tree)
}

// ═══════════════════════════════════════════════════════════════════════════
// INTERNAL IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════

/// Recursively project ParseTree node to ASTNode
fn build_node(tree: &ParseTree) -> Result<ASTNode, ASTBuildError> {
    let mut children = Vec::new();

    // Project each ParseNode child to ASTChild
    for child in &tree.children {
        match child {
            ParseNode::Rule(subtree) => {
                // Recursively project non-terminal
                let ast_node = build_node(subtree)?;
                children.push(ASTChild::Node(ast_node));
            }
            ParseNode::Terminal(token) => {
                // Copy terminal verbatim
                children.push(ASTChild::Terminal(token.clone()));
            }
        }
    }

    // Derive span from children (NEVER invent spans)
    let span = derive_span(&children, &tree.span)?;

    Ok(ASTNode {
        kind: tree.rule.clone(),
        children,
        span,
    })
}

/// Derive AST node span from children
///
/// Rules (non-negotiable):
/// 1. AST span MUST be derived from children
/// 2. start = first child span start
/// 3. end = last child span end
/// 4. If no children, copy ParseTree span directly
/// 5. No invented spans
/// 6. No widening "just in case"
fn derive_span(children: &[ASTChild], fallback: &Span) -> Result<Span, ASTBuildError> {
    if children.is_empty() {
        // Edge case: no children - use fallback (ParseTree's span)
        return Ok(fallback.clone());
    }

    // Get first child span
    let first_span = match &children[0] {
        ASTChild::Node(node) => &node.span,
        ASTChild::Terminal(token) => &token.span,
    };

    // Get last child span
    let last_span = match &children[children.len() - 1] {
        ASTChild::Node(node) => &node.span,
        ASTChild::Terminal(token) => &token.span,
    };

    let start = first_span.start;
    let end = last_span.end;

    // Sanity check: spans should be ordered
    if start > end {
        return Err(ASTBuildError {
            message: format!("child spans out of order: start={} > end={}", start, end),
        });
    }

    Ok(Span::new(start, end))
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::token::TokenKind;

    /// Helper: create a Token for testing
    fn token(kind: TokenKind, lexeme: &str, start: usize, end: usize) -> Token {
        Token {
            kind,
            lexeme: lexeme.to_string(),
            span: Span::new(start, end),
        }
    }

    /// Helper: create a simple ParseTree for testing
    fn parse_tree(rule: &str, children: Vec<ParseNode>, start: usize, end: usize) -> ParseTree {
        ParseTree {
            rule: rule.to_string(),
            children,
            span: Span::new(start, end),
        }
    }

    #[test]
    fn test_single_terminal() {
        // ParseTree: Rule("expr", [Terminal("42")])
        let tok = token(TokenKind::IntLit, "42", 0, 2);
        let tree = parse_tree("expr", vec![ParseNode::Terminal(tok.clone())], 0, 2);

        let ast = build_generic_ast(&tree).unwrap();

        assert_eq!(ast.kind, "expr");
        assert_eq!(ast.children.len(), 1);
        assert_eq!(ast.span, Span::new(0, 2));

        match &ast.children[0] {
            ASTChild::Terminal(t) => assert_eq!(t, &tok),
            _ => panic!("expected terminal"),
        }
    }

    #[test]
    fn test_nested_rules() {
        // ParseTree: Rule("outer", [Rule("inner", [Terminal("x")])])
        let tok = token(TokenKind::Ident, "x", 5, 6);
        let inner = parse_tree("inner", vec![ParseNode::Terminal(tok.clone())], 5, 6);
        let outer = parse_tree("outer", vec![ParseNode::Rule(inner)], 5, 6);

        let ast = build_generic_ast(&outer).unwrap();

        assert_eq!(ast.kind, "outer");
        assert_eq!(ast.children.len(), 1);
        assert_eq!(ast.span, Span::new(5, 6));

        match &ast.children[0] {
            ASTChild::Node(node) => {
                assert_eq!(node.kind, "inner");
                assert_eq!(node.children.len(), 1);
                match &node.children[0] {
                    ASTChild::Terminal(t) => assert_eq!(t, &tok),
                    _ => panic!("expected terminal"),
                }
            }
            _ => panic!("expected node"),
        }
    }

    #[test]
    fn test_multiple_children() {
        // ParseTree: Rule("seq", [Terminal("a"), Terminal("b"), Terminal("c")])
        let tok_a = token(TokenKind::Ident, "a", 0, 1);
        let tok_b = token(TokenKind::Ident, "b", 2, 3);
        let tok_c = token(TokenKind::Ident, "c", 4, 5);

        let tree = parse_tree(
            "seq",
            vec![
                ParseNode::Terminal(tok_a.clone()),
                ParseNode::Terminal(tok_b.clone()),
                ParseNode::Terminal(tok_c.clone()),
            ],
            0,
            5,
        );

        let ast = build_generic_ast(&tree).unwrap();

        assert_eq!(ast.kind, "seq");
        assert_eq!(ast.children.len(), 3);
        assert_eq!(ast.span, Span::new(0, 5));
    }

    #[test]
    fn test_empty_children() {
        // Edge case: ParseTree with no children
        let tree = parse_tree("empty", vec![], 10, 10);

        let ast = build_generic_ast(&tree).unwrap();

        assert_eq!(ast.kind, "empty");
        assert_eq!(ast.children.len(), 0);
        assert_eq!(ast.span, Span::new(10, 10)); // fallback to ParseTree span
    }

    #[test]
    fn test_span_derived_from_children() {
        // Span should be derived from first and last child
        let tok_first = token(TokenKind::Ident, "x", 10, 11);
        let tok_middle = token(TokenKind::Ident, "y", 15, 16);
        let tok_last = token(TokenKind::Ident, "z", 20, 21);

        let tree = parse_tree(
            "test",
            vec![
                ParseNode::Terminal(tok_first),
                ParseNode::Terminal(tok_middle),
                ParseNode::Terminal(tok_last),
            ],
            0,
            100, // ParseTree span is wider - should be ignored
        );

        let ast = build_generic_ast(&tree).unwrap();

        // Span should be from first child start (10) to last child end (21)
        assert_eq!(ast.span, Span::new(10, 21));
    }

    #[test]
    fn test_determinism() {
        // Same input should produce bitwise identical output
        let tok = token(TokenKind::IntLit, "42", 0, 2);
        let tree = parse_tree("expr", vec![ParseNode::Terminal(tok)], 0, 2);

        let ast1 = build_generic_ast(&tree).unwrap();
        let ast2 = build_generic_ast(&tree).unwrap();

        assert_eq!(ast1, ast2);
    }
}
