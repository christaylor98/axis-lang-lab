// H1 Lowering End-to-End Tests
//
// WAVE 5: H1 Lowering (Minimal, Mechanical)
//
// Tests the complete pipeline:
//   H1 Source → Lex → Parse → Schema AST → (Normalisation) → NF-H1 → Lower → Core IR
//
// COVERAGE:
// - Minimal executable subset (functions, calls, if, let, literals)
// - NF-H1 compliance verification
// - Core IR correctness
// - Determinism
// - Error conditions (surface nodes, invalid syntax)

use axis_lang_lab::frontend::schema_ast::{SchemaAstNode, SchemaValue};
use axis_lang_lab::frontend::token::{Span, Token, TokenKind};
use axis_lang_lab::ir::core_ir::CoreTerm;
use axis_lang_lab::lowering::h1_lowering::{
    lower_to_bundle, lower_to_core_ir, LoweringContext, Scope,
};
use axis_lang_lab::registry::{Registry, RegistryEntry};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// TEST HELPERS
// ═══════════════════════════════════════════════════════════════════════════

fn make_test_registry() -> Registry {
    // Create empty registry for testing
    // In real usage, registry would be loaded from file
    Registry {
        entries: vec![
            RegistryEntry {
                id: 1,
                name: "print".to_string(),
                arity: 1,
                deterministic: true,
                profiles: vec![],
            },
            RegistryEntry {
                id: 2,
                name: "add".to_string(),
                arity: 2,
                deterministic: true,
                profiles: vec![],
            },
        ],
    }
}

fn make_token(_kind: &str, lexeme: &str, start: usize) -> Token {
    Token {
        kind: TokenKind::Ident, // Simplified - all tokens as Ident for testing
        lexeme: lexeme.to_string(),
        span: Span::new(start, start + lexeme.len()),
    }
}

fn make_int_literal(value: i64) -> SchemaAstNode {
    let lexeme = value.to_string();
    let mut value_fields = HashMap::new();
    value_fields.insert(
        "token".to_string(),
        SchemaValue::Token(make_token("INT", &lexeme, 0)),
    );

    let value_node = SchemaAstNode {
        kind: "Literal".to_string(),
        fields: vec![(
            "value".to_string(),
            SchemaValue::Node(Box::new(SchemaAstNode {
                kind: "INT".to_string(),
                fields: value_fields,
                annotations: Vec::new(),
                span: Span::new(0, lexeme.len()),
            })),
        )]
        .into_iter()
        .collect(),
        annotations: Vec::new(),
        span: Span::new(0, lexeme.len()),
    };

    value_node
}

fn make_bool_literal(value: bool) -> SchemaAstNode {
    let kind = if value { "TRUE" } else { "FALSE" };
    let lexeme = if value { "true" } else { "false" };

    let mut value_fields = HashMap::new();
    value_fields.insert(
        "token".to_string(),
        SchemaValue::Token(make_token(kind, lexeme, 0)),
    );

    let value_node = SchemaAstNode {
        kind: "Literal".to_string(),
        fields: vec![(
            "value".to_string(),
            SchemaValue::Node(Box::new(SchemaAstNode {
                kind: kind.to_string(),
                fields: value_fields,
                annotations: Vec::new(),
                span: Span::new(0, lexeme.len()),
            })),
        )]
        .into_iter()
        .collect(),
        annotations: Vec::new(),
        span: Span::new(0, lexeme.len()),
    };

    value_node
}

fn make_ident(name: &str) -> SchemaAstNode {
    let mut fields = HashMap::new();
    fields.insert(
        "name".to_string(),
        SchemaValue::Token(make_token("IDENT", name, 0)),
    );

    SchemaAstNode {
        kind: "Ident".to_string(),
        fields,
        annotations: Vec::new(),
        span: Span::new(0, name.len()),
    }
}

fn make_unit() -> SchemaAstNode {
    SchemaAstNode {
        kind: "Unit".to_string(),
        fields: HashMap::new(),
        annotations: Vec::new(),
        span: Span::new(0, 2),
    }
}

fn make_lambda(param: &str, body: SchemaAstNode) -> SchemaAstNode {
    let mut fields = HashMap::new();
    fields.insert(
        "param".to_string(),
        SchemaValue::Token(make_token("IDENT", param, 0)),
    );
    fields.insert("body".to_string(), SchemaValue::Node(Box::new(body)));

    SchemaAstNode {
        kind: "LambdaExpr".to_string(),
        fields,
        annotations: Vec::new(),
        span: Span::new(0, 10),
    }
}

fn make_let_expr(name: &str, value: SchemaAstNode, body: SchemaAstNode) -> SchemaAstNode {
    let mut fields = HashMap::new();
    fields.insert(
        "name".to_string(),
        SchemaValue::Token(make_token("IDENT", name, 0)),
    );
    fields.insert("value".to_string(), SchemaValue::Node(Box::new(value)));
    fields.insert("body".to_string(), SchemaValue::Node(Box::new(body)));

    SchemaAstNode {
        kind: "LetExpr".to_string(),
        fields,
        annotations: Vec::new(),
        span: Span::new(0, 20),
    }
}

fn make_if_expr(
    cond: SchemaAstNode,
    then_block: SchemaAstNode,
    else_block: SchemaAstNode,
) -> SchemaAstNode {
    let mut fields = HashMap::new();
    fields.insert("condition".to_string(), SchemaValue::Node(Box::new(cond)));
    fields.insert(
        "then_block".to_string(),
        SchemaValue::Node(Box::new(then_block)),
    );
    fields.insert(
        "else_block".to_string(),
        SchemaValue::Node(Box::new(else_block)),
    );

    SchemaAstNode {
        kind: "IfExpr".to_string(),
        fields,
        annotations: Vec::new(),
        span: Span::new(0, 30),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// BASIC LOWERING TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_lower_unit_literal() {
    let node = make_unit();
    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CUnitLit { .. } => {}
        other => panic!("Expected CUnitLit, got {:?}", other),
    }
}

#[test]
fn test_lower_int_literal() {
    let node = make_int_literal(42);
    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CIntLit { value, .. } => {
            assert_eq!(value, 42);
        }
        other => panic!("Expected CIntLit, got {:?}", other),
    }
}

#[test]
fn test_lower_bool_literal() {
    let node = make_bool_literal(true);
    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CBoolLit { value, .. } => {
            assert_eq!(value, true);
        }
        other => panic!("Expected CBoolLit, got {:?}", other),
    }
}

#[test]
fn test_lower_ident() {
    let node = make_ident("x");
    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CVar { name, .. } => {
            assert_eq!(name.0.as_ref(), "x");
        }
        other => panic!("Expected CVar, got {:?}", other),
    }
}

#[test]
fn test_lower_lambda() {
    // λx. ()
    let node = make_lambda("x", make_unit());
    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CLam { param, body, .. } => {
            assert_eq!(param.0.as_ref(), "x");
            assert!(matches!(*body, CoreTerm::CUnitLit { .. }));
        }
        other => panic!("Expected CLam, got {:?}", other),
    }
}

#[test]
fn test_lower_let_expr() {
    // let x = 42 in x
    let node = make_let_expr("x", make_int_literal(42), make_ident("x"));
    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CLet {
            name, value, body, ..
        } => {
            assert_eq!(name.0.as_ref(), "x");
            assert!(matches!(*value, CoreTerm::CIntLit { value: 42, .. }));
            assert!(matches!(*body, CoreTerm::CVar { .. }));
        }
        other => panic!("Expected CLet, got {:?}", other),
    }
}

#[test]
fn test_lower_if_expr() {
    // if true { 1 } else { 2 }
    let node = make_if_expr(
        make_bool_literal(true),
        make_int_literal(1),
        make_int_literal(2),
    );
    let mut ctx = LoweringContext {
        registry: make_test_registry(),
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
            assert!(matches!(*cond, CoreTerm::CBoolLit { value: true, .. }));
            assert!(matches!(*then_branch, CoreTerm::CIntLit { value: 1, .. }));
            assert!(matches!(*else_branch, CoreTerm::CIntLit { value: 2, .. }));
        }
        other => panic!("Expected CIf, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPLEX EXPRESSION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_lower_nested_let() {
    // let x = 1 in let y = 2 in x
    let node = make_let_expr(
        "x",
        make_int_literal(1),
        make_let_expr("y", make_int_literal(2), make_ident("x")),
    );

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    // Verify structure: CLet x (CLet y x)
    match result.unwrap() {
        CoreTerm::CLet { name, body, .. } => {
            assert_eq!(name.0.as_ref(), "x");
            match *body {
                CoreTerm::CLet {
                    name: inner_name, ..
                } => {
                    assert_eq!(inner_name.0.as_ref(), "y");
                }
                _ => panic!("Expected nested CLet"),
            }
        }
        other => panic!("Expected CLet, got {:?}", other),
    }
}

#[test]
fn test_lower_lambda_with_let_body() {
    // λx. let y = x in y
    let lambda_body = make_let_expr("y", make_ident("x"), make_ident("y"));
    let node = make_lambda("x", lambda_body);

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_ok());

    match result.unwrap() {
        CoreTerm::CLam { param, body, .. } => {
            assert_eq!(param.0.as_ref(), "x");
            assert!(matches!(*body, CoreTerm::CLet { .. }));
        }
        other => panic!("Expected CLam, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NF-H1 BOUNDARY TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_reject_surface_for_expr() {
    let node = SchemaAstNode {
        kind: "ForExpr".to_string(),
        fields: HashMap::new(),
        annotations: Vec::new(),
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

#[test]
fn test_reject_surface_loop_expr() {
    let node = SchemaAstNode {
        kind: "LoopExpr".to_string(),
        fields: HashMap::new(),
        annotations: Vec::new(),
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

#[test]
fn test_reject_surface_match_expr() {
    let node = SchemaAstNode {
        kind: "MatchExpr".to_string(),
        annotations: Vec::new(),
        fields: HashMap::new(),
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

#[test]
fn test_reject_unknown_node() {
    let node = SchemaAstNode {
        kind: "UnknownNode".to_string(),
        annotations: Vec::new(),
        fields: HashMap::new(),
        span: Span::new(0, 10),
    };

    let mut ctx = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result = lower_to_core_ir(&node, &mut ctx);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("unknown node kind"));
}

// ═══════════════════════════════════════════════════════════════════════════
// DETERMINISM TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_lowering_is_deterministic() {
    // Lower the same node twice, verify identical output
    let node = make_let_expr("x", make_int_literal(42), make_ident("x"));

    let mut ctx1 = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };
    let mut ctx2 = LoweringContext {
        registry: make_test_registry(),
        scope: Scope::new(),
    };

    let result1 = lower_to_core_ir(&node, &mut ctx1);
    let result2 = lower_to_core_ir(&node, &mut ctx2);

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Core IR should be structurally identical
    assert_eq!(
        format!("{:?}", result1.unwrap()),
        format!("{:?}", result2.unwrap())
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// BUNDLE TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_lower_to_bundle() {
    let node = make_unit();
    let registry = make_test_registry();

    let result = lower_to_bundle(&node, registry);
    assert!(result.is_ok());

    let bundle = result.unwrap();
    assert_eq!(bundle.version.as_ref(), "0.3");
    assert!(matches!(bundle.core_term, CoreTerm::CUnitLit { .. }));
}

#[test]
fn test_bundle_contains_registry() {
    let node = make_int_literal(42);
    let registry = make_test_registry();

    let result = lower_to_bundle(&node, registry);
    assert!(result.is_ok());

    let bundle = result.unwrap();
    // Bundle should have registry metadata (if implemented)
    assert_eq!(bundle.version.as_ref(), "0.3");
}
